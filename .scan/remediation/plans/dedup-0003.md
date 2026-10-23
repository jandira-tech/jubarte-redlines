# dedup-0003 remediation plan (TDD red/green)

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0003/` (reducer round
`dedup-0003`, claimed worker `669ab03c-dc29-4698-a3e5-ef63cb53fa4d` per
`prompt.md`). This plan covers the six findings this round contributed
(source refs `669ab03c:0`–`669ab03c:5` in `output/result.json`
`provenance.sourceFindingIds`); the other thirteen findings in the result
carry only earlier rounds' refs and belong to those rounds' folders.

Patch: `.scan/remediation/patches/dedup-0003.patch` (unified diff, applies
with `git apply` from the repository root — verified with
`git apply --check .scan/remediation/patches/dedup-0003.patch`, which
passed for all eight files; re-verified by the second pair of eyes after
it added the top-level CDATA subcase of finding 1).

**Not run (forbidden for this task): cargo/mbx/rustc builds and tests.**
Red states below were established by reading the current code paths the
findings cite; the applier must run the red/green cycle. Per AGENTS.md:
`cargo nextest run -E 'test(PATTERN)'` for each test below, then
formatting, `clippy -D warnings`, the touched suites under coverage, and a
`jubarte --help` smoke test.

Coverage of the six findings:

| Finding (title in result.json) | Worker ref | Status |
| --- | --- | --- |
| A DTD literal bypasses package XML depth checks | 669ab03c:3 | fixed + tests (incl. the mapped raw finding csf_d22452ac's top-level CDATA subcase) |
| Rendering an HTML chunk can allocate gigabytes from a single colspan value | 669ab03c:1 | fixed + tests |
| Repeated HTML chunk references bypass conversion memory budgets | 669ab03c:2 | fixed + tests |
| WMF record-boundary failures amplify polygon work and retained text | 669ab03c:4 | fixed + tests |
| Metadata scrub leaves property parts under alternate names | 669ab03c:5 | fixed + tests |
| Processing renamed auxiliary XML bypasses XML depth limits | 669ab03c:0 | fixed at the confirmed consumers + tests (scope note in §6) |

---

## 1. DTD literal bypasses package XML depth checks (669ab03c:3)

**Root cause** (verified in code): `src/admission.rs:798`–`822`
`check_xml_depth` matches `Ok(_) => {}`, so a `DOCTYPE` is one ignored
event during admission, while the checked gate
`validate_xml_with_limits` (`src/xmllinq/parse.rs:954`) refuses
`Event::DocType`. Independently, the permissive parser
`src/xmllinq/parse.rs:539`–`544` skips a `<!` declaration only to the
first `>` — which can sit inside a quoted internal-subset literal — and
then parses the rest of the literal as real elements through the
recursive `parse_element` (`src/xmllinq/parse.rs:660`) with no depth
budget. `WmlDocument::from_bytes` admits (`src/wml_document.rs:50`) then
lazily parses the main part unchecked (`src/wml_document.rs:94`).

**Top-level CDATA subcase** (named by the mapped raw finding
`csf_d22452ac5ec901578d340334`, "Opaque DTD and top-level CDATA bypass
package XML depth checks": *"A DTD-only refusal does not fix the
top-level CDATA subcase"*): the checked gate also refuses CDATA outside
the root (`src/xmllinq/parse.rs:964`), but `check_xml_depth` ignores
`Event::CData` the same way it ignored `Event::DocType`, and the
permissive `<!` skipper treats a CDATA body's first `[` as an internal
subset — so a body holding `]` before a `>` (e.g.
`<![CDATA[a]><d><d>…`) ends the "declaration" at that `>` and the rest
of the body parses as top-level elements with no depth budget. The fix
mirrors the DTD pair exactly: admission refuses CDATA outside the root
(in-root CDATA stays admitted — Word-valid), and `parse_document` gets
an explicit `<![CDATA[` branch that consumes the complete `…]]>`
grammar, like `parse_element` already does in-root.

**Failing tests (red)**

- `src/xmllinq/parse.rs`, `checked_xml_tests::
  a_dtd_internal_subset_is_skipped_whole_not_to_the_first_gt` — parses
  `<!DOCTYPE w:document [ <!ENTITY deep "<a><b><c>…"> ]><w:document/>`
  and asserts the document's root is the real `w:document` and that the
  document node holds exactly one element child. Red today: the parser
  resumes inside the literal, the root becomes the literal's `<b>` and
  `w:document` is never reached. Also asserts `validate_xml` refuses the
  DTD (already green, protects the checked gate's side).
- `src/admission.rs`, `tests::a_dtd_in_the_main_part_is_refused_like_the_checked_gate`
  — `admit` on a minimal package whose main part carries the same DTD
  must refuse with `AdmissionErrorKind::InvalidXml`, and
  `check_xml_depth` itself must return the
  `DTD declarations are unsupported` error. Red today: the package
  admits.
- `tests/input_admission.rs`, `wml_document_refuses_a_dtd_main_part` —
  end-to-end: `WmlDocument::from_bytes` on the same package must fail
  with `INVALID_XML`. Red today: it opens.
- `src/xmllinq/parse.rs`, `checked_xml_tests::
  a_top_level_cdata_is_skipped_whole_even_with_brackets_in_its_body` —
  parses `<![CDATA[a]><d>deep markup</d>]]><w:document/>` and asserts
  the root is the real `w:document` with exactly one element child, and
  that `validate_xml` refuses it (the gate's CDATA side, already green).
  Red today: the `<!` skip stops at the `>` inside the body, `<d>`
  becomes the root.
- `src/admission.rs`, `tests::
  a_top_level_cdata_in_the_main_part_is_refused_like_the_checked_gate`
  — `admit` on a package whose main part opens with the same CDATA must
  refuse with `AdmissionErrorKind::InvalidXml`, `check_xml_depth`
  itself must return the `CDATA outside XML root` error, and a control
  package whose CDATA sits inside the root still admits. Red today: the
  package admits.
- `tests/input_admission.rs`,
  `wml_document_refuses_a_top_level_cdata_main_part` — end-to-end:
  `WmlDocument::from_bytes` must fail with `INVALID_XML`. Red today: it
  opens.

**Minimal source change**

- `src/admission.rs` `check_xml_depth`: new arm
  `Ok(Event::DocType(_)) => return Err(invalid("DTD declarations are unsupported…"))`
  (same wording as the checked gate) plus
  `Ok(Event::CData(_)) if depth == 0 => return Err(invalid("CDATA outside XML root…"))`
  (also the gate's wording), and the fn becomes `pub(crate)`
  for finding 6 below.
- `src/xmllinq/parse.rs` `parse_document`: a new `<![CDATA[` branch
  skips to `]]>` whole (the same call `parse_element` uses in-root);
  the `<!` branch now scans the complete declaration — quote-aware
  (`'`/`"`) and internal-subset-aware (`[… ]`) — so the closing `>` is
  the one that really ends the declaration; no literal markup reaches
  `parse_element`.

**Existing behaviour that must keep passing**

- `checked_xml_rejects_invalid_or_unbounded_structure` (parse.rs) already
  refuses `<!DOCTYPE a><a/>` through `validate_xml`.
- `deep_or_malformed_xml_is_refused`,
  `mismatched_unclosed_and_stray_end_tags_are_refused`,
  `pathological_tags_and_namespaces_are_refused_at_admission`,
  `a_main_part_without_an_xml_extension_is_still_depth_checked`
  (admission.rs) — depth/malformedness policy unchanged.
- `admit_accepts_unquoted_and_valueless_attributes_the_permissive_parser_reads`
  (tests/input_admission.rs) — the tolerant attribute policy is
  untouched (this change only adds the DocType arm).
- Word fidelity: Word never writes DTDs or CDATA outside the root into
  OOXML parts (in-root CDATA stays admitted and stays text), so refusing
  them costs no Word-valid document; `validate_xml` already refused them
  on every edit path (`checked_xml_rejects_invalid_or_unbounded_structure`
  covers both `<!DOCTYPE a><a/>` and `<![CDATA[x]]><a/>`).

## 2. HTML chunk colspan → billions of gridCol (669ab03c:1)

**Root cause** (verified): `src/convert/altchunk.rs` `Builder::start`
"td"/"th" accepted any `usize` colspan (`.max(1)` only), and `end_table`
summed spans with `.sum::<usize>()` and emitted one `<w:gridCol>` per
column (`for _ in 0..cols`), so `colspan="4294967295"` produced billions
of appends before any parser/output budget could act.

**Failing test (red)**

- `src/convert/altchunk.rs`, `regression_tests::
  a_colspan_beyond_words_column_limit_clamps_the_grid` — for spans
  `"64"`, `"1000"`, `"4294967295"` the generated WML must hold exactly
  `MAX_GRID_COLUMNS` (63) `w:gridCol` elements and a
  `<w:gridSpan w:val="63"/>`; a row of 200 authored cells must keep 200
  `w:tc` cells but a 63-column grid. Red today: `colspan='64'` yields 64
  gridCols (the assertion fails fast on the first loop entry; the huge
  values are the demonstrated amplification and are covered once green).

**Minimal source change**

- `src/convert/altchunk.rs`: `const MAX_GRID_COLUMNS: usize = 63`
  (Word's own maximum number of table columns), colspan clamped with
  `.clamp(1, MAX_GRID_COLUMNS)`, and the per-row span sum computed with
  `fold(0usize, saturating_add)` then `.clamp(1, MAX_GRID_COLUMNS)` —
  the sum can no longer overflow and the emitted grid is bounded.

**Existing behaviour that must keep passing**

- `invalid_or_zero_colspan_falls_back_to_one_column`,
  `html_tables_keep_spans_empty_cells_and_header_formatting`
  (colspan 2 → 2 gridCols, `w:tcW w:w="9360"` with gridSpan 2),
  `omitted_cell_and_row_end_tags_keep_the_table`,
  `ragged_authored_rows_share_one_grid_without_inventing_filler_cells_or_reordering_payload`
  — all use spans ≤ 63 and are byte-identical before/after.
- Word fidelity: a wider table is not Word-valid anyway (63-column
  limit); clamping only affects markup Word itself could not open.

## 3. Repeated chunk references bypass conversion budgets (669ab03c:2)

**Root cause** (verified): `altchunk::expand` converts each
`w:altChunk` reference independently and appends every conversion to one
`String` with no cumulative budget; a small admitted package can
reference one chunk N times and demand N conversions' worth of output
and work.

**Failing test (red)**

- `src/convert/altchunk.rs`, `regression_tests::
  repeated_chunk_references_stop_at_the_expansion_budget` —
  `expand_within(xml, load, 20)` on two references to one chunk: the
  first conversion (~200 bytes of WML for `<p>Hello</p>`) already spends
  the budget, so the second reference contributes nothing, the document
  keeps its own markup (`<w:body>`/`<w:sectPr/>`). Red form: the new API
  does not exist today (compile-fail red); behaviourally, `expand`
  today appends every reference.

**Minimal source change**

- `src/convert/altchunk.rs`: `expand` delegates to a new
  `expand_within(xml, load, budget)`; the default budget is
  `InputLimits::default().max_uncompressed_bytes` (256 MiB — the package
  admission aggregate). Once `spent > budget`, further chunk references
  are consumed (tags skipped, rest of the document kept) but not
  converted or appended, so neither `load` nor `html_to_wml` runs again.

**Existing behaviour that must keep passing**

- `an_alt_chunk_is_replaced_by_its_paragraphs`,
  `expansion_keeps_surrounding_content_and_chunk_order`,
  `expansion_without_chunks_does_not_load_any_parts`,
  `missing_id_does_not_load_an_unrelated_part`,
  `memory_story_chunk_relationships_expand_exactly_without_mutating_package_or_source`,
  `valid_html_declarations_headings_and_self_closed_spans_keep_exact_source_formats`
  (unterminated chunk case) — single/missing references are far below
  the default budget and byte-identical.
- New control `repeated_references_under_the_default_budget_all_expand`
  — the budget is not a cache: two references under the default budget
  both expand.
- Known bound (documented, accepted): a *single* conversion of one
  `max_part_bytes`-sized HTML part still amplifies ~15× before the
  budget check trips on the next reference; that per-chunk ratio is the
  converter's normal behaviour and is input-bounded.

## 4. WMF record-boundary failures (669ab03c:4)

**Root cause** (verified): `raster_wmf` (`src/convert/metafile.rs:477`–
`685`) checks each record's declared `size*2` against the remaining file
but hands handlers the whole `data` slice, so count-driven handlers read
past the record into later records' bytes, and the outer loop advances
only by the declared size — the same bytes are consumed again as records.
Affected arms: `0x0A32` EXTTEXTOUT and `0x0521` TEXTOUT (retain text
sliced from later bytes), `0x0324` POLYGON, `0x0538` POLYPOLYGON,
`0x0325` POLYLINE (decode/allocate `n` points regardless of the record's
extent).

**Failing tests (red)**

- `src/convert/metafile.rs`, `wmf_object_tests::
  a_polygon_record_lying_about_its_size_does_not_paint_across_records` —
  builds an honest WMF (brush + select + 3-point triangle), asserts the
  honest form fills the triangle; then the identical bytes with the
  polygon record's declared size shrunk to the minimum 3 words
  (`wmf_with_lying_size` helper) must paint nothing (the record's extent
  cannot hold its points). Red today: the lying record still fills —
  its points are read across the boundary. The lying record's leftover
  bytes deterministically decode as an oversized size and terminate the
  loop (traced: next "size" word is `0x000A0003` words).
- `…an_exttextout_record_lying_about_its_size_retains_no_text` — an
  honest EXTTEXTOUT keeps its text (`texts.len() == 1`); the same record
  with a minimum declared size retains no `MetaText`. Red today: the
  text is sliced from beyond the record and retained.

**Minimal source change**

- `src/convert/metafile.rs` `raster_wmf`: compute
  `let rec_end = off + size2;` once per record; every count-driven
  handler now validates, with checked arithmetic, that its fixed fields
  and count-driven arrays fit inside `rec_end` before decoding,
  allocating or retaining: EXTTEXTOUT requires `at + n <= rec_end`;
  TEXTOUT requires the padded string plus trailing `y`/`x` to fit;
  POLYGON/POLYLINE require `payload + 2 + 4*n <= rec_end`;
  POLYPOLYGON pre-walks the ring counts (`2 + 2*rings + Σ4·n_r ≤ extent`)
  before allocating. A refusing record is skipped; the loop still
  advances by the declared size, so no revisiting amplification.
  EMF is untouched (its point helper already bounds `off + size`).

**Existing behaviour that must keep passing**

- `a_polypolygon_fills_with_the_brush_selected_by_zero_based_index`,
  `a_rectangle_fills_with_the_brush_and_is_outlined_with_the_pen`,
  `text_records_come_out_as_text_in_the_selected_font`,
  `exhausted_and_invalid_object_handles_cannot_replace_the_selected_brush`,
  the rest of `wmf_object_tests` — every honest record's declared size
  already covers its parameters (the `wmf()` test builder sizes records
  correctly), so all continue to paint identically.
- `hostile_input_tests::wmf_huge_record_size_terminates` and the EMF
  hostile tests — termination behaviour unchanged.

## 5. Metadata scrub under alternate property part names (669ab03c:5)

**Root cause** (verified): `scrub_properties` (`src/scrub.rs:357`) read
only the three hard-coded names `docProps/core.xml`, `docProps/app.xml`,
`docProps/custom.xml`; a package whose property relationships target
other part names kept its creators, editors, dates, manager/company and
custom properties, and (for custom) even kept the relationship, while
`scrub` returned success.

**Failing test (red)**

- `src/scrub.rs`, `public_memory_leak_boundary_tests::
  docprops_scrub_reaches_property_parts_the_relationships_name` — takes
  the fixture package, removes the conventional property parts and their
  relationships, adds `metadata/zcore.xml` (creator/lastModifiedBy/
  created), `metadata/zapp.xml` (Manager/Company) and
  `metadata/zcustom.xml` (a `Reviewer` custom property holding the same
  name), each with its package relationship and content-type override.
  After `scrub` with default options: `leaks(out, "Jane Doe")` and
  `leaks(out, "Bob")` are empty, the renamed core part carries no
  `creator`, the renamed custom part is gone, and no
  custom-properties relationship remains. Red today: the metadata
  survives everywhere.

**Minimal source change**

- `src/scrub.rs`: new `property_parts(pkg, rel_type, conventional)`
  resolves every internal package relationship of the core-,
  extended- and custom-properties types through
  `pkg.resolve_rel_target("/", &r.target)` (the same call
  `PartFs::remove_untargeted_part` uses for package-level rels), plus
  the conventional name as fallback; `scrub_properties` iterates those
  parts (deduplicated, sorted), sanitizes each core/extended part with
  the unchanged element lists, removes every custom-properties part with
  its content-type override, and always removes the
  custom-properties relationships (previously skipped when the
  conventional part was absent).

**Existing behaviour that must keep passing**

- The fixture-based behaviour in `tests/input_admission.rs`
  (`scrub_refuses_a_crowd_of_entries`, `scrub` opening valid packages)
  and `src/scrub.rs`'s own tests — conventional packages resolve to the
  same three names (relationship target `docProps/core.xml` and the
  fallback dedupe to one entry).
- Title/subject/application and revision-date retention: the removal
  lists are unchanged.
- Out of minimal scope (not changed, noted honestly): property parts
  with no relationship at all under a non-`docProps` name (orphan parts)
  and a `leaks`-style privacy postcondition check inside `scrub` remain
  as the finding's broader remediation suggests; this patch closes the
  relationship-targeted gap the finding evidenced.

## 6. Renamed auxiliary XML bypasses XML depth limits (669ab03c:0)

**Root cause** (verified): `admit` selects parts for XML checking by
file extension (`src/admission.rs:534`, `.xml`/`.rels`, plus the main
part whatever its name at `:556`–`570`), while the layout, revision
listing and comparison follow relationships and parse the actual targets
with the unchecked permissive parser: theme `src/convert/mod.rs:3305`,
styles `:3480`, numbering `:6557`, converter comments/commentsExtended
`:7669`–`7689` and `load_comments` `:7781`, endnotes `:8527`, footnotes
`:8700`, headers/footers `chrome_part_xml` `:20795`; comparer
`get_revisions` notes loop `src/document_comparer.rs:6318`–`6328` and
compare `parse_part_root` `:6676`–`6689`. (The finding excludes
`src/comments.rs` `CommentFamily::load`, which already calls
`validate_xml` — verified at `src/comments.rs:467`.)

**Failing tests (red)**

- `src/convert/mod.rs`, `theme_slot_tests::
  a_renamed_theme_part_past_the_depth_budget_keeps_default_fonts` — a
  package whose theme relationship targets `word/theme.bin`: the same
  theme nesting zero deep still yields `major == Some("Calibri")`
  (control), while 300-deep wrapping must leave the default theme
  (`major == None`), exactly like a missing theme part. Red today: the
  renamed part is parsed and its fonts load.
- `tests/input_admission.rs`,
  `get_revisions_skips_a_renamed_notes_part_past_the_depth_budget` — a
  footnotes relationship targeting `word/notes.bin` whose single
  footnote nests 300 deep around a `w:ins` revision: `get_revisions`
  must succeed but list no revision from that part
  (`revision_to_json` output contains no `"Deep"`). Control: the same
  notes under `word/notes.xml` is refused by admission's extension gate
  with `INVALID_XML` (green before and after). Red today: the renamed
  part is parsed and the deep revision is listed.

**Minimal source change**

- `src/admission.rs`: `check_xml_depth` becomes `pub(crate)` (same
  tolerant policy admission uses — unquoted/valueless attributes still
  pass, exactly as for `.xml` parts today).
- `src/convert/mod.rs`: new `part_xml_admissible(name, xml)` helper
  (runs `check_xml_depth` under `InputLimits::compare()`, the budget
  `with_layout` admits under via `document_comparer::admit_package`);
  applied as an `Option::filter`/early return at every confirmed
  consumer: `load_theme`, `load_stylesheet` (styles fetch),
  `load_numbering`, `comments_extended` (both parts), `load_comments`,
  `EndnoteBag::load` (endnotes), `load_footnotes`,
  `chrome_part_xml` (headers/footers). A refusing part renders as
  absent — the same default as a missing part.
- `src/document_comparer.rs`: `get_revisions`' notes loop guards the
  parse with `check_xml_depth(part, …, settings.input_limits)` (the
  caller's active limits, as the finding's remediation asks), and the
  compare path's `parse_part_root` takes the active `limits` parameter
  (passed `settings.input_limits` at all six call sites).

**Existing behaviour that must keep passing**

- Every `.xml`-named part already passes this exact check at admission
  (`check_xml_depth` with the tolerant `TagPolicy`), so admitted
  documents render identically; the tolerance for unquoted/valueless
  attributes is preserved
  (`admit_accepts_unquoted_and_valueless_attributes_the_permissive_parser_reads`).
- `get_revisions_admits_under_the_settings_budget` and
  `an_explicit_roomier_count_budget_survives_identical_compare`
  (tests/input_admission.rs) — settings budgets still govern, including
  through the notes guards; `issue650_raised_attribute_limits_survive_changed_compare`
  exercises comparison with raised tag limits and no notes parts
  (`parse_part_root` returns `None` as before).
- Theme/style/numbering/comment/note/chrome loader tests in
  `src/convert/mod.rs` (e.g. `load_theme_reads_minor_east_asia_typeface`,
  `load_theme_reads_script_font_jpan`) — shallow well-formed fixtures
  still load.
- Known deviation (documented, accepted): the convert consumers guard
  under the fixed `InputLimits::compare()` (the pipeline's own admission
  budget), not a per-caller setting; only the comparer paths take the
  caller's limits. A caller tightening `max_xml_depth` below 256 for
  admission still sees renamed parts held to 256 rather than the
  tighter value — strictly stronger than the previous behaviour (no
  check at all).

**Scope note (honest):** this closes every consumer the finding
confirmed as reachable (theme, styles, numbering, comments,
commentsExtended, footnotes, endnotes, headers/footers, revision
listing, comparison). The finding's full remediation — one centralized
checked XML-access boundary for *every* `parse_xdocument` consumer — is a
larger refactor across `src/convert/mod.rs`'s remaining minor call sites
(settings parsing, font-table scans and test-only callers, which the
finding did not confirm as reachable); those are left to a follow-up
round rather than risked blind in one patch.
