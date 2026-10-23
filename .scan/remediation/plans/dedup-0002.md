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
