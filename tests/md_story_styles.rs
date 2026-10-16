// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A story paragraph's style (headers and footers) survives the round trip
//! docx → view → docx: the view names it only when it is not the story's
//! default (Word's `header`/`footer`), and the writer stamps the default
//! only where the view is silent.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use std::io::Read as _;

const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const FOOTER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// A paragraph of `style` (`None`: no `w:pStyle`) holding `text`.
fn styled(style: Option<&str>, text: &str) -> String {
    let ppr = style
        .map(|s| format!(r#"<w:pPr><w:pStyle w:val="{s}"/></w:pPr>"#))
        .unwrap_or_default();
    format!(r#"<w:p>{ppr}<w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#)
}

/// A styles part: Normal, the header and footer styles under `header_id`
/// and `footer_id`, and List Paragraph.
fn styles_xml(header_id: &str, footer_id: &str) -> String {
    let w = common::docx::W_NS;
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="{header_id}"><w:name w:val="header"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="{footer_id}"><w:name w:val="footer"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="ListParagraph"><w:name w:val="List Paragraph"/><w:basedOn w:val="Normal"/></w:style></w:styles>"#
    )
}

/// A docx whose section has a default header holding `header` paragraphs
/// and a default footer holding `footer` paragraphs, with `styles`.
fn docx(header: &str, footer: &str, styles: &str) -> Vec<u8> {
    let (w, r) = (common::docx::W_NS, common::docx::R_NS);
    let hdr = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="{w}" xmlns:r="{r}">{header}</w:hdr>"#
    );
    let ftr = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="{w}" xmlns:r="{r}">{footer}</w:ftr>"#
    );
    let header_rel = format!("{RELS}/header");
    let footer_rel = format!("{RELS}/footer");
    let styles_rel = format!("{RELS}/styles");
    let parts = [
        common::docx::Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: &header_rel,
            xml: &hdr,
        },
        common::docx::Part {
            name: "word/footer1.xml",
            content_type: FOOTER_CT,
            rel_type: &footer_rel,
            xml: &ftr,
        },
        common::docx::Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: &styles_rel,
            xml: styles,
        },
    ];
    common::docx::docx_with_sect(
        &common::docx::para("Body."),
        &parts,
        r#"<w:headerReference w:type="default" r:id="rIdX0"/><w:footerReference w:type="default" r:id="rIdX1"/>"#,
    )
}

/// The view of `docx`, the docx it writes, and that docx's view; both
/// views without their `sha256:` line, which names the bytes.
fn round_trip(docx: &[u8]) -> (String, Vec<u8>, String) {
    let options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    let without_sha = |view: String| {
        view.lines()
            .filter(|l| !l.starts_with("sha256: "))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let view = read(docx, &options).expect("the docx reads").markdown;
    let again = markdown_to_docx(&view, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let view_again = read(&again, &options).expect("the docx reads").markdown;
    (without_sha(view), again, without_sha(view_again))
}

/// The `w:pStyle` values of each paragraph of the part whose name starts
/// with `prefix` (`word/header`), `-` for a paragraph without one.
fn part_styles(docx: &[u8], prefix: &str) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let name = zip
        .file_names()
        .filter_map(Result::ok)
        .find(|n| n.starts_with(prefix) && n.ends_with(".xml"))
        .unwrap_or_else(|| panic!("no {prefix}*.xml part"))
        .to_string();
    let mut xml = String::new();
    zip.by_name(&name)
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let mut styles = Vec::new();
    let mut rest = xml.as_str();
    while let Some(at) = [rest.find("<w:p>"), rest.find("<w:p ")]
        .into_iter()
        .flatten()
        .min()
    {
        let end = rest[at..].find("</w:p>").map_or(rest.len(), |i| at + i);
        let paragraph = &rest[at..end];
        styles.push(
            paragraph
                .split("<w:pStyle w:val=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .unwrap_or("-")
                .to_string(),
        );
        rest = &rest[end..];
    }
    styles
}

#[test]
fn a_story_paragraph_without_a_style_keeps_none() {
    let styles = styles_xml("Header", "Footer");
    let docx = docx(
        &styled(Some("Header"), "Draft"),
        &styled(None, "Page"),
        &styles,
    );
    let (view, again, view_again) = round_trip(&docx);
    let footer = view
        .lines()
        .find(|l| l.contains("id: footer1"))
        .unwrap_or_else(|| panic!("{view}"));
    assert!(footer.contains(", style: Normal"), "{footer}");
    let header = view
        .lines()
        .find(|l| l.contains("id: header1"))
        .unwrap_or_else(|| panic!("{view}"));
    assert!(
        !header.contains("style"),
        "the default prints no clause: {header}"
    );
    assert_eq!(part_styles(&again, "word/footer"), ["-"]);
    assert_eq!(part_styles(&again, "word/header"), ["Header"]);
    assert_eq!(view_again, view);
}

#[test]
fn story_paragraphs_of_different_styles_keep_each() {
    let styles = styles_xml("Header", "Footer");
    let docx = docx(
        &format!(
            "{}{}",
            styled(Some("Header"), "Draft"),
            styled(Some("ListParagraph"), "Item")
        ),
        &format!(
            "{}{}",
            styled(Some("ListParagraph"), "One"),
            styled(Some("ListParagraph"), "Two")
        ),
        &styles,
    );
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(", styles: [Header, ListParagraph]"), "{view}");
    assert!(view.contains(", style: ListParagraph"), "{view}");
    assert_eq!(
        part_styles(&again, "word/header"),
        ["Header", "ListParagraph"]
    );
    assert_eq!(
        part_styles(&again, "word/footer"),
        ["ListParagraph", "ListParagraph"]
    );
    assert_eq!(view_again, view);
}

#[test]
fn a_localized_header_style_is_the_default() {
    // Word names its own styles; the id is the language's (`a3`).
    let styles = styles_xml("a3", "a4");
    let docx = docx(
        &styled(Some("a3"), "Draft"),
        &styled(Some("a4"), "Page"),
        &styles,
    );
    let (view, again, _) = round_trip(&docx);
    assert!(!view.contains("style: "), "{view}");
    assert!(!view.contains("styles: "), "{view}");
    assert_eq!(part_styles(&again, "word/header"), ["Header"]);
    assert_eq!(part_styles(&again, "word/footer"), ["Footer"]);
}

/// A docx with only a default header holding `header`, with `styles`.
fn header_only(header: &str, styles: &str) -> Vec<u8> {
    let (w, r) = (common::docx::W_NS, common::docx::R_NS);
    let hdr = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="{w}" xmlns:r="{r}">{header}</w:hdr>"#
    );
    let header_rel = format!("{RELS}/header");
    let styles_rel = format!("{RELS}/styles");
    let parts = [
        common::docx::Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: &header_rel,
            xml: &hdr,
        },
        common::docx::Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: &styles_rel,
            xml: styles,
        },
    ];
    common::docx::docx_with_sect(
        &common::docx::para("Body."),
        &parts,
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

/// The styles part of `docx`.
fn styles_of(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name("word/styles.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

#[test]
fn a_style_the_view_defines_has_a_name_without_commas() {
    // pi on #491: `w:name` may hold no comma (Word reads one as an alias
    // list; the schema's pattern is `[^,]*`), `w:styleId` may.
    let styles = styles_xml("Header", "Footer").replace(
        "</w:styles>",
        r#"<w:style w:type="paragraph" w:styleId="List,Para"><w:name w:val="List Para"/></w:style></w:styles>"#,
    );
    let docx = docx(
        &format!(
            "{}{}",
            styled(Some("Header"), "Draft"),
            styled(Some("List,Para"), "Item")
        ),
        &styled(Some("Footer"), "Page"),
        &styles,
    );
    let (_, again, _) = round_trip(&docx);
    assert_eq!(part_styles(&again, "word/header"), ["Header", "List,Para"]);
    let styles = styles_of(&again);
    let definition = styles
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"List,Para\""))
        .unwrap_or_else(|| panic!("no List,Para: {styles}"));
    let name = definition
        .split("<w:name w:val=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap_or_else(|| panic!("{definition}"));
    assert!(!name.contains(','), "{definition}");
}

#[test]
fn normal_names_normal_when_another_style_is_the_default() {
    // pi on #491: with BodyText the default, an explicit Normal must stay
    // Normal, and a paragraph without a style must stay without one.
    let w = common::docx::W_NS;
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="BodyText"><w:name w:val="Body Text"/></w:style><w:style w:type="paragraph" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Header"><w:name w:val="header"/></w:style><w:style w:type="paragraph" w:styleId="Footer"><w:name w:val="footer"/></w:style></w:styles>"#
    );
    let docx = docx(
        &format!(
            "{}{}",
            styled(Some("Header"), "Draft"),
            styled(Some("Normal"), "Second")
        ),
        &styled(None, "Page"),
        &styles,
    );
    let (view, again, view_again) = round_trip(&docx);
    assert_eq!(part_styles(&again, "word/header"), ["Header", "Normal"]);
    assert_eq!(part_styles(&again, "word/footer"), ["-"]);
    assert_eq!(view_again, view);
}

#[test]
fn a_header_in_the_footer_style_gets_its_definition() {
    // pi on #491: a header-only document whose header paragraph is in
    // Footer: the output must define Footer.
    let docx = header_only(
        &styled(Some("Footer"), "Draft"),
        &styles_xml("Header", "Footer"),
    );
    let (_, again, _) = round_trip(&docx);
    assert_eq!(part_styles(&again, "word/header"), ["Footer"]);
    assert!(
        styles_of(&again).contains("w:styleId=\"Footer\""),
        "{}",
        styles_of(&again)
    );
}

#[test]
fn a_trailing_paragraph_a_mark_brings_back_keeps_its_style() {
    // Codex on #491: an empty last paragraph kept for its tracked mark is
    // made again; the visible ones' `style: Normal` must not reach it.
    let mark = r#"<w:p><w:pPr><w:pStyle w:val="Header"/><w:rPr><w:ins w:id="7" w:author="A" w:date="2026-01-01T00:00:00Z"/></w:rPr></w:pPr></w:p>"#;
    let docx = docx(
        &format!("{}{mark}", styled(Some("Normal"), "Draft")),
        &styled(Some("Footer"), "Page"),
        &styles_xml("Header", "Footer"),
    );
    let (view, again, view_again) = round_trip(&docx);
    assert_eq!(
        part_styles(&again, "word/header"),
        ["-", "Header"],
        "{view}"
    );
    assert_eq!(view_again, view);
}

#[test]
fn a_style_only_a_header_uses_keeps_its_look() {
    // Codex on #491: the legend describes the styles stories use too, so
    // a header-only style is not rebuilt bare on Normal.
    let styles = styles_xml("Header", "Footer").replace(
        "</w:styles>",
        r#"<w:style w:type="paragraph" w:customStyle="1" w:styleId="BrandHeader"><w:name w:val="Brand Header"/><w:basedOn w:val="Header"/><w:rPr><w:b/></w:rPr></w:style></w:styles>"#,
    );
    let docx = docx(
        &styled(Some("BrandHeader"), "Draft"),
        &styled(Some("Footer"), "Page"),
        &styles,
    );
    let (view, again, view_again) = round_trip(&docx);
    assert_eq!(part_styles(&again, "word/header"), ["BrandHeader"]);
    let styles = styles_of(&again);
    let definition = styles
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"BrandHeader\""))
        .unwrap_or_else(|| panic!("no BrandHeader: {styles}"));
    assert!(definition.contains("<w:b/>"), "{view}\n{definition}");
    assert!(definition.contains("Brand Header"), "{definition}");
    assert_eq!(view_again, view);
}

#[test]
fn a_character_style_a_header_paragraph_names_stays_out_of_the_legend() {
    // `LineNumber` is a character style a header paragraph names (a real
    // corpus shape): the legend describes paragraph styles only, so the
    // view writes no paragraph style named Word's `line number`.
    let styles = styles_xml("Header", "Footer").replace(
        "</w:styles>",
        r#"<w:style w:type="character" w:styleId="LineNumber"><w:name w:val="line number"/></w:style></w:styles>"#,
    );
    let docx = docx(
        &styled(Some("LineNumber"), "Draft"),
        &styled(Some("Footer"), "Page"),
        &styles,
    );
    let (view, again, _) = round_trip(&docx);
    assert_eq!(part_styles(&again, "word/header"), ["LineNumber"]);
    assert!(!view.contains("  LineNumber:"), "{view}");
}

#[test]
fn normal_is_described_once_when_no_style_is_marked_default() {
    // A header paragraph with no style under a styles part that marks no
    // default: `Normal` is the legend's default line, not a second one.
    let styles = styles_xml("Header", "Footer").replace(r#" w:default="1""#, "");
    let docx = docx(
        &format!("{}{}", styled(Some("Header"), "Draft"), styled(None, "Two")),
        &styled(Some("Footer"), "Page"),
        &styles,
    );
    let (view, again, view_again) = round_trip(&docx);
    assert_eq!(view.matches("\n  Normal:").count(), 1, "{view}");
    assert_eq!(part_styles(&again, "word/header"), ["Header", "-"]);
    assert_eq!(view_again, view);
}
