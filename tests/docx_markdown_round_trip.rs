// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word to Markdown and back: the Markdown `docx_to_markdown` writes for a
//! Word document with tracked changes and comments, written to Word again
//! by `markdown_to_docx`, reads as the same Markdown.

use std::path::{Path, PathBuf};

use jubarte::markdown::{
    DocxOptions, MarkdownOptions, TrackChanges, docx_to_markdown, markdown_to_docx,
};

const DOCUMENTS: [&str; 20] = [
    "all",
    "comments",
    "deletions",
    "additions",
    "comments-deletions",
    "comments-additions",
    "substitutions",
    "breaks",
    "tables",
    "structure",
    "hard-delimiters",
    "hard-unicode",
    "hard-links",
    "hard-notes",
    "hard-lists",
    "hard-comments",
    "hard-adjacent",
    "hard-headings",
    "hard-formatting",
    "hard-tables",
];

fn path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/from-docx/critic/synthetic")
        .join(name)
}

fn read(docx: &[u8], track_changes: TrackChanges) -> String {
    let options = MarkdownOptions {
        track_changes,
        ..MarkdownOptions::default()
    };
    docx_to_markdown(docx, &options).unwrap().markdown
}

/// Where the Markdown read back differs, and why the Word documents still
/// hold the same changes. Accepting and rejecting agree for all of them
/// but the formatting change.
const KNOWN: [(&str, TrackChanges, &str); 6] = [
    (
        "tables",
        TrackChanges::All,
        "adjacent insertions and deletions pair into substitutions differently",
    ),
    (
        "hard-adjacent",
        TrackChanges::All,
        "adjacent insertions and deletions pair into substitutions differently",
    ),
    (
        "hard-tables",
        TrackChanges::All,
        "adjacent insertions and deletions pair into substitutions differently",
    ),
    (
        "hard-headings",
        TrackChanges::All,
        "an added heading is written as Word writes a paragraph added whole, \
         its own mark inserted, which reads back as `## {++Heading\\n\\n++}`",
    ),
    (
        "hard-comments",
        TrackChanges::All,
        "a comment on inserted text starts before the w:ins, not inside it",
    ),
    (
        "hard-formatting",
        TrackChanges::Reject,
        "CriticMarkup has no formatting changes, so rejecting keeps the new formatting",
    ),
];

#[test]
fn markdown_read_from_word_reads_the_same_after_a_round_trip() {
    let dump = std::env::var_os("ROUND_TRIP_DUMP").map(PathBuf::from);
    let mut failures = Vec::new();
    for name in DOCUMENTS {
        let original = std::fs::read(path(&format!("{name}.docx"))).unwrap();
        let markup = read(&original, TrackChanges::All);
        let written = markdown_to_docx(&markup, &DocxOptions::default())
            .unwrap()
            .docx;
        for choice in [
            TrackChanges::All,
            TrackChanges::Accept,
            TrackChanges::Reject,
        ] {
            let before = read(&original, choice);
            let after = read(&written, choice);
            let known = KNOWN.iter().any(|(n, c, _)| *n == name && *c == choice);
            if (before == after) == known {
                if let Some(dir) = &dump {
                    std::fs::write(dir.join(format!("{name}.{choice:?}.before.md")), &before)
                        .unwrap();
                    std::fs::write(dir.join(format!("{name}.{choice:?}.after.md")), &after)
                        .unwrap();
                }
                failures.push(format!(
                    "{name} {choice:?} {}",
                    if known {
                        "now matches: take it out of KNOWN"
                    } else {
                        "differs"
                    }
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
