// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::markdown::patch_redline` and `patch_documents`: the patch of a
//! Word redline, each hunk at the `body:p:N` id `jubarte text` and edit plans
//! use, in the new version (or the old one, for a removed paragraph).

mod common;

use common::markdown_pairs::resolve;
use jubarte::inspect::{paragraphs, stories};
use jubarte::markdown::{
    Attribution, Locator, Patch, PatchOptions, RedlineOptions, Source, patch_documents,
    patch_markdown, patch_redline,
};

fn options(old: &str, new: &str) -> PatchOptions {
    PatchOptions {
        old_name: old.to_string(),
        new_name: new.to_string(),
        owner: Attribution {
            author: "Arthur Rodrigues".to_string(),
            date: "2026-09-30T14:05:00Z".to_string(),
        },
    }
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Letters and digits only, link targets and note references left out:
/// what a paragraph's text and a Markdown block have in common.
fn letters(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        let skipped = if rest.starts_with("](") {
            rest.find(')')
        } else if rest.starts_with("[^") {
            rest.find(']')
        } else {
            None
        };
        if let Some(end) = skipped {
            rest = &rest[end + 1..];
            continue;
        }
        if c.is_alphanumeric() {
            out.push(c);
        }
        rest = &rest[c.len_utf8()..];
    }
    out
}

/// Every paragraph `jubarte text` lists: the body's, then each story's.
fn all_paragraphs(docx: &[u8]) -> Vec<jubarte::inspect::Paragraph> {
    let mut all = paragraphs(docx).unwrap();
    for story in stories(docx).unwrap() {
        all.extend(story.paragraphs);
    }
    all
}

/// The index of a hunk's body paragraph.
fn body_index(at: &Locator) -> usize {
    match at {
        Locator::Paragraph { story, index } if story == "body" => *index,
        other => panic!("{other:?} is not a body paragraph"),
    }
}

/// Every hunk is at a paragraph id whose paragraph, in `new` (or `old` when
/// removed), has text the hunk's paragraph holds.
fn assert_located(patch: &Patch, old: &[u8], new: &[u8], name: &str) {
    let (old, new) = (all_paragraphs(old), all_paragraphs(new));
    assert!(!patch.hunks.is_empty(), "{name}: no hunks");
    for hunk in &patch.hunks {
        let id = hunk.at.to_string();
        let side = if hunk.removed { &old } else { &new };
        let paragraph = side
            .iter()
            .find(|p| p.id == id)
            .unwrap_or_else(|| panic!("{name}: no paragraph {id}"));
        let block = letters(&resolve(&hunk.text, !hunk.removed));
        let text = letters(&paragraph.text);
        assert!(
            !text.is_empty() && block.contains(&text),
            "{name}: {id} is {:?}, the hunk is\n{}",
            paragraph.text,
            hunk.text
        );
    }
}

const SYNTHETIC: &str = "tests/fixtures/from-docx/critic/synthetic";

/// Fixtures whose changes are all in the body; `hard-tables` has a text box
/// and is checked on its own below.
const BODY_ONLY: &[&str] = &[
    "additions",
    "all",
    "breaks",
    "comments-additions",
    "comments-deletions",
    "comments",
    "deletions",
    "hard-adjacent",
    "hard-comments",
    "hard-delimiters",
    "hard-formatting",
    "hard-headings",
    "hard-links",
    "hard-lists",
    "hard-notes",
    "hard-unicode",
    "structure",
    "substitutions",
    "tables",
];

#[test]
fn every_hunk_of_a_redline_is_at_its_paragraphs_id() {
    for name in BODY_ONLY {
        let docx = read(&format!("{SYNTHETIC}/{name}.docx"));
        let patch = patch_redline(&docx, &options("a.docx", "b.docx")).unwrap();
        assert_located(
            &patch,
            &read(&format!("{SYNTHETIC}/{name}.rejected.docx")),
            &read(&format!("{SYNTHETIC}/{name}.accepted.docx")),
            name,
        );
    }
}

#[test]
fn changes_by_others_name_them_and_the_owners_do_not() {
    let docx = read(&format!("{SYNTHETIC}/all.docx"));
    let patch = patch_redline(&docx, &options("a.docx", "b.docx")).unwrap();
    let text = patch.render(0);
    assert!(
        text.starts_with(
            "--- a/a.docx\n+++ b/b.docx\tArthur Rodrigues\t2026-09-30T14:05:00Z\n@@ [body:p:"
        ),
        "{text}"
    );
    assert!(
        text.contains("{>>Ana Lima (2026-09-01T00:35:00Z): Legal to review.<<}"),
        "{text}"
    );
    assert!(
        text.contains("{+ without undue delay+}{>>Bo Chen (2026-09-01T00:42:00Z)<<}"),
        "{text}"
    );
    assert!(!text.contains("Arthur Rodrigues ("), "{text}");
}

#[test]
fn a_text_box_change_is_at_the_paragraph_that_holds_the_box() {
    let docx = read(&format!("{SYNTHETIC}/hard-tables.docx"));
    let patch = patch_redline(&docx, &options("a.docx", "b.docx")).unwrap();
    let accepted = paragraphs(&read(&format!("{SYNTHETIC}/hard-tables.accepted.docx"))).unwrap();
    let boxed = patch
        .hunks
        .iter()
        .find(|h| h.text.starts_with("Box "))
        .expect("the text box's hunk");
    let index = body_index(&boxed.at);
    assert!(
        accepted[index]
            .text
            .starts_with("A paragraph with a text box"),
        "{:?}",
        accepted[index]
    );
}

#[test]
fn two_word_documents_are_located_in_each() {
    let (old, new) = (
        read("tests/fixtures/redline/original.docx"),
        read("tests/fixtures/redline/modified.docx"),
    );
    let patch = patch_documents(
        Source::Docx(&old),
        Source::Docx(&new),
        &RedlineOptions::default(),
        &options("original.docx", "modified.docx"),
    )
    .unwrap();
    assert_located(&patch, &old, &new, "original/modified");
    // The comparer's changes are the owner's: no attribution (`Name (date)`
    // with no comment text) follows any of them.
    assert!(!patch.render(0).contains(")<<}"), "{patch}");
}

#[test]
fn markdown_against_word_is_located_in_the_word_document() {
    let docx = read(&format!("{SYNTHETIC}/all.accepted.docx"));
    let markdown = std::fs::read_to_string(format!("{SYNTHETIC}/all.accept.md")).unwrap();
    let edited = markdown.replacen("twelve", "eighteen", 1);
    assert_ne!(edited, markdown);
    // Word to Markdown: located in the Word document, now the old side.
    let patch = patch_documents(
        Source::Docx(&docx),
        Source::Markdown(&edited),
        &RedlineOptions::default(),
        &options("all.docx", "all.md"),
    )
    .unwrap();
    assert_eq!(patch.hunks.len(), 1, "{patch}");
    assert!(
        patch.render(0).contains("[-**twelve**-]{+**eighteen**+}"),
        "{patch}"
    );
    let index = body_index(&patch.hunks[0].at);
    assert!(paragraphs(&docx).unwrap()[index].text.contains("twelve"));
    // Markdown to Word: the same paragraph, the other way round.
    let patch = patch_documents(
        Source::Markdown(&edited),
        Source::Docx(&docx),
        &RedlineOptions::default(),
        &options("all.md", "all.docx"),
    )
    .unwrap();
    assert!(
        patch.render(0).contains("[-**eighteen**-]{+**twelve**+}"),
        "{patch}"
    );
    assert_eq!(body_index(&patch.hunks[0].at), index);
}

#[test]
fn two_markdown_documents_keep_their_lines() {
    let (old, new) = ("A.\n\nB.\n\nC.\n", "A.\n\nB, too.\n\nC.\n");
    let options = options("a.md", "b.md");
    let patch = patch_documents(
        Source::Markdown(old),
        Source::Markdown(new),
        &RedlineOptions::default(),
        &options,
    )
    .unwrap();
    assert_eq!(patch, patch_markdown(old, new, &options));
    assert_eq!(patch.hunks[0].at, Locator::Line(3));
}

#[test]
fn identical_word_documents_give_an_empty_patch() {
    let docx = read("tests/fixtures/redline/original.docx");
    let patch = patch_documents(
        Source::Docx(&docx),
        Source::Docx(&docx),
        &RedlineOptions::default(),
        &options("a.docx", "a.docx"),
    )
    .unwrap();
    assert!(patch.hunks.is_empty(), "{patch}");
    assert_eq!(patch.to_string(), "");
}
