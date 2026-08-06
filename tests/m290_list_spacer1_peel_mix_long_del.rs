// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M290 — list_spacer1 × list_with_break_exported_broken:
//! Tip free-LCS meshes short pure-I `b` with long base residual
//! `14.9Entire Agreement…` as MIX, then pure-D for remaining base.
//! Word peels the long del into its own pure-D after short MIX/I `b`.
//! Peel long del children from mid MIX (short ins) onto a new pure-D after
//! MIX when followed by pure-D stream.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
    )
    .join(name);
    std::fs::read(p).ok()
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn body_paras(doc: &str) -> Vec<String> {
    doc.split("</w:p>")
        .filter(|c| c.contains("<w:p") || c.contains("<w:p>"))
        .map(|s| format!("{s}</w:p>"))
        .collect()
}

fn shape(p: &str) -> char {
    let has_ins = p.contains("<w:ins ") || p.contains("<w:ins>");
    let has_del = p.contains("<w:del ") || p.contains("<w:del>") || p.contains("<w:delText");
    match (has_ins, has_del) {
        (true, true) => 'M',
        (true, false) => 'I',
        (false, true) => 'D',
        _ => 'E',
    }
}

fn collect_t(p: &str) -> String {
    let mut out = String::new();
    let mut rest = p;
    while let Some(i) = rest.find("<w:t") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else { break };
        let content = &after[gt + 1..];
        let Some(end) = content.find("</w:t>") else {
            break;
        };
        out.push_str(&content[..end]);
        rest = &content[end + 6..];
    }
    out
}

fn collect_del(p: &str) -> String {
    let mut out = String::new();
    let mut rest = p;
    while let Some(i) = rest.find("<w:delText") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else { break };
        let content = &after[gt + 1..];
        let Some(end) = content.find("</w:delText>") else {
            break;
        };
        out.push_str(&content[..end]);
        rest = &content[end + 12..];
    }
    out
}

#[test]
fn list_spacer1_long_del_not_meshed_into_short_b() {
    let Some(a) = load("super_editor__list_spacer1_06383c66.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__list_with_break_exported_broken_45f7bd19.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    let shapes: String = paras.iter().map(|p| shape(p)).collect();

    // Find para with ins "b"
    let b_idx = paras.iter().position(|p| {
        let t = collect_t(p);
        t.trim() == "b" || t.trim().starts_with('b')
    });
    assert!(b_idx.is_some(), "expected ins b; shapes={shapes}");
    let bi = b_idx.unwrap();
    let b_del = collect_del(&paras[bi]);
    // Long agreement residual must NOT sit inside short b MIX.
    assert!(
        !b_del.contains("14.9") && !b_del.contains("Entire Agreement"),
        "long base residual must be pure-D not meshed into short b; b_del={b_del:?} shapes={shapes}"
    );
    // Pure-D stream carries 14.9.
    let all_del: String = paras.iter().map(|p| collect_del(p)).collect();
    assert!(
        all_del.contains("14.9") || all_del.contains("Entire Agreement"),
        "base residual still present as pure-D; all_del={all_del:?} shapes={shapes}"
    );
    // Prefer I/M then D* (no long del inside short b).
    assert!(
        shapes.contains('D'),
        "expected pure-D residual; shapes={shapes}"
    );
}
