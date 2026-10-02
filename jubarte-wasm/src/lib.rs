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

fn js_err(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&format!("jubarte-wasm: {e}"))
}

/// Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).
///
/// Mirrors `jubarte::document_comparer::compare_documents`.
#[wasm_bindgen(js_name = compareDocuments)]
pub fn compare_documents(
    original: &[u8],
    modified: &[u8],
    author: &str,
) -> Result<Vec<u8>, JsValue> {
    jubarte::document_comparer::compare_documents(original, modified, author).map_err(js_err)
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
/// serialized by the shared `revisions_to_json`.
#[wasm_bindgen(js_name = getRevisions)]
pub fn get_revisions(docx: &[u8]) -> Result<String, JsValue> {
    let settings = jubarte::comparer::WmlComparerSettings::default();
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
#[cfg(feature = "pdf")]
#[wasm_bindgen(js_name = docxToPdf)]
pub fn docx_to_pdf(
    docx: &[u8],
    compress: Option<bool>,
    revisions: Option<String>,
    revision_palette: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    let revisions = jubarte::convert::RevisionStyle::from_choice(
        revisions.as_deref().unwrap_or("conventional"),
        revision_palette.as_deref(),
    )
    .map_err(|e| JsValue::from_str(&e))?;
    let options = jubarte::convert::PdfOptions {
        compress: compress.unwrap_or(false),
        revisions,
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
    std::str::from_utf8(bytes)
        .map(jubarte::markdown::Source::Markdown)
        .map_err(|e| js_err(format!("a side is neither a .docx nor UTF-8 Markdown: {e}")))
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
    let patch = jubarte::markdown::patch_documents(
        old,
        new,
        &jubarte::markdown::RedlineOptions::default(),
        &options,
    )
    .map_err(js_err)?;
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
/// "none", "keep_sections": bool}`, each optional.
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

/// What this build can do, as JSON (`runtime: "wasm"`): PDF only in the full
/// build, PNG never.
///
/// Mirrors `jubarte::capabilities::capabilities`.
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

/// What this build can do, as JSON (`runtime: "wasm"`): PDF only in the full
/// build, PNG never.
///
/// Mirrors `jubarte::capabilities::capabilities`.
#[wasm_bindgen]
pub fn capabilities() -> Result<String, JsValue> {
    let mut manifest = jubarte::capabilities::capabilities("wasm");
    manifest.operations.pdf = cfg!(feature = "pdf");
    manifest.operations.png = false;
    serde_json::to_string_pretty(&manifest).map_err(js_err)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn capabilities_report_the_wasm_runtime_without_png() {
        let manifest: serde_json::Value = serde_json::from_str(&capabilities().unwrap()).unwrap();
        assert_eq!(manifest["runtime"], "wasm");
        assert_eq!(manifest["operations"]["png"], false);
        assert_eq!(manifest["operations"]["pdf"], cfg!(feature = "pdf"));
        assert_eq!(manifest["operations"]["edit"], true);
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
}
