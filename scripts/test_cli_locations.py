#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/cli_locations.py: what the catalogue does when the binary hangs.

  python3 scripts/test_cli_locations.py
"""

from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import cli_locations  # noqa: E402


class Timeout(unittest.TestCase):
    def test_a_timed_out_probe_stops_the_run_instead_of_writing_into_the_catalogue(self):
        # pi review of #461 (6b): a timeout used to become the probe's "stderr",
        # so docs/locations.md held "timed out after 120s" on a loaded machine.
        def hang(args, **_):
            raise subprocess.TimeoutExpired(args, 120)

        with mock.patch.object(cli_locations.subprocess, "run", hang):
            with self.assertRaisesRegex(RuntimeError, r"timed out after \d+s: .*jubarte read x\.docx"):
                cli_locations.run(Path("target/debug/jubarte"), ["read", "x.docx"], Path("."))


if __name__ == "__main__":
    unittest.main()
