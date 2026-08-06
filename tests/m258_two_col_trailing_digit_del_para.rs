// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M258 — two_col_index×tab: Word residual ends pure-I "6" then pure-D para "5".
//! Tip kept I6+D5 together; peel trailing single-digit del on last residual.

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

/// Collapse body revision atoms for one paragraph (after outermost pPr).
/// Prefer the last `</w:pPr>` before body content so nested `pPrChange/pPr`
/// does not steal the body slice.
fn para_atoms(p: &str) -> Vec<(char, String)> {
    let body = if let Some(idx) = p.rfind("</w:pPr>") {
        &p[idx + "</w:pPr>".len()..]
    } else {
        p
    };
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(pos) = rest.find("<w:") {
        rest = &rest[pos..];
        let tag = if rest.starts_with("<w:del") {
            'D'
        } else if rest.starts_with("<w:ins") {
            'I'
        } else if rest.starts_with("<w:r") {
            '='
        } else {
            rest = &rest[3..];
            continue;
        };
        let name = match tag {
            'D' => "w:del",
            'I' => "w:ins",
            _ => "w:r",
        };
        let end_tag = format!("</{name}>");
        let Some(end) = rest.find(&end_tag) else {
            break;
        };
        let chunk = &rest[..end + end_tag.len()];
        rest = &rest[end + end_tag.len()..];
        let mut text = String::new();
        for part in chunk.split("<w:t").skip(1) {
            if let (Some(a), Some(b)) = (part.find('>'), part.find("</w:t>")) {
                text.push_str(&part[a + 1..b]);
            }
        }
        for part in chunk.split("<w:delText").skip(1) {
            if let (Some(a), Some(b)) = (part.find('>'), part.find("</w:delText>")) {
                text.push_str(&part[a + 1..b]);
            }
        }
        if text.is_empty() && chunk.contains("<w:br") {
            text.push('\n');
        }
        if text.is_empty() {
            continue;
        }
        out.push((tag, text));
    }
    out
}

#[test]
fn two_col_index_trailing_digit_is_own_pure_d_para() {
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
    let paras: Vec<&str> = doc.split("</w:p>").filter(|p| p.contains("<w:p")).collect();
    let atom_lists: Vec<Vec<(char, String)>> = paras.iter().map(|p| para_atoms(p)).collect();

    // Pure-D para whose only text atom is digit "5".
    let pure_d5 = atom_lists.iter().any(|atoms| {
        !atoms.iter().any(|(k, _)| *k == 'I' || *k == '=')
            && atoms.iter().filter(|(k, _)| *k == 'D').count() >= 1
            && atoms
                .iter()
                .filter(|(_, t)| !t.trim().is_empty())
                .all(|(k, t)| *k == 'D' && t.trim() == "5")
    });
    assert!(
        pure_d5,
        "expected pure-D para with only digit 5; atoms={atom_lists:?}"
    );

    // No residual para still has both ins "6" and del "5".
    let mixed_6_5 = atom_lists.iter().any(|atoms| {
        let has_i6 = atoms.iter().any(|(k, t)| *k == 'I' && t.contains('6'));
        let has_d5 = atoms.iter().any(|(k, t)| *k == 'D' && t.trim() == "5");
        has_i6 && has_d5
    });
    assert!(
        !mixed_6_5,
        "I6 and D5 must not share a para; atoms={atom_lists:?}"
    );
}
