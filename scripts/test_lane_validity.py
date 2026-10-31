#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/lane_validity.py: every validator error signature in the release's
redlines already exists in one of their sources or in Word's own redline
(#1757, #1798; checklist before#24).

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

# stub validate-docx: `file<TAB>id<TAB>description<TAB>part<TAB>xpath` per
# kind the file's text names; `ID@part`, `ID@part|description` or
# `ID@part|description|xpath` per line; a `FAIL` line is a file it cannot
# open (OPEN_FAILED row); exit 1 when it printed any
VALIDATOR = """#!/bin/sh
n=0
while IFS= read -r kind; do
  [ -n "$kind" ] || continue
  if [ "${kind#FAIL}" != "$kind" ]; then
    printf '%s\tOPEN_FAILED\tSpecified part does not exist in the package\t\t\n' "$1"; n=1; continue
  fi
  rest="${kind#*@}"; part="${rest%%|*}"; tail_="${rest#*|}"
  desc="${tail_%|*}"; [ "$tail_" = "$rest" ] && desc=desc
  xp=""; case "$tail_" in *\\|*) xp="${tail_##*|}";; esac
  printf '%s\t%s\t%s\t%s\t%s\n' "$1" "${kind%%@*}" "$desc" "$part" "$xp"; n=1
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

    def pair(self, key: str, base: str = "", nxt: str = "", redline: str | None = "",
             word: str | None = None) -> None:
        for name, kinds in ((f"corpus/{key}-base.docx", base), (f"corpus/{key}-next.docx", nxt)):
            (self.root / name).parent.mkdir(parents=True, exist_ok=True)
            (self.root / name).write_text(kinds)
        docx = ""
        if word is not None:
            docx = f"corpus/{key}-word.docx"
            (self.root / docx).write_text(word)
        if redline is not None:
            (self.redlines / f"{key}.docx").write_text(redline)
        self.rows.append(",".join([key, f"corpus/{key}-base.docx", "0" * 64, f"corpus/{key}-next.docx", "0" * 64,
                                  docx] + [""] * 15))

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
        self.assertIn("3 redlines; 2 with validator errors; 0 with a signature neither source nor "
                      "Word's redline has", r.stdout)

    def test_a_kind_of_the_redlines_own_is_named(self) -> None:
        self.pair("k1", base="Sch_Bad@/word/document.xml\n",
                  redline="Sch_Bad@/word/document.xml\nSch_New@/word/comments.xml\n")
        self.pair("k2")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("OWN k1", r.stdout)
        self.assertIn("Sch_New", r.stdout)
        self.assertNotIn("OWN k2", r.stdout)

    def test_the_same_id_in_another_part_family_is_its_own_signature(self) -> None:
        self.pair("k1", base="Sch_Bad@/word/document.xml\n", redline="Sch_Bad@/word/footer1.xml\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("/word/footer.xml", r.stdout)

    def test_a_renamed_part_matches_its_family(self) -> None:
        # redlineB_header3.xml is the header family header3.xml belongs to
        self.pair("k1", base="Sch_Attr@/word/header3.xml\nSch_Attr@/word/footer3.xml\n",
                  redline="Sch_Attr@/word/redlineB_header3.xml\nSch_Attr@/word/redlineB_footer3.xml\n")
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("0 with a signature neither source nor", r.stdout)

    def test_a_signature_words_redline_shares_is_word_not_own(self) -> None:
        # w:ins under m:r in the redline folds to the w:del Word writes
        self.pair("k1", redline="Sch_InvalidElementContentExpectingComplex@/word/document.xml"
                  "|expected <w:ins> before <m:rPr>|/w:document/w:body/m:oMathPara/m:oMath/m:r/w:ins\n",
                  word="Sch_InvalidElementContentExpectingComplex@/word/document.xml"
                  "|expected <w:del> before <m:rPr>|/w:document/w:body/m:oMathPara/m:oMath/m:r/w:del\n")
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("WORD k1", r.stdout)
        self.assertNotIn("OWN k1", r.stdout)

    def test_a_signature_of_its_own_stays_own_beside_word(self) -> None:
        self.pair("k1", redline="Sch_InvalidElementContentExpectingComplex@/word/document.xml"
                  "|expected <w:ins> before <m:rPr>|/w:document/w:body/m:oMathPara/m:oMath/m:r/w:ins\n"
                  "Sch_New@/word/comments.xml\n",
                  word="Sch_InvalidElementContentExpectingComplex@/word/document.xml"
                  "|expected <w:del> before <m:rPr>|/w:document/w:body/m:oMathPara/m:oMath/m:r/w:del\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("WORD k1", r.stdout)
        self.assertIn("OWN k1", r.stdout)
        self.assertIn("Sch_New", r.stdout)

    def test_an_unreadable_source_is_named_and_not_own(self) -> None:
        self.pair("k1", base="FAIL\n", redline="Sch_Link@/word/styles.xml\n")
        self.pair("k2", redline="Sch_New@/word/document.xml\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("UNREADABLE SOURCE k1 corpus/k1-base.docx", r.stdout)
        self.assertNotIn("OWN k1", r.stdout)
        self.assertIn("OWN k2", r.stdout)

    def test_a_named_word_redline_missing_is_refused(self) -> None:
        self.pair("k1", word="")
        (self.root / "corpus/k1-word.docx").unlink()
        r = self.check()
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("corpus/k1-word.docx", r.stderr)

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
