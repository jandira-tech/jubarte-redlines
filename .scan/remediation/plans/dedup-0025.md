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
