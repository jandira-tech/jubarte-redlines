# dedup-0018 remediation plan (TDD red/green)

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0018/`
Reducer round `dedup-0018`, claimed worker `b094fb0a-b007-4b40-b038-06c42dd08587`.
This round's own contributions are the findings whose evidence refs start with
`dedup0018`; the patch below fixes those instances. Instances of the same
aggregates that earlier rounds contributed (renderer chrome parsing, comparer
notes, `edit`'s `existing_comment_ids`, `changes_in`) are named as documented
gaps at the end and are intentionally not in this patch.

Every fix below was authored by reading code only. No cargo/mbx/rustc command
was run (forbidden for this task): the tests were **not** executed here. The
red/green reasoning states, per test, what the pre-patch and post-patch code
does on paper, from the source read in this session.

## Vulnerability 1 — ZIP footer count mismatch bypasses entry-count admission

Finding "ZIP footer and count interpretation mismatch bypasses entry-count
admission"; this round's evidence `dedup0018-n10-zip-count`, `dedup0018-n10-zip-budget`,
`dedup0018-n10-zip-preindex`.

- Root cause (read at `src/admission.rs:595-602`): `declared_entry_count` reads
  the EOCD **total**-entries field at `+10..+12`, but the pinned reader
  (zip 9.0.5, `src/read/mod.rs:69-71` of the crate:
  `value.eocd.data.number_of_files_on_this_disk as usize`) indexes entries by
  the **on-this-disk** field at `+8..+10`. A footer declaring 1 entry in total
  and 60,000 on this disk passes `preindex_budget` (`src/admission.rs:399-408`)
  while the reader indexes 60,000 entries — the `max_entries` admission control
  and the duplicate-collapse check at `src/admission.rs:451-456` are both keyed
  to the wrong number.
- Failing test (red): `src/admission.rs`, `mod tests`, new test
  `an_eocd_disk_count_above_the_total_is_the_declared_count`. It rewrites only
  the `+8..+10` field of an admitted 3-entry package to 60,000 and asserts
  `declared_entry_count(&bytes).unwrap() == 60_000` plus an `InputLimit`
  refusal from `admit` under default limits. Pre-patch the function returns 3
  (it never reads `+8..+10`), so the first assertion fails — red.
- Minimal source change: in `declared_entry_count`, read both u16 fields and
  use the larger as the declared count (`total.max(on_disk)`). The ZIP64 path
  is untouched: zip 9.0.5 uses the EOCD64 total at `+32..40` exactly as
  `declared_entry_count` already does (verified in the crate source at
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zip-9.0.5/src/read/mod.rs:53-65`).
- Existing behaviour that must keep passing: every test in
  `src/admission.rs` `mod tests` and `tests/input_admission.rs` that counts
  entries — `leaks_fail_closed_on_a_crowd_of_entries` (10,013 entries refused),
  the `zip64_tail_probe` / `zip64_sentinel_count_probe` rows (both count
  fields rewritten together, so `max` is unchanged),
  `a_zip64_end_record_is_read_at_the_locator_offset`, and all
  `docx_with` admissions, where the writer fills both fields equally, so
  `max` equals the old value.

## Vulnerability 2 — leak scan parses unbounded XML (stack overflow)

Finding "Scanning a document for retained text can overflow the stack"; this
round's evidence `dedup0018-n2-leaks-api`, `dedup0018-n2-leaks-xml`,
`dedup0018-n2-leaks-parse`, `dedup0018-n2-xml-recursion`, and the expected
control `dedup0018-n2-xml-checked` (`try_parse_xdocument_with_limits` exists at
`src/xmllinq/parse.rs:129-136` and the leak scan does not use it).

- Root cause (read at `src/scrub.rs:527-529`): `xml_holds` calls the
  permissive `dom.parse_xdocument`, whose `parse_element` recurses once per
  element (`src/xmllinq/parse.rs:660-663`) with no depth bound; `leaks` has no
  admission prerequisite (`src/scrub.rs:403-404`), so a `.xml`-named part with
  tens of thousands of nested elements kills the process that runs the
  redaction check.
- Failing test (red): `src/scrub.rs`, `mod public_memory_leak_boundary_tests`,
  new test `an_xml_part_past_the_depth_budget_is_reported_not_dom_parsed`. It
  adds `customXml/deep.xml` (300 nested `<a>`, over the compare budget's
  `max_xml_depth` of 256) to the standard fixture package and asserts
  `leaks(&bytes, "absent needle")` reports a `"{name} (cannot scan XML: …)"`
  finding, and that a shallow custom-named part still scans clean. Pre-patch,
  the 300-deep part parses fine, holds no needle, and is reported nowhere —
  the assertion fails — red. (The crash itself needs ~50k nesting; the test
  pins the policy boundary that prevents it.)
- Minimal source change: `xml_holds` takes the active `InputLimits`, runs
  `crate::xmllinq::parse::validate_xml_with_limits` first and returns
  `Result<bool, String>`; `leaks_within` turns `Err` into a fail-closed
  finding (`"{name} (cannot scan XML: {error})"`) and stops, exactly as the
  neighbouring "cannot read" entry findings do (`src/scrub.rs:456-460`) —
  `edit`'s redaction check refuses on any finding, which is the documented
  fail-closed contract (`src/scrub.rs:390-398`).
- Existing behaviour that must keep passing: the two `xml_holds` unit tests
  (updated call sites only, same assertions — their inputs are shallow,
  single-root, well-formed, and validate), and
  `public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources`
  plus `decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not`
  (exact finding lists on Word-written fixture parts, which pass strict
  validation — the same validation `inspect::parse_part_within`
  (`src/inspect.rs:794-805`) already applies to story parts of real packages).
  Risk noted: any fixture part that strict validation refuses would now be
  reported; the preserved tests above are the guard against that.

## Vulnerability 3 — legacy DOC field nesting scans the stack per character

Finding "Reading or converting nested legacy DOC fields can monopolize the
processing thread"; this round's evidence `dedup0018-n8-legacy-read`,
`dedup0018-n8-legacy-fields`.

- Root cause (read at `src/legacy_doc.rs:1602-1604`): `blocks` computes
  `fields.iter().any(|&code| code)` for **every** story character — O(depth)
  per character, O(n·depth) overall; a 1 MB story of `\u{13}` marks costs
  ~5·10¹¹ boolean tests after Vulnerability 5 caps the story length.
- Test (behaviour guard, not red): `src/legacy_doc.rs`, `mod tests`, new test
  `deeply_nested_field_codes_suppress_all_their_content` — 4,001 unclosed
  field begins suppress everything after them, output `["a"]`. This is a pure
  performance fix with no observable functional difference (the counter
  computes the same `any()` predicate), so no deterministic red test exists;
  the guard pins the semantics the counter must preserve. The existing
  `a_field_nested_in_a_field_code_shows_only_the_outer_result`
  (`src/legacy_doc.rs:1946-1968`) is the second, already-green guard.
- Minimal source change: maintain `open_fields: usize` (count of `true`
  frames): `+1` on `\u{13}`, `-1` when the separator clears a `true` top, `-1`
  when a `pop` removes a `true`; `in_code = open_fields > 0`. The `Vec<bool>`
  stays (the separator targets the innermost frame), so results are identical.

## Vulnerability 4 — WMF text records read past their declared extent

Finding "WMF record-boundary failures amplify polygon work and retained text";
this round's evidence `dedup0018-n4-wmf-text` (EXTTEXTOUT `0x0A32` /
TEXTOUT `0x0521` at `src/convert/metafile.rs:602-621`).

- Root cause (read at `src/convert/metafile.rs:477-489` and `602-621`): the
  record loop knows each record's extent (`off + size2`, validated at
  `:486-488`), but the text handlers read `count`, string bytes and trailing
  coordinates from the whole buffer: a 6-byte record can claim a 65,535-byte
  string that physically belongs to the records after it, and N tiny records
  each retain the same following bytes as `MetaText`.
- Failing test (red): `src/convert/metafile.rs`, `mod hostile_input_tests`,
  new test `wmf_text_records_stop_at_their_declared_extent`. TEXTOUT variant:
  a record declaring count 8 but holding only `"AB"` + y + x, followed by
  literal filler bytes — asserts no text is emitted (its coordinates lie
  outside the record). EXTTEXTOUT variant: same lie, asserts the retained
  text is exactly `"AB"` (truncated at the record end). Pre-patch the TEXTOUT
  text is `"AB\x01\x00\x01\x00fo"` and the EXTTEXTOUT text is 8 characters —
  both assertions fail — red.
- Minimal source change: bound both handlers' reads by `off + size2`:
  EXTTEXTOUT's string becomes `data.get(at..(at + n).min(off + size2))`;
  TEXTOUT reads its trailing y/x only when `at + 4 <= off + size2` and bounds
  its string the same way. Well-formed records are unchanged (their fields
  already sit inside the extent).
- Existing behaviour that must keep passing: `omitted_cell_and_row_end_tags_keep_the_table`
  and the EMF text tests (`emf_exttextoutw_stays_unpainted_after_mini_365` —
  EMF parsing is untouched); `wmf_huge_record_size_terminates`.

## Vulnerability 5 — aliased legacy pieces expand a tiny stream into a huge story

Finding "Aliased legacy DOC pieces expand small physical text into enormous
logical stories"; this round's evidence `dedup0018-n7-legacy-read`,
`dedup0018-n7-legacy-pieces`, `dedup0018-n7-legacy-main`.

- Root cause (read at `src/legacy_doc.rs:1148-1226`): `main_text` materializes
  one `StoryChar` per logical CP for whatever `ccp_text` the FIB declares;
  pieces share CP boundaries (contiguous by PLC construction, `:1091-1094`),
  but each piece may re-point its whole CP range at the same few physical
  bytes, so 5,000 pieces over a 10-byte stream materialize 50,000 characters
  (and a crafted FIB scales that to billions) before any per-CP bounds check
  can fail.
- Failing test (red): `src/legacy_doc.rs`, `mod tests`, new test
  `a_story_longer_than_the_word_stream_is_refused` — 5,000 ten-CP pieces all
  at `fc = 0` over `b"HelloWorld"`: asserts `main_text` errs with "longer
  than the WordDocument stream" and that a story the stream holds
  (`ccp_text = 10`) still reads. Pre-patch the aliased call returns
  `Ok(50_000)` — the error assertion fails — red.
- Minimal source change: refuse at the top of `main_text` when
  `u64::from(ccp_text) > word.len() as u64`. Every character of the main
  story occupies at least one byte of the `WordDocument` stream (compressed
  pieces are 1 byte/char, Unicode 2), so no file Word opens is refused; the
  retained story becomes O(stream size) rather than O(FIB count × stream).
- Existing behaviour that must keep passing:
  `a_surrogate_pair_spans_pieces_and_a_lone_half_is_marked`
  (`ccp_text` 4 and 5 over a 14-byte stream — under the bound),
  `a_piece_table_that_does_not_cover_the_story_is_refused` (`ccp_text` 10 =
  stream length 10 — not refused by the new check, still refused by the
  coverage checks), and every `read(SOURCE)`-based test on
  `tests/fixtures/legacy/services.doc`.

## Vulnerability 6 — ragged legacy tables pad to an unbounded rectangle

Finding "Ragged legacy DOC tables expand into an unbounded padded rectangle";
this round's evidence `dedup0018-n9-legacy-pad`, `dedup0018-n9-legacy-rows`,
`dedup0018-n9-legacy-rowprop`.

- Root cause (read at `src/legacy_doc.rs:511-527`): the Markdown writer pads
  every row to the widest row's cell count — width is attacker-controlled
  through row-end PAPX marks, and `doc_to_docx` goes through the same
  Markdown intermediate (`src/legacy_doc.rs:242-249`), so both public routes
  allocate rows × width cells.
- Failing test (red): `src/legacy_doc.rs`, `mod tests`, new test
  `a_table_wider_than_words_column_ceiling_is_trimmed_to_it` — a one-row
  table of 100 cells: asserts the separator row carries exactly 63 `---|`,
  `c62 |` is present and `c63 |` is not. Pre-patch width is 100 — red.
- Minimal source change: clamp the padded width to a new
  `MAX_TABLE_COLUMNS: usize = 63` — Word's own table column ceiling (a Word
  table holds at most 63 columns), so a wider row is not a shape Word can
  represent, and columns beyond it are dropped exactly as Word's HTML import
  drops them.
- Existing behaviour that must keep passing: `tables_become_github_tables`
  (2 columns), every `to_markdown`/`doc_to_markdown(SOURCE)` expectation, and
  `doc_to_docx` round-trips.

## Vulnerability 7 — a single colspan allocates gigabytes of grid

Finding "Rendering an HTML chunk can allocate gigabytes from a single colspan
value"; this round's evidence `dedup0018-n0-chunk-consumer`,
`dedup0018-n0-chunk-transform`, `dedup0018-n0-span-input`,
`dedup0018-n0-span-count`.

- Root cause (read at `src/convert/altchunk.rs:562-567` and `469-491`): a
  parsed `colspan` is only lower-clamped, so `cols` (the max per-row span sum)
  can be any `usize` — the emitted `<w:gridCol/>` loop then allocates ~25
  bytes × cols, and the span `sum::<usize>()` itself can overflow (debug
  panic) on a crafted row.
- Failing test (red): `src/convert/altchunk.rs`, `mod tests`, new test
  `a_colspan_past_words_column_ceiling_is_capped_not_allocated` —
  `colspan="10000"`: asserts 63 `<w:gridCol` and `gridSpan w:val="63"`; a
  `colspan="3"` control asserts 3 columns and `gridSpan w:val="3"`.
  Pre-patch the first count is 10,000 — red.
- Minimal source change: clamp the parsed span to
  `MAX_TABLE_COLUMNS = 63` (the same Word ceiling as Vulnerability 6), and
  compute `cols` with a saturating fold clamped to the same ceiling. Memory
  for the grid becomes O(1); cells stay linear in chunk size.
- Existing behaviour that must keep passing:
  `omitted_cell_and_row_end_tags_keep_the_table` (no spans declared — span 1),
  `a_class_rule_and_inline_bold_become_run_properties`,
  `an_alt_chunk_is_replaced_by_its_paragraphs`, and every fixture-based
  altChunk conversion (real documents never declare spans past 63, because
  Word cannot hold such a table).

## Vulnerability 8 — CSS selector groups clone the declaration map per selector

Finding "HTML selector groups multiply CSS values into excessive memory"; this
round's evidence `dedup0018-n1-css-owned-values`, `dedup0018-n1-css-per-selector`,
`dedup0018-n1-css-eager`, `dedup0018-n1-chunk-consumer`,
`dedup0018-n1-chunk-transform`.

- Root cause (read at `src/convert/altchunk.rs:152-171`): `parse_css` inserts
  a deep `props.clone()` for every selector of a comma group, so a style
  block of N selectors × M declaration bytes retains N×M bytes before any
  body element is examined.
- Tests: `src/convert/altchunk.rs`, `mod tests`, new test
  `a_selector_group_shares_one_declaration_map` — (a) regression guard: every
  selector of `p, div, span { color: red }` carries `color: red`, and a
  selector repeated across blocks merges later-over-earlier (passes before
  and after); (b) the structural green assertion `Rc::ptr_eq(&rules["p"],
  &rules["div"])` — one declaration map per group — which only compiles
  against the fixed representation (the red state is the per-selector clone
  the finding's evidence records).
- Minimal source change: rules become `HashMap<String, Rc<Props>>`; a group
  allocates one `Rc<Props>` shared by its new selectors; a selector an
  earlier block already holds merges through `Rc::make_mut` (copies only when
  another selector still shares the earlier map — behaviour identical to
  today's `entry().or_default().extend(props.clone())`).
  `Builder::props` (`src/convert/altchunk.rs:299-305`) iterates
  `r.iter().cloned()` instead of cloning the map. Total CSS memory becomes
  linear in the input.
- Existing behaviour that must keep passing:
  `a_class_rule_and_inline_bold_become_run_properties` (class rules,
  inline style, font/size resolution) and all altChunk fixture conversions.

## Vulnerability 9 — cyclic drawing dependencies recurse without bound

Finding "Cyclic Markdown drawing dependencies terminate conversion through
unbounded recursion"; this round's evidence `dedup0018-n3-drawing-decode`
(`src/markdown/from_docx/drawing.rs:1397-1408`), recursion at
`dep_part` (`src/markdown/from_docx/drawing.rs:1956-1988`, self-call `:1974`).

- Root cause: `decode` accepts any base64 JSON as a `drawing:` payload, and
  the capture-side caps (`MAX_DEPS`/`MAX_DEP_RELS`, `src/markdown/from_docx/drawing.rs:61-63`,
  enforced only at `:596-617` on the write path) are not applied to decoded
  payloads; `dep_part` then recurses through `part` indices with no
  visited-set, so a dep whose closure names itself (or a deep chain) runs the
  stack out. The threat model records the same gap
  (`output/threatmodel.md:221`).
- Failing tests (red): `src/markdown/from_docx/drawing.rs`, `mod tests`,
  - `a_cyclic_dependency_closure_is_refused_not_recursed`: a one-dep payload
    whose entry relationship (`drawing.rels`) and the dep's own relationship
    both target dep 0's index — asserts `materialize` returns `None`. The
    entry relationship is what makes `materialize` reach `dep_part` at all;
    without it the walk never starts and the assertion fails on both sides
    of the fix (a second-review correction to this patch). Pre-patch the
    call recurses until the stack overflows (the test process aborts) — red
    in the strongest sense; post-patch it is a typed `None`, the same
    refusal a payload that cannot name its targets already gets (`:1982`).
  - `a_payload_past_the_dependency_caps_is_refused_at_decode`: a payload with
    `MAX_DEPS + 1` deps errs at `decode`, and a 1-dep payload still decodes.
    Pre-patch both decode — red.
- Minimal source change: `decode` enforces the write-side caps
  (`deps.len() <= MAX_DEPS`, per-dep `rels.len() <= MAX_DEP_RELS`) on the
  read side; `dep_part` takes a `visiting: &mut Vec<bool>` sized to
  `deps.len()`, refuses an index already on the active path, and clears the
  mark on success so legitimate diamond dependencies still materialize.
  Depth is bounded by `MAX_DEPS` (64), so chains are bounded too. Both
  `materialize` callers (`src/markdown/package.rs:1740`,
  `src/markdown/view_stories.rs:1542`) go through `dep_part`, so body and
  header/footer materialization are covered without signature changes.
- Existing behaviour that must keep passing: the round-trip tests in
  `src/markdown/from_docx/drawing.rs` (`xml_round_trips_verbatim`,
  `unchanged_blank_box_keeps_controls_and_remains_editable`,
  `doc_pr_ids_normalize_and_renumber`) and the docx↔markdown drawing
  round-trip suites; payloads this engine writes always satisfy the caps.

## Vulnerability 10 — revision processing parses renamed auxiliary XML unchecked

Finding "Processing renamed auxiliary XML bypasses XML depth limits"; this
round's evidence `dedup0018-n6-aux-entry` (`src/document_comparer.rs:4961-4987`,
the public `accept_revisions`/`reject_revisions` byte facades),
`dedup0018-n6-aux-selection` (`src/revision_processor.rs:3689-3728`,
`revision_bearing_parts` selects headers/footers/endnotes/footnotes/comments/
styles by relationship type with no name constraint),
`dedup0018-n6-aux-parse` (`src/revision_processor.rs:3784-3792`,
`RevisionIds::new` parses each selected part with unchecked
`dom.parse_xdocument`), `dedup0018-n6-aux-call`
(`src/revision_processor.rs:3897-3905`, `resolve_with_comments` runs it, plus
the numbering `Levels::parse` at `:3901`).

- Root cause: admission applies XML depth/tag checks by `.xml`/`.rels` suffix
  (`src/admission.rs:531-569`), but the revision consumers select parts by
  relationship type, so a renamed `notes.bin` reaches the recursive DOM
  parser unmeasured.
- Failing test (red): `src/revision_processor.rs`,
  `mod public_block_control_ownership_tests`, new test
  `a_renamed_deep_notes_part_is_refused_like_notes_xml`. It builds the
  standard test package plus a footnotes relationship to a 300-deep
  `word/notes.bin` and asserts `accept_revisions` refuses with
  `INVALID_XML`, the part name, and "nests deeper than 256"; a `.xml`-named
  twin is refused by admission already (sanity row), and a shallow renamed
  notes part still accepts (valid arbitrary names remain supported).
  Pre-patch the `.bin` row returns `Ok` — `expect_err` panics — red.
- Minimal source change: new `pub(crate) fn check_revision_parts(pkg, limits)`
  in `src/revision_processor.rs` runs the admission XML policy
  (`check_xml_depth`, made `pub(crate)` in `src/admission.rs:788`) over every
  `revision_bearing_parts` entry plus the numbering part, whatever their
  names, mapping refusal through `crate::opc::refused`. The three byte
  facades with `Result` channels call it right after opening the package:
  `accept_revisions`, `reject_revisions` (`InputLimits::compare()`), and
  `accept_source_revisions` (the caller's `limits`). This is exactly the
  treatment `inspect::parse_part_within` (`src/inspect.rs:794-805`) already
  gives relationship-selected story parts.
- Existing behaviour that must keep passing: every test in
  `mod public_block_control_ownership_tests` (the `package()` helper's
  parts are shallow Word XML), the `resolved()`-based revision tests, and
  `changes`/compare suites that round-trip real packages.

## Explicitly not fixed in this patch (documented gaps)

The same aggregates carry instances contributed by earlier rounds; per the
round instructions this patch fixes this round's contributions only:

- Renderer chrome (theme/styles/numbering/fontTable) and converter comment
  parses under renamed targets (`src/convert/mod.rs` sinks — dedup0016/dedup0017
  evidence), `get_revisions`/compare notes parsing
  (`src/document_comparer.rs:6318-6328`, `6676-6689`),
  `pre_process_markup` styles/notes/charts (`:4684-4750`, `:4861-4878`),
  `edit`'s `existing_comment_ids` (`src/edit.rs:2154-2158`, `:5552-5563`), and
  `changes_in`'s identical unchecked parse (`src/changes.rs:486-490`) — all
  need the same `check_xml_depth`-at-the-consumer boundary; the
  `check_revision_parts` helper added here is the template.
- WMF polygon/polypolygon handlers' out-of-record reads (other rounds'
  evidence in the same finding) — only the two text records this round
  contributed are confined.

## Verification (not run here)

No cargo/mbx/rustc command was run (forbidden for this task), so no test,
build, `cargo fmt --check` or clippy pass was executed. The patch was
authored by reading code and audited hunk by hunk against the repository
files (every context line re-read at its site; every hunk header's line
counts recomputed against the hunk body). `git apply --check` could not be
run by the first pass (the session's shell stopped spawning midway through
that task: `spawn /bin/zsh ENOENT`, repeated), which is also why the patch
is hand-computed rather than `git diff`-generated.

**Second review (this file's current state).** A second reviewer re-checked
the patch against the repository source and corrected it in place; from the
repository root:

    git apply --check .scan/remediation/patches/dedup-0018.patch   # PASS
    git apply .scan/remediation/patches/dedup-0018.patch && git apply -R .scan/remediation/patches/dedup-0018.patch   # apply/revert cycle clean

Both were run and passed; the working tree was left unmodified. Defects the
second review found and fixed in the patch (bodies and hunk headers):

- Several hunk headers carried wrong line counts or start lines (scrub.rs
  `-524,50 +533,59` → `-524,51 +533,60`; legacy_doc.rs `-510,7 +515,13` →
  `-510,6 +515,11`, `-1146,6` → `-1147,6`, and five more start/count drifts;
  altchunk, metafile, drawing, revision_processor, document_comparer
  similarly). All headers were recomputed from the bodies.
- A stray separator blank in scrub.rs between two test hunks, and an added
  blank line in altchunk.rs that had lost its `+` prefix.
- Three context lines in the legacy_doc.rs `blocks()` hunk were short four
  leading spaces each.
- Two context lines were simply wrong: legacy_doc.rs `};` where the source
  closes an array with `];`, and `fn plain(...)` as trailing context of the
  wide-table test where the source has `#[test] fn
  line_breaks_become_hard_breaks()` (the wrong line was dropped).
- Two context lines had been skipped entirely (drawing.rs
  `used.insert(name.clone());` in `dep_part`; revision_processor.rs
  `pkg.set_part(MAIN, format!(...))` in the `package()` helper); both were
  restored.
- The drawing cycle test put its cyclic relationship only in `deps[0].rels`
  while `materialize` walks the top-level `drawing.rels`, so the recursion
  was never entered and the `.is_none()` assertion failed on both sides of
  the fix; the test now carries an entry relationship into dep 0.

The second review also re-verified, by reading source only: `validate_xml_
with_limits` is `pub` in `xmllinq::parse` and returns `Result<(), String>`;
`check_xml_depth`'s new `pub(crate)` visibility is enough for the
`revision_processor` call; `numbering_part` returns `Option<String>` and
`revision_bearing_parts` `Vec<(String, bool)>`, matching the chained
iteration; `OpcError::Io`'s Display (oxml-opc 0.14.2, thiserror) forwards
the `io::Error`, whose Display forwards `AdmissionError`'s
`"{INVALID_XML}: {part}: nests deeper than 256"` — so the renamed-notes
test's three `contains` assertions hold; zip 9.0.5 (Cargo.lock) indexes
non-ZIP64 archives by the EOCD on-this-disk count and ZIP64 by the EOCD64
total, exactly the two fields `declared_entry_count` now takes the max of;
and every preserved-behaviour test named below exists in the tree.

Then, per AGENTS.md, sequential cargo only:

    cargo nextest run -E 'test(an_eocd_disk_count_above_the_total) or test(an_xml_part_past_the_depth_budget) or test(deeply_nested_field_codes) or test(wmf_text_records_stop_at_their_declared_extent) or test(a_story_longer_than_the_word_stream) or test(a_table_wider_than_words) or test(a_colspan_past_words) or test(a_selector_group_shares) or test(a_cyclic_dependency_closure) or test(a_payload_past_the_dependency_caps) or test(a_renamed_deep_notes_part)'
    cargo nextest run -E 'test(a_needle_in_a_numeric_attribute) or test(element_names_are_no_leak) or test(public_leaks_reports) or test(decoded_entities_and_split) or test(a_field_nested_in_a_field_code) or test(a_surrogate_pair_spans) or test(tables_become_github_tables) or test(omitted_cell_and_row_end_tags) or test(a_class_rule_and_inline_bold) or test(a_page_break_stays_inside)'
    cargo fmt --check && cargo clippy --lib --tests -- -D warnings

Red confirmation (before applying the source hunks, with the test hunks
applied) is expected for: the EOCD count test, the deep-XML leak finding test,
the aliased-pieces test, the wide-table test, the colspan test, the
EXTTEXTOUT/TEXTOUT extent assertions, the decode-cap test, the renamed
deep-notes test, and — by process abort — the drawing cycle test. The
field-counter and CSS group tests are behaviour guards for pure-performance
or representation changes and pass both sides by design.
