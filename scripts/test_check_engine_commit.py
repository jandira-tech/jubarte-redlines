#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/check_engine_commit.sh refuses an npm build its stamp cannot vouch for.

Each case builds a throwaway repository: an engine source, a docs file, and
the jubarte-wasm/npm/ENGINE_COMMIT.txt stamp that build-npm.sh writes. The
checker must accept a build whose stamped commit is an ancestor of the
checked revision with the same engine sources, and refuse every other case.
"""

from __future__ import annotations

import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
CHECKER = HERE / "check_engine_commit.sh"
STAMP = Path("jubarte-wasm/npm/ENGINE_COMMIT.txt")


class CheckEngineCommit(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="check_engine_commit_"))
        (self.tmp / "scripts").mkdir()
        shutil.copy(CHECKER, self.tmp / "scripts" / CHECKER.name)
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "test")
        self.git("config", "commit.gpgsign", "false")
        self.write("src/lib.rs", "pub fn engine() {}\n")
        self.write("docs/notes.md", "notes\n")
        self.engine = self.commit("engine")

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    def git(self, *args: str) -> str:
        return subprocess.run(
            ["git", "-C", str(self.tmp), *args],
            check=True, capture_output=True, text=True,
        ).stdout.strip()

    def write(self, rel: str, text: str) -> None:
        path = self.tmp / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def commit(self, message: str) -> str:
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)
        return self.git("rev-parse", "HEAD")

    def stamp(self, rev: str) -> str:
        """Commit the artifacts as build-npm.sh leaves them: stamped with rev."""
        self.write(str(STAMP), rev + "\n")
        return self.commit("build(npm): artifacts")

    def check(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["bash", str(self.tmp / "scripts" / CHECKER.name), *args],
            cwd=self.tmp, capture_output=True, text=True, timeout=60,
        )

    def test_a_build_stamped_with_its_own_engine_passes(self) -> None:
        self.stamp(self.engine)
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_docs_change_after_the_build_passes(self) -> None:
        self.stamp(self.engine)
        self.write("docs/notes.md", "more notes\n")
        self.commit("docs")
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_an_engine_change_after_the_build_is_refused(self) -> None:
        self.stamp(self.engine)
        self.write("src/lib.rs", "pub fn engine() { changed() }\n")
        self.commit("engine change")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("src/lib.rs", r.stderr)
        self.assertIn(self.engine[:12], r.stderr)

    def test_every_engine_input_is_watched(self) -> None:
        # Every input to the wasm build: the crate, its lockfile, the files
        # it embeds (include_bytes!/include_str!), and the binding crate with
        # the RUSTFLAGS in its .cargo/config.toml.
        for rel in [
            "Cargo.toml", "Cargo.lock", "assets/fonts/a.ttf",
            "keys/release-signing.pub", "schemas/edit-plan-v2.schema.json",
            "jubarte-wasm/src/lib.rs", "jubarte-wasm/Cargo.toml",
            "jubarte-wasm/Cargo.lock", "jubarte-wasm/.cargo/config.toml",
        ]:
            with self.subTest(rel=rel):
                base = self.git("rev-parse", "HEAD")
                self.stamp(base)
                self.write(rel, f"changed {rel}\n")
                self.commit(f"change {rel}")
                r = self.check()
                self.assertEqual(r.returncode, 1, r.stdout)
                self.assertIn(rel, r.stderr)

    def test_the_lock_the_build_resolved_ships_with_the_artifacts(self) -> None:
        # build-npm.sh re-resolves jubarte-wasm/Cargo.lock after its clean-tree
        # check, and release.sh step 8 commits that lock with the artifacts:
        # the lock the artifacts were built with is the artifacts commit's.
        self.write("jubarte-wasm/Cargo.lock", "old\n")
        base = self.commit("lock")
        self.write("jubarte-wasm/Cargo.lock", "re-resolved\n")
        self.stamp(base)
        r = self.check()
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_lock_change_after_the_artifacts_commit_is_refused(self) -> None:
        self.write("jubarte-wasm/Cargo.lock", "old\n")
        base = self.commit("lock")
        self.write("jubarte-wasm/Cargo.lock", "re-resolved\n")
        self.stamp(base)
        self.write("jubarte-wasm/Cargo.lock", "later\n")
        self.commit("lock bump")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("jubarte-wasm/Cargo.lock", r.stderr)

    def test_a_stamp_from_another_branch_is_refused(self) -> None:
        self.git("checkout", "-q", "-b", "side")
        self.write("src/side.rs", "pub fn side() {}\n")
        side = self.commit("side")
        self.git("checkout", "-q", "main")
        self.stamp(side)
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("not an ancestor", r.stderr)

    def test_a_missing_stamp_is_refused(self) -> None:
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(str(STAMP), r.stderr)

    def test_a_stamp_naming_no_commit_is_refused(self) -> None:
        self.stamp("0123456789abcdef0123456789abcdef01234567")
        r = self.check()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("not a commit", r.stderr)

    def test_the_stamp_is_read_from_the_checked_revision(self) -> None:
        # A release checks the tagged commit, not whatever the working tree
        # holds: an uncommitted restamp must not vouch for the tag.
        self.stamp(self.engine)
        self.write("src/lib.rs", "pub fn engine() { changed() }\n")
        tagged = self.commit("engine change")
        self.write(str(STAMP), tagged + "\n")
        r = self.check(tagged)
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("src/lib.rs", r.stderr)

    def test_an_unknown_revision_is_a_usage_error(self) -> None:
        self.stamp(self.engine)
        r = self.check("no-such-rev")
        self.assertEqual(r.returncode, 2, r.stdout)
        self.assertIn("no-such-rev", r.stderr)


if __name__ == "__main__":
    unittest.main()
