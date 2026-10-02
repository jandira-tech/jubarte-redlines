#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""gemini-extension.json: the shape Gemini CLI reads, in step with Cargo.toml.

The manifest keys follow the Gemini CLI extension reference
(geminicli.com/docs/extensions/reference). Standard library only.
"""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "gemini-extension.json"


def cargo_version() -> str:
    m = re.search(r'^version = "(\d+\.\d+\.\d+)"$', (ROOT / "Cargo.toml").read_text(), re.M)
    assert m, "Cargo.toml has no [package] version"
    return m.group(1)


class GeminiExtension(unittest.TestCase):
    def setUp(self) -> None:
        self.data = json.loads(MANIFEST.read_text())

    def test_name_is_a_valid_extension_directory_name(self) -> None:
        self.assertRegex(self.data["name"], r"^[a-z0-9-]+$")
        self.assertEqual(self.data["name"], "jubarte-redlines")

    def test_version_matches_cargo(self) -> None:
        self.assertEqual(self.data["version"], cargo_version())

    def test_description_is_present(self) -> None:
        self.assertTrue(self.data["description"].strip())

    def test_every_server_is_a_command_with_list_args(self) -> None:
        servers = self.data["mcpServers"]
        self.assertTrue(servers)
        for name, server in servers.items():
            with self.subTest(server=name):
                self.assertIsInstance(server["command"], str)
                self.assertIsInstance(server["args"], list)
                self.assertTrue(all(isinstance(a, str) for a in server["args"]))
                # The reference forbids `trust` in an extension's servers.
                self.assertNotIn("trust", server)

    def test_the_server_is_jubarte_mcp_contained_to_the_workspace(self) -> None:
        args = self.data["mcpServers"]["jubarte"]["args"]
        self.assertIn("jubarte-mcp", args)
        self.assertEqual(args[args.index("--root") + 1], "${workspacePath}")
        self.assertIn("jubarte-redlines[mcp]", args)

    def test_context_file_and_skill_exist(self) -> None:
        self.assertTrue((ROOT / self.data["contextFileName"]).is_file())
        self.assertTrue((ROOT / "skills" / "jubarte-documents" / "SKILL.md").is_file())


if __name__ == "__main__":
    unittest.main()
