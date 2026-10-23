@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_8dcaf50784fd95a7f0c2ec28 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_0e2b085a251b34412cd71a4c | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_d46d1a3ef7bb52454cbd3c68 | Large Chicago numbering values can exhaust memory while reading or rendering a DOCX | medium
csf_64b247931cd1f053217fde7a | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_d22452ac5ec901578d340334 | Opaque DTD and top-level CDATA bypass package XML depth checks | medium
csf_49256aa9e91ee0159c917b13 | Rendering an admitted DOCX can retain gigabytes of duplicate embedded fonts | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.

## TDD red/green plan
# dedup-0006 remediation plan (TDD red/green)

Scope: the six retained findings in
`.scan/artifacts/deep_discovery/dedup/dedup-0006/output/result.json` that carry
this round's own `dedup0006-*` evidence refs (found at array indices 1, 3, 4,
5, 15, 22). The remaining 24 findings inherit earlier rounds' evidence and are
owned by their folders. The reducer prompt (`prompt.md`) names the single
claimed worker `b18fd97a-ff35-44ef-8b35-40b208e00e7c`; its source findings are
absorbed by the six below.

Patch: `../patches/dedup-0006.patch` (one unified diff, applies with
`git apply` from the repository root, new tests included).

All six are availability findings (CWE-674 / CWE-407 / CWE-789 / CWE-400):
an untrusted document drives disproportionate memory, CPU or stack through a
public parser/converter boundary. None touches Word-faithful output of valid
documents; every fix leaves valid input byte-identical and only bounds
adversarial magnitudes.

---

## V1 — "Processing renamed auxiliary XML bypasses XML depth limits" (index 1)

**Root cause.** `src/admission.rs:533-548` applies the XML depth/tag scan
(`check_xml_depth`) only to parts whose name ends `.xml`/`.rels` (plus the
renamed main part at `:556-569`). Relationship-selected consumers read their
part whatever its name and hand it straight to the recursive permissive
parser (`Dom::parse_xdocument`, recursion at `src/xmllinq/parse.rs:660`):
theme (`src/convert/mod.rs:3305`), styles (`:3480`), numbering (`:6559`),
comments/commentsExtended/footnotes/endnotes (`part_xml_by_rel_kind`,
`:8473`), headers/footers (`load_chrome_part`, `:20771`), fontTable
(`src/convert/font_table.rs:70`), comparer notes (`src/document_comparer.rs`
`:6323`, `:6677`, `:4740`).

**Fix (minimal).** One centralized checked read on the OPC layer, reusing the
admission scan unchanged:

- `src/admission.rs` — `check_xml_depth` becomes `pub(crate)`.
- `src/opc/mod.rs` — `PartFs` stores the `InputLimits` it was opened under
  (`read_package` already receives them) and gains
  `part_string_checked(name) -> Option<String>`: `part_string` plus the same
  `check_xml_depth` policy; over-budget XML reads as **absent**, exactly as a
  missing part, which is the failure mode every listed consumer already has.
- Switch the twelve consumer reads listed above to `part_string_checked`.

Treat-as-absent (rather than a typed `INVALID_XML` refusal) is a deliberate
minimal patch decision: a typed refusal would need new error plumbing through
ten+ renderer/comparer call sites, while absence preserves every
valid-document behavior. For `.xml`-named parts nothing changes at all —
admission already refused those packages before the renderer ran, so the new
scan can only ever filter parts that were never measured.

**Failing tests (red pre-patch, green post-patch).**

- `src/opc/mod.rs`, `mod tests` ::
  `part_string_checked_gates_renamed_parts_like_the_admission_suffix_scan` —
  pins the boundary contract: `part_string("word/styles.bin")` is `Some` for a
  257-deep part while `part_string_checked` is `None`; depth 256 (the
  `InputLimits::compare` budget) passes; a shallow arbitrary-name part
  (`word/plain.bin`) and an absent part behave as before. Pre-patch this is
  red as a missing-API compile error; the two runtime-red tests below
  demonstrate the vulnerability itself through existing API.
- `src/convert/font_table.rs`, `mod tests` ::
  `a_font_table_part_past_the_depth_budget_is_absent` — a fontTable
  relationship to `meta/faces.bin` whose part is a real `<w:fonts>` root
  (WML namespace) carrying one `<w:font w:name="Deep"/>` entry above a
  300-deep `<w:nest>` filler: `load_font_table` yields an empty table.
  Pre-patch the part parses and the table holds `Deep` (red).
  Second-review correction (round 2): the first fixture nested bare,
  namespace-less, attribute-less `<w:font>` 300 deep — unbound `w:` names
  never match `W::name("font")` (`resolve` maps them to no-namespace
  `XName`s) and no `w:name` attribute exists, so `parse_font_table_xml`
  returned an empty table before the patch too; the assertion passed both
  sides and pinned nothing. The namespaced entry plus filler nest makes it
  genuinely red.
- `tests/input_admission.rs` ::
  `revision_listing_skips_an_over_deep_renamed_notes_part` — a footnotes
  relationship to `word/notes.bin` nesting 100 000 deep through the public
  `get_revisions`: the notes part is skipped and the (revision-free) document
  lists zero revisions. Pre-patch `parse_xdocument` recurses 100 000 frames
  and the test process dies (red).

**Existing behaviour that must keep passing.**

- `src/convert/font_table.rs` ::
  `load_font_table_follows_the_font_table_relationship` (a *shallow* renamed
  fontTable part still loads — arbitrary valid part names remain supported).
- `tests/inspect_paragraphs.rs` :: `summary_reads_only_wordprocessingml_parts_for_revisions`
  and the whole of `tests/input_admission.rs` (admission decisions unchanged).
- Renderer/comparer corpus tests (`tests/m_accept_word_corpus.rs`, the
  `m*.rs` comparer family): real documents are far below depth 256, so the
  checked read returns exactly what `part_string` returned.

## V2 — "Reading or converting nested legacy DOC fields can monopolize the processing thread" (index 3)

**Root cause.** `Story::blocks` (`src/legacy_doc.rs:1591-1619`) recomputes
`fields.iter().any(|&code| code)` for **every** story character — O(depth)
per character, O(n²) for a `.doc` that stacks begin/separator field markers
(reachable from `legacy_doc::read` at `:179` and the CLI convert path
`src/bin/jubarte.rs:3127-3133`).

**Fix (minimal).** Keep an O(1) `open_code` counter beside the `Vec<bool>`
stack: begin (`\u{13}`) +1, separator (`\u{14}`) clears the top frame and
decrements only when it was still code, end (`\u{15}`) decrements when the
popped frame was code; `in_code = open_code > 0`. Semantics are unchanged
("any code ancestor suppresses text", repeated separators and unmatched ends
are no-ops); the work becomes linear in characters, which the compound-file
piece table already bounds. Saturating ops because this module denies
`clippy::arithmetic_side_effects` outside tests. No explicit depth budget is
added: with the scan linear, the remaining work is bounded by input size.

**Failing test (red pre-patch).** `src/legacy_doc.rs`, `mod tests` ::
`deep_stacks_of_field_markers_stay_linear_work` — 200 000 begin/separator
pairs, a visible `V`, 200 000 ends: asserts one paragraph with text `V`.
Pre-patch that input costs ~4×10¹⁰ boolean visits (the test does not finish);
post-patch it is a linear pass. Two companion tests pin the preserved
semantics (they pass before and after, guarding against regressions):
`field_code_hides_until_each_separator_even_nested` (nested result text stays
hidden until its own separator; outer result visible) and
`repeated_separators_and_unmatched_ends_change_nothing`.

**Existing behaviour that must keep passing.** `tests/legacy_doc.rs` and
`tests/legacy_doc_convert.rs` (whole-file legacy conversion fixtures), and the
inline `blocks`/table tests in `src/legacy_doc.rs:1723+`.

## V3 — "Large Chicago numbering values can exhaust memory while reading or rendering a DOCX" (index 4)

**Root cause.** `chicago_label` (`src/convert/mod.rs:6015`) does
`MARKS[i % 4].repeat(i / 4 + 1)` with the full u32 (`start`, `startOverride`,
section `pgNumType/@start` all reach it via `format_num`), so
`start=4294967295` asks for ~3 GiB before any later bound. The alphabetic and
Roman siblings cycle at explicit maxima (`ALPHA_LABEL_MAX`, `ROMAN_LABEL_MAX`,
`:6491-6515`); Chicago had none.

**Fix (minimal).** `CHICAGO_LABEL_MAX = 4 * 30` and
`let i = (n.max(1) - 1) % CHICAGO_LABEL_MAX;` — the label sequence starts
again at `*` past thirty mark-repetitions, mirroring the documented
alphabetic/Roman cycle pattern. Word never writes labels near that many
marks, so no observable valid-document output changes; labels for n ≤ 120
are bit-identical to before.

**Failing test (red pre-patch).** `src/convert/mod.rs`,
`mod page_num_fmt_labels` :: `chicago_starts_again_past_thirty_repetitions` —
`chicago_label(120) == "§"×30`, `chicago_label(121) == "*"`, and
`chicago_label(u32::MAX)` has ≤ 30 chars and equals
`chicago_label((u32::MAX - 1) % CHICAGO_LABEL_MAX + 1)` (pre-patch: a ~1 GiB
allocation and a wrong label at 121 — red on both counts). Second-review
correction: the cycle identity first written was
`u32::MAX % CHICAGO_LABEL_MAX + 1` (= 16 → `"§"×4`), which does **not** equal
`chicago_label(u32::MAX)` (`(u32::MAX-1) % 120 = 14` → `"‡"×4`) — the
assertion failed on correct code and is fixed to the `(n−1) % MAX + 1` form
`alpha_label`/`roman_label` already use.

**Existing behaviour that must keep passing.** `chicago_cycles_star_dagger_double_section`
(1/2/5) and `zero_labels_remain_explicit_in_field_and_east_asian_number_formats`
(`chicago_label(0) == "*"`) in the same module, plus every list/page-number
corpus test — small values take the identical path.

## V4 — "Rendering an HTML chunk can allocate gigabytes from a single colspan value" (index 5)

**Root cause.** `src/convert/altchunk.rs:562-567` parses `colspan` as an
unclamped `usize` and `end_table` (`:469-476`) sums row spans with
`sum::<usize>()` then writes one `w:gridCol` per claimed column — a single
`colspan="1000000000"` in a tiny admitted altChunk generates ~10⁹ elements
before DOM/layout checks (entry: `with_layout` → `altchunk::expand`,
`src/convert/mod.rs:789-798`).

**Fix (minimal).** `MAX_CELL_SPAN = 1024` (per-cell clamp at parse) and
`MAX_TABLE_COLUMNS = 4096` (per-row aggregate cap, computed with
`saturating_add` so the sum can neither overflow nor multiply past the
grid). Ordinary tables — Word's own are ≤ 63 columns — are untouched; the
`w:gridSpan` written for a cell still mirrors its (clamped) span, so the
generated WML stays well-formed. Clamping rather than a typed refusal keeps
`html_to_wml`/`expand`'s `String` API (and the whole altChunk pipeline)
unchanged.

**Failing test (red pre-patch).** `src/convert/altchunk.rs`,
`mod residual_import_contract_tests` ::
`oversized_colspan_claims_are_clamped_before_grid_generation` — two cells
claiming 10⁹ and 2³²−1 columns produce ≤ 100 KB of WML with exactly
`2 × MAX_CELL_SPAN` grid columns and a clamped `w:gridSpan w:val="1024"`;
eight `colspan="1000"` cells cap the row at `MAX_TABLE_COLUMNS`; a
`colspan="3"` cell still gets exactly 3 grid columns and
`w:gridSpan w:val="3"`. Pre-patch the first assertion alone would need
~5×10⁹ appends (the test does not finish).

**Existing behaviour that must keep passing.** `memory_story_chunk…`,
`ragged_authored_rows_share_one_grid…` and the other altChunk contract tests
(ordinary HTML tables render identically), plus `tests/docx_to_markdown_*`
altChunk round-trips.

## V5 — "Opaque DTD and top-level CDATA bypass package XML depth checks" (index 15)

**Root cause.** Admission's scanner (quick-xml) consumes a whole DOCTYPE —
quoted entity literals and internal-subset comments included — as one opaque
`DocType` event, so the package admits at depth zero. The permissive DOM's
top-level skip (`src/xmllinq/parse.rs:539-544`) ends at the **first `>`**,
which can sit inside the quoted literal; the remainder of the literal then
parses as really-nested elements and `parse_element` recurses as deep as the
literal nests (`:660`). The same mismatch exists for top-level `<![CDATA[…]]>`
(no top-level CDATA branch at all). Reachable through rendering and
`WmlDocument::main_document` (`src/wml_document.rs:85-96`).

**Fix (minimal).** Make the DOM skip consume the *complete* construct, so
both representations agree lexically:

- top-level `<![CDATA[` skips to `]]>`;
- top-level `<!…` (DOCTYPE) skips past quotes, `<!-- … -->` comments and an
  internal subset `[…]`, ending at the `>` that closes the declaration.

No admission-side DTD rejection: `tests/inspect_paragraphs.rs:231`
(`summary_reads_only_wordprocessingml_parts_for_revisions`) requires a
DTD-bearing `customXml` item to still admit (the *strict* reader refuses it
per part; the package must not fail). Repairing the DOM's lexical boundary
closes the stack-overflow path while keeping that tested tolerance.

**Failing test (red pre-patch).** `src/xmllinq/parse.rs`,
`mod xml_lexical_boundary_contract_tests` ::
`opaque_declarations_and_top_level_cdata_skip_their_whole_construct` — three
100 000-deep proofs (quoted `<!ENTITY>` literal, internal-subset comment
containing `>`, top-level CDATA) each followed by a real `<w:document/>`:
the DOM must hold exactly one top-level child, the real root. Pre-patch the
hidden literal becomes 100 000 nested elements (process dies). Also asserts
in-root CDATA and a plain `<!DOCTYPE d>` keep their meaning.

**Integration test (red pre-patch).** `tests/input_admission.rs` ::
`wml_document_parses_a_dtd_hidden_literal_without_recursing_into_it` — a full
package whose `word/document.xml` carries the DTD-hidden literal admits
(quick-xml treats the DTD as opaque) and `WmlDocument::main_document_root`
returns the real `w:document` root. Pre-patch the lazy parse overflows.

**Existing behaviour that must keep passing.** All of
`mod checked_xml_tests` and `mod xml_lexical_boundary_contract_tests` in
`src/xmllinq/parse.rs` (including `"<a x='1' x='2'/>"`/`"<!DOCTYPE a><a/>"`
validate rejections), `tests/inspect_paragraphs.rs:231`, and the M1.8
round-trip gates (real OOXML carries no top-level DTD/CDATA, so the skip
paths are never taken for them).

## V6 — "Rendering an admitted DOCX can retain gigabytes of duplicate embedded fonts" (index 22)

**Root cause.** `load_embedded_fonts` (`src/convert/font_table.rs:205-231`)
deobfuscates (`bytes.to_vec()`), validates and retains a fresh `Arc` for
**every** font-table alias/slot, so thousands of distinct `w:font` names
sharing one embedded part retain thousands of full copies (~3.2 GB for
10 000 aliases of a 320 KB font), even though `with_layout`
(`src/convert/mod.rs:806-807`) never uses those faces.

**Fix (minimal).** A per-call `HashMap<(part path, fontKey),
Option<Arc<[u8]>>>` decode cache: the first alias resolving a part/key decodes
and validates once (an invalid font caches `None`, preserving today's skip);
every other alias sharing that part/key gets a clone of the same `Arc`.
Retained bytes are then bounded by the *distinct* parts, which admission's
inflated-byte budget already bounds — no separate aggregate budget needed.
Map contents (keys, validity decisions) are identical to before.

**Failing test (red pre-patch).** `src/convert/font_table.rs`, `mod tests` ::
`aliased_embedded_fonts_share_one_decoded_copy` — 16 rare-family aliases all
embedding `rId1` (the repository's `tests/fixtures/fonts/STIXTwoMath.otf`):
all 16 slots decode and `Arc::ptr_eq` proves they share one buffer.
Pre-patch 16 distinct buffers exist (red). `package_bytes` (a bytes-capable
sibling of the module's existing `package` helper) builds the in-memory zip.

**Existing behaviour that must keep passing.** `font_table_parses_altname_family_and_sig`,
`font_table_lookup_is_case_insensitive`, `load_font_table_missing_part_is_empty`,
`load_font_table_follows_the_font_table_relationship`,
`recognized_plain_sfnt_signatures_never_apply_document_obfuscation`, and the
embedded-font renderer corpus (`tests/convert_font_report.rs`, Word-variant
pitch tests in `src/convert/font.rs`).

---

## Second-review corrections (round 2)

- `a_font_table_part_past_the_depth_budget_is_absent` (V1): fixture
  corrected as described above — the original could not fail pre-patch.
- `wml_document_parses_a_dtd_hidden_literal_without_recursing_into_it`
  (V5): the DOCTYPE name in the fixture read `w::document` (a typo); both
  readers treat the declaration opaquely, so behavior was identical, but
  the fixture now reads `w:document` as a real Word file would.
- Verified this round: `git apply --check` on the corrected patch; every
  hunk anchor against the current source; the twelve V1 consumer sites;
  V2's counter invariant (`open_code` == count of `true` frames through
  every begin/separator/end transition); V3's arithmetic for n ≤ 120, n=0,
  121 and `u32::MAX` (`(u32::MAX-1) % 120 = 14` → `‡`×4, matching
  `chicago_label(15)`); V4's grid arithmetic (`9360/cols` with
  cols ∈ [1, 4096], `col_w × span ≤ 9360 × 1024`); V5's scanner on the
  three proof shapes (quoted literal, subset comment containing `>`,
  top-level CDATA), the plain `<!DOCTYPE d>` case, and the
  `from_bytes` → quick-xml-opaque-DTD → lazy-DOM path; V6's cache typing
  and the STIX fixture path. Still not run (forbidden in this checkout):
  cargo/mbx/clippy/the tests.

## Verification performed in this session

- `git apply --check /tmp/dedup0006/dedup-0006.patch` run from
  `/Users/arthrod/temp/T/jubarte-redlines` — **passed** (exit 0, all nine
  files check clean). The published copy at
  `.scan/remediation/patches/dedup-0006.patch` is byte-identical.
- Standalone `rustfmt --edition 2024 --check` (stable toolchain binary, run
  on scratch copies under `/tmp`, never in the checkout) parsed **every**
  edited file with zero diffs after formatting — i.e. all new code is
  syntactically valid and rustfmt-clean per the repo's `rustfmt.toml`.
- **Not run** (forbidden by the ask: no cargo/mbx/rustc in this checkout):
  `cargo nextest`, clippy, and the tests themselves. The red/green claims
  above are grounded in the cited source paths, not in executed tests.
  Suggested maintainer commands, per AGENTS.md, after applying:
  - `cargo nextest run -E 'test(part_string_checked) + test(a_font_table_part_past) + test(revision_listing_skips) + test(wml_document_parses_a_dtd) + test(opaque_declarations) + test(oversized_colspan) + test(chicago_starts_again) + test(field_code_hides) + test(repeated_separators) + test(deep_stacks_of_field) + test(aliased_embedded)'`
  - `cargo nextest run --test input_admission`, `cargo nextest run -E 'test(legacy)'`,
    the font/altchunk/convert inline suites, then clippy with `-D warnings`
    and a `jubarte --help` smoke test.

## Files touched (all under the two deliverable paths; no repository file was modified this session)

`src/admission.rs`, `src/opc/mod.rs`, `src/xmllinq/parse.rs`,
`src/convert/mod.rs`, `src/convert/font_table.rs`, `src/convert/altchunk.rs`,
`src/legacy_doc.rs`, `src/document_comparer.rs`, `tests/input_admission.rs`.

No `#[allow]` is introduced anywhere; the two lint-denying modules
(`legacy_doc.rs`, `opc/mod.rs`) use saturating arithmetic to stay clean.


## Patch
```diff
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
index 7def5f9..77bf8f8 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -10,6 +10,14 @@
 use std::collections::HashMap;
 use std::fmt::Write as _;
 
+/// The most columns one HTML cell may claim (`colspan`) and one table may
+/// generate. A `colspan` is one attribute against the WML it generates: a
+/// billion-column claim wrote a billion `w:gridCol` elements before any
+/// layout or output guard could see them, and a row's summed claims could
+/// overflow the sum itself. Word's own tables sit far below either bound.
+const MAX_CELL_SPAN: usize = 1024;
+const MAX_TABLE_COLUMNS: usize = 4096;
+
 /// `xml` with every `w:altChunk` replaced by its chunk's paragraphs.
 /// `load` returns a chunk's bytes by relationship id.
 pub(crate) fn expand(xml: &str, load: impl Fn(&str) -> Option<Vec<u8>>) -> String {
@@ -469,7 +477,11 @@ impl Builder {
         let cols = table
             .rows
             .iter()
-            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
+            .map(|r| {
+                r.iter()
+                    .fold(0usize, |cols, c| cols.saturating_add(c.span))
+                    .min(MAX_TABLE_COLUMNS)
+            })
             .max()
             .unwrap_or(1)
             .max(1);
@@ -563,7 +575,7 @@ impl Builder {
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
                         .unwrap_or(1)
-                        .max(1),
+                        .clamp(1, MAX_CELL_SPAN),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
@@ -1380,6 +1392,28 @@ mod residual_import_contract_tests {
         assert_eq!(source, before);
     }
 
+    #[test]
+    fn oversized_colspan_claims_are_clamped_before_grid_generation() {
+        // One attribute claiming a billion columns used to write a billion
+        // `w:gridCol` elements (tens of gigabytes of WML) before the DOM
+        // or any layout limit could refuse; the claim is clamped, and a
+        // row's summed claims cap the grid width they add up to.
+        let wml = html_to_wml(
+            r#"<table><tr><td colspan="1000000000">one</td><td colspan="4294967295">two</td></tr></table>"#,
+        );
+        assert!(wml.len() < 100_000, "{} bytes of generated WML", wml.len());
+        assert_eq!(wml.matches("<w:gridCol").count(), 2 * MAX_CELL_SPAN);
+        assert!(wml.contains(r#"<w:gridSpan w:val="1024"/>"#));
+        // A row of many cells cannot multiply past the grid bound either.
+        let cells = r#"<td colspan="1000">a</td>"#.repeat(8);
+        let wide = html_to_wml(&format!(r#"<table><tr>{cells}</tr></table>"#));
+        assert_eq!(wide.matches("<w:gridCol").count(), MAX_TABLE_COLUMNS);
+        // Ordinary small spans are untouched.
+        let ok = html_to_wml(r#"<table><tr><td colspan="3">x</td></tr></table>"#);
+        assert_eq!(ok.matches("<w:gridCol").count(), 3);
+        assert!(ok.contains(r#"<w:gridSpan w:val="3"/>"#));
+    }
+
     #[test]
     fn ragged_authored_rows_share_one_grid_without_inventing_filler_cells_or_reordering_payload() {
         let source = "<table><tr><td>A</td></tr><tr><td>B</td><td>C</td></tr></table><p>After</p>";
diff --git a/src/convert/font_table.rs b/src/convert/font_table.rs
index c1063f8..f1da4e4 100644
--- a/src/convert/font_table.rs
+++ b/src/convert/font_table.rs
@@ -68,7 +68,7 @@ fn font_table_part(pkg: &PartFs) -> String {
 }
 
 pub(crate) fn load_font_table(pkg: &PartFs) -> FontTable {
-    pkg.part_string(&font_table_part(pkg))
+    pkg.part_string_checked(&font_table_part(pkg))
         .map(|xml| parse_font_table_xml(&xml))
         .unwrap_or_default()
 }
@@ -204,6 +204,12 @@ pub(crate) fn deobfuscate_odttf(bytes: &[u8], font_key: &str) -> Vec<u8> {
 /// slot is then a metric twin under its own name, not the family.
 pub(crate) fn load_embedded_fonts(pkg: &PartFs, table: &FontTable) -> super::font::EmbeddedFonts {
     let mut out = HashMap::new();
+    // One decode per source part and font key: distinct `w:font` names may
+    // point their embed slots at one part, and a copy per alias retained
+    // every one of them (thousands of aliases of one font asked for
+    // gigabytes of identical bytes). Every alias that resolves the same
+    // part and key shares the cached `Arc`.
+    let mut decoded: HashMap<(String, String), Option<std::sync::Arc<[u8]>>> = HashMap::new();
     let part = font_table_part(pkg);
     let rels = pkg.read_rels_for(&part);
     for entry in table.iter() {
@@ -218,14 +224,18 @@ pub(crate) fn load_embedded_fonts(pkg: &PartFs, table: &FontTable) -> super::fon
                 continue;
             };
             let path = pkg.resolve_rel_target(&part, &rel.target);
-            let Some(raw) = pkg.part_bytes(&path) else {
+            let Some(data) = decoded
+                .entry((path, font_key.clone()))
+                .or_insert_with_key(|(path, font_key)| {
+                    let raw = pkg.part_bytes(path)?;
+                    let data = deobfuscate_odttf(raw, font_key);
+                    (ttf_parser::Face::parse(&data, 0).is_ok()).then(|| data.into())
+                })
+                .clone()
+            else {
                 continue;
             };
-            let data = deobfuscate_odttf(raw, font_key);
-            if ttf_parser::Face::parse(&data, 0).is_err() {
-                continue;
-            }
-            out.insert((entry.name.to_ascii_lowercase(), bold, italic), data.into());
+            out.insert((entry.name.to_ascii_lowercase(), bold, italic), data);
         }
     }
     out
@@ -278,6 +288,93 @@ mod tests {
         assert!(table.get("somerare").is_some());
     }
 
+    #[test]
+    fn a_font_table_part_past_the_depth_budget_is_absent() {
+        // The fontTable relationship selects `meta/faces.bin`; admission's
+        // depth scan keys on the `.xml`/`.rels` suffix, so the renderer
+        // must read the part through the checked boundary or the renamed
+        // XML reaches the permissive parser ungated. The part carries one
+        // real font entry (`Deep`) above a 300-deep filler nest: without
+        // the gate the entry parses and the table is non-empty.
+        let deep = format!(
+            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{}<w:font w:name="Deep"/>{}</w:fonts>"#,
+            "<w:nest>".repeat(300),
+            "</w:nest>".repeat(300)
+        );
+        let pkg = package(&[
+            ("[Content_Types].xml", CT),
+            ("_rels/.rels", ROOT_RELS),
+            ("word/document.xml", DOC),
+            (
+                "word/_rels/document.xml.rels",
+                r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdF" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" Target="meta/faces.bin"/></Relationships>"#,
+            ),
+            ("word/meta/faces.bin", deep.as_str()),
+        ]);
+        assert_eq!(load_font_table(&pkg).iter().count(), 0);
+    }
+
+    fn package_bytes(parts: &[(&str, &[u8])]) -> PartFs {
+        use std::io::{Cursor, Write};
+        let mut buf = Vec::new();
+        {
+            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
+            let opt = zip::write::SimpleFileOptions::default();
+            for (name, bytes) in parts {
+                z.start_file(*name, opt).unwrap();
+                z.write_all(bytes).unwrap();
+            }
+            z.finish().unwrap();
+        }
+        PartFs::open(&buf).expect("package")
+    }
+
+    const FONT: &[u8] = include_bytes!("../../tests/fixtures/fonts/STIXTwoMath.otf");
+
+    #[test]
+    fn aliased_embedded_fonts_share_one_decoded_copy() {
+        // Distinct `w:font` rows may embed the same part under one r:id;
+        // each alias used to copy, validate and retain its own full font
+        // (thousands of aliases of one embedded font asked for gigabytes).
+        let aliases: String = (0..16)
+            .map(|i| {
+                format!(
+                    r#"<w:font w:name="Rare{i}"><w:embedRegular r:id="rId1" w:fontKey="{{00000000-0000-0000-0000-000000000001}}"/></w:font>"#
+                )
+            })
+            .collect();
+        let pkg = package_bytes(&[
+            ("[Content_Types].xml", CT.as_bytes()),
+            ("_rels/.rels", ROOT_RELS.as_bytes()),
+            ("word/document.xml", DOC.as_bytes()),
+            (
+                "word/fontTable.xml",
+                format!(
+                    r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">{aliases}</w:fonts>"#
+                )
+                .as_bytes(),
+            ),
+            (
+                "word/_rels/fontTable.xml.rels",
+                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/font" Target="fonts/f1.otf"/></Relationships>"#,
+            ),
+            ("word/fonts/f1.otf", FONT),
+        ]);
+        let table = load_font_table(&pkg);
+        let embedded = load_embedded_fonts(&pkg, &table);
+        assert_eq!(embedded.len(), 16);
+        let first = embedded
+            .get(&("rare0".to_string(), false, false))
+            .expect("rare0 decoded");
+        let last = embedded
+            .get(&("rare15".to_string(), false, false))
+            .expect("rare15 decoded");
+        assert!(
+            std::sync::Arc::ptr_eq(first, last),
+            "aliases of one part share the decoded font"
+        );
+    }
+
     #[test]
     fn font_table_parses_every_embedded_face_slot() {
         let table = parse_font_table_xml(
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
index d900e05..4299c4c 100644
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -3304,8 +3304,8 @@ fn parse_len(s: &str) -> Option<f32> {
 
 fn load_theme(pkg: &PartFs) -> ThemeFonts {
     let Some(xml) = pkg
-        .part_string(&main_rel_part(pkg, "theme", "word/theme/theme1.xml"))
-        .or_else(|| pkg.part_string("word/theme/theme2.xml"))
+        .part_string_checked(&main_rel_part(pkg, "theme", "word/theme/theme1.xml"))
+        .or_else(|| pkg.part_string_checked("word/theme/theme2.xml"))
     else {
         return ThemeFonts::default();
     };
@@ -3477,7 +3477,8 @@ fn load_stylesheet(pkg: &PartFs) -> StyleSheet {
     defaults.wp_justify = settings_flag(pkg, "wpJustification");
     defaults.para.sum_spacing = settings_flag(pkg, "doNotUseHTMLParagraphAutoSpacing");
     let mut raw: std::collections::HashMap<String, RawStyle> = std::collections::HashMap::new();
-    let Some(xml) = pkg.part_string(&main_rel_part(pkg, "styles", "word/styles.xml")) else {
+    let Some(xml) = pkg.part_string_checked(&main_rel_part(pkg, "styles", "word/styles.xml"))
+    else {
         return StyleSheet {
             defaults,
             by_id: HashMap::new(),
@@ -6012,10 +6013,20 @@ fn ordinal_text_ones(one: &str) -> String {
     }
 }
 
+/// The last Chicago label written before the marks start again at `*`:
+/// thirty repetitions, mirroring the alphabetic and Roman cycles Word's
+/// probes landed on ([`ALPHA_LABEL_MAX`], [`ROMAN_LABEL_MAX`]). Word never
+/// writes a label near this many marks, and a list start or page counter
+/// near `u32::MAX` used to repeat the mark a billion times before any
+/// later bound could apply, so past it the sequence starts again.
+const CHICAGO_LABEL_MAX: u32 = 4 * 30;
+
+/// `chicago`: the footnote sequence `*`, `†`, `‡`, `§`, then each mark
+/// repeated (`**`, `††`, …).
 fn chicago_label(n: u32) -> String {
     const MARKS: [&str; 4] = ["*", "†", "‡", "§"];
-    let i = n.saturating_sub(1) as usize;
-    MARKS[i % 4].repeat(i / 4 + 1)
+    let i = (n.max(1) - 1) % CHICAGO_LABEL_MAX;
+    MARKS[(i % 4) as usize].repeat((i / 4) as usize + 1)
 }
 
 fn ideograph_digital_label(n: u32) -> String {
@@ -6556,7 +6567,7 @@ fn bullet_glyph(raw: &str) -> String {
 
 fn load_numbering(pkg: &PartFs) -> Numbering {
     let numbering_part = main_rel_part(pkg, "numbering", "word/numbering.xml");
-    let Some(xml) = pkg.part_string(&numbering_part) else {
+    let Some(xml) = pkg.part_string_checked(&numbering_part) else {
         return Numbering::default();
     };
     parse_numbering_xml(&xml, |rid| resolve_media(pkg, &numbering_part, rid))
@@ -8480,12 +8491,12 @@ fn part_xml_by_rel_kind(pkg: &PartFs, main: &str, kind: &str) -> Option<String>
             .filter(|i| i.rel_type.ends_with(kind) || i.target.contains(kind));
         for item in by_type.chain(loose) {
             let path = pkg.resolve_rel_target(main, &item.target);
-            if let Some(xml) = pkg.part_string(&path) {
+            if let Some(xml) = pkg.part_string_checked(&path) {
                 return Some(xml);
             }
         }
     }
-    pkg.part_string(&format!("word/{kind}.xml"))
+    pkg.part_string_checked(&format!("word/{kind}.xml"))
 }
 
 /// Separator / continuation notes are chrome, not note text. Since plan
@@ -20768,10 +20779,9 @@ fn load_chrome_part(
     let Some(path) = rel_target_path(pkg, main, rid) else {
         return empty_chrome();
     };
-    let Some(bytes) = pkg.part_bytes(&path) else {
+    let Some(mut xml) = pkg.part_string_checked(&path) else {
         return empty_chrome();
     };
-    let mut xml = String::from_utf8_lossy(bytes).into_owned();
     // The page background's anchor paragraph: af0035cc's one-paragraph
     // header (Normal, after=10 at 1.15) runs two lines deep in Word and
     // pushes the body 14.8pt below its 72pt margin.
@@ -36377,9 +36387,10 @@ fn body_op_yrange(ops: &[Op]) -> Option<(f32, f32)> {
 #[cfg_attr(all(coverage_nightly, test), coverage(off))]
 mod page_num_fmt_labels {
     use super::{
-        NumFmt, chicago_label, decimal_fullwidth_label, format_num, ideograph_digital_label,
-        ideograph_enclosed_circle_label, ideograph_legal_traditional_label,
-        ideograph_zodiac_traditional_label, japanese_counting_label, parse_num_fmt,
+        CHICAGO_LABEL_MAX, NumFmt, chicago_label, decimal_fullwidth_label, format_num,
+        ideograph_digital_label, ideograph_enclosed_circle_label,
+        ideograph_legal_traditional_label, ideograph_zodiac_traditional_label,
+        japanese_counting_label, parse_num_fmt,
     };
 
     #[test]
@@ -36395,6 +36406,20 @@ mod page_num_fmt_labels {
         assert_eq!(chicago_label(5), "**");
     }
 
+    #[test]
+    fn chicago_starts_again_past_thirty_repetitions() {
+        // 120 is the last label (§ thirty times); 121 starts again at *,
+        // as the alphabetic and Roman labels do past their maxima. A start
+        // near u32::MAX (or a section page counter there) must not repeat
+        // the mark hundreds of millions of times before anything could
+        // bound the allocation.
+        assert_eq!(chicago_label(CHICAGO_LABEL_MAX), "§".repeat(30));
+        assert_eq!(chicago_label(CHICAGO_LABEL_MAX + 1), "*");
+        let huge = chicago_label(u32::MAX);
+        assert!(huge.chars().count() <= 30, "{} marks", huge.chars().count());
+        assert_eq!(huge, chicago_label((u32::MAX - 1) % CHICAGO_LABEL_MAX + 1));
+    }
+
     #[test]
     fn ideograph_digital_uses_cjk_digits() {
         assert_eq!(ideograph_digital_label(1), "一");
diff --git a/src/document_comparer.rs b/src/document_comparer.rs
index febe703..7385894 100644
--- a/src/document_comparer.rs
+++ b/src/document_comparer.rs
@@ -4737,8 +4737,8 @@ pub fn pre_process_markup(
     let Some(main_xml) = pkg.part_string(&main) else {
         return Ok(Vec::new());
     };
-    let fn_xml = fn_part.as_deref().and_then(|p| pkg.part_string(p));
-    let en_xml = en_part.as_deref().and_then(|p| pkg.part_string(p));
+    let fn_xml = fn_part.as_deref().and_then(|p| pkg.part_string_checked(p));
+    let en_xml = en_part.as_deref().and_then(|p| pkg.part_string_checked(p));
 
     let mut dom = Dom::new();
     let main_doc = dom.parse_xdocument(&main_xml);
@@ -6320,7 +6320,7 @@ pub fn get_revisions(
         (fn_part.as_str(), W::footnote()),
         (en_part.as_str(), W::endnote()),
     ] {
-        if let Some(x) = pkg.part_string(part) {
+        if let Some(x) = pkg.part_string_checked(part) {
             let nd = dom.parse_xdocument(&x);
             if let Some(r) = dom.root(nd) {
                 preprocess::remove_existing_powertools_markup(&mut dom, r);
@@ -6674,7 +6674,7 @@ fn compare_documents_impl(
     // definition by its reference's correlation status and rebuilds the
     // withRevisions parts renumbered 1..n.
     fn parse_part_root(dom: &mut Dom, pkg: &PartFs, name: &str) -> Option<NodeId> {
-        let xml = pkg.part_string(name)?;
+        let xml = pkg.part_string_checked(name)?;
         let d = dom.parse_xdocument(&xml);
         dom.root(d)
     }
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
index 1881053..ccfd827 100644
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -1594,6 +1594,12 @@ impl Story<'_> {
         // Field nesting: true while inside a field's code (before its
         // separator).
         let mut fields: Vec<bool> = Vec::new();
+        // How many open frames are still in their code phase. `fields`
+        // holds one bool per open field; scanning the whole stack for a
+        // code frame on every character made nested fields quadratic (a
+        // .doc can stack thousands of begin/separator pairs), so the
+        // code-phase count is kept beside the stack instead.
+        let mut open_code = 0usize;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
@@ -1601,20 +1607,26 @@ impl Story<'_> {
 
         for story in chars {
             // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
+            let in_code = open_code > 0;
             let text = match story.ch {
                 '\u{13}' => {
                     fields.push(true);
+                    open_code = open_code.saturating_add(1);
                     continue;
                 }
                 '\u{14}' => {
                     if let Some(top) = fields.last_mut() {
-                        *top = false;
+                        if *top {
+                            *top = false;
+                            open_code = open_code.saturating_sub(1);
+                        }
                     }
                     continue;
                 }
                 '\u{15}' => {
-                    fields.pop();
+                    if fields.pop() == Some(true) {
+                        open_code = open_code.saturating_sub(1);
+                    }
                     continue;
                 }
                 _ if in_code => continue,
@@ -1787,6 +1799,98 @@ mod tests {
         );
     }
 
+    fn field_char(ch: char, cp: u32) -> StoryChar {
+        StoryChar {
+            ch,
+            cp,
+            fc: 0,
+            modifier: CharModifier::default(),
+        }
+    }
+
+    fn blocks_of(chars: &[StoryChar]) -> Vec<Block> {
+        let papx = FkpIndex { runs: Vec::new() };
+        let chpx = FkpIndex { runs: Vec::new() };
+        let lists = Lists::default();
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        story.blocks(chars)
+    }
+
+    fn paragraph_text(blocks: &[Block]) -> String {
+        let Some(Block::Paragraph(p)) = blocks.first() else {
+            panic!("expected a paragraph, got {blocks:?}");
+        };
+        p.spans
+            .iter()
+            .map(|span| span.text.as_str())
+            .collect::<String>()
+    }
+
+    #[test]
+    fn field_code_hides_until_each_separator_even_nested() {
+        // Field begin (\u{13}) opens a code phase, the separator (\u{14})
+        // ends it, end (\u{15}) pops the frame. Inside any code — even a
+        // nested field's result — text is hidden until that nested
+        // separator arrives.
+        let chars: Vec<StoryChar> = [
+            '\u{13}', 'C', '\u{14}', '\u{13}', 'N', '\u{14}', 'R', '\u{15}', 'S', '\u{15}', 'T',
+            '\r',
+        ]
+        .into_iter()
+        .enumerate()
+        .map(|(cp, ch)| field_char(ch, u32::try_from(cp).unwrap()))
+        .collect();
+        let blocks = blocks_of(&chars);
+        assert_eq!(blocks.len(), 1);
+        assert_eq!(paragraph_text(&blocks), "RST");
+    }
+
+    #[test]
+    fn repeated_separators_and_unmatched_ends_change_nothing() {
+        let chars: Vec<StoryChar> = [
+            '\u{13}', '\u{14}', '\u{14}', 'A', '\u{15}', '\u{15}', 'B', '\r',
+        ]
+        .into_iter()
+        .enumerate()
+        .map(|(cp, ch)| field_char(ch, u32::try_from(cp).unwrap()))
+        .collect();
+        let blocks = blocks_of(&chars);
+        assert_eq!(blocks.len(), 1);
+        assert_eq!(paragraph_text(&blocks), "AB");
+    }
+
+    #[test]
+    fn deep_stacks_of_field_markers_stay_linear_work() {
+        // 200_000 begin/separator pairs leave every frame in its result
+        // phase, so the visible text stays visible and the field stack is
+        // walked once. Scanning the whole stack per character made this
+        // input ~4e10 boolean visits; the code-phase counter keeps the
+        // story scan linear in its characters.
+        let mut chars: Vec<StoryChar> = Vec::new();
+        let mut cp = 0_u32;
+        for _ in 0..200_000 {
+            chars.push(field_char('\u{13}', cp));
+            cp += 1;
+            chars.push(field_char('\u{14}', cp));
+            cp += 1;
+        }
+        chars.push(field_char('V', cp));
+        for _ in 0..200_000 {
+            chars.push(field_char('\u{15}', cp));
+            cp += 1;
+        }
+        chars.push(field_char('\r', cp));
+        let blocks = blocks_of(&chars);
+        assert_eq!(blocks.len(), 1);
+        assert_eq!(paragraph_text(&blocks), "V");
+    }
+
     #[test]
     fn line_breaks_become_hard_breaks() {
         let document = LegacyDocument {
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
index 9865bda..1277abf 100644
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -356,6 +356,7 @@ fn read_package(bytes: &[u8], limits: crate::admission::InputLimits) -> Result<P
         part_rels: HashMap::new(),
         source_order,
         repaired_duplicate_ids: Vec::new(),
+        limits,
     };
     package.pkg.content_types = read_content_types(
         &parts
@@ -467,6 +468,9 @@ pub struct PartFs {
     /// `(part, Id)` of each relationship the source repeated identically and
     /// [`PartFs::open`] kept once (`""` is the package's own relationships).
     repaired_duplicate_ids: Vec<(String, String)>,
+    /// The budget the package was opened under. [`PartFs::part_string_checked`]
+    /// applies its XML policy to parts the admission suffix scan never saw.
+    limits: crate::admission::InputLimits,
 }
 
 /// Canonical target resolution shared by package APIs and text story labels.
@@ -563,6 +567,21 @@ impl PartFs {
             .map(|b| String::from_utf8_lossy(b).into_owned())
     }
 
+    /// [`PartFs::part_string`] for a part a consumer will parse as XML,
+    /// returning it only when it passes the depth and tag policy admission
+    /// applies to `.xml`/`.rels` parts. A relationship-selected part keeps
+    /// whatever extension its producer chose (`styles.bin`, `notes.dat`),
+    /// so the admission scan by suffix never measured it; reading such a
+    /// part here makes over-budget XML absent — exactly as a missing part
+    /// — instead of letting it reach the permissive recursive parser
+    /// ungated.
+    pub(crate) fn part_string_checked(&self, name: &str) -> Option<String> {
+        let xml = self.part_string(name)?;
+        crate::admission::check_xml_depth(name, xml.as_bytes(), self.limits)
+            .ok()
+            .map(|()| xml)
+    }
+
     /// `PartFS.setPart(name, data)` — replace or add a part.
     ///
     /// A `…/_rels/<part>.rels` name replaces that part's relationships: the
@@ -1014,6 +1033,49 @@ mod tests {
         }
     }
 
+    #[test]
+    fn part_string_checked_gates_renamed_parts_like_the_admission_suffix_scan() {
+        // Admission measures XML depth by the `.xml`/`.rels` suffix; a part
+        // a relationship selects keeps any name. The checked read must
+        // apply the same policy to those parts, or their XML reaches the
+        // permissive parser ungated (renamed auxiliary XML).
+        let deep = |depth: usize| {
+            format!(
+                "{}{}",
+                "<w:style>".repeat(depth),
+                "</w:style>".repeat(depth)
+            )
+        };
+        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
+        for (name, data) in [
+            ("[Content_Types].xml", "<Types/>"),
+            ("word/styles.bin", deep(257).as_str()),
+            ("word/styles-at-the-budget.bin", deep(256).as_str()),
+            ("word/plain.bin", "<w:styles/>"),
+        ] {
+            writer
+                .start_file(name, SimpleFileOptions::default())
+                .unwrap();
+            writer.write_all(data.as_bytes()).unwrap();
+        }
+        let fs = PartFs::open(&writer.finish().unwrap().into_inner()).unwrap();
+        // The permissive read never applied the XML policy…
+        assert!(fs.part_string("word/styles.bin").is_some());
+        // …the checked read refuses what admission would have refused had
+        // the part been named styles.xml (256 under InputLimits::compare).
+        assert!(fs.part_string_checked("word/styles.bin").is_none());
+        assert!(
+            fs.part_string_checked("word/styles-at-the-budget.bin")
+                .is_some()
+        );
+        // An arbitrary shallow part name keeps working.
+        assert_eq!(
+            fs.part_string_checked("word/plain.bin").as_deref(),
+            Some("<w:styles/>")
+        );
+        assert!(fs.part_string_checked("word/absent.bin").is_none());
+    }
+
     fn package_with_header_rels() -> PartFs {
         let mut buf = Vec::new();
         {
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..1f1dc5a 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -536,10 +536,57 @@ impl<'a> Parser<'a> {
                 self.pos = end.map(|e| e + 3).unwrap_or(len);
                 continue;
             }
+            if self.starts_with("<![CDATA[") {
+                // CDATA outside any element: the section is opaque, as a
+                // reader's CData event is — its body may hold markup-looking
+                // text that must not become elements.
+                let end = self.index_of("]]>", self.pos);
+                self.pos = end.map(|e| e + 3).unwrap_or(len);
+                continue;
+            }
             if self.starts_with("<!") {
-                // DOCTYPE — skip
-                let end = self.index_of(">", self.pos);
-                self.pos = end.map(|e| e + 1).unwrap_or(len);
+                // DOCTYPE — skip the whole declaration, as one DocType
+                // event: quoted literals, comments and the internal subset
+                // ([…]) can each hold '>' that ending at the first '>' would
+                // cut the declaration at, exposing contents no XML gate ever
+                // measured as deeply nested markup.
+                let mut at = self.pos;
+                let mut quote = '\0';
+                let mut subset = false;
+                let mut comment = false;
+                while at < len {
+                    let ch = self.c[at];
+                    if comment {
+                        if ch == '-'
+                            && self.c.get(at + 1) == Some(&'-')
+                            && self.c.get(at + 2) == Some(&'>')
+                        {
+                            at += 3;
+                            comment = false;
+                            continue;
+                        }
+                    } else if quote != '\0' {
+                        if ch == quote {
+                            quote = '\0';
+                        }
+                    } else {
+                        match ch {
+                            '\'' | '"' => quote = ch,
+                            '-' if at > self.pos
+                                && self.c[at - 1] == '!'
+                                && self.c.get(at + 1) == Some(&'-') =>
+                            {
+                                comment = true;
+                            }
+                            '[' => subset = true,
+                            ']' => subset = false,
+                            '>' if !subset => break,
+                            _ => {}
+                        }
+                    }
+                    at += 1;
+                }
+                self.pos = if at < len { at + 1 } else { len };
                 continue;
             }
             if self.cur() == '<' {
@@ -1179,6 +1226,37 @@ mod xml_lexical_boundary_contract_tests {
         assert_eq!(dom.value(root), "owned");
         assert!(validate_xml("<?xml-stylesheet type='text/css'?>\n<p>owned</p><?tail?>").is_ok());
     }
+
+    #[test]
+    fn opaque_declarations_and_top_level_cdata_skip_their_whole_construct() {
+        // quick-xml (the gate's reader) consumes an internal subset or a
+        // top-level CDATA section as one opaque event, so its depth scan
+        // never sees the markup-looking text inside. The DOM must skip the
+        // same extent: ending a DOCTYPE at the first '>' re-exposes a
+        // quoted literal's (or subset comment's) contents as real elements
+        // that recurse as deep as the literal nests.
+        let nest = |n: usize| format!("{}{}", "<a>".repeat(n), "</a>".repeat(n));
+        for opaque in [
+            format!("<!DOCTYPE d [<!ENTITY e \"{}\">]>", nest(100_000)),
+            format!("<!DOCTYPE d [<!-- > {} -->]>", nest(100_000)),
+            format!("<![CDATA[{}]]>", nest(100_000)),
+        ] {
+            let xml = format!("{opaque}<w:document/>");
+            let mut dom = Dom::new();
+            let document = parse_xdocument(&mut dom, &xml);
+            // Only the real root: the hidden literal parsed as elements
+            // would add top-level roots (and, before that, recurse to
+            // build them).
+            assert_eq!(dom.child_count(document), 1, "{}…", &opaque[..40]);
+            let root = dom.root(document).unwrap();
+            assert_eq!(dom.name(root).unwrap().local_name(), "document");
+        }
+        // In-root CDATA and ordinary declarations keep their meaning.
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, "<!DOCTYPE d><a><![CDATA[x > y]]></a>");
+        let root = dom.root(document).unwrap();
+        assert_eq!(dom.value(root), "x > y");
+    }
 }
 
 #[cfg(test)]
diff --git a/tests/input_admission.rs b/tests/input_admission.rs
index 2f9769e..6eda5a3 100644
--- a/tests/input_admission.rs
+++ b/tests/input_admission.rs
@@ -1390,3 +1390,53 @@ fn issue650_markdown_honors_explicit_tighter_budgets() {
         assert!(result.unwrap_err().to_string().contains("INVALID_XML"));
     }
 }
+
+#[test]
+fn revision_listing_skips_an_over_deep_renamed_notes_part() {
+    // The footnotes relationship selects `word/notes.bin`; admission's
+    // XML depth scan keys on the `.xml`/`.rels` suffix, so this part
+    // was never measured. Reading it through the checked boundary
+    // makes it absent, as a missing notes part is — not 100_000 frames
+    // of recursive parsing.
+    let deep = format!(
+        "{}{}",
+        "<w:footnote>".repeat(100_000),
+        "</w:footnote>".repeat(100_000)
+    );
+    let bytes = package(|writer, options| {
+        writer
+            .start_file("word/_rels/document.xml.rels", options)
+            .unwrap();
+        writer
+            .write_all(
+                br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdN" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="notes.bin"/></Relationships>"#,
+            )
+            .unwrap();
+        writer.start_file("word/notes.bin", options).unwrap();
+        writer.write_all(deep.as_bytes()).unwrap();
+    });
+    let revisions =
+        jubarte::document_comparer::get_revisions(&bytes, &WmlComparerSettings::default())
+            .expect("the over-deep renamed part is skipped, not parsed");
+    assert!(revisions.is_empty());
+}
+
+#[test]
+fn wml_document_parses_a_dtd_hidden_literal_without_recursing_into_it() {
+    // quick-xml (the admission scanner's reader) consumes the whole
+    // internal subset — quoted literal included — as one DocType event,
+    // so the package admits at depth zero. The DOM's declaration skip
+    // must agree: stopping at the first `>` inside the literal parsed
+    // the hidden markup as deeply nested elements.
+    let hidden = format!("{}{}", "<a>".repeat(100_000), "</a>".repeat(100_000));
+    let document = format!(
+        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE w:document [<!ENTITY e \"{hidden}\">]><w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body/></w:document>"
+    );
+    let bytes = package_with_document(&document);
+    let mut wml = WmlDocument::from_bytes(&bytes).expect("admission treats the DTD as opaque");
+    let root = wml.main_document_root().expect("the real root parses");
+    assert_eq!(
+        wml.dom().name(root).expect("named root").local_name(),
+        "document"
+    );
+}

```
