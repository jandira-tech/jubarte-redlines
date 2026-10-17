#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Integration checks for the temporary zip 9 oracle dependency patch."""

import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FORK = "https://github.com/jandira-tech/ooxmlsdk"
REV = "d7c3ae016da8ef661d9a04e12c47003d8fe51eff"


class OracleDependency(unittest.TestCase):
    def test_patch_uses_pinned_fork(self) -> None:
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        self.assertEqual(
            manifest["patch"]["crates-io"]["ooxmlsdk"],
            {"git": FORK, "rev": REV},
        )
        self.assertEqual(manifest["dev-dependencies"]["ooxmlsdk"]["version"], "0.13.2")

    def test_oracle_and_derive_resolve_from_same_revision(self) -> None:
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        packages = {p["name"]: p for p in lock["package"]}
        for name in ("ooxmlsdk", "ooxmlsdk-derive"):
            with self.subTest(package=name):
                self.assertEqual(packages[name].get("source"), f"git+{FORK}?rev={REV}#{REV}")
        self.assertEqual(
            {p["version"] for p in lock["package"] if p["name"] == "zip"},
            {"9.0.4"},
        )

    def test_fork_is_explicitly_allowed(self) -> None:
        policy = tomllib.loads((ROOT / "deny.toml").read_text())
        self.assertIn(FORK, policy["sources"].get("allow-git", []))

    def test_vendored_oracle_is_removed(self) -> None:
        self.assertFalse((ROOT / "vendor" / "ooxmlsdk").exists())
        self.assertNotIn("vendor/ooxmlsdk", (ROOT / "Cargo.toml").read_text())
        self.assertNotIn("vendor/", (ROOT / "REUSE.toml").read_text())
