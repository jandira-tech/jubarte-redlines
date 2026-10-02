// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `append::append_documents`: B's body after A's, with B's relationships,
//! styles, numbering and notes carried into A's package.

mod common;

use common::docx::{Part, R_NS, docx, docx_with, docx_with_sect_pr, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::append::{AppendOptions, SectionBreak, append_documents};
use jubarte::inspect::{paragraphs, summary};

const PNG_1X1: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0x0D, b'I', b'H', b'D', b'R', 0, 0, 0,
    1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1F, 0x15, 0xC4, 0x89, 0, 0, 0, 0x0A, b'I', b'D', b'A', b'T',
    0x78, 0x9C, 0x63, 0, 1, 0, 0, 5, 0, 1, 0x0D, 0x0A, 0x2D, 0xB4, 0, 0, 0, 0, b'I', b'E', b'N',
    b'D', 0xAE, 0x42, 0x60, 0x82,
];

const DRAWING: &str = r#"<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="914400" cy="914400"/><wp:docPr id="1" name="Pic" descr="dot"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="dot.png"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rIdX0"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;

const SEPARATORS: &str = r#"<w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>"#;

const FOOTNOTES_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml";
const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";

fn with_image(text: &str) -> Vec<u8> {
    let image_rel = format!("{R_NS}/image");
    let doc = docx_with(
        &(para(text) + DRAWING),
        &[Part {
            name: "word/media/dot.png",
            content_type: "image/png",
            rel_type: &image_rel,
            xml: "",
        }],
    );
    common::docx::replace_entry(&doc, "word/media/dot.png", PNG_1X1)
}

fn b_with_image() -> Vec<u8> {
    with_image("B says hello.")
}

fn texts(docx: &[u8]) -> Vec<String> {
    paragraphs(docx)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn continuous() -> AppendOptions {
    AppendOptions {
        section_break: SectionBreak::Continuous,
        keep_sections: false,
    }
}

#[test]
fn text_follows_a_then_b_and_the_image_relationship_is_carried() {
    let a = docx(&para("A says hi."));
    let out = append_documents(&a, &b_with_image(), &AppendOptions::default()).unwrap();
    assert_word_valid_package(&out.docx);
    let texts = texts(&out.docx);
    assert_eq!(texts[0], "A says hi.");
    assert!(texts.iter().any(|t| t == "B says hello."), "{texts:?}");
    assert_eq!(summary(&out.docx).unwrap().images, 1);
    let rels = part_string(&out.docx, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains("/image"), "{rels}");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
}

#[test]
fn numbering_from_both_sides_keeps_distinct_ids() {
    let numbering = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="0"><w:nsid w:val="0A0B0C0D"/><w:multiLevelType w:val="hybridMultilevel"/><w:tmpl w:val="0A0B0C0E"/><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="•"/><w:lvlJc w:val="left"/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#;
    let listed = |text: &str| {
        let numbering_rel = format!("{R_NS}/numbering");
        docx_with(
            &format!(
                r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#
            ),
            &[Part {
                name: "word/numbering.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
                rel_type: &numbering_rel,
                xml: numbering,
            }],
        )
    };
    let out = append_documents(&listed("a item"), &listed("b item"), &continuous()).unwrap();
    assert_word_valid_package(&out.docx);
    let xml = part_string(&out.docx, "word/numbering.xml").unwrap();
    assert_eq!(xml.matches("<w:num ").count(), 2);
    assert_eq!(xml.matches("<w:abstractNum ").count(), 2);
    // Every w:abstractNum precedes every w:num (schema order).
    assert!(
        xml.rfind("<w:abstractNum ").unwrap() < xml.find("<w:num ").unwrap(),
        "{xml}"
    );
    // The copy is a list of its own: Word joins lists that share an nsid.
    assert_eq!(xml.matches("0A0B0C0D").count(), 1, "{xml}");
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert!(
        doc.contains(r#"<w:numId w:val="1""#) && doc.contains(r#"<w:numId w:val="2""#),
        "{doc}"
    );
}

#[test]
fn b_comments_are_dropped_with_a_warning() {
    let a = docx(&para("A."));
    let b = jubarte::edit::apply_plan(
        &docx(&para("B.")),
        &jubarte::edit::EditPlan::from_json(
            r#"{"schema_version":1,"author":"X","operations":[{"kind":"comment","paragraph":"body:p:0","text":"note"}]}"#,
        )
        .unwrap(),
    )
    .unwrap()
    .clean;
    let out = append_documents(&a, &b, &AppendOptions::default()).unwrap();
    assert_word_valid_package(&out.docx);
    assert_eq!(
        out.warnings,
        vec!["COMMENTS_DROPPED: 1 comment of B was not carried".to_string()]
    );
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert!(!doc.contains("comment"), "{doc}");
    assert_eq!(summary(&out.docx).unwrap().comments, 0);
    assert!(texts(&out.docx).iter().any(|t| t == "B."));
}

#[test]
fn next_page_puts_a_page_break_between_and_continuous_does_not() {
    let a = docx(&para("A."));
    let b = docx(&para("B."));
    let out = append_documents(&a, &b, &AppendOptions::default()).unwrap();
    assert_word_valid_package(&out.docx);
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert_eq!(doc.matches(r#"w:type="page""#).count(), 1, "{doc}");
    assert!(doc.find("A.").unwrap() < doc.find(r#"w:type="page""#).unwrap());
    assert!(doc.find(r#"w:type="page""#).unwrap() < doc.find("B.").unwrap());
    for options in [
        continuous(),
        AppendOptions {
            section_break: SectionBreak::None,
            keep_sections: false,
        },
    ] {
        let out = append_documents(&a, &b, &options).unwrap();
        let doc = part_string(&out.docx, "word/document.xml").unwrap();
        assert!(!doc.contains(r#"w:type="page""#), "{doc}");
        assert_eq!(texts(&out.docx), vec!["A.", "B."]);
        assert_eq!(summary(&out.docx).unwrap().sections, 1);
    }
}

fn with_footnote(body_text: &str, note_text: &str) -> Vec<u8> {
    let footnotes_rel = format!("{R_NS}/footnotes");
    let footnotes = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{SEPARATORS}<w:footnote w:id="1"><w:p><w:r><w:t>{note_text}</w:t></w:r></w:p></w:footnote></w:footnotes>"#
    );
    docx_with(
        &format!(
            r#"<w:p><w:r><w:t>{body_text}</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#
        ),
        &[Part {
            name: "word/footnotes.xml",
            content_type: FOOTNOTES_CT,
            rel_type: &footnotes_rel,
            xml: &footnotes,
        }],
    )
}

#[test]
fn footnotes_of_b_follow_a_s_with_fresh_ids() {
    let out = append_documents(
        &with_footnote("A body", "A note"),
        &with_footnote("B body", "B note"),
        &continuous(),
    )
    .unwrap();
    assert_word_valid_package(&out.docx);
    assert_eq!(summary(&out.docx).unwrap().footnotes, 2);
    let notes = part_string(&out.docx, "word/footnotes.xml").unwrap();
    assert!(
        notes.contains("A note") && notes.contains("B note"),
        "{notes}"
    );
    assert!(notes.contains(r#"w:id="2""#), "{notes}");
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert!(
        doc.contains(r#"<w:footnoteReference w:id="1""#)
            && doc.contains(r#"<w:footnoteReference w:id="2""#),
        "{doc}"
    );
    // B's reference names B's note.
    let b_ref = doc.find(r#"<w:footnoteReference w:id="2""#).unwrap();
    assert!(doc.find("B body").unwrap() < b_ref, "{doc}");
    let b_note = notes.find(r#"w:id="2""#).unwrap();
    assert!(b_note < notes.find("B note").unwrap(), "{notes}");
}

#[test]
fn a_without_notes_gets_a_notes_part_with_separators() {
    let out = append_documents(
        &docx(&para("A.")),
        &with_footnote("B body", "B note"),
        &continuous(),
    )
    .unwrap();
    assert_word_valid_package(&out.docx);
    assert_eq!(summary(&out.docx).unwrap().footnotes, 1);
    let rels = part_string(&out.docx, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains("/footnotes"), "{rels}");
    let ct = part_string(&out.docx, "[Content_Types].xml").unwrap();
    assert!(ct.contains(FOOTNOTES_CT), "{ct}");
    let notes = part_string(&out.docx, "word/footnotes.xml").unwrap();
    assert!(
        notes.contains("separator") && notes.contains("B note"),
        "{notes}"
    );
}

fn styles(styles: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{styles}</w:styles>"#
    )
}

fn styled(body: &str, sheet: &str) -> Vec<u8> {
    let styles_rel = format!("{R_NS}/styles");
    docx_with(
        body,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: &styles_rel,
            xml: &styles(sheet),
        }],
    )
}

fn styled_para(style: &str, text: &str) -> String {
    format!(r#"<w:p><w:pPr><w:pStyle w:val="{style}"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#)
}

#[test]
fn styles_pair_by_name_and_type_and_colliding_ids_are_renamed() {
    let a = styled(
        &styled_para("Heading1", "A head"),
        r#"<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="Fancy"><w:name w:val="Fancy"/><w:pPr><w:jc w:val="center"/></w:pPr></w:style>"#,
    );
    let b = styled(
        &(styled_para("Titulo1", "B head")
            + &styled_para("Fancy", "B fancy")
            + &styled_para("Extra", "B extra")),
        r#"<w:style w:type="paragraph" w:styleId="Titulo1"><w:name w:val="Heading 1"/></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="Fancy"><w:name w:val="Plain"/><w:pPr><w:jc w:val="right"/></w:pPr></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="Extra"><w:name w:val="Extra"/><w:basedOn w:val="Fancy"/></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="Unused"><w:name w:val="Unused"/></w:style>"#,
    );
    let out = append_documents(&a, &b, &continuous()).unwrap();
    assert_word_valid_package(&out.docx);
    let by_text = |text: &str| {
        paragraphs(&out.docx)
            .unwrap()
            .into_iter()
            .find(|p| p.text == text)
            .unwrap()
    };
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    // A built-in name pairs in any case: B's "Heading 1" is A's Heading1.
    assert!(!doc.contains("Titulo1"), "{doc}");
    assert_eq!(by_text("B head").style.as_deref(), Some("Heading1"));
    // Same id, different name: B's copy is renamed and keeps its look.
    assert!(doc.contains(r#"<w:pStyle w:val="FancyB""#), "{doc}");
    let sheet = part_string(&out.docx, "word/styles.xml").unwrap();
    assert!(sheet.contains(r#"w:styleId="FancyB""#), "{sheet}");
    assert!(sheet.contains(r#"<w:basedOn w:val="FancyB""#), "{sheet}");
    assert!(sheet.contains(r#"w:styleId="Extra""#), "{sheet}");
    assert!(!sheet.contains("Unused"), "{sheet}");
    assert_eq!(sheet.matches(r#"w:styleId="Heading1""#).count(), 1);
    assert_eq!(sheet.matches(r#"w:styleId="Fancy""#).count(), 1);
    assert_eq!(by_text("A head").style.as_deref(), Some("Heading1"));
}

#[test]
fn a_without_styles_receives_b_s_referenced_styles() {
    let b = styled(
        &styled_para("Extra", "B extra"),
        r#"<w:style w:type="paragraph" w:customStyle="1" w:styleId="Extra"><w:name w:val="Extra"/></w:style>"#,
    );
    let out = append_documents(&docx(&para("A.")), &b, &continuous()).unwrap();
    assert_word_valid_package(&out.docx);
    let sheet = part_string(&out.docx, "word/styles.xml").unwrap();
    assert!(sheet.contains(r#"w:styleId="Extra""#), "{sheet}");
    let rels = part_string(&out.docx, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains("/styles"), "{rels}");
}

#[test]
fn keep_sections_gives_b_its_own_section_and_header() {
    let header_rel = format!("{R_NS}/header");
    let header = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>B header</w:t></w:r></w:p></w:hdr>"#;
    let b = docx_with_sect_pr(
        &para("B landscape."),
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: &header_rel,
            xml: header,
        }],
        r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/><w:pgSz w:w="15840" w:h="12240" w:orient="landscape"/></w:sectPr>"#,
    );
    let a = docx(&para("A portrait."));
    let out = append_documents(
        &a,
        &b,
        &AppendOptions {
            section_break: SectionBreak::NextPage,
            keep_sections: true,
        },
    )
    .unwrap();
    assert_word_valid_package(&out.docx);
    let facts = summary(&out.docx).unwrap();
    assert_eq!(facts.sections, 2);
    assert_eq!(facts.headers, 1);
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    // A's section ends at the join; B's section closes the body.
    let inner = doc.find("<w:sectPr").unwrap();
    let last = doc.rfind("<w:sectPr").unwrap();
    assert!(doc.find("A portrait.").unwrap() < inner, "{doc}");
    assert!(inner < doc.find("B landscape.").unwrap(), "{doc}");
    assert!(doc[inner..last].contains(r#"w:w="12240""#), "{doc}");
    assert!(doc[last..].contains("landscape") && doc[last..].contains("headerReference"));
    assert!(!doc.contains(r#"w:type="page""#), "{doc}");
    let snapshot = jubarte::inspect::inspect_json(&out.docx).unwrap();
    assert!(snapshot.contains("B header"), "{snapshot}");

    let out = append_documents(
        &a,
        &b,
        &AppendOptions {
            section_break: SectionBreak::Continuous,
            keep_sections: true,
        },
    )
    .unwrap();
    assert_word_valid_package(&out.docx);
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    let last = doc.rfind("<w:sectPr").unwrap();
    assert!(
        doc[last..].contains(r#"<w:type w:val="continuous""#),
        "{doc}"
    );
}

#[test]
fn without_keep_sections_b_takes_a_s_last_section() {
    let b = docx_with_sect_pr(
        &para("B."),
        &[],
        r#"<w:sectPr><w:pgSz w:w="15840" w:h="12240" w:orient="landscape"/></w:sectPr>"#,
    );
    let out = append_documents(&docx(&para("A.")), &b, &continuous()).unwrap();
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert!(!doc.contains("landscape"), "{doc}");
    assert_eq!(summary(&out.docx).unwrap().sections, 1);
}

#[test]
fn external_hyperlinks_keep_their_target() {
    let b =
        docx(r#"<w:p><w:hyperlink r:id="rIdLink"><w:r><w:t>site</w:t></w:r></w:hyperlink></w:p>"#);
    let rels = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdLink" Type="{R_NS}/hyperlink" Target="https://example.com/x" TargetMode="External"/></Relationships>"#
    );
    let b = common::docx::replace_entry(&b, "word/_rels/document.xml.rels", rels.as_bytes());
    let out = append_documents(&docx(&para("A.")), &b, &continuous()).unwrap();
    assert_word_valid_package(&out.docx);
    let rels = part_string(&out.docx, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains("https://example.com/x"), "{rels}");
    assert!(rels.contains(r#"TargetMode="External""#), "{rels}");
}

#[test]
fn images_on_both_sides_keep_unique_drawing_ids() {
    let out = append_documents(
        &with_image("A pic."),
        &b_with_image(),
        &AppendOptions::default(),
    )
    .unwrap();
    assert_word_valid_package(&out.docx);
    assert_eq!(
        summary(&out.docx).unwrap().images,
        1,
        "same bytes, one part"
    );
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert!(
        doc.contains(r#"wp:docPr id="1""#) && doc.contains(r#"wp:docPr id="2""#),
        "{doc}"
    );
}

#[test]
fn revision_and_bookmark_ids_of_b_do_not_collide_with_a_s() {
    let marked = |text: &str| {
        docx(&format!(
            r#"<w:p><w:bookmarkStart w:id="0" w:name="mark{text}"/><w:ins w:id="1" w:author="X" w:date="2026-01-01T00:00:00Z"><w:r><w:t>{text}</w:t></w:r></w:ins><w:bookmarkEnd w:id="0"/></w:p>"#
        ))
    };
    let out = append_documents(&marked("A"), &marked("B"), &continuous()).unwrap();
    assert_word_valid_package(&out.docx);
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert_eq!(doc.matches(r#"w:id="1""#).count(), 1, "{doc}");
    assert_eq!(doc.matches(r#"w:id="0""#).count(), 2, "{doc}");
    let b_at = doc.find("markB").unwrap();
    let b_start_id = &doc[doc[..b_at].rfind("w:id=").unwrap()..b_at];
    let b_end = doc[b_at..].find("bookmarkEnd").unwrap() + b_at;
    let b_end_id = &doc[b_end..b_end + 30];
    let id = b_start_id.split('"').nth(1).unwrap();
    assert_ne!(id, "0");
    assert!(b_end_id.contains(&format!(r#"w:id="{id}""#)), "{doc}");
}

#[test]
fn input_that_is_not_a_package_is_refused() {
    let error = append_documents(b"not a zip", &docx(&para("B.")), &continuous()).unwrap_err();
    assert!(!error.to_string().is_empty());
    let error = append_documents(&docx(&para("A.")), b"", &continuous()).unwrap_err();
    assert!(!error.to_string().is_empty());
}

#[test]
fn options_read_from_json_with_defaults() {
    let options: AppendOptions = serde_json::from_str("{}").unwrap();
    assert_eq!(options.section_break, SectionBreak::NextPage);
    assert!(!options.keep_sections);
    let options: AppendOptions =
        serde_json::from_str(r#"{"section_break":"continuous","keep_sections":true}"#).unwrap();
    assert_eq!(options.section_break, SectionBreak::Continuous);
    assert!(options.keep_sections);
}

#[test]
fn cli_folds_three_documents_left() {
    let dir = std::env::temp_dir().join(format!("jubarte-append-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (name, text) in [("a", "First."), ("b", "Second."), ("c", "Third.")] {
        std::fs::write(dir.join(format!("{name}.docx")), docx(&para(text))).unwrap();
    }
    let out = dir.join("out.docx");
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(&dir)
        .args([
            "append",
            "a.docx",
            "b.docx",
            "c.docx",
            "-o",
            "out.docx",
            "--section-break",
            "continuous",
        ])
        .output()
        .unwrap();
    assert!(status.status.success(), "{status:?}");
    let bytes = std::fs::read(&out).unwrap();
    assert_word_valid_package(&bytes);
    assert_eq!(texts(&bytes), vec!["First.", "Second.", "Third."]);
    // Refuses to overwrite without --force.
    let again = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(&dir)
        .args(["append", "a.docx", "b.docx", "-o", "out.docx"])
        .output()
        .unwrap();
    assert!(!again.status.success());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_carried_header_brings_its_own_image_and_loses_its_comments() {
    let header_rel = format!("{R_NS}/header");
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="{R_NS}"><w:p><w:commentRangeStart w:id="7"/><w:r><w:t>Logo</w:t></w:r><w:commentRangeEnd w:id="7"/><w:r><w:commentReference w:id="7"/></w:r></w:p>{}</w:hdr>"#,
        DRAWING.replace("rIdX0", "rIdH1")
    );
    let b = docx_with_sect_pr(
        &para("B."),
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: &header_rel,
                xml: &header,
            },
            Part {
                name: "word/media/logo.png",
                content_type: "image/png",
                rel_type: "",
                xml: "",
            },
        ],
        r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>"#,
    );
    let b = common::docx::replace_entry(&b, "word/media/logo.png", PNG_1X1);
    let header_rels = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdH1" Type="{R_NS}/image" Target="media/logo.png"/></Relationships>"#
    );
    let b = add_entry(&b, "word/_rels/header1.xml.rels", header_rels.as_bytes());
    let out = append_documents(
        &docx(&para("A.")),
        &b,
        &AppendOptions {
            section_break: SectionBreak::NextPage,
            keep_sections: true,
        },
    )
    .unwrap();
    assert_word_valid_package(&out.docx);
    let facts = summary(&out.docx).unwrap();
    assert_eq!((facts.headers, facts.images, facts.comments), (1, 1, 0));
    assert_eq!(
        out.warnings,
        vec!["COMMENTS_DROPPED: 1 comment of B was not carried".to_string()]
    );
    let header = part_string(&out.docx, "word/header1.xml").unwrap();
    assert!(!header.contains("comment"), "{header}");
    let rels = part_string(&out.docx, "word/_rels/header1.xml.rels").unwrap();
    assert!(rels.contains("/image"), "{rels}");
}

/// `docx` with one more ZIP entry.
fn add_entry(docx: &[u8], name: &str, bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let entry = file.name().to_string();
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut data).unwrap();
        zip.start_file(entry.as_str(), opts).unwrap();
        zip.write_all(&data).unwrap();
    }
    zip.start_file(name, opts).unwrap();
    zip.write_all(bytes).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn a_kept_section_s_tracked_change_takes_a_fresh_id() {
    let a = docx(
        r#"<w:p><w:ins w:id="0" w:author="X" w:date="2026-01-01T00:00:00Z"><w:r><w:t>A</w:t></w:r></w:ins></w:p>"#,
    );
    let b = docx_with_sect_pr(
        &para("B."),
        &[],
        r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:sectPrChange w:id="0" w:author="X" w:date="2026-01-01T00:00:00Z"><w:sectPr/></w:sectPrChange></w:sectPr>"#,
    );
    let out = append_documents(
        &a,
        &b,
        &AppendOptions {
            section_break: SectionBreak::NextPage,
            keep_sections: true,
        },
    )
    .unwrap();
    assert_word_valid_package(&out.docx);
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert_eq!(doc.matches(r#"w:id="0""#).count(), 1, "{doc}");
}
