//! Jubarte desktop — drop two Word documents, get a tracked-changes redline.
//!
//! Thin Tauri shell over the `jubarte` engine: the heavy lifting
//! (`compare_documents`, `get_revisions`) happens on a blocking thread; the
//! webview gets a serialized outcome plus a lightweight ins/del preview model
//! parsed straight out of the produced `word/document.xml`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use jubarte::comparer::{WmlComparerRevisionType, WmlComparerSettings};
use jubarte::document_comparer;
use jubarte::opc::PartFs;
use serde::Serialize;
use tauri::{Emitter, Manager};

/// Preview stops growing past these bounds so the IPC payload stays light on
/// book-length documents; the full fidelity lives in the written `.docx`.
const PREVIEW_MAX_PARAGRAPHS: usize = 3000;
const PREVIEW_MAX_CHARS: usize = 300_000;

/// Files handed over by Finder/Explorer ("Open with… → Jubarte") before the
/// webview was ready to receive events; the frontend drains this on startup.
struct PendingFiles(Mutex<Vec<String>>);

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
            Some(FileInfo { path: p, name, size: meta.len(), modified_ms })
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
                    b"creator" => 1,
                    b"lastModifiedBy" => 2,
                    _ => cur,
                };
            }
            Ok(Event::End(e)) => {
                if matches!(e.local_name().as_ref(), b"creator" | b"lastModifiedBy") {
                    cur = 0;
                }
            }
            Ok(Event::Text(t)) if cur != 0 => {
                let text = t
                    .xml10_content()
                    .map(|c| c.into_owned())
                    .unwrap_or_else(|_| String::from_utf8_lossy(t.as_ref()).into_owned());
                if cur == 1 { creator.push_str(&text) } else { last_mod.push_str(&text) }
            }
            // 0.40 surfaces entities as their own event between text chunks.
            Ok(Event::GeneralRef(r)) if cur != 0 => {
                let name: &[u8] = r.as_ref();
                let resolved = match r.resolve_char_ref() {
                    Ok(Some(c)) => Some(c),
                    _ => match name {
                        b"amp" => Some('&'),
                        b"lt" => Some('<'),
                        b"gt" => Some('>'),
                        b"quot" => Some('"'),
                        b"apos" => Some('\''),
                        _ => None,
                    },
                };
                if let Some(c) = resolved {
                    if cur == 1 { creator.push(c) } else { last_mod.push(c) }
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
fn take_pending_files(state: tauri::State<'_, PendingFiles>) -> Vec<String> {
    std::mem::take(&mut *state.0.lock().unwrap())
}

#[tauri::command]
async fn create_redline(
    original: String,
    modified: String,
    author: String,
    filename: Option<String>,
) -> Result<RedlineOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_compare(&original, &modified, &author, filename.as_deref())
    })
    .await
    .map_err(|_| "The comparison crashed — this document pair may hit an engine bug.".to_string())?
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
    std::fs::copy(&src, &dest).map(|_| ()).map_err(|e| e.to_string())
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
    cmd.status()
        .map_err(|e| e.to_string())
        .and_then(|s| if s.success() { Ok(()) } else { Err("could not open".into()) })
}

fn run_compare(
    original: &str,
    modified: &str,
    author: &str,
    filename: Option<&str>,
) -> Result<RedlineOutcome, String> {
    let t0 = std::time::Instant::now();
    let orig = std::fs::read(original)
        .map_err(|e| format!("Could not read the original ({e})"))?;
    let modif = std::fs::read(modified)
        .map_err(|e| format!("Could not read the modified document ({e})"))?;
    let author = if author.trim().is_empty() { "Jubarte" } else { author.trim() };

    let redline = document_comparer::compare_documents(&orig, &modif, author)
        .map_err(|e| format!("Comparison failed: {e}"))?;

    // Honour an explicit output name from the "File name" field; otherwise fall
    // back to the CLI's `<orig>_v_<mod>.docx` convention. Both dedupe with ` (n)`.
    let output_path = match filename.map(str::trim).filter(|f| !f.is_empty()) {
        Some(name) => unique_named_output_path(original, name),
        None => unique_output_path(original, modified),
    };
    std::fs::write(&output_path, &redline)
        .map_err(|e| format!("Could not write the redline next to the original ({e})"))?;

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

/// `<original-dir>/<orig-stem>_v_<mod-stem>.docx` (the CLI's convention),
/// suffixed ` (2)`, ` (3)`, … instead of silently overwriting an earlier run.
fn unique_output_path(original: &str, modified: &str) -> String {
    let stem = |p: &str| {
        Path::new(p)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "document".into())
    };
    let dir = Path::new(original)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let base = format!("{}_v_{}", stem(original), stem(modified));
    let mut candidate = dir.join(format!("{base}.docx"));
    let mut n = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{base} ({n}).docx"));
        n += 1;
    }
    candidate.to_string_lossy().into_owned()
}

/// A user-typed output name, written into the original's directory. Only the
/// file-name component is honoured (any typed path segments are dropped, so the
/// output can never escape that directory), the `.docx` extension is enforced,
/// and existing files are preserved via the same ` (n)` dedupe.
fn unique_named_output_path(original: &str, name: &str) -> String {
    let dir = Path::new(original)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let raw = Path::new(name)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stem = raw
        .strip_suffix(".docx")
        .or_else(|| raw.strip_suffix(".DOCX"))
        .unwrap_or(&raw)
        .trim();
    let stem = if stem.is_empty() { "redline" } else { stem };
    let mut candidate = dir.join(format!("{stem}.docx"));
    let mut n = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{stem} ({n}).docx"));
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

    fn attr_author(e: &BytesStart) -> Option<String> {
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
        runs.push(PreviewRun { kind, text: text.to_string(), author });
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
                b"ins" => {
                    ins_d += 1;
                    author_stack.push(attr_author(&e));
                }
                b"del" => {
                    del_d += 1;
                    author_stack.push(attr_author(&e));
                }
                b"moveTo" => {
                    mvto_d += 1;
                    author_stack.push(attr_author(&e));
                }
                b"moveFrom" => {
                    mvfrom_d += 1;
                    author_stack.push(attr_author(&e));
                }
                b"r" => run_d += 1,
                b"t" | b"delText" if run_d > 0 => in_text = true,
                _ => {}
            },
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"ins" => {
                    ins_d -= 1;
                    author_stack.pop();
                }
                b"del" => {
                    del_d -= 1;
                    author_stack.pop();
                }
                b"moveTo" => {
                    mvto_d -= 1;
                    author_stack.pop();
                }
                b"moveFrom" => {
                    mvfrom_d -= 1;
                    author_stack.pop();
                }
                b"r" => run_d -= 1,
                b"t" | b"delText" => in_text = false,
                b"p" => paragraphs.push(PreviewParagraph { runs: std::mem::take(&mut runs) }),
                _ => {}
            },
            Ok(Event::Empty(e)) if run_d > 0 => match e.local_name().as_ref() {
                b"tab" => push_text(&mut runs, kind, author, "\t", &mut total_chars),
                b"br" | b"cr" => push_text(&mut runs, kind, author, "\n", &mut total_chars),
                _ => {}
            },
            Ok(Event::Text(t)) if in_text => {
                let text = t
                    .xml10_content()
                    .map(|c| c.into_owned())
                    .unwrap_or_else(|_| String::from_utf8_lossy(t.as_ref()).into_owned());
                push_text(&mut runs, kind, author, &text, &mut total_chars);
            }
            // 0.40 reports entities as separate events between Text chunks.
            Ok(Event::GeneralRef(r)) if in_text => {
                let name: &[u8] = r.as_ref();
                let resolved = match r.resolve_char_ref() {
                    Ok(Some(c)) => Some(c),
                    _ => match name {
                        b"amp" => Some('&'),
                        b"lt" => Some('<'),
                        b"gt" => Some('>'),
                        b"quot" => Some('"'),
                        b"apos" => Some('\''),
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
    std::env::args()
        .skip(1)
        .filter(|a| a.to_lowercase().ends_with(".docx") && Path::new(a).exists())
        .collect()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(PendingFiles(Mutex::new(initial_docx_args())))
        .invoke_handler(tauri::generate_handler![
            stat_files,
            default_author,
            document_author,
            take_pending_files,
            create_redline,
            open_path,
            reveal_path,
            save_copy,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Jubarte")
        .run(|_app, _event| {
            // Finder "Open with… → Jubarte" (also two files at once): macOS
            // delivers them as Opened events, possibly before the webview
            // exists — stash for the startup drain AND emit for a live window.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = _event {
                let paths: Vec<String> = urls
                    .iter()
                    .filter_map(|u| u.to_file_path().ok())
                    .map(|p| p.to_string_lossy().into_owned())
                    .filter(|p| p.to_lowercase().ends_with(".docx"))
                    .collect();
                if !paths.is_empty() {
                    if let Some(st) = _app.try_state::<PendingFiles>() {
                        st.0.lock().unwrap().extend(paths.iter().cloned());
                    }
                    let _ = _app.emit("files-opened", paths);
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::author_from_core_xml;

    const NS: &str = "xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\"";

    fn core(inner: &str) -> String {
        format!("<?xml version=\"1.0\"?><cp:coreProperties {NS}>{inner}</cp:coreProperties>")
    }

    #[test]
    fn prefers_creator_over_last_modified_by() {
        let xml = core("<dc:creator>Jamie Coppin</dc:creator><cp:lastModifiedBy>Joshua Jenkins</cp:lastModifiedBy>");
        assert_eq!(author_from_core_xml(&xml).as_deref(), Some("Jamie Coppin"));
    }

    #[test]
    fn falls_back_to_last_modified_by_when_creator_empty() {
        // Real-world shape: dc:creator is a system name that was cleared, the
        // human is in lastModifiedBy.
        let xml = core("<dc:creator></dc:creator><cp:lastModifiedBy>Michelle Champagne</cp:lastModifiedBy>");
        assert_eq!(author_from_core_xml(&xml).as_deref(), Some("Michelle Champagne"));
    }

    #[test]
    fn falls_back_when_creator_is_only_whitespace() {
        let xml = core("<dc:creator>   </dc:creator><cp:lastModifiedBy>K. Nguyen</cp:lastModifiedBy>");
        assert_eq!(author_from_core_xml(&xml).as_deref(), Some("K. Nguyen"));
    }

    #[test]
    fn returns_none_when_both_missing_or_empty() {
        assert_eq!(author_from_core_xml(&core("<dc:creator></dc:creator>")), None);
        assert_eq!(author_from_core_xml(&core("")), None);
    }

    #[test]
    fn decodes_entities_in_a_name() {
        let xml = core("<dc:creator>Ben &amp; Jerry &lt;legal&gt;</dc:creator>");
        assert_eq!(author_from_core_xml(&xml).as_deref(), Some("Ben & Jerry <legal>"));
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let xml = core("<dc:creator>  Arthur Rodrigues  </dc:creator>");
        assert_eq!(author_from_core_xml(&xml).as_deref(), Some("Arthur Rodrigues"));
    }

    #[test]
    fn ignores_unrelated_docx_metadata() {
        let xml = core("<dc:title>Master Services Agreement</dc:title><dc:subject>none</dc:subject><cp:keywords>a b c</cp:keywords><dc:creator>Acme Counsel</dc:creator>");
        assert_eq!(author_from_core_xml(&xml).as_deref(), Some("Acme Counsel"));
    }
}
