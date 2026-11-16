<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Baseline addenda

Hand-maintained measurements appended after T0.1. `scripts/split_baseline.sh
record` reproduces this file verbatim at the end of `baseline.md`, so a
re-recording never loses these entries; add new ones below, newest first.

## T4.4 (jubarte-render extracted) - 2026-11-16

- Fourth workspace crate: the render product (src/convert, now
  crates/jubarte-render/src/convert) with the 16 embedded TTFs and both
  font license files moved beside it (D6; assets/fonts/extra stays at
  the root for release.yml and install.sh, both untouched - AC2). The
  facade re-exports jubarte_render::convert; the option types still
  live in jubarte-opc's render_options and convert re-exports them, so
  every jubarte::convert:: path holds. The render crate's deps: the two
  floors plus ttf-parser, rustybuzz, tiny-skia, image, flate2, zip, and
  tempfile for its tests.
- AC1: the render tree names none of jubarte-redlines, similar,
  pulldown-cmark, schemars or clap. AC3: cargo package -p
  jubarte-render lists exactly 16 .ttf. LayoutFacts and its fields
  gained the V2 promotion (the fields layer reads them).
- Font-path follow-ups: release_metadata's font paths, the include
  guard and the font-dependency guard now read the render crate's
  manifest; convert_docx_to_pdf's Carlito include rebased; REUSE.toml,
  LICENSES.md and SRC_MANIFEST.md paths follow; the evidence-engine and
  engine-commit scripts hash crates/ now, with the watched-input test
  extended to a crates fixture. The root include drops /assets/fonts/**
  (extra stays in the repository; the render crate ships its own).
- A listing lesson for T4.5+: a library's own decorative root
  re-exports rebase into phantom facade paths - jubarte-render's
  `pub use convert::{...}` made jubarte::PdfOptions look nameable at
  the root. The re-exports were dropped (convert already provides every
  path) and the listing re-recorded clean (21 reviewed lines).
- Battery: S1 both feature sets, clippy both, fmt, RUSTDOCFLAGS doc,
  guards 33/33, goldens identical (the renderer moved byte-faithfully:
  every PDF golden reproduced). tuning_audit and docs/TUNING_AUDIT.md
  follow the moved tree (the parser accepts crates/ rows; 136 doc paths
  rebased), and the evidence scripts hash crates/ now. Three
  path-depth lessons found by the full suite, all fixed: the finalize
  snapshot and Strict01 census anchor to CARGO_MANIFEST_DIR/../.., and
  the supplemental-font folder helper is TWO up from the crate (a
  first three-up fix pointed at the repository's parent - the
  instrumented probe caught it). Full workspace suite: 10,691/10,691.

## T4.3b (jubarte-redlines extracted) - 2026-11-16

- Third workspace crate: the compare product (comparer,
  document_comparer, revision_processor, wml_document, comparison_log)
  on jubarte-opc and jubarte-dom through the A5 shims. accept/+
  reject_revisions_keeping_comments moved into the doc layer's
  changes.rs first, and the facade composes jubarte::document_comparer
  per A6 (the crate's module plus those two), so every public path
  holds. The crate re-exports admission/opc/strict_translation/
  xmllinq/namespaces (A2): one dependency is enough for a consumer.
- schemars left both floors by feature: jubarte-dom's ids and
  jubarte-opc's settings derive JsonSchema only under `schema`
  (forwarded opc -> dom), which the root enables for the edit plan's
  schema. The compare tree is clean of renderer, markdown and CLI deps
  (AC1: zero matches).
- KNOWN_UPWARD loses document_comparer.rs -> changes (the file moved);
  one entry remains, the documented debug.rs -> text_diff deviation.
- The include rebasing across the moved tree needed an existence-tested
  rewrite (a first regex pass corrupted three files; restored from git
  and redone per-file - lesson recorded for T4.4+).
- gen_facade_surface skips glob and anonymous re-export lines (a
  composed module's `pub use ...::*` lines cannot take `as _`).
- Battery: S1 both feature sets; AC1 tree clean; AC2 guards green;
  AC3 a scratch crate comparing the fixture pair on the crate alone;
  S3 listing re-baselined (62 reviewed lines) + facade_surface 888
  probes; S4 goldens identical; S6 clippy both, fmt, RUSTDOCFLAGS doc;
  AC4 bench gate: the required no-LTO consumer configuration regressed
  +32% to +56% on all four benchmarks (comparer's Dom calls now cross
  crate boundaries without cross-crate inlining); #[inline] on the nine
  hottest accessors moved nothing and was reverted. D8 applied: thin
  LTO at the root [profile.release] and [profile.bench] - the
  workspace's own builds IMPROVED 5-11% against pre-split on all four
  benchmarks. The no-LTO consumer gap stands, recorded here per the
  plan, for T4.9 to re-measure. (Release CLI with thin LTO:
  41,343,248 bytes.)

## T4.3a (root package renamed to jubarte) - 2026-11-15

- The root package is `jubarte` (D1); the [lib] section is gone (name
  and path are the defaults), documentation points at docs.rs/jubarte,
  and the four bindings depend on it under the new key with refreshed
  lockfiles. deny.toml's build-script entries, release.sh's mechanical
  names (semver-checks -p, the packaged crate tarball, the crates.io
  resume probe), the README crates.io/docs.rs badges and AGENTS.md's
  naming section follow. bump-version's badge and install-pin regexes
  accept both names now.
- Found along the way: a T4.0a typo in the semver lint table
  (repr_c_plain_enum_struct_variant_fields_reordered for
  repr_c_plain_struct_fields_reordered), library_readmes now resolves
  the workspace-inherited version, release_metadata's lock guard checks
  the jubarte entry, and jubarte-dom's wasm-only utc_now gained the V2
  promotion the native dead-code analysis could not see (the wasm
  binding compiles that cfg block).
- AC review of the remaining jubarte-redlines hits: GitHub repo URLs,
  PyPI/npm package names (python and wasm bindings, README installs,
  npx/pip instructions), the app lock check (T4.3b's world), benchmark
  data under tools/, committed fixtures, and codemod tests over
  historic trees.
- Battery: cargo metadata (jubarte, jubarte-dom, jubarte-opc); all
  four bindings check green from their own directories (wasm for
  wasm32); test_release_sh 282/282, test_release_workflow OK, bump
  tests 8/8 and the Python ones, semver_lints PASS, library_readmes
  regenerated and green; guards 15/15; clippy both feature sets, fmt,
  RUSTDOCFLAGS doc; goldens identical; full workspace suite
  10,691/10,691.

## T4.2 (jubarte-opc extracted) - 2026-11-15

- Second workspace crate: crates/jubarte-opc (opc, admission,
  markup_simplifier, strict_translation, settings, word_default_theme,
  builtin_styles, borders, data_stores, validate_dom, render_options,
  numbering, table_dom), on jubarte-dom through the A5 shims (the dom
  modules under their old crate:: names) so no moved line changed paths.
  The facade re-exports the five public modules (#[doc(inline)] pub use)
  and keeps the eight private ones as pub(crate) use.
- Plan correction: markdown/data_stores.rs (the writer's store builder)
  could NOT merge into data_stores - its builder walks the writer's
  Element model (super::from_docx). It stays a markdown module
  (MODULES maps markdown::data_stores -> markdown now); only the
  opc-clean lookup half is in jubarte-opc.
- V2 promotions: 81 by the dead-code list plus the workspace loop's
  (validate_dom's keeps_words/table_revised family, admission's
  preindex_budget/tally_inflated/shown/part_declared_over,
  opc's refused/open_within/resolve_rel_target and the ZIP archive
  probes, borders' parse/xml/with_effects, table_dom's TblLook fields,
  zip32_plan). admission's test-support helpers (zip_names,
  limit_test_support) moved to cfg(any(test, feature = "test-support"))
  per A4, and admission's checked-math deny now exempts that feature
  too (test scaffolding, not untrusted input). settings' test fixture
  path rebased three directories up (a cfg(test) include, allowed by E5).
- rustdoc: short-form intra-doc links in module headers stopped
  resolving inside the crate (explicit crate:: paths work; recorded
  here for T4.3+ to reuse the pattern), and one public doc's link to a
  private helper became backticked text (A5).
- api-baseline re-recorded with a 342-line reviewed drift: same classes
  as T4.1 (doc(hidden) items leave; signatures re-render through the
  new crate paths) plus the five facade re-export use lines.
- Battery: check -p no-default-features and all-features, clippy both
  feature sets, fmt, RUSTDOCFLAGS doc, tree (dom + opc only), guards,
  facade_surface, goldens identical, full workspace suite (below).

## T4.1 (jubarte-dom extracted) - 2026-11-15

- First workspace crate: crates/jubarte-dom (xmllinq, namespaces, ids,
  unid, perf, util, ooxml, limits) with its own Cargo.toml (workspace
  tables, =0.18.0 pin, perf-profile and test-support features),
  README and LICENSE copy. The facade re-exports every public module at
  its jubarte:: path (doc(inline) pub use) and the two private ones as
  pub(crate) use, so no call site changed (D9).
- Cross-layer internals became #[doc(hidden)] pub (V2, 43 promotions by
  the dead-code list plus nine by the workspace check: hint_where,
  well_known_prefix, is_namespace_prefix_list, TagPolicy with
  new/open/empty/close, Dom::set_attributes, CivilDateTime's fields),
  xmllinq::shrink_arena_to_fit moved behind
  cfg(any(test, feature = "test-support")) (A4), and util::datetime
  opened as a doc(hidden) module.
- coverage_gates accepts crates/*/src roots now (the plan's file list
  said so).
- api-baseline re-recorded with a 168-line reviewed drift: rustdoc JSON
  omits doc(hidden) items entirely, so the V2 promotions and the
  limits/ooxml subtrees leave the listing (their types stay nameable
  through the public re-exports that always carried them, and the
  compile is the stronger gate); the facade listing adds the six
  re-export use lines. The T4.0 cross-crate machinery carries
  everything else: each library's own listing joins the facade listing
  rebased under jubarte:: paths, and cargo doc now builds per package
  (-p), because a bare workspace doc run writes only the primary
  crate's JSON.

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
