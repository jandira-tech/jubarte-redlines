#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# criterion_abab.sh PREV_TAG STATE_DIR [CANDIDATE_DIR] — the release's
# criterion gate (#1797), the shape jubarte-loop/release_0.14.0/gates/
# criterion_abab.sh ran by hand. A = the previous release's tag, checked out
# at STATE_DIR/prev (reused when it already holds that tag); B = the
# candidate checkout (default: this script's crate root). All four runs go
# interleaved A B A B into ONE CRITERION_HOME under STATE_DIR — never /tmp,
# never a saved baseline file — and are then compared offline with
# --load-baseline (nothing re-measured):
#
#   a2 vs a1, b2 vs b1   this machine's noise, named on the report
#   b1,b2 vs a1,a2       the release's change; a case is a REGRESSION only
#                        when the MEAN of its four B-vs-A change points
#                        clears the noise floor below
#
# Noise floor: +10%. 0.14.0's own run moved the same binary against itself
# up to +9.7% (a2 vs a1, tables_bookmark_vmerge; b2 vs b1 +6.5%), so a
# single A-then-B run is not evidence and the gate's bar sits above that
# measured self-noise (#1797).
#
# One cargo at a time, each worktree in its own target dir (mbx); never
# CARGO_TARGET_DIR. The previous release's `jubarte` binary is built too:
# the parity ladder's --against names it.
set -u

PREV_TAG=${1:?usage: criterion_abab.sh PREV_TAG STATE_DIR [CANDIDATE_DIR]}
STATE=$(mkdir -p "${2:?usage: criterion_abab.sh PREV_TAG STATE_DIR [CANDIDATE_DIR]}" && cd "$2" && pwd)
CAND=${3:-$(cd "$(dirname "$0")/.." && pwd)}
CRITERION_HOME=$STATE/criterion_home
PREV_DIR=$STATE/prev
CMP_LOG=$STATE/criterion_abab_cmp.log
NOISE_FLOOR_PCT=10
mkdir -p "$CRITERION_HOME"
export CRITERION_HOME
: > "$CMP_LOG"

say() { printf '%s\n' "$*"; }
load() { command -v sysctl >/dev/null 2>&1 && sysctl -n vm.loadavg 2>/dev/null || echo "?"; }
die() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

# A's checkout: the previous release's tag in its own worktree, reused only
# when it already holds that tag.
if [ -d "$PREV_DIR" ]; then
  [ "$(git -C "$PREV_DIR" rev-parse HEAD)" = "$(git rev-parse "$PREV_TAG^{commit}")" ] \
    || die "$PREV_DIR holds $(git -C "$PREV_DIR" rev-parse --short HEAD), not $PREV_TAG — remove it and rerun"
else
  git worktree add "$PREV_DIR" "$PREV_TAG" \
    || die "cannot check out $PREV_TAG at $PREV_DIR"
fi

# The ladder's --against binary: the previous release's jubarte.
say "== build the previous release's jubarte in $PREV_DIR"
(cd "$PREV_DIR" && cargo build --release --bin jubarte) \
  || die "cargo build failed in $PREV_DIR"

fail=0
run() { # run WORKTREE BASELINE
  say "== $2 ($1) $(date '+%H:%M:%S') load $(load)"
  if (cd "$1" && cargo bench --bench redline -- --save-baseline "$2") \
       > "$STATE/criterion_$2.log" 2>&1; then
    say "   exit 0 $(date '+%H:%M:%S')"
  else
    say "   FAILED — see $STATE/criterion_$2.log"
    fail=1
  fi
}
run "$PREV_DIR" a1
run "$CAND" b1
run "$PREV_DIR" a2
run "$CAND" b2

cmp_run() { # cmp_run NEW OLD: NEW's saved run against OLD's, nothing re-measured
  printf '### %s vs %s\n' "$1" "$2" >> "$CMP_LOG"
  say "== $1 vs $2"
  (cd "$CAND" && cargo bench --bench redline -- --load-baseline "$1" --baseline "$2") 2>&1 \
    | tee -a "$CMP_LOG" \
    | grep -E '^[a-zA-Z0-9_.-]+/|time:|change:|Performance|No change|Change within' \
    | sed 's/^/   /'
}
# self-noise first, then every B-vs-A combination
cmp_run a2 a1
cmp_run b2 b1
cmp_run b1 a1
cmp_run b2 a2
cmp_run b1 a2
cmp_run b2 a1

say "== gate: a case regresses only when its mean B-vs-A change clears the noise floor"
say "   noise floor +${NOISE_FLOOR_PCT}% (0.14.0 measured +9.7% A-vs-A self-noise, #1797)"
awk -v floor="$NOISE_FLOOR_PCT" '
  /^### /  { vs = $2 " vs " $4; nm = "" ; next }
  /^[a-zA-Z0-9_.-]+\// { nm = $1; next }
  /^[[:space:]]*change:/ {
    line = $0; i = 0
    while (match(line, /[-+][0-9.]+%/)) {
      i++; pct[i] = substr(line, RSTART, RLENGTH) + 0
      line = substr(line, RSTART + RLENGTH)
    }
    if (!i) next
    mid = (i >= 2) ? pct[2] : pct[1]
    sum[vs "|" nm] += mid; cnt[vs "|" nm]++
    next
  }
  END {
    bad = 0
    for (k in cnt) {
      split(k, a, "|"); vs = a[1]; nm = a[2]
      mean = sum[k] / cnt[k]
      printf "   %-40s %s mean %+.1f%%\n", nm, vs, mean
      # gated: B against A only; a2-a1 and b2-b1 are the noise report
      if (vs ~ /^b. vs a/ && mean > floor) {
        printf "REGRESSION %s mean %+.1f%% clears the +%d%% noise floor\n", nm, mean, floor
        bad = 1
      }
    }
    exit bad
  }
' "$CMP_LOG" || fail=1

if [ "$fail" = 0 ]; then
  say "done $(date '+%H:%M:%S') — no regression past the noise floor"
else
  say "done $(date '+%H:%M:%S') — FAILED (a failed run or a regression above)"
fi
exit "$fail"
