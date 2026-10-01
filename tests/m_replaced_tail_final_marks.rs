// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A kept paragraph over a rewritten tail: Word inserts the revised tail,
//! deletes the original's, and pairs the two stories' final marks, so the
//! last inserted paragraph joins the first deleted one under its deleted
//! mark. Word 16 probes, 2026-10-01 (tests/fixtures/word_probes/final_marks;
//! `*_word_redline.docx` is Word's own Compare of the pair):
//! - `kept_title`, `kept_title3`: one kept title over 2 and 3 unrelated
//!   paragraphs;
//! - `kept_junk`: 40 unrelated paragraphs around one shared `(dolore)`
//!   paragraph, which Word keeps as an anchor;
//! - `table_end`: three paragraphs replaced by a nested table and the empty
//!   paragraph after it, which pairs with the original's closing mark.
//!
//! The tail came out of the LCS deleted-first, and neither final-mark pass
//! took that order: the first deleted paragraph's mark stayed live, and
//! accepting the redline kept an empty paragraph the revision never had.

mod common;

use common::docx::part_string;
use jubarte::changes::list_changes;
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};

const PROBES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/word_probes/final_marks"
);

/// The changes of a redline, in document order:
/// `Insertion paragraph_mark ""`, `Deletion text "consectetur …"`.
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

fn probe(name: &str) {
    let read =
        |suffix: &str| std::fs::read(format!("{PROBES}/{name}_{suffix}.docx")).expect("probe");
    let (a, b) = (read("a"), read("b"));
    let ours = compare_documents(&a, &b, "Comparison").expect("compare");
    assert_eq!(changes(&ours), changes(&read("word_redline")), "{name}");
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
fn a_kept_title_over_a_rewritten_tail_pairs_the_final_marks() {
    probe("kept_title");
}

#[test]
fn a_kept_title_over_a_longer_rewritten_tail_pairs_the_final_marks() {
    probe("kept_title3");
}

#[test]
fn a_shared_paragraph_anchors_two_rewritten_halves() {
    probe("kept_junk");
}

#[test]
fn a_revision_ending_after_a_table_pairs_its_empty_paragraph_like_word() {
    probe("table_end");
}
