// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M320 — support_tickets × table_bookmark_end: first tables cell-mesh
//! (R1C1×Ticket ID). M319 — Heading2 pure-I "Test 1…" stays pure-I (no MIX
//! with pure-D "Support Tickets"); stamp body folds without heading style
//! still M90-fold.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn support_tickets_x_table_bookmark_no_test1_title_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("support_tickets_table.docx");
    let b = src.join("table_bookmark_end.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: corpus missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    let mixed = xml.split("<w:p").any(|p| {
        p.contains("w:ins")
            && p.contains("w:del")
            && p.contains("Test 1")
            && p.contains("Support Tickets")
    });
    assert!(
        !mixed,
        "Word keeps pure-I Test 1 header and pure-D Support Tickets title"
    );
}

#[test]
fn support_tickets_x_table_bookmark_first_table_cell_mesh() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("support_tickets_table.docx");
    let b = src.join("table_bookmark_end.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: corpus missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    // First body table must cell-mesh B's R1C1 with A's Ticket ID (M38 / Word).
    let start = xml.find("<w:tbl>").expect("a table in the body");
    let mut depth = 0usize;
    let mut i = start;
    let end = loop {
        if xml[i..].starts_with("<w:tbl>") {
            depth += 1;
            i += 7;
        } else if xml[i..].starts_with("</w:tbl>") {
            depth -= 1;
            i += 8;
            if depth == 0 {
                break i;
            }
        } else {
            i += 1;
        }
        if i >= xml.len() {
            panic!("unbalanced w:tbl");
        }
    };
    let tbl = &xml[start..end];
    assert!(
        tbl.contains("<w:ins ") && tbl.contains("R1C1"),
        "first table must carry B t1 inserted content (R1C1)"
    );
    assert!(
        tbl.contains("<w:del ") && tbl.contains("Ticket ID"),
        "first table must carry A deleted content (Ticket ID) — Word first-slot mesh"
    );
}
