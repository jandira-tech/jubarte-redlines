#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Unit tests for convert_sweep path discovery and the Jaccard ratchet.

No jubarte binary, no scorer, no sibling checkouts required: the tests build
a fake T/ tree and assert the contract the real sweep will use.
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import convert_sweep as cs  # noqa: E402
import page1_delta as p1  # noqa: E402


def _touch(path: Path, text: str = "x") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


class Discover76Tests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.cases = self.root / "docxide-pdf" / "tests" / "fixtures" / "cases"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _case(self, name: str, *, docx: bool = True, pdf: bool = True) -> None:
        d = self.cases / name
        if docx:
            _touch(d / "input.docx")
        if pdf:
            _touch(d / "reference.pdf")

    def test_discovers_only_complete_pairs(self) -> None:
        self._case("case1")
        self._case("case13")
        self._case("case58", pdf=False)
        jobs = cs.discover_76(self.root / "docxide-pdf")
        stems = [j.stem for j in jobs]
        self.assertEqual(stems, ["case1", "case13"])
        self.assertEqual(jobs[0].docx.name, "input.docx")
        self.assertEqual(jobs[0].ref.name, "reference.pdf")

    def test_fast_skips_case13(self) -> None:
        self._case("case1")
        self._case("case13")
        jobs = cs.discover_76(self.root / "docxide-pdf", fast=True)
        self.assertEqual([j.stem for j in jobs], ["case1"])

    def test_missing_tree_is_empty_not_crash(self) -> None:
        jobs, reason = cs.discover_76_or_skip(self.root / "no-such")
        self.assertEqual(jobs, [])
        self.assertTrue(reason)


class Discover398Tests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.corpus = Path(self.tmp.name) / "corpus"
        (self.corpus / "docx_source").mkdir(parents=True)
        (self.corpus / "pdf_source").mkdir()
        (self.corpus / "docx_source_randomized").mkdir()
        (self.corpus / "pdf_source_randomized").mkdir()

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_reads_fixture_list_both_pools(self) -> None:
        _touch(self.corpus / "docx_source" / "alpha.docx")
        _touch(self.corpus / "pdf_source" / "alpha.pdf")
        _touch(self.corpus / "docx_source_randomized" / "file_1.docx")
        _touch(self.corpus / "pdf_source_randomized" / "file_1.pdf")
        listing = self.corpus / "docx_to_pdf_no_redline_fixtures.txt"
        listing.write_text(
            "# comment\n"
            "source\talpha\n"
            "source_randomized\tfile_1\n"
        )
        jobs = cs.discover_398(self.corpus)
        self.assertEqual(
            [j.stem for j in jobs],
            ["source__alpha", "source_randomized__file_1"],
        )
        self.assertTrue(jobs[0].docx.as_posix().endswith("docx_source/alpha.docx"))
        self.assertTrue(
            jobs[1].ref.as_posix().endswith("pdf_source_randomized/file_1.pdf")
        )

    def test_missing_corpus_is_empty_not_crash(self) -> None:
        jobs, reason = cs.discover_398_or_skip(self.corpus / "missing")
        self.assertEqual(jobs, [])
        self.assertTrue(reason)


class RatchetTests(unittest.TestCase):
    def test_tsv_roundtrip(self) -> None:
        rows = [
            cs.ScoreRow("case1", 44.7, 61.2, 100.0),
            cs.ScoreRow("case2", 4.0, 10.8, 100.0),
        ]
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "base.tsv"
            cs.write_tsv(rows, path)
            got = cs.read_tsv(path)
        self.assertEqual(got[0].stem, "case1")
        self.assertAlmostEqual(got[0].jaccard, 44.7)
        self.assertAlmostEqual(got[1].ssim, 10.8)

    def test_read_tsv_skips_hash_comments(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "base.tsv"
            path.write_text(
                "# provenance\n"
                "stem\tjaccard\tssim\ttext_boundary\n"
                "case1\t44.7\t61.2\t100.0\n"
            )
            got = cs.read_tsv(path)
        self.assertEqual(len(got), 1)
        self.assertEqual(got[0].stem, "case1")

    def test_row_drop_over_one_is_regression(self) -> None:
        # Pad with unchanged rows so a 0.8 single-row dip does not also
        # trip the mean-drop ratchet.
        pad = [cs.ScoreRow(f"p{i}", 50.0, 80.0, 100.0) for i in range(10)]
        base = [cs.ScoreRow("a", 50.0, 80.0, 100.0), *pad]
        ok_now = [cs.ScoreRow("a", 49.2, 80.0, 100.0), *pad]
        ok = cs.compare_to_baseline(ok_now, base)
        self.assertTrue(ok.ok, ok)
        bad_now = [cs.ScoreRow("a", 48.9, 80.0, 100.0), *pad]
        bad = cs.compare_to_baseline(bad_now, base)
        self.assertFalse(bad.ok)
        self.assertIn("a", bad.regressions)

    def test_mean_drop_over_point_two_is_regression(self) -> None:
        base = [
            cs.ScoreRow("a", 50.0, 0.0, 0.0),
            cs.ScoreRow("b", 50.0, 0.0, 0.0),
        ]
        now = [
            cs.ScoreRow("a", 49.7, 0.0, 0.0),
            cs.ScoreRow("b", 49.7, 0.0, 0.0),
        ]
        result = cs.compare_to_baseline(now, base)
        self.assertFalse(result.ok)
        self.assertLess(result.mean_delta, -0.2)

    def test_convert_failure_is_regression(self) -> None:
        result = cs.compare_to_baseline(
            [cs.ScoreRow("a", 50.0, 0.0, 0.0)],
            [cs.ScoreRow("a", 50.0, 0.0, 0.0)],
            failed=["a"],
        )
        self.assertFalse(result.ok)


class LiveTreeTests(unittest.TestCase):
    """Skipped in CI when the sibling checkouts are absent."""

    def test_real_76_is_seventy_six_complete_pairs(self) -> None:
        root = Path("/Users/arthrod/temp/T/docxide-pdf")
        if not (root / "tests" / "fixtures" / "cases").is_dir():
            self.skipTest("docxide-pdf sibling missing")
        jobs = cs.discover_76(root)
        self.assertEqual(len(jobs), 76)
        self.assertTrue(all(j.docx.is_file() and j.ref.is_file() for j in jobs))

    def test_real_398_is_three_hundred_ninety_eight(self) -> None:
        root = Path(
            "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/"
            "no_comments_pdf_was_generated_by_word"
        )
        if not (root / "docx_to_pdf_no_redline_fixtures.txt").is_file():
            self.skipTest("neurotic corpus sibling missing")
        jobs = cs.discover_398(root)
        self.assertEqual(len(jobs), 398)


class Page1DeltaTests(unittest.TestCase):
    def test_first_ink_row_is_first_nonwhite_band(self) -> None:
        mask = [False, False, True, True, False, True]
        self.assertEqual(p1.first_ink_row(mask), 2)

    def test_first_ink_row_none_when_blank(self) -> None:
        self.assertIsNone(p1.first_ink_row([False, False]))

    def test_band_pitch_median_of_gaps(self) -> None:
        # ink at 10, 20, 30, 50 → gaps 10, 10, 20 → median 10
        mask = [False] * 60
        for r in (10, 20, 30, 50):
            mask[r] = True
        self.assertEqual(p1.band_pitch(mask), 10.0)

    def test_delta_is_candidate_minus_reference(self) -> None:
        ref = [False] * 20
        cand = [False] * 20
        ref[5] = True
        cand[8] = True
        d = p1.ink_delta(ref, cand)
        self.assertEqual(d.first_ink, 3)

    def test_ppm_mask_p5_finds_ink_row(self) -> None:
        # 2x3 P5: white, black, white rows.
        ppm = (
            b"P5\n2 3\n255\n"
            + bytes([255, 255])
            + bytes([0, 0])
            + bytes([255, 255])
        )
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "t.ppm"
            path.write_bytes(ppm)
            mask = p1.ppm_mask(path)
        self.assertEqual(mask, [False, True, False])


if __name__ == "__main__":
    unittest.main()
