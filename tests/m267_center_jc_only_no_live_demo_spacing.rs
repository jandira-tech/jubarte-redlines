// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M267 — center_alignment × center_bold title: Word emits pPrChange(jc only)
//! with **no** live demo `line=276`. M251's keep-for-nonempty-old treated
//! jc-only history like tabs and kept live spacing. Same-pipeline LO is flat
//! either way; keep Word shape (strip jc-only) while preserving M251 tabs keep.

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

fn outer_ppr(para: &str) -> String {
    let Some(i) = para.find("<w:pPr") else {
        return String::new();
    };
    let rest = &para[i..];
    let bytes = rest.as_bytes();
    let mut depth = 0i32;
    let mut j = 0;
    while j + 5 < bytes.len() {
        if rest[j..].starts_with("<w:pPr") {
            if let Some(end) = rest[j..].find('>') {
                let tag = &rest[j..j + end + 1];
                if tag.ends_with("/>") {
                    if depth == 0 {
                        return rest[..=j + end].to_string();
                    }
                    j += end + 1;
                    continue;
                }
            }
            depth += 1;
            j += 5;
        } else if rest[j..].starts_with("</w:pPr>") {
            depth -= 1;
            j += 8;
            if depth == 0 {
                return rest[..j].to_string();
            }
        } else {
            j += 1;
        }
    }
    rest.to_string()
}

fn live_ppr_fragment(ppr: &str) -> &str {
    ppr.split("pPrChange").next().unwrap_or(ppr)
}

#[test]
fn center_align_x_center_bold_title_no_live_demo_spacing() {
    let Some(a) = load("center_alignment_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: neurotic corpus missing");
        return;
    };
    let Some(b) = load("center_bold_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: neurotic corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    assert!(!paras.is_empty());
    let ppr = outer_ppr(&paras[0]);
    assert!(
        ppr.contains("pPrChange") && ppr.contains("center"),
        "title must keep jc history; pPr={ppr}"
    );
    let live = live_ppr_fragment(&ppr);
    assert!(
        !live.contains("w:spacing") && !live.contains("w:line=\"276\""),
        "Word omits live demo line=276 when pPrChange is jc-only; live={live}"
    );
}
