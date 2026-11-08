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
sys.path.insert(0, str(GATE.parent))
import check_glibc_floor

# `readelf -lW` of a Rust musl binary: static-pie, so DYN with no INTERP.
STATIC_PIE_HEADERS = """
Elf file type is DYN (Position-Independent Executable file)
Program Headers:
  Type           Offset   VirtAddr           PhysAddr           FileSiz  MemSiz   Flg Align
  PHDR           0x000040 0x0000000000000040 0x0000000000000040 0x000230 0x000230 R   0x8
  LOAD           0x000000 0x0000000000000000 0x0000000000000000 0x2a1b30 0x2a1b30 R   0x1000
  DYNAMIC        0x3f1d28 0x00000000003f2d28 0x00000000003f2d28 0x000170 0x000170 RW  0x8
  GNU_STACK      0x000000 0x0000000000000000 0x0000000000000000 0x000000 0x000000 RW  0x10
"""
# The same for a glibc binary: it names its loader.
DYNAMIC_HEADERS = """
Elf file type is DYN (Position-Independent Executable file)
Program Headers:
  PHDR           0x000040 0x0000000000000040 0x0000000000000040 0x0002d8 0x0002d8 R   0x8
  INTERP         0x000318 0x0000000000000318 0x0000000000000318 0x00001c 0x00001c R   0x1
      [Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]
  LOAD           0x000000 0x0000000000000000 0x0000000000000000 0x2a1b30 0x2a1b30 R   0x1000
"""


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

    def test_a_static_pie_has_no_interpreter(self) -> None:
        self.assertTrue(check_glibc_floor.static_from_program_headers(STATIC_PIE_HEADERS))

    def test_a_dynamic_binary_names_its_interpreter(self) -> None:
        self.assertFalse(check_glibc_floor.static_from_program_headers(DYNAMIC_HEADERS))

    def test_file_and_ldd_wording_for_static_pie_is_static(self) -> None:
        # `file` says static-pie; glibc's ldd says statically linked.
        file_out = "jubarte: ELF 64-bit LSB pie executable, x86-64, static-pie linked, stripped"
        self.assertTrue(check_glibc_floor.static_from_descriptions(file_out, ""))
        self.assertTrue(check_glibc_floor.static_from_descriptions("", "\tstatically linked\n"))
        self.assertTrue(check_glibc_floor.static_from_descriptions("", "\tnot a dynamic executable\n"))

    def test_file_and_ldd_wording_for_a_dynamic_binary_is_not(self) -> None:
        file_out = "jubarte: ELF 64-bit LSB pie executable, x86-64, dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2"
        ldd_out = "\tlibc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x7f)\n"
        self.assertFalse(check_glibc_floor.static_from_descriptions(file_out, ldd_out))

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
