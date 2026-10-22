// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//! A substitution has one date slot, so unequal dates remain separate marks.
mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

fn dated_pair(old: Option<&str>, new: Option<&str>) -> Vec<u8> {
    let revision = |kind: &str, id: usize, text: &str, date: Option<&str>| {
        let date = date.map_or(String::new(), |date| format!(r#" w:date="{date}""#));
        let node = if kind == "del" { "delText" } else { "t" };
        format!(
            r#"<w:{kind} w:id="{id}" w:author="Ann Lee"{date}><w:r><w:{node}>{text}</w:{node}></w:r></w:{kind}>"#
        )
    };
    common::docx::docx(&format!(
        "<w:p>{}{}</w:p>",
        revision("del", 1, "old", old),
        revision("ins", 2, "new", new)
    ))
}

fn view(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            dates: true,
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .unwrap()
    .markdown
}

#[test]
fn unequal_inline_dates_stay_on_two_marks_and_round_trip() {
    let old = "2026-10-01T09:00:00Z";
    let new = "2026-10-02T10:00:00Z";
    let before = view(&dated_pair(Some(old), Some(new)));
    assert!(
        before.contains(&format!(
            "{{--old--}}{{r1+2 @AL {old}}}{{++new++}}{{r1+2 @AL {new}}}"
        )),
        "{before}"
    );
    assert!(
        before.contains("2 marks shown as 1 change: 1 substitution"),
        "{before}"
    );
    let back = markdown_to_docx(&before, &DocxOptions::default())
        .unwrap()
        .docx;
    let after = view(&back);
    for date in [old, new] {
        assert!(after.contains(&format!("@AL {date}}}")), "{after}");
    }
    assert!(!after.contains("{~~"), "{after}");
}

#[test]
fn equal_or_absent_dates_still_pair() {
    for date in [None, Some("2026-10-01T09:00:00Z")] {
        let markdown = view(&dated_pair(date, date));
        assert!(markdown.contains("{~~old~>new~~}{r1+2 @AL}"), "{markdown}");
    }
}

#[test]
fn unequal_plain_dates_also_remain_two_marks() {
    let docx = dated_pair(Some("2026-10-01T09:00:00Z"), Some("2026-10-02T10:00:00Z"));
    let markdown = jubarte::markdown::docx_to_markdown(
        &docx,
        &jubarte::markdown::MarkdownOptions {
            dates: true,
            ..Default::default()
        },
    )
    .unwrap()
    .markdown;
    assert!(!markdown.contains("{~~"), "{markdown}");
    for date in ["2026-10-01T09:00:00Z", "2026-10-02T10:00:00Z"] {
        assert!(markdown.contains(date), "{markdown}");
    }
}
