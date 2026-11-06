// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Accepting a comment whose only anchor is a deletion still publishes when
//! one part sits between the inspect budget and the compare budget.

mod common;

use std::io::Read;
use std::path::Path;
use std::process::Command;

use common::docx::{Part, W_NS, docx_with};
use jubarte::admission::{InputLimits, admit};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn oversized_removed_comment() -> Vec<u8> {
    let pad = "\0".repeat(InputLimits::default().max_part_bytes as usize + 1);
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Ann" w:date="2026-01-01T00:00:00Z"><w:p><w:r><w:t>Noted</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    docx_with(
        r#"<w:p><w:del w:id="1" w:author="Ann" w:date="2026-01-01T00:00:00Z"><w:commentRangeStart w:id="0"/><w:r><w:delText>Gone</w:delText></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:del></w:p>"#,
        &[
            Part {
                name: "word/comments.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: &comments,
            },
            Part {
                name: "word/media/pad.bin",
                content_type: "application/octet-stream",
                rel_type: "",
                xml: &pad,
            },
        ],
    )
}

#[test]
fn accept_publishes_a_removed_comment_past_the_inspect_budget() {
    let bytes = oversized_removed_comment();
    assert!(
        admit(&bytes, InputLimits::default()).is_err(),
        "the part must sit past the inspect budget"
    );
    assert!(
        admit(&bytes, InputLimits::compare()).is_ok(),
        "the part must stay inside the compare budget"
    );
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("source.docx"), &bytes).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &["accept", "source.docx", "-o", "accepted.docx"],
    );
    assert_eq!(code, 0, "{out}{err}");
    let published = std::fs::read(dir.path().join("accepted.docx")).expect("accept publishes");
    assert!(
        published.starts_with(b"PK"),
        "published bytes are not a package"
    );
    assert!(
        err.contains("warning: accept removed comment c0 (Ann: \"Noted\")"),
        "{err}"
    );
    let mut document = String::new();
    zip::ZipArchive::new(std::io::Cursor::new(published))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut document)
        .unwrap();
    assert!(!document.contains("Gone"), "{document}");
}
