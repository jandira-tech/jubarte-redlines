// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M278b — full 06-22 regression: marketing_agenda / pre_separated×diff free-LCS
//! is lead pure-I + **single** pure-I table + trailing pure-D (LO~100). M278 moved
//! pure-D before that single TI and cratered LO ~100→50. Require ≥2 pure-I tables
//! in mid so pirates×border (many TI) still relocates pure-D early.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load_wb(name: &str) -> Option<Vec<u8>> {
    let p = Path::new("/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source")
        .join(name);
    std::fs::read(p).ok()
}

fn load_sd(name: &str) -> Option<Vec<u8>> {
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

fn body_blocks(doc: &str) -> Vec<(char, String)> {
    // Split body children roughly by paragraph/table open tags order via sequential scan.
    // Use full document and walk w:body children via simple tag split.
    let body_start = doc.find("<w:body").unwrap_or(0);
    let body = &doc[body_start..];
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(rel) = rest.find("<w:") {
        rest = &rest[rel..];
        if rest.starts_with("<w:sectPr") {
            break;
        }
        if rest.starts_with("<w:tbl") || rest.starts_with("<w:tbl ") {
            let end = rest.find("</w:tbl>").map(|i| i + 8).unwrap_or(rest.len());
            let chunk = &rest[..end];
            let has_ins = chunk.contains("<w:ins ") || chunk.contains("<w:ins>");
            let has_del = chunk.contains("<w:del ")
                || chunk.contains("<w:del>")
                || chunk.contains("<w:delText");
            let kind = match (has_ins, has_del) {
                (true, true) => 'X',  // TM
                (true, false) => 'T', // TI
                (false, true) => 'U', // TD
                _ => '=',
            };
            out.push((kind, chunk.chars().take(40).collect()));
            rest = &rest[end..];
            continue;
        }
        if rest.starts_with("<w:p ") || rest.starts_with("<w:p>") {
            let end = rest.find("</w:p>").map(|i| i + 6).unwrap_or(rest.len());
            let chunk = &rest[..end];
            let has_ins = chunk.contains("<w:ins ") || chunk.contains("<w:ins>");
            let has_del = chunk.contains("<w:del ")
                || chunk.contains("<w:del>")
                || chunk.contains("<w:delText");
            let kind = match (has_ins, has_del) {
                (true, true) => 'M',
                (true, false) => 'I',
                (false, true) => 'D',
                _ => 'E',
            };
            out.push((kind, String::new()));
            rest = &rest[end..];
            continue;
        }
        rest = &rest[3..];
    }
    out
}

#[test]
fn marketing_agenda_single_ti_stays_before_pure_d() {
    // marketing_strategy × meeting_agenda_table_2 — keys used in full 06-22 top regs.
    let names = [
        (
            "marketing_strategy_2026_suggesting_insertions.docx",
            "meeting_agenda_table_2.docx",
        ),
        (
            "Marketing_Strategy_2026_Suggesting_Insertions.docx",
            "Meeting_Agenda_Table_2.docx",
        ),
    ];
    let mut a_bytes = None;
    let mut b_bytes = None;
    for (an, bn) in names {
        if let (Some(a), Some(b)) = (load_wb(an), load_wb(bn)) {
            a_bytes = Some(a);
            b_bytes = Some(b);
            break;
        }
        if let (Some(a), Some(b)) = (load_sd(an), load_sd(bn)) {
            a_bytes = Some(a);
            b_bytes = Some(b);
            break;
        }
    }
    // Also try glob-ish listing via known full-key names in corpus
    if a_bytes.is_none() {
        for root in [
            "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source",
            "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
        ] {
            let p = Path::new(root);
            if !p.exists() {
                continue;
            }
            let mut marketing = None;
            let mut agenda = None;
            if let Ok(rd) = std::fs::read_dir(p) {
                for e in rd.flatten() {
                    let n = e.file_name().to_string_lossy().to_string();
                    let low = n.to_ascii_lowercase();
                    if low.contains("marketing_strategy") && low.ends_with(".docx") {
                        marketing = Some(e.path());
                    }
                    if low.contains("meeting_agenda") && low.ends_with(".docx") {
                        agenda = Some(e.path());
                    }
                }
            }
            if let (Some(m), Some(a)) = (marketing, agenda) {
                a_bytes = std::fs::read(m).ok();
                b_bytes = std::fs::read(a).ok();
                break;
            }
        }
    }
    let (Some(a), Some(b)) = (a_bytes, b_bytes) else {
        eprintln!("skip: marketing/agenda corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let blocks = body_blocks(&document_xml(&out));
    let sig: String = blocks.iter().map(|(k, _)| *k).collect();
    // Find first pure-I table (T) and first pure-D para (D)
    let first_ti = sig.find('T');
    let first_d = sig.find('D');
    assert!(
        first_ti.is_some() && first_d.is_some(),
        "expected TI and pure-D; sig={sig}"
    );
    assert!(
        first_ti.unwrap() < first_d.unwrap(),
        "single pure-I table must stay before pure-D residual (M278b); sig={sig}"
    );
}
