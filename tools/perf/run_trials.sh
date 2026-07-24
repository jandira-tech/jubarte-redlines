#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# P0-LAB-01 — thin wrapper: build named binaries + run permanent ABBA matrix + summarize.
#
# Usage:
#   tools/perf/run_trials.sh <base_bin_or_commit> <cand_bin_or_commit> <out_dir> [rounds]
#
# If args are paths to executables, they are used as-is. Otherwise treated as
# git commits: builds release binaries into out_dir/bins/{base,cand}.
#
# Refuses to start when 1-minute loadavg > ncpu (contaminated wall).
# Override with ALLOW_LOAD=1.
set -euo pipefail

BASE_IN="${1:?usage: run_trials.sh <base> <cand> <out_dir> [rounds]}"
CAND_IN="${2:?}"
OUT="${3:?}"
ROUNDS="${4:-2}"
CRATE="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$CRATE"

mkdir -p "$OUT/bins"

# Load gate (macOS sysctl + Linux /proc/loadavg — CR #3642397978)
if [[ "${ALLOW_LOAD:-0}" != "1" ]]; then
  loadavg=""
  if loadavg=$(sysctl -n vm.loadavg 2>/dev/null | awk '{print $2}'); then
    :
  elif [[ -r /proc/loadavg ]]; then
    loadavg=$(awk '{print $1}' /proc/loadavg)
  fi
  if [[ -n "$loadavg" ]]; then
    ncpu=$(getconf _NPROCESSORS_ONLN 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1)
    # bash arithmetic needs integers — scale *100
    la100=$(python3 -c "print(int(float('$loadavg')*100))")
    nc100=$((ncpu * 100))
    if (( la100 > nc100 )); then
      echo "error: loadavg $loadavg > ncpu $ncpu; set ALLOW_LOAD=1 to override" >&2
      exit 3
    fi
  fi
fi

resolve_bin() {
  local in="$1" label="$2"
  if [[ -x "$in" ]]; then
    echo "$in"
    return
  fi
  # build from commit
  local dest="$OUT/bins/${label}"
  echo "building $label from $in → $dest" >&2
  git archive "$in" | tar -x -C "$OUT/bins" 2>/dev/null || {
    # fallback: cargo build current tree for "HEAD"/"." 
    true
  }
  if [[ "$in" == "HEAD" || "$in" == "." || "$in" == "current" ]]; then
    cargo build --release --bin jubarte --features cli 2>"$OUT/bins/${label}.build.log"
    cp -f target/release/jubarte "$dest"
    echo "$dest"
    return
  fi
  echo "error: not an executable and commit-build not fully supported for '$in'" >&2
  echo "pass an executable path for base/cand" >&2
  exit 2
}

BASE="$(resolve_bin "$BASE_IN" base)"
CAND="$(resolve_bin "$CAND_IN" cand)"

echo "base=$BASE" | tee "$OUT/bins.txt"
echo "cand=$CAND" | tee -a "$OUT/bins.txt"
shasum -a 256 "$BASE" "$CAND" | tee -a "$OUT/bins.txt"

"$CRATE/tools/perf/run_abba_matrix.sh" "$BASE" "$CAND" "$OUT" "$ROUNDS"
# Enforce wall-time gate by default (CR #3642397982). Opt out with ALLOW_REGRESS=1.
if [[ "${ALLOW_REGRESS:-0}" == "1" ]]; then
  python3 "$CRATE/tools/perf/summarize.py" "$OUT/summary.tsv" --json "$OUT/verdict.json" --allow-regress
else
  python3 "$CRATE/tools/perf/summarize.py" "$OUT/summary.tsv" --json "$OUT/verdict.json"
fi
echo "wrote $OUT/verdict.json"
