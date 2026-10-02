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
4. Once release signing is on (see [Signed releases](#signed-releases)),
   also checks the archive's signature against the release keys built
   into the binary. An unsigned archive, or one no trusted key signed, is
   refused and nothing is written.
5. Replaces the running binary with the one in the archive.

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

## Signed releases

`SHA256SUMS.txt` catches a corrupted download, not a tampered one: whoever
can replace a release archive can replace the sums file next to it. A
signature closes that gap. Release archives are signed with
[zipsign](https://github.com/Kijewski/zipsign) (ed25519), and the
updater checks the signature against public keys compiled into the
binary, which an attacker who controls only the release assets cannot
change.

Signing is staged so that no existing release or installed binary breaks:

- Until a public key is committed, the trusted key list
  (`RELEASE_KEYS` in `src/update.rs`) is empty and the updater checks
  `SHA256SUMS.txt` only, exactly as before.
- `FIRST_SIGNED_RELEASE` in `src/update.rs` names the first signed
  release. A binary that trusts a key requires a valid signature on that
  release and every later one.
- Binaries released before signing began carry no key. They update to the
  first signed release by checksum only; from then on, each update is
  signature-checked.
- `jubarte self-update --version` to a release older than
  `FIRST_SIGNED_RELEASE` installs an archive that was never signed, so
  that downgrade is checked by its SHA-256 alone, and the command says so.
- A pre-release of the first signed version (`0.11.0-rc.1` before
  `0.11.0`) sorts before it and is checksum-only too.

The signature is bound to the archive's file name (zipsign's default
context), which is the asset name the updater downloads. Renaming an
archive after signing, or serving one platform's archive under another's
name, fails the check.

### Turning signing on (maintainers)

Do these steps once, in order, before the release that should be the
first signed one. `X.Y.Z` below is that release.

1. Install the signer, at the version `release.yml` uses:

   ```bash
   cargo install zipsign --locked --version 0.2.1
   ```

2. Generate the key pair outside the repository. The private key never
   enters git (`.gitignore` ignores everything under `keys/` except
   `*.pub`, as a backstop):

   ```bash
   zipsign gen-key ~/secure/jubarte-release-signing.key keys/release-signing.pub
   ```

   Keep the private key offline or in a password manager. Anyone holding
   it can sign releases that every signed-era binary will install.

3. Store the private key as the repository secret `ZIPSIGN_PRIVATE_KEY`,
   base64 on one line:

   ```bash
   base64 < ~/secure/jubarte-release-signing.key | tr -d '\n' \
     | gh secret set ZIPSIGN_PRIVATE_KEY --repo jandira-tech/jubarte-redlines
   ```

   From now on `release.yml` signs every archive. A release with the
   secret set but no key committed yet is signed but not checked by
   anyone, which is a harmless dry run.

4. Commit the public key and turn the check on, in one commit. In
   `src/update.rs`:

   ```rust
   const RELEASE_KEYS: &[self_update::VerifyingKey] =
       &[*include_bytes!("../keys/release-signing.pub")];
   const FIRST_SIGNED_RELEASE: Option<&str> = Some("X.Y.Z");
   ```

   A key file that is not exactly 32 bytes fails the build, and the
   `the_shipped_key_set_and_threshold_agree` test fails if only one of
   the two constants is set. Never raise `FIRST_SIGNED_RELEASE` once it
   ships: later builds would then accept unsigned archives for the
   releases in between.

5. Cut release `X.Y.Z` as usual (`scripts/release.sh`, VERSIONING.md).
   With a key committed, `release.yml` refuses to publish when the secret
   is missing, and it checks every signed archive against every committed
   `keys/*.pub`, so a secret that does not match the committed key stops
   the release before anything is published. To check a published
   archive by hand:

   ```bash
   zipsign verify tar jubarte-X.Y.Z-linux-x86_64.tar.gz keys/release-signing.pub
   zipsign verify zip jubarte-X.Y.Z-windows-x86_64.zip keys/release-signing.pub
   ```

   Verify under the asset's own file name: the name is part of what was
   signed.

If `release.yml` skips the GitHub release and `scripts/release.sh`
publishes it from the workflow artifacts (VERSIONING.md, "When release.yml
skips the GitHub release"), those archives were never signed. Signed-era
binaries refuse them, so that release does not reach them through
`self-update`. Ship a signed patch release instead.

### Rotating the key

zipsign checks any-of: an archive may carry several signatures, and it
passes when any trusted key matches one of them.

1. Generate the new pair as above, as `keys/release-signing-2.pub`.
2. Add the new key to `RELEASE_KEYS` next to the old one.
3. Set `ZIPSIGN_PRIVATE_KEY` to both private keys, one base64 key per
   line. Every archive then carries both signatures, so binaries that
   trust only the old key keep updating, and so do new ones.
4. Release with both keys for as long as old binaries need to catch up.
5. Then delete the old `.pub`, drop it from `RELEASE_KEYS`, and set the
   secret to the new key alone. A binary that still trusts only the old
   key can no longer update itself and must be reinstalled by hand.

If the old key leaked, skip the overlap: drop it from `RELEASE_KEYS` and
the secret at once. Whoever holds it can sign for the binaries that trust
it, so those must be reinstalled by hand either way.

## Building without it

Self-update is the `self-update` Cargo feature, on by default. Build with
`--no-default-features --features cli` for a binary with no network code;
the command then reports that it was not built in.

The implementation is the [`self_update`](https://crates.io/crates/self_update)
crate (release lookup, checksum and signature verification, archive
extraction, binary replacement) in `src/update.rs`.
`tests/self_update_signature.rs` signs a throwaway archive the way
`release.yml` does and checks it with the updater's verifier.
