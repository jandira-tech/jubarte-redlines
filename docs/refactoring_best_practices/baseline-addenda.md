<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Baseline addenda

Hand-maintained measurements appended after T0.1. `scripts/split_baseline.sh
record` reproduces this file verbatim at the end of `baseline.md`, so a
re-recording never loses these entries; add new ones below, newest first.

## T4.0 (workspace scaffolding, API tooling) - 2026-11-15

- Root manifest is the workspace manifest (A3): members ["."], the four
  binding workspaces excluded, workspace.package and workspace.lints
  tables, every dependency line moved verbatim to workspace.dependencies.
  cargo metadata: 1 package.
- api_snapshot.py: the doc(hidden) filter is in (no current item leaves
  the listing - the five hidden sites in comparer/mod.rs never reached
  the api surface) and the cross-crate re-export resolution is in
  (ForeignIndex over the workspace's other libraries; plain uses list
  the foreign item under its facade path, glob uses expand the foreign
  module's public children). 38 script tests pass, including the two
  new synthetic cross-crate cases.
- tests/facade_surface.rs: 886 existence-probe uses generated from the
  api listing by scripts/gen_facade_surface.py (module-chain-public
  filter, associated items excluded, rustfmt-ordered). Negative probe:
  hiding pub mod namespaces breaks its build with 126 errors.
- bump-version: the workspace version and the = pins bump together
  (node --test 8/8).
- api-baseline re-recorded: +18 internal lines (T3.1 font statics, T3.2
  staging_suffix), the only additions since T2.5.

## T3.1 (font statics) - 2026-11-15

- FontId::bytes() now returns 16 module-level statics (one per distinct
  font program) instead of 47 inline include_bytes! arms. Release CLI
  40,358,448 bytes vs 40,409,312 baseline (smaller); after the post-commit
  build-npm.sh rebuild the full and slim wasm are 23,163,493 and
  23,112,788 bytes - byte-identical sizes to the baseline (wasm-opt
  already merged the duplicate programs). Goldens reproduced.

## T2.5 (render_options) - 2026-11-15

- MarkLines, RevisionMark, RevisionPalette (+ both impls), RevisionStyle
  (+ impl), PdfOptions, CommentPlacement and RenderRequest moved to the
  pub(crate) module crate::render_options (opc layer); convert re-exports
  them, so every jubarte::convert::… path still resolves (doctests
  14/14). Plan correction: there is no RevisionColor type; the palette's
  colours are [u8; 3] fields on RevisionMark.
- api-baseline re-recorded: the listing keys items by their DEFINING
  module, so 151 public lines renamed convert::… -> render_options::…
  even though the convert paths remain valid through pub use. Same shape
  as the end state (jubarte-opc::render_options owning the types).

### Render inventory (T2.5 deliverable)

Public items above the renderer whose bodies (not just their option
types) call into convert, to be gated with `#[cfg(feature = "render")]`
by T4.5-T4.8:

- doc layer
  - jubarte::audit::word_layout (audit.rs:147) calls
    docx_render_report and returns convert's RenderReport; LayoutPass
    (audit.rs:161) and the audit helpers at 276-277 carry it too.
  - jubarte::fields: FieldError::Layout (fields.rs:79) holds a
    ConvertError; update_fields (115) calls layout_facts (170);
    LayoutFacts appears in the public update machinery (257) and tests
    (1499, 2299).
  - jubarte::debug's render check (debug/render.rs:1162 calls
    docx_render_report).
- markdown layer
  - jubarte::markdown::page_texts (markdown/mod.rs:1098) calls
    convert::render (1113); markdown/pages.rs:47 links
    RenderReport::pages in docs only.
- edit layer
  - capabilities.rs links convert::docx_to_pdf / docx_to_png /
    diff_render in doc comments only (125, 127, 155); no body calls.

## T2.4 (numbering, table_dom) - 2026-11-15

- api-baseline.txt re-recorded again: the w:numFmt label family
  (NumFmt, format_num and 23 label functions), append_numbering with
  numbering_part/max_attribute/attribute_values, and the four table DOM
  helpers (table_look/TblLook/cell_is_deleted/wrapped_children) moved to
  the pub(crate) modules crate::numbering and crate::table_dom, so the
  listing's internal lines renamed with them. All 259 diff lines are
  internal; no publicly nameable path changed.

## T2.3 (comment_settlement) - 2026-11-15

- api-baseline.txt re-recorded after a reviewed, intentional drift: the
  module moved from the private `jubarte::edit::comment_settlement` path to
  the private `jubarte::comment_settlement` path, so the listing's internal
  lines renamed with it, `changes::ResolvedChanges::kept_comments` renders
  its element type through the new path, and `EditError` gained a
  `From<SettlementError>` impl. No publicly nameable path changed:
  `jubarte::edit::{KeptComment, RemovedComment}` still resolve (re-export)
  and no public item was added or lost (diff reviewed line by line).
- Plan deviation recorded on #2428: the plan's critic_literal move to
  debug.rs is infeasible (it calls markdown's CriticMarkup reader through
  diff_markdown; defining it in the doc layer would trade the
  debug.rs->text_diff edge for a worse debug.rs->markdown edge). The edge
  stays in KNOWN_UPWARD until Phase 4 composes the critic view.

## T1.1 (test suites) - 2026-11-15

- Test cases: 10,692 before and after the move (nothing lost, nothing added
  beyond `suite_membership`'s 2 tests, which the baseline count already
  included because the file existed when T0.1 was re-recorded).
- Test binaries: 599 before (597 top-level test files + lib + bin) -> 30
  after (21 standalone + 7 suites + lib + bin).
- Suite membership: it_cli 78, it_core 17, it_doc 25, it_edit 46,
  it_markdown 55, it_redlines 351, it_render 4 (576 moved). The plan's A8
  expectation of it_edit 45 / it_markdown 56 differs by one file,
  `markdown_redline.rs`, whose `jubarte::markdown::redline` references map
  to the edit layer through A8's own submodule override.
- `cargo nextest run --all-features --no-run` after the move, cache-hot:
  3 s wall (T0.1 recorded 122 s cache-hot with 599 binaries; the binary
  count, not the suite split, is what changed).
