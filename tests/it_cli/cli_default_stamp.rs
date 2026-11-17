// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! What signs and dates the changes a command makes (Arthur, 2026-10-14):
//! changing a file (`compare`, `edit`, `add`) signs them by the flags, else
//! Jubarte and now, never by the files' own `dc:creator` and
//! `dcterms:created`; those describe the document, not this change.

use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

fn jubarte(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// `name`.docx holding `text`, its core properties naming `creator` and
/// `created` (when given).
fn docx(dir: &Path, name: &str, text: &str, creator: Option<&str>, created: Option<&str>) {
    let md = dir.join(format!("{name}.md"));
    std::fs::write(&md, format!("{text}\n")).unwrap();
    let path = dir.join(format!("{name}.docx"));
    jubarte(
        dir,
        &[
            "convert",
            md.to_str().unwrap(),
            "-o",
            path.to_str().unwrap(),
            "--force",
        ],
    );
    let mut core = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" \
         xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" \
         xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">",
    );
    if let Some(creator) = creator {
        core.push_str(&format!("<dc:creator>{creator}</dc:creator>"));
    }
    if let Some(created) = created {
        core.push_str(&format!(
            "<dcterms:created xsi:type=\"dcterms:W3CDTF\">{created}</dcterms:created>"
        ));
    }
    core.push_str("</cp:coreProperties>");
    let bytes = std::fs::read(&path).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    let mut has_core = false;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        let name = entry.name().expect("decode ZIP fixture name").into_owned();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).unwrap();
        if name == "docProps/core.xml" {
            has_core = true;
            data = core.clone().into_bytes();
        }
        writer.start_file(name, options).unwrap();
        writer.write_all(&data).unwrap();
    }
    assert!(has_core, "{name}.docx has no docProps/core.xml");
    std::fs::write(&path, writer.finish().unwrap().into_inner()).unwrap();
}

fn part(docx: &Path, name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(docx).unwrap()).unwrap();
    let mut text = String::new();
    zip.by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

/// The `w:author` and `w:date` of the first insertion in `docx`.
fn first_insertion(docx: &Path) -> (String, String) {
    let xml = part(docx, "word/document.xml");
    let at = xml.find("<w:ins ").unwrap();
    let tag = &xml[at..at + xml[at..].find('>').unwrap()];
    let attr = |name: &str| {
        let from = tag.find(&format!("{name}=\"")).unwrap() + name.len() + 2;
        tag[from..from + tag[from..].find('"').unwrap()].to_string()
    };
    (attr("w:author"), attr("w:date"))
}

/// `date` is a timestamp taken between `before` and `after`.
fn now_between(before: &str, date: &str, after: &str) {
    assert!(
        before <= date && date <= after,
        "{before} <= {date} <= {after}"
    );
}

#[test]
fn compare_signs_redline_now_whatever_the_files_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    docx(
        d,
        "a",
        "One two.",
        Some("Ann Orig"),
        Some("2023-01-02T03:04:05Z"),
    );
    docx(
        d,
        "b",
        "One three.",
        Some("Bea Mod"),
        Some("2024-05-06T07:08:09Z"),
    );
    let before = jubarte::convert::utc_now_iso8601();
    jubarte(
        d,
        &["compare", "a.docx", "b.docx", "-o", "r.docx", "--force"],
    );
    let after = jubarte::convert::utc_now_iso8601();
    let (author, date) = first_insertion(&d.join("r.docx"));
    assert_eq!(author, "Jubarte");
    now_between(&before, &date, &after);
    // The flags name someone else.
    jubarte(
        d,
        &[
            "compare",
            "a.docx",
            "b.docx",
            "-o",
            "r.docx",
            "--force",
            "-a",
            "Flag Name",
            "-d",
            "2022-02-02T02:02:02Z",
        ],
    );
    assert_eq!(
        first_insertion(&d.join("r.docx")),
        ("Flag Name".into(), "2022-02-02T02:02:02Z".into())
    );
}

#[test]
fn edit_and_add_sign_redline_now_whatever_the_files_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    docx(
        d,
        "e",
        "One two.",
        Some("Cy Edit"),
        Some("2021-07-08T09:10:11Z"),
    );
    for (verb, content, out) in [
        ("edit", "One three.", "e2.docx"),
        ("add", "Added.", "e3.docx"),
    ] {
        let before = jubarte::convert::utc_now_iso8601();
        jubarte(
            d,
            &[
                verb,
                "e.docx",
                "-p",
                "p0",
                "--content",
                content,
                "-a",
                "Jubarte",
                "-o",
                out,
            ],
        );
        let after = jubarte::convert::utc_now_iso8601();
        let (author, date) = first_insertion(&d.join(out));
        assert_eq!(author, "Jubarte", "{verb}");
        now_between(&before, &date, &after);
    }
    jubarte(
        d,
        &[
            "edit",
            "e.docx",
            "-p",
            "p0",
            "--content",
            "One three.",
            "-o",
            "e4.docx",
            "--author",
            "Typed",
            "-d",
            "2020-01-01T00:00:00Z",
        ],
    );
    assert_eq!(
        first_insertion(&d.join("e4.docx")),
        ("Typed".into(), "2020-01-01T00:00:00Z".into())
    );
}
