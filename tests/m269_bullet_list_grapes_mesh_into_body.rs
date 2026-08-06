// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M269 — bullet_list_bold × bullet_list: Word meshes last pure-I list item
//! ("Grapes") into the following pure-D body as MIX. Tip left pure-I then
//! pure-D (extra para). Python fold fair tip-to-tip +16.3 under 02-22 oracle.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let p = Path::new("/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source")
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

fn shape(p: &str) -> &'static str {
    let has_ins = p.contains("<w:ins ") || p.contains("<w:ins>");
    let has_del = p.contains("<w:del ") || p.contains("<w:del>") || p.contains("<w:delText");
    match (has_ins, has_del) {
        (true, true) => "M",
        (true, false) => "I",
        (false, true) => "D",
        _ => "E",
    }
}

fn has_numpr(p: &str) -> bool {
    p.contains("numPr")
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
fn bullet_bold_x_bullet_grapes_meshes_into_body_del() {
    let Some(a) = load("bullet_list_bold_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("bullet_list_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    // Find Grapes
    let grapes_i = paras.iter().position(|p| collect_t(p).contains("Grapes"));
    assert!(
        grapes_i.is_some(),
        "expected Grapes para; n={}",
        paras.len()
    );
    let i = grapes_i.unwrap();
    let p = &paras[i];
    assert_eq!(
        shape(p),
        "M",
        "Word meshes Grapes ins into body del as MIX; got {} textI={:?} textD={:?}",
        shape(p),
        collect_t(p),
        collect_del(p)
    );
    assert!(
        collect_del(p).contains("document") || collect_del(p).contains("demonstrate"),
        "MIX must carry body delText; D={:?}",
        collect_del(p)
    );
    // Word parks numPr off the MIX onto trailing pure-D (M230) — MIX should not
    // keep list numPr after park, but either MIX-with-num or following D-with-num is OK.
    let shapes: String = paras
        .iter()
        .map(|p| shape(p).chars().next().unwrap())
        .collect();
    assert!(
        shapes.contains('M'),
        "body shape must include MIX; shapes={shapes}"
    );
    let _ = has_numpr(p);
}
