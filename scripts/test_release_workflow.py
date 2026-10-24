#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Registry publishing contract of .github/workflows/release.yml (#1642).

crates.io, PyPI and npm take the release through OIDC trusted publishing:
each publish job asks GitHub for an ID token and the registry trades it for
a short-lived upload token, so no registry secret exists. The registries
trust the pair (workflow file `release.yml`, environment `release`), so
both names are part of the contract.

Run with PyYAML: `uv run --with pyyaml python3 scripts/test_release_workflow.py`.
"""

from __future__ import annotations

import re
import sys
import unittest
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
WORKFLOW = HERE.parent / ".github" / "workflows" / "release.yml"
sys.path.insert(0, str(HERE))

from check_release_artifacts import REQUIRED_WHEEL_TAGS

PUBLISH_JOBS = ("publish-crates", "publish-pypi", "publish-npm")
ATTEST_PERMISSIONS = {
    "contents": "write",
    "id-token": "write",
    "attestations": "write",
    "artifact-metadata": "write",
}
SHA_PIN = re.compile(r"^[^@\s]+@[0-9a-f]{40}$")


def load() -> dict:
    return yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))


def runs(job: dict) -> str:
    """Every `run:` script of a job, in order, joined."""
    return "\n".join(s.get("run", "") for s in job["steps"])


def matrix_artifacts(job: dict) -> list[str]:
    return [row["artifact"] for row in job["strategy"]["matrix"]["include"]]


def wheel_artifact(tag: str) -> str:
    """The wheels job's artifact name for a platform tag of REQUIRED_WHEEL_TAGS."""
    for prefix, family in (("manylinux_2_28_", "manylinux"), ("musllinux_1_2_", "musllinux")):
        if tag.startswith(prefix):
            return f"{family}-{tag.removeprefix(prefix)}"
    if tag.startswith("macosx_"):
        return "macos-aarch64" if tag.endswith("_arm64") else "macos-x86_64"
    if tag == "win_amd64":
        return "windows-x86_64"
    raise AssertionError(f"no wheels-job artifact for {tag}")


def step_index(job: dict, predicate) -> int:
    for i, s in enumerate(job["steps"]):
        if predicate(s):
            return i
    raise AssertionError("no step matches")


class TrustedPublishing(unittest.TestCase):
    def setUp(self) -> None:
        self.wf = load()
        self.jobs = self.wf["jobs"]

    def test_the_workflow_file_is_the_one_the_registries_trust(self) -> None:
        self.assertEqual(WORKFLOW.name, "release.yml")

    def test_every_registry_has_a_publish_job(self) -> None:
        for name in PUBLISH_JOBS:
            self.assertIn(name, self.jobs)

    def test_publish_jobs_run_in_the_release_environment(self) -> None:
        for name in PUBLISH_JOBS:
            env = self.jobs[name].get("environment")
            env = env.get("name") if isinstance(env, dict) else env
            self.assertEqual(env, "release", name)

    def test_only_the_publish_jobs_and_the_attestation_may_mint_an_id_token(self) -> None:
        self.assertNotIn("id-token", self.wf.get("permissions", {}))
        for name, job in self.jobs.items():
            perms = job.get("permissions", {})
            if name in PUBLISH_JOBS:
                self.assertEqual(perms, {"contents": "read", "id-token": "write"}, name)
            elif name == "release":
                self.assertEqual(perms, ATTEST_PERMISSIONS, name)
            else:
                self.assertNotIn("id-token", perms, name)

    def test_the_attesting_job_cannot_pass_for_a_publisher(self) -> None:
        # The registries trust tokens from the release environment only; a
        # token minted outside it carries no environment claim they accept.
        self.assertNotIn("environment", self.jobs["release"])

    def test_no_registry_secret_is_read_anywhere(self) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")
        for secret in ("NPM_TOKEN", "NODE_AUTH_TOKEN", "PYPI", "UV_PUBLISH_TOKEN",
                       "CARGO_REGISTRY_TOKEN }}", "CRATES"):
            self.assertNotRegex(text, r"secrets\.\w*" + re.escape(secret), secret)
        for name in PUBLISH_JOBS:
            self.assertNotIn("secrets.", yaml.safe_dump(self.jobs[name]), name)

    def test_every_publish_skips_a_version_the_registry_holds(self) -> None:
        # A rerun of the workflow (or a release.sh --local-publish that got
        # there first) must not fail on "version already exists".
        for name in PUBLISH_JOBS:
            ids = [s.get("id") for s in self.jobs[name]["steps"]]
            self.assertIn("live", ids, name)

    def test_no_build_cache_feeds_a_shipped_artifact(self) -> None:
        # A restored cache is a path for a poisoned entry into a release
        # binary or wheel (zizmor cache-poisoning): release builds start clean.
        for name, job in self.jobs.items():
            for step in job.get("steps", []):
                uses = step.get("uses", "")
                with self.subTest(job=name, uses=uses):
                    self.assertNotIn("rust-cache", uses)
                    self.assertNotIn("actions/cache", uses)
                    if "maturin-action" in uses:
                        self.assertEqual(str(step["with"].get("sccache")), "false")

    def test_every_action_is_pinned_to_a_commit(self) -> None:
        for name, job in self.jobs.items():
            for s in job.get("steps", []):
                uses = s.get("uses")
                if uses and not uses.startswith("./"):
                    self.assertRegex(uses, SHA_PIN, f"{name}: {uses}")


class Matrices(unittest.TestCase):
    """Each platform list is written in more than one place; they agree."""

    def setUp(self) -> None:
        self.jobs = load()["jobs"]

    def test_the_binary_matrix_is_the_list_release_sh_names(self) -> None:
        text = (HERE / "release.sh").read_text(encoding="utf-8")
        m = re.search(r"^\s*for a in (linux-\S+(?: \S+)*); do$", text, re.M)
        self.assertIsNotNone(m, "release.sh lost its list of release archives")
        self.assertEqual(matrix_artifacts(self.jobs["binaries"]), m.group(1).split())

    def test_the_wheel_matrix_is_the_set_check_release_artifacts_requires(self) -> None:
        artifacts = matrix_artifacts(self.jobs["wheels"])
        self.assertEqual(len(artifacts), len(set(artifacts)))
        self.assertEqual(sorted(artifacts), sorted(wheel_artifact(t) for t in REQUIRED_WHEEL_TAGS))

    def test_the_wheel_matrix_is_the_readme_row(self) -> None:
        readme = (HERE.parent / "README.md").read_text(encoding="utf-8")
        row = re.search(r"^\| Python release wheels \| (.+?) \(abi3", readme, re.M)
        self.assertIsNotNone(row, "README lost its Python release wheels row")
        archs = [a for group in row.group(1).split(", ") for a in group.split(" ", 1)[1].split("/")]
        self.assertEqual(len(archs), len(matrix_artifacts(self.jobs["wheels"])))


class Provenance(unittest.TestCase):
    """Every release asset carries a GitHub build-provenance attestation
    (SLSA, Sigstore-signed), so `gh attestation verify` ties a download to
    the release.yml run that built it."""

    def setUp(self) -> None:
        self.job = load()["jobs"]["release"]
        self.steps = self.job["steps"]

    def named(self, name: str) -> int:
        return step_index(self.job, lambda s: s.get("name") == name)

    def test_the_release_attests_every_checksummed_asset(self) -> None:
        i = step_index(self.job, lambda s: s.get("uses", "").startswith("actions/attest@"))
        self.assertEqual(self.steps[i]["with"], {"subject-checksums": "dist/SHA256SUMS.txt"})

    def test_attests_the_signed_bytes_before_anything_is_published(self) -> None:
        # Signing rewrites the archives, so the attestation follows it.
        i = step_index(self.job, lambda s: s.get("uses", "").startswith("actions/attest@"))
        self.assertLess(self.named("Sign archives"), self.named("Checksums"))
        self.assertLess(self.named("Checksums"), i)
        self.assertLess(i, self.named("Publish release"))


class Crates(unittest.TestCase):
    def setUp(self) -> None:
        self.job = load()["jobs"]["publish-crates"]

    def test_runs_on_the_self_hosted_runners(self) -> None:
        self.assertEqual(self.job["runs-on"], "self-hosted")

    def test_the_token_comes_from_the_auth_action(self) -> None:
        i = step_index(self.job, lambda s: "rust-lang/crates-io-auth-action@" in s.get("uses", ""))
        auth = self.job["steps"][i]
        pub = self.job["steps"][
            step_index(self.job, lambda s: "cargo publish --locked" in s.get("run", ""))
        ]
        self.assertEqual(
            pub["env"]["CARGO_REGISTRY_TOKEN"], f"${{{{ steps.{auth['id']}.outputs.token }}}}"
        )
        self.assertLess(i, self.job["steps"].index(pub))

    def test_the_crate_carries_its_release_note_before_it_ships(self) -> None:
        script = runs(self.job)
        self.assertLess(script.index("package.metadata.release-notes"),
                        script.index("cargo publish --locked"))


class PyPI(unittest.TestCase):
    def setUp(self) -> None:
        self.job = load()["jobs"]["publish-pypi"]

    def test_runs_on_the_self_hosted_runners(self) -> None:
        self.assertEqual(self.job["runs-on"], "self-hosted")

    def test_waits_for_every_wheel(self) -> None:
        self.assertIn("wheels", self.job["needs"])

    def test_uploads_through_trusted_publishing(self) -> None:
        self.assertIn("uv publish --trusted-publishing always", runs(self.job))

    def test_the_wheel_set_is_checked_first(self) -> None:
        script = runs(self.job)
        self.assertLess(script.index("scripts/check_release_artifacts.py"),
                        script.index("uv publish"))


class Npm(unittest.TestCase):
    def setUp(self) -> None:
        self.job = load()["jobs"]["publish-npm"]

    def test_runs_on_a_github_hosted_runner(self) -> None:
        # npm trusted publishing refuses self-hosted runners (npm docs,
        # "Supported CI/CD providers"); ENEEDAUTH is all it says.
        self.assertRegex(self.job["runs-on"], r"^ubuntu-")

    def test_needs_an_npm_that_speaks_oidc(self) -> None:
        self.assertIn("11.5.1", runs(self.job))

    def test_the_wasm_package_goes_before_the_cli_that_depends_on_it(self) -> None:
        script = runs(self.job)
        self.assertLess(script.index("jubarte-wasm/npm"), script.index("jubarte-wasm/cli"))

    def test_a_prerelease_does_not_take_the_latest_tag(self) -> None:
        self.assertIn("--tag next", runs(self.job))

    def test_a_refused_publish_names_the_trusted_publisher_causes(self) -> None:
        # 0.13.0: npm answered E404 on PUT and never reached npm. A trusted
        # publisher made after 2026-09-03 allows only `npm stage publish`
        # unless it allows npm publish, and a new one expires unless its
        # first publish lands within two days. The log says so.
        publishes = [s for s in self.job["steps"] if "npm publish" in s.get("run", "")]
        self.assertEqual(len(publishes), 2)
        for step in publishes:
            pkg = "jubarte-wasm" if step["working-directory"].endswith("/npm") else "jubarte-redlines"
            script = step["run"]
            with self.subTest(pkg=pkg):
                self.assertRegex(script, r'npm publish "\$\{tag\[@\]\}" \|\| \{')
                self.assertIn("::error::", script)
                self.assertIn(f"npm trust github {pkg} --file release.yml", script)
                self.assertIn("--allow-publish", script)
                self.assertIn("two days", script)


if __name__ == "__main__":
    unittest.main()
