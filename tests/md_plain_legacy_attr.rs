// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! The old plain-view attribution wrapper is refused, the same as a handle tag.

mod common;

use jubarte::markdown::{DocxOptions, markdown_to_docx};

fn error_of(markdown: &str) -> String {
    let result = markdown_to_docx(markdown, &DocxOptions::default());
    assert!(result.is_err(), "plain wrapper still converts");
    result.unwrap_err().to_string()
}

#[test]
fn old_plain_attribution_wrapper_is_refused() {
    let error = error_of("{++x++}{>>r1 Alice (2026-10-01T09:00:00Z)<<}");
    assert!(
        error.contains("line 1") && error.contains("{r1 Alice (2026-10-01T09:00:00Z)}"),
        "{error}"
    );

    let error =
        error_of("{++x++}{>>r1 Alice (2026-10-01T09:00:00Z) on r3 Bob (2026-10-02T09:00:00Z)<<}");
    assert!(
        error.contains("line 1")
            && error.contains("{r1 Alice (2026-10-01T09:00:00Z) re r3 Bob (2026-10-02T09:00:00Z)}"),
        "{error}"
    );
}

#[test]
fn a_free_plain_note_away_from_a_closer_still_converts() {
    for source in [
        "text {>>r1 Alice (2026-10-01T09:00:00Z)<<}",
        "{++x++} {>>r1 Alice (2026-10-01T09:00:00Z)<<}",
    ] {
        let docx = markdown_to_docx(source, &DocxOptions::default())
            .expect(source)
            .docx;
        assert!(common::docx::part_string(&docx, "word/document.xml").is_some());
        let comments = common::docx::part_string(&docx, "word/comments.xml").expect(source);
        assert!(
            comments.contains("w:author=\"r1 Alice\"")
                && comments.contains("w:date=\"2026-10-01T09:00:00Z\""),
            "{source}: {comments}"
        );
    }
}
