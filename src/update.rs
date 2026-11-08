// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte self-update`: replace this binary with a GitHub release when the
//! user runs `jubarte self-update` (docs/SELF_UPDATE.md).
//!
//! The release lookup, SHA-256 check against `SHA256SUMS.txt`, archive
//! extraction and binary swap are the
//! [`self_update`](https://docs.rs/self_update) crate's; the zipsign
//! signature check is [`verify_signature`](crate::update::verify_signature)
//! (`zipsign.rs`), which `self_update`
//! runs on the downloaded archive before extracting it. This module builds
//! the call the binary makes: it picks the release archive for this platform
//! (`asset_target`), decides whether the target is newer
//! (`plan`) and refuses to ask on a non-interactive stdin
//! (`preflight`). The signature rule lives in `signature_required`,
//! split out so it is unit-tested without I/O.
//!
//! Entry points: `run` for the command itself, `plan` / `step` /
//! `preflight` / `signature_required` for the decisions.

use std::io::{BufRead, IsTerminal, Write};

use self_update::backends::github;
use self_update::version::cmp_versions;

mod zipsign;
pub use zipsign::{ReleaseKey, verify_signature};

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
/// const RELEASE_KEYS: &[ReleaseKey] =
///     &[*include_bytes!("../keys/release-signing.pub")];
/// ```
///
/// A file that is not exactly 32 bytes fails the build. Set
/// [`FIRST_SIGNED_RELEASE`] in the same commit; docs/SELF_UPDATE.md has the
/// steps, including key rotation.
const RELEASE_KEYS: &[ReleaseKey] = &[];

/// The first release whose archives `release.yml` signed, without a leading
/// `v` (`Some("0.11.0")`). `None` until [`RELEASE_KEYS`] holds a key.
const FIRST_SIGNED_RELEASE: Option<&str> = None;

/// Bytes `jubarte self-update` will download, or inflate from the one
/// expected member of a release zip, before it refuses and leaves the
/// installed binary in place.
///
/// Published archives are about 18 MiB (`v0.17.0`). This is several times
/// that size. The tar.gz member's own inflation cap is a separate check.
const UPDATE_ARCHIVE_BYTE_BUDGET: u64 = 128 * 1024 * 1024;

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

/// What `run` does once the release is resolved: report and stop, or ask
/// (unless `--yes`) and install. Pure, so the "`--check` never installs"
/// rule is unit-tested without the network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Print `text` and stop. Nothing is downloaded or replaced.
    Report {
        /// The line to print.
        text: String,
    },
    /// Confirm unless `opts.yes`, then replace the running binary.
    Install {
        /// The installed version.
        from: String,
        /// The release to install.
        to: String,
    },
}

/// `decision` is [`plan`]'s answer; `resolved` is the bare release version
/// the lookup returned (the up-to-date message quotes it).
#[must_use]
pub fn step(opts: &Options, decision: &Plan, resolved: &str) -> Step {
    match decision {
        Plan::UpToDate { current } => Step::Report {
            text: format!("jubarte {current} is up to date (latest release {resolved})."),
        },
        Plan::Install { from, to } if opts.check => Step::Report {
            text: format!(
                "jubarte {from} is installed; {to} is available. Run `jubarte self-update` to install it."
            ),
        },
        Plan::Install { from, to } => Step::Install {
            from: from.clone(),
            to: to.clone(),
        },
    }
}

/// `jubarte self-update`. `Ok` means the run finished without error: the
/// binary is replaced only when an install completes, and `--check`, an
/// up-to-date release or a declined prompt leave it as it was. On failure
/// the installed binary is unchanged. Writes progress to
/// stdout, the prompt to stderr, and reads the answer from stdin.
///
/// The flow is: [`preflight`] → resolve the release for this platform
/// ([`asset_target`]) → decide ([`plan`]) → act ([`step`]) → optional prompt
/// → optional signature rule ([`signature_required`]) → `self_update`
/// install. A prompt `y` / `Y` / `yes` confirms; anything else is "no".
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
/// - the archive is over [`UPDATE_ARCHIVE_BYTE_BUDGET`], lists the expected
///   binary more than once, or inflates that binary past the budget;
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
        .verify_binary(extracted_binary_is_regular_file)
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
    let (from, to) = match step(opts, &decision, bare(release.version())) {
        Step::Report { text } => {
            println!("{text}");
            return Ok(());
        }
        Step::Install { from, to } => (from, to),
    };
    if !opts.yes && !confirm(&from, &to)? {
        println!("Nothing changed.");
        return Ok(());
    }

    let signed = signature_required(&to, FIRST_SIGNED_RELEASE, !RELEASE_KEYS.is_empty())?;
    let member = resolved_bin_path(target, &to);
    // Before extraction: cap the download and the inflated member, and refuse
    // an archive that lists that member twice. The signature decision above
    // is unchanged: this hook runs it only when that decision already said to.
    builder.verify_archive(move |archive| {
        confine_update_zip(archive, &member, UPDATE_ARCHIVE_BYTE_BUDGET)
            .map_err(self_update::Error::archive_verification_rejected)?;
        if signed {
            verify_signature(archive, RELEASE_KEYS)
                .map_err(self_update::Error::archive_verification_rejected)?;
        }
        Ok(())
    });
    if !signed && !RELEASE_KEYS.is_empty() {
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

/// The `verify_binary` hook `run` registers (ZIP-16/#639): refuse the
/// extracted binary when it is anything but a regular file. `self_update`
/// already refuses a symlink *zip* entry in its single-file path, but its
/// tar path delegates to `tar`'s `unpack_in`, which restores a symlink
/// entry as a real link whose target is not confined to the staging
/// directory — a crafted release archive could then install any path's
/// contents as the new binary. `symlink_metadata`, not `metadata`, sees
/// the entry itself rather than its target.
fn extracted_binary_is_regular_file(path: &std::path::Path) -> self_update::Result<()> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| {
        self_update::Error::verification_rejected(format!(
            "the extracted binary {} cannot be inspected: {e}",
            path.display()
        ))
    })?;
    if metadata.is_file() {
        Ok(())
    } else {
        Err(self_update::Error::verification_rejected(format!(
            "the extracted binary {} is not a regular file",
            path.display()
        )))
    }
}

/// Path of the binary inside a release archive once `{{ version }}` is the
/// release being installed. `self_update` fills that token itself; the
/// pre-extract check has to use the same path.
fn resolved_bin_path(target: &str, version: &str) -> String {
    bin_path_in_archive(target).replace("{{ version }}", version)
}

/// Gate the downloaded release archive before `self_update` extracts it.
///
/// `run` registers this as the `verify_archive` hook, which runs after the
/// checksum and before a byte of the archive is unpacked. `member` is the
/// binary path inside the archive (`jubarte-<version>-<target>/jubarte` or
/// `.exe`). A file larger than `budget` is refused. A zip whose expected
/// member appears more than once, or whose expected member inflates past
/// `budget`, is refused. Anything that is not a zip is left alone once the
/// download itself fits: the tar.gz arm is checked separately.
fn confine_update_zip(archive: &std::path::Path, member: &str, budget: u64) -> Result<(), String> {
    let len = std::fs::metadata(archive)
        .map_err(|e| {
            format!(
                "update archive {} cannot be inspected: {e}",
                archive.display()
            )
        })?
        .len();
    if len > budget {
        return Err(format!(
            "update archive is {len} bytes, over the {budget}-byte download budget"
        ));
    }
    let mut file = std::fs::File::open(archive)
        .map_err(|e| format!("update archive {} cannot be opened: {e}", archive.display()))?;
    // A zip is told by its end record, not by a leading `PK`: a zipsign
    // signature sits before the zip and zip/self_update still read it
    // (#2084 review). No end record and no `PK`: not a zip (a tar.gz).
    let Some(copies) = zip_member_copies(&mut file, len, member)? else {
        if zip_magic(&mut file)? {
            return Err("update archive has no zip end record".into());
        }
        return Ok(());
    };
    if copies > 1 {
        return Err(format!("update archive lists {member} more than once"));
    }
    if copies == 0 {
        return Err(format!("update archive has no member {member}"));
    }
    inflate_member_within_budget(archive, member, budget)
}

fn zip_magic(file: &mut std::fs::File) -> Result<bool, String> {
    use std::io::{Read, Seek, SeekFrom};
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("update archive cannot be rewound: {e}"))?;
    let mut magic = [0u8; 2];
    let read = file
        .read(&mut magic)
        .map_err(|e| format!("update archive cannot be read: {e}"))?;
    Ok(read == 2 && magic == [0x50, 0x4b])
}

/// How many central-directory entries name `member`, or `None` when the
/// file has no zip end record.
///
/// `zip`'s reader keeps one entry per name, so a repeated member is invisible
/// there and the first copy would be the one installed. The count comes from
/// the central directory itself. Each name is the one `zip` resolves (an
/// adopted 0x7075 Unicode Path included), and a directory that a leading
/// prefix moved past its recorded offset is found where `zip` finds it.
fn zip_member_copies(
    file: &mut std::fs::File,
    len: u64,
    member: &str,
) -> Result<Option<u32>, String> {
    const EOCD_LEN: u64 = 22;
    const MAX_COMMENT: u64 = 65_535;
    if len < EOCD_LEN {
        return Ok(None);
    }
    let tail_len = len.min(EOCD_LEN.saturating_add(MAX_COMMENT));
    let tail_at = len.saturating_sub(tail_len);
    let tail_bytes = usize::try_from(tail_len)
        .map_err(|_| "update archive is too large to inspect".to_string())?;
    let mut tail = vec![0u8; tail_bytes];
    read_exact_at(file, tail_at, &mut tail)?;
    let Some(eocd) = find_eocd(&tail) else {
        return Ok(None);
    };
    let entries = le_u16(&tail, eocd.saturating_add(10))?;
    let cd_size = u64::from(le_u32(&tail, eocd.saturating_add(12))?);
    let cd_offset = u64::from(le_u32(&tail, eocd.saturating_add(16))?);
    if entries == u16::MAX || cd_size == u64::from(u32::MAX) || cd_offset == u64::from(u32::MAX) {
        return Err("update archive uses ZIP64, which this check refuses".into());
    }
    // Bytes before the zip (a zipsign signature) shift the directory by
    // the gap between where it ends and where the end record sits; zip
    // reads it at the recorded offset first, then shifted.
    let eocd_at = tail_at.saturating_add(eocd as u64);
    let recorded_end = cd_offset
        .checked_add(cd_size)
        .ok_or_else(|| "update archive central directory is out of range".to_string())?;
    let shift = eocd_at
        .checked_sub(recorded_end)
        .ok_or_else(|| "update archive central directory runs past its end record".to_string())?;
    let mut signature = [0u8; 4];
    let at_recorded = cd_size == 0
        || (read_exact_at(file, cd_offset, &mut signature).is_ok()
            && signature == [0x50, 0x4b, 0x01, 0x02]);
    let base = if at_recorded { 0 } else { shift };
    let cd_start = cd_offset
        .checked_add(base)
        .ok_or_else(|| "update archive central directory is out of range".to_string())?;
    let cd_end = recorded_end
        .checked_add(base)
        .ok_or_else(|| "update archive central directory is out of range".to_string())?;
    if cd_end > len {
        return Err("update archive central directory runs past the file".into());
    }
    let mut at = cd_start;
    let mut copies = 0u32;
    let mut seen = 0u16;
    let mut name_and_extra = Vec::new();
    while at < cd_end {
        let mut header = [0u8; 46];
        read_exact_at(file, at, &mut header)?;
        if header.get(..4) != Some(&[0x50, 0x4b, 0x01, 0x02]) {
            return Err("update archive central directory is truncated".into());
        }
        let name_len = u64::from(le_u16(&header, 28)?);
        let extra_len = u64::from(le_u16(&header, 30)?);
        let comment_len = u64::from(le_u16(&header, 32)?);
        let step = 46u64
            .checked_add(name_len)
            .and_then(|n| n.checked_add(extra_len))
            .and_then(|n| n.checked_add(comment_len))
            .ok_or_else(|| "update archive central directory entry is out of range".to_string())?;
        let next = at
            .checked_add(step)
            .ok_or_else(|| "update archive central directory entry is out of range".to_string())?;
        if next > cd_end {
            return Err("update archive central directory entry runs past the directory".into());
        }
        let name_at = at
            .checked_add(46)
            .ok_or_else(|| "update archive member name is out of range".to_string())?;
        let name_bytes = usize::try_from(name_len)
            .map_err(|_| "update archive member name is too long".to_string())?;
        let extra_bytes = usize::try_from(extra_len)
            .map_err(|_| "update archive extra field is too long".to_string())?;
        // One buffer for every entry: name then extra field, each at most
        // 64 KiB, as the header records them.
        name_and_extra.resize(name_bytes.saturating_add(extra_bytes), 0);
        if !name_and_extra.is_empty() {
            read_exact_at(file, name_at, &mut name_and_extra)?;
        }
        let (raw_name, extra) = name_and_extra.split_at(name_bytes);
        let flags = le_u16(&header, 8)?;
        let name = crate::admission::zip_entry_name_raw(raw_name, flags, extra)
            .map_err(|e| format!("update archive member name cannot be read: {e}"))?;
        if member_names_match(name.as_bytes(), member) {
            copies = copies.saturating_add(1);
        }
        seen = seen
            .checked_add(1)
            .ok_or_else(|| "update archive has too many entries".to_string())?;
        at = next;
    }
    if seen != entries {
        return Err("update archive central directory does not match its end record".into());
    }
    Ok(Some(copies))
}

fn find_eocd(tail: &[u8]) -> Option<usize> {
    let last = tail.len().checked_sub(22)?;
    let mut at = last;
    loop {
        if tail.get(at..at.checked_add(4)?) == Some(&[0x50, 0x4b, 0x05, 0x06][..]) {
            let comment = usize::from(le_u16(tail, at.saturating_add(20)).ok()?);
            let end = at.checked_add(22)?.checked_add(comment)?;
            if end == tail.len() {
                return Some(at);
            }
        }
        if at == 0 {
            return None;
        }
        at = at.saturating_sub(1);
    }
}

fn le_u16(bytes: &[u8], at: usize) -> Result<u16, String> {
    let mut buf = [0u8; 2];
    buf.copy_from_slice(
        bytes
            .get(
                at..at
                    .checked_add(2)
                    .ok_or_else(|| "update archive field is out of range".to_string())?,
            )
            .ok_or_else(|| "update archive field is out of range".to_string())?,
    );
    Ok(u16::from_le_bytes(buf))
}

fn le_u32(bytes: &[u8], at: usize) -> Result<u32, String> {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(
        bytes
            .get(
                at..at
                    .checked_add(4)
                    .ok_or_else(|| "update archive field is out of range".to_string())?,
            )
            .ok_or_else(|| "update archive field is out of range".to_string())?,
    );
    Ok(u32::from_le_bytes(buf))
}

fn member_names_match(entry: &[u8], member: &str) -> bool {
    let entry = entry.strip_prefix(b"./").unwrap_or(entry);
    if entry.ends_with(b"/") {
        return false;
    }
    let member = member.strip_prefix("./").unwrap_or(member).as_bytes();
    entry.len() == member.len()
        && entry
            .iter()
            .zip(member)
            .all(|(got, want)| *got == *want || (*got == b'\\' && *want == b'/'))
}

fn read_exact_at(file: &mut std::fs::File, at: u64, buf: &mut [u8]) -> Result<(), String> {
    use std::io::{Read, Seek, SeekFrom};
    file.seek(SeekFrom::Start(at))
        .map_err(|e| format!("update archive seek failed: {e}"))?;
    file.read_exact(buf)
        .map_err(|e| format!("update archive read failed: {e}"))
}

/// Count the bytes the expected member actually inflates to, and stop one
/// byte past `budget`. The declared uncompressed size is not the limit:
/// `self_update` copies the stream with no cap of its own.
fn inflate_member_within_budget(
    archive: &std::path::Path,
    member: &str,
    budget: u64,
) -> Result<(), String> {
    use std::io::{Read, copy, sink};
    let file = std::fs::File::open(archive)
        .map_err(|e| format!("update archive {} cannot be opened: {e}", archive.display()))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| format!("update archive is not a readable zip: {e}"))?;
    let mut matched = None;
    for index in 0..zip.len() {
        let entry = zip
            .by_index(index)
            .map_err(|e| format!("update archive entry {index} cannot be read: {e}"))?;
        let name = entry
            .name()
            .map_err(|e| format!("update archive entry {index} has no name: {e}"))?;
        if member_names_match(name.as_bytes(), member) {
            matched = Some(index);
            break;
        }
    }
    let Some(index) = matched else {
        return Err(format!("update archive has no member {member}"));
    };
    let mut entry = zip
        .by_index(index)
        .map_err(|e| format!("update member {member} cannot be read: {e}"))?;
    let inflated = copy(
        &mut (&mut entry).take(budget.saturating_add(1)),
        &mut sink(),
    )
    .map_err(|e| format!("update member {member} could not be inflated: {e}"))?;
    if inflated > budget {
        Err(format!(
            "update member {member} inflates past the {budget}-byte budget"
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
    fn check_reports_and_never_installs() {
        let install = Plan::Install {
            from: "0.10.0".into(),
            to: "0.11.0".into(),
        };
        // --check turns an install decision into a report: Step::Install is
        // unreachable with check set, so nothing is ever downloaded or replaced.
        let look = Options {
            check: true,
            ..Options::default()
        };
        assert_eq!(
            step(&look, &install, "0.11.0"),
            Step::Report {
                text: "jubarte 0.10.0 is installed; 0.11.0 is available. Run `jubarte self-update` to install it.".into()
            }
        );
        // --check wins over --yes too: an explicit look never installs.
        let look_yes = Options {
            check: true,
            yes: true,
            ..Options::default()
        };
        assert!(matches!(
            step(&look_yes, &install, "0.11.0"),
            Step::Report { .. }
        ));
        // Without --check the same decision installs.
        assert_eq!(
            step(&Options::default(), &install, "0.11.0"),
            Step::Install {
                from: "0.10.0".into(),
                to: "0.11.0".into()
            }
        );
        // An up-to-date binary reports and stops whatever the flags say.
        let up_to_date = Plan::UpToDate {
            current: "0.11.0".into(),
        };
        for opts in [
            look,
            look_yes,
            Options {
                yes: true,
                ..Options::default()
            },
            Options::default(),
        ] {
            assert!(matches!(
                step(&opts, &up_to_date, "0.11.0"),
                Step::Report { .. }
            ));
        }
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

    /// ZIP-16 (#639): the binary swap must never install a link or a
    /// special file. `self_update`'s zip path refuses a symlink entry, but
    /// its tar path hands the entry to `tar`'s `unpack_in`, which restores
    /// a symlink as a real link whose target the crate does not confine to
    /// the staging dir — a crafted release could install any path as the
    /// new binary. The extracted path is checked with `symlink_metadata`
    /// (not `metadata`, which follows the link).
    #[test]
    fn a_special_extracted_binary_is_refused_before_the_swap() {
        let dir = tempfile::tempdir().unwrap();
        let regular = dir.path().join("jubarte");
        std::fs::write(&regular, b"binary").unwrap();
        extracted_binary_is_regular_file(&regular).expect("a regular file installs");
        extracted_binary_is_regular_file(dir.path())
            .expect_err("a directory at the extracted path must be refused");
        #[cfg(unix)]
        {
            let link = dir.path().join("linked");
            std::os::unix::fs::symlink(&regular, &link).unwrap();
            extracted_binary_is_regular_file(&link)
                .expect_err("a symlink at the extracted path must be refused");
            let dangling = dir.path().join("dangling");
            std::os::unix::fs::symlink(dir.path().join("no-such-target"), &dangling).unwrap();
            extracted_binary_is_regular_file(&dangling)
                .expect_err("a dangling symlink must be refused too");
        }
    }

    const UPDATE_MEMBER: &str = "jubarte-9.9.9-windows-x86_64/jubarte.exe";

    fn write_update_archive(dir: &std::path::Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn deflated_update_zip(member: &str, payload: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options: zip::write::FullFileOptions<'_, '_> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer.start_file(member, options).expect("start member");
        writer.write_all(payload).expect("write member");
        writer.finish().expect("finish zip").into_inner()
    }

    /// ZIP-28 (#656): the downloaded file itself has a byte budget, before
    /// any member is inflated.
    #[test]
    fn update_archive_over_the_download_budget_is_refused() {
        let bytes = deflated_update_zip(UPDATE_MEMBER, b"#!/jubarte");
        let dir = tempfile::tempdir().unwrap();
        let archive = write_update_archive(dir.path(), "jubarte.zip", &bytes);
        let budget = (bytes.len() as u64).saturating_sub(1);
        let err = confine_update_zip(&archive, UPDATE_MEMBER, budget)
            .expect_err("an archive larger than the download budget was accepted");
        assert!(
            err.contains("download budget"),
            "the refusal must name the download budget, got {err}"
        );
    }

    /// ZIP-28 (#656): the expected member can inflate far past the archive
    /// that carried it. The download fits; the unpacked member must not.
    #[test]
    fn update_archive_inflation_is_capped() {
        let payload = vec![0u8; 8_000];
        let bytes = deflated_update_zip(UPDATE_MEMBER, &payload);
        assert!(
            bytes.len() < payload.len(),
            "the fixture must be smaller on disk than the member it inflates to, got {} bytes",
            bytes.len()
        );
        let dir = tempfile::tempdir().unwrap();
        let archive = write_update_archive(dir.path(), "jubarte.zip", &bytes);
        let budget = bytes.len() as u64;
        let err = confine_update_zip(&archive, UPDATE_MEMBER, budget)
            .expect_err("an inflated member past the budget was extracted with no cap");
        assert!(
            err.contains("inflates past"),
            "the refusal must name the inflation cap, got {err}"
        );
        assert_eq!(
            std::fs::read_dir(dir.path()).unwrap().count(),
            1,
            "a refused archive must stay the only file in the directory"
        );
    }

    /// ZIP-28 (#656): two central-directory entries with the expected name
    /// are ambiguous. The first match must not be installed.
    #[test]
    fn update_archive_duplicate_member_is_refused() {
        let name = UPDATE_MEMBER.as_bytes();
        let bytes = crate::admission::zip_names::zip(&[
            crate::admission::zip_names::RawEntry::raw(name, b"FIRST"),
            crate::admission::zip_names::RawEntry::raw(name, b"SECOND"),
        ]);
        let dir = tempfile::tempdir().unwrap();
        let archive = write_update_archive(dir.path(), "jubarte.zip", &bytes);
        let err = confine_update_zip(&archive, UPDATE_MEMBER, UPDATE_ARCHIVE_BYTE_BUDGET)
            .expect_err("an archive that lists the expected member twice was accepted");
        assert!(
            err.contains("more than once"),
            "the refusal must name the repeated member, got {err}"
        );
    }

    /// #2084 review (Bugbot): a zipsign-signed zip starts with its
    /// signature block, not `PK`, and its offsets are not rewritten; zip and
    /// self_update still read it. The checks must run on it too.
    fn prefixed(prefix_len: usize, zip: &[u8]) -> Vec<u8> {
        let mut out = vec![0xA5u8; prefix_len];
        out.extend_from_slice(zip);
        out
    }

    #[test]
    fn update_archive_with_a_prefix_still_refuses_a_repeated_member() {
        let name = UPDATE_MEMBER.as_bytes();
        let bytes = prefixed(
            64,
            &crate::admission::zip_names::zip(&[
                crate::admission::zip_names::RawEntry::raw(name, b"FIRST"),
                crate::admission::zip_names::RawEntry::raw(name, b"SECOND"),
            ]),
        );
        let dir = tempfile::tempdir().unwrap();
        let archive = write_update_archive(dir.path(), "jubarte.zip", &bytes);
        let err = confine_update_zip(&archive, UPDATE_MEMBER, UPDATE_ARCHIVE_BYTE_BUDGET)
            .expect_err("a prefixed archive skipped the repeated-member check");
        assert!(err.contains("more than once"), "{err}");
    }

    #[test]
    fn update_archive_with_a_prefix_still_caps_inflation() {
        let payload = vec![0u8; 8_000];
        let bytes = prefixed(64, &deflated_update_zip(UPDATE_MEMBER, &payload));
        let dir = tempfile::tempdir().unwrap();
        let archive = write_update_archive(dir.path(), "jubarte.zip", &bytes);
        let budget = bytes.len() as u64;
        let err = confine_update_zip(&archive, UPDATE_MEMBER, budget)
            .expect_err("a prefixed archive skipped the inflation cap");
        assert!(err.contains("inflates past"), "{err}");
    }

    #[test]
    fn update_archive_with_a_prefix_and_one_member_is_allowed() {
        let bytes = prefixed(64, &deflated_update_zip(UPDATE_MEMBER, b"#!/jubarte"));
        let dir = tempfile::tempdir().unwrap();
        let archive = write_update_archive(dir.path(), "jubarte.zip", &bytes);
        confine_update_zip(&archive, UPDATE_MEMBER, UPDATE_ARCHIVE_BYTE_BUDGET)
            .expect("a signed (prefixed) release zip with one member");
    }

    /// #2084 review (Bugbot): zip names an entry by its adopted 0x7075
    /// Unicode Path, so a second entry whose raw name differs but whose
    /// Unicode Path is the member is the same installed file.
    #[test]
    fn update_archive_unicode_path_copy_is_a_repeated_member() {
        let name = UPDATE_MEMBER.as_bytes();
        let bytes = crate::admission::zip_names::zip(&[
            crate::admission::zip_names::RawEntry::raw(name, b"FIRST"),
            crate::admission::zip_names::RawEntry::raw(b"decoy.bin", b"SECOND").unicode_path(name),
        ]);
        let dir = tempfile::tempdir().unwrap();
        let archive = write_update_archive(dir.path(), "jubarte.zip", &bytes);
        let err = confine_update_zip(&archive, UPDATE_MEMBER, UPDATE_ARCHIVE_BYTE_BUDGET)
            .expect_err("a Unicode Path copy of the member was not counted");
        assert!(err.contains("more than once"), "{err}");
    }

    #[test]
    fn update_archive_single_member_is_allowed() {
        let bytes = deflated_update_zip(UPDATE_MEMBER, b"#!/jubarte");
        let dir = tempfile::tempdir().unwrap();
        let archive = write_update_archive(dir.path(), "jubarte.zip", &bytes);
        confine_update_zip(&archive, UPDATE_MEMBER, UPDATE_ARCHIVE_BYTE_BUDGET)
            .expect("one member inside the budget is the release zip");
    }

    #[test]
    fn update_archive_non_zip_inside_budget_is_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let archive = write_update_archive(dir.path(), "jubarte.tar.gz", b"not a zip");
        confine_update_zip(&archive, UPDATE_MEMBER, UPDATE_ARCHIVE_BYTE_BUDGET)
            .expect("a tar.gz inside the download budget is not a zip refusal");
    }
}
