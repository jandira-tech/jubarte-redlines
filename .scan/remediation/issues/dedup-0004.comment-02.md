## Full TDD red/green plan (1/1)

# dedup-0004 remediation plan (TDD red/green)

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0004/` — the round whose
own evidence refs are `dedup0004-n1…n10`. Eight of the 22 findings in
`output/result.json` carry those refs; this plan and the patch cover those
eight. Material inherited from earlier rounds (`dedup0002-*` refs, the
WMF/Chicago/tab-leader/LaTeX/border/UTF-16-redaction/symlink/Strict-probe
findings) is out of this round's scope.

| # | Finding (title in `result.json`) | Vuln class |
|---|---|---|
| V1 | Rendering an HTML chunk can allocate gigabytes from a single colspan value (`dedup0004-n1-*`) | CWE-789 |
| V2 | Rendering a small DOCX table can exhaust host memory (`dedup0004-n2-*`) | CWE-789 |
| V3 | Repeated HTML chunk references bypass conversion memory budgets (`dedup0004-n3-*`) | CWE-789 |
| V4 | Aliased legacy DOC pieces expand small physical text into enormous logical stories (`dedup0004-n4-*`) | CWE-789 |
| V5 | Opaque DTD and top-level CDATA bypass package XML depth checks (`dedup0004-n6-*`) | CWE-674/CWE-789 |
| V6 | Processing renamed auxiliary XML bypasses XML depth limits (`dedup0004-n7-*`) | CWE-674/CWE-789 |
| V7 | ZIP footer and count interpretation mismatch bypasses entry-count admission (`dedup0004-n8-*`) | budget bypass |
| V8 | Metadata scrub leaves property parts under alternate names (`dedup0004-n10-*`) | incomplete sanitization |

Every code claim below was verified this session by reading the cited file
and line in this checkout, or by reading the zip 9.0.5 crate source in the
cargo registry. **No build or test was run** — this checkout forbids
concurrent Cargo processes, and the ask forbids running them; the patch was
authored by reading code and validated only with `git apply --check` (it
applies clean at the repository root, exit 0, 10 files, +713/−44).

---

## V1 + V3 — altChunk HTML colspan and repeated references

**Sources read:** `src/convert/altchunk.rs:15-44` (`expand` infallible, no
budget), `:469-491` (`end_table` writes one `<w:gridCol>` per summed span),
`:558-574` (`colspan` parsed as `usize`, `.max(1)` only);
`src/convert/mod.rs:789-798` (the one production `expand` call site, in
`with_layout`). Confirmed there is no other production caller
(`grep -rn "altchunk::expand|html_to_wml" src/` — only `mod.rs:791`).

**Failing tests (red):**
- `src/convert/altchunk.rs`, `mod regression_tests::a_huge_colspan_stops_at_the_table_column_budget`
  — asserts `html_to_wml("<table><tr><td colspan='1000000000'>A</td>…")`
  emits exactly `MAX_TABLE_COLUMNS` (`16_384`) `<w:gridCol>` elements and
  under 1 MB of markup, and that three accumulated spans of `16384` share the
  same cap. Red today: the single attribute emits 1,000,000,000 gridCols
  (~21 GB) — the red run is an OOM/abort, which is the vulnerability.
- `src/convert/altchunk.rs`, `mod regression_tests::repeated_chunk_references_are_bounded_by_the_expansion_budget`
  — 50 references to one `<p>Hello</p>` chunk under a 1,000-byte expansion
  budget must refuse with `altChunk references expand past 1000 bytes…`;
  under a 1,000,000-byte budget all 50 still expand in order. Red today:
  `expand` returns `String` and never refuses.

**Minimal source change:** `expand` becomes fallible
(`Result<String, String>`) by delegating to a private `expand_with_budget`
that charges every converted chunk against `MAX_EXPANDED_WML_BYTES`
(64 MiB, the `InputLimits::default` package budget) **before appending**;
`colspan` gains `.min(super::MAX_TABLE_COLUMNS)`; `end_table`'s row sum
becomes a `saturating_add` fold clamped to the same budget (no `usize` wrap
on wasm32). `with_layout` maps the refusal to
`ConvertError::OpenPackage` (`src/convert/mod.rs:791`).

**Existing behaviour that must keep passing:** every existing `expand` test
in `src/convert/altchunk.rs` (updated only to `.unwrap()` the `Result`):
`an_alt_chunk_is_replaced_by_its_paragraphs`,
`expansion_keeps_surrounding_content_and_chunk_order`,
`expansion_without_chunks_does_not_load_any_parts`,
`missing_id_does_not_load_an_unrelated_part`,
`memory_story_chunk_relationships_expand_exactly…`,
`valid_html_declarations…` (unterminated chunk), and
`invalid_or_zero_colspan_falls_back_to_one_column` (span `"0"`/`"invalid"`
still yields one column); `tests/convert_docx_to_pdf.rs` and the corpus
gates over real altChunk documents (the croatian_regulations fixture).

**Deviation from the finding's remediation, on purpose:** the finding asks
for per-refusal of *individual* huge colspans. This patch clamps the span to
the renderer's shared table budget instead of refusing, mirroring the
engine's existing treatment of pathological table numerics
(`clamp_to_word_max_width`, `src/convert/mod.rs:9870-9894`, clamps to Word's
22-inch rule rather than refusing). A refusal channel does not exist in
`html_to_wml`; the cumulative-budget refusal (which does exist now at
`expand`) is what bounds total work, and the clamp bounds each table's grid.

---

## V2 — DOCX `w:gridSpan` / `w:gridBefore` / `w:gridAfter`

**Sources read:** `src/convert/mod.rs:14204-14216` (`cell_span` parses
`usize`, `.max(1)` only), `:14340-14353` (`row_grid_skip`, `.filter(*n > 0)`
only), `:13210-13216` (gridBefore push, unclamped), `:13466` (cell span
consumed; `grid_at += colspan`), `:13555-13561` (gridAfter push),
`:13600-13616` (row occupancy `sum()` sizes `vec![f32]`/`resize`),
`:14596-14648` (`resolve_table_merges`: `open.resize(col + span)`,
`col += span` — plain `+`, wraps on wasm32). `stretch_stale_spans`
(`:14172-14200`) only widens spans inside `cols.len()`, so it cannot
re-amplify after the clamp.

**Failing tests (red):**
- `src/convert/mod.rs`, `mod table_tests::a_huge_grid_span_stops_at_the_table_column_budget`
  — a 3-line table with `gridSpan=1000000000` and `gridBefore=999999999`
  must lay out with `cols.len() == MAX_TABLE_COLUMNS + 1` (the budgeted span
  plus the following cell's own column), `rows[0][0].colspan ==
  MAX_TABLE_COLUMNS`, and the skip cell `rows[1][0].colspan ==
  MAX_TABLE_COLUMNS`. Red today: `vec![each; 1_000_000_000]` ≈ 4 GB — the
  red run is an OOM/abort (nextest isolates it to one process).
- `src/convert/mod.rs`, `mod table_tests::a_grid_span_at_the_exact_budget_is_kept`
  — `gridSpan=16384` keeps exactly `MAX_TABLE_COLUMNS` columns: the budget
  is inclusive, guarding against an off-by-one that would clip a legitimate
  boundary table.

**Minimal source change:** `pub(crate) const MAX_TABLE_COLUMNS: usize =
16_384` beside `cell_span` (Word's own tables cap at 63 columns, 16,384 is
2⁶× that headroom, `f32` widths at the budget are 64 KiB); `cell_span` and
`row_grid_skip` clamp each parsed span to it; the two skip push sites and
the cell site clamp to the row's remaining headroom
(`MAX.saturating_sub(grid_at).max(1)`, so a row's claim is bounded by the
budget plus one column per physical cell — cells are bounded by part bytes);
`resolve_table_merges` uses `saturating_add` for `col + span` and the
resize, closing the wasm32 wrap.

**Existing behaviour that must keep passing:** `mod table_tests`
(`nested_table_rows_are_not_flattened` and the merge/skip tests, including
the `gridSpan="2"`/`gridBefore='1'` cases at `src/convert/mod.rs:44954`,
`62601-62605`, `65341`), the 91-corpus-document stale-span behavior
(`stretch_stale_spans` comment at `:13596-13599`), and every corpus table
rendering gate — no Word-valid document can reach a quarter of the budget.

**Deviation, on purpose:** the finding asks for a "typed resource-limit
refusal". `table_block` returns `Block` infallibly across a 66k-line call
graph; threading `Result` through it is a refactor, not a minimal fix. The
clamp keeps rendering alive for every Word-valid document and bounds every
derived vector, which is the security property; the codebase's precedent
for over-wide tables is Word's 22-inch clamp, not a refusal.

---

## V4 — legacy `.doc` aliased pieces

**Sources read:** `src/legacy_doc.rs:1148-1226` (`main_text`: coverage check
requires contiguity from CP 0 but the decode loop iterates *every* piece
with `cp_start < ccp_text`, so overlapping aliases multiply output; each
iteration pushes a `StoryChar`); `:180-193` (`read` calls `main_text`
before any later table rejection — the finding's "expansion precedes later
checks"). `main_text` already returns `Result<_, LegacyDocError>`.

**Failing tests (red):**
- `src/legacy_doc.rs`, `mod tests::a_story_larger_than_its_budget_is_refused_before_it_materializes`
  — a piece table claiming `ccp_text = u32::MAX` is refused with "the main
  story claims 4294967295 characters; the limit is 10" before any character
  is materialized. Red today: it enters the loop (4.29 billion iterations).
- `src/legacy_doc.rs`, `mod tests::aliased_pieces_are_charged_per_decoded_character`
  — eleven pieces aliasing two physical characters describe 22 logical
  characters: within budget 22 they all decode (fast-save aliases stay
  legal), at budget 21 the refusal ("the piece table expands past 21
  characters") fires before the expansion. Red today: no budget exists.

**Minimal source change:** `const MAX_STORY_CHARS: usize = 64 * 1024 * 1024`
(64 Mi characters ≈ 100× any real `.doc` story; a physical stream bounds an
honest story); `main_text` delegates to `main_text_within(…, budget)` which
(1) refuses `ccp_text > budget` up front and (2) charges every decoded
character, refusing before pushing past the budget — aliases cannot expand a
few KB into gigabytes of `StoryChar` metadata.

**Existing behaviour that must keep passing:** `mod tests` in
`src/legacy_doc.rs` — `a_piece_past_the_stream_is_refused_not_truncated`,
`split_surrogate_pieces_pair_across_the_piece_boundary`,
`a_piece_table_that_does_not_cover_the_story_is_refused` (fast-save empty
pieces, surrogate pairs, coverage errors), `tests/legacy_doc.rs` fixtures,
and `doc_to_markdown`/`doc_to_docx` over real `.doc` files.

---

## V5 — opaque DTD / top-level CDATA vs. the depth gate

**Sources read:** `src/admission.rs:788-823` (`check_xml_depth`:
`Ok(_) => {}` lets `Event::DocType` and depth-0 `Event::CData` pass);
`src/xmllinq/parse.rs:539-544` (the permissive DOM's top-level `<!` skip
ends at the *first* `>`, so a `>` inside a quoted entity value or CDATA body
exposes the hidden markup as real elements), `:660-663` (recursive
`parse_element`); `src/xmllinq/parse.rs:919-975`
(`validate_xml_with_limits` already refuses both — "DTD declarations are
unsupported", "CDATA outside XML root" — so only the *tolerant* gate and the
DOM disagreed). quick-xml is 0.42 (`Cargo.toml:112`); both event variants
are already matched in this file.

**Failing tests (red):**
- `src/admission.rs`, `mod tests::a_dtd_or_top_level_cdata_part_is_refused_as_invalid_xml`
  — a part with `<!DOCTYPE a [<!ENTITY x "<a><a><a>">]>`, a bare
  `<!DOCTYPE a>`, or a top-level `<![CDATA[…]]>` is refused `InvalidXml`;
  in-root `<![CDATA[<a>]]>` still admits. Red today: all three pass
  admission and then mis-parse in the DOM.
- `src/xmllinq/parse.rs`, `mod opaque_construct_and_nesting_cap_tests::a_quoted_dtd_literal_hides_no_elements`
  and `::a_top_level_cdata_section_is_skipped_whole` — the hidden `<a>`
  markup never becomes elements; the document keeps exactly one root. Red
  today: the skip ends at the quoted `>` and the hidden tags parse.

**Minimal source change:** (a) `check_xml_depth` refuses `Event::DocType`
and depth-0 `Event::CData` with the checked gate's exact messages — the
tolerant and strict gates now agree; (b) the permissive DOM skips a top-level
`<![CDATA[` to its `]]>` and any other `<!` construct to the first `>` that
is neither inside a quoted string nor inside an open `[...]` internal subset
(new `Parser::index_of_unquoted_gt`), so even a directly-fed DOM cannot
mis-read hidden markup; (c) a `MAX_DOM_NESTING = 512` recursion guard in
`parse_element` (twice the admission depth; deeper markup is skipped without
nodes, parser position always advances) bounds the stack on every path that
never saw a checked gate.

**Existing behaviour that must keep passing:** `checked_xml_tests` and
`xml_lexical_boundary_contract_tests` in `src/xmllinq/parse.rs` (in-root
CDATA content and line-end normalization at `:1107-1113`), every admission
test in `src/admission.rs` (`a_plain_docx_is_admitted`,
`budgets_admit_their_exact_boundary`), and the M1.8 byte-identical
round-trip gate. Fixtures: no fixture part in this checkout contains
`<!DOCTYPE` (checked: `grep -rl "<!DOCTYPE" tests/fixtures/` empty, and the
first 4 KiB of every part across 40 sampled `.docx` fixtures).

---

## V6 — renamed auxiliary XML bypasses the XML budgets

**Sources read:** `src/admission.rs:531-553` (only `.xml`/`.rels` parts are
XML-checked; others are drained as bytes), `:556-570` (the main part is
checked whatever its name — so only *auxiliary* parts escape);
`src/document_comparer.rs:6318-6333` (`get_revisions` parses both
relationship-selected notes parts with the permissive DOM),
`:6461-6480` (`notes_part_names`: OPC relationships are authoritative and
legal under any name), `:4713-4750` (`pre_process_markup`), `:6675-6686`
(`parse_part_root` in `compare_documents_impl`);
`src/convert/mod.rs` sinks (3317, 3497, 7675/7689, 7786, 8527, 8700, 20795 —
theme, styles, comments/commentsExtended, headers/footers, numbering).

**Failing test (red):** `tests/input_admission.rs`,
`get_revisions_enforces_the_xml_budget_on_a_renamed_notes_part` — a package
whose footnotes relationship targets `word/notes.bin` (a name the admission
scan drains as bytes) containing 300-deep XML must make `get_revisions`
return a typed `INVALID_XML` refusal under the default comparer budget
(depth 256); a 3-deep renamed notes part must still read. Red today: the
deep part parses silently (and a deep-enough one aborts the process).

**Minimal source change, two layers:**
1. *Consumer gate where limits are in hand:* `get_revisions` runs
   `admission::check_part_xml(part, &xml, settings.input_limits)` (a new
   `pub(crate)` wrapper over the tolerant `check_xml_depth`) before parsing
   each notes part; refusal propagates as the comparer's typed `OpcError`
   (`crate::opc::refused`). The tolerant policy is used — **not**
   `try_parse_xdocument_with_limits` — because the strict gate refuses
   unquoted/valueless attributes that real WW-ism files carry
   (`checked_xml_keeps_refusing_attributes_that_are_not_xml`,
   `src/xmllinq/parse.rs:1002-1010`); the strict gate here would break
   documents that compare correctly today.
2. *Parser backstop for every other consumer:* the `MAX_DOM_NESTING` guard
   from V5 bounds the stack for every renamed part any consumer parses
   (`load_theme`, styles, comments, headers/footers, markdown reference
   documents, generated markup) — a deep renamed part can no longer
   terminate the host.

**Existing behaviour that must keep passing:** the entire comparer suite
(`tests/compare_mode.rs`, `compare_boundary_matrix.rs`, corpus redline
gates) — a `.xml`-named notes part was already checked by admission and is
now checked twice (idempotent); `tests/input_admission.rs`'s existing
admission refusals; `memory_story…` fixtures whose notes live under
conventional names.

**Residual, deliberately deferred:** renamed parts of *ordinary* depth still
parse without a per-part typed refusal on the render path (only `get_revisions`
refuses typed). Closing that fully means converting ~15 infallible sinks in
`src/convert/mod.rs` to a checked boundary — a refactor of a 66k-line file
that should not be hand-authored without a compiler in the loop. The stack
overflow (the crash this finding demonstrates) is closed for all consumers
by layer 2; the plan records the follow-up.

---

## V7 — ZIP footer/count mismatch

**Sources read:** `src/admission.rs:580-625` (`declared_entry_count` reads
the EOCD *total* field at offset 10) vs. zip 9.0.5
`src/read/mod.rs:50-90` (`CentralDirectoryInfo` takes
`number_of_files_on_this_disk`, the EOCD *per-disk* field at offset 8, for
ZIP32; ZIP32 has no per-disk/total consistency check) — the mismatch is
real and was verified in the registry source
(`~/.cargo/registry/src/…/zip-9.0.5/src/read/mod.rs`). `admit()` at
`:448-456` only checks `archive.len() < declared` (duplicate direction);
`src/opc/mod.rs:282-293` (`read_package`) never caps the opened count.

**Failing tests (red):**
- `src/admission.rs`, `mod tests::a_one_sided_footer_cannot_undercount_past_the_entry_budget`
  — a 4-entry package whose EOCD total field is rewritten to 3 (per-disk
  left at 4) with `max_entries = 3` must refuse `InputLimit`, "4 ZIP
  entries; the limit is 3". Red today: `declared = 3 ≤ 3` passes preindex,
  `4 < 3` is false, and admission succeeds on all 4 entries.
- `src/opc/mod.rs`, `mod tests::a_one_sided_footer_cannot_undercount_past_the_entry_budget`
  — the same one-sided footer under `PartFs::open_within` with
  `max_entries = 3` refuses `INPUT_LIMIT`/“4 ZIP entries”; with
  `max_entries = 4` the same bytes still open (the cap is on the actual
  count, not the footer's word).

**Minimal source change:** both `admit()` (after `open_archive_unchecked`)
and `read_package()` (after `open_archive`) enforce
`archive.len() > limits.max_entries → input_over_entries(...)`, i.e. the
budget on the count actually indexed — the defense-in-depth the finding
itself names ("retain a post-open active archive.len cap"). The zip index
allocation for a few over-budget entries is trivial; what the cap removes is
the admission bypass and the unbounded per-entry work that followed.

**Existing behaviour that must keep passing:** every existing
`tests/input_admission.rs` budget test (`every_package_entry_point_refuses_a_crowd_of_entries`
still refuses 10,010-entry packages at preindex — faster than the post-open
cap), `admission_accepts_a_package_with_prepended_data`, the
`zip_names`/dual-header/Unicode-path admission suite, and `PartFs::open`
over every fixture (a valid package's indexed count equals its declared
count, so the new cap never fires).

**Residual, deliberately deferred:** rewriting `declared_entry_count` to
reproduce zip 9.0.5's exact EOCD/ZIP64 selection (comment-skip validation,
ZIP64 size/offset sentinels) is upstream-behavior reimplementation work;
with the actual-count cap the budget can no longer be bypassed by any
selection disagreement, which is the reachable vulnerability. The
duplicate-detection false-refusal for a false EOCD in a comment (a valid
file wrongly refused `DUPLICATE_PART` today when the fake count under-reads)
pre-exists and is out of scope.

---

## V8 — scrub misses relationship-named property parts

**Sources read:** `src/scrub.rs:39-43` (the three conventional `docProps`
paths), `:357-385` (`scrub_properties` loads those constants; a relocated
custom part's relationship and content-type override are left behind);
`src/opc/mod.rs:859` (`package_relationships()`), `:673-683`
(`resolve_rel_target`), and the production precedent
`src/document_comparer.rs:6181-6189` (`custom_props_part` resolves the
custom-properties part through the package relationship exactly this way).
The `Relationship` fields (`rel_type`, `target`, `target_mode`) are public
(`oxml-opc-0.14.2/src/relationship.rs`).

**Failing test (red):** `tests/scrub_and_redact.rs`,
`default_scrub_removes_properties_under_relationship_named_parts` — the
`identifying_document` payload relocated to `metadata/deal-{core,app,custom}.xml`
with package relationships pointing at them: after `scrub`, "Jane Secret",
"Secret Corp", the revision and dates are gone from the bytes, the relocated
custom part is removed with its relationship and content-type override, and
the intentional `dc:title`/`Application` survive; the package stays
`assert_word_valid_package`. Red today: nothing is removed (the conventional
paths hold different, absent parts) and scrub returns success.

**Minimal source change:** `scrub_properties` iterates
`all_property_parts(pkg, REL, conventional)` — the conventional name plus
every distinct internal package-relationship target of the core-,
extended- and custom-properties relationship types (`CORE_REL`, `APP_REL`,
`CUSTOM_REL`) — sanitizing each core/app part it finds and removing each
custom part that exists plus its override, then dropping the
custom-properties relationships unconditionally (a relationship to a part
the scrub just removed must not survive). External targets are skipped.

**Existing behaviour that must keep passing:**
`default_scrub_removes_authors_rsids_properties_and_comments` and
`an_alias_alone_renames_comment_authors_and_people_and_keeps_the_rest` in
`tests/scrub_and_redact.rs` (conventional names still scrub; the untouched-
options case still leaves properties alone),
`scrub_removes_the_author_and_rsids_everywhere`, `tests/scrub_cli.rs`, and
the `introduced()` no-new-findings guard inside `scrub` itself.

---

## What this patch does not do (honest limits)

**Second-pair-of-eyes corrections (this round's review, applied to the
patch):** the admission one-sided-footer test now builds a 4-entry package
(`docx_with(&[])` is 3 entries, so nothing could exceed `max_entries: 3`);
`index_of_unquoted_gt` is internal-subset aware (a DTD's `<!ENTITY …>` ends
with an unquoted `>` inside `[...]`; stopping there left `]>` to break the
document loop, so the test's sole root never parsed); the DOM tests expect
`XName::get("document", "")` (the permissive parser stores an unbound
`w:document` under its local name) and 0-based `depth_of` values (1 for a
lone root, 100 for a 100-deep chain).

- No build, test, clippy or `--help` smoke run was executed (forbidden in
  this ask; concurrent Cargo in this checkout is banned). The patch was
  validated with `git apply --check` only. Before landing, run per AGENTS.md:
  `cargo fmt`, `cargo clippy --lib --tests -- -D warnings`, then the touched
  suites: `cargo nextest run -E 'test(a_huge_) or test(a_grid_span) or
  test(repeated_chunk) or test(a_story_larger) or test(aliased_pieces) or
  test(a_dtd_or_top) or test(a_one_sided) or test(a_quoted_dtd) or
  test(a_top_level_cdata) or test(unadmitted_xml) or
  test(get_revisions_enforces) or test(default_scrub_removes_properties)'`,
  plus the regression files named in each section
  (`tests/input_admission.rs`, `tests/scrub_and_redact.rs`,
  `tests/scrub_cli.rs`, and the in-crate modules `table_tests`,
  `regression_tests` (altchunk), `checked_xml_tests`, `legacy_doc::tests`).
- V2/V6 render-path consumers keep permissive parsing for renamed parts of
  ordinary depth; only the stack-overflow vector is closed there (see V6's
  residual).
- The 14 findings without `dedup0004-*` evidence refs (earlier rounds'
  material: WMF text retention, Chicago numbering, tab leaders, LaTeX
  nesting, border loops, UTF-16 redaction residue, staging symlinks,
  Markdown drawing cycles, Strict-probe headers, `cyclic Markdown
  dependencies`) are not addressed here — they belong to their own rounds.
- The remediation's preferred *typed refusal* for table spans (V2) and
  individual colspan (V1) is implemented as a Word-precedented clamp; the
  refusal channel exists only where a `Result` was already in hand
  (altChunk expansion, legacy story budget, notes gate, entry cap).
