<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# `jubarte self-update`

The CLI updates itself from the project's GitHub releases, and only when
you run this command. No other command contacts the network, and nothing
checks for updates in the background.

```text
jubarte self-update --check          # print the installed and latest versions
jubarte self-update                  # ask, then install the latest release
jubarte self-update --yes            # install without asking (scripts, CI)
jubarte self-update --version 0.10.0 # install that release (also a downgrade)
```

## What it does

1. Reads the release list of `jandira-tech/jubarte-redlines` from the
   GitHub API. `GH_TOKEN` or `GITHUB_TOKEN`, when set, lifts GitHub's
   unauthenticated budget of 60 requests an hour.
2. Picks the archive for this machine:
   `jubarte-<version>-<os>-<arch>.tar.gz` (`.zip` on Windows), where
   `<os>` is `macos`, `linux` or `windows` and `<arch>` is `aarch64` or
   `x86_64`.
3. Downloads it and checks its SHA-256 against the release's
   `SHA256SUMS.txt`. A release without that file, or an archive whose hash
   differs, is refused and nothing is written.
4. Replaces the running binary with the one in the archive.

It never installs an older release unless you name it with `--version`,
and it asks before replacing the binary unless you pass `--yes`. Without a
terminal to ask on (a pipe, CI) it refuses up front, before contacting
GitHub, unless `--yes` or `--check` is given.

The archive's `fonts/` folder is not installed. Fonts live in jubarte's
per-user font folder; re-run `scripts/install.sh --fonts-only` from a
checkout to refresh them.

## Other installs

The command first ships in 0.10.0: install that release by hand once,
and later releases arrive through `jubarte self-update`.

A binary installed by `cargo install` can update this way too; the next
`cargo install` will simply replace it again. The Python package
(`pip install jubarte-redlines`) and the npm package (`jubarte-wasm`) are
libraries: update them with their package managers.

## Building without it

Self-update is the `self-update` Cargo feature, on by default. Build with
`--no-default-features --features cli` for a binary with no network code;
the command then reports that it was not built in.

The implementation is the [`self_update`](https://crates.io/crates/self_update)
crate (release lookup, checksum verification, archive extraction, binary
replacement) in `src/update.rs`.
