#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Unit tests for redline-sweep.sh's validation baseline key extraction.

Execute the production filter alone: no Rust binary, Word or .NET required.
"""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class BaselineKeys(unittest.TestCase):
    def keys(self, contents):
        # Exercise the actual pipeline, so removing the comment filter causes
        # a regression without running the expensive document generation stage.
        lines = (HERE / "redline-sweep.sh").read_text().splitlines()
        commands = [line.strip() for line in lines
                    if line.lstrip().startswith("awk ") and '"$OUT/.baseline_keys"' in line]
        self.assertEqual(len(commands), 1, "locate the production baseline filter")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            baseline = root / "baseline with spaces.tsv"
            baseline.write_text(contents, encoding="utf-8")
            result = subprocess.run(
                ["bash", "-o", "pipefail", "-c", commands[0]],
                env=dict(os.environ, BASELINE=str(baseline), OUT=str(root), LC_ALL="C"),
                capture_output=True, text=True, timeout=5,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            return (root / ".baseline_keys").read_text().splitlines()

    def test_tabbed_comments_do_not_become_fixed_findings(self):
        self.assertEqual(self.keys(
            "# pair_stem\terror_id\tdescription\n"
            "#retired\tOldError\tno longer a baseline finding\n"
            "active\tKnownError\tdescription\n"
            "# trailing comment\tmetadata\n"
        ), ["active\tKnownError"])

    def test_real_keys_remain_sorted_unique_and_keep_embedded_hashes(self):
        self.assertEqual(self.keys(
            "zeta\tErrorB\told description\n"
            "#ignored\tCommentError\n"
            "alpha#revision\tErrorA\tdescription\n"
            "zeta\tErrorB\tnew description\n"
            "alpha#revision\tErrorC\n"
            "\n"
            "single-field\n"
        ), ["alpha#revision\tErrorA", "alpha#revision\tErrorC", "zeta\tErrorB"])

    def test_empty_or_comment_only_baseline_has_no_keys(self):
        for contents in ("", "# provenance\n# pair\terror\n\n"):
            with self.subTest(contents=contents):
                self.assertEqual(self.keys(contents), [])


if __name__ == "__main__":
    unittest.main()
