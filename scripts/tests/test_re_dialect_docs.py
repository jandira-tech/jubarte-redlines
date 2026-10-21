#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Published dialect guides include the breaking migration rules."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]


class DialectDocsTests(unittest.TestCase):
    def test_published_guides_explain_attribution_and_literal_braces(self):
        for path in ["README.crates.md", "examples/cli/visualize/VISUALIZE.md"]:
            with self.subTest(path=path):
                text = (ROOT / path).read_text()
                for example in ["{r3 @BO re r1 @AL}", "¶{r7 @AL}", r"\{r1 @AL}"]:
                    self.assertTrue(example in text, f"{path}: missing {example}")
                self.assertTrue("different inline dates" in text.lower(), f"{path}: missing date rule")

    def test_unreleased_breaking_table_covers_the_six_rules(self):
        text = (ROOT / "CHANGELOG.md").read_text().split("## [Unreleased]", 1)[1]
        text = text.split("\n## [", 1)[0]
        self.assertTrue("### Breaking" in text, "missing Unreleased / Breaking")
        for name in ["Attribution tag", "Parent chain", "Merged ids", "Substitution pairing", "Inserted paragraph mark", "Literal escape"]:
            self.assertTrue(f"| {name} |" in text, f"missing migration row: {name}")


if __name__ == "__main__":
    unittest.main()
