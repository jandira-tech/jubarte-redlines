#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/coverage_gate.py, and the CI step that runs it.

The gate measured only the root package after the 0.19.0 split:
`cargo llvm-cov report` takes no `--workspace`, so its report held the four
files of `src/bin/` (91.6% lines) while Codecov saw the whole workspace.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path

import coverage_gate

ROOT = Path(__file__).resolve().parent.parent
CI = ROOT / ".github" / "workflows" / "ci.yml"


def metadata(*names: str) -> dict:
    packages = []
    for name in names:
        where = "/w" if name == "jubarte" else f"/w/crates/{name}"
        packages.append({"name": name, "id": f"id-{name}", "manifest_path": f"{where}/Cargo.toml"})
    return {"packages": packages, "workspace_members": [p["id"] for p in packages]}


def report(files: list[str], lines=(96, 100), branches=(90, 100)) -> dict:
    def summary(covered, count):
        return {"covered": covered, "count": count, "percent": 100 * covered / count}

    return {
        "data": [
            {
                "files": [{"filename": f} for f in files],
                "totals": {"lines": summary(*lines), "branches": summary(*branches)},
            }
        ]
    }


class Gate(unittest.TestCase):
    META = metadata("jubarte", "jubarte-dom", "jubarte-redlines")
    ALL = ["/w/src/lib.rs", "/w/crates/jubarte-dom/src/lib.rs", "/w/crates/jubarte-redlines/src/lib.rs"]

    def test_a_report_over_every_member_above_the_floors_passes(self):
        self.assertEqual(coverage_gate.check(report(self.ALL), self.META), [])

    def test_a_root_only_report_is_refused_whatever_its_totals(self):
        # What the gate saw from 0.19.0 on: src/bin/ only.
        failures = coverage_gate.check(report(["/w/src/bin/jubarte.rs"]), self.META)
        self.assertEqual(failures, ["the coverage report holds no file from: jubarte-dom, jubarte-redlines"])

    def test_root_files_do_not_stand_in_for_a_crate(self):
        # /w/crates/... sits under the root package's directory too; the
        # deepest package directory owns the file.
        files = ["/w/src/lib.rs", "/w/crates/jubarte-dom/src/lib.rs"]
        self.assertEqual(
            coverage_gate.check(report(files), self.META),
            ["the coverage report holds no file from: jubarte-redlines"],
        )

    def test_each_floor_fails_on_its_own(self):
        self.assertEqual(
            coverage_gate.check(report(self.ALL, lines=(94, 100)), self.META),
            ["lines coverage is below 95%"],
        )
        self.assertEqual(
            coverage_gate.check(report(self.ALL, branches=(87, 100)), self.META),
            ["branches coverage is below 88%"],
        )

    def test_an_empty_metric_fails(self):
        self.assertEqual(
            coverage_gate.check(report(self.ALL, branches=(0, 0.0001)), self.META),
            ["branches coverage is below 88%"],
        )

    def test_dependencies_outside_the_workspace_are_not_required(self):
        meta = metadata("jubarte")
        meta["packages"].append({"name": "zip", "id": "id-zip", "manifest_path": "/reg/zip/Cargo.toml"})
        self.assertEqual(coverage_gate.check(report(["/w/src/lib.rs"]), meta), [])


class Workflow(unittest.TestCase):
    def step(self) -> str:
        text = CI.read_text(encoding="utf-8")
        start = text.index("cargo llvm-cov report")
        return text[text.rindex("run: |", 0, start) : text.index("\n\n", start)]

    def test_the_report_names_every_workspace_member(self):
        step = self.step()
        self.assertIn("cargo metadata --no-deps --format-version 1", step)
        self.assertRegex(step, r'cargo llvm-cov report --branch --json "\$\{packages\[@\]\}"')

    def test_ci_runs_the_gate_script_not_an_inline_copy(self):
        step = self.step()
        self.assertIn("python3 scripts/coverage_gate.py coverage.json", step)
        self.assertNotRegex(step, re.compile(r"<<'PY'"))

    def test_ci_runs_this_test(self):
        self.assertIn("python3 scripts/test_coverage_gate.py", CI.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
