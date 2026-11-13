#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""The CodeQL config ignores generated wasm-bindgen glue and nothing else
under jubarte-wasm/npm. package.json and the READMEs there are hand-written.
"""

from __future__ import annotations

import fnmatch
import unittest
from pathlib import Path

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


def ignored_patterns(text: str) -> list[str]:
    lines = text.splitlines()
    start = lines.index("paths-ignore:")
    patterns = []
    for line in lines[start + 1 :]:
        if not line.startswith("  - "):
            break
        patterns.append(line.removeprefix("  - ").strip().strip("'\""))
    return patterns


class CodeqlConfig(unittest.TestCase):
    def test_generated_glue_is_ignored_and_hand_written_files_are_not(self) -> None:
        text = CONFIG.read_text()
        patterns = ignored_patterns(text)
        self.assertTrue(patterns, "paths-ignore lists nothing")
        self.assertNotIn("jubarte-wasm/npm/**", patterns)
        for path in GENERATED:
            self.assertTrue(
                any(fnmatch.fnmatch(path, pattern) for pattern in patterns),
                f"{path} is not ignored by {patterns}",
            )
        for path in HAND_WRITTEN:
            self.assertFalse(
                any(fnmatch.fnmatch(path, pattern) for pattern in patterns),
                f"{path} is ignored by {patterns}",
            )


if __name__ == "__main__":
    unittest.main()
