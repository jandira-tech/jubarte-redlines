#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Where the workflows run, and how they fall back when the self-hosted
runners are down.

Linux jobs run on the self-hosted runners, through one switch: the
LINUX_RUNNER repository variable, unset (self-hosted) by default, set to
`ubuntu-latest` while the self-hosted runners are offline. mbx's cache
server sits on the self-hosted network, so every mbx step is gated to
those runners. macOS and Windows legs are GitHub-hosted; no job names a
Blacksmith runner any more.
"""

import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
WORKFLOWS = sorted((ROOT / ".github" / "workflows").glob("*.yml"))
SWITCH = "${{ vars.LINUX_RUNNER || 'self-hosted' }}"
GATES = ("(vars.LINUX_RUNNER || 'self-hosted') == 'self-hosted'", "matrix.os == 'self-hosted'")


def jobs():
    for path in WORKFLOWS:
        for name, job in (yaml.safe_load(path.read_text()).get("jobs") or {}).items():
            yield f"{path.name}:{name}", job


def runners(job) -> list[str]:
    runs_on = job.get("runs-on", "")
    if runs_on == "${{ matrix.os }}":
        matrix = job["strategy"]["matrix"]
        listed = matrix.get("os", [])
        included = [row["os"] for row in matrix.get("include", []) if "os" in row]
        return [str(os) for os in [*listed, *included]]
    return [str(runs_on)]


class Runners(unittest.TestCase):
    def test_no_job_names_a_blacksmith_runner(self) -> None:
        for where, job in jobs():
            with self.subTest(job=where):
                self.assertFalse(any("blacksmith" in r for r in runners(job)), runners(job))

    def test_self_hosted_jobs_go_through_the_switch(self) -> None:
        for where, job in jobs():
            with self.subTest(job=where):
                self.assertNotIn("self-hosted", runners(job), "use the LINUX_RUNNER switch")

    def test_the_test_matrix_keeps_a_switched_linux_leg(self) -> None:
        ci = yaml.safe_load((ROOT / ".github/workflows/ci.yml").read_text())
        self.assertEqual(
            ci["jobs"]["test"]["strategy"]["matrix"]["os"],
            [SWITCH, "macos-latest", "windows-latest"],
        )

    def test_every_mbx_step_is_gated_to_the_self_hosted_runners(self) -> None:
        for where, job in jobs():
            for step in job.get("steps", []):
                if step.get("uses") != "./.github/actions/mbx":
                    continue
                with self.subTest(job=where):
                    self.assertTrue(any(g in str(step.get("if", "")) for g in GATES), step)

    def test_checkouts_use_actions_checkout(self) -> None:
        for path in WORKFLOWS:
            with self.subTest(workflow=path.name):
                self.assertNotIn("useblacksmith/", path.read_text())


if __name__ == "__main__":
    unittest.main()
