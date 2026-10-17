#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
#
# check_engine_commit.sh [REV]: refuse an npm build that REV's
# jubarte-wasm/npm/ENGINE_COMMIT.txt cannot vouch for.
#
# build-npm.sh stamps the commit it built from. The stamp vouches for REV
# (default HEAD) only when that commit is in REV's history and no engine
# input changed between them; otherwise npm would ship artifacts that REV's
# sources do not produce. The stamp is read from REV's tree, not the working
# tree, so release.sh can check the tagged commit itself.
#
# Exit 0: the build matches REV's engine. 1: it does not. 2: usage error.
set -euo pipefail
cd "$(dirname "$0")/.."

STAMP_FILE=jubarte-wasm/npm/ENGINE_COMMIT.txt
# Every input to the wasm build: the crate and its lockfile, the files it
# embeds with include_bytes!/include_str! (fonts, the release key, the plan
# schema; test-only embeds of tests/fixtures excluded), and the binding crate
# with its .cargo/config.toml RUSTFLAGS. The binding's lockfile is checked
# apart, below.
ENGINE_PATHS=(
  src Cargo.toml Cargo.lock assets keys schemas
  jubarte-wasm/src jubarte-wasm/Cargo.toml jubarte-wasm/.cargo
)
WASM_LOCK=jubarte-wasm/Cargo.lock

refuse() { echo "check_engine_commit: $*" >&2; exit 1; }

[ $# -le 1 ] || { echo "usage: $0 [REV]" >&2; exit 2; }
rev=${1:-HEAD}
git rev-parse -q --verify "$rev^{commit}" >/dev/null \
  || { echo "check_engine_commit: $rev is not a revision of this repository" >&2; exit 2; }

stamp=$(git show "$rev:$STAMP_FILE" 2>/dev/null | tr -d '[:space:]' || true)
[ -n "$stamp" ] || refuse "$rev has no $STAMP_FILE: the npm build names no engine commit"
git rev-parse -q --verify "$stamp^{commit}" >/dev/null \
  || refuse "$STAMP_FILE names $stamp, which is not a commit in this repository"
git merge-base --is-ancestor "$stamp" "$rev" \
  || refuse "$STAMP_FILE names ${stamp:0:12}, which is not an ancestor of $rev: its artifacts cannot be rebuilt from $rev's history"

changed=$(git diff --name-only "$stamp" "$rev" -- "${ENGINE_PATHS[@]}")
# build-npm.sh re-resolves the binding's lockfile after its clean-tree check,
# and release.sh step 8 commits the result with the artifacts: the lock may
# change in the artifacts commit (the last to write the stamp) and nowhere
# else between the stamp and REV.
artifacts=$(git log -1 --format=%H "$stamp..$rev" -- "$STAMP_FILE")
if [ -n "$artifacts" ]; then
  lock=$(git diff --name-only "$stamp" "$artifacts^" -- "$WASM_LOCK"
         git diff --name-only "$artifacts" "$rev" -- "$WASM_LOCK")
else
  lock=$(git diff --name-only "$stamp" "$rev" -- "$WASM_LOCK")
fi
changed=$(printf '%s\n%s\n' "$changed" "$lock" | sed '/^$/d' | sort -u)
if [ -n "$changed" ]; then
  {
    echo "check_engine_commit: the engine changed after the npm build (${stamp:0:12}); rebuild with jubarte-wasm/build-npm.sh:"
    printf '%s\n' "$changed" | head -20 | sed 's/^/  /'
    n=$(printf '%s\n' "$changed" | wc -l | tr -d ' ')
    [ "$n" -le 20 ] || echo "  … and $((n - 20)) more"
  } >&2
  exit 1
fi
echo "npm build matches the engine at $(git rev-parse --short "$rev^{commit}") (built from ${stamp:0:12})"
