#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Tests for table_styles_asset.py (review of #1524)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import table_styles_asset


class Sources(unittest.TestCase):
    def test_with_no_argument_the_repository_source_is_read(self) -> None:
        self.assertEqual(table_styles_asset.sources([]), [table_styles_asset.SOURCE])

    def test_extra_sources_add_to_the_repository_source(self) -> None:
        # The docstring's promise: extra sources add families, never replace
        # the Grid and List Tables read from the repository's own source.
        self.assertEqual(
            table_styles_asset.sources(["old.docx"]),
            [table_styles_asset.SOURCE, Path("old.docx")],
        )


if __name__ == "__main__":
    unittest.main()
