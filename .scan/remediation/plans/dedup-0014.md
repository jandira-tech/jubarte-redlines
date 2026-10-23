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
