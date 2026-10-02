#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""README pins rewritten by scripts/bump-version.mjs.

Each case runs a copy of the script against a throwaway Cargo.toml and
README.md, so the checkout is never touched.
"""

from __future__ import annotations

import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
BUMP = HERE / "bump-version.mjs"

CARGO = '[package]\nname = "jubarte-redlines"\nversion = "0.10.1"\n'


@unittest.skipUnless(shutil.which("bun"), "bun is not installed")
class ReadmePins(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="bump_version_"))
        (self.tmp / "scripts").mkdir()
        shutil.copy(BUMP, self.tmp / "scripts" / "bump-version.mjs")
        (self.tmp / "Cargo.toml").write_text(CARGO)

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    def bump(self, readme: str, version: str) -> str:
        (self.tmp / "README.md").write_text(readme)
        r = subprocess.run(
            ["bun", str(self.tmp / "scripts" / "bump-version.mjs"), version],
            capture_output=True, text=True, timeout=60,
        )
        self.assertEqual(r.returncode, 0, r.stderr)
        return (self.tmp / "README.md").read_text()

    def test_library_pin_follows_a_minor_bump(self) -> None:
        # The README kept `version = "0.9"` through 0.10.0 and 0.10.1.
        out = self.bump(
            'jubarte-redlines = { version = "0.9", default-features = false }\n',
            "0.11.0",
        )
        self.assertIn('jubarte-redlines = { version = "0.11", default-features = false }', out)

    def test_library_pin_is_left_alone_by_a_patch_bump(self) -> None:
        pin = 'jubarte-redlines = { version = "0.10", default-features = false }\n'
        self.assertEqual(self.bump(pin, "0.10.2"), pin)

    def test_socket_badge_follows_the_bump(self) -> None:
        out = self.bump(
            "https://badge.socket.dev/cargo/package/jubarte-redlines/0.10.1\n", "0.10.2"
        )
        self.assertIn("jubarte-redlines/0.10.2", out)


if __name__ == "__main__":
    unittest.main()
