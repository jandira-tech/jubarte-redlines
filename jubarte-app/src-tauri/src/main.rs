//! Jubarte desktop — drop two Word documents, get a tracked-changes redline;
//! drop one, get a PDF.
//!
//! Thin Tauri shell over the `jubarte` engine: the heavy lifting
//! (`compare_documents`, `get_revisions`, `docx_to_pdf_with`) happens on a
//! blocking thread; the webview gets a serialized outcome plus, for a redline,
//! a lightweight ins/del preview model parsed straight out of the produced
//! `word/document.xml`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod facts;
mod finder;
#[cfg(target_os = "macos")]
mod finder_service;
mod fingerprint;
mod menu;
mod quota;
mod storekit;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use jubarte::admission::{AdmissionError, AdmissionErrorKind};
use jubarte::comparer::{WmlComparerRevisionType, WmlComparerSettings};
use jubarte::convert::{self, PdfOptions, RevisionStyle};
use jubarte::document_comparer;
use jubarte::opc::PartFs;
use serde::Serialize;
use tauri::Manager;

use finder::{Pending, PendingFiles};

/// Preview stops growing past these bounds so the IPC payload stays light on
/// book-length documents; the full fidelity lives in the written `.docx`.
const PREVIEW_MAX_PARAGRAPHS: usize = 3000;
const PREVIEW_MAX_CHARS: usize = 300_000;

#[derive(Serialize, Clone)]
struct FileInfo {
    path: String,
    name: String,
    size: u64,
    /// Unix mtime in ms — the frontend orders a two-file drop by it
    /// (older file is presumed the original).
    modified_ms: u64,
}

#[derive(Serialize)]
struct PreviewRun {
    /// "same" | "ins" | "del" | "moveins" | "movedel"
    kind: &'static str,
    text: String,
    author: Option<String>,
}

#[derive(Serialize)]
struct PreviewParagraph {
    runs: Vec<PreviewRun>,
}

#[derive(Serialize)]
struct RedlineOutcome {
    output_path: String,
    output_name: String,
    insertions: usize,
    deletions: usize,
    moves: usize,
    format_changes: usize,
    paragraphs: Vec<PreviewParagraph>,
    truncated: bool,
    elapsed_ms: u128,
}

#[derive(Serialize, Debug)]
struct ConvertOutcome {
    output_path: String,
    output_name: String,
    pages: usize,
    bytes: u64,
    elapsed_ms: u128,
}

#[tauri::command]
fn stat_files(paths: Vec<String>) -> Vec<FileInfo> {
    paths
        .into_iter()
        .filter(|p| p.to_lowercase().ends_with(".docx"))
        .filter_map(|p| {
            let meta = std::fs::metadata(&p).ok()?;
            let name = Path::new(&p).file_name()?.to_string_lossy().into_owned();
            let modified_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            Some(FileInfo {
                path: p,
                name,
                size: meta.len(),
                modified_ms,
            })
        })
        .collect()
}

#[tauri::command]
fn default_author() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "Jubarte".into())
}

/// The modified document's author, for pre-filling "Revisions by" — so the
/// tracked changes are attributed to whoever produced the modified version
/// rather than to whoever happens to be running this machine. Returns `""`
/// (the frontend then falls back to [`default_author`]) when the file has no
/// usable author or can't be read.
#[tauri::command]
fn document_author(path: String) -> String {
    std::fs::read(&path)
        .ok()
        .and_then(|bytes| read_core_author(&bytes))
        .unwrap_or_default()
}

/// Pull `docProps/core.xml` out of a `.docx` and read its author.
fn read_core_author(docx: &[u8]) -> Option<String> {
    let pkg = PartFs::open(docx).ok()?;
    let xml = pkg.part_string("docProps/core.xml")?;
    author_from_core_xml(&xml)
}

/// Prefer `dc:creator` (the literal author), fall back to `cp:lastModifiedBy`
/// (who last edited it). Returns a trimmed, non-empty name or `None`.
///
/// Keyed on element *local* names so it's namespace-prefix agnostic, and
/// resolves the handful of entities that can legally appear in a name.
fn author_from_core_xml(xml: &str) -> Option<String> {
    use quick_xml::events::Event;

    // 0 = elsewhere, 1 = inside dc:creator, 2 = inside cp:lastModifiedBy
    let mut cur = 0u8;
    let mut creator = String::new();
    let mut last_mod = String::new();
    let mut reader = quick_xml::Reader::from_str(xml);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                cur = match e.local_name().as_ref() {
                    "creator" => 1,
                    "lastModifiedBy" => 2,
                    _ => cur,
                };
            }
            Ok(Event::End(e)) => {
                if matches!(e.local_name().as_ref(), "creator" | "lastModifiedBy") {
                    cur = 0;
                }
            }
            Ok(Event::Text(t)) if cur != 0 => {
                let text = t.xml10_content().into_owned();
                if cur == 1 {
                    creator.push_str(&text)
                } else {
                    last_mod.push_str(&text)
                }
            }
            // 0.40 surfaces entities as their own event between text chunks.
            Ok(Event::GeneralRef(r)) if cur != 0 => {
                let name: &str = &r;
                let resolved = match r.resolve_char_ref() {
                    Ok(Some(c)) => Some(c),
                    _ => match name {
                        "amp" => Some('&'),
                        "lt" => Some('<'),
                        "gt" => Some('>'),
                        "quot" => Some('"'),
                        "apos" => Some('\''),
                        _ => None,
                    },
                };
                if let Some(c) = resolved {
                    if cur == 1 {
                        creator.push(c)
                    } else {
                        last_mod.push(c)
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break, // best effort — a partial read is fine
            _ => {}
        }
    }

    let clean = |s: String| {
        let t = s.trim().to_string();
        (!t.is_empty()).then_some(t)
    };
    clean(creator).or_else(|| clean(last_mod))
}

#[tauri::command]
fn take_pending_files(state: tauri::State<'_, PendingFiles>) -> Pending {
    std::mem::take(&mut *state.0.lock().unwrap())
}

/// A run makes a preview: the window shows it, and nothing is spent until
/// the user takes it ([`take_result`]).
#[tauri::command]
async fn create_redline(
    app: tauri::AppHandle,
    original: String,
    modified: String,
    author: String,
    fingerprint: Option<String>,
    filename: Option<String>,
) -> Result<RedlineOutcome, String> {
    let out_dir = output_dir(&app, PREVIEWS).join("redlines");
    in_background(
        "The comparison crashed — this document pair may hit an engine bug.",
        move || {
            run_compare(
                &original,
                &modified,
                &author,
                fingerprint.as_deref(),
                filename.as_deref(),
                &out_dir,
            )
        },
    )
    .await
}

/// A conversion is a preview too, taken the same way as a redline.
#[tauri::command]
async fn convert_document(
    app: tauri::AppHandle,
    input: String,
    revisions: String,
    revision_palette: Option<String>,
    filename: Option<String>,
) -> Result<ConvertOutcome, String> {
    let out_dir = output_dir(&app, PREVIEWS).join("conversions");
    in_background(
        "The conversion crashed — this document may hit an engine bug.",
        move || {
            run_convert(
                &input,
                &revisions,
                revision_palette.as_deref(),
                filename.as_deref(),
                &out_dir,
            )
        },
    )
    .await
}

/// Run `work` on a blocking thread; a panic becomes the `crashed` message.
async fn in_background<T: Send + 'static>(
    crashed: &str,
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|_| crashed.to_string())
        .and_then(|r| r)
}

/// Where runs write: the app's cache, emptied at each launch. Nothing in it
/// has spent a free use, and Show in Finder never opens it.
const PREVIEWS: &str = "previews";

/// The previews the user took this session, and where each one went: Open,
/// Show in Finder and Save a copy all take the same file, and spend once.
#[derive(Default)]
struct Taken(Mutex<HashMap<PathBuf, PathBuf>>);

#[derive(Serialize, Debug, PartialEq)]
struct TakenResult {
    output_path: String,
    output_name: String,
}

/// The kind of output a preview is (`redlines` or `conversions`): a file
/// directly in that folder of `previews`, and nowhere else.
fn preview_kind(previews: &Path, path: &Path) -> Option<&'static str> {
    let kind = ["redlines", "conversions"]
        .into_iter()
        .find(|kind| path.parent() == Some(previews.join(kind).as_path()))?;
    path.is_file().then_some(kind)
}

/// Copies a preview into `dir` under its own name, or the first free
/// `name (n)`. The preview stays where the window shows it from.
fn keep(preview: &Path, dir: &Path) -> Result<PathBuf, String> {
    let stem = preview
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned());
    let ext = preview
        .extension()
        .map(|s| s.to_string_lossy().into_owned());
    let (Some(stem), Some(ext)) = (stem, ext) else {
        return Err("This result has no file name.".into());
    };
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let dest = PathBuf::from(first_free(dir, &stem, &ext));
    std::fs::copy(preview, &dest).map_err(|e| e.to_string())?;
    Ok(dest)
}

/// The user opens, shows in Finder or saves a result: this is a use. The
/// free-tier gate runs here, not when the preview is made: a preview the user
/// never takes is free. A free use is reserved before the copy (under the
/// quota mutex, so two takes cannot both slip through at the last one) and
/// handed back if the copy fails. Entitled users skip the free pool; their
/// takes still bump the counter.
#[tauri::command]
async fn take_result(
    app: tauri::AppHandle,
    taken: tauri::State<'_, Taken>,
    path: String,
) -> Result<TakenResult, String> {
    let preview = PathBuf::from(path);
    let previews = output_dir(&app, PREVIEWS);
    let Some(kind) = preview_kind(&previews, &preview) else {
        return Err("This result is no longer available. Make it again.".into());
    };
    let entitled = storekit::is_entitled_for_gate().await;
    // Held to the end: a second Open while this one copies waits, then finds it taken.
    let mut taken = taken.0.lock().unwrap();
    let dest = match taken.get(&preview) {
        Some(dest) => dest.clone(),
        None => {
            if !entitled && !quota::try_reserve_free_use(&app) {
                return Err(quota::FREE_LIMIT_ERR.to_string());
            }
            match keep(&preview, &output_dir(&app, kind)) {
                Ok(dest) => {
                    if entitled {
                        quota::record_use(&app);
                    }
                    taken.insert(preview, dest.clone());
                    dest
                }
                Err(e) => {
                    if !entitled {
                        quota::release_free_use(&app);
                    }
                    return Err(e);
                }
            }
        }
    };
    Ok(TakenResult {
        output_name: dest
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        output_path: dest.to_string_lossy().into_owned(),
    })
}

/// Sandbox-writable output directory. The user-selected read-write
/// entitlement only covers the picked input files, NOT their parent folder,
/// so a file written next to the original is blocked under the MAS sandbox
/// ("cannot save into disk"). Write into the app's own cache container
/// (always writable); "Save a copy" then powerbox-writes the real destination
/// the user chooses.
fn output_dir(app: &tauri::AppHandle, kind: &str) -> std::path::PathBuf {
    app.path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(kind)
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    open_with(&[&path])
}

#[tauri::command]
fn reveal_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return open_with(&["-R", &path]);
    #[cfg(not(target_os = "macos"))]
    return open_with(&[Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(path)
        .as_str()]);
}

#[tauri::command]
fn save_copy(src: String, dest: String) -> Result<(), String> {
    std::fs::copy(&src, &dest)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn open_with(args: &[&str]) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.args(args);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]).args(args);
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.args(args);
        c
    };
    cmd.status().map_err(|e| e.to_string()).and_then(|s| {
        if s.success() {
            Ok(())
        } else {
            Err("could not open".into())
        }
    })
}

/// A revision timestamp as Word writes `w:date`: UTC, whole seconds, `Z`.
fn revision_date(at: std::time::SystemTime) -> String {
    let secs = at
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    let rem = secs % 86_400;
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Days since 1970-01-01 to a proleptic Gregorian (year, month, day): Howard
/// Hinnant's `civil_from_days`, so the app needs no date crate.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// The message a failed compare shows. An input the engine refused before
/// opening it (a `.doc` or encrypted file, RTF, a package over the limits)
/// carries its reason and the document it names; show that, without the
/// "I/O error: CODE:" wrapping the engine's `OpcError` puts around it.
fn compare_error(e: &jubarte::opc::OpcError) -> String {
    let refusal = match e {
        jubarte::opc::OpcError::Io(io) => io
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<AdmissionError>()),
        _ => None,
    };
    let Some(refusal) = refusal else {
        return format!("Comparison failed: {e}");
    };
    match refusal.message.split_once(" document: ") {
        Some((side, why)) if refusal.kind == AdmissionErrorKind::LegacyDocument => {
            format!("The {side} document is {why}.")
        }
        Some((side, why)) => format!("Can't compare the {side} document: {why}."),
        None => format!("Can't compare: {}.", refusal.message),
    }
}

/// The message for a file `convert` cannot take by its signature. The
/// engine's `ConvertError` carries a refusal only as text, so the document is
/// sniffed first, the way `compare` admits both documents before opening them.
fn convert_refusal(docx: &[u8]) -> Result<(), String> {
    jubarte::admission::sniff(docx).map_err(|refusal| {
        if refusal.kind == AdmissionErrorKind::LegacyDocument {
            format!("The document is {}.", refusal.message)
        } else {
            format!("Can't convert the document: {}.", refusal.message)
        }
    })
}

/// `fingerprint` is Settings' agent fingerprint, written into the redline as
/// a custom document property (`fingerprint::stamp`).
fn run_compare(
    original: &str,
    modified: &str,
    author: &str,
    fingerprint: Option<&str>,
    filename: Option<&str>,
    out_dir: &Path,
) -> Result<RedlineOutcome, String> {
    let t0 = std::time::Instant::now();
    let orig = std::fs::read(original).map_err(|e| format!("Could not read the original ({e})"))?;
    let modif = std::fs::read(modified)
        .map_err(|e| format!("Could not read the modified document ({e})"))?;
    let author = if author.trim().is_empty() {
        "Jubarte"
    } else {
        author.trim()
    };

    // The engine's plain `compare_documents` pins every revision to 1970 for
    // reproducible output; a redline someone reads is dated when it was made.
    let date = revision_date(std::time::SystemTime::now());
    let redline = document_comparer::compare_documents_with_options(&orig, &modif, author, &date)
        .map_err(|e| compare_error(&e))?;
    let redline = match fingerprint {
        Some(text) => fingerprint::stamp(&redline, text)?,
        None => redline,
    };

    // Honour an explicit output name from the "File name" field; otherwise fall
    // back to the CLI's `<orig>_v_<mod>.docx` convention. Both dedupe with ` (n)`.
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("Could not create the output folder ({e})"))?;
    let output_path = match filename.map(str::trim).filter(|f| !f.is_empty()) {
        Some(name) => unique_named_output_path(out_dir, name, "docx", "redline"),
        None => unique_output_path(out_dir, original, modified),
    };
    std::fs::write(&output_path, &redline)
        .map_err(|e| format!("Could not write the redline ({e})"))?;

    // Counting and preview are best-effort decoration on top of the already
    // written file — a panic in either must not turn success into failure.
    let counts = std::panic::catch_unwind(|| {
        document_comparer::get_revisions(&redline, &WmlComparerSettings::default())
    })
    .ok()
    .and_then(Result::ok)
    .unwrap_or_default();
    let (mut insertions, mut deletions, mut moves, mut format_changes) = (0, 0, 0, 0);
    for r in &counts {
        match r.revision_type {
            WmlComparerRevisionType::Inserted => insertions += 1,
            WmlComparerRevisionType::Deleted => deletions += 1,
            WmlComparerRevisionType::Moved => moves += 1,
            WmlComparerRevisionType::FormatChanged => format_changes += 1,
        }
    }

    let (paragraphs, truncated) = PartFs::open(&redline)
        .ok()
        .and_then(|pkg| {
            let main = pkg
                .main_document_part()
                .unwrap_or_else(|| "word/document.xml".into());
            pkg.part_string(&main)
        })
        .map(|xml| parse_preview(&xml))
        .unwrap_or((Vec::new(), false));

    Ok(RedlineOutcome {
        output_name: Path::new(&output_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        output_path,
        insertions,
        deletions,
        moves,
        format_changes,
        paragraphs,
        truncated,
        elapsed_ms: t0.elapsed().as_millis(),
    })
}

/// One `.docx` to a PDF in `out_dir`, named after the document (or the
/// typed `filename`) with the same ` (n)` dedupe as a redline. The PDF's
/// streams are deflated: about a seventh of the plain size on text.
/// `palette` is Settings' custom marks (`RevisionPalette::parse`), given
/// with `revisions` "custom" only.
fn run_convert(
    input: &str,
    revisions: &str,
    palette: Option<&str>,
    filename: Option<&str>,
    out_dir: &Path,
) -> Result<ConvertOutcome, String> {
    let t0 = std::time::Instant::now();
    let docx = std::fs::read(input).map_err(|e| format!("Could not read the document ({e})"))?;
    convert_refusal(&docx)?;
    let revisions = RevisionStyle::from_choice(revisions, palette)?;
    let pdf = convert::docx_to_pdf_with(
        &docx,
        PdfOptions {
            compress: true,
            revisions,
        },
    )
    .map_err(|e| format!("Conversion failed: {e}"))?;

    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("Could not create the output folder ({e})"))?;
    let fallback = Path::new(input)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = filename
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .unwrap_or(&fallback);
    let output_path = unique_named_output_path(out_dir, name, "pdf", "document");
    std::fs::write(&output_path, &pdf).map_err(|e| format!("Could not write the PDF ({e})"))?;

    Ok(ConvertOutcome {
        output_name: Path::new(&output_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        output_path,
        pages: convert::pdf_page_count(&pdf),
        bytes: pdf.len() as u64,
        elapsed_ms: t0.elapsed().as_millis(),
    })
}

/// `<out_dir>/<orig-stem>_v_<mod-stem>.docx` (the CLI's naming convention,
/// written into the sandbox-writable output dir — the MAS sandbox forbids
/// creating files next to the user-picked originals),
/// suffixed ` (2)`, ` (3)`, … instead of silently overwriting an earlier run.
fn unique_output_path(out_dir: &Path, original: &str, modified: &str) -> String {
    let stem = |p: &str| {
        Path::new(p)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "document".into())
    };
    let base = format!("{}_v_{}", stem(original), stem(modified));
    first_free(out_dir, &base, "docx")
}

/// A user-typed output name, written into the sandbox-writable output dir.
/// Only the file-name component is honoured (any typed path segments are
/// dropped, so the output can never escape that directory), a typed `.docx`
/// or `.pdf` in any case gives way to `ext`, an empty name becomes
/// `fallback`, and existing files are preserved via the same ` (n)` dedupe.
fn unique_named_output_path(out_dir: &Path, name: &str, ext: &str, fallback: &str) -> String {
    let raw = Path::new(name.trim())
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let lower = raw.to_ascii_lowercase();
    let stem = [".docx", ".pdf"]
        .iter()
        .find(|known| lower.ends_with(*known))
        .map_or(raw.as_str(), |known| &raw[..raw.len() - known.len()])
        .trim();
    first_free(out_dir, if stem.is_empty() { fallback } else { stem }, ext)
}

/// `<dir>/<stem>.<ext>`, or the first free `<stem> (n).<ext>` from n = 2.
fn first_free(dir: &Path, stem: &str, ext: &str) -> String {
    let mut candidate = dir.join(format!("{stem}.{ext}"));
    let mut n = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{stem} ({n}).{ext}"));
        n += 1;
    }
    candidate.to_string_lossy().into_owned()
}

/// Walk the redline's `document.xml` into a flat paragraph/run model the
/// frontend can render: text inside `w:ins`/`w:moveTo` is an insertion,
/// `w:del`/`w:moveFrom` a deletion (jubarte may emit `w:t` under `w:moveFrom`,
/// so classification keys off the wrappers, not the text element name).
fn parse_preview(xml: &str) -> (Vec<PreviewParagraph>, bool) {
    use quick_xml::events::{BytesStart, Event};

    fn attr_author(e: &BytesStart<'_>) -> Option<String> {
        e.try_get_attribute("w:author")
            .ok()
            .flatten()
            .and_then(|a| a.normalized_value(quick_xml::XmlVersion::Implicit1_0).ok())
            .map(|v| v.into_owned())
    }

    let mut reader = quick_xml::Reader::from_str(xml);
    let mut paragraphs: Vec<PreviewParagraph> = Vec::new();
    let mut runs: Vec<PreviewRun> = Vec::new();
    let (mut ins_d, mut del_d, mut mvto_d, mut mvfrom_d, mut run_d) = (0i32, 0, 0, 0, 0);
    let mut author_stack: Vec<Option<String>> = Vec::new();
    let mut in_text = false;
    let mut total_chars = 0usize;
    let mut truncated = false;

    let push_text = |runs: &mut Vec<PreviewRun>,
                     kind: &'static str,
                     author: Option<String>,
                     text: &str,
                     total_chars: &mut usize| {
        *total_chars += text.len();
        if let Some(last) = runs.last_mut()
            && last.kind == kind
            && last.author == author
        {
            last.text.push_str(text);
            return;
        }
        runs.push(PreviewRun {
            kind,
            text: text.to_string(),
            author,
        });
    };

    loop {
        if paragraphs.len() >= PREVIEW_MAX_PARAGRAPHS || total_chars >= PREVIEW_MAX_CHARS {
            truncated = true;
            break;
        }
        let kind: &'static str = if del_d > 0 || mvfrom_d > 0 {
            if mvfrom_d > 0 { "movedel" } else { "del" }
        } else if ins_d > 0 || mvto_d > 0 {
            if mvto_d > 0 { "moveins" } else { "ins" }
        } else {
            "same"
        };
        let author = author_stack.iter().rev().flatten().next().cloned();
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                "ins" => {
                    ins_d += 1;
                    author_stack.push(attr_author(&e));
                }
                "del" => {
                    del_d += 1;
                    author_stack.push(attr_author(&e));
                }
                "moveTo" => {
                    mvto_d += 1;
                    author_stack.push(attr_author(&e));
                }
                "moveFrom" => {
                    mvfrom_d += 1;
                    author_stack.push(attr_author(&e));
                }
                "r" => run_d += 1,
                "t" | "delText" if run_d > 0 => in_text = true,
                _ => {}
            },
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                "ins" => {
                    ins_d -= 1;
                    author_stack.pop();
                }
                "del" => {
                    del_d -= 1;
                    author_stack.pop();
                }
                "moveTo" => {
                    mvto_d -= 1;
                    author_stack.pop();
                }
                "moveFrom" => {
                    mvfrom_d -= 1;
                    author_stack.pop();
                }
                "r" => run_d -= 1,
                "t" | "delText" => in_text = false,
                "p" => paragraphs.push(PreviewParagraph {
                    runs: std::mem::take(&mut runs),
                }),
                _ => {}
            },
            Ok(Event::Empty(e)) => match e.local_name().as_ref() {
                "tab" if run_d > 0 => push_text(&mut runs, kind, author, "\t", &mut total_chars),
                "br" | "cr" if run_d > 0 => {
                    push_text(&mut runs, kind, author, "\n", &mut total_chars)
                }
                _ => {}
            },
            Ok(Event::Text(t)) if in_text => {
                let text = t.xml10_content().into_owned();
                push_text(&mut runs, kind, author, &text, &mut total_chars);
            }
            // 0.40 reports entities as separate events between Text chunks.
            Ok(Event::GeneralRef(r)) if in_text => {
                let name: &str = &r;
                let resolved = match r.resolve_char_ref() {
                    Ok(Some(c)) => Some(c),
                    _ => match name {
                        "amp" => Some('&'),
                        "lt" => Some('<'),
                        "gt" => Some('>'),
                        "quot" => Some('"'),
                        "apos" => Some('\''),
                        _ => None,
                    },
                };
                if let Some(c) = resolved {
                    push_text(&mut runs, kind, author, &c.to_string(), &mut total_chars);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break, // partial preview is fine; the .docx is authoritative
            _ => {}
        }
    }
    (paragraphs, truncated)
}

/// `.docx` paths passed on the command line (Windows/Linux "Open with…").
fn initial_docx_args() -> Vec<String> {
    finder::docx_only(std::env::args().skip(1).filter(|a| Path::new(a).exists()))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .menu(menu::build)
        .on_menu_event(menu::handle)
        .manage(Taken::default())
        .manage(PendingFiles(Mutex::new(Pending {
            paths: initial_docx_args(),
            intent: None,
        })))
        .setup(|app| {
            // Handle renewals / out-of-band transactions for the whole session.
            storekit::start_transaction_listener();
            quota::init(app.handle());
            // Last session's previews were never taken: nothing to keep.
            let _ = std::fs::remove_dir_all(output_dir(app.handle(), PREVIEWS));
            // Finder's right-click "Redline with Jubarte" (NSServices).
            #[cfg(target_os = "macos")]
            finder_service::register(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            stat_files,
            default_author,
            document_author,
            take_pending_files,
            create_redline,
            convert_document,
            take_result,
            open_path,
            reveal_path,
            save_copy,
            facts::about,
            facts::instant_preview_limit,
            facts::legal_document,
            quota::quota_status,
            storekit::storekit_fetch_products,
            storekit::storekit_purchase,
            storekit::storekit_current_entitlement,
            storekit::storekit_restore,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Jubarte")
        .run(|_app, _event| {
            // Finder "Open with… → Jubarte" or a drop on the Dock icon (also two
            // files at once), possibly before the webview exists.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = _event {
                let paths = urls
                    .iter()
                    .filter_map(|u| u.to_file_path().ok())
                    .map(|p| p.to_string_lossy().into_owned());
                finder::deliver(_app, paths.collect(), None);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::{
        PartFs, author_from_core_xml, keep, parse_preview, preview_kind, revision_date,
        run_compare, run_convert, unique_named_output_path,
    };
    use std::path::{Path, PathBuf};
    use std::time::{Duration, UNIX_EPOCH};

    /// A one-paragraph document with a tracked insertion, written to
    /// `<dir>/prose_a.docx`. Built here so the tests need no engine fixture.
    fn prose(dir: &Path) -> String {
        use std::io::Write;
        const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
        let parts = [
            (
                "[Content_Types].xml",
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#.to_owned(),
            ),
            (
                "_rels/.rels",
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.to_owned(),
            ),
            (
                "word/document.xml",
                format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="{W}"><w:body><w:p><w:r><w:t xml:space="preserve">The term is </w:t></w:r><w:ins w:id="1" w:author="A" w:date="2026-10-01T00:00:00Z"><w:r><w:t>twelve</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> months.</w:t></w:r></w:p></w:body></w:document>"#
                ),
            ),
        ];
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, xml) in parts {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(xml.as_bytes()).unwrap();
        }
        let path = dir.join("prose_a.docx");
        std::fs::write(&path, zip.finish().unwrap().into_inner()).unwrap();
        path.to_string_lossy().into_owned()
    }

    /// A fresh, empty directory under the system temp dir.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("jubarte-app-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn name_of(path: &str) -> &str {
        Path::new(path).file_name().unwrap().to_str().unwrap()
    }

    #[test]
    fn only_a_file_directly_in_a_preview_folder_is_a_preview() {
        let previews = scratch("previews-kind");
        for kind in ["redlines", "conversions", "other"] {
            std::fs::create_dir_all(previews.join(kind)).unwrap();
        }
        std::fs::write(previews.join("redlines/a_v_b.docx"), b"x").unwrap();
        std::fs::write(previews.join("conversions/a.pdf"), b"x").unwrap();
        std::fs::write(previews.join("other/a.pdf"), b"x").unwrap();
        std::fs::write(previews.join("loose.pdf"), b"x").unwrap();

        let kind = |p: &str| preview_kind(&previews, &previews.join(p));
        assert_eq!(kind("redlines/a_v_b.docx"), Some("redlines"));
        assert_eq!(kind("conversions/a.pdf"), Some("conversions"));
        // Somewhere else, a folder, a file that is gone, or a way out.
        assert_eq!(kind("other/a.pdf"), None);
        assert_eq!(kind("loose.pdf"), None);
        assert_eq!(kind("redlines"), None);
        assert_eq!(kind("conversions/gone.pdf"), None);
        assert_eq!(kind("redlines/../conversions/a.pdf"), None);
        assert_eq!(preview_kind(&previews, Path::new("/etc/hosts")), None);
    }

    #[test]
    fn keeping_a_preview_copies_it_under_a_free_name_and_leaves_it_in_place() {
        let root = scratch("previews-keep");
        let preview = root.join("previews/conversions/Lease.pdf");
        std::fs::create_dir_all(preview.parent().unwrap()).unwrap();
        std::fs::write(&preview, b"%PDF one").unwrap();
        let kept = root.join("conversions");

        let first = keep(&preview, &kept).unwrap();
        assert_eq!(first, kept.join("Lease.pdf"));
        assert_eq!(std::fs::read(&first).unwrap(), b"%PDF one");
        assert!(preview.is_file(), "the window still shows the preview");

        // Another Lease.pdf kept later does not replace the first.
        std::fs::write(&preview, b"%PDF two").unwrap();
        let second = keep(&preview, &kept).unwrap();
        assert_eq!(second, kept.join("Lease (2).pdf"));
        assert_eq!(std::fs::read(&first).unwrap(), b"%PDF one");
        assert_eq!(std::fs::read(&second).unwrap(), b"%PDF two");

        assert!(keep(&root.join("previews/conversions/gone.pdf"), &kept).is_err());
    }

    const SAVE_AS: &str = "a Word 97-2003 (.doc) or encrypted document; \
                           open it in Word and save it as .docx without a password.";

    /// An OLE compound file: what a `.doc`, or a password-protected `.docx`, is.
    fn legacy(dir: &Path) -> String {
        let path = dir.join("Old.doc");
        std::fs::write(&path, b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1 a .doc body").unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn a_legacy_or_encrypted_document_reads_as_a_save_as_hint() {
        let src = scratch("refused-src");
        let (doc, docx) = (legacy(&src), prose(&src));
        let out = scratch("refused");
        let compare = |a: &str, b: &str| run_compare(a, b, "A", None, None, &out).err().unwrap();
        assert_eq!(
            compare(&doc, &docx),
            format!("The original document is {SAVE_AS}")
        );
        assert_eq!(
            compare(&docx, &doc),
            format!("The modified document is {SAVE_AS}")
        );
        let convert = run_convert(&doc, "conventional", None, None, &out)
            .err()
            .unwrap();
        assert_eq!(convert, format!("The document is {SAVE_AS}"));
        assert_eq!(std::fs::read_dir(&out).unwrap().count(), 0);
    }

    #[test]
    fn another_refusal_names_the_document_without_an_io_prefix() {
        let src = scratch("refused-rtf-src");
        let rtf = src.join("memo.docx");
        std::fs::write(&rtf, b"{\\rtf1\\ansi hello}").unwrap();
        let out = scratch("refused-rtf");
        let rtf = rtf.to_str().unwrap();
        assert_eq!(
            run_compare(rtf, &prose(&src), "A", None, None, &out)
                .err()
                .unwrap(),
            "Can't compare the original document: an RTF file, not a .docx package."
        );
        assert_eq!(
            run_convert(rtf, "conventional", None, None, &out)
                .err()
                .unwrap(),
            "Can't convert the document: an RTF file, not a .docx package."
        );
    }

    #[test]
    fn convert_writes_a_pdf_named_after_the_document() {
        let out = scratch("convert");
        let doc = prose(&scratch("convert-src"));
        let first = run_convert(&doc, "conventional", None, None, &out).unwrap();
        assert_eq!(first.output_name, "prose_a.pdf");
        assert_eq!(first.output_path, out.join("prose_a.pdf").to_string_lossy());
        let pdf = std::fs::read(&first.output_path).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert_eq!(first.bytes, pdf.len() as u64);
        assert_eq!(first.pages, jubarte::convert::pdf_page_count(&pdf));
        assert!(first.pages >= 1);
        // A second run keeps the first file.
        let second = run_convert(&doc, "word", None, None, &out).unwrap();
        assert_eq!(second.output_name, "prose_a (2).pdf");
        assert!(Path::new(&first.output_path).exists());
    }

    #[test]
    fn convert_honours_a_typed_name_but_keeps_the_pdf_extension() {
        let out = scratch("convert-named");
        let doc = prose(&scratch("convert-named-src"));
        let r = run_convert(
            &doc,
            "conventional",
            None,
            Some(" ../Board pack.docx "),
            &out,
        )
        .unwrap();
        assert_eq!(r.output_name, "Board pack.pdf");
        assert_eq!(Path::new(&r.output_path).parent(), Some(out.as_path()));
    }

    #[test]
    fn convert_reports_bad_input_and_unknown_revision_styles() {
        let out = scratch("convert-bad");
        let missing = run_convert("/nonexistent/x.docx", "conventional", None, None, &out);
        assert!(
            missing
                .unwrap_err()
                .starts_with("Could not read the document")
        );
        let doc = prose(&scratch("convert-bad-src"));
        let style = run_convert(&doc, "sepia", None, None, &out);
        assert!(style.unwrap_err().contains("sepia"));
        let junk = out.join("junk.docx");
        std::fs::write(&junk, b"not a zip").unwrap();
        let bad = run_convert(junk.to_str().unwrap(), "conventional", None, None, &out);
        assert!(bad.unwrap_err().starts_with("Conversion failed"));
        assert_eq!(
            std::fs::read_dir(&out).unwrap().count(),
            1,
            "no PDF for a failure"
        );
    }

    #[test]
    fn a_redline_carries_the_agent_fingerprint_only_when_one_is_set() {
        let out = scratch("fingerprint");
        let doc = prose(&scratch("fingerprint-src"));
        let custom = |path: &str| {
            PartFs::open(&std::fs::read(path).unwrap())
                .unwrap()
                .part_string("docProps/custom.xml")
        };
        let stamped =
            run_compare(&doc, &doc, "A", Some(" agent=claude run=7 "), None, &out).unwrap();
        let xml = custom(&stamped.output_path).expect("custom properties");
        assert!(
            xml.contains("name=\"AgentFingerprint\"><vt:lpwstr>agent=claude run=7</vt:lpwstr>"),
            "{xml}"
        );
        for none in [None, Some("   ")] {
            let plain = run_compare(&doc, &doc, "A", none, None, &out).unwrap();
            assert_eq!(custom(&plain.output_path), None);
        }
    }

    #[test]
    fn convert_paints_settings_custom_marks() {
        let out = scratch("convert-custom");
        let doc = prose(&scratch("convert-custom-src"));
        // The spec settings.js paletteSpec writes for its defaults.
        let spec = "inserted=#0000FF:underline,deleted=#FF0000:strike,\
                    moved-from=#008000:double-strike,moved-to=#008000:double-underline";
        let r = run_convert(&doc, "custom", Some(spec), None, &out).unwrap();
        assert!(std::fs::read(&r.output_path).unwrap().starts_with(b"%PDF"));
        let none = run_convert(&doc, "custom", None, None, &out);
        assert!(none.unwrap_err().contains("needs a revision palette"));
        let stray = run_convert(&doc, "word", Some(spec), None, &out);
        assert!(stray.unwrap_err().contains("needs revisions \"custom\""));
        let bad = run_convert(&doc, "custom", Some("inserted=#00F:wavy"), None, &out);
        assert!(bad.unwrap_err().contains("#RRGGBB"));
        assert_eq!(
            std::fs::read_dir(&out).unwrap().count(),
            1,
            "no PDF for a refused palette"
        );
    }

    #[test]
    fn typed_names_take_the_requested_extension_in_any_case() {
        let out = scratch("names");
        let named = |name: &str, ext: &str| {
            name_of(&unique_named_output_path(&out, name, ext, "redline")).to_owned()
        };
        assert_eq!(named("a.DOCX", "docx"), "a.docx");
        assert_eq!(named("a.Docx", "docx"), "a.docx");
        assert_eq!(named("a.docx", "pdf"), "a.pdf");
        assert_eq!(named("a.PDF", "pdf"), "a.pdf");
        assert_eq!(named("v1.2 final", "pdf"), "v1.2 final.pdf");
        assert_eq!(named("   ", "pdf"), "redline.pdf");
        assert_eq!(named("sub/dir/x", "docx"), "x.docx");
        std::fs::write(out.join("a.pdf"), b"").unwrap();
        assert_eq!(named("a", "pdf"), "a (2).pdf");
    }

    fn date_at(secs: u64) -> String {
        revision_date(UNIX_EPOCH + Duration::from_secs(secs))
    }

    #[test]
    fn revision_date_is_utc_iso_8601_like_word() {
        assert_eq!(date_at(0), "1970-01-01T00:00:00Z");
        assert_eq!(date_at(1_790_844_321), "2026-10-01T08:45:21Z");
        assert_eq!(date_at(4_102_444_800), "2100-01-01T00:00:00Z");
    }

    #[test]
    fn revision_date_handles_leap_days() {
        assert_eq!(date_at(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(date_at(1_835_395_199), "2028-02-28T23:59:59Z");
        assert_eq!(date_at(1_835_395_200), "2028-02-29T00:00:00Z");
    }

    const NS: &str = "xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\"";

    fn core(inner: &str) -> String {
        format!("<?xml version=\"1.0\"?><cp:coreProperties {NS}>{inner}</cp:coreProperties>")
    }

    #[test]
    fn prefers_creator_over_last_modified_by() {
        let xml = core(
            "<dc:creator>Jamie Coppin</dc:creator><cp:lastModifiedBy>Joshua Jenkins</cp:lastModifiedBy>",
        );
        assert_eq!(author_from_core_xml(&xml).as_deref(), Some("Jamie Coppin"));
    }

    #[test]
    fn falls_back_to_last_modified_by_when_creator_empty() {
        // Real-world shape: dc:creator is a system name that was cleared, the
        // human is in lastModifiedBy.
        let xml = core(
            "<dc:creator></dc:creator><cp:lastModifiedBy>Michelle Champagne</cp:lastModifiedBy>",
        );
        assert_eq!(
            author_from_core_xml(&xml).as_deref(),
            Some("Michelle Champagne")
        );
    }

    #[test]
    fn falls_back_when_creator_is_only_whitespace() {
        let xml =
            core("<dc:creator>   </dc:creator><cp:lastModifiedBy>K. Nguyen</cp:lastModifiedBy>");
        assert_eq!(author_from_core_xml(&xml).as_deref(), Some("K. Nguyen"));
    }

    #[test]
    fn returns_none_when_both_missing_or_empty() {
        assert_eq!(
            author_from_core_xml(&core("<dc:creator></dc:creator>")),
            None
        );
        assert_eq!(author_from_core_xml(&core("")), None);
    }

    #[test]
    fn decodes_entities_in_a_name() {
        let xml = core("<dc:creator>Ben &amp; Jerry &lt;legal&gt;</dc:creator>");
        assert_eq!(
            author_from_core_xml(&xml).as_deref(),
            Some("Ben & Jerry <legal>")
        );
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let xml = core("<dc:creator>  Arthur Rodrigues  </dc:creator>");
        assert_eq!(
            author_from_core_xml(&xml).as_deref(),
            Some("Arthur Rodrigues")
        );
    }

    #[test]
    fn ignores_unrelated_docx_metadata() {
        let xml = core(
            "<dc:title>Master Services Agreement</dc:title><dc:subject>none</dc:subject><cp:keywords>a b c</cp:keywords><dc:creator>Acme Counsel</dc:creator>",
        );
        assert_eq!(author_from_core_xml(&xml).as_deref(), Some("Acme Counsel"));
    }

    #[test]
    fn preview_classifies_revisions_and_decodes_text() {
        let xml = concat!(
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#,
            r#"<w:p><w:r><w:t>Terms &amp; </w:t></w:r><w:r><w:tab/></w:r>"#,
            r#"<w:ins w:author="A &amp; B"><w:r><w:t>new</w:t></w:r></w:ins>"#,
            r#"<w:del w:author="Cy"><w:r><w:delText>old</w:delText></w:r></w:del></w:p>"#,
            r#"</w:body></w:document>"#,
        );
        let (paragraphs, truncated) = parse_preview(xml);
        assert!(!truncated);
        assert_eq!(paragraphs.len(), 1);
        let runs: Vec<(&str, &str, Option<&str>)> = paragraphs[0]
            .runs
            .iter()
            .map(|r| (r.kind, r.text.as_str(), r.author.as_deref()))
            .collect();
        assert_eq!(
            runs,
            [
                ("same", "Terms & \t", None),
                ("ins", "new", Some("A & B")),
                ("del", "old", Some("Cy")),
            ]
        );
    }

    /// A body of paragraphs, and the three shapes a move takes in jubarte's
    /// output: the range markers sit inside the paragraph, around the wrapper.
    fn body(paras: &[String]) -> String {
        format!(
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{}</w:body></w:document>"#,
            paras.concat()
        )
    }
    fn plain(t: &str) -> String {
        format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>")
    }
    fn moved(side: &str, name: &str, t: &str) -> String {
        format!(
            r#"<w:p><w:move{side}RangeStart w:id="1" w:name="{name}"/><w:move{side} w:id="2"><w:r><w:t>{t}</w:t></w:r></w:move{side}><w:move{side}RangeEnd w:id="1"/></w:p>"#
        )
    }
    #[test]
    fn preview_shows_a_move_where_it_left_and_where_it_landed_unlabelled() {
        let xml = body(&[
            plain("2. Fees and Payment."),
            moved("From", "move1", "Late payments accrue interest."),
            plain("3. Confidentiality."),
            moved("To", "move1", "Late payments accrue interest."),
        ]);
        let (paragraphs, _) = parse_preview(&xml);
        assert_eq!(paragraphs[1].runs[0].kind, "movedel");
        assert_eq!(paragraphs[3].runs[0].kind, "moveins");
        let json = serde_json::to_string(&paragraphs).expect("preview serializes");
        assert!(!json.contains("movedFrom"), "{json}");
    }

    /// The whole app bar moves the window: Tauri's `deep` drag region takes a
    /// press on any descendant (the title, the version), and a button without
    /// its own drag attribute (Settings) still takes the click. A bare
    /// attribute dragged only from the bar's bare background.
    #[test]
    fn the_app_bar_drags_everywhere_but_its_buttons() {
        let html = include_str!("../../src/index.html");
        let start = html.find(r#"<div class="appbar""#).expect("app bar");
        let end = start + html[start..].find("\n  </div>").expect("app bar end");
        let bar = &html[start..end];
        assert!(
            bar.starts_with(r#"<div class="appbar" data-tauri-drag-region="deep">"#),
            "{bar}"
        );
        assert!(bar.contains(r#"<button type="button" class="appbar-btn" id="open-settings""#));
        assert_eq!(bar.matches("data-tauri-drag-region").count(), 1, "{bar}");
    }
}
