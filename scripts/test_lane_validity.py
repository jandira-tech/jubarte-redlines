#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/lane_validity.py: every validator error kind in the release's
redlines already exists in one of their sources (#1757, checklist before#24).

The validator is a stub that prints the OpenXmlValidator's TSV for the
kinds a file's name asks for, so each test controls the kinds exactly.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE / "lane_validity.py"
RELEASE_SH = HERE / "release.sh"

# stub validate-docx: `file<TAB>id<TAB>description<TAB>part` per kind the
# file's text names (one `ID@part` per line); exit 1 when it printed any
VALIDATOR = """#!/bin/sh
n=0
while IFS= read -r kind; do
  [ -n "$kind" ] || continue
  printf '%s\\t%s\\tdesc\\t%s\\n' "$1" "${kind%@*}" "${kind#*@}"; n=1
done < "$1"
exit $n
"""
HEADER = ("key,base,base_sha256,next,next_sha256,docx,docx_sha256,pdf,pdf_sha256,state,id,sets,oracle,"
          "oracle_pdf,oracle_pdf_sha256,docxodus_pdf,docxodus_pdf_sha256,jubarte_docx,jubarte_docx_sha256,"
          "jubarte_pdf,jubarte_pdf_sha256")


class LaneValidity(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="lane-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.root = self.tmp / "bench"
        self.redlines = self.tmp / "sweep"
        self.redlines.mkdir()
        self.validator = self.tmp / "validate-docx"
        self.validator.write_text(VALIDATOR)
        self.validator.chmod(0o755)
        self.rows: list[str] = []

    def pair(self, key: str, base: str = "", nxt: str = "", redline: str | None = "") -> None:
        for name, kinds in ((f"corpus/{key}-base.docx", base), (f"corpus/{key}-next.docx", nxt)):
            (self.root / name).parent.mkdir(parents=True, exist_ok=True)
            (self.root / name).write_text(kinds)
        if redline is not None:
            (self.redlines / f"{key}.docx").write_text(redline)
        self.rows.append(",".join([key, f"corpus/{key}-base.docx", "0" * 64, f"corpus/{key}-next.docx", "0" * 64]
                                  + [""] * 16))

    def check(self, *extra: str) -> subprocess.CompletedProcess[str]:
        sample = self.tmp / "sample_redline_9.9.9_x.csv"
        sample.write_bytes(("\r\n".join([HEADER, *self.rows]) + "\r\n").encode())
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(sample), "--root", str(self.root), "--redlines", str(self.redlines),
             "--validator", str(self.validator), *extra],
            capture_output=True, text=True, timeout=60,
        )

    def test_kinds_the_sources_already_have_pass(self) -> None:
        self.pair("k1", base="Sch_Bad@/word/document.xml\n", redline="Sch_Bad@/word/document.xml\n")
        self.pair("k2", nxt="Sem_X@/word/styles.xml\n", redline="Sem_X@/word/styles.xml\n")
        self.pair("k3")
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("3 redlines; 2 with validator errors; 0 with a kind neither source has", r.stdout)

    def test_a_kind_of_the_redlines_own_is_named(self) -> None:
        self.pair("k1", base="Sch_Bad@/word/document.xml\n",
                  redline="Sch_Bad@/word/document.xml\nSch_New@/word/comments.xml\n")
        self.pair("k2")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("OWN k1", r.stdout)
        self.assertIn("Sch_New", r.stdout)
        self.assertNotIn("OWN k2", r.stdout)

    def test_the_same_id_in_another_part_is_its_own_kind(self) -> None:
        self.pair("k1", base="Sch_Bad@/word/document.xml\n", redline="Sch_Bad@/word/footer1.xml\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("/word/footer1.xml", r.stdout)

    def test_a_missing_redline_fails(self) -> None:
        # a sweep pointed at the wrong folder validates nothing
        self.pair("k1", redline=None)
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("ABSENT", r.stdout)

    def test_a_missing_source_is_refused(self) -> None:
        self.pair("k1")
        (self.root / "corpus/k1-next.docx").unlink()
        r = self.check()
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("corpus/k1-next.docx", r.stderr)

    def test_a_missing_validator_is_refused(self) -> None:
        self.pair("k1")
        self.validator.unlink()
        r = self.check()
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("validate-docx", r.stderr)

    def test_a_csv_of_another_layout_is_refused(self) -> None:
        other = self.tmp / "other.csv"
        other.write_text("pair_stem,base\np1,x\n")
        r = subprocess.run([sys.executable, str(SCRIPT), str(other), "--root", str(self.root), "--redlines",
                            str(self.redlines), "--validator", str(self.validator)],
                           capture_output=True, text=True, timeout=60)
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("other.csv", r.stderr)

    def test_release_sh_runs_these_tests_and_names_the_tool(self) -> None:
        text = RELEASE_SH.read_text()
        self.assertTrue('"${py312[@]}" scripts/test_lane_validity.py' in text, "step 5 runs these tests")
        self.assertTrue("python3 scripts/lane_validity.py" in text, "before#24 names the tool")


if __name__ == "__main__":
    unittest.main()
