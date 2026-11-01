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


# What each GitHub-hosted runner label runs on: (OS family, architecture).
# Every release build is native, so a job could run what it ships.
RUNNERS = {
    "ubuntu-latest": ("linux", "x86_64"),
    "ubuntu-24.04-arm": ("linux", "aarch64"),
    "macos-15-intel": ("macos", "x86_64"),
    "macos-latest": ("macos", "aarch64"),
    "windows-latest": ("windows", "x86_64"),
}
TRIPLE_OS = {"linux": "unknown-linux-gnu", "macos": "apple-darwin", "windows": "pc-windows-msvc"}
# A `-musl` label is the static Linux build (#2030): its triple's libc part.
MUSL_TRIPLE = "unknown-linux-musl"
# The glibc Linux archives build in the wheels' manylinux_2_28 image (#2030),
# pinned by digest, so the binary needs no glibc newer than 2.28.
GLIBC_FLOOR = "2.28"
MANYLINUX_IMAGE = re.compile(r"^quay\.io/pypa/manylinux_2_28_(x86_64|aarch64)@sha256:[0-9a-f]{64}$")
ARCHS = {"x86_64": "x86_64", "x64": "x86_64", "aarch64": "aarch64", "arm64": "aarch64"}
WHEEL_OS = {"manylinux": "linux", "musllinux": "linux", "macos": "macos", "windows": "windows"}
# the wheels job's `manylinux` value per family (none on macOS and Windows)
MANYLINUX = {"manylinux": "2_28", "musllinux": "musllinux_1_2"}


def label_platform(artifact: str) -> tuple[str, str]:
    """`linux-aarch64` -> ("linux", "aarch64"): what an artifact's name promises.

    `linux-aarch64-musl` promises the same OS and CPU; label_is_musl tells it apart.
    """
    family, arch = artifact.removesuffix("-musl").split("-", 1)
    return family, ARCHS[arch]


def label_is_musl(artifact: str) -> bool:
    return artifact.startswith("linux-") and artifact.endswith("-musl")


def wheel_platform(artifact: str) -> tuple[str, str]:
    """A wheels-job artifact as the README names it: ("manylinux_2_28", "aarch64")."""
    family, arch = label_platform(artifact)
    tag = MANYLINUX.get(family)
    if tag is None:
        return family, arch
    return (tag if tag.startswith(family) else f"{family}_{tag}"), arch


def readme_platforms(cell: str) -> set[tuple[str, str]]:
    """`macOS x86_64/arm64, manylinux_2_28 x86_64/arm64` -> {("macos", "x86_64"), …}."""
    out = set()
    for group in cell.split(", "):
        name, archs = group.split(" ", 1)
        out |= {(name.lower(), ARCHS[a]) for a in archs.split("/")}
    return out


def runner_problems(row: dict, family: str, arch: str) -> list[str]:
    runner = RUNNERS.get(row["os"])
    if runner is None:
        return [f"unknown runner {row['os']}"]
    return [] if runner == (family, arch) else [f"{row['os']} runs {runner}, not {(family, arch)}"]


def binary_row_problems(row: dict) -> list[str]:
    family, arch = label_platform(row["artifact"])
    want = f"{arch}-{MUSL_TRIPLE if label_is_musl(row['artifact']) else TRIPLE_OS[family]}"
    problems = [] if row["target"] == want else [f"target {row['target']}, not {want}"]
    image = row.get("image")
    if row["target"].endswith("-linux-gnu"):
        m = MANYLINUX_IMAGE.match(image or "")
        if m is None:
            problems.append(f"image {image}, not quay.io/pypa/manylinux_2_28_{arch}@sha256:<digest>")
        elif m.group(1) != arch:
            problems.append(f"image {image} builds {m.group(1)}, not {arch}")
    elif image is not None:
        problems.append(f"image {image} on a {row['target']} row: only the glibc Linux rows build in a container")
    return problems + runner_problems(row, family, arch)


def wheel_row_problems(row: dict) -> list[str]:
    family, arch = label_platform(row["artifact"])
    problems = [] if ARCHS.get(row["target"]) == arch else [f"target {row['target']}, not {arch}"]
    # a string: unquoted, `2_28` is YAML 1.1's integer 228
    if row.get("manylinux") != MANYLINUX.get(family):
        problems.append(f"manylinux {row.get('manylinux')}, not {MANYLINUX.get(family)}")
    return problems + runner_problems(row, WHEEL_OS[family], arch)


# label, Rust target triple, GitHub runner, PyPI wheel platforms built on that runner.
# The wheel tags are REQUIRED_WHEEL_TAGS, in that order, for the same OS and CPU.
BINARY_MATRIX: tuple[tuple[str, str, str, tuple[str, ...]], ...] = (
    ("linux-x86_64", "x86_64-unknown-linux-gnu", "ubuntu-latest",
     ("manylinux_2_28_x86_64", "musllinux_1_2_x86_64")),
    ("linux-aarch64", "aarch64-unknown-linux-gnu", "ubuntu-24.04-arm",
     ("manylinux_2_28_aarch64", "musllinux_1_2_aarch64")),
    ("linux-x86_64-musl", "x86_64-unknown-linux-musl", "ubuntu-latest",
     ("manylinux_2_28_x86_64", "musllinux_1_2_x86_64")),
    ("linux-aarch64-musl", "aarch64-unknown-linux-musl", "ubuntu-24.04-arm",
     ("manylinux_2_28_aarch64", "musllinux_1_2_aarch64")),
    ("macos-x86_64", "x86_64-apple-darwin", "macos-15-intel",
     ("macosx_10_12_x86_64",)),
    ("macos-aarch64", "aarch64-apple-darwin", "macos-latest",
     ("macosx_11_0_arm64",)),
    ("windows-x86_64", "x86_64-pc-windows-msvc", "windows-latest",
     ("win_amd64",)),
)


def _wheel_tags_for_binary_row(row: dict, wheels_by_artifact: dict[str, dict]) -> tuple[str, ...]:
    """PyPI tags whose wheels row is this binary row's runner and CPU."""
    family, arch = label_platform(row["artifact"])
    tags = []
    for tag in REQUIRED_WHEEL_TAGS:
        artifact = wheel_artifact(tag)
        wheel = wheels_by_artifact.get(artifact)
        if wheel is None or wheel.get("os") != row.get("os"):
            continue
        wheel_family, wheel_arch = label_platform(artifact)
        if WHEEL_OS[wheel_family] == family and wheel_arch == arch:
            tags.append(tag)
    return tuple(tags)


def observed_binary_mapping(text: str) -> list[tuple[str, str, str, tuple[str, ...]]]:
    """Each binaries row of a release.yml text: label, triple, runner, wheel platforms."""
    workflow = yaml.safe_load(text)
    binaries = workflow["jobs"]["binaries"]["strategy"]["matrix"]["include"]
    wheels = workflow["jobs"]["wheels"]["strategy"]["matrix"]["include"]
    wheels_by_artifact = {row["artifact"]: row for row in wheels}
    return [
        (row["artifact"], row.get("target"), row.get("os"),
         _wheel_tags_for_binary_row(row, wheels_by_artifact))
        for row in binaries
    ]


def binary_matrix_problems(text: str) -> list[str]:
    """Where `text` breaks BINARY_MATRIX. Empty when the workflow matches it."""
    observed = observed_binary_mapping(text)
    problems: list[str] = []
    labels = [row[0] for row in observed]
    if len(labels) != len(set(labels)):
        problems.append("duplicate binary artifact label")
    by_label = {row[0]: row for row in observed}
    expected_labels = [row[0] for row in BINARY_MATRIX]
    missing = [label for label in expected_labels if label not in by_label]
    extra = [label for label in labels if label not in set(expected_labels)]
    if missing:
        problems.append("missing binary row " + ", ".join(missing))
    if extra:
        problems.append("unexpected binary row " + ", ".join(extra))
    if not missing and not extra and labels != expected_labels:
        problems.append(f"binary row order {labels}, not {expected_labels}")
    for label, target, runner, wheels in BINARY_MATRIX:
        row = by_label.get(label)
        if row is None:
            continue
        _label, got_target, got_runner, got_wheels = row
        if got_target != target:
            problems.append(f"{label} target {got_target}, not {target}")
        if got_runner != runner:
            problems.append(f"{label} runner {got_runner}, not {runner}")
        if got_wheels != wheels:
            problems.append(f"{label} wheel platforms {got_wheels}, not {wheels}")
    return problems


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
        self.assertEqual(readme_platforms(row.group(1)),
                         {wheel_platform(a) for a in matrix_artifacts(self.jobs["wheels"])})

    def test_both_build_jobs_run_on_their_rows_runner(self) -> None:
        # #1767: the row checks read row["os"]; a job that hard-codes its
        # runner builds every row on it while they stay green
        for job in ("binaries", "wheels"):
            self.assertEqual(self.jobs[job]["runs-on"], "${{ matrix.os }}", job)

    def test_every_binary_row_builds_what_its_label_names(self) -> None:
        for row in self.jobs["binaries"]["strategy"]["matrix"]["include"]:
            with self.subTest(artifact=row["artifact"]):
                self.assertEqual(binary_row_problems(row), [])

    def test_every_wheel_row_builds_what_its_label_names(self) -> None:
        for row in self.jobs["wheels"]["strategy"]["matrix"]["include"]:
            with self.subTest(artifact=row["artifact"]):
                self.assertEqual(wheel_row_problems(row), [])


class RowMutations(unittest.TestCase):
    """Codex review of #1743 (#1748): an x86-64 build in an ARM64-named
    archive passed every test. Each check catches its wrong row."""

    def test_a_wrong_architecture_target_is_caught(self) -> None:
        row = {"os": "ubuntu-24.04-arm", "target": "x86_64-unknown-linux-gnu", "artifact": "linux-aarch64"}
        self.assertTrue(binary_row_problems(row))

    def test_a_wrong_os_target_is_caught(self) -> None:
        row = {"os": "macos-latest", "target": "aarch64-unknown-linux-gnu", "artifact": "macos-aarch64"}
        self.assertTrue(binary_row_problems(row))

    def test_a_runner_of_another_architecture_is_caught(self) -> None:
        row = {"os": "ubuntu-latest", "target": "aarch64-unknown-linux-gnu", "artifact": "linux-aarch64"}
        self.assertTrue(binary_row_problems(row))

    def test_a_glibc_row_built_on_the_host_is_caught(self) -> None:
        # #2030: ubuntu-latest's own glibc (2.43 in 0.16.0) is what the binary needs.
        row = {"os": "ubuntu-latest", "target": "x86_64-unknown-linux-gnu", "artifact": "linux-x86_64"}
        self.assertTrue(binary_row_problems(row))

    def test_a_glibc_row_in_an_unpinned_or_newer_image_is_caught(self) -> None:
        for image in ("quay.io/pypa/manylinux_2_28_x86_64:latest",
                      "quay.io/pypa/manylinux_2_34_x86_64@sha256:" + "a" * 64,
                      "quay.io/pypa/manylinux_2_28_aarch64@sha256:" + "a" * 64):
            with self.subTest(image=image):
                row = {"os": "ubuntu-latest", "target": "x86_64-unknown-linux-gnu",
                       "artifact": "linux-x86_64", "image": image}
                self.assertTrue(binary_row_problems(row))

    def test_a_musl_label_with_a_glibc_target_is_caught(self) -> None:
        row = {"os": "ubuntu-latest", "target": "x86_64-unknown-linux-gnu", "artifact": "linux-x86_64-musl",
               "image": "quay.io/pypa/manylinux_2_28_x86_64@sha256:" + "a" * 64}
        self.assertTrue(binary_row_problems(row))

    def test_an_unknown_runner_is_caught(self) -> None:
        row = {"os": "ubuntu-26.04-riscv", "target": "x86_64-unknown-linux-gnu", "artifact": "linux-x86_64"}
        self.assertTrue(binary_row_problems(row))

    def test_a_wheel_of_another_architecture_is_caught(self) -> None:
        row = {"os": "ubuntu-24.04-arm", "target": "x86_64", "manylinux": "2_28", "artifact": "manylinux-aarch64"}
        self.assertTrue(wheel_row_problems(row))

    def test_a_wheel_of_another_libc_is_caught(self) -> None:
        row = {"os": "ubuntu-latest", "target": "x86_64", "manylinux": "2_28", "artifact": "musllinux-x86_64"}
        self.assertTrue(wheel_row_problems(row))

    def test_a_readme_with_another_architecture_differs(self) -> None:
        self.assertNotEqual(readme_platforms("macOS x86_64/arm64, Windows x86_64"),
                            readme_platforms("macOS x86_64/arm64, Windows arm64"))


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


class BinaryReleaseMatrix(unittest.TestCase):
    """#1748: each binaries-matrix label names its triple, runner and wheels.

    BINARY_MATRIX is that map. A copy of the workflow text whose linux-aarch64
    target is the x86-64 triple fails it.
    """

    def test_every_binaries_row_maps_label_target_runner_and_wheel_platform(self) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")
        observed = {
            label: (target, runner, wheels)
            for label, target, runner, wheels in observed_binary_mapping(text)
        }
        self.assertEqual([row[0] for row in observed_binary_mapping(text)],
                         [label for label, *_rest in BINARY_MATRIX])
        for label, target, runner, wheels in BINARY_MATRIX:
            with self.subTest(artifact=label):
                self.assertEqual(observed[label], (target, runner, wheels))
        self.assertEqual(binary_matrix_problems(text), [])

    def test_a_yaml_copy_with_the_arm64_row_targeted_at_x86_64_fails(self) -> None:
        text = WORKFLOW.read_text(encoding="utf-8")
        needle = (
            "          - os: ubuntu-24.04-arm\n"
            "            target: aarch64-unknown-linux-gnu\n"
            "            artifact: linux-aarch64\n"
        )
        mutated = text.replace(
            needle,
            needle.replace("target: aarch64-unknown-linux-gnu", "target: x86_64-unknown-linux-gnu"),
            1,
        )
        self.assertEqual(text.count(needle), 1)
        self.assertNotEqual(mutated, text)
        problems = binary_matrix_problems(mutated)
        self.assertTrue(problems)
        joined = " ".join(problems)
        self.assertIn("linux-aarch64", joined)
        self.assertIn("x86_64-unknown-linux-gnu", joined)
        self.assertEqual(binary_matrix_problems(text), [])


class LinuxPortability(unittest.TestCase):
    """#2030: the Linux CLI archives run on every supported distribution.

    0.16.0's linux-x86_64 binary needed GLIBC_2.43 (ubuntu-latest's own), so
    it failed on Ubuntu 24.04 (2.39), Debian 12 and RHEL 9, and on Alpine.
    """

    def setUp(self) -> None:
        self.job = load()["jobs"]["binaries"]
        self.steps = self.job["steps"]

    def step(self, needle: str) -> int:
        return step_index(self.job, lambda s: needle in s.get("run", ""))

    def test_the_glibc_rows_build_in_their_manylinux_image(self) -> None:
        i = self.step('"$IMAGE"')
        build = self.steps[i]
        self.assertEqual(build.get("if"), "matrix.image")
        self.assertEqual(build["env"]["IMAGE"], "${{ matrix.image }}")
        self.assertIn("docker run", build["run"])
        self.assertIn("cargo build --release --locked --bin jubarte", build["run"])

    def test_the_host_build_skips_the_container_rows(self) -> None:
        hosts = [s for s in self.steps
                 if "cargo build --release --locked --bin jubarte" in s.get("run", "")
                 and "docker run" not in s["run"]]
        self.assertEqual(len(hosts), 1)
        self.assertEqual(hosts[0].get("if"), "${{ !matrix.image }}")

    def test_the_musl_rows_get_musl_tools(self) -> None:
        i = self.step("musl-tools")
        self.assertEqual(self.steps[i].get("if"), "endsWith(matrix.target, '-musl')")
        self.assertLess(i, self.step("cargo build --release --locked --bin jubarte"))

    def test_a_gate_holds_each_linux_binary_to_its_floor_before_upload(self) -> None:
        i = self.step("GLIBC_" + GLIBC_FLOOR)
        gate = self.steps[i]
        self.assertEqual(gate.get("if"), "runner.os == 'Linux'")
        script = gate["run"]
        self.assertIn("objdump -T", script)
        self.assertIn("NEEDED", script)  # the musl build links nothing
        upload = step_index(self.job, lambda s: s.get("uses", "").startswith("actions/upload-artifact@"))
        self.assertLess(i, upload)

    def test_the_gate_runs_each_binary_in_old_and_new_distributions(self) -> None:
        script = self.steps[self.step("GLIBC_" + GLIBC_FLOOR)]["run"]
        for image in ("ubuntu:20.04", "debian:10", "ubuntu:24.04", "alpine:3.22"):
            with self.subTest(image=image):
                self.assertRegex(script, re.escape(image) + r"@sha256:[0-9a-f]{64}")
        self.assertIn("--version", script)
        self.assertIn("compare", script)


if __name__ == "__main__":
    unittest.main()
