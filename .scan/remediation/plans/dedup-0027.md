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
