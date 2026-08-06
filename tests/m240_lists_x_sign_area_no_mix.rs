// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! lists_sub_paragraph × longer_header_sign_area: Word pure-ins signature
//! blocks then pure-del list residual (IIDd). Folding last signature into the
//! ListParagraph del produced IMd and dragged LO.

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
fn lists_x_sign_area_no_mix_signature_into_list_del() {
    let Some(a) = load("super_editor__lists_sub_paragraph_31ff3fed.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__longer_header_sign_area_7c80b525.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let shape = body_para_kinds(&doc);
    eprintln!("shape={shape}");
    // Word: IIDd — pure-ins signature lead, not MIX of By:… into ListParagraph del.
    assert!(
        shape.starts_with('I') || shape.starts_with('i'),
        "lead pure-ins signature; shape={shape}"
    );
    assert!(
        !shape.starts_with("IM") && !shape.starts_with("Im") && !shape.starts_with("iM"),
        "must not fold signature pure-I into first list pure-D (Word IIDd); shape={shape}"
    );
    assert!(
        doc.contains("By:") || doc.contains("____"),
        "signature text present"
    );
    // List del residual should exist as pure-D (or d), not only inside MIX.
    assert!(
        shape.contains('D') || shape.contains('d'),
        "list residual pure-del expected; shape={shape}"
    );
}
