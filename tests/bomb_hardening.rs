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

/// [`package`] with `extra` parts beside the main one, unreferenced: the
/// budgets read every entry, whatever references it.
fn package_with(document: &str, extra: &[(&str, &[u8])]) -> Vec<u8> {
    let base = package(document);
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(base)).unwrap();
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    for i in 0..archive.len() {
        zip.raw_copy_file(archive.by_index(i).unwrap()).unwrap();
    }
    for (name, body) in extra {
        zip.start_file(*name, opts).unwrap();
        zip.write_all(body).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn ok_body() -> String {
    document("", "<w:p><w:r><w:t>BODY_OK</w:t></w:r></w:p>")
}

/// `limits` with the element budget at `cap`.
fn capped(cap: u64) -> InputLimits {
    InputLimitOverrides {
        max_xml_elements: Some(cap),
        ..InputLimitOverrides::default()
    }
    .apply(InputLimits::default())
}

/// The refusal `read` gives `docx` under `limits` (no page markers).
fn view_error(docx: &[u8], limits: InputLimits) -> String {
    docx_to_markdown(
        docx,
        &MarkdownOptions {
            input_limits: limits,
            page_markers: false,
            ..MarkdownOptions::default()
        },
    )
    .map(|view| format!("wrote a view of {} bytes", view.markdown.len()))
    .unwrap_or_else(|e| e.to_string())
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
        assert!(
            !dir.join(format!("redline{index}.docx")).exists(),
            "{label}"
        );
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

    // The grammar the Python and npm twins copy word for word.
    let refusal = |bad: &str| {
        let refused = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .env("JUBARTE_TIMEOUT", bad)
            .arg("--help")
            .output()
            .unwrap();
        assert_eq!(refused.status.code(), Some(2), "{bad}");
        String::from_utf8_lossy(&refused.stderr).into_owned()
    };
    for bad in ["soon", " 10 ", "0x10", "1_0", "1e"] {
        assert_eq!(
            refusal(bad),
            format!("error: JUBARTE_TIMEOUT: '{bad}' is not a number of seconds\n")
        );
    }
    for bad in ["0", "-1", "inf", "NaN", "+Infinity"] {
        assert_eq!(
            refusal(bad),
            format!(
                "error: JUBARTE_TIMEOUT: '{bad}': the timeout must be a finite number of seconds above 0\n"
            )
        );
    }
    // Empty is no bound.
    let unbounded = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .env("JUBARTE_TIMEOUT", "")
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(unbounded.status.code(), Some(0));
    // A value that is not Unicode is refused, never read as unset.
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let raw = std::ffi::OsStr::from_bytes(b"\x801");
        let refused = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .env("JUBARTE_TIMEOUT", raw)
            .arg("--help")
            .output()
            .unwrap();
        assert_eq!(refused.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&refused.stderr).starts_with("error: JUBARTE_TIMEOUT: '"),
            "{refused:?}"
        );
    }
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

#[test]
fn tags_the_tolerant_parser_reads_as_elements_all_count() {
    // `<0>`, `<->` and `< x>` are no XML names, but the tolerant parser
    // builds an element for each (reviewer finding, 2026-10-31).
    for tag in ["<0/>", "<-/>", "< x/>", "<!x>", "<\u{e9}/>"] {
        let docx = package(&document("", &tag.repeat(40)));
        assert!(
            admit(&docx, capped(40)).is_err(),
            "{tag}: admit counted fewer than 40"
        );
        let error = view_error(&docx, capped(40));
        assert!(
            error.contains("XML elements, the element limit"),
            "{tag}: {error}"
        );
    }
}

#[test]
fn the_budget_spans_every_part_and_every_name() {
    // 11 package elements, then 30 in each of two other parts: one named
    // `.xml`, one a VML drawing that the tolerant parser reads all the same.
    let thirty = "<v:shape/>".repeat(30);
    let docx = package_with(
        &document("", "<w:p><w:r><w:t>a</w:t></w:r></w:p>"),
        &[
            (
                "word/styles.xml",
                format!("<w:styles>{thirty}</w:styles>").as_bytes(),
            ),
            (
                "word/drawings/vmlDrawing1.vml",
                format!("<xml>{thirty}</xml>").as_bytes(),
            ),
        ],
    );
    // Content types 4, package rels 2, document 5, styles 31, VML 31.
    for cap in [73, 72] {
        let limits = capped(cap);
        let (admitted, viewed) = (admit(&docx, limits), view_error(&docx, limits));
        if cap == 73 {
            assert!(admitted.is_ok(), "{admitted:?}");
            assert!(viewed.starts_with("wrote a view"), "{viewed}");
        } else {
            // admit and the package open draw the same line.
            assert_eq!(admitted.unwrap_err().code(), "INPUT_LIMIT");
            assert!(viewed.contains("past 72 XML elements"), "{viewed}");
        }
    }
}

#[test]
fn media_comments_cdata_and_instructions_charge_nothing() {
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend(b"<w:p/>".repeat(100));
    let docx = package_with(
        &document(
            "<?pi <w:p/><w:p/> <!DOCTYPE x>?><!-- <w:p/><w:p/> -->",
            "<w:p><w:r><w:t><![CDATA[<w:p/><w:p/><!DOCTYPE y>]]></w:t></w:r></w:p>",
        ),
        &[("word/media/image1.png", &png)],
    );
    // 4 + 2 + 5: the PNG, the instruction, the comment and the CDATA
    // section hold tags and a DOCTYPE, and none of them counts or refuses.
    let admitted = admit(&docx, capped(11));
    assert!(admitted.is_ok(), "{admitted:?}");
    assert!(view_error(&docx, capped(11)).starts_with("wrote a view"));
    assert!(admit(&docx, capped(10)).is_err());
}

#[test]
fn a_dtd_is_refused_in_any_xml_part_and_anywhere_in_it() {
    let dtd = r#"<!DOCTYPE x [<!ENTITY a "b">]>"#;
    let cases: Vec<(&str, Vec<u8>)> = vec![
        (
            "after the root",
            package(&format!("{}{dtd}", ok_body())),
        ),
        (
            "lower case",
            package(&document("<!doctype w:document>", "<w:p/>")),
        ),
        (
            "a bare entity declaration",
            package(&document(r#"<!ENTITY a "b">"#, "<w:p/>")),
        ),
        (
            "in a relationships part",
            package_with(
                &ok_body(),
                &[(
                    "word/_rels/document.xml.rels",
                    format!(r#"<?xml version="1.0"?>{dtd}<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#).as_bytes(),
                )],
            ),
        ),
        (
            "in UTF-16",
            package_with(&ok_body(), &[("word/styles.xml", &utf16(&format!("{dtd}<w:styles/>")))]),
        ),
    ];
    for (label, docx) in cases {
        let refused = admit(&docx, InputLimits::compare()).unwrap_err();
        assert_eq!(refused.code(), "INVALID_XML", "{label}: {refused}");
        let error = view_error(&docx, InputLimits::default());
        assert!(
            error.contains("DTD declarations are unsupported"),
            "{label}: {error}"
        );
    }
}

/// `text` as UTF-16LE with its byte-order mark, as SharePoint writes
/// `customXml` items.
fn utf16(text: &str) -> Vec<u8> {
    let mut out = vec![0xFF, 0xFE];
    out.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
    out
}

#[test]
fn an_altchunk_html_doctype_is_not_a_dtd_refusal() {
    // Word's HTML import reads an altChunk that opens `<!DOCTYPE html>`;
    // only XML parts carry the DTD rule. Its tags still count.
    let html = b"<!DOCTYPE html><html><body><p>chunk</p></body></html>";
    let docx = package_with(&ok_body(), &[("word/afchunk.htm", html)]);
    let admitted = admit(&docx, InputLimits::default());
    assert!(admitted.is_ok(), "{admitted:?}");
    // 4 + 2 + 5 + 3 HTML elements (`html`, `body`, `p`).
    assert!(admit(&docx, capped(14)).is_ok());
    assert!(admit(&docx, capped(13)).is_err());
}

#[test]
fn debug_holds_the_element_budget() {
    // `jubarte debug` opens under the comparer's 4,000,000 budget.
    let dir = std::env::temp_dir().join(format!("jubarte-bomb-debug-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("flood.docx");
    std::fs::write(&file, flood(1_340_000)).unwrap();
    let started = std::time::Instant::now();
    let run = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("debug")
        .arg(&file)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_ne!(run.status.code(), Some(0), "{stderr}");
    assert!(
        stderr.contains("4000000 XML elements, the element limit"),
        "{stderr}"
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(60));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_utf16_part_charges_the_budget_as_its_text_reads() {
    // SharePoint writes `customXml` items in UTF-16; the tolerant parser
    // decodes them, so their tags count like any (pi review, 2026-10-31).
    let item = utf16(&format!("<vendor>{}</vendor>", "<w:p/>".repeat(40)));
    let docx = package_with(&ok_body(), &[("customXml/item1.xml", &item)]);
    // 4 + 2 + 5 for the package, 41 for the item.
    assert!(admit(&docx, capped(52)).is_ok());
    assert_eq!(admit(&docx, capped(51)).unwrap_err().code(), "INPUT_LIMIT");
    let error = view_error(&docx, capped(51));
    assert!(
        error.contains("customXml/item1.xml brings the package past 51"),
        "{error}"
    );
}

/// The compare settings with the story budget at `cap`.
fn story_capped(cap: u64) -> jubarte::comparer::WmlComparerSettings {
    let limits = InputLimitOverrides {
        max_story_units: Some(cap),
        ..InputLimitOverrides::default()
    }
    .apply(InputLimits::compare());
    jubarte::comparer::WmlComparerSettings::default().with_input_limits(limits)
}

/// The comparer turns every character into an atom of 2-4 KB: 66 MiB of
/// spaces in one `w:t` (B1, four elements) passed 6 GB in four seconds,
/// and 1 MiB of text peaked at 2.1-3.9 GB (word_exp cmp_scale,
/// 2026-10-31). A side's stories past `max_story_units` characters and
/// elements are refused before any atom is made, on either side.
#[test]
fn compare_holds_a_story_budget_of_characters_and_elements() {
    use jubarte::document_comparer::compare_documents_with_settings;
    let long = package(&document(
        "",
        &format!("<w:p><w:r><w:t>{}</w:t></w:r></w:p>", "x".repeat(100)),
    ));
    let short = package(&ok_body());
    for (a, b) in [(&long, &short), (&short, &long)] {
        let err = compare_documents_with_settings(a, b, &story_capped(60))
            .expect_err("a story past the budget")
            .to_string();
        assert!(
            err.contains("INPUT_LIMIT") && err.contains("60 story units"),
            "{err}"
        );
        compare_documents_with_settings(a, b, &story_capped(1_000)).expect("under the budget");
    }
    assert_eq!(InputLimits::compare().max_story_units, 2_000_000);
    assert_eq!(
        InputLimitOverrides::from_json(r#"{"max_story_units": 7}"#)
            .unwrap()
            .max_story_units,
        Some(7)
    );
}

/// A header is its own story under the same budget: 200,000 characters in
/// one is refused by name at a 100,000-unit cap.
#[test]
fn compare_holds_the_story_budget_in_a_header() {
    use jubarte::document_comparer::compare_documents_with_settings;
    use jubarte::opc::PartFs;
    let original = std::fs::read("tests/fixtures/f4/original.docx").unwrap();
    let mut pkg = PartFs::open(&original).unwrap();
    let header = String::from_utf8(pkg.part_bytes("word/header1.xml").unwrap().to_vec()).unwrap();
    let flood = format!(
        "<w:p><w:r><w:t>{}</w:t></w:r></w:p></w:hdr>",
        "x".repeat(200_000)
    );
    pkg.set_part(
        "word/header1.xml",
        header.replacen("</w:hdr>", &flood, 1).into_bytes(),
    );
    let flooded = pkg.to_zip().unwrap();
    for (a, b) in [(&flooded, &original), (&original, &flooded)] {
        let err = compare_documents_with_settings(a, b, &story_capped(100_000))
            .expect_err("a header past the budget")
            .to_string();
        assert!(
            err.contains("INPUT_LIMIT") && err.contains("header1.xml"),
            "{err}"
        );
    }
}
