#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/semver_lints.py: the patch gate denies every lint the installed
cargo-semver-checks would otherwise skip (Allow) or only report (Warn).

Codex review of #1743 (#1744): 0.51.0 ships `inherent_method_added` and
`pub_const_added` at Allow, so a patch that grows the public API passed
`check-release --release-type patch`.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
SCRIPT = HERE / "semver_lints.py"
RELEASE_SH = HERE / "release.sh"


def lint(required: str, level: str) -> str:
    return f'SemverQuery(\n    id: "x",\n    required_update: {required},\n    lint_level: {level},\n)\n'


class Audit(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="semverlints-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.lints = self.tmp / "lints"
        self.lints.mkdir()
        (self.lints / "method_added.ron").write_text(lint("Minor", "Allow"))
        (self.lints / "sealed_became_unsealed.ron").write_text(lint("Major", "Warn"))
        (self.lints / "fn_missing.ron").write_text(lint("Major", "Deny"))

    def audit(self, table: str, *args: str, **env: str) -> subprocess.CompletedProcess[str]:
        manifest = self.tmp / "Cargo.toml"
        manifest.write_text(
            '[package]\nname = "x"\nversion = "0.1.0"\n\n'
            f"[package.metadata.cargo-semver-checks.lints]\n{table}"
        )
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--manifest", str(manifest), *args],
            capture_output=True, text=True, timeout=60, env=dict(os.environ, **env),
        )

    def test_a_table_denying_every_skipped_lint_passes(self) -> None:
        r = self.audit('method_added = "deny"\nsealed_became_unsealed = { level = "deny" }\n',
                       "--lints-dir", str(self.lints))
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_lint_left_at_its_level_is_named(self) -> None:
        r = self.audit('method_added = "deny"\n', "--lints-dir", str(self.lints))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("sealed_became_unsealed (Major, Warn)", r.stderr)
        self.assertNotIn("fn_missing", r.stderr)

    def test_a_lint_raised_only_to_warn_is_not_denied(self) -> None:
        r = self.audit('method_added = "warn"\nsealed_became_unsealed = "deny"\n',
                       "--lints-dir", str(self.lints))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("method_added (Minor, Allow)", r.stderr)

    def test_no_lint_sources_is_its_own_refusal(self) -> None:
        r = self.audit('method_added = "deny"\n', "--lints-dir", str(self.tmp / "absent"))
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("cargo install", r.stderr)

    def test_the_installed_version_finds_its_sources_under_cargo_home(self) -> None:
        home = self.tmp / "cargo-home"
        src = home / "registry" / "src" / "index.crates.io-x" / "cargo-semver-checks-1.2.3" / "src"
        shutil.copytree(self.lints, src / "lints")
        table = 'method_added = "deny"\nsealed_became_unsealed = "deny"\n'
        r = self.audit(table, "--tool-version", "1.2.3", CARGO_HOME=str(home))
        self.assertEqual(r.returncode, 0, r.stderr)
        r = self.audit(table, "--tool-version", "1.2.4", CARGO_HOME=str(home))
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("1.2.4", r.stderr)

    def test_this_manifest_denies_what_the_installed_tool_skips(self) -> None:
        r = subprocess.run([sys.executable, str(SCRIPT), "--manifest", str(ROOT / "Cargo.toml")],
                           capture_output=True, text=True, timeout=120)
        if r.returncode == 2 and "no cargo-semver-checks lint sources" in r.stderr:
            self.skipTest(r.stderr.strip())
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_the_patch_gate_audits_before_it_checks(self) -> None:
        text = RELEASE_SH.read_text()
        gate = text[text.index("bump_gate() {"):]
        gate = gate[:gate.index("\n}\n")]
        self.assertLess(gate.index("semver_lints_ok"), gate.index("cargo semver-checks check-release"))
        self.assertIn('"${py312[@]}" scripts/test_semver_lints.py', text)


@unittest.skipUnless(shutil.which("cargo-semver-checks"), "cargo-semver-checks is not installed")
class RealCrate(unittest.TestCase):
    """The defect end to end: a patch that adds a public method and says so
    under `### Changed` passes check-release unless the table denies it."""

    def setUp(self) -> None:
        self.crate = Path(tempfile.mkdtemp(prefix="semverlints-crate-"))
        self.addCleanup(shutil.rmtree, self.crate, ignore_errors=True)
        self.git = ["git", "-C", str(self.crate), "-c", "user.name=t", "-c", "user.email=t@t"]
        subprocess.run([*self.git, "init", "-q", "-b", "main"], check=True)
        (self.crate / "src").mkdir()
        (self.crate / ".gitignore").write_text("/target\n")
        self.write("0.1.0", "", "")
        subprocess.run([*self.git, "add", "-A"], check=True)
        subprocess.run([*self.git, "commit", "-q", "-m", "0.1.0"], check=True)
        subprocess.run([*self.git, "tag", "v0.1.0"], check=True)

    def write(self, version: str, method: str, table: str) -> None:
        (self.crate / "Cargo.toml").write_text(
            f'[package]\nname = "semverprobe"\nversion = "{version}"\nedition = "2021"\n\n'
            f"[package.metadata.cargo-semver-checks.lints]\n{table}"
        )
        (self.crate / "src" / "lib.rs").write_text(
            f"pub struct Doc;\n\nimpl Doc {{\n    pub fn open() -> Self {{ Doc }}\n{method}}}\n"
        )

    def check(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["cargo", "semver-checks", "check-release", "--baseline-rev", "v0.1.0",
             "--release-type", "patch", "--default-features"],
            cwd=self.crate, capture_output=True, text=True, timeout=600,
        )

    def test_a_new_method_in_a_patch(self) -> None:
        method = "    pub fn close(&self) {}\n"
        self.write("0.1.1", method, "")
        self.assertEqual(self.check().returncode, 0, "the tool's own levels pass it (the defect)")
        self.write("0.1.1", method, 'inherent_method_added = "deny"\n')
        r = self.check()
        self.assertNotEqual(r.returncode, 0, r.stdout)
        self.assertIn("inherent_method_added", r.stdout + r.stderr)


if __name__ == "__main__":
    unittest.main()
