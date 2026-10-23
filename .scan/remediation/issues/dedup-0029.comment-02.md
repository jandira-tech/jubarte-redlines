## Full TDD red/green plan (1/1)

# dedup-0029 remediation plan

Round: `dedup-0029` (reducer prompt `claimedWorkerIds: ["6607bdd6-b3df-4cdc-898f-2626a65b8476"]`).
This round's own contributions, identified in `output/result.json` (63 findings total), are the
14 findings that either carry `codeEvidence[].id` values prefixed `dedup0029` (12) or whose
`provenance.sourceFindingIds` name only worker `6607bdd6…` (2). This patch fixes seven of them —
the ones whose minimal fix is behavior-preserving and safe to author by reading code — and lists
the remaining seven with the reason each is not in this patch.

**Verification status (honest):** no `cargo`/`mbx`/`rustc` was run — the ask forbids it. What was
run, exactly:

- `git apply --check .scan/remediation/patches/dedup-0029.patch` from the repository root →
  exit 0, all seven files check clean (787 insertions, 46 deletions).
- `rustfmt --check --edition 2024` (toolchain 1.99.1, run on scratch copies under `/tmp`, never
  on the checkout) → every edited file parses and is formatting-clean. `rustfmt` is a formatter,
  not a build/test command; it was the strongest syntax validation available inside the
  constraint.
- Second-pair review (dedup-0029 issue opener): re-ran both checks above on a fresh `/tmp`
  scratch copy of `src` with the patch applied (`git apply`, then `rustfmt --check --edition
  2024` per file, toolchain 1.99.1) — all seven files exit 0; re-verified the oxml-opc 0.14.2
  attribute-decode fact in the registry source; read every touched source region and every
  helper the new tests use (`package_with_document_rels`/`RELS_NS`/`HYPERLINK`, `plain`,
  `story_chars`, `FkpIndex`, `Lists`, `Story`, `page_with`, `Op::text`, `load_font_table`,
  the `LiberationMono-Regular.ttf` asset already `include_bytes!`-ed in `font_table.rs`) and
  every `PartFs::resolve_rel_target` call site (all pass `rels.items[].target`, already
  decoded). No patch defect found; two red/green claims in §1 and §2 were corrected (see
  below).
- Dependency fact verified by reading sources: oxml-opc 0.14.2
  `src/relationship.rs` `parse_relationship` reads every attribute through quick-xml 0.41
  `Attribute::decoded_and_normalized_value` (`quick-xml-0.41.0/src/events/attributes.rs:205`,
  which resolves the predefined entities, depth 1) — the basis for fix 5.

The TDD cycle must be completed by a maintainer: apply the patch, run the named tests red on the
pre-patch tree where noted, green after. Suggested commands (per AGENTS.md, sequential cargo
only): `cargo nextest run -E 'test(placeholder) | test(colspan) | test(baseline) | test(ampersand) | test(duplicate) | test(field_) | test(alias)'`
then the touched suites `src/opc/mod.rs`, `src/legacy_doc.rs`, `src/convert/{altchunk,raster,font_table}.rs`,
`src/xmllinq`, plus `cargo clippy -- -D warnings` and a `jubarte --help` smoke test.

---

## 1. Repeated relationship IDs trigger quadratic repair work (finding idx 27)

- **Where:** `src/opc/mod.rs:116` (`drop_repeated_relationships`, called from `PartFs::open_within`
  for the package rels and every part's rels).
- **Defect:** the first-occurrence lookup uses a `HashMap`, but duplicate reporting used
  `repaired.contains(&item.id)` — a linear scan of a growing `Vec`. `n` distinct IDs each repeated
  identically cost `n(n−1)/2` string comparisons on shallow metadata that passes every byte,
  depth, and attribute budget.
- **Failing test (new):** `src/opc/mod.rs`, `mod tests`,
  `many_distinct_duplicate_pairs_repair_once_each_with_linear_membership_work` — builds a
  package with 4 096 distinct relationship IDs each repeated identically; asserts
  `repaired_duplicate_ids().len() == n`, first-seen order (`rId0` first, `rId4095` last), and —
  through the new `DUP_MEMBERSHIP_PROBES` counter (the repo's established test-only mechanism
  counter pattern, `src/xmllinq/mod.rs:269-285` `ATTR_NAME_PROBES`) — `probes <= n`. Red/green:
  the semantic assertions (count, order, conflict refusal) pass on the pre-fix tree too — the
  old code was correct, only slow — so the counter documents the new contract. It counts one
  membership check a duplicate (the `ATTR_NAME_PROBES` convention for set-indexed lookups,
  `src/xmllinq/mod.rs:269-274` "bulk index lookups count once per incoming name"): a bump
  placed before the old `Vec::contains` would also count `n`, not `n²/2`, so the bound pins
  one-check-a-duplicate rather than witnessing the old scan's cost. The true red form
  instruments the old mechanism's per-comparison loop (a hand-rolled scan bumping inside the
  probe), which counts `n(n−1)/2` and fails the bound.
- **Minimal change:** a `HashSet<&str> reported` answers membership while the `Vec<String>`
  keeps first-seen reporting order; one `#[cfg(test)]`-gated probe bump beside the probe.
- **Must keep passing:** `a_repeated_relationship_id_is_repaired_when_identical_and_refused_otherwise`
  (repair recorded for `validate`, conflicting duplicates still `OpcError::InvalidRelationship`,
  clean packages report nothing) and `validate`'s `DUPLICATE_RID` reporting order.

## 2. Report text extraction quadratically groups baselines on tall pages (finding idx 62)

- **Where:** `src/convert/raster.rs:80` `page_text` (`convert/mod.rs:641` calls it for every page
  when building report text; that path performs no raster allocation, so `MAX_PAGE_PIXELS`
  never applies).
- **Defect:** `lines.iter_mut().find(|(k, _)| *k == key)` scans every existing baseline for every
  text op; a document-controlled tall page with `b` distinct baselines and `n` text ops costs
  `n·b` key comparisons.
- **Failing test (new):** `src/convert/raster.rs`, `mod tests`,
  `page_text_groups_many_distinct_baselines_with_linear_lookups` — 256/1 024/4 096 ops on as many
  distinct baselines; asserts one output line per baseline, first-seen order, and
  `BASELINE_LOOKUPS` (new test-only counter) growing with the op count
  (`counts[2] <= 16 * counts[0]`). The counter counts one lookup a text op (the same
  `ATTR_NAME_PROBES` bulk convention), so a bump placed before the old `.find` scan would also
  count `n` — the bound pins one-lookup-a-op, and the true red form instruments the old scan's
  per-comparison closure (`n·b` total, which fails it). The output assertions (one line a
  baseline, first-seen order, space rules) are the semantic guards and are unchanged by the
  refactor.
- **Minimal change:** a `HashMap<i64, usize>` maps baseline key → index into `lines`; `lines`
  keeps `(key, pieces)` in first-seen order so output text is byte-identical.
- **Must keep passing:** `page_text_groups_by_baseline_and_spaces_only_real_gaps`,
  `hidden_text_paints_nothing_but_reads_back`, and every other `page_text` assertion (the
  grouping order and space-insertion rules are untouched).

## 3. Legacy DOC placeholder selection repeatedly rescans predictable text (finding idx 61)

- **Where:** `src/legacy_doc.rs:312` `empty_placeholder`, called unconditionally from
  `document_to_docx` (`src/legacy_doc.rs:243`).
- **Defect:** candidates `jubarteemptyparagraph0,1,…` were each checked with
  `texts.iter().all(|text| !text.contains(candidate))`; a document containing the sequential
  tokens 0…99999 forces ~100 000 full-text scans (candidates × text).
- **Failing tests (new), `src/legacy_doc.rs`, `mod tests`:
  - `sequential_occupied_candidates_select_with_one_scan_a_text_block` — a document holding
    candidates 0…1 999 plus an empty paragraph selects `jubarteemptyparagraph2000` with
    `PLACEHOLDER_SCANS` (new test-only counter) `<= blocks`. On the old mechanism the identical
    instrumentation counts `2 001 × 2` block scans, so the bound fails.
  - `a_document_without_empty_paragraphs_skips_placeholder_selection` — a document with no empty
    paragraph converts with zero scans (the remediation's "skip placeholder selection when no
    empty paragraph requires it").
  - `empty_placeholder_skips_exactly_the_occupied_numbers` — equivalence guard: digit-run prefix
    occupation picks exactly the candidates the old substring test picked
    (`…paragraph12xyz` occupies 1 and 12; a leading zero occupies only 0).
  - `a_document_with_empty_paragraphs_still_converts_and_cleans_placeholders` — the full
    doc round trip still removes every placeholder run from `word/document.xml`.
- **Minimal change:** `document_to_docx` hoists the (order-independent) `empties` count above
  placeholder selection and skips the scan when it is 0; `empty_placeholder` makes one pass per
  text block collecting the digit runs that follow the fixed prefix (each run's ≤ 10 canonical
  prefixes are the occupied numbers — provably the same acceptance set as the substring test,
  since a candidate string occurs iff its number begins one of those runs), then walks `n = 0…`
  against that set. Module style kept: `.get()` slicing and `saturating_add` only, honouring the
  module's `deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)`.
- **Must keep passing:** `empty_paragraphs…` and heading/round-trip tests around
  `document_to_docx` (`src/legacy_doc.rs:2120-2139` asserts empty paragraphs survive as empty,
  placeholder-free `<w:p>`), and `to_markdown` (untouched).

## 4. Reading or converting nested legacy DOC fields can monopolize the thread (finding idx 3)

- **Where:** `src/legacy_doc.rs:1604` `Story::blocks` — `let in_code = fields.iter().any(|&code| code);`
  ran for every story character, scanning the whole field stack each time.
- **Defect:** a Word 97-2003 story with ~100 000 begin/separator pairs (all frames in result
  state) makes every following character inspect 100 000 booleans: quadratic work during
  `read`, `doc_to_docx`, `doc_to_markdown`.
- **Failing tests (new), `src/legacy_doc.rs`, `mod tests`:
  - `deeply_nested_field_results_read_without_rescanning_the_field_stack` — 10 000 nested
    begin/separator pairs, one visible `R`, 10 000 ends; asserts `["aRb"]`. On the pre-fix code
    this input performs ~3×10⁸ stack inspections (the red form of this test is its runtime; the
    output assertion itself is the semantic guard).
  - `field_marker_edge_shapes_keep_their_visible_text` — repeated separator, unmatched end,
    separator with no begin, code-ancestor suppression: all keep the exact visible text the
    stack scan produced.
- **Minimal change:** keep `fields: Vec<bool>` (a separator/end must know its own frame's state)
  but add `code_frames: usize`, maintained on begin (`saturating_add`), separator
  (decrement only when the top was `true`), and end (decrement when the popped frame was `true`);
  `in_code` becomes `code_frames > 0` — an O(1) read with identical semantics.
- **Must keep passing:** `a_field_nested_in_a_field_code_shows_only_the_outer_result`
  (`src/legacy_doc.rs:1946`), the surrogate/piece-table story tests, and every
  `doc_to_docx`/`doc_to_markdown` fixture (`tests/legacy_doc_convert.rs`).
- **Deliberately not implemented:** the finding's optional depth/work budget that would refuse
  over-deep field nesting. It changes which documents convert (a Word-faithfulness decision per
  AGENTS.md) and is not needed to remove the quadratic; recorded here as follow-up material.

## 5. Entity decoding repeatedly scans the input suffix (finding idx 29)

Three coordinated defects; the first two are work-only, the third is also a correctness fix.

### 5a. `unescape_xml_text` (`src/xmllinq/parse.rs:764`)

- **Defect:** for each `&`, `chars[i + 1..].iter().position(|&c| c == ';')` searched the whole
  remaining suffix; a million bare ampersands (or unterminated references) in one attribute is
  quadratic. This helper runs for every attribute value and text node the DOM parses, including
  `.rels` targets and `validate` paths.
- **Failing tests (new), `src/xmllinq/parse.rs`, `mod xml_lexical_boundary_contract_tests`:
  - `long_bare_ampersand_runs_decode_with_linear_scan_work` — `&`-runs at 4 096/16 384/65 536
    and `&quuu` runs: asserts literal output and, via the new `ENTITY_SCAN_STEPS` counter
    (`src/xmllinq/mod.rs`, the `ATTR_NAME_PROBES` pattern), steps grow linearly
    (`counts[2] <= 4 * counts[1]`, `steps <= 10 × occurrences`). The same instrumentation over
    the old whole-suffix search counts ~`n²/2` and fails.
  - `grammar_bounded_reference_walk_keeps_the_literal_fallback_cases` — 16 edge shapes
    (bare `&`, `&#6x;`, `&#x4G;`, `&#x;`, `&#00000000000000000065;` → `A`, 5+-letter names,
    multibyte characters after a short reference) asserting byte-identical output to the old
    decoder.
- **Minimal change:** a new `entity_at(chars, at)` walks the reference grammar — `#`-digit run,
  `#x`-hex run, or at most the four letters of `quot`/`apos` — and requires the closing `;`
  right after it. `decode_entity` is untouched, so decoding results are unchanged; only the
  terminator search is bounded by each reference's own length (numeric digit runs are consumed
  monotonically, keeping long zero-padded references decodable).

### 5b. `decode_entities` (`src/convert/altchunk.rs:724`)

- **Defect:** `tail.find(';').filter(|&e| e <= 10)` bounded the accepted entity *length* but
  still scanned the entire suffix before filtering; ampersand-heavy HTML/MHT altChunk text is
  quadratic.
- **Failing test (new):** `src/convert/altchunk.rs`, `mod tests`,
  `a_multibyte_character_never_splits_the_entity_window` — `;` inside the ten-byte window is
  still found when a multibyte character straddles the window edge (`&nbsp;aaaa🎉` →
  `\u{a0}aaaa🎉`), bare `&` runs stay literal, `&#x1F433;` still decodes. Red form: the work
  bound is the same counter argument as 5a (the search itself is what changed); the assertions
  pin the semantics.
- **Minimal change:** search `;` only within the window —
  `char_indices().take_while(|(i, _)| *i <= 10).find(...)` — byte-boundary-safe by construction,
  accepting exactly the `e <= 10` offsets the old filter accepted.

### 5c. `PartFs::resolve_rel_target` double decode (`src/opc/mod.rs:680`)

- **Defect:** oxml-opc 0.14.2 already hands over decoded attribute values (verified in the
  dependency source, see header), so the method's second `unescape_xml_text` re-decoded: a part
  literally named `a&amp;b.xml` (XML `Target="a&amp;amp;b.xml"`) resolved to the non-existent
  `a&b.xml`, and ampersand-heavy targets were needlessly rescanned on every resolve.
- **Failing test (new):** `src/opc/mod.rs`, `mod tests`,
  `an_escaped_ampersand_target_decodes_exactly_once` — a package with the part
  `word/a&amp;b.xml` and the escaped target; asserts `resolve_rel_target` returns
  `word/a&amp;b.xml` and that `part_bytes` finds it. Red on the old code, which returns
  `word/a&b.xml` and misses the part. Green after removing the redundant decode.
- **Minimal change:** the method now calls the pure resolver directly; the comment records the
  dependency fact and tells raw-text holders to decode once themselves.
- **Must keep passing:** every current caller (checked: all 20+ in `src/`, none outside) passes
  `rels.items[].target` — already-decoded values; `public_and_package_relationship_readers_agree`
  (which round-trips an `&amp;` target) and the `&`-bearing external-target test at
  `src/opc/mod.rs:980` are the guards.

## 6. Rendering an HTML chunk can allocate gigabytes from a single colspan (finding idx 5)

- **Where:** `src/convert/altchunk.rs` — `start()`'s `td`/`th` arm accepted any `usize` colspan,
  and `end_table()` emitted one `<w:gridCol/>` per summed column before any parser or budget
  saw the generated WML (`with_layout`, `src/convert/mod.rs:791`, expands altChunks after
  admission).
- **Defect:** `colspan="4294967295"` in a few hundred bytes demands a 4-billion-iteration
  expansion (~85 GB of markup); a short row whose spans sum high does the same.
- **Failing test (new):** `src/convert/altchunk.rs`, `mod tests`,
  `a_colspan_past_words_table_limit_caps_at_63_columns` — `colspan` of 64, 1000, 1000000000 and
  4294967295 each generate exactly 63 `gridCol`s and < 10 KB of markup; three `colspan='30'`
  cells (sum 90) also cap at 63; the inclusive `colspan='63'` keeps today's merged-cell shape
  (`gridSpan w:val="63"`). Red on the old code, which emits one gridCol per column
  (1 000 ≠ 63; the 4294967295 case manifests as unbounded output growth).
- **Minimal change:** `const MAX_TABLE_COLUMNS: usize = 63` — Word's own table column limit (a
  row holds at most 63 cells; wider HTML tables are not representable in WordprocessingML) —
  applied at both the parsed per-cell span and the generated grid width.
- **Must keep passing:** `html_tables_keep_spans_empty_cells_and_header_formatting`
  (colspan 2 → 2 gridCols, spans/widths/borders exactly as today) and
  `invalid_or_zero_colspan_falls_back_to_one_column` (0/invalid/-1 → 1).
- **Word-faithfulness note:** tables wider than 63 columns previously produced packages Word
  itself cannot represent; capping them is a limit Word already imposes. Rows whose span sum
  exceeds 63 after per-cell clamping produce gridSpans beyond the capped grid — the same
  inconsistency Word's own HTML import tolerates for unrepresentable tables; recorded as a
  known edge rather than silently reshaped.

## 7. Rendering an admitted DOCX retains gigabytes of duplicate embedded fonts (finding idx 22)

- **Where:** `src/convert/font_table.rs:205` `load_embedded_fonts` — every font-table alias that
  resolves the same embedded part re-copied and re-parsed the full font
  (`deobfuscate_odttf` copies all bytes) and stored a private buffer per alias name.
- **Defect:** thousands of distinct alias names referencing one embedded font retain thousands
  of full copies after admission, with no aggregate budget on this path.
- **Failing test (new):** `src/convert/font_table.rs`, `mod embedded_font_alias_tests`,
  `many_aliases_of_one_embedded_font_share_one_decoded_payload` — four aliases over one
  `rId1`/fontKey: asserts every slot holds the same `Arc` (`Arc::ptr_eq`) and the decoded bytes.
  Red on the old code (four separate allocations), green after.
  `distinct_keys_and_malformed_fonts_stay_distinct_and_refused` guards the boundaries: different
  fontKeys never share a buffer, malformed fonts are refused for every alias.
- **Minimal change:** a per-call `HashMap<(part, fontKey), Option<Arc<[u8]>>>` decodes (and
  validates with `ttf_parser::Face::parse`) once per part+key, shares the `Arc` across aliases
  and styles, and remembers refusals so malformed fonts are not re-decoded per alias.
  `EmbeddedFonts` was already `Arc<[u8]>`-valued (`src/convert/font.rs:1449`), so sharing is the
  remediation's own shape.
- **Must keep passing:** `case8_odttf_parses_after_deobfuscation_when_present` (real package,
  real `.odttf`), the obfuscation/GUID contract tests, and the catalogue-skip rule (an embedded
  copy of a family the catalogue paints is still left out).
- **Not implemented:** the finding's optional aggregate decoded-font byte budget; with sharing,
  retained memory is one copy per distinct (part, key), which the package's own byte budget
  already bounds. A typed refusal budget would need `InputLimits` plumbing through
  `with_layout` and is left as follow-up.

---

## This round's findings not patched here (7)

| Finding (idx) | Why not in this patch |
|---|---|
| 0 — DOCX table grid spans exhaust memory | Composite of 13+ earlier rounds' evidence; the budget system it needs (`InputLimits`-style generated-output accounting through `with_layout`'s table grid) is a design decision, not a minimal blind-authorable fix. |
| 1 — renamed auxiliary XML bypasses XML depth limits | Requires an admission-policy decision (which consumed XML parts are depth-checked and at what budget) across `admit_package`/`PartFs` consumers; changing it blind risks refusing Word-valid packages. |
| 8 — WMF record-boundary failures | Needs per-record-type count validation across all `metafile.rs` handlers plus cumulative work budgets; too large to author safely without compiling. |
| 9 — aliased legacy DOC pieces expand logical stories | Needs configurable logical-character/expansion budgets plumbed through `read`/`doc_to_*` and the CLI — public API design, not a minimal fix. |
| 30 — PR code runs on shared self-hosted runners | CI infrastructure decision (`runs-on`/trigger policy on `.github/workflows/*`); not a code fix I can validate, and changing runners blind can break the project's build farm. |
| 32 — ragged legacy DOC tables pad to a rectangle | The quota needs a generated-cell/output budget in an infallible writer (`push_block` → `String`), i.e. the same budget plumbing as 0/9; no clean constant exists without a policy decision. |
| 56 — repeated compressed-image references | Conversion-wide decoded-pixel accounting and shared decode cache across image/group/VML consumers — a rendering-pipeline change too broad for this patch. |

## Patch

`.scan/remediation/patches/dedup-0029.patch` — one unified diff, 7 files, applies with
`git apply` from the repository root (verified: `git apply --check` exit 0; repository working
tree left clean).
