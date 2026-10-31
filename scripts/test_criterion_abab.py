#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/criterion_abab.sh: the release's criterion gate (#1797).

The previous release's tag (A) and the candidate (B) bench interleaved
A B A B into ONE CRITERION_HOME under the release's state dir — never
/tmp, never a saved baseline file — and the saved runs are compared
offline with --load-baseline. A case is a regression only when its mean
B-vs-A change clears the noise floor, which the script names with its
provenance: +10%, above the +9.7% self-noise 0.14.0 measured (#1797).

cargo and git are stubs: no build, no benchmark, no network.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE / "criterion_abab.sh"
RELEASE_SH = HERE / "release.sh"

GIT_STUB = """#!/bin/sh
while [ $# -gt 0 ]; do
  case "$1" in
    -C) shift 2 ;;
    rev-parse) echo "${STUB_REV:-aaaaaaa}"; exit 0 ;;
    worktree) mkdir -p "$3"; exit 0 ;;
    *) shift ;;
  esac
done
exit 0
"""

CARGO_STUB = """#!/bin/sh
echo "$PWD|$*|CRITERION_HOME=$CRITERION_HOME" >> "$CARGO_CALLS"
new=""
old=""
take=""
for a in "$@"; do
  if [ "$take" = new ]; then take=""; new="$a"; continue; fi
  if [ "$take" = old ]; then take=""; old="$a"; continue; fi
  case "$a" in
    --load-baseline) take=new ;;
    --baseline) take=old ;;
  esac
done
[ -n "$new" ] || exit 0
f="$PCTS/${new}_${old}"
[ -f "$f" ] || exit 0
echo "tables_bookmark_vmerge/generate"
echo "  change: [$(cat "$f")]"
exit 0
"""


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
        self.pcts = self.tmp / "pcts"
        self.pcts.mkdir()
        self.cargo_calls = self.tmp / "cargo_calls"
        self.state = self.tmp / "release_9.9.9" / "gates"

    def run_gate(self, pcts: dict[str, str]) -> subprocess.CompletedProcess[str]:
        for name, pct in pcts.items():
            (self.pcts / name).write_text(pct + "\n")
        env = dict(
            os.environ,
            PATH=f"{self.bin}:{os.environ['PATH']}",
            CARGO_CALLS=str(self.cargo_calls),
            PCTS=str(self.pcts),
        )
        return subprocess.run(
            ["bash", str(SCRIPT), "v9.9.8", str(self.state)],
            capture_output=True, text=True, timeout=120, env=env,
        )

    def test_abab_interleaves_into_one_criterion_home_under_the_state_dir(self) -> None:
        self.run_gate({"a2_a1": "+0.0% +0.0% +0.0%", "b1_a1": "+0.0% +0.0% +0.0%"})
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

    def test_a_mean_b_vs_a_past_the_noise_floor_fails(self) -> None:
        # +9.7% mean is inside noise (0.14.0 measured that much against itself);
        # +11% clears the +10% floor the row names
        below = self.run_gate({f"b{x}_a{y}": "+8.0% +9.7% +11.0%" for x in "12" for y in "12"})
        self.assertEqual(below.returncode, 0, below.stdout + below.stderr)
        self.assertIn("10%", below.stdout + below.stderr)
        self.assertIn("9.7", below.stdout + below.stderr)
        over = self.run_gate({f"b{x}_a{y}": "+10.0% +12.0% +14.0%" for x in "12" for y in "12"})
        self.assertEqual(over.returncode, 1, over.stdout + over.stderr)
        self.assertIn("REGRESSION", over.stdout + over.stderr)
        self.assertIn("tables_bookmark_vmerge", over.stdout + over.stderr)

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


if __name__ == "__main__":
    unittest.main()
