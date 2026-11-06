#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The glibc floor gate parses version needs and refuses a newer one."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GATE = ROOT / "scripts" / "check_glibc_floor.py"


def run(*args: str, symbols: str) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "readelf.txt"
        path.write_text(symbols, encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(GATE), *args, "--symbols", str(path)],
            capture_output=True,
            text=True,
            check=False,
        )


class GlibcFloor(unittest.TestCase):
    def test_2_28_and_older_pass(self) -> None:
        result = run("--max", "2.28", symbols="GLIBC_2.17\nGLIBC_2.28\n")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_2_34_fails_the_2_28_floor(self) -> None:
        result = run("--max", "2.28", symbols="GLIBC_2.28\nGLIBC_2.34\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("GLIBC_2.34", result.stderr)
        self.assertIn("GLIBC_2.28", result.stderr)

    def test_2_43_is_the_version_ubuntu_latest_links(self) -> None:
        result = run("--max", "2.28", symbols="GLIBC_2.38\nGLIBC_2.43\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("GLIBC_2.43", result.stderr)

    def test_musl_refuses_any_glibc_symbol(self) -> None:
        result = run("--musl", symbols="GLIBC_2.17\n")
        self.assertEqual(result.returncode, 1)
        self.assertIn("GLIBC_2.17", result.stderr)

    def test_musl_symbol_text_with_no_glibc_passes(self) -> None:
        result = run("--musl", symbols="Symbol table has no version needs\n")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_a_missing_binary_is_an_error(self) -> None:
        result = subprocess.run(
            [sys.executable, str(GATE), "no-such-jubarte"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("not a file", result.stderr)


if __name__ == "__main__":
    unittest.main()
