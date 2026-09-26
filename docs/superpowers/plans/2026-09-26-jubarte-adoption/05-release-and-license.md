# Distribution, Provenance and Apache Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make installation and licensing predictable enough for production adoption, with releases traceable to tested source and artifacts.

**Architecture:** Build once on clean CI, test installed artifacts, then publish those same artifacts with scoped short-lived credentials. Treat license migration as a separately reviewed provenance change with an explicit source boundary and retained upstream notices.

**Tech Stack:** Existing GitHub Actions, maturin/PyO3, wasm-pack, npm, PyPI trusted publishing, Cargo, REUSE/SPDX, SHA-256 manifests.

---

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

## Current distribution is a starting asset

Do not rebuild release infrastructure from an obsolete draft. `.github/workflows/release.yml` already produces four Python wheels and an sdist attached to a GitHub release. `scripts/release.sh` publishes to crates.io/npm/PyPI and verifies summaries. Keep useful version/changelog/engine-stamp behavior, but replace user-token publication and local fallback builds with tested CI artifacts.

Published-wheel observation during planning: 0.9.2 exists, CPython abi3-py310 wheels for macOS x86_64/arm64 and Linux x86_64/aarch64. The registry metadata indicated manylinux_2_34 for observed Linux artifacts; do not copy an older claim of manylinux_2_17 support into new docs. Verify exact filename tags and test a matching oldest glibc container before changing portability claims. [PyPI metadata](https://pypi.org/pypi/jubarte-redlines/json)

## Support matrix and first missing platform

| Surface | First launch coverage | Explicit exclusions until proven |
|---|---|---|
| Python | CPython 3.10 and current stable on macOS arm64/x86_64; Linux glibc x86_64/aarch64; add Windows x86_64 wheel | PyPy, free-threaded Python, musllinux and Windows ARM64 |
| Node WASM | Keep raw Node ≥18 contract if retained; run existing minimum plus active supported Node LTS consumers | Unsupported bundler/worker runtime claims; Node 18 need not be recommended for new production use |
| Browser WASM | Current Chromium/Firefox with explicit asset-loading recipes | Unmeasured browsers, huge-document memory behavior and all edge runtimes |
| CLI | Existing release matrix, run `--help` on each actual host | Cross-compiled binary without runtime smoke is build evidence only |
| Rust | MSRV 1.88 plus current stable, default/library-only configurations | Nightly coverage tooling does not become product MSRV |

The build toolchain and consumer support policy are different. npm OIDC currently requires Node ≥22.14 and npm ≥11.5.1; use a pinned tested release toolchain without silently changing runtime exports. [npm trusted publishing](https://docs.npmjs.com/trusted-publishers/)

## Task R1: test every distribution as a consumer

**Files:** modify `.github/workflows/release.yml`, `.github/workflows/ci.yml`; create installed-consumer fixtures under `jubarte-python/tests/integration/` and `jubarte-wasm/consumer-tests/`.

- [ ] Extend the existing wheel matrix with Windows x86_64, using an actual Windows runner and maturin. Keep each produced tag honest; do not relabel wheel tags by renaming files.
- [ ] Build wheels and sdist once. Download each platform wheel into a fresh environment on a matching host and install with `--only-binary=:all:` so a missing wheel fails instead of invoking a compiler.
- [ ] Run coverage-enabled facade tests against installed modules, outside the source path. Exercise import, compare/accept/reject/revisions, PDF rendering and any new inspection/edit methods. The test environment must not find an editable source package ahead of the installed wheel.
- [ ] Install from sdist in a separate build job to verify packaged Rust source/fonts/path dependencies. Runtime wheel users should not need Rust or a network font download.
- [ ] Build all four WASM targets sequentially, compile/type-check facade declarations, pack once, install tarball into separate CJS/ESM/TS/browser fixtures with install scripts disabled, and test actual exports/assets.
- [ ] Verify all advertised fonts/license files and engine stamps are present in wheel, sdist, crate and npm tarball. Test cold import and first conversion; report download/unpacked sizes and peak memory separately from warm compute.
- [ ] Commit these gates before publication changes. A failing installed-artifact test blocks publishing, not just GitHub release attachment.

Recommended GitHub wheel test step, within the existing platform job after build:

```yaml
- uses: actions/setup-python@v6
  with:
    python-version: '3.10'
- name: Install the produced wheel and test tools
  shell: bash
  run: |
    python -m pip install --only-binary=:all: jubarte-python/dist/*.whl
    python -m pip install pytest pytest-cov
- name: Test installed Python package
  shell: bash
  run: |
    python -m pytest jubarte-python/tests --import-mode=importlib \
      --cov=jubarte_redlines --cov-report=term-missing --cov-branch
```

Resolve action tags to reviewed immutable SHAs during implementation, using each action's own published revision; do not invent SHA pins in a plan. For coverage of installed `.so` Rust code, run a separate instrumented Rust/binding integration job; Python coverage measures Python lines only.

## Task R2: OIDC publication of tested artifacts

**Files:** modify `.github/workflows/release.yml`, `scripts/release.sh`, `VERSIONING.md`, registry settings outside the repository.

Configure a PyPI trusted publisher bound to `jandira-tech/jubarte-redlines`, the exact workflow filename and a release environment. Grant `id-token: write` only in the publication job. A checked-in workflow alone does not create registry trust. Use TestPyPI for the first credential/metadata dry run. [PyPI trusted publishing](https://docs.pypi.org/trusted-publishers/using-a-publisher/)

The publication job must depend on version checks, all artifact tests and provenance validation; it downloads exact wheel/sdist artifacts and verifies their hashes. Its core is:

```yaml
pypi-publish:
  needs: [version, wheels, installed-package-tests]
  runs-on: ubuntu-latest
  environment: pypi
  permissions:
    id-token: write
    contents: read
  steps:
    - uses: actions/download-artifact@v8
      with:
        pattern: tested-pypi-*
        path: dist
        merge-multiple: true
    - uses: pypa/gh-action-pypi-publish@release/v1
      with:
        packages-dir: dist/
```

`installed-package-tests` and `tested-pypi-*` are the concrete job/artifact names to introduce in R1, not existing source names. The final job must include the tested sdist, not only the wheel files. Before publishing, compare hashes against a manifest stored outside `dist/` so the publisher sees only distribution files.

npm follows the same build→pack→test→hash→publish graph with its configured OIDC workflow and appropriate provenance. `npm publish` publishes the tested tarball, not a freshly rebuilt directory. Check package metadata repository URL matches source. Keep source/build permissions read-only; registry permissions exist only at the final job.

Refactor `scripts/release.sh` to prepare a release and watch CI; remove `UV_PUBLISH_TOKEN` preflight, direct `uv publish` and local wheel fallback after a CI artifact failure. A missing artifact is a failed release, not permission to produce an untested platform wheel. Retain idempotent existence checks, but compare existing published digest/version/source metadata before declaring a retry complete. Never overwrite an immutable published version.

- [ ] Configure TestPyPI/npm test publication settings after the complete workflow is reviewable.
- [ ] Run an unpublished artifact workflow and inspect every digest/test report.
- [ ] Verify no token is required in build logs and no write permission is granted to PR-triggered jobs.
- [ ] Run the first authorized release; install from the registry into clean supported environments and verify metadata/behavior again.
- [ ] Document rollback: yank a bad PyPI version where appropriate, deprecate an npm version, publish a corrected version, and preserve the bad artifact's audit trail. Do not silently delete/reupload artifacts as a repair strategy.

## Task R3: release receipt and capability checks

**Files:** create `schemas/release-manifest-v1.json`, `scripts/check_release_artifacts.py`; update the release workflow and binding smoke scripts.

A release manifest records schema version, source commit, engine version, build workflow/run identifier, toolchain, target triple, artifact filename/SHA-256, license expression, capability list, font resources and test-report hashes. Artifact names are relative basenames; no local user paths. A manifest with matching hashes proves what was built/tested, not that all behavior is correct.

Capability JSON should include separate booleans/versions for compare, revisions, PDF, inspection, edit kinds and runtime-specific font/resource features. Full/slim package capability differences are intentional. Reject a request for an unavailable feature before loading a missing export or starting a write.

Test deterministic manifest generation with in-memory byte maps, exact expected digest and strict version/license/capability mismatch cases. Keep artifact I/O in integration tests. Compare package versions across root Cargo, Python Cargo, npm package, generated package metadata and engine stamp; static source version matches do not prove a stale WASM binary was rebuilt.

## Apache-2.0: recommended, conditional migration

For this adoption goal, Apache-2.0 is desirable because it provides a permissive licensing route with an explicit patent grant. Migration requires authority over the code being relicensed and retention of applicable third-party notices; editing the license string is not a rights analysis. Apache's license defines the licensor as the rights holder or its authorized entity and sets redistribution/notice obligations. [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)

This plan does not establish who owns every historical contribution. Current repository headers and `AGENTS.md` assertions are evidence to investigate, not a substitute for contribution/assignment records. Review the Rust port's upstream provenance, external contributions, generated assets, tests/corpora and embedded/downloaded fonts. Preserve MIT attribution and OFL/Apache font licenses as their actual licenses. Do not relicense third-party material by applying a repository-wide replacement.

## Task L1: produce a rights and scope inventory

**Files:** create private review inventory plus a public provenance summary under `docs/licensing/`; read `NOTICE`, `LICENSES.md`, `REUSE.toml`, all manifests and contribution history. No license changes in this task.

- [ ] Run the existing REUSE baseline. This planning pass observed a clean baseline on the earlier checkout; rerun on the implementation commit.

```sh
uv tool run --from 'reuse[charset-normalizer]' reuse lint
git shortlog -sne --all
git log --format='%H %aN <%aE>' -- src jubarte-python jubarte-wasm
```

- [ ] For each path group, record origin, current license, rights holder, evidence of permission/assignment, proposed license and reviewer decision. Git author names alone do not prove ownership or assignment.
- [ ] Scope the initial change to engine/Python/WASM/CLI distribution and first-party documentation. Explicitly resolve the app's `LicenseRef-Proprietary` boundary; do not automatically migrate the app merely because it is a child directory.
- [ ] Separate acquired/generated fixtures from synthetic first-party ones. Keep private customer documents out of public distributions and benchmark downloads.
- [ ] Obtain review/clearance for missing rights evidence before a migration patch is applied. If uncertain contributions cannot be cleared, seek permission, exclude/reimplement the affected portion with documented provenance, or keep the existing license for that release. Do not claim the historical AGPL releases have become Apache retroactively.

## Task L2: apply one coherent license migration after clearance

**Exact modifications:** root `LICENSE` becomes the authoritative unmodified Apache-2.0 text; root project metadata `license = "Apache-2.0"`; binding manifests/npm metadata match; `jubarte-python/LICENSE` and rebuilt npm packaged LICENSE match; first-party SPDX headers within approved scope change to Apache-2.0. Update README badge/license section, `NOTICE`, `LICENSES.md`, `REUSE.toml` default annotations and first-party docs. Keep `LICENSES/LicenseRef-*` attribution texts and upstream/source font licenses unchanged.

Representative manifest diffs, conditional on L1 clearance:

```diff
--- a/Cargo.toml
+++ b/Cargo.toml
@@
-license = "AGPL-3.0-only"
+license = "Apache-2.0"
--- a/jubarte-python/Cargo.toml
+++ b/jubarte-python/Cargo.toml
@@
-license = "AGPL-3.0-only"
+license = "Apache-2.0"
--- a/jubarte-python/pyproject.toml
+++ b/jubarte-python/pyproject.toml
@@
-license = "AGPL-3.0-only"
+license = "Apache-2.0"
--- a/jubarte-wasm/Cargo.toml
+++ b/jubarte-wasm/Cargo.toml
@@
-license = "AGPL-3.0-only"
+license = "Apache-2.0"
--- a/jubarte-wasm/npm/package.json
+++ b/jubarte-wasm/npm/package.json
@@
-  "license": "AGPL-3.0-only",
+  "license": "Apache-2.0",
```

These are proposed change specifications, not a stand-alone complete or applied relicense patch. The rights-approved path inventory determines the complete header set; the full license text must be sourced unchanged from the official license, with provenance. `REUSE.toml` needs scoped overrides for any retained-license app/third-party material rather than a blanket all-files claim.

- [ ] Generate the full diff from the approved inventory, preserving copyright holders and compound upstream license obligations. Do not run a blind global text replacement across all source/assets/history.
- [ ] Add a migration note stating first Apache release version and unchanged historical version licensing. Describe commercial support separately from license permissions.
- [ ] Update dependency license policy to the intended distribution constraints and inspect transitive dependency reports; removing an allow entry without checking the dependency graph is not a licensing audit.
- [ ] Run REUSE, cargo-deny, package inventories and installed-artifact license checks. Verify wheel `.dist-info` license metadata, npm license/NOTICE/fonts and Cargo packaged files from actual archives.
- [ ] Commit/review the license change separately from behavior changes. Publish only after the reviewed scope and registry artifacts agree.

A DCO/sign-off policy or contribution agreement may improve provenance for future changes, but do not impose paperwork without a clear project need. Whatever policy is chosen, state what contributors are granting and keep a lightweight contribution path for docs/tests. A permissive license plus an unapproachable contribution process still loses users.
