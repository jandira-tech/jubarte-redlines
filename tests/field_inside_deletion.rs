// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A field inserted inside a deletion keeps its deleted field code.

mod common;

use common::docx::docx;
use jubarte::validate::validate;

#[test]
fn a_field_inserted_inside_a_deletion_is_not_a_state_mismatch() {
    let body = r#"<w:p><w:del w:id="1" w:author="A"><w:ins w:id="2" w:author="B"><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:delInstrText> PAGE </w:delInstrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:ins></w:del></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    assert!(
        !findings.iter().any(|f| f.code == "FIELD_CODE_STATE"),
        "deleted field code was reported: {findings:?}"
    );
}

/// A move destination inside a deletion is deleted the same way: validate's
/// repair (`under_deletion`) already writes its text as deleted text, so
/// the field-state check must agree with it.
#[test]
fn a_field_moved_to_inside_a_deletion_is_not_a_state_mismatch() {
    let body = r#"<w:p><w:del w:id="1" w:author="A"><w:moveTo w:id="2" w:author="B"><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:delInstrText> PAGE </w:delInstrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:moveTo></w:del></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    assert!(
        !findings.iter().any(|f| f.code == "FIELD_CODE_STATE"),
        "deleted field code was reported: {findings:?}"
    );
}
