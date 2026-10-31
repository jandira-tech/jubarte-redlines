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

import ast
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
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

# The first lines of every gh stub: an engine call names its repository
# (#1747), so a stray GH_REPO in the releaser's shell cannot redirect it.
GH_BOUND = """case " $* " in
  *" -R jandira-tech/jubarte-redlines "*) ;;
  *) echo "gh not bound to jandira-tech/jubarte-redlines: $*" >&2; exit 8 ;;
esac
"""


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

    def step7(self) -> str:
        text = RELEASE_SH.read_text()
        start = text.index('say "7. Publish dry-runs"')
        return text[start:text.index('say "8.', start)]

    def test_npm_dry_run_skips_a_published_version(self) -> None:
        self.assertRegex(
            self.step7(), r"if npm_has; then[^\n]*\n(?:[^\n]*\n)*?else\n[^\n]*npm publish --dry-run"
        )

    def test_cargo_dry_run_skips_a_published_version(self) -> None:
        self.assertRegex(
            self.step7(), r"if crates_has; then[^\n]*\n(?:[^\n]*\n)*?else\n[^\n]*cargo publish --dry-run"
        )


class ApiDocDrift(unittest.TestCase):
    """Step 5 is a required release review: the rendered rustdoc is opened for
    the releaser, and a machine-readable API snapshot lands in docs/api/ so the
    next release can diff the surface."""

    def step6(self) -> str:
        text = RELEASE_SH.read_text()
        start = text.index('say "6. API docs')
        return text[start:text.index('say "7.', start)]

    def test_opens_rendered_docs_for_review(self) -> None:
        self.assertIn(
            "cargo doc --no-deps --document-private-items --open", self.step6()
        )

    def test_snapshots_machine_readable_api(self) -> None:
        self.assertIn("scripts/api_snapshot.py", self.step6())

    def test_diffs_against_previous_release_snapshot(self) -> None:
        # The 0.11.2 review was a 700-line `diff -u` of the two listings,
        # blanket impls and crate-private items mixed into the public
        # surface. The drift report rebuilds both sides from the rustdoc
        # JSON, public surface first.
        s6 = self.step6()
        self.assertIn('docs/api/jubarte-$PREV_TAG.json.gz', s6)
        self.assertIn('python3 scripts/api_snapshot.py --drift "$PREV_TAG" "v$VER"', s6)
        self.assertNotIn("diff -u", s6)

    def test_the_private_docs_build_without_warnings(self) -> None:
        # The docs the releaser reads carried 12 rustdoc warnings in 0.11.2
        # (dead links, `<tab>` read as HTML): only the public build of step 5
        # denied them.
        self.assertIn(
            'RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items --open',
            self.step6(),
        )

    def test_the_gates_test_the_drift_report(self) -> None:
        self.assertIn('"${py312[@]}" scripts/test_api_snapshot.py', step(5))

    def test_release_commit_adds_docs_api(self) -> None:
        text = RELEASE_SH.read_text()
        step8 = text[text.index('say "8. Release commit'):text.index('say "9.')]
        self.assertIn("docs/api", step8.split("git commit")[0])


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
        step9 = text[text.index('say "9. crates.io"'):text.index('say "10.')]
        self.assertIn(".DS_Store", step9.split("cargo publish")[0])


def step(n: int) -> str:
    """Text of release.sh step `n`, up to the next numbered step."""
    text = RELEASE_SH.read_text()
    start = text.index(f'say "{n}. ')
    nxt = [text.find(f'say "{n + 1}. ', start), text.find('say "POINT OF NO RETURN"', start)]
    ends = [i for i in nxt if i > start]
    return text[start:min(ends) if ends else len(text)]


class WorkspaceFormatting(unittest.TestCase):
    def test_step5_formats_every_workspace(self) -> None:
        # Each binding has its own workspace: root cargo fmt cannot reach it.
        manifests = (
            "Cargo.toml", "jubarte-wasm/Cargo.toml",
            "jubarte-python/Cargo.toml", "jubarte-rust-inproc/Cargo.toml",
        )
        for manifest in manifests:
            with self.subTest(manifest=manifest):
                self.assertIn(f"cargo fmt --all --manifest-path {manifest} --check", step(5))


class NpmCli(unittest.TestCase):
    """`npx jubarte-redlines` ships beside jubarte-wasm, on its version."""

    def test_the_cli_package_follows_the_engine_version(self) -> None:
        s1 = step(1)
        self.assertIn('(cd jubarte-wasm/cli && npm pkg set "version=$VER" "dependencies.jubarte-wasm=^$VER"', s1)
        add = next(l for l in step(8).splitlines() if "jubarte-wasm/Cargo.lock jubarte-wasm/npm/package.json" in l)
        self.assertIn("jubarte-wasm/cli/package.json", add)

    def test_the_cli_is_dry_run_published_and_verified(self) -> None:
        self.assertRegex(step(7), r"if npm_cli_has; then[^\n]*\n(?:[^\n]*\n)*?else\n[^\n]*\(cd jubarte-wasm/cli && npm publish --dry-run")
        self.assertIn('check "npm        jubarte-redlines $VER" "eventually npm_cli_has"', step(12))

    def test_the_cli_publishes_after_the_wasm_it_depends_on(self) -> None:
        s10 = step(10)
        wasm = s10.index("(cd jubarte-wasm/npm && npm publish")
        cli = s10.index("(cd jubarte-wasm/cli && npm publish")
        self.assertLess(wasm, cli)
        self.assertIn("if npm_cli_has; then", s10)


class Lessons0101(unittest.TestCase):
    """What stopped or dirtied the v0.10.1 release, one guard each."""

    def test_lockfiles_resolve_the_bumped_path_dependency(self) -> None:
        # `cargo metadata --no-deps` resolves nothing, so the tagged commit
        # kept jubarte-redlines 0.10.0 in three sub-workspace locks.
        s1 = step(1)
        self.assertNotIn("--no-deps", s1)
        self.assertIn("cargo update --offline", s1)
        for d in ("jubarte-wasm", "jubarte-rust-inproc"):
            self.assertIn(d, s1)

    def test_the_desktop_app_is_released_by_its_own_repository(self) -> None:
        # tests/release_metadata.rs failed on main when jubarte-app stayed
        # 0.10.0. The app is its own repository now: this script touches none
        # of its files and runs its scripts/release-engine.sh instead.
        for n in (1, 2, 8):
            code = [line for line in step(n).splitlines() if not line.lstrip().startswith("#")]
            self.assertFalse([line for line in code if "jubarte-app/" in line], n)
        self.assertIn("app_release --preflight", step(0))
        self.assertIn("app_release", step(13))

    def test_a_resume_keeps_the_wasm_build_it_already_committed(self) -> None:
        # A resumed run rebuilt the package with a later ENGINE_COMMIT than
        # the one npm already shipped.
        s8 = step(8)
        guard = s8.index("regenerate npm artifacts for v$VER")
        self.assertLess(guard, s8.index("jubarte-wasm/build-npm.sh"))

    def test_step8_refuses_an_npm_build_the_tag_does_not_vouch_for(self) -> None:
        # A resumed run keeps the npm build it finds; nothing checked that its
        # ENGINE_COMMIT.txt still named the tagged engine (#1088). The check
        # runs on the tag after both the fresh and the resumed branch.
        s8 = step(8)
        call = s8.index('scripts/check_engine_commit.sh "$TAG^{commit}"')
        self.assertGreater(call, s8.rindex('git tag -a "$TAG"'))
        self.assertGreater(call, s8.index("npm artifacts rebuilt + smoke-tested"))
        self.assertLess(call, s8.index("check_now 8"))
        line = next(l for l in s8.splitlines() if "scripts/check_engine_commit.sh" in l)
        self.assertIn("|| die", line)

    def test_the_gates_test_the_engine_commit_check(self) -> None:
        text = RELEASE_SH.read_text()
        gates = text[text.index('say "5. Gates'):text.index('say "5. Gates — SKIPPED')]
        self.assertIn('"${py312[@]}" scripts/test_check_engine_commit.py', gates)
        row = next(l for l in text.splitlines() if l.startswith("8|auto|Release commit"))
        self.assertIn("scripts/check_engine_commit.sh v@V@", row)

    def test_the_artifacts_commit_takes_the_wasm_lock(self) -> None:
        s8 = step(8)
        add = next(l for l in s8.splitlines() if "git add jubarte-wasm/npm" in l)
        self.assertIn("jubarte-wasm/Cargo.lock", add)

    def test_the_artifacts_commit_regenerates_the_js_reference(self) -> None:
        # docs/javascript.md quotes jubarte-wasm/npm/node/jubarte_wasm.d.ts.
        # Step 5 ran before the rebuild, so v0.11.0 shipped a reference
        # generated from the 0.10.1 typings, missing 13 new functions.
        s8 = step(8)
        regen = s8.index("scripts/gen_wasm_api.py")
        self.assertLess(s8.index("jubarte-wasm/build-npm.sh"), regen)
        add = next(l for l in s8.splitlines() if "git add jubarte-wasm/npm" in l)
        self.assertIn("docs/javascript.md", add)

    def test_the_artifacts_commit_regenerates_the_npm_cli_reference(self) -> None:
        # docs/javascript.md's cli-npm block is the npm CLI's --help, run on
        # the wasm build. Step 6 ran it on the committed artifacts of an
        # older engine, so the 0.12.0 dry run rewrote the block back to help
        # text the release had already changed.
        s8 = step(8)
        regen = s8.index("--marker cli-npm")
        self.assertLess(s8.index("jubarte-wasm/build-npm.sh"), regen)
        self.assertLess(regen, s8.index("git add jubarte-wasm/npm"))

    def test_pypi_is_checked_on_the_project_listing(self) -> None:
        # PyPI's CDN keeps a 404 for /pypi/<name>/<version>/json once asked
        # before the upload; v0.11.0's verify failed on a release that was
        # live. The project listing is purged on upload.
        line = next(l for l in RELEASE_SH.read_text().splitlines() if l.startswith("pypi_has()"))
        self.assertNotIn("/$VER/json", line)
        self.assertIn("pypi.org/pypi/jubarte-redlines/json", line)

    def test_the_gates_build_the_public_docs_without_warnings(self) -> None:
        # The 0.11.0 docs carried 41 dead intra-doc links (rendered as bare
        # brackets on docs.rs) because nothing built the public docs with
        # warnings denied; step 5 builds them with private items, warnings on.
        s5 = step(5)
        self.assertIn('RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features', s5)

    def test_the_gates_keep_the_tracked_python_lockfile(self) -> None:
        # jubarte-python/uv.lock is tracked (f8fe3542). Deleting it after
        # pytest left the release tree dirty, and a resumed run then died in
        # preflight; whatever uv rewrote in it ships in the release commit.
        self.assertNotIn("rm -f jubarte-python/uv.lock", step(5))
        s8 = step(8)
        start = s8.index("git add Cargo.toml")
        add = s8[start : s8.index("git commit", start)]
        self.assertIn("jubarte-python/uv.lock", add)

    def test_the_release_commit_takes_the_gemini_manifest(self) -> None:
        # bump-version.mjs rewrites gemini-extension.json's version.
        s8 = step(8)
        start = s8.index("git add Cargo.toml")
        add = s8[start : s8.index("git commit", start)]
        self.assertIn("gemini-extension.json", add)

    def test_npm_publish_takes_a_one_time_password(self) -> None:
        # npm answered EOTP to the non-interactive publish.
        s10 = step(10)
        self.assertIn("NPM_OTP", s10)
        self.assertIn("--otp", s10)

    def test_pypi_takes_the_workflow_wheels_when_no_release_exists(self) -> None:
        # The Windows binary failed, release.yml skipped the GitHub release,
        # and the wheels existed only as workflow artifacts.
        self.assertIn("gh run download", step(11))

    def test_a_skipped_github_release_is_created_from_the_artifacts(self) -> None:
        self.assertIn("gh release create", step(11))

    def test_the_api_snapshot_is_byte_stable(self) -> None:
        # gzip stamped the write time, so every rerun dirtied docs/api/.
        snap = (HERE / "api_snapshot.py").read_text()
        self.assertIn("mtime=0", snap)

    def test_windows_checks_out_long_paths(self) -> None:
        wf = (HERE.parent / ".github/workflows/release.yml").read_text()
        self.assertIn("core.longpaths", wf)


class Repairs0112(unittest.TestCase):
    """The 0.11.2 release-machinery repairs: a resumed run must not ship a
    half-bumped tree (F4), a stale sdist must not be picked (F5), the two
    publish = false crates follow the engine (F6), a missing wheel set needs
    explicit consent (F18), and the evidence's sha256 columns are verified
    against the bench's real files when a bench checkout exists."""

    def test_step1_bumps_the_two_publish_false_crates(self) -> None:
        s1 = step(1)
        for f in ("jubarte-wasm/Cargo.toml", "jubarte-rust-inproc/Cargo.toml"):
            self.assertIn(f, s1)

    def test_step1_dies_on_a_half_bumped_tree(self) -> None:
        s1 = step(1)
        self.assertIn('grep -q "^version = \\"$VER\\"$"', s1)
        self.assertIn("half-bumped", s1)
        for f in ("jubarte-python/Cargo.toml", "jubarte-wasm/npm/package.json",
                  "gemini-extension.json", "README.md"):
            self.assertIn(f, s1)
        # the release commit carries the two crate manifests step 1 now bumps
        s8 = step(8)
        add = next(l for l in s8.splitlines() if "jubarte-rust-inproc/Cargo.lock" in l)
        self.assertIn("jubarte-rust-inproc/Cargo.toml", add)
        self.assertIn("jubarte-wasm/Cargo.toml", s8)

    def test_the_readme_pin_is_checked_only_where_the_readme_has_one(self) -> None:
        # The 0.11.0 README rewrite dropped the Socket badge; step 1 then died
        # on "README.md (Socket badge) is not on 0.11.2" with nothing to bump.
        text = RELEASE_SH.read_text()
        start = text.index("stale_readme_pin() {")
        fn = text[start:text.index("\n}\n", start) + 3]

        def stale(readme: str) -> bool:
            with tempfile.TemporaryDirectory() as d:
                (Path(d) / "README.md").write_text(readme)
                run = subprocess.run(["bash", "-c", "set -euo pipefail\nVER=0.11.2\n" + fn + "\nstale_readme_pin"],
                                     cwd=d, capture_output=True, text=True)
            return run.returncode == 0

        pin = "https://badge.socket.dev/cargo/package/jubarte-redlines/"
        self.assertFalse(stale("# jubarte\n\nno badge here\n"))
        self.assertFalse(stale(f"[![s]({pin}0.11.2)](x)\n"))
        self.assertTrue(stale(f"[![s]({pin}0.11.0)](x)\n"))
        self.assertTrue(stale(f"[![s]({pin}0.11.2)]({pin}0.11.0)\n"))
        self.assertIn('stale_readme_pin && half_bumped "README.md (Socket badge)"', step(1))

    def test_step7_wipes_the_stale_sdist_folder(self) -> None:
        s7 = step(7)
        wipe = s7.index("rm -rf target/release-check")
        self.assertLess(wipe, s7.index("uvx maturin sdist"))

    def test_step11_refuses_partial_wheels_without_consent(self) -> None:
        s11 = step(11)
        consent = s11.index('[ "$NO_WAIT" = 1 ]')
        self.assertIn("|| die", s11[consent:])
        self.assertIn("--no-wait", s11)
        self.assertIn("only the local-platform wheel + sdist will reach PyPI (--no-wait given)", s11)

    def test_a_run_that_lost_a_wheel_job_never_becomes_a_public_release(self) -> None:
        s11 = step(11)
        body = s11[s11.index("release_from_artifacts() {"):]
        body = body[:body.index("\n}\n")]
        check = body.index('scripts/check_release_artifacts.py dist/release --version "$VER"')
        self.assertIn("|| die", body[check:body.index("gh release create")])

    def test_step3_verifies_the_shas_against_the_bench_when_it_exists(self) -> None:
        s3 = step(3)
        self.assertIn("NEUROTIC_DOCX_BENCH", s3)
        self.assertIn('--bench-root "$BENCH_ROOT"', s3)
        self.assertIn("format-checked only", s3)  # the no-bench branch says what is not proven

    def test_nothing_the_release_commit_stages_is_ignored(self) -> None:
        # A "jubarte-app/" line in .gitignore once made `git add` refuse the
        # then-vendored app's version files, and step 8 would have stopped
        # before the release commit. Nothing it stages may be ignored, and
        # the app (untracked since) is no part of it.
        s8 = step(8)
        start = s8.index("git add Cargo.toml")
        staged = s8[start:s8.index("git commit", start)].replace("\\\n", " ").split()[2:]
        self.assertIn("jubarte-wasm/Cargo.toml", staged)
        self.assertFalse([f for f in staged if f.startswith("jubarte-app/")], staged)
        run = subprocess.run(["git", "check-ignore", "--no-index", *staged],
                             cwd=HERE.parent, capture_output=True, text=True)
        self.assertEqual(run.stdout, "", "the release commit stages paths .gitignore ignores")


class AppStep(unittest.TestCase):
    """The desktop app is its own repository (arthrod/jubarte-app), cloned
    untracked at jubarte-app/ or named by JUBARTE_APP_DIR. Its
    scripts/release-engine.sh is the app's whole part of a release (the site,
    its facts, its version files, the printed App Store and bench commands)
    and is tested there; here, only that this script runs it, and when."""

    def test_step_0_runs_the_app_preflight_before_anything_changes(self) -> None:
        text = RELEASE_SH.read_text()
        call = "app_release --preflight"
        self.assertLess(text.index('say "0. Preflight"'), text.index(call))
        self.assertLess(text.index(call), text.index('say "1. '))
        s0 = step(0)
        self.assertIn('APP_REPO=${JUBARTE_APP_DIR:-jubarte-app}', s0)
        self.assertIn('"$APP_REPO/scripts/release-engine.sh" "$VER" --engine-dir "$PWD"', s0)
        self.assertIn("git clone https://github.com/arthrod/jubarte-app", s0)

    def test_step_13_runs_it_after_verify_and_is_the_last_step(self) -> None:
        text = RELEASE_SH.read_text()
        self.assertLess(text.index('say "12. Verify'), text.index('say "13. App'))
        self.assertIn("app_release \\", step(13))
        self.assertNotIn('say "14. ', text)

    def test_nothing_here_still_calls_the_moved_scripts(self) -> None:
        text = RELEASE_SH.read_text()
        for gone in ("release_downstream.sh", "scripts/check_release_facts.py",
                     "scripts/check_site_live.py"):
            self.assertNotIn(gone, text.replace("jubarte-app/scripts/", ""), gone)
            self.assertFalse((HERE / Path(gone).name).exists(), gone)

    def test_the_app_is_not_tracked_here(self) -> None:
        tracked = subprocess.run(["git", "ls-files", "jubarte-app"], cwd=HERE.parent,
                                 capture_output=True, text=True, check=True).stdout
        self.assertEqual(tracked, "")
        self.assertIn("/jubarte-app/", (HERE.parent / ".gitignore").read_text().splitlines())


# stub cargo for the app gate: records the call, the directory it ran in
# and what its `../..` (the engine beside the app) holds; $CARGO_RC is its
# exit status. Its one output line is what a failed gate's log tail shows.
CARGO_APP_STUB = r"""#!/bin/bash
echo "cargo $*" >> "$CALLS"
pwd > "$CALLS.cwd"
ls ../.. > "$CALLS.layout"
echo "cargo stub: $*"
exit "${CARGO_RC:-0}"
"""


class AppGate(unittest.TestCase):
    """#1816: 0.14.0 was tagged and published before anything compiled the
    app against the released engine — the checklist row was a [you] reminder
    and the app's own CI was red. Before step 4 the release now runs the
    app's own `cargo test` in the layout the app consumes the engine in —
    the engine at HEAD with the app at jubarte-app/ inside it, so
    src-tauri's `path = "../.."` resolves to it — and a compile error or a
    failing test stops the release before the tag."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="appgate-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "bin").mkdir()
        (self.tmp / "bin" / "cargo").write_text(CARGO_APP_STUB)
        (self.tmp / "bin" / "cargo").chmod(0o755)
        # the gate's mktemp -d lands under the test's own root, so its
        # cleanup — not the ambient TMPDIR — decides what a run leaves
        (self.tmp / "tmp").mkdir()
        self.calls = self.tmp / "calls"
        self.engine = self.repo("engine", {
            "Cargo.toml": '[package]\nname = "jubarte-redlines"\nversion = "0.14.0"\n',
        })
        # the bump is uncommitted from step 1 to step 8: not in HEAD
        (self.engine / "uncommitted").write_text("the bump is not in HEAD\n")
        self.app = self.repo("app", {
            "src-tauri/Cargo.toml":
                '[dependencies]\njubarte = { package = "jubarte-redlines", path = "../.." }\n',
            # the app's lock on main still names the previous engine — its
            # own release pull request is what re-records it (the real lock
            # held app and engine 0.14.0 while both manifests said 0.16.0)
            "src-tauri/Cargo.lock":
                '# the app lock\n[[package]]\nname = "jubarte-redlines"\nversion = "0.13.0"\n',
        })
        self.script = (
            "set -euo pipefail\n"
            'step() { echo "  - $*"; }\ndie() { echo "ERROR: $*" >&2; exit 1; }\n'
            f"APP_REPO={self.app}\n"
            f'{shell_functions("app_test_gate")}\napp_test_gate\n'
        )

    def repo(self, name: str, files: dict[str, str]) -> Path:
        repo = self.tmp / name
        repo.mkdir()
        git = ["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t"]
        subprocess.run([*git, "init", "-q", "-b", "main"], check=True)
        for rel, text in files.items():
            path = repo / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        subprocess.run([*git, "add", "."], check=True)
        subprocess.run([*git, "commit", "-q", "-m", name], check=True)
        return repo

    def gate_env(self, **env: str) -> dict[str, str]:
        return dict(os.environ, PATH=f"{self.tmp / 'bin'}:{os.environ['PATH']}",
                    CALLS=str(self.calls), TMPDIR=str(self.tmp / "tmp"), **env)

    def gate(self, **env: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(["bash", "-c", self.script], cwd=self.engine,
                              env=self.gate_env(**env), capture_output=True,
                              text=True, timeout=60)

    def test_a_failing_cargo_test_stops_the_release_before_step_4(self) -> None:
        r = self.gate(CARGO_RC="1")
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("cargo test", r.stderr)
        # and its call site sits between steps 3 and 4, before the tag (8)
        text = RELEASE_SH.read_text()
        call = re.search(r"^\s+app_test_gate$", text, re.M)
        self.assertIsNotNone(call, "release.sh never calls app_test_gate")
        self.assertLess(text.index('say "3. '), call.start())
        self.assertLess(call.start(), text.index('say "4. '))

    def test_a_passing_cargo_test_continues_the_release(self) -> None:
        r = self.gate()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("cargo test", self.calls.read_text())
        cwd = Path(str(self.calls) + ".cwd").read_text().strip()
        self.assertTrue(cwd.endswith("jubarte-app/src-tauri"), cwd)
        # `../..` is the fresh engine at HEAD — its manifest, and the app
        # inside it — never the release worktree's uncommitted bump
        layout = Path(str(self.calls) + ".layout").read_text().split()
        self.assertIn("Cargo.toml", layout)
        self.assertIn("jubarte-app", layout)
        self.assertNotIn("uncommitted", layout)

    def test_the_gate_is_skipped_only_like_the_other_gates(self) -> None:
        text = RELEASE_SH.read_text()
        call = re.search(r"^\s+app_test_gate$", text, re.M).start()
        gate = text[call - 400:call]
        self.assertIn('[ "$SKIP_GATES" = 0 ]', gate)
        self.assertNotIn("--skip-app", text)
        self.assertNotIn("SKIP_APP", text)

    def test_a_lock_naming_an_older_engine_does_not_stop_the_gate(self) -> None:
        # the app's lock on main records the previous release's versions and
        # the engine's version moves every release, so a stale lock is the
        # normal case: cargo runs without --locked/--frozen in the throwaway
        # layout (the app's real checkout is an archive copy, untouched)
        lock = (self.app / "src-tauri" / "Cargo.lock").read_text()
        self.assertIn('version = "0.13.0"', lock)
        r = self.gate()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual("cargo test\n", self.calls.read_text())

    def test_a_failing_gate_removes_its_layout_and_prints_the_cargo_tail(self) -> None:
        r = self.gate(CARGO_RC="1")
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        # the failed run's own cargo output is the clue, not a kept layout
        self.assertIn("cargo stub: test", r.stderr)
        self.assertNotIn("kept at", r.stderr)
        self.assertEqual(list((self.tmp / "tmp").iterdir()), [])

    def test_keeping_the_layout_on_a_failure_is_an_explicit_ask(self) -> None:
        r = self.gate(CARGO_RC="1", JUBARTE_APP_GATE_KEEP="1")
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        gates = list((self.tmp / "tmp").glob("jubarte-app-gate.*"))
        self.assertEqual(len(gates), 1)
        self.assertIn(str(gates[0]), r.stderr)

    def test_an_interrupt_removes_the_gate_dir(self) -> None:
        cargo = self.tmp / "bin" / "cargo"
        cargo.write_text("#!/bin/bash\nexec sleep 60\n")
        cargo.chmod(0o755)
        proc = subprocess.Popen(
            ["bash", "-c", self.script], cwd=self.engine, env=self.gate_env(),
            start_new_session=True,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        )
        try:
            gates = []
            for _ in range(100):
                gates = list((self.tmp / "tmp").glob("jubarte-app-gate.*"))
                if gates:
                    break
                time.sleep(0.1)
            self.assertEqual(len(gates), 1, "the gate layout never appeared")
            os.killpg(proc.pid, signal.SIGINT)
            proc.communicate(timeout=15)
        finally:
            if proc.poll() is None:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait()
        self.assertNotEqual(proc.returncode, 0)
        self.assertEqual(list((self.tmp / "tmp").iterdir()), [])

    def test_an_existing_tag_does_not_skip_the_gate(self) -> None:
        # the tag skip existed only because --locked made a resume refuse the
        # app's stale lock; a tag proves no app-gate pass, so the gate runs
        # unless --skip-gates
        text = RELEASE_SH.read_text()
        block = text[text.index('say "App gate'):text.index('say "4. ')]
        self.assertNotIn("refs/tags/$TAG", block)
        call = re.search(r"^\s+app_test_gate$", block, re.M)
        self.assertIsNotNone(call)
        self.assertIn('[ "$SKIP_GATES" = 0 ]', block[:call.start()])

    def test_the_checklist_row_is_a_gate_not_a_reminder(self) -> None:
        rows = [line.split("|") for line in checklist_block().splitlines()
                if line.count("|") == 3 and "cargo test" in line]
        self.assertEqual(len(rows), 1, rows)
        phase, mark, item, _proof = rows[0]
        self.assertEqual(mark, "auto")
        self.assertIn("app-at-main", item)
        self.assertIn("src-tauri", item)


class PushMain(unittest.TestCase):
    """scripts/push_main.sh against a real origin: one that takes a direct
    push, and one whose main takes changes only through a pull request (a
    pre-receive hook stands in for the ruleset, a stub `gh` for GitHub)."""

    HOOK = """#!/bin/sh
while read old new ref; do
  if [ "$ref" = refs/heads/main ] && [ "${VIA_PULL_REQUEST:-}" != 1 ]; then
    echo "GH013: Repository rule violations found for refs/heads/main" >&2
    exit 1
  fi
done
"""
    GH = "#!/bin/sh\n" + GH_BOUND + """# stub gh: `pr view` knows no pull request until `pr create`; `pr merge`
# merges the head into main with a merge commit, as GitHub's merge does.
echo "$@" >> "$GH_LOG"
case "$1 $2" in
  "pr view") [ -f "$GH_LOG.open" ] && echo OPEN || exit 1 ;;
  "pr create") : > "$GH_LOG.open" ;;
  "pr merge")
    work=$(mktemp -d) && git clone -q "$ORIGIN" "$work/c" && cd "$work/c" \
      && git -c user.name=gh -c user.email=gh@example.com merge -q --no-ff -m "Merge pull request" "origin/$3" \
      && VIA_PULL_REQUEST=1 git push -q origin HEAD:main ;;
esac
"""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.origin = self.tmp / "origin.git"
        self.work = self.tmp / "work"
        self.log = self.tmp / "gh.log"
        self.log.write_text("")
        bin_dir = self.tmp / "bin"
        bin_dir.mkdir()
        (bin_dir / "gh").write_text(self.GH)
        (bin_dir / "gh").chmod(0o755)
        self.env = {**os.environ, "PATH": f"{bin_dir}{os.pathsep}{os.environ['PATH']}",
                    "GH_LOG": str(self.log), "ORIGIN": str(self.origin),
                    "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.com",
                    "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.com"}
        self.git("init", "-q", "--bare", str(self.origin), cwd=self.tmp)
        self.git("symbolic-ref", "HEAD", "refs/heads/main", cwd=self.origin)
        self.git("clone", "-q", str(self.origin), str(self.work), cwd=self.tmp)
        self.git("checkout", "-q", "-b", "main")
        self.commit("base")
        self.git("push", "-q", "origin", "main")

    def git(self, *args: str, cwd: Path | None = None) -> str:
        return subprocess.run(["git", *args], cwd=cwd or self.work, env=self.env,
                              capture_output=True, text=True, check=True).stdout.strip()

    def commit(self, name: str) -> str:
        (self.work / name).write_text(name)
        self.git("add", name)
        self.git("commit", "-q", "-m", name)
        return self.git("rev-parse", "HEAD")

    def protect_main(self) -> None:
        hook = self.origin / "hooks" / "pre-receive"
        hook.write_text(self.HOOK)
        hook.chmod(0o755)

    def push_main(self) -> subprocess.CompletedProcess[str]:
        script = f'. "{HERE / "push_main.sh"}" && push_main release/v9.9.9 "chore(release): v9.9.9" "body"'
        return subprocess.run(["bash", "-c", script], cwd=self.work, env=self.env,
                              capture_output=True, text=True)

    def origin_main(self) -> str:
        return self.git("rev-parse", "main", cwd=self.origin)

    def test_an_open_main_takes_the_push_and_no_pull_request(self) -> None:
        release = self.commit("release")
        result = self.push_main()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.origin_main(), release)
        self.assertEqual(self.log.read_text(), "")

    def test_a_main_that_wants_a_pull_request_gets_one_merged_at_once(self) -> None:
        self.protect_main()
        release = self.commit("release")
        result = self.push_main()
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.log.read_text()
        self.assertIn("pr create --base main --head release/v9.9.9", calls)
        self.assertIn("pr merge release/v9.9.9 --merge", calls)
        # a merge commit: the gated (and tagged) commit itself is in main
        parents = self.git("rev-list", "--parents", "-n", "1", "main", cwd=self.origin).split()
        self.assertEqual(len(parents), 3)
        self.assertEqual(parents[2], release)
        # and the local main is the one origin holds
        self.assertEqual(self.git("rev-parse", "HEAD"), self.origin_main())

    def test_a_rerun_merges_the_open_pull_request_without_a_second_one(self) -> None:
        self.protect_main()
        self.commit("release")
        Path(str(self.log) + ".open").write_text("")
        result = self.push_main()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn("pr create", self.log.read_text())
        self.assertIn("pr merge release/v9.9.9 --merge", self.log.read_text())

    def test_a_refused_merge_fails_the_push(self) -> None:
        self.protect_main()
        self.commit("release")
        self.env["ORIGIN"] = str(self.tmp / "nowhere.git")  # the stub's merge cannot happen
        result = self.push_main()
        self.assertNotEqual(result.returncode, 0)

    def test_the_release_goes_through_it(self) -> None:
        text = RELEASE_SH.read_text()
        self.assertIn("scripts/push_main.sh", text)
        self.assertIn("push_main ", text)
        self.assertNotIn("git push origin main", text)
        self.assertNotIn("git push -q origin HEAD:main", text)


class Header(unittest.TestCase):
    def test_the_step_list_numbers_the_steps_the_script_prints(self) -> None:
        """The header's "What it does" list counts the steps as `say` names them."""
        import re

        text = RELEASE_SH.read_text()
        header = text.split("# What it does, in order:", 1)[1].split("\n#\n", 1)[0]
        listed = [int(n) for n in re.findall(r"^#\s+(\d+)\. ", header, re.M)]
        printed = list(dict.fromkeys(int(n) for n in re.findall(r'say "(\d+)\. ', text)))
        self.assertEqual(listed, printed)



class WheelSetGate(unittest.TestCase):
    """Step 10 refuses to publish CI wheels that miss an advertised platform:
    scripts/check_release_artifacts.py runs on the downloaded wheel set
    before `uv publish`, and a failure is fatal (`die`), not a warning."""

    def step10(self) -> str:
        text = RELEASE_SH.read_text()
        start = text.index('say "11. PyPI"')
        return text[start:text.index('say "12.', start)]

    def test_check_runs_before_uv_publish(self) -> None:
        step = self.step10()
        check = step.index("scripts/check_release_artifacts.py")
        self.assertLess(check, step.index("uv publish --token"))

    def test_check_failure_is_fatal(self) -> None:
        self.assertRegex(
            self.step10(),
            r'python3 scripts/check_release_artifacts.py dist/pypi --version "\$VER"[^\n]*\n?[^\n]*\|\| die ',
        )

    def test_check_is_a_tested_script(self) -> None:
        self.assertTrue((HERE / "check_release_artifacts.py").is_file())
        self.assertTrue((HERE / "test_check_release_artifacts.py").is_file())



class LibraryReadmes(unittest.TestCase):
    """release.sh cuts one README per library, gates on it, and ships it."""

    def setUp(self) -> None:
        self.text = RELEASE_SH.read_text(encoding="utf-8")

    def section(self, start: str, end: str) -> str:
        i = self.text.index(start)
        return self.text[i:self.text.index(end, i)]

    def test_version_sync_cuts_them_for_the_release(self) -> None:
        sync = self.section('say "1.', 'say "2.')
        self.assertRegex(
            sync,
            r'python3 scripts/library_readmes.py --version "\$VER"[^\n]*\n?[^\n]*\|\| die ',
        )

    def test_dry_run_step_proves_every_registry_ships_its_cut(self) -> None:
        dry = self.section('say "7.', 'if [ "$DRY_RUN" = 1 ]')
        self.assertIn('library_readmes.py --check --version "$VER"', dry)
        self.assertIn("grep -qx README.crates.md", dry)
        self.assertIn("PKG-INFO", dry)
        self.assertIn("jubarte-wasm/npm jubarte-wasm/cli", dry)

    def test_the_release_commit_carries_them(self) -> None:
        commit = self.section('git add Cargo.toml', 'git commit -m "chore(release)')
        for path in (
            "README.crates.md",
            "jubarte-python/README.md",
            "jubarte-wasm/npm/README.md",
            "jubarte-wasm/cli/README.md",
        ):
            self.assertIn(path, commit)

    def test_gates_run_the_generator_tests(self) -> None:
        gates = self.section('say "5. Gates', 'say "5. Gates — SKIPPED')
        self.assertIn('"${py312[@]}" scripts/test_library_readmes.py', gates)
        self.assertIn('"${py312[@]}" scripts/test_check_release_info.py', gates)

    def test_the_crate_points_at_its_generated_readme(self) -> None:
        cargo = (HERE.parent / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('readme = "README.crates.md"', cargo)
        self.assertIn('"/README.crates.md"', cargo)


class QuietGates(unittest.TestCase):
    """A passing gate prints its unittest summary and nothing else. The 0.11.2
    log carried `RESULT: REGRESSION`, `missing corpus listing` and a dozen
    ResourceWarnings from script tests that passed, and read as a failed
    release."""

    SUMMARY = re.compile(
        r"\A[.sx]+\n-{70}\nRan \d+ tests? in [\d.]+s\n\nOK(?: \([^)\n]*\))?\n\Z"
    )

    def script_gates(self):
        """The script tests step 5 runs, this file aside (it runs them)."""
        found = re.findall(r'^\s*"\$\{py312\[@\]\}" (\S*test_\w+\.py)$', step(5), re.M)
        return [rel for rel in found if Path(rel).name != Path(__file__).name]

    def test_every_script_gate_prints_only_its_summary(self) -> None:
        gates = self.script_gates()
        self.assertGreaterEqual(len(gates), 4, gates)
        for rel in gates:
            with self.subTest(gate=rel):
                r = subprocess.run(
                    [sys.executable, rel], cwd=HERE.parent,
                    capture_output=True, text=True, timeout=300,
                )
                self.assertEqual(r.returncode, 0, r.stderr)
                self.assertEqual(r.stdout, "")
                self.assertRegex(r.stderr, self.SUMMARY)

    def test_no_script_leaves_a_file_open(self) -> None:
        # `json.dump(x, open(p, "w"))` and `open(p).read()` leave the closing
        # to the garbage collector: a ResourceWarning per call under unittest.
        leaks = []
        for script in sorted([*HERE.glob("*.py"), *(HERE.parent / "planning").glob("*.py")]):
            tree = ast.parse(script.read_text(encoding="utf-8"))
            managed = {
                id(item.context_expr)
                for node in ast.walk(tree) if isinstance(node, ast.With)
                for item in node.items
            }
            leaks += [
                f"{script.relative_to(HERE.parent)}:{node.lineno}"
                for node in ast.walk(tree)
                if isinstance(node, ast.Call)
                and isinstance(node.func, ast.Name) and node.func.id == "open"
                and id(node) not in managed
            ]
        self.assertEqual(leaks, [], "open() outside a `with`")


class LeftOutTargets(unittest.TestCase):
    """cargo warns once for every tests/*.rs and examples/*.rs the crate's
    `include` leaves out of the package: 431 `warning:` lines in the 0.11.2
    dry run, every one of them by design."""

    LEFT_OUT = "warning: ignoring test `a` as `tests/a.rs` is not included in the published package"

    def run_packaged(self, fake: str) -> subprocess.CompletedProcess[str]:
        text = RELEASE_SH.read_text()
        start = text.index("\npackaged() {")
        function = text[start:text.index("\n}\n", start) + 3]
        return subprocess.run(
            ["bash", "-c", f"set -euo pipefail\n{function}\nfake() {{ {fake}; }}\npackaged fake"],
            capture_output=True, text=True, timeout=60,
        )

    def test_the_left_out_warnings_are_dropped_and_the_rest_kept(self) -> None:
        r = self.run_packaged(
            f"echo listed; echo '{self.LEFT_OUT}' >&2; echo 'warning: a real one' >&2"
        )
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(r.stdout, "listed\n")
        self.assertEqual(r.stderr, "warning: a real one\n")

    def test_a_failed_command_still_fails(self) -> None:
        r = self.run_packaged(f"echo '{self.LEFT_OUT}' >&2; echo 'error: no' >&2; return 3")
        self.assertEqual(r.returncode, 3)
        self.assertEqual(r.stderr, "error: no\n")

    def test_a_run_with_nothing_but_left_out_warnings_passes(self) -> None:
        r = self.run_packaged(f"echo '{self.LEFT_OUT}' >&2")
        self.assertEqual((r.returncode, r.stdout, r.stderr), (0, "", ""))

    def test_the_dry_run_goes_through_it(self) -> None:
        self.assertIn("packaged cargo publish --dry-run --locked --allow-dirty", step(7))


def shell_functions(*names: str) -> str:
    """The definitions of release.sh's functions `names` that exist, in order:
    one-line, backslash-continued or `{ … }` block."""
    text = RELEASE_SH.read_text()
    out = []
    for name in names:
        m = re.search(rf"^{re.escape(name)}\(\) ", text, re.M)
        if m is None:
            continue
        start = m.start()
        first = text[start:text.index("\n", start)].rstrip()
        if first.endswith("{"):
            out.append(text[start:text.index("\n}\n", start) + 3])
            continue
        end = text.index("\n", start)
        while text[start:end].rstrip().endswith("\\"):
            end = text.index("\n", end + 1)
        out.append(text[start:end + 1])
    return "".join(out)


# stub gh for step 11: the release, its asset names and the release.yml run,
# each driven by the environment; every asset listing is counted in $STATE.
GH_RELEASE_STUB = "#!/bin/bash\n" + GH_BOUND + r"""case "$*" in
  "release view v9.9.9 -R jandira-tech/jubarte-redlines")
    [ -z "${NOREL:-}" ] ;;
  "release view v9.9.9 --json assets"*)
    [ -z "${NOREL:-}" ] || exit 1
    n=$(( $(cat "$STATE" 2>/dev/null || echo 0) + 1 )); echo "$n" > "$STATE"
    echo jubarte-9.9.9-macos-aarch64.tar.gz
    if [ -z "${PARTIAL:-}" ] && [ "$n" -ge 3 ]; then
      for t in macosx_10_12_x86_64 macosx_11_0_arm64 manylinux_2_28_x86_64 \
               manylinux_2_28_aarch64 musllinux_1_2_x86_64 musllinux_1_2_aarch64 win_amd64; do
        echo "jubarte_redlines-9.9.9-cp310-abi3-$t.whl"
      done
      echo jubarte_redlines-9.9.9.tar.gz
    fi ;;
  "run list"*)
    echo "42 ${RUN_STATE:-in_progress}" ;;
  *)
    echo "unexpected gh $*" >&2; exit 9 ;;
esac
"""


class ReleaseWait(unittest.TestCase):
    """0.11.2: release.yml creates the GitHub release and then uploads its
    assets; step 11 stopped waiting once the release existed, found no wheel
    and died, minutes before all seven wheels and the sdist were attached."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="relwait-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "gh").write_text(GH_RELEASE_STUB)
        (self.tmp / "gh").chmod(0o755)
        self.state = self.tmp / "listings"

    def wait(self, **env: str) -> subprocess.CompletedProcess[str]:
        functions = shell_functions("ghrel_has", "ghrel_wheels", "release_run", "wait_for_release")
        script = (
            "set -euo pipefail\nTAG=v9.9.9; VER=9.9.9\n"
            'step() { echo "  - $*"; }\ndie() { echo "ERROR: $*" >&2; exit 1; }\n'
            "sleep() { :; }\n"
            'release_from_artifacts() { echo "from artifacts $1"; }\n'
            f"{functions}\nwait_for_release\n"
            'echo "listings $(cat "$STATE" 2>/dev/null || echo 0)"\n'
        )
        full = dict(os.environ, PATH=f"{self.tmp}:{os.environ['PATH']}", STATE=str(self.state), **env)
        return subprocess.run(["bash", "-c", script], cwd=HERE.parent, env=full,
                              capture_output=True, text=True, timeout=60)

    def test_a_release_without_its_wheels_yet_is_waited_for(self) -> None:
        r = self.wait()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("listings 3\n", r.stdout)
        self.assertNotIn("from artifacts", r.stdout)

    def test_a_finished_run_ends_the_wait_on_a_partial_release(self) -> None:
        # the download below then names the missing wheels; no second release
        r = self.wait(PARTIAL="1", RUN_STATE="completed")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("from artifacts", r.stdout)
        self.assertLessEqual(int(r.stdout.rsplit("listings ", 1)[1]), 3)

    def test_a_run_without_a_release_is_released_from_its_artifacts(self) -> None:
        r = self.wait(NOREL="1", RUN_STATE="completed")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("from artifacts 42", r.stdout)

    def test_a_run_still_going_after_the_bound_stops_the_script(self) -> None:
        r = self.wait(NOREL="1")
        self.assertEqual(r.returncode, 1)
        self.assertIn("has not finished", r.stderr)


class WheelDownload(unittest.TestCase):
    """0.11.3: step 11 ran inside `script` (for its log), so gh had a
    terminal; the download's progress display queried it, failed, and the
    discarded error read as "no wheels" while the release held all seven.
    The stub gh fails the same way whenever its stdin or stdout is a
    terminal, and the run gets one (a pty), as the release did."""

    GH = "#!/bin/bash\n" + GH_BOUND + r"""case "$*" in
  "release download"*)
    if [ -t 0 ] || [ -t 1 ]; then echo "error querying terminal: OSC 11" >&2; exit 1; fi
    [ -z "${DLFAIL:-}" ] || { echo "HTTP 502: Bad Gateway" >&2; exit 1; }
    dir=$(echo "$*" | sed 's/.*--dir \([^ ]*\).*/\1/')
    : > "$dir/jubarte_redlines-9.9.9-cp310-abi3-win_amd64.whl" ;;
  "release view v9.9.9 --json assets"*)
    [ -z "${NOWHEELS:-}" ] || { echo jubarte-9.9.9-macos-aarch64.tar.gz; exit 0; }
    for t in macosx_10_12_x86_64 macosx_11_0_arm64 manylinux_2_28_x86_64 \
             manylinux_2_28_aarch64 musllinux_1_2_x86_64 musllinux_1_2_aarch64 win_amd64; do
      echo "jubarte_redlines-9.9.9-cp310-abi3-$t.whl"
    done
    echo jubarte_redlines-9.9.9.tar.gz ;;
  *) echo "unexpected gh $*" >&2; exit 9 ;;
esac
"""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="wheeldl-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "bin").mkdir()
        (self.tmp / "bin" / "gh").write_text(self.GH)
        (self.tmp / "bin" / "gh").chmod(0o755)
        (self.tmp / "dist" / "pypi").mkdir(parents=True)
        (self.tmp / "scripts").mkdir()
        shutil.copy(HERE / "check_release_artifacts.py", self.tmp / "scripts")

    def download(self, **env: str) -> tuple[int, str]:
        """Runs download_ci_wheels with a terminal on stdin, stdout and
        stderr; returns its exit status and everything it printed."""
        import pty

        functions = shell_functions("ghrel_wheels", "download_ci_wheels")
        script = (
            "set -euo pipefail\nTAG=v9.9.9; VER=9.9.9\n"
            'die() { echo "ERROR: $*" >&2; exit 1; }\n'
            f"{functions}\n"
            'if download_ci_wheels; then echo GOT; else echo NONE; fi\n'
        )
        full = dict(os.environ, PATH=f"{self.tmp / 'bin'}:{os.environ['PATH']}", **env)
        main, sub = pty.openpty()
        proc = subprocess.Popen(["bash", "-c", script], cwd=self.tmp, env=full,
                                stdin=sub, stdout=sub, stderr=sub)
        os.close(sub)
        out = b""
        while True:
            try:
                chunk = os.read(main, 4096)
            except OSError:  # Linux: EIO once the child side closes
                break
            if not chunk:
                break
            out += chunk
        os.close(main)
        return proc.wait(timeout=60), out.decode(errors="replace")

    def test_the_download_gets_no_terminal(self) -> None:
        code, out = self.download()
        self.assertEqual(code, 0, out)
        self.assertIn("GOT", out)
        self.assertNotIn("OSC 11", out)

    def test_a_failed_download_of_a_whole_release_stops_and_says_why(self) -> None:
        code, out = self.download(DLFAIL="1")
        self.assertEqual(code, 1, out)
        self.assertIn("HTTP 502: Bad Gateway", out)  # gh's own words are kept
        self.assertIn("lists every wheel, but gh would not download them", out)
        self.assertNotIn("GOT", out)

    def test_a_release_without_its_wheels_falls_through_to_the_consent_check(self) -> None:
        code, out = self.download(DLFAIL="1", NOWHEELS="1")
        self.assertEqual(code, 0, out)
        self.assertIn("NONE", out)
        self.assertIn("HTTP 502: Bad Gateway", out)

    def test_step_11_downloads_through_it(self) -> None:
        self.assertIn("if download_ci_wheels; then", step(11))
        self.assertNotIn("gh release download", step(11))


class RegistryLag(unittest.TestCase):
    """0.11.2: step 12 called PyPI missing seconds after `uv publish`; the
    project listing named 0.11.2 a minute later."""

    CURL_STUB = r"""#!/bin/bash
n=$(( $(cat "$STATE" 2>/dev/null || echo 0) + 1 )); echo "$n" > "$STATE"
if [ "$n" -ge "${READY_AT:-99}" ]; then
  echo '{"releases": {"9.9.8": [], "9.9.9": []}}'
else
  echo '{"releases": {"9.9.8": []}}'
fi
"""

    def probe(self, ready_at: str) -> tuple[subprocess.CompletedProcess[str], int]:
        tmp = Path(tempfile.mkdtemp(prefix="reglag-"))
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        (tmp / "curl").write_text(self.CURL_STUB)
        (tmp / "curl").chmod(0o755)
        state = tmp / "calls"
        script = (
            "set -euo pipefail\nVER=9.9.9\nsleep() { :; }\n"
            f"{shell_functions('pypi_has', 'eventually')}\n"
            "if eventually pypi_has; then echo listed; else echo missing; fi\n"
        )
        env = dict(os.environ, PATH=f"{tmp}:{os.environ['PATH']}", STATE=str(state), READY_AT=ready_at)
        r = subprocess.run(["bash", "-c", script], env=env, capture_output=True, text=True, timeout=60)
        return r, int(state.read_text()) if state.exists() else 0

    def test_a_listing_that_lags_the_upload_is_asked_again(self) -> None:
        r, calls = self.probe("3")
        self.assertEqual((r.stdout, calls), ("listed\n", 3), r.stderr)

    def test_a_version_that_never_appears_is_missing_after_a_bounded_wait(self) -> None:
        r, calls = self.probe("99")
        self.assertEqual((r.stdout, calls), ("missing\n", 12), r.stderr)

    def test_every_registry_check_of_step_12_retries(self) -> None:
        s12 = step(12)
        for probe in ("crates_has", "npm_has", "npm_cli_has", "pypi_has"):
            self.assertRegex(s12, rf'check "[^"]*" "eventually {probe}"')
        self.assertNotIn("sleep 20", s12)


class WasmCrate(unittest.TestCase):
    def test_the_wasm_crate_names_the_repository(self) -> None:
        # wasm-pack printed "Optional field missing from Cargo.toml:
        # 'repository'" on each of the four builds of a release.
        engine = (HERE.parent / "Cargo.toml").read_text(encoding="utf-8")
        repository = re.search(r'^repository = "[^"]+"$', engine, re.M).group(0)
        wasm = (HERE.parent / "jubarte-wasm" / "Cargo.toml").read_text(encoding="utf-8")
        package = wasm[wasm.index("[package]"):wasm.index("[lib]")]
        self.assertIn(repository + "\n", package)


CHECKLIST_FLAG = "--checklist"


def checklist_block() -> str:
    """The checklist_items() block of release.sh — the one source of truth
    --checklist, the per-step "CHECK NOW" blocks and the closing summary all
    print from."""
    text = RELEASE_SH.read_text()
    start = text.index("checklist_items() {")
    return text[start : text.index("\nITEMS\n", start) + len("\nITEMS\n")]


def you_item_steps():
    """Step numbers whose checklist lines carry a [you] item."""
    steps = []
    for line in checklist_block().splitlines():
        parts = line.split("|")
        if len(parts) == 4 and parts[1] == "you" and parts[0].isdigit():
            if int(parts[0]) not in steps:
                steps.append(int(parts[0]))
    return sorted(steps)


class Checklist(unittest.TestCase):
    """--checklist [VERSION]: prints the whole release checklist, every
    phase, every line marked [auto] or [you] with its proving command, and
    exits 0 without touching anything — it leaves before the preflight, so
    no summaries, no main branch and no clean tree are needed."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.out = subprocess.run(
            ["bash", str(RELEASE_SH), CHECKLIST_FLAG, "0.11.3"],
            capture_output=True, text=True, timeout=60,
        )

    def test_exits_zero(self) -> None:
        self.assertEqual(self.out.returncode, 0, self.out.stderr)

    def test_prints_every_phase(self) -> None:
        out = self.out.stdout
        self.assertIn("BEFORE the script", out)
        self.assertIn("AFTER the script", out)
        # every step the script runs, and every phase an item sits in, prints
        # (the first checklist carried a dangling empty "step 14" header)
        text = RELEASE_SH.read_text()
        for n in sorted({int(n) for n in re.findall(r'say "(\d+)\. ', text)}):
            self.assertIn("step %d " % n, out)
        phases = {line.split("|", 1)[0] for line in checklist_block().splitlines()
                  if line.count("|") == 3}
        for ph in phases - {"before", "after"}:
            self.assertIn("step %s " % ph, out)

    def test_marks_and_proves_every_line(self) -> None:
        self.assertIn("[auto]", self.out.stdout)
        self.assertIn("[you]", self.out.stdout)
        self.assertIn("proof:", self.out.stdout)

    def test_leaves_before_any_step_runs(self) -> None:
        # No step, no network: the exit sits between argument parsing and the
        # summaries validation, well before the preflight.
        text = RELEASE_SH.read_text()
        self.assertLess(
            text.index('if [ "$CHECKLIST" = 1 ]'),
            text.index('say "0. Preflight"'),
        )

    def test_works_off_main_on_a_dirty_tree_without_summaries(self) -> None:
        tmp = Path(tempfile.mkdtemp(prefix="checklist_"))
        try:
            (tmp / "scripts").mkdir()
            shutil.copy(RELEASE_SH, tmp / "scripts" / "release.sh")
            subprocess.run(
                ["git", "-C", str(tmp), "init", "-q", "-b", "not-main"], check=True
            )
            (tmp / "dirt").write_text("untracked dirt\n")
            r = subprocess.run(
                ["bash", str(tmp / "scripts" / "release.sh"), CHECKLIST_FLAG],
                capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn("BEFORE the script", r.stdout)
            self.assertIn("[you]", r.stdout)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    # Each item that had to be inspected by hand during the 0.11.2 release
    # must be findable in the --checklist output by a stable keyword.
    def kw(self, keyword: str) -> None:
        self.assertIn(keyword, self.out.stdout, "--checklist must name %r" % keyword)

    def test_item_1_release_info_evidence(self) -> None:
        self.kw("release_info")
        self.kw("identity")  # the two-binary identity run

    def test_item_2_binding_parity(self) -> None:
        self.kw("parity")

    def test_item_3_ci_of_the_release_pr(self) -> None:
        self.kw("CI")

    def test_item_4_windows_stack_and_old_python(self) -> None:
        self.kw("1 MiB")
        self.kw("Python 3.12")

    def test_stack_proof_names_the_runnable_script_and_all_required_inputs(self) -> None:
        lines = self.out.stdout.splitlines()
        stack_line = next(n for n, line in enumerate(lines) if "stack: the release binary" in line)
        self.assertIn(
            # the version is filled in, like every @V@ of the checklist (#1757)
            "python3 scripts/stack1m.py <release binary> ../neurotic_docx_bench <out dir> "
            "release_info/sample_*_0.11.3_*.csv",
            lines[stack_line + 1],
        )

    def test_item_5_unused_dependencies(self) -> None:
        self.kw("unused")

    def test_item_6_keywords_per_channel(self) -> None:
        self.kw("keywords")

    def test_item_7_changelogs_and_numbers(self) -> None:
        self.kw("CHANGELOG")
        self.kw("NUMBERS")

    def test_item_8_app_repository_release_files(self) -> None:
        self.kw("facts.jsonl")
        self.kw("app-at-main")  # the app builds against the engine at the tag

    def test_item_9_credentials_and_human_steps(self) -> None:
        self.kw("UV_PUBLISH_TOKEN")
        self.kw("rustdoc")
        self.kw("point of no return")  # the typed tag confirmation

    def test_item_10_wheels_on_the_release(self) -> None:
        self.kw("wheel")

    def test_item_11_downstream_site_and_bench(self) -> None:
        self.kw("jubarte.pro")
        self.kw("Hugging Face")
        self.kw("RESULTS.md")

    def test_item_11_the_site_figures_are_automatic(self) -> None:
        # Moving website_data into the site and deploying it is the script's
        # own work now: an [auto] line with its proving command, no [you] line.
        lines = self.out.stdout.splitlines()
        auto = [n for n, line in enumerate(lines)
                if line.strip().startswith("[auto]") and "website_data" in line]
        self.assertTrue(auto, "--checklist must carry an [auto] website_data line")
        self.assertTrue(any("check_site_live.py 0.11.3" in lines[n + 1] for n in auto))
        self.assertFalse([line for line in lines
                          if line.strip().startswith("[you]") and "merge the website_data" in line])

    def test_item_11_the_app_checkout_is_named_and_checked_first(self) -> None:
        self.kw("JUBARTE_APP_DIR")
        self.kw('jubarte-app/scripts/release-engine.sh 0.11.3 --engine-dir "$PWD" --preflight')

    def test_item_12_mac_app_store(self) -> None:
        self.kw("App Store")

    def test_item_13_developer_id_notarization(self) -> None:
        self.kw("notarize")
        self.kw("DMG")

    def test_item_14_post_release_reproduction(self) -> None:
        self.kw("reproduces")
        self.kw("--binary")  # the bench rerun without --binary

    def test_item_15_licensing_and_stale_docs(self) -> None:
        self.kw("REUSE")
        self.kw("adoption")

    # The gaps the 0.13.0 checklist close: the website's pages, release.yml,
    # the full Miri suite, the Ring-3 probe, the self-update path and the
    # App Store's in-app purchase must all be findable by a stable keyword.
    def test_item_16_the_websites_pages_are_read(self) -> None:
        self.kw("/download")
        self.kw("/benchmark")
        self.kw("/demo")
        self.kw("/terms")

    def test_item_17_release_yml_is_reviewed(self) -> None:
        self.kw("SHA-pinned")
        self.kw("ZIPSIGN_PRIVATE_KEY")

    def test_item_18_the_full_miri_suite_runs_first(self) -> None:
        self.kw("miri")

    def test_item_19_ring3_word_probe(self) -> None:
        self.kw("Ring-3")

    def test_item_20_self_update_serves_the_release(self) -> None:
        self.kw("self-update")

    def test_item_21_the_app_store_item_lists_the_iap(self) -> None:
        self.kw("in-app purchase")

    def test_item_22_the_measured_sentence_is_cross_checked(self) -> None:
        self.kw("NUMBERS sentence")


class ChecklistPerStep(unittest.TestCase):
    """A real run must print each step's [you] items at the moment the step
    reaches them (check_now <step>), and the closing summary must print every
    [you] item still owed after the script ends — all from the one block."""

    def test_every_step_with_a_you_item_prints_it(self) -> None:
        steps = you_item_steps()
        self.assertTrue(steps, "the checklist must carry per-step [you] items")
        for n in steps:
            self.assertIn("check_now %d" % n, step(n), "step %d" % n)

    def test_the_closing_summary_prints_the_owed_items(self) -> None:
        text = RELEASE_SH.read_text()
        tail = text[text.index('say "Released jubarte') :]
        self.assertIn("checklist_items", tail)
        self.assertIn('"after"', tail)

    def test_before_and_after_items_live_in_the_one_block(self) -> None:
        block = checklist_block()
        self.assertIn("before|you|", block)
        self.assertIn("after|you|", block)


class QualityGate1797(unittest.TestCase):
    """#1797: the release is measured against the previous release, run on the
    same corpus and machine — never a stored <baseline> file. The previous
    release is found relative to the release being made (from v@V@^ once its
    tag exists, from HEAD before it), B is the tag when resuming, the verdict
    keeps the issue's >5% bar against this run's own self-noise, and the
    ladder's corpus is named with --corpus."""

    def row(self) -> str:
        return next(line for line in checklist_block().splitlines()
                    if "quality gate" in line)

    def test_the_gate_row_names_the_previous_release_not_a_literal_baseline(self) -> None:
        row = self.row()
        self.assertNotIn("<baseline>", row)
        self.assertIn("previous release", row)

    def test_the_previous_release_is_found_relative_to_the_release_being_made(self) -> None:
        # an old-release resume must not pick a tag newer than the release:
        # describe walks from v@V@^ once the tag exists, from HEAD before it
        row = self.row()
        self.assertIn("v@V@^", row)
        self.assertIn("--match 'v[0-9]*'", row)
        self.assertIn("echo 'v@V@^' || echo HEAD", row)

    def test_b_is_the_tag_when_resuming_and_head_before_it(self) -> None:
        row = self.row()
        self.assertIn("echo v@V@ || echo HEAD", row)

    def test_the_gate_row_names_the_corpus_explicitly_and_how_to_get_it(self) -> None:
        row = self.row()
        proof = row.split("|", 3)[3]
        self.assertIn("--corpus", proof)
        self.assertIn("ladder_corpus", row)
        self.assertIn("corpus/word", row)  # where the bench now keeps it
        self.assertIn("fixtures download", row)  # how to get it

    def test_the_gate_row_names_the_verdict_not_a_fixed_noise_floor(self) -> None:
        # the issue's bar stays >5% per case, gated by this run's own
        # self-noise; the 0.14.0 +9.7% observation never became a +10% pass
        row = self.row()
        self.assertNotIn("noise floor", row)
        self.assertNotIn("9.7", row)
        self.assertIn(">5%", row)
        self.assertIn("self-noise", row)
        self.assertIn("UNDECIDABLE", row)

    def test_the_gate_row_runs_against_the_previous_binary_and_load_baseline(self) -> None:
        row = self.row()
        self.assertIn("--against", row)
        self.assertIn("scripts/criterion_abab.sh", row)
        self.assertIn("CRITERION_HOME", row)
        proof = row.split("|", 3)[3]
        self.assertNotIn("/tmp", proof)
        self.assertNotIn("mktemp", proof)

    def test_the_gates_run_the_ladder_and_abab_tests(self) -> None:
        text = RELEASE_SH.read_text()
        self.assertIn('"${py312[@]}" tools/test_parity_ladder.py', text)
        self.assertIn('"${py312[@]}" scripts/test_criterion_abab.py', text)


# stub gh for the CI-publish wait: the release.yml run and its publish job's
# conclusion ("" while the job runs), driven by the environment.
GH_PUBLISH_STUB = "#!/bin/bash\n" + GH_BOUND + r"""case "$*" in
  "run list"*)
    echo "42 in_progress" ;;
  "run view 42 --json jobs"*)
    echo "${JOB_STATE:-}" ;;
  *)
    echo "unexpected gh $*" >&2; exit 9 ;;
esac
"""


class CiPublishing(unittest.TestCase):
    """#1642: release.yml publishes crates.io, PyPI and npm with OIDC
    trusted publishing; steps 9-11 wait for it instead of publishing with
    the releaser's tokens, and --local-publish keeps the token path."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="cipub-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "gh").write_text(GH_PUBLISH_STUB)
        (self.tmp / "gh").chmod(0o755)

    def wait(self, **env: str) -> subprocess.CompletedProcess[str]:
        functions = shell_functions("release_run", "eventually", "ci_job_state", "ci_published")
        script = (
            "set -euo pipefail\nTAG=v9.9.9; VER=9.9.9; n=0\n"
            'step() { echo "  - $*"; }\ndie() { echo "ERROR: $*" >&2; exit 1; }\n'
            "sleep() { :; }\n"
            'probe() { n=$((n + 1)); [ "$n" -ge "${LIVE_AT:-999999}" ]; }\n'
            f"{functions}\n"
            'ci_published "crates.io" "publish crates.io" probe\n'
            'echo "probes $n"\n'
        )
        full = dict(os.environ, PATH=f"{self.tmp}:{os.environ['PATH']}", **env)
        return subprocess.run(["bash", "-c", script], cwd=HERE.parent, env=full,
                              capture_output=True, text=True, timeout=60)

    def test_a_registry_that_holds_the_version_ends_the_wait(self) -> None:
        r = self.wait(LIVE_AT="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("published by release.yml", r.stdout)
        self.assertIn("probes 1\n", r.stdout)

    def test_a_running_job_is_waited_for(self) -> None:
        r = self.wait(LIVE_AT="4")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("probes 4\n", r.stdout)

    def test_a_failed_job_stops_the_script_and_names_the_fallback(self) -> None:
        r = self.wait(JOB_STATE="failure")
        self.assertEqual(r.returncode, 1)
        self.assertIn('"publish crates.io"', r.stderr)
        self.assertIn("--local-publish", r.stderr)
        self.assertIn("environment release", r.stderr)

    def test_a_successful_job_waits_out_the_registry_lag(self) -> None:
        r = self.wait(JOB_STATE="success", LIVE_AT="3")
        self.assertEqual(r.returncode, 0, r.stderr)

    def test_a_successful_job_the_registry_never_lists_stops_the_script(self) -> None:
        r = self.wait(JOB_STATE="success")
        self.assertEqual(r.returncode, 1)
        self.assertIn("does not list 9.9.9", r.stderr)

    def test_steps_9_to_11_wait_for_the_jobs_release_yml_names(self) -> None:
        workflow = (HERE.parent / ".github" / "workflows" / "release.yml").read_text()
        calls = {9: [], 10: [], 11: []}
        for n in calls:
            calls[n] = re.findall(r'ci_published "[^"]+" "([^"]+)"', step(n))
            self.assertTrue(calls[n], f"step {n} does not wait for release.yml")
            for job in calls[n]:
                self.assertIn(f"name: {job}\n", workflow, job)

    def test_the_token_publishes_run_only_with_local_publish(self) -> None:
        for n, cmd in ((9, "cargo publish --locked"), (10, "npm publish"),
                       (11, "uv publish --token")):
            s = step(n)
            self.assertLess(s.index('"$LOCAL_PUBLISH" = 1'), s.index(cmd), n)

    def test_the_preflight_asks_for_tokens_only_with_local_publish(self) -> None:
        s = step(0)
        gate = s.index('"$LOCAL_PUBLISH" = 1')
        for cred in ("credentials.toml", "npm whoami", "UV_PUBLISH_TOKEN"):
            self.assertLess(gate, s.index(cred), cred)

    def test_no_wait_without_local_publish_is_refused(self) -> None:
        r = subprocess.run(
            ["bash", str(RELEASE_SH), "0.10.0", *SUMMARIES, DOCS_FLAG, "docs", "--no-wait"],
            capture_output=True, text=True, timeout=60,
        )
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertIn("--no-wait needs --local-publish", r.stderr)

    def test_a_fallback_release_ships_the_sdist_pypi_got(self) -> None:
        # The run's sdist artifact is the one publish-pypi uploads; a second
        # maturin build would give the GitHub release different bytes.
        body = shell_functions("release_from_artifacts")
        self.assertIn("gh run download", body)
        self.assertNotIn("maturin sdist", body)
        self.assertRegex(body, r'check_release_artifacts.py dist/release --version "\$VER" --sdist')

    def test_local_publish_is_a_documented_flag(self) -> None:
        self.assertIn("--local-publish", RELEASE_SH.read_text().split("set -euo pipefail")[0])


# stub cargo for the bump gate: every call is recorded in $CALLS; the
# semver-checks verdict is $SEMVER (0 passes).
CARGO_SEMVER_STUB = r"""#!/bin/bash
echo "cargo $*" >> "$CALLS"
case "$1" in
  semver-checks) exit "${SEMVER:-0}" ;;
  *) echo "unexpected cargo $*" >&2; exit 9 ;;
esac
"""


class BumpGate(unittest.TestCase):
    """#1742: docs/VERSIONING.md's bump rule is proven before step 1 writes
    anything. Pre-1.0, a patch carries no new feature (`### Added`) and no
    change cargo-semver-checks reports; a minor or major may."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="bumpgate-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "bin").mkdir()
        (self.tmp / "bin" / "cargo").write_text(CARGO_SEMVER_STUB)
        (self.tmp / "bin" / "cargo").chmod(0o755)
        self.calls = self.tmp / "calls"

    def gate(self, ver: str, prev: str, section: str, **env: str) -> subprocess.CompletedProcess[str]:
        (self.tmp / "CHANGELOG.md").write_text(
            f"# Changelog\n\n## [Unreleased]\n\n## [{ver}] - 2026-10-25\n\n{section}\n"
            f"## [{prev}] - 2026-10-22\n\n### Fixed\n\n- old\n"
        )
        functions = shell_functions("bump_kind", "changelog_section", "bump_gate")
        script = (
            f"set -euo pipefail\nVER={ver}; TAG=v{ver}\n"
            'step() { echo "  - $*"; }\ndie() { echo "ERROR: $*" >&2; exit 1; }\n'
            'semver_lints_ok() { return "${LINTS:-0}"; }\n'
            f"{functions}\nbump_gate v{prev}\n"
        )
        full = dict(os.environ, PATH=f"{self.tmp / 'bin'}:{os.environ['PATH']}",
                    CALLS=str(self.calls), **env)
        return subprocess.run(["bash", "-c", script], cwd=self.tmp, env=full,
                              capture_output=True, text=True, timeout=60)

    def cargo_calls(self) -> str:
        return self.calls.read_text() if self.calls.exists() else ""

    def test_the_bump_kind_reads_the_two_versions(self) -> None:
        functions = shell_functions("bump_kind")
        for prev, ver, want in (("0.13.0", "0.13.1", "patch"), ("0.13.0", "0.14.0", "minor"),
                                ("0.13.2", "0.14.0", "minor"), ("0.13.0", "1.0.0", "major")):
            r = subprocess.run(["bash", "-c", f"{functions}\nbump_kind {prev} {ver}"],
                               capture_output=True, text=True, timeout=30)
            self.assertEqual((r.returncode, r.stdout.strip()), (0, want), (prev, ver, r.stderr))

    def test_a_version_not_above_the_previous_one_is_no_bump(self) -> None:
        functions = shell_functions("bump_kind")
        for prev, ver in (("0.13.0", "0.13.0"), ("0.13.1", "0.13.0"), ("0.14.0", "0.13.9")):
            r = subprocess.run(["bash", "-c", f"{functions}\nbump_kind {prev} {ver}"],
                               capture_output=True, text=True, timeout=30)
            self.assertNotEqual(r.returncode, 0, (prev, ver))

    def test_a_patch_with_new_features_takes_the_minor_instead(self) -> None:
        r = self.gate("0.13.1", "0.13.0", "### Added\n\n- **`read -p` repeats**.\n")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("### Added", r.stderr)
        self.assertIn("0.14.0", r.stderr)
        self.assertIn("docs/VERSIONING.md", r.stderr)
        self.assertEqual(self.cargo_calls(), "", "the heading alone decides; no build")

    def test_a_patch_semver_checks_refuses_stops_the_release(self) -> None:
        r = self.gate("0.13.1", "0.13.0", "### Fixed\n\n- a fix\n", SEMVER="1")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("0.14.0", r.stderr)
        calls = self.cargo_calls()
        self.assertIn("semver-checks check-release", calls)
        self.assertIn("--baseline-rev v0.13.0", calls)
        self.assertIn("--release-type patch", calls)
        self.assertIn("--default-features", calls)

    def test_a_lint_table_that_skips_a_lint_stops_the_patch_first(self) -> None:
        r = self.gate("0.13.1", "0.13.0", "### Fixed\n\n- a fix\n", LINTS="1")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("cargo-semver-checks.lints", r.stderr)
        self.assertEqual(self.cargo_calls(), "", "no check runs with lints left out")

    def test_a_clean_patch_passes(self) -> None:
        r = self.gate("0.13.1", "0.13.0", "### Fixed\n\n- a fix\n")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("semver-checks", self.cargo_calls())

    def test_a_minor_may_change_the_api(self) -> None:
        r = self.gate("0.14.0", "0.13.0", "### Added\n\n- a feature\n### Changed\n\n- an API\n")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.cargo_calls(), "")

    def test_step_0_runs_it_before_step_1_writes_anything(self) -> None:
        s0 = step(0)
        self.assertIn("bump_gate", s0)
        self.assertRegex(s0, r"for t in [^\n]*cargo-semver-checks")

    def test_the_previous_release_is_never_this_tag(self) -> None:
        # a resumed run (the tag exists) still gates against the release before
        self.assertRegex(step(0), r'PREV_RELEASE=\$\(last_release_tag "\$TAG"\)')

    def test_a_version_that_is_no_release_is_named_not_compared(self) -> None:
        # Codex review of #1743: `[ "rc" -gt 0 ]` is bash's "integer
        # expression expected", which names neither version.
        functions = shell_functions("bump_kind")
        for prev, ver in (("0.14.0-rc.1", "0.14.0"), ("0.13.0", "0.14"), ("0.13.x", "0.14.0")):
            r = subprocess.run(["bash", "-c", f"set -euo pipefail\n{functions}\nbump_kind {prev} {ver}"],
                               capture_output=True, text=True, timeout=30)
            self.assertNotEqual(r.returncode, 0, (prev, ver))
            self.assertEqual(r.stdout, "", (prev, ver))
            self.assertIn("X.Y.Z", r.stderr, (prev, ver))
            self.assertNotIn("integer expression", r.stderr, (prev, ver))


class PreviousRelease(unittest.TestCase):
    """Codex review of #1743: the release the bump, the CHANGELOG and the API
    drift are measured from is the newest plain vX.Y.Z tag; a prerelease tag
    (v0.14.0-rc.1) is no release."""

    def setUp(self) -> None:
        self.repo = Path(tempfile.mkdtemp(prefix="prevrel-"))
        self.addCleanup(shutil.rmtree, self.repo, ignore_errors=True)
        self.git = ["git", "-C", str(self.repo), "-c", "user.name=t", "-c", "user.email=t@t"]
        subprocess.run([*self.git, "init", "-q", "-b", "main"], check=True)

    def commit(self, subject: str, *tags: str) -> None:
        subprocess.run([*self.git, "commit", "-q", "--allow-empty", "-m", subject], check=True)
        for tag in tags:
            subprocess.run([*self.git, "tag", "-a", "-m", tag, tag], check=True)

    def last(self, *args: str) -> subprocess.CompletedProcess[str]:
        functions = shell_functions("last_release_tag")
        self.assertIn("last_release_tag()", functions)
        script = f"set -euo pipefail\n{functions}\nlast_release_tag {' '.join(args)}\n"
        return subprocess.run(["bash", "-c", script], cwd=self.repo,
                              capture_output=True, text=True, timeout=30)

    def test_a_prerelease_tag_is_no_release(self) -> None:
        self.commit("0.13.0", "v0.13.0")
        self.commit("candidate", "v0.14.0-rc.1")
        self.commit("fix after the candidate")
        r = self.last('"v0.14.0"')
        self.assertEqual((r.returncode, r.stdout.strip()), (0, "v0.13.0"), r.stderr)

    def test_a_resumed_run_still_finds_the_release_before(self) -> None:
        self.commit("0.13.0", "v0.13.0")
        self.commit("candidate", "v0.14.0-rc.1")
        self.commit("release 0.14.0", "v0.14.0")
        r = self.last('"v0.14.0"')
        self.assertEqual((r.returncode, r.stdout.strip()), (0, "v0.13.0"), r.stderr)
        # without an exclusion it is the newest release, the tag itself on a
        # resumed run (step 6 then keeps the snapshot it finds)
        r = self.last()
        self.assertEqual((r.returncode, r.stdout.strip()), (0, "v0.14.0"), r.stderr)

    def test_no_release_tag_fails(self) -> None:
        self.commit("first")
        self.commit("candidate", "v0.1.0-rc.1")
        self.assertNotEqual(self.last('"v0.1.0"').returncode, 0)

    def test_every_step_takes_the_previous_release_from_it(self) -> None:
        text = RELEASE_SH.read_text()
        body = text[text.index("last_release_tag() "):]
        body = body[body.index("\n}\n"):]
        self.assertNotIn("git describe", body, "a step finds the previous tag by itself")
        self.assertRegex(step(4), r"PREV_TAG=\$\(last_release_tag[ )]")


# stub gh for the changelog completeness check: closing issues and titles of
# the PRs the fixture repository merged.
GH_PR_STUB = "#!/bin/bash\n" + GH_BOUND + r"""case "$*" in
  "pr view 11 "*) printf '21\nfix(edit): a thing\n' ;;
  "pr view 13 "*) printf 'feat(cli): another (#23)\n' ;;
  "pr view 15 "*) printf '25\nperf(convert): faster\n' ;;
  "pr view 31 "*) echo "GraphQL: Could not resolve to a PullRequest with the number of 31. (repository.pullRequest)" >&2; exit 1 ;;
  *) echo "unexpected gh $*" >&2; exit 9 ;;
esac
"""


class ChangelogCompleteness(unittest.TestCase):
    """#1742: the release notes are the CHANGELOG section verbatim; every
    fix/feat/perf/security PR merged since the previous tag is cited in it,
    by its own number or an issue it closes, or skipped on purpose."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="clcomplete-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "bin").mkdir()
        (self.tmp / "bin" / "gh").write_text(GH_PR_STUB)
        (self.tmp / "bin" / "gh").chmod(0o755)
        repo = self.tmp / "repo"
        repo.mkdir()
        git = ["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t"]
        subprocess.run([*git, "init", "-q", "-b", "main"], check=True)
        subprocess.run([*git, "commit", "-q", "--allow-empty", "-m", "base"], check=True)
        subprocess.run([*git, "tag", "v0.1.0"], check=True)
        for subject in ("Merge pull request #11 from o/fix/a-thing",
                        "Merge pull request #12 from o/chore/tidy",
                        "feat(cli): another (#13)",
                        "docs: a page (#14)",
                        "Merge pull request #15 from o/perf/faster",
                        "Merge pull request #16 from o/pre-release-0-1-1",
                        # a PR's own commit, naming the issue it fixes; gh pr
                        # view refuses 31, which is no pull request
                        "fix(edit): an inner commit (#31)"):
            subprocess.run([*git, "commit", "-q", "--allow-empty", "-m", subject], check=True)
        self.repo = repo

    def unlisted(self, section: str) -> subprocess.CompletedProcess[str]:
        (self.repo / "CHANGELOG.md").write_text(
            f"## [0.1.1] - 2026-10-25\n\n{section}\n## [0.1.0] - 2026-10-01\n\n- old (#99)\n"
        )
        functions = shell_functions("changelog_section", "user_facing_prs", "unlisted_prs")
        script = f"set -euo pipefail\nVER=0.1.1\n{functions}\nunlisted_prs v0.1.0\n"
        full = dict(os.environ, PATH=f"{self.tmp / 'bin'}:{os.environ['PATH']}")
        return subprocess.run(["bash", "-c", script], cwd=self.repo, env=full,
                              capture_output=True, text=True, timeout=60)

    def test_only_user_facing_prs_are_owed(self) -> None:
        functions = shell_functions("user_facing_prs")
        r = subprocess.run(["bash", "-c", f"{functions}\nuser_facing_prs v0.1.0"],
                           cwd=self.repo, capture_output=True, text=True, timeout=30)
        self.assertEqual(r.stdout.split(), ["11", "13", "15", "31"], r.stderr)

    def test_a_pr_cited_by_its_closing_issue_or_its_number_is_listed(self) -> None:
        r = self.unlisted("### Fixed\n\n- a thing (#21)\n- another (#13)\n- faster (#15)\n")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(r.stdout.split(), ["31"])

    def test_an_issue_a_fix_commit_names_stays_owed(self) -> None:
        # 0.14.0: #1670 landed inside #1680's atomic stack merge, with no
        # merge commit of its own; its commits' "(#1668)" is all that shows it.
        r = self.unlisted("### Fixed\n\n- a thing (#21)\n- another (#13)\n- faster (#15)\n")
        self.assertEqual(r.stdout.split(), ["31"], r.stderr)
        r = self.unlisted("### Fixed\n\n- a thing (#21)\n- another (#13)\n- faster (#15)\n- inner (#31)\n")
        self.assertEqual(r.stdout.strip(), "", r.stderr)

    def test_a_pr_cited_by_an_issue_its_title_names_is_listed(self) -> None:
        r = self.unlisted("### Fixed\n\n- a thing (#21)\n- another (#23)\n- faster (#25)\n<!-- changelog-skip: #31 -->\n")
        self.assertEqual(r.stdout.strip(), "", r.stderr)

    def test_a_missing_pr_is_named(self) -> None:
        r = self.unlisted("### Fixed\n\n- a thing (#21)\n<!-- changelog-skip: #31 -->\n")
        self.assertEqual(r.stdout.split(), ["13", "15"], r.stderr)

    def test_a_pr_can_be_skipped_on_purpose(self) -> None:
        r = self.unlisted("### Fixed\n\n- a thing (#21)\n<!-- changelog-skip: #13 #15 #31 -->\n")
        self.assertEqual(r.stdout.strip(), "", r.stderr)

    def test_an_older_section_does_not_count(self) -> None:
        r = self.unlisted("### Fixed\n\n- a thing (#21)\n- another (#13)\n")
        self.assertIn("15", r.stdout.split())

    def test_step_2_dies_on_an_unlisted_pr(self) -> None:
        s2 = step(2)
        self.assertIn("unlisted_prs", s2)
        self.assertIn("die", s2.split("unlisted_prs", 1)[1])


# stub gh for the discovery cases: PR 41 is a feature merged from a topic
# branch, PR 43 a fix from a chore branch; PR 45 cannot be read at all.
GH_DISCOVERY_STUB = "#!/bin/bash\n" + GH_BOUND + r"""echo "$*" >> "$CALLS"
case "$*" in
  "pr view 41 "*) printf 'feat(cli): a new thing\n' ;;
  "pr view 43 "*) printf '53\nfix(edit): a thing\n' ;;
  "pr view 45 "*) echo "error connecting to api.github.com" >&2; exit 1 ;;
  *) echo "GraphQL: Could not resolve to a PullRequest" >&2; exit 1 ;;
esac
"""


class ChangelogDiscovery(unittest.TestCase):
    """Codex review of #1743 (#1745): a section that cites no number is an
    empty citation set, not a silent exit; a PR is owed by its branch or by
    its title (the merge commit's body), and a failed discovery is a named
    refusal, never an empty set."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="cldiscover-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "bin").mkdir()
        (self.tmp / "bin" / "gh").write_text(GH_DISCOVERY_STUB)
        (self.tmp / "bin" / "gh").chmod(0o755)
        self.calls = self.tmp / "calls"
        self.repo = self.tmp / "repo"
        self.repo.mkdir()
        self.git = ["git", "-C", str(self.repo), "-c", "user.name=t", "-c", "user.email=t@t"]
        subprocess.run([*self.git, "init", "-q", "-b", "main"], check=True)
        self.commit("base")
        subprocess.run([*self.git, "tag", "v0.1.0"], check=True)

    def commit(self, subject: str, body: str = "") -> None:
        msg = ["-m", subject, *(["-m", body] if body else [])]
        subprocess.run([*self.git, "commit", "-q", "--allow-empty", *msg], check=True)

    def run_fn(self, call: str, section: str = "### Fixed\n\n- a fix\n") -> subprocess.CompletedProcess[str]:
        (self.repo / "CHANGELOG.md").write_text(
            f"## [0.1.1] - 2026-10-25\n\n{section}\n## [0.1.0] - 2026-10-01\n\n- old (#99)\n"
        )
        functions = shell_functions("changelog_section", "user_facing_prs", "unlisted_prs")
        script = f"set -euo pipefail\nVER=0.1.1\n{functions}\n{call}\n"
        full = dict(os.environ, PATH=f"{self.tmp / 'bin'}:{os.environ['PATH']}", CALLS=str(self.calls))
        return subprocess.run(["bash", "-c", script], cwd=self.repo, env=full,
                              capture_output=True, text=True, timeout=60)

    def test_a_section_without_references_and_nothing_owed_passes(self) -> None:
        # Called directly, under errexit: step 2 calls it inside $(…), where
        # bash clears errexit, so the grep that finds no number only bites
        # a direct call.
        self.commit("docs: a page (#14)")
        r = self.run_fn("unlisted_prs v0.1.0; echo done")
        self.assertEqual((r.returncode, r.stdout.strip()), (0, "done"), r.stderr)

    def test_a_section_without_references_names_what_is_owed(self) -> None:
        self.commit("feat(cli): a thing (#13)")
        r = self.run_fn("unlisted_prs v0.1.0; echo done")
        self.assertEqual((r.returncode, r.stdout.split()), (0, ["13", "done"]), r.stderr)

    def test_a_pr_is_owed_by_its_title_whatever_its_branch(self) -> None:
        self.commit("Merge pull request #41 from o/topic/new-thing", "feat(cli): a new thing")
        self.commit("Merge pull request #43 from o/chore/tidy", "fix(edit): a thing")
        self.commit("Merge pull request #47 from o/fix/md-pages", "test(md): a page count")
        self.commit("Merge pull request #48 from o/chore/untrack", "chore: untrack the scan")
        self.commit("Merge pull request #49 from o/pre-release-0-1-1", "Pre-release 0.1.1")
        r = self.run_fn("user_facing_prs v0.1.0")
        self.assertEqual((r.returncode, r.stdout.split()), (0, ["41", "43", "47"]), r.stderr)
        r = self.run_fn("unlisted_prs v0.1.0", "### Fixed\n\n- a thing (#53)\n- pages (#47)\n")
        self.assertEqual((r.returncode, r.stdout.split()), (0, ["41"]), r.stderr)

    def test_a_pr_gh_cannot_read_is_a_named_refusal(self) -> None:
        self.commit("fix(edit): unreadable (#45)")
        r = self.run_fn("unlisted_prs v0.1.0")
        self.assertNotEqual(r.returncode, 0, r.stdout)
        self.assertIn("#45", r.stderr)
        self.assertIn("error connecting", r.stderr)

    def test_a_history_git_cannot_read_is_a_named_refusal(self) -> None:
        r = self.run_fn("unlisted_prs v9.9.9")
        self.assertNotEqual(r.returncode, 0, r.stdout)
        self.assertIn("v9.9.9", r.stderr)

    def test_gh_reads_this_repository_whatever_GH_REPO_says(self) -> None:
        self.commit("feat(cli): a new thing (#41)")
        r = self.run_fn("unlisted_prs v0.1.0")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("-R jandira-tech/jubarte-redlines", self.calls.read_text())

    def test_step_2_dies_when_discovery_fails(self) -> None:
        self.assertRegex(step(2), r'missing_prs=\$\(unlisted_prs "\$PREV_RELEASE"\) \\\n\s*\|\| die')


# stub gh for the CI cancel: three runs in flight, one of them release.yml.
GH_RUNS_STUB = "#!/bin/bash\n" + GH_BOUND + r"""case "$*" in
  "run list"*) printf '101 CI\n102 Release\n103 Docs\n' ;;
  "run cancel"*) echo "$*" >> "$CALLS" ;;
  *) echo "unexpected gh $*" >&2; exit 9 ;;
esac
"""


class CiCancel(unittest.TestCase):
    """Queued CI is cancelled right before the tag goes up, so release.yml
    gets the runners first; the push to main has just started CI again."""

    def test_every_run_but_the_release_is_cancelled(self) -> None:
        tmp = Path(tempfile.mkdtemp(prefix="cicancel-"))
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        (tmp / "gh").write_text(GH_RUNS_STUB)
        (tmp / "gh").chmod(0o755)
        calls = tmp / "calls"
        functions = shell_functions("cancel_queued_ci")
        script = f'set -euo pipefail\nstep() {{ echo "  - $*"; }}\n{functions}\ncancel_queued_ci\n'
        r = subprocess.run(["bash", "-c", script],
                           env=dict(os.environ, PATH=f"{tmp}:{os.environ['PATH']}", CALLS=str(calls),
                                    GH_REPO="other/repo"),
                           capture_output=True, text=True, timeout=30)
        self.assertEqual(r.returncode, 0, r.stderr)
        cancelled = calls.read_text().split("\n")
        self.assertIn("run cancel 101", cancelled[0])
        self.assertIn("run cancel 103", calls.read_text())
        self.assertNotIn("102", calls.read_text())

    def test_it_runs_after_main_and_before_the_tag(self) -> None:
        text = RELEASE_SH.read_text()
        start = text.index('say "POINT OF NO RETURN"')
        tail = text[start:text.index('say "9.', start)]
        self.assertLess(tail.index("push_main "), tail.index("cancel_queued_ci"))
        self.assertLess(tail.index("cancel_queued_ci"), tail.index('git push origin "$TAG"'))


GH_ATTEST_STUB = "#!/bin/sh\n" + GH_BOUND + """echo "$*" >> "$CALLS"
case "$1 $2" in
  "release download")
    dir=""; pat=""
    while [ $# -gt 0 ]; do
      case "$1" in -D) dir=$2; shift ;; -p) pat=$2; shift ;; esac
      shift
    done
    mkdir -p "$dir" && : > "$dir/$pat"
    exit "${DOWNLOAD_RC:-0}" ;;
  "attestation verify")
    # the attestation names the ref and commit release.yml ran on
    case " $* " in
      *" --source-ref ${ATTESTED_REF:-refs/tags/v9.9.9} --source-digest ${ATTESTED_SHA:-$TAG_SHA} "*) ;;
      *) exit 1 ;;
    esac
    exit "${ATTEST_RC:-0}" ;;
esac
exit 0
"""


class BuildProvenance(unittest.TestCase):
    """release.yml attests every asset (actions/attest); step 12 downloads one
    archive and verifies it against the release workflow that signed it."""

    def run_attested(self, rc: int, **env: str) -> tuple[subprocess.CompletedProcess, str]:
        tmp = Path(tempfile.mkdtemp(prefix="attest-"))
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        self.work = tmp
        (tmp / "gh").write_text(GH_ATTEST_STUB)
        (tmp / "gh").chmod(0o755)
        git = ["git", "-C", str(tmp), "-c", "user.name=t", "-c", "user.email=t@t"]
        subprocess.run([*git, "init", "-q", "-b", "main"], check=True)
        subprocess.run([*git, "commit", "-q", "--allow-empty", "-m", "release"], check=True)
        subprocess.run([*git, "tag", "-a", "-m", "v9.9.9", "v9.9.9"], check=True)
        self.sha = subprocess.run([*git, "rev-parse", "HEAD"], capture_output=True, text=True,
                                  check=True).stdout.strip()
        calls = tmp / "calls"
        script = f'set -euo pipefail\nTAG=v9.9.9 VER=9.9.9\n{shell_functions("gh_attested")}\ngh_attested\n'
        r = subprocess.run(["bash", "-c", script], cwd=tmp,
                           env=dict(os.environ, PATH=f"{tmp}:{os.environ['PATH']}",
                                    CALLS=str(calls), ATTEST_RC=str(rc), TAG_SHA=self.sha, **env),
                           capture_output=True, text=True, timeout=30)
        return r, calls.read_text() if calls.exists() else ""

    def test_an_attested_archive_passes(self) -> None:
        r, calls = self.run_attested(0)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("release download v9.9.9 -p jubarte-9.9.9-linux-x86_64.tar.gz", calls)
        verify = [c for c in calls.splitlines() if c.startswith("attestation verify")]
        self.assertEqual(len(verify), 1, calls)
        self.assertIn("-R jandira-tech/jubarte-redlines", verify[0])
        self.assertIn("--signer-workflow jandira-tech/jubarte-redlines/.github/workflows/release.yml",
                      verify[0])
        self.assertIn("--deny-self-hosted-runners", verify[0])
        self.assertIn("--source-ref refs/tags/v9.9.9", verify[0])
        self.assertIn(f"--source-digest {self.sha}", verify[0])

    def test_an_older_releases_archive_under_this_name_fails(self) -> None:
        # Codex review of #1743 (#1749): the repository, workflow and runner
        # alone pass an archive another tag's run attested.
        r, _ = self.run_attested(0, ATTESTED_REF="refs/tags/v9.9.8")
        self.assertNotEqual(r.returncode, 0)
        r, _ = self.run_attested(0, ATTESTED_SHA="0" * 40)
        self.assertNotEqual(r.returncode, 0)

    def test_a_failed_download_fails_and_leaves_nothing_behind(self) -> None:
        # #1768: the early return skipped the cleanup
        r, _ = self.run_attested(0, DOWNLOAD_RC="1")
        self.assertNotEqual(r.returncode, 0)
        self.assertFalse((self.work / "dist" / "attest").exists())

    def test_an_unattested_archive_fails(self) -> None:
        r, _ = self.run_attested(1)
        self.assertNotEqual(r.returncode, 0)

    def test_step_12_reports_provenance_without_failing_the_release(self) -> None:
        # A release made from run artifacts (release.yml's release job did
        # not run) has no attestation; that is a warning, not a failure.
        s12 = step(12)
        self.assertIn("gh_attested", s12)
        self.assertNotRegex(s12, r'check "[^"]*"\s+"?(eventually )?gh_attested')


# stub gh for the fallback release: the run's artifacts, and the notes the
# release is created with (copied to $NOTES_OUT).
GH_FALLBACK_STUB = "#!/bin/bash\n" + GH_BOUND + r"""case "$1 $2" in
  "run download")
    mkdir -p dist/release-art/x
    for t in macosx_10_12_x86_64 macosx_11_0_arm64 manylinux_2_28_x86_64 \
             manylinux_2_28_aarch64 musllinux_1_2_x86_64 musllinux_1_2_aarch64 win_amd64; do
      : > "dist/release-art/x/jubarte_redlines-9.9.9-cp310-abi3-$t.whl"
    done
    : > dist/release-art/x/jubarte_redlines-9.9.9.tar.gz
    for a in linux-x86_64 linux-aarch64 macos-x86_64 macos-aarch64; do
      : > "dist/release-art/x/jubarte-9.9.9-$a.tar.gz"
    done
    : > dist/release-art/x/jubarte-9.9.9-windows-x86_64.zip ;;
  "release create")
    while [ $# -gt 0 ]; do
      [ "$1" = --notes-file ] && cp "$2" "$NOTES_OUT"
      shift
    done ;;
  *) echo "unexpected gh $*" >&2; exit 9 ;;
esac
"""


class FallbackProvenance(unittest.TestCase):
    """Codex review of #1743 (#1749): a release made from run artifacts
    (release.yml's release job did not run) has no attestation, and its
    notes say so instead of the CHANGELOG's claim standing alone."""

    def test_the_fallback_notes_disclose_no_attestation(self) -> None:
        tmp = Path(tempfile.mkdtemp(prefix="fallback-"))
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        (tmp / "bin").mkdir()
        (tmp / "bin" / "gh").write_text(GH_FALLBACK_STUB)
        (tmp / "bin" / "gh").chmod(0o755)
        (tmp / "scripts").mkdir()
        shutil.copy(HERE / "check_release_artifacts.py", tmp / "scripts")
        (tmp / "CHANGELOG.md").write_text("## [9.9.9] - 2026-10-25\n\n- a fix\n")
        notes = tmp / "notes.md"
        script = (
            "set -euo pipefail\nTAG=v9.9.9; VER=9.9.9; PREV_TAG=v9.9.8; GITHUB_SUMMARY='the summary'\n"
            'step() { echo "  - $*"; }\ndie() { echo "ERROR: $*" >&2; exit 1; }\n'
            f'{shell_functions("release_from_artifacts")}\nrelease_from_artifacts 42\n'
        )
        r = subprocess.run(["bash", "-c", script], cwd=tmp,
                           env=dict(os.environ, PATH=f"{tmp / 'bin'}:{os.environ['PATH']}",
                                    NOTES_OUT=str(notes)),
                           capture_output=True, text=True, timeout=60)
        self.assertEqual(r.returncode, 0, r.stderr)
        text = notes.read_text()
        self.assertTrue(text.startswith("the summary"), text)
        # #1768: the release job may have attested these bytes before it
        # died, so the note says only what is known
        self.assertIn("were not published by release.yml's release job", text)
        self.assertIn("gh attestation verify", text)
        self.assertNotIn("carry no build-provenance attestation", text)
        self.assertIn("- a fix", text)

    def test_the_changelog_claims_provenance_for_the_release_job_only(self) -> None:
        text = (HERE.parent / "CHANGELOG.md").read_text()
        claim = next(line for line in text.splitlines() if line.startswith("- **Build provenance"))
        self.assertIn("release job", claim)


# stub gh api for the security gate: each endpoint answers the JSON fixture
# $FIX/<kind>.json through the real jq, so the -q filters themselves run.
GH_SECURITY_STUB = r"""#!/bin/bash
[ "$1" = api ] || { echo "unexpected gh $*" >&2; exit 9; }
case "$2" in
  repos/jandira-tech/jubarte-redlines) kind=repo ;;
  repos/jandira-tech/jubarte-redlines/security-advisories*) kind=advisories ;;
  repos/jandira-tech/jubarte-redlines/dependabot/alerts*) kind=dependabot ;;
  repos/jandira-tech/jubarte-redlines/code-scanning/alerts*) kind=code-scanning ;;
  *) echo "unexpected gh $*" >&2; exit 9 ;;
esac
[ "${FAIL:-}" != "$kind" ] || { echo "HTTP 403: Resource not accessible" >&2; exit 1; }
q=""
while [ $# -gt 0 ]; do [ "$1" = -q ] && q=$2; shift; done
jq -r "$q" "$FIX/$kind.json"
"""


@unittest.skipUnless(shutil.which("jq"), "jq is not installed")
class SecurityGate(unittest.TestCase):
    """#1750 (checklist before#1, its automated half): SECURITY.md lets no
    confirmed medium-or-higher finding ship. Step 0 stops on a repository
    advisory in triage (a private report) or an open medium+ Dependabot or
    code-scanning alert, and lists drafts and lower alerts."""

    def gate(self, advisories=(), dependabot=(), code=(), admin=True, **env: str) -> subprocess.CompletedProcess[str]:
        tmp = Path(tempfile.mkdtemp(prefix="secgate-"))
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        (tmp / "gh").write_text(GH_SECURITY_STUB)
        (tmp / "gh").chmod(0o755)
        import json
        (tmp / "repo.json").write_text(json.dumps({"permissions": {"admin": admin, "push": True}}))
        (tmp / "advisories.json").write_text(json.dumps(list(advisories)))
        (tmp / "dependabot.json").write_text(json.dumps(list(dependabot)))
        (tmp / "code-scanning.json").write_text(json.dumps(list(code)))
        script = (
            "set -euo pipefail\n"
            'step() { echo "  - $*"; }\ndie() { echo "ERROR: $*" >&2; exit 1; }\n'
            f'{shell_functions("security_findings", "security_blocking", "security_gate")}\nsecurity_gate\n'
        )
        return subprocess.run(["bash", "-c", script], cwd=tmp,
                              env=dict(os.environ, PATH=f"{tmp}:{os.environ['PATH']}", FIX=str(tmp),
                                       **{"VER": "9.9.9", **env}),
                              capture_output=True, text=True, timeout=30)

    @staticmethod
    def draft(ghsa: str, severity: str | None, *patched: str | None) -> dict:
        return {"ghsa_id": ghsa, "state": "draft", "severity": severity,
                "vulnerabilities": [{"package": {"ecosystem": "rust", "name": "jubarte-redlines"},
                                     "patched_versions": v} for v in patched]}

    @staticmethod
    def alert(n: int, severity: str | None) -> dict:
        return {"number": n, "state": "open",
                "security_advisory": {"severity": severity, "ghsa_id": f"GHSA-dep{n}"},
                "dependency": {"package": {"name": "opentelemetry-api"}}}

    @staticmethod
    def scan(n: int, severity: str | None) -> dict:
        return {"number": n, "state": "open",
                "rule": {"id": "js/xss", "security_severity_level": severity},
                "most_recent_instance": {"location": {"path": "src/x.rs"}}}

    def test_nothing_open_passes(self) -> None:
        r = self.gate(advisories=[{"ghsa_id": "GHSA-old", "state": "published", "severity": "high"}])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertNotIn("GHSA-old", r.stdout)

    def test_a_private_report_in_triage_stops_the_release(self) -> None:
        r = self.gate(advisories=[{"ghsa_id": "GHSA-tri1", "state": "triage", "severity": None}])
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("GHSA-tri1", r.stderr)

    def test_a_low_draft_advisory_is_listed_not_blocking(self) -> None:
        r = self.gate(advisories=[self.draft("GHSA-drf1", "low", None)])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("GHSA-drf1", r.stdout)

    def test_an_unpatched_high_draft_stops_the_release(self) -> None:
        # #1761: moving a report from triage to draft fixed nothing
        r = self.gate(advisories=[self.draft("GHSA-drf2", "high", None)])
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("GHSA-drf2", r.stderr)
        self.assertIn("patched version", r.stderr)

    def test_an_unrated_draft_stops_the_release(self) -> None:
        r = self.gate(advisories=[self.draft("GHSA-drf3", None)])
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("GHSA-drf3", r.stderr)

    def test_a_draft_this_release_patches_is_listed_not_blocking(self) -> None:
        # the release that is the fix ships before the advisory is published
        for patched in ("9.9.9", ">= 9.9.9"):
            r = self.gate(advisories=[self.draft("GHSA-drf4", "critical", None, patched)])
            self.assertEqual(r.returncode, 0, (patched, r.stderr))
            self.assertIn("GHSA-drf4", r.stdout)

    def test_a_draft_another_version_patches_stops_the_release(self) -> None:
        for patched in ("9.9.10", "19.9.9", "9.9.9-rc.1", "999"):
            r = self.gate(advisories=[self.draft("GHSA-drf5", "medium", patched)])
            self.assertEqual(r.returncode, 1, (patched, r.stdout))

    def test_a_token_that_cannot_see_private_reports_is_refused(self) -> None:
        # GitHub lists unpublished advisories only to admins and security
        # managers; any other token gets the published ones, successfully
        r = self.gate(admin=False)
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("admin", r.stderr)
        self.assertIn("unpublished", r.stderr)

    def test_an_open_medium_dependabot_alert_stops_the_release(self) -> None:
        r = self.gate(dependabot=[self.alert(7, "medium")])
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("#7", r.stderr)
        self.assertIn("opentelemetry-api", r.stderr)

    def test_a_low_dependabot_alert_is_listed_not_blocking(self) -> None:
        r = self.gate(dependabot=[self.alert(8, "low")])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("#8", r.stdout)

    def test_an_open_high_code_scanning_alert_stops_the_release(self) -> None:
        r = self.gate(code=[self.scan(45, "high"), self.scan(46, None)])
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("#45", r.stderr)
        self.assertNotIn("#46", r.stderr.split("ERROR")[-1])

    def test_an_unreadable_api_is_a_refusal(self) -> None:
        for kind in ("repo", "advisories", "dependabot", "code-scanning"):
            r = self.gate(FAIL=kind)
            self.assertEqual(r.returncode, 1, (kind, r.stdout))
            self.assertIn("cannot read", r.stderr, kind)

    def test_step_0_runs_it_before_the_bump_gate(self) -> None:
        s0 = step(0)
        self.assertLess(s0.index("\nsecurity_gate\n"), s0.index("bump_gate \"$PREV_RELEASE\""))


# stub gh api for HEAD's check runs: the JSON fixture $FIX/runs.json through
# the real jq, or an unreadable API when $FAIL is set.
GH_CHECKS_STUB = r"""#!/bin/bash
case "$1 $2" in
  "api repos/jandira-tech/jubarte-redlines/commits/"*/check-runs*) ;;
  *) echo "unexpected gh $*" >&2; exit 9 ;;
esac
[ -z "${FAIL:-}" ] || { echo "HTTP 502" >&2; exit 1; }
q=""
while [ $# -gt 0 ]; do [ "$1" = -q ] && q=$2; shift; done
jq -r "$q" "$FIX/runs.json"
"""


class ReleaseTree(unittest.TestCase):
    """#1751 (checklist before#4 and before#9): the canonical checkout is the
    owner's, never the release tree; and HEAD's CI that did not pass is put
    in front of the releaser before anything is written."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="reltree-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)

    def canonical(self, cwd: Path, canon: str) -> int:
        script = f'set -euo pipefail\n{shell_functions("canonical_checkout")}\ncanonical_checkout\n'
        return subprocess.run(["bash", "-c", script], cwd=cwd,
                              env=dict(os.environ, JUBARTE_CANONICAL_CHECKOUT=canon),
                              capture_output=True, text=True, timeout=30).returncode

    def test_the_canonical_checkout_is_recognised_through_a_link(self) -> None:
        real = self.tmp / "temp" / "jubarte-redlines"
        real.mkdir(parents=True)
        (self.tmp / "T").symlink_to(self.tmp / "temp")
        self.assertEqual(self.canonical(real, str(self.tmp / "T" / "jubarte-redlines")), 0)
        self.assertEqual(self.canonical(self.tmp / "T" / "jubarte-redlines", str(real)), 0)

    def test_a_worktree_elsewhere_is_no_canonical_checkout(self) -> None:
        (self.tmp / "canon").mkdir()
        (self.tmp / "release-wt").mkdir()
        self.assertNotEqual(self.canonical(self.tmp / "release-wt", str(self.tmp / "canon")), 0)
        self.assertNotEqual(self.canonical(self.tmp / "release-wt", str(self.tmp / "absent")), 0)

    def test_step_0_refuses_it_first(self) -> None:
        s0 = step(0)
        first = s0.index("canonical_checkout")
        self.assertLess(first, s0.index("git fetch"))
        self.assertIn("die", s0[first:first + 300])
        self.assertIn("worktree", s0[first:first + 300])

    def ci(self, runs: list[dict], **env: str) -> subprocess.CompletedProcess[str]:
        import json
        (self.tmp / "gh").write_text(GH_CHECKS_STUB)
        (self.tmp / "gh").chmod(0o755)
        (self.tmp / "runs.json").write_text(json.dumps({"check_runs": runs}))
        script = (f'set -euo pipefail\nstep() {{ echo "  - $*"; }}\n'
                  f'git() {{ echo 0123456789abcdef0123456789abcdef01234567; }}\n'
                  f'{shell_functions("head_ci_report")}\nhead_ci_report\necho survived\n')
        return subprocess.run(["bash", "-c", script], cwd=self.tmp,
                              env=dict(os.environ, PATH=f"{self.tmp}:{os.environ['PATH']}", FIX=str(self.tmp), **env),
                              capture_output=True, text=True, timeout=30)

    @unittest.skipUnless(shutil.which("jq"), "jq is not installed")
    def test_heads_red_checks_are_shown_and_the_run_goes_on(self) -> None:
        r = self.ci([{"name": "test (ubuntu)", "status": "completed", "conclusion": "failure"},
                     {"name": "clippy", "status": "completed", "conclusion": "success"},
                     {"name": "coverage", "status": "completed", "conclusion": "cancelled"},
                     {"name": "docs", "status": "in_progress", "conclusion": None},
                     {"name": "fuzz", "status": "completed", "conclusion": "skipped"}])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("survived", r.stdout)
        for name in ("test (ubuntu) failure", "coverage cancelled", "docs in_progress"):
            self.assertIn(name, r.stdout)
        for name in ("clippy", "fuzz"):
            self.assertNotIn(name, r.stdout)

    @unittest.skipUnless(shutil.which("jq"), "jq is not installed")
    def test_all_green_says_so(self) -> None:
        r = self.ci([{"name": "clippy", "status": "completed", "conclusion": "success"}])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("every check run of HEAD passed", r.stdout)

    @unittest.skipUnless(shutil.which("jq"), "jq is not installed")
    def test_no_check_run_at_all_is_named_not_passed(self) -> None:
        # #1763: a HEAD whose workflows never ran read as green
        r = self.ci([])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("survived", r.stdout)
        self.assertIn("CHECK NOW  HEAD has no check run", r.stdout)
        self.assertNotIn("passed", r.stdout.replace("nothing passed", ""))

    @unittest.skipUnless(shutil.which("jq"), "jq is not installed")
    def test_only_skipped_check_runs_are_named_not_passed(self) -> None:
        r = self.ci([{"name": "fuzz", "status": "completed", "conclusion": "skipped"},
                     {"name": "lint", "status": "completed", "conclusion": "neutral"}])
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("survived", r.stdout)
        self.assertIn("CHECK NOW  none of HEAD's 2 check runs succeeded", r.stdout)
        self.assertNotIn("every check run of HEAD passed", r.stdout)

    def test_an_unreadable_ci_is_named_not_fatal(self) -> None:
        r = self.ci([], FAIL="1")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("survived", r.stdout)
        self.assertIn("cannot read", r.stdout + r.stderr)

    def test_step_0_reports_it(self) -> None:
        self.assertIn("\nhead_ci_report\n", step(0))


class AutomatedReview(unittest.TestCase):
    """#1742: checks a script can make are made by the script, not owed by a
    person: the release.yml review, the scripts' Python floor, the crate's
    keyword limits, and the API sign-off (now cargo-semver-checks)."""

    def test_step_0_checks_release_yml(self) -> None:
        self.assertIn("scripts/test_release_workflow.py", step(0))

    def test_step_0_checks_the_release_environment_and_the_signing_secret(self) -> None:
        s0 = step(0)
        self.assertIn("environments/release", s0)
        self.assertIn("ZIPSIGN_PRIVATE_KEY", s0)

    def test_the_script_tests_run_on_the_python_floor(self) -> None:
        self.assertIn("--python 3.12", step(5))

    def test_step_7_checks_the_crate_keyword_limits(self) -> None:
        self.assertIn("crate_keywords_ok", step(7))

    def test_the_keyword_check_counts_against_crates_io(self) -> None:
        tmp = Path(tempfile.mkdtemp(prefix="kw-"))
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        functions = shell_functions("crate_keywords_ok")
        for keywords, categories, ok in (
            ('["docx", "redline"]', '["text-processing"]', True),
            ('["a", "b", "c", "d", "e", "f"]', '["text-processing"]', False),
            ('["docx"]', '["a", "b", "c", "d", "e", "f"]', False),
            ('["a-keyword-that-is-far-too-long"]', '["text-processing"]', False),
        ):
            (tmp / "Cargo.toml").write_text(
                f'[package]\nname = "x"\nversion = "0.1.0"\nkeywords = {keywords}\ncategories = {categories}\n'
            )
            r = subprocess.run(["bash", "-c", f"{functions}\ncrate_keywords_ok"], cwd=tmp,
                               capture_output=True, text=True, timeout=30)
            self.assertEqual(r.returncode == 0, ok, (keywords, categories, r.stderr))

    def test_the_rendered_docs_open_only_on_request(self) -> None:
        s6 = step(6)
        self.assertIn('RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items', s6)
        self.assertRegex(s6, r'if \[ "\$OPEN_DOCS" = 1 \]')


class ChangelogPlaceholders(unittest.TestCase):
    """The section ships verbatim as the GitHub release notes: a TODO in it
    stops the release at step 2, a word that merely contains one does not."""

    def found(self, section: str) -> subprocess.CompletedProcess[str]:
        tmp = Path(tempfile.mkdtemp(prefix="clph-"))
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        (tmp / "CHANGELOG.md").write_text(f"## [0.1.1] - 2026-10-25\n\n{section}\n## [0.1.0] - 2026-10-01\n\n- TODO old\n")
        functions = shell_functions("changelog_section", "changelog_placeholders")
        return subprocess.run(["bash", "-c", f"VER=0.1.1\n{functions}\nchangelog_placeholders"],
                              cwd=tmp, capture_output=True, text=True, timeout=30)

    def test_a_placeholder_is_found_with_its_line(self) -> None:
        for word in ("TODO", "TBD", "FIXME", "XXX"):
            r = self.found(f"### Fixed\n\n- {word}: describe the fix\n")
            self.assertEqual(r.returncode, 0, word)
            self.assertIn(word, r.stdout)

    def test_a_word_containing_one_is_not(self) -> None:
        r = self.found("### Fixed\n\n- TODOS list and XXXL sizes render\n")
        self.assertNotEqual(r.returncode, 0, r.stdout)

    def test_step_2_dies_on_one(self) -> None:
        s2 = step(2)
        self.assertIn("changelog_placeholders", s2)
        self.assertIn("die", s2.split("changelog_placeholders", 1)[1])


# gh at a command's start: the subcommands that act on one repository.
GH_REPO_CALL = re.compile(
    r"(?:^|[;&|(!{]|\$\(|\bif\b|\bthen\b|\bdo\b)\s*gh\s+"
    r"(?:release|run|pr|secret|attestation|workflow|issue|repo|cache|variable)\b")


def shell_commands(path: Path) -> list[str]:
    """The logical commands of a shell script: comment lines and heredoc
    bodies dropped, backslash continuations joined, and any message text
    (die/step/echo/printf) cut off, since no gh runs inside one."""
    out, buf, heredoc = [], "", None
    for line in path.read_text().splitlines():
        if heredoc is not None:
            if line.strip() == heredoc:
                heredoc = None
            continue
        if line.lstrip().startswith("#"):
            continue
        if line.rstrip().endswith("\\"):
            buf += line.rstrip()[:-1] + " "
            continue
        buf += line
        m = re.search(r"<<-?\s*['\"]?(\w+)['\"]?", buf.replace("<<<", ""))
        out.append(re.split(r"\b(?:die|step|echo|printf) [\"']", buf)[0])
        buf = ""
        if m:
            heredoc = m.group(1)
    return out


class EngineRepoBinding(unittest.TestCase):
    """Codex review of #1743 (#1747): gh honours GH_REPO, so a stray one in
    the releaser's shell cancelled, listed or created things in another
    repository. Every engine gh call names jandira-tech/jubarte-redlines;
    the app's step gets no GH_REPO at all."""

    def test_every_engine_gh_call_names_the_repository(self) -> None:
        unbound = []
        for script in (RELEASE_SH, HERE / "push_main.sh"):
            for cmd in shell_commands(script):
                starts = [m.end() for m in GH_REPO_CALL.finditer(cmd)]
                for i, start in enumerate(starts):
                    seg = cmd[start:starts[i + 1] if i + 1 < len(starts) else len(cmd)]
                    if "-R jandira-tech/jubarte-redlines" not in seg:
                        unbound.append(f"{script.name}: {cmd.strip()}")
        self.assertEqual(unbound, [])

    def test_the_scan_sees_the_calls(self) -> None:
        # the scan would pass vacuously on a script it cannot read
        calls = [c for c in shell_commands(RELEASE_SH) if GH_REPO_CALL.search(c)]
        self.assertGreaterEqual(len(calls), 12, calls)
        self.assertTrue(any("gh run cancel" in c for c in calls))
        self.assertTrue(any(c.startswith("ghrel_has()") for c in calls), "a one-line function's call")
        self.assertFalse(any("rerun <id>" in c for c in calls), "a die message is no call")

    def test_a_stray_GH_REPO_never_reaches_the_app_step(self) -> None:
        tmp = Path(tempfile.mkdtemp(prefix="appgh-"))
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        (tmp / "scripts").mkdir()
        (tmp / "scripts" / "release-engine.sh").write_text('#!/bin/sh\necho "GH_REPO=${GH_REPO-unset}"\n')
        (tmp / "scripts" / "release-engine.sh").chmod(0o755)
        text = RELEASE_SH.read_text()
        app = re.search(r"^app_release\(\) .*$", text, re.M)
        self.assertIsNotNone(app)
        script = f"set -euo pipefail\nAPP_REPO={tmp}; VER=9.9.9\n{app.group(0)}\napp_release --preflight\n"
        r = subprocess.run(["bash", "-c", script], env=dict(os.environ, GH_REPO="other/repo"),
                           capture_output=True, text=True, timeout=30)
        self.assertEqual((r.returncode, r.stdout.strip()), (0, "GH_REPO=unset"), r.stderr)



class AdoptionAndHeaders(unittest.TestCase):
    """#1754 (checklist 4#2 and 5#1): a docs/adoption row this release ships
    is not still labelled main, and every tracked file that can carry a
    comment carries its SPDX header — `reuse lint` cannot see a missing one,
    because REUSE.toml's `path = "**"` covers every file."""

    # REUSE-IgnoreStart (headers these tests write, not this file's)
    HEADER = "# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC\n#\n# SPDX-License-Identifier: AGPL-3.0-only\n"
    LATE = "//! docs\n" + "// filler\n" * 600 + "// SPDX-License-Identifier: AGPL-3.0-only\n"
    # REUSE-IgnoreEnd

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="adoption-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)

    def run_fn(self, name: str) -> subprocess.CompletedProcess[str]:
        script = f"set -euo pipefail\n{shell_functions(name)}\nout=$({name}) || exit 7\nprintf '%s' \"$out\"\n"
        return subprocess.run(["bash", "-c", script], cwd=self.tmp, capture_output=True, text=True, timeout=60)

    def write(self, rel: str, text: str) -> None:
        (self.tmp / rel).parent.mkdir(parents=True, exist_ok=True)
        (self.tmp / rel).write_text(text)

    def test_a_row_still_labelled_main_is_listed(self) -> None:
        self.write("docs/adoption/a.md", "| Tool | jubarte | Status |\n|---|---|---|\n"
                   "| pandoc | `jubarte read` | released |\n| soffice | `jubarte convert` |  main |\n")
        self.write("docs/adoption/README.md", "- **main**: merged, not yet released, on `main`.\n")
        r = self.run_fn("adoption_main_rows")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("a.md:4:", r.stdout)
        self.assertNotIn("pandoc", r.stdout)
        self.assertNotIn("README", r.stdout)

    def test_released_and_pending_rows_pass(self) -> None:
        self.write("docs/adoption/a.md", "| x | y | released |\n| z | `main` branch | pending |\n")
        r = self.run_fn("adoption_main_rows")
        self.assertEqual((r.returncode, r.stdout), (0, ""), r.stderr)

    def test_a_wrapped_main_label_is_listed(self) -> None:
        # #1764: the label in code or bold slipped past a bare-word match
        self.write("docs/adoption/a.md", "| x | y | `main` |\n| z | w | **main** |\n| v | u | released |\n")
        r = self.run_fn("adoption_main_rows")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual([line.split(":")[1] for line in r.stdout.splitlines()], ["1", "2"])

    def test_no_adoption_pages_is_a_refusal_not_a_pass(self) -> None:
        self.assertEqual(self.run_fn("adoption_main_rows").returncode, 7)

    def git_repo(self) -> None:
        git = ["git", "-C", str(self.tmp), "-c", "user.name=t", "-c", "user.email=t@t"]
        subprocess.run([*git, "init", "-q"], check=True)
        subprocess.run([*git, "add", "-A"], check=True)

    def test_a_tracked_file_without_its_header_is_listed(self) -> None:
        self.write("scripts/ok.py", "#!/usr/bin/env python3\n" + self.HEADER + "print(1)\n")
        self.write("scripts/bare.py", "#!/usr/bin/env python3\nprint(1)\n")
        self.write("tests/late.rs", self.LATE)
        self.write("scripts/probe.zsh", "#!/usr/bin/env zsh\necho hi\n")
        self.write("docs/notes.md", "no header needed\n")
        self.git_repo()
        self.write("scripts/untracked.py", "print(1)\n")  # not in the release
        r = self.run_fn("headerless_files")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(sorted(r.stdout.split()), ["scripts/bare.py", "scripts/probe.zsh", "tests/late.rs"])

    def test_commentable_config_files_are_asked_for_one(self) -> None:
        # #1764: jubarte-wasm/Cargo.toml and a pyproject.toml carried none
        self.write("ok/Cargo.toml", self.HEADER + "[package]\n")
        for rel in ("jubarte-wasm/Cargo.toml", "eval/pyproject.toml", ".github/workflows/ci.yml",
                    "action.yaml", "stubs/x.pyi", "setup.cfg", "tox.ini"):
            self.write(rel, "x = 1\n")
        self.git_repo()
        r = self.run_fn("headerless_files")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(sorted(r.stdout.split()),
                         sorted([".github/workflows/ci.yml", "action.yaml", "eval/pyproject.toml",
                                 "jubarte-wasm/Cargo.toml", "setup.cfg", "stubs/x.pyi", "tox.ini"]))

    def test_generated_files_and_fixtures_are_not_asked_for_one(self) -> None:
        # wasm-bindgen writes npm/'s glue and docs/api snapshots it; REUSE.toml
        # carries tests/fixtures/' provenance (anymd's copied scripts among them)
        self.write("jubarte-wasm/npm/web/jubarte_wasm.js", "export {}\n")
        self.write("jubarte-wasm/npm/node/jubarte_wasm_bg.wasm.d.ts", "export {}\n")
        self.write("docs/api/jubarte-wasm-node-v0.13.0.d.ts", "export {}\n")
        self.write("tests/fixtures/from-docx/critic/synthesize.py", "print(1)\n")
        self.write("jubarte-wasm/npm/build.mjs", "export {}\n")  # hand-written beside them
        self.git_repo()
        r = self.run_fn("headerless_files")
        self.assertEqual((r.returncode, r.stdout.split()), (0, ["jubarte-wasm/npm/build.mjs"]), r.stderr)

    def test_this_tree_has_no_headerless_file(self) -> None:
        script = f"set -euo pipefail\n{shell_functions('headerless_files')}\nheaderless_files\n"
        r = subprocess.run(["bash", "-c", script], cwd=RELEASE_SH.parent.parent,
                           capture_output=True, text=True, timeout=120)
        self.assertEqual((r.returncode, r.stdout), (0, ""), r.stderr)

    def test_steps_4_and_5_die_on_them(self) -> None:
        # step 5's header check sits beside the security scan, above the
        # gates block, so --skip-gates does not skip it
        text = RELEASE_SH.read_text()
        s4 = step(4)
        s5 = text[text.index("\nbash scripts/security-check.sh\n"):text.index('if [ "$SKIP_GATES" = 0 ]; then\n  say "5.')]
        self.assertIn("adoption_main_rows", s4)
        self.assertIn("die", s4[s4.index("adoption_main_rows"):][:600])
        self.assertIn("headerless_files", s5)
        self.assertIn("die", s5[s5.index("headerless_files"):][:600])
        self.assertIn("4|auto|docs/adoption", RELEASE_SH.read_text())



class TagShape(unittest.TestCase):
    """#1755 (readiness 12#1, 12#2): the annotated tag release.yml reads has
    the subject `jubarte vX.Y.Z` and the github summary in its body. 0.13.0's
    tag held the summary as its subject with an empty body: release.yml
    prepended nothing, and step 12's gh_note could never pass."""

    SUMMARY = "-- New: a flag; PDFs mean 79.19 (LibreOffice 54.61)."

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="tagshape-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.git = ["git", "-C", str(self.tmp), "-c", "user.name=t", "-c", "user.email=t@t"]
        subprocess.run([*self.git, "init", "-q"], check=True)
        subprocess.run([*self.git, "commit", "-q", "--allow-empty", "-m", "release"], check=True)

    def shape(self) -> int:
        script = f"set -euo pipefail\n{shell_functions('tag_shape_ok')}\ntag_shape_ok\n"
        return subprocess.run(["bash", "-c", script], cwd=self.tmp, capture_output=True, text=True, timeout=30,
                              env=dict(os.environ, TAG="v9.9.9", GITHUB_SUMMARY=self.SUMMARY)).returncode

    def tag(self, *args: str) -> None:
        subprocess.run([*self.git, "tag", *args], check=True)

    def test_the_shape_release_sh_writes_passes(self) -> None:
        self.tag("-a", "v9.9.9", "-m", "jubarte v9.9.9", "-m", self.SUMMARY)
        self.assertEqual(self.shape(), 0)

    def test_the_summary_as_subject_with_an_empty_body_fails(self) -> None:
        self.tag("-a", "v9.9.9", "-m", self.SUMMARY)  # 0.13.0's shape
        self.assertNotEqual(self.shape(), 0)

    def test_another_subject_fails(self) -> None:
        self.tag("-a", "v9.9.9", "-m", "v9.9.9", "-m", self.SUMMARY)
        self.assertNotEqual(self.shape(), 0)

    def test_a_lightweight_tag_fails(self) -> None:
        self.tag("v9.9.9")
        self.assertNotEqual(self.shape(), 0)

    def test_no_tag_fails(self) -> None:
        self.assertNotEqual(self.shape(), 0)

    def test_step_8_checks_every_tag_it_keeps_or_writes(self) -> None:
        s8 = step(8)
        block = s8[s8.index("tag_shape_ok"):s8.index("check_engine_commit.sh")]
        self.assertGreaterEqual(block.count("tag_shape_ok"), 2, "the kept tag, then the tag written")
        # a tag on origin is the one release.yml reads: it must be the local
        # one, and a wrong shape there dies unless the notes were put right
        self.assertIn("ls-remote", block)
        self.assertIn("gh_note || {", block)
        self.assertNotIn("release notes will lack it", s8)
        # step(8) ends at POINT OF NO RETURN, before main or the tag is pushed
        self.assertNotIn("push_main", s8)

    def block(self, *, notes_ok: bool, release: bool = True) -> subprocess.CompletedProcess[str]:
        """Step 8's tag block, run as written against a bare origin; gh
        answers `release view` as `release` says."""
        text = step(8)
        start = text.index("remote_tag=$(git ls-remote")
        body = text[start:text.index("# A resumed run keeps the npm build", start)]
        script = ("set -euo pipefail\n" + shell_functions("tag_shape_ok")
                  + 'die() { echo "DIE: $*"; exit 9; }\nstep() { echo "  - $*"; }\n'
                  + f"gh_note() {{ {'true' if notes_ok else 'false'}; }}\n"
                  + f'gh() {{ [ "$1 $2" = "release view" ] || exit 8; {"true" if release else "false"}; }}\n'
                  + body + "echo survived\n")
        return subprocess.run(["bash", "-c", script], cwd=self.tmp, capture_output=True, text=True, timeout=60,
                              env=dict(os.environ, TAG="v9.9.9", GITHUB_SUMMARY=self.SUMMARY,
                                       GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@t"))

    def origin(self) -> None:
        bare = self.tmp.parent / (self.tmp.name + "-origin.git")
        self.addCleanup(shutil.rmtree, bare, ignore_errors=True)
        subprocess.run(["git", "init", "-q", "--bare", str(bare)], check=True)
        subprocess.run([*self.git, "remote", "add", "origin", str(bare)], check=True)

    def push_tag(self) -> None:
        subprocess.run([*self.git, "push", "-q", "origin", "refs/tags/v9.9.9"], check=True)

    def test_the_block_writes_a_fresh_tag_in_shape(self) -> None:
        self.origin()
        r = self.block(notes_ok=False)
        self.assertIn("survived", r.stdout, r.stdout + r.stderr)
        self.assertEqual(self.shape(), 0)

    def test_the_block_recreates_a_local_tag_of_the_wrong_shape(self) -> None:
        self.origin()
        self.tag("-a", "v9.9.9", "-m", self.SUMMARY)  # 0.13.0's shape, not pushed
        r = self.block(notes_ok=False)
        self.assertIn("re-creating", r.stdout, r.stdout + r.stderr)
        self.assertEqual(self.shape(), 0)

    def test_a_pushed_tag_of_the_wrong_shape_dies_unless_the_notes_carry_it(self) -> None:
        self.origin()
        self.tag("-a", "v9.9.9", "-m", self.SUMMARY)
        self.push_tag()
        r = self.block(notes_ok=False)
        self.assertIn("DIE:", r.stdout)
        self.assertIn("gh release edit", r.stdout)
        r = self.block(notes_ok=True)
        self.assertIn("survived", r.stdout, r.stdout + r.stderr)
        self.assertIn("its GitHub notes carry it", r.stderr)

    def test_a_pushed_tag_of_the_wrong_shape_without_a_release_names_the_tag_fix(self) -> None:
        # #1766: `gh release edit` cannot fix notes of a release that does not exist
        self.origin()
        self.tag("-a", "v9.9.9", "-m", self.SUMMARY)
        self.push_tag()
        r = self.block(notes_ok=False, release=False)
        self.assertIn("DIE:", r.stdout)
        self.assertIn("no GitHub release", r.stdout)
        self.assertIn("git push origin :refs/tags/v9.9.9", r.stdout)
        self.assertNotIn("gh release edit", r.stdout)

    def test_a_pushed_tag_that_is_not_the_local_one_dies(self) -> None:
        self.origin()
        self.tag("-a", "v9.9.9", "-m", "jubarte v9.9.9", "-m", self.SUMMARY)
        self.push_tag()
        subprocess.run([*self.git, "tag", "-d", "v9.9.9"], check=True, capture_output=True)
        self.tag("-a", "v9.9.9", "-m", "jubarte v9.9.9", "-m", self.SUMMARY + " again")
        r = self.block(notes_ok=True)
        self.assertIn("DIE:", r.stdout)
        self.assertIn("is not the local tag", r.stdout)

    def test_gh_note_is_defined_before_step_8_calls_it(self) -> None:
        text = RELEASE_SH.read_text()
        self.assertLess(text.index("\ngh_note() "), text.index('say "8. '))



class InstallProofs(unittest.TestCase):
    """#1756 (checklist after#9, after#10, after#11): step 12 installs each
    channel the way a user does. 0.13.0 never reached npm (both packages
    stop at 0.12.0) and no step said so."""

    FNS = ("answers_version", "host_archive", "archive_runs", "pip_installs", "npx_runs", "npm_requires", "docsrs_built")

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="install-")).resolve()
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)

    def run_fn(self, call: str, stubs: str = "", **env: str) -> subprocess.CompletedProcess[str]:
        script = f"set -euo pipefail\n{shell_functions(*self.FNS)}\n{stubs}\n{call}\n"
        return subprocess.run(["bash", "-c", script], cwd=self.tmp, capture_output=True, text=True, timeout=60,
                              env=dict(os.environ, VER="9.9.9", TAG="v9.9.9", ASSETS=str(self.tmp / "assets"), **env))

    def test_the_host_archive_is_named_as_release_yml_names_it(self) -> None:
        for system, machine, name in (("Darwin", "arm64", "macos-aarch64"), ("Linux", "x86_64", "linux-x86_64"),
                                      ("Linux", "aarch64", "linux-aarch64"), ("Darwin", "x86_64", "macos-x86_64")):
            r = self.run_fn("host_archive", f'uname() {{ [ "$1" = -s ] && echo {system} || echo {machine}; }}')
            self.assertEqual(r.stdout.strip(), f"jubarte-9.9.9-{name}.tar.gz", r.stderr)
        r = self.run_fn("host_archive", 'uname() { [ "$1" = -s ] && echo SunOS || echo sparc; }')
        self.assertNotEqual(r.returncode, 0)

    def archive(self, answers: str = "jubarte 9.9.9", listed: bool = True, corrupt: bool = False) -> None:
        name = "jubarte-9.9.9-macos-aarch64"
        root = self.tmp / "build" / name
        root.mkdir(parents=True)
        (root / "jubarte").write_text(f"#!/bin/sh\necho '{answers}'\n")
        (root / "jubarte").chmod(0o755)
        assets = self.tmp / "assets"
        assets.mkdir()
        subprocess.run(["tar", "czf", str(assets / f"{name}.tar.gz"), "-C", str(root.parent), name], check=True)
        digest = subprocess.run(["shasum", "-a", "256", f"{name}.tar.gz"], cwd=assets,
                                capture_output=True, text=True, check=True).stdout
        if corrupt:
            digest = "0" * 64 + digest[64:]
        (assets / "SHA256SUMS.txt").write_text((digest if listed else "") + "1" * 64 + "  other.zip\n")

    GH = ('uname() { [ "$1" = -s ] && echo Darwin || echo arm64; }\n'
          'gh() { local d=""; while [ $# -gt 0 ]; do [ "$1" = -D ] && d=$2; shift; done; cp "$ASSETS"/* "$d"/; }')

    def test_the_archive_matching_its_sum_and_answering_the_version_passes(self) -> None:
        self.archive()
        r = self.run_fn("archive_runs", self.GH)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertFalse((self.tmp / "dist" / "install-proof").exists(), "the proof cleans up after itself")

    def test_an_archive_whose_sum_differs_fails(self) -> None:
        self.archive(corrupt=True)
        self.assertNotEqual(self.run_fn("archive_runs", self.GH).returncode, 0)

    def test_an_archive_sha256sums_does_not_list_fails(self) -> None:
        self.archive(listed=False)
        self.assertNotEqual(self.run_fn("archive_runs", self.GH).returncode, 0)

    def test_an_archive_answering_another_version_fails(self) -> None:
        self.archive(answers="jubarte 9.9.8")
        self.assertNotEqual(self.run_fn("archive_runs", self.GH).returncode, 0)

    def test_pip_installs_into_a_fresh_environment_and_reads_the_engine(self) -> None:
        stub = 'uv() { echo "$*" > "$ASSETS.argv"; echo "${ENGINE:-9.9.9}"; }'
        r = self.run_fn("pip_installs", stub)
        self.assertEqual(r.returncode, 0, r.stderr)
        argv = Path(str(self.tmp / "assets") + ".argv").read_text()
        # --no-build-package: only a wheel install passes; a releaser's
        # machine has Rust and would build the sdist (#1765)
        for word in ("--isolated", "--no-project", "jubarte-redlines==9.9.9", "--refresh-package jubarte-redlines",
                     "--no-build-package jubarte-redlines", "--python 3.12", 'capabilities()["engine_version"]'):
            self.assertIn(word, argv)
        self.assertNotEqual(self.run_fn("pip_installs", stub, ENGINE="9.9.8").returncode, 0)
        self.assertNotEqual(self.run_fn("pip_installs", "uv() { return 1; }").returncode, 0)

    NPX = ('npm() { [ "$1" = init ] && echo "{}" > package.json; }\n'
           'npx() { echo "$*" > "$ASSETS.argv"; { pwd -P; ls; } > "$ASSETS.cwd"; [ -n "$ANSWER" ] && echo "$ANSWER"; }')

    def test_npx_runs_the_published_cli(self) -> None:
        for answer, rc in (("jubarte-redlines 9.9.9", 0), ("jubarte 9.9.9", 0), ("jubarte-redlines 9.9.8", 1), ("", 1)):
            r = self.run_fn("npx_runs", self.NPX, ANSWER=answer)
            self.assertEqual(r.returncode != 0, bool(rc), (answer, r.stderr))
        self.assertIn("-y --prefer-online jubarte-redlines@9.9.9 --version", Path(str(self.tmp / "assets") + ".argv").read_text())

    def test_npx_runs_in_an_empty_project_of_its_own(self) -> None:
        # #1765: npx answers from a node_modules it finds before the registry
        # (a planted jubarte-redlines 9.9.9, a version npm never had, passed)
        (self.tmp / "node_modules" / "jubarte-redlines").mkdir(parents=True)
        r = self.run_fn("npx_runs", self.NPX, ANSWER="jubarte-redlines 9.9.9")
        self.assertEqual(r.returncode, 0, r.stderr)
        cwd, *listing = Path(str(self.tmp / "assets") + ".cwd").read_text().split()
        self.assertEqual(cwd, str(self.tmp / "dist" / "install-proof" / "npx"))
        self.assertEqual(listing, ["package.json"])
        self.assertFalse((self.tmp / "dist" / "install-proof").exists(), "the proof cleans up after itself")

    NPM = ('npm() { case "$1" in init) echo "{}" > package.json ;; i) mkdir -p node_modules/jubarte-wasm\n'
           '  printf \'{"name":"jubarte-wasm","version":"%s","main":"index.js"}\' "${GOT:-9.9.9}" > node_modules/jubarte-wasm/package.json\n'
           '  echo "module.exports = { compareDocuments: ${FN:-() => 1} };" > node_modules/jubarte-wasm/index.js ;; esac; }')

    @unittest.skipUnless(shutil.which("node"), "node is not installed")
    def test_npm_installs_the_wasm_package_and_requires_it(self) -> None:
        r = self.run_fn("npm_requires", self.NPM)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertFalse((self.tmp / "dist" / "install-proof").exists())
        self.assertNotEqual(self.run_fn("npm_requires", self.NPM, GOT="9.9.8").returncode, 0)
        self.assertNotEqual(self.run_fn("npm_requires", self.NPM, FN="1").returncode, 0)
        self.assertNotEqual(self.run_fn("npm_requires", "npm() { return 1; }").returncode, 0)

    def test_docs_rs_status(self) -> None:
        built = 'curl() { echo \'{"doc_status":true,"version":"9.9.9"}\'; }'
        self.assertEqual(self.run_fn("docsrs_built", built).returncode, 0)
        self.assertNotEqual(self.run_fn("docsrs_built", 'curl() { echo \'{"doc_status":false}\'; }').returncode, 0)
        self.assertNotEqual(self.run_fn("docsrs_built", "curl() { return 22; }").returncode, 0)

    def test_step_12_dies_on_the_installs_and_only_warns_on_docs_rs(self) -> None:
        s12 = step(12)
        for fn in ("archive_runs", "eventually pip_installs", "eventually npx_runs", "eventually npm_requires"):
            line = next((l for l in s12.splitlines() if l.startswith("check ") and fn in l), None)
            self.assertIsNotNone(line, fn)
        self.assertNotIn("check ", "".join(l for l in s12.splitlines() if "docsrs_built" in l))
        self.assertIn("docsrs_built", s12)
        # the call, not the definition above it (#1765)
        call = next(m.start() for m in re.finditer(r"^check .*eventually npm_requires", s12, re.M))
        self.assertLess(call, s12.index('[ "$ok" = 1 ] || die'))



class ProofTools(unittest.TestCase):
    """#1757: every proof names a tool a repository holds. The 0.13.0
    checklist proved five items with tools that lived only in a release work
    folder (parity_build_v2.sh, apply_changelogs_v2.py,
    lane_validity_vs_sources.py, queue.sh, build_app_0.11.2.sh), one of
    which had been deleted and rebuilt from session logs."""

    # a `(cd OTHER && …)` group runs another repository's tools
    FOREIGN = re.compile(r"\(cd [^)]*\)")
    # scripts/… or tools/… run from this checkout: not ./scripts/… or app/scripts/…
    TOOL = re.compile(r"(?<![\w./-])(?:scripts|tools)/[\w.-]+(?:/[\w.-]+)*")

    def test_every_tool_named_from_this_checkout_exists(self) -> None:
        missing = []
        for line in checklist_block().splitlines():
            parts = line.split("|")
            if len(parts) != 4:
                continue
            for path in self.TOOL.findall(self.FOREIGN.sub("", parts[3])):
                path = path.rstrip(".")
                # a script, not a folder a tool writes to (tools/parity/parity-work)
                if Path(path).suffix in {".py", ".sh", ".mjs", ".js"} and not (HERE.parent / path).exists():
                    missing.append((parts[0], path))
        self.assertEqual(missing, [])

    def test_no_proof_names_a_release_work_folder_tool(self) -> None:
        block = checklist_block()
        for name in ("parity_build_v2", "apply_changelogs_v2", "lane_validity_vs_sources", "queue.sh",
                     "build_app_0.11.2", "agents/"):
            self.assertFalse(name in block, f"the checklist still names {name}")

    def test_step_5_runs_the_tests_of_these_tools(self) -> None:
        s5 = step(5)
        for test in ("scripts/test_lane_validity.py", "scripts/test_redline_sweep.py"):
            self.assertIn(f'"${{py312[@]}}" {test}', s5)
        self.assertIn("pytest -q -p no:cacheprovider tools/parity", s5)

    def test_the_word_sweeps_read_the_redline_sample_only(self) -> None:
        # sample_*_V_*.csv also matches the conversion sample, which
        # redline-sweep.sh and lane_validity.py refuse (exit 2)
        lines = [line for line in checklist_block().splitlines()
                 if ("redline-sweep.sh" in line or "lane_validity.py" in line) and "release_info/" in line]
        self.assertGreaterEqual(len(lines), 2, "Ring-3 and lane validity")
        for line in lines:
            self.assertNotIn("sample_*_", line)
            self.assertIn("release_info/sample_redline_@V@_*.csv", line)


class Lessons0140(unittest.TestCase):
    """What the 0.14.0 readiness run learned, kept in the checklist so the
    next release does not learn it again."""

    def rows(self, needle: str) -> list[list[str]]:
        return [line.split("|") for line in checklist_block().splitlines()
                if line.count("|") == 3 and needle in line]

    def one_row(self, needle: str) -> str:
        rows = self.rows(needle)
        self.assertEqual(len(rows), 1, f"one checklist row names {needle!r}")
        return "|".join(rows[0])

    def test_ring3_opens_word_through_the_bench_and_reads_word_own_failures(self) -> None:
        # #1794: --probe drove Word through an osascript of its own. 0.14.0's
        # 23 failures crash Word 16.115 on Word's own redlines of most of
        # the pairs too: the row says how that is read, and who waives it.
        row = self.one_row("Ring-3 Word open probe")
        for needle in ("word_pdf.py", "word_watchdog.sh", "NEUROTIC_DOCX_BENCH", "alone",
                       "Word's own redline", "only the owner waives"):
            self.assertIn(needle, row)
        self.assertNotIn("osascript", row)

    def test_a_skipped_miri_suite_is_disclosed(self) -> None:
        # 0.14.0: miri not installed; the owner released without it
        row = self.one_row("cargo +nightly miri nextest run")
        self.assertIn("skips it", row)
        self.assertIn("release notes", row)

    def test_only_release_sh_tags_and_pushes(self) -> None:
        # 0.13.0 pushed main and its tag by hand, and step 13 never ran:
        # jubarte.pro and the app's facts stayed on 0.12.0
        row = self.one_row("a run in flight owns its worktree")
        self.assertIn("only release.sh pushes main and the tag", row)
        self.assertIn("step 13 never ran", row)

    def test_the_bench_flow_resumes_and_pins_libreoffice(self) -> None:
        row = self.one_row("jubarte_release_info @V@ --engine-dir <engine> --binary")
        self.assertIn("--only", row)
        self.assertIn("SOFFICE=", row)
        self.assertIn("26.8.0.3", row)

    def test_a_missing_previous_snapshot_is_backfilled(self) -> None:
        row = self.one_row("backfill")
        phase, mark, _item, proof = row.split("|")
        self.assertEqual((phase, mark), ("before", "you"))
        self.assertIn("git worktree add", proof)
        # codex on #1743 (#1778): the tag's own script may be the one #1769
        # fixed, so this checkout's runs in the tag's worktree
        self.assertLess(proof.index("cp scripts/api_snapshot.py <dir>/scripts/"),
                        proof.index("python3 scripts/api_snapshot.py"))

    def test_step_6_names_a_missing_previous_snapshot(self) -> None:
        # 0.13.0 skipped step 6: "no previous-release snapshot — … is the
        # baseline" hid that v0.13.0 exists and its snapshot does not
        s6 = ApiDocDrift.step6(self)
        self.assertIn("no snapshot of $PREV_TAG", s6)
        self.assertIn("backfill", s6)
        self.assertLess(s6.index("cp scripts/api_snapshot.py <dir>/scripts/"),
                        s6.index("python3 scripts/api_snapshot.py ${PREV_TAG#v}"))

    def test_hand_run_python_proofs_pin_3_12(self) -> None:
        # uv's default here was a free-threaded 3.15t; no abi3 wheel serves it
        row = self.one_row("free-threaded")
        self.assertIn("--python 3.12", row)

    def test_no_message_folds_a_list_with_an_unquoted_echo(self) -> None:
        # $(echo $list) globs each word (SC2086) and hides the separator
        self.assertNotRegex(RELEASE_SH.read_text(), r"\$\(echo \$\w+\)")

    def test_the_app_scripts_run_under_bash(self) -> None:
        row = self.one_row("never sh or zsh")
        for script in ("release-engine.sh", "build-release-app.sh", "notarize-direct.sh"):
            self.assertIn(script, row)



class VersionAnswerTests(unittest.TestCase):
    """Step 12 reads a binary's `--version`, which carries the build commit
    as SemVer build metadata since #1924 (`jubarte 0.16.0+00b32d2`)."""

    def answers(self, got: str, ver: str = "0.16.0") -> bool:
        script = f'{shell_functions("answers_version")}\nVER={ver}\nanswers_version "$1"'
        return subprocess.run(["bash", "-c", script, "-", got]).returncode == 0

    def test_a_bare_version_answers(self) -> None:
        self.assertTrue(self.answers("jubarte 0.16.0"))
        self.assertTrue(self.answers("jubarte-redlines 0.16.0"))

    def test_the_build_commit_answers(self) -> None:
        self.assertTrue(self.answers("jubarte 0.16.0+00b32d2"))
        self.assertTrue(self.answers("jubarte-redlines 0.16.0+e65644b4f"))

    def test_a_dirty_build_or_another_version_does_not(self) -> None:
        self.assertFalse(self.answers("jubarte 0.16.0+00b32d2.dirty"))
        self.assertFalse(self.answers("jubarte 0.16.1"))
        self.assertFalse(self.answers("jubarte 0x16x0"))
        self.assertFalse(self.answers("jubarte 0.16.0+xyz"))
        self.assertFalse(self.answers("jubarte 0.16.0 extra"))

    def test_step_12_reads_versions_through_it(self) -> None:
        text = RELEASE_SH.read_text()
        for name in ("archive_runs() {", "npx_runs() {"):
            start = text.index(name)
            self.assertIn("answers_version", text[start : text.index("\n}\n", start)])


if __name__ == "__main__":
    unittest.main()
