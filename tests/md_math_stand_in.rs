// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A literal stand-in inside a dollar formula stays math when CriticMarkup is off.

mod common;

use jubarte::markdown::{DocxOptions, markdown_to_docx};

#[test]
fn a_literal_stand_in_inside_dollars_stays_math_when_critic_is_off() {
    let options = DocxOptions {
        critic: false,
        ..DocxOptions::default()
    };
    for formula in ["$a\u{2E0E}b$\n", "$a\u{2E2E}b$\n"] {
        let docx = markdown_to_docx(formula, &options).expect("write").docx;
        let xml = common::docx::part_string(&docx, "word/document.xml").unwrap();
        assert!(
            xml.contains("<m:oMath"),
            "stand-in formula became text: {formula} {xml}"
        );
    }
}

/// The stand-in comes out as the formula's own character: no revision
/// mark, nothing lost, whichever way CriticMarkup is set.
#[test]
fn a_literal_stand_in_inside_a_formula_is_its_character() {
    for critic in [false, true] {
        let options = DocxOptions {
            critic,
            ..DocxOptions::default()
        };
        for marker in ['\u{2E0E}', '\u{2E0F}', '\u{2E10}', '\u{2E11}'] {
            let formula = format!("$a{marker}b$\n");
            let docx = markdown_to_docx(&formula, &options).expect("write").docx;
            let xml = common::docx::part_string(&docx, "word/document.xml").unwrap();
            assert!(
                xml.contains(marker) && !xml.contains("<w:ins") && !xml.contains("<w:del"),
                "critic {critic}, {marker:?}: {xml}"
            );
        }
    }
}

/// In the document's title a formula is text: a literal stand-in stays
/// itself, it does not turn into CriticMarkup.
#[test]
fn a_literal_stand_in_in_a_title_formula_stays_itself() {
    let options = DocxOptions {
        critic: false,
        ..DocxOptions::default()
    };
    let docx = markdown_to_docx("---\ntitle: \"$a\u{2E0E}b$\"\n---\n\nBody\n", &options)
        .expect("write")
        .docx;
    let core = common::docx::part_string(&docx, "docProps/core.xml").unwrap_or_default();
    assert!(!core.contains("{++"), "{core}");
}

/// In alt text a formula is text: a literal stand-in stays itself.
#[test]
fn a_literal_stand_in_in_an_alt_formula_stays_itself() {
    for critic in [false, true] {
        let options = DocxOptions {
            critic,
            ..DocxOptions::default()
        };
        let docx = markdown_to_docx("![x $a\u{2E0E}b$ y](missing.png)\n", &options)
            .expect("write")
            .docx;
        let xml = common::docx::part_string(&docx, "word/document.xml").unwrap();
        assert!(!xml.contains("{++"), "critic {critic}: {xml}");
    }
}

/// Outside math too, CriticMarkup off reads a stand-in as the character.
#[test]
fn a_literal_stand_in_in_body_text_stays_itself_when_critic_is_off() {
    let options = DocxOptions {
        critic: false,
        ..DocxOptions::default()
    };
    let docx = markdown_to_docx("a\u{2E0E}b\u{2E0F}c\n", &options)
        .expect("write")
        .docx;
    let xml = common::docx::part_string(&docx, "word/document.xml").unwrap();
    assert!(xml.contains('\u{2E0E}') && !xml.contains("<w:ins"), "{xml}");
}
