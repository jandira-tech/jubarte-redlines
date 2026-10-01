// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A whole story replaced still pairs the two stories' final marks. Word
//! inserts the revision, then deletes the original, and the revision's last
//! words join the original's first deleted paragraph under its deleted mark;
//! the original's closing mark stands for the revised one. In 1053 Word 16
//! redlines of the bench corpus (corpus/word/tracking_without_comments,
//! no tables, the revision ending on text) the revision's last inserted
//! paragraph never kept an inserted mark. Unpaired, accepting our redline
//! kept an empty paragraph the revision never had (189 of 2472 corpus
//! pairs).
//!
//! Tables bound the rule: a revision ending on an empty paragraph after a
//! table pairs that paragraph's mark with the original's closing mark, while
//! an original ending that way keeps its empty paragraph live after the
//! deleted table and the revision's last paragraph on its inserted mark
//! (Word's nested_table_rowspan × numbered_list).

mod common;

use common::docx::{docx, para, part_string};
use jubarte::changes::list_changes;
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};

const WORD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/corpus/neurotic_docx_bench/corpus/word"
);

/// The changes of a redline, in document order:
/// `Insertion text "One two"`, `Deletion paragraph_mark ""`.
fn changes(docx: &[u8]) -> Vec<String> {
    list_changes(docx)
        .expect("list changes")
        .into_iter()
        .map(|c| format!("{:?} {} {:?}", c.kind, c.target, c.text))
        .collect()
}

fn paragraphs(docx: &[u8]) -> usize {
    let xml = part_string(docx, "word/document.xml").expect("document part");
    xml.matches("<w:p>").count() + xml.matches("<w:p ").count() + xml.matches("<w:p/>").count()
}

/// Our redline of a clean corpus pair, beside Word's own.
fn against_word(a: &str, b: &str, redline: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let read = |path: String| std::fs::read(path).expect("corpus fixture");
    let (a, b) = (
        read(format!("{WORD}/clean/docx/{a}")),
        read(format!("{WORD}/clean/docx/{b}")),
    );
    let word = read(format!("{WORD}/tracking_without_comments/docx/{redline}"));
    let ours = compare_documents(&a, &b, "Comparison").expect("compare");
    assert_eq!(changes(&ours), changes(&word));
    (a, b, ours)
}

#[test]
fn a_whole_story_replaced_joins_the_revised_last_words_to_the_first_deletion() {
    let (a, b, ours) = against_word(
        "b5a6dd6c66_super_editor__diff_before_019353a2.docx",
        "2ad2fc5ab0_super_editor__diff_before10_2f84ca55.docx",
        "b5a6dd6c66_super_editor__diff_before_019353a2__vs__2ad2fc5ab0_super_editor__diff_before10_2f84ca55_redline_585342dcd0.docx",
    );
    assert_eq!(
        paragraphs(&accept_revisions(&ours).expect("accept")),
        paragraphs(&b)
    );
    assert_eq!(
        paragraphs(&reject_revisions(&ours).expect("reject")),
        paragraphs(&a)
    );
}

#[test]
fn an_original_ending_after_a_table_keeps_its_empty_paragraph_live() {
    against_word(
        "0b2a46481b_nested_table_rowspan.docx",
        "bdd82b239b_numbered_list_demo_id_paraid_overflow.docx",
        "0b2a46481b_nested_table_rowspan__vs__bdd82b239b_numbered_list_demo_id_paraid_overflow_redline_c053ccd3b8.docx",
    );
}

#[test]
fn a_revision_ending_after_a_table_pairs_its_empty_paragraph() {
    let table = |inner: &str| {
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>\
             <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid><w:tr><w:tc><w:tcPr>\
             <w:tcW w:w=\"4000\" w:type=\"dxa\"/></w:tcPr>{inner}</w:tc></w:tr></w:tbl>"
        )
    };
    let a = docx(&(para("square root of x") + &para("cube root of three x") + &para("x")));
    let b = docx(&(table(&(table(&para("CCC")) + "<w:p/>")) + "<w:p/>"));
    let ours = compare_documents(&a, &b, "Comparison").expect("compare");
    let xml = part_string(&ours, "word/document.xml").expect("document part");
    let body = &xml[..xml.rfind("<w:sectPr").expect("section")];
    assert!(
        body.trim_end().ends_with("</w:p>"),
        "the story still ends on its paragraph"
    );
    assert_eq!(
        paragraphs(&accept_revisions(&ours).expect("accept")),
        paragraphs(&b)
    );
    assert_eq!(
        paragraphs(&reject_revisions(&ours).expect("reject")),
        paragraphs(&a)
    );
}
