// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M282 — exported_list_font × exporttest: Word meshes pure-D residual
//! `APPOINTMENT` onto the last pure-I list item `b` as MIX. Tip (post-M270)
//! parks pure-D after the first list item `a`.

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
fn exporttest_last_list_item_mixes_appointment_del() {
    let Some(a) = load("super_editor__exported_list_font_8e6db734.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__exporttest_68b3b898.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    assert!(!paras.is_empty(), "empty body");

    // No standalone pure-D APPOINTMENT early in the list.
    let early_pure_d = paras.iter().enumerate().find(|(_, p)| {
        shape(p) == 'D' && collect_del(p).to_ascii_uppercase().contains("APPOINTMENT")
    });
    assert!(
        early_pure_d.is_none(),
        "APPOINTMENT must not stay pure-D mid-list; shapes={shapes} hit={early_pure_d:?}"
    );

    // Last non-empty residual is MIX with ins `b` + del APPOINTMENT.
    let last = paras
        .iter()
        .rev()
        .find(|p| !collect_t(p).trim().is_empty() || !collect_del(p).trim().is_empty())
        .expect("content");
    assert_eq!(
        shape(last),
        'M',
        "last residual MIX b|APPOINTMENT; shapes={shapes}"
    );
    let i = collect_t(last);
    let d = collect_del(last);
    assert!(
        i.to_ascii_lowercase().contains('b'),
        "ins b; I={i:?} shapes={shapes}"
    );
    assert!(
        d.to_ascii_uppercase().contains("APPOINTMENT"),
        "del APPOINTMENT; D={d:?} shapes={shapes}"
    );
}
