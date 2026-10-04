// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `legacy_doc`: a Word 97-2003 `.doc` read into blocks, Markdown and a
//! Word-valid `.docx`, and hostile bytes refused without a panic. The
//! fixture is LibreOffice's "MS Word 97" export of
//! `tests/fixtures/legacy/services.md` (no Word-saved `.doc` is in the
//! repository yet; `docs/adoption/plans.md` lists it).

mod common;

use common::validity::assert_word_valid_package;
use jubarte::legacy_doc::{Block, doc_to_docx, doc_to_markdown, read};

fn fixture() -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/legacy/services.doc"),
    )
    .unwrap()
}

/// Each block as one line: `H1 text`, `- text` / `1. text` with two
/// spaces per level, `P text`, `T a|b;c|d`.
fn outline(blocks: &[Block]) -> Vec<String> {
    blocks
        .iter()
        .map(|block| match block {
            Block::Paragraph(p) => match (p.heading, p.list) {
                (Some(level), _) => format!("H{level} {}", p.text()),
                (None, Some(item)) => format!(
                    "{}{} {}",
                    "  ".repeat(usize::from(item.level)),
                    if item.ordered { "1." } else { "-" },
                    p.text()
                ),
                (None, None) => format!("P {}", p.text()),
            },
            Block::Table(rows) => format!(
                "T {}",
                rows.iter()
                    .map(|r| r.join("|"))
                    .collect::<Vec<_>>()
                    .join(";")
            ),
        })
        .collect()
}

#[test]
fn headings_lists_paragraphs_and_the_table_are_read_in_order() {
    let document = read(&fixture()).unwrap();
    assert_eq!(
        outline(&document.blocks),
        [
            "H1 Services Agreement",
            "P This Agreement is made between Acme Corp and Beta LLC.",
            "H2 1. Fees",
            "P The fee is $1,000 per month [net 30] and #2 applies; use a_b * c.",
            "T Item|Price|Notes;Setup|500|one-off;Support|100|monthly",
            "H2 2. Term",
            "- First bullet",
            "  - Nested bullet",
            "- Second bullet",
            "1. Notice in writing",
            "1. Cure within ten days",
            "P The term is twelve months.",
        ]
    );
}

#[test]
fn bold_and_italic_runs_are_kept() {
    let document = read(&fixture()).unwrap();
    let Block::Paragraph(intro) = &document.blocks[1] else {
        panic!("{:?}", document.blocks[1]);
    };
    let formatted: Vec<(&str, bool, bool)> = intro
        .spans
        .iter()
        .map(|s| (s.text.as_str(), s.bold, s.italic))
        .collect();
    assert_eq!(
        formatted,
        [
            ("This Agreement is made between ", false, false),
            ("Acme Corp", true, false),
            (" and ", false, false),
            ("Beta LLC", false, true),
            (".", false, false),
        ]
    );
    let markdown = doc_to_markdown(&fixture()).unwrap();
    assert!(
        markdown.contains("Cure within ***ten*** days"),
        "{markdown}"
    );
}

#[test]
fn markdown_escapes_what_markdown_would_read_as_syntax() {
    let markdown = doc_to_markdown(&fixture()).unwrap();
    assert!(
        markdown.contains(
            "The fee is \\$1,000 per month \\[net 30\\] and \\#2 applies; use a\\_b \\* c."
        ),
        "{markdown}"
    );
    assert!(
        markdown.starts_with("# Services Agreement\n\n"),
        "{markdown}"
    );
    assert!(markdown.contains("## 1\\. Fees\n"), "{markdown}");
}

#[test]
fn the_docx_is_word_valid_and_keeps_the_text() {
    let docx = doc_to_docx(&fixture()).unwrap();
    assert_word_valid_package(&docx);
    let text = jubarte::inspect::markdown(&docx).unwrap();
    assert!(
        text.contains("The fee is $1,000 per month [net 30] and #2 applies; use a_b * c."),
        "{text}"
    );
}

#[test]
fn truncated_or_corrupted_files_are_refused_without_a_panic() {
    let doc = fixture();
    for len in (0..doc.len()).step_by(61) {
        let _ = read(&doc[..len]);
    }
    for at in (0..doc.len()).step_by(37) {
        for value in [0x00, 0xFF, 0x7F] {
            let mut bad = doc.clone();
            bad[at] = value;
            let _ = read(&bad);
        }
    }
}

#[test]
fn an_encrypted_flag_is_refused_with_legacy_doc() {
    let mut doc = fixture();
    // fEncrypted in the FIB's flags word (offset 0x0A of WordDocument):
    // find the stream by its wIdent and set the bit.
    let at = doc
        .windows(2)
        .enumerate()
        .skip(512)
        .find(|(i, w)| *w == [0xEC, 0xA5] && i % 512 == 0)
        .map(|(i, _)| i)
        .expect("WordDocument stream");
    doc[at + 0x0B] |= 0x01;
    let error = read(&doc).unwrap_err().to_string();
    assert!(error.starts_with("LEGACY_DOC: an encrypted"), "{error}");
}
