# dedup-0001 — remediation plan (TDD red/green)

Scan round: `.scan/artifacts/deep_discovery/dedup/dedup-0001/output/result.json`
(12 findings; `claimedWorkerIds` 3d0b17d0…:0..4 and 8e5f16fa…:0..10). No
`dedup0001`-prefixed evidence refs exist in the round's output (verified:
`grep -o 'dedup[-_]0001[a-z0-9-]*' result.json threatmodel.md` finds nothing),
so all 12 findings of this round's aggregate are treated as this round's own.

Every finding below was re-verified against the current source at
`/Users/arthrod/temp/T/jubarte-redlines` (paths and line numbers are from the
files as read; the patch carries exact hunks). All are availability issues
(memory/CPU/stack amplification) reachable from untrusted documents through
public parser boundaries; none grants code execution or file access.

**No build or test command was run while preparing this patch** (the ask
forbids concurrent Cargo in this checkout). Tests are authored by reading the
code; the red/green claims state what each test demonstrates and are executed
only when the patch is applied and the suite is run. Commands to run then:

```
cargo nextest run -E 'test(chicago) or test(wmf) or test(eocd) or test(pieces) or test(field) or test(script) or test(reference) or test(depth) or test(colspan) or test(gridspan) or test(grid_before) or test(tab_leader) or test(theme)'
cargo nextest run --test resource_limits
cargo nextest run -E 'test(alt_chunk) or test(piece_property) or test(outer_result) or test(budgets_admit) or test(a_plain_docx)'
```

then the module suites the touched files own (`src/xmllinq`, `src/legacy_doc`,
`src/admission`, `src/convert`, `src/markdown`), `cargo fmt --all` (the new
`use` list in `page_num_fmt_labels` and the test modules were hand-formatted),
clippy with `-D warnings`, and a `jubarte --help` smoke test per AGENTS.md.

---

## 1. `w:gridSpan`/`w:gridBefore`/`w:gridAfter` table allocations (CWE-789)

**Verified.** `cell_span` (`src/convert/mod.rs:14204-14224`) parses `w:gridSpan`
as `usize` with only `.max(1)`; `row_grid_skip` (`:14340-14353`) parses
`w:gridBefore`/`w:gridAfter` with only `*n > 0`; `table_block` sums row
occupancy unchecked (`:13601-13616`) and sizes `cols` with
`vec![each; occupancy]` / `cols.resize(occupancy.max(1), 80.0)`;
`resolve_table_merges` (`:14571-14645`) resizes its merge vector with
`col + span` and advances `col += span` unchecked. A one-cell table with
`w:gridSpan w:val="1000000000"` requests a ~4 GiB `Vec<f32>` from a ~1 KiB
package that passes every admission budget.

**Failing test (red).** `tests/resource_limits.rs`
- `a_huge_gridspan_renders_within_budget` — one-row table, single cell with
  `w:gridSpan w:val="1000000000"`; asserts `docx_to_pdf(&docx(...))` is `Ok`
  and one page. Pre-fix the render aborts on the gigabyte `vec!`/`resize`.
- `a_huge_grid_before_renders_within_budget` — same with
  `w:gridBefore w:val="2000000000"` (the placeholder-span path,
  `grid_skip_cell`).

**Minimal source change.** `src/convert/mod.rs`:
- new documented budgets beside `cell_span`:
  `pub(crate) const MAX_GRID_SPAN: usize = 1024` (a single span; Word tables
  top out at 63 columns) and `pub(crate) const MAX_TABLE_COLUMNS: usize = 4096`
  (columns a rendered grid may hold);
- `cell_span`: `.unwrap_or(1).max(1)` → `… .min(MAX_GRID_SPAN)`;
- `row_grid_skip`: `.filter(|n| *n > 0)` → then `.map(|n| n.min(MAX_GRID_SPAN))`;
- occupancy: per-row `.sum()` → `.fold(0usize, usize::saturating_add)` and the
  running max clamped with `.min(MAX_TABLE_COLUMNS)` before any `resize`;
- `resolve_table_merges`: `let span = raw.colspan.max(1)` →
  `.clamp(1, MAX_GRID_SPAN)`; both `col += span` sites become
  `col = col.saturating_add(span).min(MAX_TABLE_COLUMNS)` and the
  `open.resize`/`take` use that clamped end.

**Existing behaviour preserved.** Word's own tables hold at most 63 columns,
65× under the grid budget; every corpus fixture and the `m38_table_alignment`
family render identically. Only spans > 1024 or aggregate grids > 4096 —
impossible in Word-valid documents — clamp.

## 2. Auxiliary XML parts under non-XML names bypass the depth gate

**Verified.** Admission depth-checks only parts whose *name* ends `.xml` or
`.rels` (`src/admission.rs:533-543`, plus the main part whatever its name,
`:556-570`). Rendering selects parts by *relationship* regardless of name
(`main_rel_part`, `src/convert/mod.rs:21310-21320`) and parses them with the
recursive DOM builder: theme `:3305-3317`, styles `:3480-3497`, numbering
`:6557+`, headers/footers, comments, notes. A deeply nested
`word/theme1.bin` (or `styles.bin`) admitted without any XML policy reaches
`Parser::parse_element`'s unbounded recursion (`src/xmllinq/parse.rs:660-663`).

**Failing test (red).** `tests/resource_limits.rs`
- `a_renamed_deeply_nested_theme_part_renders_bounded` — package built with
  `tests/common/docx.rs::docx_with` carrying `word/theme1.bin` (Override
  content type `…theme+xml`, relationship type `…/relationships/theme`) whose
  XML nests `<a:e>` 300 000 deep; asserts `docx_to_pdf` is `Ok` with one page.
  Pre-fix `load_theme`'s `parse_xdocument` recurses 300 000 frames and takes
  the stack.

**Minimal source change.** Shared with findings 3 and 12 (one sink): a
construction-depth cap in `src/xmllinq/parse.rs` —
`const MAX_DOM_DEPTH: usize = 512` (twice the strictest built-in admission
profile of 256, far under any thread stack), a `depth: usize` counter on
`Parser`, and in `parse_element`, after an open tag is consumed, an element at
the cap is closed as a leaf (its would-be children become siblings of the
capped node, mirroring the existing `/>` early return). Parser tests below
assert the bound.

**Existing behaviour preserved.** Every admitted part nests ≤ 256 < 512, so no
admitted document can hit the cap; the M1.8 byte-identical round-trip and all
corpus parsing are unchanged. Callers that raise `max_xml_depth` beyond 512
get truncation where they used to risk a stack overflow (documented on the
constant).

## 3. Shallow `<p a=//>` quote confusion overflows the DOM parser stack

**Verified.** quick-xml classifies a tag by its raw suffix
(`strip_suffix(b"/>")`, zip-crate neighbour `quick-xml-0.42.0`
`src/reader/mod.rs` `emit_start`), so admission's `check_xml_depth`
(`src/admission.rs:788-823`) sees `<p a=//>` as `Event::Empty` — N siblings at
depth 0. The permissive parser (`src/xmllinq/parse.rs:574-585`) takes *any*
byte after `=` as the value quote: the first `/` opens, the second closes an
empty value, the tag is then an open element (`:624`), and every following
sibling nests one level deeper (`:660-663`).

**Failing test (red).** `src/xmllinq/parse.rs`, new `dom_depth_cap_tests`
- `quote_confused_siblings_build_a_bounded_tree` — `"<p a=//>".repeat(200_000)`
  parses to a DOM whose measured depth ≤ `MAX_DOM_DEPTH + 1`. Pre-fix the
  200 000-deep recursion aborts the test process.
- `over_deep_nesting_is_truncated_at_the_cap` — `"<a>"×5_000` parses with
  depth ≤ cap (the same sink unadmitted input finds through finding 2's and
  12's routes).
- `admitted_depth_documents_keep_their_full_tree` — 256-deep nesting still
  parses to depth 256 exactly (the cap never fires for admitted input).

**Minimal source change.** The `MAX_DOM_DEPTH` cap of finding 2 (the
remediation's "enforce a real construction-depth limit" option; making the
tolerant tokenizer byte-identical to quick-xml's for every malformed shape is
not achievable without changing what real-world WW-isms the parser accepts).

**Existing behaviour preserved.** Documents at admission depth (≤ 256) parse
to exactly the tree they do today; the parser's documented tolerance for
unquoted/mixed-quote attributes is untouched.

## 4. Legacy DOC field stack is rescanned per character (quadratic CPU)

**Verified.** `Story::blocks` (`src/legacy_doc.rs:1591-1676`) computes
`let in_code = fields.iter().any(|&code| code)` at `:1604` for **every**
character, and `\u{13}` field begins with no ends leave an all-`true` stack:
N begins followed by M characters is Θ(N·M). A ~100 KiB crafted `.doc` makes
`read`/`doc_to_markdown`/`doc_to_docx` (public, `src/legacy_doc.rs:180,225,235`)
and the CLI's Word-input hint run for minutes.

**Failing tests (red).** `src/legacy_doc.rs` in-file tests module
- `bare_field_begins_consume_linear_work` — 100 000 `\u{13}` begins followed
  by `"...text...\r"`; asserts the blocks hold the trailing paragraph text
  (the run must finish; pre-fix the same input costs ~10^10 boolean scans —
  minutes — so the test practically cannot pass. This red is a runtime
  demonstration, not an assertion failure; noted here so nobody mistakes it
  for a deterministically-failing assert).
- `field_code_suppression_is_unchanged_for_mixed_nesting` — begin/separator/
  end sequences incl. code-after-separator, nested results and unbalanced
  ends produce the same visible text as today (asserts the exact strings the
  O(1) rewrite must keep; passes before and after — the refactor's safety
  net, beside the existing `a_field_nested_in_a_field_code_shows_only_the_outer_result`).

**Minimal source change.** `src/legacy_doc.rs:1596-1620`: keep the `fields`
stack, add `let mut open_code: usize` (frames currently in their code
section); begin `+1`; separator flips the top only when it was `true` (`-1`);
end pops and `-1` when the popped frame was `true`; `in_code = open_code > 0`.
Bit-identical semantics to `any()`, O(1) per character. No nesting budget is
added: the stack itself is one `bool` per input begin — memory linear in the
input, like every other per-character structure there; the vulnerability was
the quadratic rescan.

**Existing behaviour preserved.** `_ if in_code => continue` and every other
arm see the same boolean; the existing nested-field test and the corpus
`.doc` conversions are unchanged.

## 5. `chicago_label` repeats its mark ~n/4 times unbounded

**Verified.** `chicago_label(n: u32)` (`src/convert/mod.rs:6015-6019`) does
`MARKS[i % 4].repeat(i / 4 + 1)`; `w:start`/`startOverride`/`pgNumType@w:start`
reach it as `u32` through `format_num` (`:5861`) and `section_page_label`
(`:32032`), and the Markdown views through `list_number` (`:5849`). A
Chicago start of `u32::MAX` asks for a ~2 GiB `String` (3-byte UTF-8 marks)
before any later clipping.

**Failing test (red).** `src/convert/mod.rs` `page_num_fmt_labels` tests
- `chicago_labels_stay_bounded_at_extreme_counters` — `chicago_label` at
  `u32::MAX`, `u32::MAX - 1`, `1 << 30` returns ≤ `CHICAGO_LABEL_MAX_REPEATS`
  marks, all from `*†‡§`; the exact small values (`1, 2, 4, 5, 8, 9`) keep
  today's strings. Pre-fix the first assert cannot even allocate.

**Minimal source change.** `chicago_label` clamps the repeat count to a new
`pub(crate) const CHICAGO_LABEL_MAX_REPEATS: usize = 32`, in the file's own
tradition (`ALPHA_LABEL_MAX` "past thirty z's a label starts again",
`ROMAN_LABEL_MAX`): labels up to 128 marks (value 512) are byte-identical;
past that the repetition holds at 32 marks. `w:start` is attacker paper, not
a Word-reachable value — Word's own list counters never reach 512 Chicago
marks (its labels restart long before), so no Word-faithful output changes.

**Existing behaviour preserved.** `chicago_cycles_star_dagger_double_section`,
`zero_labels_remain_explicit…`, and the `page_field_uses_sectpr_chicago`
render probe (start=1 → `*`) all still pass.

## 6. altChunk HTML `colspan` expands to a gridCol per column

**Verified.** `src/convert/altchunk.rs:558-567` parses `colspan` as `usize`
with `.max(1)` only; `end_table` (`:469-491`) sets `cols` to the rows'
unchecked span **sum** and then writes one `<w:gridCol>` per column plus
`col_w * cell.span`. `colspan="1000000000"` in a 60-byte HTML chunk generates
gigabytes of WML before the body is ever parsed.

**Failing test (red).** `src/convert/altchunk.rs` `tests`
- `a_huge_html_colspan_is_capped_not_expanded` —
  `html_to_wml` on a one-cell table with `colspan="1000000000"` returns WML
  with at most `crate::convert::MAX_TABLE_COLUMNS` `<w:gridCol>` elements and
  a capped `w:gridSpan`. Pre-fix the generated string alone exhausts memory.

**Minimal source change.** `src/convert/altchunk.rs`: the colspan parse
clamps with `.min(super::MAX_GRID_SPAN)`; `end_table`'s sum becomes
`.fold(0usize, usize::saturating_add)` and the result
`.min(super::MAX_TABLE_COLUMNS)` — the same two budgets finding 1 introduced,
applied at the HTML door.

**Existing behaviour preserved.** `an_alt_chunk_is_replaced_by_its_paragraphs`
and the regression_tests module (real chunks, small colspans) are unchanged.

## 7. Tab leaders repeat a mark for an attacker-sized interval

**Verified.** `parse_tab_stops` (`src/convert/mod.rs:3243-3266`) accepts any
`f32` for `w:pos` (including `1e22`/inf/NaN). `paint_tab_leader`
(`:26381-26402`) computes `n = (last - first) as usize` from unclamped
coordinates — `x1` is the mapped stop — and calls `mark.repeat(n)`. A
`w:pos="1e22"` left stop with a dot leader saturates `n` to `usize::MAX` and
aborts on capacity overflow; a merely huge stop allocates hundreds of MiB of
leader glyphs even though the stop lies far outside any page.

**Failing test (red).** `tests/resource_limits.rs`
- `a_huge_tab_leader_renders_bounded` — paragraph with
  `<w:tab w:val="left" w:leader="dot" w:pos="1e22"/>` plus a `<w:tab/>` run;
  asserts `docx_to_pdf` is `Ok` with one page. Pre-fix the render aborts
  inside `repeat`.

**Minimal source change.** `src/convert/mod.rs`:
- `parse_tab_stops` drops nonfinite positions
  (`.filter(|pos| pos.is_finite())`) — "reject nonfinite coordinates";
- `paint_tab_leader` returns on nonfinite `x0`/`x1`, clamps `first` at ≥ 0
  and `last` at a new `const MAX_TAB_LEADER_CELLS: f32 = 8192.0` (~22 500 pt
  of leader at the narrowest 2.75 pt cell — beyond any physical page, so
  nothing Word prints is clipped) before `repeat`.

**Existing behaviour preserved.** The Word 16 dot-leader probes (leaders of a
few hundred cells at most) and every tab fixture render byte-identically.

## 8. Flat LaTeX script chains build an unbounded AST the OMML writer walks

**Verified.** `nested_list` (`src/markdown/math.rs:867-914`) wraps the
previous node on every bare `^`/`_` (`take_base` → `scripts`), never passing
the `MOST_NESTING`-guarded `list` (`:833-841`) — `a^1^1^1…` builds a chain as
deep as the input. The OMML writer recurses per node
(`node` → `argument` → `nodes`, `:1722-1780`), so a long chain exhausts the
stack (and traps a WASM instance).

**Failing test (red).** `src/markdown/math.rs` `tests`
- `a_flat_script_chain_is_bounded` — `omml` on `a` + `"^1".repeat(20_000)`
  returns XML with at most `MOST_SCRIPT_WRAPS` `<m:sSup>` elements (and a
  3-wrap chain keeps exactly 3). Pre-fix the writer recursion overflows.

**Minimal source change.** `src/markdown/math.rs`: new
`const MOST_SCRIPT_WRAPS: usize = 8` beside `MOST_NESTING` (chained bare
scripts are already invalid LaTeX — "double superscript" — past the first);
`nested_list` counts consecutive wraps, resets on any atom, and past the
budget consumes the `^`/`_` and appends its argument as plain sibling nodes
instead of wrapping. Iterative writing and nonrecursive destruction are not
needed once the AST is bounded (the tree drops through `Vec`, depth-bounded
like the writer).

**Existing behaviour preserved.** `primes_are_a_superscript_as_word_builds_them`
and every shape test (≤ 2 wraps) are unchanged; `MOST_NESTING` group nesting
is untouched.

## 9. WMF polygon records read points across record boundaries (quadratic CPU)

**Verified.** Each WMF record is size-checked against the remaining data
(`src/convert/metafile.rs:477-489`), but the polygon handlers trust the
declared point counts: `0x0324` (`:628-641`), `0x0538` (`:644-662`) and
`0x0325` (`:663-681`) loop `for _ in 0..n` at `p += 4` with no record bound.
A minimal record declaring 65 535 points consumes the bytes of every later
record (which then parse again): Θ(file²) point decodes. The EMF side already
confines (`read_emf_points`, `:1061-1083`).

**Failing tests (red).** `src/convert/metafile.rs` `hostile_input_tests`
- `wmf_polygon_cannot_consume_later_records` — POLYGON record declaring 65535
  points in a 4-word record, followed by an EXTTEXTOUT writing "Hi"; asserts
  `raster_wmf` returns `Some` and the text survives. Pre-fix the polygon read
  runs off the data and the whole rasterization returns `None`.
- `wmf_polypolygon_is_confined_to_its_record` — same shape via POLYPOLYGON
  (one ring claiming 65535 points) with an EXTTEXTOUT "Ho".

**Minimal source change.** `src/convert/metafile.rs`: hoist
`let end = off + size2` beside `payload`; the three handlers clamp their
counts to what the record's own bytes hold
(`n.min(end.saturating_sub(payload + 2) / 4)`, ring count clamped by
`(end - payload - 2) / 2` and each ring by the remaining point room, checked
subtractions). Well-formed records (declared count == room) parse
identically; EMF untouched.

**Existing behaviour preserved.** Every well-formed WMF in the corpus
(b88ac900's logo, 5a6c's diagram) parses to the same raster; the existing
hostile-input tests (`wmf_huge_record_size_terminates`, EMF set) still pass.

## 10. Aliased legacy DOC pieces expand tiny streams into huge logical stories

**Verified.** `main_text` (`src/legacy_doc.rs:1148-1230`) materializes one
`StoryChar` (16 B) per logical CP for CP 0..`ccp_text` (`:1187`), and the
piece table may alias: contiguous pieces may all point at the same physical
range (`fc`), so 4 095 pieces over one 1 MiB stream describe ~4 GiB of
characters (ccp_text is a raw `u32` from the FIB, `:919`). ~49 KiB of piece
records demand tens of GiB before rendering.

**Failing test (red).** `src/legacy_doc.rs` in-file tests
- `aliased_pieces_are_refused_before_materialization` — one piece tiling
  CP 0..1000 at `fc 0` over a 1-byte `WordDocument` slice; asserts
  `main_text(b"a", …, 1000)` is `Err`. Pre-fix it returns 1000 `StoryChar`s
  from one stream byte.
- `piece_counts_that_fit_the_stream_keep_reading` — the boundary: a story no
  longer than the stream still reads (guards the false-refusal direction;
  the existing `a_piece_property_modifier…` test at `:2269` covers a real
  read).

**Minimal source change.** `src/legacy_doc.rs`, at the top of `main_text`:
- `const MAX_STORY_CHARS: u32 = 64 * 1024 * 1024` (an ~130 000-page story,
  far past any real `.doc`) — `ccp_text` above it is refused with a typed
  `LegacyDocError`;
- aliasing refusal: `ccp_text > word.len()` is impossible for a faithful
  file — every character occupies at least one stream byte (compressed 1,
  Unicode 2) — so a longer claim is refused. No legitimate document can trip
  either check; a configurable budget is deferred (it would need CLI and
  binding surface; the two structural checks need none).

**Existing behaviour preserved.** All corpus `.doc` fixtures and the existing
`main_text`/`blocks` unit tests (stories far under both bounds) are unchanged.

## 11. False EOCD in a comment bypasses the entry-count budget

**Verified.** `preindex_budget` → `declared_entry_count`
(`src/admission.rs:397-408, 578-625`) takes the **highest-offset**
`PK\5\6` candidate with no validation. A false EOCD planted inside the real
record's comment (count 0, comment length 0xFFFF) is read as `declared = 0`,
while zip 9.0.5's backward search *validates* candidates (comment must fit
the file — `quick-xml`-neighbour check in
`zip-9.0.5/src/format/find_central_directory.rs:113-115`), rejects the false
one and opens the real, larger directory. `admit`'s only post-open count use
is `(archive.len() as u64) < declared` for duplicate detection
(`:451-456`) — `0` never trips it — so a package with more than
`max_entries` entries is fully indexed and processed.

**Failing test (red).** `src/admission.rs` `tests`
- `a_false_eocd_in_a_comment_cannot_bypass_the_entry_budget` — 3-entry
  package; the real EOCD gets a 22-byte comment that *is* a false EOCD
  (count 0, comment length `u16::MAX`); `admit` under `max_entries: 2` must
  refuse with `INPUT_LIMIT` mentioning "3 ZIP entries". Pre-fix the package
  is admitted.

**Minimal source change.** `src/admission.rs`, in `admit` right after the
archive opens: `if archive.len() > limits.max_entries { return
Err(input_over_entries(archive.len() as u64, &limits)) }` — the advertised
budget is enforced on the directory the reader actually opened, independent
of what any EOCD claims. (Making `declared_entry_count` byte-compatible with
zip's candidate validation would duplicate the reader's logic; the length
check subsumes the bypass. The `< declared` duplicate heuristic stays for
honest footers; exact name duplicates are already caught by the canonical
name set at `:473-492`.)

**Existing behaviour preserved.** Honest packages (`declared == len ≤
max_entries`) behave identically; `budgets_admit_their_exact_boundary`
(max_entries exactly 3) still admits.

## 12. Markdown reference DOCX parses unchecked XML

**Verified.** `markdown/package.rs:assemble` (`:103-130`) opens the
caller-supplied reference with `PartFs::open` (byte/structure checks only, no
XML admission) and `final_section` (`:548-561`) hands the whole main part to
`dom.parse_xdocument` — the same unbounded recursion; a deep reference
terminates the host running `markdown_to_docx` with `DocxOptions::reference`.

**Failing test (red).** `src/markdown/package.rs` `tests`
- `a_deeply_nested_reference_document_parses_bounded` — a 200 000-deep
  `<w:p>` reference body; asserts `final_section` returns `None` (no final
  `w:sectPr`) instead of recursing to death.

**Minimal source change.** None beyond finding 2's `MAX_DOM_DEPTH` cap (the
sink is the same `parse_xdocument`; the reference's parts are shallow in any
real template, ≤ 100 nesting in Word-written files).

**Existing behaviour preserved.** The default template and every fixture
reference (shallow) produce the same frames; the docx_markdown_round_trip
suite is unchanged.

---

## What the patch does not do (stated, not hidden)

- No `InputLimitOverrides`/CLI knobs are added for the new budgets (findings
  1, 5, 10 asked for "documented" bounds; the chosen constants sit far above
  every Word-reachable value, so configurability has no user yet).
- Findings 2/3/12 are fixed by one construction cap rather than by teaching
  admission to classify every consumed part; per-part `INVALID_XML` refusals
  for renamed parts are a possible follow-up, not required for the availability
  fix.
- Finding 4's red demonstration is a runtime blow-up (the test workload is
  infeasible pre-fix), not a clean assertion failure; the committed test is
  green-only and its pre-fix cost is documented above.
- Nothing here was compiled or executed: the patch is authored by reading the
  code, per the ask's constraint. `git apply --check` was run (see result).
