// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M345 — whitespace pure-I fold must not thrash pure-D list layout.
//!
//! basic_list × sd_1707_list_enter: Word keeps ListParagraph+numPr on the first
//! pure-D "List item 1". M341 fold order ate an empty pure-I into that pure-D
//! and left only the del mark (pagefair 99→66). Same for
//! pre_separated_list × diff_before7.
//!
//! two_column × vrect: Word keeps trailing empty pure-I before wholesale pure-D;
//! folding contaminated first pure-D with vrect spacing (79→34).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn document_xml(bytes: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    xml
}

fn compare(a: &PathBuf, b: &PathBuf) -> String {
    let out = compare_documents_with_settings(
        &std::fs::read(a).unwrap(),
        &std::fs::read(b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    document_xml(&out)
}

/// First pure-D carrying delText "List item 1" must keep ListParagraph + numPr.
#[test]
fn basic_list_first_pure_del_keeps_list_ppr() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__basic_list_0fcfe705.docx");
    let b = src.join("super_editor__sd_1707_list_enter_track_changes_with_fd93fd8b.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let xml = compare(&a, &b);
    // Find the paragraph that deletes "List item 1".
    let marker = "List item 1";
    let Some(pos) = xml.find(marker) else {
        panic!("delText List item 1 missing");
    };
    // Walk back to nearest <w:p
    let before = &xml[..pos];
    let p_start = before.rfind("<w:p").expect("p open");
    let p_end = xml[pos..].find("</w:p>").map(|j| pos + j).expect("p close");
    let p = &xml[p_start..p_end];
    assert!(
        p.contains("ListParagraph") && p.contains("numPr"),
        "first pure-D List item 1 must keep ListParagraph+numPr; pPr slice={}",
        &p[..p.len().min(400)]
    );
}

/// First pure-D of two_column residual must not inherit vrect empty-para spacing.
#[test]
fn two_column_x_vrect_first_del_no_ins_spacing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__two_column_two_page_0b8a37c5.docx");
    let b = src.join("super_editor__vrect_node_c8e51f22.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let xml = compare(&a, &b);
    let marker = "This is sample text to demonstrate a two-column";
    let Some(pos) = xml.find(marker) else {
        panic!("two-column delText missing");
    };
    let before = &xml[..pos];
    let p_start = before.rfind("<w:p").expect("p open");
    let p_end = xml[pos..].find("</w:p>").map(|j| pos + j).expect("p close");
    let p = &xml[p_start..p_end];
    // Bare pure-D: del mark only — no spacing carried from vrect empty pure-I.
    assert!(
        !p.contains("w:spacing"),
        "first two-column pure-D must not inherit empty pure-I spacing; got {}",
        &p[..p.len().min(350)]
    );
}
