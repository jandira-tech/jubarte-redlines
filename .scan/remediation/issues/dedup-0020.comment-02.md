## Full TDD red/green plan (1/1)

# dedup-0020 remediation plan

Round: `dedup-0020` (claimed workers `b88710a6-…`, `0ffd62ef-…`), one round of
the serial dedup. Inputs read: `.scan/artifacts/deep_discovery/dedup/dedup-0020/prompt.md`,
`output/result.json` (16 MB; findings extracted with jq), `output/threatmodel.md`
(scanled for concrete src/ anchors), and the repository sources the findings
point at. This round's own contributions are the findings whose evidence refs
carry the `dedup0020-*` prefix; all 17 of them are covered below.

Patch: `.scan/remediation/patches/dedup-0020.patch` (applies with
`git apply` from the repository root; verified here with
`git apply --check /tmp/dedup0020-fixed.patch` → exit 0, all 16 files checked).
**No build, test, or format command was run** — AGENTS.md forbids concurrent
Cargo in this checkout, so the patch was authored by reading code; every test
below is written but *not executed*. Commands to run after applying (in order,
from the repository root, with no other Cargo running):

```
cargo fmt --check            # or cargo fmt for the touched files
cargo clippy --all-targets -- -D warnings
cargo nextest run -E 'test(a_differential_empty_tag_shape_builds_a_bounded_dom) or test(bare_ampersand_runs_decode_without_rescanning_the_suffix) or test(a_false_eocd_in_the_comment_cannot_hide_the_real_entry_count) or test(identical_duplicate_repair_stays_linear_over_many_ids) or test(a_relationship_target_resolves_with_exactly_one_decode) or test(deeply_nested_field_results_read_in_constant_work_per_character) or test(an_aliased_piece_table_past_the_story_budget_is_refused_before_expansion) or test(a_chicago_counter_past_the_cap_marks_at_the_cap) or test(an_ilvl_past_every_defined_level_resolves_over_the_defined_keys) or test(a_level_defined_past_the_supported_domain_is_not_materialized) or test(an_absurd_span_or_grid_skip_cannot_size_the_grid) or test(an_absurd_html_colspan_cannot_size_the_generated_grid) or test(a_long_bare_ampersand_run_decodes_in_linear_work) or test(a_short_text_record_cannot_read_the_next_records_bytes) or test(a_short_polygon_record_cannot_consume_the_next_records_points) or test(an_oversized_opening_tag_stops_the_probe_within_its_head_budget) or test(an_imported_dependency_cycle_is_refused_not_recursed) or test(a_shared_dependency_still_materializes_for_every_reference) or test(the_font_index_stages_without_following_a_preplanted_link)'
# behaviour that must keep passing (existing suites that the change touches):
cargo nextest run -E 'test(chicago) or test(numbering) or test(table) or test(wmf) or test(rel_targets) or test(repair) or test(piece_table) or test(field) or test(strict) or test(drawing)'
uv run --group dev pytest jubarte-python/tests/test_cli.py   # python staging test
node --test jubarte-wasm/cli/test/cli.test.mjs               # node staging test
```

Each section below names: the finding (aggregate index in `result.json` and
title), the failing test that demonstrates it (file, test name, what it
asserts, and *how it fails before the fix* — assert, crash, or timeout), the
minimal source change, and the existing behaviour that must keep passing.

---

## 1. [0] + [26] Table spans size layout vectors from authored XML

Aggregate findings 0 (`resource-exhaustion.table-grid-span`) and 26
(`resource-exhaustion.document-table-span`), incl. refs `dedup0020-a0-*` and
`dedup0020-b0-*`.

**Defect.** `w:gridSpan`/`w:gridBefore`/`w:gridAfter` (and imported HTML
`colspan`) parse as unbounded `usize`; `table_block` sums a row's spans into
`occupancy` and sizes `cols` (`vec![each; occupancy]`), `column_prefs`
(`vec![..; grid.len()]`) and `resolve_table_merges`'s `open.resize(col + span)`
from it. A ten-digit span requests gigabytes (`src/convert/mod.rs:13601-13618`,
`:14204-14223`, `:14340-14353`, `:14637-14644`; `src/convert/altchunk.rs:562`).

**Failing test (red = allocation abort/OOM).**
`src/convert/mod.rs`, `mod table_tests`:
`an_absurd_span_or_grid_skip_cannot_size_the_grid` — builds a one-cell table
with `w:gridSpan w:val='1000000000'` (and a `w:gridBefore` variant) through
`table_block` and asserts `cols.len()` and the row's summed `colspan` land
exactly at the clamp: 1024 for the gridSpan row, 1025 for the gridBefore row
(its skip clamps to the cap and its one real cell keeps a column of its own —
every cell always keeps at least one). Before the fix the first case executes
`vec![each; 1_000_000_000]` (≈4 GB) and dies.
`src/convert/altchunk.rs`, `mod tests`:
`an_absurd_html_colspan_cannot_size_the_generated_grid` — `colspan='1000000000'`
through `html_to_wml`; asserts ≤ 1024 generated `<w:gridCol` and
`<w:gridSpan w:val="1024"/>`. Before the fix `end_table` writes a billion
`<w:gridCol>` elements (`src/convert/altchunk.rs:469-492`).

**Minimal change.** New `const MAX_GRID_COLUMNS: usize = 1024` and
`clamp_row_spans(&mut raw_rows)` in `src/convert/mod.rs`, called in
`table_block` immediately after `stretch_stale_spans` (before occupancy is
summed); the HTML importer clamps the parsed colspan with
`.min(super::MAX_GRID_COLUMNS)`. Per-cell span and per-row occupancy are capped
once, at the boundary every derived vector reads.

**Must keep passing.** `vmerge_gridspan_cell_covers_two_rows_and_cols`,
`content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins`
(`src/convert/mod.rs`), `tests/m4d_table.rs`, `tests/m38_table_alignment.rs` —
real merged/skipped tables (spans ≤ 1024) render identically; `stretch_stale_spans`
runs before the clamp as before.

## 2. [2] Malformed XML hides recursion from admission (DOM stack overflow)

Aggregate finding 2 (`uncontrolled-recursion.xml-parser-differential`, refs
`dedup0020-a6-*`).

**Defect.** Admission's tolerant scanner reads `<p a=//>` as a self-closed
tag; the permissive DOM parser treats any character after `=` as a quote, reads
it as a start tag, and unclosed start tags nest — one Rust stack frame per
tag, with no construction-depth bound (`src/xmllinq/parse.rs:574-584`,
`:660-663`).

**Failing test (red = stack-overflow abort).**
`src/xmllinq/parse.rs`, `mod tag_policy_tests`:
`a_differential_empty_tag_shape_builds_a_bounded_dom` — parses
`"<p a=//>".repeat(50_000)` with `parse_xdocument` and asserts the first-child
element chain is ≤ `MAX_DOM_DEPTH` (1024). Before the fix the parser recurses
50,000 deep and the test process dies of stack exhaustion.

**Minimal change.** `Parser` gains a `depth` field; the child-recursion site
guards on `MAX_DOM_DEPTH` (1024, four times admission's default XML depth) and,
past it, `skip_deep_content()` consumes the element's remaining content
textually (every branch advances; no nodes built) instead of recursing. The
tokenizer itself is unchanged, so every shape the parser accepted before it
still accepts — only construction depth is bounded.

**Must keep passing.** `issue650_infallible_parser_preserves_over_default_*`
(same module), `admit_accepts_unquoted_and_valueless_attributes_the_permissive_parser_reads`
(`tests/input_admission.rs:435`) — the permissive contract (never refuse, never
spin) and admit-vs-DOM equality for tolerant shapes are untouched. The
*other* half of the finding (making the two tokenizers agree, or admitting
`w:val=on`-style values differently) is **not** in this patch; the depth bound
removes the reachable stack exhaustion without touching admission semantics.

## 3. [3] Legacy DOC field stack is rescanned per character

Aggregate finding 3 (`algorithmic-complexity.legacy-doc-field-stack`, refs
`dedup0020-b8-legacy-fields`).

**Defect.** `Story::blocks` computes `fields.iter().any(|&code| code)` for
every character of the story (`src/legacy_doc.rs:1604`); begin/separator pairs
leave an all-false stack, so each later character scans all of it — quadratic
CPU on a small `.doc`.

**Failing test (red = timeout; correctness assert included).**
`src/legacy_doc.rs`, `mod tests`:
`deeply_nested_field_results_read_in_constant_work_per_character` — 10,000
`\u{13}\u{14}` pairs then 10,000 visible characters; asserts the paragraph text
is exactly the 10,000 characters. Before the fix this performs ~10⁸ boolean
scans (seconds; scales quadratically), after it is one comparison per
character. The deterministic assertion pins the *semantics*; the work claim is
by construction (a counter replaces the scan).

**Minimal change.** A `code_fields` counter maintained at begin/separator/end
(`saturating_add`/`saturating_sub` — the module denies plain arithmetic in
non-test builds); `in_code = code_fields > 0`. Identical truth value in every
state: separator only decrements when the top frame was true, end only when
the popped frame was true.

**Must keep passing.** `a_field_nested_in_a_field_code_shows_only_the_outer_result`
(nested-code suppression), the legacy fixtures in `tests/legacy_doc_convert.rs`
(headings/lists/table, `styles.doc` field result "Page check: first").

## 4. [4] Chicago numbering repeats a marker per unit of an authored counter

Aggregate finding 4 (`resource-exhaustion.chicago-number-label`, refs
`dedup0020-b5-chicago-*`).

**Defect.** `chicago_label` computes `MARKS[i % 4].repeat(i / 4 + 1)` with no
bound (`src/convert/mod.rs:6015-6019`); a list start/startOverride or section
page start near `u32::MAX` makes every consumer of the shared `format_num`
(list markers, section page labels, Markdown list views, endnote references)
allocate its count in marks.

**Failing test (red = ≈3 GiB `repeat` allocation).**
`src/convert/mod.rs`, `mod page_num_fmt_labels`:
`a_chicago_counter_past_the_cap_marks_at_the_cap` — asserts
`chicago_label(1024) == "§".repeat(256)` (at the cap, unchanged),
`chicago_label(1025) == "*".repeat(256)` and
`chicago_label(u32::MAX) == "‡".repeat(256)`. Before the fix the last case
tries to repeat a 3-byte marker 1,073,741,824 times.

**Minimal change.** `const MAX_MARKS: usize = 256` inside `chicago_label`;
the repeat count is `(i / 4 + 1).min(MAX_MARKS)`. One function bounds every
caller; no Word document carries a 256-mark label.

**Must keep passing.** `chicago_cycles_star_dagger_double_section`
(`chicago_label(1/2/5)`), `chicago_label(0)` in the sibling test, and the
numbering/page-label suites (`mod numbering_tests`, `tests/agent_text_view.rs`).

## 5. [8] WMF records read past their declared extent

Aggregate finding 8 (`resource-amplification.wmf-record-payload`, refs
`dedup0020-b6-wmf-*`).

**Defect.** `raster_wmf` validates `size*2` against the remaining file but
hands the whole `data` slice to every handler; polygon/polypolygon/polyline
counts and EXTTEXTOUT/TEXTOUT string counts then read bytes belonging to the
*following* records, which the outer loop revisits — overlapping consumption
(`src/convert/metafile.rs:477-687`).

**Failing tests (red = wrong outcome asserted).**
`src/convert/metafile.rs`, `mod wmf_object_tests`:
- `a_short_text_record_cannot_read_the_next_records_bytes` — a 14-byte
  EXTTEXTOUT record claiming a 60-byte string; asserts `raster_wmf` returns no
  retained texts. Before the fix the string is sliced out of the next record's
  bytes and retained (with thousands of such records, gigabytes of overlapping
  strings).
- `a_short_polygon_record_cannot_consume_the_next_records_points` — a POLYGON
  record claiming 100 points with none in its extent; asserts `raster_wmf` is
  `None`. Before the fix the points are read from the following record and the
  image rasterizes (`Some`).

**Minimal change.** One line of new state: `let rec = &data[..off + size2];`
after the declared-size check; every handler read (`read_u16/read_u32/read_i16`,
`WmfFont::parse`, the two `get(at..at + n)` string slices) moves from `data`
to `rec`. A read past the record's end is `None`, exactly as a read past the
file's end always was; a valid record never reads that far. Valid-WMF
behaviour is byte-identical; per-record work is bounded by declared bytes.

**Must keep passing.** `a_polypolygon_fills_with_the_brush_selected_by_zero_based_index`,
`a_rectangle_fills_with_the_brush_and_is_outlined_with_the_pen`,
`wmf_huge_record_size_terminates`, `emf_*` tests (EMF paths untouched),
`tests/strict01*` clipart rendering.

## 6. [9] Aliased legacy pieces expand small text into enormous stories

Aggregate finding 9 (`unbounded-allocation.legacy-logical-story`, refs
`dedup0020-a2-legacy-*`, `dedup0020-b7-legacy-*`).

**Defect.** `main_text` keeps one `StoryChar` per logical CP; pieces may all
point at one physical FC range, so thousands of pieces describe billions of
logical characters from a few KB of piece table (`src/legacy_doc.rs:1148-1226`).

**Failing test (red = wrong error).**
`src/legacy_doc.rs`, `mod tests`:
`an_aliased_piece_table_past_the_story_budget_is_refused_before_expansion` —
4,096 pieces each covering 1,000,000 CPs over the same ten bytes; asserts the
error mentions `the limit is`. Before the fix the refusal (if any) comes later
as "a text piece points past the WordDocument stream" — and with a large
enough stream the expansion allocates first.

**Minimal change.** `const MAX_MAIN_STORY_CHARS: u32 = 16 * 1024 * 1024` at
the top of `main_text`; a declared `ccp_text` past it is refused before the
piece table is walked. 16 Mi characters ≈ 32 MiB of UTF-16 story ≈ 256 MiB of
retained `StoryChar` metadata — the same 256 MiB figure as the DOCX admission
budget. Because the materialization loop never exceeds `ccp_text`, this bounds
every aliased expansion; legitimate fast-save aliases within the budget still
expand (positive cases exist in `a_surrogate_pair_spans_pieces…` and
`a_piece_table_that_does_not_cover_the_story_is_refused`).

**Must keep passing.** All existing `main_text` unit tests (surrogate pairs
split across pieces, empty pieces as no gap, coverage/decreasing refusals) and
the three `.doc` fixtures in `tests/legacy_doc_convert.rs`.

## 7. [10] ZIP footer and count interpretation mismatch bypasses entry admission

Aggregate finding 10 (`zip-entry-budget.eocd-selection-differential`, refs
`dedup0020-b10-*`).

**Defect.** `preindex_budget`/`declared_entry_count` read the *last*
EOCD-looking signature (`src/admission.rs:580-601`); the zip 9.0.5 reader
validates candidates and retries earlier footers (`zip-9.0.5/src/read/zip_archive.rs:178-241`,
`format/find_central_directory.rs:85-151`). A false EOCD appended past the real
one (count 1, cd-offset past itself) declares 1 entry while the reader indexes
the real count; neither `admit` nor `read_package` reconciled the actual
`archive.len()` against `max_entries` (only `len < declared` was checked,
`src/admission.rs:451`; `src/opc/mod.rs:282-289`).

**Failing test (red = admitted).**
`tests/input_admission.rs`:
`a_false_eocd_in_the_comment_cannot_hide_the_real_entry_count` — an 11-entry
package (via the file's `package()` helper) with a 22-byte false EOCD appended
(offsets: sig, disks, `1`/`1` counts, size 0, cd-offset `0xFFFF_FFFF`,
comment-length 0); `admit` under `max_entries: 10` must fail with
`11 ZIP entries … the limit is 10`. Before the fix `admit` succeeds (declared
1 ≤ 10, `11 < 1` false). The crafted footer is valid against zip 9.0.5's
actual fallback semantics, which were read in the vendored source.

**Minimal change.** After the archive opens, both `admit`
(`src/admission.rs:449`) and `read_package` (`src/opc/mod.rs:284`) refuse when
`archive.len() > limits.max_entries` (typed `INPUT_LIMIT` via the existing
`input_over_entries`). Legitimate packages index ≤ declared ≤ max_entries and
never hit it; the exact-duplicate `< declared` check keeps its own code.

**Must keep passing.** `document_open_bounds_opc_metadata_attributes`,
`admit_keeps_refusing_over_the_count_cap_and_duplicates_in_html_mode`,
`rel_targets…` (`src/opc/mod.rs`), every existing over-budget admission test in
`tests/input_admission.rs` (the pre-index refusals are unchanged — the new
check only adds a floor under the reader's own count). The fuller repair
(validate EOCD fields at the pre-index, ZIP64 trigger parity, raw duplicate
detection) is **not** in this patch; the post-open cap is the
defense-in-depth the finding's own remediation names.

## 8. [20] Markdown drawing cycles recurse without bound

Aggregate finding 20 (`uncontrolled-recursion.markdown-drawing-dependencies`,
refs `dedup0020-a1-*`).

**Defect.** `materialize` → `dep_part` follows `rel.part` indices
recursively with no visiting state; a one-node self-reference recurses
forever (`src/markdown/from_docx/drawing.rs:1881-1905`, `:1956-2008`).

**Failing tests (red = stack-overflow abort; green = refusal).**
`src/markdown/from_docx/drawing.rs`, `mod tests`:
- `an_imported_dependency_cycle_is_refused_not_recursed` — a `Drawing` whose
  `rels[0].part = 0` and `deps[0].rels[0].part = 0`; asserts `materialize`
  returns `None`. Before the fix the test process dies of stack exhaustion.
- `a_shared_dependency_still_materializes_for_every_reference` — two
  relationships to the same dep index; asserts `Some` and the part written.
  This guards the remove-on-exit of the visiting set (without it, the second
  reference would read as a cycle and break shared media).

**Minimal change.** `materialize` creates `active: HashSet<usize>`;
`dep_part` becomes a thin wrapper that inserts the index (returning `None` on
revisit — the typed refusal is the existing `None` materialize contract),
calls the old body as `write_dep_part`, and removes the index on every exit.
`name_for` and the byte-equality dedup are unchanged.

**Must keep passing.** The drawing round-trip suites
(`tests/md_roundtrip*.rs`, `src/markdown/from_docx/drawing.rs` capture tests)
— captured graphs are acyclic, so behaviour is identical; header/footer
materialization goes through the same `materialize` and is covered.

## 9. [21] Public Strict probe retains unbounded head

Aggregate finding 21 (`unbounded-decompression.strict-format-probe`, refs
`dedup0020-a3-strict-*`).

**Defect.** `main_part_is_strict` retains everything since the last `<`; an
unfinished root tag (or comment) keeps that at offset 0, so nothing is ever
drained, the whole part is retained and rescanned per 16 KiB chunk
(`src/strict_translation.rs:670-717`).

**Failing test (red = new API absent; property otherwise only observable as
memory/time).**
`src/strict_translation.rs`, `mod tests`:
`an_oversized_opening_tag_stops_the_probe_within_its_head_budget` — a
document.xml of `<w:document ` + 256 KiB of spaces; asserts
`main_part_is_strict_within(&pkg, 64 * 1024) == false` (probe stops inside
the budget) and the default-budget classification is unchanged (`false`: no
`>` at all). Before the fix `main_part_is_strict_within` does not exist
(compile-time red); the retained-state bound it enforces is the fix's whole
content.

**Minimal change.** `main_part_is_strict_within(bytes, max_head)` carries the
body; after each chunk append it returns `false` when `head.len() > max_head`.
`main_part_is_strict` delegates with `STRICT_PROBE_HEAD_BYTES = 1 MiB` — far
above any real root tag (the existing 100 KiB-comment test stays green) and
far below the whole-part retention the probe had.

**Must keep passing.** `a_long_preamble_does_not_hide_a_strict_root` (100 KiB
comment classifies under the 1 MiB default), the strict-rewrite and
zero-churn tests in the same module.

## 10. [27] Identical relationship-ID repair is quadratic

Aggregate finding 27 (`algorithmic-complexity.relationship-repair`, refs
`dedup0020-b9-rels-*`).

**Defect.** `drop_repeated_relationships` uses `repaired.contains(&item.id)`
inside the record loop — a linear scan of a growing `Vec` per duplicate
(`src/opc/mod.rs:116`).

**Failing test (red = timeout at N = 150,000; deterministic assertions pin
behaviour).**
`src/opc/mod.rs`, `mod tests`:
`identical_duplicate_repair_stays_linear_over_many_ids` — 150,000 distinct
Ids each repeated identically; asserts each Id reported once in first-seen
order, one copy kept, and conflicting duplicates still refused. Before the
fix this is ~1.1 × 10¹⁰ string comparisons (minutes); after, set membership.

**Minimal change.** `repaired_ids: HashSet<String>` beside the reporting
`Vec`; `repaired_ids.insert(item.id.clone())` gates the push. Order,
once-only reporting and the conflict refusal are bit-identical.

**Must keep passing.** `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`,
`rel_attributes_survive_round_trips_unescaped`, `validate`'s `DUPLICATE_RID`
reporting (`src/validate.rs:857`), `tests/validate_public.rs`.

## 11. [28] Numbering levels iterate the authored numeric domain

Aggregate finding 28 (`resource-exhaustion.numbering-level-range`, refs
`dedup0020-b2-ilvl-*`).

**Defect.** `resolve_ilvl` and `ensure_level` scan `(0..ilvl).rev()` —
billions of probes for an undefined high `ilvl`; a *defined* high level makes
`next_marker`'s `for level in 0..resolved` seed billions of counters
(`src/convert/mod.rs:5518-5582`, `:5610`, definition parse at `:6618-6645`
and `:6672-6681`).

**Failing tests (red = timeout; assertions exact).**
`src/convert/mod.rs`, `mod numbering_tests`:
- `an_ilvl_past_every_defined_level_resolves_over_the_defined_keys` — a
  level-0-only abstract, `next_marker("1", u32::MAX)`; asserts `"1."`.
  Before the fix `resolve_ilvl` counts down through ~4 × 10⁹ integers.
- `a_level_defined_past_the_supported_domain_is_not_materialized` — defines
  `w:lvl w:ilvl="4000000000"` and asks for it; asserts the level-0 marker
  `"1."`. Before the fix the defined level materializes and the counter loop
  runs 4 × 10⁹ times.

**Minimal change.** Both reverse scans become `keys().filter(|&&i| i <
ilvl).max()` over the actually defined levels — identical result (deepest
defined level below `ilvl`, else `ilvl`). Level *definitions* past
`MAX_NUMBERING_LEVEL = 1024` (Word writes 0..=8) are dropped at both parse
sites, so a marker never materializes for an out-of-domain level; paragraphs
naming it fall back to the deepest defined level, exactly as any undefined
level does.

**Must keep passing.** `a_label_naming_a_deeper_level_is_no_label_but_still_counts`,
`unused_parent_slot_shows_its_start_override`, `lvl_restart_names_the_level_that_restarts_it`,
`lvl_override_replaces_the_abstract_is_lgl` (all 0..=2 levels — unchanged).

## 12. [29] Entity decoding rescans the suffix; relationship targets decode twice

Aggregate finding 29 (`algorithmic-complexity.entity-delimiter-search`, refs
`dedup0020-a5-rels-*`).

**Defect.** (a) `unescape_xml_text` searches the whole remaining `Vec<char>`
per `&` (`src/xmllinq/parse.rs:771-788`); (b) `decode_entities` calls
`tail.find(';').filter(|&e| e <= 10)` — the scan runs before the rejection
(`src/convert/altchunk.rs:733`); (c) `PartFs::resolve_rel_target` unescapes
targets that oxml-opc 0.14.2 already decoded when parsing
(`src/opc/mod.rs:680`; confirmed in
`oxml-opc-0.14.2/src/relationship.rs:924-951`: `decoded_and_normalized_value`
on every attribute) — a second decode corrupts targets holding a literal
`&amp;` and re-runs the quadratic scan on bare ampersands.

**Failing tests.**
- `src/xmllinq/parse.rs`, `mod tag_policy_tests`:
  `bare_ampersand_runs_decode_without_rescanning_the_suffix` (red = timeout at
  100,000 `&`) — asserts a bare-ampersand run decodes to itself, and
  `x&amp;y`, `&#65;&#x42;`, `&#x10FFFF;` decode while `&nbsp;`, `a&b`, `a&`
  stay literal, exactly as before.
- `src/convert/altchunk.rs`, `mod tests`:
  `a_long_bare_ampersand_run_decodes_in_linear_work` (red = timeout) — same
  shape for the HTML decoder, plus real-entity assertions.
- `src/opc/mod.rs`, `mod tests`:
  `a_relationship_target_resolves_with_exactly_one_decode` (red = assert) — a
  `.rels` target written `media/a&amp;amp;b.png` decodes once to
  `media/a&amp;b.png` and must resolve to `word/media/a&amp;b.png`; before the
  fix the resolver returns `word/media/a&b.png`.

**Minimal change.** (a) the `;` search window is capped at 16 chars — longer
than every canonical reference (`&#4294967295;`, 12 chars, is the longest), so
every canonical decodable and every preserved case is byte-identical. One
deliberate edge changes: a numeric body longer than the window — only
reachable through zero padding or a leading `+` (`&#0000000000000065;`), which
`u32::from_str` accepts — now stays literal where it used to decode; Word
writes canonical references only; (b) the HTML decoder
searches `;` only within the first 11 bytes (`char_indices().take_while(|(i,_)|
*i <= 10)`), matching the old `e <= 10` filter without scanning past it;
(c) `PartFs::resolve_rel_target` drops its internal `unescape_xml_text` call
and forwards to the pure resolver — the one-decode contract its own round-trip
tests already assert.

**Must keep passing.** `migration_keeps_canonical_metadata_and_decodes_literal_entities_once`,
`rel_attributes_survive_round_trips_unescaped`,
`public_and_package_relationship_readers_agree` (whose targets carry
`?a=1&amp;b=2`), `a_class_rule_and_inline_bold_become_run_properties`
(`Title &amp; more`), `an_mht_chunk_yields_its_quoted_printable_html`.

## 13. [54] Release script puts the PyPI token in argv

Aggregate finding 54 (`credential-exposure.publish-token-argv`, refs
`dedup0020-a4-token-*`).

**Defect.** Step 11 runs `uv publish --token "$UV_PUBLISH_TOKEN" …`
(`scripts/release.sh:1069`); the credential sits in the child process's
command line, readable by any local account that can list processes.

**Check (scripted, no harness exists for release.sh).** Red before:
`grep -n 'uv publish --token' scripts/release.sh` → line 1069. Green after:
no match, while step 0's `UV_PUBLISH_TOKEN` presence check and the
credentials comment remain. Run as part of reviewing the release script.

**Minimal change.** `uv publish dist/pypi/*` — uv reads `UV_PUBLISH_TOKEN`
from the environment itself; the comment block is updated to say so. No other
behaviour of the step changes.

## 14. [57] Predictable plan/font-cache staging follows symlinks

Aggregate finding 57 (`symlink-following.plan-staging`, refs
`dedup0020-b1-*`).

**Defect.** The Python CLI writes `.{plan}.{pid}.partial` with a truncating
write (`jubarte-python/python/jubarte_redlines/__main__.py:408-414`); the
Node CLI the same (`jubarte-wasm/cli/bin/jubarte-redlines.mjs:420-427`); the
native font index writes `font-index.tsv.{pid}.tmp` via `fs::write`
(`src/convert/font.rs:3222-3227`). A preplanted symlink at the predictable
name redirects the truncating write into another user's file.

**Failing tests.**
- `jubarte-python/tests/test_cli.py`:
  `test_plan_recording_never_writes_through_a_preplanted_partial` (red =
  assert) — preplants `.plan.json.{pid}.partial` as a symlink to a victim
  (in-process `main()`, so the pid is the test's), records a plan, asserts the
  link is still a link, the victim still holds its text, the plan landed and
  no `.partial` the recording staged remains (the preplanted link itself
  stays, so the leftover check excludes it by name). Before the fix the
  victim's content becomes the plan JSON.
- `src/convert/font.rs`, `regression_tests`:
  `the_font_index_stages_without_following_a_preplanted_link` (red = assert) —
  preplants `font-index.tsv.{pid}.tmp` as a symlink, saves the index, asserts
  the victim untouched, the index written with its header, the link still a
  link.
- `jubarte-wasm/cli/test/cli.test.mjs`:
  `edit --plan records through exclusive unpredictable staging` — regression
  only (the child pid is not predictable from the test): records a plan,
  asserts it lands, the victim file beside it is untouched and no staging file
  remains. The Python in-process test covers the shared mechanism's red.

**Minimal change.** Python: `tempfile.mkstemp` (exclusive, unpredictable,
0600) in the plan's directory, `os.fdopen` write, `os.replace`; the staging
steps sit inside the existing `try`, so a directory that cannot stage raises
the same `CliError` a failed write always did. Node:
`fs.openSync(name, "wx")` with a `crypto.randomUUID()` suffix, `writeSync`,
`renameSync`; EEXIST retries a fresh name. Rust: `OpenOptions::create_new`
under a `RandomState`-salted name, `write_all` + `sync_all`, `rename`; a save
that cannot stage is skipped exactly as a failed one was (the next lookup
rescans).

**Must keep passing.** `test_cli.py`'s plan/symlink tests (link-to-plan
resolution, refused/dry-run keeping the plan, `--consume` semantics — the
`realpath` behaviour is untouched); `cli.test.mjs` `a bare --plan is
FILE_plan.json…` and `edit and add --plan record into one plan…`; the font
index round-trip tests (`src/convert/font.rs`).

---

## Documented, not patched in this round

Two findings from this round's contributions are **not** fixed by this patch;
both need design or infrastructure decisions a reading-only patch should not
guess:

- **[1] Renamed auxiliary XML bypasses XML depth limits**
  (`uncontrolled-recursion.unchecked-auxiliary-xml`, refs `dedup0020-b4-*`).
  The fix the finding itself names — one centralized checked XML-parse
  boundary keyed on *relationship use* rather than filename, covering
  `main_rel_part`/`part_xml_by_rel_kind` consumers in `src/convert/mod.rs`
  (theme, styles, numbering, comments/commentsExtended, notes, headers and
  footers), `document_comparer.rs` (`get_revisions` :6318-6328,
  `pre_process_markup` :4724-4750, styles/charts), `revision_processor.rs`
  (:3699-3794), `edit.rs` (:2154-2158, :5552-5563) and `convert/font_table.rs`
  — touches ~15 call sites whose error paths and InputLimits plumbing differ.
  The DOM-depth cap in this patch (finding [2]) removes the unbounded stack
  growth on every one of those routes, which lowers the finding from
  process-termination to bounded-memory parsing; the per-consumer typed
  refusal under `InputLimits` remains open.
- **[30] PR code runs on shared self-hosted runners**
  (`ci.untrusted-code-self-hosted-runner`, refs `dedup0020-b3-ci-*`).
  A repository patch cannot decide the runner topology: routing PR builds to
  GitHub-hosted/ephemeral runners and keeping the self-hosted pool for
  trusted refs (the remediation) is an owner infrastructure decision over
  `.github/workflows/{ci,adoption,docs,md-roundtrip}.yml` and
  `.github/actions/mbx/action.yml`. No code change is included.

## Verification status (honest)

- `git apply --check` on the produced patch from the repository root:
  **passed this session** (exit 0; 16 files).
- `python3 -m py_compile` on the edited `__main__.py` and `test_cli.py`, and
  `node --check` on the edited `.mjs` files: **passed this session**.
- Cargo build/clippy/tests, pytest, `node --test`: **not run** (forbidden in
  this checkout for this task). The Rust edits were authored by reading the
  sources cited above and cross-checking every changed expression against its
  original; they still need `cargo fmt`, `clippy -D warnings` and the test
  runs listed at the top once a Cargo slot is free.
- Second-review pass (this round): every hunk was re-checked against the
  repository sources, `git apply --check` re-run on the corrected patch
  (pass), the corrected Python staging reconstructed and `py_compile`d plus
  behavior-checked against a preplanted symlink (pass), and the zip 9.0.5
  EOCD-fallback and oxml-opc 0.14.2 attribute-decode claims verified in the
  registry sources. Four defects were corrected in place: the gridBefore
  variant of the table test asserted `<= 1024` where the clamped row lays out
  at 1025 (one real cell after the capped skip keeps its own column) — now
  exact expectations; the Python staging test's "no `.partial` remains"
  glob also matched the preplanted link, which correctly stays — now excluded
  by name; `tempfile.mkstemp` moved inside the `try` so an unstagedable plan
  directory raises the CLI's `CliError` rather than a raw `OSError`; and the
  entity-window comment no longer claims byte-identical output for padded
  numeric references past the window (they stay literal — a documented edge).
