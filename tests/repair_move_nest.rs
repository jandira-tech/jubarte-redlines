// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A move source inside an insertion inside a deletion keeps deleted text.
//! Repairing that package twice must settle, with nothing left to report.

mod common;

use common::docx::{docx, part_string};
use jubarte::validate::{repair, validate};

#[test]
fn a_move_source_inside_an_insertion_inside_a_deletion_settles() {
    let body = r#"<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:ins w:id="2" w:author="B" w:date="2026-01-01T00:00:00Z"><w:moveFrom w:id="3" w:author="C" w:date="2026-01-01T00:00:00Z"><w:r><w:t>Moved words</w:t></w:r></w:moveFrom></w:ins></w:del></w:p>"#;
    let input = docx(body);
    let once = repair(&input).unwrap();
    let twice = repair(&once.docx).unwrap();
    assert_eq!(
        once.docx, twice.docx,
        "the second repair changed the package"
    );
    let findings = validate(&twice.docx).unwrap();
    assert!(findings.is_empty(), "a finding remained: {findings:?}");
    let xml = part_string(&twice.docx, "word/document.xml").unwrap();
    assert!(xml.contains("<w:delText>Moved words</w:delText>"), "{xml}");
}
