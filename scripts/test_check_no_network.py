#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Tests for check_no_network.py (#2018)."""

from __future__ import annotations

import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import check_no_network


def run(*args: str) -> tuple[int, str, str]:
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = check_no_network.main(list(args))
    return code, out.getvalue(), err.getvalue()


CLEAN_TREE = """jubarte-redlines v0.17.0 (/x)
clap v4.5.0
mimalloc v0.1.43
zip v8.0.0
"""


class Crates(unittest.TestCase):
    def test_a_network_crate_is_found_at_any_depth_and_prefix(self) -> None:
        tree = "jubarte-redlines v0.17.0\n├── ureq v2.10.0\n│   └── rustls v0.23.1\n"
        self.assertEqual(check_no_network.network_crates(tree), ["rustls", "ureq"])

    def test_a_name_that_only_contains_a_network_crate_is_not_one(self) -> None:
        tree = "hyperlink-rs v1.0.0\nmy-ureq-shim v0.1.0\n"
        self.assertEqual(check_no_network.network_crates(tree), [])


class Sources(unittest.TestCase):
    def tree(self, files: dict[str, str]) -> Path:
        root = Path(tempfile.mkdtemp()) / "src"
        for name, text in files.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")
        return root

    def test_the_self_update_code_may_use_the_network(self) -> None:
        src = self.tree(
            {
                "update.rs": "use self_update::backends::github;\n",
                "update/zipsign.rs": "let s = std::net::TcpStream::connect(a);\n",
                "lib.rs": "fn hyperlink() {} // std::net is named in a comment only\n",
            }
        )
        self.assertEqual(check_no_network.network_uses(src), [])

    def test_a_network_api_elsewhere_is_named_with_its_line(self) -> None:
        src = self.tree(
            {
                "lib.rs": "fn a() {}\nuse std::net::TcpStream;\n",
                "bin/jubarte.rs": "let r = ureq::get(url).call();\n",
            }
        )
        self.assertEqual(
            check_no_network.network_uses(src),
            [
                "src/bin/jubarte.rs:1: let r = ureq::get(url).call();",
                "src/lib.rs:2: use std::net::TcpStream;",
            ],
        )

    def test_a_function_named_after_self_update_is_not_a_use(self) -> None:
        src = self.tree({"bin/jubarte.rs": "fn run_self_update() {}\n"})
        self.assertEqual(check_no_network.network_uses(src), [])


class Main(unittest.TestCase):
    def write(self, text: str) -> Path:
        path = Path(tempfile.mkdtemp()) / "tree.txt"
        path.write_text(text, encoding="utf-8")
        return path

    def test_a_clean_tree_and_sources_pass(self) -> None:
        src = Sources.tree(Sources(), {"lib.rs": "fn a() {}\n"})
        code, out, _ = run("--tree", str(self.write(CLEAN_TREE)), "--src", str(src))
        self.assertEqual(code, 0, out)

    def test_a_network_crate_fails(self) -> None:
        src = Sources.tree(Sources(), {"lib.rs": "fn a() {}\n"})
        code, out, _ = run("--tree", str(self.write(CLEAN_TREE + "reqwest v0.12.0\n")), "--src", str(src))
        self.assertEqual(code, 1)
        self.assertIn("network crate without self-update: reqwest", out)

    def test_usage_errors_exit_2(self) -> None:
        self.assertEqual(run("--bogus")[0], 2)
        self.assertEqual(run("--src", "/nonexistent/dir")[0], 2)
        self.assertEqual(run("--tree", "/nonexistent/tree.txt")[0], 2)

    def test_this_repository_passes(self) -> None:
        code, out, err = run()
        self.assertEqual(code, 0, out + err)


if __name__ == "__main__":
    unittest.main()
