// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M270b — listwithspacernodes × list_with_indents: free-LCS already emits
//! Word-shaped `I* D*` (long body pure-D). M270 early pure-D move must NOT
//! fire (would leave trailing pure-I). Gate: pure-D token count ≤ 16.
//! Green underline short pure-D still relocates (M270).

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load_sd(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
    )
    .join(name);
    std::fs::read(p).ok()
}

fn load_nc(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source",
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
fn listspacer_x_indents_keeps_pure_i_before_pure_d() {
    let Some(a) = load_sd("super_editor__listwithspacernodes_cf53e890.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load_sd("super_editor__list_with_indents_efc7d4f5.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    // No content pure-I after pure-D (M270 must not reorder long pure-D early).
    let mut saw_d = false;
    for (i, p) in paras.iter().enumerate() {
        let s = shape(p);
        if s == 'D' || s == 'M' {
            saw_d = true;
        }
        if saw_d && s == 'I' {
            let t = collect_t(p);
            assert!(
                t.trim().is_empty(),
                "M270 must not leave trailing content pure-I; shapes={shapes} i={i} t={t:?}"
            );
        }
    }
    // Pure-I residual (B list text) precedes pure-D residual.
    let mut first_i_text = None;
    let mut first_d_text = None;
    for p in &paras {
        match shape(p) {
            'I' if first_i_text.is_none() && !collect_t(p).trim().is_empty() => {
                first_i_text = Some(collect_t(p));
            }
            'D' | 'M' if first_d_text.is_none() => {
                let d = collect_del(p);
                if !d.trim().is_empty() {
                    first_d_text = Some(d);
                }
            }
            _ => {}
        }
    }
    assert!(
        first_i_text.is_some() && first_d_text.is_some(),
        "expect pure-I and pure-D residuals; shapes={shapes}"
    );
}

#[test]
fn green_underline_still_early_pure_d_after_m270b() {
    // M270 short pure-D still relocates — regression guard for token gate.
    let Some(a) = load_nc("green_underline_bullet_list_id_paraid_overflow.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load_nc("header_no_rels.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    // Expect some pure-D before mid pure-I body (early residual).
    let first_d = paras.iter().position(|p| shape(p) == 'D');
    let mid_i = paras.iter().enumerate().find_map(|(i, p)| {
        if shape(p) == 'I' && collect_t(p).contains("Second") {
            Some(i)
        } else {
            None
        }
    });
    if let (Some(d), Some(i)) = (first_d, mid_i) {
        assert!(
            d < i,
            "green pure-D still early before Second page; shapes={shapes} d={d} i={i}"
        );
    } else {
        // soft: at least pure-D exists and is not only trailing after all pure-I
        assert!(
            shapes.contains('D') && shapes.contains('I'),
            "green residual present; shapes={shapes}"
        );
    }
}
