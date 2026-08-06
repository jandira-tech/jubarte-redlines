// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M292 — auto_page_break × list_enter_track_changes:
//! Tip free-LCS meshes pure-I heading `Heading. Body copy for repro` with long
//! base residual as MIX, then pure-D stream. Word peels to pure-I heading +
//! pure-D residual. M290 only peels ins ≤2 chars (list_spacer1 `b`). This peel
//! covers mid-length headings without numPr before pure-D.

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
fn auto_page_x_list_enter_heading_not_meshed_with_notice_del() {
    let Some(a) = load("super_editor__sd_1495_auto_page_break_854a2dd9.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__sd_1707_list_enter_track_changes_with_fd93fd8b.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    let shapes: String = paras.iter().map(|p| shape(p)).collect();

    let heading = paras.iter().find(|p| collect_t(p).contains("Heading"));
    assert!(heading.is_some(), "expected Heading para; shapes={shapes}");
    let h = heading.unwrap();
    assert_eq!(
        shape(h),
        'I',
        "Heading must be pure-I (not MIX with notice del); shapes={shapes} del={:?}",
        collect_del(h)
    );
    assert!(
        !collect_del(h).contains("Generally") && !collect_del(h).contains("notice"),
        "long notice residual must not mesh into Heading; del={:?}",
        collect_del(h)
    );
    let notice_d = paras.iter().any(|p| {
        shape(p) == 'D'
            && (collect_del(p).contains("Generally") || collect_del(p).contains("notice"))
    });
    assert!(notice_d, "notice residual as pure-D; shapes={shapes}");
}

#[test]
fn list_spacer1_short_b_still_peeled() {
    // M290 regression.
    let Some(a) = load("super_editor__list_spacer1_06383c66.docx") else {
        return;
    };
    let Some(b) = load("super_editor__list_with_break_exported_broken_45f7bd19.docx") else {
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    let b_para = paras.iter().find(|p| {
        let t = collect_t(p);
        t.trim() == "b" || t.trim().starts_with('b')
    });
    if let Some(bp) = b_para {
        assert!(
            !collect_del(bp).contains("14.9"),
            "M290 still peels 14.9 from b"
        );
    }
}
