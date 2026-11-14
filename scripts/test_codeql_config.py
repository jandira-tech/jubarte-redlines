#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""The CodeQL config ignores generated wasm-bindgen glue and nothing else
under jubarte-wasm/npm. package.json and the READMEs there are hand-written.

Run with PyYAML: `uv run --with pyyaml python3 scripts/test_codeql_config.py`.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / ".github/codeql/codeql-config.yml"

GENERATED = (
    "jubarte-wasm/npm/node/jubarte_wasm.js",
    "jubarte-wasm/npm/node/jubarte_wasm.d.ts",
    "jubarte-wasm/npm/node/jubarte_wasm_bg.wasm",
    "jubarte-wasm/npm/node/jubarte_wasm_bg.wasm.d.ts",
    "jubarte-wasm/npm/node-slim/jubarte_wasm.js",
    "jubarte-wasm/npm/web/jubarte_wasm.js",
    "jubarte-wasm/npm/web-slim/jubarte_wasm.d.ts",
)

HAND_WRITTEN = (
    "jubarte-wasm/npm/package.json",
    "jubarte-wasm/npm/README.md",
    "jubarte-wasm/npm/README.fragment.md",
    "jubarte-wasm/npm/node/package.json",
    "jubarte-wasm/npm/LICENSE",
    "jubarte-wasm/npm/ENGINE_COMMIT.txt",
)

# One directory level under npm/, the four names build-npm.sh copies.
EXPECTED = (
    "jubarte-wasm/npm/*/jubarte_wasm.js",
    "jubarte-wasm/npm/*/jubarte_wasm.d.ts",
    "jubarte-wasm/npm/*/jubarte_wasm_bg.wasm",
    "jubarte-wasm/npm/*/jubarte_wasm_bg.wasm.d.ts",
)


def codeql_match(path: str, pattern: str) -> bool:
    """CodeQL globs: ``*`` stays in one path segment and ``**`` crosses them."""
    parts: list[str] = []
    i = 0
    while i < len(pattern):
        if pattern.startswith("**", i):
            parts.append(".*")
            i += 2
        elif pattern[i] == "*":
            parts.append("[^/]*")
            i += 1
        else:
            parts.append(re.escape(pattern[i]))
            i += 1
    return re.fullmatch("".join(parts), path) is not None


def patterns_of(text: str) -> list[str]:
    """The ignore list of a config CodeQL will accept.

    ``yaml.safe_load`` rejects a document that does not parse. Anything
    other than a name plus a list of path strings is refused too: a query
    suite key, or a ``paths-ignore`` that is not a list, would change the
    scan or make CodeQL reject the file.
    """
    loaded = yaml.safe_load(text)
    if not isinstance(loaded, dict):
        raise AssertionError(f"CodeQL config is {type(loaded).__name__}, not a mapping")
    if set(loaded) != {"name", "paths-ignore"}:
        raise AssertionError(f"unexpected CodeQL config keys: {sorted(map(str, loaded))}")
    name = loaded["name"]
    if not isinstance(name, str) or not name.strip():
        raise AssertionError(f"CodeQL config name must be a non-empty string, got {name!r}")
    patterns = loaded["paths-ignore"]
    if not isinstance(patterns, list) or not all(isinstance(item, str) and item for item in patterns):
        raise AssertionError(f"paths-ignore must be a list of non-empty strings, got {patterns!r}")
    return patterns


class CodeqlConfig(unittest.TestCase):
    def test_generated_glue_is_ignored_and_hand_written_files_are_not(self) -> None:
        patterns = patterns_of(CONFIG.read_text())
        self.assertEqual(patterns, list(EXPECTED))
        for path in GENERATED:
            self.assertTrue(
                any(codeql_match(path, pattern) for pattern in patterns),
                f"{path} is not ignored by {patterns}",
            )
        for path in HAND_WRITTEN:
            self.assertFalse(
                any(codeql_match(path, pattern) for pattern in patterns),
                f"{path} is ignored by {patterns}",
            )
        nested = "jubarte-wasm/npm/node/extra/jubarte_wasm.js"
        self.assertFalse(
            any(codeql_match(nested, pattern) for pattern in patterns),
            "a star must not cross a directory",
        )

    def test_a_config_that_changes_the_scan_is_refused(self) -> None:
        extra_key = "name: jubarte\npaths-ignore:\n  - a.js\nqueries:\n  - uses: security-extended\n"
        with self.assertRaises(AssertionError):
            patterns_of(extra_key)
        not_a_list = "name: jubarte\npaths-ignore: jubarte-wasm/npm/**\n"
        with self.assertRaises(AssertionError):
            patterns_of(not_a_list)
        with self.assertRaises(yaml.YAMLError):
            patterns_of("name: jubarte\npaths-ignore: [\n")


if __name__ == "__main__":
    unittest.main()
