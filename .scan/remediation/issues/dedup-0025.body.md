@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
### csf_09abb95061370ae3d7d342ca | Redaction succeeds while UTF-16 XML retains the removed text
severity: medium
summary: The redact operation can return successful clean and redline documents while a supported BOM-marked UTF-16LE/BE auxiliary XML part still contains the same confidential text. Its XML leak branch uses lossy UTF-8 and misses the retained copy, which a recipient can recover after sharing. Even the scanner's no-root fallback searches the already lossily decoded string, so it cannot recover a multi-character value split by UTF-16 NUL bytes.
remediation:
Decode every XML part with the same BOM-aware policy as admission before semantic leak scanning. Fail closed on unsupported/undecodable/unparseable representations and retain raw UTF8 and both UTF16 byte-order checks as defense in depth. Use the same decoding across leak checks and package validation.

### csf_32d60b431e9d66bf961b0a29 | Redaction succeeds while UTF-16 XML retains the removed text
severity: medium
summary: The redact operation can return successful clean and redline documents while a supported BOM-marked UTF-16LE/BE auxiliary XML part still contains the same confidential text. Its XML leak branch uses lossy UTF-8 and misses the retained copy, which a recipient can recover after sharing. Even the scanner's no-root fallback searches the already lossily decoded string, so it cannot recover a multi-character value split by UTF-16 NUL bytes.
remediation:
Decode every XML part with the same BOM-aware policy as admission before semantic leak scanning. Fail closed on unsupported/undecodable/unparseable representations and retain raw UTF8 and both UTF16 byte-order checks as defense in depth. Use the same decoding across leak checks and package validation.

### csf_68acfaf376a60cf8c3d8e2d4 | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion
severity: medium
summary: A drawing: image destination in ordinary Markdown is base64-decoded and deserialized into a dependency graph without validation. Package materialization recursively follows each dep.rels.part before writing the dependent part and has no active-node or completed-node map. A one-node self-reference causes indefinite recursion and stack exhaustion, even with default Markdown conversion options and no image loader. Header/footer drawing payloads also reach this same materializer through view_stories::story_part. Capture memoizes a part before descending, so even capture-produced envelopes can encode a cycle; capture count/byte limits do not establish an acyclic imported graph. The supplied discovery-0030 trace additionally records that an empty boxes collection bypasses text-box-specific Writer finish checks; these checks do not validate the dependency graph.
remediation:
Validate dependency indices, count/byte/depth limits and graph shape at import. Materialize with an active/completed traversal state and index-to-output map, rejecting cycles or reserving identities to support cycles without recursive expansion. Memoize shared subgraphs. Enforce explicit dependency-edge and traversal-work budgets before assembly; when rejecting active-node revisits return a typed conversion refusal. Apply the same graph validation and traversal state to header/footer drawing materialization, not only body drawings.

### csf_2cc2c8f85ad501547f61a5af | Public Strict-format probe inflates unbounded document headers
severity: medium
summary: The public main_part_is_strict helper reads word/document.xml without admission or an inflation/preamble cap. A compressible oversized/unfinished root opening tag or long XML comment keeps the last '<' at offset zero; the full growing head remains retained and is repeatedly scanned until the root tag completes or EOF, allowing memory/CPU exhaustion for direct library consumers. The assigned audit establishes direct public-library exposure and found no in-repository production caller; ordinary rendering is not claimed to call this detector.
remediation:
Apply bounded admission before indexing/inflation and explicit cumulative inflated-byte/preamble/work limits to this public helper. Use a streaming lexical probe with bounded retained state for comments, declarations and chunk-spanning root tags; refuse excessive probes with a checked/limited API and safe default limits.

### csf_64b247931cd1f053217fde7a | Rendering an HTML chunk can allocate gigabytes from a single colspan value
severity: medium
summary: A document author can place a huge positive colspan in a small embedded HTML/MHT altChunk table. Ordinary DOCX rendering generates one grid XML element per effective column after original package admission and before DOM/layout checks. A count of 1000000000 drives a billion appends; 4294967295 generates over 85 GB of markup, exhausting CPU/memory. The supplied positive 4294967295 framing fits both 32-bit and 64-bit usize and needs no arithmetic overflow. Assigned discovery-0031 corroborates colspan=1000000000 causing tens of gigabytes of generated XML and work before layout; rejecting invalid, negative or zero spans does not bound valid positive values. Assigned discovery-0032 supplies a static colspan=1000000000 framing of one billion 20-byte grid elements, about 20 GB of generated text before DOM construction; this is not a runtime measurement.
remediation:
Enforce a practical maximum cell span and checked per-row/aggregate column counts before generating any table XML. Charge all HTML/MHT-generated WML, including repeated/nested tables, to a cumulative byte/work budget inside altChunk expansion and return a typed input-limit refusal before allocation or DOM construction.
Additional retained remediation detail: Bound individual spans and aggregate columns before summation or iteration, use checked arithmetic, and impose a cumulative generated-WML byte/element budget shared across every HTML/MHT altChunk. Refuse excessive expansion with a typed input-limit error before allocation.

### csf_32625277093e7f286a267e2b | Rendering an HTML chunk can allocate gigabytes from a single colspan value
severity: medium
summary: A document author can place a huge positive colspan in a small embedded HTML/MHT altChunk table. Ordinary DOCX rendering generates one grid XML element per effective column after original package admission and before DOM/layout checks. A count of 1000000000 drives a billion appends; 4294967295 generates over 85 GB of markup, exhausting CPU/memory. The supplied positive 4294967295 framing fits both 32-bit and 64-bit usize and needs no arithmetic overflow. Assigned discovery-0031 corroborates colspan=1000000000 causing tens of gigabytes of generated XML and work before layout; rejecting invalid, negative or zero spans does not bound valid positive values.
remediation:
Enforce a practical maximum cell span and checked per-row/aggregate column counts before generating any table XML. Charge all HTML/MHT-generated WML, including repeated/nested tables, to a cumulative byte/work budget inside altChunk expansion and return a typed input-limit refusal before allocation or DOM construction.

### csf_e02194eb587697016bdfe08d | Rendering nested HTML tables bypasses the XML depth limit
severity: medium
summary: HTML table expansion runs after package admission and can generate arbitrarily deep WordprocessingML, which is passed to the unchecked recursive DOM parser. The supplied corroboration preserves the innermost-paragraph prerequisite for retaining nested cell WML and confirms that huge colspan is unnecessary; exact abort depth is platform-dependent.
remediation:
Bound HTML nesting/generated output and validate expanded WML under active depth/tag limits before DOM construction.

### csf_9e40f23e742aa43493a0fb92 | Rendering a small DOCX table can exhaust host memory
severity: medium
summary: An untrusted DOCX author can set huge positive w:gridSpan, w:gridBefore or w:gridAfter counts in a small, shallow table. Rendering sums those counts into grid occupancy and directly sizes column, preference and merge vectors. Counts of 1000000000/1073741824 request about 4 GB/4 GiB for f32 columns; 4294967295 requests approximately 16 GiB before additional allocations, despite package admission. Assigned discovery-0031 corroborates the existing one-billion-column, 4,000,000,000-byte f32-vector framing through both span and row-skip inputs; these are static counts. Assigned discovery-0032 corroborates the live span and both synthetic row-skip paths, including merge-slot expansion; separate Markdown span clamping does not protect rendering.
remediation:
Enforce practical per-span and total-column budgets for gridSpan, gridBefore and gridAfter before excessive RawCell, column, preference or merge allocation/work. Use checked occupancy sums and col+span arithmetic, apply the same budget to every derived table vector, and return a typed resource-limit refusal on excess. Propagate a caller-adjustable semantic column limit through all public rendering surfaces and charge later column/merge vectors to that same budget.
Additional retained remediation detail: Enforce explicit maximum live cell spans, row skips, aggregate table columns and merge slots before any allocation. Use checked sums and a cumulative layout resource budget, returning a typed refusal when exceeded.

### csf_0e2b085a251b34412cd71a4c | Reading or converting nested legacy DOC fields can monopolize the processing thread
severity: medium
summary: An attacker-authored Word 97–2003 DOC can repeatedly begin fields and enter their result sections, leaving an increasingly long all-false field stack. Story::blocks scans the entire stack before every input character, giving quadratic CPU work during reading/conversion and even the native Word-input refusal hint. Assigned discovery-0031 supplies a balanced result-state framing with approximately 100000 nested fields in a few hundred kilobytes, implying billions of boolean inspections. This is heap-stack scanning CPU cost, not recursive call-stack overflow.
remediation:
Maintain an O(1) active-code-frame count, updating it correctly on begin, separator and end while preserving suppression under any code ancestor. Enforce explicit field-nesting and processing-work budgets for balanced and malformed legacy field streams before excessive work. Keep legacy diagnostic probes cheap and bounded, or enforce the same field/work budgets before invoking the full reader solely for a refusal hint.

### csf_88d031991ca3e20e4aae2f24 | Reading or converting nested legacy DOC fields can monopolize the processing thread
severity: medium
summary: An attacker-authored Word 97–2003 DOC can repeatedly begin fields and enter their result sections, leaving an increasingly long all-false field stack. Story::blocks scans the entire stack before every input character, giving quadratic CPU work during reading/conversion and even the native Word-input refusal hint. Assigned discovery-0031 supplies a balanced result-state framing with approximately 100000 nested fields in a few hundred kilobytes, implying billions of boolean inspections. This is heap-stack scanning CPU cost, not recursive call-stack overflow.
remediation:
Maintain an O(1) active-code-frame count, updating it correctly on begin, separator and end while preserving suppression under any code ancestor. Enforce explicit field-nesting and processing-work budgets for balanced and malformed legacy field streams before excessive work. Keep legacy diagnostic probes cheap and bounded, or enforce the same field/work budgets before invoking the full reader solely for a refusal hint.

## TDD red/green plan
# dedup-0025 remediation plan

Round: dedup-0025 (reducer prompt `claimedWorkerIds: ["913193c4-a76e-49cf-9c9d-c1abd00d83d6"]`).
Source of findings: `.scan/artifacts/deep_discovery/dedup/dedup-0025/output/result.json`
(61 aggregated findings; 12 carry this round's own `dedup0025-n*` evidence refs, extracted with
`jq`). This plan covers the seven findings whose fix is minimal and local, in TDD red/green form,
and records why the other five are not patched here.

Patch: `../patches/dedup-0025.patch` — one unified diff, applies with `git apply` from the
repository root (verified with `git apply --check`, see Verification).

Per AGENTS.md: tests live beside the behavior they protect (all new tests are in the same
`#[cfg(test)]` modules as the code they guard), no `#[allow(...)]` suppressions are introduced,
and the modules that deny `clippy::arithmetic_side_effects`/`clippy::indexing_slicing` in
non-test builds (`src/legacy_doc.rs:8`, `src/admission.rs:8`, `src/strict_translation.rs:8`)
only get saturating/checked arithmetic from this patch.

---

## 1. Redaction succeeds while UTF-16 XML retains the removed text (medium)

Evidence refs: `dedup0025-n8-redaction`, `dedup0025-n8-decode`, `dedup0025-n8-fallback`,
`dedup0025-n8-preserve`, `dedup0025-n8-accept-encoding` (plus inherited `dedup0005-n4-*`).
Root cause: `leaks_within` (`src/scrub.rs:467-472`) scans `.xml`/`.rels` parts as
`String::from_utf8_lossy`. A BOM-marked UTF-16LE/BE XML part (admission reads these:
`src/admission.rs:764-782` `part_text`) becomes NUL-interleaved text, so neither the semantic
`xml_holds` scan nor its no-root `contains` fallback finds the needle. `check_redactions`
(`src/edit.rs:4727`) then reports no leak and the redaction "succeeds" while the confidential
text stays in the outgoing ZIP (`src/opc/mod.rs:659-664`). The non-XML branch also checks only
UTF-16LE bytes (`src/scrub.rs:425`), missing a UTF-16BE binary copy.

### Failing test
`src/scrub.rs`, module `public_memory_leak_boundary_tests`, test
`utf16_xml_and_utf16be_parts_hold_the_secret_like_any_other_copy`.
Builds (via the module's existing `package()` fixture) a document with:
`customXml/item1.xml` (BOM UTF-16LE XML holding the secret in a comment and an attribute),
`customXml/item2.xml` (BOM UTF-16BE, same), `customXml/item3.xml` (BOM UTF-16LE, not XML once
decoded — exercises the no-root fallback) and `customXml/utf16be.bin` (raw UTF-16BE bytes), and
asserts `leaks(&bytes, secret)` returns exactly those four part names.
RED before the fix: the returned list is empty (or misses the parts), so the `assert_eq!` fails.
GREEN after: all four are reported.

### Minimal source change
- `src/admission.rs`: `part_text` becomes `pub(crate)` (one word) so the scanner uses the same
  BOM-aware decode as admission.
- `src/scrub.rs`: the XML branch calls `xml_holds(&crate::admission::part_text(&bytes), needle)`
  instead of lossy UTF-8; the binary branch keeps the UTF-8 check and now checks both byte
  orders (`utf16le`/`utf16be`). Doc comment of `leaks` updated.
  No raw-byte check is added to the XML branch on purpose: it would flag numeric attribute
  values the semantic scan deliberately skips (see the lock test below).

### Existing behaviour that must keep passing
`src/scrub.rs` tests: `public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources`
(the UTF-16LE `.bin` part is still found; the package round-trips),
`decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not`
(`leaks(..., "1234").is_empty()` over `w:before='1234'` — the reason there is no raw-byte XML
check), `a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is`,
`element_names_are_no_leak`, and the `bounded_input_tests` module (fail-closed budgets).

---

## 2. Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion (medium)

Evidence refs: `dedup0025-n1-entry`, `dedup0025-n1-decode`, `dedup0025-n1-accept`,
`dedup0025-n1-materialize`, `dedup0025-n1-cycle`, `dedup0025-n1-caller`.
Root cause: a `drawing:` destination is base64+JSON deserialized (`src/markdown/from_docx/drawing.rs:1397`)
with caller-chosen dependency indices; `dep_part` (`drawing.rs:1956-2008`) follows
`rel.part → deps[index]` recursively with no in-progress or completed set, so a one-node
self-reference (or any cycle) recurses until the stack is gone. Both callers
(`src/markdown/package.rs:1740`, `src/markdown/view_stories.rs:1542`) already handle
`materialize` returning `None` by skipping the drawing.

### Failing test
`src/markdown/from_docx/drawing.rs`, module `tests`, test
`a_dependency_cycle_is_refused_instead_of_recursing_without_end`.
Hand-builds a `Drawing` whose only relationship targets dep 0 and dep 0's only relationship
targets dep 0 (via the module-private `PayRel`/`Dep` structs; the fixture package is
`tests/fixtures/redline/original.docx` through a `story_package()` helper), calls `materialize`
and asserts the result `.is_none()`.
RED before: the process aborts on stack overflow (the test fails by crashing).
GREEN after: `None` — the drawing is refused, exactly what an undecodable payload already gets.

### Minimal source change
`src/markdown/from_docx/drawing.rs`: `materialize` opens `active: HashSet<usize>` (the walk's
in-progress indices) and `written: HashMap<usize, String>` (index → emitted part name) and
threads both into `dep_part`. `dep_part` becomes a guard: memo hit returns the recorded name,
`active.insert` failure returns `None` (cycle), otherwise the original body runs in a new
`dep_closure(drawing, dep, index, …)` and the index leaves `active` on every path.
Memoization is the "written once" rule the byte-equality comment at the tail already implies
(the second reference previously collapsed through that same check after wasting a rename), and
it bounds shared-subgraph work to one materialization per index.

### Existing behaviour that must keep passing
New lock test `a_directed_dependency_closure_still_materializes` (an acyclic two-references-one-
part payload still materializes its part), plus the existing drawing round-trip coverage behind
`markdown_to_docx` (`src/markdown/write.rs` acceptance path, `src/markdown/package.rs` and
`view_stories.rs` materialization callers, `xml_round_trips_verbatim`,
`doc_pr_ids_normalize_and_renumber`).

---

## 3. Public Strict-format probe inflates unbounded document headers (medium)

Evidence refs: `dedup0025-n12-read`, `dedup0025-n12-finish`, `dedup0025-n12-export`.
Root cause: `main_part_is_strict` (`src/strict_translation.rs:670-717`, public via
`src/lib.rs:144`) trims its retained head to the bytes since the last `<`; a part whose only
`<` sits at offset zero keeps every byte, re-running `from_utf8_lossy` + `root_open` over the
growing head per 16 KiB chunk until EOF — unbounded retention and quadratic rescanning, with no
read budget of any kind. It is a public helper with no in-repo production caller.

### Failing test
`src/strict_translation.rs`, module `tests`, test `an_endless_root_head_is_refused_as_undecided`.
A `word/document.xml` whose root opening tag starts with the Strict namespace URI and never
closes (2 MiB of attribute filler, no `>`). Asserts `!main_part_is_strict(&pkg)`.
RED before: the endless head ends in `decl.contains(STRICT_MAIN)` → the function returns
`true`, so the assertion fails (after retaining and rescanning the whole head).
GREEN after: the head budget refuses the probe with `false`.

### Minimal source change
`src/strict_translation.rs`: new `const MAX_ROOT_HEAD_BYTES: usize = 1024 * 1024` next to
`STRICT_MAIN`; the read loop counts total bytes (`saturating_add`, the module denies plain
arithmetic outside tests) and returns `false` past `InputLimits::default().max_part_bytes`,
and returns `false` when the trimmed head alone passes `MAX_ROOT_HEAD_BYTES`. The function doc
records that an over-budget part answers `false` — undecided, the same answer a malformed part
gets. The existing 100 KiB-preamble test stays green because its comment trims away and its
root closes well inside 1 MiB.

### Existing behaviour that must keep passing
`a_long_preamble_does_not_hide_a_strict_root` (100 KiB comment preamble, #561),
`transitional_package_returned_byte_identical`,
`strict_package_is_rewritten_to_transitional`,
`oversized_part_is_left_untouched_under_a_small_budget`, and new lock test
`a_truncated_root_head_inside_the_budget_still_decides` (a tiny unclosed root tag still decides
from the declarations it holds — the truncation semantics at the tail of the probe).

---

## 4. Rendering an HTML chunk can allocate gigabytes from a single colspan value (medium)
## 5. Rendering nested HTML tables bypasses the XML depth limit (medium)

Evidence refs: `dedup0025-n5-span`, `dedup0025-n5-count`, `dedup0025-n5-emit`,
`dedup0025-n5-chunk`, `dedup0025-n5-entry`; `dedup0025-n6-table`, `dedup0025-n6-cell`,
`dedup0025-n6-retain`, `dedup0025-n6-recurse`, `dedup0025-n6-entry`.
Root cause (both): `w:altChunk` HTML/MHT is expanded into WML after admission
(`src/convert/mod.rs:789-798`) by `src/convert/altchunk.rs`. A `colspan` parsed as `usize`
(`altchunk.rs:562-567`) flows into `cols = max row sum of spans` (`altchunk.rs:469-476`) and
`for _ in 0..cols` emits one `w:gridCol` per claimed column (`altchunk.rs:488-491`) — a
`colspan` of 4294967295 generates ~85 GB of markup before any DOM or layout check. Separately,
nested `<table>` inside cells nests `<w:tbl>` WML without limit, and the expanded XML is parsed
by the recursive DOM parser with no depth re-check (`src/xmllinq/parse.rs:660-663`) — stack
exhaustion.

### Failing tests (both in `src/convert/altchunk.rs`, module `tests`)
- `a_colspan_past_the_cap_emits_a_bounded_grid`: `html_to_wml` on
  `<td colspan='4294967295'>x</td>` asserts `wml.matches("<w:gridCol").count() ==
  MAX_TABLE_COLUMNS` (1000) and `<w:gridSpan w:val="1000"/>` is present.
  RED before: the emission loop runs ~4.3e9 times — the test dies on time/memory.
  GREEN after: 1000 grid columns, span clamped to 1000.
- `tables_nested_past_the_cap_flatten_into_the_enclosing_table`: 40 nested
  `<table><tr><td>` wrappers around the text `deep`; asserts the WML holds exactly
  `MAX_NESTED_TABLES` (32) `<w:tbl>` elements and still contains `>deep</w:t>`.
  RED before: 40 tables are emitted (count 40 ≠ 32).
  GREEN after: the deepest 8 flatten into the enclosing table and the text survives.

### Minimal source change
`src/convert/altchunk.rs`: `const MAX_TABLE_COLUMNS: usize = 1000` (the HTML standard's own
colspan clamp; Word lays out at most 63 columns) — applied where the span is parsed (td/th arm)
and to the row-sum `cols` in `end_table`; `const MAX_NESTED_TABLES: usize = 32` — `start`'s
`"table"` arm counts opens past the cap in a new `Builder::skipped_tables` field and returns,
and `end`'s `"table"` arm closes a counted skip instead of popping a table that was never
pushed (self-closed `<table/>` at the cap stays balanced because `html_to_wml` calls `end`
right after a self-closed `start`). Skipped tables' rows/cells still flow to the enclosing
table, so content is imported, not dropped — the graceful-degradation reading of Word's own
HTML import limits. Generated WML depth stays ≤ ~4 elements per level × 32, inside the 256
admission depth budget.

### Existing behaviour that must keep passing
`a_class_rule_and_inline_bold_become_run_properties`, `an_alt_chunk_is_replaced_by_its_paragraphs`
(the Croatian-regulation fixture), `an_mht_chunk_yields_its_quoted_printable_html`, and the
`regression_tests` module in the same file (chunk order, missing chunks, surrounding content).

---

## 6. Rendering a small DOCX table can exhaust host memory (medium)

Evidence refs: `dedup0025-n4-span`, `dedup0025-n4-skip`, `dedup0025-n4-skip-cell`,
`dedup0025-n4-allocate`, `dedup0025-n4-raw`, `dedup0025-n4-entry`.
Root cause: `cell_span` (`src/convert/mod.rs:14204`) parses `w:gridSpan` and `row_grid_skip`
(`src/convert/mod.rs:14340`) parses `w:gridBefore`/`w:gridAfter` as unbounded `usize`. The row
occupancy sum (`src/convert/mod.rs:13601-13616`) then sizes `cols` (`vec![each; occupancy]` /
`cols.resize(occupancy, 80.0)`) and `resolve_table_merges`' `open` vector
(`src/convert/mod.rs:14637-14638`) follows: a `gridSpan` of 4294967295 asks for ~17 GiB of
`f32` columns from one attribute, long before any output exists.

### Failing test
`src/convert/mod.rs`, module `complete_coordinate_layout_contract_tests` (the module that owns
the `table()`/`dom()`/`sheet()` fixtures), test
`huge_spans_lay_out_a_bounded_grid_instead_of_allocating_it`.
A one-column table whose row carries `gridBefore 2000000000` and a cell with
`gridSpan 4294967295`; asserts `geom.pref.len() <= MAX_GRID_COLUMNS + 2` (`pref` is the
per-column preference vector `table_block` returns, i.e. the final grid width).
RED before: the occupancy path allocates a ~25 GiB `f32` vector — the test dies on allocation.
GREEN after: spans clamp to the budget, the grid is ≤ 4098 columns and the table lays out.

### Minimal source change
`src/convert/mod.rs`: `const MAX_GRID_COLUMNS: usize = 4096` (Word's own hard limit is 63
columns; 4096 leaves every corpus table untouched) documented at `cell_span`;
`.min(MAX_GRID_COLUMNS)` at both parse sites (`cell_span`, `row_grid_skip`), plus a row-level
budget pass after `stretch_stale_spans` and before the occupancy sum: each cell keeps at least
one column, and the row's total claim stops at the budget, so `occupancy`, the `cols` vectors
and `resolve_table_merges`' `col + span` arithmetic stay proportional to the part even for a
row of many padded spans. `stretch_stale_spans` itself runs before the pass unchanged (its
widening is bounded by the authored `tblGrid`).

### Existing behaviour that must keep passing
`content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins`
(small authored `gridSpan`/`gridBefore` still lay out with exact widths), the
`grid_padded` short-row semantics (the "91 corpus documents" comment), and the table corpus
behind `table_block` — no real document claims > 4096 columns in one row.

---

## 7. Reading or converting nested legacy DOC fields can monopolize the processing thread (medium)

Evidence refs: `dedup0025-n3-entry`, `dedup0025-n3-fields`, `dedup0025-n3-reader`.
Root cause: `Story::blocks` (`src/legacy_doc.rs:1591-1618`) computes
`in_code = fields.iter().any(|&code| code)` for every character. `\u{13}` pushes a frame,
`\u{14}` marks the top false, so N open-then-separated fields leave an N-long all-false stack
that every following character walks end to end: quadratic work from a small DOC through
`read`/`doc_to_docx` (`src/legacy_doc.rs:206-216`) and the CLI's legacy path
(`src/bin/jubarte.rs:3125-3131`).

### Failing test
`src/legacy_doc.rs`, module `legacy_byte_source_boundary_tests` (the module that already builds
a hand-owned `Story` around `FkpIndex { runs: Vec::new() }`).
- Lock test `field_codes_stay_suppressed_and_results_stay_through_any_nesting`
  (`\u{13}CODE\u{14}RESULT\u{15}AFTER` → `"RESULTAFTER"`; the nested case → `"EF"`): pins the
  suppression semantics the counter must reproduce (green before and after — the behaviour lock).
- Scale test `an_all_false_field_stack_costs_no_per_character_walk`: 20 000
  `\u{13}\u{14}` pairs then 20 000 `x` characters assert the exact output `"x".repeat(20_000)`.
  Its assertions are green before and after; its RED state is runtime — before the fix each of
  the 20 000 text characters walked the 20 000-entry stack (4×10⁸ boolean reads, seconds in a
  debug test run), after the fix it is one counter comparison. This is stated plainly: the
  red/green demonstration for a pure-quadratic defect is time, not an assertion.

### Minimal source change
`src/legacy_doc.rs`: `open_codes` counts frames still in their code; updated on begin
(saturating +1), separator (saturating sub `usize::from(*top)` before clearing the top — a
double separator decrements nothing) and end (decrement only when the popped frame was still
code). The per-character `any` scan and the `in_code` local disappear; suppression is
`_ if open_codes > 0`. All arithmetic is saturating (the module denies plain arithmetic
outside tests).

### Existing behaviour that must keep passing
`unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters` (same module),
`tables_become_github_tables`, `line_breaks_become_hard_breaks`,
`emphasis_markers_hug_the_text_and_lists_get_markers`, and the whole legacy fixture suite over
`tests/fixtures/legacy/services.doc` — the suppression semantics are bit-identical
(`any(fields) ⇔ open_codes > 0`).

---

## Findings from this round documented but not patched here

Honest scope statements, each grounded in the same evidence set:

- **Processing renamed auxiliary XML bypasses XML depth limits** (`dedup0025-n0-*`, 12 refs):
  the fix the finding asks for is a centralized checked XML parse boundary across every
  semantic consumer (theme, styles, numbering, comments, notes, headers/footers, settings,
  comparer/`get_revisions`) — an architectural change across a dozen modules, not a minimal
  patch that can be authored safely without compiling.
- **Aliased legacy DOC pieces expand small physical text into enormous logical stories**
  (`dedup0025-n2-*`): needs configurable piece-count / logical-character / work budgets
  threaded through `read`/`doc_to_markdown`/`doc_to_docx` and the CLI, plus a bounded
  refusal-hint probe — new public budget surface, out of minimal-patch scope.
- **Untrusted Markdown reference documents reach unchecked recursive XML parsing**
  (`dedup0025-n9-*`): the fix must admit reference packages under active `InputLimits` across
  Rust/Python/WASM/CLI reference paths — cross-binding budget plumbing, same reason.
- **Rendering an admitted DOCX can retain gigabytes of duplicate embedded fonts**
  (`dedup0025-n10-*`): cache decoded font buffers by resolved part + obfuscation key
  (`src/convert/font_table.rs:177-231`, `src/convert/font.rs:1445-1476`) with an aggregate
  decoded-font budget; touches the font pipeline used by every render, too broad to author
  unverified here.
- **Python and Node plan recording can overwrite unrelated files through staging symlinks**
  (`dedup0025-n7-*`): the fix is `O_EXCL`-style exclusive staging in
  `jubarte-python/python/jubarte_redlines/__main__.py:403-414` and
  `jubarte-wasm/cli/bin/jubarte-redlines.mjs:398-426` (predictable `.<plan>.<PID>.partial`
  names, truncating writes). Python/JS edits cannot be compile-checked under this round's
  constraints, so they are documented rather than patched blind.

---

## Verification

Ran (this session, from the repository root unless noted):

- `jq` extraction of the 61 findings and the 12 `dedup0025-n*`-carrying ones from
  `.scan/artifacts/deep_discovery/dedup/dedup-0025/output/result.json` (worker attribution via
  `provenance.sourceFindingIds`; the round's own refs via `codeEvidence[].id`).
- Read of every touched source region cited above (paths and line numbers in each section).
- `git apply --check --verbose .scan/remediation/patches/dedup-0025.patch` → all seven files
  check clean against the current working tree.

Not run — forbidden for this round (no Cargo process may run in this checkout): `cargo fmt`,
`cargo clippy -D warnings`, `cargo nextest run` for the named tests, `--help` smoke. The patch
was authored by reading code; every replacement was applied with exactly-once string
substitution over copies in `/tmp` and diffed, and the diff was verified to apply, but the
tests have not been executed. First run when the checkout is free:
`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, then
`cargo nextest run -E 'test(utf16_xml_and_utf16be) | test(a_dependency_cycle) | test(a_directed_dependency) | test(an_endless_root_head) | test(a_truncated_root_head) | test(a_colspan_past_the_cap) | test(tables_nested_past_the_cap) | test(huge_spans_lay_out) | test(field_codes_stay) | test(an_all_false_field_stack)'`
(each name is unique across the suite; drop the `-E` filter terms for any that were renamed).


## Patch
```diff
diff --git a/src/admission.rs b/src/admission.rs
index d556610..358b1d5 100644
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -765,7 +765,7 @@ fn check_part_name(name: &str) -> Result<(), AdmissionError> {
 /// with a byte-order mark (SharePoint's `customXml` items often are) and
 /// Word opens it, so it is transcoded; other bytes decode lossily, as the
 /// rest of the engine reads them.
-fn part_text(xml: &[u8]) -> Cow<'_, str> {
+pub(crate) fn part_text(xml: &[u8]) -> Cow<'_, str> {
     let utf16 = |rest: &[u8], unit: fn([u8; 2]) -> u16| {
         let units = rest.as_chunks::<2>().0.iter().map(|&pair| unit(pair));
         Cow::Owned(
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f9..1f85c4d 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -146,6 +146,19 @@ fn quoted_printable(s: &str) -> String {
     String::from_utf8_lossy(&out).into_owned()
 }
 
+/// The most columns one imported HTML table lays out. The HTML standard
+/// clamps a `colspan` at 1000 and Word lays no table out wider than 63
+/// columns; past the cap a span is a document's padding, not columns to
+/// emit — a `colspan` of 4294967295 used to append a `w:gridCol` per
+/// claimed column.
+const MAX_TABLE_COLUMNS: usize = 1000;
+
+/// The most tables nested one inside another. Word nests a handful deep;
+/// past the cap the inner content still imports, flattened into the
+/// enclosing table, instead of generating WML whose nesting no admission
+/// gate sees (the chunk is expanded after its package was admitted).
+const MAX_NESTED_TABLES: usize = 32;
+
 type Props = HashMap<String, String>;
 
 /// `selector { prop: value; … }` rules; a selector list shares its block.
@@ -276,6 +289,8 @@ struct Builder {
     fmts: Vec<(String, Fmt)>,
     tables: Vec<Table>,
     cell_spans: Vec<usize>,
+    /// `<table>` opens past the nesting cap: their end tags close nothing.
+    skipped_tables: usize,
 }
 
 const BLOCKS: [&str; 10] = [
@@ -472,7 +487,10 @@ impl Builder {
             .map(|r| r.iter().map(|c| c.span).sum::<usize>())
             .max()
             .unwrap_or(1)
-            .max(1);
+            .max(1)
+            // A row of padded spans cannot widen the grid past the cap:
+            // every claimed column became an emitted `w:gridCol`.
+            .min(MAX_TABLE_COLUMNS);
         let col_w = 9360 / cols;
         let mut s = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>");
         if table.border {
@@ -542,6 +560,10 @@ impl Builder {
             }
             "table" => {
                 self.flush();
+                if self.tables.len() >= MAX_NESTED_TABLES {
+                    self.skipped_tables += 1;
+                    return;
+                }
                 let border = attr(open, "border").is_some_and(|b| b.trim() != "0");
                 self.tables.push(Table {
                     border,
@@ -563,7 +585,10 @@ impl Builder {
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
                         .unwrap_or(1)
-                        .max(1),
+                        .max(1)
+                        // The HTML standard clamps a colspan at 1000; a
+                        // vaster one is padding, not columns to emit.
+                        .min(MAX_TABLE_COLUMNS),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
@@ -594,7 +619,13 @@ impl Builder {
             t if BLOCKS.contains(&t) => self.flush(),
             "td" | "th" if self.in_open_cell() => self.end_cell(),
             "td" | "th" => {}
-            "table" => self.end_table(),
+            "table" => {
+                if self.skipped_tables > 0 {
+                    self.skipped_tables -= 1;
+                } else {
+                    self.end_table();
+                }
+            }
             _ => {}
         }
     }
@@ -620,6 +651,7 @@ pub(crate) fn html_to_wml(html: &str) -> String {
         )],
         tables: Vec::new(),
         cell_spans: Vec::new(),
+        skipped_tables: 0,
     };
     let src = &html[body_at..];
     let mut i = 0;
@@ -817,6 +849,34 @@ mod tests {
         assert!(html.contains("<p class=\"x\">Članak</p>"), "{html}");
     }
 
+    #[test]
+    fn a_colspan_past_the_cap_emits_a_bounded_grid() {
+        // A colspan of 4294967295 used to append a `w:gridCol` per claimed
+        // column — tens of gigabytes of markup from one attribute.
+        let wml = html_to_wml(
+            "<html><body><table><tr><td colspan='4294967295'>x</td></tr></table></body></html>",
+        );
+        assert_eq!(wml.matches("<w:gridCol").count(), MAX_TABLE_COLUMNS, "{wml}");
+        assert!(wml.contains("<w:gridSpan w:val=\"1000\"/>"), "{wml}");
+    }
+
+    #[test]
+    fn tables_nested_past_the_cap_flatten_into_the_enclosing_table() {
+        let depth = MAX_NESTED_TABLES + 8;
+        let mut html = String::from("<html><body>");
+        for _ in 0..depth {
+            html.push_str("<table><tr><td>");
+        }
+        html.push_str("deep");
+        for _ in 0..depth {
+            html.push_str("</td></tr></table>");
+        }
+        html.push_str("</body></html>");
+        let wml = html_to_wml(&html);
+        assert_eq!(wml.matches("<w:tbl>").count(), MAX_NESTED_TABLES, "{wml}");
+        assert!(wml.contains(">deep</w:t>"), "{wml}");
+    }
+
     mod regression_tests {
         use super::*;
 
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
index d900e05..8555352 100644
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -13598,6 +13598,17 @@ fn table_block(
     // tcW 10080 cell over its four-column grid (gridSpan absent), not the
     // first 84pt column; 91 corpus documents carry such a row.
     stretch_stale_spans(&mut raw_rows, &cols);
+    // Budget the claims at row level too: every cell keeps the one column
+    // it is, but a row of padded spans cannot claim past the budget, so
+    // the column, preference and merge vectors stay proportional to the
+    // part instead of to a span's claimed width.
+    for row in &mut raw_rows {
+        let mut budget = MAX_GRID_COLUMNS;
+        for cell in row.iter_mut() {
+            cell.colspan = cell.colspan.min(budget.max(1)).max(1);
+            budget = budget.saturating_sub(cell.colspan);
+        }
+    }
     let mut occupancy = 0usize;
     let grid_len = cols.len();
     for row in &raw_rows {
@@ -14201,6 +14212,13 @@ fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
     }
 }
 
+/// The most grid columns one table's spans may claim. Word lays a table
+/// out in at most 63 columns; a span past the budget is a document's error
+/// or padding, not a wider table — the column, preference and merge
+/// vectors a span asked to size were gigabytes (a `gridSpan` of
+/// 4294967295 wanted 16 GiB of `f32` columns from one attribute).
+const MAX_GRID_COLUMNS: usize = 4096;
+
 fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
     let Some(pr) = first_named(dom, cell, "tcPr") else {
         return (1, VMerge::None);
@@ -14212,7 +14230,8 @@ fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|s| s.parse().ok())
         .unwrap_or(1)
-        .max(1);
+        .max(1)
+        .min(MAX_GRID_COLUMNS);
     let vmerge = match direct_named(dom, pr, "vMerge") {
         None => VMerge::None,
         Some(n) => match dom.attribute(n, &W::val()) {
@@ -14342,7 +14361,8 @@ fn row_grid_skip(dom: &Dom, row: NodeId, side: &str) -> Option<(usize, PrefWidth
     let span = direct_named(dom, pr, &format!("grid{side}"))
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|v| v.parse::<usize>().ok())
-        .filter(|n| *n > 0)?;
+        .filter(|n| *n > 0)
+        .map(|n| n.min(MAX_GRID_COLUMNS))?;
     let pref = direct_named(dom, pr, &format!("w{side}"))
         .filter(|n| attr_any(dom, *n, "type").is_none_or(|t| t == "dxa"))
         .and_then(|n| attr_any(dom, n, "w"))
@@ -62587,6 +62607,27 @@ mod complete_coordinate_layout_contract_tests {
         }
     }
 
+    #[test]
+    fn huge_spans_lay_out_a_bounded_grid_instead_of_allocating_it() {
+        // A `gridSpan`/`gridBefore` in the billions used to size the column
+        // vectors by the width it claimed (4294967295 → a 17 GiB `f32`
+        // grid): past the budget it is the document's error, and the table
+        // still lays out.
+        let Block::Table { geom, .. } = table(
+            "<w:tbl><w:tblGrid><w:gridCol w:w='1000'/></w:tblGrid>\
+             <w:tr><w:trPr><w:gridBefore w:val='2000000000'/></w:trPr>\
+             <w:tc><w:tcPr><w:gridSpan w:val='4294967295'/></w:tcPr><w:p/></w:tc></w:tr></w:tbl>",
+        )
+        else {
+            panic!("table fixture");
+        };
+        assert!(
+            geom.pref.len() <= MAX_GRID_COLUMNS + 2,
+            "a row's spanned width stays inside the column budget: {}",
+            geom.pref.len()
+        );
+    }
+
     #[test]
     fn content_autofit_skips_span_and_grid_before_placeholders_without_inventing_margins() {
         let fonts = fonts();
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
index 1881053..1abe6e7 100644
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -1592,32 +1592,39 @@ impl Story<'_> {
         let mut out = Vec::new();
         let mut spans: Vec<Span> = Vec::new();
         // Field nesting: true while inside a field's code (before its
-        // separator).
+        // separator). `open_codes` counts the code frames, so the check a
+        // character makes is O(1): a document can open fields without end
+        // and leave a long all-false stack each character would walk.
         let mut fields: Vec<bool> = Vec::new();
+        let mut open_codes = 0usize;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
         let mut cell: Vec<String> = Vec::new();
 
         for story in chars {
-            // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
             let text = match story.ch {
                 '\u{13}' => {
                     fields.push(true);
+                    open_codes = open_codes.saturating_add(1);
                     continue;
                 }
                 '\u{14}' => {
                     if let Some(top) = fields.last_mut() {
+                        open_codes = open_codes.saturating_sub(usize::from(*top));
                         *top = false;
                     }
                     continue;
                 }
                 '\u{15}' => {
-                    fields.pop();
+                    if fields.pop().is_some_and(|code| code) {
+                        open_codes = open_codes.saturating_sub(1);
+                    }
                     continue;
                 }
-                _ if in_code => continue,
+                // Inside any field's code, even a nested field's result is
+                // code.
+                _ if open_codes > 0 => continue,
                 '\u{0C}' if self.section_marks.binary_search(&story.cp).is_err() => {
                     // A page break: Markdown has no pages, so a line break.
                     '\n'
@@ -2817,6 +2824,80 @@ mod legacy_byte_source_boundary_tests {
         assert_eq!(word, [b'A', 0xb2, 0x03]);
     }
 
+    fn owned_story() -> (FkpIndex, FkpIndex, Lists) {
+        (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() }, Lists::default())
+    }
+
+    fn story_text(story: &Story<'_>, text: &str) -> String {
+        let chars = text
+            .chars()
+            .enumerate()
+            .map(|(i, ch)| StoryChar {
+                ch,
+                cp: i as u32,
+                fc: i as u32,
+                modifier: CharModifier::default(),
+            })
+            .collect::<Vec<_>>();
+        story
+            .blocks(&chars)
+            .iter()
+            .flat_map(|block| match block {
+                Block::Paragraph(paragraph) => paragraph
+                    .spans
+                    .iter()
+                    .map(|span| span.text.as_str())
+                    .collect::<Vec<_>>(),
+                _ => Vec::new(),
+            })
+            .collect()
+    }
+
+    #[test]
+    fn field_codes_stay_suppressed_and_results_stay_through_any_nesting() {
+        let (papx, chpx, lists) = owned_story();
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        assert_eq!(story_text(&story, "\u{13}CODE\u{14}RESULT\u{15}AFTER"), "RESULTAFTER");
+        // A nested field's result is code while an outer code is open; a
+        // field's code after its inner field closes stays code until its
+        // own separator, and only what no open code covers is text.
+        assert_eq!(story_text(&story, "\u{13}A\u{13}B\u{14}C\u{15}D\u{14}E\u{15}F"), "EF");
+    }
+
+    #[test]
+    fn an_all_false_field_stack_costs_no_per_character_walk() {
+        // Twenty thousand fields opened and separated leave a twenty
+        // thousand entry all-false stack; every character used to walk it
+        // end to end before it could prove no code was open.
+        let (papx, chpx, lists) = owned_story();
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        let mut text = String::with_capacity(40_000);
+        for _ in 0..20_000 {
+            text.push('\u{13}');
+            text.push('\u{14}');
+        }
+        for _ in 0..20_000 {
+            text.push('x');
+        }
+        assert_eq!(
+            story_text(&story, &text),
+            "x".repeat(20_000),
+            "separated fields suppress nothing"
+        );
+    }
+
     #[test]
     fn unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters() {
         let papx = FkpIndex { runs: Vec::new() };
diff --git a/src/markdown/from_docx/drawing.rs b/src/markdown/from_docx/drawing.rs
index a944602..ef54d3b 100644
--- a/src/markdown/from_docx/drawing.rs
+++ b/src/markdown/from_docx/drawing.rs
@@ -1886,12 +1886,27 @@ pub(crate) fn materialize(
     used: &mut HashSet<String>,
     next_doc_pr: &mut u32,
 ) -> Option<String> {
+    // A payload's graph may name a part that names it back, or a cycle of
+    // parts: the walk holds the indices it is inside (a revisit is a cycle,
+    // refused) and the names it has written (a shared subgraph is written
+    // once), so no relationship chain recurses without end.
+    let mut active: HashSet<usize> = HashSet::new();
+    let mut written: HashMap<usize, String> = HashMap::new();
     let mut xml = drawing.xml.clone();
     for rel in &drawing.rels {
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(index), _) if index < drawing.deps.len() => {
-                let name = dep_part(drawing, index, package, existing, used, next_doc_pr)?;
+                let name = dep_part(
+                    drawing,
+                    index,
+                    package,
+                    existing,
+                    used,
+                    next_doc_pr,
+                    &mut active,
+                    &mut written,
+                )?;
                 (rel.kind.clone(), relative_rel_target(part, &name), false)
             }
             (_, _, Some(raw)) => (rel.kind.clone(), relative_rel_target(part, raw), false),
@@ -1952,7 +1967,9 @@ fn renumber_doc_pr(xml: &mut String, next: &mut u32) {
 }
 
 /// Write dependent part `index` and its own dependency closure; its new
-/// part name.
+/// part name. `active` holds the indices the walk is inside (a revisit is
+/// a cycle, refused rather than recursed) and `written` the names already
+/// materialized, so a shared subgraph is written once.
 fn dep_part(
     drawing: &Drawing,
     index: usize,
@@ -1960,8 +1977,47 @@ fn dep_part(
     existing: &HashSet<String>,
     used: &mut HashSet<String>,
     next_doc_pr: &mut u32,
+    active: &mut HashSet<usize>,
+    written: &mut HashMap<usize, String>,
 ) -> Option<String> {
+    if let Some(name) = written.get(&index) {
+        return Some(name.clone());
+    }
     let dep = drawing.deps.get(index)?;
+    if !active.insert(index) {
+        return None;
+    }
+    let name = dep_closure(
+        drawing,
+        dep,
+        index,
+        package,
+        existing,
+        used,
+        next_doc_pr,
+        active,
+        written,
+    );
+    active.remove(&index);
+    if let Some(name) = &name {
+        written.insert(index, name.clone());
+    }
+    name
+}
+
+/// `dep`'s part and the dependency closure its relationships reach, called
+/// only with `index` inside `active`.
+fn dep_closure(
+    drawing: &Drawing,
+    dep: &Dep,
+    index: usize,
+    package: &mut PartFs,
+    existing: &HashSet<String>,
+    used: &mut HashSet<String>,
+    next_doc_pr: &mut u32,
+    active: &mut HashSet<usize>,
+    written: &mut HashMap<usize, String>,
+) -> Option<String> {
     let name = name_for(&dep.name, existing, used);
     used.insert(name.clone());
     // The part's own relationships name the parts the recursion is writing;
@@ -1971,7 +2027,16 @@ fn dep_part(
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(inner), _) if inner < drawing.deps.len() => {
-                let inner_name = dep_part(drawing, inner, package, existing, used, next_doc_pr)?;
+                let inner_name = dep_part(
+                    drawing,
+                    inner,
+                    package,
+                    existing,
+                    used,
+                    next_doc_pr,
+                    active,
+                    written,
+                )?;
                 (
                     rel.kind.clone(),
                     relative_rel_target(&name, &inner_name),
@@ -2036,6 +2101,104 @@ mod tests {
         parse_xml(xml.as_bytes()).unwrap()
     }
 
+    fn story_package() -> PartFs {
+        PartFs::open(include_bytes!(
+            "../../../tests/fixtures/redline/original.docx"
+        ))
+        .unwrap()
+    }
+
+    #[test]
+    fn a_dependency_cycle_is_refused_instead_of_recursing_without_end() {
+        // Part 0's only relationship names part 0: the materializer used to
+        // follow the chain forever.
+        let image = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
+        let drawing = Drawing {
+            rels: vec![PayRel {
+                sym: "r1".into(),
+                kind: image.into(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps: vec![Dep {
+                name: "word/media/self.png".into(),
+                kind: "image/png".into(),
+                xml: "<png/>".into(),
+                rels: vec![PayRel {
+                    sym: "d0.1".into(),
+                    kind: image.into(),
+                    part: Some(0),
+                    ..PayRel::default()
+                }],
+                ..Dep::default()
+            }],
+            ..Drawing::default()
+        };
+        let mut package = story_package();
+        let existing: HashSet<String> = package.parts().into_iter().collect();
+        let mut used = HashSet::new();
+        let mut next = 1u32;
+        assert!(
+            materialize(
+                &drawing,
+                &mut package,
+                "word/document.xml",
+                &existing,
+                &mut used,
+                &mut next
+            )
+            .is_none(),
+            "a cyclic dependency graph is refused, not materialized"
+        );
+    }
+
+    #[test]
+    fn a_directed_dependency_closure_still_materializes() {
+        let image = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
+        let drawing = Drawing {
+            xml: "<w:drawing/>".into(),
+            rels: vec![
+                PayRel {
+                    sym: "r1".into(),
+                    kind: image.into(),
+                    part: Some(0),
+                    ..PayRel::default()
+                },
+                PayRel {
+                    sym: "r2".into(),
+                    kind: image.into(),
+                    part: Some(0),
+                    ..PayRel::default()
+                },
+            ],
+            deps: vec![Dep {
+                name: "word/media/shared.png".into(),
+                kind: "image/png".into(),
+                xml: "<png/>".into(),
+                ..Dep::default()
+            }],
+            ..Drawing::default()
+        };
+        let mut package = story_package();
+        let existing: HashSet<String> = package.parts().into_iter().collect();
+        let mut used = HashSet::new();
+        let mut next = 1u32;
+        let xml = materialize(
+            &drawing,
+            &mut package,
+            "word/document.xml",
+            &existing,
+            &mut used,
+            &mut next,
+        )
+        .expect("an acyclic closure materializes");
+        assert!(xml.contains("<w:drawing/>"), "{xml}");
+        assert!(
+            package.part_bytes("word/media/shared.png").is_some(),
+            "the shared part is written once"
+        );
+    }
+
     #[test]
     fn xml_round_trips_verbatim() {
         let xml = r#"<wp:anchor xmlns:wp="wp" wp14:anchorId="AB"><wp:docPr id="2" name="P"/><x:y a="1&quot;2"><t>x &amp; y</t></x:y></wp:anchor>"#;
diff --git a/src/scrub.rs b/src/scrub.rs
index 641d427..76b1a3d 100644
--- a/src/scrub.rs
+++ b/src/scrub.rs
@@ -387,7 +387,10 @@ fn scrub_properties(pkg: &mut PartFs) {
 /// The parts of `docx` where `needle` still occurs, sorted: in a text node,
 /// an attribute value that can hold text, an XML comment, the joined run
 /// text of a paragraph (a text split across runs), or the raw bytes of a
-/// part that is not XML (UTF-8 or UTF-16LE). An empty needle occurs nowhere.
+/// part that is not XML (UTF-8, UTF-16LE or UTF-16BE). XML parts are decoded
+/// with admission's BOM-aware policy first, so a UTF-16 copy is scanned as
+/// the text it holds, not as NUL-interleaved bytes. An empty needle occurs
+/// nowhere.
 /// An unreadable archive or entry is a finding: an incomplete scan cannot
 /// certify a redaction.
 ///
@@ -422,7 +425,8 @@ pub(crate) fn leaks_within(
         Ok(zip) => zip,
         Err(error) => return vec![format!("the package (cannot read: {error})")],
     };
-    let utf16: Vec<u8> = needle.encode_utf16().flat_map(u16::to_le_bytes).collect();
+    let utf16le: Vec<u8> = needle.encode_utf16().flat_map(u16::to_le_bytes).collect();
+    let utf16be: Vec<u8> = needle.encode_utf16().flat_map(u16::to_be_bytes).collect();
     let mut total = 0u64;
     for i in 0..zip.len() {
         let mut entry = match zip.by_index(i) {
@@ -466,9 +470,14 @@ pub(crate) fn leaks_within(
         }
         let xml = name.ends_with(".xml") || name.ends_with(".rels");
         let hit = if xml {
-            xml_holds(&String::from_utf8_lossy(&bytes), needle)
+            // BOM-aware, as admission reads the part: a UTF-16 XML part's
+            // lossy UTF-8 reading interleaves NULs, and the copy it still
+            // holds would otherwise certify a redaction that kept it.
+            xml_holds(&crate::admission::part_text(&bytes), needle)
         } else {
-            contains(&bytes, needle.as_bytes()) || contains(&bytes, &utf16)
+            contains(&bytes, needle.as_bytes())
+                || contains(&bytes, &utf16le)
+                || contains(&bytes, &utf16be)
         };
         if hit {
             found.insert(name);
@@ -716,6 +725,44 @@ mod public_memory_leak_boundary_tests {
         }
     }
     #[test]
+    fn utf16_xml_and_utf16be_parts_hold_the_secret_like_any_other_copy() {
+        let secret = "PrivateReviewer";
+        let mut pkg = package("<w:p><w:r><w:t>Independent body payload</w:t></w:r></w:p>");
+        let item = format!("<metadata><!-- {secret} --><note who='{secret}'/></metadata>");
+        // A BOM-marked UTF-16 XML part, as SharePoint writes customXml items.
+        let utf16le = [0xFF, 0xFE]
+            .into_iter()
+            .chain(item.encode_utf16().flat_map(u16::to_le_bytes))
+            .collect::<Vec<u8>>();
+        let utf16be = [0xFE, 0xFF]
+            .into_iter()
+            .chain(item.encode_utf16().flat_map(u16::to_be_bytes))
+            .collect::<Vec<u8>>();
+        // Not XML once decoded: the no-root fallback has to find it too.
+        let bare = [0xFF, 0xFE]
+            .into_iter()
+            .chain(secret.encode_utf16().flat_map(u16::to_le_bytes))
+            .collect::<Vec<u8>>();
+        pkg.set_part("customXml/item1.xml", utf16le);
+        pkg.set_part("customXml/item2.xml", utf16be);
+        pkg.set_part("customXml/item3.xml", bare);
+        pkg.set_part(
+            "customXml/utf16be.bin",
+            secret.encode_utf16().flat_map(u16::to_be_bytes).collect(),
+        );
+        pkg.add_content_type_override("/customXml/utf16be.bin", "application/octet-stream");
+        let bytes = pkg.to_zip().unwrap();
+        assert_eq!(
+            leaks(&bytes, secret),
+            [
+                "customXml/item1.xml",
+                "customXml/item2.xml",
+                "customXml/item3.xml",
+                "customXml/utf16be.bin"
+            ]
+        );
+    }
+    #[test]
     fn decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not() {
         let body = "<w:p><w:pPr><w:spacing w:before='1234' w:after='-1234'/></w:pPr><w:r><w:t>Jane </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>Doe &amp; </w:t></w:r><w:r><w:t>Associates</w:t></w:r></w:p>";
         let mut pkg = package(body);
diff --git a/src/strict_translation.rs b/src/strict_translation.rs
index 38bb072..b538d86 100644
--- a/src/strict_translation.rs
+++ b/src/strict_translation.rs
@@ -663,10 +663,21 @@ fn replace_all(text: &str, table: &[(&str, &str)]) -> String {
 /// part Strict rather than Transitional.
 const STRICT_MAIN: &str = "http://purl.oclc.org/ooxml/wordprocessingml/main";
 
+/// The most bytes of a main part's head the probe retains while it waits
+/// for the root element's opening tag to close (its `<` may sit at offset
+/// zero, so no later `<` ever trims it). No Word document's preamble or
+/// root tag is this long; past the budget the probe answers undecided —
+/// not Strict — instead of retaining and rescanning a head that grows
+/// with the part.
+const MAX_ROOT_HEAD_BYTES: usize = 1024 * 1024;
+
 /// Whether the package's main document part (`word/document.xml`) is ISO
 /// Strict — decided by the namespace its root element declares, the same
 /// fact Word reads the dialect from. A Transitional package that merely
 /// *quotes* a Strict URI in its text is not a Strict package (#531).
+/// A part past the probe's head or part budgets answers `false` as well,
+/// the same answer a malformed part gets: an endless preamble is
+/// undecided, not Strict.
 pub fn main_part_is_strict(bytes: &[u8]) -> bool {
     let Ok(mut archive) = crate::opc::open_archive(Cursor::new(bytes)) else {
         return false;
@@ -680,6 +691,7 @@ pub fn main_part_is_strict(bytes: &[u8]) -> bool {
     // preamble can push the root past (#561). Only the tail since the
     // last `<` can still hold the tag, so the head stays small.
     let mut head: Vec<u8> = Vec::new();
+    let mut read_total: u64 = 0;
     let mut chunk = vec![0u8; 16 * 1024];
     loop {
         let Ok(read) = part.read(&mut chunk) else {
@@ -688,6 +700,13 @@ pub fn main_part_is_strict(bytes: &[u8]) -> bool {
         if read == 0 {
             break;
         }
+        read_total = read_total.saturating_add(read as u64);
+        if read_total > InputLimits::default().max_part_bytes {
+            // A main part past the product's part budget is not decided
+            // by this probe; the admission that allows one refuses the
+            // same shape elsewhere.
+            return false;
+        }
         head.extend(chunk.iter().copied().take(read));
         let window = String::from_utf8_lossy(&head);
         if let Some((start, _)) = root_open(&window)
@@ -701,6 +720,9 @@ pub fn main_part_is_strict(bytes: &[u8]) -> bool {
             }
             None => head.clear(),
         }
+        if head.len() > MAX_ROOT_HEAD_BYTES {
+            return false;
+        }
     }
     let head = String::from_utf8_lossy(&head);
     let Some((start, _)) = root_open(&head) else {
@@ -1150,6 +1172,48 @@ mod tests {
         )));
     }
 
+    /// A main part that never closes its root opening tag is refused once
+    /// past the probe's head budget. Before the budget the whole endless
+    /// head was retained and rescanned — and a Strict URI inside it read
+    /// as Strict.
+    #[test]
+    fn an_endless_root_head_is_refused_as_undecided() {
+        let doc = format!(
+            "<w:document xmlns:w=\"http://purl.oclc.org/ooxml/wordprocessingml/main\" {}",
+            "x".repeat(2 * 1024 * 1024)
+        );
+        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
+        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
+        writer.start_file("word/document.xml", options).unwrap();
+        writer.write_all(doc.as_bytes()).unwrap();
+        let pkg = writer.finish().unwrap().into_inner();
+        assert!(
+            !main_part_is_strict(&pkg),
+            "an endless root head is undecided, not Strict"
+        );
+    }
+
+    /// A truncated root tag inside the budget still decides from the
+    /// declarations it holds: the truncation semantics stay.
+    #[test]
+    fn a_truncated_root_head_inside_the_budget_still_decides() {
+        let package = |uri: &str| {
+            let doc = format!(r#"<w:document xmlns:w="{uri}" "#);
+            let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
+            let options =
+                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
+            writer.start_file("word/document.xml", options).unwrap();
+            writer.write_all(doc.as_bytes()).unwrap();
+            writer.finish().unwrap().into_inner()
+        };
+        assert!(main_part_is_strict(&package(
+            "http://purl.oclc.org/ooxml/wordprocessingml/main"
+        )));
+        assert!(!main_part_is_strict(&package(
+            "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
+        )));
+    }
+
     /// A Transitional package is returned byte-for-byte unchanged (zero-churn).
     #[test]
     fn transitional_package_returned_byte_identical() {

```
