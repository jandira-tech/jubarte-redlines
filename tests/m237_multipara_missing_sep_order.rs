// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! multipara_cell × missing_separator: Word pure-ins B lead (empty/PARTIES/…)
//! then MIX/del of A's title + pure-D table. Ours put MIX(title) first
//! (MIIII D…) — residual order wrong for LO (~70).

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

/// Classify body top-level paragraphs: I / D / M / = (rough Word LO shape).
fn body_para_kinds(doc: &str) -> Vec<char> {
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or(doc);
    let paras: Vec<&str> = {
        let mut out = Vec::new();
        let mut rest = body;
        while let Some(start) = rest.find("<w:p") {
            let slice = &rest[start..];
            // only top-level-ish: require following space or >
            if !slice.starts_with("<w:p ") && !slice.starts_with("<w:p>") {
                rest = &rest[start + 4..];
                continue;
            }
            if let Some(end) = slice.find("</w:p>") {
                out.push(&slice[..end + 6]);
                rest = &slice[end + 6..];
            } else {
                break;
            }
        }
        out
    };
    paras
        .into_iter()
        .map(|p| {
            let ppr = p.split("<w:r").next().unwrap_or(p);
            let ins_p = ppr.contains("<w:ins");
            let del_p = ppr.contains("<w:del");
            let n_ins = p.matches("<w:ins").count();
            let n_del = p.matches("<w:del").count();
            if del_p && !ins_p && n_ins == 0 {
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
            }
        })
        .collect()
}

#[test]
fn multipara_x_missing_separator_pure_ins_b_before_title_del() {
    let Some(a) = load("behavior__sd_2672_multipara_cell_4d1f068e.docx") else {
        eprintln!("skip: multipara fixture missing");
        return;
    };
    let Some(b) = load("super_editor__missing_separator_41c823b9.docx") else {
        eprintln!("skip: missing_separator fixture missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let kinds = body_para_kinds(&doc);
    let shape: String = kinds.iter().collect();
    eprintln!("shape={shape}");

    // Word: IIIIMDDDD… — pure-ins B lead before any title del/MIX.
    // Engine may mark lead as 'I' (pPr ins) or 'i' (body ins only).
    assert!(!kinds.is_empty(), "expected body paragraphs; shape={shape}");
    assert!(
        matches!(kinds[0], 'I' | 'i'),
        "lead must be pure-ins B (Word IIIIMD…), not title MIX/del; shape={shape}"
    );
    assert!(
        !matches!(kinds[0], 'M' | 'D' | 'd'),
        "lead must not be A title del/MIX first; shape={shape}"
    );
    assert!(doc.contains("PARTIES"), "B text PARTIES must appear");
    // First occurrence of SD-2672 title del must come AFTER at least one pure-I.
    let first_i = kinds.iter().position(|&k| matches!(k, 'I' | 'i'));
    let first_title_del = {
        let body = doc
            .split("<w:body")
            .nth(1)
            .and_then(|s| s.split("</w:body>").next())
            .unwrap_or(&doc);
        let mut rest = body;
        let mut idx = 0usize;
        let mut found = None;
        while let Some(start) = rest.find("<w:p") {
            let slice = &rest[start..];
            if !slice.starts_with("<w:p ") && !slice.starts_with("<w:p>") {
                rest = &rest[start + 4..];
                continue;
            }
            if let Some(end) = slice.find("</w:p>") {
                let p = &slice[..end + 6];
                if p.contains("SD-2672") {
                    found = Some(idx);
                    break;
                }
                idx += 1;
                rest = &slice[end + 6..];
            } else {
                break;
            }
        }
        found
    };
    assert!(
        first_title_del.is_some(),
        "expected del/mix of A title SD-2672; shape={shape}"
    );
    let ti = first_title_del.unwrap();
    let ii = first_i.expect("pure-I");
    assert!(
        ii < ti,
        "pure-ins B must precede A title del/MIX (Word IIIIMD…); i={ii} title={ti} shape={shape}"
    );
    // Must not start with title MIX (pre-M237 regression MIIII…).
    assert!(
        !shape.starts_with('M'),
        "must not open with title MIX; shape={shape}"
    );
    // M238: Word keeps Heading1 on the MIX/del title (A's style), not B's
    // empty pure-I spacing-only pPr.
    let title_para = {
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
            if p.contains("SD-2672") {
                break Some(p.to_string());
            }
            rest = &slice[end + 6..];
        }
    };
    let title = title_para.expect("title para with SD-2672");
    assert!(
        title.contains("Heading1") || title.contains("w:val=\"Heading1\""),
        "MIX/del title must keep A's Heading1 pStyle (Word); title pPr missing style"
    );
}
