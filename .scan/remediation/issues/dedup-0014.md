@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings being tackled

### csf_0b398d69c585073b8c510a3d — Rendering a long paragraph performs quadratic suffix scans (severity: medium)

**Summary:** Wrapping computes every remaining token suffix before line breaking, even when the paragraph contains no tabs and does not need suffix widths.

**Remediation:** Compute suffix widths in a single reverse pass with resets at tab boundaries, or skip suffix work for tab-free segments.

Locations: `src/convert/mod.rs` 33295-33309 (user_input), 33319-33327 (sink).

### csf_91c179305a37cdf9c1219fe0 — Comparing large tables can allocate a quadratic matrix (severity: medium)

**Summary:** Table alignment allocates an unchecked (old_rows+1) by (new_rows+1) usize matrix. A few-megabyte pair of 20,000-row tables can request about 3.2 GB before output.

**Remediation:** Check row products against a comparison work/memory budget before all nested scans/allocations and use bounded-memory alignment or a typed refusal for excessive work.

Locations: `src/comparer/lcs.rs` 4080-4087 (entrypoint); `src/comparer/lcs_table.rs` 638-642 (propagation), 51-65 (root_control), 147-159 (sink).

### csf_eb4256ff45478e77c3906750 — Scanning a document for retained text can overflow the stack (severity: medium)

**Summary:** Public `scrub::leaks()` enforces ZIP byte budgets but directly parses ordinary XML parts without the admission depth/tag policy. Any nonempty search needle activates this unchecked XML scan even when the needle is absent from the malicious XML; an empty needle returns immediately. Normal edit input admission may block deep conventionally named input before its postcheck.

**Remediation:** Validate each XML part with active depth/tag limits before DOM construction, reporting policy failure as a leak-scan refusal, or use a bounded streaming scan. Preserve deliberate empty-needle behavior; for nonempty needles, XML policy or encoding failures must produce a fail-closed incomplete-scan diagnostic even when the searched text is absent. Prefer the existing checked XML parser and preserve the failing part's context.

Locations: `src/scrub.rs` 403-425 (entrypoint), 448-475 (propagation), 527-533 (root_control); `src/xmllinq/parse.rs` 660-663 (sink).

### csf_e02194eb587697016bdfe08d — Rendering nested HTML tables bypasses the XML depth limit (severity: medium)

**Summary:** HTML table expansion runs after package admission and can generate arbitrarily deep WordprocessingML, which is passed to the unchecked recursive DOM parser. The supplied corroboration preserves the innermost-paragraph prerequisite for retaining nested cell WML and confirms that huge colspan is unnecessary; exact abort depth is platform-dependent.

**Remediation:** Bound HTML nesting/generated output and validate expanded WML under active depth/tag limits before DOM construction.

Locations: `src/convert/altchunk.rs` 543-550 (user_input), 495-517 (propagation); `src/convert/mod.rs` 789-799 (root_control); `src/xmllinq/parse.rs` 660-663 (sink).

---

## TDD remediation plan

# dedup-0014 remediation plan — TDD red/green

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0014/`
Reducer round: `dedup-0014`, claimedWorkerId `151e01f5-c817-4557-8cb8-6366387dad22`
(verified in `output/../../prompt.md`).

## Scope

`output/result.json` holds 50 deduplicated findings. This round's **own**
contributions are the evidence refs prefixed `dedup0014-`; the four findings
below are sourced **solely** from worker `151e01f5-…` (`provenance.sourceFindingIds`
have no other worker) and are the ones this plan fixes:

| # (result.json idx) | ruleId | Vulnerability |
|---|---|---|
| 46 | `quadratic-paragraph-suffix-scan` | `wrap_runs_segment` computes every token-suffix tail quadratically |
| 47 | `unbounded-table-alignment-matrix` | table alignment allocates unchecked quadratic DP matrices |
| 48 | `unchecked-privacy-xml-recursion` | `scrub::leaks` parses XML parts without the admission depth/tag policy |
| 49 | `post-expansion-xml-depth-bypass` | altChunk HTML expansion re-parses post-admission XML without depth limits |

Eleven further findings carry `dedup0014-*` evidence but are inherited
merges dominated by earlier rounds' workers (idx 1, 3, 4, 5, 6, 9, 13, 21,
22, 28, 29 — e.g. the altChunk colspan allocation, renamed-aux-XML depth
bypass, tab-leader strings, legacy DOC amplification). They are **out of
scope here**: their root evidence and refs belong to earlier dedup rounds
(`dedup0002`…`dedup0013`), which own their remediation. This plan does not
touch them, and nothing in this patch changes their reachability.

All four in-scope findings are CWE-407/„uncontrolled resource consumption"
class: untrusted document bytes cross a public boundary (`convert`,
`compare`, `scrub`) and select CPU/memory/stack use no budget bounds.

## Verification status (honest)

- `git apply --check .scan/remediation/patches/dedup-0014.patch` from the
  repository root: **PASSED** (run in this session; checks all 5 files).
- `cargo fmt`, `cargo clippy -D warnings`, `cargo nextest run`: **NOT RUN** —
  the ask forbids running cargo/mbx/rustc in this checkout. New code was
  written to match rustfmt output by construction (line widths, chain
  splits, let-chain forms copied from the same files) and uses no
  `#[allow]`.
- Line numbers cited below were read in this session from the current
  checkout.
- Second-pair-of-eyes check (reviewer session): `git apply --check` re-run
  **PASSED** on the corrected patch, and the patch was applied to a scratch
  copy of the five files — the result is brace-balanced and semantically
  checked (anchors, types, test idioms and route preconditions verified
  against the checkout). One patch defect was found and fixed in place: the
  final hunk (`tests/convert_docx_to_pdf.rs`) declared 5 old lines but its
  body carried 3, and three added lines inside `nested_tables_html` lacked
  their `+` prefix; this git accepted the malformed hunk leniently, but a
  stricter `git apply`/`patch(1)` could reject it as corrupt. The hunk is
  now a canonical append (`@@ -49222,3 +49222,81 @@`, 3 context + 78
  additions) that produces a byte-identical result. Cargo checks remain
  **NOT RUN** in the reviewer session too (same prohibition).

---

## 1. Quadratic paragraph suffix scan (`quadratic-paragraph-suffix-scan`)

**Root cause** (evidence `dedup0014-n3-wrap-unit`, `dedup0014-n3-wrap-tail`):
`wrap_runs_segment` (`src/convert/mod.rs:33228`) splits paragraph runs into
word/space units, then eagerly builds `tails` — for **every** unit index it
scans the whole remaining suffix until a tab-containing unit
(`src/convert/mod.rs:33319-33327`). A tab-free paragraph of N units pays
N·(N−1)/2 unit visits. `tails[ui]` is read in exactly one place,
`src/convert/mod.rs:33374` inside the `tab_w` closure, which only runs for
tab-containing units (`tab_unit`, `src/convert/mod.rs:33334`, guards at
`33393-33397` and `33425-33429`) — verified by grep over the function body:
the only other mention is the binding itself.

**Fix**: new `wrap_tab_tails(&units)` (`src/convert/mod.rs`, inserted before
`wrap_runs_segment`) computes a tail **only for tab units**, using the
byte-identical per-index expression (same `take_while`/`sum` chain, so f32
summation order — and therefore every rendered tab position — is unchanged).
Tab units' suffix spans are disjoint (each stops at the next tab unit), so
total work is O(N); a tab-free paragraph computes nothing. `wrap_runs_segment`
now calls `let tails = wrap_tab_tails(&units);` — no other statement in the
function changes.

**Tests** (new module `wrap_tab_tail_tests` in `src/convert/mod.rs`, after
`wrap_runs_segment`):

- RED: `wrapping_a_tab_free_paragraph_is_not_quadratic` — wraps one
  paragraph of 60 000 space-separated words (~120 000 units) through
  `wrap_runs_tabbed` with `tabs = None` and asserts it finishes in under
  30 s. Pre-fix the eager scan does ~7.2·10⁹ unit visits (minutes in a
  debug build), so the bound fails; post-fix the work is linear
  (milliseconds). This is a complexity-cliff bound, not a stopwatch: the
  gap is >4 orders of magnitude. It was **not executed** in this session
  (no cargo); the 30 s margin assumes nextest's default debug profile.
- GREEN (behaviour lock, deterministic): `tab_tails_match_the_scanned_definition_and_skip_tab_free_units`
  — hand-built units with two tab units assert `wrap_tab_tails` equals the
  old eager definition at every tab index (`tails[1] == 18.0` stops before
  the second tab; `tails[4] == 9.0` runs to the end) and is `0.0` at
  never-read non-tab indices.

**Must keep passing**: every layout/wrap fixture in `src/convert/mod.rs`'s
755 inline tests (tab stops, centred/right alignment `5fb9cedf`, dotted
leaders `air_pollution_permit_form`, squeeze rules) — the per-index tail
values are bit-identical, so line breaking cannot move. Existing callers at
`src/convert/mod.rs:26142`, `33199`, `53041`, `53132-53209`, `58584`,
`61402`, `61439`, `62807` are untouched.

## 2. Unbounded table alignment matrix (`unbounded-table-alignment-matrix`)

**Root cause** (evidence `dedup0014-n7-lcs-*`):
`apply_lcs_to_table_rows` (`src/comparer/lcs_table.rs:57`) and
`align_rows_by_shared_words` (`src/comparer/lcs_table.rs:153`) allocate
`vec![vec![0usize; n + 1]; m + 1]` straight from document row counts — a
20 000×20 000 pair requests ~3.2 GB before output. Reachable from
`do_lcs_algorithm_for_table` (`src/comparer/lcs_table.rs:626`): the
shared-word route (Word mode, `settings.merge_replaced_paragraphs`, line
638) and the same-row-count content-LCS route (line 666).

**Fix**: `const TABLE_ROW_CELL_CAP: usize = 1_000_000;` in
`src/comparer/lcs_table.rs`, following the crate's existing budget pattern
(`LCS_CELL_CAP` in `src/comparer/cross_para.rs:37`, gate style of
`cross_para.rs:1562` and `lcs.rs:9754`):

- `shared_word_row_sequences` declines (`None`) when
  `rows1.len().saturating_mul(rows2.len()) > TABLE_ROW_CELL_CAP` before any
  tokenization or allocation (`src/comparer/lcs_table.rs`, after the empty
  check). `None` is the route's existing "not mine" contract — the caller
  falls through to the horizontal-partition, positional and whole-table
  routes, exactly as for merged tables or content-free rows.
- The same-row-count content-LCS call is guarded with
  `total.saturating_mul(total) <= TABLE_ROW_CELL_CAP`; over-cap pairs take
  the very next statement, `positional_rows` (line 672), which is that
  branch's existing fallback.

`apply_lcs_to_table_rows` itself stays a pure DP (its only reachable
callers are the two gated sites; the other call sites are its own tests).

**Tests** (new, in `deterministic_table_dispatch_tests`,
`src/comparer/lcs_table.rs`):

- RED: `oversized_row_pairs_decline_the_shared_word_matrix` — two 1001-row
  tables built with the module's `source()` helper (labels `old row {i}` /
  `new row {i}`, so shared tokens exist and no row is identical — every
  other precondition of the route holds). Asserts the product exceeds the
  cap and `shared_word_row_sequences(...)` returns `None`. Pre-fix it
  fills a 1002×1002 matrix and returns `Some` → assertion fails.
- RED: `oversized_same_count_tables_stay_positional_without_the_matrix` —
  1001 rows a side, first row identical, 1000 changed (the
  `hash_diff > total/3 && < total` shape that selects the content LCS),
  `powertools_faithful` settings (`merge_replaced_paragraphs: false`, so
  the shared-word route is skipped — same setup as the existing
  `seven_aligned_rows_use_content_lcs_while_six_use_positional_pairs`).
  Asserts `do_lcs_algorithm_for_table` pairs all 1001 rows with status
  `Unknown` (positional). Pre-fix the content LCS marks ~1000 rows
  Deleted/Inserted → assertion fails.

**Must keep passing**: `shared_word_alignment_tests` (direct
`align_rows_by_shared_words` cases, unchanged function),
`apply_lcs_to_table_rows` tests (`deterministic_table_dispatch_tests`),
`seven_aligned_rows_use_content_lcs_while_six_use_positional_pairs`
(7 rows ≪ cap, content LCS still fires), and every corpus/fidelity
comparison — real fixture tables are tens to low hundreds of rows, far
under 1 000 000 cells.

## 3. Privacy scan parses XML without the depth/tag policy (`unchecked-privacy-xml-recursion`)

**Root cause** (evidence `dedup0014-n8-*`): `leaks_within`
(`src/scrub.rs:409`) enforces ZIP byte budgets (`preindex_budget`,
per-part caps, `tally_inflated`) but its XML branch calls `xml_holds`
(`src/scrub.rs:527`), which builds a DOM with the **permissive** recursive
`parse_xdocument` (`src/xmllinq/parse.rs:98`, recursion at `:660`). A
`.xml` part nested a few thousand elements deep passes every byte budget
and overflows the stack; `preindex_budget` checks neither depth nor tags
(`src/admission.rs:399-408`).

**Fix**:

- `src/admission.rs`: `check_xml_depth` (line 788) becomes `pub(crate)`.
  It is the exact gate package admission applies to every `.xml`/`.rels`
  part (`src/admission.rs:543`, `:569`): streaming quick-xml scan, tolerant
  tag policy, `max_xml_depth` (256 under `InputLimits::compare()`), and it
  handles UTF-16/BOM via `part_text`. Body unchanged.
- `src/scrub.rs` `leaks_within`: after the inflated tally, an XML-named
  part is first checked with `crate::admission::check_xml_depth(&name,
  &bytes, limits)`; on refusal the scan reports
  `format!("{name} ({error})")` and stops — the same fail-closed shape the
  budget refusals already use (`src/scrub.rs:443-466`) and the contract the
  existing `unreadable_entry_is_a_finding` test states: a part that cannot
  be scanned structurally cannot certify a redaction.

**Test** (RED, deterministic): `deeply_nested_xml_part_is_a_refusal_not_a_dom_parse`
in `src/scrub.rs`'s `bounded_input_tests` — a two-entry ZIP whose
`word/deep.xml` holds 300 nested `<a>` elements (fits every byte budget).
Asserts the findings name `word/deep.xml` and carry `nests deeper than
256`. Pre-fix the findings are empty (the DOM parses 300 deep without
incident and finds nothing) → assertion fails.

**Must keep passing**: `public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources`,
`decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not`
(well-formed XML parts are unaffected — the gate passes and `xml_holds`
runs exactly as before), and the `bounded_input_tests` set
(`compressed_limit`, `count_limit`, `part_limit`, `aggregate_limit`,
`exact_budget_control` — all `.bin`/budget paths untouched). Note one
intentional behaviour change, required by the remediation ("reporting
policy failure as a leak-scan refusal"): a `.xml`/`.rels` part that is
malformed or over-deep is now a named refusal instead of a raw byte scan;
before it could only ever under-report.

## 4. altChunk expansion bypasses the XML depth limit (`post-expansion-xml-depth-bypass`)

**Root cause** (evidence `dedup0014-n11-*`): `with_layout`
(`src/convert/mod.rs:769`) admits the package (XML depth ≤ 256 per part),
then — for `w:altChunk` documents — rewrites the main part with
`altchunk::expand` (`src/convert/mod.rs:790-796`). The HTML importer nests
tables without a depth cap (`src/convert/altchunk.rs:543-550`) and emits
nested `w:tbl`/`w:tr`/`w:tc` WML inside the parent cell
(`src/convert/altchunk.rs:495-513`); `chunk.html` is not an XML part, so
admission never scanned it. The expanded document then reaches the
unchecked recursive `parse_xdocument` (`src/convert/mod.rs:798`) — ~3
elements of depth per nesting level, unbounded.

**Fix**: in `with_layout`, the expanded document is re-checked with the
**same** admission gate before parsing:
`crate::admission::check_xml_depth(&main, expanded.as_bytes(),
crate::admission::InputLimits::compare())`, mapping a refusal to
`ConvertError::OpenPackage(error.to_string())`. Using admission's tolerant
gate (not the stricter `validate_xml_with_limits`) matters: the original
`document.xml` passed exactly that policy, so no previously-admitted
document becomes invalid unless the expansion itself deepens past 256 —
which is precisely the case being refused. Non-altChunk documents take the
unchanged `else` branch (no extra pass, no behaviour change).

**Tests** (new in `tests/convert_docx_to_pdf.rs`, builder `altchunk_docx`
— a minimal package whose body is `<w:altChunk r:id="rIdChunk"/>` resolved
through `word/_rels/document.xml.rels` to `word/chunk.html` — and
`nested_tables_html(levels)`):

- RED: `conversion_refuses_altchunk_expansion_past_the_xml_depth_limit` —
  90 nested single-cell HTML tables (~275 nested WML elements) must be
  refused with an error containing `nests deeper than 256`. Pre-fix the
  document parses (270-ish frames are survivable in a test thread) and
  converts → `expect_err` fails. (The unbounded end of the same path —
  ~10⁵ levels — is a stack-overflow abort, which cannot be asserted
  cleanly; the 256 boundary is the enforced policy.)
- GREEN (preservation): `bounded_altchunk_table_nesting_still_converts` —
  60 levels (~186 elements, under the budget) still renders
  (`pdf_page_count(&pdf) >= 1`), holding the remediation requirement that
  bounded nested tables keep rendering.

**Must keep passing**: the altChunk unit tests in
`src/convert/altchunk.rs` (`an_alt_chunk_is_replaced_by_its_paragraphs`,
`expansion_keeps_surrounding_content_and_chunk_order`,
`omitted_cell_and_row_end_tags_keep_the_table`, …) — `expand` and
`html_to_wml` are untouched; every conversion test whose documents stay
within the depth budget (all real fixtures: admission already caps their
parts at 256) takes an identical path. The 90-level refusal is new
behaviour exactly at the policy boundary.

---

## Files touched

| File | Change |
|---|---|
| `src/convert/mod.rs` | `wrap_tab_tails` helper + call; altChunk expansion depth re-check; `wrap_tab_tail_tests` module |
| `src/comparer/lcs_table.rs` | `TABLE_ROW_CELL_CAP` + two gates; two cap tests |
| `src/scrub.rs` | depth/tag gate in `leaks_within`; deep-XML refusal test |
| `src/admission.rs` | `check_xml_depth` made `pub(crate)` (body unchanged) |
| `tests/convert_docx_to_pdf.rs` | `altchunk_docx`/`nested_tables_html` builders + two tests |

Patch: `.scan/remediation/patches/dedup-0014.patch` (applies with
`git apply` from the repository root — verified with `git apply --check`).

Post-landing verification (for the committer, per AGENTS.md): `cargo fmt
--check`, `cargo clippy -D warnings`, then the touched suites —
`cargo nextest run -E 'test(wrap_tab_tail)'`, `cargo nextest run -E
'test(deterministic_table_dispatch)'`, `cargo nextest run -E
'test(bounded_input)'`, `cargo nextest run --test convert_docx_to_pdf` —
and the neighbouring wrap/compare suites for regressions.

---

## Patch

```diff
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -788,9 +788,21 @@
         .ok_or(ConvertError::MissingDocument)?;
     // Word merges `w:altChunk` content (HTML / MHT) into the body on open.
     let xml = if xml.contains("<w:altChunk") {
-        altchunk::expand(&xml, |rid| {
+        let expanded = altchunk::expand(&xml, |rid| {
             rel_target_path(&pkg, &main, rid).and_then(|p| pkg.part_bytes(&p).map(<[u8]>::to_vec))
-        })
+        });
+        // Admission checked the package's own parts, not this rewritten
+        // body: nested HTML tables expand to WordprocessingML far deeper
+        // than any part was, and the DOM parser recurses per element.
+        // Re-check the expanded document under the same admission gate
+        // (depth and tag policy) before it is parsed.
+        crate::admission::check_xml_depth(
+            &main,
+            expanded.as_bytes(),
+            crate::admission::InputLimits::compare(),
+        )
+        .map_err(|error| ConvertError::OpenPackage(error.to_string()))?;
+        expanded
     } else {
         xml
     };
@@ -33225,6 +33237,28 @@
     (lines, ends_br)
 }
 
+/// Tail widths for [`wrap_runs_segment`]: for every tab unit, the width of
+/// the units after it up to the next tab. Only a tab unit ever consults its
+/// tail (`tab_w` runs for tab units alone), and one tab unit's span ends
+/// where the next tab unit begins, so scanning just the tab units' suffixes
+/// — with the same per-index scan as before — yields the same values with
+/// linear total work. A tab-free paragraph, whose eager per-unit suffix
+/// scans cost N*(N-1)/2 unit visits, now computes nothing.
+fn wrap_tab_tails(units: &[(Vec<WrapPiece<'_>>, bool)]) -> Vec<f32> {
+    let mut tails = vec![0.0; units.len()];
+    for (i, (u, _)) in units.iter().enumerate() {
+        if u.iter().all(|(_, tok, _)| !tok.contains('\t')) {
+            continue;
+        }
+        tails[i] = units[i + 1..]
+            .iter()
+            .take_while(|(u, _)| u.iter().all(|(_, tok, _)| !tok.contains('\t')))
+            .map(|(u, _)| u.iter().map(|(_, _, w)| w).sum::<f32>())
+            .sum();
+    }
+    tails
+}
+
 fn wrap_runs_segment(
     fonts: &Fonts,
     runs: &[TextRun],
@@ -33316,15 +33350,7 @@
     // The text after each unit up to the next tab: a centre or right stop
     // sets it back by half or all of it, as at paint time. Measured from
     // the stop alone, 5fb9cedf's centred title overflowed and wrapped.
-    let tails: Vec<f32> = (0..units.len())
-        .map(|i| {
-            units[i + 1..]
-                .iter()
-                .take_while(|(u, _)| u.iter().all(|(_, tok, _)| !tok.contains('\t')))
-                .map(|(u, _)| u.iter().map(|(_, _, w)| w).sum::<f32>())
-                .sum()
-        })
-        .collect();
+    let tails = wrap_tab_tails(&units);
     for (ui, (unit, is_space)) in units.into_iter().enumerate() {
         let mut w: f32 = unit.iter().map(|(_, _, w)| w).sum();
         // A tab jumps to the next stop from where it stands (00996ee5's
@@ -33561,6 +33587,81 @@
 
 #[cfg(test)]
 #[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod wrap_tab_tail_tests {
+    use super::*;
+
+    fn embedded_fonts() -> Fonts<'static> {
+        let mut fonts = Fonts::new();
+        // Use the dependency's embedded catalogue to bypass every system
+        // font override: unit tests must not read installed font files.
+        for face in FaceId::all() {
+            let key = face.key();
+            fonts.insert_embedded(&key.family, key.bold, key.italic, face.bytes());
+        }
+        fonts
+    }
+
+    #[test]
+    fn tab_tails_match_the_scanned_definition_and_skip_tab_free_units() {
+        let run = TextRun::new("", default_run_style());
+        let unit = |toks: &[(&str, f32)]| {
+            toks.iter()
+                .map(|&(tok, w)| (&run, tok, w))
+                .collect::<Vec<WrapPiece<'_>>>()
+        };
+        let units = vec![
+            (unit(&[("one", 10.0)]), false),
+            (unit(&[("\t", 0.0)]), true),
+            (unit(&[("two", 4.0), ("three", 6.0)]), false),
+            (unit(&[("four", 8.0)]), false),
+            (unit(&[("\t", 0.0)]), true),
+            (unit(&[("five", 2.0)]), false),
+            (unit(&[("six", 7.0)]), false),
+        ];
+        let tails = wrap_tab_tails(&units);
+        assert_eq!(tails.len(), units.len());
+        // The eager definition the wrap loop used to compute for every
+        // unit, evaluated per index; only a tab unit's tail is ever read.
+        let scanned = |i: usize| {
+            units[i + 1..]
+                .iter()
+                .take_while(|(u, _)| u.iter().all(|(_, tok, _)| !tok.contains('\t')))
+                .map(|(u, _)| u.iter().map(|(_, _, w)| w).sum::<f32>())
+                .sum::<f32>()
+        };
+        for i in 0..units.len() {
+            let tab = units[i].0.iter().any(|(_, tok, _)| tok.contains('\t'));
+            assert_eq!(tails[i], if tab { scanned(i) } else { 0.0 }, "unit {i}");
+        }
+        // The first tab's tail stops at the second tab; the last runs out.
+        assert_eq!(tails[1], 18.0);
+        assert_eq!(tails[4], 9.0);
+    }
+
+    #[test]
+    fn wrapping_a_tab_free_paragraph_is_not_quadratic() {
+        // One paragraph of 60 000 space-separated words forms ~120 000
+        // units. The eager tail scan visited N*(N-1)/2 of them (~7.2e9)
+        // before any line was laid out; with no tab unit to read a tail,
+        // the work must stay linear. The bound is generous on purpose: it
+        // is a complexity cliff (minutes in a debug build before), not a
+        // stopwatch.
+        let words = "word ".repeat(60_000);
+        let runs = vec![TextRun::new(words, default_run_style())];
+        let fonts = embedded_fonts();
+        let started = std::time::Instant::now();
+        let lines = wrap_runs_tabbed(&fonts, &runs, 400.0, 400.0, false, None).0;
+        let elapsed = started.elapsed();
+        assert!(lines.len() > 1000, "60 000 words at 400pt wrap to thousands of lines");
+        assert!(
+            elapsed.as_secs() < 30,
+            "wrapping took {elapsed:?}; the suffix scan is quadratic again"
+        );
+    }
+}
+
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
 fn layout(
     fonts: &Fonts,
     page: &PageSetup,
diff --git a/src/comparer/lcs_table.rs b/src/comparer/lcs_table.rs
--- a/src/comparer/lcs_table.rs
+++ b/src/comparer/lcs_table.rs
@@ -18,6 +18,13 @@
 use crate::namespaces::{PT, W};
 use crate::xmllinq::{Dom, NodeId};
 
+/// Largest row-by-row DP matrix the table alignment routes allocate:
+/// `(m + 1) * (n + 1)` `usize` cells, about 1 000 rows a side. A bigger
+/// pair declines to the positional and whole-table routes — the quadratic
+/// matrix is a resource-exhaustion surface on untrusted pairs, as
+/// `LCS_CELL_CAP` is for the word resolvers in `cross_para`.
+const TABLE_ROW_CELL_CAP: usize = 1_000_000;
+
 fn as_group(u: &ComparisonUnit) -> Option<&ComparisonUnitGroup> {
     match u {
         ComparisonUnit::Group(g) => Some(g),
@@ -252,6 +259,11 @@
     if rows1.is_empty() || rows2.is_empty() {
         return None;
     }
+    if rows1.len().saturating_mul(rows2.len()) > TABLE_ROW_CELL_CAP {
+        // Too many DP cells for the shared-word matrix; the caller keeps
+        // the positional and whole-table routes for a pair this size.
+        return None;
+    }
     if table_has_merges(dom, rows1) || table_has_merges(dom, rows2) {
         return None;
     }
@@ -663,7 +675,11 @@
         // a few changed rows" case: do not collapse positionally when at least two
         // rows differ but most still match, fall through to row-level LCS.
         let use_content_lcs = hash_diff > 1 && hash_diff < total && hash_diff > total / 3;
-        if can_collapse && use_content_lcs && total >= 7 {
+        if can_collapse
+            && use_content_lcs
+            && total >= 7
+            && total.saturating_mul(total) <= TABLE_ROW_CELL_CAP
+        {
             let r = apply_lcs_to_table_rows(rows1, rows2);
             if !r.is_empty() {
                 return Some(r);
@@ -1873,6 +1889,65 @@
             }
         }
     }
+
+    #[test]
+    fn oversized_row_pairs_decline_the_shared_word_matrix() {
+        // 1001 × 1001 rows is past TABLE_ROW_CELL_CAP: the shared-word
+        // route must decline (None) instead of allocating the
+        // (m+1)×(n+1) DP matrix the alignment would fill.
+        let count = 1001;
+        let mut dom = Dom::new();
+        let left: Vec<String> = (0..count).map(|i| format!("old row {i}")).collect();
+        let right: Vec<String> = (0..count).map(|i| format!("new row {i}")).collect();
+        let left_refs: Vec<&str> = left.iter().map(String::as_str).collect();
+        let right_refs: Vec<&str> = right.iter().map(String::as_str).collect();
+        let a = source(&mut dom, &left_refs, None);
+        let b = source(&mut dom, &right_refs, None);
+        let rows_a = &as_group(&a).unwrap().contents;
+        let rows_b = &as_group(&b).unwrap().contents;
+        assert!(
+            rows_a.len().saturating_mul(rows_b.len()) > TABLE_ROW_CELL_CAP,
+            "the fixture must exceed the cap"
+        );
+        // "row" and the row numbers are shared tokens, so the route has
+        // signal and would run without the cap; over it the pair declines
+        // to the positional and whole-table routes.
+        assert!(shared_word_row_sequences(&dom, rows_a, rows_b).is_none());
+    }
+
+    #[test]
+    fn oversized_same_count_tables_stay_positional_without_the_matrix() {
+        // Same row count, one identical row and 1000 changed ones: the
+        // content-LCS heuristic applies, but the pair is past
+        // TABLE_ROW_CELL_CAP, so the rows pair positionally instead of
+        // allocating the quadratic LCS matrix.
+        let count = 1001;
+        let mut dom = Dom::new();
+        let mut left: Vec<String> = (1..count).map(|i| format!("old row {i}")).collect();
+        left.insert(0, "same header".to_string());
+        let mut right: Vec<String> = (1..count).map(|i| format!("new row {i}")).collect();
+        right.insert(0, "same header".to_string());
+        let left_refs: Vec<&str> = left.iter().map(String::as_str).collect();
+        let right_refs: Vec<&str> = right.iter().map(String::as_str).collect();
+        let a = source(&mut dom, &left_refs, None);
+        let b = source(&mut dom, &right_refs, None);
+        let out = do_lcs_algorithm_for_table(
+            &dom,
+            std::slice::from_ref(&a),
+            std::slice::from_ref(&b),
+            &WmlComparerSettings::powertools_faithful(),
+        )
+        .unwrap();
+        assert_eq!(out.len(), count);
+        assert!(
+            out.iter()
+                .all(|sequence| sequence.correlation_status == CorrelationStatus::Unknown),
+            "over-cap rows pair positionally: {:?}",
+            out.iter()
+                .map(|sequence| sequence.correlation_status)
+                .collect::<Vec<_>>()
+        );
+    }
 }
 
 #[cfg(test)]
diff --git a/src/scrub.rs b/src/scrub.rs
--- a/src/scrub.rs
+++ b/src/scrub.rs
@@ -465,6 +465,14 @@
             break;
         }
         let xml = name.ends_with(".xml") || name.ends_with(".rels");
+        if xml && let Err(error) = crate::admission::check_xml_depth(&name, &bytes, limits) {
+            // The DOM scan below parses permissively, and the byte budgets
+            // above say nothing about nesting: a part too deep (or too
+            // malformed) to scan structurally cannot certify a redaction,
+            // so refuse it by name, as the budget refusals do.
+            found.insert(format!("{name} ({error})"));
+            break;
+        }
         let hit = if xml {
             xml_holds(&String::from_utf8_lossy(&bytes), needle)
         } else {
@@ -802,5 +810,25 @@
             leaks_within(&bytes, "aaaa", fixture::limits()),
             ["word/media/a.bin"]
         );
+    }
+    #[test]
+    fn deeply_nested_xml_part_is_a_refusal_not_a_dom_parse() {
+        // 300 nested elements fit every byte budget, yet the permissive DOM
+        // parser recurses per element; the scan must refuse the part by
+        // name instead of parsing it.
+        use std::io::Write as _;
+        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
+        let options = zip::write::SimpleFileOptions::default()
+            .compression_method(zip::CompressionMethod::Deflated);
+        writer.start_file("[Content_Types].xml", options).unwrap();
+        writer.write_all(b"<Types/>").unwrap();
+        let deep = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
+        writer.start_file("word/deep.xml", options).unwrap();
+        writer.write_all(deep.as_bytes()).unwrap();
+        let bytes = writer.finish().unwrap().into_inner();
+        let findings = leaks_within(&bytes, "absent needle", InputLimits::compare());
+        let text = findings.join(", ");
+        assert!(text.contains("word/deep.xml"), "{text}");
+        assert!(text.contains("nests deeper than 256"), "{text}");
     }
 }
diff --git a/src/admission.rs b/src/admission.rs
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -785,7 +785,11 @@
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
diff --git a/tests/convert_docx_to_pdf.rs b/tests/convert_docx_to_pdf.rs
--- a/tests/convert_docx_to_pdf.rs
+++ b/tests/convert_docx_to_pdf.rs
@@ -49222,3 +49222,81 @@
         );
     }
 }
+
+/// A package whose body is one `w:altChunk` holding `html`, the chunk part
+/// reachable through the document's relationship (`chunk.html` is not an
+/// XML part, so package admission never scans it).
+fn altchunk_docx(html: &str) -> Vec<u8> {
+    let document = format!(
+        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
+         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"\
+          xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
+         <w:body><w:altChunk r:id=\"rIdChunk\"/><w:sectPr/></w:body></w:document>"
+    );
+    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
+        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
+        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
+        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
+        <Default Extension=\"html\" ContentType=\"text/html\"/>\
+        <Override PartName=\"/word/document.xml\" \
+          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
+        </Types>";
+    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
+        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
+        <Relationship Id=\"rId1\" \
+          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
+          Target=\"word/document.xml\"/>\
+        </Relationships>";
+    let doc_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
+        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
+        <Relationship Id=\"rIdChunk\" \
+          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/aFChunk\" \
+          Target=\"chunk.html\"/>\
+        </Relationships>";
+    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
+    let opts = SimpleFileOptions::default();
+    zip.start_file("[Content_Types].xml", opts).unwrap();
+    zip.write_all(content_types.as_bytes()).unwrap();
+    zip.start_file("_rels/.rels", opts).unwrap();
+    zip.write_all(rels.as_bytes()).unwrap();
+    zip.start_file("word/document.xml", opts).unwrap();
+    zip.write_all(document.as_bytes()).unwrap();
+    zip.start_file("word/_rels/document.xml.rels", opts).unwrap();
+    zip.write_all(doc_rels.as_bytes()).unwrap();
+    zip.start_file("word/chunk.html", opts).unwrap();
+    zip.write_all(html.as_bytes()).unwrap();
+    zip.finish().unwrap().into_inner()
+}
+
+/// `levels` single-cell HTML tables nested inside each other.
+fn nested_tables_html(levels: usize) -> String {
+    let mut html = String::new();
+    for _ in 0..levels {
+        html.push_str("<table><tr><td>");
+    }
+    html.push_str("leaf");
+    for _ in 0..levels {
+        html.push_str("</td></tr></table>");
+    }
+    html
+}
+
+#[test]
+fn conversion_refuses_altchunk_expansion_past_the_xml_depth_limit() {
+    // Every nested HTML table level emits w:tbl > w:tr > w:tc, so 90
+    // levels expand past the 256-element depth admission enforces on the
+    // package's own XML parts — but the expansion runs after admission,
+    // and the expanded body used to reach the recursive DOM parser
+    // unchecked.
+    let err = docx_to_pdf(&altchunk_docx(&nested_tables_html(90)))
+        .expect_err("over-deep expansion must be refused, not parsed");
+    assert!(format!("{err}").contains("nests deeper than 256"), "{err}");
+}
+
+#[test]
+fn bounded_altchunk_table_nesting_still_converts() {
+    // 60 levels (~186 nested elements) stay inside the depth budget and
+    // must keep rendering as ordinary nested tables.
+    let pdf = docx_to_pdf(&altchunk_docx(&nested_tables_html(60))).expect("bounded nesting");
+    assert!(pdf_page_count(&pdf) >= 1);
+}
```
