<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Provider Adoption Plan 1 of 4: S1 to S10 Implementation Plan

> **Execution:** inline, by one engineer, no subagents. Run every Cargo
> command from the repository root, one at a time, in the default `target/`
> (`AGENTS.md`). Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Baseline:** `main` at `b420d64` (2026-10-02). `cargo check --all-targets
> --all-features` passes on rustc 1.97. Everything in "What was verified"
> was read from this tree or fetched live on 2026-10-02; nothing below is
> quoted from memory.

**Goal:** Close the ten gaps (S1 to S10 of the method mapping) that keep the
four provider DOCX skills from replacing their LibreOffice, pandoc, Poppler
and hand-written-XML paths with one `jubarte` call, and leave behind the
evidence a provider's skill maintainer needs to make that swap.

**Architecture:** Every capability lands in the Rust engine first (one
module, one wire type, one stable error code), then in the CLI, the Python
`Document`/`EditPlan` facade, and the WASM exports, in that order, each with
its own test. Validity is proved with the Ring-1 invariants already in
`tests/common/validity.rs`, which Task 2 promotes into the library. New edit
operations are built in the clean copy and tracked through the comparer, the
way `insert_paragraph` is today.

**Tech Stack:** Rust 1.88+ edition 2024, `quick-xml` 0.42, `serde`,
`similar`, `tiny-skia`, `image`; PyO3 0.29 abi3-py310 via maturin;
wasm-bindgen; GitHub Actions with `PyO3/maturin-action`.

---

## Why a provider would take this (the adoption case)

What each provider ships today, verified live on 2026-10-02:

| Provider | Verified tool | Documented failure or cost | Retired by |
|---|---|---|---|
| Anthropic `skills/docx/SKILL.md` (license: Proprietary) | `soffice.py --headless --convert-to pdf` + `pdftoppm -jpeg -r 100` | LibreOffice in a sandbox, two binaries, a shim | already (`convert --png`) |
| Anthropic | `unzip`, `find -type l -delete`, `merge_runs.py`, hand-edit `word/document.xml`, `zip -Xr` | agent writes `w:ins`/`w:del` by hand; schema order is "enforced" by the reader | already (`edit`) + S1 |
| Anthropic | `validate.py --original --author --auto-repair` | XSD only; "passes XSD but fails in Word" is undetected | **S3** |
| Anthropic | `accept_changes.py` (LibreOffice) | "joins them correctly, except when the deleted paragraph is followed by an empty spacer paragraph" (quoted from the skill) | already (`accept`) |
| Anthropic | `comment.py` with `--parent`, six cross-linked parts, markers pasted by hand | "until you place those markers, the comment exists but is not visible" | **S2** |
| Anthropic | `pandoc -t markdown` | lossy, no paragraph coordinates | already (`text`) + S7 |
| Anthropic | docx-js for creation; "Page size defaults to A4" footgun | eleven listed footguns | **S7** (`--page letter`) |
| OpenAI Codex `doc` skill (`firecrawl/openai-skills` mirror of `skills/.curated/doc`) | `python-docx` + `scripts/render_docx.py` (pdf2image, Poppler, `soffice -env:UserInstallation=...`) | three system dependencies; Codex issue #38313 reports no page-range render and no timeout | already (`convert --png`) + S9 |
| OpenAI Codex `doc` | `python-docx` for edits | no tracked changes at all | S1, S2, **S8** |
| ChatGPT container skill (`/home/oai/skills/docx/SKILL.md`, seen only through a third-party issue excerpt; **not verified**) | `render_docx.py`, `fields_materialize`, redact, scrub, protection, comments | reported, not read | **S4, S5, S6** |
| Google Gemini CLI | `UseJunior/safe-docx` MCP extension | jubarte has no MCP server (plan 3, S16) | out of this plan |
| Z.ai | `postcheck.py`, `fix_footer_fields.py`, `add_toc_placeholders.py` (**not re-verified**) | WPS parity out of scope | **S6** partially |

The ordering below is the mapping document's: trust before capability. A
provider's reviewer screens install (S10) and validity (S3) before looking at
any feature. S12 (input admission on every entry point) sits in plan 2 but
must land before any security pitch; see "Dependencies across the four
plans".

**License.** The repository is AGPL-3.0-only (`LICENSE`, `Cargo.toml`,
PyPI, npm). This plan does not change it; the rights review is
[05-release-and-license.md](2026-09-26-jubarte-adoption/05-release-and-license.md).
Every capability in this plan is therefore kept reachable through a
separately installed CLI (`jubarte`, `uvx jubarte-redlines`,
`npx jubarte-redlines`), so a provider skill can call it as an external tool
without linking it. Whether that satisfies a given provider's counsel is their
call; the plan only makes sure the technical path exists.

## What was verified on 2026-10-02 (corrections to the mapping document)

The mapping document could not read `src/`. These readings change its plan:

| Mapping claim | What the tree says | Consequence |
|---|---|---|
| `markdown_to_docx`, CriticMarkup, `apply_markdown`, `rewrite`, `insert_paragraph like`, `jubarte diff`, uvx/npx CLIs exist "only in PRs" | All on `main` (`src/markdown/`, `src/edit/rewrite.rs`, `jubarte-wasm/cli/`, `jubarte_redlines.__main__`), unreleased since 0.10.1 (`CHANGELOG.md` Unreleased) | S7 becomes: release, add `--page`, add Python/WASM creation entry points |
| Edit plan fields are `op` / `with` | Wire tag is `kind`; fields are `paragraph`, `find`, `replacement` (`src/edit.rs:118-270`); unknown fields are refused with `INVALID_PLAN` by `check_operation_keys` (`src/edit.rs:783`) | S1 must extend `check_operation_keys` as well as the structs |
| Refusal only on non-unique anchor | `find_range` (`src/edit.rs:1938`) already counts hits into `EditOutcome.matches` and refuses `AMBIGUOUS_ANCHOR`; paragraph selectors list the matching ids | S1 is a small change at one call site |
| Font substitution "is not reported in `--report`" | `RenderReport.fonts: Vec<FontReportEntry{requested, step, physical, bold, italic, synthetic}>` is in `convert --report`, Python `FontResolution`, and `render()` | S9 shrinks to a `substituted` flag and `--fail-on-substitution` |
| Admission everywhere (S12) | `admission::admit` is called only from `inspect::Opened::open` (`src/inspect.rs:290`) | plan 2; prerequisite for the security pitch |
| Comments "can only be added" | True for the API; `TODO.md` §6 already specifies add, delete, modify, reply, list, surroundings, resolve; accept/reject already maintain all five comment parts (`src/revision_processor/comments.rs`) | S2 follows TODO §6 verbatim |
| Validator is internal | Ring 1 lives in `tests/common/validity.rs` (912 lines, 12 checks); `jubarte debug` runs five triage checks as prose (`src/debug.rs:58-120`); `tools/validate-docx` is .NET OpenXmlValidator | S3 promotes Ring 1 to `src/validate.rs` and gives `debug`'s triage a structured output |
| `capabilities().limits.stories == ["body"]` | Headers, footers, footnotes and endnotes are editable (`tests/edit_stories.rs`, skill §1) | manifest is stale; fixed in Task 2 |
| No Windows wheel, glibc 2.34 floor | PyPI 0.10.1: four wheels, all `manylinux_2_34`; `release.yml` wheels matrix has no Windows, no musl; CI tests already pass on `windows-latest`; `release.yml` binaries job already builds `windows-x86_64` with `core.longpaths` | S10 is a workflow change plus a published-artifact check |
| MCP sketch uses `FastMCP` | MCP Python SDK 2.2.0: `from mcp.server import MCPServer`; `Client(server)` connects in-process for tests | plan 3 |
| `compare` keeps input revisions | Word-visual mode accepts both inputs first when either carries tracked changes (`compare_documents_impl`, `docs/C4_preexisting_revisions_decision.md`) | S19 (plan 4) cannot go through compare |

## Gates for every task

Run after each implementation step, before each commit:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --test <the test file of the task>
cargo run --bin jubarte -- --help >/dev/null     # CLI smoke
uv tool run --from 'reuse[charset-normalizer]' reuse lint   # new files carry SPDX headers
```

Coverage on the touched crate before the task's last commit:

```bash
cargo llvm-cov --all-features --lcov --output-path target/lcov.info
# the CI floor is 80% lines (ci.yml "Line coverage floor")
```

Python tasks: `cd jubarte-python && uv run --with maturin maturin develop
--release && uv run --with pytest --with pytest-cov pytest -q
--cov=jubarte_redlines --cov-branch --cov-report=term-missing`.

Every produced `.docx` in a new test goes through
`common::validity::assert_word_valid_package` (Task 2 makes that the library
`validate()`), and the release gate adds Ring 2 (`scripts/redline-sweep.sh
--validate`) and Ring 3 (Word probe, macOS) as `VERSIONING.md` step 1 says.

## Task order and dependencies

| Task | Suggestion | Depends on | Unlocks |
|---|---|---|---|
| 1 | S10 wheels and Windows | nothing | every provider install |
| 2 | S3 `validate`/`repair` | nothing | tests of 3, 7, 8, 9; plans 2 to 4 |
| 3 | S2 comment threads | 2 (validity oracle) | Anthropic `comment.py`, OpenAI comments tools |
| 4 | S1 `occurrence` | nothing | fewer retries in every agent loop |
| 5 | S9 substitution flag | nothing | OpenAI/Codex render QA |
| 6 | S7 Markdown creation shipped | 1 (release) | creation path for Google, Z.ai |
| 7 | S6 field and TOC refresh | 2 | OpenAI container, Z.ai field scripts |
| 8 | S8 structural operations | 2, 4 | python-docx retirement |
| 9 | S4 scrub and redact | 2 | OpenAI container privacy tools |
| 10 | S5 settings and protection | 2 | OpenAI `set_protection.py`, Z.ai settings |
| 11 | Evidence for providers | all | the pitch |

Across plans: plan 2's S12 (admission on compare, convert, changes) is a
prerequisite for presenting any of this to a provider's security reviewer;
plan 4's S18 (`LEGACY_DOC`) is a one-line branch inside the same admission
function and should ride S12's commit.

---

### Task 1: S10, Windows wheel, glibc 2.28 floor, musllinux, and an artifact check

**Files:**
- Modify: `.github/workflows/release.yml:143-181` (the `wheels` job)
- Create: `scripts/check_release_artifacts.py`
- Create: `scripts/test_check_release_artifacts.py`
- Modify: `.github/workflows/ci.yml:149-160` (`convert-sweep-unit` job runs the new test)
- Modify: `scripts/release.sh` (step 10, before `uv publish`)
- Modify: `README.md:699-706` (platform table), `jubarte-python/pyproject.toml` (classifiers), `CHANGELOG.md` (Unreleased / Added)

Facts: PyPI 0.10.1 ships `macosx_10_12_x86_64`, `macosx_11_0_arm64`,
`manylinux_2_34_aarch64`, `manylinux_2_34_x86_64`. `maturin-action` with no
`manylinux` input builds on the host, which is why the tag is 2_34
(ubuntu-latest glibc). The action's README documents `manylinux: 2_28`
(quay.io/pypa/manylinux_2_28 containers for x86_64 and aarch64 natively on
`ubuntu-24.04-arm`), `manylinux: musllinux_1_2`, and host builds on
`windows-latest`. CI already runs `cargo test` on `windows-latest` with
`core.longpaths`, and the binaries job already ships `windows-x86_64`.

- [ ] **Step 1: Write the failing test for the artifact check**

```python
# scripts/test_check_release_artifacts.py
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""The release gate refuses a wheel set that misses an advertised platform."""
from __future__ import annotations

import unittest

from check_release_artifacts import REQUIRED_WHEEL_TAGS, missing_platforms

PUBLISHED_0_10_1 = [
    "jubarte_redlines-0.10.1-cp310-abi3-macosx_10_12_x86_64.whl",
    "jubarte_redlines-0.10.1-cp310-abi3-macosx_11_0_arm64.whl",
    "jubarte_redlines-0.10.1-cp310-abi3-manylinux_2_34_aarch64.whl",
    "jubarte_redlines-0.10.1-cp310-abi3-manylinux_2_34_x86_64.whl",
    "jubarte_redlines-0.10.1.tar.gz",
]


class MissingPlatforms(unittest.TestCase):
    def test_0_10_1_set_misses_windows_musl_and_the_2_28_floor(self) -> None:
        missing = missing_platforms(PUBLISHED_0_10_1, version="0.10.1")
        self.assertEqual(
            missing,
            {
                "win_amd64",
                "manylinux_2_28_x86_64",
                "manylinux_2_28_aarch64",
                "musllinux_1_2_x86_64",
                "musllinux_1_2_aarch64",
            },
        )

    def test_a_complete_set_passes_and_a_foreign_version_is_ignored(self) -> None:
        names = [f"jubarte_redlines-0.11.0-cp310-abi3-{tag}.whl" for tag in REQUIRED_WHEEL_TAGS]
        names += ["jubarte_redlines-0.11.0.tar.gz", "jubarte_redlines-0.10.1-cp310-abi3-win_amd64.whl"]
        self.assertEqual(missing_platforms(names, version="0.11.0"), set())

    def test_a_2_34_wheel_does_not_satisfy_the_2_28_floor(self) -> None:
        names = [f"jubarte_redlines-0.11.0-cp310-abi3-{tag}.whl" for tag in REQUIRED_WHEEL_TAGS]
        names.remove("jubarte_redlines-0.11.0-cp310-abi3-manylinux_2_28_x86_64.whl")
        names.append("jubarte_redlines-0.11.0-cp310-abi3-manylinux_2_34_x86_64.whl")
        self.assertEqual(missing_platforms(names, version="0.11.0"), {"manylinux_2_28_x86_64"})


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run it and watch it fail**

Run: `python3 scripts/test_check_release_artifacts.py`
Expected: `ModuleNotFoundError: No module named 'check_release_artifacts'`

- [ ] **Step 3: Write the check script**

```python
#!/usr/bin/env python3
# scripts/check_release_artifacts.py
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Refuse to publish a wheel set that lacks an advertised platform.

`python3 scripts/check_release_artifacts.py dist/ --version 0.11.0` exits 1
and names every missing tag. The required set is the README's platform
table; change both together.
"""
from __future__ import annotations

import argparse
import pathlib
import re
import sys

REQUIRED_WHEEL_TAGS: tuple[str, ...] = (
    "macosx_10_12_x86_64",
    "macosx_11_0_arm64",
    "manylinux_2_28_x86_64",
    "manylinux_2_28_aarch64",
    "musllinux_1_2_x86_64",
    "musllinux_1_2_aarch64",
    "win_amd64",
)

_WHEEL = re.compile(r"^jubarte_redlines-(?P<version>[^-]+)-cp310-abi3-(?P<tag>.+)\.whl$")


def missing_platforms(filenames: list[str], *, version: str) -> set[str]:
    """Required tags with no wheel of exactly `version`. A manylinux_2_34
    wheel does not satisfy the 2_28 requirement: pip on glibc 2.28 would
    refuse it."""
    present: set[str] = set()
    for name in filenames:
        m = _WHEEL.match(pathlib.PurePosixPath(name).name)
        if m and m.group("version") == version:
            present.add(m.group("tag"))
    return {tag for tag in REQUIRED_WHEEL_TAGS if tag not in present}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dist", type=pathlib.Path)
    parser.add_argument("--version", required=True)
    args = parser.parse_args(argv)
    names = [p.name for p in args.dist.iterdir()]
    missing = missing_platforms(names, version=args.version)
    if missing:
        print("missing wheels for " + args.version + ": " + ", ".join(sorted(missing)), file=sys.stderr)
        return 1
    print(f"all {len(REQUIRED_WHEEL_TAGS)} advertised wheel platforms present for {args.version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Step 4: Run the test and watch it pass**

Run: `python3 scripts/test_check_release_artifacts.py`
Expected: `Ran 3 tests ... OK`

- [ ] **Step 5: Change the wheels matrix**

Replace the `matrix.include` of the `wheels` job in
`.github/workflows/release.yml` with:

```yaml
        include:
          - os: ubuntu-latest
            target: x86_64
            manylinux: 2_28
            artifact: manylinux-x86_64
          - os: ubuntu-24.04-arm
            target: aarch64
            manylinux: 2_28
            artifact: manylinux-aarch64
          - os: ubuntu-latest
            target: x86_64
            manylinux: musllinux_1_2
            artifact: musllinux-x86_64
          - os: ubuntu-24.04-arm
            target: aarch64
            manylinux: musllinux_1_2
            artifact: musllinux-aarch64
          - os: macos-15-intel
            target: x86_64
            artifact: macos-x86_64
          - os: macos-latest
            target: aarch64
            artifact: macos-aarch64
          - os: windows-latest
            target: x64
            artifact: windows-x86_64
```

and the steps with:

```yaml
    steps:
      # Git for Windows stops at MAX_PATH (260) without core.longpaths (see
      # the binaries job and tests/repo_paths_fit_windows.rs).
      - if: runner.os == 'Windows'
        run: git config --global core.longpaths true
      - uses: actions/checkout@v7
        with:
          persist-credentials: false
      - uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.target }}
          manylinux: ${{ matrix.manylinux || 'auto' }}
          args: --release --locked --out dist --compatibility pypi
          working-directory: jubarte-python
          sccache: "true"
      # Install the wheel that was just built and run the engine once; a
      # wheel that imports is the artifact that ships, not a rebuilt one.
      - if: runner.os != 'Linux' || matrix.manylinux == '2_28'
        shell: bash
        run: |
          set -euo pipefail
          python3 -m pip install --no-index --find-links jubarte-python/dist jubarte-redlines
          python3 -c "import jubarte_redlines as j; print(j.capabilities()['engine_version'])"
      - if: matrix.manylinux == 'musllinux_1_2'
        run: |
          set -euo pipefail
          docker run --rm -v "$PWD/jubarte-python/dist:/dist:ro" python:3.12-alpine \
            sh -ec 'pip install --no-index --find-links /dist jubarte-redlines && python -c "import jubarte_redlines as j; print(j.capabilities()[\"engine_version\"])"'
      - uses: actions/upload-artifact@v7
        with:
          name: wheel-${{ needs.version.outputs.version }}-${{ matrix.artifact }}
          path: jubarte-python/dist/
          if-no-files-found: error
```

`--locked` makes the wheel build fail when `jubarte-python/Cargo.lock`
drifts, which is what a release wants. If `--compatibility pypi` is rejected
by the pinned maturin, drop that flag; it is a pre-upload check, not the
build.

- [ ] **Step 6: Gate the release script on the artifact check**

In `scripts/release.sh`, in step 10 (PyPI), before `uv publish`:

```bash
python3 scripts/check_release_artifacts.py dist --version "$VER" \
  || die "wheel set incomplete; see scripts/check_release_artifacts.py"
```

Add to `.github/workflows/ci.yml` `convert-sweep-unit` job:

```yaml
      - run: python3 scripts/test_check_release_artifacts.py
```

- [ ] **Step 7: Documentation**

README platform table row "Python release wheels": "macOS x86_64/arm64,
manylinux_2_28 x86_64/arm64, musllinux_1_2 x86_64/arm64, Windows x86_64
(abi3, CPython 3.10+)". `pyproject.toml` classifiers add
`"Operating System :: Microsoft :: Windows"`, `"Operating System :: POSIX ::
Linux"`, `"Operating System :: MacOS"`. CHANGELOG Unreleased / Added:
"Python wheels for Windows x86_64 and musl Linux; the glibc floor drops from
2.34 to 2.28 (RHEL 8, Debian 10, Ubuntu 20.04)."

- [ ] **Step 8: Prove it on a tag before the real release**

Push a pre-release tag `v0.11.0-rc.1` (matching a temporary `Cargo.toml`
version on a branch) and confirm the release workflow attaches seven wheels
and the Windows binary. Do not publish the rc to PyPI. Delete the tag after.

- [ ] **Step 9: Commit**

```bash
git add .github/workflows/release.yml .github/workflows/ci.yml scripts/check_release_artifacts.py scripts/test_check_release_artifacts.py scripts/release.sh README.md jubarte-python/pyproject.toml CHANGELOG.md
git commit -m "build(release): Windows and musl wheels, glibc 2.28 floor, wheel-set gate before publish"
```

**Evidence this task hands a provider:** a PyPI file list with seven wheels,
and `uvx jubarte-redlines capabilities` working on Windows, Alpine and
a glibc 2.28 image, captured in `docs/adoption/install-matrix.md` (Task 11).

---

### Task 2: S3, public `validate` and `repair`

**Files:**
- Create: `src/validate.rs` (the Ring-1 checks, structured)
- Modify: `src/lib.rs` (`pub mod validate;`)
- Modify: `src/debug.rs:2168` (`report` keeps its prose; add `pub fn findings(a: &[u8]) -> Vec<validate::Finding>` for the five `TRIAGE` checks)
- Modify: `tests/common/validity.rs` (delegate `check_word_valid_package` to `jubarte::validate::validate`, keep the public helper names so 100+ tests do not change)
- Modify: `src/bin/jubarte.rs:136` (`Command::Validate`)
- Modify: `src/capabilities.rs` (`operations.validate`, `operations.repair`; fix `limits.stories`)
- Modify: `jubarte-python/src/lib.rs`, `python/jubarte_redlines/{_native.pyi,document.py,models.py,__main__.py,__init__.py}`
- Modify: `jubarte-wasm/src/lib.rs` (`validateDocument`, `repairDocument`)
- Test: `tests/validate_public.rs`, `jubarte-python/tests/test_validate.py`
- Docs: `skills/jubarte-documents/SKILL.md` §3, `README.md`, `CHANGELOG.md`

Design, fixed before coding:

```rust
/// One thing wrong with a package. `path` is `part#element-chain`
/// (`word/document.xml#w:body/w:p[3]/w:r[2]`), stable across runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Stable code (`TEXT_INSIDE_DELETION`, `MC_UNBOUND_PREFIX`, ...).
    pub code: String,
    /// Package part the finding sits in.
    pub part: String,
    /// Element chain inside the part, or empty for package-level findings.
    pub path: String,
    /// Human-readable detail.
    pub message: String,
    /// True when Word refuses or repairs the file for this (Ring-1 and
    /// Ring-3 evidence); false for leads (`DUPLICATE_DOCPR_ID`).
    pub word_fatal: bool,
    /// True when [`repair`] fixes it deterministically.
    pub repairable: bool,
}

/// Findings in document order; an empty vector is a pass.
pub fn validate(docx: &[u8]) -> Result<Vec<Finding>, ValidateError>;

/// The package with every repairable finding fixed, the findings it fixed
/// and the ones it could not. Untouched parts keep their bytes.
pub struct Repaired { pub docx: Vec<u8>, pub repaired: Vec<Finding>, pub remaining: Vec<Finding> }
pub fn repair(docx: &[u8]) -> Result<Repaired, ValidateError>;

/// `validate.py --original --author` without an XSD: every visible-text
/// difference between `original` and `edited` must sit in a revision by
/// `author`. Rejecting that author's changes must give back `original`'s
/// text; any residue is an `UNTRACKED_EDIT` finding at its paragraph id.
pub fn audit_tracked(original: &[u8], edited: &[u8], author: &str) -> Result<Vec<Finding>, ValidateError>;
```

Codes and their sources (all existing logic):

| Code | From | word_fatal | repairable |
|---|---|---|---|
| `MC_UNBOUND_PREFIX` | `check_namespace_qname_contexts` (PR #270) | yes | yes: bind on the part root |
| `MISSING_CONTENT_TYPE`, `OVERRIDE_WITHOUT_PART`, `MALFORMED_XML` | `check_content_types_and_xml` | yes | no |
| `DANGLING_RELATIONSHIP`, `DUPLICATE_RID`, `MISSING_REL_TARGET` | `check_relationship_integrity` | yes | yes: `comparer::parts::reconcile_dangling_relationships` |
| `PICTURE_BULLET_UNDEFINED` | `check_picture_bullets` | yes | no |
| `DUPLICATE_REVISION_ID` | `check_revision_and_drawing_ids` | no (lead) | yes: renumber |
| `DUPLICATE_DOCPR_ID` | same | no (lead) | yes: `comparer::fixups::fix_up_drawing_ids_in_package` |
| `PARA_ID_OUT_OF_RANGE` | `check_para_text_id_bounds` | yes | yes |
| `TEXT_INSIDE_DELETION`, `DELTEXT_OUTSIDE_DELETION`, `MOVEFROM_WITH_DELTEXT` | `check_del_text_under_del`, `check_deleted_text_has_deletion` | yes | yes: swap `w:t`/`w:delText` |
| `BOOKMARK_IN_SINGLE_VALUE_CONTROL` | `check_bookmarks_outside_single_value_controls` | yes | yes: drop the pair |
| `COMMENT_ANCHOR_ORPHAN`, `COMMENT_WITHOUT_ANCHOR`, `COMMENT_PARTS_INCONSISTENT`, `COMMENT_PARENT_CYCLE` | `check_comment_graph` and helpers | yes | partly (drop orphan anchors) |
| `FIELD_UNBALANCED`, `FIELD_SPLIT_DELETION` | `debug::Check::Fields` | yes | no |
| `EMPTY_FIELD_CODE`, `CELL_WITHOUT_PARAGRAPH`, `ROW_WITHOUT_CELL`, `NESTED_SAME_REVISION`, `SECTPR_NOT_LAST` | `debug::Check::Structure` | yes | `CELL_WITHOUT_PARAGRAPH` yes |
| `UNTRACKED_EDIT`, `FOREIGN_AUTHOR` | `audit_tracked` | n/a | no |

XSD validation stays where it is: `tools/validate-docx` (OpenXmlValidator,
Ring 2) runs in the release sweep. Do not vendor the ECMA-376 schemas into
the crate; their license has not been reviewed for redistribution and the
agent-facing need is "will Word open it", which XSD does not answer.

- [ ] **Step 1: Write the failing integration test**

```rust
// tests/validate_public.rs
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::validate`: Ring-1 invariants as data, with repair and the
//! tracked-edit audit.

mod common;

use common::docx::{docx, para};
use jubarte::document_comparer::compare_documents;
use jubarte::inspect::markdown;
use jubarte::validate::{audit_tracked, repair, validate};

#[test]
fn text_inside_a_deletion_is_word_fatal_and_repairable() {
    let body = r#"<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    let f = findings.iter().find(|f| f.code == "TEXT_INSIDE_DELETION").expect("finding");
    assert!(f.word_fatal && f.repairable);
    assert_eq!(f.part, "word/document.xml");
    assert!(f.path.starts_with("w:body/w:p[0]"), "{}", f.path);

    let fixed = repair(&docx(body)).unwrap();
    assert_eq!(fixed.repaired.len(), 1);
    assert!(fixed.remaining.is_empty());
    assert!(validate(&fixed.docx).unwrap().is_empty());
    assert!(common::docx::part_string(&fixed.docx, "word/document.xml").unwrap().contains("<w:delText>gone</w:delText>"));
}

#[test]
fn an_unbound_mc_requires_prefix_is_word_fatal() {
    let body = r#"<w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:t>x</w:t></mc:Choice><mc:Fallback><w:t>x</w:t></mc:Fallback></mc:AlternateContent></w:r></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    assert!(findings.iter().any(|f| f.code == "MC_UNBOUND_PREFIX" && f.word_fatal && f.message.contains("wps")));
}

#[test]
fn a_clean_compare_output_validates_empty() {
    let redline = compare_documents(&docx(&para("one two")), &docx(&para("one three")), "R").unwrap();
    assert_eq!(validate(&redline).unwrap(), Vec::new());
}

#[test]
fn the_audit_names_an_untracked_edit_by_paragraph() {
    let original = docx(&(para("Fee is 10.") + &para("Term is 2 years.")));
    // A hand edit: the second paragraph changed with no w:ins/w:del at all.
    let edited = docx(&(para("Fee is 10.") + &para("Term is 3 years.")));
    let findings = audit_tracked(&original, &edited, "Reviewer").unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "UNTRACKED_EDIT");
    assert!(findings[0].message.contains("body:p:1"), "{}", findings[0].message);
    // The same edit done by the engine is fully tracked.
    let tracked = compare_documents(&original, &edited, "Reviewer").unwrap();
    assert!(audit_tracked(&original, &tracked, "Reviewer").unwrap().is_empty());
    assert_eq!(markdown(&tracked).unwrap().matches("[body:p:").count(), 2);
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test --all-features --test validate_public`
Expected: `error[E0433]: failed to resolve: could not find validate in jubarte`

- [ ] **Step 3: Move the checks into the library**

Create `src/validate.rs` by moving the twelve `check_*` functions from
`tests/common/validity.rs` verbatim, replacing `report.fail(msg)` with
`out.push(Finding{..})` carrying the code from the table above, and
replacing the `jubarte::` imports with `crate::`. Element chains come from
a small helper:

```rust
/// `w:body/w:p[3]/w:r[2]`: local names with the index among same-named
/// siblings, root excluded.
fn element_path(dom: &Dom, node: NodeId) -> String {
    let mut parts = Vec::new();
    let mut cur = Some(node);
    while let Some(n) = cur {
        let Some(parent) = dom.parent(n) else { break };
        if dom.parent(parent).is_none() { break; }
        let name = dom.name(n).map(|x| x.local_name().to_string()).unwrap_or_default();
        let index = dom.elements(parent, dom.name(n)).iter().position(|&e| e == n).unwrap_or(0);
        parts.push(format!("w:{name}[{index}]"));
        cur = Some(parent);
    }
    parts.reverse();
    parts.join("/")
}
```

(`Dom::parent`, `Dom::elements(node, Option<&XName>)`, `Dom::name` exist in
`src/xmllinq/mod.rs`; check their exact signatures before use.)

`validate()` opens with `crate::admission::admit(docx, InputLimits::default())`,
then `PartFs::open`, runs every check, then the five `debug::TRIAGE` checks
through the new `debug::findings`, and sorts findings by `(part, path)`.

`repair()` applies, in this order and only when the finding is present:
`MC_UNBOUND_PREFIX` (reuse the prefix-binding logic from commit `f3e080e`,
`src/xmllinq/serialize.rs`), `TEXT_INSIDE_DELETION` and
`DELTEXT_OUTSIDE_DELETION` (rename `w:t`/`w:delText`, the same transform
`revision_processor` uses), `BOOKMARK_IN_SINGLE_VALUE_CONTROL` (remove the
start/end pair), `DANGLING_RELATIONSHIP` (`reconcile_dangling_relationships`
with no sources, which drops the attribute), `DUPLICATE_DOCPR_ID`
(`fixups::fix_up_drawing_ids_in_package`), `DUPLICATE_REVISION_ID`
(renumber from `max+1`), `CELL_WITHOUT_PARAGRAPH` (append `<w:p/>`), then
re-runs `validate()` to fill `remaining`.

`audit_tracked()`:

```rust
pub fn audit_tracked(original: &[u8], edited: &[u8], author: &str) -> Result<Vec<Finding>, ValidateError> {
    use crate::changes::{list_changes, reject_changes, ChangeFilter};
    let mut out = Vec::new();
    let listed = list_changes(edited).map_err(ValidateError::from)?;
    for c in listed.iter().filter(|c| c.author.as_deref() != Some(author)) {
        out.push(Finding::new("FOREIGN_AUTHOR", "word/document.xml", "", format!("{} by {}", c.id, c.author.clone().unwrap_or_default()), false, false));
    }
    let filter = ChangeFilter { authors: Some(vec![author.to_string()]), ..ChangeFilter::default() };
    let reverted = reject_changes(edited, &filter).map_err(ValidateError::from)?;
    // Paragraph-by-paragraph text: the same projection `jubarte text` prints.
    let before = crate::inspect::paragraphs(original)?;
    let after = crate::inspect::paragraphs(&reverted)?;
    if before.len() != after.len() {
        out.push(Finding::new("UNTRACKED_EDIT", "word/document.xml", "", format!("{} paragraphs before, {} after rejecting {author}'s changes", before.len(), after.len()), false, false));
        return Ok(out);
    }
    for (b, a) in before.iter().zip(&after) {
        if b.text != a.text {
            out.push(Finding::new("UNTRACKED_EDIT", "word/document.xml", a.id.clone(), format!("{} differs after rejecting {author}'s changes: {:?} vs {:?}", a.id, b.text, a.text), false, false));
        }
    }
    Ok(out)
}
```

Then rewrite `tests/common/validity.rs` so `check_word_valid_package` calls
`jubarte::validate::validate` and maps findings to `errors`, and
`assert_word_valid_package` panics with the findings' messages. Run the
whole suite once: `cargo test --all-features` must stay green (the Ring-1
probes in `tests/m_validity_ring1.rs` prove each check still fires).

- [ ] **Step 4: Run the test and watch it pass**

Run: `cargo test --all-features --test validate_public --test m_validity_ring1`
Expected: all pass.

- [ ] **Step 5: CLI**

Add to `enum Command` in `src/bin/jubarte.rs`:

```rust
    /// Word-validity findings beyond the schema: what makes Word refuse or
    /// repair the file. Exit 0 clean, 2 findings, 1 unreadable.
    Validate {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// One JSON object per finding.
        #[arg(long)]
        json: bool,
        /// Write the repaired package here; remaining findings still exit 2.
        #[arg(long, value_name = "FILE")]
        repair: Option<PathBuf>,
        /// Audit tracked edits: every text change against ORIGINAL must be a
        /// revision by --author.
        #[arg(long, value_name = "FILE", requires = "author")]
        original: Option<PathBuf>,
        #[arg(long, value_name = "NAME")]
        author: Option<String>,
    },
```

Prose output, one line per finding: `{code}\t{part}#{path}\t{message}`,
with a `word_fatal` star. Python `__main__` gets the same `validate`
subcommand.

- [ ] **Step 6: Bindings**

`jubarte-python/src/lib.rs`: `validate_json(docx) -> String` (JSON array),
`repair_json(docx) -> (bytes, String)`, `audit_tracked_json(original,
edited, author) -> String`. `document.py`: `Document.validate() ->
tuple[Finding, ...]`, `Document.repair() -> Repaired`,
`Document.audit_tracked(original, author=)`; `models.py` adds frozen
`Finding` and `Repaired` dataclasses; `_native.pyi` mirrors.
`jubarte-wasm/src/lib.rs`: `validateDocument(docx) -> string`,
`repairDocument(docx) -> Uint8Array` with the findings in a JSON side
channel like `EditOutput.json`.

Python test:

```python
# jubarte-python/tests/test_validate.py
import jubarte_redlines as jubarte
from docx_fixture import docx

DELETED = '<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>'


def test_validate_reports_and_repair_fixes_text_inside_a_deletion() -> None:
    doc = jubarte.Document.from_bytes(docx(DELETED))
    codes = [f.code for f in doc.validate()]
    assert codes == ["TEXT_INSIDE_DELETION"]
    repaired = doc.repair()
    assert [f.code for f in repaired.repaired] == ["TEXT_INSIDE_DELETION"]
    assert repaired.document.validate() == ()
```

- [ ] **Step 7: Capabilities and the stale manifest**

In `src/capabilities.rs`: add `validate: bool` and `repair: bool` to
`Operations` (`#[serde(default)]`), and set `limits.stories` to
`["body", "header", "footer", "footnotes", "endnotes"]` with the doc
comment "Stories `inspect` and `edit` address; text boxes are reported in
`summary` but not editable." Update the two assertions in
`tests/agent_contracts.rs` (`manifest.limits.stories`) and the unit test in
`capabilities.rs`.

- [ ] **Step 8: Docs**

Skill §3 "Verify": replace the accept-equals-clean paragraph's last sentence
with: "`jubarte validate review/redline.docx --original contract.docx
--author Claude` is the every-edit-is-tracked check and the Word-validity
check in one; it replaces `validate.py --original --author`. `--repair out.docx`
fixes what it can and lists what it cannot." README: a "Validate" row in the
command table. CHANGELOG Unreleased / Added.

- [ ] **Step 9: Commit**

```bash
git add src/validate.rs src/lib.rs src/debug.rs src/capabilities.rs src/bin/jubarte.rs tests/common/validity.rs tests/validate_public.rs tests/agent_contracts.rs jubarte-python jubarte-wasm/src/lib.rs skills/jubarte-documents/SKILL.md README.md CHANGELOG.md
git commit -m "feat(validate): public Word-validity findings, repair, and the tracked-edit audit"
```

**Evidence this task hands a provider:** three fixtures that pass
OpenXmlValidator and fail in Word (`w:delInstrText` outside `w:del`,
crossed fields from 0.9.3, unbound `mc:Choice Requires` from PR #270), each
with `validate.py` output (clean) beside `jubarte validate` output (finding,
`word_fatal: true`). Stored under `docs/adoption/validate-vs-xsd/`.

---

### Task 3: S2, comment threads (TODO §6)

**Files:**
- Create: `src/comments.rs` (read and write the comment part family)
- Modify: `src/edit.rs:129-270` (new `OperationKind` variants), `:783` (`check_operation_keys`), `:2584` (`write_comments_part` writes `w14:paraId` and the extended parts)
- Modify: `src/capabilities.rs` (`operations.comment_threads`, `edit_operations`)
- Modify: `src/bin/jubarte.rs` (`Command::Comments`)
- Modify: `jubarte-python/src/lib.rs`, `python/jubarte_redlines/{document.py,models.py,_native.pyi,__main__.py}`
- Modify: `jubarte-wasm/src/lib.rs` (`listComments`)
- Test: `tests/edit_comment_threads.rs`, `jubarte-python/tests/test_comments.py`
- Docs: skill §2, `TODO.md` §6 (mark shipped items), CHANGELOG

Facts: `write_comments_part` (`src/edit.rs:2584-2655`) writes only
`comments.xml` entries (`w:comment` with `w:id`, author, date, initials, an
`w:annotationRef` run and the text runs). It writes no `w14:paraId` and no
`commentsExtended.xml`, so replies and resolution have nowhere to live.
Word threads replies through `w15:commentEx w15:paraId="..."
w15:paraIdParent="..." w15:done="0|1"` in `commentsExtended.xml`, keyed by
the `w14:paraId` of the comment's **last** paragraph in `comments.xml`;
`commentsIds.xml` maps that paraId to a `w16cid:durableId`;
`commentsExtensible.xml` carries `w16cex:dateUtc`; `people.xml` lists
authors. Ring 1 already validates this graph (`check_comment_graph`,
`check_comments_extended`, `check_comments_ids`,
`check_comments_extensible`, `check_hex_id` with the `word_para_bound`
rule: a paraId is 8 hex digits below `0x80000000`). The comparer carries
all four parts byte-identical when B's comment set covers A's
(`src/comparer/comments.rs` rule 1), so threads written into the clean copy
survive into the redline.

Wire types:

```rust
/// One comment with its thread position and where it sits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentRecord {
    /// `w:id` of the comment.
    pub id: u32,
    pub author: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initials: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Comment text, paragraphs joined by `\n`.
    pub text: String,
    /// `w:id` of the comment this one replies to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<u32>,
    /// `w15:done`.
    pub done: bool,
    /// Story and paragraph id of the range start (`body:p:12`), when anchored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<String>,
    /// The anchored text, exact.
    pub anchor_text: String,
    /// Up to 80 chars before and after the anchor in its paragraph.
    pub before: String,
    pub after: String,
}

pub fn list_comments(docx: &[u8]) -> Result<Vec<CommentRecord>, CommentError>;
```

New plan operations (wire tag `kind`):

| kind | fields | effect |
|---|---|---|
| `reply_comment` | `comment_id: u32`, `text` | new `w:comment` by the plan's author, `w15:paraIdParent` = the parent's paraId, anchored on the parent's range |
| `resolve_comment` | `comment_id`, `done: bool` (default true) | `w15:done` on the comment and its replies |
| `edit_comment` | `comment_id`, `text` | replaces the comment's paragraphs, keeps id, author, date, thread |
| `delete_comment` | `comment_id` | removes the comment, its replies, its range markers and reference, and its rows in the three extended parts; `people.xml` pruned by `prune_people` |
| `comment` | existing, plus optional `through: Selector` | one range from the start of `paragraph` to the end of `through` (the "several paragraphs" item of TODO §6) |

Refusals: `UNKNOWN_COMMENT` (no such id), `COMMENT_NOT_IN_BODY` (a
`comment` whose paragraph is a header/footer story; Word cannot anchor
there, as the skill already says). The list command's `--author NAME` and
`--latest` (one record per thread: the newest) filters are Rust-side so the
three surfaces agree.

- [ ] **Step 1: Write the failing test**

```rust
// tests/edit_comment_threads.rs
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comment threads: add, list, reply, resolve, edit, delete; every output
//! is Word-valid and survives Accept All.

mod common;

use common::docx::{docx, para};
use common::validity::assert_word_valid_package;
use jubarte::comments::list_comments;
use jubarte::document_comparer::accept_revisions;
use jubarte::edit::{EditPlan, apply_plan};

fn plan(json: &str) -> EditPlan {
    EditPlan::from_json(json).unwrap()
}

#[test]
fn reply_and_resolve_round_trip() {
    let source = docx(&para("The cap is 10."));
    let first = apply_plan(&source, &plan(r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"body:p:0","find":"cap","text":"Too low"}]}"#)).unwrap();
    assert_word_valid_package(&first.clean);
    let listed = list_comments(&first.clean).unwrap();
    assert_eq!(listed.len(), 1);
    let root = &listed[0];
    assert_eq!((root.author.as_str(), root.text.as_str(), root.done, root.parent), ("Ann", "Too low", false, None));
    assert_eq!(root.anchor_text, "cap");
    assert_eq!(root.paragraph.as_deref(), Some("body:p:0"));

    let second = apply_plan(&first.clean, &plan(&format!(r#"{{"schema_version":1,"author":"Bob","operations":[
        {{"kind":"reply_comment","comment_id":{id},"text":"Agreed"}},
        {{"kind":"resolve_comment","comment_id":{id}}}]}}"#, id = root.id))).unwrap();
    assert_word_valid_package(&second.clean);
    assert_word_valid_package(&second.redline);
    let thread = list_comments(&second.clean).unwrap();
    assert_eq!(thread.len(), 2);
    assert_eq!(thread[1].parent, Some(root.id));
    assert_eq!(thread[1].author, "Bob");
    assert!(thread[0].done && thread[1].done);
    // The thread rides the redline and Accept All.
    assert_eq!(list_comments(&second.redline).unwrap().len(), 2);
    assert_eq!(list_comments(&accept_revisions(&second.redline).unwrap()).unwrap().len(), 2);
}

#[test]
fn edit_and_delete_leave_a_valid_package() {
    let source = docx(&(para("One.") + &para("Two.")));
    let with = apply_plan(&source, &plan(r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"body:p:0","through":"body:p:1","text":"Both paragraphs"}]}"#)).unwrap();
    let id = list_comments(&with.clean).unwrap()[0].id;
    let edited = apply_plan(&with.clean, &plan(&format!(r#"{{"schema_version":1,"author":"Ann","operations":[
        {{"kind":"edit_comment","comment_id":{id},"text":"Both, reworded"}}]}}"#))).unwrap();
    assert_eq!(list_comments(&edited.clean).unwrap()[0].text, "Both, reworded");
    let deleted = apply_plan(&edited.clean, &plan(&format!(r#"{{"schema_version":1,"author":"Ann","operations":[
        {{"kind":"delete_comment","comment_id":{id}}}]}}"#))).unwrap();
    assert_word_valid_package(&deleted.clean);
    assert!(list_comments(&deleted.clean).unwrap().is_empty());
    let xml = common::docx::part_string(&deleted.clean, "word/document.xml").unwrap();
    assert!(!xml.contains("commentRangeStart") && !xml.contains("commentReference"));
}

#[test]
fn unknown_ids_and_header_anchors_are_refused() {
    let source = docx(&para("x"));
    let e = apply_plan(&source, &plan(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"reply_comment","comment_id":7,"text":"?"}]}"#)).unwrap_err();
    assert_eq!(e.code, "UNKNOWN_COMMENT");
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test --all-features --test edit_comment_threads`
Expected: `could not find comments in jubarte`, then after the module
stub, `INVALID_PLAN: unknown kind "reply_comment"`.

- [ ] **Step 3: Implement**

1. `src/comments.rs`: `list_comments` opens through
   `crate::inspect::Opened::open` (admitted), finds the comments part via
   `Opened::related("comments")`, parses the four parts with
   `quick_xml`/`xmllinq`, joins `commentsExtended` rows to comments by the
   last paragraph's `w14:paraId`, and locates each anchor by walking every
   story for `w:commentRangeStart w:id`, projecting the paragraph with
   `inspect::project_paragraph` to get `anchor_text`, `before`, `after`.
   Internal writers in the same module: `ensure_para_ids`, `write_extended`,
   `write_ids`, `write_extensible`, `ensure_person`, `remove_comment`.
   paraId generation: deterministic from the comment id
   (`0x1000_0000 + id * 0x11`, masked below `0x8000_0000`), checked unique
   against the existing ids in the part.
2. `src/edit.rs`: four `OperationKind` variants; `check_operation_keys`
   rows `"reply_comment" => &["comment_id","text"]`, `"resolve_comment" =>
   &["comment_id","done"]`, `"edit_comment" => &["comment_id","text"]`,
   `"delete_comment" => &["comment_id"]`, and `"comment"` gains `"through"`.
   `Transaction::resolve_one` validates ids against `list_comments(&self.base)`;
   `Transaction::apply` performs the part edits through `comments.rs`;
   `write_comments_part` now always stamps `w14:paraId` on every comment
   paragraph and writes `commentsExtended.xml`, `commentsIds.xml`,
   `commentsExtensible.xml` for every comment in the part (new and old), so
   the family is whole whenever jubarte touched it. A plain `comment` on a
   document with no comments part keeps producing all four parts; Ring 1's
   `check_comment_family_packaging` is the oracle.
3. `capabilities.rs`: `edit_operations` grows by four, `operations.comment_threads: true`; update `tests/agent_contracts.rs` (the list is asserted in order).
4. CLI `jubarte comments FILE [--json] [--author NAME] [--latest]`.
5. Python: `Document.comments() -> tuple[Comment, ...]`,
   `EditPlan.reply_comment(comment_id, text=)`, `.resolve_comment(comment_id,
   done=True)`, `.edit_comment(comment_id, text=)`, `.delete_comment(comment_id)`,
   `.comment(..., through=)`; WASM `listComments(docx) -> string` (JSON lines).

- [ ] **Step 4: Run tests and watch them pass**

Run: `cargo test --all-features --test edit_comment_threads --test edit_plan --test m_validity_ring1 --test agent_contracts`
Expected: pass. Then the full suite once (the comparer carryover tests
`m147_comments_union_carryover` must not change).

- [ ] **Step 5: Python test**

```python
# jubarte-python/tests/test_comments.py
import jubarte_redlines as jubarte
from docx_fixture import docx, para


def test_reply_and_resolve_round_trip() -> None:
    doc = jubarte.Document.from_bytes(docx(para("The cap is 10.")))
    first = doc.edit(jubarte.EditPlan(author="Ann").comment("body:p:0", find="cap", text="Too low"))
    (root,) = first.clean.comments()
    assert (root.text, root.done, root.parent) == ("Too low", False, None)
    second = first.clean.edit(
        jubarte.EditPlan(author="Bob").reply_comment(root.id, text="Agreed").resolve_comment(root.id)
    )
    thread = second.clean.comments()
    assert thread[1].parent == root.id and thread[0].done and thread[1].done
```

- [ ] **Step 6: Docs and TODO**

Skill §2: add the four kinds and `through` to the operation list, and a
line in §1: "`jubarte comments FILE --json` lists every comment with its
thread (`parent`, `done`) and the anchored text with its surroundings."
`TODO.md` §6: tick Add (range), Delete, Modify, Reply, List, See
surroundings, Resolve; leave "Remember which party cares" open (it is a
client-side memory, not an engine feature). CHANGELOG.

- [ ] **Step 7: Commit**

```bash
git add src/comments.rs src/edit.rs src/lib.rs src/capabilities.rs src/bin/jubarte.rs tests/edit_comment_threads.rs tests/agent_contracts.rs jubarte-python jubarte-wasm/src/lib.rs skills/jubarte-documents/SKILL.md TODO.md CHANGELOG.md
git commit -m "feat(comments): threads in edit plans and the APIs: list, reply, resolve, edit, delete, multi-paragraph ranges"
```

**Evidence this task hands a provider:** `examples/agents/comment-thread/`:
one plan that adds, replies and resolves, with `report.jsonl`, the
`jubarte comments --json` listing, and a Word screenshot of the threaded
balloon, beside the seven manual steps Anthropic's `comment.py` section
describes.

---

### Task 4: S1, `occurrence` for repeated anchors

**Files:**
- Modify: `src/edit.rs:131-180` (`Replace`, `Insert`, `Delete`, `Comment` gain `occurrence: Option<usize>`), `:783` (`check_operation_keys`), `:1938` (`find_range`), `:2007` (`check_insert_position` callers)
- Modify: `jubarte-python/python/jubarte_redlines/models.py:459-520` (`occurrence=` keyword on `replace`, `insert`, `delete`, `comment`)
- Test: `tests/edit_occurrence.rs`, add one case to `jubarte-python/tests/test_edit_models.py`
- Docs: skill §2 gotchas, CHANGELOG

- [ ] **Step 1: Write the failing test**

```rust
// tests/edit_occurrence.rs
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A repeated anchor is editable by occurrence; the refusal says how many.

mod common;

use common::docx::{docx, para};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;

fn source() -> Vec<u8> {
    docx(&para("fee fee fee"))
}

#[test]
fn the_second_occurrence_is_replaced() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"replace","paragraph":"body:p:0","find":"fee","occurrence":2,"replacement":"cost"}]}"#).unwrap();
    let out = apply_plan(&source(), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "fee cost fee");
    assert_eq!(out.report.operations[0].matches, 3);
}

#[test]
fn an_out_of_range_occurrence_names_the_range() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"delete","paragraph":"body:p:0","find":"fee","occurrence":4}]}"#).unwrap();
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
    assert!(e.message.contains("occurs 3 times") && e.message.contains("occurrence 1..=3"), "{}", e.message);
}

#[test]
fn without_occurrence_the_refusal_now_says_how_to_fix_it() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"insert","paragraph":"body:p:0","after":"fee","text":"!"}]}"#).unwrap();
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
    assert!(e.message.contains("set \"occurrence\" to 1..=3"), "{}", e.message);
}

#[test]
fn occurrence_zero_is_an_invalid_edit() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"comment","paragraph":"body:p:0","find":"fee","occurrence":0,"text":"?"}]}"#).unwrap();
    assert_eq!(apply_plan(&source(), &plan).unwrap_err().code, "INVALID_EDIT");
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test --all-features --test edit_occurrence`
Expected: four failures with `INVALID_PLAN ... unknown field "occurrence"`.

- [ ] **Step 3: Implement**

In each of the four variants add:

```rust
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Which occurrence of the anchor to use when it repeats (1-based).
        /// Without it the anchor must occur exactly once.
        occurrence: Option<usize>,
```

In `check_operation_keys`, append `"occurrence"` to the `replace`, `insert`,
`delete` and `comment` rows. Change `find_range` to take
`occurrence: Option<usize>` and replace its `match hits.as_slice()` with:

```rust
        let start = match (hits.as_slice(), occurrence) {
            ([], _) => {
                return Err(("ANCHOR_NOT_FOUND".into(), format!("{find:?} does not occur in the paragraph")));
            }
            (_, Some(0)) => {
                return Err(("INVALID_EDIT".into(), "occurrence is 1-based".into()));
            }
            ([one], None) => *one,
            (many, None) => {
                return Err((
                    "AMBIGUOUS_ANCHOR".into(),
                    format!("{find:?} occurs {} times in the paragraph; set \"occurrence\" to 1..={}", many.len(), many.len()),
                ));
            }
            (many, Some(n)) if n <= many.len() => many[n - 1],
            (many, Some(n)) => {
                return Err((
                    "AMBIGUOUS_ANCHOR".into(),
                    format!("{find:?} occurs {} times in the paragraph; occurrence {n} is outside occurrence 1..={}", many.len(), many.len()),
                ));
            }
        };
```

Thread `occurrence` through the `Replace`, `Delete`, `Comment` arms of
`resolve_one` and through the `after`/`before` lookups of `Insert` (they
call `find_range` too). `EditOutcome.matches` keeps the total count.

Python `models.py`: add `occurrence: int | None = None` to `replace`,
`insert`, `delete`, `comment`; `_with_optional(op, occurrence=occurrence)`
only when not `None`; a `ValueError` for `occurrence < 1`.

- [ ] **Step 4: Run tests and watch them pass**

Run: `cargo test --all-features --test edit_occurrence --test edit_plan --test agent_contracts`
Expected: pass.

- [ ] **Step 5: Docs**

Skill §2 gotcha becomes: "`find` must occur exactly once in that paragraph
unless you give `occurrence` (1-based); the refusal says how many times it
occurs. Overlapping occurrences count." CHANGELOG.

- [ ] **Step 6: Commit**

```bash
git add src/edit.rs tests/edit_occurrence.rs jubarte-python/python/jubarte_redlines/models.py jubarte-python/tests/test_edit_models.py skills/jubarte-documents/SKILL.md CHANGELOG.md
git commit -m "feat(edit): occurrence selects a repeated anchor; the refusal names the range"
```

---

### Task 5: S9, `substituted` in the font report and `--fail-on-substitution`

**Files:**
- Modify: `src/convert/font.rs:139-180` (`FontReportEntry::substituted()`, `font_report_json` emits it)
- Modify: `src/bin/jubarte.rs:195-246` (`Convert` gains `--fail-on-substitution`), `run_convert` (exit 4)
- Modify: `jubarte-python/python/jubarte_redlines/models.py:738` (`FontResolution.substituted`), `RenderReport.substitutions`
- Test: `tests/convert_font_report.rs`, CLI unit test beside `convert_subcommand_parses_font_report` (`src/bin/jubarte.rs:2465`)
- Docs: skill §3, CHANGELOG

Definition: `substituted` is true when `step` is `WordSubstitution`,
`OpenFallback`, `Generic` or `Unknown`; false for `Embedded`, `Explicit`,
`AltName`, `Theme`. `synthetic` (style faked) stays separate.

- [ ] **Step 1: Failing test**

```rust
// tests/convert_font_report.rs
mod common;
use common::docx::docx;
use jubarte::convert::{PdfOptions, docx_render_report};

#[test]
fn an_unknown_family_is_reported_as_substituted() {
    let body = r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="NoSuchFont" w:hAnsi="NoSuchFont"/></w:rPr><w:t>hello</w:t></w:r></w:p>"#;
    let report = docx_render_report(&docx(body), PdfOptions::default()).unwrap();
    let entry = report.fonts.iter().find(|f| f.requested == "NoSuchFont").expect("requested family");
    assert!(entry.substituted());
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
    assert_eq!(json["fonts"][0]["substituted"], true);
}
```

- [ ] **Step 2: Run, expect `no method named substituted`.**

- [ ] **Step 3: Implement** the method and the JSON field; in the CLI,
after `render`, when the flag is set and any entry is substituted, print
`substituted: NoSuchFont -> Carlito (open_fallback)` lines to stderr and
return exit code 4 (document it in `--help`: "0 ok, 1 error, 4 a requested
font was substituted"). Python: `FontResolution.substituted: bool` and
`RenderReport.substitutions -> tuple[FontResolution, ...]`.

- [ ] **Step 4: Run and pass; Step 5: docs** (skill §3: "`--report` lists
every font and whether it was substituted; `--fail-on-substitution` turns
that into exit 4 for CI"); **Step 6: Commit**
`feat(convert): substituted flag in the font report; --fail-on-substitution`.

---

### Task 6: S7, ship Markdown creation; `--page`; creation from Python and WASM

**Files:**
- Modify: `src/markdown/mod.rs:99-130` (`DocxOptions.page: PageSize`), `src/markdown/package.rs:24-30` (default section by page size)
- Modify: `src/bin/jubarte.rs:675-715` (`MarkdownArgs` gains `--page letter|a4`)
- Modify: `src/bin/jubarte.rs:332` (`Text` gains `--track-changes all|accept|reject` and delegates to `markdown::docx_to_markdown` when given; without it the id projection stays)
- Modify: `jubarte-python/src/lib.rs` (`markdown_to_docx`), `document.py` (`jubarte_redlines.from_markdown(...) -> Document`), `_native.pyi`, `__main__.py` (`convert` accepts `.md` input)
- Modify: `jubarte-wasm/src/lib.rs` (`markdownToDocx(text, optionsJson)`)
- Test: `tests/markdown_page_size.rs`, `jubarte-python/tests/test_markdown.py`, `jubarte-wasm/npm-smoke.mjs`
- Release: `scripts/release.sh 0.11.0 ...` with its six notes; `docs/api/` snapshots regenerate

- [ ] **Step 1: Failing test**

```rust
// tests/markdown_page_size.rs
mod common;
use jubarte::markdown::{DocxOptions, PageSize, markdown_to_docx};

#[test]
fn a4_writes_the_a4_section_and_letter_stays_default() {
    let a4 = markdown_to_docx("# T\n\ntext\n", &DocxOptions { page: PageSize::A4, ..DocxOptions::default() }).unwrap().docx;
    let xml = common::docx::part_string(&a4, "word/document.xml").unwrap();
    assert!(xml.contains(r#"<w:pgSz w:w="11906" w:h="16838"/>"#), "{xml}");
    let letter = markdown_to_docx("# T\n\ntext\n", &DocxOptions::default()).unwrap().docx;
    assert!(common::docx::part_string(&letter, "word/document.xml").unwrap().contains(r#"<w:pgSz w:w="12240" w:h="15840"/>"#));
}
```

- [ ] **Step 2: Run, expect `no field page`.**

- [ ] **Step 3: Implement**: `pub enum PageSize { Letter, A4 }` (default
Letter, `#[serde(rename_all = "lowercase")]`), `package.rs` picks the
`DEFAULT_SECTION` by it (A4: `w:w="11906" w:h="16838"`, margins 1134 dxa
as Word's A4 default is 2 cm... keep one-inch 1440 to match the Letter
default and say so in the doc comment). Ignored when `reference` is given
(its section wins), with a warning in `WrittenDocx.warnings`. CLI
`--page`. `jubarte text --track-changes` maps to
`markdown::docx_to_markdown` with `MarkdownOptions{track_changes}`; the
help text says the output then has no `[body:p:N]` ids (CriticMarkup
instead). Python `_native.markdown_to_docx(text, reference=None,
page="letter", author="Redline", date=None, critic=True,
track_changes="all") -> bytes` and `jubarte_redlines.from_markdown(text,
**same) -> Document`; WASM `markdownToDocx(text, optionsJson) -> Uint8Array`.

- [ ] **Step 4: Tests pass** (Rust, pytest, `node jubarte-wasm/npm-smoke.mjs`).

- [ ] **Step 5: Release 0.11.0.** Follow `VERSIONING.md` step 1 gates
(including Ring 3 on macOS), then `scripts/release.sh 0.11.0` with the six
notes; `--changelog-summary` names: Markdown creation and diff, `rewrite`,
per-change comments, uvx/npx CLIs, plus whatever of Tasks 1 to 5 has landed.
Regenerate `docs/api/` snapshots (release step 5). This is the step that
makes the mapping document's "Markdown creation exists only in unmerged
PRs" false on PyPI, npm and crates.io, not only on `main`.

- [ ] **Step 6: Commit** (`feat(markdown): --page letter|a4; text
--track-changes; Markdown creation from Python and WASM`), then the release
commit by the script.

**Evidence this task hands a provider:** the Anthropic skill's creation
section lists eleven docx-js footguns; `docs/adoption/creation.md` shows the
same letter written as 30 lines of Markdown with CriticMarkup, rendered by
`jubarte convert draft.md --page letter --png`, with no footgun applicable.

---

### Task 7: S6, field and TOC refresh from jubarte's own layout

**Files:**
- Create: `src/fields.rs`
- Modify: `src/convert/mod.rs` (expose `pub(crate) fn layout_facts(docx) -> Result<LayoutFacts, ConvertError>`: `page_count`, `bookmark_pages: BTreeMap<String, u32>`, `paragraph_pages: Vec<u32>` indexed like `inspect::paragraphs`)
- Modify: `src/edit.rs` (`OperationKind::InsertToc`, plan-level `update_fields: bool`), `check_operation_keys`
- Modify: `src/bin/jubarte.rs` (`Command::Fields { Update }`), `src/capabilities.rs` (`operations.fields`)
- Bindings: `Document.update_fields()`, `EditPlan.insert_toc(...)`, `EditPlan(update_fields=True)`; WASM `updateFields`
- Test: `tests/fields_update.rs`
- Docs: skill, CHANGELOG, `docs/WORD_DIFFERENCES.md` (page numbers are jubarte's layout, not Word's)

Facts: the layout already resolves `PAGEREF` by bookmark page after
pagination (`bookmark_pages` and `pageref_ops` in the layout state,
`src/convert/mod.rs:19596-19597`, patched at `:28220`) and computes
`page_count` and `NUMPAGES`; `REF` copies bookmark text
(`document_bookmark_texts`, `:9946-9970`); an empty TOC field is detected
(`para_is_empty_toc_field`, `:8207`). None of this is written back into the
package. S6 writes it back.

Scope: refresh cached results of `PAGEREF`, `REF`, `NUMPAGES`, `SEQ`
(count per identifier), and TOC entries' `PAGEREF`s; generate a TOC
(`insert_toc`) from `Heading1..Heading{levels}` paragraphs with
`_TocNNNNNNNN` bookmarks, hyperlinks to them, a right tab with dot leader
and a `PAGEREF \h` field each, styled `TOC1..TOCn` (added to `styles.xml`
from the built-in definitions in `src/builtin_styles.rs` when absent).
`PAGE` is not materialized (it differs per page). Field codes stay, so Word
can still refresh; `w:updateFields` is **not** set (Task 10 can set it).

- [ ] **Step 1: Failing test**

```rust
// tests/fields_update.rs
mod common;
use common::docx::{docx, part_string};
use common::validity::assert_word_valid_package;
use jubarte::fields::update_fields;
use jubarte::inspect::paragraphs;

const HEADING_STYLES: &str = r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>A</w:t></w:r></w:p>"#;

fn two_headings_and_an_empty_toc() -> Vec<u8> {
    // A TOC with no cached result, Heading1 "A", a page break, Heading1 "B".
    let body = String::new()
        + r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TOC \o "1-3" \h </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#
        + HEADING_STYLES
        + r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#
        + r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>B</w:t></w:r></w:p>"#
        + r#"<w:p><w:r><w:t xml:space="preserve">Pages: </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> NUMPAGES </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
    docx(&body)
}

#[test]
fn toc_entries_and_numpages_come_from_the_layout() {
    let updated = update_fields(&two_headings_and_an_empty_toc()).unwrap();
    assert_word_valid_package(&updated.docx);
    let texts: Vec<String> = paragraphs(&updated.docx).unwrap().into_iter().map(|p| p.text).collect();
    // The TOC paragraph grew into entries: "A\t1" and "B\t2".
    assert!(texts.iter().any(|t| t == "A\t1"), "{texts:?}");
    assert!(texts.iter().any(|t| t == "B\t2"), "{texts:?}");
    assert!(texts.last().unwrap().ends_with("Pages: 2"), "{texts:?}");
    assert_eq!(updated.fields.iter().filter(|f| f.kind == "PAGEREF").count(), 2);
    assert_eq!(updated.fields.iter().filter(|f| f.kind == "NUMPAGES").count(), 1);
    // Field codes are intact so Word can refresh.
    let xml = part_string(&updated.docx, "word/document.xml").unwrap();
    assert!(xml.contains(r#" TOC \o "1-3" \h "#) && xml.contains(" NUMPAGES "));
}
```

(`Heading1` must exist in `styles.xml` for the layout to page-break and
for the TOC walker; `docx()` has no styles part, so the implementation must
fall back to the built-in style definitions, as `convert` already does for
originals without one. If the fixture needs an explicit styles part, use
`docx_with` with a `word/styles.xml` `Part` and note it in the test.)

- [ ] **Step 2: Run, expect `could not find fields in jubarte`.**

- [ ] **Step 3: Implement**

`src/fields.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FieldUpdate { pub kind: String, pub code: String, pub paragraph: String, pub old: String, pub new: String }
pub struct Updated { pub docx: Vec<u8>, pub fields: Vec<FieldUpdate>, pub page_count: usize }
pub fn update_fields(docx: &[u8]) -> Result<Updated, FieldError>;
```

1. Open through `inspect::Opened` (admitted), collect complex fields per
   story with their `begin`/`separate`/`end` runs (reuse
   `debug::Check::Fields` walker, make it `pub(crate)`), parse the code
   (`PAGEREF name`, `REF name [\h \r \n \w \p]`, `NUMPAGES`, `SEQ ident`,
   `TOC [\o "1-3"] [\h] [\u] [\t ...]`).
2. For a TOC: build the entries from heading paragraphs (style id
   `Heading{n}` or style name `heading n`, levels from `\o`), adding a
   `w:bookmarkStart/End w:name="_TocNNNNNNNN"` around each heading's runs
   when absent (ids from the next free bookmark id, see `validate`'s
   `collect_ids`), and write entry paragraphs between `separate` and `end`:
   `TOC{n}` style, `w:hyperlink w:anchor="_Toc..."`, the heading text, a
   `w:tab` and a `PAGEREF _Toc... \h` field with a placeholder result.
3. Call `convert::layout_facts(&package_bytes)` **once** on the package as
   it now stands, then write each `PAGEREF`'s page, `NUMPAGES`'s
   `page_count`, `REF`'s bookmark text, `SEQ`'s running count into the
   result runs (one `w:r` with `w:t`, keeping the first result run's `w:rPr`).
4. Serialize; `validate()` must be clean (the test asserts it).

`convert::layout_facts` is the one engine change: after `layout()` returns,
map every bookmark name to its page and every body paragraph index to the
page of its first line. Add it as a `pub(crate)` function beside
`docx_render_report` and keep `RenderReport` unchanged.

`insert_toc` edit op: `{"kind":"insert_toc","paragraph":Selector,"position":"before|after","levels":3,"title":"Contents"}` writes the field (and optional title paragraph styled `TOCHeading`) into the clean copy, then `update_fields` runs on the clean copy when the plan sets `"update_fields": true`; the redline comes from compare as usual (0.9.3: "a same-code field stays whole", so the refreshed caches do not produce spurious revisions).

- [ ] **Step 4: Tests pass; also `cargo test --all-features --test convert_docx_to_pdf`** (the layout change must not alter rendering).

- [ ] **Step 5: CLI, bindings, docs**: `jubarte fields update FILE -o OUT [--json]`; Python `Document.update_fields() -> Updated`; WASM `updateFields`. Skill §2: "`insert_toc` and `\"update_fields\": true`; page numbers are jubarte's layout, which matches Word on most documents but is not Word (`docs/WORD_DIFFERENCES.md`)". `docs/WORD_DIFFERENCES.md` gets the caveat. CHANGELOG.

- [ ] **Step 6: Commit** `feat(fields): refresh PAGEREF, REF, NUMPAGES, SEQ and TOC caches from jubarte's layout; insert_toc`.

**Evidence this task hands a provider:** OpenAI's container skill
materializes `SEQ`/`REF` "for deterministic headless rendering" through
LibreOffice; Z.ai ships `add_toc_placeholders.py` because its generator
cannot fill a TOC. `docs/adoption/fields.md`: one document, `jubarte fields
update`, the resulting `pages.json`, and Word's own Update Field result on
the same file for comparison (page numbers equal or the difference named).

---

### Task 8: S8, structural edit operations

Split into commits 8a to 8g. Each operation is built in the clean copy and
redlined by compare; each test asserts the clean text, the redline's
revision kinds through `jubarte::changes::list_changes`, and
`assert_word_valid_package` on both outputs. Reuse: the Markdown writer's
XML builders (`src/markdown/xml.rs`: tables with `w:tblHeader`, DXA
`w:tcW`, `w:gridCol`; `drawing_xml`; run properties),
`markdown::package::numbering` (fresh `abstractNumId`/`numId`,
`w:abstractNum` before every `w:num`) and `footnotes` (part creation with
separators), and `comparer::parts::carry_relationship` for images.

**8a `insert_table`**

Wire: `{"kind":"insert_table","paragraph":Selector,"position":"after","rows":[["Item","Qty"],["Bolt","40"]],"header_row":true,"widths_dxa":[6000,3360],"style":"TableGrid"}`.
Writes `w:tbl` with `w:tblW w:type="dxa"` equal to the sum of widths,
`w:tblGrid`, `w:tblHeader` on the first row when asked, `w:tcW` on every
cell (the "dual widths" docx-js footgun, solved once here), one paragraph
per cell copying the anchor's `w:pPr` style, and `TableGrid` added to
`styles.xml` from `builtin_styles` when absent. Refuses `widths_dxa` whose
length differs from the column count (`INVALID_EDIT`) and ragged rows.

Test (`tests/edit_insert_table.rs`):

```rust
#[test]
fn an_inserted_table_is_tracked_and_valid() {
    let source = docx(&(para("Intro.") + &para("Outro.")));
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"insert_table","paragraph":"body:p:0","position":"after","header_row":true,
         "rows":[["Item","Qty"],["Bolt","40"]],"widths_dxa":[6000,3360]}]}"#).unwrap();
    let out = apply_plan(&source, &plan).unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let xml = part_string(&out.clean, "word/document.xml").unwrap();
    assert!(xml.contains("<w:tblHeader/>") && xml.contains(r#"<w:tcW w:w="6000" w:type="dxa"/>"#));
    let texts: Vec<_> = paragraphs(&out.clean).unwrap().into_iter().map(|p| (p.text, p.in_table)).collect();
    assert_eq!(texts[1], ("Item".to_string(), true));
    let kinds: Vec<_> = list_changes(&out.redline).unwrap().into_iter().map(|c| (c.kind, c.target)).collect();
    assert!(kinds.iter().any(|(k, t)| *k == ChangeKind::Insertion && *t == "table_row"), "{kinds:?}");
}
```

**8b `list`**

Wire: `{"kind":"list","paragraphs":[Selector, ...],"kind_of_list":"bullet|decimal|lower_letter","level":0,"restart":true}` (the wire field is `list` with a `kind_of_list` field because `kind` is the tag). Creates one `w:abstractNum` (with `w:nsid`, `w:multiLevelType w:val="hybridMultilevel"`, `w:tmpl`, nine `w:lvl`s following `markdown::xml::numbering`) and one `w:num`, sets `w:numPr` (`w:ilvl`, `w:numId`) on each paragraph and `ListParagraph` style when the paragraph has no style. The redline shows `w:pPrChange` per paragraph (compare's `detect_format_changes`). Test: `paragraphs(clean)[i].numbered == true`, `jubarte debug --check numbering` lists the level, `list_changes` has a `Formatting` change per paragraph, validity clean.

**8c `format_run`**

Wire: `{"kind":"format_run","paragraph":Selector,"find":"text","occurrence":N,"format":{"bold":true,"italic":false,"underline":true,"highlight":"yellow","font":"Arial","size_pt":11,"color":"FF0000","strike":false,"caps":false}}`.
Extends `RunFormat` with `font`, `size_pt`, `color`, `strike`, `caps`
(`RPR_ORDER` at `src/edit.rs:3341` already orders `w:rPr` children; add
`rFonts`, `color`, `sz`/`szCs`, `strike`, `caps` in schema order), reuses
`format_range` (`:2985`). The redline shows `w:rPrChange`. Test: the run
splits around the range, `Span` in `inspect` reports the new direct
formatting, `list_changes` has one `Formatting` change whose `text` is the
range.

**8d `insert_footnote`**

Wire: `{"kind":"insert_footnote","paragraph":Selector,"after":"anchor text","occurrence":N,"text":"Note text"}`. Creates `word/footnotes.xml` with separators when absent (`markdown::package::footnotes` does this), appends `w:footnote w:id="N"` (`FootnoteText` style, `FootnoteReference` run), inserts the `w:footnoteReference` run after the anchor. Test: `summary.footnotes == 1`, story `footnotes:p:0` text equals the note, redline valid, `list_changes` has an insertion whose `text` contains the reference.

**8e `insert_image`**

Wire: `{"kind":"insert_image","paragraph":Selector,"position":"after","image_base64":"...","content_type":"image/png","width_emu":2743200,"alt":"Diagram"}` (bytes inline in the plan: plans are JSON; the Python builder takes `bytes`). Decodes with `image` to get the pixel size when `width_emu` is omitted (max 6.5 in, the Markdown writer's cap), adds `word/media/imageN.png` with a content-type `Default` for the extension, a relationship from the main part, and `w:drawing` via `markdown::xml::drawing_xml` with `wp:docPr descr=alt` (the a11y audit in plan 3 checks `descr`). Refuses content types other than png/jpeg/gif/bmp/tiff (`UNSUPPORTED_IMAGE`). Test: `summary.images == 1`, relationship resolves, validity clean, redline insertion.

**8f `page_setup`**

Wire: `{"kind":"page_setup","section":"last|all","page":"letter|a4|{"width_dxa":..,"height_dxa":..}","orientation":"portrait|landscape","margins_dxa":{"top":1440,"right":1440,"bottom":1440,"left":1440,"header":720,"footer":720}}`. Rewrites `w:pgSz` (swapping w/h for landscape, `w:orient="landscape"`) and `w:pgMar` in `CT_SectPr` order (`sectPr` children: headerReference*, footerReference*, footnotePr, endnotePr, type, pgSz, pgMar, paperSrc, pgBorders, lnNumType, pgNumType, cols, ...). The redline shows `w:sectPrChange`. Test: the final `w:sectPr` carries the new values and `convert --report` page count changes accordingly for a long fixture.

**8g `inspect --tables`**

`inspect_json` gains `tables: [{index, rows: [[{paragraph_ids: [...], text}]], header_rows: n, widths_dxa: [...]}]` for the body, and the `Snapshot` Python model gains `tables`. This is the read side of OpenAI's `docx_table_to_csv.py`. Test: a 2x2 table reports its cell texts and paragraph ids that round-trip into a `replace`.

Each sub-task: failing test, run (expect `unknown kind`), implement
(`OperationKind` variant, `check_operation_keys` row, `resolve_one` arm,
`apply` arm, `capabilities.edit_operations`, `tests/agent_contracts.rs`
list, Python builder, skill §2 line), run, commit
`feat(edit): <op>`.

**Evidence this task hands a provider:** `docs/adoption/python-docx.md`:
the ten most common python-docx snippets in OpenAI's `doc` skill (add
table, add picture, set style, add heading, set margins, add footnote,
bold a run, add list, set landscape, read table) beside the equivalent plan
operation, each with "tracked: yes/no" (python-docx: no for all ten).

---

### Task 9: S4, privacy scrub and redaction

**Files:**
- Create: `src/scrub.rs`
- Modify: `src/edit.rs` (`OperationKind::Redact`), `src/bin/jubarte.rs` (`Command::Scrub`), `src/capabilities.rs`
- Bindings: `Document.scrub(...)`, `EditPlan.redact(...)`, WASM `scrubDocument`
- Test: `tests/scrub_and_redact.rs`, `jubarte-python/tests/test_scrub.py`

Design:

```rust
pub struct ScrubOptions {
    /// Replace every revision and comment author with this name (`people.xml` included).
    pub author_alias: Option<String>,
    /// Drop `w:rsid*` attributes, `w:rsid` elements and `w:rsids` in settings.
    pub rsids: bool,
    /// Blank `dc:creator`, `cp:lastModifiedBy`, `cp:revision`, `dcterms:created/modified`
    /// in `docProps/core.xml`; drop `docProps/custom.xml`; blank `Company`/`Manager` in `app.xml`.
    pub docprops: bool,
    /// Remove every comment (ranges, references, the four parts).
    pub comments: bool,
}
pub fn scrub(docx: &[u8], options: &ScrubOptions) -> Result<Vec<u8>, ScrubError>;
```

Reuse: `markup_simplifier::remove_rsid_transform` (`src/markup_simplifier.rs:133`)
for rsids; `revision_processor::comments::prune_people` and `remove_part`
for people and comment parts; author attributes are `w:author` on
`w:ins|del|moveFrom|moveTo|rPrChange|pPrChange|sectPrChange|tblPrChange|trPrChange|tcPrChange|numberingChange|cellIns|cellDel|cellMerge|comment`
and `w15:author` on `w15:person`.

`redact` op: `{"kind":"redact","paragraph":Selector,"find":"text","occurrence":N}`.
Applies to the **base** before the transaction's clean/redline split: the
text becomes U+2588 FULL BLOCK characters of the same count (layout keeps
its width), so the redline never carries the text inside a `w:del`. After
the plan runs, the whole output package (every part, including
`word/comments.xml`, headers, footnotes, `docProps/*.xml`) is searched for
the redacted string; any hit refuses the plan with `REDACTION_LEAK` naming
the part. This is the one operation whose output is checked against its
own input text, because leaking the string anywhere defeats the purpose.

- [ ] **Step 1: Failing tests**

```rust
// tests/scrub_and_redact.rs
mod common;
use common::docx::{docx, para};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::compare_documents;
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::scrub::{ScrubOptions, scrub};
use std::io::Read;

fn all_bytes(docx: &[u8]) -> Vec<u8> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut out = Vec::new();
    for i in 0..z.len() {
        z.by_index(i).unwrap().read_to_end(&mut out).unwrap();
    }
    out
}

fn contains(hay: &[u8], needle: &str) -> bool {
    hay.windows(needle.len()).any(|w| w == needle.as_bytes())
}

#[test]
fn scrub_removes_the_author_and_rsids_everywhere() {
    let red = compare_documents(&docx(&para("a")), &docx(&para("b")), "Jane Secret").unwrap();
    assert!(contains(&all_bytes(&red), "Jane Secret"));
    let out = scrub(&red, &ScrubOptions { author_alias: Some("Reviewer".into()), rsids: true, docprops: true, comments: false }).unwrap();
    assert_word_valid_package(&out);
    let bytes = all_bytes(&out);
    assert!(!contains(&bytes, "Jane Secret") && !contains(&bytes, "w:rsidR=") && contains(&bytes, "Reviewer"));
}

#[test]
fn redact_leaves_blocks_and_no_copy_of_the_text() {
    let source = docx(&para("Account 12345678 is closed."));
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"redact","paragraph":"body:p:0","find":"12345678"}]}"#).unwrap();
    let out = apply_plan(&source, &plan).unwrap();
    for doc in [&out.clean, &out.redline] {
        assert_word_valid_package(doc);
        assert!(!contains(&all_bytes(doc), "12345678"));
    }
    assert_eq!(jubarte::inspect::paragraphs(&out.clean).unwrap()[0].text, "Account ████████ is closed.");
}

#[test]
fn a_redaction_that_survives_in_a_comment_is_refused() {
    let source = docx(&para("Account 12345678 is closed."));
    let commented = apply_plan(&source, &EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"comment","paragraph":"body:p:0","text":"12345678 again"}]}"#).unwrap()).unwrap().clean;
    let e = apply_plan(&commented, &EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"redact","paragraph":"body:p:0","find":"12345678"}]}"#).unwrap()).unwrap_err();
    assert_eq!(e.code, "REDACTION_LEAK");
    assert!(e.message.contains("word/comments.xml"));
}
```

- [ ] **Step 2: Run, expect `could not find scrub`.** **Step 3: Implement**
as designed; `scrub` ends with `validate()` and returns `ScrubError::Invalid`
if it introduced a finding. CLI `jubarte scrub FILE -o OUT [--author-alias
NAME] [--rsids] [--docprops] [--comments]` (no flag means all four with
alias "Author"). **Step 4: Pass. Step 5: Docs** (skill §2: `redact`; a
new §6 "Before sending a document out": `scrub`). **Step 6: Commit**
`feat(scrub): author alias, rsids, docProps, comments; redact plan operation with leak check`.

---

### Task 10: S5, `settings` plan operation (track revisions, update fields, protection)

**Files:**
- Modify: `src/edit.rs` (`OperationKind::Settings`), `src/bin/jubarte.rs` (no new command; plans only), `src/capabilities.rs`
- Create: `src/settings.rs` (ordered writer; shared with `fields` for `w:updateFields`)
- Test: `tests/edit_settings.rs`, plus an order assertion in `tests/schema_consistency.rs` against `tests/data/wml_main_schema.json`

Wire: `{"kind":"settings","track_revisions":true,"update_fields":true,"protection":{"edit":"trackedChanges|readOnly|comments|forms","enforcement":true}}`.
Writes into `word/settings.xml` (created with relationship and override when
absent; `ensure_factory_package_chrome` in `src/document_comparer.rs:3150`
shows the three steps) in `CT_Settings` order. The subset this task
orders, in schema sequence: `writeProtection, view, zoom, ...,
revisionView, trackRevisions, doNotTrackMoves, doNotTrackFormatting,
documentProtection, autoFormatOverride, styleLockTheme, styleLockQFSet,
defaultTabStop, ..., updateFields, hdrShapeDefaults, footnotePr, endnotePr,
compat, docVars, rsids, ...`. The exact full order is asserted from
`wml_main_schema.json` by the schema-consistency test so the hand table
cannot drift. `password` is refused with `UNSUPPORTED` (Word's legacy hash
needs `w:cryptProviderType`, `w:cryptAlgorithmSid`, spin count and salt;
out of scope and documented). Settings apply to clean and redline alike
(they are not revisions).

- [ ] **Step 1: Failing test**

```rust
// tests/edit_settings.rs
mod common;
use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::summary;

#[test]
fn settings_are_written_in_schema_order_into_a_new_settings_part() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"settings","track_revisions":true,"update_fields":true,"protection":{"edit":"trackedChanges"}}]}"#).unwrap();
    let out = apply_plan(&docx(&para("x")), &plan).unwrap();
    assert_word_valid_package(&out.clean);
    let xml = part_string(&out.clean, "word/settings.xml").unwrap();
    let track = xml.find("<w:trackRevisions/>").unwrap();
    let prot = xml.find(r#"<w:documentProtection w:edit="trackedChanges" w:enforcement="1"/>"#).unwrap();
    let update = xml.find("<w:updateFields/>").unwrap();
    assert!(track < prot && prot < update, "{xml}");
    assert!(summary(&out.clean).unwrap().track_changes);
}
```

- [ ] **Step 2: Run, expect `unknown kind "settings"`.** **Step 3:
Implement** `src/settings.rs` with `SETTINGS_ORDER: &[&str]` and
`fn set_child(dom, settings_root, local, attrs)` that inserts at the ordered
position or replaces the existing child; `edit.rs` arm. **Step 4: Pass.
Step 5: Docs** (skill §2; note that `protection` without a password is
Word's "enforce without password", which any user can turn off, which is
also what `set_protection.py` does). **Step 6: Commit**
`feat(edit): settings operation: trackRevisions, updateFields, documentProtection`.

---

### Task 11: Evidence for providers

**Files:**
- Create: `docs/adoption/README.md` (index), `docs/adoption/install-matrix.md`, `docs/adoption/validate-vs-xsd/`, `docs/adoption/creation.md`, `docs/adoption/fields.md`, `docs/adoption/python-docx.md`, `docs/adoption/anthropic-docx-skill.md`, `docs/adoption/openai-doc-skill.md`
- Create: `examples/agents/comment-thread/`, `examples/agents/accept-spacer-paragraph/`
- Modify: `skills/jubarte-documents/SKILL.md` (every task above touched it; this task reads it whole once and removes anything the engine no longer needs the agent to know)

Each provider file has the same four sections, written for the person who
maintains that skill:

1. **What your skill does today** (quoted, with the commit or fetch date).
2. **The one-line replacement** per script, with the exit codes and the
   JSON an agent reads.
3. **What you lose**: `.doc` (keep soffice), XSD (keep `validate.py` if you
   want the schema check; `jubarte validate` covers what XSD misses),
   WPS (Z.ai), and the license position (AGPL-3.0-only; external tool call).
4. **How to verify without trusting us**: the fixtures and commands,
   runnable in their sandbox: the spacer-paragraph accept case their own
   skill documents, the three Word-fatal files that pass XSD, the comment
   thread, the Windows install.

`examples/agents/accept-spacer-paragraph/` holds the smallest document that
reproduces the failure Anthropic's skill describes ("except when the deleted
paragraph is followed by an empty spacer paragraph"): `redline.docx`, the
LibreOffice result (an empty bullet), `jubarte accept` result (joined), and
the `jubarte text` of both. If LibreOffice cannot be reproduced locally,
say so in the README and ship only jubarte's result and Word's.

Run the six-task evaluation of
[06-agent-adoption.md](2026-09-26-jubarte-adoption/06-agent-adoption.md)
(A4) on the released 0.11.0 with pinned Codex and Claude settings and put
the numbers in `docs/adoption/README.md`. No number is quoted that was not
run after Task 6's release.

- [ ] Commit: `docs(adoption): provider replacement guides, install matrix, runnable evidence`.

---

## Counterarguments the plan does not hide

- **AGPL-3.0-only.** Nothing here changes it. A provider may decline on
  license alone; the external-CLI path is the best this plan can do.
- **Affiliated evidence.** Fidelity numbers come from `neurotic_docx_bench`,
  run by the jubarte author. Task 11 therefore ships reproducible fixtures,
  not scores.
- **Page numbers in S6 are jubarte's layout, not Word's.** On dense
  documents they can differ by one page; the field codes stay so Word can
  refresh. Say this in every place S6 is mentioned.
- **S8 is the largest surface and the most likely to carry Word-validity
  regressions.** Ring 1 in every test, Ring 2 and Ring 3 at release.
- **A provider may prefer a model that writes OOXML behind a validator.**
  S3 gives them that validator too; the pitch does not depend on the editor.
