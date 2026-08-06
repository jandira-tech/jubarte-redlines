// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! pci_table × pre_separated_list: Word pure-ins every B list item then
//! pure-del A table residual (IIIIIIIII DDDD…). Free LCS mixed list lines
//! with cell text (MMMMIMMMM D…).

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
fn pci_table_x_presep_list_pure_ins_b_then_pure_del_a() {
    let Some(a) = load("super_editor__pci_table_dc840852.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__pre_separated_list_bd330d28.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let shape = body_para_kinds(&doc);
    eprintln!("shape={shape}");
    assert!(doc.contains("List item 1"), "B list lead text present");
    // First body para with List item 1 must be pure-ins, not MIX with table del.
    let first_list = {
        let body = doc
            .split("<w:body")
            .nth(1)
            .and_then(|s| s.split("</w:body>").next())
            .unwrap_or(&doc);
        let mut rest = body;
        loop {
            let Some(start) = rest.find("<w:p") else {
                break None;
            };
            let slice = &rest[start..];
            if !slice.starts_with("<w:p ") && !slice.starts_with("<w:p>") {
                rest = &rest[start + 4..];
                continue;
            }
            let Some(end) = slice.find("</w:p>") else {
                break None;
            };
            let p = &slice[..end + 6];
            if p.contains("List item 1") {
                break Some(p.to_string());
            }
            rest = &slice[end + 6..];
        }
    };
    let p0 = first_list.expect("List item 1 para");
    assert!(
        p0.contains("<w:ins") && p0.contains("List item 1"),
        "List item 1 must appear under ins; p0={p0}"
    );
    // Body must not carry deleted table text on the list lead (pre-M243 MIX).
    assert!(
        !p0.contains("delText") && !p0.contains("Requirement"),
        "list lead must not embed pure-D table body text; p0={p0}"
    );
    // Word pure-ins lead: pPr ins only (M243c strips dual del pilcrow).
    let ppr = p0.split("<w:r").next().unwrap_or(&p0);
    assert!(
        ppr.contains("<w:ins") || p0.contains("<w:ins"),
        "list lead needs ins mark; p0={p0}"
    );
    assert!(
        !ppr.contains("<w:del"),
        "pure-ins list lead must not keep del pilcrow; shape={shape} ppr={ppr}"
    );
    assert!(
        shape.chars().take(4).all(|c| matches!(c, 'I' | 'i')),
        "lead B list pure-ins run (Word IIII…D); shape={shape}"
    );
}
