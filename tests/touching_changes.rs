// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Two changes a view prints under separate tags read back as one when
//! the writer has nothing to put between them. Converting that view warns,
//! and names the tags and the change they become.

mod common;

use jubarte::changes::list_logical_changes;
use jubarte::markdown::{DocxOptions, markdown_to_docx};

fn written(markdown: &str) -> jubarte::markdown::WrittenDocx {
    markdown_to_docx(markdown, &DocxOptions::default()).expect("the view writes")
}

#[test]
fn separate_tags_that_touch_warn_they_read_back_as_one_change() {
    let markdown = "pay {--old--}{r1 @A}{++new++}{r2 @A} now\n";
    let out = written(markdown);
    let ids: Vec<String> = list_logical_changes(&out.docx)
        .expect("the changes list")
        .into_iter()
        .map(|change| change.id)
        .collect();
    assert_eq!(
        out.warnings,
        ["r1 and r2 touch in the DOCX, so they read back as one change, r1+2"],
        "changes: {ids:?}"
    );
    assert_eq!(ids, ["r1+2".to_string()]);
    common::validity::assert_word_valid_package(&out.docx);
}

#[test]
fn one_joined_tag_does_not_warn_that_its_pieces_touch() {
    let out = written("pay {~~old~>new~~}{r1+2 @A} now\n");
    assert!(
        out.warnings
            .iter()
            .all(|warning| !warning.contains("touch")),
        "{:?}",
        out.warnings
    );
}

#[test]
fn live_text_between_separate_tags_does_not_join_them() {
    let out = written("pay {--old--}{r1 @A} x {++new++}{r2 @A} now\n");
    assert!(
        out.warnings
            .iter()
            .all(|warning| !warning.contains("touch")),
        "{:?}",
        out.warnings
    );
    let ids: Vec<String> = list_logical_changes(&out.docx)
        .expect("the changes list")
        .into_iter()
        .map(|change| change.id)
        .collect();
    assert_eq!(ids, ["r1".to_string(), "r2".to_string()]);
}

/// Prose that mentions ids is not a tag: an untagged substitution next to
/// text that says `r1` and `r2` gives no warning.
#[test]
fn ids_in_prose_are_not_tags() {
    let out = written("pay {~~old~>new~~} now, as r1 and r2 say\n");
    assert!(
        out.warnings
            .iter()
            .all(|warning| !warning.contains("touch")),
        "{:?}",
        out.warnings
    );
}

/// The plain view's notes (`{r1 Ana Lima (date)}`) are tags too.
#[test]
fn plain_notes_that_touch_still_warn() {
    let out = written(
        "pay {--old--}{r1 Ana Lima (2026-10-01T09:00:00Z)}{++new++}{r2 Ana Lima (2026-10-01T09:00:00Z)} now\n",
    );
    assert!(
        out.warnings
            .iter()
            .any(|warning| warning.starts_with("r1 and r2 touch")),
        "{:?}",
        out.warnings
    );
}
