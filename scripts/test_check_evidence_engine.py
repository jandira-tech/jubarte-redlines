#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/check_evidence_engine.py: the commit the release evidence names
builds the same CLI engine as the release tree (#1752, checklist 3#2).

Each case builds a throwaway repository: an engine commit the results name,
then a release tree that changes something after it.
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE / "check_evidence_engine.py"
RELEASE_SH = HERE / "release.sh"

CARGO_TOML = """[package]
name = "jubarte-redlines"
version = "0.1.0"
edition = "2021"

[package.metadata.release-notes]
"0.1.0" = "first"

[dependencies]
zip = "8"
"""
CARGO_LOCK = """version = 4

[[package]]
name = "jubarte-redlines"
version = "0.1.0"
dependencies = [
 "zip",
]

[[package]]
name = "zip"
version = "8.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"""


class EvidenceEngine(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = Path(tempfile.mkdtemp(prefix="evidence-engine-"))
        self.addCleanup(shutil.rmtree, self.repo, ignore_errors=True)
        self.git = ["git", "-C", str(self.repo), "-c", "user.name=t", "-c", "user.email=t@t"]
        subprocess.run([*self.git, "init", "-q", "-b", "main"], check=True)
        self.write("src/lib.rs", "pub fn f() {}\n")
        self.write("assets/font.txt", "a font\n")
        self.write("Cargo.toml", CARGO_TOML)
        self.write("Cargo.lock", CARGO_LOCK)
        self.write("README.md", "readme\n")
        self.engine = self.commit("engine")

    def write(self, path: str, text: str) -> None:
        (self.repo / path).parent.mkdir(parents=True, exist_ok=True)
        (self.repo / path).write_text(text)

    def commit(self, subject: str) -> str:
        subprocess.run([*self.git, "add", "-A"], check=True)
        subprocess.run([*self.git, "commit", "-q", "--allow-empty", "-m", subject], check=True)
        return subprocess.run([*self.git, "rev-parse", "HEAD"], capture_output=True, text=True,
                              check=True).stdout.strip()

    def results(self, redline: str, conversion: str | None = None) -> None:
        for kind, commit in (("redline", redline), ("conversion", conversion or redline)):
            self.write(f"release_info/results_{kind}_0.2.0_10-25-26_09-00.json",
                       json.dumps({"tools": {"jubarte": {"commit": commit, "version": "jubarte 0.1.0"}}}))

    def check(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run([sys.executable, str(SCRIPT), "0.2.0"], cwd=self.repo,
                              capture_output=True, text=True, timeout=60)

    def test_a_tree_that_changed_no_engine_input_passes(self) -> None:
        self.write("README.md", "a docs change\n")
        self.write("scripts/release.sh", "echo a script change\n")
        self.results(self.engine)
        self.commit("docs and scripts")
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn(self.engine[:12], r.stdout)

    def test_a_source_change_after_the_evidence_is_named(self) -> None:
        self.write("src/lib.rs", "pub fn f() { g() }\n")
        self.results(self.engine)
        self.commit("a fix after the evidence")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("src/lib.rs", r.stderr)

    def test_an_embedded_asset_change_is_named(self) -> None:
        self.write("assets/font.txt", "another font\n")
        self.results(self.engine)
        self.commit("a font")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("assets/font.txt", r.stderr)

    def test_a_binary_asset_change_is_named_not_a_traceback(self) -> None:
        # pi on #1752 (#1772): the fonts under assets/ are binary, and every
        # changed input was decoded as UTF-8
        (self.repo / "assets/fonts").mkdir(parents=True)
        (self.repo / "assets/fonts/Fake.ttf").write_bytes(b"\x00\x01\x00\x00\xff\xfe\x81")
        engine = self.commit("a binary font")
        (self.repo / "assets/fonts/Fake.ttf").write_bytes(b"\x00\x01\x00\x00\xff\xfe\x82")
        self.write("Cargo.toml", CARGO_TOML.replace('zip = "8"', 'zip = "9"'))
        self.results(engine)
        self.commit("another font, another zip")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertNotIn("Traceback", r.stderr)
        self.assertIn("assets/fonts/Fake.ttf", r.stderr)
        # the manifest comparison after the loop still runs
        self.assertIn("Cargo.toml", r.stderr)

    def test_a_path_holding_a_space_is_named_whole(self) -> None:
        self.write("assets/a font.txt", "one\n")
        engine = self.commit("a spaced name")
        self.write("assets/a font.txt", "two\n")
        self.results(engine)
        self.commit("it changed")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("! assets/a font.txt\n", r.stderr)

    def test_a_layout_change_counts_as_an_engine_change(self) -> None:
        # conservative by design: comments are set aside, layout is not —
        # whitespace runs collapse, but `{ 1 }` and `{1}` differ (#1772)
        r = self.changed_rs("pub fn f() -> u8 { 1 }\n", "pub fn f() -> u8 {1}\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        r = self.changed_rs("pub fn f() -> u8 {  1  }\n", "pub fn f() -> u8 {\n    1\n}\n")
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_the_docstrings_promise_no_layout_insensitivity(self) -> None:
        for path in (SCRIPT, RELEASE_SH):
            text = path.read_text()
            for claim in ("or layout", "token for token"):
                self.assertFalse(claim in text, f"{path.name} still says {claim!r}")

    def test_the_version_bump_and_release_notes_are_no_engine_change(self) -> None:
        # step 1's bump, step 4's notes and a semver-checks lint table (#1744)
        self.write("Cargo.toml", CARGO_TOML.replace('version = "0.1.0"', 'version = "0.2.0"')
                   .replace('"0.1.0" = "first"', '"0.1.0" = "first"\n"0.2.0" = "second"')
                   + '\n[package.metadata.cargo-semver-checks.lints]\ninherent_method_added = "deny"\n')
        self.write("Cargo.lock", CARGO_LOCK.replace('name = "jubarte-redlines"\nversion = "0.1.0"',
                                                    'name = "jubarte-redlines"\nversion = "0.2.0"'))
        self.results(self.engine)
        self.commit("release 0.2.0")
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_dependency_change_is_named(self) -> None:
        self.write("Cargo.toml", CARGO_TOML.replace('zip = "8"', 'zip = "9"'))
        self.write("Cargo.lock", CARGO_LOCK.replace('version = "8.1.0"', 'version = "9.0.0"'))
        self.results(self.engine)
        self.commit("zip 9")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("Cargo.toml", r.stderr)
        self.assertIn("Cargo.lock", r.stderr)

    def changed_rs(self, before: str, after: str) -> subprocess.CompletedProcess[str]:
        self.write("src/lib.rs", before)
        engine = self.commit("engine")
        self.write("src/lib.rs", after)
        self.results(engine)
        self.commit("after the evidence")
        return self.check()

    def test_a_comment_only_change_is_no_engine_change(self) -> None:
        # 0.12.0: a doc comment of src/convert/mod.rs changed after the evidence
        r = self.changed_rs(
            "/// one\n/* outer /* nested */ still */\npub fn f() -> u8 { 1 } // tail\n",
            "/// one, reworded\n//! more\n/* outer /* nested, edited */ still */\npub fn f() -> u8 { 1 }\n",
        )
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("comment-only", r.stdout)

    def test_a_comment_only_change_outside_src_is_named(self) -> None:
        # empryo on #1743 (#1781): an .rs under assets/ is an input as it
        # stands, compared byte for byte like every non-source input
        self.write("assets/helper.rs", "// one\npub fn h() {}\n")
        engine = self.commit("an asset that is Rust")
        self.write("assets/helper.rs", "// two\npub fn h() {}\n")
        self.results(engine)
        self.commit("its comment changed")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("assets/helper.rs", r.stderr)

    def test_code_beside_a_comment_is_compared(self) -> None:
        r = self.changed_rs("pub fn f() -> u8 { 1 } // one\n", "pub fn f() -> u8 { 2 } // one\n")
        self.assertEqual(r.returncode, 1, r.stdout)

    def test_a_string_holding_slashes_is_code(self) -> None:
        r = self.changed_rs('const U: &str = "http://a";\n', 'const U: &str = "http://b";\n')
        self.assertEqual(r.returncode, 1, r.stdout)
        r = self.changed_rs('const U: &str = r#"say "//" a"#;\n', 'const U: &str = r#"say "//" b"#;\n')
        self.assertEqual(r.returncode, 1, r.stdout)
        r = self.changed_rs('const U: &str = "a \\" // b";\n', 'const U: &str = "a \\" // c";\n')
        self.assertEqual(r.returncode, 1, r.stdout)

    def test_a_char_literal_and_a_lifetime_are_code(self) -> None:
        r = self.changed_rs("fn f<'a>(s: &'a str) -> char { if s.is_empty() { '\"' } else { '/' } } // x\n",
                            "fn f<'a>(s: &'a str) -> char { if s.is_empty() { '\"' } else { '*' } } // x\n")
        self.assertEqual(r.returncode, 1, r.stdout)

    def test_package_fields_cargo_does_not_build_from_are_set_aside(self) -> None:
        # 0.11.2: keywords and categories changed after the evidence
        self.write("Cargo.toml", CARGO_TOML.replace('edition = "2021"',
                   'edition = "2021"\nkeywords = ["docx"]\ncategories = ["text-processing"]\ndescription = "x"'))
        self.results(self.engine)
        self.commit("crates.io fields")
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_package_field_cargo_builds_from_is_named(self) -> None:
        self.write("Cargo.toml", CARGO_TOML.replace('edition = "2021"', 'edition = "2024"'))
        self.results(self.engine)
        self.commit("edition 2024")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("Cargo.toml", r.stderr)

    def test_results_naming_two_commits_disagree(self) -> None:
        other = self.commit("another engine")
        self.results(self.engine, other)
        self.commit("results")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("disagree", r.stderr)

    def test_a_commit_this_repository_lacks_is_a_usage_error(self) -> None:
        self.results("0" * 40)
        self.commit("results")
        r = self.check()
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("git fetch", r.stderr)

    def test_missing_results_are_a_usage_error(self) -> None:
        r = self.check()
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("results_redline_0.2.0_", r.stderr)

    def test_step_3_runs_it(self) -> None:
        text = RELEASE_SH.read_text()
        s3 = text[text.index('say "3. '):text.index('say "4. ')]
        self.assertIn('scripts/check_evidence_engine.py "$VER"', s3)
        self.assertIn("die", s3.split("check_evidence_engine.py", 1)[1][:400])
        self.assertIn('"${py312[@]}" scripts/test_check_evidence_engine.py', text)


if __name__ == "__main__":
    unittest.main()
