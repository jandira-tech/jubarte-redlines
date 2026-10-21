// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The `watermark` operation writes Word's own VML text watermark (Insert >
//! Watermark) into every default header, creating a header when a section
//! has none. It is header content, not a revision: the clean copy and the
//! redline both carry it and neither tracks it.

mod common;

use common::docx::{Part, R_NS, W_NS, docx, docx_with_sect, docx_with_sect_pr, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::convert::{PdfOptions, docx_to_png};
use jubarte::edit::{EditPlan, apply_plan};

const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";

fn plan(ops: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"A","operations":[{ops}]}}"#
    ))
    .unwrap()
}

fn with_header(header_body: &str) -> Vec<u8> {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office">{header_body}</w:hdr>"#
    );
    let rel = format!("{R_NS}/header");
    docx_with_sect(
        &para("Body text."),
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: &rel,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

#[test]
fn a_watermark_lands_in_a_new_default_header_and_paints() {
    let source = docx(&para("Body text."));
    let out = apply_plan(&source, &plan(r#"{"kind":"watermark","text":"DRAFT"}"#)).unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let header = part_string(&out.clean, "word/header1.xml").expect("a default header was created");
    assert!(
        header.contains(
            r#"<v:textpath style="font-family:&quot;Calibri&quot;;font-size:1pt" string="DRAFT"/>"#
        ),
        "{header}"
    );
    assert!(header.contains("PowerPlusWaterMarkObject1"), "{header}");
    assert!(header.contains(r#"o:spid="_x0000_s2049""#), "{header}");
    assert!(
        header.contains("width:527.85pt;height:131.95pt;rotation:315;"),
        "{header}"
    );
    assert!(header.contains(r##"fillcolor="#C0C0C0""##), "{header}");
    assert!(
        header.contains(r#"<w:docPartGallery w:val="Watermarks"/>"#),
        "{header}"
    );
    let doc = part_string(&out.clean, "word/document.xml").unwrap();
    assert!(
        doc.contains(r#"<w:sectPr><w:headerReference w:type="default""#),
        "{doc}"
    );
    let types = part_string(&out.clean, "[Content_Types].xml").unwrap();
    assert!(types.contains(r#"PartName="/word/header1.xml""#), "{types}");
    let rels = part_string(&out.clean, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains(r#"Target="header1.xml""#), "{rels}");
    // It paints: rendering is deterministic, and the watermarked first page
    // differs from the unmarked one.
    let render = |bytes: &[u8]| docx_to_png(bytes, PdfOptions::default(), 50.0).unwrap();
    let before = render(&source);
    assert_eq!(before[0], render(&source)[0], "rendering is deterministic");
    assert_ne!(before[0], render(&out.clean)[0], "the watermark paints");
}

#[test]
fn the_redline_carries_the_watermark_untracked() {
    let source = docx(&para("Body text."));
    let out = apply_plan(&source, &plan(r#"{"kind":"watermark","text":"DRAFT"}"#)).unwrap();
    let header =
        part_string(&out.redline, "word/header1.xml").expect("the redline carries the new header");
    assert!(header.contains(r#"string="DRAFT""#), "{header}");
    assert!(!header.contains("<w:ins "), "{header}");
    let doc = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(
        doc.contains(r#"<w:headerReference w:type="default""#),
        "{doc}"
    );
    assert_eq!(
        out.report.revisions.inserted, 0,
        "{:?}",
        out.report.revisions
    );
    assert_eq!(out.report.operations[0].kind, "watermark");
    assert_eq!(out.report.operations[0].status, "ok");
}

#[test]
fn a_second_watermark_is_refused() {
    let source = docx(&para("x"));
    let e = apply_plan(
        &source,
        &plan(r#"{"kind":"watermark","text":"DRAFT"},{"kind":"watermark","text":"COPY"}"#),
    )
    .unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(e.operation.as_deref(), Some("op-2"));
    assert!(
        e.message.contains("remove the existing watermark first"),
        "{}",
        e.message
    );
}

#[test]
fn an_existing_default_header_keeps_its_text_and_gains_the_watermark() {
    let source = with_header(r#"<w:p><w:r><w:t>Confidential</w:t></w:r></w:p>"#);
    let out = apply_plan(
        &source,
        &plan(
            r#"{"kind":"watermark","text":"COPY","color":"ff0000","diagonal":false,"font":"Arial"},{"kind":"replace","paragraph":{"story":"header1","index":0},"find":"Confidential","replacement":"Secret"}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    assert!(
        part_string(&out.clean, "word/header2.xml").is_none(),
        "no second header"
    );
    let header = part_string(&out.clean, "word/header1.xml").unwrap();
    assert!(header.contains("Secret"), "{header}");
    assert!(header.contains(r#"string="COPY""#), "{header}");
    assert!(header.contains("font-family:&quot;Arial&quot;"), "{header}");
    assert!(header.contains(r##"fillcolor="#FF0000""##), "{header}");
    assert!(!header.contains("rotation:"), "horizontal: {header}");
    let doc = part_string(&out.clean, "word/document.xml").unwrap();
    assert_eq!(doc.matches("<w:headerReference").count(), 1, "{doc}");
    let redline = part_string(&out.redline, "word/header1.xml").unwrap();
    assert!(redline.contains(r#"string="COPY""#), "{redline}");
    assert!(
        redline.contains("<w:ins "),
        "the header text edit is tracked: {redline}"
    );
    let control = &redline[..redline.find("</w:sdt>").expect("watermark control")];
    assert!(
        !control.contains("<w:ins "),
        "the watermark is not tracked: {redline}"
    );
}

#[test]
fn a_watermark_and_a_commented_deletion_share_the_comparison_base() {
    let source = docx(&(para("Keep.") + &para("Drop this.")));
    let out = apply_plan(
        &source,
        &plan(
            r#"{"kind":"delete_paragraph","paragraph":{"index":1},"comment":"Not needed."},{"kind":"watermark","text":"DRAFT"}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let header = part_string(&out.redline, "word/header1.xml").unwrap();
    assert!(header.contains(r#"string="DRAFT""#), "{header}");
    assert!(!header.contains("<w:ins "), "{header}");
    let doc = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(doc.contains("<w:del "), "{doc}");
    assert_eq!(out.report.comments_added, 1);
}

#[test]
fn a_header_that_already_holds_a_watermark_is_refused() {
    let source = with_header(
        r##"<w:p><w:r><w:pict><v:shape id="PowerPlusWaterMarkObject357" o:spid="_x0000_s2049" type="#_x0000_t136"><v:textpath string="OLD"/></v:shape></w:pict></w:r></w:p>"##,
    );
    let e = apply_plan(&source, &plan(r#"{"kind":"watermark","text":"NEW"}"#)).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(e.operation.as_deref(), Some("op-1"));
}

#[test]
fn each_section_without_a_default_header_of_its_own_is_covered() {
    // Section 1 (mid-document sectPr) and section 2 (final sectPr) have no
    // headers: section 1 gets one, section 2 inherits it as in Word.
    let sect = r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#;
    let body = format!(
        r#"<w:p><w:pPr>{sect}</w:pPr><w:r><w:t>One.</w:t></w:r></w:p>{}"#,
        para("Two.")
    );
    let source = docx_with_sect_pr(&body, &[], sect);
    let out = apply_plan(&source, &plan(r#"{"kind":"watermark","text":"DRAFT"}"#)).unwrap();
    assert_word_valid_package(&out.clean);
    let doc = part_string(&out.clean, "word/document.xml").unwrap();
    assert_eq!(doc.matches("<w:headerReference").count(), 1, "{doc}");
    assert!(part_string(&out.clean, "word/header2.xml").is_none());
}

#[test]
fn two_sections_with_their_own_headers_each_get_a_shape() {
    let header = |text: &str| {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:hdr>"#
        )
    };
    let (h1, h2) = (header("First"), header("Second"));
    let rel = format!("{R_NS}/header");
    let sect = |rid: &str| {
        format!(
            r#"<w:sectPr><w:headerReference w:type="default" r:id="{rid}"/><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="708" w:footer="708" w:gutter="0"/></w:sectPr>"#
        )
    };
    let body = format!(
        r#"<w:p><w:pPr>{}</w:pPr><w:r><w:t>One.</w:t></w:r></w:p>{}"#,
        sect("rIdX0"),
        para("Two.")
    );
    let source = docx_with_sect_pr(
        &body,
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: &rel,
                xml: &h1,
            },
            Part {
                name: "word/header2.xml",
                content_type: HEADER_CT,
                rel_type: &rel,
                xml: &h2,
            },
        ],
        &sect("rIdX1"),
    );
    let out = apply_plan(&source, &plan(r#"{"kind":"watermark","text":"DRAFT"}"#)).unwrap();
    assert_word_valid_package(&out.clean);
    let first = part_string(&out.clean, "word/header1.xml").unwrap();
    let second = part_string(&out.clean, "word/header2.xml").unwrap();
    assert!(first.contains("PowerPlusWaterMarkObject1"), "{first}");
    assert!(first.contains(r#"o:spid="_x0000_s2049""#), "{first}");
    assert!(second.contains("PowerPlusWaterMarkObject2"), "{second}");
    assert!(second.contains(r#"o:spid="_x0000_s2050""#), "{second}");
    // A4, 1in margins: text width 451.3pt, so 0.9 of it and a 4:1 ratio.
    assert!(first.contains("width:406.17pt;height:101.54pt;"), "{first}");
}

#[test]
fn bad_parameters_are_refused() {
    let source = docx(&para("x"));
    for (op, what) in [
        (r#"{"kind":"watermark","text":""}"#, "empty text"),
        (
            r#"{"kind":"watermark","text":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}"#,
            "65 characters",
        ),
        (r#"{"kind":"watermark","text":"a\tb"}"#, "control character"),
        (
            r#"{"kind":"watermark","text":"X","color":"red"}"#,
            "named colour",
        ),
        (
            r##"{"kind":"watermark","text":"X","color":"#C0C0C0"}"##,
            "hash",
        ),
        (r#"{"kind":"watermark","text":"X","font":""}"#, "empty font"),
        (
            r#"{"kind":"watermark","text":"X","font":"A\"B"}"#,
            "quote in font",
        ),
    ] {
        let e = apply_plan(&source, &plan(op)).unwrap_err();
        assert_eq!(e.code, "INVALID_PLAN", "{what}: {e:?}");
        assert_eq!(e.operation.as_deref(), Some("op-1"), "{what}");
    }
    let e = EditPlan::from_json(
        r#"{"schema_version":1,"author":"A","operations":[{"kind":"watermark","text":"X","paragraph":"p:0"}]}"#,
    )
    .unwrap_err();
    assert_eq!(e.code, "INVALID_PLAN", "{e:?}");
}

#[test]
fn markup_characters_in_the_text_are_escaped() {
    let source = docx(&para("x"));
    let out = apply_plan(&source, &plan(r#"{"kind":"watermark","text":"A & <B>"}"#)).unwrap();
    assert_word_valid_package(&out.clean);
    let header = part_string(&out.clean, "word/header1.xml").unwrap();
    assert!(header.contains(r#"string="A &amp; &lt;B&gt;""#), "{header}");
}

#[test]
fn two_sections_sharing_one_header_mark_it_once() {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>Shared</w:t></w:r></w:p></w:hdr>"#
    );
    let rel = format!("{R_NS}/header");
    // No page size either: Word's default page is Letter.
    let sect = r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/></w:sectPr>"#;
    let body = format!(
        r#"<w:p><w:pPr>{sect}</w:pPr><w:r><w:t>One.</w:t></w:r></w:p>{}"#,
        para("Two.")
    );
    let source = docx_with_sect_pr(
        &body,
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: &rel,
            xml: &header,
        }],
        sect,
    );
    let out = apply_plan(&source, &plan(r#"{"kind":"watermark","text":"DRAFT"}"#)).unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let header = part_string(&out.clean, "word/header1.xml").unwrap();
    assert_eq!(header.matches("<v:shape ").count(), 1, "{header}");
    assert!(
        header.contains("width:527.85pt;height:131.95pt;"),
        "{header}"
    );
    assert!(header.contains("Shared"), "{header}");
}

/// The same package with every `word/` part moved under `doc/`: the main
/// part is wherever the package relationship points.
fn relocated(docx: &[u8]) -> Vec<u8> {
    use std::io::{Cursor, Read, Write};
    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        let name = file.name().replacen("word/", "doc/", 1);
        let data = match name.as_str() {
            "[Content_Types].xml" | "_rels/.rels" => String::from_utf8(data)
                .unwrap()
                .replace("/word/", "/doc/")
                .replace("\"word/", "\"doc/")
                .into_bytes(),
            _ => data,
        };
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&data).unwrap();
    }
    out.finish().unwrap().into_inner()
}

#[test]
fn a_new_header_sits_beside_a_main_part_outside_word() {
    let source = relocated(&docx(&para("Body text.")));
    assert!(part_string(&source, "doc/document.xml").is_some());
    let out = apply_plan(&source, &plan(r#"{"kind":"watermark","text":"DRAFT"}"#)).unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let header = part_string(&out.clean, "doc/header1.xml").expect("header beside the main part");
    assert!(header.contains(r#"string="DRAFT""#), "{header}");
    assert!(part_string(&out.clean, "word/header1.xml").is_none());
    let types = part_string(&out.clean, "[Content_Types].xml").unwrap();
    assert!(types.contains(r#"PartName="/doc/header1.xml""#), "{types}");
}

#[test]
fn under_keep_the_redline_carries_the_watermark_too() {
    let source = docx(&para("Body text."));
    let plan = EditPlan::from_json(
        r#"{"schema_version":1,"author":"A","existing_revisions":"keep","operations":[{"kind":"watermark","text":"DRAFT"}]}"#,
    )
    .unwrap();
    let out = apply_plan(&source, &plan).unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    for (name, bytes) in [("clean", &out.clean), ("redline", &out.redline)] {
        let header = part_string(bytes, "word/header1.xml")
            .unwrap_or_else(|| panic!("{name} has the new header"));
        assert!(header.contains(r#"string="DRAFT""#), "{name}: {header}");
        assert!(!header.contains("<w:ins "), "{name}: {header}");
    }
}
