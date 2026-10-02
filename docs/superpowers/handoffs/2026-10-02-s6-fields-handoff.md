<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Handoff: Plan 1, Task 7 (S6, field and TOC refresh), branch `adopt/s6-fields`

Written 2026-10-02 at the end of a session that was cut off before any code
was written. Everything below was read from the tree at `b420d64` (main) or
from the plan branch; nothing is from memory. No subagents were spawned, so
there is no agent work to fold in.

## State of the branch

- `adopt/s6-fields` is `main` at `b420d64` plus this handoff file. No source
  changes, no tests, no spike yet.
- Toolchain present in the container: cargo 1.97.0, rustc 1.97.0, uv 0.8.17.
  `cargo build` has not been run yet, so the first build is cold.
- The plan is not on main. Get it with:

  ```bash
  git fetch origin ccr-a92b0695-1adjlu
  git show origin/ccr-a92b0695-1adjlu:docs/superpowers/plans/2026-10-02-provider-adoption-1-s01-s10.md > /tmp/plan.md
  ```

  Task 7 is lines 1218 to 1343. Read "What was verified" (line 71), "Gates
  for every task" (89) and "Task order" (117) first.

## Scope (from the plan and the task brief)

Deliverables, all additive, nothing outside the task:

1. `src/fields.rs`: `update_fields(docx) -> Result<Updated, FieldError>`
   refreshing cached results of `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and TOC
   entries; `FieldUpdate { kind, code, paragraph, old, new }` (Serialize),
   `Updated { docx, fields, page_count }`. `PAGE` is never materialized.
   Field codes stay. `w:updateFields` is not set.
2. `convert::layout_facts(docx) -> Result<LayoutFacts, ConvertError>`
   (`pub(crate)`): `page_count`, `bookmark_pages`, `paragraph_pages`
   indexed like `inspect::paragraphs`, from one layout pass.
3. `OperationKind::InsertToc { paragraph, position, levels, title }` and a
   plan-level `update_fields: bool`; `check_operation_keys` row.
4. CLI `jubarte fields update FILE -o OUT [--json]`.
5. `capabilities`: `operations.fields` and `"insert_toc"` in
   `edit_operations`.
6. Python `Document.update_fields()`, `EditPlan.insert_toc(...)`,
   `EditPlan(update_fields=True)`; WASM `updateFields`.
7. `tests/fields_update.rs` (the plan's Step 1 test, adapted), docs:
   skill §2, CHANGELOG under `## [Unreleased]`, `docs/WORD_DIFFERENCES.md`
   caveat (page numbers are jubarte's layout, not Word's).

The brief says to start with a spike proving the layout's `bookmark_pages`
map is populated for body bookmarks after `layout()`, and to record the
result in the PR.

Conflict discipline from the brief: append new `OperationKind` variants at
the END of the enum, new rows at the END of `check_operation_keys`, new
names at the END of `capabilities.edit_operations` and of the list in
`tests/agent_contracts.rs`, new `Command` variants at the END of the enum,
new code in new files, CHANGELOG bullets under `## [Unreleased]` only. Keep
the `src/convert/mod.rs` change as small as possible.

## What the layout already does (src/convert/mod.rs, line numbers at b420d64)

- `with_pages` (439 to 563) is the single pipeline: strict to transitional,
  open `PartFs`, altChunk expansion, fonts, stylesheet, `collect_blocks`
  (6830), then `layout(&fonts, &page, &hf, &blocks, compat_mode, footnotes)`
  at 561, which returns only `Vec<Page>`. The `Layout` state is dropped.
- `layout()` starts at 29452. Its tail (around 30060 to 30081):
  `paint_page_footnotes`, `patch_chap_page`, `patch_pagerefs`,
  `patch_numpages(fonts, &mut lay.pages)`, then `lay.pages`. Other callers
  of `layout()` are unit tests at 41219 and 41275.
- `Layout` fields: `bookmark_pages: HashMap<String, String>` (19596),
  `pageref_ops: Vec<(usize, usize, String)>` (19597),
  `known_bookmarks: HashSet<String>` (19600), `bookmark_texts` (19602).
  `Layout::new` initializes them at about 20062.
- `bookmark_pages` values are page LABELS (strings), from
  `chap_page_label()` (28207): the section page label plus a chapter
  prefix when `chap_style` is set. That is what Word writes into a PAGEREF
  result and a TOC line, so keep labels; the plan's `BTreeMap<String, u32>`
  would lose "iii" and "A-3".
- `bookmark_pages` is filled at two sites:
  - 30027, body paragraphs, after the paragraph is laid out: the label of
    the page where the paragraph ENDS (a one-line heading: same page).
  - 26697, table cell paragraphs, `para.bookmarks` and
    `para.blank_bookmarks`.
- `patch_pagerefs` (28219) rewrites PAGEREF text ops from the map;
  `patch_numpages` (28320) replaces `NUMPAGES_MARK` (`"@@N@@"`, 28309)
  with `pages.len()`.
- `para_bookmark_names` (9889) collects every `w:bookmarkStart` descendant
  of the paragraph. A `w:bookmarkStart` that is a body-level sibling of
  `w:p` (Word does that for `_GoBack` and table bookmarks) is in no block
  and therefore never in `bookmark_pages`. The TOC generator must put its
  `_Toc` bookmarks inside the heading paragraph; say so in the PR.
- `document_bookmark_texts` (9954) is the `REF` text source;
  `BOOKMARK_NOT_DEFINED` and `REF_NOT_FOUND` constants near 10061;
  `apply_field_results` at 10065.
- `para_is_empty_toc_field` (8209) and `is_toc_style` (8202) exist.
- `Block` enum at 1505: `Paragraph { runs, style, list, images, boxes,
  bookmarks }`. No source paragraph index. Construction sites: 7505, 7544,
  8012 (a paragraph split at page or column breaks; bookmarks only on
  piece 0), 9884 (`paragraph_block`), tests 41229 and 41285. `CellPara`
  (1704) has `bookmarks` and `blank_bookmarks`; its construction sites are
  8979, 11290, 11315, 12082, 12144.
- Line 7470: a blank paragraph that holds a bookmark before a page break
  stays as a block ("One holding a bookmark stays, so PAGEREF keeps the
  page before the break"). So bookmarks are NOT layout neutral. Do not
  inject synthetic per-paragraph bookmarks to get paragraph pages; that
  would change pagination in the facts pass and the gate says the layout
  change must not alter rendering.
- `RenderReport` (303), `render()` (357), `docx_render_report` (435).
  `docx_to_pdf_inner` (281) sets the `REVISIONS` thread local from
  `options.revisions` around the layout; `layout_facts` should do the same
  with `PdfOptions::default()` so pagination equals the default convert.
- `inspect::body_paragraph_nodes` (src/inspect.rs:523): every `w:p` under
  the body in document order, text boxes excluded, table cells INCLUDED;
  `paragraphs_of` numbers them `body:p:N`. `paragraph_pages` must follow
  this order.

## Design decided (not yet implemented)

1. `LayoutFacts { page_count: usize, bookmark_pages: BTreeMap<String,
   String>, paragraph_pages: Vec<u32> }` (1-based page ordinal of the
   paragraph's first line; 0 or a sentinel when the paragraph laid out
   nothing). Keep it `pub(crate)`.
2. Rename the body of `layout()` to `layout_with_facts()` returning
   `(Vec<Page>, LayoutFacts)`; keep `fn layout(...) -> Vec<Page>` as a
   one-line wrapper so the two unit tests stay untouched.
3. Split `with_pages` into `with_layout<T>(docx, emit: impl FnOnce(&Fonts,
   &[Page], &LayoutFacts) -> T)` and keep `with_pages` as a wrapper that
   drops the facts. `pub(crate) fn layout_facts(docx) -> Result<LayoutFacts,
   ConvertError>` calls `with_layout` under the `REVISIONS` guard.
4. Paragraph pages: add `origin: Option<usize>` to `Block::Paragraph` and
   `CellPara` (index into `body_paragraph_nodes` order). Put a
   `HashMap<NodeId, usize>` into `WalkCtx` (built once from
   `inspect::body_paragraph_nodes(dom, body)` in `collect_blocks`) and set
   `origin` in `paragraph_block` and the cell paragraph builder; split
   pieces keep it on piece 0 only; synthetic blocks (7505, 7544, endnotes,
   tests) get `None`. In `layout()`'s block loop, before a `Block::Paragraph`
   record `p0 = lay.pages.len() - 1` and `o0 = lay.pages[p0].ops.len()`;
   after it, the first page `p >= p0` whose ops grew (for `p0`: len > o0;
   later pages: non-empty) is the first-line page; store into
   `lay.para_pages: HashMap<usize, usize>`. At the cell site (26697) use
   `self.pages.len() - 1`. Floats spliced behind (`behind_end`) still grow
   the same page's op count, so the heuristic holds.
5. Spike test (write first, in `tests/fields_update.rs` or a `#[cfg(test)]`
   in convert): body = Heading1 "A" with `w:bookmarkStart w:name="_Toc1"`
   inside the paragraph, `w:br w:type="page"`, Heading1 "B" with `_Toc2`.
   Expect `bookmark_pages == {"_Toc1": "1", "_Toc2": "2"}`, `page_count ==
   2`, `paragraph_pages == [1, 2, 2]` or `[1, 1, 2]` depending on where the
   break paragraph lands (record what you observe). Note `docx()` in
   tests/common/docx.rs writes no styles part; check whether the layout
   falls back to built-in Heading1 metrics (the plan says `convert` does).
6. `src/fields.rs`:
   - Open through `inspect::Opened::open` (admitted), walk the body for
     complex fields: `w:fldChar begin`, `w:instrText` (concatenate),
     `separate`, `end`, with a stack (model on `debug::check_fields`, line
     404 of src/debug.rs; either make that walker `pub(crate)` or write a
     small one in fields.rs). Also handle `w:fldSimple` with `w:instr`.
   - Parse codes: first word is the kind (`PAGEREF name [\h]`, `REF name
     [\h \r \n \w \p]`, `NUMPAGES`, `SEQ ident [\* ...]`, `TOC [\o "1-3"]
     [\h] [\u] [\t ...]`).
   - TOC: entries from paragraphs whose style id is `Heading{n}` or whose
     style name is `heading n`, `n` within `\o`'s range (default 1-3). Add
     `w:bookmarkStart/End w:name="_TocNNNNNNNN"` around the heading's runs
     when none present (ids from the next free `w:id` among all
     bookmarkStart in the body). Write entry paragraphs between `separate`
     and `end`: `w:pStyle TOC{n}`, `w:hyperlink w:anchor="_Toc..."`, the
     heading text, `w:tab`, then a nested `PAGEREF _Toc... \h` complex
     field with a placeholder result. `TOC1..TOCn` and `TOCHeading`
     styles: `src/builtin_styles.rs` only lists NAMES (`"toc 1"`, `"toc
     heading"`, `is_built_in`), it has NO definitions, so the plan's
     "built-in definitions in src/builtin_styles.rs" is wrong. Write the
     style XML in fields.rs (look at `src/markdown/` for how it writes
     Heading styles into styles.xml and reuse the pattern): TOC n = based
     on Normal, next Normal, `w:ind w:left=(n-1)*220`, `w:tabs` right stop
     at the text width with `w:leader="dot"`, `w:spacing w:after="100"`;
     TOC Heading based on Heading1 with `w:outlineLvl 9`.
   - Then `convert::layout_facts(&package_bytes)` ONCE on the package as it
     now stands (serialize the DOM, `pkg.set_part`, `pkg.to_zip`), and write
     each result: PAGEREF -> label (or `Error! Bookmark not defined.` when
     unknown), NUMPAGES -> page_count, REF -> bookmarked paragraph text (or
     `Error! Reference source not found.`), SEQ -> running count per
     identifier. Replace the runs between `separate` and `end` by one
     `w:r` carrying the first result run's `w:rPr` and one `w:t`.
   - Return `Updated`; the test asserts `assert_word_valid_package`.
7. `src/edit.rs`: append `InsertToc { paragraph: Selector, #[serde(default)]
   position: Side, #[serde(default = "three")] levels: u8, title:
   Option<String> }` after `MergeParagraphs` (enum at 131; last variant
   ends about 268). Add `#[serde(default, skip_serializing_if = "Not::not")]
   pub update_fields: bool` to `EditPlan` (43; it is
   `deny_unknown_fields`, so every constructor in the crate, Python and
   WASM must set it, and `commented_base` at 2494 builds an `EditPlan`
   literal). `check_operation_keys` (783): append `"insert_toc" =>
   &["position", "levels", "title"]`. Kind name mapping at 2721. Resolution
   in `resolve()` (1231) like `InsertParagraph`; application in `apply()`
   step 2 (2380) by building the field paragraph (and the optional
   `TOCHeading` title paragraph) with the DOM API (`new_element`,
   `set_attribute_value`, `add`, `add_after_self`, `add_before_self`).
   In `apply_plan` (840), after `tx.finish()` returns `(clean, marked)`,
   when `plan.update_fields` run `fields::update_fields` on `clean` (and on
   `marked` when `Some`, since the comparer reads it) before the compare.
   Record the refreshed fields in the report if cheap (a new optional
   `fields: Vec<FieldUpdate>` with `skip_serializing_if`).
8. `src/capabilities.rs`: add `#[serde(default)] pub fields: bool` at the
   END of `Operations`, append `"insert_toc"` to `edit_operations`, bump
   the unit test's `len()` from 9 to 10. `tests/agent_contracts.rs` line
   23 area lists the kinds; append an `insert_toc` sample at the END.
9. CLI (`src/bin/jubarte.rs`): `Command` enum at 136, last variants are
   `Capabilities` (377), `SelfUpdate` (389), `Debug` (420). Append
   `Fields { #[command(subcommand)] sub: FieldsCommand }` with
   `FieldsCommand::Update { file, output, force, json }`. Dispatch near
   2049. Helpers to copy: `run_inspect` (983), `run_text` (1037),
   `exit_code` (1405); `Accept` shows the `-o/--force` pattern.
10. Python (`jubarte-python/src/lib.rs`, tests in `jubarte-python/tests/`,
    `docx_fixture.py` has a fixture builder) and WASM
    (`jubarte-wasm/src/lib.rs`): not read yet. Grep for `insert_paragraph`
    and `render` to find the `EditPlan` builder and the `Document` class.
11. Docs: `skills/jubarte-documents/SKILL.md` §2 "Edit with a plan" (line
    67); `CHANGELOG.md` `## [Unreleased]` / `### Added` at the top;
    `docs/WORD_DIFFERENCES.md` has numbered sections 1 to 9 under
    `## Differences` (§5 is "Revised field numbers (PAGE, NUMPAGES)"), add
    §10 for refreshed field caches coming from jubarte's layout.

## Gates (run before every commit, one Cargo process at a time)

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --test fields_update
cargo test --all-features --test convert_docx_to_pdf   # before and after the layout change
cargo test --all-features --test convert_docx_to_png
cargo test --all-features --test edit_plan
cargo test --all-features --test agent_contracts
cargo run --bin jubarte -- --help >/dev/null
uv tool run --from 'reuse[charset-normalizer]' reuse lint
cargo llvm-cov --all-features --lcov --output-path target/lcov.info   # 80% line floor
cd jubarte-python && uv run --with maturin maturin develop --release && \
  uv run --with pytest --with pytest-cov pytest -q --cov=jubarte_redlines --cov-branch --cov-report=term-missing
```

Red/green: write the failing test, see it fail, implement, see it pass,
commit. Every new file carries the SPDX header its neighbours use. No
`#[allow(...)]`.

## Finish

```bash
git fetch origin main && git rebase origin/main
# rerun the gates
git push -u origin adopt/s6-fields
```

Open a DRAFT pull request against `main` with `mcp__github__create_pull_request`
(repo `jandira-tech/jubarte-redlines`). The PR body states what the plan
asked, what was done, the exact test and gate output with coverage numbers,
every deviation from the plan and why (so far: labels instead of `u32` for
`bookmark_pages`; `builtin_styles.rs` has no definitions; body-level
bookmarks outside paragraphs are not paged; `paragraph_pages` via an
`origin` index rather than synthetic bookmarks), the spike result, and
anything left undone.

## Test helpers (tests/common)

- `docx(body_xml) -> Vec<u8>`, `docx_with(body_xml, &[Part])`,
  `part_string(docx, name) -> Option<String>` in `tests/common/docx.rs`.
- `assert_word_valid_package(&[u8])` in `tests/common/validity.rs:35` (the
  plan's `validate()` is Task 2 on a sibling branch; use this).
- `tests/edit_plan.rs` shows how plans are built and applied in tests.
