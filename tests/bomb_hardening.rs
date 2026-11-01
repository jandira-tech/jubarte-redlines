// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Hostile packages from the 2026-10-31 bomb review (#2019): an element
//! flood under every byte limit (#2014) and a DOCTYPE, which Word refuses in
//! every form (#2015), are refused by every command before a part is parsed;
//! `read` applies its own budget before its layout pass (#2013).

use std::io::Write;
use std::process::Command;

use jubarte::admission::{InputLimitOverrides, InputLimits, admit};
use jubarte::markdown::{MarkdownOptions, ReadOptions, docx_to_markdown, read};

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// A minimal valid docx whose main part is `document` verbatim.
fn package(document: &str) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    let entries: [(&str, &str); 3] = [
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
        ),
        ("word/document.xml", document),
    ];
    for (name, body) in entries {
        zip.start_file(name, opts).unwrap();
        zip.write_all(body.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn document(prelude: &str, body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>{prelude}<w:document xmlns:w="{W}"><w:body>{body}</w:body></w:document>"#
    )
}

/// `paragraphs` one-run paragraphs: three elements each (`w:p`, `w:r`,
/// `w:t`), in a package of eight more.
fn flood(paragraphs: usize) -> Vec<u8> {
    let body = "<w:p><w:r><w:t>a</w:t></w:r></w:p>".repeat(paragraphs);
    package(&document("", &body))
}

fn no_markers() -> ReadOptions {
    ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    }
}

fn read_error(docx: &[u8], options: &ReadOptions) -> String {
    match read(docx, options) {
        Ok(view) => panic!("read wrote a view: {} bytes", view.markdown.len()),
        Err(e) => e.to_string(),
    }
}

#[test]
fn an_element_flood_past_the_budget_is_refused_before_any_part_is_parsed() {
    // 680,000 paragraphs = 2,040,008 elements: past the default 2,000,000,
    // inside the compare budget (4,000,000), and a few hundred KB zipped.
    let docx = flood(680_000);
    assert!(docx.len() < 1_000_000, "{} bytes", docx.len());
    for options in [no_markers(), ReadOptions::default()] {
        let error = read_error(&docx, &options);
        assert!(error.contains("INPUT_LIMIT"), "{error}");
        assert!(
            error.contains("2000000 XML elements, the element limit"),
            "{error}"
        );
    }
    let refused = admit(&docx, InputLimits::default()).unwrap_err();
    assert_eq!(refused.code(), "INPUT_LIMIT", "{refused}");
    // The comparer's larger budget admits it.
    assert!(admit(&docx, InputLimits::compare()).is_ok());
}

#[test]
fn the_element_budget_is_an_override_like_the_other_caps() {
    let docx = flood(10);
    let tight = InputLimitOverrides {
        max_xml_elements: Some(20),
        ..InputLimitOverrides::default()
    }
    .apply(InputLimits::default());
    let error = docx_to_markdown(
        &docx,
        &MarkdownOptions {
            input_limits: tight,
            page_markers: false,
            ..MarkdownOptions::default()
        },
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("past 20 XML elements"), "{error}");
    assert!(admit(&docx, tight).is_err());
    assert_eq!(
        InputLimitOverrides::from_json(r#"{"max_xml_elements": 7}"#)
            .unwrap()
            .max_xml_elements,
        Some(7)
    );
    // The default reads it.
    assert!(read(&docx, &no_markers()).is_ok());
}

#[test]
fn media_and_comments_do_not_charge_the_element_budget() {
    // A comment full of tag-like text adds nothing: the package is the
    // content types (4 elements), the package rels (2) and a 5-element
    // document, 11 in all.
    let commented = document(
        "<!-- <w:p><w:p><w:p><w:p><w:p><w:p><w:p><w:p> -->",
        "<w:p><w:r><w:t>a</w:t></w:r></w:p>",
    );
    let tight = InputLimitOverrides {
        max_xml_elements: Some(11),
        ..InputLimitOverrides::default()
    }
    .apply(InputLimits::default());
    let admitted = admit(&package(&commented), tight);
    assert!(admitted.is_ok(), "{:?}", admitted.err());
    let one_more = InputLimitOverrides {
        max_xml_elements: Some(10),
        ..InputLimitOverrides::default()
    }
    .apply(InputLimits::default());
    assert!(admit(&package(&commented), one_more).is_err());
}

/// Every DOCTYPE shape Word refused on 2026-10-31 (word_exp X1, X2, X3, B5).
fn doctypes() -> Vec<(&'static str, Vec<u8>)> {
    let ok = "<w:p><w:r><w:t>BODY_OK</w:t></w:r></w:p>";
    vec![
        ("bare", package(&document("<!DOCTYPE w:document>", ok))),
        (
            "internal entity",
            package(&document(
                r#"<!DOCTYPE w:document [<!ENTITY a "EXPANDED">]>"#,
                "<w:p><w:r><w:t>ENT[&a;]</w:t></w:r></w:p>",
            )),
        ),
        (
            "'>' inside the subset",
            package(&document(
                r#"<!DOCTYPE w:document [<!ENTITY a "<w:p><w:r><w:t>SMUGGLED</w:t></w:r></w:p>">]>"#,
                ok,
            )),
        ),
        (
            "external entity",
            package(&document(
                r#"<!DOCTYPE w:document [<!ENTITY lol "lol"><!ENTITY xxe SYSTEM "file:///etc/hosts">]>"#,
                "<w:p><w:r><w:t>&lol;&xxe;</w:t></w:r></w:p>",
            )),
        ),
    ]
}

#[test]
fn a_doctype_is_refused_by_read_and_by_admission() {
    for (label, docx) in doctypes() {
        for options in [no_markers(), ReadOptions::default()] {
            let error = read_error(&docx, &options);
            assert!(
                error.contains("INVALID_XML") && error.contains("DTD declarations are unsupported"),
                "{label}: {error}"
            );
        }
        let refused = admit(&docx, InputLimits::compare()).unwrap_err();
        assert_eq!(refused.code(), "INVALID_XML", "{label}: {refused}");
        assert!(refused.to_string().contains("DTD"), "{label}: {refused}");
    }
}

#[test]
fn validate_and_compare_refuse_a_doctype_on_the_command_line() {
    let dir = std::env::temp_dir().join(format!("jubarte-bomb-doctype-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let plain = dir.join("plain.docx");
    std::fs::write(
        &plain,
        package(&document("", "<w:p><w:r><w:t>BODY_OK</w:t></w:r></w:p>")),
    )
    .unwrap();
    for (index, (label, docx)) in doctypes().into_iter().enumerate() {
        let file = dir.join(format!("doctype{index}.docx"));
        std::fs::write(&file, docx).unwrap();
        let validate = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .arg("validate")
            .arg(&file)
            .output()
            .unwrap();
        assert_ne!(validate.status.code(), Some(0), "{label}: validate passed");
        let compare = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .arg("compare")
            .arg(&file)
            .arg(&plain)
            .arg("-o")
            .arg(dir.join(format!("redline{index}.docx")))
            .arg("--force")
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&compare.stderr);
        assert_eq!(compare.status.code(), Some(1), "{label}: {stderr}");
        assert!(stderr.contains("DTD"), "{label}: {stderr}");
        assert!(!dir.join(format!("redline{index}.docx")).exists(), "{label}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn jubarte_timeout_bounds_any_task_with_exit_124_and_no_output() {
    let dir = std::env::temp_dir().join(format!("jubarte-bomb-timeout-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // 600,000 paragraphs (1.8M elements): inside the element budget, and
    // many seconds of view and layout work.
    let file = dir.join("slow.docx");
    std::fs::write(&file, flood(600_000)).unwrap();
    let view = dir.join("view.md");
    let started = std::time::Instant::now();
    let bounded = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .env("JUBARTE_TIMEOUT", "0.5")
        .arg("read")
        .arg(&file)
        .arg("-o")
        .arg(&view)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&bounded.stderr);
    assert_eq!(bounded.status.code(), Some(124), "{stderr}");
    assert!(
        stderr.contains("DEADLINE_EXCEEDED: timed out after 0.5s (JUBARTE_TIMEOUT)"),
        "{stderr}"
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(30));
    assert!(!view.exists());

    for bad in ["0", "-1", "soon", "inf"] {
        let refused = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .env("JUBARTE_TIMEOUT", bad)
            .arg("--help")
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert_eq!(refused.status.code(), Some(2), "{bad}: {stderr}");
        assert!(stderr.contains("error: JUBARTE_TIMEOUT: "), "{bad}: {stderr}");
    }
    // Empty is no bound.
    let unbounded = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .env("JUBARTE_TIMEOUT", "")
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(unbounded.status.code(), Some(0));
    // Unset is no bound either: the timeout is never required.
    let small = dir.join("small.docx");
    std::fs::write(&small, flood(10)).unwrap();
    let unset = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .env_remove("JUBARTE_TIMEOUT")
        .arg("read")
        .arg(&small)
        .output()
        .unwrap();
    assert_eq!(
        unset.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&unset.stderr)
    );
    std::fs::remove_dir_all(&dir).unwrap();
}
