// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! WebAssembly bindings for the canonical **jubarte-redlines** Word-mode compare.
//!
//! Built with **wasm-pack** + **wasm-opt -O3** (Binaryen) — the standard
//! Rust→browser/Node pipeline that currently produces the fastest practical
//! wasm-bindgen artefacts for pure compute crates.
//!
//! # JS API (Node target)
//!
//! ```js
//! import init, { compareDocuments, docxToPdf } from "./pkg/jubarte_wasm.js";
//! await init();
//! const redline = compareDocuments(baseBytes, nextBytes, "jubarte-wasm");
//! // redline: Uint8Array
//! const pdf = docxToPdf(redline);
//! // pdf: Uint8Array
//! ```
//!
//! Agents read and edit through the same surface as the CLI and Python:
//!
//! ```js
//! const snapshot = JSON.parse(inspectDocument(bytes));  // ids, text, spans
//! const text = documentMarkdown(bytes);                 // [pN] ids
//! const out = applyEditPlan(bytes, JSON.stringify(plan));
//! if (out.ok) { out.clean; out.redline; out.patch; JSON.parse(out.json) }  // report
//! else { JSON.parse(out.json).code }                    // e.g. AMBIGUOUS_ANCHOR
//! const { text, hunks } = JSON.parse(diffDocuments(      // the changes as a patch
//!   oldDocx, new TextEncoder().encode(markdown), "Ana Lima", new Date().toISOString().slice(0, 19) + "Z"));
//! const draft = markdownToDocx("Due in {~~30~>45~~} days.", JSON.stringify({ page: "a4" }));  // Word bytes
//! ```
//!
//! # Build
//!
//! ```sh
//! wasm-pack build --target nodejs --release
//! ```

use wasm_bindgen::prelude::*;

/// Current wasm linear-memory size in 64 KiB pages.
///
/// Diagnostic builds only: the release package deliberately omits this export.
#[cfg(all(target_arch = "wasm32", feature = "memory-metrics"))]
#[wasm_bindgen(js_name = wasmMemoryPages)]
pub fn wasm_memory_pages() -> usize {
    core::arch::wasm32::memory_size::<0>()
}

/// One-shot init: panic hook → `console.error`. Safe to call multiple times.
#[wasm_bindgen(js_name = initPanicHook)]
pub fn init_panic_hook() {
    #[cfg(feature = "console-panic")]
    console_error_panic_hook::set_once();
}

/// An error as the thrown string; an admission refusal reads code first
/// after the prefix (`jubarte-wasm: LEGACY_DOC: …`), whatever wrapped it.
fn js_err(e: impl std::fmt::Display) -> JsValue {
    let message = e.to_string();
    let message = jubarte::admission::code_first(&message).unwrap_or(message);
    JsValue::from_str(&format!("jubarte-wasm: {message}"))
}

/// `WmlComparerSettings::default()` with `input_limits_json` (a JSON object
/// such as `{"max_part_bytes": 67108864}`) laid over its compare budget.
fn settings_with_limits(
    input_limits_json: Option<&str>,
) -> Result<jubarte::comparer::WmlComparerSettings, String> {
    let settings = jubarte::comparer::WmlComparerSettings::default();
    let Some(json) = input_limits_json else {
        return Ok(settings);
    };
    let overrides = jubarte::admission::InputLimitOverrides::from_json(json)?;
    let base = settings.input_limits;
    Ok(settings.with_input_limits(overrides.apply(base)))
}

/// Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).
///
/// Mirrors `jubarte::document_comparer::compare_documents`.
/// `inputLimitsJson` (optional) overrides the admission budget key by key:
/// `{"max_compressed_bytes", "max_entries", "max_part_bytes",
/// "max_uncompressed_bytes", "max_xml_depth"}`. A package past the budget
/// throws with `INPUT_LIMIT`; an unknown key throws `invalid input limits`.
/// The default budget allows 2 GiB inflated, more than a 32-bit WASM heap
/// holds, so browser hosts should lower it.
#[wasm_bindgen(js_name = compareDocuments)]
pub fn compare_documents(
    original: &[u8],
    modified: &[u8],
    author: &str,
    input_limits_json: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    let settings = settings_with_limits(input_limits_json.as_deref())
        .map_err(js_err)?
        .with_author(author);
    jubarte::document_comparer::compare_documents_with_settings(original, modified, &settings)
        .map_err(js_err)
}

/// Accept every tracked revision (package-wide) → clean DOCX bytes.
///
/// Mirrors `jubarte::document_comparer::accept_revisions`.
#[wasm_bindgen(js_name = acceptRevisions)]
pub fn accept_revisions(docx: &[u8]) -> Result<Vec<u8>, JsValue> {
    jubarte::document_comparer::accept_revisions(docx).map_err(js_err)
}

/// Reject every tracked revision (package-wide) → base DOCX bytes.
///
/// Mirrors `jubarte::document_comparer::reject_revisions`.
#[wasm_bindgen(js_name = rejectRevisions)]
pub fn reject_revisions(docx: &[u8]) -> Result<Vec<u8>, JsValue> {
    jubarte::document_comparer::reject_revisions(docx).map_err(js_err)
}

/// List the tracked changes as a JSON array string, one object per
/// logical change: a change Word wrote as several marks is one object
/// with a composite id (`r1+2`) and its `members`, and an author's
/// deletion followed by their insertion is one `replacement`. Each `id`
/// is one `acceptChanges` / `rejectChanges` select by. The objects are
/// the ones `jubarte changes --json` prints: `id`, `ref` (the same id),
/// `kind` (`insertion`, `deletion`, `move`, `formatting` or
/// `replacement`), `target`, `author`, `date`, `text`, `paragraph`,
/// `move_name`, `move_side`, `inside`, `holds_comments` when the change
/// holds comment marks, and `members` (each mark's `id`, `kind`,
/// `target`, `date` and `text`) for a change of several marks.
/// `authors` keeps those authors' changes (exact names), as `jubarte
/// changes --author`; omitted or empty, every change.
///
/// Mirrors `jubarte::changes::list_logical_changes` and `by_authors`.
#[wasm_bindgen(js_name = listChanges)]
pub fn list_changes(docx: &[u8], authors: Option<Vec<String>>) -> Result<String, JsValue> {
    let changes = jubarte::changes::list_logical_changes(docx).map_err(js_err)?;
    let changes = jubarte::changes::by_authors(changes, &authors.unwrap_or_default());
    serde_json::to_string(&changes).map_err(js_err)
}

/// List every comment as a JSON array string (the objects `jubarte comments
/// --json` prints: `id`, `author`, `initials`, `date`, `text`, `parent`,
/// `done`, `paragraph`, `last_paragraph` when the range ends in a later
/// paragraph, `anchor_text`, `before`, `after`). `author` keeps one
/// author's comments; `latest` keeps the newest comment of each thread.
///
/// Mirrors `jubarte::comments::list_comments` and `select_comments`.
#[wasm_bindgen(js_name = listComments)]
pub fn list_comments(
    docx: &[u8],
    author: Option<String>,
    latest: Option<bool>,
) -> Result<String, JsValue> {
    let comments = jubarte::comments::list_comments(docx).map_err(js_err)?;
    let comments =
        jubarte::comments::select_comments(comments, author.as_deref(), latest.unwrap_or(false));
    serde_json::to_string(&comments).map_err(js_err)
}

/// The listing `jubarte changes` prints: a line per change with its short
/// id (`r0`), kind, target, author, paragraph and text (then `(N elements)`
/// for a change Word wrote as several marks), then `N change(s)`.
/// `authors` filters as in [`listChanges`](list_changes).
///
/// Mirrors `jubarte::changes::changes_text`.
#[wasm_bindgen(js_name = changesText)]
pub fn changes_text(docx: &[u8], authors: Option<Vec<String>>) -> Result<String, JsValue> {
    let changes = jubarte::changes::list_logical_changes(docx).map_err(js_err)?;
    let changes = jubarte::changes::by_authors(changes, &authors.unwrap_or_default());
    Ok(jubarte::changes::changes_text(&changes))
}

/// The listing `jubarte comments` prints: a line per comment with its short
/// id (`c5`), paragraph, author, text and anchor, then `N comment(s)`.
/// `author` and `latest` filter as in [`listComments`](list_comments).
///
/// Mirrors `jubarte::comments::comments_text`.
#[wasm_bindgen(js_name = commentsText)]
pub fn comments_text(
    docx: &[u8],
    author: Option<String>,
    latest: Option<bool>,
) -> Result<String, JsValue> {
    let comments = jubarte::comments::list_comments(docx).map_err(js_err)?;
    let comments =
        jubarte::comments::select_comments(comments, author.as_deref(), latest.unwrap_or(false));
    Ok(jubarte::comments::comments_text(&comments))
}

fn change_filter_of(filter_json: &str) -> Result<String, String> {
    let filter: jubarte::changes::ChangeFilter =
        serde_json::from_str(filter_json).map_err(|e| format!("invalid change filter: {e}"))?;
    let filter = filter.normalized().map_err(|e| e.to_string())?;
    serde_json::to_string(&filter).map_err(|e| e.to_string())
}

/// `filterJson` (as [`acceptChanges`](accept_changes) takes it) with each id
/// in its settled spelling (`r1+2` → `r1`, `r2`). An id that is no change
/// id, the expired `body:rev:1` among them, throws `jubarte-wasm: c5:
/// expected a change id …` (naming `r1` for `body:rev:1`), which the
/// binary reports as a usage error before reading the document.
///
/// Mirrors `jubarte::changes::ChangeFilter::normalized`.
#[wasm_bindgen(js_name = changeFilter)]
pub fn change_filter_json(filter_json: &str) -> Result<String, JsValue> {
    change_filter_of(filter_json).map_err(js_err)
}

fn change_filter(filter_json: &str) -> Result<jubarte::changes::ChangeFilter, JsValue> {
    serde_json::from_str(filter_json).map_err(|e| js_err(format!("invalid change filter: {e}")))
}

/// Accept the changes `filterJson` selects and keep the rest tracked, as
/// Word's Accept This Change does. The filter is `{"ids": [...],
/// "authors": [...], "kinds": [...]}`: a change is selected when it matches
/// every list given (`{}` selects every change; an empty list, none).
///
/// Mirrors `jubarte::changes::accept_changes`.
#[wasm_bindgen(js_name = acceptChanges)]
pub fn accept_changes(docx: &[u8], filter_json: &str) -> Result<Vec<u8>, JsValue> {
    jubarte::changes::accept_changes(docx, &change_filter(filter_json)?).map_err(js_err)
}

/// Reject the changes `filterJson` selects and keep the rest tracked
/// (filter as in `acceptChanges`).
///
/// Mirrors `jubarte::changes::reject_changes`.
#[wasm_bindgen(js_name = rejectChanges)]
pub fn reject_changes(docx: &[u8], filter_json: &str) -> Result<Vec<u8>, JsValue> {
    jubarte::changes::reject_changes(docx, &change_filter(filter_json)?).map_err(js_err)
}

/// List the tracked revisions in a DOCX as a JSON array string, grouped as
/// Docxodus's `GetRevisions` groups them; each object has
/// (`type`/`author`/`date`/`part`/`moveGroupId`/`isMoveSource`/`formatChange`/`text`).
///
/// Mirrors `jubarte::document_comparer::get_revisions` with default settings,
/// serialized by the shared `revisions_to_json`. `inputLimitsJson` as in
/// `compareDocuments`.
#[wasm_bindgen(js_name = getRevisions)]
pub fn get_revisions(docx: &[u8], input_limits_json: Option<String>) -> Result<String, JsValue> {
    let settings = settings_with_limits(input_limits_json.as_deref()).map_err(js_err)?;
    let revs = jubarte::document_comparer::get_revisions(docx, &settings).map_err(js_err)?;
    Ok(jubarte::document_comparer::revisions_to_json(&revs))
}

/// Render a DOCX package (bytes) → PDF bytes (Word-style layout).
///
/// Mirrors `jubarte::convert::docx_to_pdf`. Fonts come from the embedded
/// Carlito / Liberation set; the native system/cloud font overrides are
/// no-ops under wasm (no filesystem), which only changes glyph sourcing,
/// never layout metrics.
/// `compress` (optional, default `false`) deflates the PDF's streams
/// (`/FlateDecode`): much smaller output, no longer plain text.
/// `revisions` (optional, default `"conventional"`) paints tracked changes:
/// `"conventional"`, `"word"` (Microsoft Word's markup) or `"custom"` with
/// `revisionPalette` (`"deleted=#AA0000:strike,..."`).
/// `moveComments` (optional, default `false`) lists the comments after the
/// last page instead of in balloons beside the text; `changedOnly`
/// (optional, default `false`) keeps only the pages a tracked change
/// touches, the pages a comment is anchored on and each section's first
/// page on which a revised header or footer paints (else the first page).
#[cfg(feature = "pdf")]
#[wasm_bindgen(js_name = docxToPdf)]
pub fn docx_to_pdf(
    docx: &[u8],
    compress: Option<bool>,
    revisions: Option<String>,
    revision_palette: Option<String>,
    move_comments: Option<bool>,
    changed_only: Option<bool>,
) -> Result<Vec<u8>, JsValue> {
    let revisions = jubarte::convert::RevisionStyle::from_choice(
        revisions.as_deref().unwrap_or("conventional"),
        revision_palette.as_deref(),
    )
    .map_err(|e| JsValue::from_str(&e))?;
    let options = jubarte::convert::PdfOptions {
        compress: compress.unwrap_or(false),
        revisions,
        comments: if move_comments.unwrap_or(false) {
            jubarte::convert::CommentPlacement::End
        } else {
            jubarte::convert::CommentPlacement::Margin
        },
        comment_notes: false,
        changed_only: changed_only.unwrap_or(false),
    };
    jubarte::convert::docx_to_pdf_with(docx, options).map_err(js_err)
}

/// Number of pages in a PDF (cheap object scan; `0` if the bytes are not a
/// readable PDF).
///
/// Mirrors `jubarte::convert::pdf_page_count`.
#[cfg(feature = "pdf")]
#[wasm_bindgen(js_name = pdfPageCount)]
pub fn pdf_page_count(pdf: &[u8]) -> usize {
    jubarte::convert::pdf_page_count(pdf)
}

/// SHA-256 (lowercase hex) of the bytes: the `source_sha256` guard an edit
/// plan carries.
///
/// Mirrors `jubarte::inspect::source_sha256`.
#[wasm_bindgen(js_name = sourceSha256)]
pub fn source_sha256(docx: &[u8]) -> String {
    jubarte::inspect::source_sha256(docx)
}

/// The inspection snapshot as JSON: `schema_version`, `source_sha256`,
/// `summary`, `paragraphs` (ids, text, style, formatting spans,
/// limitations) and top-level `tables` with recursive `nested` entries.
/// Snapshot schema 3 lists logical changes (one entry, with `members`, for
/// a change Word wrote as several marks), counts top-level tables, and gives
/// nested entries `parent` cell paths and no `index`. Oversized or malformed packages are refused before parsing.
///
/// Mirrors `jubarte::inspect::inspect_json`.
#[wasm_bindgen(js_name = inspectDocument)]
pub fn inspect_document(docx: &[u8]) -> Result<String, JsValue> {
    jubarte::inspect::inspect_json(docx).map_err(js_err)
}

/// Body paragraphs as Markdown, each preceded by its `[pN]` id: the
/// coordinates an edit plan uses.
///
/// Mirrors `jubarte::inspect::markdown`.
#[wasm_bindgen(js_name = documentMarkdown)]
pub fn document_markdown(docx: &[u8]) -> Result<String, JsValue> {
    jubarte::inspect::markdown(docx).map_err(js_err)
}

/// Markdown without paragraph ids, with tracked changes kept or resolved.
#[wasm_bindgen(js_name = documentMarkdownWithChanges)]
pub fn document_markdown_with_changes(docx: &[u8], track_changes: &str) -> Result<String, JsValue> {
    let choice = jubarte::markdown::TrackChanges::parse(track_changes)
        .ok_or_else(|| js_err("track_changes must be all, accept or reject"))?;
    jubarte::markdown::docx_to_markdown(
        docx,
        &jubarte::markdown::MarkdownOptions {
            track_changes: choice,
            extract_media: None,
            ..jubarte::markdown::MarkdownOptions::default()
        },
    )
    .map(|read| read.markdown)
    .map_err(js_err)
}

/// `readView` options: the `read` command's flags.
#[derive(Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ReadViewOptions {
    track_changes: Option<String>,
    comments: Option<String>,
    dates: bool,
    page_markers: Option<bool>,
    paragraphs: Option<String>,
    head: Option<usize>,
    tail: Option<usize>,
    changed: bool,
    by: Option<String>,
    source: Option<String>,
}

fn read_view_json(docx: &[u8], options_json: Option<&str>) -> Result<String, String> {
    let options: ReadViewOptions = match options_json {
        Some(json) => serde_json::from_str(json).map_err(|e| format!("readView options: {e}"))?,
        None => ReadViewOptions::default(),
    };
    let track_changes = track_changes_of(options.track_changes.as_deref())?;
    let comments = match options.comments.as_deref() {
        None | Some("inline") => true,
        Some("none") => false,
        Some(other) => return Err(format!("comments must be inline or none, not {other:?}")),
    };
    let select = jubarte::markdown::Select::from_flags(
        options.paragraphs.as_deref(),
        options.head,
        options.tail,
        options.changed,
        options.by.as_deref(),
    )?;
    let view = jubarte::markdown::read(
        docx,
        &jubarte::markdown::ReadOptions {
            track_changes,
            comments,
            dates: options.dates,
            page_markers: options.page_markers.unwrap_or(true),
            select,
            source: options.source,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "markdown": view.markdown, "warnings": view.warnings }).to_string())
}

/// The agent view (`read`) as JSON `{markdown, warnings}`: a YAML header,
/// then Markdown with `<!-- pN -->` id lines and every change and comment
/// with its id. Options (JSON, all optional): `trackChanges` (all, accept,
/// reject), `comments` (inline, none), `dates`, `pageMarkers` (default
/// true: the layout pass numbers the pages), `paragraphs` (`"p3,p10-p20,t0"`),
/// `head`, `tail`, `changed` (only the blocks with marks) with `by` (one
/// author's: a handle or a full name), `source` (the name printed as
/// `source:`).
///
/// Mirrors `jubarte::markdown::read`.
#[wasm_bindgen(js_name = readView)]
pub fn read_view(docx: &[u8], options_json: Option<String>) -> Result<String, JsValue> {
    read_view_json(docx, options_json.as_deref()).map_err(js_err)
}

/// `tablesText` options: `read --tables`'s `--track-changes`.
#[derive(Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct TablesTextOptions {
    track_changes: Option<String>,
}

fn tables_text_of(docx: &[u8], options_json: Option<&str>) -> Result<String, String> {
    let options: TablesTextOptions = match options_json {
        Some(json) => serde_json::from_str(json).map_err(|e| format!("tablesText options: {e}"))?,
        None => TablesTextOptions::default(),
    };
    let options = jubarte::markdown::ReadOptions {
        track_changes: track_changes_of(options.track_changes.as_deref())?,
        ..jubarte::markdown::ReadOptions::default()
    };
    jubarte::markdown::read_tables(docx, &options)
        .map(|view| view.markdown)
        .map_err(|e| e.to_string())
}

/// `trackChanges` as the view reads it: `all` (the default), `accept` or
/// `reject`, with their `accepted` and `rejected` spellings.
fn track_changes_of(choice: Option<&str>) -> Result<jubarte::markdown::TrackChanges, String> {
    match choice {
        None => Ok(jubarte::markdown::TrackChanges::All),
        Some(choice) => jubarte::markdown::TrackChanges::parse(choice)
            .ok_or_else(|| "trackChanges must be all, accept or reject".to_string()),
    }
}

/// The grids `jubarte read --tables` prints: a `table N: RxC …` line per
/// top-level body table, then a line per row of `ids=text` cells; nested
/// tables follow their cell row, indented two spaces per level and labelled
/// `nested in CELL_PATH`. `no tables` prints when there are none. Options (JSON, all optional): `trackChanges` (`reject`
/// reads the rejected text).
///
/// Mirrors `jubarte::markdown::read_tables` without a selection.
#[wasm_bindgen(js_name = tablesText)]
pub fn tables_text(docx: &[u8], options_json: Option<String>) -> Result<String, JsValue> {
    tables_text_of(docx, options_json.as_deref()).map_err(js_err)
}

/// `tablesView` options: `read --tables` with the view's options.
#[derive(Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct TablesViewOptions {
    track_changes: Option<String>,
    comments: Option<String>,
    dates: bool,
    page_markers: Option<bool>,
    paragraphs: Option<String>,
    source: Option<String>,
}

fn tables_view_of(docx: &[u8], options_json: Option<&str>) -> Result<String, String> {
    let options: TablesViewOptions = match options_json {
        Some(json) => serde_json::from_str(json).map_err(|e| format!("tablesView options: {e}"))?,
        None => TablesViewOptions::default(),
    };
    let comments = match options.comments.as_deref() {
        None | Some("inline") => true,
        Some("none") => false,
        Some(other) => return Err(format!("comments must be inline or none, not {other:?}")),
    };
    let view = jubarte::markdown::read_tables(
        docx,
        &jubarte::markdown::ReadOptions {
            track_changes: track_changes_of(options.track_changes.as_deref())?,
            comments,
            dates: options.dates,
            page_markers: options.page_markers.unwrap_or(true),
            select: jubarte::markdown::Select::from_flags(
                options.paragraphs.as_deref(),
                None,
                None,
                false,
                None,
            )?,
            source: options.source,
        },
    )
    .map_err(|e| match e {
        jubarte::markdown::TablesError::Usage(message) => format!("USAGE: {message}"),
        jubarte::markdown::TablesError::Read(e) => e.to_string(),
    })?;
    Ok(serde_json::json!({ "markdown": view.markdown, "warnings": view.warnings }).to_string())
}

/// What `jubarte read --tables` prints, as JSON `{markdown, warnings}`:
/// the grids (`tablesText`), or with `paragraphs` (`"t0"`, `"p11"`,
/// `"p3-p20"`) the grids of the tables the view's selection reaches (a
/// `pN` inside a table, that table), each with the tables nested in it,
/// then an empty line and the view `readView` prints of that selection.
/// Options (JSON, all optional): `trackChanges`, `comments`, `dates`,
/// `pageMarkers`, `paragraphs`, `source`, as `readView` reads them. A pick
/// that reaches no table throws `jubarte-wasm: USAGE: …`, as the CLI
/// exits 2.
///
/// Mirrors `jubarte::markdown::read_tables`.
#[wasm_bindgen(js_name = tablesView)]
pub fn tables_view(docx: &[u8], options_json: Option<String>) -> Result<String, JsValue> {
    tables_view_of(docx, options_json.as_deref()).map_err(js_err)
}

/// Whether a command line (without the program name) asks for JSON: a
/// real `--json` flag before any `--`, read as the binary reads it (the
/// `--json` in `--text --json` is a value). An error then prints its
/// [`runtime_json_error`] line on stdout too.
///
/// Mirrors `jubarte::cli::wants_json`.
#[wasm_bindgen(js_name = wantsJson)]
pub fn wants_json(arguments: Vec<String>) -> bool {
    jubarte::cli::wants_json(&arguments)
}

/// The line an error prints on stdout under `--json`:
/// `{"error":{"code":…,"message":…}}`, its code the message's leading
/// `CODE:`, else `USAGE` for an exit 2 and `ERROR` for any other.
///
/// Mirrors `jubarte::cli::runtime_json_error`.
#[wasm_bindgen(js_name = runtimeJsonError)]
pub fn runtime_json_error(message: &str, exit: u8) -> String {
    jubarte::cli::runtime_json_error(message, exit)
}

/// `readJson` options: the `read` command's selection, and the text its
/// `--track-changes` settles.
#[derive(Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ReadJsonOptions {
    track_changes: Option<String>,
    paragraphs: Option<String>,
    head: Option<usize>,
    tail: Option<usize>,
    changed: bool,
    by: Option<String>,
    source: Option<String>,
    /// The path as the command line spells it: the snapshot's summary
    /// carries the input's validity findings (#1309).
    validity_file: Option<String>,
}

fn read_json_of(docx: &[u8], options_json: Option<&str>) -> Result<String, String> {
    let options: ReadJsonOptions = match options_json {
        Some(json) => serde_json::from_str(json).map_err(|e| format!("readJson options: {e}"))?,
        None => ReadJsonOptions::default(),
    };
    let select = jubarte::markdown::Select::from_flags(
        options.paragraphs.as_deref(),
        options.head,
        options.tail,
        options.changed,
        options.by.as_deref(),
    )?;
    let track_changes = track_changes_of(options.track_changes.as_deref())?;
    let validity_file = options.validity_file;
    let options = jubarte::markdown::ReadOptions {
        track_changes,
        select,
        source: options.source,
        ..jubarte::markdown::ReadOptions::default()
    };
    let mut snapshot =
        jubarte::markdown::read_snapshot(docx, &options).map_err(|e| e.to_string())?;
    snapshot.summary.input_validity = validity_file
        .as_deref()
        .map(|file| jubarte::validate::input_validity_of(docx, file));
    serde_json::to_string(&snapshot).map_err(|e| e.to_string())
}

/// The snapshot `jubarte read --json` prints, with its `changes` and
/// `comments`. Options (JSON, all optional) select as in
/// [`readView`](read_view): `paragraphs`, `head`, `tail`, `changed`, `by`;
/// `source` names the document; `trackChanges` `accept` or `reject`
/// settles the text and names the choice in `track_changes`;
/// `validityFile`, the path as the command line spells it, adds
/// `summary.input_validity`, the input's validity findings counted. The
/// snapshot keeps the selected paragraphs and their changes and comments.
///
/// Mirrors `jubarte::markdown::read_snapshot`.
#[wasm_bindgen(js_name = readJson)]
pub fn read_json(docx: &[u8], options_json: Option<String>) -> Result<String, JsValue> {
    read_json_of(docx, options_json.as_deref()).map_err(js_err)
}

/// `forEdit` options: `read --for-edit`'s flags.
#[derive(Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ForEditOptions {
    paragraphs: Option<String>,
    head: Option<usize>,
    tail: Option<usize>,
    find: Option<String>,
    regex: Option<String>,
    ignore_case: bool,
    changed: bool,
    by: Option<String>,
    context: Option<usize>,
    limit: Option<usize>,
    max_chars: Option<usize>,
    all: bool,
    cursor: Option<String>,
    accept_change: Option<Vec<String>>,
    reject_change: Option<Vec<String>>,
    existing_revisions: Option<String>,
    track: Option<String>,
    /// The path as the command line spells it: the result carries the
    /// input's validity findings (#1340).
    validity_file: Option<String>,
}

fn for_edit_of(docx: &[u8], options_json: Option<&str>) -> Result<String, String> {
    let options: ForEditOptions = match options_json {
        Some(json) => serde_json::from_str(json).map_err(|e| format!("forEdit options: {e}"))?,
        None => ForEditOptions::default(),
    };
    let existing_revisions = options
        .existing_revisions
        .map(|mode| {
            jubarte::cli::ExistingArg::from_choice(&mode).ok_or_else(|| {
                format!("USAGE: invalid value '{mode}' for '--existing-revisions <MODE>'")
            })
        })
        .transpose()?;
    let track = options
        .track
        .map(|choice| {
            jubarte::cli::TrackArg::from_choice(&choice)
                .ok_or_else(|| format!("USAGE: invalid value '{choice}' for '--track <CHOICE>'"))
        })
        .transpose()?;
    let args = jubarte::cli::ReadArgs {
        paragraphs: options.paragraphs.into_iter().collect(),
        head: options.head,
        tail: options.tail,
        find: options.find,
        regex: options.regex,
        ignore_case: options.ignore_case,
        changed: options.changed,
        by: options.by,
        context: options.context,
        limit: options.limit,
        max_chars: options.max_chars,
        all: options.all,
        cursor: options.cursor,
        accept_change: options.accept_change.unwrap_or_default(),
        reject_change: options.reject_change.unwrap_or_default(),
        existing_revisions,
        track,
        ..jubarte::cli::ReadArgs::default()
    };
    let validity = options
        .validity_file
        .as_deref()
        .map(|file| jubarte::validate::input_validity_of(docx, file));
    jubarte::edit::for_edit::for_edit_json(docx, &args.for_edit_request(), validity)
        .map_err(|e| e.to_string())
}

/// The working-document JSON `jubarte read --for-edit` prints
/// (`jubarte-read-for-edit-v1`): source hash, paired target ids, exact
/// working text with `before_sha256`, and per-operation readiness. Options
/// (JSON, all optional): `paragraphs` (`"p3,p10-p20,t0"`), `head`, `tail`,
/// `find`/`regex` (+ `ignoreCase`) on the original final text, `changed`
/// with `by`, `context` (read-only original neighbours), `limit`,
/// `maxChars`, `all`, `cursor`, `acceptChange`/`rejectChange` (revision
/// ids), `existingRevisions` (`keep`, `accept`, `reject`, `refuse`,
/// `auto`) and `track` (`on`, `off`); `validityFile`, the path as the
/// command line spells it, adds `input_validity`. A `USAGE` failure throws
/// `jubarte-wasm: USAGE: …`, as the CLI exits 2; the other refusals throw
/// `jubarte-wasm: CODE: …`, as the CLI exits 3.
///
/// Mirrors `jubarte::edit::for_edit::for_edit_json`.
#[wasm_bindgen(js_name = forEdit)]
pub fn for_edit(docx: &[u8], options_json: Option<String>) -> Result<String, JsValue> {
    for_edit_of(docx, options_json.as_deref()).map_err(js_err)
}

/// Options of [`flagPlan`](flag_plan).
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct FlagPlanOptions {
    author: Option<String>,
    date: Option<String>,
    existing_revisions: Option<String>,
}

fn flag_plan_json(
    verb: &str,
    operations_json: &str,
    docx: &[u8],
    options_json: Option<&str>,
) -> Result<String, String> {
    let options: FlagPlanOptions = match options_json {
        Some(json) => serde_json::from_str(json).map_err(|e| format!("options: {e}"))?,
        None => FlagPlanOptions::default(),
    };
    let (plan, notes) = jubarte::edit::flags::plan_from_flags_json(
        verb,
        operations_json,
        options.author.as_deref().unwrap_or("Redline"),
        options.date.as_deref(),
        options.existing_revisions.as_deref().unwrap_or("auto"),
        docx,
    )?;
    Ok(serde_json::json!({ "plan": plan, "notes": notes }).to_string())
}

/// The plan the `edit`/`add` operation flags describe, as JSON `{plan,
/// notes}`: `plan` is the plan's JSON for [`applyEditPlan`](apply_edit_plan),
/// `notes` the `op-N: …` lines to print. `verb` is `edit` or `add`;
/// `operationsJson` the parsed command's `operations` array. Options (JSON,
/// all optional): `author` (default `Redline`), `date`,
/// `existingRevisions` (auto, keep, accept, reject, refuse; default auto).
/// An error is a usage error.
///
/// Mirrors `jubarte::edit::flags::plan_from_flags`.
#[wasm_bindgen(js_name = flagPlan)]
pub fn flag_plan(
    verb: &str,
    operations_json: &str,
    docx: &[u8],
    options_json: Option<String>,
) -> Result<String, JsValue> {
    flag_plan_json(verb, operations_json, docx, options_json.as_deref()).map_err(js_err)
}

/// Why `--editing-mode` cannot apply `plan` to `docx`: the tracked changes
/// it keeps and does not itself accept or reject, or a reply to a comment
/// on a paragraph it deletes; `undefined` when neither.
///
/// Mirrors `jubarte::edit::flags::editing_mode_conflict`.
#[wasm_bindgen(js_name = editingModeConflict)]
pub fn editing_mode_conflict(plan: &str, docx: &[u8]) -> Option<String> {
    jubarte::edit::flags::editing_mode_conflict_json(plan, docx)
}

/// `record_plan` as JSON: `{ok: true, recorded: {plan, json, notes,
/// summary}}`, or `{ok: false, usage, message}` (`usage` false when `held`
/// is not a plan).
fn record_plan_json(
    verb: &str,
    operations_json: &str,
    docx: &[u8],
    options_json: &str,
    held: Option<&str>,
) -> String {
    use jubarte::edit::record::{record_json, RecordError};
    match record_json(verb, operations_json, docx, held, options_json) {
        Ok(recorded) => format!(r#"{{"ok":true,"recorded":{recorded}}}"#),
        Err(e) => {
            let (usage, message) = match e {
                RecordError::Usage(m) => (true, m),
                RecordError::Plan(m) => (false, m),
            };
            serde_json::json!({ "ok": false, "usage": usage, "message": message }).to_string()
        }
    }
}

/// `edit`/`add --plan PLAN.json` with operation flags: the operations
/// recorded into `held` (PLAN.json's text, absent before the first
/// recording), as JSON `{ok: true, recorded: {plan, json, notes,
/// summary}}`: preview `plan`, write `json`, print `notes` and `summary`.
/// On failure `{ok: false, usage, message}`: a usage error (exit 2), or
/// `held` is not a plan (exit 3). Options (JSON): `author`, `date`,
/// `existingRevisions`, `file`, `plan`, `defaultPlan` (a bare `--plan`),
/// `dryRun`, `rewriteOthersComments`.
///
/// Mirrors `jubarte::edit::record::record`.
#[wasm_bindgen(js_name = recordPlan)]
pub fn record_plan(
    verb: &str,
    operations_json: &str,
    docx: &[u8],
    options_json: &str,
    held: Option<String>,
) -> String {
    record_plan_json(verb, operations_json, docx, options_json, held.as_deref())
}

/// The blocks of `docx` (a redline) carrying `author`'s marks, as the agent
/// view `edit` and `add` print (every changed block when none does);
/// `accepted` reads its accepted text, `source` is the name printed as
/// `source:`.
///
/// Mirrors `jubarte::markdown::changed_view`.
#[wasm_bindgen(js_name = changedView)]
pub fn changed_view(
    docx: &[u8],
    author: &str,
    accepted: bool,
    source: Option<String>,
) -> Result<String, JsValue> {
    jubarte::markdown::changed_view(docx, author, accepted, source.as_deref()).map_err(js_err)
}

/// The lines `edit -p rN --accept/--reject` prints for a pure resolution:
/// one per resolved change from `source` (the bytes before resolution) and
/// `resolvedJson` (`{"accepted": [...], "rejected": [...]}` short ids).
///
/// Mirrors `jubarte::edit::resolution::resolution_text`.
#[wasm_bindgen(js_name = resolutionText)]
pub fn resolution_text(docx: &[u8], resolved_json: &str) -> Result<String, JsValue> {
    let resolved: jubarte::edit::ResolvedRevisions =
        serde_json::from_str(resolved_json).map_err(js_err)?;
    jubarte::edit::resolution::resolution_text(docx, &resolved).map_err(js_err)
}

/// What [`applyEditPlan`](apply_edit_plan) and
/// [`previewEditPlan`](preview_edit_plan) return. A refused plan is data, not
/// an exception, so every operation's outcome stays readable.
#[wasm_bindgen]
pub struct EditOutput {
    ok: bool,
    clean: Option<Vec<u8>>,
    redline: Option<Vec<u8>>,
    patch: Option<String>,
    json: String,
}

#[wasm_bindgen]
impl EditOutput {
    /// `true` when the plan was applied (or resolved, for a preview).
    #[wasm_bindgen(getter)]
    pub fn ok(&self) -> bool {
        self.ok
    }

    /// The edited document without tracked changes; `undefined` on refusal
    /// and for previews.
    #[wasm_bindgen(getter)]
    pub fn clean(&self) -> Option<Vec<u8>> {
        self.clean.clone()
    }

    /// The source compared against the clean copy (Word tracked changes);
    /// `undefined` on refusal and for previews.
    #[wasm_bindgen(getter)]
    pub fn redline(&self) -> Option<Vec<u8>> {
        self.redline.clone()
    }

    /// The changes the redline tracks as a patch (see
    /// [`diffDocuments`](diff_documents)), by the plan's author and date;
    /// `undefined` on refusal and for previews.
    #[wasm_bindgen(getter)]
    pub fn patch(&self) -> Option<String> {
        self.patch.clone()
    }

    /// The report JSON when `ok`, else the error JSON (`code`, `operation`,
    /// `message`, `outcomes`).
    #[wasm_bindgen(getter)]
    pub fn json(&self) -> String {
        self.json.clone()
    }
}

fn to_json(value: &impl serde::Serialize) -> Result<String, JsValue> {
    serde_json::to_string(value).map_err(js_err)
}

/// Apply an edit plan (JSON) to a DOCX: the clean copy, the Word redline,
/// the patch of the plan's changes and the per-operation report. `name`
/// (default `document.docx`) is the file the patch names; under
/// `existing_revisions: keep` the patch shows the plan's own changes, not
/// the other party's the redline also holds.
///
/// Mirrors `jubarte::edit::apply_plan_json`.
#[wasm_bindgen(js_name = applyEditPlan)]
pub fn apply_edit_plan(
    docx: &[u8],
    plan_json: &str,
    name: Option<String>,
) -> Result<EditOutput, JsValue> {
    Ok(match jubarte::edit::apply_plan_json(docx, plan_json) {
        Ok(result) => {
            let name = name.as_deref().unwrap_or("document.docx");
            let patch_of =
                if result.report.existing_revisions == jubarte::edit::ExistingRevisions::Keep {
                    jubarte::markdown::patch_own_changes
                } else {
                    jubarte::markdown::patch_redline
                };
            let options = patch_options(name, name, &result.report.author, &result.report.date);
            EditOutput {
                ok: true,
                json: to_json(&result.report)?,
                patch: Some(
                    patch_of(&result.redline, &options)
                        .map_err(js_err)?
                        .to_string(),
                ),
                clean: Some(result.clean),
                redline: Some(result.redline),
            }
        }
        Err(error) => EditOutput {
            ok: false,
            clean: None,
            redline: None,
            patch: None,
            json: to_json(&error)?,
        },
    })
}

/// Resolve every operation of an edit plan without producing documents.
///
/// Mirrors `jubarte::edit::preview_plan`.
#[wasm_bindgen(js_name = previewEditPlan)]
pub fn preview_edit_plan(docx: &[u8], plan_json: &str) -> Result<EditOutput, JsValue> {
    let resolved =
        jubarte::edit::EditPlan::from_json_with_notes(plan_json).and_then(|(plan, notes)| {
            jubarte::edit::preview_plan(docx, &plan).map(|mut report| {
                report.notes = notes;
                report
            })
        });
    Ok(EditOutput {
        ok: resolved.is_ok(),
        clean: None,
        redline: None,
        patch: None,
        json: match resolved {
            Ok(report) => to_json(&report)?,
            Err(error) => to_json(&error)?,
        },
    })
}

fn patch_options(
    old_name: &str,
    new_name: &str,
    author: &str,
    date: &str,
) -> jubarte::markdown::PatchOptions {
    jubarte::markdown::PatchOptions {
        old_name: old_name.to_string(),
        new_name: new_name.to_string(),
        owner: jubarte::markdown::Attribution {
            author: author.to_string(),
            date: date.to_string(),
        },
    }
}

/// A side of [`diffDocuments`](diff_documents): a Word package (it starts
/// with a ZIP signature), else UTF-8 Markdown.
fn side(bytes: &[u8]) -> Result<jubarte::markdown::Source<'_>, JsValue> {
    if bytes.starts_with(b"PK\x03\x04") {
        return Ok(jubarte::markdown::Source::Docx(bytes));
    }
    std::str::from_utf8(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes))
        .map(jubarte::markdown::Source::Markdown)
        .map_err(|e| js_err(format!("a side is neither a .docx nor UTF-8 Markdown: {e}")))
}

/// Explicit input kinds supplied by the shared CLI or API caller.
#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum SideFormat {
    Docx,
    Md,
}

fn typed_side(
    bytes: &[u8],
    format: Option<SideFormat>,
) -> Result<jubarte::markdown::Source<'_>, String> {
    let format = format.unwrap_or(if bytes.starts_with(b"PK\x03\x04") {
        SideFormat::Docx
    } else {
        SideFormat::Md
    });
    match format {
        SideFormat::Docx => {
            if !bytes.starts_with(b"PK\x03\x04") {
                return Err("invalid DOCX: expected a ZIP package".to_string());
            }
            Ok(jubarte::markdown::Source::Docx(bytes))
        }
        // The native, Python and npm readers drop a UTF-8 BOM too.
        SideFormat::Md => std::str::from_utf8(bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes))
            .map(jubarte::markdown::Source::Markdown)
            .map_err(|e| format!("invalid UTF-8 Markdown: {e}")),
    }
}

#[derive(Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ViewFormat {
    #[default]
    Github,
    Word,
    Normal,
    Context,
    SideBySide,
}

impl From<ViewFormat> for jubarte::text_diff::TextFormat {
    fn from(format: ViewFormat) -> Self {
        match format {
            ViewFormat::Github => Self::Github,
            ViewFormat::Word => Self::Word,
            ViewFormat::Normal => Self::Normal,
            ViewFormat::Context => Self::Context,
            ViewFormat::SideBySide => Self::SideBySide,
        }
    }
}

fn default_context() -> u32 {
    3
}

fn deserialize_context<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    <u32 as serde::Deserialize>::deserialize(deserializer).map_err(|e| {
        serde::de::Error::custom(format!(
            "context must be an integer in the u32 range (0..4294967295): {e}"
        ))
    })
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ViewOptions {
    #[serde(default)]
    format: ViewFormat,
    old_name: Option<String>,
    new_name: Option<String>,
    #[serde(default = "default_context", deserialize_with = "deserialize_context")]
    context: u32,
    #[serde(default)]
    accept_changes: bool,
    #[serde(default)]
    full_lines: bool,
    old_format: Option<SideFormat>,
    new_format: Option<SideFormat>,
}

fn document_view(old: &[u8], next: &[u8], options_json: Option<&str>) -> Result<String, String> {
    let options: ViewOptions = serde_json::from_str(options_json.unwrap_or("{}"))
        .map_err(|e| format!("invalid diff view options: {e}"))?;
    let (old, next) = (
        typed_side(old, options.old_format)?,
        typed_side(next, options.new_format)?,
    );
    let name = |given: Option<String>, source: &jubarte::markdown::Source<'_>, default: &str| {
        given.unwrap_or_else(|| match source {
            jubarte::markdown::Source::Docx(_) => format!("{default}.docx"),
            jubarte::markdown::Source::Markdown(_) => format!("{default}.md"),
        })
    };
    let defaults = jubarte::text_diff::TextOptions::default();
    let options = jubarte::text_diff::TextOptions {
        unified: jubarte::text_diff::UnifiedOptions {
            old_name: name(options.old_name, &old, "old"),
            new_name: name(options.new_name, &next, "new"),
            context: options.context as usize,
        },
        format: options.format.into(),
        accept_changes: options.accept_changes,
        window: if options.full_lines {
            None
        } else {
            defaults.window
        },
    };
    jubarte::text_diff::diff_documents_view(old, next, &options)
}

/// Document review view. `optionsJson` is a strict camelCase object with
/// `format` (github, word, normal, context, side-by-side), `oldName`,
/// `newName`, `context` (u32), `acceptChanges`, `fullLines`, `oldFormat`
/// and `newFormat` (docx/md). Defaults use the core display window; Word
/// always accepts both inputs' revisions before creating new CriticMarkup.
#[wasm_bindgen(js_name = diffDocumentsView)]
pub fn diff_documents_view(
    old: &[u8],
    next: &[u8],
    options_json: Option<String>,
) -> Result<String, JsValue> {
    document_view(old, next, options_json.as_deref()).map_err(js_err)
}

/// Complete, unwrapped document snapshots as a Git text patch. `context`
/// is validated before wasm-bindgen can coerce booleans or wrap u32 values.
#[wasm_bindgen(js_name = diffDocumentsUnified, skip_typescript)]
pub fn diff_documents_unified(
    old: &[u8],
    next: &[u8],
    old_name: Option<String>,
    new_name: Option<String>,
    context: JsValue,
) -> Result<String, JsValue> {
    let context = if context.is_undefined() {
        3
    } else {
        let number = context
            .as_f64()
            .filter(|number| {
                number.is_finite()
                    && number.fract() == 0.0
                    && (0.0..=f64::from(u32::MAX)).contains(number)
            })
            .ok_or_else(|| js_err("context must be an integer in the u32 range (0..4294967295)"))?;
        number as u32
    };
    let (old, next) = (side(old)?, side(next)?);
    let name = |given: Option<String>, source: &jubarte::markdown::Source<'_>, default: &str| {
        given.unwrap_or_else(|| match source {
            jubarte::markdown::Source::Docx(_) => format!("{default}.docx"),
            jubarte::markdown::Source::Markdown(_) => format!("{default}.md"),
        })
    };
    let options = jubarte::text_diff::UnifiedOptions {
        old_name: name(old_name, &old, "old"),
        new_name: name(new_name, &next, "new"),
        context: context as usize,
    };
    jubarte::text_diff::diff_documents(old, next, &options).map_err(js_err)
}

#[wasm_bindgen(typescript_custom_section)]
const UNIFIED_TYPES: &str = r#"
export function diffDocumentsUnified(old: Uint8Array, next: Uint8Array, oldName?: string, newName?: string, context?: number): string;
"#;

/// Shared clap parsing, with no filesystem, clock or process access.
#[wasm_bindgen(js_name = parseCli)]
pub fn parse_cli(
    arguments_json: &str,
    program: Option<String>,
    supported_json: Option<String>,
) -> Result<String, JsValue> {
    let arguments: Vec<String> = serde_json::from_str(arguments_json).map_err(js_err)?;
    let supported: Vec<String> =
        serde_json::from_str(supported_json.as_deref().unwrap_or("[]")).map_err(js_err)?;
    Ok(jubarte::cli::parse_json(
        &arguments,
        program.as_deref().unwrap_or("jubarte-redlines"),
        &supported,
    ))
}

/// The complete document as CriticMarkup; existing paragraph patches stay separate.
#[wasm_bindgen(js_name = diffDocumentsCritic)]
pub fn diff_documents_critic(
    old: &[u8],
    next: &[u8],
    author: Option<String>,
    date: Option<String>,
) -> Result<String, JsValue> {
    let (old, next) = (side(old)?, side(next)?);
    let mut options = jubarte::markdown::RedlineOptions::default();
    // The CLI's default author (D12), not the comparer's legacy one.
    options.settings.author_for_revisions = author.unwrap_or_else(|| "Redline".to_string());
    if let Some(date) = date {
        options.settings.date_time_for_revisions = date;
    }
    match (old, next) {
        (jubarte::markdown::Source::Markdown(old), jubarte::markdown::Source::Markdown(next)) => {
            Ok(jubarte::markdown::diff_markdown(old, next))
        }
        _ => jubarte::markdown::redline(old, next, &options)
            .and_then(|docx| {
                jubarte::markdown::docx_to_markdown(
                    &docx,
                    &jubarte::markdown::MarkdownOptions {
                        input_limits: options.settings.input_limits,
                        ..Default::default()
                    },
                )
            })
            .map(|read| read.markdown)
            .map_err(js_err),
    }
}

/// DOCX/Markdown comparison written as a Word redline, for host CLI I/O.
#[wasm_bindgen(js_name = redlineDocuments)]
pub fn redline_documents(
    old: &[u8],
    next: &[u8],
    author: &str,
    date: &str,
) -> Result<Vec<u8>, JsValue> {
    let options = jubarte::markdown::RedlineOptions {
        settings: jubarte::comparer::WmlComparerSettings {
            author_for_revisions: author.to_string(),
            date_time_for_revisions: date.to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    jubarte::markdown::redline(side(old)?, side(next)?, &options).map_err(js_err)
}

/// The changes from `old` to `next` as a patch, JSON `{"text", "hunks":
/// [{"at", "removed", "text"}]}`: only the changed paragraphs, each whole,
/// with `[-old-]{+new+}` changes and CriticMarkup comments, at its
/// `pN` id in a Word document or `line:N` in Markdown. When a side is a
/// Word document, a replacement inside a paragraph reads insertion first,
/// `{+new+}[-old-]`, as Word Compare writes it.
///
/// Each side is a `.docx` package or UTF-8 Markdown
/// (`new TextEncoder().encode(text)`). `author` and `date` (ISO 8601) own
/// the changes; `columns` wraps the lines (72 by default, 0 does not);
/// the names default to `old.docx`/`old.md` and `new.docx`/`new.md`.
///
/// Mirrors `jubarte::markdown::patch_documents`.
#[wasm_bindgen(js_name = diffDocuments)]
pub fn diff_documents(
    old: &[u8],
    next: &[u8],
    author: &str,
    date: &str,
    columns: Option<u32>,
    old_name: Option<String>,
    new_name: Option<String>,
) -> Result<String, JsValue> {
    let (old, next) = (side(old)?, side(next)?);
    let name = |given: Option<String>, source: &jubarte::markdown::Source<'_>, default: &str| {
        given.unwrap_or_else(|| match source {
            jubarte::markdown::Source::Docx(_) => format!("{default}.docx"),
            jubarte::markdown::Source::Markdown(_) => format!("{default}.md"),
        })
    };
    let options = patch_options(
        &name(old_name, &old, "old"),
        &name(new_name, &next, "new"),
        author,
        date,
    );
    let redline = jubarte::markdown::RedlineOptions {
        settings: jubarte::comparer::WmlComparerSettings {
            author_for_revisions: author.to_string(),
            date_time_for_revisions: date.to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let patch =
        jubarte::markdown::patch_documents(old, next, &redline, &options).map_err(js_err)?;
    let hunks: Vec<serde_json::Value> = patch
        .hunks
        .iter()
        .map(|h| serde_json::json!({"at": h.at.to_string(), "removed": h.removed, "text": h.text}))
        .collect();
    let columns = columns.map_or(jubarte::markdown::DEFAULT_COLUMNS, |c| c as usize);
    to_json(&serde_json::json!({"text": patch.render(columns), "hunks": hunks}))
}

/// The JSON-lines form of a report (`load`, one `op` per operation,
/// `summary`), for agent logs.
#[wasm_bindgen(js_name = editReportJsonl)]
pub fn edit_report_jsonl(report_json: &str) -> Result<String, JsValue> {
    let report: jubarte::edit::EditReport = serde_json::from_str(report_json).map_err(js_err)?;
    Ok(report.to_jsonl())
}

/// What [`appendDocuments`](append_documents) returns.
#[wasm_bindgen]
pub struct AppendOutput {
    docx: Vec<u8>,
    warnings: Vec<String>,
}

#[wasm_bindgen]
impl AppendOutput {
    /// The joined document.
    #[wasm_bindgen(getter)]
    pub fn docx(&self) -> Vec<u8> {
        self.docx.clone()
    }

    /// What was not carried, as a JSON array of `CODE: message` strings
    /// (`COMMENTS_DROPPED: ...`).
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> String {
        serde_json::to_string(&self.warnings).unwrap_or_else(|_| "[]".to_string())
    }
}

/// Append B after A, carrying B's images, links, headers, styles, lists and
/// notes. `optionsJson` is `{"section_break": "next_page" | "continuous" |
/// "none", "keep_sections": bool, "comments": "drop" | "carry"}`, each
/// optional; B's comments are dropped (warned) unless `"carry"`.
///
/// Mirrors `jubarte::append::append_documents`.
#[wasm_bindgen(js_name = appendDocuments)]
pub fn append_documents(
    a: &[u8],
    b: &[u8],
    options_json: Option<String>,
) -> Result<AppendOutput, JsValue> {
    let options: jubarte::append::AppendOptions = match options_json.as_deref() {
        Some(json) if !json.trim().is_empty() => serde_json::from_str(json)
            .map_err(|e| js_err(format!("invalid append options: {e}")))?,
        _ => jubarte::append::AppendOptions::default(),
    };
    let out = jubarte::append::append_documents(a, b, &options).map_err(js_err)?;
    Ok(AppendOutput {
        docx: out.docx,
        warnings: out.warnings,
    })
}

/// Word-validity findings beyond the schema as a JSON array (`code`, `set`,
/// `severity`, `part`, `location`, `ref`, `message`, `word_fatal`,
/// `repairable`; `ref` is the short id the CLI prints for `location`);
/// `[]` is a pass.
///
/// Mirrors `jubarte::validate::validate`.
#[wasm_bindgen(js_name = validateDocument)]
pub fn validate_document(docx: &[u8]) -> Result<String, JsValue> {
    let findings = jubarte::validate::validate(docx).map_err(js_err)?;
    serde_json::to_string(&findings).map_err(js_err)
}

/// The one stderr line `read` and `changes` give an input with validity
/// findings (`input has 2 validity warnings; jubarte validate FILE lists
/// them`), or `undefined` when there is none or the package cannot be
/// validated; `file` is the path as the command line spells it.
///
/// Mirrors `jubarte::validate::input_notice`.
#[wasm_bindgen(js_name = inputNotice)]
pub fn input_notice(docx: &[u8], file: &str) -> Option<String> {
    jubarte::validate::validate(docx)
        .ok()
        .and_then(|findings| jubarte::validate::input_notice(&findings, file))
}

/// The last line `changes --json` and `comments --json` print, the input's
/// validity findings counted (`{"input_validity":{"errors":0,"warnings":2,
/// "next":"jubarte validate FILE"}}`); `file` is the path as the command
/// line spells it. A package that cannot be validated counts none.
///
/// Mirrors `jubarte::validate::input_validity_line`.
#[wasm_bindgen(js_name = inputValidityLine)]
pub fn input_validity_line(docx: &[u8], file: &str) -> Result<String, JsValue> {
    jubarte::validate::input_validity_line(&jubarte::validate::input_validity_of(docx, file))
        .map_err(js_err)
}

/// Check that every text change from `original` to `edited` is a revision
/// by `author`. Returns the findings (`UNTRACKED_EDIT`, `FOREIGN_AUTHOR`,
/// and `PERMISSION_RANGE_DROPPED` for a permission endpoint of `original`
/// that `edited` lost) as a JSON array.
///
/// Mirrors `jubarte::validate::audit_tracked`.
#[wasm_bindgen(js_name = auditTracked)]
pub fn audit_tracked(original: &[u8], edited: &[u8], author: &str) -> Result<String, JsValue> {
    let findings = jubarte::validate::audit_tracked(original, edited, author).map_err(js_err)?;
    serde_json::to_string(&findings).map_err(js_err)
}

/// Output of [`repairDocument`](repair_document).
#[wasm_bindgen]
pub struct RepairOutput {
    docx: Vec<u8>,
    json: String,
}

#[wasm_bindgen]
impl RepairOutput {
    /// The package with every repairable finding fixed.
    #[wasm_bindgen(getter)]
    pub fn docx(&self) -> Vec<u8> {
        self.docx.clone()
    }

    /// `{"repaired": [...], "remaining": [...]}`: the findings fixed and the
    /// ones the output still has.
    #[wasm_bindgen(getter)]
    pub fn json(&self) -> String {
        self.json.clone()
    }
}

/// The package with every repairable finding fixed, with the findings it
/// fixed and could not fix in `json`.
///
/// Mirrors `jubarte::validate::repair`.
#[wasm_bindgen(js_name = repairDocument)]
pub fn repair_document(docx: &[u8]) -> Result<RepairOutput, JsValue> {
    let repaired = jubarte::validate::repair(docx).map_err(js_err)?;
    let json = serde_json::json!({
        "repaired": repaired.repaired,
        "remaining": repaired.remaining,
    });
    Ok(RepairOutput {
        docx: repaired.docx,
        json: serde_json::to_string(&json).map_err(js_err)?,
    })
}

/// What [`updateFields`](update_fields) returns.
#[cfg(feature = "pdf")]
#[wasm_bindgen]
pub struct FieldsOutput {
    docx: Vec<u8>,
    json: String,
}

#[cfg(feature = "pdf")]
#[wasm_bindgen]
impl FieldsOutput {
    /// The document with refreshed field results.
    #[wasm_bindgen(getter)]
    pub fn docx(&self) -> Vec<u8> {
        self.docx.clone()
    }

    /// `{"page_count", "fields": [{"kind", "code", "paragraph", "old", "new"}]}`.
    #[wasm_bindgen(getter)]
    pub fn json(&self) -> String {
        self.json.clone()
    }
}

/// Refresh the cached results of `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and
/// `TOC` fields from jubarte's layout (page numbers are jubarte's, not
/// Word's). Full build only: it needs the layout the PDF export links.
///
/// Mirrors `jubarte::fields::update_fields`.
#[cfg(feature = "pdf")]
#[wasm_bindgen(js_name = updateFields)]
pub fn update_fields(docx: &[u8]) -> Result<FieldsOutput, JsValue> {
    let updated = jubarte::fields::update_fields(docx).map_err(js_err)?;
    let json = serde_json::json!({
        "page_count": updated.page_count,
        "fields": updated.fields,
    })
    .to_string();
    Ok(FieldsOutput {
        docx: updated.docx,
        json,
    })
}

/// Remove who touched a document: author names (as one alias), rsids, the
/// people and dates in the document properties, and comments.
/// `optionsJson` is `{"author_alias": string, "rsids": bool, "docprops":
/// bool, "comments": bool}`, a field left out off; without it, everything
/// goes under the alias `Author`.
///
/// Mirrors `jubarte::scrub::scrub`.
#[wasm_bindgen(js_name = scrubDocument)]
pub fn scrub_document(docx: &[u8], options_json: Option<String>) -> Result<Vec<u8>, JsValue> {
    let options: jubarte::scrub::ScrubOptions = match options_json.as_deref() {
        Some(json) if !json.trim().is_empty() => {
            serde_json::from_str(json).map_err(|e| js_err(format!("invalid scrub options: {e}")))?
        }
        _ => jubarte::scrub::ScrubOptions::default(),
    };
    jubarte::scrub::scrub(docx, &options).map_err(js_err)
}

/// This build's manifest (`runtime: "wasm"`): PDF and field refresh only
/// in the full build, PNG never; the `-o` formats lose what the build
/// cannot write.
fn manifest() -> jubarte::capabilities::Capabilities {
    let mut manifest = jubarte::capabilities::capabilities("wasm");
    manifest.operations.pdf = cfg!(feature = "pdf");
    manifest.operations.png = false;
    manifest.operations.fields = cfg!(feature = "pdf");
    if !cfg!(feature = "pdf") {
        manifest
            .audit_rules
            .retain(|code| code != "FONT_SUBSTITUTED");
    }
    for extensions in manifest.formats.values_mut() {
        extensions.retain(|e| e != ".png" && (cfg!(feature = "pdf") || e != ".pdf"));
    }
    manifest
}

/// What this build can do, as JSON (`runtime: "wasm"`): PDF and field
/// refresh only in the full build, PNG never.
///
/// Mirrors `jubarte::capabilities::capabilities`.
#[wasm_bindgen]
pub fn capabilities() -> Result<String, JsValue> {
    serde_json::to_string_pretty(&manifest()).map_err(js_err)
}

/// Focused edit discovery; no filters returns this runtime's edit support matrix.
#[wasm_bindgen(js_name = capabilityQuery)]
pub fn capability_query(operation: Option<String>, field: Option<String>) -> String {
    jubarte::capabilities::discovery("wasm", operation.as_deref(), field.as_deref()).to_string()
}

/// Render a focused query in the same text form as the native CLI.
#[wasm_bindgen(js_name = capabilityQueryText)]
pub fn capability_query_text(operation: Option<String>, field: Option<String>) -> String {
    jubarte::capabilities::discovery_text(&jubarte::capabilities::discovery(
        "wasm",
        operation.as_deref(),
        field.as_deref(),
    ))
}

/// [`capabilities`] as the text summary `jubarte capabilities` prints; of
/// `manifestJson` when given (a manifest a host narrowed to what it does),
/// else of this build's.
///
/// Mirrors `jubarte::capabilities::capabilities_text`.
#[wasm_bindgen(js_name = capabilitiesText)]
pub fn capabilities_text(manifest_json: Option<String>) -> Result<String, JsValue> {
    let manifest = match manifest_json {
        Some(json) => serde_json::from_str(&json).map_err(js_err)?,
        None => manifest(),
    };
    Ok(jubarte::capabilities::capabilities_text(&manifest))
}

/// The error line for an input no task reads, as the binary prints it:
/// `LEGACY_DOC: a.docx is a Word 97-2003 …`, or (`word`) `INVALID_PACKAGE:
/// a.docx is not a ZIP file: …`; `undefined` when the bytes may be read.
///
/// Mirrors `jubarte::cli::refused_input`.
#[wasm_bindgen(js_name = refusedInput)]
pub fn refused_input(path: &str, bytes: &[u8], word: bool) -> Option<String> {
    jubarte::cli::refused_input(std::path::Path::new(path), bytes, word)
}

/// Why `planJson` is no edit plan, as the binary prints it (`INVALID_PLAN:
/// …`); `undefined` for a plan. A plan that parses can still be refused when
/// applied; that refusal carries the per-operation report.
///
/// Mirrors `jubarte::edit::EditPlan::from_json`.
#[wasm_bindgen(js_name = planError)]
pub fn plan_error(plan_json: &str) -> Option<String> {
    jubarte::edit::EditPlan::from_json(plan_json)
        .err()
        .map(|e| e.to_string())
}

/// What [`renderPdf`](render_pdf) returns.
#[cfg(feature = "pdf")]
#[wasm_bindgen]
pub struct RenderOutput {
    pdf: Vec<u8>,
    json: String,
}

#[cfg(feature = "pdf")]
#[wasm_bindgen]
impl RenderOutput {
    /// The PDF.
    #[wasm_bindgen(getter)]
    pub fn pdf(&self) -> Vec<u8> {
        self.pdf.clone()
    }

    /// `{"page_count", "page_starts"}`: the pages written and each one's
    /// first line (60 characters at most), as an edit bundle's `render`
    /// line lists them.
    #[wasm_bindgen(getter)]
    pub fn json(&self) -> String {
        self.json.clone()
    }
}

/// An edited document's PDF, compressed, its changes painted as
/// `revisions` (`conventional`, `word` or `custom` with
/// `revisionPalette`), with the page facts `edit --pdf` reports. Full build
/// only.
///
/// Mirrors `jubarte::convert::render`.
#[cfg(feature = "pdf")]
#[wasm_bindgen(js_name = renderPdf)]
pub fn render_pdf(
    docx: &[u8],
    revisions: Option<String>,
    revision_palette: Option<String>,
) -> Result<RenderOutput, JsValue> {
    let revisions = jubarte::convert::RevisionStyle::from_choice(
        revisions.as_deref().unwrap_or("conventional"),
        revision_palette.as_deref(),
    )
    .map_err(|e| JsValue::from_str(&e))?;
    let rendered = jubarte::convert::render(
        docx,
        jubarte::convert::PdfOptions {
            compress: true,
            revisions,
            ..jubarte::convert::PdfOptions::default()
        },
        jubarte::convert::RenderRequest {
            pdf: true,
            png_dpi: None,
            pages: None,
        },
    )
    .map_err(js_err)?;
    let json = serde_json::json!({
        "page_count": rendered.report.page_count,
        "page_starts": rendered.report.page_starts(),
    })
    .to_string();
    Ok(RenderOutput {
        pdf: rendered.pdf.unwrap_or_default(),
        json,
    })
}

/// A Word document as `convert --to md` writes it: Markdown with its
/// changes kept, accepted or rejected (`trackChanges`: `all`, `accept`,
/// `reject`), and unless `pageMarkers` is false `<!-- page N of M -->` lines
/// from the layout, its changes painted as `revisions`. JSON
/// `{"markdown", "warning"}`: the warning says why the markers are missing.
/// Full build only.
///
/// Mirrors `jubarte::markdown::with_page_markers`.
#[cfg(feature = "pdf")]
#[wasm_bindgen(js_name = convertMarkdown)]
pub fn convert_markdown(
    docx: &[u8],
    track_changes: &str,
    page_markers: bool,
    revisions: Option<String>,
    revision_palette: Option<String>,
) -> Result<String, JsValue> {
    let choice = jubarte::markdown::TrackChanges::parse(track_changes)
        .ok_or_else(|| js_err("track_changes must be all, accept or reject"))?;
    let read = jubarte::markdown::docx_to_markdown(
        docx,
        &jubarte::markdown::MarkdownOptions {
            track_changes: choice,
            extract_media: None,
            ..jubarte::markdown::MarkdownOptions::default()
        },
    )
    .map_err(|e| js_err(format!("convert failed: {e}")))?;
    let (markdown, warning) = if page_markers {
        let revisions = jubarte::convert::RevisionStyle::from_choice(
            revisions.as_deref().unwrap_or("conventional"),
            revision_palette.as_deref(),
        )
        .map_err(|e| JsValue::from_str(&e))?;
        jubarte::markdown::with_page_markers(docx, &read.markdown, choice, revisions)
    } else {
        (read.markdown, None)
    };
    Ok(serde_json::json!({"markdown": markdown, "warning": warning}).to_string())
}

fn wrote_line_of(
    path: &str,
    format: &str,
    size: usize,
    pages: Option<usize>,
) -> Result<String, String> {
    use jubarte::cli::Format;
    let format = match format {
        "docx" => Format::Docx,
        "md" => Format::Md,
        "pdf" => Format::Pdf,
        "png" => Format::Png,
        "json" => Format::Json,
        other => return Err(format!("unknown format {other:?}")),
    };
    Ok(jubarte::cli::wrote_line(
        std::path::Path::new(path),
        format,
        size,
        pages,
    ))
}

/// The line a writing task prints on stderr for the file it wrote:
/// `wrote r.pdf (pdf, 48213 bytes, 3 pages)`. `format` is `docx`, `md`,
/// `pdf`, `png` or `json`; `pages` is a PDF's.
///
/// Mirrors `jubarte::cli::wrote_line`.
#[wasm_bindgen(js_name = wroteLine)]
pub fn wrote_line(
    path: &str,
    format: &str,
    size: usize,
    pages: Option<usize>,
) -> Result<String, JsValue> {
    wrote_line_of(path, format, size, pages).map_err(js_err)
}

/// Markdown with CriticMarkup → DOCX bytes; errors as text so native tests
/// can reach them (a `JsValue` exists only on wasm). The options are
/// [`jubarte::markdown::docx_options_from_json`]'s.
fn markdown_docx(
    text: &str,
    options_json: Option<String>,
    reference: Option<Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let options = jubarte::markdown::docx_options_from_json(
        options_json.as_deref().unwrap_or("{}"),
        reference.as_deref(),
    )?;
    jubarte::markdown::markdown_to_docx(text, &options)
        .map(|written| written.docx)
        .map_err(|e| e.to_string())
}

/// The author and timestamp a Markdown document gives the changes and
/// comments that name none, as JSON `[author, date]` (`null` for each it
/// does not tell): its front matter, else the reference `.docx`'s creator
/// and creation time. The CLI falls back to `Redline` and now.
#[wasm_bindgen(js_name = markdownStamp)]
pub fn markdown_stamp(text: &str, reference: Option<Vec<u8>>) -> String {
    serde_json::to_string(&jubarte::markdown::markdown_stamp(
        text,
        reference.as_deref(),
    ))
    .expect("two optional strings")
}

/// Markdown with CriticMarkup → DOCX bytes, as `jubarte convert draft.md`.
///
/// `optionsJson` (every field optional): `page` (`"letter"` default, or
/// `"a4"`), `author` and `date` (for a change or comment that names none;
/// given, they beat the front matter's `author:` and `last_modified:`,
/// `modified:`, `date:`, `created:`, which beat `"Redline"` and the fixed
/// epoch, so the same Markdown writes the same bytes; the CLI dates them now
/// instead), `critic` (`true`: CriticMarkup becomes tracked changes and
/// comments), `track_changes` (or `trackChanges`: `"all"`,
/// `"accept"`, `"reject"`), `tracking` (`true`/`false`: Word's Track
/// Changes switch in the output, whatever the front matter's
/// `track_changes:` says) and `scholar` (`true`: set as LaTeX's article
/// class, page numbers from page 2, as `--scholar`). An unknown field is
/// an error. `reference`, a
/// `.docx`, lends its styles and page setup, and then `page` is ignored.
/// Images are written as their alt text, and the engine's warnings are not
/// returned.
#[wasm_bindgen(js_name = markdownToDocx)]
pub fn markdown_to_docx(
    text: &str,
    options_json: Option<String>,
    reference: Option<Vec<u8>>,
) -> Result<Vec<u8>, JsValue> {
    markdown_docx(text, options_json, reference).map_err(js_err)
}

/// Audit findings as JSON `{findings, rules, layout}` (see `jubarte validate --rules`).
/// `rules` is a comma-separated list of rule sets (`a11y`, `style`,
/// `structure`) or codes; omitted or empty runs every rule. The slim build
/// has no layout pass: it leaves `FONT_SUBSTITUTED` out (naming it is an
/// error) and does not compare `NUMPAGES` caches with a page count.
#[wasm_bindgen(js_name = auditDocument)]
pub fn audit_document(docx: &[u8], rules: Option<String>) -> Result<String, JsValue> {
    let rules = rules.unwrap_or_default();
    let rules: Vec<&str> = rules
        .split(',')
        .map(str::trim)
        .filter(|rule| !rule.is_empty())
        .collect();
    #[cfg(feature = "pdf")]
    let report = jubarte::audit::audit_report(docx, &rules);
    #[cfg(not(feature = "pdf"))]
    let report = jubarte::audit::audit_report_with(docx, &rules, None);
    let report = report.map_err(js_err)?;
    serde_json::to_string_pretty(&report).map_err(js_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_views_validate_options_and_formats_without_js_coercions() {
        let old = b"Due in 30 days.\n";
        let new = b"Due in 60 days.\n";
        for format in ["github", "word", "normal", "context", "side-by-side"] {
            let options = serde_json::json!({"format": format, "context": 0, "fullLines": true, "oldName": "before.md", "newName": "after.md", "oldFormat": "md", "newFormat": "md"}).to_string();
            let text = document_view(old, new, Some(&options)).unwrap();
            assert!(text.contains("30") && text.contains("60"), "{text}");
        }
        assert!(document_view(old, new, None)
            .unwrap()
            .contains("diff --git"));
        for value in ["true", "false", "-1", "1.5", "4294967296", "null", "\"3\""] {
            assert!(
                document_view(old, new, Some(&format!("{{\"context\":{value}}}")))
                    .unwrap_err()
                    .contains("context")
            );
        }
        for options in [
            "{",
            "null",
            "[]",
            "{\"format\":\"critic\"}",
            "{\"typo\":1}",
            "{\"acceptChanges\":null}",
            "{\"oldFormat\":\"txt\"}",
        ] {
            assert!(document_view(old, new, Some(options)).is_err(), "{options}");
        }
        assert!(document_view(old, new, Some("{\"context\":4294967295}")).is_ok());
    }

    #[test]
    fn a_markdown_side_drops_its_utf8_bom() {
        let plain = b"Due in 30 days.\n";
        let bom = b"\xEF\xBB\xBFDue in 30 days.\n";
        for options in [None, Some(r#"{"oldFormat":"md","newFormat":"md"}"#)] {
            assert_eq!(
                document_view(bom, plain, options).unwrap(),
                "",
                "{options:?}"
            );
        }
    }

    #[test]
    fn document_view_typed_sources_preserve_or_accept_docx_histories() {
        let old = word("Due in {~~30~>45~~} days.\n");
        let new = word("Due in {~~60~>45~~} days.\n");
        let marked = document_view(
            &old,
            &new,
            Some(r#"{"oldFormat":"docx","newFormat":"docx"}"#),
        )
        .unwrap();
        // The writer uses Word typing order: deletion before insertion.
        // The snapshot preserves that document order.
        assert!(marked.contains("{--30--}{++45++}") && marked.contains("{--60--}{++45++}"));
        for options in [r#"{"acceptChanges":true}"#, r#"{"format":"word"}"#] {
            assert_eq!(document_view(&old, &new, Some(options)).unwrap(), "");
        }
        for options in [r#"{"oldFormat":"docx"}"#, r#"{"newFormat":"docx"}"#] {
            assert!(document_view(b"plain", b"text", Some(options))
                .unwrap_err()
                .contains("DOCX"));
        }
        assert!(
            document_view(&[255], b"text", Some(r#"{"oldFormat":"md"}"#))
                .unwrap_err()
                .contains("UTF-8")
        );
        assert!(document_view(b"plain", &[255], Some(r#"{"newFormat":"md"}"#)).is_err());
    }

    #[test]
    fn append_documents_puts_b_after_a_and_reads_options() {
        let a = word("A first.\n");
        let b = word("B second.\n");
        let out = append_documents(&a, &b, None).unwrap();
        assert_eq!(out.warnings(), "[]");
        let text = jubarte::inspect::markdown(&out.docx()).unwrap();
        assert!(
            text.find("A first.").unwrap() < text.find("B second.").unwrap(),
            "{text}"
        );
        let joined = append_documents(
            &a,
            &b,
            Some(r#"{"section_break":"continuous"}"#.to_string()),
        )
        .unwrap();
        let paragraphs = jubarte::inspect::paragraphs(&joined.docx()).unwrap();
        assert!(paragraphs.iter().all(|p| !p.page_break));
        let manifest: serde_json::Value = serde_json::from_str(&capabilities().unwrap()).unwrap();
        assert_eq!(manifest["operations"]["append"], true);
    }

    #[test]
    fn scrub_document_renames_authors_and_reads_options() {
        let red = jubarte::document_comparer::compare_documents(&word("a\n"), &word("b\n"), "Jane")
            .unwrap();
        let authors = |docx: &[u8]| -> Vec<Option<String>> {
            jubarte::changes::list_changes(docx)
                .unwrap()
                .into_iter()
                .map(|c| c.author)
                .collect()
        };
        let all = scrub_document(&red, None).unwrap();
        assert!(authors(&all).iter().all(|a| a.as_deref() == Some("Author")));
        let kept = scrub_document(&red, Some(r#"{"rsids":true}"#.to_string())).unwrap();
        assert!(authors(&kept).iter().all(|a| a.as_deref() == Some("Jane")));
        let manifest: serde_json::Value = serde_json::from_str(&capabilities().unwrap()).unwrap();
        assert_eq!(manifest["operations"]["scrub"], true);
    }

    #[test]
    fn append_documents_carries_comments_when_asked() {
        let plan = jubarte::edit::EditPlan::from_json(
            r#"{"schema_version":1,"author":"Ann","operations":[{"kind":"comment","paragraph":"p0","text":"keep"}]}"#,
        )
        .unwrap();
        let b = jubarte::edit::apply_plan(&word("B.\n"), &plan)
            .unwrap()
            .clean;
        let a = word("A.\n");
        let carried =
            append_documents(&a, &b, Some(r#"{"comments":"carry"}"#.to_string())).unwrap();
        assert_eq!(carried.warnings(), "[]");
        let comments = jubarte::comments::list_comments(&carried.docx()).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "keep");
        let dropped = append_documents(&a, &b, None).unwrap();
        assert!(dropped.warnings().contains("COMMENTS_DROPPED"));
        assert!(jubarte::comments::list_comments(&dropped.docx())
            .unwrap()
            .is_empty());
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn docx_to_pdf_moves_comments_and_keeps_changed_pages() {
        let plan = jubarte::edit::EditPlan::from_json(
            r#"{"schema_version":1,"author":"Ann","operations":[{"kind":"comment","paragraph":"p0","text":"Too low"}]}"#,
        )
        .unwrap();
        let noted = jubarte::edit::apply_plan(&word("The cap is 10.\n"), &plan)
            .unwrap()
            .clean;
        let pages = |pdf: Vec<u8>| pdf_page_count(&pdf);
        assert_eq!(
            pages(docx_to_pdf(&noted, None, None, None, None, None).unwrap()),
            1
        );
        assert_eq!(
            pages(docx_to_pdf(&noted, None, None, None, Some(true), None).unwrap()),
            2,
            "the comments are listed on a page after the last"
        );
        let long: String = (0..120).map(|i| format!("Paragraph {i}.\n\n")).collect();
        let red = jubarte::document_comparer::compare_documents(
            &word(&long),
            &word(&long.replacen("Paragraph 0.", "Paragraph zero.", 1)),
            "Ann",
        )
        .unwrap();
        let whole = pages(docx_to_pdf(&red, None, None, None, None, None).unwrap());
        assert!(whole > 1, "{whole} pages");
        assert_eq!(
            pages(docx_to_pdf(&red, None, None, None, None, Some(true)).unwrap()),
            1
        );
    }

    #[test]
    fn capabilities_report_the_wasm_runtime_without_png() {
        let manifest: serde_json::Value = serde_json::from_str(&capabilities().unwrap()).unwrap();
        assert_eq!(manifest["runtime"], "wasm");
        assert_eq!(manifest["operations"]["png"], false);
        assert_eq!(manifest["operations"]["pdf"], cfg!(feature = "pdf"));
        assert_eq!(manifest["operations"]["edit"], true);
        // `apply_plan` here takes no --skip-failed.
        assert_eq!(manifest["operations"]["edit_skip_failed"], false);
        assert_eq!(manifest["operations"]["fields"], cfg!(feature = "pdf"));
        let yes = |on: bool| if on { "yes" } else { "no" };
        let text = capabilities_text(None).unwrap();
        assert!(text.starts_with("engine: jubarte "), "{text}");
        assert!(text.contains(" (wasm)\n"), "{text}");
        assert!(text.contains("\npng: no\n"), "{text}");
        assert!(text.contains(&format!("\npdf: {}\n", yes(cfg!(feature = "pdf")))));
        assert!(text.contains(&format!("\nfields: {}\n", yes(cfg!(feature = "pdf")))));
        assert!(!text.contains(".png"), "{text}");
        assert_eq!(text.contains(" .pdf"), cfg!(feature = "pdf"), "{text}");
        let formats = manifest["formats"].to_string();
        assert!(!formats.contains(".png"), "{formats}");
        assert_eq!(formats.contains(".pdf"), cfg!(feature = "pdf"), "{formats}");
    }

    #[test]
    fn capabilities_text_reads_a_narrowed_manifest() {
        let mut manifest: serde_json::Value =
            serde_json::from_str(&capabilities().unwrap()).unwrap();
        manifest["operations"]["scrub"] = false.into();
        let text = capabilities_text(Some(manifest.to_string())).unwrap();
        assert!(text.contains("\nscrub: no\n"), "{text}");
        assert!(text.contains("\nedit: yes\n"), "{text}");
    }

    #[test]
    fn refused_input_and_plan_error_are_the_binary_lines() {
        assert_eq!(
            refused_input("fake.docx", b"hello", true).as_deref(),
            Some(
                "INVALID_PACKAGE: fake.docx is not a ZIP file: no end-of-central-directory record"
            )
        );
        // Markdown is no ZIP and is read.
        assert_eq!(refused_input("a.md", b"# Title", false), None);
        assert!(
            refused_input("ole.docx", b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1rest", false)
                .is_some_and(|line| line.starts_with("LEGACY_DOC: ole.docx is a Word 97-2003"))
        );
        assert_eq!(
            plan_error("not json at all").as_deref(),
            Some("INVALID_PLAN: expected ident at line 1 column 2")
        );
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn update_fields_writes_numpages_and_reports_it() {
        let source = word("One\n\nTwo\n");
        let out = update_fields(&source).unwrap();
        let report: serde_json::Value = serde_json::from_str(&out.json()).unwrap();
        assert_eq!(report["page_count"], 1);
        assert_eq!(report["fields"], serde_json::json!([]));
        assert!(!out.docx().is_empty());
    }

    #[test]
    fn input_limits_json_overrides_the_compare_budget_key_by_key() {
        let base = jubarte::admission::InputLimits::compare();
        assert_eq!(settings_with_limits(None).unwrap().input_limits, base);
        let tight = settings_with_limits(Some(r#"{"max_entries": 2}"#)).unwrap();
        assert_eq!(
            tight.input_limits,
            jubarte::admission::InputLimits {
                max_entries: 2,
                ..base
            }
        );
        let typo = settings_with_limits(Some(r#"{"max_entrys": 2}"#)).unwrap_err();
        assert!(typo.starts_with("invalid input limits"), "{typo}");
        // `js_err` needs a JS host, so only the accepted path runs natively.
        let redline = compare_documents(
            &word(OLD),
            &word(NEW),
            "A",
            Some(r#"{"max_entries": 100}"#.into()),
        )
        .unwrap();
        assert!(get_revisions(&redline, None)
            .unwrap()
            .contains("\"Inserted\""));
    }

    #[test]
    fn audit_document_locates_findings_and_selects_rules() {
        let docx = word("• typed bullet\n");
        let report: serde_json::Value =
            serde_json::from_str(&audit_document(&docx, Some("style, a11y".into())).unwrap())
                .unwrap();
        assert_eq!(report["layout"], false);
        let bullet = report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["code"] == "LITERAL_BULLET")
            .expect("a typed bullet is a finding");
        assert_eq!(bullet["location"], "p0");
        assert_eq!(report["rules"].as_array().unwrap().len(), 7);
        let all: serde_json::Value =
            serde_json::from_str(&audit_document(&docx, None).unwrap()).unwrap();
        let manifest: serde_json::Value = serde_json::from_str(&capabilities().unwrap()).unwrap();
        assert_eq!(all["rules"], manifest["audit_rules"]);
        let full = cfg!(feature = "pdf");
        assert_eq!(
            all["rules"].as_array().unwrap().len(),
            if full { 9 } else { 8 }
        );
        assert_eq!(all["layout"], full);
    }

    const OLD: &str = "# Terms\n\nPayment is due in 30 days.\n\n- Delivery\n- Warranty\n";
    const NEW: &str = "# Terms\n\nPayment is due in 45 days.\n\n- Delivery\n- Warranty\n";
    const OWNER: (&str, &str) = ("Arthur Rodrigues", "2026-09-30T14:05:00Z");

    fn word(markdown: &str) -> Vec<u8> {
        jubarte::markdown::markdown_to_docx(markdown, &Default::default())
            .unwrap()
            .docx
    }

    fn diffed(old: &[u8], new: &[u8], columns: Option<u32>) -> serde_json::Value {
        let json = diff_documents(old, new, OWNER.0, OWNER.1, columns, None, None).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn markdown_sides_are_diffed_by_line_and_word_sides_by_paragraph_id() {
        let out = diffed(OLD.as_bytes(), NEW.as_bytes(), None);
        assert_eq!(
            out["text"],
            "--- a/old.md\n+++ b/new.md\tArthur Rodrigues\t2026-09-30T14:05:00Z\n\
             @@ [line:3] @@\nPayment is due in [-30-]{+45+} days.\n"
        );
        assert_eq!(out["hunks"][0]["at"], "line:3");
        assert_eq!(out["hunks"][0]["removed"], false);
        let out = diffed(&word(OLD), NEW.as_bytes(), None);
        let text = out["text"].as_str().unwrap();
        assert!(text.starts_with("--- a/old.docx\n+++ b/new.md\t"), "{text}");
        // Compare keeps its insertion-first order, unlike typed replacements
        // and the Markdown-only diff above. The patch follows its XML.
        assert!(
            text.contains("@@ [p1] @@\nPayment is due in {+45+}[-30-] days.\n"),
            "{text}"
        );
        let json = diff_documents(
            &word(OLD),
            &word(NEW),
            OWNER.0,
            OWNER.1,
            None,
            Some("contract.docx".into()),
            Some("contract-v2.docx".into()),
        )
        .unwrap();
        assert!(
            json.contains("--- a/contract.docx\\n+++ b/contract-v2.docx\\t"),
            "{json}"
        );
    }

    #[test]
    fn columns_wrap_the_patch_and_zero_does_not() {
        let long = "word ".repeat(40);
        let (old, new) = (format!("{long}old.\n"), format!("{long}new.\n"));
        let body = |columns| {
            let out = diffed(old.as_bytes(), new.as_bytes(), columns);
            out["text"].as_str().unwrap().lines().skip(3).count()
        };
        assert!(body(None) > 1);
        assert_eq!(body(None), body(Some(72)));
        assert_eq!(body(Some(0)), 1);
    }

    #[test]
    fn an_applied_plan_carries_the_patch_of_its_redline() {
        let source = word(OLD);
        let plan = r#"{"schema_version":1,"author":"Claude","date":"2026-09-25T12:00:00Z","operations":[
            {"kind":"replace","paragraph":{"index":1},"find":"30","replacement":"45"}]}"#;
        let out = apply_edit_plan(&source, plan, None).unwrap();
        assert!(out.ok());
        assert_eq!(
            out.patch().unwrap(),
            "--- a/document.docx\n+++ b/document.docx\tClaude\t2026-09-25T12:00:00Z\n\
             @@ [p1] @@\nPayment is due in [-30-]{+45+} days.\n"
        );
        // The bundle's patch names the edited file, as the binary's does.
        let named = apply_edit_plan(&source, plan, Some("terms.docx".into())).unwrap();
        assert!(named
            .patch()
            .unwrap()
            .starts_with("--- a/terms.docx\n+++ b/terms.docx\t"));
        assert!(preview_edit_plan(&source, plan).unwrap().patch().is_none());
    }

    #[test]
    fn a_plan_that_keeps_revisions_patches_only_its_own_changes() {
        let source = word("Due in {~~30~>45~~} days.\n\nOther terms.\n");
        let plan = r#"{"schema_version":1,"author":"Claude","date":"2026-09-25T12:00:00Z","existing_revisions":"keep","operations":[
            {"kind":"replace","paragraph":{"index":1},"find":"Other","replacement":"Further"}]}"#;
        let out = apply_edit_plan(&source, plan, None).unwrap();
        assert!(out.ok(), "{}", out.json());
        let patch = out.patch().unwrap();
        assert!(patch.contains("[-Other-]{+Further+}"), "{patch}");
        assert!(
            !patch.contains("30"),
            "the other party's change is not the plan's: {patch}"
        );
    }

    #[test]
    fn flag_plan_builds_the_plan_the_cli_flags_describe() {
        let source = word(OLD);
        let ops = r#"[{"at":"p1","anchor":"30","content":"45","delete":false,"resolve":false,"before":false,"comment":false,"styles":[]}]"#;
        let out: serde_json::Value = serde_json::from_str(
            &flag_plan_json("edit", ops, &source, Some(r#"{"author":"Ann"}"#)).unwrap(),
        )
        .unwrap();
        let plan = out["plan"].as_str().unwrap();
        assert!(plan.contains(r#""author":"Ann""#), "{plan}");
        let edited = jubarte::edit::apply_plan_json(&source, plan).unwrap();
        let view = jubarte::markdown::changed_view(&edited.redline, "Ann", false, None).unwrap();
        assert!(view.contains("{~~30~>45~~}"), "{view}");
        assert!(flag_plan_json("edit", "[]", &source, None).is_err());
        assert!(flag_plan_json("edit", ops, &source, Some("{")).is_err());
    }

    #[test]
    fn record_plan_records_into_the_held_plan_and_says_why_it_cannot() {
        let source = word(OLD);
        let ops = r#"[{"at":"p1","anchor":"30","content":"45"}]"#;
        let options = r#"{"author":"Ann","file":"a.docx","plan":"p.json"}"#;
        let first: serde_json::Value =
            serde_json::from_str(&record_plan_json("edit", ops, &source, options, None)).unwrap();
        assert_eq!(first["ok"], true, "{first}");
        let held = first["recorded"]["json"].as_str().unwrap();
        let second: serde_json::Value = serde_json::from_str(&record_plan_json(
            "add",
            r#"[{"at":"p1","comment":true,"content":"Why?"}]"#,
            &source,
            options,
            Some(held),
        ))
        .unwrap();
        assert!(
            second["recorded"]["summary"]
                .as_str()
                .unwrap()
                .starts_with("recorded 1 operation in p.json: 2 operations"),
            "{second}"
        );
        let plan = second["recorded"]["plan"].as_str().unwrap();
        let edited = jubarte::edit::apply_plan_json(&source, plan).unwrap();
        let view = jubarte::markdown::changed_view(&edited.redline, "Ann", false, None).unwrap();
        assert!(
            view.contains("{~~30~>45~~}") && view.contains("Why?"),
            "{view}"
        );
        let bob = r#"{"author":"Bob","file":"a.docx","plan":"p.json"}"#;
        let refused: serde_json::Value =
            serde_json::from_str(&record_plan_json("edit", ops, &source, bob, Some(held))).unwrap();
        assert_eq!(
            (&refused["ok"], &refused["usage"]),
            (&false.into(), &true.into())
        );
        let not_plan: serde_json::Value =
            serde_json::from_str(&record_plan_json("edit", ops, &source, options, Some("{}")))
                .unwrap();
        assert_eq!(not_plan["usage"], false, "{not_plan}");
    }

    #[test]
    fn comments_list_threads_as_a_json_array_and_filter() {
        let source = word(OLD);
        let first = jubarte::edit::apply_plan_json(
            &source,
            r#"{"schema_version":1,"author":"Ann","operations":[
            {"kind":"comment","paragraph":{"index":1},"find":"30","text":"Too short"}]}"#,
        )
        .unwrap();
        let second = jubarte::edit::apply_plan_json(
            &first.clean,
            r#"{"schema_version":1,"author":"Bob","operations":[
            {"kind":"reply_comment","comment_id":0,"text":"Agreed"}]}"#,
        )
        .unwrap();
        let all: serde_json::Value =
            serde_json::from_str(&list_comments(&second.clean, None, None).unwrap()).unwrap();
        assert_eq!(all.as_array().unwrap().len(), 2);
        assert_eq!(all[0]["anchor_text"], "30");
        assert_eq!(all[1]["parent"], 0);
        let bob: serde_json::Value =
            serde_json::from_str(&list_comments(&second.clean, Some("Bob".into()), None).unwrap())
                .unwrap();
        assert_eq!(bob.as_array().unwrap().len(), 1);
        let latest: serde_json::Value =
            serde_json::from_str(&list_comments(&second.clean, None, Some(true)).unwrap()).unwrap();
        assert_eq!(latest[0]["text"], "Agreed");
        assert_eq!(list_comments(&source, None, None).unwrap(), "[]");
    }

    #[test]
    fn a_refused_plan_is_data_with_its_code() {
        let out = apply_edit_plan(
            b"not a zip",
            r#"{"schema_version":1,"author":"A","operations":[]}"#,
            None,
        )
        .unwrap();
        assert!(!out.ok());
        assert!(out.clean().is_none() && out.redline().is_none() && out.patch().is_none());
        let error: serde_json::Value = serde_json::from_str(&out.json()).unwrap();
        assert_eq!(error["code"], "INVALID_PACKAGE");
        let preview = preview_edit_plan(b"x", "{").unwrap();
        assert!(!preview.ok());
        assert!(preview.json().contains("INVALID_PLAN"));
    }

    fn document_xml(docx: &[u8]) -> String {
        jubarte::opc::PartFs::open(docx)
            .unwrap()
            .part_string("word/document.xml")
            .unwrap()
    }

    fn page_width(docx: &[u8]) -> String {
        let xml = document_xml(docx);
        let at = xml.find("<w:pgSz ").unwrap();
        let tag = &xml[at..at + xml[at..].find('>').unwrap()];
        let width = tag.find("w:w=\"").unwrap() + 5;
        tag[width..width + tag[width..].find('"').unwrap()].to_string()
    }

    #[test]
    fn markdown_is_written_as_word_on_letter_or_a4() {
        let letter = markdown_docx("# Terms\n\nBody.\n", None, None).unwrap();
        assert_eq!(&letter[..2], b"PK");
        assert_eq!(page_width(&letter), "12240");
        let a4 = markdown_docx("Body.\n", Some(r#"{"page":"a4"}"#.into()), None).unwrap();
        assert_eq!(page_width(&a4), "11906");
        assert_eq!(
            page_width(&markdown_docx("Body.\n", Some("{}".into()), None).unwrap()),
            "12240"
        );
    }

    #[test]
    fn markdown_options_set_the_owner_and_resolve_the_changes() {
        const DRAFT: &str = "Payment is due in {~~30~>45~~} days.\n";
        let kept = markdown_docx(
            DRAFT,
            Some(r#"{"author":"Legal","date":"2026-10-02T00:00:00Z"}"#.into()),
            None,
        )
        .unwrap();
        let xml = document_xml(&kept);
        assert!(xml.contains("w:author=\"Legal\""), "{xml}");
        assert!(xml.contains("w:date=\"2026-10-02T00:00:00Z\""), "{xml}");

        for (options, text) in [
            (
                r#"{"track_changes":"accept"}"#,
                "Payment is due in 45 days.",
            ),
            (r#"{"trackChanges":"reject"}"#, "Payment is due in 30 days."),
            (
                r#"{"critic":false}"#,
                "Payment is due in {~~30~>45~~} days.",
            ),
        ] {
            let docx = markdown_docx(DRAFT, Some(options.into()), None).unwrap();
            let paragraphs = jubarte::inspect::paragraphs(&docx).unwrap();
            assert_eq!(paragraphs[0].text, text, "{options}");
        }
    }

    #[test]
    fn a_reference_lends_its_page_and_bad_options_are_refused() {
        let reference =
            markdown_docx("Template.\n", Some(r#"{"page":"a4"}"#.into()), None).unwrap();
        let docx = markdown_docx("Body.\n", None, Some(reference)).unwrap();
        assert_eq!(page_width(&docx), "11906");
        for bad in [
            r#"{"page":"legal"}"#,
            r#"{"track_changes":"keep"}"#,
            r#"{"pages":"a4"}"#,
            "{",
        ] {
            assert!(
                markdown_docx("Body.\n", Some(bad.into()), None).is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn markdown_to_docx_takes_scholar_and_tracking() {
        let part = |docx: &[u8], name: &str| {
            jubarte::opc::PartFs::open(docx)
                .unwrap()
                .part_string(name)
                .unwrap()
        };
        let docx = markdown_docx(
            "# Title\n\nBody.\n",
            Some(r#"{"scholar":true,"tracking":true}"#.into()),
            None,
        )
        .unwrap();
        assert!(part(&docx, "word/styles.xml").contains("Latin Modern Roman"));
        assert!(part(&docx, "word/settings.xml").contains("<w:trackRevisions"));
        assert!(part(&docx, "word/document.xml").contains("<w:titlePg/>"));
        let plain = markdown_docx("Body.\n", None, None).unwrap();
        assert!(!part(&plain, "word/styles.xml").contains("Latin Modern Roman"));
        assert!(!part(&plain, "word/settings.xml").contains("<w:trackRevisions"));
    }

    #[test]
    fn the_critic_view_s_default_author_is_redline() {
        let (old, new) = (word("Due in 30 days.\n"), word("Due in 45 days.\n"));
        let view = diff_documents_critic(&old, &new, None, None).unwrap();
        assert!(view.contains("Redline"), "{view}");
        assert!(!view.contains("Open-Xml-PowerTools"), "{view}");
    }

    #[test]
    fn listings_are_the_binary_text() {
        let docx = word("Due in {~~30~>45~~} days.\n\n| A | B |\n|---|---|\n| 1 | 2 |\n");
        let changes = changes_text(&docx, None).unwrap();
        // Word typing order writes the deletion, then the insertion; the
        // listing is the binary's: one logical change of both marks, by the
        // neutral default author (#916).
        assert_eq!(
            changes,
            "r1+2\treplacement\ttext\tJubarte\tp0\t\"30 \u{2192} 45\"\t(2 elements)\n1 change(s)\n"
        );
        let tables = tables_text(&docx, None).unwrap();
        assert!(tables.starts_with("table 0: 2x2"), "{tables}");
        assert_eq!(
            tables_text(&word("No grid.\n"), None).unwrap(),
            "no tables\n"
        );
        assert_eq!(comments_text(&docx, None, None).unwrap(), "0 comment(s)\n");
    }

    #[test]
    fn input_notice_counts_the_inputs_findings() {
        // #1256: the line `read` and `changes` print, as the CLI's.
        let clean = word("Due in 30 days.\n");
        assert_eq!(input_notice(&clean, "a.docx"), None);
        let mut pkg = jubarte::opc::PartFs::open(&clean).unwrap();
        let xml = String::from_utf8(pkg.part_bytes("word/document.xml").unwrap().to_vec()).unwrap();
        // Text inside a deletion as `w:t`: one Word-fatal error.
        let broken = xml.replacen(
            "<w:body>",
            r#"<w:body><w:p><w:del w:id="90" w:author="A"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#,
            1,
        );
        assert_ne!(broken, xml);
        pkg.set_part("word/document.xml", broken.into_bytes());
        let docx = pkg.to_zip().unwrap();
        assert_eq!(
            input_notice(&docx, "my file.docx").as_deref(),
            Some("input has 1 validity error; jubarte validate 'my file.docx' lists it")
        );
        assert_eq!(input_notice(b"no package", "x.docx"), None);
        // #1309: the JSON reads carry the counts instead.
        assert_eq!(
            input_validity_line(&clean, "a.docx").unwrap(),
            r#"{"input_validity":{"errors":0,"warnings":0}}"#
        );
        assert_eq!(
            input_validity_line(&docx, "my file.docx").unwrap(),
            r#"{"input_validity":{"errors":1,"warnings":0,"next":"jubarte validate 'my file.docx'"}}"#
        );
        let snapshot: serde_json::Value = serde_json::from_str(
            &read_json_of(&docx, Some(r#"{"validityFile":"my file.docx"}"#)).unwrap(),
        )
        .unwrap();
        assert_eq!(
            snapshot["summary"]["input_validity"],
            serde_json::json!({"errors":1,"warnings":0,"next":"jubarte validate 'my file.docx'"})
        );
        // Without it, the snapshot is the library's, as before.
        let plain: serde_json::Value =
            serde_json::from_str(&read_json_of(&docx, None).unwrap()).unwrap();
        assert!(plain["summary"].get("input_validity").is_none(), "{plain}");
    }

    #[test]
    fn tables_take_the_view_picks_and_the_rejected_text() {
        // #1255, #1161: `--tables -p` reads the view's selection, as the CLI.
        let docx =
            word("| A | B |\n|---|---|\n| {~~30~>45~~} | 2 |\n\nBetween.\n\n| C |\n|---|\n| 3 |\n");
        let markdown = |json: &str| -> String {
            let view: serde_json::Value =
                serde_json::from_str(&tables_view_of(&docx, Some(json)).unwrap()).unwrap();
            view["markdown"].as_str().unwrap().to_string()
        };
        let view = |json: &str| -> String {
            let view: serde_json::Value =
                serde_json::from_str(&read_view_json(&docx, Some(json)).unwrap()).unwrap();
            view["markdown"].as_str().unwrap().to_string()
        };
        let all = tables_text_of(&docx, None).unwrap();
        assert_eq!(markdown("{}"), all);
        let grids: Vec<&str> = all.split_inclusive('\n').collect();
        let (first, second) = (grids[..3].concat(), grids[3..].concat());
        assert!(
            second.starts_with("table 1: ") && second.contains('C'),
            "{all}"
        );
        // The grids it reaches, an empty line, then the view of the pick.
        for (pick, grids) in [
            ("t1", second.clone()),
            ("t1,t0", all.clone()),
            ("p2", first.clone()),
            ("p3-p5", all.clone()),
        ] {
            let json = format!(r#"{{"paragraphs":"{pick}","pageMarkers":false}}"#);
            assert_eq!(
                markdown(&json),
                format!("{grids}\n{}", view(&json)),
                "{pick}"
            );
        }
        let rejected = markdown(r#"{"paragraphs":"t0","trackChanges":"reject"}"#);
        let (grids, _) = rejected.split_once("\n---\n").unwrap();
        assert!(grids.contains("30") && !grids.contains("45"), "{rejected}");
        // A pick that reaches no table is a usage error; the view's own
        // refusal is the view's.
        let outside = tables_view_of(&docx, Some(r#"{"paragraphs":"p4"}"#)).unwrap_err();
        assert_eq!(
            outside,
            "USAGE: p4 is not in a table; --tables -p takes tN, or a pN inside a table"
        );
        let missing = tables_view_of(&docx, Some(r#"{"paragraphs":"t5"}"#)).unwrap_err();
        assert!(
            missing.ends_with("t5 is not a table of this document") && !missing.contains("USAGE"),
            "{missing}"
        );
        assert!(tables_view_of(&docx, Some(r#"{"bogus":1}"#)).is_err());
        assert!(tables_text_of(&docx, Some(r#"{"paragraphs":"t0"}"#)).is_err());
        let rejected = tables_text_of(&docx, Some(r#"{"trackChanges":"reject"}"#)).unwrap();
        assert!(
            rejected.contains("30") && !rejected.contains("45"),
            "{rejected}"
        );
    }

    #[test]
    fn json_errors_read_the_command_line_as_the_binary_does() {
        let args = |a: &[&str]| a.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
        assert!(wants_json(args(&["read", "f.docx", "--json"])));
        assert!(!wants_json(args(&["read", "f.docx"])));
        assert!(!wants_json(args(&["read", "f.docx", "--", "--json"])));
        assert!(!wants_json(args(&[
            "edit", "replace", "f.docx", "--find", "x", "--text", "--json"
        ])));
        let line: serde_json::Value = serde_json::from_str(&runtime_json_error(
            "INVALID_PACKAGE: f.docx is not a ZIP file",
            1,
        ))
        .unwrap();
        assert_eq!(line["error"]["code"], "INVALID_PACKAGE");
        assert_eq!(
            line["error"]["message"],
            "INVALID_PACKAGE: f.docx is not a ZIP file"
        );
        let usage: serde_json::Value =
            serde_json::from_str(&runtime_json_error("bad -p", 2)).unwrap();
        assert_eq!(usage["error"]["code"], "USAGE");
    }

    #[test]
    fn read_json_selects_as_the_view_does() {
        let docx = word("One.\n\nDue in {~~30~>45~~} days.\n");
        let whole: serde_json::Value =
            serde_json::from_str(&read_json_of(&docx, None).unwrap()).unwrap();
        assert_eq!(whole["paragraphs"].as_array().unwrap().len(), 2);
        // Snapshot schema 3 lists logical changes: the substitution is one
        // replacement of its two marks.
        let changes = whole["changes"].as_array().unwrap();
        assert_eq!(changes.len(), 1, "{changes:?}");
        assert_eq!(changes[0]["kind"], "replacement");
        assert_eq!(changes[0]["members"].as_array().unwrap().len(), 2);
        let one: serde_json::Value = serde_json::from_str(
            &read_json_of(&docx, Some(r#"{"paragraphs":"p0","source":"x.docx"}"#)).unwrap(),
        )
        .unwrap();
        assert_eq!(one["paragraphs"].as_array().unwrap().len(), 1);
        assert_eq!(one["changes"].as_array().unwrap().len(), 0);
        assert!(read_json_of(&docx, Some(r#"{"paragraphs":"c5"}"#))
            .unwrap_err()
            .contains("expected pN"));
        assert!(read_json_of(&docx, Some(r#"{"typo":1}"#)).is_err());
    }

    #[test]
    fn wrote_lines_and_change_filters_are_the_binary_s() {
        assert_eq!(
            wrote_line_of("out/r.pdf", "pdf", 10, Some(3)).unwrap(),
            "wrote out/r.pdf (pdf, 10 bytes, 3 pages)"
        );
        assert_eq!(
            wrote_line_of("r.md", "md", 1, None).unwrap(),
            "wrote r.md (md, 1 bytes)"
        );
        assert!(wrote_line_of("r.x", "tiff", 1, None).is_err());
        assert_eq!(
            change_filter_of(r#"{"ids":["r1+2"],"authors":["A"]}"#).unwrap(),
            r#"{"ids":["r1","r2"],"authors":["A"]}"#
        );
        let bad = change_filter_of(r#"{"ids":["c5"]}"#).unwrap_err();
        assert!(bad.starts_with("c5: expected a change id"), "{bad}");
        assert!(change_filter_of(r#"{"typo":[]}"#)
            .unwrap_err()
            .contains("invalid change filter"));
    }
}
