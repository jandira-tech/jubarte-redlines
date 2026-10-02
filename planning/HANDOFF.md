<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Handoff: review-response implementation

## 0. Current status (supersedes sections 5 and 6 where they differ)

The session resumed after the usage limit and the agent work was integrated
onto `ccr-17e4f046-849j1u` (PR #273). State per proposed change:

| Item | State | Where |
| --- | --- | --- |
| P1, P4, P16, P6 (narrow) | done | `admission.rs`, `comparer/mod.rs`, `document_comparer.rs`, `wml_document.rs`, `tests/input_admission.rs` |
| P2, P3 | done | `admission.rs` (checked add, `get`, saturating cap) |
| P5 | done | `capabilities.rs`, `inspect.rs` (`STORY_KINDS`), `edit.rs` |
| P7 | merged into P1 (bounded Strict scan) | `strict_translation.rs` |
| P8, P11, P12, P14, P24, P25 | done (docs, dedup, `fuzz/` crate) | see CHANGELOG; fuzz targets are NOT built or run here (no cargo-fuzz or nightly) |
| P9, P19 | done, old names kept as deprecated aliases | `util/sha1.rs`; the `0.10.2` in `#[deprecated(since)]` is an assumption |
| P10 | done | `util/words.rs`; adds `unicode-properties` as a direct dependency |
| P13 | done, with one deviation | lib and CLI use `#![forbid(unsafe_code)]`; the package lint stays `deny` because `examples/mem_attribute.rs` has a counting allocator that needs `unsafe`. The two denies are `cfg_attr(not(test), ...)` on `admission`, `strict_translation`, `opc` |
| P15 | done | `.github/workflows`, `dependabot.yml` |
| P17 | done | `comparer/parts.rs`, `tests/compare_is_reproducible.rs` |
| P21, P23 | done | `markup_simplifier.rs`, `builtin_styles.rs` |
| P22 | half done | end-name checking is on and the whole repo corpus (819 packages) still passes; selecting parts by content type instead of extension was NOT done, because no engine path parses `.vml` parts as XML that I found (not exhaustively checked) |
| Remaining `process_footnote_endnote` panics | done | `comparer/footnotes.rs`; breaking for exhaustive matches on `RectifyError` |
| Python tests for the refusals | done: `jubarte-python/tests/test_document.py` passes, 17 tests, against a wheel built with `maturin develop` | `jubarte-python` |
| P20 (signed self-update) | not started, orthogonal to the comparer | |
| P6 (full settings redesign) | deferred | |

Verification on the integrated tree (HEAD at the time of writing):
`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`
and the targeted suites (lib 874, input_admission 10, edit_plan 53,
m30_notes_pipeline 8, m4h4_footnotes, edit_stories, agent_contracts,
m_cli_agent, compare_is_reproducible, m2_markup_simplifier, m_cli_no_panic,
sha1 tests) pass. I did not run the whole suite. CI on the PR is red only on
the four `convert_docx_to_pdf` tests, which also fail on the base.

## Earlier handoff (written when the session was cut off)

Written 2026-10-02, branch `ccr-17e4f046-849j1u`, draft PR #273
(https://github.com/jandira-tech/jubarte-redlines/pull/273). The session was
cut off by the account's session limit (resets 08:00 UTC); three of the four
implementing agents died mid-task from the same limit. Everything below is
what exists, where it is, and what has NOT been validated. Nothing in this
file is a conclusion about anyone's behaviour; it is state.

## 1. Where the work lives

| Item | Location | State |
|---|---|---|
| Plan (P1-P25 assessment) | `planning/review_response_plan.md` on this branch | committed (9054f17), amended by the verification findings in section 3 below, NOT yet edited into the plan file |
| P1 + P4 + P16 + narrow P6 (my own work) | main checkout, this branch, commit "wip(safety)" (see section 2) | targeted tests green; full suite, clippy, fmt-check NOT run |
| P2 + P3 (admission overflows) | worktree `.claude/worktrees/agent-abc1a3cd4077509b2`, branch `worktree-agent-abc1a3cd4077509b2`, commit `b5ced27` | committed by the agent; its final report never arrived, so test/clippy results are unknown; `cargo clean` not run there |
| P15 (CI hardening) | worktree `.claude/worktrees/agent-a508280ef898fcf05`, branch `worktree-agent-a508280ef898fcf05`, commit `199ac80` | complete and reported (section 4) |
| P9 + P19 (FNV rename, smoke test, Debug derives) | worktree `.claude/worktrees/agent-a8dbc90fb0203ef41`, branch `worktree-agent-a8dbc90fb0203ef41`, commit `981e663` | agent was interrupted while running clippy; I committed its working tree as a `wip:` commit. Its last message: "All green so far. Now clippy with -D warnings". Treat as unvalidated |
| P5 (manifest stories) | worktree `.claude/worktrees/agent-aed2703d8de42f0e1`, branch `worktree-agent-aed2703d8de42f0e1`, commit `36a617e` | agent was interrupted right after writing the red tests ("Tests are written. Now confirm the red state"). The implementation may be partial. I committed the tree as `wip:`. Treat as red/unfinished |

Worktrees are git worktrees of this repository; the branches are local only
(nothing was pushed from them). Each has its own `target/` (the P15 one has
none). Disk was at 25 G used / 14 G free when the session ended; run
`cargo clean` in each worktree before building anything else, or
`git worktree remove --force` after merging.

Merge order that avoids conflicts: P2+P3 (`admission.rs` only) first, then my
safety commit (touches `admission.rs` only by adding `InputLimits::compare()`
after the `Default` impl, so the merge is a clean non-overlapping hunk), then
P15 (YAML/README/CHANGELOG), then P9+P19, then P5. All five touch
`CHANGELOG.md` under Unreleased; expect trivial conflicts there.

## 2. My own change (P1, P4, P16, narrow P6) in detail

Design decision, made on the fact report in section 3.6: NO public signature
changes. `compare_documents*` keep `Result<Vec<u8>, OpcError>`. `OpcError` is
a foreign enum (rdocx-opc 0.1), and the crate already tunnels semantic
refusals as `OpcError::Io(InvalidData)` (`document_comparer.rs` fn
`invalid_content`). Both bindings convert errors through `Display`
(`jubarte-python/src/lib.rs:29-31`, `jubarte-wasm/src/lib.rs:54-56`), so they
need no change. All 209 `WmlComparerSettings` struct literals outside the
`Default` impl use `..spread`, so adding a field is additive.

Files changed (uncommitted at the time of writing; committed as "wip(safety)"
right after this file):

- `src/admission.rs`: `impl InputLimits { pub const fn compare() }` = 512 MiB
  file, 10_000 entries, 512 MiB/part, 2 GiB total, depth 256. Module doc
  updated (the "comparer keeps its historical tolerance" sentence is gone).
- `src/opc/mod.rs`: `pub(crate) fn refused<E: Error + Send + Sync + 'static>(E)
  -> OpcError` = `OpcError::Io(io::Error::new(InvalidData, err))`. The typed
  error is the `io::Error` source, so callers can `get_ref()` + downcast.
- `src/strict_translation.rs`: `strict_to_transitional_docx(bytes)` now
  delegates to new `pub fn strict_to_transitional_docx_within(bytes,
  InputLimits)`. Inside: `Cursor::new(bytes)` (no `to_vec` copy), no
  `Vec::with_capacity(f.size())`, every entry read through
  `.take(cap.saturating_add(1))` with `cap = max_part_bytes.min(remaining
  package budget)`; over-cap returns the input unchanged (the function's
  existing failure contract). Extension check left case-sensitive on purpose
  (P7 follow-up). Unit test `oversized_part_is_left_untouched_under_a_small_budget`.
- `src/comparer/mod.rs`: new `pub fn try_compare_bodies_faithful_with_notes(...)
  -> Result<NodeId, footnotes::RectifyError>`; the old
  `compare_bodies_faithful_with_notes` is now a wrapper with a `# Panics`
  doc that `unwrap_or_else(panic!)`s (kept infallible so 127 test call sites
  and `compare_bodies_faithful` compile unchanged; NOT `#[deprecated]`,
  because that would fail `-D warnings` on those tests). The function body's
  `?` replaced the panic at the old line 718; the body's final `root` is now
  `Ok(root)`. New field `pub input_limits: InputLimits` on
  `WmlComparerSettings` (default `InputLimits::compare()`), with doc.
  `in_stamp_residual` got `#[doc(hidden)]` and an honest doc; it cannot be
  made `pub(crate)` because struct-update syntax from outside the module
  fails on a private field (E0451), which would break every `..default()`
  literal including `jubarte-python/src/lib.rs:290`. That is the whole of P6
  I did; see section 3 for why the full redesign is deferred.
- `src/document_comparer.rs`: new `fn admit_input(side, bytes, limits)`
  (prefixes the message with "original document:"/"modified document:" and
  rebuilds an `AdmissionError` so the downcast target stays
  `AdmissionError`). `compare_documents_impl` admits `original` before the
  identical-input fast path and `modified` right after it; both
  `strict_to_transitional_docx` calls became `_within(…, settings.input_limits)`;
  the notes call uses `try_compare_bodies_faithful_with_notes(...).map_err(crate::opc::refused)?`.
- `src/wml_document.rs`: `from_bytes` admits with `InputLimits::compare()`
  first. The `document_byte_array` double copy is NOT removed (pub field;
  nobody reads it; deferred to a breaking release).
- `tests/input_admission.rs` (new, 10 tests, all passing): compare refuses an
  oversized part on either side, refuses too many entries, admits before the
  identical-input shortcut, default entry point refuses 10_013 entries,
  refusal downcasts to `AdmissionError` with code `INPUT_LIMIT`, small
  packages and the fixture pair still compare, `WmlDocument::from_bytes`
  admits. Inputs are built with the `zip` crate's normal writer (no byte
  patching); the earlier attempt to hand-craft a malformed ZIP tripped the
  safety classifier, so keep to legitimate archives plus small budgets.
- `tests/m30_notes_pipeline.rs`: new test
  `missing_with_revisions_part_is_a_typed_error_not_a_panic` asserting
  `Err(RectifyError::MissingTargetPart { kind: "footnotes" })` and that the
  before-side definition is still in place. Two earlier versions of its
  assertion were wrong (B.2 stamps `pt:*` scratch on the before part and
  redlines the after part before rectify runs); the committed assertion is
  the one the rectify contract actually promises.
- `tests/m_cli_no_panic.rs`: `wml_document_missing_main_returns_err` now
  accepts the refusal at `from_bytes` time (admission reports "main document
  part word/document.xml is missing"), falling back to the old
  `main_document()` check if a shell ever opens.

Validation done: `cargo test --test input_admission --test m30_notes_pipeline
--test m_cli_no_panic` all green (10 + 6 + 10 tests); `cargo test --lib
strict_translation` green (5 tests). `cargo fmt` was run.

Validation NOT done (do these before marking the PR ready):
1. `cargo test --all-features --no-fail-fast` (full suite, ~275 test files
   call `compare_documents`; the admission gate must refuse no corpus
   fixture; if one is refused, look at which `AdmissionErrorKind` and
   decide whether the fixture or the gate is wrong).
2. `cargo clippy --all-targets --all-features -- -D warnings`.
3. `cargo fmt --check` (fmt was run, check was not).
4. CLI `--help` smoke test.
5. CHANGELOG.md entry under Unreleased for: compare admission + `input_limits`
   + `InputLimits::compare()` + `strict_to_transitional_docx_within` +
   `try_compare_bodies_faithful_with_notes` + `WmlDocument::from_bytes`
   admission. Not written yet.
6. Python/WASM: no source change needed, but add the two refusal cases to
   the Python test suite as the plan says (abort vs exception is only
   observable there).
7. Performance note for the CHANGELOG: admission is one extra inflate pass
   over each input (admit, then Strict scan, then PartFs::open); the plan's
   D7 follow-up is to admit once into an in-memory part map.

## 3. Verification findings that amend `planning/review_response_plan.md`

Five read-only agents verified the plan against source. Each finding below
is file:line-grounded in their reports (transcripts under
`/tmp/claude-0/-home-user/f8a0d696-6130-5377-a383-b87d03cc74ed/tasks/`, which
die with the container; the facts are reproduced here).

### 3.1 Corrections to my plan
- **P25 (INVALID as I wrote it).** The "full-string backstop" claim is wrong.
  `lcs.rs:853 extend_common_run` and `lcs.rs:1062` decide common-run equality
  on `sha1_key128()` alone; `atoms.rs:185-186, 326-329` say so. The doc
  comments at `sha1.rs:112-117`, `lcs.rs:836-840`, `lcs.rs:1022-1023`,
  `lcs.rs:1056-1060` describe a string check that no longer runs. The
  `with_colliding_key` test helper (`atoms.rs:232`) only corrupts the u64 key,
  so it cannot detect this. Correct risk statement: inputs are SHA-1 hex, so
  an attacker cannot choose the FNV input bytes; a generic 128-bit collision
  is ~2^64 work. Action: fix the four stale doc comments (or restore
  `&& sha1()==sha1()` on key128 hits, one memcmp per hit). I relayed this to
  the P9 agent so its `sha1.rs` doc rewrite does not repeat the stale claim.
- **P9**: the rename IS a public API change (`jubarte::util::sha1::*` and the
  `util/mod.rs:12` re-export; `tests/perf_sha1_key.rs:20` and
  `tests/sha1_key_invariant.rs:13` import through the external path; both
  names are in `docs/api/jubarte-v0.10.*.api.txt`). I told the agent to keep
  deprecated aliases. `ComparisonUnit::sha1_key/sha1_key128` also return FNV.
- **P10**: two callers, not one: `src/edit/rewrite.rs:31` and
  `src/markdown/diff.rs:22` (aliased import `word_tokens as tokens`, used at
  :417 and :449-450), so CriticMarkup output and the CLI/Python
  `diff_markdown` are in the blast radius. `unicode-properties` 0.1.4 with
  `general-category` is already in `Cargo.lock` via rustybuzz, so no new
  crate is needed.
- **P17 (upgrade from "skip" to "actionable")**: `pt:Unid` attributes are
  stripped, but `src/comparer/parts.rs:144-157 dest_uri_for_reconciled_part`
  names copied media `word/media/P{unid}.ext` with the process-global
  counter, and that name reaches the output ZIP and the rels. Same compare
  run twice in one process (server, WASM instance, parallel tests) gives
  different bytes. Fix: per-compare counter or content-derived name; add a
  run-twice regression test with an inserted image; fix the `unid.rs:5-13`
  reproducibility doc.
- **P21**: code claim holds (`markup_simplifier.rs:108-112`), but no
  production code calls `transform_element_to_single_character_runs`; only
  `tests/m2_markup_simplifier.rs` and the public API. Second failure mode: a
  `w:r` with 2+ characters silently truncates to `v[0]`. Fix is an API
  contract (return `Option`/assert precondition), not a comparer crash.
- **P22**: both claims valid, and a stricter validator already exists:
  `src/xmllinq/parse.rs:418 validate_xml` (check_end_names on, rejects
  multiple roots, unclosed root, DTD) with a hard-coded depth 256; reuse it
  parameterised on depth. Fixture survey: 819 docx, 12,189 xml/rels parts,
  zero rejections under expat; entry extensions are only xml/rels/png/jpeg/
  odttf/bin/emf/wmf/xlsx/gif/jpg/tif/tiff; no XML-typed part under a
  non-.xml name exists in the fixtures. Content-type selection needs a
  pre-read of `[Content_Types].xml` because it may not be the first entry.
- **P7**: valid; also reached without admission from `src/convert/mod.rs:445`
  (docx to PDF/PNG). My `_within` bound covers the Strict scan there, but the
  convert path still has no admission before `PartFs::open`.
- **P24**: "Lossless" is `src/lib.rs:7` and `Cargo.toml:16`, not line 3.
  `sanitize_sdt_properties` runs in both modes (`comparer/mod.rs:413-414`);
  hyperlink unwrapping is Word mode only (`mod.rs:784/809`) and only for
  internal anchor links without `r:id`; also lossy: SDT flattening
  (`mod.rs:812`) and `mc:AlternateContent` resolution (`mod.rs:408-409`).
- **P20**: valid; `src/update.rs:143-154` uses `self_update` with
  `checksum_from_asset("SHA256SUMS.txt")`, no `signatures` feature, sums file
  from the same release job (`release.yml:210-215`). Integrity against
  corruption only, no authenticity.
- **P23**: valid; `builtin_styles.rs:394-398` allocates via
  `to_ascii_lowercase()`; one production call site `document_comparer.rs:1610`
  inside `style_match_key`, which runs per style per lookup. Fix:
  `binary_search_by(|probe| probe.bytes().cmp(name.bytes().map(|b| b.to_ascii_lowercase())))`.
- **P8**: orphan paragraph is `comparer/mod.rs:1275-1277` (not 1274-1281);
  `cached_xname!` at `namespaces.rs:83-91` has the same literal `$local` bug
  as `ns_struct!`; `SECT_GEOMETRY` (`mod.rs:154`) has 6 items, comment at
  137-138 lists 5 (`docGrid` missing).
- **P11**: the five repeats with no justifying comment are
  `strip_trailing_empty_pure_ins` (953), `ensure_empty_pure_i_before_short_title_del`
  (959), `strip_empty_pure_ins_before_trailing_pure_dels` (961),
  `strip_last_pure_del_mark_only_ppr` (948), `strip_last_pure_del_mark_when_pprchange`
  (950). The comment at 1076-1080 belongs to `hoist_hyperlinks_out_of_revisions`
  (1086) but the textbox block was inserted between them. The `default_line`
  closure at 785-792 is duplicated verbatim at 1047-1054.
- **P12**: constants already exist to promote: `DEFAULT_DATE`
  (`document_comparer.rs:190`, pub), `WORD_FACTORY_SPACING` (:243),
  `THEME_REL` (:3257), `OFFICE_DOCUMENT_REL` (`admission.rs:138`), `MC::URI`
  (`namespaces.rs:47`; `comparer/mod.rs:656-658` should use `MC::ns()`).
  Do NOT replace `convert/mod.rs:10299` (epoch formatting assertion).
  `"word/document.xml"` fallback appears at ~25 production sites; reuse one
  helper (`inspect.rs:450 main_part`, `changes.rs:232`).
- **P19**: `ComparisonLog` (`comparison_log.rs:28`) is `#[derive(Default)]`
  only; `CompareContext` (`comparer/mod.rs:1371`) derives nothing and is
  referenced nowhere in src/tests/benches/examples/bindings (dead public
  type).
- **P15**: `dependabot.yml` already existed (the plan assumed none); the
  msrv job used `dtolnay/rust-toolchain@master`; `release.yml` was also
  unpinned; `--workspace` would not have linted the bindings because the
  root `[workspace]` is empty and each binding is a standalone package.

### 3.2 Facts that drove the P1/P4/P6/P16 design (agent report, all verified)
- Only `document_comparer.rs:6289` passes `Some(notes)`; every other caller
  of `compare_bodies_faithful*` passes `None`.
- `footnotes.rs process_footnote_endnote` (:989) also panics on the notes
  path: 13 `.expect` calls (:1030-1127) and `panic!("Internal error")` at
  :1149. NOT addressed by P4; follow-up.
- Orphan references are already refused earlier by `pre_process_markup`
  (`document_comparer.rs:4512-4519`), so `MissingNoteDef` from rectify is
  hard to reach from the package API; `MissingTargetPart` is the reachable
  one.
- `WmlComparerSettings`: 209 struct literals, all `..spread` except
  `Default`; `in_stamp_residual` written only at `lcs.rs:1527`, read only at
  `lcs.rs:3021`. `detail_threshold` is written directly at ~20 sites in
  `lcs.rs`, so a builder that hides fields would need `pub(crate)` setters.
  Full P6 (CompareMode enum, private fields) is deferred: FRU on private
  fields breaks every external literal.
- `WmlDocument`: used only by three test files; `document_byte_array` and
  `file_name` are never read; `part_fs_mut` has zero callers.
- `admit` is called in production only from `inspect.rs:290` with
  `InputLimits::default()`; `capabilities.rs:160` reports those defaults as
  the manifest's `input` budget (that is the inspect/edit budget; the
  compare budget is now separate and larger; consider surfacing it).

## 4. Agent P15 result (complete)
Commit `199ac80` on `worktree-agent-a508280ef898fcf05`: all 26 `uses:` in
`ci.yml` and 11 in `release.yml` pinned to 40-char SHAs with tag comments,
every SHA fetched from api.github.com (table with source URLs is in the
agent's commit message/CHANGELOG entry; cross-checked against
`jubarte-app/.github/workflows/ci.yml` which already pinned the same SHAs);
lint job now installs `wasm32-unknown-unknown` and runs clippy in
`jubarte-rust-inproc`, `jubarte-python`, `jubarte-wasm` (jubarte-app left as
a documented TODO: needs GTK/WebKit); msrv job runs `cargo test
--all-features --no-fail-fast` on 1.88; README:662 now says "MSRV testing on
Rust 1.88 (the all-feature test suite, on Linux)"; dependabot cargo entry
grouped minor/patch. YAML parsed with PyYAML; `reuse lint` compliant. Risk
the agent flagged: the new binding clippy steps and the MSRV test run may
turn CI red once on pre-existing warnings; `jubarte-python/src/lib.rs:247`
fails `cargo fmt --check` already (not gated).

## 5. Agents P2+P3, P9+P19, P5 (interrupted)
- **P2+P3** committed `b5ced27` "fix(admission): bounds-check ZIP64 locator
  offset and saturate part cap" but never reported. Before merging: read the
  diff, run `cargo test --lib admission`, `cargo clippy --all-targets
  --all-features -- -D warnings`, `cargo fmt --check` in that worktree. Its
  prompt asked for tests (a) offset past buffer end, (b) offset near
  `usize::MAX`, (c) truncated record, (d) `u64::MAX` limits admitting a
  minimal package; check all four exist.
- **P9+P19** `981e663` (wip): files touched: CHANGELOG, LCS_PERF_PLAN.md,
  `comparer/atoms.rs`, `comparer/mod.rs` (CompareContext Debug),
  `comparison_log.rs`, `lib.rs`, `util/mod.rs`, `util/sha1.rs`,
  `tests/perf_sha1_key.rs`, `tests/sha1_key_invariant.rs`. Agent said tests
  were green and was starting clippy. Verify the `sha1.rs` doc does NOT claim
  a string backstop (section 3.1 P25) and that deprecated aliases exist.
- **P5** `36a617e` (wip): files touched: CHANGELOG,
  `skills/jubarte-documents/SKILL.md`, `src/capabilities.rs`, `src/edit.rs`,
  `src/inspect.rs`, `tests/agent_contracts.rs`, `tests/edit_stories.rs`,
  `tests/m_cli_agent.rs`. Agent had written the red tests and was about to
  confirm red; the implementation is likely incomplete. Expect failing tests
  until finished. Its brief: single source of truth for story names in the
  edit module consumed by both the selector parser and `capabilities`,
  manifest lists exactly the names the parser accepts, `cfg!`-derived
  operations where a feature gate exists.

## 6. Not started
P13 (lint policy: `unsafe_code = "forbid"`, scoped `indexing_slicing` /
`arithmetic_side_effects` denies on admission/strict_translation/opc; do
after P2 merges), P14 (cargo-fuzz targets; `cargo fuzz` is not installed
here), P22, P17 (now actionable, see 3.1), P21, P23, P10, P8/P11/P12 docs
and dedup, P24 doc fix, P25 doc fix, remaining `process_footnote_endnote`
panics, convert-path admission (P7 note), python test cases for the two
refusals.

## 7. PR #273 and notifications
Draft PR #273 carries only the planning doc plus whatever is pushed with
this handoff. Subscribed; the safety-net check-in `trig_01XiQLjWdV3Wep6mWMTZEywP`
was armed for 07:00 UTC. At shutdown, 4 notifications were unread (3 GitHub
events on #273, 1 scheduled trigger); they were not read, so the next
session should call `ReadNotifications` first and look at #273's CI on the
head that includes the "wip(safety)" commit, since the full suite and clippy
were not run locally.

## 8. Environment notes for the next session
- Baseline `cargo test --no-run --lib` takes ~4 min and 2.3 GB `target/` on
  this 4-core box; each worktree build adds the same. Three parallel agent
  builds pushed disk to 25 G used. Run `cargo clean` in finished worktrees.
- `cargo llvm-cov` and `cargo fuzz` are not installed; coverage numbers were
  not produced.
- Crafting malformed ZIP bytes by hand in a test file triggered the safety
  classifier once; use the `zip` crate writer with small `InputLimits`
  instead, which is what `tests/input_admission.rs` does.

## 9. Patch export
The four agent branches were local to the container. Their commits are
exported with `git format-patch b420d64..HEAD` into
`planning/handoff-patches/<item>/`; apply with `git am` onto a branch based
on `b420d64` (or on this branch, resolving CHANGELOG.md), then delete the
directory once merged. The patches are the authoritative copy if the
worktrees are gone.

## 10. PR #273 notifications read at shutdown (state as of 07:51 UTC)
- CI is RED on every head pushed so far: `coverage`, `test (ubuntu-latest)`
  and `test (windows-latest)` failed on `9054f17` (the docs-only planning
  commit), on `9d4689f`, and on `a920d04`. Because `9054f17` changes no
  Rust, these failures are most likely pre-existing on the base
  (`b420d64`) or environmental, not caused by the WIP safety commit. That is
  an inference from the head SHAs, not something I verified: the job logs
  were not read. First action next session: open the failing job logs
  (run 36972190934 for `9054f17`, 36977718296 for `9d4689f`, 36977749152
  for `a920d04`, jobs under
  https://github.com/jandira-tech/jubarte-redlines/actions) and check
  whether the same test fails on base `b420d64`. If it does, that is a
  base-branch failure to report once on the PR, not something to fix here;
  if it does not, it is caused by one of the commits on this branch.
- `arthrod` marked the PR ready for review at 07:20 UTC (it is no longer a
  draft). It carries WIP code plus `planning/handoff-patches/`; consider
  converting it back to draft, or trimming the patches directory, before a
  human reviews it.
- Review bots are not usable right now: CodeRabbit hit its free review
  limit (next included review about an hour after 07:20), Sourcery's
  250,000-character budget is spent (retry in about 4 days), Codex
  and Qodo report exhausted allowance / inactive subscription. No review
  finding exists on the PR; no action was taken on any of those comments.
- The safety-net check-in `trig_01XiQLjWdV3Wep6mWMTZEywP` fired at 07:01 and
  was not acted on because this session was already in handoff mode; it is
  one-shot and will not fire again. The PR remains subscribed.
