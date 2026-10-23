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

use common::docx::{Part, docx, docx_with_sect};
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

#[test]
fn a_record_moved_to_the_same_words_later_in_the_paragraph_is_reported() {
    // Paragraph text `abab` stays; Alice's record moves from the first `ab`
    // to the second, split in two touching fragments.
    let original = para(&[alice(5, "ab"), plain("ab")].concat());
    let edited = para(&[plain("ab"), alice(5, "a"), alice(9, "b")].concat());
    assert_changed_history(&original, &edited);
}

/// A run that is bold now; Alice's record holds `old`, whose element names
/// may use any namespace the run's ancestors or `old` itself declare.
fn alice_recorded(id: u32, old: &str, text: &str) -> String {
    run(id, "Alice", OLD, old, text)
}

const W14: &str = "http://schemas.microsoft.com/office/word/2010/wordml";
const W15: &str = "http://schemas.microsoft.com/office/word/2012/wordml";

#[test]
fn an_attribute_value_cannot_spell_a_second_attribute() {
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let one = format!(r#"<w:rFonts w:ascii="Arial,{{{w}}}hAnsi=Calibri"/>"#);
    let two = r#"<w:rFonts w:ascii="Arial" w:hAnsi="Calibri"/>"#;
    let original = para(&alice_recorded(5, &one, "alpha beta gamma"));
    let edited = para(
        &[
            alice_recorded(5, &one, "alpha "),
            alice_recorded(9, two, "beta gamma"),
        ]
        .concat(),
    );
    assert_changed_history(&original, &edited);
}

/// Alice's record keeps bold under `w14`, italic under `w15`, first match
/// wins: `swapped` lists the italic Choice first.
fn choices(swapped: bool) -> String {
    let (bold, italic) = (
        r#"<mc:Choice Requires="w14"><w:b/></mc:Choice>"#,
        r#"<mc:Choice Requires="w15"><w:i/></mc:Choice>"#,
    );
    let (first, second) = if swapped {
        (italic, bold)
    } else {
        (bold, italic)
    };
    format!(
        r#"<mc:AlternateContent xmlns:w14="{W14}" xmlns:w15="{W15}">{first}{second}<mc:Fallback/></mc:AlternateContent>"#
    )
}

#[test]
fn the_order_of_recorded_choices_is_kept() {
    let original = para(&alice_recorded(5, &choices(false), "alpha beta gamma"));
    let same = para(
        &[
            alice_recorded(5, &choices(false), "alpha "),
            alice_recorded(9, &choices(false), "beta gamma"),
        ]
        .concat(),
    );
    assert_same_history(&original, &same);
    let swapped = para(
        &[
            alice_recorded(5, &choices(false), "alpha "),
            alice_recorded(9, &choices(true), "beta gamma"),
        ]
        .concat(),
    );
    assert_changed_history(&original, &swapped);
}

fn required(prefix: &str, namespace: &str) -> String {
    format!(
        r#"<mc:AlternateContent xmlns:{prefix}="{namespace}"><mc:Choice Requires="{prefix}"><w:b/></mc:Choice><mc:Fallback><w:i/></mc:Fallback></mc:AlternateContent>"#
    )
}

#[test]
fn a_required_prefix_is_compared_by_its_namespace() {
    let original = para(&alice_recorded(5, &required("x", W14), "alpha beta gamma"));
    let respelled = para(
        &[
            alice_recorded(5, &required("x", W14), "alpha "),
            alice_recorded(9, &required("y", W14), "beta gamma"),
        ]
        .concat(),
    );
    assert_same_history(&original, &respelled);
    let rebound = para(
        &[
            alice_recorded(5, &required("x", W14), "alpha "),
            alice_recorded(9, &required("x", "urn:unsupported"), "beta gamma"),
        ]
        .concat(),
    );
    assert_changed_history(&original, &rebound);
}

#[test]
fn a_record_spread_over_a_neighbouring_drawing_is_reported() {
    // Runs with no text still cover something: a tab, a drawing.
    let tab = |id: u32| {
        format!(
            r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="Alice" w:date="{OLD}"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:tab/></w:r>"#
        )
    };
    let drawing = |record: &str| format!(r#"<w:r>{record}<w:drawing/></w:r>"#);
    let original = para(&[tab(5), drawing("")].concat());
    let recorded = format!(
        r#"<w:rPr><w:b/><w:rPrChange w:id="9" w:author="Alice" w:date="{OLD}"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr>"#
    );
    let edited = para(&[tab(5), drawing(&recorded)].concat());
    assert_changed_history(&original, &edited);
}

#[test]
fn an_on_off_spelling_is_not_a_different_payload() {
    let split = para(
        &[
            alice(5, "alpha "),
            run(9, "Alice", OLD, r#"<w:i w:val="true"/>"#, "beta gamma"),
        ]
        .concat(),
    );
    assert_same_history(&whole(), &split);
    let off = para(
        &[
            alice(5, "alpha "),
            run(9, "Alice", OLD, r#"<w:i w:val="false"/>"#, "beta gamma"),
        ]
        .concat(),
    );
    assert_changed_history(&whole(), &off);
}

/// A centred paragraph Alice once had `jc` before.
fn aligned(jc: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="6" w:author="Alice" w:date="{OLD}"><w:pPr><w:jc w:val="{jc}"/></w:pPr></w:pPrChange></w:pPr><w:r><w:t>alpha</w:t></w:r></w:p>"#
    )
}

#[test]
fn a_changed_paragraph_property_record_is_reported() {
    assert_same_history(&aligned("left"), &aligned("left"));
    assert_changed_history(&aligned("left"), &aligned("right"));
}

/// A document whose header holds `header`, one paragraph's runs.
fn with_header(header: &str) -> Vec<u8> {
    let xml = format!(
        r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p>{header}</w:p></w:hdr>"#
    );
    docx_with_sect(
        &para(&plain("body")),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &xml,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

#[test]
fn a_header_record_is_compared_by_coverage_too() {
    let original = with_header(&[alice(5, "ab"), plain("ab")].concat());
    let split = with_header(&[alice(5, "a"), alice(9, "b"), plain("ab")].concat());
    let moved = with_header(&[plain("ab"), alice(5, "a"), alice(9, "b")].concat());
    let foreign = |edited: &[u8]| -> Vec<Finding> {
        audit_tracked(&original, edited, BOB)
            .unwrap()
            .into_iter()
            .filter(|f| f.code == "FOREIGN_AUTHOR")
            .collect()
    };
    assert_eq!(foreign(&split), []);
    assert!(
        !foreign(&moved).is_empty(),
        "a moved header record is reported"
    );
}
