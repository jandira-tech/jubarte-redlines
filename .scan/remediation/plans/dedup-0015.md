# dedup-0015 remediation plan (TDD red/green)

Source of truth for this round: `.scan/artifacts/deep_discovery/dedup/dedup-0015/output/result.json`
(54 findings). This round's own contributions are the findings whose code evidence ids
start with `dedup0015` (worker `c92c3039-cca3-4e78-a9a6-7f6ddf62b310`, per
`.scan/artifacts/deep_discovery/dedup/dedup-0015/prompt.md`). 14 findings carry such
evidence; this plan covers all 14. Seven are patched in
`.scan/remediation/patches/dedup-0015.patch`; seven are documented below with the reason
no patch is included (each needs a redesign or an owner decision that cannot be safely
authored blind — see "Deferred findings").

Patch shape: one unified diff over `src/admission.rs`, `src/convert/mod.rs`,
`src/convert/altchunk.rs`, `src/opc/mod.rs`, `src/scrub.rs`, `src/xmllinq/parse.rs`.
It applies with `git apply` from the repository root (verified with
`git apply --check` during preparation). Tests live beside the behavior they protect,
inside each file's existing `#[cfg(test)]` modules, and are deterministic (AGENTS.md).

Verification status of this plan (honest statement): per the task constraints, no
`cargo`/`mbx`/`rustc` command was run — the patch was authored by reading the code, and
checked only with `git apply --check` and a brace/paren-delta comparison against the
pristine files. Every "existing behaviour that must keep passing" item below was
confirmed by reading the cited test in this session, not by executing it. After
applying, run (sequential, per AGENTS.md):

- `cargo nextest run -E 'test(huge_) or test(an_excessive) or test(a_bare_ampersand) or test(the_entity_search) or test(bom_marked) or test(many_distinct_repeated) or test(valid_merged_and_skipped)'`
- `cargo nextest run -E 'test(chicago) or test(ilvl) or test(colspan) or test(entities) or test(repeated_relationship) or test(leaks)'` (guards the surrounding contracts)
- `cargo fmt --check` and clippy with `-D warnings` on the touched files (formatting was
  hand-matched to rustfmt style; it was not run).

---

## Patched findings

### 1. "Rendering a small DOCX table can exhaust host memory" (medium)

- Root cause (read at `src/convert/mod.rs`): `cell_span` parsed `w:gridSpan` with
  `.max(1)` and no ceiling; `row_grid_skip` parsed `w:gridBefore`/`w:gridAfter` the same
  way; `table_block` then computed `occupancy = row.iter().map(|c| c.colspan).sum()` and
  `cols.resize(occupancy…)` sized a `Vec<f32>` (and `resolve_table_merges` an
  `Vec<Option<…>>` via `open.resize(col + span, …)`) directly from those sums. A tiny
  table with `gridSpan=4294967295` multiplies a few package bytes into a ~16 GiB
  allocation before layout; the raw `.sum()`/`col + span` can also wrap.
- Failing test (in patch): `src/convert/mod.rs`, `mod table_tests`,
  `huge_authored_spans_cannot_size_derived_vectors` — builds a two-row table with
  `w:gridBefore w:val="4294967295"`, `w:gridSpan w:val="4000000000"` and
  `w:gridAfter w:val="2000000000"`, calls `table_block`, and asserts
  `cols.len() <= MAX_GRID_COLUMNS` (1024) while rows and their real cell texts
  ("A", "B", "C") survive. Red pre-patch: the call attempts a multi-gigabyte
  `cols`/`open` resize (test aborts on OOM; on 32-bit the sum wraps and the assertion
  fails). Green post-patch: bounded.
- Minimal source change: new `const MAX_GRID_COLUMNS: usize = 1024` (Word's own grid is
  ≤63 columns; 1024 leaves 16× margin). `cell_span` clamps the parsed span with
  `.clamp(1, MAX_GRID_COLUMNS)`; `row_grid_skip` appends `.min(MAX_GRID_COLUMNS)`;
  `table_block` computes occupancy with `saturating_add` and caps it at the budget
  before any resize; `resolve_table_merges` uses `col.saturating_add(span)` and resizes
  `open` only to `min(col + span, MAX_GRID_COLUMNS)`.
- Must keep passing: `tests/convert_docx_to_pdf.rs` gridBefore row (≈line 9366,
  `w:gridBefore w:val="1"` + `w:wBefore w:w="7"`), the `gridSpan w:val="2"` tables in
  `tests/agent_text_view.rs` (≈lines 450, 479), and the in-file `table_tests`
  (`nested_table_rows_are_not_flattened`, `percent_column_preferences…`,
  `fully_deleted_tablegrid_still_collects_rows`). The patch also adds
  `valid_merged_and_skipped_columns_still_lay_out` to pin the ordinary merged/skip
  layout (3 declared gridCol, gridBefore=1 skip cell kept, `colspan 2` kept).

### 2. "Large Chicago numbering values can exhaust memory while reading or rendering a DOCX" (medium)

- Root cause (read at `src/convert/mod.rs:6015`): `chicago_label` did
  `MARKS[i % 4].repeat(i / 4 + 1)` with `i = n - 1`. `w:start`/`w:startOverride`
  (`src/convert/mod.rs:6750`, `6717`) and `w:pgNumType/@w:start` (5091) are untrusted
  u32s; section page labels reach the same formatter (`PageNumFmt::Chicago =>
  format_num(NumFmt::Chicago, self.section_page)` at 32032), as do Markdown
  `list_number` views (`src/markdown/from_docx/mod.rs`).
- Failing test (in patch): `src/convert/mod.rs`, `mod page_num_fmt_labels`,
  `huge_chicago_counters_stay_bounded` — asserts `chicago_label(u32::MAX) ==
  "‡".repeat(64)` (and the −1/−2 cycle neighbours, and 1_000_000_000), plus unchanged
  small counters (`chicago_label(6) == "††"`, `(9) == "***"`, `(0) == "*"`). Red
  pre-patch: `u32::MAX` requests `"‡" × 1_073_741_824` ≈ 3.2 GiB (OOM); green
  post-patch: ≤ 192 bytes.
- Minimal source change: `const MAX_REPEATS: usize = 64` inside `chicago_label`;
  `let repeats = (i / 4 + 1).min(MAX_REPEATS);`. The mark keeps cycling (`i % 4`)
  so labels stay Word-shaped; only the repetition stops growing past the budget. All
  callers (list marker, section page label, `list_number`) go through this one
  formatter, so one bound covers them.
- Must keep passing: `chicago_cycles_star_dagger_double_section`
  (`chicago_label(1/2/5)`), `zero_labels_remain_explicit…` (`chicago_label(0) ==
  "*"`) at `src/convert/mod.rs` ≈36392/56390, and the Markdown numbered-list views
  that share `list_number`.

### 3. "Rendering an HTML chunk can allocate gigabytes from a single colspan value" (medium)

- Root cause (read at `src/convert/altchunk.rs`): the `td`/`th` handler pushed
  `colspan` `.max(1)` with no ceiling, and `end_table` computed
  `cols = max(row-sum of spans)` with `.sum::<usize>()` then emitted one
  `<w:gridCol/>` per column (`for _ in 0..cols`) and `col_w * cell.span` per cell —
  a single authored `colspan=1000000000` drives a billion appends after admission
  and before the DOM sees anything.
- Failing test (in patch): `src/convert/altchunk.rs`, `mod regression_tests`,
  `an_excessive_colspan_is_trimmed_before_grid_generation` — `colspan` of 1000000000
  and 4294967295 each yield exactly 1024 `<w:gridCol />` and
  `<w:gridSpan w:val="1024"/>`; two 600 spans clamp as a row aggregate to 1024 while
  both cells' text survives; an ordinary `colspan='2'` table keeps 3 grid columns and
  its exact span. Red pre-patch: the huge spans generate up to ~85 GB of markup
  (hang/OOM); green post-patch: bounded output.
- Minimal source change: `const MAX_TABLE_COLUMNS: usize = 1024` (same 63-column Word
  budget rationale as above); span push uses `.clamp(1, MAX_TABLE_COLUMNS)`; the row
  aggregate uses `fold(0usize, saturating_add)` and `.clamp(1, MAX_TABLE_COLUMNS)`.
- Must keep passing: `html_tables_keep_spans_empty_cells_and_header_formatting`
  (colspan 2 exact `tcW`/`gridSpan` output), `invalid_or_zero_colspan_falls_back_to_one_column`,
  `unclosed_html_table_cells_are_finalized_once_with_source_spans_and_empty_cells`,
  `ragged_authored_rows_share_one_grid_without_inventing_filler_cells_or_reordering_payload`.

### 4. "Redaction succeeds while UTF-16 XML retains the removed text" (medium)

- Root cause (read at `src/scrub.rs::leaks_within`, ≈466): an XML-named part
  (`.xml`/`.rels`) was scanned via `xml_holds(&String::from_utf8_lossy(&bytes), …)`.
  A BOM-marked UTF-16LE/BE part — exactly the shape admission supports
  (`src/admission.rs::part_text` documents SharePoint's UTF-16 `customXml` items, and
  `utf16_and_non_utf8_parts_are_scanned_not_refused` proves Word opens them) — decodes
  to NUL-interleaved mojibake under lossy UTF-8, so the secret is invisible to the
  scan. `edit`'s `check_redactions` (`src/edit.rs:4727`) then reports the redaction
  clean while the recipient can recover the text. The binary branch also checked only
  UTF-16LE bytes, never BE.
- Failing test (in patch): `src/scrub.rs`, `mod public_memory_leak_boundary_tests`,
  `bom_marked_utf16_xml_parts_are_transcoded_before_the_leak_scan` — a package with
  `customXml/le.xml` (UTF-16LE + BOM) and `customXml/be.xml` (UTF-16BE + BOM) holding
  the secret in an XML comment, plus a big-endian `.bin` copy, must report all three
  parts; an absent needle still reports nothing. Red pre-patch: `leaks()` returns an
  empty Vec (assertion failure — a clean, deterministic red); green post-patch: the
  three parts are reported.
- Minimal source change: `src/admission.rs::part_text` becomes `pub(crate)`
  (doc note added), and `leaks_within` scans XML parts through it — one BOM-aware
  decode shared with admission. The binary branch keeps the raw UTF-8 needle and now
  checks both UTF-16LE and UTF-16BE byte patterns as defense in depth.
- Must keep passing: `public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources`
  (UTF-8 XML and `.bin` owners, idempotence, package round-trip),
  `decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not`,
  `a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is`,
  and admission's `utf16_and_non_utf8_parts_are_scanned_not_refused`.

### 5. "Repeated relationship IDs trigger quadratic repair work" (medium)

- Root cause (read at `src/opc/mod.rs::drop_repeated_relationships`, ≈99): each
  identical duplicate did `if !repaired.contains(&item.id)` — a linear scan of a
  growing `Vec<String>`. n distinct Ids each repeated twice is ~n²/2 string
  comparisons during `PartFs::open`, on shallow metadata well under the byte limits.
- Test (in patch): `src/opc/mod.rs`, `mod tests`,
  `many_distinct_repeated_relationship_ids_repair_once_in_first_seen_order` — a
  `word/_rels/document.xml.rels` with 10,000 distinct Ids, each repeated identically,
  opens with every Id reported exactly once in first-seen order (`rId0` … `rId9999`)
  and exactly one record kept per Id. Honest note: this test pins the repair contract;
  at n=10,000 the pre-fix quadratic pass still completes, so the red for this finding
  is the construction (quadratic work by inspection), not a failing assertion — a
  deterministic operation-count assertion would need instrumentation the patch
  deliberately does not add.
- Minimal source change: a `HashSet<&str> reported` beside the Vec; membership becomes
  `reported.insert(item.id.as_str())` and the Vec keeps only the deterministic
  first-seen order. `HashSet` is already imported in the file.
- Must keep passing: `a_repeated_relationship_id_is_repaired_when_identical_and_refused_otherwise`
  (single duplicate repair, conflicting duplicate `OpcError::InvalidRelationship`
  refusal, clean package empty list).

### 6. "Unbounded list levels cause billions of iterations or allocations" (medium)

- Root cause (read at `src/convert/mod.rs`): `resolve_ilvl` fell back with
  `(0..ilvl).rev().find(…)` and `ensure_level` with the same reverse scan over the raw
  u32; `next_marker` seeds counters `for level in 0..resolved`; `render` loops
  `for i in 0..=ilvl`. `w:ilvl` is parsed unclamped at four sites: the abstract
  `w:lvl` (≈6619), both `w:lvlOverride` reads (≈6676 full-level replacement, ≈6714
  startOverride), and paragraph/style `num_pr` (≈12711). `ilvl=4294967295` over a
  level-0 list scans ~4·10⁹ integers per lookup, and a defined level that high makes
  `ensure_level`'s `format!("%{}.", ilvl + 1)` overflow-panic.
- Failing test (in patch): `src/convert/mod.rs`, `mod numbering_tests`,
  `huge_ilvl_values_resolve_over_defined_levels_in_bounded_work` — asserts
  `resolve_ilvl("0", u32::MAX) == 0` and `next_marker("1", u32::MAX)` returns "1."
  then "2." (Word's undefined-ilvl-continues-parent rule, bounded); an authored
  `w:lvl w:ilvl="4294967295"` and `w:lvlOverride w:ilvl="4294967295"` with
  `startOverride 7` key the clamped level 8 and number from 7 ("7."); a full
  nine-level 0..=8 definition still registers every level. Red pre-patch: the
  first two assertions do not terminate (the 4-billion-integer reverse scan) — the
  test fails by timeout; green post-patch: instant.
- Minimal source change: `const MAX_LIST_ILVL: u32 = 8` (Word's nine levels);
  `.min(MAX_LIST_ILVL)` at all four parse sites; `resolve_ilvl` and `ensure_level`
  clamp the incoming value first (which also bounds `next_marker`'s `0..resolved`
  seeding and `render`'s `0..=ilvl` to ≤ 9 iterations).
- Must keep passing: `a_label_naming_a_deeper_level_is_no_label_but_still_counts`,
  `unused_parent_slot_shows_its_start_override`, and every other
  `numbering_tests` case (Word probes cited there).

### 7. "Entity decoding repeatedly scans the input suffix" (medium)

- Root cause (read at `src/convert/altchunk.rs::decode_entities` and
  `src/xmllinq/parse.rs::unescape_xml_text`): the HTML helper called
  `tail.find(';').filter(|&e| e <= 10)` — `find` scans the whole remaining input
  before the filter rejects it; the XML helper scanned `chars[i+1..]` for `;` and
  built the body string from whatever it found. A bare-`&` run (or
  `&`-heavy `.rels` Target normalization through `PartFs::resolve_rel_target`,
  which calls `unescape_xml_text` per lookup at `src/opc/mod.rs:680`) is quadratic.
- Failing tests (in patch):
  - `src/convert/altchunk.rs`, `mod regression_tests`,
    `a_bare_ampersand_run_decodes_in_one_linear_pass` — 200,000 `"a&"` pairs decode
    with exact length and no `;`; supported/unknown/truncated entities behave
    exactly as before. Red pre-patch: quadratic (~2·10¹⁰ byte compares) — fails by
    timeout; green post-patch: linear.
  - `src/xmllinq/parse.rs`, `mod xml_lexical_boundary_contract_tests`,
    `the_entity_search_window_is_bounded` — the linear case, a 64-char entity body
    that stays literal, and — the deterministic red — a numeric reference padded
    with 40 leading zeros (`&#000…065;`), which used to decode to "A" and now stays
    literal because the grammar is bounded; `&#x1F600;&amp;&#X41;` still decodes.
- Minimal source change: the HTML helper searches an 11-byte window taken with
  `tail.as_bytes()[..tail.len().min(11)]` (byte search; ';' is ASCII, so a multibyte
  character straddling the edge cannot panic a slice) — the accepted entity set is
  byte-for-byte identical to the old `e <= 10` filter. The XML helper gets
  `const MAX_ENTITY_BODY: usize = 32` (HTML's longest named entity is 31 chars) and
  searches `chars[i+1 .. i+1+32]` only; bodies longer than the window are literal.
  The redundant per-lookup decode in `PartFs::resolve_rel_target` is left in place —
  with the bounded window it is linear, and removing it would change behavior for
  raw-target callers.
- Must keep passing: `entities_preserve_unknown_and_invalid_unicode_references`
  (altchunk) and `undecodable_character_references_remain_literal_and_uppercase_hex_decodes`
  (xmllinq) — every listed input/output pair is unchanged by the window (all bodies
  there are ≤ 10 characters), which the new tests also re-assert.

---

## Deferred findings (no patch in this round)

Each is actionable in principle but needs a redesign or an infrastructure decision that
cannot be authored verifiably without running the code; inventing a blind patch would
risk Word-faithful behaviour (AGENTS.md puts Word parity above hardening speed).

1. **"WMF record-boundary failures amplify polygon work and retained text" (medium)** —
   `src/convert/metafile.rs`. Fix requires confining every record handler to a slice of
   its declared extent and validating counts (polygon/polypolygon/polyline points,
   EXTTEXTOUT options/rectangle/count, TEXTOUT padding) with checked arithmetic across
   many record types. Too large to author blind with confidence; needs the WMF fixture
   corpus and a build loop.
2. **"ZIP footer and count interpretation mismatch bypasses entry-count admission" (medium)** —
   `src/opc/archive.rs` / `src/admission.rs`. The fix must reconcile this crate's
   preindex footer parser with `zip` 9.0.5's validated EOCD/ZIP64 selection
   (dependency-version-specific behaviour). Wrong EOCD-selection assumptions would
   reject valid packages; not attempted without running the suite.
3. **"Metadata scrub leaves property parts under alternate names" (medium)** —
   `src/scrub.rs`. Remediation is to resolve core/app/custom property parts through
   package relationships and content types and to remove part/relationship/content-type
   records coherently, plus a postcondition check — a redesign of
   `scrub_properties`/`apply` with Word-fidelity risk for intentional
   title/subject/application retention.
4. **"Rendering an admitted DOCX can retain gigabytes of duplicate embedded fonts" (medium)** —
   `src/convert/font.rs` / `font_table.rs`. Fix is a decoded-font cache keyed by
   (part, effective key) shared as `Arc` plus an aggregate budget across all four style
   slots; needs the font-loading tests to validate no double-decode regressions.
5. **"Python CLI output/plan and npm direct-output symlinks can overwrite unrelated files" (low)** and
   **"Python and Node plan recording can overwrite unrelated files through staging symlinks" (low)** —
   `jubarte-python/python/jubarte_redlines/__main__.py`, `mcp_server.py`,
   `jubarte-wasm/cli/bin/jubarte-redlines.mjs`. Fix is exclusive unpredictable staging
   (`O_EXCL`/`mkstemp`/`wx`) plus atomic no-clobber publication in two non-Rust
   adapters, with their own test runners; not verifiable from this Rust-only patch.
6. **"Pull-request code runs on shared self-hosted infrastructure" (medium)** —
   `.github/workflows/{ci,adoption,docs,md-roundtrip}.yml` share self-hosted runners
   with PRs. The fix is an infrastructure/policy decision (GitHub-hosted or disposable
   runners for fork PRs, cache-write authorization server-side) that changes how this
   repository's own CI and mbx cache behave; an owner must choose the runner topology
   before any patch is written.

---

## How the patch preserves functionality

- Every budget (1024 grid columns, 64 Chicago repeats, 8 list levels, 32-char entity
  bodies, 11-byte HTML entity window) sits far above anything Word itself produces
  (63-column grid maximum, nine list levels, ≤31-char entity names) and only trims
  attacker-scale values, so Word-valid documents render identically.
- Clamping keeps content: overspanned cells and skipped columns stay in the layout,
  Chicago marks keep cycling, undefined high ilvl still resolves to the parent level
  (Word's rule), and over-window XML "entities" stay literal exactly as unknown
  entities always did.
- The redaction fix only widens the scan (BOM-aware decode plus a BE byte check);
  no previously reported leak can become unreported, and `part_text` is shared with
  admission rather than duplicated.
- No `#[allow]` suppressions are introduced; no behaviour of the cited keep-passing
  tests is redefined.
