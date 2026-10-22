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
import tempfile
import threading
import time
import unittest
import unittest.mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import cli_id_sources
import cli_locations


class Timeout(unittest.TestCase):
    def test_an_expired_control_id_is_a_long_id(self):
        self.assertIsNotNone(cli_locations.LONG_ID.search("control body:sdt:3"))

    def test_an_html_character_reference_is_no_retired_id(self):
        # The view writes a kept space as `&#32;`; that is no `#32`.
        self.assertIsNone(cli_locations.RETIRED_ID.search("note&#32;joined"))
        self.assertIsNotNone(cli_locations.RETIRED_ID.search("see #32 here"))

    def test_a_timed_out_probe_stops_the_run_instead_of_writing_into_the_catalogue(self):
        # pi review of #461 (6b): a timeout used to become the probe's "stderr",
        # so docs/locations.md held "timed out after 120s" on a loaded machine.
        def hang(args, **_):
            raise subprocess.TimeoutExpired(args, 120)

        with unittest.mock.patch.object(cli_locations.subprocess, "run", hang):
            with self.assertRaisesRegex(RuntimeError, r"timed out after \d+s: .*jubarte read x\.docx"):
                cli_locations.run(Path("target/debug/jubarte"), ["read", "x.docx"], Path("."))

    def test_a_timeout_cancels_the_probes_still_queued(self):
        # sourcery review of #471: pool.map submits every probe, and leaving
        # the executor waited for all of them, each up to the timeout.
        calls = []
        lock = threading.Lock()

        def hang(*_):
            with lock:
                calls.append(1)
            time.sleep(0.05)
            raise RuntimeError("timed out after 120s: jubarte read x.docx")

        pairs = [(f"r{n}", "x.docx") for n in range(40)]
        with tempfile.TemporaryDirectory() as scratch, unittest.mock.patch.object(
            cli_locations, "verdict", hang
        ), unittest.mock.patch.object(cli_locations, "change_ids", lambda *_: []):
            with self.assertRaisesRegex(RuntimeError, "timed out"):
                cli_locations.probe_table(Path("jubarte"), pairs, Path(scratch), "t")
        jobs = 4 * len(pairs)
        self.assertLess(len(calls), jobs // 4, f"{len(calls)} of {jobs} probes ran after the first timeout")


class Where(unittest.TestCase):
    def test_a_second_function_of_one_name_is_named_once_by_its_module(self):
        # `refused` lives in bin/jubarte/check.rs and bin/jubarte.rs; the
        # second one read is keyed by its module, which `where` printed twice
        # (`bin::jubarte::bin::jubarte::refused`).
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            (root / "src/bin/jubarte").mkdir(parents=True)
            (root / "src/bin/jubarte/check.rs").write_text("fn refused() {\n}\n")
            (root / "src/bin/jubarte.rs").write_text("fn refused() {\n}\n")
            names = sorted(cli_id_sources.where(f) for f in cli_id_sources.functions(root).values())
        self.assertEqual(names, ["`bin::jubarte::check::refused`", "`bin::jubarte::refused`"])


class LinkCalls(unittest.TestCase):
    def test_a_bare_call_links_to_the_namesake_in_its_own_or_an_enclosing_module(self):
        # `edit/for_edit.rs` calls `settle(` and means `edit::settle`; a
        # private `settle` in the sibling `edit/hint.rs` is not in its
        # scope. Two free functions of one name dropped the call entirely.
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            (root / "src/edit").mkdir(parents=True)
            (root / "src/edit.rs").write_text("fn settle() {\n}\n")
            (root / "src/edit/hint.rs").write_text("fn settle() {\n}\n")
            (root / "src/edit/for_edit.rs").write_text("fn for_edit() {\n    settle();\n}\n")
            fns = cli_id_sources.functions(root)
            cli_id_sources.link_calls(fns)
        caller = next(f for f in fns.values() if f.name == "for_edit")
        called = sorted(cli_id_sources.where(fns[key]) for key in caller.calls)
        self.assertEqual(called, ["`edit::settle`"])


if __name__ == "__main__":
    unittest.main()
