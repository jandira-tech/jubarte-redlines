#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The lockfiles pin pulldown-cmark at the RUSTSEC-2026-0433 fix (#2055).

0.13.5 is the affected release. cargo-deny reads each workspace lock, so
every manifest security-check.sh audits must resolve 0.13.6.
"""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LOCKS = (
    ROOT / "Cargo.lock",
    ROOT / "jubarte-python" / "Cargo.lock",
    ROOT / "jubarte-wasm" / "Cargo.lock",
    ROOT / "jubarte-rust-inproc" / "Cargo.lock",
)


def locked_version(lock: Path, name: str) -> str | None:
    """The version of the last `name` package record in a Cargo.lock."""
    version: str | None = None
    seen = False
    for line in lock.read_text(encoding="utf-8").splitlines():
        if line == f'name = "{name}"':
            seen = True
            continue
        if seen and line.startswith("version = "):
            version = line.split('"')[1]
            seen = False
    return version


class PulldownCmarkLock(unittest.TestCase):
    def test_every_audited_lock_resolves_0_13_6(self) -> None:
        for lock in LOCKS:
            self.assertEqual(
                locked_version(lock, "pulldown-cmark"),
                "0.13.6",
                lock.relative_to(ROOT).as_posix(),
            )


if __name__ == "__main__":
    unittest.main()
