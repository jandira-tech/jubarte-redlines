// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! An edit in every paragraph leaves no paragraph whole on either side, yet
//! the two documents are one revision: Word marks each changed word in its
//! own paragraph, however long the document. Word 16 probes, 2026-10-01
//! (tests/fixtures/word_probes/every_paragraph; `*_word_redline.docx` is
//! Word's own Compare of the pair):
//! - `case28`, `case40`: `Case N … xNy …` → `xNz`, 2 revisions a paragraph;
//! - `seed_word`: 27 lorem paragraphs whose seed word `alpha` becomes `zulu`
//!   around one shared `(dolore)` paragraph, 6 revisions a paragraph.
//!
//! The unrelated-sources shortcut and the LCS detail threshold measured the
//! longest common run against the whole document, so past 27 paragraphs
//! the revision was inserted whole and the original deleted whole.

mod common;

use common::docx::{docx, para};
use jubarte::changes::list_changes;
use jubarte::document_comparer::compare_documents;

const PROBES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/word_probes/every_paragraph"
);

/// The changes of a redline, in document order:
/// `Insertion paragraph_mark ""`, `Insertion text "x0z"`.
fn changes(docx: &[u8]) -> Vec<String> {
    list_changes(docx)
        .expect("list changes")
        .into_iter()
        .map(|c| format!("{:?} {} {:?}", c.kind, c.target, c.text))
        .collect()
}

fn probe(name: &str) {
    let read =
        |suffix: &str| std::fs::read(format!("{PROBES}/{name}_{suffix}.docx")).expect("probe");
    let ours = compare_documents(&read("a"), &read("b"), "Comparison").expect("compare");
    assert_eq!(changes(&ours), changes(&read("word_redline")), "{name}");
}

#[test]
fn a_change_in_every_paragraph_stays_in_its_paragraph() {
    probe("case28");
}

#[test]
fn a_change_in_every_paragraph_stays_put_in_a_longer_story() {
    probe("case40");
}

#[test]
fn a_seed_word_replaced_in_every_paragraph_is_marked_word_by_word() {
    probe("seed_word");
}

#[test]
fn a_defined_term_renamed_throughout_is_marked_word_by_word() {
    let clause = |term: &str, i: usize| {
        para(&format!(
            "{i}. The {term} shall deliver the Goods within {i} days, and the Buyer \
             acknowledges that the {term} remains liable under section {}.",
            i + 1
        ))
    };
    let side = |term: &str| docx(&(1..=60).map(|i| clause(term, i)).collect::<String>());
    let ours = compare_documents(&side("Seller"), &side("Vendor"), "Comparison").expect("compare");
    let changes = changes(&ours);
    assert_eq!(changes.len(), 240, "{changes:#?}");
    assert!(
        changes
            .iter()
            .all(|c| c.ends_with("text \"Seller\"") || c.ends_with("text \"Vendor\"")),
        "{changes:#?}"
    );
}
