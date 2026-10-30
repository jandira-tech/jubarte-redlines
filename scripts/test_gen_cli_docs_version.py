#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Keep generated help independent of the checkout's build identity."""

import sys
import unittest

import gen_cli_docs


class HelpVersion(unittest.TestCase):
    @staticmethod
    def help_text(text):
        # Integration boundary: a tiny CLI prints controlled help text.
        return gen_cli_docs.run_help([sys.executable, "-c", f"print({text!r}, end='')"], [])

    def test_only_header_build_metadata_is_removed(self):
        for program in ("jubarte", "jubarte-redlines"):
            for suffix in ("+abc1234", "+abc1234.dirty", ""):
                with self.subTest(program=program, suffix=suffix):
                    text = f"{program} 0.15.0{suffix}\nRead documents\nexample+abc1234"
                    self.assertEqual(
                        self.help_text(text),
                        f"{program} 0.15.0\nRead documents\nexample+abc1234",
                    )

    def test_other_help_and_prerelease_versions_are_preserved(self):
        for text in ("Read documents", "jubarte 0.15.0-rc.1", "other 0.15.0+abc1234"):
            with self.subTest(text=text):
                self.assertEqual(self.help_text(text), text)

    def test_empty_help_is_preserved(self):
        self.assertEqual(self.help_text(""), "")

    def test_cli_failure_reports_the_exit_status(self):
        with self.assertRaisesRegex(SystemExit, r"--help` exited 2: broken"):
            gen_cli_docs.run_help(
                [sys.executable, "-c", "import sys; print('broken', file=sys.stderr); sys.exit(2)"],
                [],
            )

    def test_summary_uses_the_description_below_the_version_header(self):
        for header in ("", "jubarte 0.15.0+abc1234.dirty\n", "jubarte-redlines 0.15.0\n"):
            text = f"{header}Read documents\n\nTasks:\n  read  Read a document\n"
            runner = [sys.executable, "-c", f"print({text!r}, end='')"]
            with self.subTest(header=header):
                self.assertEqual(
                    gen_cli_docs.build_summary(runner, "jubarte"),
                    "| Command | What it does |\n|---|---|\n"
                    "| `jubarte ORIGINAL MODIFIED` | Read documents |\n"
                    "| `jubarte read` | Read a document |",
                )


if __name__ == "__main__":
    unittest.main()
