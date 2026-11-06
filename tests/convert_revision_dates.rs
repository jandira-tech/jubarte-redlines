// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! `convert -t md` then back to docx keeps each change's own date.

mod common;

use common::docx::{docx, part_string};
use std::process::Command;

const EARLY: &str = "2026-01-01T09:00:00Z";
const LATER: &str = "2026-02-02T15:30:00Z";

fn mark(kind: &str, id: u32, date: &str, text: &str) -> String {
    let node = if kind == "del" { "delText" } else { "t" };
    format!(
        r#"<w:{kind} w:id="{id}" w:author="Ann Lee" w:date="{date}"><w:r><w:{node}>{text}</w:{node}></w:r></w:{kind}>"#
    )
}

/// One author, two instants: the early date once and the later date twice.
fn source() -> Vec<u8> {
    docx(&format!(
        "<w:p>{}{}{}</w:p>",
        mark("ins", 1, EARLY, "one"),
        mark("del", 2, LATER, "two"),
        mark("ins", 3, LATER, "three"),
    ))
}

fn date_count(xml: &str, date: &str) -> usize {
    xml.matches(&format!(r#"w:date="{date}""#)).count()
}

fn run(args: &[&str], dir: &std::path::Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn convert_round_trip_keeps_each_changes_date() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("different_dates.docx"), source()).unwrap();
    let original = part_string(
        &std::fs::read(dir.path().join("different_dates.docx")).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    assert_eq!(date_count(&original, EARLY), 1, "{original}");
    assert_eq!(date_count(&original, LATER), 2, "{original}");

    run(
        &[
            "convert",
            "different_dates.docx",
            "-t",
            "md",
            "--no-page-markers",
            "-o",
            "dates.md",
        ],
        dir.path(),
    );
    let markdown = std::fs::read_to_string(dir.path().join("dates.md")).unwrap();
    assert!(
        markdown.contains(EARLY),
        "convert dropped the earlier date: {markdown}"
    );
    assert!(
        markdown.contains(LATER),
        "convert dropped the later date: {markdown}"
    );

    run(
        &["convert", "dates.md", "-t", "docx", "-o", "back.docx"],
        dir.path(),
    );
    let back = part_string(
        &std::fs::read(dir.path().join("back.docx")).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    assert_eq!(
        date_count(&back, EARLY),
        1,
        "round trip lost the earlier date: {back}\n{markdown}"
    );
    assert_eq!(
        date_count(&back, LATER),
        2,
        "round trip collapsed the later dates: {back}\n{markdown}"
    );
}

#[test]
fn convert_round_trip_leaves_a_shared_date_in_the_header() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("same.docx"),
        docx(&format!(
            "<w:p>{}{}</w:p>",
            mark("del", 1, LATER, "two"),
            mark("ins", 2, LATER, "three"),
        )),
    )
    .unwrap();
    run(
        &[
            "convert",
            "same.docx",
            "-t",
            "md",
            "--no-page-markers",
            "-o",
            "same.md",
        ],
        dir.path(),
    );
    let markdown = std::fs::read_to_string(dir.path().join("same.md")).unwrap();
    assert!(
        markdown.contains(&format!("2 revisions, {LATER}")),
        "{markdown}"
    );
    assert!(
        !markdown.contains(&format!("@AL {LATER}")),
        "a shared date stays in the header: {markdown}"
    );
    run(
        &["convert", "same.md", "-t", "docx", "-o", "same-back.docx"],
        dir.path(),
    );
    let back = part_string(
        &std::fs::read(dir.path().join("same-back.docx")).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    assert_eq!(date_count(&back, LATER), 2, "{back}\n{markdown}");
    assert_eq!(date_count(&back, EARLY), 0, "{back}");
}
