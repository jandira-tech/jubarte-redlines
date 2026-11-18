# Repository Reorganization, Crate Split & Optimization Plan (v4)

> **Status legend** (added 2026-11-14): `- [x]` = done, `- [ ]` = pending.
> "Done (in-crate)" means the code was extracted into the corresponding
> `src/` module (`src/limits.rs`, `src/ooxml.rs`, `src/util/datetime.rs`,
> `src/validate_dom.rs`, `src/admission.rs`, `src/markup_simplifier.rs`,
> `src/borders.rs`, `src/data_stores.rs`) and the original paths re-export it;
> the physical move into `crates/jubarte-*` happens in the crate-split phase.
> `tests/crate_layering.rs` enforces the target layering in the meantime.
>
> **Status audit 2026-11-16 (updated)**: Phase 1–3 extraction is complete
> through T4.9 (commits `ce4a8740b`–`01cf08020`); all eight
> `crates/jubarte-*` crates exist, `jubarte-cli` is extracted (T4.8,
> commits `6d105445a`/`0bb1de9a5`), and the facade + workspace DAG gates
> landed (T4.9, commits `d644a058c`/`01cf08020`). The earlier T4.8
> stragglers are fixed: `crates/jubarte-cli/src/update.rs:1823` now reads
> `../../../tests/...`, and `crates/jubarte-cli/build.rs` resolves the repo
> root via `CARGO_MANIFEST_DIR/../..` so `JUBARTE_BUILD_VERSION` keeps its
> `+<sha>` suffix. Remaining open work: §2.3 WASM feature split, §2.4
> Python dual packaging, `oxml-opc`/`quick-xml-041` retirement (§4), all
> five mega-file decompositions (§5), §7 workspace unification for the
> binding crates (still `exclude`d), per-crate `cargo publish`, and the
> root `ISSUE_TRIAGE_REPORT.md` move.

## 1. Executive Summary & Architectural Goals

`jubarte-redlines` has evolved from a Word-mode document comparison library into a **315,000+ LOC Word `.docx` platform** encompassing comparison (`redlines`), DOCX-to-PDF/PNG layout and rendering (`convert`), document inspection/validation/repair/append/scrub/fields, Markdown/CriticMarkup round-tripping, agentic editing (`EditPlan`), and a full CLI (`jubarte`), plus Python (`jubarte-python`), WebAssembly (`jubarte-wasm`), and benchmark (`jubarte-rust-inproc`) bindings:

- **Single monolithic root crate**: Every consumer compiles all 315k LOC across 122 source files. Users who only want **compare (`redlines`)** must compile the 69k LOC PDF renderer (`rustybuzz`, `ttf-parser`, `tiny-skia`, `image`) and embed **17 MB of uncompressed TTF fonts (`Carlito`, `Liberation`)**. Conversely, users who only want **DOCX-to-PDF/PNG conversion** must compile the 92k LOC comparison/LCS engine and the 25k LOC agentic edit engine.
- **Hidden WASM `slim` bloat**: Because `src/markdown/mod.rs` (`page_texts`) unconditionally calls `crate::convert::render`, `jubarte-wasm`'s `node-slim` and `web-slim` builds still link the entire PDF renderer and embedded fonts (**23 MB `.wasm`**, identical in size to the full build!).
- **4 disconnected Cargo workspaces & 4 `Cargo.lock` files**: Root `Cargo.toml`, `jubarte-python`, `jubarte-wasm`, and `jubarte-rust-inproc` each maintain separate workspaces and lockfiles (`fuzz` is a 5th).
- **595 separate integration test binaries in `tests/*.rs`**: Cargo links 595 independent test executables (and compiles `tests/common/mod.rs` 222 separate times), causing massive link-time bottlenecks and `target/` bloat.
- **5 mega-files accounting for 153,000 LOC (48% of the codebase)**:
  - [src/convert/mod.rs](../../src/convert/mod.rs) (69,056 LOC)
  - [src/comparer/finalize.rs](../../src/comparer/finalize.rs) (35,346 LOC)
  - [src/comparer/lcs.rs](../../src/comparer/lcs.rs) (21,290 LOC)
  - [src/document_comparer.rs](../../src/document_comparer.rs) (14,961 LOC)
  - [src/edit.rs](../../src/edit.rs) (12,374 LOC)
- **Dependency & code redundancy**:
  - Dual `quick-xml` versions (`0.42` and `0.41` via `oxml-opc`), even though [src/opc/mod.rs](../../src/opc/mod.rs) only imports 4 trivial types (`Part`, `PartURI`, `Relationships`, `OpcError`) from `oxml-opc` ([docs/ZIP9_OXML_OPC_MIGRATION.md](../../docs/ZIP9_OXML_OPC_MIGRATION.md)).
  - `uuid` (`v4`) pulled into native builds solely for a temp filename random suffix in [src/publication.rs](../../src/publication.rs#L179).
  - Duplicate `include_bytes!` of `Carlito-*.ttf` and `LiberationSans-*.ttf` across two functions in [src/convert/font.rs](../../src/convert/font.rs#L397-L561).

---

## 2. Target Multi-Crate & Multi-Language Product Architecture

### 2.1 Crate Dependency DAG (Strictly Acyclic in Production AND `[dev-dependencies]`)

```mermaid
graph TD
    DOM["jubarte-dom<br/>(xmllinq, namespaces, ooxml,<br/>ids, unid, limits, perf, util)"]
    OPC["jubarte-opc<br/>(opc, admission, markup_simplifier,<br/>strict_translation, settings, borders,<br/>data_stores, validate_dom, render_options)"]
    REDLINES["jubarte-redlines<br/>(Compare Product: comparer,<br/>document_comparer, revision_processor,<br/>wml_document, comparison_log)"]
    RENDER["jubarte-render<br/>(PDF/PNG Convert Product: convert,<br/>layout, fonts, pdf, raster)"]
    DOC["jubarte-doc<br/>(changes, comments, validate, repair,<br/>audit, scrub, append, fields, debug,<br/>inspect, comment_settlement)"]
    MD["jubarte-markdown<br/>(markdown reader/writer, pages,<br/>source, legacy_doc)"]
    EDIT["jubarte-edit<br/>(edit plan v1/v2, patch, redline,<br/>unified, text_diff, expect,<br/>capabilities, publication, result)"]
    CLI["jubarte-cli<br/>(clap args, edit_actions, inline,<br/>update/zipsign)"]
    FACADE["jubarte (root facade + bin)<br/>(lib: jubarte, bin: jubarte)"]

    OPC --> DOM
    REDLINES --> DOM
    REDLINES --> OPC
    RENDER --> DOM
    RENDER --> OPC
    DOC --> DOM
    DOC --> OPC
    DOC --> REDLINES
    DOC -. "optional (default 'render' feature)" .-> RENDER
    MD --> DOM
    MD --> OPC
    MD --> REDLINES
    MD -. "optional (default 'render' feature)" .-> RENDER
    MD --> DOC
    EDIT --> DOM
    EDIT --> OPC
    EDIT --> REDLINES
    EDIT -. "optional (default 'render' feature)" .-> RENDER
    EDIT --> DOC
    EDIT --> MD
    CLI --> DOM
    CLI --> OPC
    CLI --> REDLINES
    CLI --> DOC
    CLI --> MD
    CLI --> EDIT
    FACADE --> DOM
    FACADE --> OPC
    FACADE --> REDLINES
    FACADE --> RENDER
    FACADE --> DOC
    FACADE --> MD
    FACADE --> EDIT
    FACADE --> CLI
```

### 2.2 Workspace Crate Responsibilities & External Dependencies

| Crate | Path | Library Name | Purpose & Modules | External Dependencies |
| :--- | :--- | :--- | :--- | :--- |
| **`jubarte-dom`** | `crates/jubarte-dom` | `jubarte_dom` | Arena XML DOM (`xmllinq`), `namespaces`, `ooxml` (constants, `sect_rank`, `text_box_names`, `HIGHLIGHTS`), `ids`, `unid`, `limits` (`InputLimits`, `InputBudget`, `UnsupportedContentError`), `perf`, `util` (`datetime` including `utc_now_iso8601`, `source_sha256`, `decode_entities`, word tokens). | `quick-xml` (0.42), `foldhash`, `unicode-segmentation`, `unicode-properties`, `serde`, `serde_json`, `sha2`, `js-sys` (`wasm32-unknown-unknown` only) |
| **`jubarte-opc`** | `crates/jubarte-opc` | `jubarte_opc` | Native `zip 9` OPC package reader/writer (`opc`), `admission` (including `admit_package`), `markup_simplifier` (including `mc:AlternateContent` helpers), `strict_translation`, `settings`, `word_default_theme`, `builtin_styles`, `borders`, `data_stores`, `validate_dom` (9 pure DOM structural predicates), `render_options` (`PdfOptions`, `RenderRequest`, `RevisionStyle`, `RevisionPalette`, `RevisionColor`, `MarkLines`, `CommentPlacement`). | `jubarte-dom`, `zip` (9.0.5), `sha1`, `sha2`, `serde`, `serde_json` |
| **`jubarte-redlines`** | `crates/jubarte-redlines` | `jubarte_redlines` | **Standalone Compare / Redlines Product Crate**: `comparer`, `document_comparer`, `revision_processor`, `wml_document`, `comparison_log`, plus re-exported `dom`, `opc`, `admission`. Zero PDF/font dependencies. | `jubarte-dom`, `jubarte-opc`, `serde`, `serde_json`, `sha1`, `sha2`, `base64` |
| **`jubarte-render`** | `crates/jubarte-render` | `jubarte_render` | **Standalone Convert-to-PDF/PNG Product Crate**: `convert` (`docx_to_pdf`, `docx_to_png`, `render_docx`, `diff_render`, `PdfOptions`, `RevisionStyle`, layout engine, TTF subsetting, PDF writer, PNG rasterizer). Zero compare/edit dependencies. | `jubarte-dom`, `jubarte-opc`, `ttf-parser`, `rustybuzz`, `tiny-skia`, `image`, `flate2`, `serde`, `serde_json` |
| **`jubarte-doc`** | `crates/jubarte-doc` | `jubarte_doc` | Document operations: `changes` (including `accept_revisions_keeping_comments` / `reject_revisions_keeping_comments`), `comments`, `validate`, `repair`, `audit`, `scrub`, `append`, `fields`, `debug` (including `critic_literal`), `inspect`, `comment_settlement`. | `jubarte-dom`, `jubarte-opc`, `jubarte-redlines`, `jubarte-render` (optional via `render` feature), `serde`, `serde_json`, `sha2` |
| **`jubarte-markdown`** | `crates/jubarte-markdown` | `jubarte_markdown` | Markdown/CriticMarkup reader & writer (`markdown::from_docx`, `markdown::write`, `markdown::diff`, `markdown::pages`, `markdown::source`), `legacy_doc` (`.doc` OLE reader). | `jubarte-dom`, `jubarte-opc`, `jubarte-redlines`, `jubarte-render` (optional via `render` feature), `jubarte-doc`, `pulldown-cmark`, `similar`, `regex`, `base64`, `serde`, `serde_json`, `sha2` |
| **`jubarte-edit`** | `crates/jubarte-edit` | `jubarte_edit` | Agentic `EditPlan` v1/v2 (`edit`), Markdown patching (`markdown::patch`, `markdown::redline`, `markdown::unified`), `text_diff`, `expect`, `capabilities`, `publication`, `result`. | `jubarte-dom`, `jubarte-opc`, `jubarte-redlines`, `jubarte-render` (optional via `render` feature), `jubarte-doc`, `jubarte-markdown`, `schemars`, `similar`, `regex`, `serde`, `serde_json`, `sha2`, `cap-std` & `xattr` (non-wasm) |
| **`jubarte-cli`** | `crates/jubarte-cli` | `jubarte_cli` | Shared CLI parser & dispatch (`cli`, `cli::edit_actions`), typed inline action compiler (`inline`), `update` (`self_update` + `ed25519-dalek` zipsign). | `jubarte-{dom,opc,redlines,doc,markdown,edit}`, `clap`, `same-file`, `self_update` (optional), `ed25519-dalek` (optional) |
| **`jubarte`** | `.` (root) | `jubarte` (`bin: jubarte`) | **Umbrella Facade Crate & CLI Binary**: Re-exports the entire `jubarte::*` public surface with `#[doc(inline)]` and builds the `jubarte` binary. | `jubarte-*` (all library crates unconditionally linked; `default = ["cli", "fast-alloc", "self-update"]`), `mimalloc` (optional via `fast-alloc`) |

> [!NOTE]
> **Preserving `--no-default-features` & `crates.io` / `jubarte-app` Compatibility**:
> - On root `jubarte`, all library sub-crates (`jubarte-dom`, `jubarte-opc`, `jubarte-redlines`, `jubarte-render`, `jubarte-doc`, `jubarte-markdown`, `jubarte-edit`) remain unconditionally compiled in the library target, while `default = ["cli", "fast-alloc", "self-update"]` gates only the CLI/allocator/self-update dependencies. This guarantees `cargo clippy --all-targets --no-default-features` ([ci.yml:57](../../.github/workflows/ci.yml#L57)), `jubarte-app`, `fuzz`, and `scripts/check_no_network.py` continue to work without modification.
> - Rust users who want **compare only** (without PDF/fonts) depend on `jubarte-redlines`; users who want **convert to PDF/PNG only** (without compare/edit) depend on `jubarte-render`; users who want **the full suite** depend on `jubarte`.
> - For `crates.io` OIDC trusted publishing ([release.yml:472-518](../../.github/workflows/release.yml#L472-L518)), any newly created crate on `crates.io` (`jubarte-dom`, `jubarte-opc`, `jubarte-render`, `jubarte-doc`, `jubarte-markdown`, `jubarte-edit`, `jubarte-cli`, `jubarte`) must be published once with a `CARGO_REGISTRY_TOKEN` API token and have `jandira-tech/jubarte-redlines` (`release.yml`, environment `release`) registered in its `crates.io` *Settings → Trusted Publishers* prior to the first automated OIDC release.

### 2.3 WebAssembly (`jubarte-wasm`) Product Split & Subpath Exports (Shrinking `slim` from 23 MB to ~4 MB)

- [x] **Granular Feature Flags on [jubarte-wasm/Cargo.toml](../../jubarte-wasm/Cargo.toml) & [jubarte-wasm/src/lib.rs](../../jubarte-wasm/src/lib.rs)** — *done (verified 2026-11-17 on pre-release): `default = ["full", "console-panic"]`, with layered `compare` → `doc` → `markdown` → `edit` → `cli` features and a separate `pdf` (T5.1).*
  - `default = ["full", "console-panic"]`
  - `full = ["compare", "pdf", "doc", "markdown", "edit", "cli"]`
  - `slim = ["compare", "doc", "markdown", "edit", "cli"]` (compiles `jubarte-doc`, `jubarte-markdown`, `jubarte-edit` with `default-features = false` so `jubarte-render` and the 17 MB of uncompressed fonts are completely omitted, shrinking `node-slim`/`web-slim` from **23 MB to ~4 MB**!)
  - `redlines = ["compare"]` (pure compare/redlines product: `compareDocuments`, `acceptRevisions`, `rejectRevisions`, `getRevisions`)
  - `pdf` (standalone DOCX-to-PDF/PNG product: `docxToPdf`, `docxToPng`, `renderPdf`, `pdfPageCount`, `diffRender`)
  - Every `#[wasm_bindgen]` entrypoint and `#[cfg(test)]` test in [jubarte-wasm/src/lib.rs](../../jubarte-wasm/src/lib.rs) is gated by its corresponding feature (`compare`, `pdf`, `doc`, `markdown`, `edit`, `cli`; with `convert_markdown` gated by `#[cfg(all(feature = "pdf", feature = "markdown"))]`).
- [x] **Subpath Exports & Module `package.json` Files in [jubarte-wasm/npm/](../../jubarte-wasm/npm/package.json)** — *done (verified 2026-11-17 on pre-release): `npm/` ships `node`, `web`, `*-slim`, `*-redlines` and `*-pdf`; `exports` names `./redlines`, `./pdf`, `./convert` and each build (T5.2).*
  - `jubarte-wasm` (`.` / `./node` / `./web`): Full `jubarte` WASM facade.
  - `jubarte-wasm/redlines` (`./node-redlines`, `./web-redlines`) and `jubarte-wasm/slim` (`./node-slim`, `./web-slim`): Redlines & document operations without PDF/fonts.
  - `jubarte-wasm/pdf` and `jubarte-wasm/convert` (`./node-pdf`, `./web-pdf`): Standalone DOCX-to-PDF/PNG converter without compare/edit.
  - Track `package.json` files inside `npm/node-redlines/`, `npm/node-pdf/` (`{"type": "commonjs"}`) and `npm/web-redlines/`, `npm/web-pdf/` (`{"type": "module", "sideEffects": ["./snippets/*"]}`) so Node ESM/CJS resolution in [jubarte-wasm/npm-smoke.mjs](../../jubarte-wasm/npm-smoke.mjs) succeeds.
- [x] **Dual CLI Binaries in [jubarte-wasm/cli/package.json](../../jubarte-wasm/cli/package.json)** — *done (verified 2026-11-17 on pre-release): `"bin"` exposes `"jubarte"` and `"jubarte-redlines"`, both on `bin/jubarte-redlines.mjs`.*
  - Expose both `"jubarte": "bin/jubarte-redlines.mjs"` and `"jubarte-redlines": "bin/jubarte-redlines.mjs"` in `"bin"`.

### 2.4 Python (`jubarte-python`) Dual-Namespace & Product Submodules

Publishing 3 separate native PyO3 C-extension distributions on PyPI is not advisable (it would triple the 7-wheel CI build matrix to 21 wheels, duplicate the compiled `.abi3.so`/`.pyd` binary in memory when both are installed, and break PyPI OIDC trusted publisher binding). Instead:

- [ ] **Dual Top-Level Packages (`jubarte` and `jubarte_redlines`) + Product Submodules (`jubarte.redlines`, `jubarte.convert`)** — *pending (verified 2026-11-17 on pre-release): `jubarte-python/python/` still contains only `jubarte_redlines/`; `[project.scripts]` has no `jubarte` entry.*
  - Add `jubarte-python/python/jubarte/` (`__init__.py`, `__main__.py`, `redlines.py`, `convert.py`, `py.typed`) alongside `jubarte-python/python/jubarte_redlines/`.
  - In [jubarte-python/pyproject.toml](../../jubarte-python/pyproject.toml), set `python-packages = ["jubarte_redlines", "jubarte"]` under `[tool.maturin]` so `maturin` bundles both top-level packages into every wheel!
  - Add `jubarte = "jubarte_redlines.__main__:main"` alongside `jubarte-redlines` and `jubarte-mcp` in `[project.scripts]`.
- [x] **Granular Cargo Features in [jubarte-python/Cargo.toml](../../jubarte-python/Cargo.toml)** — *done (verified 2026-11-17 on pre-release): `[features]` layers `redlines` → `doc` → `markdown` → `edit` → `cli` plus `render` over optional engine crates (T5.3). It still keeps its own `Cargo.lock`, and the `cfg(any())` `jubarte` entry stays until the root becomes a virtual manifest (after 0.19.0).*

---

## 3. Complete Cross-Crate Cycle & Upward-Call Elimination Matrix (Phase 1)

Every upward or peer cross-crate reference across all 122 files in `src/` (including `render`-disabled builds and inline `#[cfg(test)]` blocks) is resolved in Phase 1:

| # | Source File(s) & Line(s) | Current Target | Resolution | Status |
| :--- | :--- | :--- | :--- | :--- |
| **1** | `src/xmllinq/parse.rs:40`, `src/admission.rs:259` | `admission::InputLimits`, `document_comparer::UnsupportedContentError` | Move `InputLimits`, `InputBudget`, `InputLimitOverrides`, `AdmissionError`, `UnsupportedContentError` to `jubarte-dom::limits`; re-export in `admission` and `document_comparer`. | Done (verified 2026-11-16): `crates/jubarte-dom/src/limits.rs`; `jubarte-opc` re-exports it. |
| **2** | `src/convert/mod.rs:777, 19217, 22569, 22605` | `document_comparer::admit_package`, `comparer::finalize::{choice_requires_understood, word_understands_namespace, resolve_alternate_content_with}` | Move `admit_package` to `jubarte-opc::admission` and the 3 `mc:AlternateContent` helpers to `jubarte-opc::markup_simplifier`; re-export at original paths. | Done (verified 2026-11-16): `admit_package` and the `mc:AlternateContent` helpers live in `crates/jubarte-opc/` (`admission.rs`, `markup_simplifier.rs`). |
| **3** | `src/comparer/revisions.rs:186, 257, 378`, `src/changes.rs:1908..1929`, `src/changes/logical.rs:385`, `src/comments.rs:247..251`, `src/edit.rs:3284`, `src/edit/inline.rs:450`, `src/cli.rs:633`, `src/bin/jubarte.rs:3205..5500` | `changes::revision_date`, `convert::{utc_now_iso8601, timestamp_order_key, timestamp_with_offset, format_word_Timestamp, parse_iso_datetime, civil_from_days}` | Consolidate all ISO-8601 / Word datetime helpers (including `utc_now_iso8601` and its `wasm32` `js_sys::Date` branch) into `jubarte-dom::util::datetime`; re-export at original paths. | Done (verified 2026-11-16): all helpers consolidated in `crates/jubarte-dom/src/util/datetime.rs`. |
| **4** | `src/comparer/mod.rs:387, 390`, `src/revision_processor.rs:1143`, `src/markdown/{from_docx/mod.rs:2094, view_stories.rs:1739, write.rs:1687}` | `edit::sections::sect_rank`, `inspect::text_box_names`, `edit::HIGHLIGHTS` | Move `sect_rank`, `text_box_names`, and `HIGHLIGHTS` to `jubarte-dom::ooxml`; re-export at original paths. | Done (verified 2026-11-16): all three live in `crates/jubarte-dom/src/ooxml.rs`. |
| **5** | `src/comparer/parts.rs:188, 749`, `src/markdown/write.rs:1384, 1455` | `inspect::source_sha256`, `convert::altchunk::decode_entities` | Move `source_sha256` and `decode_entities` to `jubarte-dom::util`; re-export at original paths. | Done (verified 2026-11-16): both in `crates/jubarte-dom/src/util/`. |
| **6** | `src/revision_processor.rs:4491`, `src/comparer/deleted_tables.rs:32`, `src/comparer/final_marks.rs:42..331`, `src/document_comparer.rs:618, 619, 8835` | 9 pure DOM predicates in `validate` (`has_unique_revision_id`, `inserted_text_before_deleted_tables`, `keeps_words`, `table_revised`, `uncovered_final_marks`, `container_blocks`, `revision_name`, `closing_paragraph`, `holds_page_field`) | Move these 9 pure DOM structural predicate functions into `jubarte-opc::validate_dom`; re-export in `jubarte-doc::validate`. | Done (verified 2026-11-16): all 9 in `crates/jubarte-opc/src/validate_dom.rs`. |
| **7** | `src/document_comparer.rs:5480-5496` | `changes::{accept_changes_keeping_comments, reject_changes_keeping_comments, ResolvedChanges, ChangeError}` | Move `accept_revisions_keeping_comments` and `reject_revisions_keeping_comments` to `jubarte-doc::changes`; re-export in `jubarte::document_comparer` in the root facade. | Done (verified 2026-11-16): `accept_revisions_keeping_comments` / `reject_revisions_keeping_comments` live in `crates/jubarte-doc/src/changes.rs` (lines ~2350). |
| **8** | `src/changes.rs:1170, 1394` | `edit::KeptComment`, `edit::comment_settlement::AnchorChanges` | Move `src/edit/comment_settlement.rs` (`KeptComment`, `RemovedComment`, `AnchorChanges`) to `jubarte-doc`; re-export in `jubarte-edit`. | Done (verified 2026-11-16): `crates/jubarte-doc/src/comment_settlement.rs`. |
| **9** | `src/debug/render.rs:873..1015`, `src/validate.rs:2013`, `src/append.rs:1044`, `src/fields.rs:548..555`, `src/markdown/from_docx/mod.rs:3408..3428` | `markdown::{borders, data_stores, package::append_numbering}`, `convert::{roman_label, alpha_label, list_number, names_deeper_level, wrapped_children, cell_is_deleted, table_look}` | Move `borders.rs`, `data_stores.rs`, `append_numbering`, numbering label helpers, and DOM table helpers into `jubarte-opc`; re-export at original paths. | Done (verified 2026-11-16): `borders.rs`, `data_stores.rs`, `markup_simplifier.rs`, `numbering.rs` (`append_numbering`, `roman_label`, `alpha_label`, `list_number`…) and `table_dom.rs` (`table_look`, `cell_is_deleted`, `wrapped_children`, `names_deeper_level`) all live in `crates/jubarte-opc/src/`. |
| **10** | `src/cli.rs:2894-2899, 3159`, `src/markdown/mod.rs:804, 1101, 1136`, `src/markdown/from_docx/mod.rs:916`, `src/fields.rs:79, 170, 257`, `src/debug/render.rs:1158-1162` | `convert::{PdfOptions, RenderRequest, RevisionStyle, RevisionPalette, RevisionColor, MarkLines, CommentPlacement, render, docx_render_report, layout_facts}` | Move the pure config/value types (`PdfOptions`, `RenderRequest`, `RevisionStyle`, `RevisionPalette`, `RevisionColor`, `MarkLines`, `CommentPlacement`) to `jubarte-opc::render_options` (re-exported in `jubarte-render` and `jubarte::convert`), and gate the actual layout calls in `fields.rs`, `debug/render.rs`, and `markdown::page_texts` behind `#[cfg(feature = "render")]`. | Done (verified 2026-11-16, commit `035ceb527`): `crates/jubarte-opc/src/render_options.rs` holds `PdfOptions`, `RenderRequest`, `RevisionStyle`, `RevisionPalette`, `RevisionColor`, `MarkLines`, `CommentPlacement`; render-gated call sites compile under the `render` feature. |
| **11** | `src/debug.rs:1808` | `text_diff::critic_literal` | Move `critic_literal` to `jubarte-doc::debug`; re-export in `jubarte-edit::text_diff`. | Done, via seam not a move (verified 2026-11-16): `critic_literal` stays `pub(crate)` in `jubarte-edit/src/text_diff.rs`; `jubarte-doc::debug` takes a `CriticEscape = fn(&str) -> String` parameter injected by callers, since `jubarte-doc` cannot name the edit layer. |
| **12** | `src/markdown/patch.rs`, `src/markdown/redline.rs`, `src/markdown/unified.rs` | `edit::{apply_plan, EditPlan, Others, ...}` | Place `patch.rs`, `redline.rs`, `unified.rs`, and `text_diff.rs` in `jubarte-edit`; re-export into `jubarte::markdown` in the root facade. | Done (verified 2026-11-16, commit `ec6f02efa`): `crates/jubarte-edit/src/markdown/{patch,redline,unified}.rs` and `crates/jubarte-edit/src/text_diff.rs`. |
| **13** | `src/edit/inline.rs:12, 463` | `cli::{edit_actions::EditAction, Cli, Command}` | Move `src/edit/inline.rs` (already `#[cfg(feature = "cli")]`) to `jubarte-cli`; re-export `#[cfg(feature = "cli")] pub use jubarte_cli::inline;` in `jubarte::edit`. | Done (verified 2026-11-16, commits `6d105445a`/`0bb1de9a5`): `src/edit/inline.rs` moved to `crates/jubarte-cli/src/inline.rs` along with `cli.rs`, `cli/edit_actions.rs`, `cli/edit_template.rs`, `update.rs`, `update/zipsign.rs`, `build.rs`; the `include_bytes!` path and unused-import stragglers are fixed. |
| **14** | `src/comparer/finalize.rs:24548..27574`, `src/settings.rs:528`, `src/append.rs:1044` (`#[cfg(test)]`) | `validate::ring1`, `comments::list_comments`, `markdown::markdown_to_docx` | Extract those cross-crate integration unit tests to `tests/it_redlines/` and `tests/it_doc/` so `[dev-dependencies]` have zero cycles. | Done (verified 2026-11-16): no `ring1` / `list_comments` / `markdown_to_docx` references remain in `comparer/finalize.rs`, `settings.rs` or `append.rs`; the `tests/it_*/` suites exist (§6). |

### 3.1 Cross-Crate Visibility (`__private`), `#[doc(inline)]` & [scripts/api_snapshot.py](../../scripts/api_snapshot.py)

- [x] **Cross-crate internals hidden — implemented without a `__private` module**: The split puts `#[doc(hidden)]` directly on the internal `pub` items/modules each sub-crate exposes (e.g. 58 sites in `jubarte-dom`, 11 in `jubarte-opc`'s `lib.rs`), achieving the same hiding without a `__private` namespace. — *verified 2026-11-16; the named `__private` module was not adopted.*
- [x] **`#[doc(inline)]` on all root `jubarte` re-exports** — *verified 2026-11-16: 36 `#[doc(inline)]` attributes in `src/lib.rs`.*
- [x] **[scripts/api_snapshot.py](../../scripts/api_snapshot.py) handles foreign-crate re-exports and `doc(hidden)`** — *verified 2026-11-16. Implemented differently than sketched: the script was rewritten around a resolver class (`resolve`/`glob_entries`/`_local_target`, ~lines 522–580) that maps sub-crate items to their facade path (`jubarte::…`), plus `_doc_hidden` (line 508) filtering `doc(hidden)` items out of the api listing. There is no `_walk` function; the behavior the two edits targeted is in place, and `tests/facade_surface.rs` + `api-baseline.txt` gate it.*

---

## 4. Dependency & Asset Optimization (Phase 2)

- [ ] **Retire `oxml-opc` and `quick-xml-041`** — *pending (verified 2026-11-17 on pre-release): `oxml-opc = 0.15.0` and `quick-xml-041` still in root `Cargo.toml`; `crates/jubarte-opc/src/opc/mod.rs` still wraps `oxml_opc::{OpcPackage, ContentTypes, Relationships, OpcError, Relationship}`.*
  - Replace the 4 types imported from `oxml-opc` in [src/opc/mod.rs](../../src/opc/mod.rs#L35) (`Part`, `PartURI`, `Relationships`, `OpcError`) with native types in `jubarte-opc` built on `zip 9.0.5` and `quick-xml 0.42` ([docs/ZIP9_OXML_OPC_MIGRATION.md](../../docs/ZIP9_OXML_OPC_MIGRATION.md)).
  - Removes `oxml-opc`, `quick-xml 0.41`, and transitive `chrono` from `Cargo.lock` and [deny.toml](../../deny.toml).
- [x] **Remove `uuid`** — *done 2026-11-16 (commit `684352f3c`): no `uuid` dependency in any `Cargo.toml`; `crates/jubarte-edit/src/publication.rs` uses a hashed staging suffix.*
  - Replace `Uuid::new_v4()` in [src/publication.rs](../../src/publication.rs#L179) with a process-atomic counter + timestamp/PID + `RandomState` hash suffix.
- [x] **Deduplicate Embedded Font Statics & Keep Root `assets/fonts/` Layout** — *done (verified 2026-11-16, commit `38a85c0fe`): named `static CARLITO_*`/`SANS_*`/`SERIF_*`/`MONO_*` constants at `crates/jubarte-render/src/convert/font.rs:678-695`, `crates/jubarte-render/assets/fonts/` holds the TTFs + OFL licenses, root `assets/fonts/` intact; remaining `include_bytes!` sites are `#[cfg(test)]` only.*
  - Define single module-level `static CARLITO_REGULAR: &[u8] = include_bytes!(...);` constants in `crates/jubarte-render/src/font.rs`.
  - Keep `assets/fonts/` (including `assets/fonts/extra/`), `schemas/`, and `keys/` at the repository root (and copy/symlink only the 16 embedded `Carlito-*` / `Liberation*` TTFs + OFL licenses into `crates/jubarte-render/assets/fonts/` and `schemas/edit-plan-v2.schema.json` into `crates/jubarte-edit/schemas/` for self-contained crate packaging) so [.github/workflows/release.yml](../../.github/workflows/release.yml#L161), [scripts/install.sh](../../scripts/install.sh#L34), [REUSE.toml](../../REUSE.toml), and [tests/release_metadata.rs](../../tests/release_metadata.rs) remain intact.

---

## 5. Decomposing the 5 Mega-Files (Phase 4)

- [x] **`crates/jubarte-render/src/` (from [src/convert/mod.rs](../../src/convert/mod.rs) — 69,056 LOC)** — *done in T7.5 (#2459): `convert/mod.rs` is a 936-line module root with 24 production siblings and 30 test files under `convert/tests/`; see baseline-addenda.md.*
  - `lib.rs`, `model.rs`, `parse/{document,styles,tables,drawings,math,fields}.rs`, `layout/{engine,paragraph,line_break,table,float,footnote,header_footer,columns,revisions,balloons}.rs`, `font.rs`, `font_table.rs`, `pdf.rs`, `raster.rs`, `altchunk.rs`.
- [x] **`crates/jubarte-redlines/src/comparer/finalize/` (from [src/comparer/finalize.rs](../../src/comparer/finalize.rs) — 35,346 LOC)** — *done in T7.1 (#2452): a directory module of 14 production files and 35 test files; see baseline-addenda.md.*
  - `mod.rs`, `coalesce.rs`, `moves.rs`, `tables.rs`, `paragraph_marks.rs`, `formatting.rs`, `notes_comments.rs`, `ids_relationships.rs`, `sect_pr.rs`.
- [x] **`crates/jubarte-redlines/src/comparer/lcs/` (from [src/comparer/lcs.rs](../../src/comparer/lcs.rs) — 21,290 LOC)** — *done in T7.2 (#2453): a directory module of 11 production files and 11 test files; see baseline-addenda.md.*
  - `mod.rs`, `myers.rs`, `hierarchical.rs`, `word_level.rs`, `table_align.rs`, `similarity.rs`.
- [x] **`crates/jubarte-redlines/src/document_comparer/` (from [src/document_comparer.rs](../../src/document_comparer.rs) — 14,961 LOC)** — *done in T7.3 (#2456): a directory module of 4 production files and 13 test files; see baseline-addenda.md.*
  - `mod.rs`, `pipeline.rs`, `headers_footers.rs`, `footnotes_endnotes.rs`, `comments_part.rs`, `revisions_list.rs`, `accept_reject.rs`.
- [x] **`crates/jubarte-edit/src/edit/` (from [src/edit.rs](../../src/edit.rs) — 12,353 lines)** — *done in T7.4 (#2458): `edit.rs` is now `edit/mod.rs` plus 7 production siblings (`transaction.rs` holds the whole `impl Transaction` block) and 2 test files under `edit/tests/`, beside the existing leaf modules; see baseline-addenda.md.*
  - `lib.rs`, `plan.rs`, `anchor.rs`, `apply.rs`, `formatting.rs`, `tables_lists.rs`, `notes_images.rs`, plus existing submodules.

---

## 6. Careful AST/Symbol-Driven Test Classification & Consolidation (Phase 5)

All 595 files in `tests/*.rs` are classified by **actual AST/symbol imports, `#[global_allocator]` / process-global state, `CARGO_BIN_EXE_jubarte` usage, `#![cfg(feature = ...)]` attributes, and CI/script invocation**:

### 6.1 19 Standalone Top-Level Test Binaries (`tests/*.rs`)

- [x] **3 Custom `#[global_allocator]` Tests** — *done (verified 2026-11-16): all three remain standalone top-level binaries.*
  - [tests/perf_atom_clone_allocs.rs](../../tests/perf_atom_clone_allocs.rs), [tests/perf_serialize_prefix_allocs.rs](../../tests/perf_serialize_prefix_allocs.rs), [tests/foreign_alignment.rs](../../tests/foreign_alignment.rs)
- [x] **1 Process-Global Atomic Perf Counter Test** — *done.*
  - [tests/perf_contract.rs](../../tests/perf_contract.rs)
- [x] **Named CI / Workflow / Repository-Metadata Gate Tests** — *done with deviation (verified 2026-11-16): 22 top-level `tests/*.rs` exist. Of the 15 named, `wasm_crate_is_self_contained.rs` does not exist; four new standalone gates were added instead: `crate_layering.rs`, `facade_surface.rs`, `suite_membership.rs`, `convert_docx_to_pdf.rs`.*
  - [tests/md_fixed_point.rs](../../tests/md_fixed_point.rs), [tests/adoption.rs](../../tests/adoption.rs), [tests/legacy_doc.rs](../../tests/legacy_doc.rs), [tests/legacy_doc_convert.rs](../../tests/legacy_doc_convert.rs), [tests/m_validity_ring1.rs](../../tests/m_validity_ring1.rs), [tests/coverage_gates.rs](../../tests/coverage_gates.rs), [tests/tuning_audit.rs](../../tests/tuning_audit.rs), [tests/release_metadata.rs](../../tests/release_metadata.rs), [tests/repo_paths_fit_windows.rs](../../tests/repo_paths_fit_windows.rs), [tests/schema_consistency.rs](../../tests/schema_consistency.rs), [tests/deny_build_scripts.rs](../../tests/deny_build_scripts.rs), [tests/readme_coverage.rs](../../tests/readme_coverage.rs), [tests/cli_version.rs](../../tests/cli_version.rs), [tests/bench_fixture_paths_resolve.rs](../../tests/bench_fixture_paths_resolve.rs), [tests/wasm_crate_is_self_contained.rs](../../tests/wasm_crate_is_self_contained.rs).

### 6.2 8 Consolidated Domain Test Suites (`tests/it_*/main.rs` — 576 files)

Classified deterministically by actual symbol/feature usage:

- [x] **`tests/it_cli/` (~78 files)** — *done (verified 2026-11-16): exists with ~79 entries.* Every remaining test that references `env!("CARGO_BIN_EXE_jubarte")`, `jubarte::cli::*`, or `jubarte::update::*`, or carries `#![cfg(feature = "cli")]` / `#![cfg(feature = "self-update")]`.
- [x] **`tests/it_security/` + `tests/it_dom_opc/` → consolidated as `tests/it_core/`** — *done with deviation (verified 2026-11-16): the plan's `it_security` (~18) and `it_dom_opc` (~15) suites were merged into a single `tests/it_core/` suite (~18 entries) — 7 `it_*` suites total, not 8.* Non-CLI tests covering ZIP/XML bomb admission, data descriptors, malformed archives, duplicate parts, BOM handling, and Strict OOXML translation (`admission_data_descriptor`, `input_admission`, `m8_strict`, `strict_nested_text_box`, `customxml_nested_text_box`, `m13_bom`, etc.).
- [x] **`tests/it_render/` (~4 files)** — *done (~5 entries).* Non-CLI tests exercising `jubarte::convert::*` (`convert_alternate_content`, `convert_docx_to_png`, `convert_revision_palette`, `m_opc_explicit_close`). (Note: the 6 CLI-spawning `convert_*.rs` tests go into `it_cli` per Rule 1.)
- [x] **`tests/it_edit/` (~44 files)** — *done (~47 entries).* Non-CLI tests exercising `jubarte::edit::*`, `jubarte::text_diff::*`, `jubarte::expect::*`, or `jubarte::capabilities::*`.
- [x] **`tests/it_markdown/` (~61 files)** — *done (~56 entries).* Non-CLI, non-edit tests exercising `jubarte::markdown::*` (including `footnote_text_alias`, `docx_markdown_round_trip`, `docx_to_markdown_critic_markup`, `md_*`, `agent_view_attribution`).
- [x] **`tests/it_doc/` (~15 files)** — *done (~28 entries).* Non-CLI, non-edit, non-markdown tests exercising `jubarte::{changes,comments,validate,repair,audit,scrub,append,fields,debug,inspect}::*`.
- [x] **`tests/it_redlines/` (~341 files)** — *done (~352 entries).* Non-CLI tests exercising `jubarte-redlines` (`jubarte::{comparer,document_comparer,revision_processor}::*`, including the non-allocator `perf_*.rs` compare tests and `m32_word_alignment`).
- [x] **`tests/it_dom_opc/`** — *done as part of `tests/it_core/` (see above); the foundational DOM/OPC tests live there (e.g. `m_part_rels_dangling.rs` is under `it_redlines/`).*

### 6.3 Mandatory Mechanical Transformations for Consolidated Test Files & Moved `src/` Tests

- [x] **`mod common;` $\rightarrow$ `use crate::common;` in-place** — *done (verified 2026-11-16): 217 consolidated files use `crate::common`; each `tests/it_*/main.rs` declares `mod common;` once.*
- [x] **Prepend `../` to relative `include_bytes!` / `include_str!` calls in `tests/it_<suite>/*.rs`** — *done (verified 2026-11-16): e.g. `it_redlines/m9_roundtrip.rs` uses `include_bytes!("../fixtures/f4/...")`.*
- [x] **Update the `#[cfg(test)]` `include_bytes!` and `CARGO_MANIFEST_DIR` paths inside `crates/*/src/`** — *done (verified 2026-11-16): the last straggler landed with T4.8 — `crates/jubarte-cli/src/update.rs:1823` now reads `include_bytes!("../../../tests/data/self_update/unsigned.tar.gz")`.*

---

## 7. Unified Workspace Configuration, CI, Scripts & Hygiene (Phase 6)

- [~] **Unified Root Cargo Workspace ([Cargo.toml](../../Cargo.toml), `.cargo/config.toml`, `crates/jubarte-cli/build.rs`)** — *partial, with deviation (verified 2026-11-16):*
  - Root `[workspace]` exists with `resolver = "3"`, `[workspace.package]`, `[workspace.lints]`, `[workspace.dependencies]` — but `members = [".", "crates/*"]` and `jubarte-python`, `jubarte-wasm`, `jubarte-rust-inproc`, `fuzz` are in `exclude`, each keeping its own workspace + `Cargo.lock` (deviation: plan wanted them as members).
  - No root `strip = "none"` `[profile.release]`; no root `.cargo/config.toml` — the `wasm32` rustflags still live in `jubarte-wasm/.cargo/config.toml`.
  - `crates/jubarte-cli/build.rs` now resolves the repo root at `CARGO_MANIFEST_DIR/../..` and reruns on `HEAD`/ref changes, so `JUBARTE_BUILD_VERSION` keeps its `+<sha>` suffix (fixed with T4.8).
  - `members = ["crates/*", ".", "jubarte-python", "jubarte-wasm", "jubarte-rust-inproc"]` and `exclude = ["fuzz"]`.
  - Root `[profile.release]` sets `strip = "none"` (protecting `jubarte-python` macOS Mach-O `LINKEDIT`).
  - Move `jubarte-wasm/.cargo/config.toml` `[target.wasm32-unknown-unknown]` `rustflags` into root `.cargo/config.toml`.
  - In `crates/jubarte-cli/build.rs`, resolve the `.git` directory across both `CARGO_MANIFEST_DIR` and `CARGO_MANIFEST_DIR/../..` so `JUBARTE_BUILD_VERSION` always includes `+<sha>`.
- [~] **Gate Tests, Scripts & Workflows Updated** — *partial (verified 2026-11-16):*
  - Done: `tests/coverage_gates.rs` walks `crates/`; `tests/tuning_audit.rs` scans `crates/jubarte-render/src/convert/**`; `scripts/check_evidence_engine.py` `INPUTS` includes `"crates"`; `scripts/check_engine_commit.sh` `ENGINE_PATHS` includes `crates`; `scripts/check_no_network.py` handles crates.
  - Pending: per-crate topological `cargo publish --workspace` — [release.yml](../../.github/workflows/release.yml) still runs a single `cargo publish --locked` for `jubarte-redlines-$VER` (line ~515).*
  - [tests/coverage_gates.rs](../../tests/coverage_gates.rs): Allow `relative.starts_with("src") || relative.starts_with("crates")`, check `crates/*/src/lib.rs`, and walk `tests/**/*.rs` recursively.
  - [tests/tuning_audit.rs](../../tests/tuning_audit.rs) & [docs/TUNING_AUDIT.md](../../docs/TUNING_AUDIT.md): Scan `crates/jubarte-render/src/**/*.rs`.
  - [tests/bench_fixture_paths_resolve.rs](../../tests/bench_fixture_paths_resolve.rs): Scan `root.join("crates")` in addition to `src` and `tests`.
  - [tests/release_metadata.rs](../../tests/release_metadata.rs): Check font/image dependencies in `crates/jubarte-render/Cargo.toml` and root `Cargo.toml`/`Cargo.lock`.
  - [scripts/check_evidence_engine.py](../../scripts/check_evidence_engine.py) & `test_check_evidence_engine.py`: Add `"crates"` to `INPUTS` and check `crates/*/src/**/*.rs` and `crates/*/Cargo.toml`.
  - [scripts/check_engine_commit.sh](../../scripts/check_engine_commit.sh) & `test_check_engine_commit.py`: Add `crates` to `ENGINE_PATHS` and update `WASM_LOCK` handling for the unified workspace.
  - [scripts/check_no_network.py](../../scripts/check_no_network.py) & `test_check_no_network.py`: Scan `crates/*/src/` and `src/`, allowing network symbols only in `crates/jubarte-cli/src/update/`.
  - `tools/parity/build.sh`, `tools/parity/parity_check.py`, `tools/parity/test_parity_check.py`: Include `crates` in `SRC_DIRS` and input hashes.
  - [.github/workflows/ci.yml](../../.github/workflows/ci.yml) & [.github/workflows/release.yml](../../.github/workflows/release.yml) & [scripts/release.sh](../../scripts/release.sh): Use `cargo publish --workspace --dry-run` in CI/Step 7, publish workspace crates in topological order (`jubarte-dom` $\rightarrow$ $\dots$ $\rightarrow$ `jubarte-redlines` $\rightarrow$ $\dots$ $\rightarrow$ `jubarte`) checking `crates.io` per crate for idempotent resume, and normalize `src-tauri/Cargo.toml` in `app_test_gate`.
- [x] **Root Directory Cleanup** — *mostly N/A (verified 2026-11-16): of the listed files only `ISSUE_TRIAGE_REPORT.md` ever existed in this repo's history and it still sits at the root; `docs/archive/` does not exist; `reproduce_issue.py` is absent. Done 2026-11-17: `ISSUE_TRIAGE_REPORT.md` lives in `docs/archive/`, which `.gitignore` keeps local; it is no longer tracked.*
  - Move historical root markdown reports (`AUDIT_ISSUES.md`, `DOC_CONVERSION_ISSUES.md`, `ISSUE_TRIAGE_REPORT.md`, `PR_REVIEW_LOG.md`, `PR2017_FINAL_VERIFICATION.md`, `RELEASE_0.16.0_STATUS.md`, `ROUNDTRIP_ISSUES.md`, `SECURITY_ISSUES.md`) to `docs/archive/`, and `reproduce_issue.py` to `scripts/`.
