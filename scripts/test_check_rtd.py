#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Tests for check_rtd.py: the ReadTheDocs books match the workspace.

Run with PyYAML: `uv run --with pyyaml python3 scripts/test_check_rtd.py`.
"""

from __future__ import annotations

import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import check_rtd

ROOT = Path(__file__).resolve().parent.parent


class Repository(unittest.TestCase):
    def test_the_checkout_passes(self) -> None:
        self.assertEqual(check_rtd.check(ROOT), [])


class Broken(unittest.TestCase):
    """A copy of the parts the check reads, broken one way per test."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp)
        shutil.copy(ROOT / "Cargo.toml", self.tmp / "Cargo.toml")
        for member in (ROOT / "crates").iterdir():
            if (member / "Cargo.toml").is_file():
                (self.tmp / "crates" / member.name).mkdir(parents=True)
                shutil.copy(member / "Cargo.toml", self.tmp / "crates" / member.name / "Cargo.toml")
        shutil.copytree(ROOT / "docs" / "rtd", self.tmp / "docs" / "rtd")
        # The books include crate examples and generated pages; copy the
        # files they name so only the breakage below fails.
        for page in (ROOT / "docs" / "rtd").glob("*/**/*.md"):
            for spec in check_rtd.INCLUDE.findall(page.read_text(encoding="utf-8")):
                source = (page.parent / spec.partition(":")[0]).resolve()
                copy = self.tmp / source.relative_to(ROOT)
                copy.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy(source, copy)
        self.assertEqual(check_rtd.check(self.tmp), [])

    def problems(self) -> str:
        return "\n".join(check_rtd.check(self.tmp))

    def test_a_member_without_a_project_fails(self) -> None:
        shutil.rmtree(self.tmp / "docs/rtd/jubarte-opc")
        self.assertIn("workspace member 'jubarte-opc' has no ReadTheDocs project", self.problems())

    def test_a_project_that_is_no_member_fails(self) -> None:
        shutil.copytree(self.tmp / "docs/rtd/jubarte-opc", self.tmp / "docs/rtd/jubarte-gone")
        self.assertIn("docs/rtd/jubarte-gone: not a workspace member", self.problems())

    def test_a_config_building_another_project_fails(self) -> None:
        config = self.tmp / "docs/rtd/jubarte-dom/.readthedocs.yaml"
        config.write_text(config.read_text().replace("build.sh jubarte-dom", "build.sh jubarte-opc"))
        self.assertIn("jubarte-dom/.readthedocs.yaml: build.commands must be", self.problems())

    def test_a_config_that_is_not_version_2_fails(self) -> None:
        config = self.tmp / "docs/rtd/jubarte/.readthedocs.yaml"
        config.write_text(config.read_text().replace("version: 2", "version: 1"))
        self.assertIn("jubarte/.readthedocs.yaml: version must be 2", self.problems())

    def test_a_summary_link_without_its_page_fails(self) -> None:
        (self.tmp / "docs/rtd/jubarte-render/src/fonts.md").unlink()
        self.assertIn("link 'fonts.md' has no file", self.problems())

    def test_a_page_missing_from_the_summary_fails(self) -> None:
        (self.tmp / "docs/rtd/jubarte-render/src/stray.md").write_text("# Stray\n")
        self.assertIn("jubarte-render/src/stray.md: not in SUMMARY.md", self.problems())

    def test_a_missing_include_fails(self) -> None:
        (self.tmp / "crates/jubarte-dom/examples/parse_xml.rs").unlink()
        self.assertIn("include '../../../../crates/jubarte-dom/examples/parse_xml.rs' does not exist", self.problems())

    def test_a_missing_anchor_fails(self) -> None:
        rust = self.tmp / "docs/rust.md"
        rust.write_text(rust.read_text().replace("ANCHOR_END: cli-rust", "ANCHOR_END: gone"))
        self.assertIn("has no ANCHOR/ANCHOR_END 'cli-rust'", self.problems())

    def test_a_page_link_without_its_page_fails(self) -> None:
        page = self.tmp / "docs/rtd/jubarte-doc/src/validate.md"
        page.write_text(page.read_text().replace("](audit.md)", "](audit2.md)"))
        self.assertIn("link 'audit2.md' has no file", self.problems())

    def test_an_anchor_end_before_its_anchor_fails(self) -> None:
        rust = self.tmp / "docs/rust.md"
        text = rust.read_text()
        start, end = "ANCHOR: cli-rust", "ANCHOR_END: cli-rust"
        self.assertEqual((text.count(start), text.count(end)), (1, 1))
        swapped = text.replace(start, "\0").replace(end, start).replace("\0", end)
        rust.write_text(swapped)
        self.assertIn("has no ANCHOR/ANCHOR_END 'cli-rust'", self.problems())

    def test_a_hardcoded_repository_url_fails(self) -> None:
        owner = check_rtd.repository_owner(self.tmp)
        assert owner is not None
        book = self.tmp / "docs/rtd/jubarte-cli/book.toml"
        book.write_text(book.read_text() + f'git-repository-url = "https://{owner}x"\n')
        problems = self.problems()
        self.assertIn("git-repository-url is set by docs/rtd/build.sh", problems)
        self.assertIn(f"names {owner}", problems)

    def test_main_reports_and_exits_1(self) -> None:
        (self.tmp / "docs/rtd/jubarte-render/src/fonts.md").unlink()
        self.assertEqual(check_rtd.main(["--root", str(self.tmp)]), 1)


if __name__ == "__main__":
    unittest.main()
