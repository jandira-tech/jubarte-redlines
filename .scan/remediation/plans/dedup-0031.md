# dedup-0031 remediation plan (TDD red/green)

Scan folder: `.scan/artifacts/deep_discovery/dedup/dedup-0031`
Reducer round: `dedup-0031`, claimed worker `c74c718d-ff92-424a-b692-7291a9bb9b5c`
(prompt.md `claimedWorkerIds`). This plan covers the ten findings of
`output/result.json` whose `provenance.sourceFindingIds` carry that worker —
this round's own contributions — and states explicitly what is **not** patched
here and why.

All of the fixes are **Word-faithful clamps**: every bound already exists as a
Word limit (63 table columns at `src/edit/structural.rs:19`, nine numbering
levels 0..=8) or as a budget far beyond any real document, so no
Word-valid document changes output. No `#[allow]` suppressions are used.

Patch: `.scan/remediation/patches/dedup-0031.patch` (applies with
`git apply` from the repository root; verified with `git apply --check` in this
session). **Not run (forbidden in this ask):** `cargo fmt --check`,
`cargo clippy -D warnings`, and every test command below — they are the
commands to run when Cargo is allowed again.

---

## 1. HTML altChunk colspan → billions of `<w:gridCol>` (result.json finding
"Rendering an HTML chunk can allocate gigabytes from a single colspan value")

**Source read:** `src/convert/altchunk.rs:558-567` (colspan parsed into an
unbounded `usize`), `:469-491` (`end_table` sums spans and emits one
`<w:gridCol>` per column), `src/convert/mod.rs:772-798` (entrypoint that
expands chunks after package admission).

**Failing test (red):** `src/convert/altchunk.rs`, module `tests`, new test
`a_huge_colspan_clamps_to_words_column_limit`. It renders
`<td colspan="1000000000">` twice through `html_to_wml` and asserts
`wml.matches("<w:gridCol").count() == 63` and that the generated
`<w:gridSpan w:val="63"/>`. Before the fix the row loop emits one grid element
per attacker integer (≈ 20 GB for `colspan=1000000000`), so the test cannot
complete; with the fix it is 63 elements.

**Minimal source change:**
- New `const MAX_TABLE_COLUMNS: usize = 63` beside the imports — the same
  Word ceiling `src/edit/structural.rs:19` (`MAX_COLUMNS`, "Word allows at
  most 63 columns") already enforces for table construction.
- `colspan` parse: `.max(1)` → `.clamp(1, MAX_TABLE_COLUMNS)`.
- `end_table` column count: the row sum becomes
  `.fold(0usize, usize::saturating_add)` (no wrap/panic on overflow) and the
  result `.clamp(1, MAX_TABLE_COLUMNS)`, bounding the grid loop and the
  `col_w * span` arithmetic.

**Must keep passing:** `an_alt_chunk_is_replaced_by_its_paragraphs`,
`a_class_rule_and_inline_bold_become_run_properties`, and the remaining
altchunk tests (small tables are untouched: every legitimate colspan ≤ 63).

## 2. DOCX `w:gridSpan` / `w:gridBefore` / `w:gridAfter` → huge column,
preference and merge vectors (finding "Rendering a small DOCX table can
exhaust host memory")

**Source read:** `src/convert/mod.rs:14204-14224` (`cell_span` parses
`w:gridSpan` unbounded), `:14340-14353` (`row_grid_skip` parses
`w:gridBefore`/`w:gridAfter` unbounded), `:13601-13616` (occupancy sum then
`cols.resize(occupancy, 80.0)`), `:14596-14648` (`resolve_table_merges`
`open.resize(col + span, None)`), `:13466-13561` (grid_at accumulation and
skip cells).

**Failing test (red):** `src/convert/mod.rs`, module `table_tests`, new test
`huge_spans_and_row_skips_clamp_to_words_column_limit`. It runs `table_block`
on a one-row table with `<w:gridSpan w:val="1000000000"/>` and on a row with
`<w:gridBefore w:val="1000000000"/>`, asserting `rows[0][0].colspan == 63`
in both cases, plus `grid_skip` and `rows[0][1].colspan == 1` for the skip.
Before the fix `cols.resize(1_000_000_000, 80.0)` requests ~4 GB (and the
merge-slot `open.resize` more), aborting the test process; with the fix both
tables lay out over 63 columns.

**Minimal source change:**
- `const MAX_TABLE_COLUMNS: usize = 63` above `cell_span`, citing
  `edit::structural`'s `MAX_COLUMNS`.
- `cell_span`: `.max(1)` → `.clamp(1, MAX_TABLE_COLUMNS)`.
- `row_grid_skip`: after `.filter(|n| *n > 0)?`, add
  `.min(MAX_TABLE_COLUMNS)`.
- Occupancy (13604): `.sum()` → `.fold(0usize, usize::saturating_add)`.

All downstream vectors (occupancy resize, `column_prefs`,
`resolve_table_merges`'s `open.resize`, synthesized indent steps) are then
proportional to input size only — an attacker integer can no longer size an
allocation. Residual, accepted: a max-size admitted part (64 MiB) with
millions of cells × 63 can still ask for ~1.7 GB of columns — that is
input-proportional amplification, not the unbounded integer path this finding
documents.

**Must keep passing:** `nested_table_rows_are_not_flattened`,
`fully_deleted_tablegrid_still_collects_rows`, `stretch_stale_spans`
behavior (its `25f1d311bd` fixture uses spans ≤ 4), and the table fixtures
across `tests/` (all real Word tables are ≤ 63 columns — Word itself refuses
wider).

## 3. Chicago numbering label repetition (finding "Large Chicago numbering
values can exhaust memory while reading or rendering a DOCX")

**Source read:** `src/convert/mod.rs:6015-6019` (`chicago_label` repeats the
mark `i / 4 + 1` times — `u32::MAX` ≈ 2^30 marks ≈ 3 GiB UTF-8), `:5849-5862`
(`list_number`/`format_num` shared callers), `:6717-6728`
(`startOverride`), `:5106` (section `w:pgNumType` start).

**Failing test (red):** `src/convert/mod.rs`, module `page_num_fmt_labels`,
new test `chicago_repetition_is_bounded_for_huge_start_values`. Asserts
`chicago_label(u32::MAX)` yields exactly 1024 marks (≤ 3 KiB, first char `‡`),
and that `chicago_label(4 * 1024)` / `chicago_label(4 * 1024 + 4)` keep 1024
marks (saturating). Before the fix the first assertion requests ~3 GiB and
the test dies; after it the label is bounded.

**Minimal source change:** `chicago_label` caps the repetition with
`.min(MAX_CYCLES)` (`MAX_CYCLES = 1024`). One shared formatter, so every
caller (list labels, `startOverride`, section page numbers, Markdown
numbered-list views, MCP agent view, endnote starts) is covered at once.
Numbers 1..=4096 keep their exact current labels.

**Must keep passing:** `chicago_cycles_star_dagger_double_section`,
`page_field_uses_sectpr_chicago` (`tests/convert_docx_to_pdf.rs:19936`),
the `chicago_label(0) == "*"` assertion in the page-number tests.

## 4. Unbounded `w:ilvl` list levels (finding "Unbounded list levels cause
billions of iterations or allocations")

**Source read:** `src/convert/mod.rs:5518-5533` (`resolve_ilvl`'s
`(0..ilvl).rev()` scan), `:5545-5582` (`ensure_level`'s reverse scan and
synthesized level), `:5584-5629` (`next_marker`'s `for level in 0..resolved`
counter inserts), `:6617-6646` (abstract `w:lvl` parse), `:6672-6736`
(`lvlOverride` parse), `:12707-12719` (`num_pr` paragraph `w:ilvl`).

**Failing test (red):** `src/convert/mod.rs`, module `numbering_tests`, new
test `levels_beyond_words_nine_never_expand_counters`. It parses an abstract
that defines level 0 and level `4294967295`, then asserts
`next_marker("1", 4294967295) == "1."` and `next_marker("1", u32::MAX) == "2."`
(the "undefined ilvl continues the parent" rule). Before the fix the defined
huge level resolves, and `for level in 0..4294967295` inserts billions of
counters (OOM); the second call would instead scan billions of levels.
With the fix both return instantly.

**Minimal source change:**
- `const MAX_ILVL: u32 = 8` above `impl Numbering` — Word's nine levels.
- `resolve_ilvl` and `ensure_level` clamp their `ilvl` argument first
  (`ilvl.min(MAX_ILVL)`), bounding every scan, the counter loop, and the
  synthesized indent step. This covers every source of the value: paragraph
  `w:numPr`, styles, `lvlOverride`, and direct `next_marker` callers.
- `parse_numbering` drops `w:lvl` records and `lvlOverride` entries whose
  `ilvl > 8` (Word ignores unsupported level metadata); such keys could
  never be reached after the clamp.

Valid documents (ilvl 0..=8) produce identical output: `min(8)` is the
identity for them and no parse record is dropped.

**Must keep passing:** `a_label_naming_a_deeper_level_is_no_label_but_still_counts`,
`unused_parent_slot_shows_its_start_override`,
`lvl_restart_names_the_level_that_restarts_it`, and the rest of
`numbering_tests` (all use levels 0..=2).

## 5. PDF font subsetting: duplicate sfnt directory records hang the writer
(finding "PDF generation can hang on a font with many retained tables")

**Source read:** `src/convert/pdf.rs:1201-1210` (directory records pushed
with no duplicate check; 32768 × 16-byte `cvt ` records ≈ 512 KiB pass),
`:1294-1310` (every record whose tag is in `SUBSET_TABLES` is retained),
`:1523-1543` (`write_sfnt`'s `while pow * 2 <= n` u16 doubling — at
n = 32768, `pow * 2` overflows: release wraps to 0 and spins forever, debug
panics; `pow * 16` overflows u16 already at 4096 tables),
`src/convert/font_table.rs:212-242` (embedded font entry point that lets the
crafted bytes reach the subsetter).

**Failing test (red):** `src/convert/pdf.rs`, module
`residual_byte_and_geometry_contract_tests`, new test
`duplicate_directory_records_cannot_loop_the_sfnt_writer`. It rebuilds
Carlito's directory with 32768 extra `cvt ` records (offsets shifted, the
duplicates pointing at in-bounds header bytes) and asserts
`subset_keep_gids` returns a program that `ttf_parser::Face::parse` accepts.
Before the fix the u16 doubling panics (debug) or spins (release); after it
the directory dedup keeps one `cvt ` and the subset is written normally.

**Minimal source change:**
- Directory read (`subset_keep_gids`): a `HashSet` drops later records with
  an already-seen tag. Behavior-preserving: every lookup (`table(b"head")`
  …) already took the first record via `find`, so duplicates were
  unreachable; a valid sfnt never repeats a tag.
- `write_sfnt`: the search-metadata computation moves to `u32` and the u16
  header fields go through a saturating `u16::try_from`, so no count can
  wrap, panic, or spin. For every real font (≤ 4095 tables) the emitted
  bytes are identical.

**Must keep passing:** `subset_keeps_ids_and_used_outlines_only`,
`subset_drops_glyph_names_and_unused_metrics`,
`a_collections_first_face_subsets_to_a_plain_font`,
`subset_cmap_and_name_keep_only_what_the_pdf_uses`,
`subset_declines_corrupt_sfnt_directory_but_keeps_valid_empty_and_out_of_range_usage`,
and `tests/convert_docx_to_pdf.rs` (unchanged bytes for real fonts).

## 6. PDF font subsetting: full-u32 cmap enumeration (finding "PDF font
subsetting can enumerate billions of codepoints")

**Source read:** `src/convert/pdf.rs:1434-1450` — `subset_cmap` iterates
`sub.codepoints(..)`, and the closure evaluates `sub.glyph_index(cp)` for
every enumerated codepoint (tuple evaluation) before the `u16::try_from`
filter; a format 12/13 group `[0, u32::MAX]` drives ~4.29 × 10⁹ callbacks.

**Failing test (red):** `src/convert/pdf.rs`, module
`residual_byte_and_geometry_contract_tests`, new test
`a_full_range_cmap_subtable_is_sampled_not_enumerated`. It patches Carlito's
cmap in place with one (3, 10) record over a format 12 table holding a
single `[0, u32::MAX]` group, then asserts `subset_cmap` returns a bounded
format 4 table (`< 100` bytes). Before the fix the enumeration runs
~4.29 billion callback+lookup iterations — minutes in a debug build; the
fix changes only the work, never the output bytes, so no functional assert
can distinguish before from after: the red is observed as that
minutes-long run (a nextest slow-timeout warning in the default profile,
a CI job timeout in practice), not an assertion failure. After the fix,
65535 lookups run in microseconds.

**Minimal source change:** replace the `codepoints` enumeration with a walk
of the output domain — `for cp in 0u16..=0xFFFE { sub.glyph_index(...) }`.
Only BMP codepoints can appear in the generated format 4 subset, so the
produced map is identical (same subtable order, same `or_insert`
first-wins), and no encoded range can size the work.

**Must keep passing:** `subset_cmap_and_name_keep_only_what_the_pdf_uses`
(cmap content for real fonts is byte-identical), the WinAnsi/Identity-H
conversion fixtures in `tests/convert_docx_to_pdf.rs`.

## 7. Legacy .doc aliased piece expansion (finding "Aliased legacy DOC pieces
expand small physical text into enormous logical stories")

**Source read:** `src/legacy_doc.rs:1148-1226` (`main_text` verifies CP
continuity and stream bounds, then materializes one `StoryChar` per logical
CP — aliasing one physical range into thousands of contiguous pieces
defeats both checks), `:915-920` (FIB `ccp_text`), `:180-195` (read
entrypoint), `src/bin/jubarte.rs:3125-3144` (refusal-hint path that parses
the legacy file for a diagnostic).

**Failing test (red):** `src/legacy_doc.rs`, module `tests`, new test
`aliased_pieces_past_the_logical_budget_are_refused`. Two framings over a
1 MiB `WordDocument` stream: 32 pieces all aliasing the same 1 Mi CP range
(story 1 Mi, expansion 32 Mi), and 33 contiguous 1 Mi pieces with
`ccp_text = 33 Mi`. Both must return an error containing
"logical text budget". Before the fix both return `Ok` after materializing
tens of millions of `StoryChar` (hundreds of MB), so the assertions fail;
after it, both are refused before any Vec grows.

**Minimal source change:** `main_text` gains `const MAX_TEXT_CP: u64 =
16 * 1024 * 1024` and two pre-materialization checks (matching the
remediation's "check FIB ccp_text and cumulative piece lengths before
materialization"): refuse `ccp_text` past the budget, and refuse when the
sum of `min(cp_end, ccp_text) - cp_start` over pieces still below `ccp_text`
passes it (the aliasing/overlap case the `ccp_text` check alone misses).
Legitimate fast-save reuse (empty pieces, split surrogates) is far below the
budget and unaffected; the CLI read/conversion and refusal-hint paths share
the refusal.

**Must keep passing:** `a_piece_table_that_does_not_cover_the_story_is_refused`,
the split-surrogate UTF-16 tests beside `main_text`,
`refuses_what_is_not_a_compound_file`, and the corpus `.doc` conversions
(real stories are orders of magnitude below 16 Mi characters; the largest
books are ~5M).

---

## Findings of this round NOT patched here (and why)

- **"Pull-request code runs on shared self-hosted infrastructure"** (CI):
  the remediation is an infrastructure policy decision — which jobs run on
  GitHub-hosted vs. self-hosted runners across `.github/workflows/{ci,
  adoption,docs,md-roundtrip}.yml` and `.github/actions/mbx/action.yml`.
  Rewiring runner labels blind, without the ability to run the workflows,
  can break the project's actual CI. This needs the repository owner's
  runner inventory; not a source patch.
- **"Python CLI output/plan and npm direct-output symlinks can overwrite
  unrelated files"** and **"Python and Node plan recording can overwrite
  unrelated files through staging symlinks"** (low): the remediation the
  findings themselves prescribe is to reuse the native publication design
  (`src/publication.rs`, `docs/FILESYSTEM_PUBLICATION.md`) — an API change
  in the Rust library plus Python (`jubarte-python/python/jubarte_redlines/
  __main__.py:101-133`, `:403-424`, `:883-903`) and Node
  (`jubarte-wasm/cli/bin/jubarte-redlines.mjs:76-112`, `:398-445`) staging
  rewrites, with documented `--force`/dangling-symlink semantics. That is a
  cross-language change whose tests cannot be executed under this ask's
  no-build constraint; doing it blind risks getting the documented
  publication semantics wrong. Recommended follow-up round: expose
  `publication.rs` staging to the Python binding, adopt `O_CREAT|O_EXCL`
  unpredictable staging in `_record` and `wx`-style staging in the Node CLI,
  port `docs/FILESYSTEM_PUBLICATION.md`'s rules into their test suites.

Nothing was patched for findings whose only statements were already covered
by earlier rounds' retained findings (e.g. the XML-depth family): this round
added corroborating evidence to them, and their fixes are owned by the
folders those findings came from.

## Verification to run when Cargo is allowed (not run in this session)

1. `cargo nextest run -E 'test(a_huge_colspan_clamps_to_words_column_limit) + test(huge_spans_and_row_skips_clamp_to_words_column_limit) + test(chicago_repetition_is_bounded_for_huge_start_values) + test(levels_beyond_words_nine_never_expand_counters) + test(duplicate_directory_records_cannot_loop_the_sfnt_writer) + test(a_full_range_cmap_subtable_is_sampled_not_enumerated) + test(aliased_pieces_past_the_logical_budget_are_refused)'`
   — green after the patch; red before it (each new test demonstrates its
   vulnerability: abort/OOM for the span and level tests, panic for the sfnt
   writer in debug, assertion failure for the legacy and Chicago tests, and
   a minutes-long — but eventually passing — run for the cmap test: the fix
   changes only the work, never the output bytes, and `.config/nextest.toml`
   sets slow-timeout termination only in the `default-miri` profile, so in
   the default profile its red shows up as a slow-timeout warning or a CI
   job timeout, not a failure).
2. The keep-passing neighbours listed per fix above, e.g.
   `cargo nextest run -E 'test(subset) + test(numbering) + test(table)'`
   and `cargo nextest run -E 'test(chicago)'`.
3. `cargo fmt --check`, `cargo clippy -- -D warnings`.
4. `jubarte --help` smoke test after the build.
