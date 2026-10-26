// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1868: on a clean file, Accept All of an edit's redline is the clean copy
//! the plan asked for. The redline tracks the edit, as Word's Track Changes
//! would, and changes nothing else: no style renamed or merged, no spacing
//! stripped or written out, no hyperlink unwrapped.

use std::io::Read as _;

use jubarte::debug::{Check, Options, report};
use jubarte::document_comparer::accept_revisions;
use jubarte::edit::{EditPlan, apply_plan};

/// A bench fixture under `tests/corpus`, by file name.
fn bench_clean(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "tests/corpus/neurotic_docx_bench/corpus/word/clean/docx/{name}"
    ))
    .unwrap()
}

/// `jubarte debug A B --check runs --check styledefs`: one `… identical`
/// line per check when `a` and `b` agree.
fn differences(a: &[u8], b: &[u8]) -> String {
    report(
        a,
        Some(b),
        &Options {
            checks: vec![Check::Runs, Check::StyleDefs],
            ..Options::default()
        },
    )
    .unwrap()
}

/// How many `w:hyperlink`s the body holds.
fn hyperlinks(docx: &[u8]) -> usize {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml.matches("<w:hyperlink ").count()
}

/// Apply the plan made of `ops` to `source`, then check that the redline
/// accepts to the clean copy and keeps every hyperlink.
fn assert_plan_accepts_to_clean(source: &[u8], ops: &str, label: &str) {
    let edited = apply_plan(
        source,
        &EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"Probe","operations":[{ops}]}}"#
        ))
        .unwrap(),
    )
    .unwrap();
    let accepted = accept_revisions(&edited.redline).unwrap();
    assert_eq!(
        differences(&accepted, &edited.clean),
        "runs identical\nstyledefs identical\n",
        "{label}: {ops}"
    );
    assert_eq!(
        hyperlinks(&edited.redline),
        hyperlinks(source),
        "{label}: {ops}"
    );
}

/// [`assert_plan_accepts_to_clean`] on the bench fixture `name`.
fn assert_accepts_to_clean(name: &str, op: &str) {
    assert_plan_accepts_to_clean(&bench_clean(name), op, name);
}

/// Through the comparer, the redline of a new paragraph renamed
/// `0b2a46481b`'s `style20` to `TableContents`, stripped `0018843794`'s
/// `spacing line=276` from every body paragraph, and renamed `8441c4ae21`'s
/// localized `Sumrio1` to `TOC1`, unwrapping its 133 TOC hyperlinks. It
/// wrote `lineRule=auto` on `00da16e738`'s closing heading, and dropped
/// the new paragraph's `rtl=0` on `2f23773d63`.
#[test]
fn a_new_paragraph_accepts_to_the_clean_copy() {
    for name in [
        "0b2a46481b_nested_table_rowspan.docx",
        "0018843794_889f4c3958e28978a0d96a95a8938d9eb0dc100e66f7c483.docx",
        "8441c4ae21_sd_2517_localized_heading_styles.docx",
        "00da16e738_heading_4_style_demo_id_paraid_overflow_2.docx",
        "2f23773d63_super_editor__pagination_blank_2a98ed7a.docx",
    ] {
        assert_accepts_to_clean(
            name,
            r#"{"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"Probe paragraph."}]}"#,
        );
    }
}

/// A one-word replacement goes through the comparer too, and must leave the
/// rest of the document as the source has it.
#[test]
fn a_replaced_word_accepts_to_the_clean_copy() {
    for (name, op) in [
        (
            "0b2a46481b_nested_table_rowspan.docx",
            r#"{"kind":"replace","paragraph":{"starts_with":"AAA"},"find":"AAA","replacement":"AAX"}"#,
        ),
        (
            "0018843794_889f4c3958e28978a0d96a95a8938d9eb0dc100e66f7c483.docx",
            r#"{"kind":"replace","paragraph":{"starts_with":"Proposal Number"},"find":"Number","replacement":"Code"}"#,
        ),
        (
            "8441c4ae21_sd_2517_localized_heading_styles.docx",
            r#"{"kind":"replace","paragraph":{"starts_with":"dolor 10"},"find":"10","replacement":"11"}"#,
        ),
    ] {
        assert_accepts_to_clean(name, op);
    }
}
