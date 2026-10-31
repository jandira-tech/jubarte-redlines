#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# criterion_abab.sh PREV_TAG STATE_DIR [CANDIDATE] — the release's
# criterion gate (#1797), the shape jubarte-loop/release_0.14.0/gates/
# criterion_abab.sh ran by hand. A = the previous release's tag; B = the
# candidate: a directory or a revision (the release's own tag when
# resuming), default this checkout's HEAD — a revision is benched in a
# clean worktree under STATE_DIR, never the caller's dirty checkout.
# Checkouts are kept for reuse only while clean and at the asked-for rev;
# a dirty or wrong-rev one is discarded and recreated. All four runs go
# interleaved A B A B into ONE CRITERION_HOME under STATE_DIR — never
# /tmp, never a saved baseline file — and are then compared offline with
# --load-baseline (nothing re-measured, printed context only):
#
#   a2 vs a1, b2 vs b1   this machine's noise, beside the verdict
#   b1,b2 vs a1,a2       the release's change
#
# The verdict never parses that text (criterion 0.8.2 prints a Unicode
# minus U+2212, and its "midpoint" is a point estimate, not a middle):
# each case reads its four runs' mean point estimates from criterion's
# saved estimates (CRITERION_HOME/<group>/<bench>/<run>/estimates.json,
# mean.point_estimate):
#
#   delta        = mean(b1,b2)/mean(a1,a2) - 1      per case
#   self-noise   = max(|a2/a1-1|, |b2/b1-1|)        this run, that case
#   REGRESSION   delta > 5% AND delta > self-noise  — the issue's >5% bar
#   UNDECIDABLE  delta > 5% but not past a self-noise above 5%
#                (rerun on a quieter machine)
#
# The gate fails on any REGRESSION and on any UNDECIDABLE — it never
# passes them — and on a failed cargo call, a missing estimates file, a
# case benched on one side only, or a differing case count.
#
# One cargo at a time, each worktree in its own target dir (mbx); never
# CARGO_TARGET_DIR. The previous release's `jubarte` binary is built too:
# the parity ladder's --against names it.
set -u
set -o pipefail

PREV_TAG=${1:?usage: criterion_abab.sh PREV_TAG STATE_DIR [CANDIDATE]}
STATE=$(mkdir -p "${2:?usage: criterion_abab.sh PREV_TAG STATE_DIR [CANDIDATE]}" && cd "$2" && pwd)
CAND=${3:-HEAD}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
CRITERION_HOME=$STATE/criterion_home
PREV_DIR=$STATE/prev
CAND_DIR=$STATE/cand
CMP_LOG=$STATE/criterion_abab_cmp.log
mkdir -p "$CRITERION_HOME"
export CRITERION_HOME
: > "$CMP_LOG"

say() { printf '%s\n' "$*"; }
load() { command -v sysctl >/dev/null 2>&1 && sysctl -n vm.loadavg 2>/dev/null || echo "?"; }
die() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

# checkout_at NAME REV: NAME under the state dir is a clean worktree at
# REV; a reused checkout only counts while `git status --porcelain` is
# empty and its HEAD is REV's commit — anything else is discarded and
# recreated (#1797).
checkout_at() {
  local dir=$STATE/$1 rev=$2
  if [ -d "$dir" ]; then
    if [ "$(git -C "$dir" rev-parse HEAD)" = "$(git rev-parse "$rev^{commit}")" ] \
       && [ -z "$(git -C "$dir" status --porcelain)" ]; then
      return 0
    fi
    say "   $dir is dirty or not at $rev — discarded"
    git worktree remove --force "$dir" 2>/dev/null || { rm -rf "$dir"; git worktree prune; }
  fi
  git worktree add "$dir" "$rev" \
    || die "cannot check out $rev at $dir"
}

# A's checkout: the previous release's tag.
checkout_at prev "$PREV_TAG"

# B: a directory as given; a revision in a clean worktree of its own
# (HEAD itself stays in this checkout).
if [ -d "$CAND" ]; then
  B=$(cd "$CAND" && pwd)
elif [ "$CAND" = HEAD ]; then
  B=$ROOT
else
  checkout_at cand "$CAND"
  B=$CAND_DIR
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
run "$B" b1
run "$PREV_DIR" a2
run "$B" b2

cmp_run() { # cmp_run NEW OLD: NEW's saved run against OLD's, printed context only
  printf '### %s vs %s\n' "$1" "$2" >> "$CMP_LOG"
  say "== $1 vs $2"
  if ! (cd "$ROOT" && cargo bench --bench redline -- --load-baseline "$1" --baseline "$2") 2>&1 \
       | tee -a "$CMP_LOG" \
       | grep -E '^[a-zA-Z0-9_.-]+/|time:|change:|Performance|No change|Change within' \
       | sed 's/^/   /'; then
    say "   comparison failed — the gate below cannot pass"
    fail=1
  fi
}
# self-noise first, then every B-vs-A combination
cmp_run a2 a1
cmp_run b2 b1
cmp_run b1 a1
cmp_run b2 a2
cmp_run b1 a2
cmp_run b2 a1

say "== gate: per case, delta = mean(b1,b2)/mean(a1,a2) - 1 against the >5% bar"
say "   and this run's self-noise max(|a2/a1-1|, |b2/b1-1|), from saved estimates"
python3 - "$CRITERION_HOME" <<'PY' || fail=1
import json
import sys
from pathlib import Path

home = Path(sys.argv[1])
RUNS = ("a1", "a2", "b1", "b2")


def point_estimate(path):
    try:
        with path.open() as f:
            return float(json.load(f)["mean"]["point_estimate"])
    except (OSError, KeyError, TypeError, ValueError):
        return None


# every <group>/<bench> that any of the four runs saved an estimate for
cases = {}
for est in sorted(home.glob("*/*/*/estimates.json")):
    run = est.parent.name
    if run not in RUNS:
        continue
    case = est.parent.parent.relative_to(home).as_posix()
    cases.setdefault(case, {})[run] = point_estimate(est)

bad = False
if not cases:
    print("   no saved estimates under", home, "— nothing measured", file=sys.stderr)
    bad = True

width = max((len(c) for c in cases), default=0)
for case in sorted(cases):
    runs = cases[case]
    missing = [r for r in RUNS if runs.get(r) is None]
    if missing:
        print(f"   MISSING {case}: no estimates.json mean point estimate for {' '.join(missing)}")
        bad = True
        continue
    a = (runs["a1"] + runs["a2"]) / 2.0
    b = (runs["b1"] + runs["b2"]) / 2.0
    delta = b / a - 1.0
    noise = max(abs(runs["a2"] / runs["a1"] - 1.0), abs(runs["b2"] / runs["b1"] - 1.0))
    if delta > 0.05 and delta > noise:
        verdict = "REGRESSION"
    elif delta > 0.05:
        verdict = "UNDECIDABLE (self-noise above 5% swallows it — rerun on a quieter machine)"
    else:
        verdict = "ok"
    print(f"   {case:<{width}} delta {delta:+.2%} self-noise {noise:+.2%} {verdict}")
    if verdict != "ok":
        bad = True

# one side benched a case the other never saw, and the counts must match
a_cases = {c for c, r in cases.items() if "a1" in r or "a2" in r}
b_cases = {c for c, r in cases.items() if "b1" in r or "b2" in r}
only_b = sorted(b_cases - a_cases)
only_a = sorted(a_cases - b_cases)
for case in only_b:
    print(f"   MISSING {case}: benched in B but never in A")
for case in only_a:
    print(f"   MISSING {case}: benched in A but never in B")
if only_a or only_b or len(a_cases) != len(b_cases):
    print(f"   case count differs: {len(a_cases)} in A, {len(b_cases)} in B")
    bad = True

sys.exit(1 if bad else 0)
PY

if [ "$fail" = 0 ]; then
  say "done $(date '+%H:%M:%S') — no regression past the >5% bar and this run's self-noise"
else
  say "done $(date '+%H:%M:%S') — FAILED (a failed run, missing estimates, a REGRESSION, or an UNDECIDABLE above)"
fi
exit "$fail"
