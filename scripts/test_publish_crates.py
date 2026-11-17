# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Tests for scripts/publish_crates.sh (plan T6.2): the per-crate order, the
resume over crates already live, the release-note gate, the pending probe
and the dry run.

A fake `cargo` logs its arguments and writes the packaged jubarte crate; a
local HTTP server stands in for crates.io's API: 200 for a crate version
that is live, 404 for one that is not, 503 for one set as unreachable.
Plain Python 3, stdlib only.
"""

import http.server
import io
import os
import re
import subprocess
import tarfile
import tempfile
import threading
import time
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
if [ -n "$FAKE_EXISTS" ] && [ "$*" = "publish -p $FAKE_EXISTS --locked" ]; then
  echo "error: crate $FAKE_EXISTS@$FAKE_VER already exists on crates.io index" >&2
  exit 101
fi
if [ -n "$FAKE_FAIL_ON" ] && [ "$*" = "$FAKE_FAIL_ON" ]; then
  echo "error: something else broke" >&2
  exit 101
fi
exit 0
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


class FakeCratesIo(http.server.BaseHTTPRequestHandler):
    live: set[str] = set()
    down: set[str] = set()
    stall: set[str] = set()
    known: set[str] = set()
    agents: list[str] = []

    def do_GET(self):
        FakeCratesIo.agents.append(self.headers.get("User-Agent", ""))
        crate = re.fullmatch(r"/api/v1/crates/([\w-]+)", self.path)
        if crate:  # the crate itself, any version
            name = crate.group(1)
            self.send_response(200 if name in self.live | self.known else 404)
            self.end_headers()
            self.wfile.write(b"{}")
            return
        m = re.fullmatch(r"/api/v1/crates/([\w-]+)/([\w.+-]+)", self.path)
        name = m.group(1) if m else ""
        if name in self.stall:
            time.sleep(5)
        code = 503 if name in self.down else 200 if name in self.live else 404
        self.send_response(code)
        self.end_headers()
        self.wfile.write(b"{}")

    def log_message(self, *args):
        pass


class Run:
    def __init__(self, ver="0.19.0", live=(), down=(), note=True, fail_on="", exists="", stall=(),
                 known=ORDER):
        self.tmp = tempfile.TemporaryDirectory()
        t = Path(self.tmp.name)
        self.repo = t / "repo"
        self.repo.mkdir()
        subprocess.run(["git", "init", "-q", str(self.repo)], check=True)
        FakeCratesIo.live, FakeCratesIo.down, FakeCratesIo.agents = set(live), set(down), []
        FakeCratesIo.stall, FakeCratesIo.known = set(stall), set(known)
        self.server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), FakeCratesIo)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        self.log = t / "cargo.log"
        self.log.write_text("")
        crate = t / "crate.tar.gz"
        crate.write_bytes(crate_with(ver, note))
        cargo = t / "cargo"
        cargo.write_text(FAKE_CARGO)
        cargo.chmod(0o755)
        self.env = {**os.environ, "CARGO": str(cargo),
                    "CRATES_IO_API": f"http://127.0.0.1:{self.server.server_port}/api/v1",
                    "CARGO_LOG": str(self.log), "FAKE_CRATE": str(crate), "FAKE_VER": ver,
                    "FAKE_FAIL_ON": fail_on, "FAKE_EXISTS": exists,
                    "CRATES_IO_MAX_TIME": "1"}
        self.ver = ver

    def __call__(self, *args):
        r = subprocess.run(["bash", str(SCRIPT), self.ver, *args], cwd=self.repo, env=self.env,
                           capture_output=True, text=True, timeout=60)
        return r, self.log.read_text().splitlines()

    def close(self):
        self.server.shutdown()
        self.server.server_close()
        self.tmp.cleanup()


def published(calls):
    return [c.split()[2] for c in calls if c.startswith("publish -p ")]


class PublishCrates(unittest.TestCase):
    def run_script(self, *args, **kw):
        run = Run(**kw)
        self.addCleanup(run.close)
        return run, *run(*args)

    def test_every_crate_is_published_in_dependency_order(self):
        _, r, calls = self.run_script()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(published(calls), ORDER)
        self.assertIn("9 published, 0 already live", r.stdout)

    def test_the_preflight_packages_the_whole_workspace(self):
        # A single-crate package resolves its exact-pinned workspace deps
        # on crates.io, which do not exist before their first release.
        _, r, calls = self.run_script()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(calls[0], "package --workspace --locked --no-verify")

    def test_every_probe_names_itself_to_crates_io(self):
        # crates.io answers 403 to a request without a User-Agent.
        self.run_script()
        self.assertTrue(FakeCratesIo.agents)
        self.assertTrue(all(a.startswith("jubarte-release") for a in FakeCratesIo.agents))

    def test_a_rerun_resumes_past_the_crates_already_live(self):
        _, r, calls = self.run_script(live=ORDER[:3])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(published(calls), ORDER[3:])
        for name in ORDER[:3]:
            self.assertIn(f"{name} 0.19.0 is already on crates.io: skipped", r.stdout)

    def test_a_crate_the_api_has_not_caught_up_on_is_skipped_not_fatal(self):
        # The API lags the index right after a publish: the probe says 404,
        # cargo says the version exists. The run goes on to the next crate.
        _, r, calls = self.run_script(exists="jubarte-dom")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(published(calls), ORDER)
        self.assertIn("jubarte-dom 0.19.0 is already on the crates.io index: skipped", r.stdout)
        self.assertIn("8 published, 1 already live", r.stdout)

    def test_an_unreachable_crates_io_stops_the_run_instead_of_republishing(self):
        _, r, calls = self.run_script(down=["jubarte-opc"])
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("jubarte-opc", r.stderr)
        self.assertIn("503", r.stderr)
        self.assertEqual(published(calls), ORDER[:1])

    def test_a_stalled_probe_times_out_and_stops_the_run(self):
        started = time.monotonic()
        _, r, calls = self.run_script(stall=["jubarte-opc"])
        self.assertLess(time.monotonic() - started, 4)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("jubarte-opc", r.stderr)
        self.assertEqual(published(calls), ORDER[:1])

    def test_a_release_fully_live_publishes_nothing(self):
        _, r, calls = self.run_script(live=ORDER)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertFalse([c for c in calls if c.startswith("publish")])
        self.assertIn("0 published, 9 already live", r.stdout)

    def test_a_crate_that_fails_stops_the_run_before_its_dependents(self):
        _, r, calls = self.run_script(fail_on="publish -p jubarte-render --locked")
        self.assertNotEqual(r.returncode, 0)
        self.assertEqual(published(calls), ORDER[:4])

    def test_a_package_without_its_release_note_uploads_nothing(self):
        _, r, calls = self.run_script(note=False)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("release-notes", r.stderr)
        self.assertFalse([c for c in calls if c.startswith("publish")])

    def test_pending_counts_the_crates_crates_io_lacks_and_publishes_nothing(self):
        _, r, calls = self.run_script("--pending", live=ORDER[:5])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(r.stdout.strip().splitlines()[-1], "pending=4")
        self.assertEqual(calls, [])

    def test_pending_counts_the_crates_crates_io_has_never_seen(self):
        # Trusted publishing works only for a crate that exists: release.yml
        # leaves a first release to the token holder.
        _, r, calls = self.run_script("--pending", known=ORDER[:2])
        self.assertEqual(r.returncode, 0, r.stderr)
        lines = r.stdout.strip().splitlines()
        self.assertIn("new=7", lines)
        self.assertEqual(lines[-1], "pending=9")
        self.assertIn("jubarte-render has never been published", r.stdout)
        self.assertEqual(calls, [])

    def test_pending_reports_no_new_crate_when_all_exist(self):
        _, r, _ = self.run_script("--pending", live=ORDER[:4])
        self.assertIn("new=0", r.stdout.splitlines())

    def test_pending_is_zero_when_every_crate_is_live(self):
        _, r, _ = self.run_script("--pending", live=ORDER)
        self.assertEqual(r.stdout.strip().splitlines()[-1], "pending=0")

    def test_the_dry_run_verifies_the_whole_workspace_in_a_fresh_target(self):
        # cargo reuses an unpacked crate of its publish-local registry when
        # the registry path is unchanged; a fresh target dir is a fresh path.
        _, r, calls = self.run_script("--dry-run")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(len(calls), 1)
        m = re.fullmatch(r"publish --workspace --dry-run --locked --allow-dirty --target-dir (\S+)", calls[0])
        self.assertTrue(m, calls[0])
        self.assertNotIn("/target", m.group(1).removeprefix(str(REPO)))

    def test_the_dry_run_leaves_no_target_behind(self):
        _, r, calls = self.run_script("--dry-run")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertFalse(Path(calls[0].split("--target-dir ")[1]).exists())

    def test_two_dry_runs_never_share_a_target(self):
        run = Run()
        self.addCleanup(run.close)
        run("--dry-run")
        _, calls = run("--dry-run")
        dirs = [c.split("--target-dir ")[1] for c in calls]
        self.assertEqual(len(set(dirs)), 2)

    def test_unknown_arguments_are_refused(self):
        _, r, calls = self.run_script("--as-prerelease")
        self.assertNotEqual(r.returncode, 0)
        self.assertEqual(calls, [])


class OrderMatchesTheWorkspace(unittest.TestCase):
    def test_each_crate_comes_after_every_workspace_crate_it_or_its_tests_need(self):
        # cargo resolves dev-dependencies when it verifies a package, so a
        # dev-dependency must be on crates.io first too.
        script = SCRIPT.read_text()
        listed = re.search(r"CRATES=\(([^)]*)\)", script).group(1).split()
        self.assertEqual(listed, ORDER)
        manifests = {"jubarte": REPO / "Cargo.toml"}
        manifests |= {p.parent.name: p for p in (REPO / "crates").glob("*/Cargo.toml")}
        self.assertEqual(sorted(manifests), sorted(ORDER), "a workspace crate is missing from CRATES")
        for name, path in manifests.items():
            doc = tomllib.loads(path.read_text())
            tables = [doc.get("dependencies", {}), doc.get("dev-dependencies", {}),
                      doc.get("build-dependencies", {})]
            for target in doc.get("target", {}).values():
                tables += [target.get(k, {}) for k in ("dependencies", "dev-dependencies", "build-dependencies")]
            deps = set().union(*tables)
            for dep in deps & set(ORDER):
                if dep != name:
                    self.assertLess(listed.index(dep), listed.index(name), f"{name} needs {dep} first")


if __name__ == "__main__":
    unittest.main()
