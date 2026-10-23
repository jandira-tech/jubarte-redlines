@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings being tackled (folder dedup-0028)

### csf_dc0d34a6ea9a964733efe547 — Aliased legacy DOC pieces expand small physical text into enormous logical stories (severity: medium; plan §1)

**Summary:** Thousands of contiguous logical pieces can all reference the same physical text range. For example, 4095 pieces referencing one common 1 MiB compressed-text range describe nearly 4 GiB of logical characters while requiring only roughly 49 KiB of piece records. main_text materializes metadata for every character, requiring tens of GiB before rendering. A second source-derived framing uses approximately 65,535 pieces reusing 64 KiB, producing over four billion logical characters from a piece table below 1 MiB; expansion precedes later auxiliary-table checks. The assigned 1024-by-65536 framing yields 67108864 logical characters and at least 768 MiB for char/CP/FC fields alone, before modifiers and allocation/output overhead; these are static counts, not measurements. Supplied discovery-0023 additionally gives 4096 contiguous one-million-character pieces sharing a 1 MB physical range (4096000000 logical characters within u32). Public read/conversion and native read_document refusal-hint parsing reach the same expansion before returning a format diagnostic; no runtime resource measurement is claimed. Additional supplied static framing: 4096 pieces reusing one 64 KiB compressed extent describe 268435456 logical characters; continuity and source-bound checks permit the reuse. This is not a measured allocation. Assigned discovery-0031 supplies 4000 one-million-character logical pieces sharing about 1 MB of physical text with about 48 KB of piece metadata, requesting about four billion StoryChars. CP continuity, checked offsets and physically bounded CFB traversal do not bound logical expansion; counts are static.
 Assigned discovery-0032 supplies 65535 contiguous pieces of 65535 compressed characters sharing one 65535-byte physical range: 4294836225 logical CPs within u32 and about 786 KB of piece records. These are static counts.

**Remediation:** Introduce configurable legacy physical-input, piece-count, cumulative logical-character, retained-metadata and expansion-work budgets. Check FIB ccp_text and checked cumulative piece lengths before materialization, charge every decoded character including repeated physical aliases, and return typed refusals through read/doc_to_markdown/doc_to_docx and CLI callers. Preserve legitimate fast-save aliases and split UTF-16 surrogate handling within budget. Replace full legacy parsing used only for diagnostic refusal hints with a cheap bounded probe, or apply the same logical/work budgets before that parser runs.
Additional retained remediation detail: Introduce a configurable legacy logical-character/output budget, check ccp_text and cumulative decoded piece work before materialization, and enforce it during decoding. Preserve valid reuse while returning a typed refusal for excessive expansion.

### csf_0e2b085a251b34412cd71a4c — Reading or converting nested legacy DOC fields can monopolize the processing thread (severity: medium; plan §2)

**Summary:** An attacker-authored Word 97–2003 DOC can repeatedly begin fields and enter their result sections, leaving an increasingly long all-false field stack. Story::blocks scans the entire stack before every input character, giving quadratic CPU work during reading/conversion and even the native Word-input refusal hint. Assigned discovery-0031 supplies a balanced result-state framing with approximately 100000 nested fields in a few hundred kilobytes, implying billions of boolean inspections. This is heap-stack scanning CPU cost, not recursive call-stack overflow.

**Remediation:** Maintain an O(1) active-code-frame count, updating it correctly on begin, separator and end while preserving suppression under any code ancestor. Enforce explicit field-nesting and processing-work budgets for balanced and malformed legacy field streams before excessive work. Keep legacy diagnostic probes cheap and bounded, or enforce the same field/work budgets before invoking the full reader solely for a refusal hint.

### csf_64b247931cd1f053217fde7a — Rendering an HTML chunk can allocate gigabytes from a single colspan value (severity: medium; plan §3)

**Summary:** A document author can place a huge positive colspan in a small embedded HTML/MHT altChunk table. Ordinary DOCX rendering generates one grid XML element per effective column after original package admission and before DOM/layout checks. A count of 1000000000 drives a billion appends; 4294967295 generates over 85 GB of markup, exhausting CPU/memory. The supplied positive 4294967295 framing fits both 32-bit and 64-bit usize and needs no arithmetic overflow. Assigned discovery-0031 corroborates colspan=1000000000 causing tens of gigabytes of generated XML and work before layout; rejecting invalid, negative or zero spans does not bound valid positive values.
 Assigned discovery-0032 supplies a static colspan=1000000000 framing of one billion 20-byte grid elements, about 20 GB of generated text before DOM construction; this is not a runtime measurement.

**Remediation:** Enforce a practical maximum cell span and checked per-row/aggregate column counts before generating any table XML. Charge all HTML/MHT-generated WML, including repeated/nested tables, to a cumulative byte/work budget inside altChunk expansion and return a typed input-limit refusal before allocation or DOM construction.
Additional retained remediation detail: Bound individual spans and aggregate columns before summation or iteration, use checked arithmetic, and impose a cumulative generated-WML byte/element budget shared across every HTML/MHT altChunk. Refuse excessive expansion with a typed input-limit error before allocation.

### csf_3201e6caaef21850d61fa69c — WMF record-boundary failures amplify polygon work and retained text (severity: medium; plan §4)

**Summary:** Rendering a DOCX with a crafted placeable WMF lets short declared records consume payload from later records, which the outer parser then revisits. Polygon/polypolygon/polyline records can cause billions of coordinate decodes from roughly 512 KiB; EXTTEXTOUT/TEXTOUT records can retain hundreds of megabytes of overlapping owned strings from roughly 205.6 KB. The common missing record-local boundary bypasses the canvas cap and original package budgets. A further framing uses alternating six-byte EXTTEXTOUT and unknown 0xFFFF record headers to claim 65,535 text bytes from subsequent records, implying several GiB of retained strings from roughly 1 MiB of media; this is a static estimate. A supplied 7-word EXTTEXTOUT framing uses 65536 fourteen-byte records plus a 65535-byte suffix in less than 1 MiB and requests about 4 GiB of decoded characters; nonzero mapping dimensions and nonblank text are required. This is a static estimate. Assigned discovery-0031 supplies an additional static eight-byte TEXTOUT framing with a 65535-byte claimed string and overlapping following bytes. About 1 MiB can request multiple gigabytes of retained text; valid whole-file reads, nonblank text and nonzero mapping extents remain required. No out-of-bounds memory read is asserted.

**Remediation:** Confine every WMF handler, including field, font, string and geometry reads, to a slice limited to its checked declared record extent. Before allocation validate fixed fields, ring/point counts and coordinate arrays, EXTTEXTOUT options/optional rectangle and text count, and TEXTOUT padded string plus trailing coordinates against that slice using checked arithmetic. Refuse inconsistent records before decoding/rasterization/replay and enforce cumulative decoded-text, record and total geometry/work budgets. Keep the already bounded EMF behavior.

### csf_8dcaf50784fd95a7f0c2ec28 — Processing renamed auxiliary XML bypasses XML depth limits (severity: medium; plan §5)

**Summary:** A DOCX author can put deeply nested auxiliary XML under a non-.xml/.rels suffix. Admission drains those bytes without XML policy, while rendering, revision listing, revision acceptance/rejection and comparison follow internal relationships and recursively build a DOM. Confirmed rendering parts include theme, styles, numbering, comments/commentsExtended, notes and headers/footers; get_revisions and comparison also parse renamed footnotes/endnotes. A small admitted package can terminate the native host or trap WASM. The assigned source additionally confirms eager relationship-selected fontTable parsing before font inspection, including renamed fontTable.bin. Comparison preprocessing also follows renamed styles and charts, alongside footnotes/endnotes. The assigned Standard source additionally identifies edit initialization's existing_comment_ids on relationship-selected comments before the later checked CommentFamily loader. Assigned discovery-0019 corroborates the same shared selector for styles, theme, numbering and fontTable under .bin names; its proof is limited to those four consumers, while earlier validated consumer paths remain retained. Assigned discovery-0027 additionally confirms relationship-selected settings bytes reaching settings_dom/settings_dom_xml and unchecked DOM construction on cache miss; settings is now a supplied validated consumer, whereas earlier recommendations remain historical source limitations. Assigned discovery-0028 supplies already-validated linked-textbox and diagram consumers: relationship-ID media targets at src/convert/mod.rs:17417-17428 and :17453-17464 are decoded and recursively parsed regardless of filename. The new source corroborates fontTable, theme, styles, numbering, settings, comments, notes and header/footer paths, and records that admission's content-type check covers the main part only. Assigned discovery-0031 additionally preserves strict normalization's extension-based pass-through: a relationship-selected fonts.bin remains unmodified before eager fontTable parsing. The raw unchecked parser's documented caller obligations are not themselves the finding.

**Remediation:** Apply active InputLimits at a centralized checked XML access/parse boundary for every semantic XML consumer regardless of filename, or resolve/classify and admit XML through actual content-type/relationship use. Cover theme, styles, numbering, converter comments/commentsExtended, notes and headers/footers; explicitly cover the now supplied validated settings_dom/settings_dom_xml cache-miss consumer, with earlier settings-only recommendations retained as historical source limitations. Preserve renamed-main checks, and enforce actual DOM construction depth or use iterative construction. Apply that same mandatory checked boundary to get_revisions and comparison notes parsing, including both footnotes and endnotes; propagate typed refusal under the active comparer InputLimits. Include pre_process_markup's relationship-selected notes at src/document_comparer.rs:4725-4750 and exposed Python get_revisions_json/WASM getRevisions; valid arbitrary part names remain supported. Include linked-textbox and diagram relationship targets in the mandatory filename-independent checked XML boundary before parse_xdocument and in regression coverage. These are now supplied validated affected consumers from discovery-0028; earlier sources' proposed-only status remains historical. Cover revision_processor RevisionIds collection and process_part for all six selected auxiliary buckets (headers, footers, endnotes, footnotes, comments and styles), including public revision acceptance/rejection. Explicitly include the now-confirmed fontTable loader (src/convert/font_table.rs:66-78; src/convert/mod.rs:806) in the mandatory checked XML boundary; parsing must be bounded before any font root/entry inspection. Explicitly cover comparison defined_style_ids (src/document_comparer.rs:4684-4696), relationship-selected footnotes/endnotes (4724-4750) and chart preprocessing (4861-4878), all reached through admitted nonidentical comparison inputs (6555-6569,6607-6625). Apply the same mandatory XML boundary before edit initialization's existing_comment_ids scan, not only in the later checked CommentFamily loader; preserve the effective checks at inspect::parse_part_within and edit::load_stories. Enforce the same checked boundary before settings DOM cache population, and preserve valid bounded nonstandard auxiliary filenames.

### csf_c2ab52b1c8614c7fd5356cf8 — Python and Node plan recording can overwrite unrelated files through staging symlinks (severity: low; plan §6)

**Summary:** The Python and Node CLIs write plans through a predictable .<plan>.<PID>.partial pathname using nonexclusive truncating writes. A local actor who can create entries in the plan directory can preplace a symlink there and make the victim overwrite another file it can write, before the intended plan replacement.
 Assigned discovery-0032 corroborates both non-dry-run recorders after successful plan preview. Intentional final-plan symlink resolution is distinct from the independently substituted staging entry.

**Remediation:** Use exclusive, unpredictable staging owned by a private directory, hold the opened handle through writing, and rename that verified file. Reuse the native publication design or equivalent mkstemp/exclusive wx APIs and never follow existing staging entries. Python can reuse native publication; Node needs equivalent secure publication semantics. Preserve intentional final-plan symlink resolution to its authorized resolved target while securing the separate staging object; successful recording installs a verified regular plan file and failure preserves prior output.
Additional retained remediation detail: Use an exclusively created private staging directory/file held by handle, verify staged bytes, and atomically publish to the authorized plan target. Refuse preexisting staging objects and preserve intended final-plan symlink semantics without following a staging symlink.

Duplicate rows for the same sinks, also tackled:

- **csf_f756bbe5509c42bdcaa64921** — duplicate row of csf_8dcaf50784fd95a7f0c2ec28 (same title/severity/summary/remediation).
- **csf_88d031991ca3e20e4aae2f24** — duplicate row of csf_0e2b085a251b34412cd71a4c (same title/severity/summary/remediation).
- **csf_32625277093e7f286a267e2b** — duplicate row of csf_64b247931cd1f053217fde7a (same title/severity/summary/remediation, minus one retained sentence each).
- **csf_80edacafa6b67d4864116a38** — duplicate row of csf_3201e6caaef21850d61fa69c (same title/severity/summary/remediation).

---

## TDD remediation plan (dedup-0028)

# dedup-0028 remediation plan (TDD red/green)

Patch: `.scan/remediation/patches/dedup-0028.patch` (`git apply` from the
repository root; `git apply --check` exit 0, verified before and after
review). Scope: the six dedup-0028 findings (#1, #3, #5, #8, #9, #36).
Everything was authored by reading the named sources; **no cargo/mbx/rustc
was run** (the ask forbids Cargo here), so compile/tests are follow-ups.
Constraints: Word fidelity first (no Word-valid input changes behavior), and
AGENTS.md's "tests beside the behavior they protect" — every new test lives
in the module it protects and asserts output, not timing, except where
bounded-time precedent exists (`a_self_linked_difat_sector_is_bounded_by_the_file`,
`wmf_huge_record_size_terminates`).

## 1. Aliased legacy DOC pieces expand small text into enormous logical stories (medium)

Finding #9 (`csf_dc0d34a6ea9a964733efe547`). Root cause
(`src/legacy_doc.rs:1148-1226`): `ccp_text` is a caller-controlled u32 from
the FIB (`:919`); pieces may alias one physical `fc` range across many
logical CP ranges (the coverage check only demands contiguous increasing
CPs) and `main_text` pushes one `StoryChar` per logical CP unbounded —
1024 pieces x 65,536 CPs over one 64 KiB range = 67,108,864 StoryChars
(over 1 GiB) from a tiny file.

Failing tests (`src/legacy_doc.rs` `mod tests`):

- `aliased_pieces_expanding_past_the_budget_are_refused` — 1024 contiguous
  pieces with `fc: 0` over 65,536 bytes, `ccp_text = 1024 * 65_536`; asserts
  an error containing `character budget`. Unfixed: `Ok` with 67M chars (and
  the gigabyte allocated): red.
- `overlapping_restarts_are_charged_to_the_cumulative_budget` — three pieces
  restarting at CP 0 over a 3 Mi stream (declared 3 Mi is within budget,
  cumulative 9 Mi): asserts refusal. Unfixed: `Ok` with 9M chars: red.
- `fast_save_aliases_within_the_budget_still_expand` — two 4-CP pieces
  sharing `fc: 0` over `"abcd"` still expand to `"abcdabcd"`.

Source change: `MAX_STORY_CHARS: u32 = 1 << 23` (8,388,608 chars = 16 MiB of
UTF-16; retained metadata ~128 MiB, the scale of `admission`'s 256 MiB
default, `src/admission.rs:63-78`). `main_text` refuses
`ccp_text > MAX_STORY_CHARS` after the coverage check, and charges each
piece's clamped span (`end.saturating_sub(piece.cp_start)`) into a running
u64 refused past the same cap — both before any character is pushed. All
arithmetic `saturating_*` (module denies `clippy::arithmetic_side_effects`
outside tests, `src/legacy_doc.rs:8-11`). Errors surface as `LEGACY_DOC:
...` like every other refusal, so CLI/bindings are unchanged. Must keep
passing: `a_piece_past_the_stream_is_refused_not_truncated`,
`a_piece_table_that_does_not_cover_the_story_is_refused`,
`a_surrogate_pair_spans_pieces_and_a_lone_half_is_marked`,
`tests/legacy_doc_convert.rs`, `tests/legacy_doc.rs`.

## 2. Nested legacy DOC fields monopolize the processing thread (medium)

Finding #3 (`csf_0e2b085a251b34412cd71a4c`). Root cause
(`src/legacy_doc.rs:1591-1620`): `Story::blocks` recomputes
`fields.iter().any(|&code| code)` per character; N begin/separator pairs
leave an N-long all-false stack scanned per character: quadratic. Paid by
`read`/`doc_to_markdown`/`doc_to_docx` (`src/legacy_doc.rs:180,225,236`) and
the CLI refusal hint (`src/bin/jubarte.rs:2740-2750`).

Failing tests (`src/legacy_doc.rs` `mod tests`):

- `a_deep_field_stack_costs_one_step_per_character` — 100,000
  begin/separator pairs then `"b\r"`; asserts one paragraph `["b"]` and
  completion under 2s (bounded-time precedent). Unfixed: ~5e9+ frame visits,
  the elapsed assert fails: red. Fixed: linear.
- `unbalanced_field_marks_keep_their_meanings` — separator/end with no open
  field are no-ops (`"\u{14}a\u{15}\u{13}b\u{15}c\r"` → `["ac"]`); a field's
  code is hidden while its result and successor show
  (`"\u{13}x\u{14}y\u{15}z\r"` → `["yz"]`). Passes before and after; guards
  the rewrite against semantic drift.

Source change (`Story::blocks`): keep the `Vec<bool>` stack but maintain
`code_frames: usize` — incremented on begin; separator decrements only when
the top frame was true; end pops and decrements only when the popped frame
was true. `in_code` becomes `code_frames > 0`, exactly "any frame is in its
code" in O(1); `saturating_add/sub` per the module lint. Must keep passing:
`a_field_nested_in_a_field_code_shows_only_the_outer_result`
(`src/legacy_doc.rs:1945`),
`a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it`,
`empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown`, and the
nested-field fixture test in `tests/legacy_doc_convert.rs:125`.

## 3. An HTML chunk can allocate gigabytes from one colspan value (medium)

Finding #5 (`csf_64b247931cd1f053217fde7a`). Root cause
(`src/convert/altchunk.rs`): `Builder::start` parses `colspan` with only
`.max(1)` (`:562-567`) and `end_table` emits one `<w:gridCol>` per summed
column (`:469-491`) before any DOM/parse bound — `4294967295` generates
>85 GB of markup from one attribute.

Failing test (`mod regression_tests`):
`a_colspan_word_cannot_honor_is_declined_not_expanded` — for colspan in
`64, 1000000, 4294967295, 18446744073709551615` asserts exactly one
`<w:gridCol` and no `<w:gridSpan`; for `colspan='63'` beside a second cell
asserts 64 grid columns and `<w:gridSpan w:val="63"/>`. Unfixed, the
`1000000` case emits a million grid columns (the `4294967295` case does not
complete — that is the vulnerability): red.

Source change: `MAX_TABLE_COLUMNS: usize = 63` — Word's own table column
limit, so a larger colspan can never be Word-faithful anyway — and the
parsed span is filtered `(1..=MAX_TABLE_COLUMNS).contains(&span)` before
`.unwrap_or(1)`, reusing the established invalid/zero-span fallback
(`colspan='0'|'invalid'` already fell back to one column). `end_table` is
untouched. Must keep passing:
`invalid_or_zero_colspan_falls_back_to_one_column`,
`html_tables_keep_spans_empty_cells_and_header_formatting` (colspan 2 still
spans), `unclosed_html_table_cells_are_finalized_once_with_source_spans_and_empty_cells`,
`ragged_authored_rows_share_one_grid_without_inventing_filler_cells_or_reordering_payload`,
and the altChunk expand/relationship contract tests.

## 4. WMF record-boundary failures amplify polygon work and retained text (medium)

Finding #8 (`csf_3201e6caaef21850d61fa69c`). Root cause
(`src/convert/metafile.rs:445-688`): the outer loop checks `size*2` against
the remaining file, but every handler read absolute offsets from the whole
`data` slice and advanced by the declared size — a record whose count
exceeds its own payload consumed bytes from the records after it, which the
outer loop revisited: repeated small EXTTEXTOUT/POLYGON/POLYPOLYGON/POLYLINE
records decode overlapping suffixes into gigabytes of retained strings or
point work. EMF's `read_emf_points` already bounds by `off + size`
(`:1068-1082`); WMF did not.

Failing tests (`mod wmf_object_tests`):

- `a_text_record_may_not_read_string_bytes_past_its_extent` — an EXTTEXTOUT
  whose count (15) passes its 8-byte payload, followed by a record whose
  header+payload the count reaches into; asserts the stream retains **no
  text at all**, and that a well-formed `ext_text` ("Visible") still writes
  exactly one text. Unfixed: one leaked text is retained (the next record's
  six header bytes, then its `A`..`E` words as a NUL-interleaved string):
  red. (Review correction: the first draft asserted no retained text
  *contains* `"ABCDEFGHI"`, but the leaked bytes interleave each letter with
  a NUL — `A\0B\0C\0D\0E` — so that substring never appears and the
  assertion passed on the unfixed tree too; the emptiness assertion is the
  one that can fail, checked against a simulation of both readers.)
- `a_polygon_record_may_not_read_points_past_its_extent` — a POLYGON
  declaring 3 points carrying 1, followed by a record holding the bytes the
  missing points used to come from; asserts `render` returns `None`, and
  that the record carrying all three points still fills the triangle.
  Unfixed: `Some(...)` painted from cross-record bytes: red. (Also verified
  against a byte-level simulation of both readers.)

Source change (`raster_wmf`): after the existing size check, take
`let rec = data.get(payload..off + size2)?;` (the record's own extent; the
preceding check bounds the arithmetic) and rewrite every handler — field,
font, string, geometry, object and state reads alike — to `rec`-relative
offsets. Counts that overrun `rec` now either decline (text: the
`rec.get(at..at+n)` guard) or refuse the render (`?` on short fixed reads);
point loops stop at the record's end. EMF and the placeable/header reads are
untouched. Must keep passing: `a_polypolygon_fills_with_the_brush_selected_by_zero_based_index`,
`a_rectangle_fills_with_the_brush_and_is_outlined_with_the_pen`,
`text_records_come_out_as_text_in_the_selected_font`,
`positive_restore_dc_restores_the_selected_font_color_and_alignment`,
`exhausted_and_invalid_object_handles_cannot_replace_the_selected_brush`,
`malformed_record_sizes_stop_replay_without_erasing_prior_paint` (size < 3
or overflowing size still just stops the replay), and the polygon/polyline
painting checks. Valid WMFs are self-contained per [MS-WMF], so confining
reads to the declared extent cannot change their rendering.

## 5. Renamed auxiliary XML bypasses XML depth limits (medium)

Finding #1 (`csf_8dcaf50784fd95a7f0c2ec28`) — this round's contribution is
the header/footer consumer `chrome_part_xml`; the finding's other consumers
(settings, fontTable, comparer notes, linked textboxes, …) stay with their
owning rounds. Root cause: `admit`'s XML scan covers only `.xml`/`.rels`
names plus the main part under any name (`src/admission.rs:534,543,558-570`),
but a `headerReference`/`footerReference` relationship can select any part
name (`load_chrome_part`, `src/convert/mod.rs:20759-20782`), and
`chrome_part_xml` (`src/convert/mod.rs:20786+`) feeds those bytes straight to
the permissive recursive DOM builder `Dom::parse_xdocument` — a renamed
header nested past `max_xml_depth` (256) overflows the builder's stack where
an `.xml`-named part would have been refused at admission.

Failing test (new `mod chrome_xml_budget_tests`, following the module's
existing `package()`/`sheet()` builders):
`a_renamed_header_part_over_the_xml_budget_lays_out_nothing` — a
`word/header1.bin` header whose run is wrapped in 300 nested `w:smartTag`
elements must lay out no runs; the same renamed part with a plain one-run
header must still lay out "Header". Unfixed: the deep DOM is built and the
run laid out, so `runs.is_empty()` fails: red.

Source change: `src/admission.rs:788` — `check_xml_depth` becomes
`pub(crate)` (it is the package's tolerant checked-XML scan: quick-xml event
loop with `TagPolicy::tolerant(limits)` and the depth counter; signature
unchanged). `chrome_part_xml` runs
`crate::admission::check_xml_depth(path, xml.as_bytes(), InputLimits::default())`
before `parse_xdocument`; an over-budget part returns `empty_chrome()`, the
same fallback as a part that does not parse. The tolerant policy accepts
what the permissive parser reads (quirky-but-openable headers keep
rendering; real `header1.xml` parts already pass this exact scan at
admission today), so only inputs beyond the depth/tag/namespace budgets
(nothing Word writes to a header) change behavior. Cost: one linear scan per
header/footer part. Must keep passing: the header/footer rendering tests of
`src/convert/mod.rs` (755 `#[test]`s in the module) and
`tests/convert_docx_to_pdf.rs`; the generated fallback header XML in
`sect_ref_chrome_of` is shallow and passes the scan.

## 6. Python and Node plan recording can overwrite unrelated files through staging symlinks (low)

Finding #36 (`csf_c2ab52b1c8614c7fd5356cf8`). Root cause: both adapters
stage the plan at a predictable `.<plan>.<PID>.partial` sibling and open it
with an ordinary truncating write — Python `Path.write_bytes`
(`jubarte-python/python/jubarte_redlines/__main__.py:403-414`), Node
`fs.writeFileSync`
(`jubarte-wasm/cli/bin/jubarte-redlines.mjs:408-427`). A local actor who can
create entries in the plan directory pre-places a symlink and the recording
clobbers the file it names. The native recorder already went
exclusive/unpredictable (`src/edit/record.rs:142-165` "Legacy temporary-name
helper, retained for API compatibility"; `src/publication.rs:447-505`
`.jubarte-<uuid>.partial` + `create_new` + permission preservation).

Failing tests:

- `jubarte-python/tests/test_cli.py` ::
  `test_recording_does_not_follow_a_preplaced_staging_entry` — pre-places
  `.{plan}.{os.getpid()}.partial` as a symlink to `victim.txt` (the test
  runs `main()` in-process, so the old name is exactly predictable), records
  a plan, then asserts the victim still reads `"precious"`, the trap is
  still a symlink, and the plan was recorded. Unfixed: `write_bytes` follows
  the link, the victim becomes the plan JSON and the link is renamed onto
  the plan name: red.
- `jubarte-wasm/cli/test/cli.test.mjs` ::
  `plan recording stages through an unpredictable exclusive file` — uses the
  file's `runWithPlanHook` syscall hook to record every `openSync` during a
  recording, then asserts each `.partial` create passed flags containing
  `x` (exclusive) and a name ending `.<uuid>.partial`, that the plan
  records, and that no staging entry remains. Unfixed: staging used
  `writeFileSync` (no `openSync` at all), so the assertion fails: red.

Source changes: Python `__main__.py` stages via
`tempfile.mkstemp(dir=target.parent, prefix=f".{target.name}.",
suffix=".partial")` — unpredictable and exclusive (`O_EXCL`) — and, when
replacing an existing plan, `os.fchmod`s the staging fd to the target's mode
(the native publisher's permission-preservation,
`src/publication.rs:447-465`; skipped where `os.fchmod` does not exist),
writes through `os.fdopen`, and `os.replace`s as before; new plans keep
mkstemp's owner-only mode. Node `jubarte-redlines.mjs` stages at
`.{name}.{randomUUID()}.partial` created with `fs.openSync(partial, "wx")`,
written via the fd, closed, then renamed; the fd is closed and the staging
removed on any failure; mode stays the default `0o666 & ~umask`. Must keep
passing: the intentional symlink-to-plan resolution ("Through a symbolic
link to the file it names, written yet or not" — the Python dangling-link
test in `test_plan_paths_as_typed_and_strict_utf8...`, the Node
`realpathSync`/`readlinkSync` resolution untouched);
`test_edit_and_add_plan_record_and_consume_as_the_binary_does` (including
`not list(tmp_path.glob(".*.partial"))` after refusals — exclusive staging
names still match that glob and are still cleaned up); the Node `--consume`
identity tests.

## Follow-up verification (not run here — cargo/pytest/node-test were out of reach)

From the repository root, after applying the patch:

1. `cargo fmt --check`.
2. `cargo clippy --all-targets -- -D warnings` (no `#[allow]` introduced).
3. Touched suites: `cargo nextest run --lib legacy_doc`;
   `cargo nextest run -E 'test(altchunk)'`; `cargo nextest run -E 'test(wmf)'`;
   `cargo nextest run -E 'test(chrome_xml_budget)'`; plus
   `tests/legacy_doc_convert.rs` and `tests/legacy_doc.rs`.
4. Python: `uv run pytest jubarte-python/tests/test_cli.py` (needs the
   native extension built).
5. Node: `node --test jubarte-wasm/cli/test/` (needs the checked-in
   jubarte-wasm build).

Verified this session: `git apply --check` (exit 0), `python3 -m py_compile`
of both edited Python files, `node --check` of both edited `.mjs` files (run
on a scratch copy with the patch applied). The Rust changes were authored
and reviewed by reading the cited sources; they have not been compiled.


---

## Patch (dedup-0028)

Applies with `git apply` from the repository root.

```diff
diff --git a/jubarte-python/python/jubarte_redlines/__main__.py b/jubarte-python/python/jubarte_redlines/__main__.py
index 686d0f0..71ceb15 100644
--- a/jubarte-python/python/jubarte_redlines/__main__.py
+++ b/jubarte-python/python/jubarte_redlines/__main__.py
@@ -33,6 +33,7 @@ import argparse
 import json
 import os
 import sys
+import tempfile
 from dataclasses import asdict
 from collections.abc import Sequence
 from pathlib import Path
@@ -402,12 +403,25 @@ def _record(args: argparse.Namespace, verb: str, doc: Document) -> int:
             print(f"note: {note}")
     if not dry_run:
         # Through a symbolic link to the file it names, written yet or not;
-        # via a hidden partial file named for this process, removed if the
-        # write fails; bytes as they are, so no platform newline translation.
+        # via a hidden partial file no other process can predict, created
+        # anew beside the plan (an entry placed at that name by someone
+        # else is never followed), removed if the write fails; bytes as
+        # they are, so no platform newline translation.
         target = Path(os.path.realpath(plan))
-        partial = target.with_name(f".{target.name}.{os.getpid()}.partial")
+        fd, staging = tempfile.mkstemp(
+            dir=target.parent, prefix=f".{target.name}.", suffix=".partial"
+        )
+        partial = Path(staging)
         try:
-            partial.write_bytes(recorded["json"].encode("utf-8"))
+            # A plan replaced keeps the mode of the file it replaces, as the
+            # native publisher does; a new plan stays owner-only.
+            if hasattr(os, "fchmod"):
+                try:
+                    os.fchmod(fd, os.stat(target).st_mode & 0o777)
+                except OSError:
+                    pass
+            with os.fdopen(fd, "wb") as handle:
+                handle.write(recorded["json"].encode("utf-8"))
             os.replace(partial, target)
         except OSError as exc:
             partial.unlink(missing_ok=True)
diff --git a/jubarte-python/tests/test_cli.py b/jubarte-python/tests/test_cli.py
index 62469a0..6d6306e 100644
--- a/jubarte-python/tests/test_cli.py
+++ b/jubarte-python/tests/test_cli.py
@@ -647,6 +647,23 @@ def test_plan_paths_as_typed_and_strict_utf8_as_the_binary(letter: Path, tmp_pat
     assert b"\r\n" not in (tmp_path / "plan.json").read_bytes()
 
 
+def test_recording_does_not_follow_a_preplaced_staging_entry(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
+    # dedup-0028: staging used to be a predictable `.<plan>.<pid>.partial`
+    # path opened by an ordinary truncating write, so a symlink placed at
+    # that name made the recording clobber the file it pointed at. Staging
+    # is now an unpredictable exclusive file; the placed link is left alone.
+    plan = tmp_path / "plan.json"
+    victim = tmp_path / "victim.txt"
+    victim.write_text("precious")
+    trap = tmp_path / f".plan.json.{os.getpid()}.partial"
+    trap.symlink_to(victim)
+    assert main(["edit", str(letter), "--plan", str(plan), "-p", "p1", "--content", "Their."]) == 0
+    capsys.readouterr()
+    assert victim.read_text() == "precious", "a preplaced staging link must not be followed"
+    assert trap.is_symlink(), "the trap link itself is untouched"
+    assert len(json.loads(plan.read_text())["operations"]) == 1, "the plan is recorded"
+
+
 def test_convert_to_md_points_at_a_read_that_works(letter, capsys):
     # pi review av2 F16: `read FILE --track-changes` with no value is a usage error.
     with pytest.raises(SystemExit) as exit:
diff --git a/jubarte-wasm/cli/bin/jubarte-redlines.mjs b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
index 77f5325..2c5572e 100644
--- a/jubarte-wasm/cli/bin/jubarte-redlines.mjs
+++ b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
@@ -13,6 +13,7 @@
 // 3 edit plan refused (its per-operation report is on stdout; nothing was
 // written).
 
+import { randomUUID } from "node:crypto";
 import fs from "node:fs";
 import { createRequire } from "node:module";
 import path from "node:path";
@@ -407,7 +408,7 @@ function recordPlan(verb, file, docx, o) {
   if (!o.quiet) for (const note of recorded.notes) console.log(`note: ${note}`);
   if (!o.dry_run) {
     // Through a symbolic link to the file it names, written yet or not; via a hidden partial
-    // file named for this process, removed if the write fails.
+    // file with an unpredictable name, removed if the write fails.
     let target = o.plan;
     try {
       target = fs.realpathSync(o.plan);
@@ -417,11 +418,24 @@ function recordPlan(verb, file, docx, o) {
         target = path.resolve(path.dirname(o.plan), fs.readlinkSync(o.plan));
       } catch {}
     }
-    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${process.pid}.partial`);
+    // Staging is unpredictable and created exclusively: an entry placed at
+    // any predictable name by someone else is never followed or overwritten.
+    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${randomUUID()}.partial`);
+    let fd;
     try {
-      fs.writeFileSync(partial, recorded.json);
+      fd = fs.openSync(partial, "wx");
+      fs.writeFileSync(fd, recorded.json);
+      fs.closeSync(fd);
+      fd = undefined;
       fs.renameSync(partial, target);
     } catch (e) {
+      if (fd !== undefined) {
+        try {
+          fs.closeSync(fd);
+        } catch {
+          // Already closed; the staging removal below is what matters.
+        }
+      }
       fs.rmSync(partial, { force: true });
       throw new CliError(`writing ${o.plan}: ${osError(e)}`);
     }
diff --git a/jubarte-wasm/cli/test/cli.test.mjs b/jubarte-wasm/cli/test/cli.test.mjs
index ee06baf..6e00250 100644
--- a/jubarte-wasm/cli/test/cli.test.mjs
+++ b/jubarte-wasm/cli/test/cli.test.mjs
@@ -734,6 +734,35 @@ test("--consume refuses an identical-bytes replacement between open and read (in
   assert.ok(!fs.existsSync(out));
 });
 
+test("plan recording stages through an unpredictable exclusive file", () => {
+  // dedup-0028: staging used to be a `.<plan>.<pid>.partial` name opened by
+  // an ordinary truncating write, so a preplaced symlink there made the
+  // recording clobber the file it named. Staging must be created
+  // exclusively (wx) under a name nobody can predict.
+  const plan = path.join(tmp, "staging.json");
+  const result = runWithPlanHook(plan, (fs, mock, assert, plan) => {
+    const opens = [];
+    const open = fs.openSync;
+    mock.method(fs, "openSync", (name, ...args) => {
+      opens.push([String(name), ...args]);
+      return open(name, ...args);
+    });
+    return () => {
+      assert.equal(process.exitCode, 0, "the plan records");
+      const staged = opens.filter(([name]) => name.endsWith(".partial"));
+      assert.ok(staged.length >= 1, "the staging file is created through openSync");
+      for (const [name, flags] of staged) {
+        assert.match(name, /\.[0-9a-f-]{36}\.partial$/, `unpredictable staging name: ${name}`);
+        assert.ok(String(flags).includes("x"), `staging ${name} is created exclusively (${flags})`);
+      }
+    };
+  }, ["edit", untracked, "--plan", plan, "-p", "p1", "--content", "Changed."]);
+  assert.equal(result.code, 0, result.err);
+  const recorded = JSON.parse(fs.readFileSync(plan, "utf8"));
+  assert.ok(Array.isArray(recorded.operations) && recorded.operations.length === 1);
+  assert.deepEqual(fs.readdirSync(tmp).filter((name) => name.endsWith(".partial")), []);
+});
+
 for (const change of ["rename", "overwrite", "append"]) {
   test(`--consume keeps a plan changed by ${change} after applying (integration)`, () => {
     const plan = path.join(tmp, `consume-${change}.json`);
diff --git a/src/admission.rs b/src/admission.rs
index d556610..89f68ab 100644
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -785,7 +785,11 @@ fn part_text(xml: &[u8]) -> Cow<'_, str> {
 /// inside the tag/namespace policy (`xmllinq::parse::TagPolicy`) so no part
 /// reaching the permissive DOM parser can carry pathological attribute or
 /// namespace-declaration widths.
-fn check_xml_depth(name: &str, xml: &[u8], limits: InputLimits) -> Result<(), AdmissionError> {
+pub(crate) fn check_xml_depth(
+    name: &str,
+    xml: &[u8],
+    limits: InputLimits,
+) -> Result<(), AdmissionError> {
     let invalid =
         |why: String| AdmissionError::new(AdmissionErrorKind::InvalidXml, format!("{name}: {why}"));
     let text = part_text(xml);
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f9..8b4bab9 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -291,6 +291,13 @@ const BLOCKS: [&str; 10] = [
     "blockquote",
 ];
 
+/// Word draws at most 63 columns in one table (its own limit), so a
+/// `colspan` above it can never be imported faithfully; it takes the
+/// invalid-span fallback instead of emitting one grid column per claimed
+/// slot, which a single attribute value could ask gigabytes for
+/// (dedup-0028).
+const MAX_TABLE_COLUMNS: usize = 63;
+
 impl Builder {
     fn fmt(&self) -> Fmt {
         self.fmts.last().map(|(_, f)| f.clone()).unwrap_or_default()
@@ -562,8 +569,8 @@ impl Builder {
                 self.cell_spans.push(
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
-                        .unwrap_or(1)
-                        .max(1),
+                        .filter(|&span| (1..=MAX_TABLE_COLUMNS).contains(&span))
+                        .unwrap_or(1),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
@@ -1054,6 +1061,27 @@ mod tests {
                 assert!(!wml.contains("<w:tblBorders>"));
             }
         }
+
+        #[test]
+        fn a_colspan_word_cannot_honor_is_declined_not_expanded() {
+            // dedup-0028: Word's tables stop at 63 columns, so a larger
+            // colspan can never be imported faithfully; it takes the
+            // invalid-span fallback instead of emitting one grid column
+            // per claimed slot (4294967295 of them from one attribute).
+            for span in ["64", "1000000", "4294967295", "18446744073709551615"] {
+                let wml = html_to_wml(&format!(
+                    "<table border='0'><tr><td colspan='{span}'>A</td></tr></table>"
+                ));
+                assert_eq!(wml.matches("<w:gridCol ").count(), 1, "{span}: {wml}");
+                assert!(!wml.contains("<w:gridSpan"), "{span}");
+            }
+            // The widest span Word can draw still spans.
+            let wml = html_to_wml(
+                "<table border='0'><tr><td colspan='63'>A</td><td>B</td></tr></table>",
+            );
+            assert_eq!(wml.matches("<w:gridCol ").count(), 64, "{wml}");
+            assert!(wml.contains("<w:gridSpan w:val=\"63\"/>"), "{wml}");
+        }
     }
 }
 
diff --git a/src/convert/metafile.rs b/src/convert/metafile.rs
index c1c75da..467d18d 100644
--- a/src/convert/metafile.rs
+++ b/src/convert/metafile.rs
@@ -487,17 +487,23 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             break;
         }
         let payload = off + 6;
+        // Every handler reads only its own record's extent (dedup-0028): a
+        // count that reached past it used to consume the bytes of the
+        // records after this one, which the outer loop then revisited, so
+        // a handful of small records could decode gigabytes of points or
+        // text from a file that never held them.
+        let rec = data.get(payload..off + size2)?;
         match func {
             0x0000 => break,
             0x020B => {
-                let y = read_i16(data, payload)? as i32;
-                let x = read_i16(data, payload + 2)? as i32;
+                let y = read_i16(rec, 0)? as i32;
+                let x = read_i16(rec, 2)? as i32;
                 map.org_x = x as f32;
                 map.org_y = y as f32;
             }
             0x020C => {
-                let y = read_i16(data, payload)? as i32;
-                let x = read_i16(data, payload + 2)? as i32;
+                let y = read_i16(rec, 0)? as i32;
+                let x = read_i16(rec, 2)? as i32;
                 if x != 0 {
                     map.ext_x = x as f32;
                 }
@@ -506,8 +512,8 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                 }
             }
             0x02FC => {
-                let style = read_u16(data, payload).unwrap_or(0);
-                let color = colorref(read_u32(data, payload + 2).unwrap_or(0));
+                let style = read_u16(rec, 0).unwrap_or(0);
+                let color = colorref(read_u32(rec, 2).unwrap_or(0));
                 let slot = objects.iter().position(|o| matches!(o, GdiObj::Empty));
                 if let Some(i) = slot {
                     // BS_NULL (1) is hollow: the boxes of 5a6c's reprint
@@ -520,9 +526,9 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                 }
             }
             0x02FA => {
-                let style = read_u16(data, payload).unwrap_or(0);
-                let color = colorref(read_u32(data, payload + 6).unwrap_or(0));
-                let width = read_i16(data, payload + 2).unwrap_or(1) as i32;
+                let style = read_u16(rec, 0).unwrap_or(0);
+                let color = colorref(read_u32(rec, 6).unwrap_or(0));
+                let width = read_i16(rec, 2).unwrap_or(1) as i32;
                 if let Some(i) = objects.iter().position(|o| matches!(o, GdiObj::Empty)) {
                     // PS_NULL (5) strokes nothing: width 0.
                     objects[i] = GdiObj::Pen {
@@ -533,7 +539,7 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             }
             0x02FB => {
                 if let Some(i) = objects.iter().position(|o| matches!(o, GdiObj::Empty)) {
-                    objects[i] = WmfFont::parse(data, payload)
+                    objects[i] = WmfFont::parse(rec, 0)
                         .map_or(GdiObj::Other, |f| GdiObj::Font(std::rc::Rc::new(f)));
                 }
             }
@@ -545,10 +551,10 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                     objects[i] = GdiObj::Other;
                 }
             }
-            0x0106 => winding = read_u16(data, payload).unwrap_or(1) == 2,
+            0x0106 => winding = read_u16(rec, 0).unwrap_or(1) == 2,
             0x012D => {
                 // Object handles are 0-based table indices (b88ac900).
-                let idx = read_u16(data, payload).unwrap_or(0) as usize;
+                let idx = read_u16(rec, 0).unwrap_or(0) as usize;
                 if let Some(obj) = objects.get(idx) {
                     match obj {
                         GdiObj::Brush(c) => dc.brush = Some(*c),
@@ -564,7 +570,7 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             0x001E => saved.push(dc.clone()),
             // RESTOREDC -n pops n states (5a6c's diagram: -1 after each box).
             0x0127 => {
-                let n = read_i16(data, payload).unwrap_or(-1);
+                let n = read_i16(rec, 0).unwrap_or(-1);
                 let keep = if n < 0 {
                     saved.len().saturating_sub(usize::from(n.unsigned_abs()))
                 } else {
@@ -575,16 +581,16 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                     saved.truncate(keep);
                 }
             }
-            0x0209 => dc.text_color = colorref(read_u32(data, payload).unwrap_or(0)),
-            0x012E => dc.text_align = read_u16(data, payload).unwrap_or(0),
+            0x0209 => dc.text_color = colorref(read_u32(rec, 0).unwrap_or(0)),
+            0x012E => dc.text_align = read_u16(rec, 0).unwrap_or(0),
             // RECTANGLE / ROUNDRECT: bottom, right, top, left; filled with
             // the brush, outlined with the pen.
             0x041B | 0x061C => {
-                let at = if func == 0x061C { payload + 4 } else { payload };
-                let bottom = read_i16(data, at)? as i32;
-                let right = read_i16(data, at + 2)? as i32;
-                let top = read_i16(data, at + 4)? as i32;
-                let left = read_i16(data, at + 6)? as i32;
+                let at = if func == 0x061C { 4 } else { 0 };
+                let bottom = read_i16(rec, at)? as i32;
+                let right = read_i16(rec, at + 2)? as i32;
+                let top = read_i16(rec, at + 4)? as i32;
+                let left = read_i16(rec, at + 6)? as i32;
                 let (x0, y0) = map.map(left, top);
                 let (x1, y1) = map.map(right, bottom);
                 let corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
@@ -601,37 +607,37 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             }
             // EXTTEXTOUT: y, x, count, options, [rect], string.
             0x0A32 => {
-                let y = read_i16(data, payload)?;
-                let x = read_i16(data, payload + 2)?;
-                let n = usize::from(read_u16(data, payload + 4)?);
-                let opts = read_u16(data, payload + 6)?;
-                let at = payload + 8 + if opts & 0x0006 != 0 { 8 } else { 0 };
-                if let Some(bytes) = data.get(at..at + n) {
+                let y = read_i16(rec, 0)?;
+                let x = read_i16(rec, 2)?;
+                let n = usize::from(read_u16(rec, 4)?);
+                let opts = read_u16(rec, 6)?;
+                let at = 8 + if opts & 0x0006 != 0 { 8 } else { 0 };
+                if let Some(bytes) = rec.get(at..at + n) {
                     texts.extend(wmf_text(&dc, &map, x, y, bytes));
                 }
             }
             // TEXTOUT: count, string (word-padded), y, x.
             0x0521 => {
-                let n = usize::from(read_u16(data, payload)?);
-                let at = payload + 2 + n.next_multiple_of(2);
-                let (y, x) = (read_i16(data, at)?, read_i16(data, at + 2)?);
-                if let Some(bytes) = data.get(payload + 2..payload + 2 + n) {
+                let n = usize::from(read_u16(rec, 0)?);
+                let at = 2 + n.next_multiple_of(2);
+                let (y, x) = (read_i16(rec, at)?, read_i16(rec, at + 2)?);
+                if let Some(bytes) = rec.get(2..2 + n) {
                     texts.extend(wmf_text(&dc, &map, x, y, bytes));
                 }
             }
             0x01F0 => {
-                let idx = read_u16(data, payload).unwrap_or(0) as usize;
+                let idx = read_u16(rec, 0).unwrap_or(0) as usize;
                 if let Some(slot) = objects.get_mut(idx) {
                     *slot = GdiObj::Empty;
                 }
             }
             0x0324 => {
-                let n = read_u16(data, payload).unwrap_or(0) as usize;
+                let n = read_u16(rec, 0).unwrap_or(0) as usize;
                 let mut pts = Vec::with_capacity(n);
-                let mut p = payload + 2;
+                let mut p = 2;
                 for _ in 0..n {
-                    let x = read_i16(data, p)? as i32;
-                    let y = read_i16(data, p + 2)? as i32;
+                    let x = read_i16(rec, p)? as i32;
+                    let y = read_i16(rec, p + 2)? as i32;
                     pts.push(map.map(x, y));
                     p += 4;
                 }
@@ -642,15 +648,15 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             // META_POLYPOLYGON: ring count, each ring's point count, then
             // every ring's points, filled as one shape (b88ac900's logo).
             0x0538 => {
-                let rings = read_u16(data, payload).unwrap_or(0) as usize;
-                let mut p = payload + 2 + 2 * rings;
+                let rings = read_u16(rec, 0).unwrap_or(0) as usize;
+                let mut p = 2 + 2 * rings;
                 let mut subpaths = Vec::with_capacity(rings);
                 for r in 0..rings {
-                    let n = read_u16(data, payload + 2 + 2 * r)? as usize;
+                    let n = read_u16(rec, 2 + 2 * r)? as usize;
                     let mut ring = Vec::with_capacity(n);
                     for _ in 0..n {
-                        let x = read_i16(data, p)? as i32;
-                        let y = read_i16(data, p + 2)? as i32;
+                        let x = read_i16(rec, p)? as i32;
+                        let y = read_i16(rec, p + 2)? as i32;
                         ring.push(map.map(x, y));
                         p += 4;
                     }
@@ -665,12 +671,12 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                     off += size2;
                     continue;
                 };
-                let n = read_u16(data, payload).unwrap_or(0) as usize;
+                let n = read_u16(rec, 0).unwrap_or(0) as usize;
                 let mut prev: Option<(i32, i32)> = None;
-                let mut p = payload + 2;
+                let mut p = 2;
                 for _ in 0..n {
-                    let x = read_i16(data, p)? as i32;
-                    let y = read_i16(data, p + 2)? as i32;
+                    let x = read_i16(rec, p)? as i32;
+                    let y = read_i16(rec, p + 2)? as i32;
                     let cur = map.map(x, y);
                     if let Some(pr) = prev {
                         canvas.stroke_line(pr.0, pr.1, cur.0, cur.1, pen, map.pen_px(pen_w));
@@ -1900,6 +1906,101 @@ mod wmf_object_tests {
         }
         assert_eq!(render(&bytes[..bytes.len() - 6]).unwrap(), expected);
     }
+
+    #[test]
+    fn a_text_record_may_not_read_string_bytes_past_its_extent() {
+        // dedup-0028: the record declares fifteen string bytes it does not
+        // carry; the reader used to take them from the record after it
+        // (its header, then A NUL B NUL C NUL D NUL E — never a contiguous
+        // "ABCDEFGHI"), whose bytes the outer loop then read again. A
+        // record's own extent is all it may read, so the record is
+        // declined and writes nothing.
+        let bytes = wmf(
+            &[
+                (0x020B, vec![0, 0]),
+                (0x020C, vec![100, 100]),
+                font(-25, 400, false, "Arial"),
+                (0x012D, vec![0]),
+                // EXTTEXTOUT whose count (15) passes its own payload.
+                (0x0A32, vec![20, 40, 15, 0]),
+                // The record whose header and payload the count reached into.
+                (
+                    0x06FF,
+                    "ABCDEFGHI"
+                        .bytes()
+                        .map(u16::from)
+                        .collect::<Vec<u16>>(),
+                ),
+            ],
+            2,
+        );
+        let (_, _, _, texts) = render(&bytes).expect("wmf");
+        assert!(
+            texts.is_empty(),
+            "a record's string may not come from the bytes after it: {texts:?}"
+        );
+        // The same record carrying its own string still writes it.
+        let bytes = wmf(
+            &[
+                (0x020B, vec![0, 0]),
+                (0x020C, vec![100, 100]),
+                font(-25, 400, false, "Arial"),
+                (0x012D, vec![0]),
+                ext_text(40, 20, "Visible"),
+            ],
+            2,
+        );
+        let (_, _, _, texts) = render(&bytes).expect("wmf");
+        assert_eq!(texts.len(), 1);
+        assert_eq!(texts[0].text, "Visible");
+    }
+
+    #[test]
+    fn a_polygon_record_may_not_read_points_past_its_extent() {
+        // dedup-0028: the record declares three points and carries one; the
+        // missing two used to be read from the record after it. Short of
+        // its own extent, the record refuses the picture instead of
+        // painting one the file never described.
+        let mut bytes = wmf(
+            &[
+                brush(0, 0, 0),
+                (0x012D, vec![0]),
+                // POLYGON: n = 3, then only the first point in the record.
+                (0x0324, vec![3, 20, 20]),
+            ],
+            2,
+        );
+        // Drop the EOF the helper appends, put a record where the missing
+        // points used to be read from, then EOF again.
+        bytes.truncate(bytes.len() - 6);
+        bytes.extend_from_slice(&(3 + 6_u32).to_le_bytes());
+        bytes.extend_from_slice(&0x06FF_u16.to_le_bytes());
+        for word in [80_u16, 20, 50, 80, 20, 50] {
+            bytes.extend_from_slice(&word.to_le_bytes());
+        }
+        bytes.extend_from_slice(&3u32.to_le_bytes());
+        bytes.extend_from_slice(&0u16.to_le_bytes());
+        assert!(
+            render(&bytes).is_none(),
+            "a polygon short of its declared points refuses the picture"
+        );
+        // The same record carrying all three points still fills.
+        let bytes = wmf(
+            &[
+                brush(0, 0, 0),
+                (0x012D, vec![0]),
+                (0x0324, vec![3, 20, 20, 80, 20, 50, 80]),
+            ],
+            2,
+        );
+        let (w, h, rgb, _) = render(&bytes).expect("wmf");
+        let at = |x: u32, y: u32| {
+            let i = ((y * h / 100) * w + x * w / 100) as usize * 3;
+            [rgb[i], rgb[i + 1], rgb[i + 2]]
+        };
+        assert_eq!(at(30, 30), [0, 0, 0], "the triangle fills");
+        assert_eq!(at(2, 2), [255, 255, 255], "outside stays white");
+    }
 }
 
 #[cfg(test)]
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
index d900e05..9079178 100644
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -20782,7 +20782,9 @@ fn load_chrome_part(
 }
 
 /// A header or footer part laid out from its xml; `path` resolves its
-/// relationships (pictures, text boxes).
+/// relationships (pictures, text boxes). The xml runs through the
+/// package's XML budgets first: a part over them lays out nothing, like a
+/// part that does not parse.
 fn chrome_part_xml(
     pkg: &PartFs,
     path: &str,
@@ -20791,6 +20793,20 @@ fn chrome_part_xml(
     sheet: &StyleSheet,
     text_w: f32,
 ) -> ChromePart {
+    // Admission's XML scan covers `.xml`/`.rels` part names (and the main
+    // part under any name); a header or footer relationship can select any
+    // other name, and the DOM builder recurses per nesting level
+    // (dedup-0028). Whatever the part is called, its budgets are enforced
+    // before that builder runs.
+    if crate::admission::check_xml_depth(
+        path,
+        xml.as_bytes(),
+        crate::admission::InputLimits::default(),
+    )
+    .is_err()
+    {
+        return empty_chrome();
+    }
     let mut part_dom = Dom::new();
     let doc = part_dom.parse_xdocument(xml);
     let Some(root) = part_dom.root(doc) else {
@@ -21229,6 +21245,84 @@ fn chrome_part_xml(
     }
 }
 
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod chrome_xml_budget_tests {
+    use super::*;
+
+    fn package() -> PartFs {
+        use std::io::{Cursor, Write};
+        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
+        zip.start_file(
+            "[Content_Types].xml",
+            zip::write::SimpleFileOptions::default(),
+        )
+        .unwrap();
+        zip.write_all(
+            br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>"#,
+        )
+        .unwrap();
+        PartFs::open(&zip.finish().unwrap().into_inner()).unwrap()
+    }
+
+    fn sheet() -> StyleSheet {
+        StyleSheet {
+            defaults: Defaults::word(),
+            by_id: HashMap::new(),
+            tables: HashMap::new(),
+            theme: ThemeFonts::default(),
+            latent: true,
+            default_table: true,
+        }
+    }
+
+    fn header(inner: &str) -> String {
+        format!(
+            r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{inner}</w:hdr>"#
+        )
+    }
+
+    /// A relationship can name a header part anything; whatever the name,
+    /// its XML meets the package's budgets before the DOM is built.
+    #[test]
+    fn a_renamed_header_part_over_the_xml_budget_lays_out_nothing() {
+        let pkg = package();
+        let sheet = sheet();
+        let run = "<w:p><w:r><w:t>Header</w:t></w:r></w:p>";
+        // 300 nested run-level elements sit past the 256-element depth
+        // budget; the unfixed read built their DOM and laid the run out.
+        let deep = header(&format!(
+            "<w:p>{}<w:r><w:t>Header</w:t></w:r>{}</w:p>",
+            "<w:smartTag>".repeat(300),
+            "</w:smartTag>".repeat(300)
+        ));
+        let chrome = chrome_part_xml(
+            &pkg,
+            "word/header1.bin",
+            &deep,
+            "headerReference",
+            &sheet,
+            468.0,
+        );
+        assert!(
+            chrome.runs.is_empty(),
+            "an over-budget part lays out nothing"
+        );
+        // A header within the budgets, under the same renamed part, still
+        // lays out its run.
+        let chrome = chrome_part_xml(
+            &pkg,
+            "word/header1.bin",
+            &header(run),
+            "headerReference",
+            &sheet,
+            468.0,
+        );
+        let text: String = chrome.runs.iter().map(|r| r.text.as_str()).collect();
+        assert!(text.contains("Header"), "a part within budget keeps its text");
+    }
+}
+
 /// A header's flow picture wider than the room beside a square-wrapped
 /// picture of its paragraph starts under that picture, its wrap distance
 /// and effect extent included (0800162a66 vs 000e3e7b: Word sets the
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
index 1881053..3fd5bc9 100644
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -1139,12 +1139,21 @@ struct StoryChar {
     modifier: CharModifier,
 }
 
+/// Most characters a `.doc` main story may hold, declared or expanded
+/// (dedup-0028): 8 Mi characters (16 MiB of UTF-16 text, more than any
+/// document the corpora carry) caps the retained per-character metadata at
+/// ~128 MiB, the scale of `admission`'s package budgets, while piece
+/// tables that alias a small physical range into gigabytes of logical
+/// story are refused before they are materialized.
+const MAX_STORY_CHARS: u32 = 1 << 23;
+
 /// The main story's characters, CP 0 to `ccp_text`.
 ///
 /// # Errors
 ///
 /// A piece that points past the `WordDocument` stream: a truncated or
-/// corrupt file, refused rather than converted short.
+/// corrupt file, refused rather than converted short. A declared or
+/// piece-expanded story longer than [`MAX_STORY_CHARS`].
 fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>> {
     let past = || LegacyDocError::new("a text piece points past the WordDocument stream");
     // The pieces run from CP 0 through the main story without a gap, or
@@ -1173,16 +1182,36 @@ fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryCh
             "the piece table does not cover the main story",
         ));
     }
+    // `ccp_text` is a caller-controlled u32 and the pieces may alias one
+    // physical range many times over, so the file's size bounds neither
+    // the declared nor the expanded story. Both pay this product policy
+    // before any character is materialized: every StoryChar costs memory
+    // the physical bytes never paid for.
+    if ccp_text > MAX_STORY_CHARS {
+        return Err(LegacyDocError::new(
+            "the main story is longer than the reader's character budget",
+        ));
+    }
     let mut out = Vec::new();
     // A high surrogate waiting for its low half, which may open the next
     // Unicode piece (a supplementary character split between two pieces).
     // Without one it is a replacement mark at its own position.
     let mut pending_high: Option<(u16, StoryChar)> = None;
+    let mut logical = 0u64;
     for piece in pieces {
         if piece.cp_start >= ccp_text {
             break;
         }
         let end = piece.cp_end.min(ccp_text);
+        // Charged before the piece expands: overlapping pieces that reuse
+        // one physical range are refused as their running total passes the
+        // cap, not after allocating it all.
+        logical = logical.saturating_add(u64::from(end.saturating_sub(piece.cp_start)));
+        if logical > u64::from(MAX_STORY_CHARS) {
+            return Err(LegacyDocError::new(
+                "the main story expands past the reader's character budget",
+            ));
+        }
         let width: u32 = if piece.compressed { 1 } else { 2 };
         for cp in piece.cp_start..end {
             let fc = cp
@@ -1594,6 +1623,11 @@ impl Story<'_> {
         // Field nesting: true while inside a field's code (before its
         // separator).
         let mut fields: Vec<bool> = Vec::new();
+        // Frames still in their code, as a count: "is any frame in its
+        // code" is answered in O(1) instead of rescanning the stack for
+        // every character (a document of begin/separator pairs made that
+        // scan quadratic; dedup-0028).
+        let mut code_frames = 0usize;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
@@ -1601,20 +1635,26 @@ impl Story<'_> {
 
         for story in chars {
             // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
+            let in_code = code_frames > 0;
             let text = match story.ch {
                 '\u{13}' => {
                     fields.push(true);
+                    code_frames = code_frames.saturating_add(1);
                     continue;
                 }
                 '\u{14}' => {
                     if let Some(top) = fields.last_mut() {
-                        *top = false;
+                        if *top {
+                            code_frames = code_frames.saturating_sub(1);
+                            *top = false;
+                        }
                     }
                     continue;
                 }
                 '\u{15}' => {
-                    fields.pop();
+                    if fields.pop() == Some(true) {
+                        code_frames = code_frames.saturating_sub(1);
+                    }
                     continue;
                 }
                 _ if in_code => continue,
@@ -1967,6 +2007,75 @@ mod tests {
         assert_eq!(texts, ["aoneb"]);
     }
 
+    #[test]
+    fn unbalanced_field_marks_keep_their_meanings() {
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        let texts = |chars: &[StoryChar]| {
+            story
+                .blocks(chars)
+                .iter()
+                .map(|b| match b {
+                    Block::Paragraph(p) => p.text(),
+                    Block::Table(_) => "table".into(),
+                })
+                .collect::<Vec<_>>()
+        };
+        // A separator or an end with no field open is nothing; a field
+        // closed without ever showing a result keeps the text around it.
+        assert_eq!(texts(&story_chars("\u{14}a\u{15}\u{13}b\u{15}c\r")), ["ac"]);
+        // Code before the separator is hidden; the result and what follows
+        // the field are shown.
+        assert_eq!(texts(&story_chars("\u{13}x\u{14}y\u{15}z\r")), ["yz"]);
+    }
+
+    #[test]
+    fn a_deep_field_stack_costs_one_step_per_character() {
+        // dedup-0028: every begin pushes a frame and every separator parks
+        // it in its result, and each later character asked "is any frame in
+        // its code?" by rescanning the stack: 100_000 pairs made that
+        // ~5e9 frame visits. The count answers in O(1).
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        let mut deep: Vec<StoryChar> = Vec::with_capacity(200_002);
+        for cp in 0..100_000u32 {
+            for control in ['\u{13}', '\u{14}'] {
+                deep.push(StoryChar {
+                    ch: control,
+                    cp,
+                    fc: cp,
+                    modifier: CharModifier::default(),
+                });
+            }
+        }
+        deep.extend(story_chars("b\r"));
+        let started = std::time::Instant::now();
+        let blocks = story.blocks(&deep);
+        assert!(started.elapsed() < std::time::Duration::from_secs(2));
+        let texts: Vec<String> = blocks
+            .iter()
+            .map(|b| match b {
+                Block::Paragraph(p) => p.text(),
+                Block::Table(_) => "table".into(),
+            })
+            .collect();
+        assert_eq!(texts, ["b"]);
+    }
+
     /// A supplementary character split across two Unicode pieces is one
     /// character, as Word reads it; a lone high surrogate is a replacement
     /// mark, never dropped.
@@ -2048,6 +2157,70 @@ mod tests {
         assert_eq!(main_text(&word, &empty, 10).unwrap().len(), 10);
     }
 
+    #[test]
+    fn aliased_pieces_expanding_past_the_budget_are_refused() {
+        // dedup-0028: 1024 pieces that each alias one 64 KiB compressed
+        // range declare a 64 Mi-character story from a 64 KiB stream and a
+        // piece table smaller than the fixtures. The budget refuses the
+        // declaration before a character is materialized (67_108_864
+        // StoryChars is over a gigabyte of metadata on the unfixed read).
+        let word = vec![b'a'; 65_536];
+        let piece = |cp_start, cp_end| Piece {
+            cp_start,
+            cp_end,
+            fc: 0,
+            compressed: true,
+            modifier: CharModifier::default(),
+        };
+        let pieces: Vec<Piece> = (0..1024u32)
+            .map(|i| piece(i * 65_536, (i + 1) * 65_536))
+            .collect();
+        let error = main_text(&word, &pieces, 1024 * 65_536)
+            .unwrap_err()
+            .to_string();
+        assert!(error.contains("character budget"), "{error}");
+    }
+
+    #[test]
+    fn overlapping_restarts_are_charged_to_the_cumulative_budget() {
+        // The declared story (3 Mi characters) is within the budget; three
+        // pieces that all restart at CP 0 expand it to 9 Mi and are refused
+        // as the running total passes the cap, not after allocating it all.
+        let span = 3 * 1024 * 1024;
+        let word = vec![b'a'; span];
+        let piece = Piece {
+            cp_start: 0,
+            cp_end: span as u32,
+            fc: 0,
+            compressed: true,
+            modifier: CharModifier::default(),
+        };
+        let error = main_text(&word, &[piece, piece, piece], span as u32)
+            .unwrap_err()
+            .to_string();
+        assert!(error.contains("character budget"), "{error}");
+    }
+
+    #[test]
+    fn fast_save_aliases_within_the_budget_still_expand() {
+        // Two logical ranges reading the same physical bytes (a fast save's
+        // piece table can point both at one range) stay one document.
+        let word = b"abcd".to_vec();
+        let piece = |cp_start, cp_end, fc| Piece {
+            cp_start,
+            cp_end,
+            fc,
+            compressed: true,
+            modifier: CharModifier::default(),
+        };
+        let text: String = main_text(&word, &[piece(0, 4, 0), piece(4, 8, 0)], 8)
+            .unwrap()
+            .iter()
+            .map(|c| c.ch)
+            .collect();
+        assert_eq!(text, "abcdabcd");
+    }
+
     #[test]
     fn a_property_table_outside_its_stream_is_refused_not_dropped() {
         let table = vec![0u8; 64];

```
