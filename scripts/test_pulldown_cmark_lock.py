#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The lockfiles pin pulldown-cmark at the RUSTSEC-2026-0433 fix (#2055).

0.13.5 is the affected release. cargo-deny reads each workspace lock, so
every committed lock security-check.sh audits must resolve 0.13.6 and
nothing else (fuzz/ commits no lock; cargo-deny resolves it fresh).
"""

from __future__ import annotations

import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LOCKS = (
    ROOT / "Cargo.lock",
    ROOT / "jubarte-python" / "Cargo.lock",
    ROOT / "jubarte-wasm" / "Cargo.lock",
    ROOT / "jubarte-rust-inproc" / "Cargo.lock",
)


def locked_versions(lock: Path, name: str) -> set[str]:
    """Every version a Cargo.lock records for `name`; one lock may hold several."""
    with lock.open("rb") as f:
        doc = tomllib.load(f)
    return {p["version"] for p in doc.get("package", []) if p.get("name") == name}


class PulldownCmarkLock(unittest.TestCase):
    def test_an_earlier_vulnerable_record_is_seen(self) -> None:
        # a lock may hold two versions of one crate; the old parser kept
        # only the last record and missed an earlier 0.13.5
        with tempfile.TemporaryDirectory() as tmp:
            lock = Path(tmp) / "Cargo.lock"
            lock.write_text(
                'version = 4\n\n'
                '[[package]]\nname = "pulldown-cmark"\nversion = "0.13.5"\n\n'
                '[[package]]\nname = "pulldown-cmark"\nversion = "0.13.6"\n',
                encoding="utf-8",
            )
            self.assertEqual(locked_versions(lock, "pulldown-cmark"), {"0.13.5", "0.13.6"})

    def test_every_audited_lock_resolves_0_13_6(self) -> None:
        for lock in LOCKS:
            self.assertEqual(
                locked_versions(lock, "pulldown-cmark"),
                {"0.13.6"},
                lock.relative_to(ROOT).as_posix(),
            )


if __name__ == "__main__":
    unittest.main()
