#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: Apache-2.0

"""Tests of scripts/check_pypi_metadata.py."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import check_pypi_metadata as check

COMPLETE = """[project]
name = "x"
description = "what it is"
readme = "README.md"
requires-python = ">=3.10"
license = "Apache-2.0"
authors = [{ name = "A", email = "a@example.org" }]
maintainers = [{ name = "A", email = "a@example.org" }]
keywords = ["a", "b", "c"]
classifiers = [
  "Development Status :: 4 - Beta",
  "Intended Audience :: Developers",
  "Operating System :: OS Independent",
  "Programming Language :: Python :: 3.12",
  "Topic :: Office/Business",
]
[project.urls]
Homepage = "https://example.org"
Documentation = "https://example.org/docs"
Repository = "https://example.org/repo"
Changelog = "https://example.org/changes"
Issues = "https://example.org/issues"
"""


class Metadata(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="pypimeta-"))
        (self.tmp / "README.md").write_text("# x\n")

    def write(self, text: str) -> Path:
        path = self.tmp / "pyproject.toml"
        path.write_text(text)
        return path

    def test_a_complete_project_has_no_problem(self) -> None:
        self.assertEqual(check.problems(self.write(COMPLETE)), [])

    def test_each_missing_piece_is_named(self) -> None:
        cases = {
            'description = "what it is"\n': "`description`",
            'maintainers = [{ name = "A", email = "a@example.org" }]\n': "`maintainers`",
            '  "Operating System :: OS Independent",\n': "Operating System",
            '  "Programming Language :: Python :: 3.12",\n': "Programming Language :: Python :: 3.",
            'Changelog = "https://example.org/changes"\n': "`Changelog`",
        }
        for line, named in cases.items():
            self.assertIn(line, COMPLETE)
            found = check.problems(self.write(COMPLETE.replace(line, "")))
            self.assertEqual(len(found), 1, (line, found))
            self.assertIn(named, found[0])

    def test_an_author_without_an_email_is_a_problem(self) -> None:
        text = COMPLETE.replace('authors = [{ name = "A", email = "a@example.org" }]', 'authors = [{ name = "A" }]')
        self.assertEqual(len(check.problems(self.write(text))), 1)

    def test_a_readme_that_is_no_file_is_a_problem(self) -> None:
        (self.tmp / "README.md").unlink()
        self.assertIn("is no file", check.problems(self.write(COMPLETE))[0])

    def test_a_link_that_is_not_https_is_a_problem(self) -> None:
        text = COMPLETE.replace('Issues = "https://example.org/issues"', 'Issues = "example.org/issues"')
        self.assertIn("`Issues`", check.problems(self.write(text))[0])

    def test_the_command_fails_and_names_the_file(self) -> None:
        path = self.write(COMPLETE.replace('license = "Apache-2.0"\n', ""))
        run = subprocess.run(
            [sys.executable, str(HERE / "check_pypi_metadata.py"), str(path)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(run.returncode, 1)
        self.assertIn(str(path), run.stderr)
        self.assertIn("`license`", run.stderr)

    def test_both_published_distributions_are_complete(self) -> None:
        for name in check.PROJECTS:
            self.assertEqual(check.problems(check.ROOT / name), [], name)

    def test_the_release_script_runs_it_before_the_dry_runs(self) -> None:
        text = (HERE / "release.sh").read_text()
        dry_runs = text[text.index('say "7. Publish dry-runs"') : text.index('say "8. Release commit')]
        self.assertIn("python3 scripts/check_pypi_metadata.py", dry_runs)


if __name__ == "__main__":
    unittest.main()
