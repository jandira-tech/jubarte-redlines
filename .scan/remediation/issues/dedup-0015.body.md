@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_9e40f23e742aa43493a0fb92 | Rendering a small DOCX table can exhaust host memory | medium
csf_33c5fede5e569e4c1531e38a | Rendering a small DOCX table can exhaust host memory | medium
csf_5a86b3f5aa767e4e64bd9e71 | Rendering a small DOCX table can exhaust host memory | medium
csf_d46d1a3ef7bb52454cbd3c68 | Large Chicago numbering values can exhaust memory while reading or rendering a DOCX | medium
csf_64b247931cd1f053217fde7a | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_8510b5bc0ddac96422dfcdc6 | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_32625277093e7f286a267e2b | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_09abb95061370ae3d7d342ca | Redaction succeeds while UTF-16 XML retains the removed text | medium
csf_32d60b431e9d66bf961b0a29 | Redaction succeeds while UTF-16 XML retains the removed text | medium
csf_cac980ab6041d4ec91b39fd1 | Repeated relationship IDs trigger quadratic repair work | medium
csf_bff2c0151aeb6ffaa6f96d4a | Repeated relationship IDs trigger quadratic repair work | medium
csf_dea8102957ca1306a0f7756b | Unbounded list levels cause billions of iterations or allocations | medium
csf_6a40bc41bdf34a737ceac40d | Entity decoding repeatedly scans the input suffix | medium
csf_3ec09f0cecec11243e8a8f5e | Entity decoding repeatedly scans the input suffix | medium
csf_eb94605aebd8f08907c46e8c | Entity decoding repeatedly scans the input suffix | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.

## TDD red/green plan
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


## Patch
```diff
diff --git a/src/admission.rs b/src/admission.rs
index d556610..72c2a4e 100644
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -764,8 +764,9 @@ fn check_part_name(name: &str) -> Result<(), AdmissionError> {
 /// A part's text for scanning. Word writes UTF-8, but a part may be UTF-16
 /// with a byte-order mark (SharePoint's `customXml` items often are) and
 /// Word opens it, so it is transcoded; other bytes decode lossily, as the
-/// rest of the engine reads them.
-fn part_text(xml: &[u8]) -> Cow<'_, str> {
+/// rest of the engine reads them. Shared with the redaction leak scan, so
+/// both read the same bytes the same way before certifying a document.
+pub(crate) fn part_text(xml: &[u8]) -> Cow<'_, str> {
     let utf16 = |rest: &[u8], unit: fn([u8; 2]) -> u16| {
         let units = rest.as_chunks::<2>().0.iter().map(|&pair| unit(pair));
         Cow::Owned(
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f9..b62249d 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -148,6 +148,12 @@ fn quoted_printable(s: &str) -> String {
 
 type Props = HashMap<String, String>;
 
+/// Word lays a table out over at most 63 grid columns. An authored HTML
+/// `colspan` (or a row of spans that sums higher) is trimmed to this budget
+/// before any `<w:gridCol>` markup is generated, so one huge attribute
+/// value cannot drive a billion appends of generated XML.
+const MAX_TABLE_COLUMNS: usize = 1024;
+
 /// `selector { prop: value; … }` rules; a selector list shares its block.
 fn parse_css(css: &str) -> HashMap<String, Props> {
     let mut rules: HashMap<String, Props> = HashMap::new();
@@ -469,10 +475,10 @@ impl Builder {
         let cols = table
             .rows
             .iter()
-            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
+            .map(|r| r.iter().fold(0usize, |acc, c| acc.saturating_add(c.span)))
             .max()
             .unwrap_or(1)
-            .max(1);
+            .clamp(1, MAX_TABLE_COLUMNS);
         let col_w = 9360 / cols;
         let mut s = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>");
         if table.border {
@@ -563,7 +569,7 @@ impl Builder {
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
                         .unwrap_or(1)
-                        .max(1),
+                        .clamp(1, MAX_TABLE_COLUMNS),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
@@ -730,7 +736,13 @@ pub(crate) fn decode_entities(s: &str) -> String {
     while let Some(a) = rest.find('&') {
         out.push_str(&rest[..a]);
         let tail = &rest[a..];
-        let semi = tail.find(';').filter(|&e| e <= 10);
+        // The longest supported entity is ten characters (`&#x1F600;`).
+        // Search only that window, never the whole remaining input, so a
+        // run of bare '&' with no ';' anywhere cannot scan quadratically.
+        // The window is taken in bytes (';' is ASCII) so a multibyte
+        // character straddling its edge cannot panic a slice.
+        let window = &tail.as_bytes()[..tail.len().min(11)];
+        let semi = window.iter().position(|&b| b == b';');
         let decoded = semi.and_then(|e| {
             let ent = &tail[1..e];
             let ch = match ent {
@@ -1054,6 +1066,49 @@ mod tests {
                 assert!(!wml.contains("<w:tblBorders>"));
             }
         }
+
+        #[test]
+        fn an_excessive_colspan_is_trimmed_before_grid_generation() {
+            // A single huge colspan must not emit one grid column per
+            // claimed column; spans that overflow a row are clamped as a
+            // whole, while ordinary bounded tables keep their exact spans.
+            for span in ["1000000000", "4294967295"] {
+                let wml = html_to_wml(&format!(
+                    "<table border='0'><tr><td colspan='{span}'>A</td></tr></table>"
+                ));
+                assert_eq!(wml.matches("<w:gridCol ").count(), 1024, "{span}: {wml}");
+                assert!(
+                    wml.contains("<w:gridSpan w:val=\"1024\"/>"),
+                    "{span}: {wml}"
+                );
+            }
+            let summed = html_to_wml(
+                "<table border='0'><tr><td colspan='600'>A</td><td colspan='600'>B</td></tr></table>",
+            );
+            assert_eq!(summed.matches("<w:gridCol ").count(), 1024, "{summed}");
+            assert!(summed.contains(">A</w:t>") && summed.contains(">B</w:t>"));
+            let ordinary = html_to_wml(
+                "<table border='0'><tr><td colspan='2'>A</td><td>B</td></tr></table>",
+            );
+            assert_eq!(ordinary.matches("<w:gridCol ").count(), 3, "{ordinary}");
+            assert!(ordinary.contains("<w:gridSpan w:val=\"2\"/>"));
+        }
+
+        #[test]
+        fn a_bare_ampersand_run_decodes_in_one_linear_pass() {
+            // Long runs of bare '&' with no ';' anywhere: each ampersand
+            // looks no further than the supported entity window instead of
+            // rescanning the remaining input, so decoding stays linear.
+            let n = 200_000;
+            let html = "a&".repeat(n);
+            let decoded = decode_entities(&html);
+            assert_eq!(decoded.len(), 2 * n);
+            assert!(!decoded.contains(';'));
+            // Entities inside the window still decode; unknown or truncated
+            // ones stay literal exactly as before.
+            assert_eq!(decode_entities("&amp;&#65;&#x1F600;"), "&A😀");
+            assert_eq!(decode_entities("&unknown; &unfinished"), "&unknown; &unfinished");
+        }
     }
 }
 
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
index d900e05..b80e18a 100644
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -5505,6 +5505,13 @@ struct Numbering {
     old_label: Option<String>,
 }
 
+/// Word defines nine list levels (0..=8). Every authored `w:ilvl` — a
+/// paragraph's, a style's, an abstract level's, an override's — is clamped
+/// to this domain before it keys or scans numbering state, so a huge level
+/// index can neither walk the integers below it nor allocate a counter per
+/// level.
+const MAX_LIST_ILVL: u32 = 8;
+
 impl Numbering {
     /// The style `(num_id, ilvl)`'s level is linked to, if any.
     fn linked_style(&self, num_id: &str, ilvl: u32) -> Option<&str> {
@@ -5516,6 +5523,10 @@ impl Numbering {
     }
 
     fn resolve_ilvl(&self, abs: &str, ilvl: u32) -> u32 {
+        // Word defines nine list levels (0..=8); a requested level past the
+        // domain is read as the parent-continuation rule below over the
+        // defined keys, never as a scan of the integers beneath it.
+        let ilvl = ilvl.min(MAX_LIST_ILVL);
         let Some(lvls) = self.levels.get(abs) else {
             return ilvl;
         };
@@ -5543,6 +5554,7 @@ impl Numbering {
     /// later. Callers that should follow Word's "undefined ilvl
     /// continues the parent" rule must `resolve_ilvl` first.
     fn ensure_level(&mut self, abs: &str, ilvl: u32) -> Option<NumLevel> {
+        let ilvl = ilvl.min(MAX_LIST_ILVL);
         if let Some(lvl) = self.levels.get(abs).and_then(|m| m.get(&ilvl)).cloned() {
             return Some(lvl);
         }
@@ -6014,8 +6026,14 @@ fn ordinal_text_ones(one: &str) -> String {
 
 fn chicago_label(n: u32) -> String {
     const MARKS: [&str; 4] = ["*", "†", "‡", "§"];
+    // Word's deepest Chicago lists stay at a handful of marks. An authored
+    // start/startOverride near u32::MAX must not allocate a label
+    // proportional to n/4 marks: past this budget the mark keeps cycling
+    // but the repetition stops growing.
+    const MAX_REPEATS: usize = 64;
     let i = n.saturating_sub(1) as usize;
-    MARKS[i % 4].repeat(i / 4 + 1)
+    let repeats = (i / 4 + 1).min(MAX_REPEATS);
+    MARKS[i % 4].repeat(repeats)
 }
 
 fn ideograph_digital_label(n: u32) -> String {
@@ -6618,7 +6636,8 @@ fn parse_numbering_xml(xml: &str, media: impl Fn(&str) -> Option<Vec<u8>>) -> Nu
         for lvl in dom.descendants(abs, Some(&W::name("lvl"))) {
             let ilvl = attr_any(&dom, lvl, "ilvl")
                 .and_then(|s| s.parse().ok())
-                .unwrap_or(0);
+                .unwrap_or(0)
+                .min(MAX_LIST_ILVL);
             if let Some(v) = first_named(&dom, lvl, "lvlRestart")
                 .and_then(|n| attr_any(&dom, n, "val"))
                 .and_then(|s| s.parse().ok())
@@ -6675,7 +6694,8 @@ fn parse_numbering_xml(xml: &str, media: impl Fn(&str) -> Option<Vec<u8>>) -> Nu
             .filter_map(|ov| {
                 let ilvl = attr_any(&dom, ov, "ilvl")
                     .and_then(|s| s.parse().ok())
-                    .unwrap_or(0);
+                    .unwrap_or(0)
+                    .min(MAX_LIST_ILVL);
                 first_named(&dom, ov, "lvl").map(|lvl| (ilvl, parse_num_level(&dom, lvl)))
             })
             .collect();
@@ -6713,7 +6733,8 @@ fn parse_numbering_xml(xml: &str, media: impl Fn(&str) -> Option<Vec<u8>>) -> Nu
         for ov in dom.descendants(num, Some(&W::name("lvlOverride"))) {
             let ilvl = attr_any(&dom, ov, "ilvl")
                 .and_then(|s| s.parse().ok())
-                .unwrap_or(0);
+                .unwrap_or(0)
+                .min(MAX_LIST_ILVL);
             let start = first_named(&dom, ov, "startOverride")
                 .and_then(|n| attr_any(&dom, n, "val"))
                 .and_then(|s| s.parse().ok())
@@ -12711,7 +12732,8 @@ fn num_pr(dom: &Dom, ppr: NodeId) -> (Option<String>, u32) {
     let ilvl = first_named(dom, num, "ilvl")
         .and_then(|n| dom.attribute(n, &W::val()))
         .and_then(|v| v.parse::<u32>().ok())
-        .unwrap_or(0);
+        .unwrap_or(0)
+        .min(MAX_LIST_ILVL);
     let num_id = first_named(dom, num, "numId")
         .and_then(|n| dom.attribute(n, &W::val()))
         .map(str::to_string);
@@ -13601,8 +13623,13 @@ fn table_block(
     let mut occupancy = 0usize;
     let grid_len = cols.len();
     for row in &raw_rows {
-        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
+        let row_span = row.iter().fold(0usize, |acc, c| acc.saturating_add(c.colspan));
+        occupancy = occupancy.max(row_span);
     }
+    // The span budget bounds every column-shaped allocation below (cols
+    // resize, column prefs, merge tracking); saturating sums keep a row of
+    // huge spans from wrapping into a small occupancy either.
+    let occupancy = occupancy.min(MAX_GRID_COLUMNS);
     if cols.len() < occupancy {
         if cols.len() == 1 && occupancy > 1 {
             let each = cols[0] / occupancy as f32;
@@ -14201,6 +14228,13 @@ fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
     }
 }
 
+/// Word lays a table out over at most 63 grid columns. Authored span and
+/// skip counts are trimmed to this budget before any derived column,
+/// preference or merge vector is sized, so a small table with a huge
+/// `w:gridSpan`/`w:gridBefore`/`w:gridAfter` cannot allocate proportional
+/// memory (1024 leaves 16× margin over Word's own limit).
+const MAX_GRID_COLUMNS: usize = 1024;
+
 fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
     let Some(pr) = first_named(dom, cell, "tcPr") else {
         return (1, VMerge::None);
@@ -14212,7 +14246,7 @@ fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|s| s.parse().ok())
         .unwrap_or(1)
-        .max(1);
+        .clamp(1, MAX_GRID_COLUMNS);
     let vmerge = match direct_named(dom, pr, "vMerge") {
         None => VMerge::None,
         Some(n) => match dom.attribute(n, &W::val()) {
@@ -14342,7 +14376,8 @@ fn row_grid_skip(dom: &Dom, row: NodeId, side: &str) -> Option<(usize, PrefWidth
     let span = direct_named(dom, pr, &format!("grid{side}"))
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|v| v.parse::<usize>().ok())
-        .filter(|n| *n > 0)?;
+        .filter(|n| *n > 0)?
+        .min(MAX_GRID_COLUMNS);
     let pref = direct_named(dom, pr, &format!("w{side}"))
         .filter(|n| attr_any(dom, *n, "type").is_none_or(|t| t == "dxa"))
         .and_then(|n| attr_any(dom, n, "w"))
@@ -14607,7 +14642,7 @@ fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>> {
                 {
                     origin.rowspan += 1;
                 }
-                col += span;
+                col = col.saturating_add(span);
                 continue;
             }
             let idx = origins[ri].len();
@@ -14634,14 +14669,18 @@ fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>> {
                 borders: raw.borders,
                 style_fill: false,
             });
-            if open.len() < col + span {
-                open.resize(col + span, None);
+            // Merge tracking is bounded by the same grid budget as the
+            // column vectors; a row whose spans run past it keeps its cells
+            // but stops growing the tracking vector.
+            let tracked = col.saturating_add(span).min(MAX_GRID_COLUMNS);
+            if open.len() < tracked {
+                open.resize(tracked, None);
             }
             let mark = (raw.vmerge == VMerge::Restart).then_some((col, ri, idx));
             for slot in open.iter_mut().skip(col).take(span) {
                 *slot = mark;
             }
-            col += span;
+            col = col.saturating_add(span);
         }
     }
     origins
@@ -36395,6 +36434,22 @@ mod page_num_fmt_labels {
         assert_eq!(chicago_label(5), "**");
     }
 
+    #[test]
+    fn huge_chicago_counters_stay_bounded() {
+        // A list start, startOverride or page start near u32::MAX must not
+        // repeat the mark n/4 times (a gigabyte label): the mark keeps
+        // cycling but the repetition stops growing past the documented
+        // budget, in list markers and section page labels alike.
+        assert_eq!(chicago_label(u32::MAX), "‡".repeat(64));
+        assert_eq!(chicago_label(u32::MAX - 1), "†".repeat(64));
+        assert_eq!(chicago_label(u32::MAX - 2), "*".repeat(64));
+        assert_eq!(chicago_label(1_000_000_000), "§".repeat(64));
+        // Small counters cycle exactly as before.
+        assert_eq!(chicago_label(6), "††");
+        assert_eq!(chicago_label(9), "***");
+        assert_eq!(chicago_label(0), "*");
+    }
+
     #[test]
     fn ideograph_digital_uses_cjk_digits() {
         assert_eq!(ideograph_digital_label(1), "一");
@@ -44450,6 +44505,38 @@ mod numbering_tests {
         assert_eq!(labels, ["1.", "i)", "ii)", "", "i)", "2."]);
     }
 
+    #[test]
+    fn huge_ilvl_values_resolve_over_defined_levels_in_bounded_work() {
+        // A paragraph or style ilvl of u32::MAX over a level-0 list follows
+        // Word's "undefined ilvl continues the parent" rule, and must do so
+        // without scanning the four billion integers below the request or
+        // seeding a counter per level.
+        let mut n = decimal_numbering();
+        assert_eq!(n.resolve_ilvl("0", u32::MAX), 0);
+        assert_eq!(n.next_marker("1", u32::MAX).trim(), "1.");
+        assert_eq!(n.next_marker("1", u32::MAX).trim(), "2.");
+        // Authored levels past Word's nine are clamped at parse, and a
+        // huge startOverride keys the same clamped level.
+        let mut deep = numbering_xml(
+            &lvl(4_294_967_295, "decimal", "%9.", ""),
+            r#"<w:num w:numId="1"><w:abstractNumId w:val="0"/>
+               <w:lvlOverride w:ilvl="4294967295"><w:startOverride w:val="7"/></w:lvlOverride></w:num>"#,
+        );
+        assert!(deep.levels["0"].contains_key(&MAX_LIST_ILVL));
+        assert!(!deep.levels["0"].contains_key(&4_294_967_295_u32));
+        assert_eq!(deep.next_marker("1", 4_294_967_295).trim(), "7.");
+        // Valid nine-level numbering is untouched.
+        let nine = numbering_xml(
+            &(0..=8)
+                .map(|l| lvl(l, "decimal", &format!("%{}.", l + 1), ""))
+                .collect::<String>(),
+            r#"<w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>"#,
+        );
+        for l in 0..=8_u32 {
+            assert!(nine.levels["0"].contains_key(&l), "level {l}");
+        }
+    }
+
     #[test]
     fn unused_parent_slot_shows_its_start_override() {
         // #117: the first ilvl=1 item under startOverride 5 on level 0
@@ -44842,6 +44929,128 @@ mod table_tests {
         (dom, tbl)
     }
 
+    #[test]
+    fn huge_authored_spans_cannot_size_derived_vectors() {
+        // One shallow table whose gridSpan/gridBefore/gridAfter counts are
+        // huge positive integers: every derived column vector stays inside
+        // the grid budget instead of allocating proportional memory, and
+        // the row's real cells survive.
+        let xml = r#"<?xml version="1.0"?>
+<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
+<w:body>
+<w:tbl>
+  <w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid>
+  <w:tr><w:trPr><w:gridBefore w:val="4294967295"/></w:trPr>
+    <w:tc><w:tcPr><w:gridSpan w:val="4000000000"/><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>
+      <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
+    <w:tc><w:tcPr><w:gridSpan w:val="4294967295"/></w:tcPr>
+      <w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>
+  </w:tr>
+  <w:tr><w:trPr><w:gridAfter w:val="2000000000"/></w:trPr>
+    <w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc></w:tr>
+</w:tbl>
+</w:body></w:document>"#;
+        let (dom, tbl) = first_tbl(xml);
+        let sheet = StyleSheet {
+            defaults: Defaults::word(),
+            by_id: HashMap::new(),
+            tables: HashMap::new(),
+            theme: ThemeFonts::default(),
+            latent: true,
+            default_table: true,
+        };
+        let mut numbering = Numbering::default();
+        match table_block(
+            &dom,
+            tbl,
+            &sheet,
+            &mut numbering,
+            &mut AuthorColors::default(),
+            &HashMap::new(),
+            None,
+        ) {
+            Block::Table { cols, rows, .. } => {
+                assert!(
+                    cols.len() <= MAX_GRID_COLUMNS,
+                    "derived columns bounded: {}",
+                    cols.len()
+                );
+                assert_eq!(rows.len(), 2);
+                let texts: Vec<&str> = rows[0]
+                    .iter()
+                    .filter(|c| !c.grid_skip)
+                    .map(|c| c.paras[0].runs[0].text.as_str())
+                    .collect();
+                assert_eq!(texts, ["A", "B"]);
+                let second: Vec<&str> = rows[1]
+                    .iter()
+                    .filter(|c| !c.grid_skip)
+                    .map(|c| c.paras[0].runs[0].text.as_str())
+                    .collect();
+                assert_eq!(second, ["C"]);
+            }
+            Block::Paragraph { .. }
+            | Block::PageBreak { .. }
+            | Block::ColumnBreak
+            | Block::SectionCols { .. } => {
+                panic!("expected table")
+            }
+        }
+    }
+
+    #[test]
+    fn valid_merged_and_skipped_columns_still_lay_out() {
+        // The span budget must not disturb ordinary merged cells and
+        // gridBefore/gridAfter skips (08648d2f, 25f1d311bd shapes).
+        let xml = r#"<?xml version="1.0"?>
+<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
+<w:body>
+<w:tbl>
+  <w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid>
+  <w:tr><w:trPr><w:gridBefore w:val="1"/></w:trPr>
+    <w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p><w:r><w:t>AB</w:t></w:r></w:p></w:tc>
+  </w:tr>
+  <w:tr><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc>
+    <w:tc><w:p><w:r><w:t>y</w:t></w:r></w:p></w:tc>
+    <w:tc><w:tcPr><w:gridSpan w:val="1"/></w:tcPr><w:p><w:r><w:t>z</w:t></w:r></w:p></w:tc>
+  </w:tr>
+</w:tbl>
+</w:body></w:document>"#;
+        let (dom, tbl) = first_tbl(xml);
+        let sheet = StyleSheet {
+            defaults: Defaults::word(),
+            by_id: HashMap::new(),
+            tables: HashMap::new(),
+            theme: ThemeFonts::default(),
+            latent: true,
+            default_table: true,
+        };
+        let mut numbering = Numbering::default();
+        match table_block(
+            &dom,
+            tbl,
+            &sheet,
+            &mut numbering,
+            &mut AuthorColors::default(),
+            &HashMap::new(),
+            None,
+        ) {
+            Block::Table { cols, rows, .. } => {
+                assert_eq!(cols.len(), 3, "declared grid is untouched");
+                assert_eq!(rows.len(), 2);
+                assert!(rows[0][0].grid_skip, "gridBefore keeps its skip cell");
+                assert_eq!(rows[0][1].colspan, 2);
+                assert_eq!(rows[1].len(), 3);
+            }
+            Block::Paragraph { .. }
+            | Block::PageBreak { .. }
+            | Block::ColumnBreak
+            | Block::SectionCols { .. } => {
+                panic!("expected table")
+            }
+        }
+    }
+
     #[test]
     fn fully_deleted_tablegrid_still_collects_rows() {
         // Word addition_removal / file_27 p3 still paints the 10-row
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
index 9865bda..fd0980f 100644
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -98,6 +98,10 @@ pub(crate) use zip32::zip32_plan;
 /// only the first is a repair that cannot guess wrong.
 fn drop_repeated_relationships(rels: &mut Relationships) -> Result<Vec<String>, OpcError> {
     let mut seen: HashMap<&str, &Relationship> = HashMap::new();
+    // Already-reported Ids: hash membership keeps the repair linear — the
+    // Vec is only the deterministic first-seen reporting order. A scan of
+    // the growing Vec made n distinct Ids, each repeated, quadratic.
+    let mut reported: HashSet<&str> = HashSet::new();
     let mut repaired: Vec<String> = Vec::new();
     let mut keep = Vec::with_capacity(rels.items.len());
     for item in &rels.items {
@@ -113,7 +117,7 @@ fn drop_repeated_relationships(rels: &mut Relationships) -> Result<Vec<String>,
                 {
                     return Err(OpcError::InvalidRelationship);
                 }
-                if !repaired.contains(&item.id) {
+                if reported.insert(item.id.as_str()) {
                     repaired.push(item.id.clone());
                 }
                 keep.push(false);
@@ -1100,6 +1104,38 @@ mod tests {
         );
     }
 
+    /// Repair of many distinct identically-repeated Ids is linear work:
+    /// reporting membership is a hash lookup, not a scan of the
+    /// already-reported list, and every Id is still reported exactly once
+    /// in first-seen order with the first record kept.
+    #[test]
+    fn many_distinct_repeated_relationship_ids_repair_once_in_first_seen_order() {
+        let record = |i: usize| {
+            format!(
+                r#"<Relationship Id="rId{i}" Type="{HYPERLINK}" Target="https://r{i}.example/" TargetMode="External"/>"#
+            )
+        };
+        let count = 10_000;
+        let once: String = (0..count).map(record).collect();
+        let twice = format!("{once}{once}");
+        let fs = PartFs::open(&package_with_document_rels(&twice)).unwrap();
+        let repaired = fs.repaired_duplicate_ids();
+        assert_eq!(repaired.len(), count, "each Id reported once");
+        assert_eq!(
+            repaired.first(),
+            Some(&("word/document.xml".to_string(), "rId0".to_string()))
+        );
+        assert_eq!(
+            repaired.last(),
+            Some(&(
+                "word/document.xml".to_string(),
+                format!("rId{}", count - 1)
+            ))
+        );
+        let kept = fs.read_rels_for("word/document.xml").unwrap();
+        assert_eq!(kept.items.len(), count, "one record per Id survives");
+    }
+
     #[test]
     fn the_package_is_written_in_the_source_order_then_new_parts_by_name() {
         // Parts and relationships live in hash maps, so the zip entries came
diff --git a/src/scrub.rs b/src/scrub.rs
index 641d427..c5d9ab7 100644
--- a/src/scrub.rs
+++ b/src/scrub.rs
@@ -387,7 +387,10 @@ fn scrub_properties(pkg: &mut PartFs) {
 /// The parts of `docx` where `needle` still occurs, sorted: in a text node,
 /// an attribute value that can hold text, an XML comment, the joined run
 /// text of a paragraph (a text split across runs), or the raw bytes of a
-/// part that is not XML (UTF-8 or UTF-16LE). An empty needle occurs nowhere.
+/// part that is not XML (UTF-8, UTF-16LE or UTF-16BE). XML parts decode
+/// with admission's BOM-aware policy (a UTF-16 `customXml` item is a part
+/// Word opens, so its text is scanned, not its lossy UTF-8 bytes). An empty
+/// needle occurs nowhere.
 /// An unreadable archive or entry is a finding: an incomplete scan cannot
 /// certify a redaction.
 ///
@@ -422,7 +425,8 @@ pub(crate) fn leaks_within(
         Ok(zip) => zip,
         Err(error) => return vec![format!("the package (cannot read: {error})")],
     };
-    let utf16: Vec<u8> = needle.encode_utf16().flat_map(u16::to_le_bytes).collect();
+    let utf16_le: Vec<u8> = needle.encode_utf16().flat_map(u16::to_le_bytes).collect();
+    let utf16_be: Vec<u8> = needle.encode_utf16().flat_map(u16::to_be_bytes).collect();
     let mut total = 0u64;
     for i in 0..zip.len() {
         let mut entry = match zip.by_index(i) {
@@ -466,9 +470,15 @@ pub(crate) fn leaks_within(
         }
         let xml = name.ends_with(".xml") || name.ends_with(".rels");
         let hit = if xml {
-            xml_holds(&String::from_utf8_lossy(&bytes), needle)
+            // BOM-aware decode, the same one admission scans with: a
+            // UTF-16LE/BE XML part must not fall through to lossy UTF-8,
+            // which cannot see its text and would certify a redaction that
+            // still carries the secret.
+            xml_holds(&crate::admission::part_text(&bytes), needle)
         } else {
-            contains(&bytes, needle.as_bytes()) || contains(&bytes, &utf16)
+            contains(&bytes, needle.as_bytes())
+                || contains(&bytes, &utf16_le)
+                || contains(&bytes, &utf16_be)
         };
         if hit {
             found.insert(name);
@@ -715,6 +725,42 @@ mod public_memory_leak_boundary_tests {
             assert_eq!(snapshot(&PartFs::open(&bytes).unwrap()), original);
         }
     }
+
+    #[test]
+    fn bom_marked_utf16_xml_parts_are_transcoded_before_the_leak_scan() {
+        // A BOM-marked UTF-16LE/BE customXml item is a part Word opens
+        // (SharePoint writes them). Scanning its bytes as lossy UTF-8
+        // cannot see the text and would report a redaction clean while the
+        // secret survives in the shared document.
+        let secret = "PrivateReviewer";
+        let mut pkg = package("<w:p><w:r><w:t>Independent body payload</w:t></w:r></w:p>");
+        let xml = format!("<metadata><!-- {secret} --></metadata>");
+        let utf16 = |be: bool| {
+            let mut bytes = vec![if be { 0xFE, 0xFF } else { 0xFF, 0xFE }];
+            bytes.extend(
+                xml.encode_utf16()
+                    .flat_map(|u| if be { u.to_be_bytes() } else { u.to_le_bytes() }),
+            );
+            bytes
+        };
+        pkg.set_part("customXml/le.xml", utf16(false));
+        pkg.set_part("customXml/be.xml", utf16(true));
+        // Defense in depth on the binary side: a big-endian copy outside an
+        // XML name is still found by its bytes.
+        pkg.set_part(
+            "customXml/be.bin",
+            secret
+                .encode_utf16()
+                .flat_map(u16::to_be_bytes)
+                .collect::<Vec<u8>>(),
+        );
+        let bytes = pkg.to_zip().unwrap();
+        assert_eq!(
+            leaks(&bytes, secret),
+            ["customXml/be.bin", "customXml/be.xml", "customXml/le.xml"]
+        );
+        assert!(leaks(&bytes, "absent reviewer").is_empty());
+    }
     #[test]
     fn decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not() {
         let body = "<w:p><w:pPr><w:spacing w:before='1234' w:after='-1234'/></w:pPr><w:r><w:t>Jane </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>Doe &amp; </w:t></w:r><w:r><w:t>Associates</w:t></w:r></w:p>";
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..243999f 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -751,6 +751,12 @@ fn extract_pseudo_attr(s: &str, key: &str) -> Option<String> {
     Some(after[1..1 + end].to_string())
 }
 
+/// The longest entity body [`unescape_xml_text`] will consider: HTML's
+/// longest named entity is 31 characters, so anything past this window is
+/// literal text rather than a reference, and the search for `;` never
+/// reaches into the unbounded input suffix behind it.
+const MAX_ENTITY_BODY: usize = 32;
+
 /// Decode XML entity and character references in attribute and text
 /// payloads (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&#nnn;`,
 /// `&#xhh;`). Port of `unescapeXmlText`.
@@ -761,6 +767,12 @@ fn extract_pseudo_attr(s: &str, key: &str) -> Option<String> {
 /// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
 /// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
 /// literal. Input with no `&` short-circuits to a clone of `s`.
+///
+/// The `;` that closes a reference is searched only within
+/// [`MAX_ENTITY_BODY`] characters of the `&`: a longer body is not a
+/// reference this decoder accepts (HTML's longest named entity is 31
+/// characters), and scanning the whole input suffix for it made a run of
+/// bare `&` quadratic.
 pub fn unescape_xml_text(s: &str) -> String {
     if !s.contains('&') {
         return s.to_string();
@@ -770,8 +782,9 @@ pub fn unescape_xml_text(s: &str) -> String {
     let mut i = 0;
     while i < chars.len() {
         if chars[i] == '&' {
-            // find ';'
-            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
+            // find ';' within the supported entity-body window
+            let window_end = (i + 1 + MAX_ENTITY_BODY).min(chars.len());
+            if let Some(semi_rel) = chars[i + 1..window_end].iter().position(|&c| c == ';') {
                 let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
                 if let Some(decoded) = decode_entity(&body) {
                     out.push_str(&decoded);
@@ -1069,6 +1082,26 @@ mod xml_lexical_boundary_contract_tests {
         }
     }
 
+    #[test]
+    fn the_entity_search_window_is_bounded() {
+        // A run of bare '&' with no ';' anywhere decodes in one linear
+        // pass: each ampersand searches only the entity-body window, not
+        // the remaining input.
+        let n = 200_000;
+        let s = "a&".repeat(n);
+        assert_eq!(unescape_xml_text(&s).len(), 2 * n);
+        // A body past the window is literal text even when a ';' follows,
+        // and a padded numeric reference no longer decodes: the grammar is
+        // bounded, exactly as the TS `[a-zA-Z]+` regex bounded names.
+        let long_name = format!("&{};", "x".repeat(64));
+        assert_eq!(unescape_xml_text(&long_name), long_name);
+        let padded = format!("&#{}65;", "0".repeat(40));
+        assert_eq!(unescape_xml_text(&padded), padded);
+        // Every supported reference, including the longest real ones,
+        // still decodes.
+        assert_eq!(unescape_xml_text("&#x1F600;&amp;&#X41;"), "😀&A");
+    }
+
     #[test]
     fn literal_attribute_whitespace_reads_as_spaces_references_survive() {
         // XML 1.0 §2.11 + §3.3.3, the value a conforming reader (Word) sees:

```
