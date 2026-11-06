// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A move between two insertions is not the direct nest Word was measured on.

mod common;

use common::docx::docx;
use jubarte::validate::validate;

#[test]
fn a_move_between_two_insertions_is_not_called_faithful() {
    let body = r#"<w:p><w:ins w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:moveFrom w:id="2" w:author="B" w:date="2026-01-01T00:00:00Z"><w:ins w:id="3" w:author="C" w:date="2026-01-01T00:00:00Z"><w:r><w:t>Nest words</w:t></w:r></w:ins></w:moveFrom></w:ins></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    let finding = findings
        .iter()
        .find(|f| f.code == "NESTED_SAME_REVISION")
        .expect("nested revision");
    assert!(
        !finding.message.contains("faithfully"),
        "the message claimed a faithful reading: {}",
        finding.message
    );
    assert_eq!(finding.severity, "warning", "{finding:?}");
    assert!(!finding.word_fatal && !finding.repairable, "{finding:?}");
    assert!(
        finding.message.contains("Word opens it"),
        "the message did not say Word opens it: {}",
        finding.message
    );
}
