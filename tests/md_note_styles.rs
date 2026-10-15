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

/// Normal, Word's note styles under their ids, LibreOffice's `Endnote`, a
/// localized `a5` named `footnote text`, `My Note`, and a `CommentText`
/// whose size (7.5pt) is the document's own.
fn styles_xml() -> String {
    let w = common::docx::W_NS;
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="FootnoteText"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="EndnoteText"><w:name w:val="endnote text"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="Endnote"><w:name w:val="Endnote"/><w:basedOn w:val="Normal"/><w:rPr><w:i/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="a5"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="My Note"><w:name w:val="My Note"/><w:basedOn w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="CommentText"><w:name w:val="annotation text"/><w:basedOn w:val="Normal"/><w:rPr><w:sz w:val="15"/></w:rPr></w:style></w:styles>"#
    )
}

/// A docx whose body references one note of `kind` (`footnote` or
/// `endnote`) per entry of `styles` (`None`: no `w:pStyle`).
fn docx(kind: &str, styles: &[Option<&str>]) -> Vec<u8> {
    let notes: Vec<Vec<Option<&str>>> = styles.iter().map(|s| vec![*s]).collect();
    docx_paragraphs(kind, &notes)
}

/// A docx with one note of `kind` per entry of `notes`, a paragraph per
/// entry of that note (its `w:pStyle`, `None` for none).
fn docx_paragraphs(kind: &str, notes_styles: &[Vec<Option<&str>>]) -> Vec<u8> {
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
    let styles = styles_xml();
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
