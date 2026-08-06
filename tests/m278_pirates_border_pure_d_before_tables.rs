// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M278 — pirates×table_border_widths: free-LCS emits pure-I tables then trailing
//! pure-D residual. Word parks pure-D residual after lead pure-I paras
//! (`III D* TM I TI…`). Relocate trailing pure-D before pure-I tables.

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

/// Body child kinds: I/D/M/= for p, TI/TD/TM for tbl.
fn body_kinds(doc: &str) -> String {
    let mut out = String::new();
    // crude split on top-level-ish markers in body
    let Some(start) = doc.find("<w:body") else {
        return out;
    };
    let body = &doc[start..];
    let Some(end) = body.find("</w:body>") else {
        return out;
    };
    let body = &body[..end];
    // walk opening tags for p and tbl at shallow depth — use sequential scan
    let mut i = 0;
    let b = body.as_bytes();
    while i + 4 < b.len() {
        if body[i..].starts_with("<w:tbl")
            || body[i..].starts_with("<w:tbl ")
            || body[i..].starts_with("<w:tbl>")
        {
            // find end of this table
            let rest = &body[i..];
            let close = rest.find("</w:tbl>").unwrap_or(rest.len());
            let tbl = &rest[..close.min(rest.len())];
            let has_ins = tbl.contains("<w:ins ") || tbl.contains("<w:ins>");
            let has_del =
                tbl.contains("<w:del ") || tbl.contains("<w:del>") || tbl.contains("<w:delText");
            out.push(match (has_ins, has_del) {
                (true, true) => 'M',
                (true, false) => 'I',
                (false, true) => 'D',
                _ => 'T',
            });
            // mark as table by lowercase? use distinct: push and continue
            // We'll use: table I/D/M as uppercase with prefix via second pass — keep simple I/D/M/= for both
            i += close + 8;
            continue;
        }
        if body[i..].starts_with("<w:p ") || body[i..].starts_with("<w:p>") {
            let rest = &body[i..];
            let close = rest.find("</w:p>").unwrap_or(rest.len());
            let p = &rest[..close.min(rest.len())];
            // skip nested by only counting if not inside tbl — our scan is linear so nested p in tbl already skipped if we jump tbl
            let has_ins = p.contains("<w:ins ") || p.contains("<w:ins>");
            let has_del =
                p.contains("<w:del ") || p.contains("<w:del>") || p.contains("<w:delText");
            out.push(match (has_ins, has_del) {
                (true, true) => 'M',
                (true, false) => 'I',
                (false, true) => 'D',
                _ => '=',
            });
            i += close + 6;
            continue;
        }
        i += 1;
    }
    out
}

#[test]
fn pirates_x_border_widths_pure_d_before_pure_ins_tables() {
    let Some(a) = load("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("behavior__sd_2343_table_border_widths_b5148e83.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let kinds = body_kinds(&doc);
    // Lead pure-I, then pure-D residual before later pure-I content.
    assert!(
        kinds.starts_with("III") || kinds.starts_with("II"),
        "lead pure-I; kinds={kinds}"
    );
    let first_d = kinds.find('D');
    assert!(first_d.is_some(), "expect pure-D residual; kinds={kinds}");
    let d = first_d.unwrap();
    assert!(
        d <= 6,
        "pure-D residual must start early (after lead), not trailing; kinds={kinds} first_d={d}"
    );
    // Pure-D run should be followed by more content (tables / pure-I), not only end.
    let after = &kinds[d..];
    let d_end = after.chars().take_while(|&c| c == 'D' || c == 'M').count();
    assert!(
        d + d_end < kinds.len().saturating_sub(1),
        "pure-D not only trailing tail; kinds={kinds}"
    );
}
