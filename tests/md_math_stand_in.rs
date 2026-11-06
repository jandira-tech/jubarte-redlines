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
