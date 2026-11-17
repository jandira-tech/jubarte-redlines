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

/// The super_editor corpus in the sibling benchmark checkout; `None` (skip)
/// when it is not there.
fn superdoc_redline(original: &str, revised: &str) -> Option<(Dom, jubarte::xmllinq::NodeId)> {
    let dir = "tests/corpus/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source";
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(dir);
    if !src.join(original).exists() || !src.join(revised).exists() {
        eprintln!("skip: {dir} missing");
        return None;
    }
    let xml = redline_xml_in(dir, original, revised);
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    Some((dom, root))
}

/// An unrelated replacement whose revised document ends on an empty paragraph
/// still pairs the two final marks: the original's last paragraph ("I will
/// add a comment to this one.") is deleted into the revised final paragraph,
/// which keeps the revised properties (justified, outline level 1, a 16 pt
/// bold Arial mark). That taller mark line puts Word's redline on two pages;
/// the original's bare properties kept it on one (harness 2.8 vs Docxodus
/// 93.9).
#[test]
fn unrelated_replacement_pairs_the_final_marks() {
    let Some((dom, root)) = superdoc_redline(
        "super_editor__diff_after8_58e5c288.docx",
        "super_editor__doc_with_spacing_e3d47bd7.docx",
    ) else {
        return;
    };
    let last = *dom.descendants(root, Some(&W::p())).last().unwrap();
    let deleted: String = dom
        .descendants(last, Some(&W::del_text()))
        .into_iter()
        .map(|t| dom.value(t))
        .collect();
    assert_eq!(deleted, "I will add a comment to this one.");
    let ppr = dom.element(last, &W::p_pr()).expect("revised properties");
    let outline = dom
        .element(ppr, &W::name("outlineLvl"))
        .expect("outlineLvl");
    assert_eq!(dom.attribute(outline, &W::val()), Some("1"));
    let spacing = dom.element(ppr, &W::spacing_el()).expect("live spacing");
    assert_eq!(dom.attribute(spacing, &W::name("before")), Some("100"));
}

/// With the final marks paired and the revised document ending on an empty
/// paragraph, its last content paragraph closes on its own inserted mark:
/// Word folds nothing across the boundary between the inserted and the
/// deleted paragraphs ("All image types …" stays whole, "sqrt_degHide :"
/// keeps its deleted mark).
#[test]
fn paired_final_marks_fold_nothing_at_the_replacement_boundary() {
    let Some((dom, root)) = superdoc_redline(
        "behavior__math_radical_tests_4c1ce187.docx",
        "behavior__multi_image_types_b962a2b8.docx",
    ) else {
        return;
    };
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .find(|&p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .any(|t| dom.value(t).starts_with("All image types"))
        })
        .expect("last inserted content paragraph");
    assert!(
        dom.descendants(para, Some(&W::del_text())).is_empty(),
        "no deleted text folds into the inserted paragraph"
    );
}

/// Word records any paragraph property the revised document adds, not only
/// alignment or spacing: the paired final paragraph of diff_after8 ×
/// doc_with_spacing gains outline level 1 and keeps the original's bare
/// properties in `w:pPrChange`, so rejecting the redline restores them.
#[test]
fn added_paragraph_properties_are_recorded_and_rejected() {
    let Some((mut dom, root)) = superdoc_redline(
        "super_editor__diff_after8_58e5c288.docx",
        "super_editor__doc_with_spacing_e3d47bd7.docx",
    ) else {
        return;
    };
    let last = *dom.descendants(root, Some(&W::p())).last().unwrap();
    let ppr = dom.element(last, &W::p_pr()).expect("revised properties");
    assert!(
        dom.element(ppr, &W::p_pr_change()).is_some(),
        "pPrChange records the original's properties"
    );
    let rejected = reject_revisions_document(&mut dom, root);
    let last = *dom.descendants(rejected, Some(&W::p())).last().unwrap();
    let outline = dom
        .element(last, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::name("outlineLvl")));
    assert!(
        outline.is_none(),
        "rejecting restores the original's properties"
    );
}

/// A one-paragraph original replaced by a document that ends on an empty
/// paragraph (fields_attrs1 × cli_legacy sample): Word pairs the final marks
/// and deletes "bold Enter your full name sentence" into the revised empty
/// final paragraph; "And there can be empty pages:" keeps its own inserted
/// mark. Folding the deleted text into that paragraph instead dropped the
/// revised final paragraph, so accepting the redline lost it and Word's
/// fourth page (harness 23.4 vs Docxodus 73.1).
#[test]
fn one_paragraph_original_pairs_the_final_marks() {
    let Some((mut dom, root)) = superdoc_redline(
        "super_editor__fields_attrs1_83837249.docx",
        "cli_legacy__sample_3a8f1f93.docx",
    ) else {
        return;
    };
    let last = *dom.descendants(root, Some(&W::p())).last().unwrap();
    let deleted: String = dom
        .descendants(last, Some(&W::del_text()))
        .into_iter()
        .map(|t| dom.value(t))
        .collect();
    assert_eq!(deleted, "bold Enter your full name sentence");
    assert!(
        dom.descendants(last, Some(&W::t())).is_empty(),
        "the final paragraph holds only the deleted text"
    );
    let accepted = accept_revisions_document(&mut dom, root);
    let texts: Vec<String> = dom
        .descendants(accepted, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect()
        })
        .collect();
    assert_eq!(
        texts[texts.len() - 2..],
        ["And there can be empty pages:", ""],
        "accepting keeps the revised final empty paragraph"
    );
}
