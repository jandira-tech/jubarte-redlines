#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Pull-request path filter of .github/workflows/docs.yml.

The workflow runs on a pull request only when it touches a path in
`on.pull_request.paths`. A test target a job runs (`--test NAME`) whose
source `tests/NAME.rs` is missing from that list can break without the
workflow ever running on the PR that broke it.

Run with PyYAML: `uv run --with pyyaml python3 scripts/test_docs_workflow.py`.
"""

from __future__ import annotations

import fnmatch
import re
import unittest
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
WORKFLOW = HERE.parent / ".github" / "workflows" / "docs.yml"
TEST_TARGET = re.compile(r"--test\s+([A-Za-z0-9_]+)")


def load() -> dict:
    return yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))


def pr_paths(workflow: dict) -> list[str]:
    on = workflow.get("on", workflow.get(True))  # PyYAML reads a bare `on:` as True
    return on["pull_request"]["paths"]


def test_targets(workflow: dict) -> set[str]:
    scripts = (s.get("run", "") for job in workflow["jobs"].values() for s in job["steps"])
    return {name for run in scripts for name in TEST_TARGET.findall(run)}


class PathFilter(unittest.TestCase):
    def test_every_test_target_the_workflow_runs_triggers_it(self) -> None:
        workflow = load()
        paths = pr_paths(workflow)
        targets = test_targets(workflow)
        self.assertTrue(targets, "docs.yml runs no --test target; this check is stale")
        uncovered = sorted(
            f"tests/{name}.rs"
            for name in targets
            if not any(fnmatch.fnmatch(f"tests/{name}.rs", p) for p in paths)
        )
        self.assertEqual(uncovered, [], "add these to on.pull_request.paths")


if __name__ == "__main__":
    unittest.main()
