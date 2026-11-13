// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The module layering the workspace crate split depends on.
//!
//! `src/` is being split into the `crates/jubarte-*` crates (dom, opc,
//! redlines, render, doc, markdown, edit, cli). A crate cannot name a crate
//! above it, so every `crate::…` path that points from a lower layer into a
//! higher one (or between the compare and render products) blocks the split.
//!
//! The test maps each source file to its target crate, collects every
//! `crate::` path (code, `use` groups and intra-doc links alike), and
//! compares the upward ones with [`KNOWN_UPWARD`]. A new upward path fails;
//! so does a listed one that is gone, so the list only ever shrinks.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Target crates, lowest first. A file may name its own layer or a lower one.
const LAYERS: [&str; 8] = [
    "dom", "opc", "redlines", "render", "doc", "markdown", "edit", "cli",
];

/// Top-level `crate::` modules (and a few submodules) and their target crate.
/// Submodule keys (`a::b`) override the top-level module's layer.
const MODULES: &[(&str, &str)] = &[
    ("xmllinq", "dom"),
    ("namespaces", "dom"),
    ("ids", "dom"),
    ("unid", "dom"),
    ("perf", "dom"),
    ("util", "dom"),
    ("ooxml", "dom"),
    ("limits", "dom"),
    ("opc", "opc"),
    ("admission", "opc"),
    ("markup_simplifier", "opc"),
    ("strict_translation", "opc"),
    ("settings", "opc"),
    ("word_default_theme", "opc"),
    ("builtin_styles", "opc"),
    ("borders", "opc"),
    ("data_stores", "opc"),
    ("validate_dom", "opc"),
    ("render_options", "opc"),
    ("comparer", "redlines"),
    ("document_comparer", "redlines"),
    ("revision_processor", "redlines"),
    ("wml_document", "redlines"),
    ("WmlDocument", "redlines"),
    ("comparison_log", "redlines"),
    ("convert", "render"),
    ("changes", "doc"),
    ("comments", "doc"),
    ("comment_settlement", "doc"),
    ("validate", "doc"),
    ("audit", "doc"),
    ("scrub", "doc"),
    ("append", "doc"),
    ("fields", "doc"),
    ("debug", "doc"),
    ("inspect", "doc"),
    ("markdown", "markdown"),
    ("legacy_doc", "markdown"),
    ("markdown::patch", "edit"),
    ("markdown::redline", "edit"),
    ("markdown::unified", "edit"),
    ("markdown::borders", "opc"),
    ("markdown::data_stores", "opc"),
    ("edit", "edit"),
    ("edit::comment_settlement", "doc"),
    ("edit::inline", "cli"),
    ("text_diff", "edit"),
    ("expect", "edit"),
    ("capabilities", "edit"),
    ("publication", "edit"),
    ("result", "edit"),
    ("cli", "cli"),
    ("update", "cli"),
];

/// Upward paths still to remove, as `file -> crate::path` (first two path
/// segments when the first names a module with submodule overrides).
const KNOWN_UPWARD: &[&str] = &[
    "src/append.rs -> markdown::markdown_to_docx",
    "src/changes.rs -> edit::KeptComment",
    "src/comparer/finalize.rs -> comments",
    "src/comparer/finalize.rs -> validate",
    "src/convert/mod.rs -> fields",
    "src/debug.rs -> text_diff",
    "src/document_comparer.rs -> changes",
    "src/inspect.rs -> edit",
    "src/inspect.rs -> edit::apply_plan_json",
    "src/markdown/mod.rs -> markdown::redline",
    "src/markdown/write.rs -> markdown::redline",
    "src/revision_processor.rs -> changes",
    "src/settings.rs -> markdown::markdown_to_docx",
];

fn rank(layer: &str) -> usize {
    LAYERS
        .iter()
        .position(|known| *known == layer)
        .unwrap_or_else(|| panic!("unknown layer {layer}"))
}

/// The layer of a `crate::` path given as its first one or two segments.
fn layer_of(segments: &[&str]) -> Option<&'static str> {
    let first = *segments.first()?;
    if let Some(second) = segments.get(1) {
        let key = format!("{first}::{second}");
        if let Some((_, layer)) = MODULES.iter().find(|(module, _)| *module == key) {
            return Some(layer);
        }
    }
    MODULES
        .iter()
        .find(|(module, _)| *module == first)
        .map(|(_, layer)| *layer)
}

/// The layer a source file belongs to; `None` for the facade and binary.
fn file_layer(relative: &Path) -> Option<&'static str> {
    let mut segments: Vec<String> = relative
        .with_extension("")
        .components()
        .skip(1)
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    if matches!(segments.first().map(String::as_str), Some("lib" | "bin")) {
        return None;
    }
    if segments.last().map(String::as_str) == Some("mod") {
        segments.pop();
    }
    let segments: Vec<&str> = segments.iter().map(String::as_str).collect();
    Some(layer_of(&segments).unwrap_or_else(|| panic!("no layer for {}", relative.display())))
}

fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Up to two leading `ident` segments of `text` (`a::b::c` gives `a`, `b`).
fn leading_segments(text: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut rest = text;
    while segments.len() < 2 {
        let end = rest.bytes().take_while(|byte| is_ident(*byte)).count();
        if end == 0 {
            break;
        }
        segments.push(&rest[..end]);
        match rest[end..].strip_prefix("::") {
            Some(next) => rest = next,
            None => break,
        }
    }
    segments
}

/// Every path named after `crate::`, expanding `crate::{a, b::c}` groups.
fn crate_paths(text: &str) -> Vec<Vec<&str>> {
    let mut paths = Vec::new();
    let mut from = 0;
    while let Some(found) = text[from..].find("crate::") {
        let start = from + found;
        from = start + "crate::".len();
        if start > 0 && is_ident(text.as_bytes()[start - 1]) {
            continue;
        }
        let rest = &text[from..];
        if let Some(group) = rest.strip_prefix('{') {
            let mut depth = 1usize;
            let mut item_start = 0;
            for (index, byte) in group.bytes().enumerate() {
                match byte {
                    b'{' => depth += 1,
                    b'}' | b',' if depth == 1 => {
                        let item = group[item_start..index].trim();
                        let segments = leading_segments(item);
                        if !segments.is_empty() {
                            paths.push(segments);
                        }
                        item_start = index + 1;
                        if byte == b'}' {
                            break;
                        }
                    }
                    b'}' => depth -= 1,
                    _ => {}
                }
            }
        } else {
            let segments = leading_segments(rest);
            if !segments.is_empty() {
                paths.push(segments);
            }
        }
    }
    paths
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Whether a file in `from` may name a path in `to`.
fn allowed(from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }
    // The compare and render products are peers: neither names the other.
    if matches!((from, to), ("redlines", "render") | ("render", "redlines")) {
        return false;
    }
    rank(to) < rank(from)
}

fn upward_edges(root: &Path) -> BTreeSet<String> {
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    let mut edges = BTreeSet::new();
    for path in files {
        let relative = path.strip_prefix(root).expect("under root");
        let Some(from) = file_layer(relative) else {
            continue;
        };
        let text = std::fs::read_to_string(&path).expect("read source");
        for segments in crate_paths(&text) {
            let Some(to) = layer_of(&segments) else {
                continue;
            };
            if allowed(from, to) {
                continue;
            }
            let shown = if segments.len() > 1
                && MODULES
                    .iter()
                    .any(|(module, _)| module.starts_with(&format!("{}::", segments[0])))
            {
                segments.join("::")
            } else {
                segments[0].to_owned()
            };
            let unix = relative.to_string_lossy().replace('\\', "/");
            edges.insert(format!("{unix} -> {shown}"));
        }
    }
    edges
}

#[test]
fn crate_paths_expand_groups_and_skip_longer_idents() {
    let text = "use crate::{a::b::c, d, e::{f, g}};\nlet x = mycrate::z; crate::h::i();";
    let paths = crate_paths(text);
    assert_eq!(
        paths,
        vec![vec!["a", "b"], vec!["d"], vec!["e"], vec!["h", "i"],]
    );
}

#[test]
fn layers_only_point_down() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let found = upward_edges(root);
    let known: BTreeSet<String> = KNOWN_UPWARD.iter().map(|edge| (*edge).to_owned()).collect();
    let new: Vec<_> = found.difference(&known).collect();
    let gone: Vec<_> = known.difference(&found).collect();
    assert!(
        new.is_empty() && gone.is_empty(),
        "upward crate paths changed.\nnew (move the item down instead): {new:#?}\n\
         gone (drop them from KNOWN_UPWARD): {gone:#?}"
    );
}
