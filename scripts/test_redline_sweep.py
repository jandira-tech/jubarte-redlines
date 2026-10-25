#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Unit tests for redline-sweep.sh's validation baseline key extraction.

Execute the production filter alone: no Rust binary, Word or .NET required.
"""

import hashlib
import os
import shutil
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



# a stub jubarte: --version, or `BASE NEXT -d … -o DEST --force --quiet`
STUB = """#!/bin/sh
[ "$1" = --version ] && { echo "jubarte 9.9.9"; exit 0; }
echo "$*" >> "$ARGV_LOG"
while [ $# -gt 0 ]; do [ "$1" = -o ] && dest=$2; shift; done
echo redline > "$dest"
"""
RELEASE_HEADER = ("key,base,base_sha256,next,next_sha256,docx,docx_sha256,pdf,pdf_sha256,state,id,sets,oracle,"
                  "oracle_pdf,oracle_pdf_sha256,docxodus_pdf,docxodus_pdf_sha256,jubarte_docx,jubarte_docx_sha256,"
                  "jubarte_pdf,jubarte_pdf_sha256")


@unittest.skipUnless(shutil.which("zsh"), "redline-sweep.sh is a zsh script")
class Sweep(unittest.TestCase):
    """The whole sweep with a stub binary (#1757, before#29/#30): it reads the
    release_info sample layout as well as the bench mapping, names the binary
    it ran, and never passes a run that generated nothing."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="sweep-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.src = self.tmp / "bench"
        (self.src / "corpus").mkdir(parents=True)
        for name in ("a.docx", "b.docx", "c.docx"):
            (self.src / "corpus" / name).write_text(name)
        self.bin = self.tmp / "jubarte"
        self.bin.write_text(STUB)
        self.bin.chmod(0o755)
        self.out = self.tmp / "out"
        self.argv = self.tmp / "argv.log"

    def sweep(self, csv_text: str) -> subprocess.CompletedProcess[str]:
        csv_path = self.tmp / "sample.csv"
        csv_path.write_bytes(csv_text.encode())
        return subprocess.run(["zsh", str(HERE / "redline-sweep.sh"), str(csv_path), str(self.src), str(self.out)],
                              env=dict(os.environ, JUBARTE_BIN=str(self.bin), ARGV_LOG=str(self.argv)),
                              capture_output=True, text=True, timeout=60)

    def release_row(self, key: str, base: str, nxt: str) -> str:
        return ",".join([key, base, "0" * 64, nxt, "0" * 64] + [""] * 16)

    def test_the_release_info_layout_is_read_crlf_and_all(self) -> None:
        rows = [RELEASE_HEADER, self.release_row("k1", "corpus/a.docx", "corpus/b.docx"),
                self.release_row("k2", "corpus/b.docx", "corpus/c.docx")]
        r = self.sweep("\r\n".join(rows) + "\r\n")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("generation: ok=2 fail=0 skipped=0", r.stdout)
        self.assertEqual(sorted(p.name for p in self.out.glob("*.docx")), ["k1.docx", "k2.docx"])
        self.assertIn(f"{self.src}/corpus/a.docx {self.src}/corpus/b.docx -d 1970-01-01T00:00:00Z -o ",
                      self.argv.read_text())

    def test_the_binary_is_named_by_path_sha256_and_version(self) -> None:
        r = self.sweep(RELEASE_HEADER + "\n" + self.release_row("k1", "corpus/a.docx", "corpus/b.docx") + "\n")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        line = f"binary: {self.bin} sha256 {hashlib.sha256(self.bin.read_bytes()).hexdigest()} (jubarte 9.9.9)"
        self.assertIn(line, r.stdout)
        self.assertIn(line, (self.out / "sweep.log").read_text())

    def test_a_release_sample_source_that_is_missing_fails(self) -> None:
        r = self.sweep(RELEASE_HEADER + "\n" + self.release_row("k1", "corpus/a.docx", "corpus/b.docx") + "\n"
                       + self.release_row("k2", "corpus/a.docx", "corpus/gone.docx") + "\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("k2", (self.out / "sweep.log").read_text())

    def test_the_bench_mapping_layout_still_works(self) -> None:
        r = self.sweep("pair_stem,base,next,origin,src_base,src_next\np1,x,y,o,corpus/a.docx,corpus/b.docx\n"
                       "p2,x,y,o,corpus/a.docx,corpus/gone.docx\n")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("generation: ok=1 fail=0 skipped=1", r.stdout)

    def test_a_run_that_generated_nothing_fails(self) -> None:
        # 0.13.0: the release CSVs read as a mapping skipped every pair and exited 0
        r = self.sweep("pair_stem,base,next,origin,src_base,src_next\np1,x,y,o,corpus/gone.docx,corpus/b.docx\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("no pair was generated", r.stderr)

    def test_a_csv_of_another_layout_is_refused(self) -> None:
        r = self.sweep("stem,docx\ns1,corpus/a.docx\n")
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("sample.csv", r.stderr)


if __name__ == "__main__":
    unittest.main()
