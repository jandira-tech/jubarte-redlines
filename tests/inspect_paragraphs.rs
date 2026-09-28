// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::inspect`: the read model an agent addresses edits against.
//! Paragraph text is the visible-run projection in body XML order (table
//! cells included, text boxes excluded); unsupported constructs are reported
//! as per-paragraph limitations, never as a refusal to read the document.

mod common;

use common::docx::{Part, docx, docx_with, para, run};
use jubarte::inspect::{InspectError, inspect_json, markdown, paragraphs, summary};

#[test]
fn text_joins_runs_split_by_word() {
    let body = format!(
        "<w:p>{}{}{}</w:p>",
        run("Third ", false, false, None),
        run("Par", false, false, None),
        run("ties.", false, false, None)
    );
    let paras = paragraphs(&docx(&body)).unwrap();
    assert_eq!(paras.len(), 1);
    assert_eq!(paras[0].text, "Third Parties.");
    assert_eq!(paras[0].index, 0);
    assert_eq!(paras[0].id, "body:p:0");
}

#[test]
fn tabs_breaks_and_page_breaks_project_exactly() {
    let body = r#"<w:p><w:r><w:t>A</w:t><w:tab/><w:t>B</w:t><w:br/><w:t>C</w:t></w:r></w:p><w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;
    let paras = paragraphs(&docx(body)).unwrap();
    assert_eq!(paras[0].text, "A\tB\nC");
    assert!(!paras[0].page_break);
    assert_eq!(paras[1].text, "");
    assert!(paras[1].page_break);
}

#[test]
fn deleted_and_moved_from_text_is_skipped_and_counted() {
    let body = r#"<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r><w:del w:id="1" w:author="a" w:date="2020-01-01T00:00:00Z"><w:r><w:delText>gone</w:delText></w:r></w:del><w:ins w:id="2" w:author="a" w:date="2020-01-01T00:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#;
    let bytes = docx(body);
    let paras = paragraphs(&bytes).unwrap();
    assert_eq!(paras[0].text, "keep new");
    assert_eq!(summary(&bytes).unwrap().revisions, 2);
}

#[test]
fn table_cell_paragraphs_keep_body_order_and_are_flagged() {
    let body = format!(
        "{}<w:tbl><w:tr><w:tc>{}</w:tc><w:tc>{}</w:tc></w:tr></w:tbl>{}",
        para("before"),
        para("cell one"),
        para("cell two"),
        para("after")
    );
    let paras = paragraphs(&docx(&body)).unwrap();
    let texts: Vec<&str> = paras.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(texts, ["before", "cell one", "cell two", "after"]);
    assert_eq!(
        paras.iter().map(|p| p.in_table).collect::<Vec<_>>(),
        [false, true, true, false]
    );
    let s = summary(&docx(&body)).unwrap();
    assert_eq!((s.paragraphs, s.tables), (4, 1));
}

#[test]
fn style_and_numbering_are_reported() {
    let body = r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:numPr><w:ilvl w:val="0"/><w:numId w:val="3"/></w:numPr></w:pPr><w:r><w:t>Scope</w:t></w:r></w:p><w:p><w:r><w:t>plain</w:t></w:r></w:p>"#;
    let paras = paragraphs(&docx(body)).unwrap();
    assert_eq!(paras[0].style.as_deref(), Some("Heading1"));
    assert!(paras[0].numbered);
    assert_eq!(paras[1].style, None);
    assert!(!paras[1].numbered);
}

#[test]
fn formatting_spans_cover_bold_italic_and_highlight_in_char_offsets() {
    let body = format!(
        "<w:p>{}{}{}</w:p>",
        run("(a) ", false, false, None),
        run("Confidentiality. ", true, false, None),
        run("Éé [fill in]", false, true, Some("yellow"))
    );
    let paras = paragraphs(&docx(&body)).unwrap();
    let p = &paras[0];
    assert_eq!(p.text, "(a) Confidentiality. Éé [fill in]");
    assert_eq!(p.runs.len(), 3);
    assert_eq!((p.runs[0].start, p.runs[0].end), (0, 4));
    assert!(!p.runs[0].bold);
    assert_eq!((p.runs[1].start, p.runs[1].end), (4, 21));
    assert!(p.runs[1].bold && !p.runs[1].italic);
    // Char offsets, not bytes: "Éé" is two chars.
    assert_eq!((p.runs[2].start, p.runs[2].end), (21, 33));
    assert!(p.runs[2].italic);
    assert_eq!(p.runs[2].highlight.as_deref(), Some("yellow"));
}

#[test]
fn unsupported_constructs_are_limitations_not_errors() {
    let body = r#"<w:p><w:r><w:t xml:space="preserve">Bullet </w:t></w:r><w:r><w:sym w:font="Symbol" w:char="F0B7"/></w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing/></mc:Choice><mc:Fallback><w:pict/></mc:Fallback></mc:AlternateContent><w:r><w:br w:type="column"/><w:t>next</w:t></w:r></w:p>"#;
    let paras = paragraphs(&docx(body)).unwrap();
    assert_eq!(paras.len(), 1);
    assert_eq!(paras[0].text, "Bullet \u{fffc}next");
    let lim = &paras[0].limitations;
    assert!(lim.iter().any(|l| l == "sym"), "{lim:?}");
    assert!(lim.iter().any(|l| l == "alternate_content"), "{lim:?}");
    assert!(lim.iter().any(|l| l == "column_break"), "{lim:?}");
}

#[test]
fn text_box_paragraphs_are_omitted_from_the_body_story() {
    let body = r#"<w:p><w:r><w:t>anchor</w:t></w:r><w:r><w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml"><v:textbox><w:txbxContent><w:p><w:r><w:t>inside box</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>"#;
    let paras = paragraphs(&docx(body)).unwrap();
    assert_eq!(paras.len(), 1);
    assert_eq!(paras[0].text, "anchor");
    assert!(paras[0].limitations.iter().any(|l| l == "text_box_omitted"));
}

#[test]
fn fields_hyperlinks_and_content_controls_project_text_and_flag() {
    let body = r#"<w:p><w:fldSimple w:instr="PAGE"><w:r><w:t>3</w:t></w:r></w:fldSimple><w:hyperlink r:id="rId9"><w:r><w:t>link</w:t></w:r></w:hyperlink><w:sdt><w:sdtContent><w:r><w:t>sdt</w:t></w:r></w:sdtContent></w:sdt><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>DATE</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>2026</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
    let paras = paragraphs(&docx(body)).unwrap();
    assert_eq!(paras[0].text, "3linksdt2026");
    let lim = &paras[0].limitations;
    for expected in ["field", "hyperlink", "content_control"] {
        assert!(lim.iter().any(|l| l == expected), "{expected} in {lim:?}");
    }
    assert_eq!(summary(&docx(body)).unwrap().fields, 2);
}

#[test]
fn not_a_docx_and_malformed_xml_are_errors() {
    // Admission refuses bytes that are not a ZIP before the reader sees them.
    assert!(matches!(
        paragraphs(b"not a zip"),
        Err(InspectError::Admission(a)) if a.code() == "INVALID_PACKAGE"
    ));
    let broken = docx("<w:p><w:r><w:t>unclosed</w:t></w:r>");
    assert!(matches!(paragraphs(&broken), Err(InspectError::Invalid(_))));
}

#[test]
fn summary_follows_relationships_for_comments_headers_and_notes() {
    let comments = r#"<?xml version="1.0" encoding="UTF-8"?><w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="0" w:author="a" w:date="2020-01-01T00:00:00Z"><w:p><w:r><w:t>hi</w:t></w:r></w:p></w:comment><w:comment w:id="1" w:author="a" w:date="2020-01-01T00:00:00Z"><w:p/></w:comment></w:comments>"#;
    let header = r#"<?xml version="1.0" encoding="UTF-8"?><w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>H</w:t></w:r></w:p></w:hdr>"#;
    let footnotes = r#"<?xml version="1.0" encoding="UTF-8"?><w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:footnote w:type="separator" w:id="-1"><w:p/></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p/></w:footnote><w:footnote w:id="1"><w:p><w:r><w:t>note</w:t></w:r></w:p></w:footnote></w:footnotes>"#;
    let bytes = docx_with(
        &para("body"),
        &[
            Part {
                name: "word/notes.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: comments,
            },
            Part {
                name: "word/header7.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: header,
            },
            Part {
                name: "word/footnotes.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
                xml: footnotes,
            },
        ],
    );
    let s = summary(&bytes).unwrap();
    assert_eq!(
        s.comments, 2,
        "comments part found through its rel, not its name"
    );
    assert_eq!(s.headers, 1);
    assert_eq!(s.footnotes, 1, "separator notes are not footnotes");
    assert_eq!(s.paragraphs, 1);
}

#[test]
fn markdown_prefixes_paragraph_ids_and_marks_formatting() {
    let body = format!(
        "<w:p>{}{}{}</w:p>{}",
        run("(a) ", false, false, None),
        run("Confidentiality.", true, false, None),
        run(" Fill in ", false, false, None),
        para("plain\tline")
    );
    let md = markdown(&docx(&body)).unwrap();
    assert_eq!(
        md,
        "[body:p:0] (a) **Confidentiality.** Fill in\n\n[body:p:1] plain\tline\n"
    );
}

#[test]
fn inspect_json_carries_schema_hash_summary_and_paragraphs() {
    let bytes = docx(&para("hello"));
    let json = inspect_json(&bytes).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["source_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(v["summary"]["paragraphs"], 1);
    assert_eq!(v["paragraphs"][0]["id"], "body:p:0");
    assert_eq!(v["paragraphs"][0]["text"], "hello");
    assert_eq!(v["paragraphs"][0]["runs"][0]["end"], 5);
}

#[test]
fn summary_reads_only_wordprocessingml_parts_for_revisions() {
    // A custom XML item the strict reader refuses (here a DTD) is not part
    // of the document text; it must not make the whole summary fail.
    let item = Part {
        name: "customXml/item1.xml",
        content_type: "application/xml",
        rel_type: "",
        xml: r#"<?xml version="1.0"?><!DOCTYPE vendor [<!ENTITY v "1">]><vendor>&v;</vendor>"#,
    };
    let bytes = docx_with(&para("Body"), &[item]);
    assert_eq!(summary(&bytes).unwrap().revisions, 0);
    assert!(inspect_json(&bytes).is_ok());
}

#[test]
fn explicit_off_run_properties_merge_with_unformatted_unicode_runs() {
    let body = r#"<w:p><w:r><w:rPr><w:b w:val="0"/><w:i w:val="off"/><w:u w:val="none"/><w:highlight w:val="none"/></w:rPr><w:t>é😀</w:t></w:r><w:r><w:t>尾</w:t></w:r><w:r><w:rPr><w:b/><w:i/><w:u w:val="single"/></w:rPr><w:t>Z</w:t></w:r></w:p>"#;
    let rows = paragraphs(&docx(body)).unwrap();
    let p = &rows[0];
    assert_eq!(p.text, "é😀尾Z");
    assert_eq!(p.runs.len(), 2);
    let plain = &p.runs[0];
    assert_eq!((plain.start, plain.end), (0, 3));
    assert!(!plain.bold && !plain.italic && !plain.underline);
    assert_eq!(plain.highlight, None);
    let styled = &p.runs[1];
    assert_eq!((styled.start, styled.end), (3, 4));
    assert!(styled.bold && styled.italic && styled.underline);
}

#[test]
fn tracking_settings_support_ooxml_boolean_spellings_and_reject_invalid_values() {
    for (attribute, expected) in [
        ("", true),
        (r#" w:val="true""#, true),
        (r#" w:val="1""#, true),
        (r#" w:val="on""#, true),
        (r#" w:val="false""#, false),
        (r#" w:val="0""#, false),
        (r#" w:val="off""#, false),
    ] {
        let xml = format!(
            r#"<w:settings xmlns:w="{}"><w:trackRevisions{attribute}/></w:settings>"#,
            common::docx::W_NS
        );
        let source = docx_with(
            &para("body"),
            &[Part {
                name: "word/custom-settings.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
                xml: &xml,
            }],
        );
        assert_eq!(
            summary(&source).unwrap().track_changes,
            expected,
            "{attribute}"
        );
    }
    let xml = format!(
        r#"<w:settings xmlns:w="{}"><w:trackRevisions w:val="maybe"/></w:settings>"#,
        common::docx::W_NS
    );
    let source = docx_with(
        &para("body"),
        &[Part {
            name: "word/settings.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
            xml: &xml,
        }],
    );
    assert!(
        matches!(summary(&source), Err(InspectError::Invalid(message)) if message.contains("trackRevisions"))
    );
}

#[test]
fn empty_body_has_no_phantom_paragraphs_or_markdown() {
    let source = docx("");
    assert!(paragraphs(&source).unwrap().is_empty());
    assert_eq!(markdown(&source).unwrap(), "");
    let facts = summary(&source).unwrap();
    assert_eq!(facts.paragraphs, 0);
    assert_eq!(facts.sections, 1);
    assert!(!facts.list_numbering);
}
