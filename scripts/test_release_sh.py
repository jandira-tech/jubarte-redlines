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

import os
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
        self.assertIn("docs/api/jubarte-", self.step6())
        self.assertIn("diff -u", self.step6())

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
        self.assertIn('check "npm        jubarte-redlines $VER" npm_cli_has', step(12))

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
        return subprocess.run(
            ["bash", str(self.tmp / "scripts" / "release_downstream.sh"), *args],
            capture_output=True, text=True, timeout=60,
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
        self.assertIn('"$SITE_DIR/scripts/release.sh" engine "$VER"', text)
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
