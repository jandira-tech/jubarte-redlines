// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M261 — two_col_index×tab: Word residual MIX " Page6" keeps live tabs+spacing
//! but NO pPrChange. Tip carried pPrChange(old tabs) from M251 spacing
//! addition on the residual. Pure-D page lines still keep pPrChange.

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

fn body_text_after_ppr(chunk: &str) -> String {
    let body = if let Some(idx) = chunk.rfind("</w:pPr>") {
        &chunk[idx + "</w:pPr>".len()..]
    } else {
        chunk
    };
    let mut text = String::new();
    for part in body.split("<w:t").skip(1) {
        if let (Some(a), Some(b)) = (part.find('>'), part.find("</w:t>")) {
            text.push_str(&part[a + 1..b]);
        }
    }
    for part in body.split("<w:delText").skip(1) {
        if let (Some(a), Some(b)) = (part.find('>'), part.find("</w:delText>")) {
            text.push_str(&part[a + 1..b]);
        }
    }
    text
}

#[test]
fn two_col_residual_mix_has_spacing_without_pprchange() {
    let Some(a) = load("super_editor__sd_1480_two_col_index_0138dccc.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__sd_1480_two_col_tab_positions_00953280.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    // Residual MIX carries both del and ins body text and ends with digit-ish
    // residual (" Page6" / Page + 6). Must have live spacing, must NOT have
    // pPrChange (Word shape).
    let mut found_residual = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("<w:p") {
            continue;
        }
        let body = if let Some(idx) = chunk.rfind("</w:pPr>") {
            &chunk[idx + "</w:pPr>".len()..]
        } else {
            chunk
        };
        let has_del = body.contains("<w:del ") || body.contains("<w:del>");
        let has_ins = body.contains("<w:ins ") || body.contains("<w:ins>");
        if !has_del || !has_ins {
            continue;
        }
        let text = body_text_after_ppr(chunk);
        // Residual after Page4 peel: whitespace + Page + 6 (or similar).
        if !(text.contains('6') && (text.contains("Page") || text.trim().starts_with(' '))) {
            continue;
        }
        found_residual = true;
        assert!(
            chunk.contains("w:spacing") && chunk.contains("w:line=\"276\""),
            "residual MIX must keep live spacing; chunk={chunk}"
        );
        assert!(
            !chunk.contains("pPrChange"),
            "residual MIX must not carry pPrChange(old tabs); chunk={chunk}"
        );
        break;
    }
    assert!(found_residual, "expected MIX residual with Page/6");

    // Pure-D page lines still keep pPrChange (Word Page1/2/3).
    let pure_d_page = doc.split("</w:p>").any(|chunk| {
        if !chunk.contains("<w:p") {
            return false;
        }
        let body = if let Some(idx) = chunk.rfind("</w:pPr>") {
            &chunk[idx + "</w:pPr>".len()..]
        } else {
            chunk
        };
        let has_del = body.contains("<w:del ") || body.contains("<w:del>");
        let has_ins = body.contains("<w:ins ") || body.contains("<w:ins>");
        if !has_del || has_ins {
            return false;
        }
        let text = body_text_after_ppr(chunk);
        text.contains("Page1") && chunk.contains("pPrChange")
    });
    assert!(
        pure_d_page,
        "pure-D Page1 must still keep pPrChange from M251"
    );
}
