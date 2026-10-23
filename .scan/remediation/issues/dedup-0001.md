@coderabbitai plan
Please enrich, fix and assess the plan below.

25 findings, deduplicated by the scan into the 12 issues the plan below fixes (all severity medium except the two low `Untrusted Markdown reference documents` findings). Each finding's title, summary and remediation is in the follow-up comment, in this order:

- csf_9e40f23e742aa43493a0fb92 (medium)
- csf_8dcaf50784fd95a7f0c2ec28 (medium)
- csf_ad0603057507f1d6be6e9942 (medium)
- csf_0e2b085a251b34412cd71a4c (medium)
- csf_d46d1a3ef7bb52454cbd3c68 (medium)
- csf_64b247931cd1f053217fde7a (medium)
- csf_f23ce2ece7c2d5de613bb58d (medium)
- csf_e38139daefc493123fe13362 (medium)
- csf_3201e6caaef21850d61fa69c (medium)
- csf_dc0d34a6ea9a964733efe547 (medium)
- csf_eaffa9e4af9c9c07d800ef1b (medium)
- csf_a69309b0b64d1048c5a4e319 (low)
- csf_6128bc397a829afd60ea9390 (low)
- csf_27ca2e0d965d8b1341a32909 (medium)
- csf_dbf35c828f9082997b9ef8fc (medium)
- csf_33c5fede5e569e4c1531e38a (medium)
- csf_8510b5bc0ddac96422dfcdc6 (medium)
- csf_7084a83c24deb556942e2d43 (medium)
- csf_5a86b3f5aa767e4e64bd9e71 (medium)
- csf_b0aa536248aea0f44dd30d47 (medium)
- csf_91956fe2d99a3a2871c06916 (medium)
- csf_f756bbe5509c42bdcaa64921 (medium)
- csf_88d031991ca3e20e4aae2f24 (medium)
- csf_32625277093e7f286a267e2b (medium)
- csf_80edacafa6b67d4864116a38 (medium)

Full texts (summary + remediation) of the findings above are posted as a follow-up comment on this issue.

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

````
diff --git a/src/admission.rs b/src/admission.rs
index d556610..1309a17 100644
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -447,6 +447,14 @@ pub fn admit(bytes: &[u8], limits: InputLimits) -> Result<AdmittedPackage, Admis
     // its typed kind.
     let mut archive = crate::opc::open_archive_unchecked(Cursor::new(bytes))
         .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("not a ZIP file: {e}")))?;
+    // The preindex EOCD scan takes its candidate at face value while the
+    // reader validates candidates (a false EOCD in the real record's
+    // comment is rejected and the real, larger directory opens), so the
+    // entry budget is enforced on the directory that actually opened, not
+    // on what any footer claims (security-scan dedup-0001).
+    if archive.len() > limits.max_entries {
+        return Err(input_over_entries(archive.len() as u64, &limits));
+    }
     // The reader keys entries by name, so exact duplicates collapse into one.
     if (archive.len() as u64) < declared {
         return Err(AdmissionError::new(
@@ -1520,6 +1528,38 @@ mod tests {
         assert_eq!(kind(&bytes, shallow), AdmissionErrorKind::InvalidXml);
     }
 
+    /// A false EOCD inside the real record's comment: the preindex scan
+    /// reads its count (0) while the ZIP reader rejects the candidate (its
+    /// comment cannot fit the file) and opens the real 3-entry directory.
+    /// The opened archive's own length must uphold the entry budget
+    /// (security-scan dedup-0001).
+    #[test]
+    fn a_false_eocd_in_a_comment_cannot_bypass_the_entry_budget() {
+        let mut bytes = docx_with(&[]);
+        let real_eocd = bytes.len() - 22;
+        assert_eq!(&bytes[real_eocd..real_eocd + 4], b"PK\x05\x06");
+        // The comment is exactly one false EOCD: entry count 0, and a
+        // comment length that can never fit, so the reader skips it.
+        let mut false_eocd = [0u8; 22];
+        false_eocd[0..4].copy_from_slice(b"PK\x05\x06");
+        false_eocd[20..22].copy_from_slice(&u16::MAX.to_le_bytes());
+        bytes[real_eocd + 20..real_eocd + 22].copy_from_slice(&22u16.to_le_bytes());
+        bytes.extend_from_slice(&false_eocd);
+        let over = InputLimits {
+            max_entries: 2,
+            ..InputLimits::default()
+        };
+        let refused = admit(&bytes, over).expect_err("the real directory has 3 entries");
+        assert_eq!(refused.kind, AdmissionErrorKind::InputLimit);
+        assert!(
+            refused.message.contains("3 ZIP entries"),
+            "{refused}"
+        );
+        // The same package with a budget that fits still admits: the check
+        // is on the opened directory, not on suspicion.
+        assert!(admit(&bytes, InputLimits::default()).is_ok());
+    }
+
     #[test]
     fn a_highly_compressed_part_stops_at_the_budget() {
         // 32 MiB of zeros deflate to ~32 KiB; written in chunks, never held.
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f9..ad642e4 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -466,13 +466,18 @@ impl Builder {
         let Some(table) = self.tables.pop() else {
             return;
         };
+        // A cell's colspan is attacker paper (a tiny HTML chunk can carry
+        // colspan="1000000000"): one grid column per span would generate
+        // gigabytes of WML before the body is parsed. Saturate the row sum
+        // and hold the grid to the same budget a DOCX table obeys.
         let cols = table
             .rows
             .iter()
-            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
+            .map(|r| r.iter().map(|c| c.span).fold(0usize, usize::saturating_add))
             .max()
             .unwrap_or(1)
-            .max(1);
+            .max(1)
+            .min(super::MAX_TABLE_COLUMNS);
         let col_w = 9360 / cols;
         let mut s = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>");
         if table.border {
@@ -563,7 +568,8 @@ impl Builder {
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
                         .unwrap_or(1)
-                        .max(1),
+                        .max(1)
+                        .min(super::MAX_GRID_SPAN),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
@@ -807,6 +813,26 @@ mod tests {
         );
     }
 
+    /// A colspan of a billion in a 60-byte chunk must not become a billion
+    /// `<w:gridCol>` elements: the span caps like a DOCX `w:gridSpan`, and
+    /// the generated grid holds at most the table-column budget
+    /// (security-scan dedup-0001).
+    #[test]
+    fn a_huge_html_colspan_is_capped_not_expanded() {
+        let html = concat!(
+            "<html><body><table><tr>",
+            "<td colspan=\"1000000000\">x</td>",
+            "</tr></table></body></html>"
+        );
+        let wml = html_to_wml(html);
+        let cols = wml.matches("<w:gridCol").count();
+        assert!(
+            cols <= crate::convert::MAX_TABLE_COLUMNS,
+            "{cols} grid columns generated"
+        );
+        assert!(wml.contains(">x<"), "{wml}");
+    }
+
     #[test]
     fn an_mht_chunk_yields_its_quoted_printable_html() {
         let mht = "MIME-Version: 1.0\r\nContent-Type: multipart/related; boundary=\"b\"\r\n\r\n\
diff --git a/src/convert/metafile.rs b/src/convert/metafile.rs
index c1c75da..412af38 100644
--- a/src/convert/metafile.rs
+++ b/src/convert/metafile.rs
@@ -487,6 +487,11 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             break;
         }
         let payload = off + 6;
+        // The record's own last byte: polygon handlers confine their point
+        // reads to it, so a malformed count cannot consume later records
+        // (which would then parse again, quadratically; security-scan
+        // dedup-0001).
+        let end = off + size2;
         match func {
             0x0000 => break,
             0x020B => {
@@ -627,6 +632,10 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             }
             0x0324 => {
                 let n = read_u16(data, payload).unwrap_or(0) as usize;
+                // Points the record cannot hold are not read across its
+                // boundary; a well-formed record (count == room) is
+                // unchanged.
+                let n = n.min(end.saturating_sub(payload + 2) / 4);
                 let mut pts = Vec::with_capacity(n);
                 let mut p = payload + 2;
                 for _ in 0..n {
@@ -642,11 +651,17 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             // META_POLYPOLYGON: ring count, each ring's point count, then
             // every ring's points, filled as one shape (b88ac900's logo).
             0x0538 => {
+                // Confinement as 0x0324: ring counts and points stay inside
+                // the record's own bytes.
                 let rings = read_u16(data, payload).unwrap_or(0) as usize;
+                let rings = rings.min(end.saturating_sub(payload + 2) / 2);
                 let mut p = payload + 2 + 2 * rings;
+                let mut room = end.saturating_sub(p) / 4;
                 let mut subpaths = Vec::with_capacity(rings);
                 for r in 0..rings {
                     let n = read_u16(data, payload + 2 + 2 * r)? as usize;
+                    let n = n.min(room);
+                    room -= n;
                     let mut ring = Vec::with_capacity(n);
                     for _ in 0..n {
                         let x = read_i16(data, p)? as i32;
@@ -666,6 +681,7 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                     continue;
                 };
                 let n = read_u16(data, payload).unwrap_or(0) as usize;
+                let n = n.min(end.saturating_sub(payload + 2) / 4);
                 let mut prev: Option<(i32, i32)> = None;
                 let mut p = payload + 2;
                 for _ in 0..n {
@@ -1168,6 +1184,72 @@ mod hostile_input_tests {
         d.extend_from_slice(&[0u8; 32]);
         assert!(rasterize(&d).is_some());
     }
+
+    /// A WMF frame with real bounds, records starting at byte 40.
+    fn wmf_frame() -> Vec<u8> {
+        let mut d = vec![0u8; 40];
+        d[0..4].copy_from_slice(&[0xD7, 0xCD, 0xC6, 0x9A]);
+        d[10..12].copy_from_slice(&1000i16.to_le_bytes()); // right
+        d[12..14].copy_from_slice(&1000i16.to_le_bytes()); // bottom
+        d
+    }
+
+    fn wmf_rec(d: &mut Vec<u8>, func: u16, payload: &[u8]) {
+        let words = (6 + payload.len()).div_ceil(2);
+        d.extend_from_slice(&(words as u32).to_le_bytes());
+        d.extend_from_slice(&func.to_le_bytes());
+        d.extend_from_slice(payload);
+        if payload.len() % 2 == 1 {
+            d.push(0);
+        }
+    }
+
+    /// META_EXTTEXTOUT: y, x, count, options, string.
+    fn wmf_textout(text: &str) -> (u16, Vec<u8>) {
+        let mut payload = Vec::new();
+        payload.extend_from_slice(&10i16.to_le_bytes());
+        payload.extend_from_slice(&10i16.to_le_bytes());
+        payload.extend_from_slice(&(text.len() as u16).to_le_bytes());
+        payload.extend_from_slice(&0u16.to_le_bytes());
+        payload.extend_from_slice(text.as_bytes());
+        (0x0A32, payload)
+    }
+
+    /// A polygon record claiming 65535 points it does not carry must not
+    /// read across its boundary into later records: previously every later
+    /// record's bytes decoded as points until the data ended and the whole
+    /// rasterization returned `None` (security-scan dedup-0001).
+    #[test]
+    fn wmf_polygon_cannot_consume_later_records() {
+        let mut d = wmf_frame();
+        wmf_rec(&mut d, 0x0324, &0xFFFFu16.to_le_bytes());
+        let (func, payload) = wmf_textout("Hi");
+        wmf_rec(&mut d, func, &payload);
+        wmf_rec(&mut d, 0x0000, &[]);
+        let (_, _, _, texts) = raster_wmf(&d).expect("the malformed polygon stays in its record");
+        assert!(
+            texts.iter().any(|t| t.text == "Hi"),
+            "the following record must still parse: {texts:?}"
+        );
+    }
+
+    /// The same confinement for META_POLYPOLYGON's ring counts.
+    #[test]
+    fn wmf_polypolygon_is_confined_to_its_record() {
+        let mut d = wmf_frame();
+        let mut rings = Vec::new();
+        rings.extend_from_slice(&1u16.to_le_bytes()); // one ring
+        rings.extend_from_slice(&0xFFFFu16.to_le_bytes()); // claiming 65535 points
+        wmf_rec(&mut d, 0x0538, &rings);
+        let (func, payload) = wmf_textout("Ho");
+        wmf_rec(&mut d, func, &payload);
+        wmf_rec(&mut d, 0x0000, &[]);
+        let (_, _, _, texts) = raster_wmf(&d).expect("the malformed ring stays in its record");
+        assert!(
+            texts.iter().any(|t| t.text == "Ho"),
+            "the following record must still parse: {texts:?}"
+        );
+    }
 }
 
 #[cfg(test)]
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
index d900e05..961a11a 100644
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -3240,7 +3240,10 @@ fn parse_tab_stops(dom: &Dom, ppr: NodeId) -> Vec<TabStop> {
         if val == "clear" {
             continue;
         }
-        if let Some(pos) = attr_any(dom, tab, "pos").and_then(|s| s.parse::<f32>().ok()) {
+        if let Some(pos) = attr_any(dom, tab, "pos")
+            .and_then(|s| s.parse::<f32>().ok())
+            .filter(|pos| pos.is_finite())
+        {
             let align = match val {
                 // ISO Strict ST_TabJc: end/start are LTR right/left
                 // (Strict01 TOC val=end leader=dot pos=9350).
@@ -6012,10 +6015,16 @@ fn ordinal_text_ones(one: &str) -> String {
     }
 }
 
+/// The most times a Chicago mark repeats. Word's own labels never approach
+/// it; past it the repetition holds, in the tradition of [`ALPHA_LABEL_MAX`]
+/// and [`ROMAN_LABEL_MAX`], so an attacker-chosen `w:start`/`startOverride`
+/// or section page start cannot size the label by `u32` arithmetic.
+pub(crate) const CHICAGO_LABEL_MAX_REPEATS: usize = 32;
+
 fn chicago_label(n: u32) -> String {
     const MARKS: [&str; 4] = ["*", "†", "‡", "§"];
     let i = n.saturating_sub(1) as usize;
-    MARKS[i % 4].repeat(i / 4 + 1)
+    MARKS[i % 4].repeat((i / 4 + 1).min(CHICAGO_LABEL_MAX_REPEATS))
 }
 
 fn ideograph_digital_label(n: u32) -> String {
@@ -13601,8 +13610,15 @@ fn table_block(
     let mut occupancy = 0usize;
     let grid_len = cols.len();
     for row in &raw_rows {
-        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
+        // A row's span sum is attacker arithmetic: saturate before the max,
+        // then hold the grid itself to Word's reachability bound.
+        let row_cols = row
+            .iter()
+            .map(|c| c.colspan)
+            .fold(0usize, usize::saturating_add);
+        occupancy = occupancy.max(row_cols);
     }
+    let occupancy = occupancy.min(MAX_TABLE_COLUMNS);
     if cols.len() < occupancy {
         if cols.len() == 1 && occupancy > 1 {
             let each = cols[0] / occupancy as f32;
@@ -14201,6 +14217,18 @@ fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
     }
 }
 
+/// One `w:gridSpan` / `w:gridBefore` / `w:gridAfter` (or an HTML colspan)
+/// after admission. Word's own tables hold at most 63 columns, so a span
+/// past this is a document no Word build renders; the value is
+/// attacker-chosen paper that must not size a vector.
+pub(crate) const MAX_GRID_SPAN: usize = 1024;
+
+/// Columns a rendered table grid may hold. Like [`MAX_GRID_SPAN`] this is a
+/// Word-reachability bound, not a policy knob: a grid past it cannot come
+/// from Word, and the row-occupancy sum that sizes `cols` vectors clamps
+/// here before any allocation.
+pub(crate) const MAX_TABLE_COLUMNS: usize = 4096;
+
 fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
     let Some(pr) = first_named(dom, cell, "tcPr") else {
         return (1, VMerge::None);
@@ -14212,7 +14240,8 @@ fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|s| s.parse().ok())
         .unwrap_or(1)
-        .max(1);
+        .max(1)
+        .min(MAX_GRID_SPAN);
     let vmerge = match direct_named(dom, pr, "vMerge") {
         None => VMerge::None,
         Some(n) => match dom.attribute(n, &W::val()) {
@@ -14342,7 +14371,8 @@ fn row_grid_skip(dom: &Dom, row: NodeId, side: &str) -> Option<(usize, PrefWidth
     let span = direct_named(dom, pr, &format!("grid{side}"))
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|v| v.parse::<usize>().ok())
-        .filter(|n| *n > 0)?;
+        .filter(|n| *n > 0)
+        .map(|n| n.min(MAX_GRID_SPAN))?;
     let pref = direct_named(dom, pr, &format!("w{side}"))
         .filter(|n| attr_any(dom, *n, "type").is_none_or(|t| t == "dxa"))
         .and_then(|n| attr_any(dom, n, "w"))
@@ -14600,14 +14630,14 @@ fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>> {
     for (ri, row) in raw_rows.into_iter().enumerate() {
         let mut col = 0usize;
         for raw in row {
-            let span = raw.colspan.max(1);
+            let span = raw.colspan.clamp(1, MAX_GRID_SPAN);
             if raw.vmerge == VMerge::Continue {
                 if let Some(&(_, or, oi)) = open.get(col).and_then(|s| s.as_ref())
                     && let Some(origin) = origins.get_mut(or).and_then(|r| r.get_mut(oi))
                 {
                     origin.rowspan += 1;
                 }
-                col += span;
+                col = col.saturating_add(span).min(MAX_TABLE_COLUMNS);
                 continue;
             }
             let idx = origins[ri].len();
@@ -14634,14 +14664,15 @@ fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>> {
                 borders: raw.borders,
                 style_fill: false,
             });
-            if open.len() < col + span {
-                open.resize(col + span, None);
+            let span_end = col.saturating_add(span).min(MAX_TABLE_COLUMNS);
+            if open.len() < span_end {
+                open.resize(span_end, None);
             }
             let mark = (raw.vmerge == VMerge::Restart).then_some((col, ri, idx));
-            for slot in open.iter_mut().skip(col).take(span) {
+            for slot in open.iter_mut().skip(col).take(span_end - col) {
                 *slot = mark;
             }
-            col += span;
+            col = span_end;
         }
     }
     origins
@@ -26378,6 +26409,11 @@ impl<'a> Layout<'a> {
         w
     }
 
+    /// Leader cells one line may hold: 8192 cells of the narrowest mark is
+    /// ~22 500pt, wider than any physical page, so no leader Word paints is
+    /// clipped by this bound.
+    const MAX_TAB_LEADER_CELLS: f32 = 8192.0;
+
     fn paint_tab_leader(&mut self, x0: f32, x1: f32, y: f32, style: &RunStyle, mark: &str) {
         let fid = self.fonts.resolve(&style.family, style.bold, style.italic);
         let face = self.fonts.get(fid);
@@ -26391,8 +26427,14 @@ impl<'a> Layout<'a> {
         // last that ends by the stop (Word 16 probes dots, 2026-10-02: TNR
         // 12 after "A" ending 99.14 fills 102..339 before a 340.5 stop;
         // 11pt cells of 2.75 start at 99 under a 90pt margin).
-        let first = (x0 / dw - 0.001).ceil();
-        let last = (x1 / dw + 0.001).floor();
+        // The stop's position is attacker paper: a nonfinite interval
+        // paints nothing, and the interval clamps to what a line can hold,
+        // so `repeat` cannot size by it.
+        if !x0.is_finite() || !x1.is_finite() {
+            return;
+        }
+        let first = (x0 / dw - 0.001).ceil().max(0.0);
+        let last = (x1 / dw + 0.001).floor().min(MAX_TAB_LEADER_CELLS);
         if last <= first {
             return;
         }
@@ -36377,9 +36419,10 @@ fn body_op_yrange(ops: &[Op]) -> Option<(f32, f32)> {
 #[cfg_attr(all(coverage_nightly, test), coverage(off))]
 mod page_num_fmt_labels {
     use super::{
-        NumFmt, chicago_label, decimal_fullwidth_label, format_num, ideograph_digital_label,
-        ideograph_enclosed_circle_label, ideograph_legal_traditional_label,
-        ideograph_zodiac_traditional_label, japanese_counting_label, parse_num_fmt,
+        CHICAGO_LABEL_MAX_REPEATS, NumFmt, chicago_label, decimal_fullwidth_label, format_num,
+        ideograph_digital_label, ideograph_enclosed_circle_label,
+        ideograph_legal_traditional_label, ideograph_zodiac_traditional_label,
+        japanese_counting_label, parse_num_fmt,
     };
 
     #[test]
@@ -36395,6 +36438,29 @@ mod page_num_fmt_labels {
         assert_eq!(chicago_label(5), "**");
     }
 
+    /// A list start or page start of `u32::MAX` must not repeat the mark a
+    /// billion times before any later clipping (security-scan dedup-0001):
+    /// the label itself is bounded, in the alpha/roman cycling tradition.
+    #[test]
+    fn chicago_labels_stay_bounded_at_extreme_counters() {
+        for n in [u32::MAX, u32::MAX - 1, 1 << 30] {
+            let label = chicago_label(n);
+            let marks = label.chars().count();
+            assert!(
+                marks <= CHICAGO_LABEL_MAX_REPEATS,
+                "{marks} marks in {label:?}"
+            );
+            assert!(
+                label.chars().all(|c| "*†‡§".contains(c)),
+                "{label:?}"
+            );
+        }
+        // The Word-probed cycling values keep their exact labels.
+        assert_eq!(chicago_label(4), "§");
+        assert_eq!(chicago_label(8), "§§");
+        assert_eq!(chicago_label(9), "***");
+    }
+
     #[test]
     fn ideograph_digital_uses_cjk_digits() {
         assert_eq!(ideograph_digital_label(1), "一");
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
index 1881053..86ea249 100644
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -1139,6 +1139,12 @@ struct StoryChar {
     modifier: CharModifier,
 }
 
+/// The most main-story characters a `.doc` may declare: 64 Mi characters is
+/// an ~130,000-page story, far past any real file, and bounds the
+/// per-character metadata `main_text` materializes (security-scan
+/// dedup-0001).
+const MAX_STORY_CHARS: u32 = 64 * 1024 * 1024;
+
 /// The main story's characters, CP 0 to `ccp_text`.
 ///
 /// # Errors
@@ -1147,6 +1153,21 @@ struct StoryChar {
 /// corrupt file, refused rather than converted short.
 fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>> {
     let past = || LegacyDocError::new("a text piece points past the WordDocument stream");
+    // `ccp_text` is raw FIB arithmetic, and pieces may alias: thousands of
+    // piece records can tile a logical story many times the physical text
+    // they point at, materializing gigabytes of `StoryChar` from a small
+    // stream. A faithful file cannot break either check — every character
+    // occupies at least one stream byte (compressed one, Unicode two).
+    if ccp_text > MAX_STORY_CHARS {
+        return Err(LegacyDocError::new(format!(
+            "the main story claims {ccp_text} characters; the limit is {MAX_STORY_CHARS}"
+        )));
+    }
+    if u64::from(ccp_text) > word.len() as u64 {
+        return Err(LegacyDocError::new(
+            "the piece table claims more main-story characters than the WordDocument stream holds",
+        ));
+    }
     // The pieces run from CP 0 through the main story without a gap, or
     // the document would come out quietly shorter than it is.
     // The CPs only ever increase ([MS-DOC] 2.8.35): a piece that runs
@@ -1592,8 +1613,12 @@ impl Story<'_> {
         let mut out = Vec::new();
         let mut spans: Vec<Span> = Vec::new();
         // Field nesting: true while inside a field's code (before its
-        // separator).
+        // separator). `open_code` counts the frames still in their code
+        // section, so the per-character check is O(1): a malformed story of
+        // bare field begins costs its length, not its length squared
+        // (security-scan dedup-0001).
         let mut fields: Vec<bool> = Vec::new();
+        let mut open_code: usize = 0;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
@@ -1601,20 +1626,26 @@ impl Story<'_> {
 
         for story in chars {
             // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
+            let in_code = open_code > 0;
             let text = match story.ch {
                 '\u{13}' => {
                     fields.push(true);
+                    open_code += 1;
                     continue;
                 }
                 '\u{14}' => {
                     if let Some(top) = fields.last_mut() {
-                        *top = false;
+                        if *top {
+                            *top = false;
+                            open_code -= 1;
+                        }
                     }
                     continue;
                 }
                 '\u{15}' => {
-                    fields.pop();
+                    if fields.pop() == Some(true) {
+                        open_code -= 1;
+                    }
                     continue;
                 }
                 _ if in_code => continue,
@@ -1967,6 +1998,69 @@ mod tests {
         assert_eq!(texts, ["aoneb"]);
     }
 
+    /// The suppression rule under any code ancestor holds for mixed and
+    /// unbalanced sequences too — the O(1) `open_code` count must answer
+    /// exactly what the per-frame scan answered (security-scan dedup-0001).
+    #[test]
+    fn field_code_suppression_is_unchanged_for_mixed_nesting() {
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let story = |text: &str| {
+            let chars = story_chars(text);
+            Story {
+                papx: &papx,
+                chpx: &chpx,
+                styles: &[],
+                lists: &lists,
+                section_marks: Vec::new(),
+            }
+            .blocks(&chars)
+        };
+        let shown = |text: &str| -> Vec<String> {
+            story(text)
+                .iter()
+                .map(|b| match b {
+                    Block::Paragraph(p) => p.text(),
+                    Block::Table(_) => "table".into(),
+                })
+                .collect()
+        };
+        // A separator flips only its own frame: text after it shows.
+        assert_eq!(shown("\u{13}IF\u{14}shown\u{15}b\r"), ["shownb"]);
+        // A second begin before the first end: everything up to both ends
+        // is code, and the unterminated frame suppresses the paragraph mark.
+        assert_eq!(shown("a\u{13}b\u{13}c\u{15}d\r"), ["a"]);
+        // An end without a begin changes nothing.
+        assert_eq!(shown("a\u{15}b\r"), ["ab"]);
+    }
+
+    /// Bare field begins used to make every following character rescan the
+    /// whole stack (quadratic work; security-scan dedup-0001). The same
+    /// workload is now linear: 100k begins plus text reads in
+    /// milliseconds, and the text still lands.
+    #[test]
+    fn bare_field_begins_consume_linear_work() {
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let mut text = String::new();
+        for _ in 0..100_000 {
+            text.push('\u{13}');
+        }
+        text.push_str("tail\r");
+        let chars = story_chars(&text);
+        let blocks = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        }
+        .blocks(&chars);
+        // The begins never ended, so the whole story stays field code: no
+        // paragraph survives, and the walk above finished in linear time.
+        assert!(blocks.is_empty(), "{blocks:?}");
+    }
+
     /// A supplementary character split across two Unicode pieces is one
     /// character, as Word reads it; a lone high surrogate is a replacement
     /// mark, never dropped.
@@ -1999,6 +2093,29 @@ mod tests {
         assert_eq!(text(4), "a😀b");
         // The story ends on a high surrogate: Word shows a replacement mark.
         assert_eq!(text(5), "a😀b\u{FFFD}");
+        // Aliased pieces (security-scan dedup-0001): the piece table may
+        // tile a logical story longer than the stream it reads, which no
+        // faithful file does — every character occupies at least one byte.
+        let aliased = Piece {
+            cp_start: 0,
+            cp_end: 1000,
+            fc: 0,
+            compressed: true,
+            modifier: CharModifier::default(),
+        };
+        let refused = main_text(b"a", &[aliased], 1000).unwrap_err();
+        assert!(
+            refused
+                .to_string()
+                .contains("more main-story characters than the WordDocument stream"),
+            "{refused}"
+        );
+        // A story no longer than its stream keeps reading.
+        let honest = main_text(b"ab\r", &[aliased], 3);
+        assert!(honest.is_ok(), "{honest:?}");
+        // An absurd declared story is refused on its own count.
+        let huge = main_text(b"x", &[aliased], u32::MAX).unwrap_err();
+        assert!(huge.to_string().contains("the limit is"), "{huge}");
         // A high surrogate before a letter is a mark, and the letter stays.
         let lone: Vec<u8> = [0xD83Du16, 0x0063]
             .iter()
diff --git a/src/markdown/math.rs b/src/markdown/math.rs
index c1fd38b..f449858 100644
--- a/src/markdown/math.rs
+++ b/src/markdown/math.rs
@@ -774,6 +774,14 @@ struct Parser {
 /// off the end of the stack.
 const MOST_NESTING: usize = 64;
 
+/// How many bare `^`/`_` may wrap the previous node in a row. A chain
+/// longer than this is already invalid LaTeX ("double superscript") and no
+/// Word equation carries it; past the budget the script's argument reads as
+/// text instead of another wrap, so a flat chain cannot build a tree the
+/// recursive OMML writer (or a dropped `Vec` of nodes) cannot survive
+/// (security-scan dedup-0001).
+const MOST_SCRIPT_WRAPS: usize = 8;
+
 /// A token as the LaTeX it was read from.
 fn token_latex(token: &Token) -> String {
     match token {
@@ -866,6 +874,9 @@ impl Parser {
 
     fn nested_list(&mut self, stops: Stops) -> Vec<Node> {
         let mut out: Vec<Node> = Vec::new();
+        // Consecutive bare scripts wrap the previous node as their base; the
+        // count bounds the chain's depth (see [`MOST_SCRIPT_WRAPS`]).
+        let mut wraps = 0usize;
         while !self.stopped(stops) {
             match self.peek() {
                 Some(Token::Change(change)) => {
@@ -873,9 +884,17 @@ impl Parser {
                     self.at += 1;
                 }
                 Some(Token::Sub | Token::Sup) => {
-                    let base = take_base(&mut out);
-                    let node = self.scripts(base);
-                    out.push(node);
+                    if wraps < MOST_SCRIPT_WRAPS {
+                        let base = take_base(&mut out);
+                        let node = self.scripts(base);
+                        out.push(node);
+                        wraps += 1;
+                    } else {
+                        // Past the budget the script's argument reads as
+                        // text: the token is consumed so the walk moves on.
+                        self.at += 1;
+                        out.extend(self.argument());
+                    }
                 }
                 // `f'` is `f` with a prime superscript, as Word builds it;
                 // `f'^2` puts the 2 after the prime.
@@ -906,6 +925,7 @@ impl Parser {
                 _ => {
                     if let Some(node) = self.atom(stops) {
                         out.push(node);
+                        wraps = 0;
                     }
                 }
             }
@@ -2117,6 +2137,25 @@ mod tests {
         assert_eq!(shape("'a"), "r('a)");
     }
 
+    /// A flat `a^1^1…` chain wraps the previous node on every `^`
+    /// without passing the group-nesting budget, building a tree as deep as
+    /// the input that the recursive OMML writer walks. Past the wrap budget
+    /// the rest reads as text (security-scan dedup-0001).
+    #[test]
+    fn a_flat_script_chain_is_bounded() {
+        let latex = format!("a{}", "^1".repeat(20_000));
+        let xml = omml(&latex, false, None, &mut |_| String::new());
+        let wraps = xml.matches("<m:sSup>").count();
+        assert!(
+            wraps <= MOST_SCRIPT_WRAPS,
+            "{wraps} nested script elements in {} bytes",
+            xml.len()
+        );
+        // Chains inside the budget keep one element per script.
+        let small = omml("a^1^2^3", false, None, &mut |_| String::new());
+        assert_eq!(small.matches("<m:sSup>").count(), 3);
+    }
+
     #[test]
     fn not_negates_the_next_relation() {
         assert_eq!(shape(r"a \not\in B"), shape(r"a \notin B"));
diff --git a/src/markdown/package.rs b/src/markdown/package.rs
index 9162eb3..609ed71 100644
--- a/src/markdown/package.rs
+++ b/src/markdown/package.rs
@@ -1866,6 +1866,21 @@ mod tests {
         assert_eq!(drawing_prefix("<theme/>"), "a:");
     }
 
+    /// A reference document is caller-supplied, not admitted, and its main
+    /// part reaches `final_section` without any depth gate. A deeply nested
+    /// reference must parse bounded (the DOM construction cap), not take
+    /// the stack (security-scan dedup-0001).
+    #[test]
+    fn a_deeply_nested_reference_document_parses_bounded() {
+        let deep = format!(
+            "<w:document xmlns:w=\"{W_NS}\"><w:body>{}<w:p/></w:body></w:document>",
+            "<w:p>".repeat(200_000)
+        );
+        // Nothing in this reference is a final section: bounded parsing
+        // returns `None` where an unbounded recursion would die.
+        assert!(final_section(&deep).is_none());
+    }
+
     #[test]
     fn the_final_section_is_found_only_as_the_bodys_last_child() {
         let body = |inner: &str| {
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..29e5c74 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -63,6 +63,15 @@ pub(crate) const MAX_NAMESPACE_PREFIX_BYTES: usize = 256;
 pub(crate) const MAX_LIVE_NS_BINDINGS: usize = 256;
 /// Total prefix+URI bytes declared by xmlns attributes in one document.
 pub(crate) const MAX_NS_DECLARED_BYTES: usize = 1024 * 1024;
+/// How deep the permissive DOM builder may nest elements. Every built-in
+/// admission profile bounds a part at [`InputLimits::max_xml_depth`] = 256,
+/// so an admitted document (round-trip gate included) never reaches this;
+/// the cap is the safety valve for XML that reaches the parser without a
+/// gate — a renamed auxiliary part, a Markdown reference document, or
+/// malformed quotes that admission's scanner reads as siblings. Past it an
+/// element closes as a leaf: the input's remaining open tags become its
+/// siblings instead of another recursion level.
+const MAX_DOM_DEPTH: usize = 512;
 
 // N02 mechanism counter: bytes of (prefix, uri) pairs the parser's
 // namespace scope has recorded. The scoped-overlay scope only ever records
@@ -104,6 +113,7 @@ pub fn parse_xdocument(dom: &mut Dom, xml: &str) -> NodeId {
         dom,
         c: chars,
         pos: 0,
+        depth: 0,
     };
     p.parse_document()
 }
@@ -139,6 +149,8 @@ struct Parser<'a> {
     dom: &'a mut Dom,
     c: Vec<char>,
     pos: usize,
+    /// Open-element depth, for the [`MAX_DOM_DEPTH`] safety valve.
+    depth: usize,
 }
 
 /// In-scope namespace bindings as an overlay stack (N02): one `HashMap`
@@ -623,6 +635,16 @@ impl<'a> Parser<'a> {
         }
         self.pos += 1; // '>'
 
+        // The safety valve: an element that opens at the cap closes as a
+        // leaf, exactly like the self-closing branch above. Its would-be
+        // children land as siblings of an ancestor's child loop, so no
+        // input — however it nests — grows the recursion.
+        if self.depth >= MAX_DOM_DEPTH {
+            scope.pop(mark);
+            return el;
+        }
+        self.depth += 1;
+
         let len = self.len();
         while self.pos < len {
             if self.starts_with("</") {
@@ -674,6 +696,7 @@ impl<'a> Parser<'a> {
                 self.dom.add(el, t);
             }
         }
+        self.depth -= 1;
         scope.pop(mark);
         el
     }
@@ -1678,3 +1701,64 @@ mod tag_policy_tests {
         );
     }
 }
+
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod dom_depth_cap_tests {
+    use super::*;
+
+    fn max_depth(dom: &Dom, doc: NodeId) -> usize {
+        let mut best = 0usize;
+        let mut stack = vec![(doc, 0usize)];
+        while let Some((node, depth)) = stack.pop() {
+            best = best.max(depth);
+            for i in 0..dom.child_count(node) {
+                stack.push((dom.child_at(node, i), depth + 1));
+            }
+        }
+        best
+    }
+
+    /// `<p a=//>` is an Empty event to quick-xml (admission sees siblings,
+    /// depth 0) but an open element to this parser: the two slashes read as
+    /// attribute-value quotes, so every following sibling nests one deeper.
+    /// The construction cap keeps the tree — and the recursion — bounded
+    /// (security-scan dedup-0001).
+    #[test]
+    fn quote_confused_siblings_build_a_bounded_tree() {
+        let xml = "<p a=//>".repeat(200_000);
+        let mut dom = Dom::new();
+        let doc = parse_xdocument(&mut dom, &xml);
+        assert!(dom.root(doc).is_some());
+        assert!(
+            max_depth(&dom, doc) <= MAX_DOM_DEPTH + 1,
+            "depth {} past the cap",
+            max_depth(&dom, doc)
+        );
+    }
+
+    /// Nesting past the cap truncates instead of recursing: the same sink
+    /// an unadmitted part finds (a renamed auxiliary part, a Markdown
+    /// reference document).
+    #[test]
+    fn over_deep_nesting_is_truncated_at_the_cap() {
+        let xml = format!("{}{}", "<a>".repeat(5_000), "</a>".repeat(5_000));
+        let mut dom = Dom::new();
+        let doc = parse_xdocument(&mut dom, &xml);
+        assert!(
+            max_depth(&dom, doc) <= MAX_DOM_DEPTH + 1,
+            "depth {} past the cap",
+            max_depth(&dom, doc)
+        );
+    }
+
+    /// Admission's own depth profile (256) sits far under the cap, so an
+    /// admitted document parses to exactly the tree it always had.
+    #[test]
+    fn admitted_depth_documents_keep_their_full_tree() {
+        let xml = format!("{}{}", "<a>".repeat(256), "</a>".repeat(256));
+        let mut dom = Dom::new();
+        let doc = parse_xdocument(&mut dom, &xml);
+        assert_eq!(max_depth(&dom, doc), 256);
+    }
+}
diff --git a/tests/resource_limits.rs b/tests/resource_limits.rs
new file mode 100644
index 0000000..b22bc08
--- /dev/null
+++ b/tests/resource_limits.rs
@@ -0,0 +1,78 @@
+// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
+//
+// SPDX-License-Identifier: AGPL-3.0-only
+
+//! Resource budgets on the public render path (security-scan dedup-0001):
+//! attacker-chosen numbers inside an admitted package — table spans, a
+//! renamed deeply nested part, a huge tab stop — must render within a
+//! bounded allocation, not size one.
+
+mod common;
+
+use common::docx::{Part, docx, docx_with};
+use jubarte::convert::{docx_to_pdf, pdf_page_count};
+
+/// One `w:gridSpan` of a billion in a one-cell table: the span parsed as
+/// bare `usize` used to size the column vector (~4 GiB for the `f32`s)
+/// before layout began. The span now clamps to Word's reachability bound
+/// and the tiny table renders.
+#[test]
+fn a_huge_gridspan_renders_within_budget() {
+    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"4675\"/></w:tblGrid>\
+                <w:tr><w:tc><w:tcPr>\
+                <w:gridSpan w:val=\"1000000000\"/>\
+                </w:tcPr><w:p/></w:tc></w:tr></w:tbl>";
+    let pdf = docx_to_pdf(&docx(body)).expect("a clamped span renders");
+    assert_eq!(pdf_page_count(&pdf), 1);
+}
+
+/// `w:gridBefore` takes the same placeholder-span path
+/// (`row_grid_skip` → `grid_skip_cell`): a billion empty leading columns
+/// must not become a billion-column grid.
+#[test]
+fn a_huge_grid_before_renders_within_budget() {
+    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"4675\"/></w:tblGrid>\
+                <w:tr><w:trPr><w:gridBefore w:val=\"2000000000\"/></w:trPr>\
+                <w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
+    let pdf = docx_to_pdf(&docx(body)).expect("a clamped skip renders");
+    assert_eq!(pdf_page_count(&pdf), 1);
+}
+
+/// A deeply nested theme stored under a non-XML name: admission's depth
+/// scan only reads `.xml`/`.rels` names, and rendering selects the part by
+/// relationship. The DOM construction cap keeps the parse bounded, where
+/// the permissive parser used to recurse to the input's depth.
+#[test]
+fn a_renamed_deeply_nested_theme_part_renders_bounded() {
+    let deep = format!(
+        "<a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">{}\
+         <a:clrScheme name=\"x\"><a:dk1><a:sysClr val=\"window\"/></a:dk1></a:clrScheme>\
+         </a:theme>",
+        "<a:e>".repeat(300_000)
+    );
+    let bytes = docx_with(
+        "<w:p><w:r><w:t>T</w:t></w:r></w:p>",
+        &[Part {
+            name: "word/theme1.bin",
+            content_type: "application/vnd.openxmlformats-officedocument.theme+xml",
+            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme",
+            xml: &deep,
+        }],
+    );
+    let pdf = docx_to_pdf(&bytes).expect("a bounded theme parses");
+    assert_eq!(pdf_page_count(&pdf), 1);
+}
+
+/// A left tab stop at `1e22` twips with a dot leader: the leader interval
+/// is attacker paper (`repeat` sized by the stop's coordinate, saturating
+/// `as usize` to `usize::MAX`). Nonfinite or beyond-page intervals now
+/// paint nothing or clamp to the line's extent.
+#[test]
+fn a_huge_tab_leader_renders_bounded() {
+    let body = "<w:p><w:pPr><w:tabs>\
+                <w:tab w:val=\"left\" w:leader=\"dot\" w:pos=\"1e22\"/>\
+                </w:tabs></w:pPr>\
+                <w:r><w:tab/></w:r><w:r><w:t>x</w:t></w:r></w:p>";
+    let pdf = docx_to_pdf(&docx(body)).expect("a clamped leader renders");
+    assert_eq!(pdf_page_count(&pdf), 1);
+}
````
