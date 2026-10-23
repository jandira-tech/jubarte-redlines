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
`rec.get(at..at+n)` guard leaves that one record unwritten) or refuse the
render (`?` on short fixed reads, and the same `?` when a declared
ring/point lies past `rec` — the polygon test asserts exactly
`render(...).is_none()`). EMF and the placeable/header reads are
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
