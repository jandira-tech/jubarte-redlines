#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# Publish the workspace's nine crates to crates.io (plan T6.2).
#
#   scripts/publish_crates.sh VERSION                 publish, resuming
#   scripts/publish_crates.sh VERSION --dry-run       package and verify all
#   scripts/publish_crates.sh VERSION --dry-run --as-prerelease
#
# A workspace publish is not atomic: a run that stops after jubarte-opc
# leaves dom and opc live. So each crate is published on its own, in the
# order its dependencies need, and crates.io is asked for <crate>/<VERSION>
# first: a crate already live is skipped, and a rerun resumes where the
# last one stopped. cargo publish waits until the index serves a crate, so
# the next crate resolves it.
#
# Before anything uploads, the packaged jubarte manifest must still carry
# VERSION's [package.metadata.release-notes] entry (release.sh step 4
# writes it; cargo normalizes the manifest when it packages).
#
# --dry-run packages and verifies every crate from its tarball with
# `cargo publish --workspace --dry-run`. --as-prerelease first moves the
# workspace version to VERSION-dryrun (CI, before the release bump): the
# published jubarte-redlines 0.18.0 is the old single crate, and verifying
# against its version would resolve that crate instead of this one. It
# rewrites Cargo.toml and Cargo.lock, so use it only on a throwaway
# checkout.
#
# Environment (tests): CARGO (default cargo), CRATES_IO_API (default
# https://crates.io/api/v1; a file:// root works).
set -euo pipefail

# Dependencies first; tests/crate_layering.rs's LAYERS, then the facade.
CRATES=(jubarte-dom jubarte-opc jubarte-redlines jubarte-render jubarte-doc
        jubarte-markdown jubarte-edit jubarte-cli jubarte)

CARGO=${CARGO:-cargo}
API=${CRATES_IO_API:-https://crates.io/api/v1}
AGENT="jubarte-release (github.com/jandira-tech/jubarte-redlines)"

die() { echo "publish_crates: $*" >&2; exit 1; }

[ $# -ge 1 ] || die "usage: publish_crates.sh VERSION [--dry-run [--as-prerelease]]"
VER=$1; shift
DRY=0; PRE=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY=1 ;;
    --as-prerelease) PRE=1 ;;
    *) die "unknown argument: $arg" ;;
  esac
done
[ "$PRE" = 0 ] || [ "$DRY" = 1 ] || die "--as-prerelease only goes with --dry-run"
[[ "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+].*)?$ ]] || die "not a version: $VER"

root=$(git rev-parse --show-toplevel)
cd "$root"

# crates.io answers 403 to a request without a User-Agent.
live() { curl -sf -A "$AGENT" "$API/crates/$1/$VER" >/dev/null; }

if [ "$DRY" = 1 ]; then
  if [ "$PRE" = 1 ]; then
    cur=$(sed -n 's/^version = "\([^"]*\)" # the release script bumps.*/\1/p' Cargo.toml)
    [ -n "$cur" ] || die "no workspace version line in Cargo.toml"
    [ "$cur" = "$VER" ] || die "Cargo.toml is at $cur, not $VER"
    VER="$VER-dryrun"
    # The workspace version and every "=" pin move together.
    sed -i.bak -e "s/^version = \"$cur\" # the release script bumps/version = \"$VER\" # the release script bumps/" \
               -e "s/version = \"=$cur\"/version = \"=$VER\"/g" Cargo.toml
    rm -f Cargo.toml.bak
    "$CARGO" update --workspace --quiet
    echo "workspace moved to $VER for the dry run"
    exec "$CARGO" publish --workspace --dry-run --allow-dirty
  fi
  exec "$CARGO" publish --workspace --dry-run --locked --allow-dirty
fi

# The release note must survive packaging before any crate uploads.
"$CARGO" package -p jubarte --locked --no-verify >/dev/null
crate="target/package/jubarte-$VER.crate"
[ -f "$crate" ] || die "$crate was not written"
tar -xzOf "$crate" "jubarte-$VER/Cargo.toml" \
  | sed -n '/^\[package.metadata.release-notes\]/,/^\[/p' \
  | grep -F "\"$VER\" = " >/dev/null \
  || die "no [package.metadata.release-notes] entry for $VER in the packaged jubarte manifest: not publishing"

published=0; skipped=0
for name in "${CRATES[@]}"; do
  if live "$name"; then
    echo "$name $VER is already on crates.io: skipped"
    skipped=$((skipped + 1))
    continue
  fi
  echo "publishing $name $VER"
  "$CARGO" publish -p "$name" --locked
  published=$((published + 1))
done
echo "crates.io: $published published, $skipped already live"
