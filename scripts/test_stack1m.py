#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/stack1m.py: sample rows become jobs, the stack limit reaches the binary."""

import io
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest import mock

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
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.tmp = Path(temporary.name)
        self.binary = self.tmp / "jubarte"
        self.binary.write_text(FAKE)
        self.binary.chmod(0o755)
        self.log = self.tmp / "stack.log"
        environment = mock.patch.dict(os.environ, {"STACK_LOG": str(self.log)})
        environment.start()
        self.addCleanup(environment.stop)
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

    def test_a_versioned_csv_name_keeps_every_row_s_own_output(self):
        versioned = self.tmp / "sample_redline_0.12.0_10-10-26_08-23.csv"
        versioned.write_text("key,base,next\nk,a.docx,b.docx\nl,c.docx,d.docx\n")
        outputs = [tail[4] for _, tail in stack1m.jobs(Path("/bench"), Path("/out"), versioned)]
        self.assertEqual(outputs, ["/out/sample_redline_0.12.0_10-10-26_08-23_0.docx",
                                   "/out/sample_redline_0.12.0_10-10-26_08-23_1.docx"])

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

    def test_quoted_unicode_paths_and_extra_metadata_are_preserved(self):
        self.redline.write_text(
            'key,base,next,base_sha256,next_sha256\r\n'
            'k,"corpus/ação, old.docx","corpus/new ""draft"".docx",abc,def\r\n',
            encoding="utf-8",
        )
        self.assertEqual(list(stack1m.jobs(Path("/bench root"), Path("/out dir"), self.redline)), [
            ("sample_redline.csv:0", ["compare", "/bench root/corpus/ação, old.docx",
                                     '/bench root/corpus/new "draft".docx', "-o",
                                     "/out dir/sample_redline_0.docx", "--force"]),
        ])

    def test_conversion_rows_get_distinct_outputs_even_for_repeated_inputs(self):
        sample = self.tmp / "sample_conversion_0.12.0.csv"
        sample.write_text("state,stem,docx,sha256\nclean,s,a.docx,abc\nrevised,s,a.docx,abc\n")
        self.assertEqual(list(stack1m.jobs(Path("/bench"), Path("/out"), sample)), [
            ("sample_conversion_0.12.0.csv:0",
             ["convert", "/bench/a.docx", "-o", "/out/sample_conversion_0.12.0_0.pdf", "--force"]),
            ("sample_conversion_0.12.0.csv:1",
             ["convert", "/bench/a.docx", "-o", "/out/sample_conversion_0.12.0_1.pdf", "--force"]),
        ])

    def test_empty_samples_produce_no_jobs(self):
        for contents in ("", "key,base,next\n", "state,stem,docx\n"):
            with self.subTest(contents=contents):
                self.redline.write_text(contents)
                self.assertEqual(list(stack1m.jobs(self.tmp, self.tmp, self.redline)), [])

    def test_main_forwards_defaults_and_creates_nested_output_directory(self):
        out = self.tmp / "nested" / "out"
        printed = io.StringIO()
        with mock.patch.object(stack1m, "run", return_value=("sample_redline.csv:0", None)) as run:
            with redirect_stdout(printed):
                code = stack1m.main([str(self.binary), str(self.tmp), str(out), str(self.redline)])
        self.assertEqual(code, 0)
        self.assertTrue(out.is_dir())
        run.assert_called_once_with(str(self.binary), 1024, 300, "sample_redline.csv:0", [
            "compare", str(self.tmp / "corpus/a.docx"), str(self.tmp / "corpus/b.docx"),
            "-o", str(out / "sample_redline_0.docx"), "--force",
        ])
        self.assertEqual(printed.getvalue(), "1 jobs at a 1024 KiB main-thread stack: 0 failed\n")

    def test_main_reports_all_failures_sorted_despite_completion_order(self):
        self.redline.write_text("key,base,next\nk,a.docx,b.docx\nl,c.docx,d.docx\n")
        out = self.tmp / "out"
        out.mkdir()
        sentinel = out / "keep.txt"
        sentinel.write_text("existing output")
        problems = {"sample_redline.csv:0": "signal 11: overflow",
                    "sample_redline.csv:1": None,
                    "sample_conversion.csv:0": "timed out after 7 s"}

        def run(binary, kib, timeout, label, tail):
            return label, problems[label]

        printed = io.StringIO()
        with mock.patch.object(stack1m, "run", side_effect=run) as invoked:
            # Yield in reverse submission order without timing-dependent sleeps.
            with mock.patch.object(stack1m.concurrent.futures, "as_completed", side_effect=reversed):
                with redirect_stdout(printed):
                    code = stack1m.main([str(self.binary), str(self.tmp), str(out),
                                         str(self.conversion), str(self.redline),
                                         "--kib", "2048", "--timeout", "7", "--jobs", "1"])
        self.assertEqual(code, 1)
        self.assertEqual(invoked.call_count, 3)
        self.assertCountEqual([call.args[3] for call in invoked.call_args_list], list(problems))
        for call in invoked.call_args_list:
            self.assertEqual(call.args[:3], (str(self.binary), 2048, 7))
        self.assertEqual(printed.getvalue().splitlines(), [
            "FAIL sample_conversion.csv:0: timed out after 7 s",
            "FAIL sample_redline.csv:0: signal 11: overflow",
            "3 jobs at a 2048 KiB main-thread stack: 2 failed",
        ])
        self.assertEqual(sentinel.read_text(), "existing output")

    def test_main_with_header_only_sample_does_not_launch_binary(self):
        self.redline.write_text("key,base,next\n")
        printed = io.StringIO()
        with mock.patch.object(stack1m, "run") as run:
            with redirect_stdout(printed):
                code = stack1m.main([str(self.binary), str(self.tmp), str(self.tmp / "out"),
                                     str(self.redline)])
        run.assert_not_called()
        self.assertEqual(code, 0)
        self.assertEqual(printed.getvalue(), "0 jobs at a 1024 KiB main-thread stack: 0 failed\n")

    def test_missing_sample_is_not_reported_as_success(self):
        with mock.patch.object(stack1m, "run") as run:
            with self.assertRaises(FileNotFoundError):
                stack1m.main([str(self.binary), str(self.tmp), str(self.tmp / "out"),
                              str(self.tmp / "missing.csv")])
        run.assert_not_called()

    def test_invalid_cli_arguments_fail_before_creating_output_or_running_jobs(self):
        out = self.tmp / "out"
        required = [str(self.binary), str(self.tmp), str(out)]
        cases = [required]  # At least one sample CSV is required.
        cases.extend(required + [str(self.redline), option, "invalid"]
                     for option in ("--kib", "--timeout", "--jobs"))
        for argv in cases:
            with self.subTest(argv=argv):
                with mock.patch.object(stack1m, "run") as run:
                    with redirect_stderr(io.StringIO()):
                        with self.assertRaises(SystemExit) as error:
                            stack1m.main(argv)
                self.assertEqual(error.exception.code, 2)
                self.assertFalse(out.exists())
                run.assert_not_called()

    def test_custom_stack_limit_reaches_the_binary(self):
        self.assertEqual(stack1m.run(str(self.binary), 2048, 5, "sample:0", ["compare"]),
                         ("sample:0", None))
        self.assertEqual(self.log.read_text().splitlines(), ["2048"])


class Run(unittest.TestCase):
    def test_success_ignores_stderr_and_propagates_timeout(self):
        result = subprocess.CompletedProcess([], 0, stderr=b"warning")
        with mock.patch.object(stack1m.subprocess, "run", return_value=result) as invoked:
            self.assertEqual(stack1m.run("/binary path", 2048, 7, "sample:0", ["convert", "a b.docx"]),
                             ("sample:0", None))
        self.assertEqual(invoked.call_args.kwargs, {
            "stdout": subprocess.DEVNULL, "stderr": subprocess.PIPE, "timeout": 7,
        })

    def test_exit_status_and_signal_are_distinguished(self):
        for status, reason in ((3, "exit 3"), (127, "exit 127"), (-11, "signal 11"), (-9, "signal 9")):
            with self.subTest(status=status):
                result = subprocess.CompletedProcess([], status, stderr=b"  diagnostic\n")
                with mock.patch.object(stack1m.subprocess, "run", return_value=result):
                    self.assertEqual(stack1m.run("jubarte", 1024, 5, "sample:2", []),
                                     ("sample:2", reason + ": diagnostic"))

    def test_failure_without_stderr_still_reports_exit_status(self):
        with mock.patch.object(stack1m.subprocess, "run",
                               return_value=subprocess.CompletedProcess([], 1, stderr=b"")):
            self.assertEqual(stack1m.run("jubarte", 1024, 5, "sample:0", []),
                             ("sample:0", "exit 1: "))

    def test_long_invalid_utf8_diagnostic_keeps_only_last_200_characters(self):
        diagnostic = ("é" * 205).encode("utf-8") + b"\xffend\n"
        with mock.patch.object(stack1m.subprocess, "run",
                               return_value=subprocess.CompletedProcess([], 2, stderr=diagnostic)):
            self.assertEqual(stack1m.run("jubarte", 1024, 5, "sample:0", []),
                             ("sample:0", "exit 2: " + "é" * 196 + "\ufffdend"))

    def test_timeout_is_returned_as_a_job_failure(self):
        with mock.patch.object(stack1m.subprocess, "run",
                               side_effect=subprocess.TimeoutExpired(["jubarte"], 7)):
            self.assertEqual(stack1m.run("jubarte", 1024, 7, "sample:3", []),
                             ("sample:3", "timed out after 7 s"))

    def test_shell_preserves_binary_path_and_literal_arguments(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "fake jubarte's binary"
            binary.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$ARG_LOG"\n')
            binary.chmod(0o755)
            log = root / "arguments"
            marker = root / "must-not-exist"
            tail = ["convert", "a b.docx", 'quote"and\'apostrophe',
                    "$(touch {})".format(marker), "*.docx", "", "--force"]
            with mock.patch.dict(os.environ, {"ARG_LOG": str(log)}):
                self.assertEqual(stack1m.run(str(binary), 1024, 5, "sample:0", tail),
                                 ("sample:0", None))
            self.assertEqual(log.read_text().splitlines(), tail)
            self.assertFalse(marker.exists())


if __name__ == "__main__":
    unittest.main()
