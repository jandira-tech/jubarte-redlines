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
//! const text = documentMarkdown(bytes);                 // [body:p:N] ids
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

/// List the tracked changes one by one as a JSON array string, each with the
/// id `acceptChanges` / `rejectChanges` select by (the same objects as
/// `jubarte changes --json`: `id`, `kind`, `target`, `author`, `date`,
/// `text`, `move_name`, `move_side`, `inside`).
///
/// Mirrors `jubarte::changes::list_changes`.
#[wasm_bindgen(js_name = listChanges)]
pub fn list_changes(docx: &[u8]) -> Result<String, JsValue> {
    let changes = jubarte::changes::list_changes(docx).map_err(js_err)?;
    serde_json::to_string(&changes).map_err(js_err)
}

/// List every comment as a JSON array string (the objects `jubarte comments
/// --json` prints: `id`, `author`, `initials`, `date`, `text`, `parent`,
/// `done`, `paragraph`, `anchor_text`, `before`, `after`). `author` keeps
/// one author's comments; `latest` keeps the newest comment of each thread.
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

/// List the tracked revisions in a DOCX as a JSON array string — the same
/// object shape as the CLI `jubarte revisions --json` lines
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
/// touches (a document without changes keeps its first page).
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
/// `summary` and `paragraphs` (ids, text, style, formatting spans,
/// limitations). Oversized or malformed packages are refused before parsing.
///
/// Mirrors `jubarte::inspect::inspect_json`.
#[wasm_bindgen(js_name = inspectDocument)]
pub fn inspect_document(docx: &[u8]) -> Result<String, JsValue> {
    jubarte::inspect::inspect_json(docx).map_err(js_err)
}

/// Body paragraphs as Markdown, each preceded by its `[body:p:N]` id: the
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

/// Apply an edit plan (JSON) to a DOCX: the clean copy, the Word redline and
/// the per-operation report.
///
/// Mirrors `jubarte::edit::apply_plan_json`.
#[wasm_bindgen(js_name = applyEditPlan)]
pub fn apply_edit_plan(docx: &[u8], plan_json: &str) -> Result<EditOutput, JsValue> {
    Ok(match jubarte::edit::apply_plan_json(docx, plan_json) {
        Ok(result) => EditOutput {
            ok: true,
            json: to_json(&result.report)?,
            patch: Some(
                jubarte::markdown::patch_redline(
                    &result.redline,
                    &patch_options(
                        "document.docx",
                        "document.docx",
                        &result.report.author,
                        &result.report.date,
                    ),
                )
                .map_err(js_err)?
                .to_string(),
            ),
            clean: Some(result.clean),
            redline: Some(result.redline),
        },
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
    let resolved = jubarte::edit::EditPlan::from_json(plan_json)
        .and_then(|plan| jubarte::edit::preview_plan(docx, &plan));
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

fn document_view(old: &[u8], new: &[u8], options_json: Option<&str>) -> Result<String, String> {
    let options: ViewOptions = serde_json::from_str(options_json.unwrap_or("{}"))
        .map_err(|e| format!("invalid diff view options: {e}"))?;
    let (old, new) = (
        typed_side(old, options.old_format)?,
        typed_side(new, options.new_format)?,
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
            new_name: name(options.new_name, &new, "new"),
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
    jubarte::text_diff::diff_documents_view(old, new, &options)
}

/// Document review view. `optionsJson` is a strict camelCase object with
/// `format` (github, word, normal, context, side-by-side), `oldName`,
/// `newName`, `context` (u32), `acceptChanges`, `fullLines`, `oldFormat`
/// and `newFormat` (docx/md). Defaults use the core display window; Word
/// always accepts both inputs' revisions before creating new CriticMarkup.
#[wasm_bindgen(js_name = diffDocumentsView)]
pub fn diff_documents_view(
    old: &[u8],
    new: &[u8],
    options_json: Option<String>,
) -> Result<String, JsValue> {
    document_view(old, new, options_json.as_deref()).map_err(js_err)
}

/// Complete, unwrapped document snapshots as a Git text patch. `context`
/// is validated before wasm-bindgen can coerce booleans or wrap u32 values.
#[wasm_bindgen(js_name = diffDocumentsUnified, skip_typescript)]
pub fn diff_documents_unified(
    old: &[u8],
    new: &[u8],
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
    let (old, new) = (side(old)?, side(new)?);
    let name = |given: Option<String>, source: &jubarte::markdown::Source<'_>, default: &str| {
        given.unwrap_or_else(|| match source {
            jubarte::markdown::Source::Docx(_) => format!("{default}.docx"),
            jubarte::markdown::Source::Markdown(_) => format!("{default}.md"),
        })
    };
    let options = jubarte::text_diff::UnifiedOptions {
        old_name: name(old_name, &old, "old"),
        new_name: name(new_name, &new, "new"),
        context: context as usize,
    };
    jubarte::text_diff::diff_documents(old, new, &options).map_err(js_err)
}

#[wasm_bindgen(typescript_custom_section)]
const UNIFIED_TYPES: &str = r#"
export function diffDocumentsUnified(old: Uint8Array, _new: Uint8Array, oldName?: string, newName?: string, context?: number): string;
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
    new: &[u8],
    author: Option<String>,
    date: Option<String>,
) -> Result<String, JsValue> {
    let (old, new) = (side(old)?, side(new)?);
    let mut options = jubarte::markdown::RedlineOptions::default();
    if let Some(author) = author {
        options.settings.author_for_revisions = author;
    }
    if let Some(date) = date {
        options.settings.date_time_for_revisions = date;
    }
    match (old, new) {
        (jubarte::markdown::Source::Markdown(old), jubarte::markdown::Source::Markdown(new)) => {
            Ok(jubarte::markdown::diff_markdown(old, new))
        }
        _ => jubarte::markdown::redline(old, new, &options)
            .and_then(|docx| jubarte::markdown::docx_to_markdown(&docx, &Default::default()))
            .map(|read| read.markdown)
            .map_err(js_err),
    }
}

/// DOCX/Markdown comparison written as a Word redline, for host CLI I/O.
#[wasm_bindgen(js_name = redlineDocuments)]
pub fn redline_documents(
    old: &[u8],
    new: &[u8],
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
    jubarte::markdown::redline(side(old)?, side(new)?, &options).map_err(js_err)
}

/// The changes from `old` to `new` as a patch, JSON `{"text", "hunks":
/// [{"at", "removed", "text"}]}`: only the changed paragraphs, each whole,
/// with `[-old-]{+new+}` changes and CriticMarkup comments, at its
/// `body:p:N` id in a Word document or `line:N` in Markdown.
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
    new: &[u8],
    author: &str,
    date: &str,
    columns: Option<u32>,
    old_name: Option<String>,
    new_name: Option<String>,
) -> Result<String, JsValue> {
    let (old, new) = (side(old)?, side(new)?);
    let name = |given: Option<String>, source: &jubarte::markdown::Source<'_>, default: &str| {
        given.unwrap_or_else(|| match source {
            jubarte::markdown::Source::Docx(_) => format!("{default}.docx"),
            jubarte::markdown::Source::Markdown(_) => format!("{default}.md"),
        })
    };
    let options = patch_options(
        &name(old_name, &old, "old"),
        &name(new_name, &new, "new"),
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
    let patch = jubarte::markdown::patch_documents(old, new, &redline, &options).map_err(js_err)?;
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

/// Word-validity findings beyond the schema as a JSON array (`code`,
/// `part`, `path`, `message`, `word_fatal`, `repairable`); `[]` is a pass.
///
/// Mirrors `jubarte::validate::validate`.
#[wasm_bindgen(js_name = validateDocument)]
pub fn validate_document(docx: &[u8]) -> Result<String, JsValue> {
    let findings = jubarte::validate::validate(docx).map_err(js_err)?;
    serde_json::to_string(&findings).map_err(js_err)
}

/// Every text change from `original` to `edited` must be a revision by
/// `author`; the findings (`UNTRACKED_EDIT`, `FOREIGN_AUTHOR`) as a JSON
/// array.
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

/// What this build can do, as JSON (`runtime: "wasm"`): PDF and field
/// refresh only in the full build, PNG never.
///
/// Mirrors `jubarte::capabilities::capabilities`.
#[wasm_bindgen]
pub fn capabilities() -> Result<String, JsValue> {
    let mut manifest = jubarte::capabilities::capabilities("wasm");
    manifest.operations.pdf = cfg!(feature = "pdf");
    manifest.operations.png = false;
    manifest.operations.fields = cfg!(feature = "pdf");
    if !cfg!(feature = "pdf") {
        manifest
            .audit_rules
            .retain(|code| code != "FONT_SUBSTITUTED");
    }
    serde_json::to_string_pretty(&manifest).map_err(js_err)
}

/// `markdownToDocx`'s options (JSON, every field optional).
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MarkdownDocxOptions {
    #[serde(default)]
    page: jubarte::markdown::PageSize,
    author: Option<String>,
    date: Option<String>,
    critic: Option<bool>,
    #[serde(alias = "trackChanges")]
    track_changes: Option<String>,
}

/// Markdown with CriticMarkup → DOCX bytes; errors as text so native tests
/// can reach them (a `JsValue` exists only on wasm).
fn markdown_docx(
    text: &str,
    options_json: Option<String>,
    reference: Option<Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let options: MarkdownDocxOptions =
        serde_json::from_str(options_json.as_deref().unwrap_or("{}")).map_err(|e| e.to_string())?;
    let track_changes = match options.track_changes.as_deref() {
        None => jubarte::markdown::TrackChanges::All,
        Some(value) => jubarte::markdown::TrackChanges::parse(value)
            .ok_or_else(|| format!("track_changes must be all, accept or reject, not {value:?}"))?,
    };
    let mut docx_options = jubarte::markdown::DocxOptions {
        reference: reference.as_deref(),
        track_changes,
        page: options.page,
        ..jubarte::markdown::DocxOptions::default()
    };
    if let Some(author) = options.author {
        docx_options.author = author;
    }
    if let Some(date) = options.date {
        docx_options.date = date;
    }
    if let Some(critic) = options.critic {
        docx_options.critic = critic;
    }
    jubarte::markdown::markdown_to_docx(text, &docx_options)
        .map(|written| written.docx)
        .map_err(|e| e.to_string())
}

/// Markdown with CriticMarkup → DOCX bytes, as `jubarte convert draft.md`.
///
/// `optionsJson` (every field optional): `page` (`"letter"` default, or
/// `"a4"`), `author` (`"Redline"`), `date` (fixed epoch, so the same Markdown
/// writes the same bytes), `critic` (`true`: CriticMarkup becomes tracked
/// changes and comments) and `track_changes` (or `trackChanges`: `"all"`,
/// `"accept"`, `"reject"`). An unknown field is an error. `reference`, a
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

/// Audit findings as JSON `{findings, rules, layout}` (see `jubarte audit`).
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
        assert!(
            document_view(old, new, None)
                .unwrap()
                .contains("diff --git")
        );
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
        assert!(marked.contains("{--30--}{++45++}") && marked.contains("{--60--}{++45++}"));
        for options in [r#"{"acceptChanges":true}"#, r#"{"format":"word"}"#] {
            assert_eq!(document_view(&old, &new, Some(options)).unwrap(), "");
        }
        for options in [r#"{"oldFormat":"docx"}"#, r#"{"newFormat":"docx"}"#] {
            assert!(
                document_view(b"plain", b"text", Some(options))
                    .unwrap_err()
                    .contains("DOCX")
            );
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
            r#"{"schema_version":1,"author":"Ann","operations":[{"kind":"comment","paragraph":"body:p:0","text":"keep"}]}"#,
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
        assert!(
            jubarte::comments::list_comments(&dropped.docx())
                .unwrap()
                .is_empty()
        );
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn docx_to_pdf_moves_comments_and_keeps_changed_pages() {
        let plan = jubarte::edit::EditPlan::from_json(
            r#"{"schema_version":1,"author":"Ann","operations":[{"kind":"comment","paragraph":"body:p:0","text":"Too low"}]}"#,
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
        assert_eq!(manifest["operations"]["fields"], cfg!(feature = "pdf"));
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
        assert!(
            get_revisions(&redline, None)
                .unwrap()
                .contains("\"Inserted\"")
        );
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
        assert_eq!(bullet["location"], "body:p:0");
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
        assert!(
            text.contains("@@ [body:p:1] @@\nPayment is due in [-30-]{+45+} days.\n"),
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
        let out = apply_edit_plan(&source, plan).unwrap();
        assert!(out.ok());
        assert_eq!(
            out.patch().unwrap(),
            "--- a/document.docx\n+++ b/document.docx\tClaude\t2026-09-25T12:00:00Z\n\
             @@ [body:p:1] @@\nPayment is due in [-30-]{+45+} days.\n"
        );
        assert!(preview_edit_plan(&source, plan).unwrap().patch().is_none());
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
}
