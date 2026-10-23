@coderabbitai plan
Please enrich, fix and assess the plan below.

## Finding: csf_9e40f23e742aa43493a0fb92

- **Title:** Rendering a small DOCX table can exhaust host memory
- **Severity:** medium
- **Rule:** resource-exhaustion.table-grid-span

**Summary:** An untrusted DOCX author can set huge positive w:gridSpan, w:gridBefore or w:gridAfter counts in a small, shallow table. Rendering sums those counts into grid occupancy and directly sizes column, preference and merge vectors. Counts of 1000000000/1073741824 request about 4 GB/4 GiB for f32 columns; 4294967295 requests approximately 16 GiB before additional allocations, despite package admission. Assigned discovery-0031 corroborates the existing one-billion-column, 4,000,000,000-byte f32-vector framing through both span and row-skip inputs; these are static counts.
 Assigned discovery-0032 corroborates the live span and both synthetic row-skip paths, including merge-slot expansion; separate Markdown span clamping does not protect rendering.

**Remediation:** Enforce practical per-span and total-column budgets for gridSpan, gridBefore and gridAfter before excessive RawCell, column, preference or merge allocation/work. Use checked occupancy sums and col+span arithmetic, apply the same budget to every derived table vector, and return a typed resource-limit refusal on excess. Propagate a caller-adjustable semantic column limit through all public rendering surfaces and charge later column/merge vectors to that same budget.
Additional retained remediation detail: Enforce explicit maximum live cell spans, row skips, aggregate table columns and merge slots before any allocation. Use checked sums and a cumulative layout resource budget, returning a typed refusal when exceeded.

## Finding: csf_8dcaf50784fd95a7f0c2ec28

- **Title:** Processing renamed auxiliary XML bypasses XML depth limits
- **Severity:** medium
- **Rule:** uncontrolled-recursion.unchecked-auxiliary-xml

**Summary:** A DOCX author can put deeply nested auxiliary XML under a non-.xml/.rels suffix. Admission drains those bytes without XML policy, while rendering, revision listing, revision acceptance/rejection and comparison follow internal relationships and recursively build a DOM. Confirmed rendering parts include theme, styles, numbering, comments/commentsExtended, notes and headers/footers; get_revisions and comparison also parse renamed footnotes/endnotes. A small admitted package can terminate the native host or trap WASM. The assigned source additionally confirms eager relationship-selected fontTable parsing before font inspection, including renamed fontTable.bin. Comparison preprocessing also follows renamed styles and charts, alongside footnotes/endnotes. The assigned Standard source additionally identifies edit initialization's existing_comment_ids on relationship-selected comments before the later checked CommentFamily loader. Assigned discovery-0019 corroborates the same shared selector for styles, theme, numbering and fontTable under .bin names; its proof is limited to those four consumers, while earlier validated consumer paths remain retained. Assigned discovery-0027 additionally confirms relationship-selected settings bytes reaching settings_dom/settings_dom_xml and unchecked DOM construction on cache miss; settings is now a supplied validated consumer, whereas earlier recommendations remain historical source limitations. Assigned discovery-0028 supplies already-validated linked-textbox and diagram consumers: relationship-ID media targets at src/convert/mod.rs:17417-17428 and :17453-17464 are decoded and recursively parsed regardless of filename. The new source corroborates fontTable, theme, styles, numbering, settings, comments, notes and header/footer paths, and records that admission's content-type check covers the main part only. Assigned discovery-0031 additionally preserves strict normalization's extension-based pass-through: a relationship-selected fonts.bin remains unmodified before eager fontTable parsing. The raw unchecked parser's documented caller obligations are not themselves the finding.

**Remediation:** Apply active InputLimits at a centralized checked XML access/parse boundary for every semantic XML consumer regardless of filename, or resolve/classify and admit XML through actual content-type/relationship use. Cover theme, styles, numbering, converter comments/commentsExtended, notes and headers/footers; explicitly cover the now supplied validated settings_dom/settings_dom_xml cache-miss consumer, with earlier settings-only recommendations retained as historical source limitations. Preserve renamed-main checks, and enforce actual DOM construction depth or use iterative construction. Apply that same mandatory checked boundary to get_revisions and comparison notes parsing, including both footnotes and endnotes; propagate typed refusal under the active comparer InputLimits. Include pre_process_markup's relationship-selected notes at src/document_comparer.rs:4725-4750 and exposed Python get_revisions_json/WASM getRevisions; valid arbitrary part names remain supported. Include linked-textbox and diagram relationship targets in the mandatory filename-independent checked XML boundary before parse_xdocument and in regression coverage. These are now supplied validated affected consumers from discovery-0028; earlier sources' proposed-only status remains historical. Cover revision_processor RevisionIds collection and process_part for all six selected auxiliary buckets (headers, footers, endnotes, footnotes, comments and styles), including public revision acceptance/rejection. Explicitly include the now-confirmed fontTable loader (src/convert/font_table.rs:66-78; src/convert/mod.rs:806) in the mandatory checked XML boundary; parsing must be bounded before any font root/entry inspection. Explicitly cover comparison defined_style_ids (src/document_comparer.rs:4684-4696), relationship-selected footnotes/endnotes (4724-4750) and chart preprocessing (4861-4878), all reached through admitted nonidentical comparison inputs (6555-6569,6607-6625). Apply the same mandatory XML boundary before edit initialization's existing_comment_ids scan, not only in the later checked CommentFamily loader; preserve the effective checks at inspect::parse_part_within and edit::load_stories. Enforce the same checked boundary before settings DOM cache population, and preserve valid bounded nonstandard auxiliary filenames.

## Finding: csf_eaffa9e4af9c9c07d800ef1b

- **Title:** ZIP footer and count interpretation mismatch bypasses entry-count admission
- **Severity:** medium
- **Rule:** zip-entry-budget.eocd-selection-differential

**Summary:** The independent preindex footer parser can count a different EOCD or count field from zip 9.0.5. False small-count EOCDs in valid comments can be skipped by zip; a same-footer total=3/on-this-disk=10001 also passes max_entries=10000 while zip indexes 10001. ZIP64 size/offset triggers differ too. Neither admission nor PartFs enforces the actual opened count; duplicate detection also trusts the unrelated declared count. Another supplied false-footer proof uses an EOCD-shaped record inside the real archive comment whose own comment length is impossible; preindex trusts its low count while the ZIP reader rejects it and backtracks.

**Remediation:** Use the delegated reader's authoritative validated EOCD/ZIP64 selection, consistent ZIP32 total/per-disk fields and all ZIP64 total/size/offset triggers. Bound raw central-record traversal/count before index allocation, reject inconsistent unsupported metadata, and retain a post-open active archive.len cap as defense in depth; that later check cannot undo earlier allocation. Reconcile selected raw counts with indexed counts for exact duplicates, including name collapse, comments, ZIP64 and prepended data. Detect raw exact-name duplicates directly or through the reader's duplicate flag rather than relying solely on a count-shortfall inference.

---

# The plan

# dedup-0002 remediation plan

Patch: `.scan/remediation/patches/dedup-0002.patch` (applies with `git apply`
from the repository root — verified with `git apply --check`, output below).
Double-checked and corrected by the review round: the original patch's new
code was not fully rustfmt-clean (three regions — see Verification); the
patch was regenerated through rustfmt itself and is now
`rustfmt --check`-clean. Stats after the fix: 6 files, 440 insertions,
33 deletions.

## Scope

This folder's own contributions are the three findings whose evidence refs
carry the `dedup0002-*` ids written by this round's reducer (claimed worker
`335ff7a0-6db6-4979-afb9-0b15e7381e94`, see
`.scan/artifacts/deep_discovery/dedup/dedup-0002/prompt.md`):

| finding (ruleId) | dedup0002 evidence refs |
| --- | --- |
| `resource-exhaustion.table-grid-span` | `dedup0002-n1-table-entry` |
| `uncontrolled-recursion.unchecked-auxiliary-xml` | `dedup0002-n0-comments-rel-target`, `dedup0002-n0-comment-consumers`, `dedup0002-n0-chrome-parse` |
| `zip-entry-budget.eocd-selection-differential` | `dedup0002-n7-package-load` |

The folder's result.json holds 18 findings; the other 15 inherit evidence
only from earlier rounds' workers (`3d0b17d0-…`, `8e5f16fa-…`, no
`dedup0002-*` refs). Per the serial-dedup instruction this patch fixes the
three above and leaves the inherited material to the round that owns it.

All three fixes are in the same three subsystems the evidence names:
`src/admission.rs`, `src/opc/mod.rs`, `src/convert/mod.rs` (plus
`src/convert/font_table.rs`, same consumer class as `load_stylesheet`).

---

## 1. `resource-exhaustion.table-grid-span` — huge table spans size layout vectors directly

### Vulnerability (verified in source)

* `src/convert/mod.rs:14204` `cell_span` parses `w:gridSpan` as `usize`
  with only `.max(1)` — `4294967295` passes.
* `src/convert/mod.rs:14340` `row_grid_skip` parses `w:gridBefore` /
  `w:gridAfter` the same way; the count is kept as the placeholder cell's
  `colspan` (`grid_skip_cell`, `src/convert/mod.rs:14379`).
* `src/convert/mod.rs:13601-13616` sums a row's colspans
  (`row.iter().map(|c| c.colspan).sum()`) into `occupancy` and uses it to
  `vec![each; occupancy]` / `cols.resize(occupancy, 80.0)` — a 4 294 967 295
  span requests ~16 GiB of `f32` columns; `gridBefore` 1 073 741 824
  requests ~4 GiB.
* `src/convert/mod.rs:14263` `column_prefs` allocates
  `vec![PrefWidth::Auto; grid.len()]` from that same grid.
* `src/convert/mod.rs:14637` `resolve_table_merges` does
  `open.resize(col + span, None)` — `col + span` is unchecked `usize`
  arithmetic (overflow panic in debug builds, wrap in release), and the row
  walk's `grid_at += colspan.max(1)` (`:13486`) and
  `stretch_stale_spans`'s `.sum()` (`:14175`) are unchecked too.

Nothing between package admission (bytes/entries/depth) and these
allocations bounds the numeric attribute (admission's XML budgets were
confirmed at `src/admission.rs:534-570`: they gate by part *bytes* and
*depth*, not attribute values).

### Failing tests (red on current code)

All in `tests/convert_docx_to_pdf.rs` (the render-path table home; helpers
`minimal_docx_body` + `docx_to_pdf` + `pdf_page_count` already live there):

1. `a_huge_grid_span_stays_inside_the_column_budget` — a one-cell table
   with `w:gridSpan w:val="4294967295"`; asserts `docx_to_pdf` returns a
   PDF with `pdf_page_count(&pdf) >= 1`. **Red now:** the layout calls
   `vec![f32; 4 294 967 295]` (~16 GiB) and the test process dies on the
   allocation abort/OOM.
2. `a_huge_grid_before_stays_inside_the_column_budget` — same shape with
   `w:gridBefore w:val="1073741824"` (~4 GiB request). **Red now:** same
   abort.
3. `summed_row_spans_stay_inside_the_column_budget` — 3 000 cells each
   `w:gridSpan w:val="1000000"` (a ~180 KiB part): summed occupancy
   3×10⁹ sizes a ~12 GiB vector, and the unchecked `.sum()` can overflow.
   **Red now:** abort/overflow.
4. Control `ordinary_spans_and_skips_still_lay_out` — gridBefore 1 +
   gridSpan 2 over a 3-column grid renders one page. Green before and
   after (Word-shape preservation).

### Minimal source change (`src/convert/mod.rs`)

* New `const MAX_TABLE_GRID_COLUMNS: usize = 1024;` beside `cell_span`.
  Word itself builds no table beyond 63 columns; 1024 is 16× that maximum
  and caps every derived vector at a few KiB.
* `cell_span`: `.min(MAX_TABLE_GRID_COLUMNS)` on the parsed span.
* `row_grid_skip`: `.map(|n| n.min(MAX_TABLE_GRID_COLUMNS))`.
* Occupancy loop: `fold` with `saturating_add`, then
  `.min(MAX_TABLE_GRID_COLUMNS)` before it sizes `cols`.
* `resolve_table_merges`: `let reach = col.saturating_add(span)
  .min(MAX_TABLE_GRID_COLUMNS);` resizes to `reach`;
  `col = col.saturating_add(span)`.
* Checked arithmetic at the remaining accumulators: `grid_at`
  (`:13467`, `:13486`) and `stretch_stale_spans` (`:14175`, `:14199`).

Deviation from the suggested remediation, stated openly: the finding
suggests "a typed resource-limit refusal", but `table_block` returns a
`Block` with no error channel, and threading one through the 66 k-line
layout would not be a minimal patch. Clamping at 16× Word's own hard
maximum renders every Word-producible table byte-identically and bounds
hostile ones; this matches the repo's treatment of derived budgets
(`MAX_PAGE_PIXELS` is also a clamp).

### Existing behaviour that must keep passing

* `tests/convert_docx_to_pdf.rs::table_cell_spacing_opens_each_row_by_twice_the_spacing`
  and every other table test in that file (they exercise `table_block`,
  spans and grids far below 1024).
* `tests/m38_table_alignment.rs`, `tests/md_table_borders.rs`,
  `tests/m4b_preprocess.rs`, `tests/inspect_tables.rs` and the internal
  table tests in `src/convert/mod.rs` (e.g. the `gridSpan w:val="2"`
  fixture at `src/convert/mod.rs:48749`).
* The corpus fixtures (500+ committed docx) through
  `src/opc/archive.rs::every_committed_docx_opens_under_header_consistency`.

---

## 2. `uncontrolled-recursion.unchecked-auxiliary-xml` — relationship-selected XML bypasses the XML policy

### Vulnerability (verified in source)

Admission applies the XML depth/tag policy **by file name**
(`src/admission.rs:534`: `is_xml = lower.ends_with(".xml") || lower.ends_with(".rels")`),
with one extension-independent exception for the *main* part only
(`src/admission.rs:557-570`). But the renderer's auxiliary parts are
selected **by relationship**, so their names are the producer's:

* `load_theme` (`:3305`), `load_stylesheet` (`:3470`, via
  `main_rel_part` `:21310`), `load_numbering` (`:6557`), the five
  `settings_*` readers (`:6865`, `:6909`, `:7192`, `:7258`, `:7285`,
  `:7322`, `:7499`), `load_font_table` (`src/convert/font_table.rs:70`);
* `part_xml_by_rel_kind` (`:8473`) → `comments`/`commentsExtended`
  (`comments_extended` `:7667-7689`, `load_comments` `:7781`),
  `footnotes` (`load_footnotes` `:8691`), `endnotes` (`EndnoteBag::load`
  `:8516`), `diagramDrawing` (`:17506`);
* headers/footers by relationship id: `load_chrome_part` (`:20759`) →
  `chrome_part_xml` (`:20786`).

Every one of them parses with the permissive, recursive
`Dom::parse_xdocument` (`src/xmllinq/parse.rs:98`, recursion at
`:660-663`) with no depth bound: a ~700 KiB `styles.bin` nesting 100 000
elements fits every byte budget and overflows the parser stack. A checked
entry point already exists (`try_parse_xdocument_with_limits`,
`src/xmllinq/parse.rs:129`) but is not used on these paths.

### Failing tests (red on current code)

All in `tests/convert_docx_to_pdf.rs`, built by two new helpers:
`docx_with_bin_aux_part(body, rel_type, target, part_xml)` (a package
whose single auxiliary part is a `.bin` reached only through
`word/_rels/document.xml.rels`) and
`deep_part(root_open, root_close, depth, inner)`.

1. `a_renamed_styles_part_past_the_depth_budget_renders_without_it` —
   `parts/look.bin` styles nesting 100 001 deep carrying
   `docDefaults sz=40`; asserts conversion succeeds, a page exists, and
   the 20pt marker (`20.00 Tf`/`20.10 Tf`, the same assertion
   `styles_and_font_table_are_found_through_relationships` uses) is
   **absent**. **Red now:** `parse_xdocument` recurses 100 k deep and the
   test process dies on stack overflow.
2. `renamed_aux_parts_past_the_depth_budget_render_without_them` — the
   same over-depth part for each remaining gate: `theme`, `numbering`,
   `settings`, `fontTable`, `comments`, `commentsExtended`, `footnotes`,
   `endnotes` (each asserts the document still converts to ≥ 1 page).
   **Red now:** stack overflow on each route.
3. `a_renamed_header_part_past_the_depth_budget_renders_without_it` —
   header chosen by `w:headerReference r:id` to `parts/hdr1.bin`, over
   depth; asserts the page still renders. **Red now:** stack overflow in
   `chrome_part_xml`.
4. Controls (green before and after):
   `a_renamed_styles_part_inside_the_depth_budget_still_applies` — the
   same renamed part at depth 201 (inside the 256 budget) still applies
   its `docDefaults sz=40` (`20.00 Tf` present), proving the gate drops
   only over-budget XML; and the existing
   `styles_and_font_table_are_found_through_relationships` (renamed
   `.xml` parts, `tests/convert_docx_to_pdf.rs:371`).

### Minimal source change

One central gate, applied wherever the render path parses a
relationship-selected part as XML:

* `src/admission.rs`: `check_xml_depth` becomes `pub(crate)` — it is
  exactly the tolerant structural policy (depth + `TagPolicy::tolerant`
  tag/namespace budgets, BOM/UTF-16-aware `part_text`) admission already
  applies to every `.xml`/`.rels` part, so the gate is the same test the
  same document already passed under its `.xml` names.
* `src/convert/mod.rs`: new `checked_part_xml(pkg, path) -> Option<String>`
  — `pkg.part_string(path)`, gated by `check_xml_depth(path, …,
  InputLimits::default())`; a refused part reads as `None` (absent),
  which is precisely how every consumer already treats a missing part
  (defaults, empty maps, `empty_chrome()`). Wired into:
  `part_xml_by_rel_kind` (covers comments, commentsExtended, footnotes,
  endnotes, diagramDrawing, and the conventional-name fallback),
  `load_theme`, `load_stylesheet`, `load_numbering`, and a new
  `settings_xml` helper replacing the seven `settings_*` part reads.
* `load_chrome_part` gates the exact string it will hand to
  `chrome_part_xml` (after the background `<w:p/>` insert) and lays a
  refused part out as `empty_chrome()`.
* `src/convert/font_table.rs`: `load_font_table` filters its part through
  the same check (same consumer class; renamed fontTable parts are
  already exercised by the existing renamed-parts test).

No behaviour change is possible for a package that reached layout with
conventional `.xml` names: admission already refused any such package
whose parts fail this identical scan, so the gate can only fire for parts
that were never name-scanned — exactly the attacker shape (or binary
garbage under an XML relationship, which previously produced a garbage
DOM). A refused over-depth part renders with the engine defaults rather
than killing the process — fail-closed, the documented contract for the
Python/WASM embeddings (`tests/input_admission.rs:10-11`).

### Existing behaviour that must keep passing

* `tests/convert_docx_to_pdf.rs::styles_and_font_table_are_found_through_relationships`
  (renamed parts still found and applied).
* `tests/convert_comment_placement.rs`, `tests/edit_comment_threads.rs`,
  `tests/comment_on_deleted_text.rs` (comment loading via
  `part_xml_by_rel_kind`), the footnote/endnote tests
  (`tests/edit_insert_footnote.rs` and friends), header/footer tests in
  `tests/convert_docx_to_pdf.rs` (chrome layout), settings-driven tests
  (`compatSetting`, `wpJustification`, …) and
  `tests/issue650_*`/`tests/input_admission.rs::issue650_*` for the tag
  policy itself.

---

## 3. `zip-entry-budget.eocd-selection-differential` — false EOCD in the comment floats the entry budget

### Vulnerability (verified in source and in zip 9.0.5's own code)

* `preindex_budget` (`src/admission.rs:399`) takes its count from
  `declared_entry_count` (`:580`), which scans **the last EOCD-like
  signature** in the tail window — including one planted inside the
  archive comment.
* The ZIP reader instead *validates* its candidates and retries earlier
  ones: `zip-9.0.5/src/format/find_central_directory.rs:100-118` (loop,
  comment-length check), `:148-167` (a directory offset at/past the
  candidate is impossible → `continue` to the earlier candidate), and
  `zip-9.0.5/src/read/zip_archive.rs:216-239` (retry after a failed
  directory read). It then indexes exactly the chosen footer's count
  (`read_central_header`, `zip_archive.rs:279-291`).
* So a false EOCD appended as the archive's comment (declared count 3,
  impossible offset) makes `preindex_budget` pass while `open_archive`
  indexes the real directory: 10 013 entries flow through
  `read_package`'s loop (`src/opc/mod.rs:289`) with no cap, and `admit`
  equally never compares `archive.len()` to `max_entries` (its only
  count check is the *undercount* direction, `src/admission.rs:451`).

### Failing tests (red on current code)

In `tests/input_admission.rs` (the entry-budget home), using a new helper
`with_false_eocd_comment(docx, declared)` that appends exactly that false
record as the archive comment (real comment length set to 22, false count
field `declared`, false offset `0x7fff_ffff` so zip rejects it):

1. `admit_refuses_the_opened_count_behind_a_false_eocd_comment` —
   `package_with_entries(30)` (33 real entries) + false footer declaring
   3, admitted under `small_limits()` (max 32): asserts
   `err.code() == "INPUT_LIMIT"` and `"33 ZIP entries"` in the message.
   **Red now:** the package *is* admitted (declared 3 ≤ 32, all 33 real
   entries well-formed), so `expect_err` panics.
2. `partfs_open_refuses_the_opened_count_behind_a_false_eocd_comment` —
   `package_with_entries(10_010)` (10 013 real) + false footer declaring
   3 through `PartFs::open` (compare budget, 10 000): asserts
   `INPUT_LIMIT` / `"10013 ZIP entries"`. **Red now:** the constructor
   indexes all 10 013 entries and returns `Ok`.
3. Control `a_false_eocd_comment_on_a_small_package_still_opens` — the
   same false footer on an honest 3-entry package: `admit` succeeds with
   `entries == 3` and `PartFs::open` still opens it. Green before and
   after (comment bytes that merely look like a record must not break a
   valid archive).

### Minimal source change

Enforce the entry budget on the archive that was actually opened,
immediately after opening (the remediation's own "independently cap
archive.len()"):

* `src/admission.rs`, in `admit` right after `open_archive_unchecked`:
  `if archive.len() > limits.max_entries { return
  Err(input_over_entries(archive.len() as u64, &limits)); }` — reusing
  the existing `INPUT_LIMIT` wording helper (`:357`).
* `src/opc/mod.rs`, in `read_package` right after `open_archive`: the
  same check wrapped through the existing `refused` adapter.

The over-declared direction is unchanged (preindex still refuses first),
and the duplicate-collapse check (`archive.len() < declared`) keeps its
role. What changes is only the candidate-mismatch shape — which no honest
writer produces, because a writer's real EOCD is the last signature and
is the one zip opens.

### Existing behaviour that must keep passing

* `tests/input_admission.rs`:
  `partfs_open_refuses_a_crowd_of_entries`,
  `scrub_refuses_a_crowd_of_entries`,
  `scrub_refuses_an_excess_declared_count_before_indexing`,
  `compare_refuses_more_entries_than_the_budget`,
  `every_package_entry_point_refuses_a_crowd_of_entries`,
  `leaks_fail_closed_on_a_crowd_of_entries`,
  `the_unadmitted_entry_points_still_open_a_valid_package`,
  `zip17_public_boundaries` and the ZIP64 probes
  (`zip64_tail_probe`, `zip64_sentinel_count_probe`, `zip64_forced_probe`),
  `admission_accepts_a_package_with_prepended_data`.
* `src/opc/archive.rs::every_committed_docx_opens_under_header_consistency`.

---

## Not covered by this patch (stated so the next round can take it)

* The auxiliary-XML gate covers the **render path** (`docx_to_pdf`) only.
  The finding's remediation also names the comparer's own XML consumers
  (`src/document_comparer.rs`: `defined_style_ids` :4684-4696, notes
  :4724-4750, chart preprocessing :4861-4878), `revision_processor.rs`
  (:3689-3794, public revision accept/reject and `get_revisions`), and
  edit initialization's `existing_comment_ids` (`src/edit.rs`). Those
  paths still parse relationship-selected parts with the permissive
  parser and are left to the rounds that own them.
* The opened-count check lands in `admit` and `read_package`
  (`PartFs::open`). `scrub::leaks_within` (`src/scrub.rs:418-424`) still
  gates only on `preindex_budget` before its own `open_archive` walk, and
  `strict_translation.rs`'s two direct opens (:671, :775) have no entry
  budget at all; neither is among the finding's named sinks, but the same
  false-EOCD shape floats their entry walk past the budget.
* The HTML/MHT altChunk colspan finding
  (`csf_64b247931cd1f053217fde7a`, `src/convert/altchunk.rs`) is a
  separate finding this folder inherits evidence for; it is not touched
  here.

## Verification performed in this session

* Read the folder inputs (`prompt.md`, `result.json` via `jq`,
  `threatmodel.md` selectively) and every source location cited above in
  the repository; also read the locked dependency's EOCD selection code
  at `~/.cargo/registry/src/…/zip-9.0.5/src/format/find_central_directory.rs`
  (the candidate loop rejects a central-directory offset at/past the EOCD
  and continues to the earlier candidate — exactly the retry the false-EOCD
  tests rely on; `Cargo.lock` pins zip 9.0.5) and `…/src/read/zip_archive.rs`.
* `git apply --check .scan/remediation/patches/dedup-0002.patch` from the
  repository root — **passed**. `git apply --stat`: 6 files, 440
  insertions, 33 deletions (after the fmt fix below).
* `git status` after all work — only `.scan/` is untracked; no repository
  source file was modified (all patch testing ran on scratch copies under
  `.scan/remediation/tmp/verify`).
* Review round (second agent), executed on scratch copies of the six
  files, never the checkout:
  * every hunk's context was diffed against the repository source — all
    anchors match;
  * `rustc --edition 2024 --crate-type lib --emit=metadata` (toolchain
    1.94.0, single files, resolution errors expected and seen, **zero
    parse-class errors** in all six patched files) — the reused closing
    braces at the end of `tests/convert_docx_to_pdf.rs` produce valid
    Rust;
  * `rustfmt --edition 2024 --check` (toolchain 1.94.0, repo
    `rustfmt.toml`): the original patch failed with three diffs (the
    `stretch_stale_spans` fold chain in `src/convert/mod.rs`, and the
    rel-type tuple array in `renamed_aux_parts_past_the_depth_budget_
    render_without_them`). The patch was regenerated from a
    rustfmt-formatted scratch tree and now passes `--check` cleanly; the
    pristine baseline also passes, so the patch carries no unrelated
    reformatting;
  * downstream consumers of a clamped `TableCell.col` were audited
    (`autofit_to_words` :9974-9989, `column_prefs` :14276-14296, and the
    `.get(...)/take(...)` sites) — all bounds-guarded, so a row of many
    clamped spans cannot index out of range;
  * the eager loaders the aux tests depend on were confirmed on the
    plain conversion path (`load_theme`/`load_stylesheet`/`settings_*`
    via `load_stylesheet`; `load_font_table` :806; `load_numbering`
    :7734, `load_comments` :7736, `comments_extended` :7667-7670,
    `EndnoteBag::load` :7750, `load_footnotes` :939), so each over-depth
    leg reaches the permissive parser today;
  * `TagPolicy::tolerant` has no per-document element-count budget
    (`src/xmllinq/parse.rs:288-345`: per-tag name/attribute/tag-byte and
    namespace budgets only), so the 201-deep control part passes and only
    the depth budget can refuse it;
  * the `20.00 Tf`/`20.10 Tf` marker assertion is the existing
    `styles_and_font_table_are_found_through_relationships` shape
    (`tests/convert_docx_to_pdf.rs:371-392`).
* Not run (forbidden by this ask's hard constraints, AGENTS.md's
  concurrent-cargo rule): `cargo nextest`, `cargo fmt --check` (the
  standalone `rustfmt --check` above is the substitute actually run),
  `cargo clippy -D warnings`, `jubarte --help`. The red/green claims are
  source-traced, not executed. Run order for whoever lands it: apply,
  `cargo fmt --check`, then the two touched suites — `cargo nextest run
  --test input_admission --test convert_docx_to_pdf` — followed by the
  keep-passing suites named per finding.


---

# The patch

```diff
diff --git a/src/admission.rs b/src/admission.rs
index d556610..b2ff842 100644
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -447,6 +447,14 @@ pub fn admit(bytes: &[u8], limits: InputLimits) -> Result<AdmittedPackage, Admis
     // its typed kind.
     let mut archive = crate::opc::open_archive_unchecked(Cursor::new(bytes))
         .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("not a ZIP file: {e}")))?;
+    // The preindex count comes from the last EOCD-like signature, but the
+    // reader validates its candidate and can open an earlier, real footer
+    // instead (a false EOCD inside the archive comment). Budget the
+    // archive that was actually opened, not the footer it declared
+    // (dedup-0002: zip-entry-budget.eocd-selection-differential).
+    if archive.len() > limits.max_entries {
+        return Err(input_over_entries(archive.len() as u64, &limits));
+    }
     // The reader keys entries by name, so exact duplicates collapse into one.
     if (archive.len() as u64) < declared {
         return Err(AdmissionError::new(
@@ -785,7 +793,11 @@ fn part_text(xml: &[u8]) -> Cow<'_, str> {
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
diff --git a/src/convert/font_table.rs b/src/convert/font_table.rs
index c1063f8..bdf5ee8 100644
--- a/src/convert/font_table.rs
+++ b/src/convert/font_table.rs
@@ -68,7 +68,20 @@ fn font_table_part(pkg: &PartFs) -> String {
 }
 
 pub(crate) fn load_font_table(pkg: &PartFs) -> FontTable {
-    pkg.part_string(&font_table_part(pkg))
+    // The part's name is the producer's, so the admission scan that
+    // selects parts by `.xml` extension cannot protect this parse: gate
+    // the XML by policy first, and read a refused part as absent
+    // (dedup-0002: uncontrolled-recursion.unchecked-auxiliary-xml).
+    let part = font_table_part(pkg);
+    pkg.part_string(&part)
+        .filter(|xml| {
+            crate::admission::check_xml_depth(
+                &part,
+                xml.as_bytes(),
+                crate::admission::InputLimits::default(),
+            )
+            .is_ok()
+        })
         .map(|xml| parse_font_table_xml(&xml))
         .unwrap_or_default()
 }
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
index d900e05..0da8149 100644
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -3303,9 +3303,8 @@ fn parse_len(s: &str) -> Option<f32> {
 }
 
 fn load_theme(pkg: &PartFs) -> ThemeFonts {
-    let Some(xml) = pkg
-        .part_string(&main_rel_part(pkg, "theme", "word/theme/theme1.xml"))
-        .or_else(|| pkg.part_string("word/theme/theme2.xml"))
+    let Some(xml) = checked_part_xml(pkg, &main_rel_part(pkg, "theme", "word/theme/theme1.xml"))
+        .or_else(|| checked_part_xml(pkg, "word/theme/theme2.xml"))
     else {
         return ThemeFonts::default();
     };
@@ -3477,7 +3476,7 @@ fn load_stylesheet(pkg: &PartFs) -> StyleSheet {
     defaults.wp_justify = settings_flag(pkg, "wpJustification");
     defaults.para.sum_spacing = settings_flag(pkg, "doNotUseHTMLParagraphAutoSpacing");
     let mut raw: std::collections::HashMap<String, RawStyle> = std::collections::HashMap::new();
-    let Some(xml) = pkg.part_string(&main_rel_part(pkg, "styles", "word/styles.xml")) else {
+    let Some(xml) = checked_part_xml(pkg, &main_rel_part(pkg, "styles", "word/styles.xml")) else {
         return StyleSheet {
             defaults,
             by_id: HashMap::new(),
@@ -6556,7 +6555,7 @@ fn bullet_glyph(raw: &str) -> String {
 
 fn load_numbering(pkg: &PartFs) -> Numbering {
     let numbering_part = main_rel_part(pkg, "numbering", "word/numbering.xml");
-    let Some(xml) = pkg.part_string(&numbering_part) else {
+    let Some(xml) = checked_part_xml(pkg, &numbering_part) else {
         return Numbering::default();
     };
     parse_numbering_xml(&xml, |rid| resolve_media(pkg, &numbering_part, rid))
@@ -6863,7 +6862,7 @@ fn lvl_indent(dom: &Dom, lvl: NodeId) -> (f32, f32) {
 
 /// `word/settings.xml` parsed once per reader; `None` when the part is absent.
 fn settings_dom(pkg: &PartFs) -> Option<ParsedSettings> {
-    let xml = pkg.part_string(&settings_part(pkg))?;
+    let xml = settings_xml(pkg)?;
     settings_dom_xml(&xml)
 }
 
@@ -6907,8 +6906,7 @@ fn settings_flag_xml(xml: &str, local: &str) -> bool {
 }
 
 fn settings_flag(pkg: &PartFs, local: &str) -> bool {
-    pkg.part_string(&settings_part(pkg))
-        .is_some_and(|xml| settings_flag_xml(&xml, local))
+    settings_xml(pkg).is_some_and(|xml| settings_flag_xml(&xml, local))
 }
 
 /// `w:compatSetting name="compatibilityMode"`. Absent → 12 (Word 2007),
@@ -7190,7 +7188,7 @@ fn split_fixed_advances(runs: &mut Vec<TextRun>, marks: &[bool], width: impl Fn(
 /// author's machine (a7110391's `C:\Users\...\NESO ... .dotx`, a4168b8a's
 /// `D:\CASA ... .dotx`); Word cannot load it and keeps the file's styles.
 fn settings_link_styles(pkg: &PartFs) -> bool {
-    pkg.part_string(&settings_part(pkg)).is_some_and(|xml| {
+    settings_xml(pkg).is_some_and(|xml| {
         settings_dom_xml(&xml).is_some_and(|s| {
             let (dom, root) = (&s.0, s.1);
             dom.descendants(root, Some(&W::name("attachedTemplate")))
@@ -7256,8 +7254,7 @@ fn link_template_styles(xml: &str) -> String {
 }
 
 fn settings_compat_mode(pkg: &PartFs) -> u8 {
-    pkg.part_string(&settings_part(pkg))
-        .map_or(12, |xml| settings_compat_mode_xml(&xml))
+    settings_xml(pkg).map_or(12, |xml| settings_compat_mode_xml(&xml))
 }
 
 /// The setting is keyed by `name` *and* `uri`: another vendor's
@@ -7283,8 +7280,7 @@ fn settings_compat_mode_xml(xml: &str) -> u8 {
 /// Word's `overrideTableStyleFontSizeAndJustification` compat setting: the
 /// default paragraph style's size and jc win over a table style's.
 fn settings_override_table_style_size(pkg: &PartFs) -> bool {
-    pkg.part_string(&settings_part(pkg))
-        .is_some_and(|xml| settings_override_table_style_size_xml(&xml))
+    settings_xml(pkg).is_some_and(|xml| settings_override_table_style_size_xml(&xml))
 }
 
 fn settings_override_table_style_size_xml(xml: &str) -> bool {
@@ -7320,8 +7316,7 @@ fn settings_mirror_margins(pkg: &PartFs) -> bool {
 /// alone squeezes; run languages do not count. A document default that
 /// names fonts turns it off too (`load_stylesheet`).
 fn settings_punct_squeeze(pkg: &PartFs) -> bool {
-    pkg.part_string(&settings_part(pkg))
-        .is_some_and(|xml| settings_punct_squeeze_xml(&xml))
+    settings_xml(pkg).is_some_and(|xml| settings_punct_squeeze_xml(&xml))
 }
 
 fn settings_punct_squeeze_xml(xml: &str) -> bool {
@@ -7497,10 +7492,9 @@ fn settings_do_not_expand_shift_return(pkg: &PartFs) -> bool {
 
 /// `w:characterSpacingControl/@w:val` (ECMA-376 17.15.1.18).
 fn settings_character_spacing(pkg: &PartFs) -> CharacterSpacing {
-    pkg.part_string(&settings_part(pkg))
-        .map_or(CharacterSpacing::DoNotCompress, |xml| {
-            settings_character_spacing_xml(&xml)
-        })
+    settings_xml(pkg).map_or(CharacterSpacing::DoNotCompress, |xml| {
+        settings_character_spacing_xml(&xml)
+    })
 }
 
 fn settings_character_spacing_xml(xml: &str) -> CharacterSpacing {
@@ -8466,6 +8460,27 @@ fn is_cover_pages_sdt(dom: &Dom, sdt: NodeId) -> bool {
         .is_some_and(|v| v.eq_ignore_ascii_case("Cover Pages"))
 }
 
+/// A part's XML admitted by the same tolerant structural policy — depth,
+/// tag and namespace budgets — that package admission applies to
+/// `.xml`/`.rels` parts, whatever this part's name is: a relationship can
+/// target `styles.bin` or `theme1.bin`, and the bytes are still parsed as
+/// XML downstream. A part that fails the policy reads as absent; every
+/// caller already treats a missing auxiliary part as "use the defaults"
+/// (dedup-0002: uncontrolled-recursion.unchecked-auxiliary-xml).
+fn checked_part_xml(pkg: &PartFs, path: &str) -> Option<String> {
+    let xml = pkg.part_string(path)?;
+    if crate::admission::check_xml_depth(
+        path,
+        xml.as_bytes(),
+        crate::admission::InputLimits::default(),
+    )
+    .is_err()
+    {
+        return None;
+    }
+    Some(xml)
+}
+
 /// The part `main` relates to by type `…/kind`, then by a type or target
 /// merely containing `kind`. The exact type goes first: a target check
 /// alone took `commentsExtended.xml` for the comments part when its
@@ -8480,12 +8495,12 @@ fn part_xml_by_rel_kind(pkg: &PartFs, main: &str, kind: &str) -> Option<String>
             .filter(|i| i.rel_type.ends_with(kind) || i.target.contains(kind));
         for item in by_type.chain(loose) {
             let path = pkg.resolve_rel_target(main, &item.target);
-            if let Some(xml) = pkg.part_string(&path) {
+            if let Some(xml) = checked_part_xml(pkg, &path) {
                 return Some(xml);
             }
         }
     }
-    pkg.part_string(&format!("word/{kind}.xml"))
+    checked_part_xml(pkg, &format!("word/{kind}.xml"))
 }
 
 /// Separator / continuation notes are chrome, not note text. Since plan
@@ -13464,7 +13479,7 @@ fn table_block(
                 &nested_at,
             );
             let (colspan, vmerge) = cell_span(dom, cell);
-            let last_col = grid_at + colspan.max(1) >= cols.len();
+            let last_col = grid_at.saturating_add(colspan.max(1)) >= cols.len();
             let at = |b: TblBorders| {
                 row_exception_cell_borders(b, ri == 0, ri + 1 == row_count, grid_at == 0, last_col)
             };
@@ -13483,7 +13498,7 @@ fn table_block(
                 }
                 None => row_borders.map(at),
             };
-            grid_at += colspan.max(1);
+            grid_at = grid_at.saturating_add(colspan.max(1));
             let (pad_l, pad_r) = cell_pad_h(dom, cell, tbl_pad_l, tbl_pad_r);
             let (pad_t, pad_b) = cell_pad_tb(dom, cell, tbl_pad_t, tbl_pad_b);
             // tblCellSpacing opens a gap above and below every cell
@@ -13601,7 +13616,12 @@ fn table_block(
     let mut occupancy = 0usize;
     let grid_len = cols.len();
     for row in &raw_rows {
-        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
+        // Checked and clamped: no Word-producible row reaches this budget,
+        // and a hostile one must not size the vectors below (dedup-0002).
+        let used = row
+            .iter()
+            .fold(0usize, |sum, cell| sum.saturating_add(cell.colspan));
+        occupancy = occupancy.max(used.min(MAX_TABLE_GRID_COLUMNS));
     }
     if cols.len() < occupancy {
         if cols.len() == 1 && occupancy > 1 {
@@ -14172,7 +14192,9 @@ fn cell_pad_tb(dom: &Dom, cell: NodeId, table_t: f32, table_b: f32) -> (f32, f32
 fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
     const STALE_SPAN_SLACK_PT: f32 = 20.0;
     for row in rows.iter_mut() {
-        let used: usize = row.iter().map(|c| c.colspan.max(1)).sum();
+        let used: usize = row
+            .iter()
+            .fold(0usize, |sum, c| sum.saturating_add(c.colspan.max(1)));
         if used >= cols.len() {
             continue;
         }
@@ -14196,11 +14218,20 @@ fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
                     spare -= extra;
                 }
             }
-            col += cell.colspan.max(1);
+            col = col.saturating_add(cell.colspan.max(1));
         }
     }
 }
 
+/// The widest table grid the layout admits, from one span or one row's
+/// summed occupancy. Word itself builds no table beyond 63 columns, so a
+/// value past this budget is not a document Word can produce — it is a
+/// numeric attribute asking the layout for gigabytes of column vectors (a
+/// `w:gridSpan` of 4 294 967 295 asked for ~16 GiB of `f32` columns), and
+/// it is clamped before any allocation (dedup-0002:
+/// resource-exhaustion.table-grid-span).
+const MAX_TABLE_GRID_COLUMNS: usize = 1024;
+
 fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
     let Some(pr) = first_named(dom, cell, "tcPr") else {
         return (1, VMerge::None);
@@ -14212,7 +14243,8 @@ fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|s| s.parse().ok())
         .unwrap_or(1)
-        .max(1);
+        .max(1)
+        .min(MAX_TABLE_GRID_COLUMNS);
     let vmerge = match direct_named(dom, pr, "vMerge") {
         None => VMerge::None,
         Some(n) => match dom.attribute(n, &W::val()) {
@@ -14342,7 +14374,8 @@ fn row_grid_skip(dom: &Dom, row: NodeId, side: &str) -> Option<(usize, PrefWidth
     let span = direct_named(dom, pr, &format!("grid{side}"))
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|v| v.parse::<usize>().ok())
-        .filter(|n| *n > 0)?;
+        .filter(|n| *n > 0)
+        .map(|n| n.min(MAX_TABLE_GRID_COLUMNS))?;
     let pref = direct_named(dom, pr, &format!("w{side}"))
         .filter(|n| attr_any(dom, *n, "type").is_none_or(|t| t == "dxa"))
         .and_then(|n| attr_any(dom, n, "w"))
@@ -14634,14 +14667,15 @@ fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>> {
                 borders: raw.borders,
                 style_fill: false,
             });
-            if open.len() < col + span {
-                open.resize(col + span, None);
+            let reach = col.saturating_add(span).min(MAX_TABLE_GRID_COLUMNS);
+            if open.len() < reach {
+                open.resize(reach, None);
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
@@ -20778,6 +20812,19 @@ fn load_chrome_part(
     if background && let Some(end) = xml.rfind("</w:hdr>") {
         xml.insert_str(end, "<w:p/>");
     }
+    // Header/footer parts arrive by relationship, so their names are the
+    // producer's: gate the exact bytes that will be parsed by the XML
+    // policy, and lay an over-budget part out as an empty one
+    // (dedup-0002: uncontrolled-recursion.unchecked-auxiliary-xml).
+    if crate::admission::check_xml_depth(
+        &path,
+        xml.as_bytes(),
+        crate::admission::InputLimits::default(),
+    )
+    .is_err()
+    {
+        return empty_chrome();
+    }
     chrome_part_xml(pkg, &path, &xml, local, sheet, text_w)
 }
 
@@ -21323,6 +21370,14 @@ fn settings_part(pkg: &PartFs) -> String {
     main_rel_part(pkg, "settings", "word/settings.xml")
 }
 
+/// The settings part's XML gated by the XML policy before the permissive
+/// parser reads it: the part's name is the producer's, so the admission
+/// scan that selects parts by `.xml` extension cannot be what protects
+/// this parse (dedup-0002: uncontrolled-recursion.unchecked-auxiliary-xml).
+fn settings_xml(pkg: &PartFs) -> Option<String> {
+    checked_part_xml(pkg, &settings_part(pkg))
+}
+
 fn parse_header_watermark(dom: &Dom, root: NodeId) -> Option<Watermark> {
     let mut marked = false;
     for gal in descendants_local(dom, root, "docPartGallery") {
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
index 9865bda..20667c4 100644
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -282,6 +282,16 @@ fn read_content_types(xml: &[u8]) -> Result<ContentTypes, OpcError> {
 fn read_package(bytes: &[u8], limits: crate::admission::InputLimits) -> Result<PartFs, OpcError> {
     crate::admission::preindex_budget(bytes, limits).map_err(refused)?;
     let mut archive = open_archive(Cursor::new(bytes))?;
+    // The reader may open an earlier, valid footer than the last
+    // EOCD-like signature the preindex count came from (a false record in
+    // the archive comment): budget the archive actually opened
+    // (dedup-0002: zip-entry-budget.eocd-selection-differential).
+    if archive.len() > limits.max_entries {
+        return Err(refused(crate::admission::input_over_entries(
+            archive.len() as u64,
+            &limits,
+        )));
+    }
     let mut parts = HashMap::new();
     let mut source_order = HashMap::new();
     let mut identities = HashSet::new();
diff --git a/tests/convert_docx_to_pdf.rs b/tests/convert_docx_to_pdf.rs
index 65815f1..dd882d5 100644
--- a/tests/convert_docx_to_pdf.rs
+++ b/tests/convert_docx_to_pdf.rs
@@ -49222,3 +49222,257 @@ fn a_label_wider_than_its_hang_tabs_the_text_to_the_next_stop() {
         );
     }
 }
+
+// ─── auxiliary XML admitted by policy, not by file name (dedup-0002) ────────
+
+/// A package whose one auxiliary part sits at a `.bin` name reached only
+/// through the main part's relationships. Admission's XML scan selects
+/// parts by `.xml`/`.rels` name, so the render path must gate the XML it
+/// parses by policy whatever the part is called
+/// (uncontrolled-recursion.unchecked-auxiliary-xml).
+fn docx_with_bin_aux_part(body: &str, rel_type: &str, target: &str, part_xml: &str) -> Vec<u8> {
+    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
+    let r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
+    let document = format!(
+        "<?xml version=\"1.0\"?><w:document xmlns:w=\"{w}\" xmlns:r=\"{r}\">\
+         <w:body>{body}</w:body></w:document>"
+    );
+    let content_types = "<?xml version=\"1.0\"?>\
+        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
+        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
+        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
+        <Default Extension=\"bin\" ContentType=\"application/octet-stream\"/>\
+        <Override PartName=\"/word/document.xml\" \
+          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
+        </Types>";
+    let rels = format!(
+        "<?xml version=\"1.0\"?>\
+         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
+         <Relationship Id=\"rId1\" Type=\"{r}/officeDocument\" Target=\"word/document.xml\"/></Relationships>"
+    );
+    let doc_rels = format!(
+        "<?xml version=\"1.0\"?>\
+         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
+         <Relationship Id=\"rIdAux\" Type=\"{r}/{rel_type}\" Target=\"{target}\"/></Relationships>"
+    );
+    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
+    let opts = SimpleFileOptions::default();
+    for (name, xml) in [
+        ("[Content_Types].xml", content_types.to_string()),
+        ("_rels/.rels", rels.to_string()),
+        ("word/document.xml", document),
+        ("word/_rels/document.xml.rels", doc_rels),
+    ] {
+        zip.start_file(name, opts).unwrap();
+        zip.write_all(xml.as_bytes()).unwrap();
+    }
+    zip.start_file(format!("word/{target}"), opts).unwrap();
+    zip.write_all(part_xml.as_bytes()).unwrap();
+    zip.finish().unwrap().into_inner()
+}
+
+/// `depth` nested `<e>` elements inside the given root, with `inner` as a
+/// sibling after the chain: the root plus the chain is the part's nesting
+/// depth. 100 000 elements stack-overflow the permissive DOM parser (a
+/// ~700 KiB part, inside every byte budget); 200 stays well inside the
+/// 256-element XML budget.
+fn deep_part(root_open: &str, root_close: &str, depth: usize, inner: &str) -> String {
+    format!(
+        "<?xml version=\"1.0\"?>{root_open}{}{inner}{}{root_close}",
+        "<e>".repeat(depth),
+        "</e>".repeat(depth)
+    )
+}
+
+const BIG_DEFAULTS: &str = "<w:docDefaults><w:rPrDefault><w:rPr>\
+     <w:rFonts w:ascii=\"Unheard Of Sans\" w:hAnsi=\"Unheard Of Sans\"/>\
+     <w:sz w:val=\"40\"/></w:rPr></w:rPrDefault></w:docDefaults>";
+
+/// Control: a renamed styles part inside the depth budget still loads and
+/// applies (the gate must not drop honest auxiliary XML).
+#[test]
+fn a_renamed_styles_part_inside_the_depth_budget_still_applies() {
+    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
+    let styles = deep_part(
+        &format!("<w:styles xmlns:w=\"{w}\">"),
+        "</w:styles>",
+        200,
+        BIG_DEFAULTS,
+    );
+    let docx = docx_with_bin_aux_part(
+        "<w:p><w:r><w:t>RenamedDepth</w:t></w:r></w:p>",
+        "styles",
+        "parts/look.bin",
+        &styles,
+    );
+    let pdf = docx_to_pdf(&docx).expect("a renamed styles part inside the budget loads");
+    let text = String::from_utf8_lossy(&pdf);
+    assert!(
+        text.contains("20.00 Tf") || text.contains("20.10 Tf"),
+        "docDefaults sz=40 from parts/look.bin must still set 20pt"
+    );
+}
+
+/// A renamed styles part nested past the XML budget must be dropped before
+/// the permissive DOM parser recurses on it: the document still renders,
+/// with the engine's own defaults instead of the part's.
+#[test]
+fn a_renamed_styles_part_past_the_depth_budget_renders_without_it() {
+    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
+    let styles = deep_part(
+        &format!("<w:styles xmlns:w=\"{w}\">"),
+        "</w:styles>",
+        100_000,
+        BIG_DEFAULTS,
+    );
+    let docx = docx_with_bin_aux_part(
+        "<w:p><w:r><w:t>RenamedDeep</w:t></w:r></w:p>",
+        "styles",
+        "parts/look.bin",
+        &styles,
+    );
+    let pdf = docx_to_pdf(&docx).expect("an over-depth styles part renders without it");
+    assert!(pdf_page_count(&pdf) >= 1);
+    let text = String::from_utf8_lossy(&pdf);
+    assert!(
+        !text.contains("20.00 Tf") && !text.contains("20.10 Tf"),
+        "the over-depth part's docDefaults must not apply"
+    );
+}
+
+/// The same gate on every other relationship-selected XML consumer: each
+/// one parses its part with the same permissive parser, and each part's
+/// name is the producer's.
+#[test]
+fn renamed_aux_parts_past_the_depth_budget_render_without_them() {
+    let body = "<w:p><w:r><w:t>AuxDeep</w:t></w:r></w:p>";
+    for (rel_type, target, root_open, root_close) in [
+        (
+            "theme",
+            "parts/theme1.bin",
+            "<a:theme xmlns:a=\"urn:x\">",
+            "</a:theme>",
+        ),
+        (
+            "numbering",
+            "parts/numbering.bin",
+            "<w:numbering>",
+            "</w:numbering>",
+        ),
+        (
+            "settings",
+            "parts/settings.bin",
+            "<w:settings>",
+            "</w:settings>",
+        ),
+        ("fontTable", "parts/fonts.bin", "<w:fonts>", "</w:fonts>"),
+        (
+            "comments",
+            "parts/comments.bin",
+            "<w:comments>",
+            "</w:comments>",
+        ),
+        (
+            "commentsExtended",
+            "parts/commentsExtended.bin",
+            "<w15:commentsExtended>",
+            "</w15:commentsExtended>",
+        ),
+        (
+            "footnotes",
+            "parts/footnotes.bin",
+            "<w:footnotes>",
+            "</w:footnotes>",
+        ),
+        (
+            "endnotes",
+            "parts/endnotes.bin",
+            "<w:endnotes>",
+            "</w:endnotes>",
+        ),
+    ] {
+        let part = deep_part(root_open, root_close, 100_000, "");
+        let docx = docx_with_bin_aux_part(body, rel_type, target, &part);
+        let pdf = docx_to_pdf(&docx).unwrap_or_else(|e| {
+            panic!("an over-depth {rel_type} part must render without it: {e}")
+        });
+        assert!(pdf_page_count(&pdf) >= 1, "{rel_type}");
+    }
+}
+
+/// Header and footer parts are chosen by relationship id, not by part
+/// type: the gate must cover that route too.
+#[test]
+fn a_renamed_header_part_past_the_depth_budget_renders_without_it() {
+    let body = "<w:p><w:r><w:t>HeadDeep</w:t></w:r></w:p>\
+         <w:sectPr><w:headerReference w:type=\"default\" r:id=\"rIdAux\"/></w:sectPr>";
+    let part = deep_part("<w:hdr>", "</w:hdr>", 100_000, "");
+    let docx = docx_with_bin_aux_part(body, "header", "parts/hdr1.bin", &part);
+    let pdf = docx_to_pdf(&docx).expect("an over-depth header part renders empty");
+    assert!(pdf_page_count(&pdf) >= 1);
+}
+
+// ─── table grid budgets (dedup-0002) ────────────────────────────────────────
+
+/// A cell's `w:gridSpan` is an ordinary integer attribute: a tiny table
+/// used to size its column vectors straight from the value, so a document
+/// under every byte budget could ask the layout for a ~16 GiB allocation.
+/// The grid is clamped before any allocation
+/// (resource-exhaustion.table-grid-span).
+#[test]
+fn a_huge_grid_span_stays_inside_the_column_budget() {
+    let docx = minimal_docx_body(
+        "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid>\
+         <w:tr><w:tc><w:tcPr><w:gridSpan w:val=\"4294967295\"/></w:tcPr>\
+         <w:p><w:r><w:t>wide</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:sectPr/>",
+    );
+    let pdf = docx_to_pdf(&docx).expect("a huge gridSpan renders inside the column budget");
+    assert!(pdf_page_count(&pdf) >= 1);
+}
+
+/// `w:gridBefore`/`w:gridAfter` are the same numeric class: a row skip of
+/// 1 073 741 824 columns asked for a ~4 GiB grid.
+#[test]
+fn a_huge_grid_before_stays_inside_the_column_budget() {
+    let docx = minimal_docx_body(
+        "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid>\
+         <w:tr><w:trPr><w:gridBefore w:val=\"1073741824\"/></w:trPr>\
+         <w:tc><w:p><w:r><w:t>after</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:sectPr/>",
+    );
+    let pdf = docx_to_pdf(&docx).expect("a huge gridBefore renders inside the column budget");
+    assert!(pdf_page_count(&pdf) >= 1);
+}
+
+/// Many moderate spans sum past any Word-producible grid (Word stops at 63
+/// columns): the summed occupancy must stay clamped, and the checked sum
+/// must not overflow.
+#[test]
+fn summed_row_spans_stay_inside_the_column_budget() {
+    let mut row = String::new();
+    for _ in 0..3000 {
+        row.push_str("<w:tc><w:tcPr><w:gridSpan w:val=\"1000000\"/></w:tcPr><w:p/></w:tc>");
+    }
+    let docx = minimal_docx_body(&format!(
+        "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{row}</w:tr></w:tbl><w:sectPr/>"
+    ));
+    let pdf = docx_to_pdf(&docx).expect("a row's summed spans render inside the column budget");
+    assert!(pdf_page_count(&pdf) >= 1);
+}
+
+/// The control: the spans and skips a real Word table uses still lay out
+/// with their merges (the budget clamps nothing Word can produce).
+#[test]
+fn ordinary_spans_and_skips_still_lay_out() {
+    let docx = minimal_docx_body(
+        "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/>\
+           <w:gridCol w:w=\"2000\"/></w:tblGrid>\
+         <w:tr><w:trPr><w:gridBefore w:val=\"1\"/></w:trPr>\
+           <w:tc><w:tcPr><w:gridSpan w:val=\"2\"/></w:tcPr>\
+           <w:p><w:r><w:t>merged</w:t></w:r></w:p></w:tc></w:tr>\
+         <w:tr><w:tc><w:p><w:r><w:t>one</w:t></w:r></w:p></w:tc>\
+           <w:tc><w:p><w:r><w:t>two</w:t></w:r></w:p></w:tc>\
+           <w:tc><w:p><w:r><w:t>three</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:sectPr/>",
+    );
+    let pdf = docx_to_pdf(&docx).expect("a normal spanned table converts");
+    assert!(pdf_page_count(&pdf) >= 1);
+}
diff --git a/tests/input_admission.rs b/tests/input_admission.rs
index 2f9769e..ab78815 100644
--- a/tests/input_admission.rs
+++ b/tests/input_admission.rs
@@ -534,6 +534,69 @@ fn scrub_refuses_an_excess_declared_count_before_indexing() {
     assert!(err.to_string().contains("10013 ZIP entries"), "{err}");
 }
 
+/// Append a false end-of-central-directory record as the archive's
+/// comment: its count field declares `declared` entries and its directory
+/// offset is impossible, so the ZIP reader rejects the candidate and opens
+/// the real footer, while the pre-index count — taken from the last
+/// EOCD-like signature — trusts the false one (dedup-0002:
+/// zip-entry-budget.eocd-selection-differential).
+fn with_false_eocd_comment(mut docx: Vec<u8>, declared: u16) -> Vec<u8> {
+    let end = docx.len() - 22;
+    assert_eq!(&docx[end..end + 4], b"PK\x05\x06");
+    // The real footer's comment covers exactly the false record.
+    docx[end + 20..end + 22].copy_from_slice(&22u16.to_le_bytes());
+    let mut false_eocd = [0u8; 22];
+    false_eocd[..4].copy_from_slice(&[0x50, 0x4b, 0x05, 0x06]);
+    false_eocd[10..12].copy_from_slice(&declared.to_le_bytes());
+    // A directory offset at/past the record itself cannot name a real
+    // central directory: zip rejects the candidate and retries the real
+    // footer (zip 9.0.5, find_central_directory.rs).
+    false_eocd[16..20].copy_from_slice(&0x7fff_ffffu32.to_le_bytes());
+    docx.extend_from_slice(&false_eocd);
+    docx
+}
+
+/// The pre-index entry budget reads the last EOCD-like signature; the ZIP
+/// reader validates its candidate and may open an earlier, real footer
+/// instead. The archive that was actually opened must obey the entry
+/// budget too, or a false footer declaring three entries floats a real
+/// crowd of entries past admission.
+#[test]
+fn admit_refuses_the_opened_count_behind_a_false_eocd_comment() {
+    let crowded = with_false_eocd_comment(package_with_entries(30), 3);
+    let err = jubarte::admission::admit(&crowded, small_limits()).expect_err(
+        "the real directory's 33 entries must refuse although the false footer declares 3",
+    );
+    assert_eq!(err.code(), "INPUT_LIMIT", "{err}");
+    assert!(err.to_string().contains("33 ZIP entries"), "{err}");
+}
+
+/// The bounded package constructor (`PartFs::open`, behind every reader
+/// that does not carry its own budget) repeats the pre-index gate, so it
+/// needs the same opened-count check.
+#[test]
+fn partfs_open_refuses_the_opened_count_behind_a_false_eocd_comment() {
+    let crowded = with_false_eocd_comment(package_with_entries(10_010), 3);
+    let Err(err) = jubarte::opc::PartFs::open(&crowded) else {
+        panic!("the real directory's 10013 entries must refuse the constructor")
+    };
+    let text = err.to_string();
+    assert!(text.contains("INPUT_LIMIT"), "{text}");
+    assert!(text.contains("10013 ZIP entries"), "{text}");
+}
+
+/// The control: a false footer on a package whose real directory sits
+/// inside the budget still opens. Comment bytes that merely look like a
+/// record must not break a valid archive, nor refuse an honest one.
+#[test]
+fn a_false_eocd_comment_on_a_small_package_still_opens() {
+    let docx = with_false_eocd_comment(package(|_, _| {}), 3);
+    let admitted = jubarte::admission::admit(&docx, small_limits())
+        .expect("the real 3-entry directory admits despite the comment");
+    assert_eq!(admitted.entries, 3);
+    jubarte::opc::PartFs::open(&docx).expect("PartFs::open reads the real directory");
+}
+
 /// `leaks` has no refusal channel, so it fails closed: a package past the
 /// admission budget is reported as a finding, never scanned partially
 /// (#645). `edit`'s redaction check turns the finding into a refusal.
```
