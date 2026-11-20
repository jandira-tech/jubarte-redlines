#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: Apache-2.0
#
# Publish the workspace's nine crates to crates.io (plan T6.2).
#
#   scripts/publish_crates.sh VERSION             publish, resuming
#   scripts/publish_crates.sh VERSION --pending   print pending=<n>, publish nothing
#   scripts/publish_crates.sh VERSION --dry-run   package and verify all nine
#
# A workspace publish is not atomic: a run that stops after jubarte-opc
# leaves dom and opc live. So each crate is published on its own, in the
# order its dependencies (and its tests' dev-dependencies, which cargo
# resolves when it verifies a package) need, and crates.io is asked for
# <crate>/<VERSION> first: a crate already live is skipped, and a rerun
# resumes where the last one stopped. The API can lag the index right
# after a publish, so cargo's own "already exists" also counts as live.
# An answer other than 200 or 404 stops the run: an unreachable crates.io
# is not evidence that a crate is missing. cargo publish waits until the
# index serves a crate, so the next crate resolves it.
#
# Before anything uploads, the packaged jubarte manifest must still carry
# VERSION's [package.metadata.release-notes] entry (release.sh step 4
# writes it; cargo normalizes the manifest when it packages).
#
# --pending lets release.yml skip its token exchange when every crate is
# already live (a rerun, or a release.sh --local-publish that got there
# first), and leave a crate's first release to a token: crates.io registers
# a trusted publisher only on a crate that exists.
#
# --dry-run packages and verifies every crate from its tarball with
# `cargo publish --workspace --dry-run`, in a fresh target directory:
# cargo unpacks its publish-local registry under a key made from that
# registry's path, and reuses an earlier unpacked crate of the same name
# and version, so a reused target verifies stale sources.
#
# Environment (tests): CARGO (default cargo), CRATES_IO_API (default
# https://crates.io/api/v1), CRATES_IO_MAX_TIME (seconds per probe, 30).
set -euo pipefail

# Dependencies first; tests/crate_layering.rs's LAYERS, then the facade.
CRATES=(jubarte-dom jubarte-opc jubarte-redlines jubarte-render jubarte-doc
        jubarte-markdown jubarte-edit jubarte-cli jubarte)

CARGO=${CARGO:-cargo}
API=${CRATES_IO_API:-https://crates.io/api/v1}
AGENT="jubarte-release (github.com/jandira-tech/jubarte)"

die() { echo "publish_crates: $*" >&2; exit 1; }

[ $# -ge 1 ] || die "usage: publish_crates.sh VERSION [--pending | --dry-run]"
VER=$1; shift
MODE=publish
for arg in "$@"; do
  case "$arg" in
    --dry-run) MODE=dry-run ;;
    --pending) MODE=pending ;;
    *) die "unknown argument: $arg" ;;
  esac
done
[[ "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+].*)?$ ]] || die "not a version: $VER"

root=$(git rev-parse --show-toplevel)
cd "$root"

# Ask crates.io for $1 (a path under /crates/); 0 for 200, 1 for 404, and
# any other answer stops the run. crates.io answers 403 to a request
# without a User-Agent. Bounded: a stalled answer must not hold the release
# job until the runner is killed; a timeout reads as 000.
ask() {
  local code
  code=$(curl -s -o /dev/null -w '%{http_code}' --connect-timeout 10 \
    --max-time "${CRATES_IO_MAX_TIME:-30}" -A "$AGENT" "$API/crates/$1") || code=000
  case "$code" in
    200) return 0 ;;
    404) return 1 ;;
    *) die "crates.io answered $code for $1: cannot tell whether it is live" ;;
  esac
}
live() { ask "$1/$VER"; }   # crates.io holds <crate>/$VER
known() { ask "$1"; }       # crates.io holds any version of <crate>

if [ "$MODE" = dry-run ]; then
  scratch=$(mktemp -d "${TMPDIR:-/tmp}/publish-crates.XXXXXX")
  # A whole verification build: gone when the run ends, pass or fail.
  trap 'rm -rf -- "$scratch"' EXIT
  "$CARGO" publish --workspace --dry-run --locked --allow-dirty --target-dir "$scratch"
  exit 0
fi

if [ "$MODE" = pending ]; then
  # Prints new=<n> (crates crates.io has never seen, which only a token
  # can publish) and, last, pending=<n> (crates missing $VER).
  pending=0; new=0
  for name in "${CRATES[@]}"; do
    # ask dies on an unclear answer; `if` would swallow that, so test $?.
    status=0; live "$name" || status=$?
    [ "$status" = 0 ] && continue
    pending=$((pending + 1))
    status=0; known "$name" || status=$?
    if [ "$status" = 0 ]; then
      echo "$name $VER is not on crates.io"
    else
      echo "$name has never been published to crates.io"
      new=$((new + 1))
    fi
  done
  echo "new=$new"
  echo "pending=$pending"
  exit 0
fi

# The release note must survive packaging before any crate uploads. The
# whole workspace is packaged: alone, jubarte resolves its exact-pinned
# workspace crates on crates.io, where a first release has none yet.
"$CARGO" package --workspace --locked --no-verify >/dev/null
crate="target/package/jubarte-$VER.crate"
[ -f "$crate" ] || die "$crate was not written"
tar -xzOf "$crate" "jubarte-$VER/Cargo.toml" \
  | sed -n '/^\[package.metadata.release-notes\]/,/^\[/p' \
  | grep -F "\"$VER\" = " >/dev/null \
  || die "no [package.metadata.release-notes] entry for $VER in the packaged jubarte manifest: not publishing"

published=0; skipped=0
for name in "${CRATES[@]}"; do
  status=0; live "$name" || status=$?
  if [ "$status" = 0 ]; then
    echo "$name $VER is already on crates.io: skipped"
    skipped=$((skipped + 1))
    continue
  fi
  echo "publishing $name $VER"
  out=0; log=$("$CARGO" publish -p "$name" --locked 2>&1) || out=$?
  printf '%s\n' "$log"
  if [ "$out" != 0 ]; then
    if grep -qE "already (exists|uploaded)" <<<"$log"; then
      echo "$name $VER is already on the crates.io index: skipped"
      skipped=$((skipped + 1))
      continue
    fi
    die "cargo publish -p $name failed (above); crates before it are live, rerun to resume"
  fi
  published=$((published + 1))
done
echo "crates.io: $published published, $skipped already live"
