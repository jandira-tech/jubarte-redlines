// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::markdown::diff_markdown`: the CriticMarkup it writes holds
//! both versions. Written to Word, accepting every change gives the new
//! document's paragraphs and rejecting every change the old one's.

mod common;

use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::inspect;
use jubarte::markdown::{
    DocxOptions, TrackChanges, diff_markdown, markdown_to_docx, resolve_critic,
};

use common::markdown_pairs::PAIRS;

/// Body paragraph texts of `markdown` written to Word, its changes kept,
/// accepted or rejected. A document diffed is text: its delimiters, if any,
/// are not CriticMarkup (`critic` off).
fn paragraphs(markdown: &str, choice: TrackChanges, critic: bool) -> Vec<String> {
    let options = DocxOptions {
        critic,
        ..DocxOptions::default()
    };
    let docx = markdown_to_docx(markdown, &options).unwrap().docx;
    assert_word_valid_package(&docx);
    let docx = match choice {
        TrackChanges::All => docx,
        TrackChanges::Accept => accept_revisions(&docx).unwrap(),
        TrackChanges::Reject => reject_revisions(&docx).unwrap(),
    };
    let texts: Vec<String> = inspect::paragraphs(&docx)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    // An empty document is one empty paragraph.
    if texts.iter().all(String::is_empty) {
        Vec::new()
    } else {
        texts
    }
}

#[test]
fn accepting_the_diff_gives_the_new_document_and_rejecting_it_the_old() {
    let mut failures = Vec::new();
    for (name, old, new) in PAIRS {
        let diff = diff_markdown(old, new);
        let accepted = paragraphs(&diff, TrackChanges::Accept, true);
        let rejected = paragraphs(&diff, TrackChanges::Reject, true);
        let want_new = paragraphs(new, TrackChanges::All, false);
        let want_old = paragraphs(old, TrackChanges::All, false);
        if accepted != want_new || rejected != want_old {
            failures.push(format!(
                "{name}:\n  diff: {diff:?}\n  accepted {accepted:?}\n  want     {want_new:?}\n  rejected {rejected:?}\n  want     {want_old:?}"
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn edits_inside_paragraphs_resolve_to_the_exact_text() {
    for (name, old, new) in PAIRS {
        let diff = diff_markdown(old, new);
        if [
            "words",
            "phrase",
            "unrelated",
            "line inside paragraph",
            "list item edited",
            "quote",
            "table cell",
            "code block",
            "link changed",
            "footnote",
        ]
        .contains(name)
        {
            assert_eq!(
                resolve_critic(&diff, TrackChanges::Accept),
                *new,
                "{name}: {diff}"
            );
            assert_eq!(
                resolve_critic(&diff, TrackChanges::Reject),
                *old,
                "{name}: {diff}"
            );
        }
    }
}

#[test]
fn an_unchanged_document_has_no_markup() {
    for (name, old, _) in PAIRS {
        if *name == "delimiters in text" {
            // Delimiters in the text are escaped, so they stay text.
            assert_eq!(diff_markdown(old, old), "a \\{++b++\\} c\n");
        } else {
            assert_eq!(diff_markdown(old, old), *old, "{name}");
        }
    }
}
