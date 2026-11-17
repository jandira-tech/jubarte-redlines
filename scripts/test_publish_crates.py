# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Tests for scripts/publish_crates.sh (plan T6.2): the per-crate order, the
resume over crates already live, the release-note gate, and the dry runs.

A fake `cargo` logs its arguments and writes the packaged jubarte crate; a
file:// root stands in for crates.io, a crate being live when
<root>/crates/<name>/<version> exists. Plain Python 3, stdlib only.
"""

import io
import os
import re
import subprocess
import tarfile
import tempfile
import tomllib
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SCRIPT = REPO / "scripts/publish_crates.sh"
ORDER = ["jubarte-dom", "jubarte-opc", "jubarte-redlines", "jubarte-render", "jubarte-doc",
         "jubarte-markdown", "jubarte-edit", "jubarte-cli", "jubarte"]

FAKE_CARGO = r"""#!/usr/bin/env bash
echo "$*" >> "$CARGO_LOG"
if [ "$1" = package ]; then
  mkdir -p target/package
  cp "$FAKE_CRATE" "target/package/jubarte-$FAKE_VER.crate"
fi
if [ -n "$FAKE_FAIL_ON" ] && [ "$*" = "$FAKE_FAIL_ON" ]; then exit 1; fi
exit 0
"""

MANIFEST = """[workspace.package]
version = "0.19.0" # the release script bumps this and the "=" pins together

[workspace.dependencies]
jubarte-dom = { path = "crates/jubarte-dom", version = "=0.19.0" }
jubarte-cli = { path = "crates/jubarte-cli", version = "=0.19.0", default-features = false }
"""


def crate_with(ver: str, note: bool) -> bytes:
    text = '[package]\nname = "jubarte"\n'
    if note:
        text += f'\n[package.metadata.release-notes]\n"{ver}" = "the summary"\n'
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tar:
        data = text.encode()
        info = tarfile.TarInfo(f"jubarte-{ver}/Cargo.toml")
        info.size = len(data)
        tar.addfile(info, io.BytesIO(data))
    return buf.getvalue()


class Run:
    def __init__(self, ver="0.19.0", live=(), note=True, fail_on=""):
        self.tmp = tempfile.TemporaryDirectory()
        t = Path(self.tmp.name)
        self.repo = t / "repo"
        self.repo.mkdir()
        subprocess.run(["git", "init", "-q", str(self.repo)], check=True)
        (self.repo / "Cargo.toml").write_text(MANIFEST)
        self.api = t / "api"
        for name in live:
            (self.api / "crates" / name).mkdir(parents=True)
            (self.api / "crates" / name / ver).write_text("{}")
        self.log = t / "cargo.log"
        self.log.write_text("")
        crate = t / "crate.tar.gz"
        crate.write_bytes(crate_with(ver, note))
        cargo = t / "cargo"
        cargo.write_text(FAKE_CARGO)
        cargo.chmod(0o755)
        self.env = {**os.environ, "CARGO": str(cargo), "CRATES_IO_API": f"file://{self.api}",
                    "CARGO_LOG": str(self.log), "FAKE_CRATE": str(crate), "FAKE_VER": ver,
                    "FAKE_FAIL_ON": fail_on}
        self.ver = ver

    def __call__(self, *args):
        r = subprocess.run(["bash", str(SCRIPT), self.ver, *args], cwd=self.repo, env=self.env,
                           capture_output=True, text=True)
        return r, self.log.read_text().splitlines()

    def close(self):
        self.tmp.cleanup()


class PublishCrates(unittest.TestCase):
    def run_script(self, *args, **kw):
        run = Run(**kw)
        self.addCleanup(run.close)
        return run, *run(*args)

    def test_every_crate_is_published_in_dependency_order(self):
        _, r, calls = self.run_script()
        self.assertEqual(r.returncode, 0, r.stderr)
        published = [c.split()[2] for c in calls if c.startswith("publish -p ")]
        self.assertEqual(published, ORDER)
        self.assertIn("9 published, 0 already live", r.stdout)

    def test_a_rerun_resumes_past_the_crates_already_live(self):
        _, r, calls = self.run_script(live=ORDER[:3])
        self.assertEqual(r.returncode, 0, r.stderr)
        published = [c.split()[2] for c in calls if c.startswith("publish -p ")]
        self.assertEqual(published, ORDER[3:])
        for name in ORDER[:3]:
            self.assertIn(f"{name} 0.19.0 is already on crates.io: skipped", r.stdout)

    def test_a_release_fully_live_publishes_nothing(self):
        _, r, calls = self.run_script(live=ORDER)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertFalse([c for c in calls if c.startswith("publish")])
        self.assertIn("0 published, 9 already live", r.stdout)

    def test_a_crate_that_fails_stops_the_run_before_its_dependents(self):
        _, r, calls = self.run_script(fail_on="publish -p jubarte-render --locked")
        self.assertNotEqual(r.returncode, 0)
        published = [c.split()[2] for c in calls if c.startswith("publish -p ")]
        self.assertEqual(published, ORDER[:4])

    def test_a_package_without_its_release_note_uploads_nothing(self):
        _, r, calls = self.run_script(note=False)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("release-notes", r.stderr)
        self.assertFalse([c for c in calls if c.startswith("publish")])

    def test_the_dry_run_verifies_the_whole_workspace(self):
        _, r, calls = self.run_script("--dry-run")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(calls, ["publish --workspace --dry-run --locked --allow-dirty"])

    def test_the_prerelease_dry_run_moves_the_version_and_every_pin(self):
        run, r, calls = self.run_script("--dry-run", "--as-prerelease")
        self.assertEqual(r.returncode, 0, r.stderr)
        text = (run.repo / "Cargo.toml").read_text()
        self.assertIn('version = "0.19.0-dryrun" # the release script bumps', text)
        self.assertEqual(text.count('"=0.19.0-dryrun"'), 2)
        self.assertNotIn('"=0.19.0"', text)
        self.assertEqual(calls, ["update --workspace --quiet", "publish --workspace --dry-run --allow-dirty"])

    def test_the_prerelease_refuses_a_version_the_manifest_is_not_at(self):
        run = Run(ver="0.20.0")
        self.addCleanup(run.close)
        r, calls = run("--dry-run", "--as-prerelease")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("is at 0.19.0, not 0.20.0", r.stderr)
        self.assertEqual(calls, [])

    def test_as_prerelease_needs_dry_run(self):
        _, r, calls = self.run_script("--as-prerelease")
        self.assertNotEqual(r.returncode, 0)
        self.assertEqual(calls, [])


class OrderMatchesTheWorkspace(unittest.TestCase):
    def test_each_crate_comes_after_every_workspace_crate_it_depends_on(self):
        script = SCRIPT.read_text()
        listed = re.search(r"CRATES=\(([^)]*)\)", script).group(1).split()
        self.assertEqual(listed, ORDER)
        manifests = {"jubarte": REPO / "Cargo.toml"}
        manifests |= {p.parent.name: p for p in (REPO / "crates").glob("*/Cargo.toml")}
        self.assertEqual(sorted(manifests), sorted(ORDER), "a workspace crate is missing from CRATES")
        for name, path in manifests.items():
            doc = tomllib.loads(path.read_text())
            deps = set(doc.get("dependencies", {}))  # dev-dependencies are not published edges
            for dep in deps & set(ORDER):
                if dep != name:
                    self.assertLess(listed.index(dep), listed.index(name), f"{name} needs {dep} first")


if __name__ == "__main__":
    unittest.main()
