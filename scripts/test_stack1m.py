#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/stack1m.py: sample rows become jobs, the stack limit reaches the binary."""

import io
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import stack1m  # noqa: E402

# A stand-in binary: records its stack limit, fails `convert`.
FAKE = """#!/bin/sh
ulimit -s >> "$STACK_LOG"
[ "$1" = compare ] && exit 0
echo "no such part" >&2
exit 3
"""


class Stack1m(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.binary = self.tmp / "jubarte"
        self.binary.write_text(FAKE)
        self.binary.chmod(0o755)
        self.log = self.tmp / "stack.log"
        os.environ["STACK_LOG"] = str(self.log)
        self.redline = self.tmp / "sample_redline.csv"
        self.redline.write_text("key,base,next\nk,corpus/a.docx,corpus/b.docx\n")
        self.conversion = self.tmp / "sample_conversion.csv"
        self.conversion.write_text("state,stem,docx\nclean,s,corpus/c.docx\n")

    def test_rows_become_compare_and_convert_jobs_under_the_bench_root(self):
        bench, out = Path("/bench"), Path("/out")
        (label, compare), = stack1m.jobs(bench, out, self.redline)
        self.assertEqual(label, "sample_redline.csv:0")
        self.assertEqual(compare[:3], ["compare", "/bench/corpus/a.docx", "/bench/corpus/b.docx"])
        self.assertEqual(compare[3:5], ["-o", "/out/sample_redline_0.docx"])
        (_, convert), = stack1m.jobs(bench, out, self.conversion)
        self.assertEqual(convert[:4], ["convert", "/bench/corpus/c.docx", "-o", "/out/sample_conversion_0.pdf"])

    def test_the_limit_reaches_the_binary_and_failures_fail_the_run(self):
        printed = io.StringIO()
        with redirect_stdout(printed):
            code = stack1m.main([str(self.binary), str(self.tmp), str(self.tmp / "out"),
                                 str(self.redline), str(self.conversion), "--kib", "1024"])
        self.assertEqual(code, 1)
        self.assertEqual(self.log.read_text().split(), ["1024", "1024"])
        self.assertIn("FAIL sample_conversion.csv:0: exit 3: no such part", printed.getvalue())
        self.assertIn("2 jobs at a 1024 KiB main-thread stack: 1 failed", printed.getvalue())

    def test_a_clean_run_exits_zero(self):
        with redirect_stdout(io.StringIO()):
            code = stack1m.main([str(self.binary), str(self.tmp), str(self.tmp / "out"), str(self.redline)])
        self.assertEqual(code, 0)


if __name__ == "__main__":
    unittest.main()
