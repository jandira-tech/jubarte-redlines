#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""tools/parity_ladder.py --against BIN (#1797).

A release is measured against the PREVIOUS RELEASE, measured now: sweep
runs the candidate and that release's binary over the same pairs and
reports only the findings the candidate has that the previous release does
not. tools/parity_baseline.tsv is a stale blessed file (last blessed
2026-09-28, before v0.10.0), so it cannot speak for the previous release.

The jubarte CLI is stubbed: a --bin writes a docx whose document.xml each
test names, so the ladder's signature findings are under test control.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("parity_ladder", HERE / "parity_ladder.py")
pl = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(pl)

W_NS = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
PLAIN = (
    f'<w:document xmlns:w="{W_NS}"><w:body>'
    f"<w:p><w:r><w:t>same</w:t></w:r></w:p>"
    f"</w:body></w:document>"
)
# an rsid attribute trips the S-rsid-leftover signature
RSID = PLAIN.replace("<w:p>", '<w:p w:rsidR="00AA">')


def docx(path: Path, xml: str) -> None:
    with zipfile.ZipFile(path, "w") as z:
        z.writestr("word/document.xml", xml)


class Against(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="ladder-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.corpus = self.tmp / "corpus"
        src = self.corpus / "docx_source"
        src.mkdir(parents=True)
        docx(src / "base.docx", PLAIN)
        docx(src / "next.docx", PLAIN)
        (self.corpus / "centralized_mapping.csv").write_text(
            "pair_stem,docx_source_base,docx_source_next,redline_docx_word\n"
            "p1,base.docx,next.docx,\n"
        )
        self.out_dir = self.tmp / "out"
        self.baseline = self.tmp / "parity_baseline.tsv"
        self.addCleanup(setattr, pl, "OUT_DIR", pl.OUT_DIR)
        self.addCleanup(setattr, pl, "BASELINE", pl.BASELINE)
        pl.OUT_DIR = str(self.out_dir)
        pl.BASELINE = str(self.baseline)
        self.good = self.bin_stub("good", PLAIN)
        self.bad = self.bin_stub("bad", RSID)

    def bin_stub(self, name: str, xml: str) -> str:
        """A fake jubarte CLI: `a b -o out --force` writes a docx holding xml."""
        stub = self.tmp / name
        stub.write_text(
            f"#!{sys.executable}\n"
            "import sys, zipfile\n"
            "out = sys.argv[sys.argv.index('-o') + 1]\n"
            f"zipfile.ZipFile(out, 'w').writestr('word/document.xml', {xml!r})\n"
        )
        stub.chmod(0o755)
        return str(stub)

    def sweep(self, candidate: str, against: str = "") -> tuple[int, str]:
        args = SimpleNamespace(
            corpus=str(self.corpus), bin=candidate, limit=0, only="", against=against
        )
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = pl.cmd_sweep(args)
        return rc, buf.getvalue()

    def test_against_reports_only_findings_the_previous_release_lacks(self) -> None:
        rc, out = self.sweep(self.bad, against=self.good)
        self.assertEqual(rc, 1, out)
        self.assertIn("NEW", out)
        self.assertIn("rsid", out)
        # the same defect on the previous release's binary is not a regression
        rc, out = self.sweep(self.bad, against=self.bad)
        self.assertEqual(rc, 0, out)
        # a clean candidate against a dirty previous release reports fixed rows
        rc, out = self.sweep(self.good, against=self.bad)
        self.assertEqual(rc, 0, out)
        self.assertIn("rsid", out)  # named as fixed, not NEW
        self.assertNotIn("NEW ", out)

    def test_against_measures_now_never_the_stale_tsv(self) -> None:
        # the blessed TSV claims the finding known; the previous release's
        # binary, measured now, does not have it
        self.baseline.write_text("p1\tS-rsid-leftover\t1x rsid attributes remain\n")
        rc, _ = self.sweep(self.bad)
        self.assertEqual(rc, 0, "the stale TSV blesses the finding away")
        rc, out = self.sweep(self.bad, against=self.good)
        self.assertEqual(rc, 1, out)
        self.assertIn("rsid", out)

    def test_sweep_without_against_keeps_the_tsv_ratchet(self) -> None:
        rc, out = self.sweep(self.bad)
        self.assertEqual(rc, 1, out)
        self.assertIn("NEW", out)

    def test_against_belongs_to_sweep_only(self) -> None:
        for mode in ("bless", "mine"):
            r = subprocess.run(
                [sys.executable, str(HERE / "parity_ladder.py"), mode,
                 "--against", self.good, "--corpus", str(self.corpus)],
                capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(r.returncode, 2, mode)
            self.assertIn("sweep", r.stderr, mode)

    def test_sweep_accepts_against_from_the_command_line(self) -> None:
        r = subprocess.run(
            [sys.executable, str(HERE / "parity_ladder.py"), "sweep",
             "--bin", self.good, "--against", self.good,
             "--corpus", str(self.corpus)],
            capture_output=True, text=True, timeout=60,
        )
        self.assertNotEqual(r.returncode, 2, r.stderr)


if __name__ == "__main__":
    unittest.main()
