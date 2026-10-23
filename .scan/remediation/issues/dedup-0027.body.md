@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_9e40f23e742aa43493a0fb92 | Rendering a small DOCX table can exhaust host memory | medium
csf_8dcaf50784fd95a7f0c2ec28 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_0e2b085a251b34412cd71a4c | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_64b247931cd1f053217fde7a | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_d22452ac5ec901578d340334 | Opaque DTD and top-level CDATA bypass package XML depth checks | medium
csf_68acfaf376a60cf8c3d8e2d4 | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion | medium
csf_f756bbe5509c42bdcaa64921 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_88d031991ca3e20e4aae2f24 | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_32625277093e7f286a267e2b | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.

## TDD red/green plan
# dedup-0027 remediation plan (TDD red/green)

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0027/`
(`result.json`, 61 merged findings). This round's own contributions are the
findings whose code evidence ids start with `dedup0027` — six findings,
extracted with:

```
jq '[.findings[] | select(.codeEvidence != null and ([.codeEvidence[] | (.id // "")] | any(startswith("dedup0027")))) | …]' result.json
```

All six are fixed by this patch. Every claim below was verified by reading the
named source lines in this session; nothing was compiled or run (per the ask:
no `cargo`/`mbx`/`rustc` in this checkout), so the red/green runs themselves
are **not run** — the patch authors them for the next build. The patch was
checked mechanically with `git apply --check` (exit 0, clean tree).

| # | ruleId | severity | Fix |
|---|--------|----------|-----|
| 1 | `resource-exhaustion.table-grid-span` | medium | grid-span budgets before any table vector is sized (`src/convert/mod.rs`) |
| 2 | `algorithmic-complexity.legacy-doc-field-stack` | medium | O(1) active-code-frame count (`src/legacy_doc.rs`) |
| 3 | `unbounded-expansion.altchunk-colspan` | medium | colspan budgets in HTML-chunk table generation (`src/convert/altchunk.rs`) |
| 4 | `uncontrolled-recursion.markdown-drawing-dependencies` | medium | cycle + work guard in drawing materialization (`src/markdown/from_docx/drawing.rs`) |
| 5 | `xml-depth.dtd-parser-differential` | medium | consume `<!` constructs by complete grammar (`src/xmllinq/parse.rs`) |
| 6 | `uncontrolled-recursion.unchecked-auxiliary-xml` (scoped) | medium | nesting budget in the shared permissive DOM parser (`src/xmllinq/parse.rs`) |

Common design rule: every budget sits far above anything Microsoft Word
writes (Word's table grid tops out at 63 columns; admission caps XML nesting
at 256), so no Word-valid, Word-faithful layout changes. Where Word itself
cannot produce the input (a 4-billion-column `gridSpan`, a 50 000-deep
renamed part, a cyclic drawing payload), the bounded path replaces a crash
or a hang.

---

## 1. `resource-exhaustion.table-grid-span` — huge `w:gridSpan` / `w:gridBefore` / `w:gridAfter`

**Vulnerability.** `cell_span` (`src/convert/mod.rs:14204-14224`) parses
`w:gridSpan` as `usize` with only `.max(1)`; `row_grid_skip`
(`src/convert/mod.rs:14340-14353`) accepts any positive `usize` for
`w:gridBefore`/`w:gridAfter`. `table_block` sums each row's colspans into
`occupancy` (`src/convert/mod.rs:13601-13616`) and sizes vectors from it:
`cols = vec![each; occupancy]`, `cols.resize(occupancy.max(1), 80.0)`,
`column_prefs` (`:14263` `vec![PrefWidth::Auto; grid.len()]`), and
`resolve_table_merges` (`:14637-14639` `open.resize(col + span, None)`).
`gridSpan=1000000000` asks for ~4 GiB of `f32` columns from ten digits of
XML; many individually valid spans summed overflow `usize` on wasm32
(`sum()` at `:13604` and `:14175`).

**Fix (minimal).** Two crate-visible budgets in `src/convert/mod.rs`
(`MAX_GRID_SPAN = 1024`, `MAX_ROW_GRID_COLUMNS = 4096` — Word's own limit is
63) and one sanitization pass over `raw_rows` inserted immediately before
`stretch_stale_spans(&mut raw_rows, &cols)` (`src/convert/mod.rs:13600`),
the single point every downstream consumer (`stretch_stale_spans`' `sum()`,
occupancy, `column_prefs`, `resolve_table_merges`) reads after:

- each cell's colspan clamped to `min(cell.colspan.max(1), MAX_GRID_SPAN,
  remaining row budget)` with a saturating running total (checked on
  wasm32 by construction: `col ≤ 4096`, `span ≤ 1024`);
- cells past the row budget are dropped (`retain_mut`).

Refusal-instead-of-clamp was rejected: `table_block` returns `Block`, is
recursive (nested tables, `src/convert/mod.rs:13232`) and has eight call
sites; threading `Result` through the block pipeline is not a minimal
change, and clamping at 16×/64× Word's limit changes no Word-written
document. `w:tblGrid` (`:13111-13123`) is left alone: its length is linear
in admitted part bytes, already bounded by admission.

**Failing tests (red)** — `src/convert/mod.rs`, module `table_tests`
(beside `table_block`, after `parse_grid_table4_reads_firstrow_white`,
ends `:45304`). New helper `laid_out_table(rows_xml)` reuses the module's
existing `Dom`/`StyleSheet`/`Numbering` boilerplate (pattern of
`nested_table_rows_are_not_flattened`, `:44776-44830`):

- `a_huge_grid_span_sizes_a_bounded_grid_not_gigabytes` — one
  `<w:tc><w:tcPr><w:gridSpan w:val="4294967295"/></w:tcPr><w:p/></w:tc>`
  row. Asserts `cols.len() <= MAX_ROW_GRID_COLUMNS` and
  `rows[0][0].colspan == MAX_GRID_SPAN`. Red today: allocation/OOM before
  any assert.
- `huge_grid_before_and_after_are_clamped_like_spans` —
  `<w:trPr><w:gridBefore w:val="1000000000"/></w:trPr>`; asserts the
  leading grid-skip cell keeps `grid_skip` and clamped `colspan`, and the
  grid stays within the row budget.
- `a_row_whose_spans_sum_past_the_column_budget_is_bounded` — eight cells
  each `gridSpan=1024` (sum 8192); asserts `cols.len() ==
  MAX_ROW_GRID_COLUMNS` and only four cells survive (checked cumulative
  accounting, no `sum()` overflow).
- `ordinary_merged_and_skipped_columns_still_lay_out` (keep-passing) — a
  4-column `w:tblGrid`, one row with `gridBefore=1`, a `gridSpan=2` cell
  and a `gridSpan=1` cell; asserts `cols.len() == 4`, `cols[0] == 200.0`
  (4000 twips), and each `TableCell`'s `col`/`colspan`/`grid_skip` exactly
  as the current code lays them out.

**Existing behaviour that must keep passing** — every other test in
`table_tests` (incl. `nested_table_rows_are_not_flattened`), the
`stretch_stale_spans` unit tests (`src/convert/mod.rs:48604-48614`) and
`resolve_table_merges` tests (`:48533-48604`), and the corpus/fidelity PDF
tests (no fixture carries spans ≥ 1024; verified by grep — the only
multi-digit `gridSpan` in tests is `val="2"`).

---

## 2. `algorithmic-complexity.legacy-doc-field-stack` — quadratic field-stack scan

**Vulnerability.** `Story::blocks` (`src/legacy_doc.rs:1591-1676`) computes
`let in_code = fields.iter().any(|&code| code);` before **every** character
(`:1604`). `.any` short-circuits on `true`, but a stream of
begin+separator pairs (`\u{13}\u{14}`, never ended) leaves an all-**false**
stack, so every following character scans the whole stack: O(N²) on a Word
97-2003 `.doc` read or conversion — including the CLI's refusal-hint probe
`jubarte::legacy_doc::read(&bytes).is_ok()` (`src/bin/jubarte.rs:2745`),
which runs the full reader just to print a hint.

**Fix (minimal).** Keep the `fields` stack, add `code_fields: usize`
counting frames still in their code, updated at the three arms
(`src/legacy_doc.rs:1606-1619`): begin `+= 1`; separator decrements only
when the top frame was `true` (it is set false exactly then); end decrements
when the popped frame was `true`. `in_code` becomes `code_fields > 0` —
identical truth value ("any frame is code") in O(1).

**Failing tests (red)** — `src/legacy_doc.rs`, module `tests`, after
`a_field_nested_in_a_field_code_shows_only_the_outer_result` (`:1945-1968`),
reusing its `story_chars`/`Story` pattern:

- `many_separated_fields_that_never_end_stay_linear` — 200 000 ×
  `\u{13}\u{14}` then `"shown\r"`; asserts the paragraph text is `shown`
  (separated results display) and, in the house timing pattern of
  `a_self_linked_difat_sector_is_bounded_by_the_file` (`:1861-1882`),
  `elapsed < 2 s`. Red today: ≈4×10¹⁰ stack comparisons.
- `separated_nested_fields_count_down_and_up_correctly` —
  `"\u{13}c\u{14}\u{13}\u{14}d\u{15}e\u{15}f\r"` asserts text `def`: an
  inner field's result inside an outer *result* displays; nested code
  suppression still holds (counter decrements and re-increments).

**Existing behaviour that must keep passing** —
`a_field_nested_in_a_field_code_shows_only_the_outer_result` (`:1945`),
`a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it`
(`:1919`), `tests/legacy_doc.rs`, `tests/legacy_doc_convert.rs`.

---

## 3. `unbounded-expansion.altchunk-colspan` — one HTML `colspan` → billions of grid elements

**Vulnerability.** `Builder::start` "td"/"th" pushes
`attr(open, "colspan").parse::<usize>().max(1)` unbounded
(`src/convert/altchunk.rs:562-567`); `end_table` sums a row's spans into
`cols` (`:469-475`) and then writes one `<w:gridCol/>` **element per
column** (`:489-491`). `colspan=4294967295` generates >85 GB of WML after
the original package was admitted and before any DOM/depth check.

**Fix (minimal).** Clamp the parsed colspan with `.min(MAX_GRID_SPAN)` and
compute `cols` with `try_fold(checked_add)` capped at
`MAX_ROW_GRID_COLUMNS` — the *same* budgets as finding 1 (imported from
`super::`), so the altChunk path and the DOCX table path agree. The
generated table then feeds the (now budgeted) `table_block` pipeline.

**Failing tests (red)** — `src/convert/altchunk.rs`, module `tests`, after
`an_mht_chunk_yields_its_quoted_printable_html` (`:810-818`):

- `a_huge_colspan_is_clamped_before_the_grid_is_generated` —
  `<td colspan="4294967295">`; asserts `<w:gridCol` count ==
  `MAX_GRID_SPAN` and `<w:gridSpan w:val="1024"/>`. Red today: the count is
  4 294 967 295 (hang/OOM).
- `a_row_of_valid_colspans_summing_past_the_budget_is_clamped` — eight
  `colspan="1024"` cells; asserts the grid holds at
  `MAX_ROW_GRID_COLUMNS` (checked sum, no overflow).
- `ordinary_colspans_still_generate_their_grid` (keep-passing) —
  `colspan="2"` + plain cell; asserts 3 `<w:gridCol` and the
  `<w:gridSpan w:val="2"/>` markup unchanged.

**Existing behaviour that must keep passing** —
`omitted_cell_and_row_end_tags_keep_the_table` (`:875`),
`an_alt_chunk_is_replaced_by_its_paragraphs` (`:793`) and the rest of the
altchunk test modules.

---

## 4. `uncontrolled-recursion.markdown-drawing-dependencies` — cyclic `drawing:` payload recurses forever

**Vulnerability.** A `![label](drawing:PAYLOAD)` destination decodes into a
`Drawing` dependency graph (`decode`, `src/markdown/from_docx/drawing.rs:1397`).
`materialize` (`:1881-1905`) → `dep_part` (`:1956-2008`) recursively
materializes each `rel.part` index (`:1974`) with **no active-node map**: a
one-node self-reference recurses until the stack ends, reached from body
drawings (`src/markdown/package.rs:1740`) and header/footer stories
(`src/markdown/view_stories.rs:1542`) alike. Capture-side limits
(`MAX_DEPS = 64`, `MAX_DEP_RELS = 64`, `:61-66`) bound the graph's size,
not its shape; a densely shared DAG still re-expands one subgraph per path
that reaches it.

**Fix (minimal).** A `Traversal { active: HashSet<usize>, steps: usize }`
created **inside** `materialize` (signature unchanged, both call sites
untouched) and threaded through `dep_part`, which becomes a guard around
the old body (`dep_part_closure`): a revisit of an active index (cycle)
returns `None` — the drawing degrades to alt text exactly as any other
unreadable payload — and a `steps` budget of
`MAX_MATERIALIZE_STEPS = MAX_DEPS * MAX_DEP_RELS` (4096) stops DAG
re-expansion. Index→name memoization (also suggested by the finding) was
deliberately **not** done: it changes the bytes of packages that share a
dep across paths (each path currently writes its own `-j2` part), which
needs corpus verification this no-build round cannot do.

**Failing tests (red)** — `src/markdown/from_docx/drawing.rs`, module
`tests` (appended after `doc_pr_ids_normalize_and_renumber`, file end; uses
`super::*` so the private `Drawing`/`PayRel`/`Dep` literals are in scope,
and `PartFs::open(include_bytes!("../../../tests/fixtures/redline/original.docx"))`
after the pattern of `src/markdown/package.rs:1987`):

- `a_cyclic_dependency_is_refused_not_recursed` — one dep whose own rel
  names index 0; asserts `materialize(...).is_none()` and no part written.
  Red today: stack overflow inside the test.
- `an_acyclic_dependency_chain_still_materializes` (keep-passing) —
  `xml: "<drawing r:embed=\"r1\"/>"`, one image dep (`b64: "aGk="`);
  asserts `Some` output with the symbol rewritten to an `rId` and
  `media/ok.png` written.
- `a_dependency_graph_that_re_expands_past_the_budget_is_refused` — 64
  deps where each names the next **twice** (2⁶³ paths); asserts `None`.
  Red today: effectively unbounded work.

**Existing behaviour that must keep passing** — the `xml_round_trips`,
`declares_prefixes_the_tree_uses`, `rel_names_match_only_the_relationship_namespace`
tests (`:2039-2074`), `unchanged_blank_box_keeps_controls_and_remains_editable`,
`doc_pr_ids_normalize_and_renumber`, and the markdown↔docx round-trip suite
(`tests/docx_markdown_round_trip.rs`, whose real payloads are DAGs well
under 4096 materializations).

---

## 5. `xml-depth.dtd-parser-differential` — opaque DTD / top-level CDATA split at the first `>`

**Vulnerability.** Package admission (`check_xml_depth`,
`src/admission.rs:788-823`) ignores `DocType`/`CData` events (`:819`
`Ok(_) => {}`), and quick-xml 0.42 ends a DOCTYPE at the grammar's real
close (verified in the vendored crate source:
`~/.cargo/registry/src/*/quick-xml-0.42.0/src/parser/dtd.rs` tracks quotes
and `[...]` internal subsets). The permissive DOM parser instead skips any
top-level `<!` to the **first** `>` (`src/xmllinq/parse.rs:539-544`). A
quoted `>` — e.g. `<!DOCTYPE d [<!ENTITY e "]>` + deep markup + `">]>`, or
a top-level `<![CDATA[ > <a>…]]>` — leaves the deep markup as residue the
parser reads as top-level elements that admission never depth-counted;
recursive `parse_element` then exhausts the stack. Consumers include
rendering and lazy `WmlDocument` main-part access.

**Fix (minimal).** Replace the first-`>` skip with a
`bang_construct_end()` scan that ends the construct where quick-xml does:
`<![CDATA[` at its `]]>`, any other declaration at the `>` outside quoted
strings and outside the internal subset's brackets. In-root CDATA
(`parse_element`, `:645-654`) is untouched. This is the finding's
"align lexical semantics / consume the complete grammar" branch; a DTD-only
refusal would not fix the CDATA subcase.

**Failing tests (red)** — `src/xmllinq/parse.rs`, module
`xml_lexical_boundary_contract_tests` (which glob-imports the parser
module; tests appended after
`legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata`,
module close at `:1182`):

- `a_doctype_with_a_quoted_close_is_consumed_whole` —
  `<!DOCTYPE d [<!ENTITY e "]>` + 100 000 nested `<a>` + `">]><p/>`;
  asserts the document's root is `p`. Red today: the residue parses as
  100 000-deep elements — stack overflow (crash) before any root exists.
- `a_top_level_cdata_section_with_a_close_inside_is_consumed_whole` —
  `<![CDATA[ > <x/> ]]><p>t</p>`; asserts root `p` with value `t`.
  Red today: root is the phantom `x`.

**Existing behaviour that must keep passing** —
`literal_text_end_of_line_reads_as_lf_references_survive` (`:1097`,
in-root CDATA), `legacy_declaration_defaults_keep_readable_roots…`
(`:1145`), `validate_xml`'s own DTD refusal (`parse.rs:954`), and
`tag_policy_tests`.

---

## 6. `uncontrolled-recursion.unchecked-auxiliary-xml` — **scoped fix**: nesting budget at the shared parser sink

**Vulnerability (this round's evidence `dedup0027-n1-*`).** Admission
depth-checks only `.xml`/`.rels` parts (`src/admission.rs:533-543`); the
main part alone gets the renamed-main fallback (`:556-570`). A renamed
auxiliary part (`word/header1.bin`) holds arbitrary bytes that admission
only drains and tallies. Revision acceptance/rejection selects parts by
**relationship** (`revision_bearing_parts`,
`src/revision_processor.rs:3689-3730`), then `RevisionIds::new` (`:3784`)
and `process_part` (`:3732-3740`) build a DOM with the permissive
`dom.parse_xdocument` — this round's sink evidence
(`src/xmllinq/parse.rs:660-663`: recursion with no depth guard). Deeply
nested XML under the `.bin` name overflows the stack in the public
`accept_revisions`/`reject_revisions` path
(`src/document_comparer.rs:4961-4988`, admitted first). The full finding
lists many more consumers (theme/styles/numbering/fontTable/settings/
comments/notes/headers/footers/linked-textboxes/diagrams/charts, plus
Python/WASM surfaces) — all of them, and the 551 `parse_xdocument` call
sites repo-wide, share this one recursive sink.

**Fix (scoped, centralized).** `MAX_PARSE_DEPTH = 1024` in
`src/xmllinq/parse.rs` (4× admission's 256; above every legitimate nesting
in the tree — the markdown reader's own stack caps at 512,
`src/markdown/from_docx/ooxml.rs:29`, and the parser's deepest test uses
64). `Parser` gains a `depth` field; `parse_element` wraps its body
(`parse_element_inner`) with increment/decrement, and the child-loop's `<`
branch, at the budget, consumes the tag via a quote-aware `skip_tag()` and
continues instead of recursing (every iteration advances `pos`, so no
spin). Over-budget nesting flattens into the deepest kept element; at or
under budget the DOM is exactly what it is today. This implements the
finding's "enforce active depth during DOM construction **or** use the
checked API" at the one sink every listed consumer shares, so a renamed
part can no longer exhaust the host stack anywhere the permissive parser
is reached.

**Honest scope note (not fixed this round).** Renamed auxiliary parts are
still not run through `check_xml_depth`/tag policy at admission, and
consumers that parse with quick-xml directly are unaffected. Full
filename-independent XML admission per the finding's remediation remains
open; this round removes the reachable stack exhaustion at the shared
sink, which is what this round's own evidence
(`dedup0027-n1-recursive-parser`) established.

**Failing tests (red)**:

- `src/xmllinq/parse.rs`, module `xml_lexical_boundary_contract_tests`:
  - `nesting_within_the_parse_budget_builds_to_full_depth` — exactly
    `MAX_PARSE_DEPTH` nested `<e>`; asserts the chain is complete
    (keep-passing guard for the budget's ceiling).
  - `nesting_past_the_parse_budget_flattens_instead_of_overflowing` —
    50 000 nested `<e>` parsed on a 2 MiB spawned thread; asserts the
    join succeeds and the built chain stops at `MAX_PARSE_DEPTH`.
    Red today: the thread dies on guard-page overflow.
- `src/revision_processor.rs`, new module `deep_renamed_part_tests` at
  end of file (package built with the house `PartFs` pattern of
  `public_block_control_ownership_tests::package`, `:4798-4819`, plus a
  `header` relationship targeting `word/header1.bin` holding 50 000-deep
  XML):
  - `accept_revisions_survives_a_deeply_nested_renamed_header` — runs
    `crate::document_comparer::accept_revisions` on a 2 MiB thread;
    asserts it returns `Ok` and the output still opens with its main
    part. Red today: admission passes (`.bin` is not XML-depth-checked)
    and revision processing overflows the stack.

**Existing behaviour that must keep passing** — every `parse_xdocument`
consumer test in `src/xmllinq/parse.rs` (incl.
`deep_rebinding_scope_grows_only_with_declared_bytes`, depth 64, `:1571`),
`checked_xml_tests`, `tag_policy_tests`, and the repository's corpus of
admitted documents (all ≤ 256 deep by admission).

---

## What the patch touches

```
src/convert/mod.rs            budgets + sanitization pass + 4 tests (table_tests)
src/convert/altchunk.rs       colspan clamp + checked grid sum + 3 tests
src/legacy_doc.rs             O(1) code-frame count + 2 tests
src/markdown/from_docx/drawing.rs  Traversal guard (cycle + steps) + 3 tests
src/xmllinq/parse.rs          bang_construct_end + MAX_PARSE_DEPTH guard + 4 tests
src/revision_processor.rs     1 test (deep renamed header through accept_revisions)
```

No behaviour changes for Word-written documents: all budgets exceed Word's
own limits (63 table columns; nesting ≤ 256 by admission), verified by
reading the code paths and grepping the test corpus for larger values
(none found). No `#[allow]` suppressions are introduced; no import was
added anywhere it would sit unused (the parse tests live under a
`use super::*` module).

## Verification still to run (not run in this no-build round)

Per AGENTS.md, on the next free checkout:

```
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo nextest run -E 'test(table_tests) + test(altchunk) + test(legacy_doc) + test(drawing) + test(lexical_boundary) + test(deep_renamed_part_tests)'
cargo nextest run --test legacy_doc --test legacy_doc_convert --test docx_markdown_round_trip
```

plus a CLI `--help` smoke test and the fidelity gates before any
performance claim. The patch file is
`.scan/remediation/patches/dedup-0027.patch` (906 lines, 6 files); it
applies from the repository root with `git apply` (verified with
`git apply --check`, exit 0).

## Note on regenerated artifacts

An external process emptied `.scan/remediation/` mid-task (observed at
17:00, after the first plan and patch had been written and checked); both
deliverables were regenerated from the same deterministic edits and
re-checked. Scratch work now lives outside the repository.


## Patch
```diff
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
index d900e05..bbb5727 100644
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -13090,6 +13090,21 @@ fn apply_tbl_style(rows: &mut [Vec<TableCell>], tdef: &TblStyle, look: &TblLook)
     }
 }
 
+/// The most grid columns one `w:gridSpan`, `w:gridBefore` or `w:gridAfter`
+/// count may occupy: attacker-typed attribute text that directly sizes the
+/// table's column, preference and merge vectors. Word's own grid tops out
+/// at 63 columns (Insert Table's limit), so a document past this budget is
+/// not Word-written; its span is clamped to the budget before anything
+/// allocates from it.
+const MAX_GRID_SPAN: usize = 1024;
+
+/// Grid columns one table row may occupy, every cell's span and skip
+/// summed: a row of many individually valid spans can still sum past any
+/// per-span budget, and the total sizes the same vectors a single span
+/// does. The running total is saturating, so the sum cannot overflow
+/// (32-bit WASM included).
+const MAX_ROW_GRID_COLUMNS: usize = 4096;
+
 /// `media` is the package and part whose rels resolve inline cell
 /// pictures (`None` where they cannot: endnotes, unit tests).
 fn table_block(
@@ -13597,6 +13612,25 @@ fn table_block(
     // spans the columns its width covers: Word lays 25f1d311bd's lone
     // tcW 10080 cell over its four-column grid (gridSpan absent), not the
     // first 84pt column; 91 corpus documents carry such a row.
+    // The spans above are counts an untrusted author types as attribute
+    // text, and everything downstream sizes a vector from them: occupancy
+    // resizes `cols`, `column_prefs` walks span columns, and merge
+    // resolution resizes its open-slot vector. Word's own grid tops out
+    // at 63 columns (Insert Table's limit), so budgets far above that
+    // keep every Word-written layout while no single span — and no row's
+    // spans summed — can ask for those vectors' unbounded sizes.
+    for row in &mut raw_rows {
+        let mut col = 0usize;
+        row.retain_mut(|cell| {
+            let room = MAX_ROW_GRID_COLUMNS.saturating_sub(col);
+            if room == 0 {
+                return false;
+            }
+            cell.colspan = cell.colspan.max(1).min(MAX_GRID_SPAN).min(room);
+            col += cell.colspan;
+            true
+        });
+    }
     stretch_stale_spans(&mut raw_rows, &cols);
     let mut occupancy = 0usize;
     let grid_len = cols.len();
@@ -45301,6 +45335,114 @@ mod table_tests {
         assert!((color[2] - 1.0).abs() < 0.01);
         assert!(parsed.first_row_bold);
     }
+
+    fn laid_out_table(rows_xml: &str) -> Block {
+        let xml = format!(
+            r#"<?xml version="1.0"?>
+<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
+<w:body>
+<w:tbl>{rows_xml}</w:tbl>
+</w:body></w:document>"#
+        );
+        let mut dom = Dom::new();
+        let doc = dom.parse_xdocument(&xml);
+        let root = dom.root(doc).expect("root");
+        let tbl = dom
+            .descendants(root, Some(&W::tbl()))
+            .into_iter()
+            .next()
+            .expect("tbl");
+        let sheet = StyleSheet {
+            defaults: Defaults::word(),
+            by_id: HashMap::new(),
+            tables: HashMap::new(),
+            theme: ThemeFonts::default(),
+            latent: true,
+            default_table: true,
+        };
+        let mut numbering = Numbering::default();
+        table_block(
+            &dom,
+            tbl,
+            &sheet,
+            &mut numbering,
+            &mut AuthorColors::default(),
+            &HashMap::new(),
+            None,
+        )
+    }
+
+    #[test]
+    fn a_huge_grid_span_sizes_a_bounded_grid_not_gigabytes() {
+        // One cell claiming 4 294 967 295 columns is ten digits of XML;
+        // the budgets clamp it before the column, preference and merge
+        // vectors are sized from it.
+        let block = laid_out_table(
+            r#"<w:tr><w:tc><w:tcPr><w:gridSpan w:val="4294967295"/></w:tcPr><w:p/></w:tc></w:tr>"#,
+        );
+        match block {
+            Block::Table { cols, rows, .. } => {
+                assert!(cols.len() <= MAX_ROW_GRID_COLUMNS, "{} columns", cols.len());
+                assert_eq!(rows[0][0].colspan, MAX_GRID_SPAN);
+            }
+            _ => panic!("expected table"),
+        }
+    }
+
+    #[test]
+    fn huge_grid_before_and_after_are_clamped_like_spans() {
+        let block = laid_out_table(
+            r#"<w:tr><w:trPr><w:gridBefore w:val="1000000000"/></w:trPr><w:tc><w:p/></w:tc></w:tr>"#,
+        );
+        match block {
+            Block::Table { cols, rows, .. } => {
+                assert!(cols.len() <= MAX_ROW_GRID_COLUMNS, "{} columns", cols.len());
+                assert!(rows[0][0].grid_skip, "the row opens on its grid-skip cell");
+                assert_eq!(rows[0][0].colspan, MAX_GRID_SPAN);
+            }
+            _ => panic!("expected table"),
+        }
+    }
+
+    #[test]
+    fn a_row_whose_spans_sum_past_the_column_budget_is_bounded() {
+        // Eight individually valid spans of 1024 columns: the sum, not
+        // any single span, must respect the row budget.
+        let cell = r#"<w:tc><w:tcPr><w:gridSpan w:val="1024"/></w:tcPr><w:p/></w:tc>"#;
+        let block = laid_out_table(&format!("<w:tr>{}</w:tr>", cell.repeat(8)));
+        match block {
+            Block::Table { cols, rows, .. } => {
+                assert_eq!(cols.len(), MAX_ROW_GRID_COLUMNS);
+                assert_eq!(rows[0].len(), 4, "1024×4 columns fill the budget");
+            }
+            _ => panic!("expected table"),
+        }
+    }
+
+    #[test]
+    fn ordinary_merged_and_skipped_columns_still_lay_out() {
+        // A Word-written 4-column grid: a one-column gridBefore, a
+        // 2-column merged cell and a single cell. Inside the budgets the
+        // layout is exactly what the renderer laid out before them.
+        let block = laid_out_table(
+            r#"<w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/><w:gridCol w:w="1360"/></w:tblGrid>
+<w:tr><w:trPr><w:gridBefore w:val="1"/></w:trPr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr>"#,
+        );
+        match block {
+            Block::Table { cols, rows, .. } => {
+                assert_eq!(cols.len(), 4);
+                assert_eq!(cols[0], 200.0);
+                let row = &rows[0];
+                assert_eq!(
+                    (row[0].col, row[0].colspan, row[0].grid_skip),
+                    (0, 1, true)
+                );
+                assert_eq!((row[1].col, row[1].colspan), (1, 2));
+                assert_eq!((row[2].col, row[2].colspan), (3, 1));
+            }
+            _ => panic!("expected table"),
+        }
+    }
 }
 
 #[cfg(test)]
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f9..e0d1351 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -10,6 +10,8 @@
 use std::collections::HashMap;
 use std::fmt::Write as _;
 
+use super::{MAX_GRID_SPAN, MAX_ROW_GRID_COLUMNS};
+
 /// `xml` with every `w:altChunk` replaced by its chunk's paragraphs.
 /// `load` returns a chunk's bytes by relationship id.
 pub(crate) fn expand(xml: &str, load: impl Fn(&str) -> Option<Vec<u8>>) -> String {
@@ -466,10 +468,17 @@ impl Builder {
         let Some(table) = self.tables.pop() else {
             return;
         };
+        // Checked sums under the same grid budgets the document table
+        // path enforces: one huge `colspan` (or a row of many valid ones
+        // summed) would write one grid column of markup per column here.
         let cols = table
             .rows
             .iter()
-            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
+            .map(|r| {
+                r.iter()
+                    .try_fold(0usize, |acc, c| acc.checked_add(c.span))
+                    .map_or(MAX_ROW_GRID_COLUMNS, |sum| sum.min(MAX_ROW_GRID_COLUMNS))
+            })
             .max()
             .unwrap_or(1)
             .max(1);
@@ -563,7 +572,8 @@ impl Builder {
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
                         .unwrap_or(1)
-                        .max(1),
+                        .max(1)
+                        .min(MAX_GRID_SPAN),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
@@ -817,6 +827,37 @@ mod tests {
         assert!(html.contains("<p class=\"x\">Članak</p>"), "{html}");
     }
 
+    #[test]
+    fn a_huge_colspan_is_clamped_before_the_grid_is_generated() {
+        // A colspan of 4 294 967 295 once wrote one <w:gridCol/> per
+        // column — gigabytes of markup from ten digits of attribute. The
+        // budgets clamp it before the generation loop runs.
+        let wml = html_to_wml("<table><tr><td colspan=\"4294967295\">x</td></tr></table>");
+        assert_eq!(wml.matches("<w:gridCol").count(), MAX_GRID_SPAN, "{wml}");
+        assert!(
+            wml.contains(&format!("<w:gridSpan w:val=\"{}\"/>", MAX_GRID_SPAN)),
+            "{wml}"
+        );
+    }
+
+    #[test]
+    fn a_row_of_valid_colspans_summing_past_the_budget_is_clamped() {
+        let cells = "<td colspan=\"1024\">a</td>".repeat(8);
+        let wml = html_to_wml(&format!("<table><tr>{cells}</tr></table>"));
+        assert_eq!(
+            wml.matches("<w:gridCol").count(),
+            MAX_ROW_GRID_COLUMNS,
+            "{wml}"
+        );
+    }
+
+    #[test]
+    fn ordinary_colspans_still_generate_their_grid() {
+        let wml = html_to_wml("<table><tr><td colspan=\"2\">a</td><td>b</td></tr></table>");
+        assert_eq!(wml.matches("<w:gridCol").count(), 3, "{wml}");
+        assert!(wml.contains("<w:gridSpan w:val=\"2\"/>"), "{wml}");
+    }
+
     mod regression_tests {
         use super::*;
 
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
index 1881053..e8aff15 100644
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -1598,23 +1598,34 @@ impl Story<'_> {
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
         let mut cell: Vec<String> = Vec::new();
+        // How many open field frames are still in their code (before their
+        // separator). `in_code` below is only "any code frame is open", so
+        // a count answers it in O(1); scanning the stack per character is
+        // quadratic when a document leaves separated, unended fields.
+        let mut code_fields = 0usize;
 
         for story in chars {
             // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
+            let in_code = code_fields > 0;
             let text = match story.ch {
                 '\u{13}' => {
                     fields.push(true);
+                    code_fields += 1;
                     continue;
                 }
                 '\u{14}' => {
                     if let Some(top) = fields.last_mut() {
+                        if *top {
+                            code_fields -= 1;
+                        }
                         *top = false;
                     }
                     continue;
                 }
                 '\u{15}' => {
-                    fields.pop();
+                    if fields.pop() == Some(true) {
+                        code_fields -= 1;
+                    }
                     continue;
                 }
                 _ if in_code => continue,
@@ -1967,6 +1978,74 @@ mod tests {
         assert_eq!(texts, ["aoneb"]);
     }
 
+    #[test]
+    fn many_separated_fields_that_never_end_stay_linear() {
+        // Each begin+separator pair leaves a frame that is no longer code;
+        // with the whole stack false, "any code frame" once scanned every
+        // frame before every character (quadratic in the pair count). The
+        // active-code count answers in O(1); the bound is the house's
+        // difat-sector timing pattern.
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let mut text = String::new();
+        for _ in 0..200_000 {
+            text.push('\u{13}');
+            text.push('\u{14}');
+        }
+        text.push_str("shown\r");
+        let chars = story_chars(&text);
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        let started = std::time::Instant::now();
+        let blocks = story.blocks(&chars);
+        assert!(
+            started.elapsed() < std::time::Duration::from_secs(2),
+            "field-code suppression must stay linear, not stack-scan per character"
+        );
+        let texts: Vec<String> = blocks
+            .iter()
+            .map(|b| match b {
+                Block::Paragraph(p) => p.text(),
+                Block::Table(_) => "table".into(),
+            })
+            .collect();
+        // Separated fields show their results (here: nothing), so the
+        // trailing word displays.
+        assert_eq!(texts, ["shown"]);
+    }
+
+    #[test]
+    fn separated_nested_fields_count_down_and_up_correctly() {
+        // {code}{inner}d: the inner field's result sits in the outer
+        // field's *result* (already separated), so it shows; text after
+        // both ends shows too. The count must fall on each end and rise
+        // again on each begin exactly when the frame is code.
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let chars = story_chars("\u{13}c\u{14}\u{13}\u{14}d\u{15}e\u{15}f\r");
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        let texts: Vec<String> = story
+            .blocks(&chars)
+            .iter()
+            .map(|b| match b {
+                Block::Paragraph(p) => p.text(),
+                Block::Table(_) => "table".into(),
+            })
+            .collect();
+        assert_eq!(texts, ["def"]);
+    }
+
     /// A supplementary character split across two Unicode pieces is one
     /// character, as Word reads it; a lone high surrogate is a replacement
     /// mark, never dropped.
diff --git a/src/markdown/from_docx/drawing.rs b/src/markdown/from_docx/drawing.rs
index a944602..c22e009 100644
--- a/src/markdown/from_docx/drawing.rs
+++ b/src/markdown/from_docx/drawing.rs
@@ -65,6 +65,11 @@ const MAX_DEP_RELS: usize = 64;
 const MAX_DEP_BYTES: u64 = 16 * 1024 * 1024;
 /// The most `drawing:` payloads one document queues.
 const MAX_PENDING: usize = 4096;
+/// The most dependent parts one materialization may write before it gives
+/// up: a cycle in the dependency graph (parts naming one another) would
+/// recurse forever, and a densely shared graph re-expands the same
+/// subgraph on every path that reaches it. Both stop here.
+const MAX_MATERIALIZE_STEPS: usize = MAX_DEPS * MAX_DEP_RELS;
 
 /// What `![label](drawing:PAYLOAD)` decodes to: the element's XML with its
 /// relationship ids symbolized (`r1`, `d3.2` — part 3's second relationship),
@@ -1887,11 +1892,16 @@ pub(crate) fn materialize(
     next_doc_pr: &mut u32,
 ) -> Option<String> {
     let mut xml = drawing.xml.clone();
+    // One traversal state per materialization: the dependency indices on
+    // the current path (a revisit is a cycle) and the parts written so
+    // far (the work budget).
+    let mut traversal = Traversal::default();
     for rel in &drawing.rels {
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(index), _) if index < drawing.deps.len() => {
-                let name = dep_part(drawing, index, package, existing, used, next_doc_pr)?;
+                let name =
+                    dep_part(drawing, index, package, existing, used, next_doc_pr, &mut traversal)?;
                 (rel.kind.clone(), relative_rel_target(part, &name), false)
             }
             (_, _, Some(raw)) => (rel.kind.clone(), relative_rel_target(part, raw), false),
@@ -1951,8 +1961,19 @@ fn renumber_doc_pr(xml: &mut String, next: &mut u32) {
     *xml = out;
 }
 
+/// Cycle and work state for one materialization walk.
+#[derive(Default)]
+struct Traversal {
+    /// Dependency indices on the current path: a graph's parts may name
+    /// one another, so a revisit is a cycle, refused instead of recursed.
+    active: HashSet<usize>,
+    /// Dependent parts written on this walk so far.
+    steps: usize,
+}
+
 /// Write dependent part `index` and its own dependency closure; its new
-/// part name.
+/// part name. A dependency that (transitively) names itself, or a walk
+/// past the work budget, refuses the whole drawing (`None`).
 fn dep_part(
     drawing: &Drawing,
     index: usize,
@@ -1960,6 +1981,29 @@ fn dep_part(
     existing: &HashSet<String>,
     used: &mut HashSet<String>,
     next_doc_pr: &mut u32,
+    traversal: &mut Traversal,
+) -> Option<String> {
+    if !traversal.active.insert(index) {
+        return None;
+    }
+    if traversal.steps >= MAX_MATERIALIZE_STEPS {
+        traversal.active.remove(&index);
+        return None;
+    }
+    traversal.steps += 1;
+    let name = dep_part_closure(drawing, index, package, existing, used, next_doc_pr, traversal);
+    traversal.active.remove(&index);
+    name
+}
+
+fn dep_part_closure(
+    drawing: &Drawing,
+    index: usize,
+    package: &mut PartFs,
+    existing: &HashSet<String>,
+    used: &mut HashSet<String>,
+    next_doc_pr: &mut u32,
+    traversal: &mut Traversal,
 ) -> Option<String> {
     let dep = drawing.deps.get(index)?;
     let name = name_for(&dep.name, existing, used);
@@ -1971,7 +2015,8 @@ fn dep_part(
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(inner), _) if inner < drawing.deps.len() => {
-                let inner_name = dep_part(drawing, inner, package, existing, used, next_doc_pr)?;
+                let inner_name =
+                    dep_part(drawing, inner, package, existing, used, next_doc_pr, traversal)?;
                 (
                     rel.kind.clone(),
                     relative_rel_target(&name, &inner_name),
@@ -2148,4 +2193,146 @@ mod tests {
         );
         assert_eq!(next, 42);
     }
+
+    const REL_IMAGE: &str =
+        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
+
+    fn payload_package() -> PartFs {
+        PartFs::open(include_bytes!(
+            "../../../tests/fixtures/redline/original.docx"
+        ))
+        .unwrap()
+    }
+
+    #[test]
+    fn a_cyclic_dependency_is_refused_not_recursed() {
+        // One dependent part whose own relationship names itself: the
+        // materializer must give the drawing up (None), not recurse until
+        // the stack ends.
+        let drawing = Drawing {
+            rels: vec![PayRel {
+                sym: "r1".into(),
+                kind: REL_IMAGE.into(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps: vec![Dep {
+                name: "media/loop.png".into(),
+                kind: "image/png".into(),
+                b64: "aGk=".into(),
+                rels: vec![PayRel {
+                    sym: "d0.1".into(),
+                    kind: REL_IMAGE.into(),
+                    part: Some(0),
+                    ..PayRel::default()
+                }],
+                ..Dep::default()
+            }],
+            ..Drawing::default()
+        };
+        let mut package = payload_package();
+        let existing: HashSet<String> = package.parts().into_iter().collect();
+        let mut used = HashSet::new();
+        let mut next = 0;
+        assert!(
+            materialize(
+                &drawing,
+                &mut package,
+                "word/document.xml",
+                &existing,
+                &mut used,
+                &mut next
+            )
+            .is_none()
+        );
+        assert!(package.part_bytes("media/loop.png").is_none());
+    }
+
+    #[test]
+    fn an_acyclic_dependency_chain_still_materializes() {
+        let drawing = Drawing {
+            xml: r#"<drawing r:embed="r1"/>"#.into(),
+            rels: vec![PayRel {
+                sym: "r1".into(),
+                kind: REL_IMAGE.into(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps: vec![Dep {
+                name: "media/ok.png".into(),
+                kind: "image/png".into(),
+                b64: "aGk=".into(),
+                ..Dep::default()
+            }],
+            ..Drawing::default()
+        };
+        let mut package = payload_package();
+        let existing: HashSet<String> = package.parts().into_iter().collect();
+        let mut used = HashSet::new();
+        let mut next = 0;
+        let xml = materialize(
+            &drawing,
+            &mut package,
+            "word/document.xml",
+            &existing,
+            &mut used,
+            &mut next,
+        )
+        .expect("an acyclic payload materializes");
+        assert!(xml.contains("rId"), "{xml}");
+        assert!(package.part_bytes("media/ok.png").is_some());
+    }
+
+    #[test]
+    fn a_dependency_graph_that_re_expands_past_the_budget_is_refused() {
+        // Every part names the next one twice: no cycle, but the number
+        // of paths through the graph doubles per level. The work budget
+        // refuses the drawing instead of walking them all.
+        let mut deps = Vec::new();
+        for i in 0..MAX_DEPS {
+            let rels = (i + 1 < MAX_DEPS)
+                .then(|| {
+                    (1..=2)
+                        .map(|n| PayRel {
+                            sym: format!("d{i}.{n}"),
+                            kind: REL_IMAGE.into(),
+                            part: Some(i + 1),
+                            ..PayRel::default()
+                        })
+                        .collect::<Vec<_>>()
+                })
+                .unwrap_or_default();
+            deps.push(Dep {
+                name: format!("media/dep{i}.png"),
+                kind: "image/png".into(),
+                rels,
+                ..Dep::default()
+            });
+        }
+        let drawing = Drawing {
+            rels: vec![PayRel {
+                sym: "r1".into(),
+                kind: REL_IMAGE.into(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps,
+            ..Drawing::default()
+        };
+        let mut package = payload_package();
+        let existing: HashSet<String> = package.parts().into_iter().collect();
+        let mut used = HashSet::new();
+        let mut next = 0;
+        assert!(
+            materialize(
+                &drawing,
+                &mut package,
+                "word/document.xml",
+                &existing,
+                &mut used,
+                &mut next
+            )
+            .is_none()
+        );
+    }
 }
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..dea46cb 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -63,6 +63,13 @@ pub(crate) const MAX_NAMESPACE_PREFIX_BYTES: usize = 256;
 pub(crate) const MAX_LIVE_NS_BINDINGS: usize = 256;
 /// Total prefix+URI bytes declared by xmlns attributes in one document.
 pub(crate) const MAX_NS_DECLARED_BYTES: usize = 1024 * 1024;
+/// Element nesting the permissive DOM parser itself will build. Admitted
+/// parts are capped at 256 by admission's depth scan; a part that reached
+/// this parser without one (a renamed auxiliary part an unchecked
+/// consumer opened, say) stops here instead of exhausting the stack —
+/// deeper tags are consumed but build no nodes. Above every legitimate
+/// nesting in the tree (the markdown reader's own stack caps at 512).
+pub(crate) const MAX_PARSE_DEPTH: usize = 1024;
 
 // N02 mechanism counter: bytes of (prefix, uri) pairs the parser's
 // namespace scope has recorded. The scoped-overlay scope only ever records
@@ -104,6 +111,7 @@ pub fn parse_xdocument(dom: &mut Dom, xml: &str) -> NodeId {
         dom,
         c: chars,
         pos: 0,
+        depth: 0,
     };
     p.parse_document()
 }
@@ -139,6 +147,7 @@ struct Parser<'a> {
     dom: &'a mut Dom,
     c: Vec<char>,
     pos: usize,
+    depth: usize,
 }
 
 /// In-scope namespace bindings as an overlay stack (N02): one `HashMap`
@@ -497,6 +506,59 @@ impl<'a> Parser<'a> {
         self.slice(start, self.pos)
     }
 
+    /// The position just past the `<!` construct at `self.pos`: a CDATA
+    /// section at its `]]>`, any other declaration (DOCTYPE and friends)
+    /// at the `>` that closes it outside quoted strings and outside the
+    /// internal subset's brackets — where quick-xml ends them too.
+    fn bang_construct_end(&self) -> usize {
+        let len = self.len();
+        if self.starts_with("<![CDATA[") {
+            return self.index_of("]]>", self.pos).map_or(len, |e| e + 3);
+        }
+        let mut quote = None;
+        let mut brackets = 0usize;
+        let mut i = self.pos + 2; // past "<!"
+        while i < len {
+            let ch = self.c[i];
+            match quote {
+                Some(q) if ch == q => quote = None,
+                Some(_) => {}
+                None => match ch {
+                    '\'' | '"' => quote = Some(ch),
+                    '[' => brackets += 1,
+                    ']' => brackets = brackets.saturating_sub(1),
+                    '>' if brackets == 0 => return i + 1,
+                    _ => {}
+                },
+            }
+            i += 1;
+        }
+        len
+    }
+
+    /// Consume one markup token (`<…>`) without interpreting it, quote
+    /// aware so an attribute's `>` does not end it. Used past the nesting
+    /// budget, where tags build no nodes but must still advance.
+    fn skip_tag(&mut self) {
+        let len = self.len();
+        let mut quote = None;
+        while self.pos < len {
+            let ch = self.c[self.pos];
+            self.pos += 1;
+            match quote {
+                Some(q) if ch == q => quote = None,
+                Some(_) => {}
+                None => {
+                    if ch == '\'' || ch == '"' {
+                        quote = Some(ch);
+                    } else if ch == '>' {
+                        return;
+                    }
+                }
+            }
+        }
+    }
+
     fn parse_document(&mut self) -> NodeId {
         let doc = self.dom.new_document();
         // N02: one overlay stack serves every top-level element; marks are
@@ -537,9 +599,14 @@ impl<'a> Parser<'a> {
                 continue;
             }
             if self.starts_with("<!") {
-                // DOCTYPE — skip
-                let end = self.index_of(">", self.pos);
-                self.pos = end.map(|e| e + 1).unwrap_or(len);
+                // DOCTYPE and other declarations are opaque here, but
+                // consumed by their complete grammar: a quoted or
+                // bracketed `>` does not close the construct. quick-xml
+                // (admission's scan, `validate_xml`) ends them in the
+                // same places, so nothing inside can surface as markup
+                // elements this parser recurses into but admission never
+                // depth-counted.
+                self.pos = self.bang_construct_end();
                 continue;
             }
             if self.cur() == '<' {
@@ -553,6 +620,13 @@ impl<'a> Parser<'a> {
     }
 
     fn parse_element(&mut self, scope: &mut NsScope) -> NodeId {
+        self.depth += 1;
+        let el = self.parse_element_inner(scope);
+        self.depth -= 1;
+        el
+    }
+
+    fn parse_element_inner(&mut self, scope: &mut NsScope) -> NodeId {
         // at '<'
         self.pos += 1; // consume '<'
         let raw_name = self.read_name();
@@ -658,6 +732,14 @@ impl<'a> Parser<'a> {
                 continue;
             }
             if self.cur() == '<' {
+                if self.depth >= MAX_PARSE_DEPTH {
+                    // Past the nesting budget (admission allows 256): a
+                    // renamed or otherwise unchecked part may nest deeper
+                    // than any real document. Consume the tag without
+                    // building it, so the descent cannot exhaust the stack.
+                    self.skip_tag();
+                    continue;
+                }
                 let child = self.parse_element(scope);
                 self.dom.add(el, child);
                 continue;
@@ -1179,6 +1261,77 @@ mod xml_lexical_boundary_contract_tests {
         assert_eq!(dom.value(root), "owned");
         assert!(validate_xml("<?xml-stylesheet type='text/css'?>\n<p>owned</p><?tail?>").is_ok());
     }
+
+    #[test]
+    fn a_doctype_with_a_quoted_close_is_consumed_whole() {
+        // The `>` inside the quoted entity value does not close the
+        // DOCTYPE (quick-xml, and admission through it, read the same
+        // grammar): the markup after the construct is the document's real
+        // root, not phantom elements the depth scan never saw. Deep
+        // nesting in the residue once recursed to a stack overflow.
+        let deep = "<a>".repeat(100_000);
+        let close = "</a>".repeat(100_000);
+        let xml = format!("<!DOCTYPE d [<!ENTITY e \"]>{deep}{close}\">]><p/>");
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &xml);
+        let root = dom.root(document).unwrap();
+        assert_eq!(dom.name(root).unwrap().local_name(), "p");
+    }
+
+    #[test]
+    fn a_top_level_cdata_section_with_a_close_inside_is_consumed_whole() {
+        // `>` inside a top-level CDATA body once split it, and the rest
+        // of the body parsed as elements. The section ends at `]]>`.
+        let xml = "<![CDATA[ > <x/> ]]><p>t</p>";
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &xml);
+        let root = dom.root(document).unwrap();
+        assert_eq!(dom.name(root).unwrap().local_name(), "p");
+        assert_eq!(dom.value(root), "t");
+    }
+
+    #[test]
+    fn nesting_within_the_parse_budget_builds_to_full_depth() {
+        let xml = format!(
+            "{}{}",
+            "<e>".repeat(MAX_PARSE_DEPTH),
+            "</e>".repeat(MAX_PARSE_DEPTH)
+        );
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &xml);
+        let mut cur = dom.root(document).unwrap();
+        let mut depth = 1usize;
+        while let Some(child) = dom.element(cur, &crate::xmllinq::XName::get("e", "")) {
+            cur = child;
+            depth += 1;
+        }
+        assert_eq!(depth, MAX_PARSE_DEPTH);
+    }
+
+    #[test]
+    fn nesting_past_the_parse_budget_flattens_instead_of_overflowing() {
+        // 50 000 nested elements on a 2 MiB stack: without the budget the
+        // recursive descent exhausts it; with it, the first
+        // MAX_PARSE_DEPTH levels build and the rest is consumed without
+        // nodes.
+        let xml = format!("{}{}", "<e>".repeat(50_000), "</e>".repeat(50_000));
+        let parsed = std::thread::Builder::new()
+            .stack_size(2 * 1024 * 1024)
+            .spawn(move || {
+                let mut dom = Dom::new();
+                let document = parse_xdocument(&mut dom, &xml);
+                let mut cur = dom.root(document).unwrap();
+                let mut depth = 1usize;
+                while let Some(child) = dom.element(cur, &crate::xmllinq::XName::get("e", "")) {
+                    cur = child;
+                    depth += 1;
+                }
+                depth
+            })
+            .expect("spawn");
+        let depth = parsed.unwrap_or_else(|_| panic!("stack overflow in the DOM parser"));
+        assert_eq!(depth, MAX_PARSE_DEPTH);
+    }
 }
 
 #[cfg(test)]
diff --git a/src/revision_processor.rs b/src/revision_processor.rs
index 5e8ad57..e44967f 100644
--- a/src/revision_processor.rs
+++ b/src/revision_processor.rs
@@ -5609,3 +5609,77 @@ mod alternate_content_paragraph_owner_tests {
         }
     }
 }
+
+
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod deep_renamed_part_tests {
+    use super::*;
+    use crate::opc::PartFs;
+
+    const MAIN: &str = "word/document.xml";
+    const HEADER_REL: &str =
+        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
+
+    /// A package whose header relationship targets a `.bin` part:
+    /// admission depth-checks `.xml`/`.rels` names only, so the part's
+    /// bytes reach revision processing whatever they hold.
+    fn package_with_renamed_header(header: &str) -> Vec<u8> {
+        let mut pkg = PartFs::open(include_bytes!(
+            "../tests/fixtures/word_probes/tokens/cell_a.docx"
+        ))
+        .unwrap();
+        for part in pkg.parts() {
+            pkg.remove_part(&part);
+        }
+        let empty =
+            b"<Relationships xmlns='http://schemas.openxmlformats.org/package/2006/relationships'/>";
+        pkg.set_part("_rels/.rels", empty.to_vec());
+        pkg.set_part(
+            "word/_rels/document.xml.rels",
+            format!(
+                "<Relationships xmlns='http://schemas.openxmlformats.org/package/2006/relationships'>\
+                 <Relationship Id='rId1' Type='{HEADER_REL}' Target='header1.bin'/>\
+                 </Relationships>"
+            )
+            .into_bytes(),
+        );
+        pkg.add_package_relationship(
+            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
+            MAIN,
+        );
+        pkg.add_content_type_override(
+            "/word/document.xml",
+            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
+        );
+        pkg.add_content_type_override("/word/header1.bin", "application/octet-stream");
+        pkg.set_part(
+            MAIN,
+            format!(
+                "<w:document xmlns:w='{}'><w:body><w:p><w:r><w:t>x</w:t></w:r></w:p><w:sectPr/></w:body></w:document>",
+                W::URI
+            )
+            .into_bytes(),
+        );
+        pkg.set_part("word/header1.bin", header.as_bytes().to_vec());
+        pkg.to_zip().unwrap()
+    }
+
+    #[test]
+    fn accept_revisions_survives_a_deeply_nested_renamed_header() {
+        // 50 000 nested elements in a renamed auxiliary part, on a 2 MiB
+        // stack: the shared parser's nesting budget keeps revision
+        // processing from recursing to exhaustion.
+        let xml = format!("{}{}", "<a>".repeat(50_000), "</a>".repeat(50_000));
+        let bytes = package_with_renamed_header(&xml);
+        let accepted = std::thread::Builder::new()
+            .stack_size(2 * 1024 * 1024)
+            .spawn(move || crate::document_comparer::accept_revisions(&bytes))
+            .expect("spawn")
+            .join()
+            .unwrap_or_else(|_| panic!("stack overflow in revision processing"));
+        let out = accepted.expect("accept_revisions");
+        let pkg = PartFs::open(&out).unwrap();
+        assert!(pkg.part_string(MAIN).is_some());
+    }
+}

```
