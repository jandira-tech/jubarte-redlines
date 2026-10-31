// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1826 — a paragraph between two deleted tables stays between them in Word
//! mode. Both bases are `tbl, ¶, tbl, ¶` and both revisions replace all of
//! it with new paragraphs. Word's redline keeps the base's middle paragraph
//! between the deleted tables, its mark deleted. jubarte moved it ahead of
//! the first table, folded into the inserted text, which left the tables
//! adjacent; rejecting that redline fused them into one table.
//!
//! - w06: `45c1c5abec_table_vmerge_colspan` → `a94f4c516b_text_box`.
//! - s278: `6867e05e76_00b0…` → `1424386e9b_0174…`.
//!
//! #1926 — that keep-in-place rule must not fire when the replaced region
//! holds a single deleted table. Word writes every inserted block first and
//! the deleted table after them. These two ladder pairs regressed to an
//! inserted paragraph after the table:
//!
//! - `employee_directory_table` → `employee_review_john_smith_suggesting_insertions`.
//! - `meeting_agenda_table` → `meeting_minutes_id_paraid_overflow`.

mod common;

use std::path::PathBuf;

use common::validity::assert_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::Dom;

fn fixture(name: &str) -> Vec<u8> {
    read_corpus(
        "tests/corpus/neurotic_docx_bench/corpus/word/clean/docx",
        name,
    )
}

fn issue(name: &str) -> Vec<u8> {
    read_corpus("tests/corpus/issue_1926", name)
}

fn read_corpus(dir: &str, name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(dir)
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The body's children, `tbl` or `p`, the sectPr left out.
fn skeleton(docx: &[u8]) -> Vec<String> {
    let pkg = PartFs::open(docx).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    dom.elements(body, None)
        .into_iter()
        .filter_map(|k| dom.name(k).map(|n| n.local_name().to_string()))
        .filter(|n| n != "sectPr")
        .collect()
}

/// The redline in `settings`'s mode, with its accept and reject.
fn round_trip(a: &[u8], b: &[u8], settings: &WmlComparerSettings) -> [Vec<String>; 3] {
    let redline = compare_documents_with_settings(a, b, settings).expect("compare");
    assert_word_valid_package(&redline);
    let accepted = accept_revisions(&redline).unwrap();
    let rejected = reject_revisions(&redline).unwrap();
    [skeleton(&redline), skeleton(&accepted), skeleton(&rejected)]
}

fn assert_keeps_the_paragraph_between_the_tables(a: &str, b: &str) {
    let (a, b) = (fixture(a), fixture(b));
    assert_eq!(skeleton(&a), ["tbl", "p", "tbl", "p"], "the base");
    for (mode, settings) in [
        ("word", WmlComparerSettings::default()),
        ("powertools", WmlComparerSettings::powertools_faithful()),
    ] {
        let [redline, accepted, rejected] = round_trip(&a, &b, &settings);
        let tables: Vec<usize> = redline
            .iter()
            .enumerate()
            .filter(|(_, n)| *n == "tbl")
            .map(|(i, _)| i)
            .collect();
        assert_eq!(tables.len(), 2, "{mode}: {redline:?}");
        assert_eq!(
            redline[tables[0] + 1],
            "p",
            "{mode}: a paragraph stays between the deleted tables: {redline:?}"
        );
        assert_eq!(rejected, skeleton(&a), "{mode}: reject gives the base");
        assert_eq!(accepted, skeleton(&b), "{mode}: accept gives the revision");
    }
}

#[test]
fn w06_keeps_the_paragraph_between_its_deleted_tables() {
    assert_keeps_the_paragraph_between_the_tables(
        "45c1c5abec_table_vmerge_colspan.docx",
        "a94f4c516b_text_box.docx",
    );
}

#[test]
fn s278_keeps_the_paragraph_between_its_deleted_tables() {
    assert_keeps_the_paragraph_between_the_tables(
        "6867e05e76_00b0d42bbea3b9bac7bc95466cf416f6c89a8cda62bbf3d7.docx",
        "1424386e9b_017447de83753a0e3b7afc659df038ef56972925336c6a5b.docx",
    );
}

/// Body-block roles, `sectPr` left out. `ins` is a paragraph that holds
/// inserted words. `del-tbl` is a table whose every row is deleted.
fn roles(docx: &[u8]) -> Vec<&'static str> {
    let pkg = PartFs::open(docx).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    dom.elements(body, None)
        .into_iter()
        .filter_map(|block| {
            if dom.name_is(block, &W::sect_pr()) {
                return None;
            }
            if dom.name_is(block, &W::tbl()) {
                let rows = dom.elements(block, Some(&W::tr()));
                let deleted = !rows.is_empty()
                    && rows.iter().all(|&row| {
                        dom.element(row, &W::tr_pr())
                            .is_some_and(|pr| dom.element(pr, &W::del()).is_some())
                    });
                return Some(if deleted { "del-tbl" } else { "tbl" });
            }
            if dom.name_is(block, &W::p()) {
                let inserted = dom.descendants(block, Some(&W::ins())).iter().any(|&ins| {
                    dom.descendants(ins, Some(&W::t()))
                        .iter()
                        .any(|&text| !dom.value_str(text).is_empty())
                });
                return Some(if inserted { "ins" } else { "p" });
            }
            None
        })
        .collect()
}

/// Visible body text, table cells included, deletions left out.
fn texts(docx: &[u8]) -> Vec<String> {
    jubarte::inspect::paragraphs(docx)
        .expect("paragraphs")
        .into_iter()
        .map(|paragraph| paragraph.text)
        .collect()
}

/// Word mode: every inserted block precedes the one deleted table, in
/// Word's order, and accept/reject give the two sources' text back.
fn assert_single_table_keeps_words_order(original: &str, revision: &str, word: &str) {
    let (original, revision, word) = (issue(original), issue(revision), issue(word));
    let redline =
        compare_documents_with_settings(&original, &revision, &WmlComparerSettings::default())
            .expect("compare");
    let accepted = accept_revisions(&redline).unwrap();
    let rejected = reject_revisions(&redline).unwrap();
    assert_eq!(
        texts(&accepted),
        texts(&revision),
        "accept gives the revision"
    );
    assert_eq!(
        texts(&rejected),
        texts(&original),
        "reject gives the original"
    );
    let ours = roles(&redline);
    let word = roles(&word);
    let table = ours
        .iter()
        .position(|role| *role == "del-tbl")
        .expect("a deleted table");
    assert!(
        ours[table + 1..].iter().all(|role| *role != "ins"),
        "an inserted paragraph follows the deleted table\n  ours: {ours:?}\n  word: {word:?}"
    );
    assert_eq!(ours, word, "body order matches Word's redline");
}

#[test]
fn directory_review_keeps_every_insertion_before_the_deleted_table() {
    assert_single_table_keeps_words_order(
        "directory.docx",
        "review.docx",
        "directory_review_word.docx",
    );
}

#[test]
fn agenda_minutes_keeps_every_insertion_before_the_deleted_table() {
    assert_single_table_keeps_words_order(
        "agenda.docx",
        "minutes.docx",
        "agenda_minutes_word.docx",
    );
}
