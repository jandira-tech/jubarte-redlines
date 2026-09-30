#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Argument contract of scripts/release.sh.

Each case runs a copy of release.sh inside a throwaway git repository on a
non-main branch, so a run that gets past argument validation stops at the
"release from main only" preflight: nothing is fetched, pushed or published.
"""

from __future__ import annotations

import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
RELEASE_SH = HERE / "release.sh"
DOCS_FLAG = "--how-readme-and-other-docs-were-updated"

SUMMARIES = [
    "--changelog-summary", "changelog note",
    "--crates-summary", "crates note",
    "--npm-summary", "npm note",
    "--pypi-summary", "pypi note",
    "--github-summary", "github note",
]


class ReleaseArgs(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="release_sh_"))
        (self.tmp / "scripts").mkdir()
        shutil.copy(RELEASE_SH, self.tmp / "scripts" / "release.sh")
        git = ["git", "-C", str(self.tmp)]
        subprocess.run([*git, "init", "-q", "-b", "not-main"], check=True)

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    def run_release(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["bash", str(self.tmp / "scripts" / "release.sh"), *args],
            capture_output=True, text=True, timeout=60,
        )

    def test_docs_flag_is_required(self) -> None:
        r = self.run_release("0.10.0", *SUMMARIES)
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertIn(f"missing required summaries: {DOCS_FLAG}", r.stderr)

    def test_docs_flag_rejects_blank_value(self) -> None:
        r = self.run_release("0.10.0", *SUMMARIES, DOCS_FLAG, "  \n ")
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertIn(DOCS_FLAG, r.stderr)

    def test_docs_flag_needs_a_value(self) -> None:
        r = self.run_release("0.10.0", *SUMMARIES, DOCS_FLAG)
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertIn(f"missing value for {DOCS_FLAG}", r.stderr)

    def test_all_six_accepted_reaches_preflight(self) -> None:
        r = self.run_release(
            "0.10.0", *SUMMARIES, DOCS_FLAG, "README install section rewritten"
        )
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("release from main only", r.stderr)

    def test_help_documents_docs_flag(self) -> None:
        r = self.run_release("--help")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn(DOCS_FLAG, r.stderr)


class ResumeDryRuns(unittest.TestCase):
    """A resumed release must not die in step 5 on a registry that already
    holds the version (npm refuses even a dry run over a published version)."""

    def step6(self) -> str:
        text = RELEASE_SH.read_text()
        start = text.index('say "6. Publish dry-runs"')
        return text[start:text.index('say "7.', start)]

    def test_npm_dry_run_skips_a_published_version(self) -> None:
        self.assertRegex(
            self.step6(), r"if npm_has; then[^\n]*\n(?:[^\n]*\n)*?else\n[^\n]*npm publish --dry-run"
        )

    def test_cargo_dry_run_skips_a_published_version(self) -> None:
        self.assertRegex(
            self.step6(), r"if crates_has; then[^\n]*\n(?:[^\n]*\n)*?else\n[^\n]*cargo publish --dry-run"
        )


class ApiDocDrift(unittest.TestCase):
    """Step 5 is a required release review: the rendered rustdoc is opened for
    the releaser, and a machine-readable API snapshot lands in docs/api/ so the
    next release can diff the surface."""

    def step5(self) -> str:
        text = RELEASE_SH.read_text()
        start = text.index('say "5. API docs')
        return text[start:text.index('say "6.', start)]

    def test_opens_rendered_docs_for_review(self) -> None:
        self.assertIn(
            "cargo doc --no-deps --document-private-items --open", self.step5()
        )

    def test_snapshots_machine_readable_api(self) -> None:
        self.assertIn("scripts/api_snapshot.py", self.step5())

    def test_diffs_against_previous_release_snapshot(self) -> None:
        self.assertIn("docs/api/jubarte-", self.step5())
        self.assertIn("diff -u", self.step5())

    def test_release_commit_adds_docs_api(self) -> None:
        text = RELEASE_SH.read_text()
        step7 = text[text.index('say "7. Release commit'):text.index('say "8.')]
        self.assertIn("docs/api", step7.split("git commit")[0])


class RegistryProbes(unittest.TestCase):
    """crates.io answers 403 to a request without a User-Agent, so a bare
    curl probe calls every published version missing."""

    def test_crates_probe_sends_a_user_agent(self) -> None:
        line = next(
            l for l in RELEASE_SH.read_text().splitlines() if l.startswith("crates_has()")
        )
        self.assertRegex(line, r"curl [^|]*(-A|--user-agent) ")

    def test_publish_purges_finder_litter_first(self) -> None:
        text = RELEASE_SH.read_text()
        step8 = text[text.index('say "8. crates.io"'):text.index('say "9.')]
        self.assertIn(".DS_Store", step8.split("cargo publish")[0])


def step(n: int) -> str:
    """Text of release.sh step `n`, up to the next numbered step."""
    text = RELEASE_SH.read_text()
    start = text.index(f'say "{n}. ')
    nxt = [text.find(f'say "{n + 1}. ', start), text.find('say "POINT OF NO RETURN"', start)]
    ends = [i for i in nxt if i > start]
    return text[start:min(ends) if ends else len(text)]


class Lessons0101(unittest.TestCase):
    """What stopped or dirtied the v0.10.1 release, one guard each."""

    def test_lockfiles_resolve_the_bumped_path_dependency(self) -> None:
        # `cargo metadata --no-deps` resolves nothing, so the tagged commit
        # kept jubarte-redlines 0.10.0 in three sub-workspace locks.
        s1 = step(1)
        self.assertNotIn("--no-deps", s1)
        self.assertIn("cargo update --offline", s1)
        for d in ("jubarte-wasm", "jubarte-rust-inproc", "jubarte-app/src-tauri"):
            self.assertIn(d, s1)

    def test_the_desktop_app_follows_the_engine_version(self) -> None:
        # tests/release_metadata.rs failed on main: jubarte-app stayed 0.10.0.
        s1 = step(1)
        for f in (
            "jubarte-app/package.json",
            "jubarte-app/src-tauri/tauri.conf.json",
            "jubarte-app/src-tauri/Cargo.toml",
            "jubarte-app/src/index.html",
        ):
            self.assertIn(f, s1)
        self.assertIn("jubarte-app/CHANGELOG.md", step(2))

    def test_a_resume_keeps_the_wasm_build_it_already_committed(self) -> None:
        # A resumed run rebuilt the package with a later ENGINE_COMMIT than
        # the one npm already shipped.
        s7 = step(7)
        guard = s7.index("regenerate npm artifacts for v$VER")
        self.assertLess(guard, s7.index("jubarte-wasm/build-npm.sh"))

    def test_the_artifacts_commit_takes_the_wasm_lock(self) -> None:
        s7 = step(7)
        add = next(l for l in s7.splitlines() if "git add jubarte-wasm/npm" in l)
        self.assertIn("jubarte-wasm/Cargo.lock", add)

    def test_npm_publish_takes_a_one_time_password(self) -> None:
        # npm answered EOTP to the non-interactive publish.
        s9 = step(9)
        self.assertIn("NPM_OTP", s9)
        self.assertIn("--otp", s9)

    def test_pypi_takes_the_workflow_wheels_when_no_release_exists(self) -> None:
        # The Windows binary failed, release.yml skipped the GitHub release,
        # and the wheels existed only as workflow artifacts.
        self.assertIn("gh run download", step(10))

    def test_a_skipped_github_release_is_created_from_the_artifacts(self) -> None:
        self.assertIn("gh release create", step(10))

    def test_the_api_snapshot_is_byte_stable(self) -> None:
        # gzip stamped the write time, so every rerun dirtied docs/api/.
        snap = (HERE / "api_snapshot.py").read_text()
        self.assertIn("mtime=0", snap)

    def test_windows_checks_out_long_paths(self) -> None:
        wf = (HERE.parent / ".github/workflows/release.yml").read_text()
        self.assertIn("core.longpaths", wf)


if __name__ == "__main__":
    unittest.main()
