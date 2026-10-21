// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A note's paragraph style survives the round trip docx → view → docx:
//! its id line names it after the id (`<!-- endnotes.p2 Endnote -->`) only
//! when it is not Word's `footnote text` or `endnote text`, and the writer
//! stamps FootnoteText or EndnoteText only where the line is silent.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use std::io::Read as _;

const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Normal, Word's note styles under their ids (EndnoteText 12pt),
/// LibreOffice's `Endnote`, a
/// localized `a5` named `footnote text`, `My Note`, and a `CommentText`
/// whose size (7.5pt) is the document's own.
fn styles_xml() -> String {
    let w = common::docx::W_NS;
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="FootnoteText"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="EndnoteText"><w:name w:val="endnote text"/><w:basedOn w:val="Normal"/><w:rPr><w:sz w:val="24"/></w:rPr></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="Endnote"><w:name w:val="Endnote"/><w:basedOn w:val="Normal"/><w:rPr><w:i/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="a5"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="My Note"><w:name w:val="My Note"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="CommentText"><w:name w:val="annotation text"/><w:basedOn w:val="Normal"/><w:rPr><w:sz w:val="15"/></w:rPr></w:style></w:styles>"#
    )
}

/// A docx whose body references one note of `kind` (`footnote` or
/// `endnote`) per entry of `styles` (`None`: no `w:pStyle`).
fn docx(kind: &str, styles: &[Option<&str>]) -> Vec<u8> {
    let notes: Vec<Vec<Option<&str>>> = styles.iter().map(|s| vec![*s]).collect();
    docx_paragraphs(kind, &notes)
}

/// [`docx`] with `styles_xml` as the document's styles part.
fn docx_with_styles(kind: &str, styles: &[Option<&str>], styles_xml: &str) -> Vec<u8> {
    let notes: Vec<Vec<Option<&str>>> = styles.iter().map(|s| vec![*s]).collect();
    docx_paragraphs_in(kind, &notes, Some(styles_xml.to_string()))
}

/// A docx with one note of `kind` per entry of `notes`, a paragraph per
/// entry of that note (its `w:pStyle`, `None` for none), with `styles` as
/// its styles part when given.
fn docx_paragraphs(kind: &str, notes_styles: &[Vec<Option<&str>>]) -> Vec<u8> {
    docx_paragraphs_in(kind, notes_styles, None)
}

/// [`docx_paragraphs`] with `custom_styles` as the styles part.
fn docx_paragraphs_in(
    kind: &str,
    notes_styles: &[Vec<Option<&str>>],
    custom_styles: Option<String>,
) -> Vec<u8> {
    let (w, r) = (common::docx::W_NS, common::docx::R_NS);
    let mut body = String::new();
    let mut notes = format!(
        r#"<w:{kind} w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:{kind}><w:{kind} w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:{kind}>"#
    );
    for (i, styles) in notes_styles.iter().enumerate() {
        let id = i + 1;
        body.push_str(&format!(
            r#"<w:p><w:r><w:t>Clause {id}.</w:t></w:r><w:r><w:{kind}Reference w:id="{id}"/></w:r></w:p>"#
        ));
        let mut paragraphs = String::new();
        for (at, style) in styles.iter().enumerate() {
            let ppr = style
                .map(|s| format!(r#"<w:pPr><w:pStyle w:val="{s}"/></w:pPr>"#))
                .unwrap_or_default();
            let mark = if at == 0 {
                format!("<w:r><w:{kind}Ref/></w:r>")
            } else {
                String::new()
            };
            paragraphs.push_str(&format!(
                r#"<w:p>{ppr}{mark}<w:r><w:t xml:space="preserve"> Note {id}, part {at}.</w:t></w:r></w:p>"#
            ));
        }
        notes.push_str(&format!(r#"<w:{kind} w:id="{id}">{paragraphs}</w:{kind}>"#));
    }
    let notes = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:{kind}s xmlns:w="{w}" xmlns:r="{r}">{notes}</w:{kind}s>"#
    );
    let part_name = format!("word/{kind}s.xml");
    let content_type =
        format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}s+xml");
    let notes_rel = format!("{RELS}/{kind}s");
    let styles_rel = format!("{RELS}/styles");
    let styles = custom_styles.unwrap_or_else(styles_xml);
    let parts = [
        common::docx::Part {
            name: &part_name,
            content_type: &content_type,
            rel_type: &notes_rel,
            xml: &notes,
        },
        common::docx::Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: &styles_rel,
            xml: &styles,
        },
    ];
    common::docx::docx_with_sect(&body, &parts, "")
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

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name(name).unwrap().read_to_string(&mut xml).unwrap();
    xml
}

/// Each real note's first paragraph's `w:pStyle` in `kind`s.xml, `-` for
/// none; separators left out.
fn note_styles(docx: &[u8], kind: &str) -> Vec<String> {
    part(docx, &format!("word/{kind}s.xml"))
        .split(&format!("<w:{kind} "))
        .skip(1)
        .filter(|n| !n.contains("w:type="))
        .map(|n| {
            let p = &n[..n.find("</w:p>").unwrap_or(n.len())];
            p.split("<w:pStyle w:val=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .unwrap_or("-")
                .to_string()
        })
        .collect()
}

/// Every paragraph's `w:pStyle` of each real note in `kind`s.xml, `-` for
/// none; separators left out.
fn paragraph_styles(docx: &[u8], kind: &str) -> Vec<Vec<String>> {
    part(docx, &format!("word/{kind}s.xml"))
        .split(&format!("<w:{kind} "))
        .skip(1)
        .filter(|n| !n.contains("w:type="))
        .map(|n| {
            n.split("<w:p>")
                .chain(n.split("<w:p ").skip(1))
                .skip(1)
                .map(|p| {
                    let p = &p[..p.find("</w:p>").unwrap_or(p.len())];
                    p.split("<w:pStyle w:val=\"")
                        .nth(1)
                        .and_then(|s| s.split('"').next())
                        .unwrap_or("-")
                        .to_string()
                })
                .collect()
        })
        .collect()
}

#[test]
fn notes_in_word_style_print_no_style() {
    let docx = docx("footnote", &[Some("FootnoteText")]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains("<!-- footnotes.p"), "{view}");
    assert!(!view.contains("FootnoteText -->"), "{view}");
    assert_eq!(note_styles(&again, "footnote"), ["FootnoteText"]);
    assert_eq!(view_again, view);
}

#[test]
fn an_endnote_in_libreoffices_style_keeps_it() {
    // file_77: LibreOffice's `Endnote` came back as `endnote text`.
    let docx = docx("endnote", &[Some("Endnote"), Some("EndnoteText")]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(" Endnote -->"), "{view}");
    assert_eq!(note_styles(&again, "endnote"), ["Endnote", "EndnoteText"]);
    let styles = part(&again, "word/styles.xml");
    let definition = styles
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"Endnote\""))
        .unwrap_or_else(|| panic!("no Endnote: {styles}"));
    assert!(definition.contains("<w:i/>"), "{view}\n{definition}");
    assert_eq!(view_again, view);
}

#[test]
fn a_note_without_a_style_keeps_none() {
    let docx = docx("footnote", &[None]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(" Normal -->"), "{view}");
    assert_eq!(note_styles(&again, "footnote"), ["-"]);
    assert_eq!(view_again, view);
}

#[test]
fn a_localized_footnote_style_is_words_own() {
    let docx = docx("footnote", &[Some("a5")]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(!view.contains(" a5 -->"), "{view}");
    assert_eq!(note_styles(&again, "footnote"), ["FootnoteText"]);
    assert_eq!(view_again, view);
}

#[test]
fn a_note_style_id_with_a_space_is_quoted() {
    let docx = docx("endnote", &[Some("My Note")]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(r#" "My Note" -->"#), "{view}");
    assert_eq!(note_styles(&again, "endnote"), ["My Note"]);
    assert_eq!(view_again, view);
}

#[test]
fn a_note_keeps_each_paragraphs_style() {
    // Codex on #495: the first paragraph's style overwrote the others'.
    let docx = docx_paragraphs(
        "endnote",
        &[vec![Some("Endnote"), Some("EndnoteText"), None]],
    );
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(" Endnote, EndnoteText, Normal -->"), "{view}");
    assert_eq!(
        paragraph_styles(&again, "endnote"),
        [["Endnote", "EndnoteText", "-"]]
    );
    assert_eq!(view_again, view);
}

#[test]
fn a_note_in_one_style_names_it_once() {
    let docx = docx_paragraphs("endnote", &[vec![Some("Endnote"), Some("Endnote")]]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(" Endnote -->"), "{view}");
    assert_eq!(
        paragraph_styles(&again, "endnote"),
        [["Endnote", "Endnote"]]
    );
    assert_eq!(view_again, view);
}

#[test]
fn a_note_whose_later_paragraph_has_a_style_names_words_own_first() {
    let docx = docx_paragraphs("footnote", &[vec![Some("FootnoteText"), Some("My Note")]]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(r#" FootnoteText, "My Note" -->"#), "{view}");
    assert_eq!(
        paragraph_styles(&again, "footnote"),
        [["FootnoteText", "My Note"]]
    );
    assert_eq!(view_again, view);
}

#[test]
fn a_note_in_comment_text_keeps_its_definition() {
    // Codex on #495: CommentText was left out of the legend for notes too.
    let docx = docx("footnote", &[Some("CommentText")]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(" CommentText -->"), "{view}");
    let styles = part(&again, "word/styles.xml");
    let definition = styles
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"CommentText\""))
        .unwrap_or_else(|| panic!("no CommentText: {styles}"));
    assert!(
        definition.contains(r#"<w:sz w:val="15"/>"#),
        "{view}\n{definition}"
    );
    assert_eq!(view_again, view);
}

/// The docx a hand-written view writes, and its warnings.
fn write(view: &str) -> (Vec<u8>, Vec<String>) {
    let out = markdown_to_docx(view, &DocxOptions::default()).expect("the view reads");
    common::validity::assert_word_valid_package(&out.docx);
    (out.docx, out.warnings)
}

#[test]
fn a_footnote_in_endnote_text_keeps_its_definition() {
    // pi on #495 (F1): the legend left out every `EndnoteText` and
    // `FootnoteText`, so a footnote in the endnote style lost its own.
    let docx = docx("footnote", &[Some("EndnoteText")]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(" EndnoteText -->"), "{view}");
    assert_eq!(note_styles(&again, "footnote"), ["EndnoteText"]);
    let styles = part(&again, "word/styles.xml");
    let definition = styles
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"EndnoteText\""))
        .unwrap_or_else(|| panic!("no EndnoteText: {styles}"));
    assert!(
        definition.contains(r#"<w:sz w:val="24"/>"#),
        "{view}\n{definition}"
    );
    assert_eq!(view_again, view);
}

#[test]
fn a_note_id_lines_style_stays_with_its_note() {
    // pi on #495 (F2): a body id line after an endnote's line made the
    // next definition a footnote but kept the endnote's style for it.
    let view = "---\nview: tracked\n---\n\n<!-- p0 -->\nSee[^1].\n\n\
                <!-- endnotes.p0 Endnote -->\n\n<!-- p1 -->\nBody.\n\n[^1]: text\n";
    let (docx, _) = write(view);
    assert_eq!(note_styles(&docx, "footnote"), ["FootnoteText"]);
}

#[test]
fn a_note_whose_breaks_disagree_names_the_styles_it_drops() {
    // pi on #495 (F3): the one paragraph takes the first style; the
    // warning names the others.
    let view = "---\nview: tracked\n---\n\n<!-- p0 -->\nSee[^1].\n\n\
                <!-- footnotes.p0-p2 A, B, C -->\n[^1]: One<br>two.\n";
    let (docx, warnings) = write(view);
    assert_eq!(note_styles(&docx, "footnote"), ["A"]);
    assert!(
        warnings.iter().any(|w| w
            == "[^1]: its id line counts 3 paragraphs and its text 2 by its <br>s: \
                the <br>s are written as line breaks, in style A; B, C are not written"),
        "{warnings:?}"
    );
}

#[test]
fn a_tab_after_a_note_id_reads_as_a_space() {
    // pi on #495 (F4): the id line took the tab as whitespace, the style
    // list did not.
    let line = |styles: &str| {
        format!(
            "---\nview: tracked\n---\n\n<!-- p0 -->\nSee[^1].\n\n\
             <!-- footnotes.p0\t{styles} -->\n[^1]: text\n"
        )
    };
    let (docx, _) = write(&line("MyNote"));
    assert_eq!(note_styles(&docx, "footnote"), ["MyNote"]);
    let (docx, _) = write(&line("MyNote Other"));
    assert_eq!(note_styles(&docx, "footnote"), ["MyNote"]);
}

#[test]
fn an_empty_quoted_note_style_names_none() {
    // pi on #495 (F5): `""` wrote `w:pStyle w:val=""` and a style without
    // an id.
    let view = "---\nview: tracked\n---\n\n<!-- p0 -->\nSee[^1].\n\n\
                <!-- footnotes.p0 \"\" -->\n[^1]: text\n";
    let (docx, _) = write(view);
    assert_eq!(note_styles(&docx, "footnote"), ["FootnoteText"]);
    let styles = part(&docx, "word/styles.xml");
    assert!(!styles.contains(r#"w:styleId="""#), "{styles}");
}

/// A document that renames Word's note style keeps its name: `FootnoteText`
/// named `Footnote Text` is not Word's `footnote text`, so the view names
/// the style and the legend the name (ids-comments, #528).
#[test]
fn a_renamed_word_note_style_keeps_its_name() {
    let styles = styles_xml()
        .replace(r#"<w:name w:val="footnote text"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="EndnoteText""#, r#"<w:name w:val="Footnote Text"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="EndnoteText""#);
    assert!(
        styles.contains(r#"<w:name w:val="Footnote Text"/>"#),
        "{styles}"
    );
    let docx = docx_with_styles("footnote", &[Some("FootnoteText")], &styles);
    let (view, again, view_again) = round_trip(&docx);
    assert!(view.contains(" FootnoteText -->"), "{view}");
    assert!(view.contains("named Footnote Text"), "{view}");
    assert_eq!(note_styles(&again, "footnote"), ["FootnoteText"]);
    let styles = part(&again, "word/styles.xml");
    let definition = styles
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"FootnoteText\""))
        .unwrap_or_else(|| panic!("no FootnoteText: {styles}"));
    assert!(
        definition.contains(r#"<w:name w:val="Footnote Text"/>"#),
        "{definition}"
    );
    assert_eq!(view_again, view);
}
