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
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import check_site_live

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
        self.assertIn("python3 scripts/test_api_snapshot.py", step(5))

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
        s8 = step(8)
        guard = s8.index("regenerate npm artifacts for v$VER")
        self.assertLess(guard, s8.index("jubarte-wasm/build-npm.sh"))

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
        for f in ("jubarte-python/Cargo.toml", "jubarte-app/src-tauri/tauri.conf.json",
                  "gemini-extension.json", "jubarte-app/src/index.html"):
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
        # A "jubarte-app/" line in .gitignore made `git add` refuse the
        # vendored app's version files, tracked as they are, and step 8
        # would have stopped before the release commit.
        s8 = step(8)
        start = s8.index("git add Cargo.toml")
        staged = s8[start:s8.index("git commit", start)].replace("\\\n", " ").split()[2:]
        self.assertIn("jubarte-app/src-tauri/Cargo.toml", staged)
        run = subprocess.run(["git", "check-ignore", "--no-index", *staged],
                             cwd=HERE.parent, capture_output=True, text=True)
        self.assertEqual(run.stdout, "", "the release commit stages paths .gitignore ignores")


DOWNSTREAM_SH = HERE / "release_downstream.sh"


class Downstream(unittest.TestCase):
    """Step 12: jubarte.pro, the jubarte-app commit, the App Store and the
    benchmark. Each case runs a copy of release_downstream.sh in a throwaway
    folder with --no-site, so nothing is deployed, pushed or uploaded."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="downstream_sh_"))
        (self.tmp / "scripts").mkdir()
        shutil.copy(DOWNSTREAM_SH, self.tmp / "scripts" / "release_downstream.sh")

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    def run_downstream(self, *args: str) -> subprocess.CompletedProcess[str]:
        # The gates run these tests inside a release, where JUBARTE_APP_DIR
        # names the real app checkout: the copy must never be handed it.
        env = {k: v for k, v in os.environ.items()
               if k not in ("JUBARTE_APP_DIR", "JUBARTE_SITE_NO_DEPLOY")}
        return subprocess.run(
            ["bash", str(self.tmp / "scripts" / "release_downstream.sh"), *args],
            capture_output=True, text=True, timeout=60, env=env,
        )

    def app_repo(self, branch: str) -> Path:
        app = self.tmp / "jubarte-app"
        app.mkdir()
        git = ["git", "-C", str(app)]
        subprocess.run([*git, "init", "-q", "-b", branch], check=True)
        (app / "package.json").write_text('{"version": "0.10.2"}\n')
        return app

    def test_needs_a_release_version(self) -> None:
        for args in ((), ("0.10",), ("v0.10.2",), ("0.10.2", "--bogus")):
            self.assertEqual(self.run_downstream(*args).returncode, 2, args)

    def test_stops_without_an_app_folder(self) -> None:
        r = self.run_downstream("0.10.2", "--no-site")
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("no jubarte-app/", r.stderr)

    def test_an_app_folder_that_is_no_checkout_is_left_alone(self) -> None:
        (self.tmp / "jubarte-app").mkdir()
        r = self.run_downstream("0.10.2", "--no-site")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("not its own checkout", r.stdout)

    def test_an_app_off_main_is_listed_not_committed(self) -> None:
        app = self.app_repo("feature")
        r = self.run_downstream("0.10.2", "--no-site")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("not main", r.stdout)
        self.assertIn("package.json", r.stdout)
        log = subprocess.run(["git", "-C", str(app), "log"], capture_output=True, text=True)
        self.assertNotEqual(log.returncode, 0, "no commit was made")

    def test_prints_the_app_store_and_bench_commands_without_running_them(self) -> None:
        (self.tmp / "jubarte-app").mkdir()
        out = self.run_downstream("0.10.2", "--no-site").stdout
        self.assertIn("not uploaded (pass --app)", out)
        self.assertIn("asc-new-version.py 0.10.2 --apply", out)
        self.assertIn("Submit for Review", out)
        self.assertIn("jubarte_release_info 0.10.2", out)
        self.assertIn("scripts/release_jubarte.py 0.10.2", out)
        self.assertIn("scripts/release.sh bench 0.10.2 --redline-tool jubarte-0.10.2", out)

    def test_release_runs_it_after_verify(self) -> None:
        s13 = RELEASE_SH.read_text().split('say "13. ', 1)[1]
        self.assertIn('scripts/release_downstream.sh "$VER"', s13)
        self.assertLess(
            RELEASE_SH.read_text().index('say "12. Verify'),
            RELEASE_SH.read_text().index('say "13. Downstream'),
        )

    def test_the_site_step_runs_the_site_release(self) -> None:
        text = DOWNSTREAM_SH.read_text()
        # Checked without a deploy first; the deploy is its own, later call.
        self.assertLess(text.index('site release "$VER" --no-deploy'),
                        text.index('site deploy "$VER"'))
        # The upload is opt-in and review is never submitted from here.
        self.assertIn('if [ "$APP" = 1 ]', text)
        self.assertNotIn("--submit", text)



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
    GH = """#!/bin/sh
# stub gh: `pr view` knows no pull request until `pr create`; `pr merge`
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

    def test_both_release_scripts_go_through_it(self) -> None:
        for name in ("release.sh", "release_downstream.sh"):
            text = (HERE / name).read_text()
            self.assertIn("scripts/push_main.sh", text, name)
            self.assertIn("push_main ", text, name)
            self.assertNotIn("git push origin main", text, name)
            self.assertNotIn("git push -q origin HEAD:main", text, name)


SITE_LIVE_PY = HERE / "check_site_live.py"

# The figures of a release, as its two results JSONs carry them (shrunk to
# what the live check reads), and the benchmark page that prints them.
EVIDENCE = {
    "conversion": {
        "jubarte": {"version": "jubarte 9.9.9", "n": 600, "failures": 0,
                    "mean": 78.7687, "median": 80.8474},
        "soffice": {"version": "LibreOffice 26.8.0.3", "n": 600, "failures": 4,
                    "mean": 54.0623, "median": 47.7343},
    },
    "redline": {
        "jubarte": {"version": "jubarte 9.9.9", "n": 1600, "failures": 0,
                    "mean": 76.7059, "median": 84.2774},
        "docxodus": {"version": "Docxodus 12.6.5 (C#)", "n": 1600, "failures": 23,
                     "mean": 69.2382, "median": 78.8939},
    },
}
STAMP = "10-03-26_19-45"


def write_evidence(folder: Path, version: str = "9.9.9", evidence: dict | None = None) -> None:
    folder.mkdir(parents=True, exist_ok=True)
    for kind, tools in (evidence or EVIDENCE).items():
        doc = {"release": version, "stamp": STAMP, "tools": tools}
        (folder / f"results_{kind}_{version}_{STAMP}.json").write_text(json.dumps(doc))


def bench_row(rank: int, name: str, pin: str, note: str, cells: tuple[str, ...]) -> str:
    """One tool's row as the site's benchmark page writes it."""
    spans = "".join(f'<span class="num" role="cell">{c}</span>\n' for c in cells)
    return (f'<div class="t-row bench-cols" role="row">\n<span role="cell">{rank}</span>\n'
            f'<div class="tool" role="rowheader"><div class="tool-name">{name}</div>'
            f'<div class="tool-pin">{pin}</div></div>\n'
            f'<div class="bar-cell" role="cell"><div class="bar"><div style="width:99.99%"></div></div>'
            f'<span class="bar-note">{note}</span></div>\n{spans}</div>\n')


def bench_page(conversion_median: str = "80.85", order: tuple[int, int] = (0, 1)) -> str:
    """jubarte.pro/benchmark for EVIDENCE: a section per sample table, a
    role="row" per tool. `conversion_median` is what jubarte's conversion row
    prints; `order` swaps the two conversion rows' figures."""
    conv = [(conversion_median, "78.77", "600", "0"), ("47.73", "54.06", "600", "4")]
    head = '<div class="t-row head" role="row"><span role="columnheader">#</span></div>\n'
    return (
        "<!doctype html><html><body><main>\n"
        '<p class="headline-v"><span>80.85</span></p>\n'
        '<section class="mt-72" id="redlines-sample">\n<h2>Redlines — 600-pair sample</h2>\n' + head
        + bench_row(1, "jubarte-9.9.9 †", "jubarte 9.9.9", "= 100: 61 · ≥ 90: 131",
                    ("84.28", "76.71", "1,600", "0"))
        + bench_row(2, "docxodus", "Docxodus 12.6.5 (C#)", "= 100: 41 · ≥ 90: 99",
                    ("78.89", "69.24", "1,600", "23"))
        + '</section>\n<section class="mt-72" id="conversion-sample">\n<h2>DOCX → PDF</h2>\n' + head
        + bench_row(1, "jubarte †", "jubarte 9.9.9", "[80.29, 81.67]", conv[order[0]])
        + bench_row(2, "soffice", "LibreOffice 26.8.0.3", "[47.31, 48.15]", conv[order[1]])
        + "</section>\n</main></body></html>\n"
    )


class SiteLive(unittest.TestCase):
    """scripts/check_site_live.py: jubarte.pro/benchmark must print the
    figures of the release's two results JSONs. A stub `curl` serves the
    page; nothing is fetched."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="site_live_"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.info = self.tmp / "release_info"
        write_evidence(self.info)

    def differences(self, page: str) -> list[str]:
        rows, problems = check_site_live.expected(self.info, "9.9.9")
        self.assertEqual(problems, [])
        return check_site_live.differences(page, rows)

    def test_a_page_that_prints_the_figures_passes(self) -> None:
        self.assertEqual(self.differences(bench_page()), [])

    def test_a_wrong_median_is_named_with_what_the_row_says(self) -> None:
        found = self.differences(bench_page(conversion_median="80.90"))
        self.assertEqual(len(found), 1, found)
        self.assertIn("conversion-sample / jubarte 9.9.9", found[0])
        self.assertIn("median 80.85", found[0])
        self.assertIn("80.90", found[0])

    def test_another_tools_figures_do_not_pass_for_this_one(self) -> None:
        found = "\n".join(self.differences(bench_page(order=(1, 0))))
        self.assertIn("conversion-sample / jubarte 9.9.9", found)
        self.assertIn("conversion-sample / LibreOffice 26.8.0.3", found)

    def test_a_page_of_the_previous_release_fails_on_every_row_of_ours(self) -> None:
        found = "\n".join(self.differences(bench_page().replace("jubarte 9.9.9", "jubarte 9.9.8")))
        self.assertIn("conversion-sample / jubarte 9.9.9: no row of the page names it", found)
        self.assertIn("redlines-sample / jubarte 9.9.9: no row of the page names it", found)

    def test_a_page_without_the_table_says_so(self) -> None:
        found = self.differences("<html><body>maintenance</body></html>")
        self.assertIn('the page has no section id="conversion-sample"', found)
        self.assertIn('the page has no section id="redlines-sample"', found)

    def test_figures_read_as_the_page_prints_them(self) -> None:
        # JavaScript's toFixed(2) and toLocaleString("en-US"): a half rounds
        # up, a float that only looks like a half does not, thousands group.
        self.assertEqual(check_site_live.fixed2(80.125), "80.13")
        self.assertEqual(check_site_live.fixed2(1.005), "1.00")
        self.assertEqual(check_site_live.fixed2(96), "96.00")
        self.assertEqual(check_site_live.grouped(1600), "1,600")
        self.assertEqual(check_site_live.grouped(600), "600")

    def test_missing_or_doubled_evidence_is_a_problem_not_a_pass(self) -> None:
        _, problems = check_site_live.expected(self.info, "9.9.8")
        self.assertEqual(len(problems), 2, problems)
        self.assertIn("results_conversion_9.9.8_*.json", problems[0])
        (self.info / "results_redline_9.9.9_10-04-26_09-00.json").write_text("{}")
        _, problems = check_site_live.expected(self.info, "9.9.9")
        self.assertTrue(any("2 stamps" in p for p in problems), problems)

    # --- the command, with a stub curl --------------------------------------

    def cli(self, *args: str, pages: tuple[str, ...] = ()) -> subprocess.CompletedProcess[str]:
        """Runs the script; the n-th curl call is served pages[n] (the last
        one from then on)."""
        bin_dir = self.tmp / "bin"
        bin_dir.mkdir(exist_ok=True)
        for n, page in enumerate(pages):
            (self.tmp / f"page{n}.html").write_text(page)
        (bin_dir / "curl").write_text(
            '#!/bin/sh\necho "curl $*" >> "$CALLS"\n'
            'n=$(($(wc -l < "$CALLS") - 1))\n'
            'while [ ! -f "$PAGES/page$n.html" ]; do n=$((n - 1)); done\n'
            'cat "$PAGES/page$n.html"\n')
        (bin_dir / "curl").chmod(0o755)
        self.calls = self.tmp / "calls.log"
        self.calls.write_text("")
        env = {**os.environ, "PATH": f"{bin_dir}{os.pathsep}{os.environ['PATH']}",
               "CALLS": str(self.calls), "PAGES": str(self.tmp)}
        return subprocess.run(
            ["python3", str(SITE_LIVE_PY), "9.9.9", "--release-info", str(self.info), *args],
            capture_output=True, text=True, timeout=60, env=env)

    def test_the_live_page_is_fetched_and_passes(self) -> None:
        r = self.cli(pages=(bench_page(),))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("https://jubarte.pro/benchmark?release=9.9.9", self.calls.read_text())
        self.assertEqual(self.calls.read_text().count("curl "), 1)

    def test_it_waits_for_the_deploy_to_spread(self) -> None:
        r = self.cli("--tries", "3", "--wait", "0",
                     pages=(bench_page(conversion_median="80.90"), bench_page()))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.calls.read_text().count("curl "), 2)

    def test_a_page_that_never_shows_the_figures_fails_loudly(self) -> None:
        r = self.cli("--tries", "2", "--wait", "0", pages=(bench_page(conversion_median="80.90"),))
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("conversion-sample / jubarte 9.9.9", r.stderr)
        self.assertIn("does not show", r.stderr)
        self.assertEqual(self.calls.read_text().count("curl "), 2)

    def test_a_built_page_is_read_without_a_fetch(self) -> None:
        built = self.tmp / "benchmark.html"
        built.write_text(bench_page())
        r = self.cli("--page", str(built))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.calls.read_text(), "")
        built.write_text(bench_page(conversion_median="80.90"))
        self.assertEqual(self.cli("--page", str(built)).returncode, 1)

    def test_no_evidence_exits_2_without_fetching(self) -> None:
        shutil.rmtree(self.info)
        r = self.cli(pages=(bench_page(),))
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertEqual(self.calls.read_text(), "")


class DownstreamSite(unittest.TestCase):
    """The site step of release_downstream.sh, run for real from a release
    worktree: its jubarte-app/ is the vendored snapshot (no site in it), the
    app repository is a separate checkout with its own origin. A stand-in
    jubarte-site/scripts/release.sh keeps the real one's kinds, order and
    refusals; pnpm, wrangler, curl, npm, node, uv and gh are stubs first on
    PATH that log each call — nothing is installed, deployed or fetched."""

    SITE_RELEASE = """#!/usr/bin/env bash
# Stand-in for the app repository's jubarte-site/scripts/release.sh, with the
# two kinds release_downstream.sh runs:
#   scripts/release.sh release x.y.z --no-deploy
#   scripts/release.sh deploy x.y.z
set -euo pipefail
cd "$(dirname "$0")/.."
echo "site $* (ENGINE=${ENGINE:-} CHANGELOG=${CHANGELOG:-})" >> "$CALLS"
case "$1" in
  release)
    [ "${3:-}" = --no-deploy ] || exit 9
    # facts.py appends only the values that changed
    grep -F "\\"value\\":\\"$2\\"" ../data/facts.jsonl >/dev/null \\
      || printf '{"key":"engine.version","value":"%s"}\\n' "$2" >> ../data/facts.jsonl
    pnpm test
    pnpm lint
    pnpm typecheck
    ;;
  deploy)
    [ -z "$(git status --porcelain --untracked-files=no -- . ../data)" ] \\
      || { echo "uncommitted changes" >&2; exit 1; }
    pnpm run deploy
    ;;
  *) exit 2 ;;
esac
"""
    PNPM = """#!/bin/sh
echo "pnpm $*" >> "$CALLS"
[ "${PNPM_FAIL:-}" = "$1" ] && exit 1
# `pnpm test` builds the site before it tests it.
[ "$1" = test ] && mkdir -p public && cp "$BUILT_PAGE" public/benchmark.html
exit 0
"""
    CURL = '#!/bin/sh\necho "curl $*" >> "$CALLS"\ncat "$LIVE_PAGE"\n'
    LOGGED = '#!/bin/sh\necho "{name} $*" >> "$CALLS"\n'
    SUBJECT = "chore(site): jubarte.pro on v9.9.9"

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="downstream_site_"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.engine = self.tmp / "engine"
        self.origin = self.tmp / "app-origin.git"
        self.app = self.tmp / "app"
        self.calls = self.tmp / "calls.log"
        self.gh_log = self.tmp / "gh.log"
        self.built = self.tmp / "built.html"
        self.live = self.tmp / "live.html"
        for f in (self.calls, self.gh_log):
            f.write_text("")
        for f in (self.built, self.live):
            f.write_text(bench_page())

        # the release worktree of the engine
        (self.engine / "scripts").mkdir(parents=True)
        for name in ("release_downstream.sh", "push_main.sh", "check_site_live.py"):
            shutil.copy(HERE / name, self.engine / "scripts" / name)
        (self.engine / "jubarte-app").mkdir()
        (self.engine / "jubarte-app" / "package.json").write_text('{"version": "9.9.9"}\n')
        (self.engine / "CHANGELOG.md").write_text("## [9.9.9] - 2026-10-09\n")
        write_evidence(self.engine / "release_info")

        bin_dir = self.tmp / "bin"
        bin_dir.mkdir()
        stubs = {"gh": PushMain.GH, "pnpm": self.PNPM, "curl": self.CURL}
        for name in ("wrangler", "npm", "node", "uv"):
            stubs[name] = self.LOGGED.format(name=name)
        for name, body in stubs.items():
            (bin_dir / name).write_text(body)
            (bin_dir / name).chmod(0o755)
        self.env = {**os.environ, "PATH": f"{bin_dir}{os.pathsep}{os.environ['PATH']}",
                    "CALLS": str(self.calls), "GH_LOG": str(self.gh_log),
                    "ORIGIN": str(self.origin), "BUILT_PAGE": str(self.built),
                    "LIVE_PAGE": str(self.live), "JUBARTE_APP_DIR": str(self.app),
                    "SITE_LIVE_TRIES": "2", "SITE_LIVE_WAIT_SECONDS": "0",
                    "PUSH_MAIN_RETRY_SECONDS": "0",
                    "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.com",
                    "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.com"}
        self.env.pop("JUBARTE_SITE_NO_DEPLOY", None)

        # the app repository: an origin, and the checkout the release is given
        self.git("init", "-q", "--bare", str(self.origin), cwd=self.tmp)
        self.git("symbolic-ref", "HEAD", "refs/heads/main", cwd=self.origin)
        self.git("clone", "-q", str(self.origin), str(self.app), cwd=self.tmp)
        self.git("checkout", "-q", "-b", "main")
        site = self.app / "jubarte-site"
        (site / "scripts").mkdir(parents=True)
        (site / "scripts" / "release.sh").write_text(self.SITE_RELEASE)
        (site / "scripts" / "release.sh").chmod(0o755)
        (site / "package.json").write_text('{"name": "jubarte-site"}\n')
        (self.app / "data").mkdir()
        (self.app / "data" / "facts.jsonl").write_text('{"key":"engine.version","value":"9.9.8"}\n')
        (self.app / "README.md").write_text("the app\n")
        (self.app / ".gitignore").write_text("node_modules/\npublic/\n")
        # what a checkout that can build the site holds, untracked
        (site / "node_modules").mkdir()
        (site / "public" / "fixtures").mkdir(parents=True)
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "base")
        self.git("push", "-q", "origin", "main")
        self.base = self.git("rev-parse", "HEAD")

    def git(self, *args: str, cwd: Path | None = None) -> str:
        return subprocess.run(["git", *args], cwd=cwd or self.app, env=self.env,
                              capture_output=True, text=True, check=True).stdout.strip()

    def run_downstream(self, *args: str, **env: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["bash", str(self.engine / "scripts" / "release_downstream.sh"), "9.9.9", *args],
            capture_output=True, text=True, timeout=120, env={**self.env, **env})

    def origin_main(self) -> str:
        return self.git("rev-parse", "main", cwd=self.origin)

    def called(self) -> str:
        return self.calls.read_text()

    def facts(self) -> str:
        return (self.app / "data" / "facts.jsonl").read_text()

    def assert_nothing_deployed(self) -> None:
        for call in ("site deploy", "pnpm run deploy", "wrangler", "curl"):
            self.assertNotIn(call, self.called())

    def protect_main(self) -> None:
        hook = self.origin / "hooks" / "pre-receive"
        hook.write_text(PushMain.HOOK)
        hook.chmod(0o755)

    # --- where the app checkout comes from ----------------------------------

    def test_a_release_worktree_says_what_to_pass(self) -> None:
        env = {k: v for k, v in self.env.items() if k != "JUBARTE_APP_DIR"}
        r = subprocess.run(
            ["bash", str(self.engine / "scripts" / "release_downstream.sh"), "9.9.9"],
            capture_output=True, text=True, timeout=60, env=env)
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("vendored snapshot", r.stderr)
        self.assertIn("JUBARTE_APP_DIR=", r.stderr)
        self.assertIn("--app-dir", r.stderr)
        self.assertEqual(self.called(), "")
        self.assertEqual(self.origin_main(), self.base)

    def test_a_named_folder_that_is_not_the_app_is_refused(self) -> None:
        r = self.run_downstream(JUBARTE_APP_DIR=str(self.engine / "jubarte-app"))
        self.assertEqual(r.returncode, 1, r.stderr)
        self.assertIn("not a checkout of the app repository", r.stderr)
        self.assertEqual(self.called(), "")

    def test_the_named_checkout_is_released_from_a_release_worktree(self) -> None:
        r = self.run_downstream()
        self.assertEqual(r.returncode, 0, r.stderr + r.stdout)
        engine = self.engine.resolve()
        self.assertEqual(self.called().splitlines(), [
            f"site release 9.9.9 --no-deploy (ENGINE={engine} CHANGELOG={engine}/CHANGELOG.md)",
            "pnpm test", "pnpm lint", "pnpm typecheck",
            f"site deploy 9.9.9 (ENGINE={engine} CHANGELOG={engine}/CHANGELOG.md)",
            "pnpm run deploy",
            "curl -fsSL https://jubarte.pro/benchmark?release=9.9.9",
        ])
        # the facts are committed in the app repository, and on its main
        self.assertEqual(self.git("log", "-1", "--format=%s"), self.SUBJECT)
        self.assertEqual(self.git("show", "--name-only", "--format=", "HEAD"), "data/facts.jsonl")
        self.assertEqual(self.origin_main(), self.git("rev-parse", "HEAD"))
        self.assertEqual(self.git("status", "--porcelain"), "")
        self.assertEqual(self.gh_log.read_text(), "")
        self.assertIn("jubarte.pro/benchmark shows", r.stdout)

    def test_the_flag_names_the_checkout_too(self) -> None:
        env = {k: v for k, v in self.env.items() if k != "JUBARTE_APP_DIR"}
        r = subprocess.run(
            ["bash", str(self.engine / "scripts" / "release_downstream.sh"), "9.9.9",
             "--app-dir", str(self.app)],
            capture_output=True, text=True, timeout=120, env=env)
        self.assertEqual(r.returncode, 0, r.stderr + r.stdout)
        self.assertEqual(self.git("log", "-1", "--format=%s"), self.SUBJECT)

    # --- once, and in order --------------------------------------------------

    def test_the_records_are_merged_once_and_only_once(self) -> None:
        self.assertEqual(self.run_downstream().returncode, 0)
        facts, head = self.facts(), self.origin_main()
        self.assertEqual(facts.count('"9.9.9"'), 1)
        again = self.run_downstream()
        self.assertEqual(again.returncode, 0, again.stderr + again.stdout)
        self.assertEqual(self.facts(), facts)
        self.assertEqual(self.origin_main(), head)
        self.assertEqual(self.git("log", "--format=%s").count(self.SUBJECT), 1)
        self.assertIn("nothing to commit", again.stdout)

    def test_a_failing_check_stops_before_the_commit_and_the_deploy(self) -> None:
        r = self.run_downstream(PNPM_FAIL="lint")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("site checks failed", r.stderr)
        self.assertIn("rerun scripts/release_downstream.sh 9.9.9", r.stderr)
        self.assertIn("pnpm lint", self.called())
        self.assert_nothing_deployed()
        self.assertEqual(self.git("rev-parse", "HEAD"), self.base)
        self.assertEqual(self.origin_main(), self.base)

    def test_a_built_page_without_the_figures_stops_before_the_commit_and_the_deploy(self) -> None:
        self.built.write_text(bench_page(conversion_median="80.90"))
        r = self.run_downstream()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("conversion-sample / jubarte 9.9.9", r.stderr)
        self.assertIn("built benchmark page", r.stderr)
        self.assert_nothing_deployed()
        self.assertEqual(self.origin_main(), self.base)

    def test_no_deploy_does_everything_else_and_says_so(self) -> None:
        for how in ({"args": ("--no-deploy",)}, {"env": {"JUBARTE_SITE_NO_DEPLOY": "1"}}):
            with self.subTest(how=how):
                self.calls.write_text("")
                r = self.run_downstream(*how.get("args", ()), **how.get("env", {}))
                self.assertEqual(r.returncode, 0, r.stderr + r.stdout)
                self.assertIn("pnpm typecheck", self.called())
                self.assert_nothing_deployed()
                self.assertIn("not deployed (--no-deploy)", r.stdout)
                self.assertIn("scripts/release_downstream.sh 9.9.9", r.stdout)
                self.assertEqual(self.git("log", "-1", "--format=%s"), self.SUBJECT)
                self.assertEqual(self.origin_main(), self.git("rev-parse", "HEAD"))

    def test_the_live_check_fails_on_a_mismatch(self) -> None:
        self.live.write_text(bench_page(conversion_median="80.90"))
        r = self.run_downstream()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("conversion-sample / jubarte 9.9.9", r.stderr)
        self.assertIn("median 80.85", r.stderr)
        self.assertIn("jubarte.pro/benchmark does not show", r.stderr)
        self.assertIn("python3 scripts/check_site_live.py 9.9.9", r.stderr)
        # it was deployed, and asked SITE_LIVE_TRIES times
        self.assertIn("pnpm run deploy", self.called())
        self.assertEqual(self.called().count("curl "), 2)

    def test_a_failed_deploy_names_the_step_and_the_rerun(self) -> None:
        r = self.run_downstream(PNPM_FAIL="run")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("the deploy failed", r.stderr)
        self.assertIn("rerun scripts/release_downstream.sh 9.9.9", r.stderr)
        self.assertNotIn("curl", self.called())
        # the facts are on main already: the rerun only deploys
        self.assertEqual(self.origin_main(), self.git("rev-parse", "HEAD"))
        again = self.run_downstream()
        self.assertEqual(again.returncode, 0, again.stderr + again.stdout)
        self.assertEqual(self.git("log", "--format=%s").count(self.SUBJECT), 1)

    # --- reaching the app's main ---------------------------------------------

    def test_a_main_that_wants_a_pull_request_gets_the_facts_through_one(self) -> None:
        self.protect_main()
        r = self.run_downstream()
        self.assertEqual(r.returncode, 0, r.stderr + r.stdout)
        gh = self.gh_log.read_text()
        self.assertIn("pr create --base main --head chore/site-v9.9.9", gh)
        self.assertIn("pr merge chore/site-v9.9.9 --merge", gh)
        parents = self.git("rev-list", "--parents", "-n", "1", "main", cwd=self.origin).split()
        self.assertEqual(len(parents), 3)
        self.assertEqual(self.git("log", "-1", "--format=%s", parents[2]), self.SUBJECT)
        # the checkout is what main holds, and that is what was deployed
        self.assertEqual(self.git("rev-parse", "HEAD"), self.origin_main())
        self.assertLess(self.called().index("site release"), self.called().index("site deploy"))

    def test_a_main_that_takes_neither_way_deploys_nothing(self) -> None:
        self.protect_main()
        r = self.run_downstream(ORIGIN=str(self.tmp / "nowhere.git"))  # the stub's merge fails
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("did not reach the app's main", r.stderr)
        self.assert_nothing_deployed()
        self.assertEqual(self.origin_main(), self.base)

    def test_a_rerun_pushes_the_commit_an_earlier_run_left_behind(self) -> None:
        self.protect_main()
        self.assertEqual(self.run_downstream(ORIGIN=str(self.tmp / "nowhere.git")).returncode, 1)
        self.assertEqual(self.git("log", "-1", "--format=%s"), self.SUBJECT)
        r = self.run_downstream()
        self.assertEqual(r.returncode, 0, r.stderr + r.stdout)
        self.assertEqual(self.git("rev-parse", "HEAD"), self.origin_main())
        self.assertIn("pnpm run deploy", self.called())

    # --- a checkout the release must not deploy from --------------------------

    def refused(self, needle: str) -> None:
        r = self.run_downstream()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(needle, r.stderr)
        self.assertEqual(self.called(), "")
        self.assertEqual(self.origin_main(), self.base)

    def test_an_app_checkout_off_main_is_refused(self) -> None:
        self.git("checkout", "-q", "-b", "release/v9.9.9")
        self.refused("is on release/v9.9.9, not main")

    def test_an_app_checkout_with_other_changes_is_refused(self) -> None:
        (self.app / "README.md").write_text("edited\n")
        self.refused("README.md")

    def test_an_app_checkout_behind_its_origin_is_refused(self) -> None:
        other = self.tmp / "other"
        self.git("clone", "-q", str(self.origin), str(other), cwd=self.tmp)
        (other / "new").write_text("new\n")
        self.git("add", "new", cwd=other)
        self.git("commit", "-q", "-m", "someone else's", cwd=other)
        self.git("push", "-q", "origin", "HEAD:main", cwd=other)
        self.base = self.origin_main()
        self.refused("behind its origin's main")

    def test_an_app_checkout_with_unpushed_commits_is_refused(self) -> None:
        (self.app / "new").write_text("new\n")
        self.git("add", "new")
        self.git("commit", "-q", "-m", "local work")
        self.refused("local work")

    def test_a_checkout_that_cannot_build_the_site_is_refused(self) -> None:
        shutil.rmtree(self.app / "jubarte-site" / "public")
        self.refused("fixtures")

    def test_an_older_site_script_is_refused(self) -> None:
        script = self.app / "jubarte-site" / "scripts" / "release.sh"
        script.write_text("#!/usr/bin/env bash\n# scripts/release.sh engine x.y.z\nexit 2\n")
        self.git("commit", "-q", "-am", "an older site")
        self.git("push", "-q", "origin", "main")
        self.base = self.origin_main()
        self.refused("update it")

    def test_a_resumed_run_takes_the_changes_an_earlier_one_left(self) -> None:
        self.assertEqual(self.run_downstream(PNPM_FAIL="typecheck").returncode, 1)
        self.assertIn("M data/facts.jsonl", self.git("status", "--porcelain"))
        r = self.run_downstream()
        self.assertEqual(r.returncode, 0, r.stderr + r.stdout)
        self.assertEqual(self.facts().count('"9.9.9"'), 1)
        self.assertEqual(self.origin_main(), self.git("rev-parse", "HEAD"))

    # --- before the release publishes anything --------------------------------

    def test_preflight_checks_the_checkout_and_runs_nothing(self) -> None:
        r = self.run_downstream("--preflight")
        self.assertEqual(r.returncode, 0, r.stderr + r.stdout)
        self.assertIn(str(self.app), r.stdout)
        self.assertEqual(self.called(), "")
        self.assertEqual(self.git("rev-parse", "HEAD"), self.base)
        self.git("checkout", "-q", "-b", "feature")
        self.assertEqual(self.run_downstream("--preflight").returncode, 1)

    def test_release_runs_the_preflight_before_it_changes_anything(self) -> None:
        text = RELEASE_SH.read_text()
        call = 'scripts/release_downstream.sh "$VER" --preflight'
        self.assertIn(call, text)
        self.assertLess(text.index('say "0. Preflight"'), text.index(call))
        self.assertLess(text.index(call), text.index('say "1. '))


REAL_APP = os.environ.get("JUBARTE_SITE_TEST_APP", "")


@unittest.skipUnless(
    REAL_APP and shutil.which("node") and shutil.which("uv"),
    "JUBARTE_SITE_TEST_APP=<an app repository checkout> runs the two repositories together",
)
class DownstreamRealSite(unittest.TestCase):
    """The same step with the app repository's own scripts: its site release
    script, sync-release, sync-bench, check-bench and facts.py run for real
    (node, uv) on a throwaway copy of that checkout, against this checkout's
    release_info. The page the stub `pnpm test` builds and the stub `curl`
    serves is rendered by the site's own benchmark template, so the live
    check reads the markup jubarte.pro really prints. Still nothing is
    installed, deployed or fetched: pnpm, wrangler, npm, gh and curl are stubs."""

    GH = PushMain.GH.replace('case "$1 $2" in', """case "$1 $2" in
  "release view")
    case "$*" in
      *'.assets[].name'*) cat "$ASSETS.txt" ;;
      *) cat "$ASSETS.json" ;;
    esac ;;""")
    PNPM = """#!/bin/sh
echo "pnpm $*" >> "$CALLS"
[ "${PNPM_FAIL:-}" = "$1" ] && exit 1
# `pnpm test` builds the site first; the benchmark page is all the check reads.
[ "$1" = test ] && node --input-type=module -e '
import { mkdirSync, writeFileSync } from "node:fs";
import { benchmarkPage } from "./site/pages/benchmark.ts";
mkdirSync("public", { recursive: true });
writeFileSync("public/benchmark.html", benchmarkPage({ bench: "convert", id: "c-1" }).body);'
exit 0
"""
    FILES = ("data/facts.jsonl", "scripts/facts.py", "jubarte-site/package.json",
             "jubarte-site/pnpm-workspace.yaml", "jubarte-site/scripts/release.sh",
             "jubarte-site/scripts/sync-bench.ts", "jubarte-site/scripts/sync-release.ts",
             "jubarte-site/scripts/check-bench.ts", "jubarte-site/test/fixtures/RESULTS.md")

    def setUp(self) -> None:
        import re

        import check_release_artifacts

        self.tmp = Path(tempfile.mkdtemp(prefix="downstream_real_"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        root = HERE.parent
        found = sorted((root / "release_info").glob("website_data_*.jsonl"))
        self.assertEqual(len(found), 1, "one release's evidence in release_info/")
        self.ver = re.match(r"website_data_(\d+\.\d+\.\d+)_", found[0].name).group(1)
        self.engine = self.tmp / "engine"
        self.origin = self.tmp / "app-origin.git"
        self.app = self.tmp / "app"
        self.calls = self.tmp / "calls.log"
        self.gh_log = self.tmp / "gh.log"
        for f in (self.calls, self.gh_log):
            f.write_text("")

        (self.engine / "scripts").mkdir(parents=True)
        for name in ("release_downstream.sh", "push_main.sh", "check_site_live.py",
                     "check_release_facts.py", "check_release_artifacts.py"):
            shutil.copy(HERE / name, self.engine / "scripts" / name)
        shutil.copytree(root / "release_info", self.engine / "release_info")
        shutil.copy(root / "CHANGELOG.md", self.engine / "CHANGELOG.md")
        (self.engine / "jubarte-app").mkdir()

        bin_dir = self.tmp / "bin"
        bin_dir.mkdir()
        stubs = {"gh": self.GH, "pnpm": self.PNPM,
                 # the download page for the site's own check, else the benchmark page
                 "curl": '#!/bin/sh\necho "curl $*" >> "$CALLS"\ncase "$*" in\n'
                         f'  */download*) echo "<h2>Engine: jubarte-redlines {self.ver}</h2>" ;;\n'
                         '  *) cat "$LIVE_PAGE" ;;\nesac\n',
                 "npm": f'#!/bin/sh\n[ "$1" = view ] && echo {self.ver}\nexit 0\n',
                 "wrangler": '#!/bin/sh\necho "wrangler $*" >> "$CALLS"\n'}
        for name, body in stubs.items():
            (bin_dir / name).write_text(body)
            (bin_dir / name).chmod(0o755)
        v = self.ver
        assets = [f"jubarte-{v}-{t}" for t in (
            "linux-x86_64.tar.gz", "linux-aarch64.tar.gz", "macos-aarch64.tar.gz",
            "macos-x86_64.tar.gz", "windows-x86_64.zip")]
        assets += [f"jubarte_redlines-{v}-cp310-abi3-{tag}.whl"
                   for tag in check_release_artifacts.REQUIRED_WHEEL_TAGS]
        assets += [f"jubarte_redlines-{v}.tar.gz", "SHA256SUMS.txt"]
        (self.tmp / "assets.txt").write_text("\n".join(assets) + "\n")
        (self.tmp / "assets.json").write_text(json.dumps([{"name": a, "size": 1} for a in assets]))
        self.env = {**os.environ, "PATH": f"{bin_dir}{os.pathsep}{os.environ['PATH']}",
                    "CALLS": str(self.calls), "GH_LOG": str(self.gh_log),
                    "ORIGIN": str(self.origin), "ASSETS": str(self.tmp / "assets"),
                    "LIVE_PAGE": str(self.app / "jubarte-site" / "public" / "benchmark.html"),
                    "JUBARTE_APP_DIR": str(self.app),
                    "SITE_LIVE_TRIES": "2", "SITE_LIVE_WAIT_SECONDS": "0",
                    "LIVE_TRIES": "2", "LIVE_WAIT_SECONDS": "0",
                    "PUSH_MAIN_RETRY_SECONDS": "0",
                    "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.com",
                    "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.com"}
        self.env.pop("JUBARTE_SITE_NO_DEPLOY", None)

        self.git("init", "-q", "--bare", str(self.origin), cwd=self.tmp)
        self.git("symbolic-ref", "HEAD", "refs/heads/main", cwd=self.origin)
        self.git("clone", "-q", str(self.origin), str(self.app), cwd=self.tmp)
        self.git("checkout", "-q", "-b", "main")
        real = Path(REAL_APP)
        for rel in self.FILES:
            (self.app / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(real / rel, self.app / rel)
        for folder in ("site", "src"):
            shutil.copytree(real / "jubarte-site" / folder, self.app / "jubarte-site" / folder)
        (self.app / ".gitignore").write_text("node_modules/\npublic/\n")
        (self.app / "jubarte-site" / "node_modules").mkdir()
        (self.app / "jubarte-site" / "public" / "fixtures").mkdir(parents=True)
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "base")
        self.git("push", "-q", "origin", "main")
        self.base = self.git("rev-parse", "HEAD")

    def git(self, *args: str, cwd: Path | None = None) -> str:
        return subprocess.run(["git", *args], cwd=cwd or self.app, env=self.env,
                              capture_output=True, text=True, check=True).stdout.strip()

    def run_downstream(self, *args: str, **env: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["bash", str(self.engine / "scripts" / "release_downstream.sh"), self.ver, *args],
            capture_output=True, text=True, timeout=300, env={**self.env, **env})

    def test_the_release_lands_on_the_apps_main_once_and_step_14_passes(self) -> None:
        r = self.run_downstream()
        self.assertEqual(r.returncode, 0, r.stderr + r.stdout)
        self.assertIn(f"shows the {self.ver} figures", r.stdout)
        self.assertIn("pnpm run deploy", self.calls.read_text())
        head = self.git("rev-parse", "HEAD")
        self.assertNotEqual(head, self.base)
        self.assertEqual(self.git("rev-parse", "main", cwd=self.origin), head)
        self.assertEqual(self.git("status", "--porcelain", "--untracked-files=no"), "")
        facts = (self.app / "data" / "facts.jsonl").read_text()
        # step 14, as release.sh runs it: the app checkout from the environment
        step14 = subprocess.run(
            ["python3", str(self.engine / "scripts" / "check_release_facts.py"), self.ver],
            capture_output=True, text=True, env=self.env)
        self.assertEqual(step14.returncode, 0, step14.stderr)

        again = self.run_downstream()
        self.assertEqual(again.returncode, 0, again.stderr + again.stdout)
        self.assertEqual((self.app / "data" / "facts.jsonl").read_text(), facts)
        self.assertEqual(self.git("rev-parse", "HEAD"), head)
        self.assertEqual(self.git("rev-parse", "main", cwd=self.origin), head)

    def test_evidence_the_site_does_not_print_is_not_committed_or_deployed(self) -> None:
        # A results file that says something else than website_data: the
        # site's own proof refuses it, before the commit and the deploy.
        doc = next((self.engine / "release_info").glob("results_conversion_*.json"))
        data = json.loads(doc.read_text())
        data["tools"]["soffice"]["failures"] += 1
        doc.write_text(json.dumps(data))
        r = self.run_downstream()
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("site checks failed", r.stderr)
        self.assertNotIn("deploy", self.calls.read_text())
        self.assertNotIn("curl", self.calls.read_text())
        self.assertEqual(self.git("rev-parse", "HEAD"), self.base)
        self.assertEqual(self.git("rev-parse", "main", cwd=self.origin), self.base)


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
        self.assertIn("python3 scripts/test_library_readmes.py", gates)
        self.assertIn("python3 scripts/test_check_release_info.py", gates)

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
        found = re.findall(r"^\s*python3 (\S*test_\w+\.py)$", step(5), re.M)
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
GH_RELEASE_STUB = r"""#!/bin/bash
case "$*" in
  "release view v9.9.9")
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

    GH = r"""#!/bin/bash
case "$*" in
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
        for n in range(15):
            self.assertIn("step %d " % n, out)
        self.assertIn("AFTER the script", out)

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
        self.kw("Python 3.8")

    def test_item_5_unused_dependencies(self) -> None:
        self.kw("unused")

    def test_item_6_keywords_per_channel(self) -> None:
        self.kw("keywords")

    def test_item_7_changelogs_and_numbers(self) -> None:
        self.kw("CHANGELOG")
        self.kw("NUMBERS")

    def test_item_8_app_repository_release_files(self) -> None:
        self.kw("facts.jsonl")
        self.kw("vendored")  # the stale snapshot that is never the app to build

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
        self.kw("scripts/release_downstream.sh 0.11.3 --preflight")

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


if __name__ == "__main__":
    unittest.main()
