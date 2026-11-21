#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: Apache-2.0

"""jubarte-wasm/npm/prepublish-check.mjs refuses an incomplete npm package.

The built files are not tracked, and npm drops a missing `files` entry
without a word, so the check is what stands between a partial build
(`build-npm.sh --only node`) and a published package that lacks targets.
Each case builds a package folder in a temporary directory.
"""

from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

NPM = Path(__file__).resolve().parent.parent / "jubarte-wasm" / "npm"
TARGETS = ["node", "web", "node-slim", "web-slim", "node-redlines", "web-redlines", "node-pdf", "web-pdf"]
BUILT = ["jubarte_wasm.js", "jubarte_wasm.d.ts", "jubarte_wasm_bg.wasm", "jubarte_wasm_bg.wasm.d.ts"]


@unittest.skipUnless(shutil.which("node"), "node is not installed")
class PrepublishCheck(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="npm_prepublish_"))
        shutil.copy(NPM / "prepublish-check.mjs", self.tmp)
        shutil.copy(NPM / "package.json", self.tmp)
        for name in ("LICENSE", "NOTICE", "LICENSE-OFL-1.1.txt", "README.md"):
            (self.tmp / name).write_text("text\n")
        (self.tmp / "ENGINE_COMMIT.txt").write_text("0123456789abcdef0123456789abcdef01234567\n")
        for target in TARGETS:
            (self.tmp / target).mkdir()
            for name in [*BUILT, "package.json"]:
                (self.tmp / target / name).write_text("{}\n")

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    def check(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["node", "prepublish-check.mjs"], cwd=self.tmp, capture_output=True, text=True, timeout=60
        )

    def test_a_complete_package_passes(self) -> None:
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_the_tracked_package_json_runs_it_before_a_publish(self) -> None:
        scripts = json.loads((NPM / "package.json").read_text()).get("scripts", {})
        self.assertEqual(scripts.get("prepublishOnly"), "node prepublish-check.mjs")

    def test_a_partial_build_has_no_stamp_and_is_refused(self) -> None:
        # what `build-npm.sh --only node` leaves
        (self.tmp / "ENGINE_COMMIT.txt").unlink()
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("ENGINE_COMMIT.txt is missing", r.stderr)

    def test_every_built_file_of_every_target_is_required(self) -> None:
        for target in TARGETS:
            for name in BUILT:
                with self.subTest(target=target, name=name):
                    path = self.tmp / target / name
                    path.unlink()
                    r = self.check()
                    self.assertEqual(r.returncode, 1, r.stdout)
                    self.assertIn(f"{target}/{name} is missing", r.stderr)
                    path.write_text("{}\n")

    def test_a_target_dropped_from_files_is_refused(self) -> None:
        # npm would pack without it and say nothing
        manifest = json.loads((self.tmp / "package.json").read_text())
        manifest["files"].remove("web-pdf/")
        (self.tmp / "package.json").write_text(json.dumps(manifest))
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn('does not list web-pdf/', r.stderr)

    def test_the_license_files_are_required(self) -> None:
        for name in ("LICENSE", "NOTICE", "LICENSE-OFL-1.1.txt"):
            with self.subTest(name=name):
                (self.tmp / name).unlink()
                r = self.check()
                self.assertEqual(r.returncode, 1, r.stdout)
                self.assertIn(f"{name} is missing", r.stderr)
                (self.tmp / name).write_text("text\n")

    def test_a_stamp_that_is_not_a_commit_hash_is_refused(self) -> None:
        (self.tmp / "ENGINE_COMMIT.txt").write_text("main\n")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("full commit hash", r.stderr)


if __name__ == "__main__":
    unittest.main()
