#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Tests for check_npm_paths.py (#1480)."""

from __future__ import annotations

import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import check_npm_paths


def run(*args: str) -> tuple[int, str, str]:
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = check_npm_paths.main(list(args))
    return code, out.getvalue(), err.getvalue()


class HomePaths(unittest.TestCase):
    def test_a_macos_home_is_found_by_the_folder_below_it(self) -> None:
        data = b"\x00panicked at /Users/alice/.cargo/registry/src/x.rs\x00"
        self.assertEqual(check_npm_paths.home_paths(data), {"/Users/*/.cargo": 1})

    def test_linux_and_windows_homes_are_found(self) -> None:
        data = b"/home/runner/work/a.rs C:\\Users\\bob\\src\\b.rs C:\\\\Users\\\\bob\\\\src"
        self.assertEqual(
            check_npm_paths.home_paths(data),
            {"/home/*/work": 1, "C:\\Users\\*\\src": 2},
        )

    def test_a_remapped_path_is_not_a_home(self) -> None:
        data = b"~/.cargo/registry/src/x.rs jubarte-redlines/src/lib.rs /rustc/abc/library"
        self.assertEqual(check_npm_paths.home_paths(data), {})

    def test_the_word_users_alone_is_not_a_home(self) -> None:
        self.assertEqual(check_npm_paths.home_paths(b"/Users/ and Users/x"), {})


class Main(unittest.TestCase):
    def setUp(self) -> None:
        self.dir = Path(tempfile.mkdtemp())

    def write(self, name: str, data: bytes) -> Path:
        path = self.dir / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def test_clean_artifacts_pass(self) -> None:
        self.write("node/a.wasm", b"\x00asm~/.cargo/registry\x00")
        self.write("node/a.js", b"export {}")
        code, out, _ = run(str(self.dir))
        self.assertEqual(code, 0)
        self.assertIn("2 artifacts, no home path", out)

    def test_a_home_path_fails_without_naming_the_user(self) -> None:
        self.write("web/a.wasm", b"/Users/alice/.cargo/x /Users/alice/.cargo/y /Users/alice/T/z")
        code, _, err = run(str(self.dir))
        self.assertEqual(code, 1)
        self.assertIn("3 home paths: /Users/*/.cargo (2), /Users/*/T (1)", err)
        self.assertIn("1 of 1 artifacts", err)
        self.assertNotIn("alice", err)
        # The file shows relative, so no absolute path, home or not.
        self.assertNotIn("check_npm_paths: /", err)

    def test_only_build_artifacts_are_searched_in_a_directory(self) -> None:
        self.write("README.md", b"built in /Users/alice/src")
        self.write("node/a.wasm", b"clean")
        self.assertEqual(run(str(self.dir))[0], 0)

    def test_a_file_given_is_searched_whatever_its_suffix(self) -> None:
        path = self.write("notes.txt", b"/home/carol/x")
        self.assertEqual(run(str(path))[0], 1)

    def test_a_missing_path_or_a_flag_is_a_usage_error(self) -> None:
        self.assertEqual(run(str(self.dir / "absent"))[0], 2)
        self.assertEqual(run("--help")[0], 2)

    def test_no_artifact_at_all_is_a_usage_error(self) -> None:
        self.write("README.md", b"text")
        self.assertEqual(run(str(self.dir))[0], 2)


if __name__ == "__main__":
    unittest.main()
