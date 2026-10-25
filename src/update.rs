// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte self-update`: replace this binary with a GitHub release when the
//! user runs `jubarte self-update` (docs/SELF_UPDATE.md).
//!
//! The release lookup, SHA-256 check against `SHA256SUMS.txt`, zipsign
//! signature check, archive extraction and binary swap are the
//! [`self_update`](https://docs.rs/self_update) crate's. This module builds
//! the call the binary makes: it picks the release archive for this platform
//! (`asset_target`), decides whether the target is newer
//! (`plan`) and refuses to ask on a non-interactive stdin
//! (`preflight`). The signature rule lives in `signature_required`,
//! split out so it is unit-tested without I/O.
//!
//! Entry points: `run` for the command itself, `plan` /
//! `preflight` / `signature_required` for the decisions.

use std::io::{BufRead, IsTerminal, Write};

use self_update::backends::github;
use self_update::version::cmp_versions;

const REPO_OWNER: &str = "jandira-tech";
const REPO_NAME: &str = "jubarte-redlines";
const SUMS_ASSET: &str = "SHA256SUMS.txt";

/// The ed25519 public keys trusted to sign release archives (zipsign), any
/// one of which must have signed a release from [`FIRST_SIGNED_RELEASE`] on.
///
/// Empty until the owner commits a key, and with no key the updater checks
/// `SHA256SUMS.txt` alone, as before. To trust a key, commit the 32-byte file
/// `zipsign gen-key` writes and list it here:
///
/// ```text
/// const RELEASE_KEYS: &[self_update::VerifyingKey] =
///     &[*include_bytes!("../keys/release-signing.pub")];
/// ```
///
/// A file that is not exactly 32 bytes fails the build. Set
/// [`FIRST_SIGNED_RELEASE`] in the same commit; docs/SELF_UPDATE.md has the
/// steps, including key rotation.
const RELEASE_KEYS: &[self_update::VerifyingKey] = &[];

/// The first release whose archives `release.yml` signed, without a leading
/// `v` (`Some("0.11.0")`). `None` until [`RELEASE_KEYS`] holds a key.
const FIRST_SIGNED_RELEASE: Option<&str> = None;

/// The flags `jubarte self-update` accepts, resolved from the CLI before
/// [`run`] sees them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// Print the installed and the latest version and install nothing
    /// (`--check`).
    pub check: bool,
    /// Install without the interactive prompt (`--yes`).
    pub yes: bool,
    /// Install this release (`0.10.0` or `v0.10.0`; `--version`). A
    /// request may downgrade; the installed version itself is up to date
    /// and is not installed again.
    pub version: Option<String>,
}

/// The decision [`plan`] returns, once the target release is known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Plan {
    /// The installed version is the target (or already newer, when no
    /// `--version` was asked for): nothing to do.
    UpToDate {
        /// The installed version.
        current: String,
    },
    /// Replace the installed version with `to`.
    Install {
        /// The installed version.
        from: String,
        /// The release to install.
        to: String,
    },
}

/// The release archive suffix this binary will look up (`macos-aarch64`,
/// `linux-x86_64`, …), matching the names `release.yml` publishes. `None`
/// for an OS / arch pair no archive is built for.
#[must_use]
pub fn asset_target() -> Option<&'static str> {
    asset_target_for(std::env::consts::OS, std::env::consts::ARCH)
}

fn asset_target_for(os: &str, arch: &str) -> Option<&'static str> {
    Some(match (os, arch) {
        ("macos", "aarch64") => "macos-aarch64",
        ("macos", "x86_64") => "macos-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("linux", "x86_64") => "linux-x86_64",
        ("windows", "x86_64") => "windows-x86_64",
        _ => return None,
    })
}

/// Path of the binary inside the release archive, as `self_update` expects
/// it (`{{ version }}` is filled in by the crate).
fn bin_path_in_archive(target: &str) -> String {
    format!(
        "jubarte-{{{{ version }}}}-{target}/jubarte{}",
        std::env::consts::EXE_SUFFIX
    )
}

/// Refuse to start when the command would have to prompt and the stdin
/// channel is not a terminal. None of the operation is touched, not even a
/// network lookup.
///
/// # Errors
///
/// A message when neither [`Options::check`] nor [`Options::yes`] is set
/// and `interactive` is `false`: passing `--check` or `--yes`, or being on a
/// terminal, lets the run continue.
pub fn preflight(opts: &Options, interactive: bool) -> Result<(), String> {
    if opts.check || opts.yes || interactive {
        Ok(())
    } else {
        Err("no terminal to confirm on; pass --yes to install or --check to only look".into())
    }
}

/// Strip one leading `v` so `v0.10.0` and `0.10.0` name the same release.
fn bare(version: &str) -> &str {
    version.strip_prefix('v').unwrap_or(version)
}

/// Decide between staying and installing for a given installed version and
/// the version `self_update` resolved on GitHub.
///
/// Without a requested version only a strictly newer release installs; a
/// requested version installs whenever it differs from the installed one
/// (downgrades included); the installed version is [`Plan::UpToDate`]. A leading `v` is stripped, so
/// `v0.10.0` and `0.10.0` name the same release.
///
/// # Errors
///
/// A message when either version is not semver.
pub fn plan(current: &str, target: &str, requested: bool) -> Result<Plan, String> {
    let (current, target) = (bare(current), bare(target));
    let order = cmp_versions(target, current).map_err(|e| format!("version {target}: {e}"))?;
    let install = if requested {
        order.is_ne()
    } else {
        order.is_gt()
    };
    Ok(if install {
        Plan::Install {
            from: current.to_string(),
            to: target.to_string(),
        }
    } else {
        Plan::UpToDate {
            current: current.to_string(),
        }
    })
}

/// `jubarte self-update`. `Ok` means the run finished without error: the
/// binary is replaced only when an install completes, and `--check`, an
/// up-to-date release or a declined prompt leave it as it was. On failure
/// the installed binary is unchanged. Writes progress to
/// stdout, the prompt to stderr, and reads the answer from stdin.
///
/// The flow is: [`preflight`] → resolve the release for this platform
/// ([`asset_target`]) → decide ([`plan`]) → optional prompt → optional
/// signature rule ([`signature_required`]) → `self_update` install. A prompt
/// `y` / `Y` / `yes` confirms; anything else is "no".
///
/// # Errors
///
/// A message for one of:
/// - the platform has no published archive ([`asset_target`] is `None`);
/// - the [`preflight`] check refused the run;
/// - the GitHub release lookup failed or returned no release;
/// - the network or GitHub error inside the `self_update` installer;
/// - the checksum download or verification did not match;
/// - a signature was required but missing or did not verify against
///   [`signature_required`];
/// - the binary swap step failed.
pub fn run(opts: &Options) -> Result<(), String> {
    let interactive = std::io::stdin().is_terminal();
    preflight(opts, interactive)?;
    let target = asset_target().ok_or_else(|| {
        format!(
            "no release archive is built for {}-{}; build from source",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let current = env!("CARGO_PKG_VERSION");

    let mut builder = github::Update::configure();
    builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name("jubarte")
        .target(target)
        .current_version(current)
        .bin_path_in_archive(bin_path_in_archive(target))
        .checksum_from_asset(SUMS_ASSET)
        .show_output(false)
        .show_download_progress(interactive)
        .no_confirm(true);
    let lookup = builder.build().map_err(|e| e.to_string())?;
    let release = match &opts.version {
        Some(v) => lookup
            .get_release_version(&format!("v{}", bare(v)))
            .map(Some),
        None => lookup
            .get_latest_release()
            .map(|releases| releases.into_vec().into_iter().next()),
    }
    .map_err(|e| format!("GitHub release lookup failed: {e}"))?
    .ok_or("GitHub lists no release")?;

    let decision = plan(current, release.version(), opts.version.is_some())?;
    let (from, to) = match decision {
        Plan::UpToDate { current } => {
            println!(
                "jubarte {current} is up to date (latest release {}).",
                bare(release.version())
            );
            return Ok(());
        }
        Plan::Install { from, to } => (from, to),
    };
    if opts.check {
        println!(
            "jubarte {from} is installed; {to} is available. Run `jubarte self-update` to install it."
        );
        return Ok(());
    }
    if !opts.yes && !confirm(&from, &to)? {
        println!("Nothing changed.");
        return Ok(());
    }

    let signed = signature_required(&to, FIRST_SIGNED_RELEASE, !RELEASE_KEYS.is_empty())?;
    if signed {
        builder.verifying_keys(RELEASE_KEYS);
    } else if !RELEASE_KEYS.is_empty() {
        eprintln!("note: {to} predates signed releases; only its SHA-256 is checked.");
    }
    let installer = builder
        .release_tag(format!("v{to}"))
        .build()
        .map_err(|e| e.to_string())?;
    installer
        .update()
        .map_err(|e| format!("update to {to} failed; the installed binary is unchanged: {e}"))?;
    println!("jubarte {from} -> {to} installed.");
    Ok(())
}

/// Whether installing `target_version` must pass a zipsign signature check.
///
/// True only when `keys_present` and the release sorts at or after
/// `first_signed`: releases published before signing began carry no
/// signature, so a `--version` downgrade to one of them is checked by its
/// SHA-256 alone. A `v` prefix on either version is stripped before the
/// comparison.
///
/// # Errors
///
/// A message when `keys_present` and `first_signed` are both set but either
/// side is not semver.
pub fn signature_required(
    target_version: &str,
    first_signed: Option<&str>,
    keys_present: bool,
) -> Result<bool, String> {
    let Some(first) = first_signed.filter(|_| keys_present) else {
        return Ok(false);
    };
    let (target, first) = (bare(target_version), bare(first));
    let order = cmp_versions(target, first)
        .map_err(|e| format!("version {target} against first signed release {first}: {e}"))?;
    Ok(order.is_ge())
}

fn confirm(from: &str, to: &str) -> Result<bool, String> {
    eprint!("Replace jubarte {from} with {to}? [y/N] ");
    std::io::stderr().flush().map_err(|e| e.to_string())?;
    let mut answer = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut answer)
        .map_err(|e| e.to_string())?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes" | "Yes" | "YES"))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn asset_targets_match_the_release_workflow_archives() {
        assert_eq!(asset_target_for("macos", "aarch64"), Some("macos-aarch64"));
        assert_eq!(asset_target_for("macos", "x86_64"), Some("macos-x86_64"));
        assert_eq!(asset_target_for("linux", "aarch64"), Some("linux-aarch64"));
        assert_eq!(asset_target_for("linux", "x86_64"), Some("linux-x86_64"));
        assert_eq!(
            asset_target_for("windows", "x86_64"),
            Some("windows-x86_64")
        );
        // No archive is published for these: refuse rather than pick a wrong one.
        assert_eq!(asset_target_for("windows", "aarch64"), None);
        assert_eq!(asset_target_for("freebsd", "x86_64"), None);
        assert!(asset_target().is_some(), "every CI host has an archive");
    }

    #[test]
    fn test_asset_target_calls_asset_target_for() {
        assert_eq!(
            asset_target(),
            asset_target_for(std::env::consts::OS, std::env::consts::ARCH)
        );
    }

    #[test]
    fn archive_binary_path_names_the_staged_folder() {
        let path = bin_path_in_archive("macos-aarch64");
        assert_eq!(
            path,
            format!(
                "jubarte-{{{{ version }}}}-macos-aarch64/jubarte{}",
                std::env::consts::EXE_SUFFIX
            )
        );
        assert!(path.contains("{{ version }}"));
    }

    #[test]
    fn without_a_terminal_only_check_or_yes_proceed() {
        let ask = Options::default();
        assert!(preflight(&ask, false).is_err());
        assert!(preflight(&ask, true).is_ok());
        let check = Options {
            check: true,
            ..Options::default()
        };
        assert!(preflight(&check, false).is_ok());
        let yes = Options {
            yes: true,
            ..Options::default()
        };
        assert!(preflight(&yes, false).is_ok());
        // Naming a version still needs an answer.
        let pinned = Options {
            version: Some("0.9.3".into()),
            ..Options::default()
        };
        assert!(preflight(&pinned, false).is_err());
    }

    #[test]
    fn latest_installs_only_when_newer() {
        assert_eq!(
            plan("0.9.3", "v0.10.0", false),
            Ok(Plan::Install {
                from: "0.9.3".into(),
                to: "0.10.0".into()
            })
        );
        assert_eq!(
            plan("0.10.0", "0.10.0", false),
            Ok(Plan::UpToDate {
                current: "0.10.0".into()
            })
        );
        // A newer local build never downgrades to the latest release.
        assert_eq!(
            plan("0.11.0", "0.10.0", false),
            Ok(Plan::UpToDate {
                current: "0.11.0".into()
            })
        );
    }

    #[test]
    fn a_requested_version_installs_even_when_older() {
        assert_eq!(
            plan("0.10.0", "0.9.3", true),
            Ok(Plan::Install {
                from: "0.10.0".into(),
                to: "0.9.3".into()
            })
        );
        assert_eq!(
            plan("0.9.3", "v0.9.3", true),
            Ok(Plan::UpToDate {
                current: "0.9.3".into()
            })
        );
    }

    #[test]
    fn a_non_semver_tag_is_an_error() {
        assert!(plan("0.10.0", "nightly", false).is_err());
    }

    #[test]
    fn without_keys_no_signature_is_required() {
        for (target, first) in [
            ("0.12.0", Some("0.11.0")),
            ("0.11.0", Some("0.11.0")),
            ("0.9.0", Some("0.11.0")),
            ("0.12.0", None),
        ] {
            assert_eq!(
                signature_required(target, first, false),
                Ok(false),
                "{target} with first signed {first:?}"
            );
        }
    }

    #[test]
    fn keys_without_a_first_signed_release_require_nothing() {
        assert_eq!(signature_required("0.12.0", None, true), Ok(false));
        assert_eq!(signature_required("v0.12.0", None, true), Ok(false));
    }

    #[test]
    fn the_first_signed_release_and_later_ones_require_a_signature() {
        let first = Some("0.11.0");
        assert_eq!(signature_required("0.11.0", first, true), Ok(true));
        assert_eq!(signature_required("v0.11.0", first, true), Ok(true));
        assert_eq!(signature_required("0.11.1", first, true), Ok(true));
        assert_eq!(signature_required("v1.0.0", first, true), Ok(true));
        // A `v` on the threshold is the same release.
        assert_eq!(
            signature_required("0.11.0", Some("v0.11.0"), true),
            Ok(true)
        );
    }

    #[test]
    fn releases_before_the_first_signed_one_are_checksum_only() {
        let first = Some("0.11.0");
        assert_eq!(signature_required("0.10.1", first, true), Ok(false));
        assert_eq!(signature_required("v0.10.1", first, true), Ok(false));
        // A pre-release of the first signed version sorts before it.
        assert_eq!(signature_required("0.11.0-rc.1", first, true), Ok(false));
    }

    #[test]
    fn a_non_semver_version_is_an_error_when_signing_is_on() {
        assert!(signature_required("nightly", Some("0.11.0"), true).is_err());
        assert!(signature_required("0.11.0", Some("next"), true).is_err());
    }

    #[test]
    fn the_shipped_key_set_and_threshold_agree() {
        // Both are set in the same commit (docs/SELF_UPDATE.md): a threshold
        // with no key would never check a signature, and a key with no
        // threshold would never be used.
        assert_eq!(
            FIRST_SIGNED_RELEASE.is_some(),
            !RELEASE_KEYS.is_empty(),
            "set FIRST_SIGNED_RELEASE and RELEASE_KEYS together"
        );
        if let Some(first) = FIRST_SIGNED_RELEASE {
            // The threshold is semver and carries no `v`. It is set to the
            // next release, before `Cargo.toml` is bumped, so it is not
            // compared with this build's version.
            assert!(!first.starts_with('v'), "write {first} without the v");
            assert_eq!(signature_required(first, Some(first), true), Ok(true));
        }
    }
}
