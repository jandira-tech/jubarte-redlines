#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Contract of scripts/repo_move.py: what the rename rewrite touches.

Each case builds a throwaway git repository, so the real tree is never
rewritten.
"""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import repo_move


def repo(files: dict[str, str]) -> Path:
    root = Path(tempfile.mkdtemp(prefix="repo_move_"))
    for path, text in files.items():
        (root / path).parent.mkdir(parents=True, exist_ok=True)
        (root / path).write_text(text, encoding="utf-8")
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    subprocess.run(["git", "add", "-A"], cwd=root, check=True)
    return root


class Rewrite(unittest.TestCase):
    def test_qualified_name_moves(self) -> None:
        self.assertEqual(
            repo_move.rewrite("https://github.com/jandira-tech/jubarte-redlines/issues"),
            "https://github.com/jandira-tech/jubarte/issues",
        )

    def test_package_names_stay(self) -> None:
        for text in (
            'jubarte-redlines = { version = "0.19.0" }',
            "pip install jubarte-redlines",
            "npx jubarte-redlines compare a.docx b.docx",
            "--remap-path-prefix=$repo=jubarte-redlines",
        ):
            self.assertEqual(repo_move.rewrite(text), text)

    def test_named_forms_move(self) -> None:
        self.assertEqual(
            repo_move.rewrite('const REPO_NAME: &str = "jubarte-redlines";'),
            'const REPO_NAME: &str = "jubarte";',
        )
        self.assertEqual(
            repo_move.rewrite("owner jandira-tech, repository jubarte-redlines, workflow release.yml"),
            "owner jandira-tech, repository jubarte, workflow release.yml",
        )

    def test_idempotent(self) -> None:
        once = repo_move.rewrite("see jandira-tech/jubarte-redlines and repository jubarte-redlines,")
        self.assertEqual(repo_move.rewrite(once), once)


class Plan(unittest.TestCase):
    OLD_URL = "https://github.com/jandira-tech/jubarte-redlines"

    def test_frozen_generated_and_live_files(self) -> None:
        root = repo(
            {
                "Cargo.toml": f'repository = "{self.OLD_URL}"\n',
                "docs/superpowers/plan.md": f"{self.OLD_URL}\n",
                "README.crates.md": f"<!-- {repo_move.GENERATED_STAMP} from README.md -->\n{self.OLD_URL}\n",
                "notes.md": "nothing here\n",
            }
        )
        changes, regenerate = repo_move.plan(root)
        self.assertEqual(sorted(changes), ["Cargo.toml"])
        self.assertEqual(regenerate, ["README.crates.md"])

    def test_check_apply_check(self) -> None:
        root = repo({"Cargo.toml": f'repository = "{self.OLD_URL}"\n'})
        args = ["--root", str(root)]
        self.assertEqual(repo_move.main(["--check", *args]), 1)
        self.assertEqual(repo_move.main(args), 0)  # the dry run writes nothing
        self.assertIn("jubarte-redlines", (root / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(repo_move.main(["--apply", *args]), 0)
        self.assertEqual(
            (root / "Cargo.toml").read_text(encoding="utf-8"),
            'repository = "https://github.com/jandira-tech/jubarte"\n',
        )
        self.assertEqual(repo_move.main(["--check", *args]), 0)


if __name__ == "__main__":
    unittest.main()
