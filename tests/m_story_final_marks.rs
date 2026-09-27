// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word pairs the two documents' final paragraph marks. The revised list ends
//! at "Ωω Omega"; the original goes on with "Meeting Agenda" and a table.
//! Word joins "Ωω Omega" to "Meeting Agenda" under the original's paragraph
//! properties with a deleted mark, and the story-final paragraph after the
//! deleted table takes the revised properties. Full LCS gave the joined
//! paragraph the revised style and a live mark and left the final paragraph
//! plain, one line higher than Word.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::W;
use jubarte::revision_processor::{accept_revisions_document, reject_revisions_document};
use jubarte::xmllinq::Dom;
use std::io::{Cursor, Read};
use std::path::PathBuf;

fn redline_xml(original: &str, revised: &str) -> String {
    redline_xml_in("tests/corpus/broken_ones_two/sources", original, revised)
}

fn redline_xml_in(dir: &str, original: &str, revised: &str) -> String {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(dir);
    let out = compare_documents_with_settings(
        &std::fs::read(src.join(original)).unwrap(),
        &std::fs::read(src.join(revised)).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

/// Without a deleted paragraph ahead of the inserted ones (file_134 ×
/// file_135: the revised document inserts three paragraphs after the shared
/// title, the original's body is deleted), Word still pairs the final marks:
/// the revised last paragraph joins the first deleted one, "Table Widths".
/// A live inserted mark there pushed every later line down one (13 vs 98).
#[test]
fn inserted_last_paragraph_joins_first_deleted_paragraph() {
    let xml = redline_xml("file_134.docx", "file_135.docx");
    let joined = xml
        .split("</w:p>")
        .find(|p| p.contains("Subtitle style provides"))
        .expect("revised last paragraph");
    assert!(
        joined.contains("Table Widths"),
        "revised last paragraph should join the first deleted one: {joined}"
    );
}

#[test]
fn revised_final_mark_pairs_with_original_final_mark() {
    let xml = redline_xml("file_205.docx", "file_206.docx");
    let joined = xml
        .split("</w:p>")
        .find(|p| p.contains("Meeting Agenda"))
        .expect("joined paragraph");
    assert!(
        joined.contains("Omega") && !joined.contains("PreformattedText"),
        "joined paragraph should keep the original's properties: {joined}"
    );
    let tail = &xml[xml.rfind("</w:tbl>").expect("deleted table")..];
    assert!(
        tail.contains("PreformattedText"),
        "story-final paragraph should carry the revised properties: {tail}"
    );
}

/// Each paragraph's text after accepting (or rejecting) every revision.
fn resolved_texts(xml: &str, accept: bool) -> Vec<String> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).unwrap();
    let root = if accept {
        accept_revisions_document(&mut dom, root)
    } else {
        reject_revisions_document(&mut dom, root)
    };
    dom.descendants(root, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect()
        })
        .collect()
}

/// A replaced tail with inserted paragraphs ahead of the deleted ones
/// (bullet_list_bold × bullet_list: four new bullets, the original's intro
/// and three bold bullets deleted). Word pairs the final marks and joins the
/// last inserted paragraph, "Grapes", to the first deleted one with a
/// deleted mark. Left unpaired, the original's last mark stayed live and
/// accepting the redline left an empty paragraph the revised document never
/// had.
#[test]
fn inserted_tail_ahead_of_deleted_tail_pairs_the_final_marks() {
    let dir = "tests/corpus/story_final_marks";
    let xml = redline_xml_in(dir, "bullet_list_bold.docx", "bullet_list.docx");
    let joined = xml
        .split("</w:p>")
        .find(|p| p.contains(">Grapes<"))
        .expect("Grapes paragraph");
    assert!(
        joined.contains("This document demonstrates"),
        "Grapes should join the first deleted paragraph: {joined}"
    );
    assert_eq!(
        resolved_texts(&xml, true),
        ["Bullet List Demo", "Apples", "Bananas", "Oranges", "Grapes"],
    );
    assert_eq!(
        resolved_texts(&xml, false),
        [
            "Bullet List Bold Demo",
            "This document demonstrates bullet lists with bold text:",
            "First bold bullet item",
            "Second bold bullet item",
            "Third bold bullet item",
        ],
    );
}
