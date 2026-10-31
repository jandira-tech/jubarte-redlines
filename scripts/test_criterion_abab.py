#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/criterion_abab.sh: the release's criterion gate (#1797).

The previous release's tag (A) and the candidate (B) bench interleaved
A B A B into ONE CRITERION_HOME under the release's state dir — never
/tmp, never a saved baseline file — and the saved runs are compared
offline with --load-baseline (printed context only). The verdict reads
each run's mean point estimate from Criterion's saved estimates
(CRITERION_HOME/<group>/<bench>/<a1|a2|b1|b2>/estimates.json): per case
delta = mean(b1,b2)/mean(a1,a2) - 1, self-noise = max(|a2/a1-1|,
|b2/b1-1|). The issue's bar stays >5% per case: delta past both 5% and
this run's self-noise is a REGRESSION; delta past 5% but under a
self-noise above 5% is UNDECIDABLE (rerun on a quieter machine); the
gate fails on either.

cargo and git are stubs: no build, no benchmark, no network. The
estimates and the comparison text are criterion 0.8.2's real output,
lifted from jubarte-loop/release_0.14.0/gates (estimates.json verbatim,
the change: lines verbatim including the Unicode minus U+2212).
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE / "criterion_abab.sh"
RELEASE_SH = HERE / "release.sh"

MINUS = "−"  # U+2212, the sign criterion 0.8.2 prints for negative values

GIT_STUB = """#!/bin/sh
CWD="$PWD"
while [ $# -gt 0 ]; do case "$1" in -C) CWD="$2"; shift 2;; *) break;; esac; done
echo "git -C $CWD $*" >> "$GIT_CALLS"
case "$1" in
  rev-parse)
    case "$2" in
      HEAD) f="$CWD/.stub_head"; [ -f "$f" ] && cat "$f" || echo "${STUB_HEAD:-hhhhhhh}" ;;
      *) echo "${STUB_TAGC:-ttttttt}" ;;
    esac ;;
  status) [ -f "$CWD/.stub_dirty" ] && cat "$CWD/.stub_dirty" ;;
  worktree)
    case "$2" in
      add) mkdir -p "$3"; echo "${STUB_TAGC:-ttttttt}" > "$3/.stub_head" ;;
      remove) rm -rf "$4" ;;
      prune) : ;;
    esac ;;
esac
exit 0
"""

# --save-baseline RUN copies the test's estimates into CRITERION_HOME the
# way criterion does (<group>/<bench>/<run>/estimates.json). A
# --load-baseline NEW --baseline OLD call prints the saved text fixture —
# real criterion 0.8.2 output, context only — and CARGO_FAIL names runs
# that exit non-zero.
CARGO_STUB = """#!/bin/sh
echo "$PWD|$*|CRITERION_HOME=$CRITERION_HOME" >> "$CARGO_CALLS"
new="" old="" save="" take=""
for a in "$@"; do
  [ -n "$take" ] && { case "$take" in s) save="$a";; n) new="$a";; o) old="$a";; esac; take=""; continue; }
  case "$a" in
    --save-baseline) take=s ;;
    --load-baseline) take=n ;;
    --baseline) take=o ;;
  esac
done
run=${save:-$new}
[ -n "$run" ] && case " $CARGO_FAIL " in *" $run "*) echo "cargo: $run failed" >&2; exit 9 ;; esac
if [ -n "$save" ]; then
  for est in "$EST/$save"/*/*/estimates.json; do
    [ -f "$est" ] || continue
    rel=${est#"$EST/$save/"}
    mkdir -p "$CRITERION_HOME/${rel%/*}/$save"
    cp "$est" "$CRITERION_HOME/${rel%/*}/$save/estimates.json"
  done
  exit 0
fi
[ -n "$new" ] || exit 0
f="$CMP/$new-$old"
[ -f "$f" ] && cat "$f"
exit 0
"""

# criterion 0.8.2 output, verbatim from release_0.14.0/gates/criterion_home/
# compare_documents_fast/canonical_dense_edits/<run>/estimates.json (the
# four mean.point_estimate values below name a case whose text change lines
# and saved estimates disagree — only the estimates may drive the verdict).
REAL_ESTIMATES = {
    "a1": '{"mean":{"confidence_interval":{"confidence_level":0.95,"lower_bound":473258135.5,"upper_bound":477349749.8275},"point_estimate":475056321.0,"standard_error":1047841.5513605069},"median":{"confidence_interval":{"confidence_level":0.95,"lower_bound":472582458.5,"upper_bound":475005708.5},"point_estimate":473918333.5,"standard_error":576091.3787268426},"median_abs_dev":{"confidence_interval":{"confidence_level":0.95,"lower_bound":910131.0588419437,"upper_bound":4317887.09834218},"point_estimate":2637268.1069791317,"standard_error":923964.1300688013},"slope":null,"std_dev":{"confidence_interval":{"confidence_level":0.95,"lower_bound":1854556.7097093875,"upper_bound":6844367.318295437},"point_estimate":4776376.79302153,"standard_error":1346831.9677775102}}',
    "a2": '{"mean":{"confidence_interval":{"confidence_level":0.95,"lower_bound":483542234.7275,"upper_bound":498181960.54875004},"point_estimate":489991368.6,"standard_error":3759236.1505673034},"median":{"confidence_interval":{"confidence_level":0.95,"lower_bound":480623312.5,"upper_bound":493569687.5},"point_estimate":481498666.5,"standard_error":3703866.950404753},"median_abs_dev":{"confidence_interval":{"confidence_level":0.95,"lower_bound":1479727.4362295866,"upper_bound":18227887.645590305},"point_estimate":6348739.940187335,"standard_error":4651150.860426685},"slope":null,"std_dev":{"confidence_interval":{"confidence_level":0.95,"lower_bound":7985005.444423245,"upper_bound":24869863.109407507},"point_estimate":17229061.371756956,"standard_error":4881467.973659015}}',
    "b1": '{"mean":{"confidence_interval":{"confidence_level":0.95,"lower_bound":464885077.1,"upper_bound":482969406.3},"point_estimate":473202766.7,"standard_error":4629456.014310386},"median":{"confidence_interval":{"confidence_level":0.95,"lower_bound":458865146.0,"upper_bound":478029104.0},"point_estimate":465740396.0,"standard_error":5831918.684019244},"median_abs_dev":{"confidence_interval":{"confidence_level":0.95,"lower_bound":6506699.106582999,"upper_bound":28878484.07190442},"point_estimate":16699944.575616717,"standard_error":5625866.945510786},"slope":null,"std_dev":{"confidence_interval":{"confidence_level":0.95,"lower_bound":11284708.194761958,"upper_bound":27622759.43953534},"point_estimate":21056310.610975284,"standard_error":4218695.308618339}}',
    "b2": '{"mean":{"confidence_interval":{"confidence_level":0.95,"lower_bound":494014349.98125,"upper_bound":517170679.11249995},"point_estimate":503778639.6,"standard_error":6019844.656876356},"median":{"confidence_interval":{"confidence_level":0.95,"lower_bound":489209667.0,"upper_bound":503519603.5},"point_estimate":498969958.5,"standard_error":3918626.0029479233},"median_abs_dev":{"confidence_interval":{"confidence_level":0.95,"lower_bound":5813831.213083863,"upper_bound":24303982.639718056},"point_estimate":18218126.948663592,"standard_error":5043542.72340025},"slope":null,"std_dev":{"confidence_interval":{"confidence_level":0.95,"lower_bound":10425383.503821617,"upper_bound":42689779.75491338},"point_estimate":27806397.53488545,"standard_error":9996690.651519243}}',
}

# The four real cases' mean.point_estimate values (0.14.0's own A B A B
# run); canonical_dense_edits uses the verbatim files above, the rest the
# same numbers in minimal JSON.
REAL_CASES = {
    "compare_documents_fast/canonical_dense_edits": {
        "a1": 475056321.0, "a2": 489991368.6,
        "b1": 473202766.7, "b2": 503778639.6,
    },
    "compare_documents_slow/short_into_long": {
        "a1": 134649754.6728042, "a2": 136823059.7226389,
        "b1": 134754574.6565873, "b2": 140482259.30601192,
    },
    "compare_documents_slow/tables_bookmark_vmerge": {
        "a1": 101144423.91542327, "a2": 110954018.24518517,
        "b1": 108193373.52324073, "b2": 107356478.45493388,
    },
    "compare_documents_slow/comment_heavy": {
        "a1": 234158083.76490077, "a2": 235963757.81799603,
        "b1": 245759580.55043656, "b2": 240518059.8257738,
    },
}

# The saved-run text criterion 0.8.2 printed on each offline comparison,
# verbatim from release_0.14.0/gates/criterion_abab.log — including the
# Unicode minus U+2212 the old awk parser could not see. The gate prints
# these lines as context and must never read them.
CMP_TEXT = {
    "a2-a1": """\
compare_documents_fast/canonical_dense_edits
                        time:   [483.53 ms 489.99 ms 498.29 ms]
                        change: [+1.7203% +3.1438% +4.8533%] (p = 0.00 < 0.05)
                        Performance has regressed.
compare_documents_slow/short_into_long
                        time:   [135.78 ms 138.10 ms 140.73 ms]
                        change: [+0.2993% +1.6140% +3.1471%] (p = 0.03 < 0.05)
                        Change within noise threshold.
compare_documents_slow/tables_bookmark_vmerge
                        time:   [107.86 ms 109.31 ms 110.76 ms]
                        change: [+6.6139% +9.6986% +13.534%] (p = 0.00 < 0.05)
                        Performance has regressed.
compare_documents_slow/comment_heavy
                        time:   [234.75 ms 237.53 ms 240.52 ms]
                        change: [−1.0185% +0.7711% +2.5319%] (p = 0.43 > 0.05)
                        No change in performance detected.
""",
    "b2-b1": """\
compare_documents_fast/canonical_dense_edits
                        time:   [494.06 ms 503.78 ms 517.33 ms]
                        change: [+3.5244% +6.4615% +9.7877%] (p = 0.00 < 0.05)
                        Performance has regressed.
compare_documents_slow/short_into_long
                        time:   [138.04 ms 143.75 ms 148.50 ms]
                        change: [+1.1589% +4.2505% +7.4716%] (p = 0.02 < 0.05)
                        Performance has regressed.
compare_documents_slow/tables_bookmark_vmerge
                        time:   [105.61 ms 107.44 ms 109.59 ms]
                        change: [−3.2142% −0.7735% +1.7987%] (p = 0.58 > 0.05)
                        No change in performance detected.
compare_documents_slow/comment_heavy
                        time:   [232.43 ms 234.68 ms 238.29 ms]
                        change: [−5.4965% −2.1328% +1.4062%] (p = 0.28 > 0.05)
                        No change in performance detected.
""",
    "b1-a1": """\
compare_documents_fast/canonical_dense_edits
                        time:   [464.91 ms 473.20 ms 482.80 ms]
                        change: [−2.1777% −0.3902% +1.6356%] (p = 0.71 > 0.05)
                        No change in performance detected.
compare_documents_slow/short_into_long
                        time:   [134.27 ms 138.66 ms 141.04 ms]
                        change: [−2.1493% +0.0778% +2.3468%] (p = 0.95 > 0.05)
                        No change in performance detected.
compare_documents_slow/tables_bookmark_vmerge
                        time:   [105.86 ms 106.49 ms 107.62 ms]
                        change: [+5.2950% +6.9692% +9.3890%] (p = 0.00 < 0.05)
                        Performance has regressed.
compare_documents_slow/comment_heavy
                        time:   [239.51 ms 246.60 ms 254.50 ms]
                        change: [+2.2958% +4.9546% +7.9682%] (p = 0.00 < 0.05)
                        Performance has regressed.
""",
    "b2-a2": """\
compare_documents_fast/canonical_dense_edits
                        time:   [493.97 ms 503.78 ms 517.16 ms]
                        change: [+0.0271% +2.8138% +5.7671%] (p = 0.06 > 0.05)
                        No change in performance detected.
compare_documents_slow/short_into_long
                        time:   [138.02 ms 143.75 ms 148.49 ms]
                        change: [+0.2942% +2.6744% +5.3898%] (p = 0.06 > 0.05)
                        No change in performance detected.
compare_documents_slow/tables_bookmark_vmerge
                        time:   [105.60 ms 107.44 ms 109.57 ms]
                        change: [−6.6452% −3.2424% +0.0713%] (p = 0.10 > 0.05)
                        No change in performance detected.
compare_documents_slow/comment_heavy
                        time:   [232.44 ms 234.68 ms 238.28 ms]
                        change: [−0.7720% +1.9301% +4.9240%] (p = 0.27 > 0.05)
                        No change in performance detected.
""",
    "b1-a2": """\
compare_documents_fast/canonical_dense_edits
                        time:   [464.90 ms 473.20 ms 482.71 ms]
                        change: [−5.7079% −3.4263% −1.0849%] (p = 0.01 < 0.05)
                        Performance has improved.
compare_documents_slow/short_into_long
                        time:   [134.32 ms 138.66 ms 141.05 ms]
                        change: [−3.7077% −1.5118% +0.8492%] (p = 0.25 > 0.05)
                        No change in performance detected.
compare_documents_slow/tables_bookmark_vmerge
                        time:   [105.86 ms 106.49 ms 107.64 ms]
                        change: [−5.8930% −2.4881% +0.6968%] (p = 0.19 > 0.05)
                        No change in performance detected.
compare_documents_slow/comment_heavy
                        time:   [239.54 ms 246.60 ms 254.52 ms]
                        change: [+1.7542% +4.1514% +7.1191%] (p = 0.00 < 0.05)
                        Performance has regressed.
""",
    "b2-a1": """\
compare_documents_fast/canonical_dense_edits
                        time:   [493.94 ms 503.78 ms 517.39 ms]
                        change: [+3.7434% +6.0461% +8.7450%] (p = 0.00 < 0.05)
                        Performance has regressed.
compare_documents_slow/short_into_long
                        time:   [138.01 ms 143.75 ms 148.50 ms]
                        change: [+2.0878% +4.3316% +7.0770%] (p = 0.00 < 0.05)
                        Performance has regressed.
compare_documents_slow/tables_bookmark_vmerge
                        time:   [105.61 ms 107.44 ms 109.59 ms]
                        change: [+4.4013% +6.1418% +8.2001%] (p = 0.00 < 0.05)
                        Performance has regressed.
compare_documents_slow/comment_heavy
                        time:   [232.45 ms 234.68 ms 238.30 ms]
                        change: [−0.1703% +2.7161% +5.9077%] (p = 0.13 > 0.05)
                        No change in performance detected.
""",
}


def minimal_estimates(point: float) -> str:
    return json.dumps({"mean": {"point_estimate": point}, "slope": None})


class CriterionAbab(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="abab-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.bin = self.tmp / "bin"
        self.bin.mkdir()
        for name, body in (("git", GIT_STUB), ("cargo", CARGO_STUB)):
            p = self.bin / name
            p.write_text(body)
            p.chmod(0o755)
        self.est = self.tmp / "est"
        self.cmp = self.tmp / "cmp"
        self.cmp.mkdir()
        for name, text in CMP_TEXT.items():
            (self.cmp / name).write_text(text)
        self.cargo_calls = self.tmp / "cargo_calls"
        self.git_calls = self.tmp / "git_calls"
        self.state = self.tmp / "release_9.9.9" / "gates"
        self.cases: dict[str, dict[str, float]] = {}
        self.verbatim: set[str] = set()

    def use_real_cases(self) -> None:
        for case, runs in REAL_CASES.items():
            self.cases[case] = dict(runs)
        self.verbatim.add("compare_documents_fast/canonical_dense_edits")

    def use_case(self, case: str, **runs: float) -> None:
        self.cases[case] = {r: runs[r] for r in ("a1", "a2", "b1", "b2") if r in runs}

    def write_estimates(self) -> None:
        for case, runs in self.cases.items():
            for run, point in runs.items():
                d = self.est / run / case
                d.mkdir(parents=True, exist_ok=True)
                body = REAL_ESTIMATES[run] if case in self.verbatim else minimal_estimates(point)
                (d / "estimates.json").write_text(body)

    def run_gate(self, *extra: str, cargo_fail: str = "") -> subprocess.CompletedProcess[str]:
        self.write_estimates()
        env = dict(
            os.environ,
            PATH=f"{self.bin}:{os.environ['PATH']}",
            CARGO_CALLS=str(self.cargo_calls),
            GIT_CALLS=str(self.git_calls),
            EST=str(self.est),
            CMP=str(self.cmp),
            CARGO_FAIL=cargo_fail,
        )
        return subprocess.run(
            ["bash", str(SCRIPT), "v9.9.8", str(self.state), *extra],
            capture_output=True, text=True, timeout=120, env=env,
        )

    # --- the run shape ------------------------------------------------

    def test_abab_interleaves_into_one_criterion_home_under_the_state_dir(self) -> None:
        self.use_case("compare_documents_fast/canonical_dense_edits",
                      a1=100, a2=101, b1=102, b2=103)
        self.run_gate()
        calls = self.cargo_calls.read_text().splitlines()
        saves = [c.split("|")[1] for c in calls if "--save-baseline" in c]
        self.assertEqual(
            saves,
            ["bench --bench redline -- --save-baseline a1",
             "bench --bench redline -- --save-baseline b1",
             "bench --bench redline -- --save-baseline a2",
             "bench --bench redline -- --save-baseline b2"],
            calls,
        )
        for call in calls:
            self.assertIn(f"CRITERION_HOME={self.state}/criterion_home", call)
        self.assertNotIn("/tmp", next(iter(calls)))
        # six offline comparisons: both self-noise checks and all four B-vs-A
        cmps = [c.split("|")[1] for c in calls if "--load-baseline" in c]
        self.assertEqual(len(cmps), 6, calls)
        self.assertTrue(all("--load-baseline" in c and "--baseline" in c for c in cmps))
        # the ladder's --against wants the previous release's binary built
        self.assertTrue(any("build --release --bin jubarte" in c for c in calls), calls)
        self.assertTrue(any("/prev|" in c for c in calls), calls)

    def test_the_gate_never_puts_criterion_state_in_tmp(self) -> None:
        text = SCRIPT.read_text()
        code = "\n".join(
            line for line in text.splitlines() if not line.lstrip().startswith("#")
        )
        self.assertNotIn("/tmp", code)
        self.assertNotIn("mktemp", code)
        self.assertIn("CRITERION_HOME=$STATE/criterion_home", code)

    def test_release_sh_runs_these_tests(self) -> None:
        self.assertIn(
            '"${py312[@]}" scripts/test_criterion_abab.py',
            RELEASE_SH.read_text(),
        )

    # --- the verdict --------------------------------------------------

    def test_real_0140_estimates_print_the_table_and_pass(self) -> None:
        # 0.14.0's own run: every delta under 5%, so loud self-noise cases
        # (canonical_dense_edits +6.5%, tables_bookmark_vmerge +9.7%) stay ok.
        self.use_real_cases()
        r = self.run_gate()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        out = r.stdout
        self.assertRegex(out, r"canonical_dense_edits\s+delta \+1\.2\d% self-noise \+6\.4\d% ok")
        self.assertRegex(out, r"tables_bookmark_vmerge delta \+1\.6\d% self-noise \+9\.7\d% ok")
        self.assertIn("no regression", out)

    def test_the_change_text_is_context_never_the_verdict(self) -> None:
        # verbatim files: estimates say +1.24% (b1/a1 −0.4%, b2/a2 +2.8%),
        # while the text the stub prints claims +6.9% on b1 vs a1 — the old
        # parser's answer. The verdict must come from estimates.json.
        self.use_real_cases()
        r = self.run_gate()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        out = r.stdout + (self.state / "criterion_abab_cmp.log").read_text()
        # the literal 0.14.0 lines reach the report, Unicode minus included
        self.assertIn(f"change: [{MINUS}2.1777% {MINUS}0.3902% +1.6356%] (p = 0.71 > 0.05)", out)
        self.assertIn(f"change: [{MINUS}5.4965% {MINUS}2.1328% +1.4062%] (p = 0.28 > 0.05)", out)
        # and the +6.9% text claim never became a verdict
        self.assertNotIn("REGRESSION", r.stdout)

    def test_delta_past_five_percent_and_this_runs_noise_is_a_regression(self) -> None:
        self.use_case("compare_documents_fast/canonical_dense_edits",
                      a1=100, a2=101, b1=118, b2=120)
        r = self.run_gate()
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("REGRESSION", r.stdout)
        self.assertIn("canonical_dense_edits", r.stdout)

    def test_delta_past_five_percent_under_loud_self_noise_is_undecidable(self) -> None:
        # +12.7% release delta, but a1->a2 alone moved +20%: this machine
        # cannot prove the release regressed, and the gate still fails.
        self.use_case("compare_documents_fast/canonical_dense_edits",
                      a1=100, a2=120, b1=118, b2=130)
        r = self.run_gate()
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("UNDECIDABLE", r.stdout)
        self.assertIn("quieter machine", r.stdout)
        self.assertNotRegex(r.stdout, r"delta [^\n]*REGRESSION")

    # --- failures are failures -----------------------------------------

    def test_a_failed_cargo_run_fails_the_gate(self) -> None:
        self.use_case("compare_documents_fast/canonical_dense_edits",
                      a1=100, a2=101, b1=102, b2=103)
        r = self.run_gate(cargo_fail="b2")
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("FAILED", r.stdout)

    def test_a_failed_or_empty_comparison_fails_the_gate(self) -> None:
        self.use_case("compare_documents_fast/canonical_dense_edits",
                      a1=100, a2=101, b1=102, b2=103)
        (self.cmp / "b1-a2").unlink()
        r = self.run_gate()
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("b1 vs a2", r.stdout)

    def test_a_missing_estimates_file_fails_with_the_case_name(self) -> None:
        self.cases["compare_documents_fast/canonical_dense_edits"] = {
            "a1": 100, "a2": 101, "b1": 102}  # b2's estimates never saved
        r = self.run_gate()
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("canonical_dense_edits", r.stdout)
        self.assertIn("MISSING", r.stdout)

    def test_a_run_that_saved_nothing_fails_the_gate(self) -> None:
        r = self.run_gate()
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("FAILED", r.stdout)

    def test_a_case_only_in_b_fails_and_counts_must_match(self) -> None:
        self.use_case("compare_documents_fast/canonical_dense_edits",
                      a1=100, a2=101, b1=102, b2=103)
        self.cases["compare_documents_slow/brand_new_case"] = {"b1": 100, "b2": 101}
        r = self.run_gate()
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("brand_new_case", r.stdout)
        self.assertIn("count", r.stdout)

    # --- checkouts -----------------------------------------------------

    def test_a_dirty_or_wrong_prev_worktree_is_recreated(self) -> None:
        self.use_case("compare_documents_fast/canonical_dense_edits",
                      a1=100, a2=101, b1=102, b2=103)
        prev = self.state / "prev"
        prev.mkdir(parents=True)
        (prev / ".stub_head").write_text("wrongrev\n")
        (prev / ".stub_dirty").write_text(" M src/redline.rs\n")
        r = self.run_gate()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        calls = self.git_calls.read_text()
        self.assertIn("worktree remove --force", calls)
        self.assertIn(f"worktree add {self.state}/prev v9.9.8", calls)
        self.assertIn("discarded", r.stdout)

    def test_a_clean_prev_worktree_at_the_tag_is_reused(self) -> None:
        self.use_case("compare_documents_fast/canonical_dense_edits",
                      a1=100, a2=101, b1=102, b2=103)
        prev = self.state / "prev"
        prev.mkdir(parents=True)
        (prev / ".stub_head").write_text("ttttttt\n")  # == the tag's commit
        r = self.run_gate()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        calls = self.git_calls.read_text()
        self.assertNotIn("worktree add", calls)
        self.assertNotIn("worktree remove", calls)

    def test_a_candidate_revision_benches_its_own_clean_checkout(self) -> None:
        # B is the tag when resuming: worktree'd under the state dir, never
        # the caller's dirty checkout (#1797 resume case).
        self.use_case("compare_documents_fast/canonical_dense_edits",
                      a1=100, a2=101, b1=102, b2=103)
        r = self.run_gate("v9.9.9")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        calls = self.git_calls.read_text()
        self.assertIn(f"worktree add {self.state}/cand v9.9.9", calls)
        bench_dirs = {c.split("|")[0] for c in self.cargo_calls.read_text().splitlines()
                      if "--save-baseline" in c}
        self.assertEqual(bench_dirs, {f"{self.state}/prev", f"{self.state}/cand"}, bench_dirs)


if __name__ == "__main__":
    unittest.main()
