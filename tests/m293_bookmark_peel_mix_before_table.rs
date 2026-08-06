// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M293 — bookmark_use_cases × broken_complex_list:
//! Tip free-LCS meshes short pure-I `a` with long bookmark residual as MIX
//! before pure-D stream and a later pure-D table. Word peels to pure-I `a` +
//! pure-D residual. M290 aborted on any body table; allow tables only *after*
//! the MIX|pure-D junction (same short-ins gates as M290).

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
fn bookmark_x_broken_short_a_not_meshed_with_bookmark_del() {
    let Some(a) = load("super_editor__bookmark_use_cases_d20f31f6.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__broken_complex_list_293fda86.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    let shapes: String = paras.iter().map(|p| shape(p)).collect();

    // Find para with ins text exactly/near "a" that previously held bookmark del.
    let a_idx = paras.iter().position(|p| {
        let t = collect_t(p).trim().to_string();
        t == "a" || t == "a "
    });
    assert!(a_idx.is_some(), "expected pure-I/MIX a; shapes={shapes}");
    let ai = a_idx.unwrap();
    // Prefer the last short "a" before bookmark residual (list ends with a).
    let a_candidates: Vec<usize> = paras
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            let t = collect_t(p).trim().to_string();
            t == "a" || t == "a "
        })
        .map(|(i, _)| i)
        .collect();
    let ai = *a_candidates.last().unwrap_or(&ai);
    let ap = &paras[ai];
    assert_eq!(
        shape(ap),
        'I',
        "list residual a must be pure-I not MIX; shapes={shapes} del={:?}",
        collect_del(ap)
    );
    assert!(
        !collect_del(ap).contains("bookmark") && !collect_del(ap).contains("paragraph"),
        "bookmark residual must not mesh into a; del={:?}",
        collect_del(ap)
    );
    let bm_d = paras.iter().any(|p| {
        shape(p) == 'D'
            && (collect_del(p).contains("bookmark")
                || collect_del(p).contains("paragraph with a simple"))
    });
    assert!(bm_d, "bookmark residual as pure-D; shapes={shapes}");
}

#[test]
fn list_spacer1_still_peeled_without_table() {
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
        let t = collect_t(p).trim().to_string();
        t == "b" || t.starts_with('b')
    });
    if let Some(bp) = b_para {
        assert!(
            !collect_del(bp).contains("14.9"),
            "M290 still peels 14.9 from b"
        );
    }
}
