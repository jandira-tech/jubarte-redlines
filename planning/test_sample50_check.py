#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Unit tests for sample50_check: sample parsing, preflight, and the bless gate.

No jubarte binary, scorer, or sibling checkout is needed.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import sample50_check as s50  # noqa: E402

ROW = "corpus\t{id}\t{docx}\t{ref}\t8.1\tdecile1\n"


class LoadRowsTests(unittest.TestCase):
    def test_relative_paths_resolve_against_planning_dir(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            sample = Path(tmp) / "s.tsv"
            sample.write_text(
                "# comment\n" + ROW.format(id="source__a", docx="../x/a.docx", ref="../x/a.pdf")
            )
            rows = s50.load_rows(str(sample))
        self.assertEqual(rows[0]["id"], "source__a")
        self.assertEqual(rows[0]["docx"], os.path.normpath(os.path.join(s50.HERE, "../x/a.docx")))

    def test_checked_in_sample_has_no_machine_specific_paths(self) -> None:
        for row in s50.load_rows(os.path.join(s50.HERE, "sample50.tsv")):
            self.assertTrue(row["docx"].startswith(s50.T + os.sep), row["docx"])
        text = (HERE / "sample50.tsv").read_text(encoding="utf-8")
        self.assertNotIn("/Users/", text)

    def test_ids_that_escape_the_work_dir_are_rejected(self) -> None:
        for bad in ("../evil", "/tmp/evil", "a/b", ""):
            with tempfile.TemporaryDirectory() as tmp:
                sample = Path(tmp) / "s.tsv"
                sample.write_text(ROW.format(id=bad, docx="a.docx", ref="a.pdf"))
                with self.assertRaises(ValueError, msg=bad):
                    s50.load_rows(str(sample))


class MainGateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.jubarte = self.root / "jubarte"
        self.scorer = self.root / "scorer"
        self.baseline = self.root / "base.json"
        self.sample = self.root / "s.tsv"
        docx, ref = self.root / "a.docx", self.root / "a.pdf"
        docx.write_text("x")
        ref.write_text("x")
        self.sample.write_text(ROW.format(id="source__a", docx=docx, ref=ref))

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _argv(self, *extra: str) -> list[str]:
        return [
            "--jubarte", str(self.jubarte), "--scorer", str(self.scorer),
            "--sample", str(self.sample), "--baseline", str(self.baseline), *extra,
        ]

    def test_missing_binary_exits_two_with_path(self) -> None:
        self.scorer.write_text("x")
        with mock.patch("sys.stderr") as err:
            self.assertEqual(s50.main(self._argv()), 2)
        self.assertIn(str(self.jubarte), "".join(c.args[0] for c in err.write.call_args_list))

    def test_failed_conversion_is_never_blessed(self) -> None:
        self.jubarte.write_text("x")
        self.scorer.write_text("x")
        scored = ({"source__a": {"jaccard": 0.0, "ssim": 0.0, "text_boundary": 0.0}}, ["source__a"])
        with mock.patch.object(s50, "convert_and_score", return_value=scored):
            self.assertEqual(s50.main(self._argv("--bless")), 1)
            self.assertEqual(s50.main(self._argv()), 1)
        self.assertFalse(self.baseline.exists(), "a failed run must not write a baseline")

    def test_clean_run_blesses_then_compares(self) -> None:
        self.jubarte.write_text("x")
        self.scorer.write_text("x")
        scored = ({"source__a": {"jaccard": 40.0, "ssim": 0.0, "text_boundary": 0.0}}, [])
        with mock.patch.object(s50, "convert_and_score", return_value=scored):
            self.assertEqual(s50.main(self._argv("--bless")), 0)
            self.assertEqual(json.loads(self.baseline.read_text())["mean"], 40.0)
            self.assertEqual(s50.main(self._argv()), 0)


if __name__ == "__main__":
    unittest.main()
