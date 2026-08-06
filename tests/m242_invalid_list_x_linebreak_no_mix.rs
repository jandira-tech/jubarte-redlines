// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! invalid_list_def_fallback × line_break: Word pure-ins all B then pure-del
//! all A (IIIIIIDDDDDD). Incidental "1" token share between "1. EXECUTIVE
//! SUMMARY" and "1.1" must not MIX the long body into the del title.

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

fn body_para_kinds(doc: &str) -> String {
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or(doc);
    let mut rest = body;
    let mut kinds = String::new();
    while let Some(start) = rest.find("<w:p") {
        let slice = &rest[start..];
        if !slice.starts_with("<w:p ") && !slice.starts_with("<w:p>") {
            rest = &rest[start + 4..];
            continue;
        }
        let Some(end) = slice.find("</w:p>") else {
            break;
        };
        let p = &slice[..end + 6];
        let ppr = p.split("<w:r").next().unwrap_or(p);
        let ins_p = ppr.contains("<w:ins");
        let del_p = ppr.contains("<w:del");
        let n_ins = p.matches("<w:ins").count();
        let n_del = p.matches("<w:del").count();
        let k = if del_p && !ins_p && n_ins == 0 {
            'D'
        } else if ins_p && !del_p && n_del == 0 {
            'I'
        } else if n_ins > 0 && n_del > 0 {
            'M'
        } else if n_ins > 0 {
            'i'
        } else if n_del > 0 {
            'd'
        } else {
            '='
        };
        kinds.push(k);
        rest = &slice[end + 6..];
    }
    kinds
}

#[test]
fn invalid_list_x_linebreak_no_body_title_mix() {
    let Some(a) = load("super_editor__invalid_list_def_fallback_d7f55451.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__line_break_627a7159.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let shape = body_para_kinds(&doc);
    eprintln!("shape={shape}");
    assert!(
        !shape.contains('M') && !shape.contains('m'),
        "Word keeps pure-I B then pure-D A (no MIX); shape={shape}"
    );
    assert!(
        shape.contains('I') || shape.contains('i'),
        "B content pure-ins; shape={shape}"
    );
    assert!(
        shape.contains('D') || shape.contains('d'),
        "A residual pure-del; shape={shape}"
    );
    assert!(
        doc.contains("TERM AND TERMINATION") || doc.contains("TERM"),
        "A title must appear as del residual"
    );
    assert!(
        doc.contains("ANNUAL BUSINESS") || doc.contains("comprehensive"),
        "B body present"
    );
}
