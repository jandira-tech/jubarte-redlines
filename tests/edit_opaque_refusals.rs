// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! An edit refused for touching text it may not edit names what holds the
//! text (a revision by id and author, a hyperlink, a field, a content
//! control) and the way around it; following the advice works.

mod common;

use common::docx::docx;
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;

const LINK: &str = r#"<w:p><w:r><w:t xml:space="preserve">See </w:t></w:r><w:hyperlink w:anchor="terms"><w:r><w:t>the terms</w:t></w:r></w:hyperlink><w:r><w:t xml:space="preserve"> below.</w:t></w:r></w:p>"#;
const SIMPLE_FIELD: &str = r#"<w:p><w:r><w:t xml:space="preserve">Page </w:t></w:r><w:fldSimple w:instr=" PAGE "><w:r><w:t>3</w:t></w:r></w:fldSimple><w:r><w:t xml:space="preserve"> of the deed.</w:t></w:r></w:p>"#;
const COMPLEX_FIELD: &str = r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TITLE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>Master Agreement</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
const CONTROL: &str = r#"<w:p><w:r><w:t xml:space="preserve">Signed by </w:t></w:r><w:sdt><w:sdtPr><w:alias w:val="Signer"/></w:sdtPr><w:sdtContent><w:r><w:t>Ann Counsel</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t>.</w:t></w:r></w:p>"#;
const THEIRS: &str = r#"<w:p><w:r><w:t xml:space="preserve">Payment within </w:t></w:r><w:ins w:id="7" w:author="Trainer 1 - LargeCo" w:date="2026-09-01T00:00:00Z"><w:r><w:t>45</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> days.</w:t></w:r></w:p>"#;

fn source() -> Vec<u8> {
    docx(&[LINK, SIMPLE_FIELD, COMPLEX_FIELD, CONTROL, THEIRS].concat())
}

fn plan(extra: &str, operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","existing_revisions":"keep",{extra}"operations":{operations}}}"#
    ))
    .unwrap()
}

#[test]
fn a_refused_edit_names_what_holds_the_text_and_the_way_around_it() {
    for (op, says) in [
        (
            r#"{"kind":"replace","paragraph":"p0","find":"terms","replacement":"rules"}"#,
            "the text sits inside a hyperlink; edit the text on either side of it, or comment on it",
        ),
        (
            r#"{"kind":"insert","paragraph":"p2","position":"end","text":" (new)"}"#,
            "the insertion point sits inside the result of a field",
        ),
        (
            r#"{"kind":"replace","paragraph":"p1","find":"3","replacement":"4"}"#,
            r#"the text sits inside the result of the field "PAGE""#,
        ),
        (
            r#"{"kind":"replace","paragraph":"p2","find":"Master","replacement":"Main"}"#,
            "the text sits inside the result of a field",
        ),
        (
            r#"{"kind":"replace","paragraph":"p3","find":"Ann","replacement":"Bo"}"#,
            r#"the text sits inside a content control (alias "Signer"); change its text with fill_control"#,
        ),
        (
            r#"{"kind":"replace","paragraph":"p4","find":"45","replacement":"60"}"#,
            r#"the text sits inside body:rev:7, a tracked insertion by "Trainer 1 - LargeCo"; settle it first ("resolve_revisions": {"accept": {"ids": ["body:rev:7"]}}, or reject)"#,
        ),
    ] {
        let e = apply_plan(&source(), &plan("", &format!("[{op}]"))).unwrap_err();
        assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{op}: {e}");
        assert!(e.message.contains(says), "{op}: {e}");
    }
}

#[test]
fn settling_the_named_revision_frees_its_text() {
    let out = apply_plan(
        &source(),
        &plan(
            r#""resolve_revisions":{"accept":{"ids":["body:rev:7"]}},"#,
            r#"[{"kind":"replace","paragraph":"p4","find":"45","replacement":"60"}]"#,
        ),
    )
    .unwrap();
    let texts: Vec<String> = paragraphs(&out.clean)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(texts[4], "Payment within 60 days.");
}
