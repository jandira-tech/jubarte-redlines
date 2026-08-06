// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M289 — list_with_break_from_word × list_with_table_break:
//! Tip free-LCS emits lead pure-I (ONE/A) then pure-D base residual then
//! pure-I table + trailing pure-I (d/TWO). Word parks pure-I table block
//! before pure-D residual: I* TI I* D*.
//! Relocate pure-D after the pure-I table block when pure-D is immediately
//! followed by a pure-I table (TI), not pure-D table (TD — pirates stays).

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

/// Compact body stream: I/D/M/E for paras, TI/TD/TM/TE for tables.
fn compact_stream(doc: &str) -> String {
    // Split body roughly on top-level-ish tags by walking after <w:body>
    let Some(body_start) = doc.find("<w:body") else {
        return String::new();
    };
    let body = &doc[body_start..];
    let mut out = String::new();
    let mut rest = body;
    while let Some(p) = rest.find('<') {
        rest = &rest[p..];
        if rest.starts_with("<w:sectPr") {
            break;
        }
        if rest.starts_with("<w:tbl") {
            let end = rest.find("</w:tbl>").map(|i| i + 8).unwrap_or(rest.len());
            let tbl = &rest[..end];
            let has_ins = tbl.contains("<w:ins ") || tbl.contains("<w:ins>");
            let has_del =
                tbl.contains("<w:del ") || tbl.contains("<w:del>") || tbl.contains("<w:delText");
            out.push_str(match (has_ins, has_del) {
                (true, true) => "TM",
                (true, false) => "TI",
                (false, true) => "TD",
                _ => "TE",
            });
            rest = &rest[end..];
            continue;
        }
        if rest.starts_with("<w:p ") || rest.starts_with("<w:p>") || rest.starts_with("<w:p/") {
            let end = if rest.starts_with("<w:p/") {
                rest.find("/>").map(|i| i + 2).unwrap_or(2)
            } else {
                rest.find("</w:p>").map(|i| i + 6).unwrap_or(rest.len())
            };
            let p = &rest[..end];
            let has_ins = p.contains("<w:ins ") || p.contains("<w:ins>");
            let has_del =
                p.contains("<w:del ") || p.contains("<w:del>") || p.contains("<w:delText");
            out.push(match (has_ins, has_del) {
                (true, true) => 'M',
                (true, false) => 'I',
                (false, true) => 'D',
                _ => 'E',
            });
            rest = &rest[end..];
            continue;
        }
        // skip other tags
        rest = &rest[1..];
    }
    out
}

#[test]
fn list_with_break_pure_d_after_pure_ins_table() {
    let Some(a) = load("super_editor__list_with_break_from_word_5cc0d638.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__list_with_table_break_ff0c4c1f.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let stream = compact_stream(&doc);
    // Word: III TI IIII D* E?  — pure-I table before pure-D residual
    // Pre-M289 tip: III D* TI IIII
    assert!(
        stream.contains("TI"),
        "expected pure-I table; stream={stream}"
    );
    let ti = stream.find("TI").expect("TI");
    let first_d = stream.find('D');
    assert!(
        first_d.is_some(),
        "expected pure-D residual; stream={stream}"
    );
    assert!(
        first_d.unwrap() > ti,
        "pure-D base residual must follow pure-I table (Word I*TI*I*D*); stream={stream}"
    );
    // Lead pure-I still before table.
    assert!(
        stream.starts_with("III") || stream.starts_with("II"),
        "lead pure-I ONE/A retained; stream={stream}"
    );
}

#[test]
fn pirates_keeps_early_pure_d_before_td_tables() {
    // M289 must not fire on pirates: pure-D followed by TD (not TI).
    let Some(a) = load("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__sd_1494_table_left_indent_11bb24c7.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let stream = compact_stream(&doc);
    let first_d = stream.find('D').expect("pure-D");
    let first_td = stream.find("TD");
    // Word/tip: pure-D early, then TD tables.
    if let Some(td) = first_td {
        assert!(
            first_d < td,
            "pirates pure-D must stay before pure-D tables; stream={stream}"
        );
    }
}
