// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `audit_tracked` compares another author's formatting history by what it
//! covers and what it recorded, not by revision id (#1656).
//!
//! A run Alice formatted, split into touching fragments that each carry
//! her record, is the history she wrote. A changed old property, author or
//! date, a fragment that lost its record, swapped records, or a record
//! moved to another paragraph is a `FOREIGN_AUTHOR` finding.
mod common;

use common::docx::docx;
use jubarte::validate::{Finding, audit_tracked};

const OLD: &str = "2026-10-17T09:00:00Z";
const BOB: &str = "Bob";

/// A run that is bold now; Alice's record holds the old properties `old`.
fn run(id: u32, author: &str, date: &str, old: &str, text: &str) -> String {
    format!(
        r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="{author}" w:date="{date}"><w:rPr>{old}</w:rPr></w:rPrChange></w:rPr><w:t xml:space="preserve">{text}</w:t></w:r>"#
    )
}

fn alice(id: u32, text: &str) -> String {
    run(id, "Alice", OLD, "<w:i/>", text)
}

fn plain(text: &str) -> String {
    format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
}

fn para(runs: &str) -> String {
    format!("<w:p>{runs}</w:p>")
}

fn foreign(original: &str, edited: &str) -> Vec<Finding> {
    audit_tracked(&docx(original), &docx(edited), BOB)
        .unwrap()
        .into_iter()
        .filter(|f| f.code == "FOREIGN_AUTHOR")
        .collect()
}

fn assert_same_history(original: &str, edited: &str) {
    let found = foreign(original, edited);
    assert!(found.is_empty(), "{found:#?}");
}

fn assert_changed_history(original: &str, edited: &str) {
    assert!(
        !foreign(original, edited).is_empty(),
        "a changed history must be reported"
    );
}

fn whole() -> String {
    para(&alice(5, "alpha beta gamma"))
}

#[test]
fn touching_fragments_with_one_record_are_the_same_history() {
    let split = para(&[alice(5, "alpha "), alice(9, "beta"), alice(10, " gamma")].concat());
    assert_same_history(&whole(), &split);
}

#[test]
fn fragments_inside_split_insertions_are_the_same_history() {
    let original = para(&format!(
        r#"<w:ins w:id="6" w:author="Alice" w:date="{OLD}">{}</w:ins>"#,
        alice(7, "alpha beta gamma")
    ));
    let edited = para(&format!(
        r#"<w:ins w:id="6" w:author="Alice" w:date="{OLD}">{}</w:ins><w:ins w:id="11" w:author="Alice" w:date="{OLD}">{}</w:ins>"#,
        alice(7, "alpha "),
        alice(12, "beta gamma")
    ));
    assert_same_history(&original, &edited);
}

#[test]
fn a_prefix_spelling_is_not_a_different_payload() {
    let respelled = format!(
        r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="9" w:author="Alice" w:date="{OLD}"><x:rPr xmlns:x="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><x:i/></x:rPr></w:rPrChange></w:rPr><w:t xml:space="preserve">beta gamma</w:t></w:r>"#
    );
    let split = para(&[alice(5, "alpha "), respelled].concat());
    assert_same_history(&whole(), &split);
}

#[test]
fn unchanged_history_beside_unchanged_text_is_the_same_history() {
    let original = para(&[plain("["), alice(5, "alpha"), plain("]")].concat());
    assert_same_history(&original, &original);
}

#[test]
fn a_changed_old_property_is_reported() {
    let edited = para(&run(
        5,
        "Alice",
        OLD,
        "<w:u w:val=\"single\"/>",
        "alpha beta gamma",
    ));
    assert_changed_history(&whole(), &edited);
}

#[test]
fn a_changed_old_property_on_one_fragment_is_reported() {
    let edited = para(
        &[
            alice(5, "alpha "),
            run(9, "Alice", OLD, "<w:u w:val=\"single\"/>", "beta gamma"),
        ]
        .concat(),
    );
    assert_changed_history(&whole(), &edited);
}

#[test]
fn a_changed_author_is_reported() {
    let edited = para(
        &[
            alice(5, "alpha "),
            run(9, "Carol", OLD, "<w:i/>", "beta gamma"),
        ]
        .concat(),
    );
    assert_changed_history(&whole(), &edited);
}

#[test]
fn a_changed_date_is_reported() {
    let edited = para(
        &[
            alice(5, "alpha "),
            run(9, "Alice", "2026-10-18T09:00:00Z", "<w:i/>", "beta gamma"),
        ]
        .concat(),
    );
    assert_changed_history(&whole(), &edited);
}

#[test]
fn a_fragment_that_lost_its_record_is_reported() {
    let edited = para(&[alice(5, "alpha "), plain("beta"), alice(9, " gamma")].concat());
    assert_changed_history(&whole(), &edited);
}

#[test]
fn swapped_records_are_reported() {
    let original = para(
        &[
            run(5, "Alice", OLD, "<w:i/>", "alpha "),
            run(6, "Alice", OLD, "<w:u w:val=\"single\"/>", "beta"),
        ]
        .concat(),
    );
    let edited = para(
        &[
            run(5, "Alice", OLD, "<w:u w:val=\"single\"/>", "alpha "),
            run(6, "Alice", OLD, "<w:i/>", "beta"),
        ]
        .concat(),
    );
    assert_changed_history(&original, &edited);
}

#[test]
fn a_record_moved_to_an_identical_paragraph_is_reported() {
    let original = [para(&alice(5, "alpha")), para(&plain("alpha"))].concat();
    let edited = [para(&plain("alpha")), para(&alice(5, "alpha"))].concat();
    assert_changed_history(&original, &edited);
}

#[test]
fn a_record_spread_over_a_neighbouring_paragraph_is_reported() {
    let original = [para(&alice(5, "alpha")), para(&plain("beta"))].concat();
    let edited = [para(&alice(5, "alpha")), para(&alice(9, "beta"))].concat();
    assert_changed_history(&original, &edited);
}

#[test]
fn a_record_spread_over_unrelated_text_is_reported() {
    let original = para(&[alice(5, "alpha"), plain(" beta")].concat());
    let edited = para(&[alice(5, "alpha"), alice(9, " beta")].concat());
    assert_changed_history(&original, &edited);
}
