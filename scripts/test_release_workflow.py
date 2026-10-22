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
import unittest
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
WORKFLOW = HERE.parent / ".github" / "workflows" / "release.yml"

PUBLISH_JOBS = ("publish-crates", "publish-pypi", "publish-npm")
SHA_PIN = re.compile(r"^[^@\s]+@[0-9a-f]{40}$")


def load() -> dict:
    return yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))


def runs(job: dict) -> str:
    """Every `run:` script of a job, in order, joined."""
    return "\n".join(s.get("run", "") for s in job["steps"])


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

    def test_only_the_publish_jobs_may_mint_an_id_token(self) -> None:
        self.assertNotIn("id-token", self.wf.get("permissions", {}))
        for name, job in self.jobs.items():
            perms = job.get("permissions", {})
            if name in PUBLISH_JOBS:
                self.assertEqual(perms, {"contents": "read", "id-token": "write"}, name)
            else:
                self.assertNotIn("id-token", perms, name)

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

    def test_every_action_is_pinned_to_a_commit(self) -> None:
        for name, job in self.jobs.items():
            for s in job.get("steps", []):
                uses = s.get("uses")
                if uses and not uses.startswith("./"):
                    self.assertRegex(uses, SHA_PIN, f"{name}: {uses}")


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


if __name__ == "__main__":
    unittest.main()
