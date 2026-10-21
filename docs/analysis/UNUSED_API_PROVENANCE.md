# Unused public API: where it came from and what it was for

Follow-up to [UNUSED_PUBLIC_API.md](UNUSED_PUBLIC_API.md). For each item that report lists, this traces the commit that added it, whether it has shipped in a release, and what the code, the changelog and the design documents say it was meant for. Then it names a concrete use, or says plainly that there is none.

Sources used: the full git history (3,537 commits; the clone was shallow before), the release API snapshots in `docs/api/` (v0.10.0 to v0.12.0), `CHANGELOG.md`, and the CLI v2 design documents in `docs/design/cli-v2/`. Machine-readable data for all 275 items: [unused-api-provenance.json](unused-api-provenance.json), produced by `scripts/trace_unused_api.py`. No source file was changed.

## The 275 items by origin

The report's three lists add up to 275 items (16 never mentioned, 65 mentioned only in their own file, 194 used inside `src/` only).

| | Items |
|---|---:|
| Already in a published release snapshot (semver applies) | 213 |
| Added after v0.12.0 (not yet published) | 62 |

Where they live (files with three or more):

| File | Published | Unpublished | What kind of code |
|---|---:|---:|---|
| `comparer/finalize.rs` | 81 | 0 | Word-parity cleanup passes, mostly named `M###` after the benchmark milestone that introduced them |
| `cli.rs` | 16 | 8 | Command-line argument types and helpers |
| `ids.rs` | 0 | 20 | The new change/comment/paragraph id grammar |
| `xmllinq/mod.rs` | 19 | 0 | `Dom` convenience methods in the LINQ-to-XML style |
| `edit.rs` | 7 | 4 | Plan types and legacy dispatcher helpers |
| `result.rs` | 0 | 9 | v2 machine result envelope |
| `edit/v2.rs` | 0 | 6 | PlanV2 contract |
| `util/sha1.rs`, `settings.rs`, `opc/mod.rs`, `comparer/atoms.rs`, `comparer/tables.rs`, `legacy_doc.rs`, `comparer/mod.rs`, `edit/record.rs`, `markdown/mod.rs` | 44 | 6 | Helpers of modules that are `pub mod` |

93 of the published items arrived in the very first commit ("Initial import of jubarte-redlines", 2026-07-24), so for those the history shows no separate intent; the doc comment and the surrounding code are the evidence.

## Why most of them are "unused"

A public item shows up in the report when nothing in `tests/`, `benches/`, `examples/`, `fuzz/` or the bindings names it. For 259 of the 275 that is a consequence of module visibility, not a leftover:

- `comparer::finalize`, `util::sha1`, `opc`, `settings`, `xmllinq` and `cli` are `pub mod`, so every `pub fn` inside is public API from the day it was written, whether or not it was meant for outsiders. The tests do reach into `comparer::finalize` (for example `coalesce_adjacent_revisions`, `move_paragraph_properties_first`, `resolve_alternate_content`), so that module is deliberately a test-facing surface. The 81 of its public functions that nothing outside `src/` names are still tested inside the file: 71 of the 79 functions are named in `finalize.rs`'s own `#[cfg(test)]` modules (many only in the story-scope invariant table), and only 8 are named in no test.
- The 62 unpublished items are scaffolding for the CLI v2 work in progress (ids grammar, result envelope, PlanV2, new CLI flags). Their callers are the commands that are not written yet.

## The 16 never-mentioned items

| Item | Added | Published | What it was for | A use for it |
|---|---|---|---|---|
| `comparer::finalize::promote_mid_pure_del_spacing_from_pprchange`, `strip_default_left_jc`, `strip_redundant_equal_spacing_pprchange` | Initial import | yes, since at least v0.10.0 | Hooks to run passes M228, M231 and M226 one at a time: the doc comments say "exposed separately so the test corpus can hit it directly". All three now just call `cleanup_spacing_and_default_jc`, which does all three jobs, and no test calls them. | None as they stand: calling one runs all three passes, so it cannot test one pass. Either mark them `#[deprecated]` aliases of `cleanup_spacing_and_default_jc` (what the crate did for `sha1_fingerprint`), or split the combined pass back into three functions if per-pass tests are wanted. |
| `edit::v2::TEXT_CODEC_VERSION` (= 1) | 2026-10-17, "PlanV2 core contract and exact text codec (V2-01, #821)" | no | The version tag of the exact logical text codec, "fixed by PlanV2". The design says to "freeze the codec in the schema version" and to bind read cursors to "projection/codec". | Report it in the `capabilities` manifest and bind it into the `read --for-edit` cursor, as the plan describes. Nothing emits it yet. |
| `result::NextAction::with_prerequisites` | 2026-10-17, "machine result envelope for the v2 routes (V2-03, #825)" | no | The proposal says next-call suggestions are "argument arrays, with prerequisites and purpose". | Used by the v2 commands when they suggest a follow-up call (for example inspect before a decision on a pending-revision refusal). |
| `result::Error::with_source_digest` | same | no | Proposal: errors "identify code, operation pointer, source hash, resolved target". `with_source` already sets the digest from a `Source`; this sets it when only a digest is known. | A refusal raised before the file is read, such as an `--expect-sha256` mismatch. |
| `result::Error::with_target` | same | no | The "resolved target" half of the same sentence: the pointer an operation resolved to. | Operation refusals such as `AMBIGUOUS_ANCHOR` in `edit apply`. |
| `result::Source::from_digest`, `Source::with_path` | same | no | `Source::new` hashes bytes. `from_digest` is for a digest the caller already has, `with_path` fills the "absolute path of the source, when known" field. | The v2 commands, once they build the `source` block of the envelope from a path and a precomputed digest. |
| `result::Counts::total_operations` | same | no | Sum of `body_operations_applied` and `review_operations_applied`, the two counters in the proposal's example envelope. | The human summary line and the exit-status decision of `edit apply`. |
| `result::exit_status_for_envelope` | same | no | Maps an envelope to the exit-status table in proposal section 12 (0, 1, 2, 3, 4, 5, 64, 124). | The v2 commands' `main`, after printing the envelope. It is a pure function over a table, so it is the easiest thing here to cover with tests now, before any command exists. |
| `revision_processor::tracked_revisions_elements` | Initial import | yes | The C# `RevisionProcessor` name list as a `Vec<XName>`. The crate's real check, `element_has_tracked_revisions`, uses the static set behind it. | A single source of truth for revision element names. Three more hand-kept lists describe parts of the same set: `CHANGES` in `debug/diff.rs:195`, `FORMATTING` in `changes.rs:427` and `REVISION_CHANGE_ELEMENTS` in `document_comparer.rs:444`. A test asserting each is a subset of `TRACKED_REVISION_LOCALS` would catch drift between them. |
| `util::sha1::sha1_fingerprint128` | name from the first perf commits; renamed and deprecated in `a2a0cf14` (2026-10-02) | yes | A deprecated alias of `fnv1a_128`. The commit and the changelog both say the aliases "stay as deprecated aliases for one release and are then removed". | None. It first shipped in v0.11.0 and is still in v0.11.2, v0.11.3 and v0.12.0, so by its own note it is overdue for removal. `sha1_fingerprint` and `util::sha1_fingerprint` are in the same state. |
| `xmllinq::Dom::has_attributes` | Initial import | yes, since v0.11.3 | Mirrors LINQ-to-XML `HasAttributes`; the doc says it equals `attr_count(id) > 0` without cloning anything. | 11 call sites (a few of them in test code) build a whole `Vec<(XName, String)>` with `dom.attributes(x)` only to call `.is_empty()` on it (`comparer/finalize.rs` 3, `document_comparer.rs` 5, `comparer/revisions.rs` 1, `revision_processor.rs` 1, `revision_processor/style_records.rs` 1). `!dom.has_attributes(x)` does the same without the allocation. |
| `xmllinq::Dom::first_node`, `last_node` | Initial import | yes, since v0.11.3 | Mirror LINQ-to-XML `FirstNode` and `LastNode`. | No call site needs them as written. Ten sites (two of them test assertions) do `dom.nodes(x).is_empty()`, which clones the child list; `child_count(x) == 0` already exists for that and `first_node(x).is_none()` would too. |

## Clusters in the 259 others

**`comparer/finalize.rs` (81, all published).** Each function is one Word-parity pass added for a benchmark milestone (`M69`, `M226`, `M392`, `M471`, ...), taken from the initial import. They are public because the module is. The pipeline calls them (that is why they are in the "used inside `src/`" list), and 71 of the 79 functions are named in the file's own test modules; only `sanitize_sdt_properties`, `unwrap_hyperlinks_to_styled_runs`, `normalize_incomplete_spacing`, `closing_paragraph_properties`, `last_body_para_spacing` and the three wrappers are named in no test. Narrowing the module to `pub(crate)` instead is a semver break, and the existing tests that import from `comparer::finalize` would have to move inside the crate.

**`ids.rs` (20, all unpublished).** The constants `CHANGE_SPELLINGS`, `COMMENT_SPELLINGS`, `PARAGRAPH_SPELLINGS`, `TABLE_SPELLINGS`, `CONTROL_SPELLINGS` and the `*_EXPECTED` refusal texts feed the capabilities manifest and the `-p` refusals; `IdError`, `parse_location`, `short_number`, `hint_where`, `comment_text_id`, `comment_id_de` and `part_story` are the id grammar's building blocks, added in "one grammar for change, comment and paragraph ids". They are used inside the crate and nowhere else. Nothing has shipped, so `pub(crate)` would cost nothing today; making them public later is the cheaper direction.

**`result.rs` and `edit/v2.rs` (15, all unpublished).** See the table above. They are the typed contract that `edit apply`, `edit template` and `review accept` will produce, per the module doc. The design documents under `docs/design/cli-v2/` are the expected callers.

**`cli.rs` (24).** The 16 published ones are clap argument types (`ReadArgs`, `EditOptions`, `CompareArgs`, `Selection`, ...) used by `bin/jubarte.rs` and `cli.rs` itself. The 8 unpublished are the retired-task-name handling (`REMOVED_TASKS`, `removed_tasks`, `removed_task_note`, `apply_retired`, `removed_error`) and `Tracking`, `EditTarget`, `slash_path`. They are public because the shared parser is also used by `examples/cli_flags.rs`, `tests/cli_parser.rs` and the Python and wasm bindings.

**`xmllinq/mod.rs` (19, all published since v0.11.3).** `Dom` methods in the LINQ-to-XML style (`new_text`, `new_pi`, `set_name`, `add_first`, `remove_nodes`, `replace_with`, `next_element`, `nodes_after_self`, `with_scratch`, ...). They round out the `Dom` API. Those used inside the crate are fine; the three in the table above have concrete call sites that would benefit from them.

**`util/sha1.rs` (6).** The SHA-1 digest and hex helpers used by the comparison-unit keys; public because `util::sha1` is. The one exception is the deprecated alias in the table above.

**`settings.rs`, `opc/mod.rs`, `comparer/atoms.rs`, `comparer/tables.rs`, `legacy_doc.rs`, `comparer/mod.rs`.** Helpers and tables of their modules. `comparer/tables.rs` holds the C# constants verbatim, with their `WmlComparer.ts` line numbers in the doc comments (`RECURSION_ELEMENTS`, `ATTRIBUTES_TO_TRIM_WHEN_CLONING`, `S_RELATIONSHIP_ATTRIBUTE_NAMES`). They exist for parity with the original, and the crate uses them.

## What I would do, in order

1. **Leave the 62 unpublished items alone.** They are scaffolding for named work items (V2-01 and V2-03 are in the commit subjects). When the v2 commands land, they are the first callers. Meanwhile, add unit tests for `exit_status_for_envelope` and `Counts::total_operations`: both are pure, both have a spec in the proposal, and no test touches them today.
2. **Use `Dom::has_attributes` at the 11 call sites** that clone attributes just to test for none. It is an allocation fix with no behaviour change, and it gives the method its first caller.
3. **Remove `sha1_fingerprint128`, `sha1_fingerprint` and `util::sha1_fingerprint`** at the next release. The changelog promised one release and they have now shipped in four (v0.11.0, v0.11.2, v0.11.3, v0.12.0).
4. **Decide the three `finalize` wrappers.** Deprecate them as aliases of `cleanup_spacing_and_default_jc`, or split that pass so each can be tested alone.
5. **Add direct tests for the `finalize` passes** that no test names (the five listed above), after measuring coverage, since most passes already appear in the file's own tests.
6. **Use `tracked_revisions_elements` (or the static set behind it) in a drift test** against the three other revision-name lists.
7. **Do not mass-narrow the published items to `pub(crate)`.** 213 of the 275 are in release snapshots, so each is a semver break for no behavioural gain.

## Limits

- "Added" is the first commit that touches the name in its file (`git log -S`), so a rename shows up as a new item and the intent behind an earlier name is not followed.
- "Published" means present in a `docs/api/jubarte-v*.api.txt` snapshot, v0.10.0 to v0.12.0. An item first listed in v0.10.0 may be older than that; the earliest snapshot kept is v0.10.0.
- "Expected use" for items from the initial import rests on the doc comment and the C# source it mirrors. Where a design document names the caller (the v2 items), it is cited; where nothing does, the entry says so.
- The proposed uses were found by reading and grepping, not by changing the code; none was compiled or benchmarked.

## Corrections (2026-10-18)

- An earlier version said the `finalize` passes had no direct test. That was wrong: `finalize.rs` has its own test modules, and 71 of the 79 functions are named there. The figures above are corrected.
- The reports counted `src/bin/jubarte.rs` as part of `src/`, but the binary is a separate crate. Six of the unpublished items (`sniff_package`, `EditTarget`, `slash_path`, `removed_error`, `Tracking`, `docx_stamp`) are used by it and so must stay public; they were kept public when the rest were narrowed.
- Follow-up done since: 38 unpublished items narrowed to `pub(crate)`, `Dom::attributes_iter` added, the `sha1_fingerprint` aliases and the three `finalize` wrappers removed, `comparer::finalize` and `comparer::tables` hidden and declared outside the version promise, and a snapshot test of every public body pass added (see the CHANGELOG and the git log).
