// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1876: `convert` draws a text `mc:AlternateContent` as Word does. Word
//! applies the Markup Compatibility rule: the first `mc:Choice` whose
//! `Requires` it understands, else the `mc:Fallback`, and draws that branch
//! once, in a run, a table cell or the body. The renderer drew both
//! branches of a run-level wrapper and nothing of a block-level one.

mod common;

use common::docx::{docx, para, part_string, replace_entry};
use jubarte::convert::{PdfOptions, docx_render_report};

const W14: &str = "http://schemas.microsoft.com/office/word/2010/wordml";
const W15: &str = "http://schemas.microsoft.com/office/word/2012/wordml";
const W16SE: &str = "http://schemas.microsoft.com/office/word/2015/wordml/symex";
/// A namespace Word does not know.
const X1: &str = "urn:jubarte:probe:unknown";

/// A package whose document element also declares `w14`, `w15`, `w16se`
/// and `x1`.
fn package(body: &str) -> Vec<u8> {
    let built = docx(body);
    let document = part_string(&built, "word/document.xml").unwrap().replacen(
        "<w:document ",
        &format!(
            r#"<w:document xmlns:w14="{W14}" xmlns:w15="{W15}" xmlns:w16se="{W16SE}" xmlns:x1="{X1}" "#
        ),
        1,
    );
    replace_entry(&built, "word/document.xml", document.as_bytes())
}

/// An `mc:AlternateContent` with one `mc:Choice` per `(requires, content)`,
/// then a `mc:Fallback` holding `fallback`.
fn alternate(choices: &[(&str, &str)], fallback: &str) -> String {
    let choices: String = choices
        .iter()
        .map(|(requires, content)| {
            format!(r#"<mc:Choice Requires="{requires}">{content}</mc:Choice>"#)
        })
        .collect();
    format!(
        "<mc:AlternateContent>{choices}<mc:Fallback>{fallback}</mc:Fallback></mc:AlternateContent>"
    )
}

fn run(text: &str) -> String {
    format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
}

/// Every page's painted text, in order.
fn painted(docx: &[u8]) -> String {
    docx_render_report(docx, PdfOptions::default())
        .expect("the document renders")
        .pages
        .iter()
        .map(|page| page.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_draws_once(text: &str, wanted: &[&str], unwanted: &[&str]) {
    for want in wanted {
        assert_eq!(text.matches(want).count(), 1, "{want:?} once in {text:?}");
    }
    for not in unwanted {
        assert!(!text.contains(not), "{not:?} is not drawn: {text:?}");
    }
}

/// A wrapper around a run, around a paragraph in a table cell and around a
/// body paragraph each draws its Choice once and no Fallback.
#[test]
fn a_text_wrapper_draws_its_choice_once_at_every_level() {
    let cell = alternate(
        &[("w14", &para("Choice cell paragraph"))],
        &para("Fallback cell paragraph"),
    );
    let body = format!(
        r#"<w:p>{lead}{inline}</w:p><w:tbl><w:tblPr><w:tblW w:type="auto" w:w="0"/></w:tblPr><w:tblGrid><w:gridCol w:w="8640"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:type="dxa" w:w="8640"/></w:tcPr>{cell_lead}{cell}</w:tc></w:tr></w:tbl>{block}"#,
        lead = run("Lead: "),
        inline = alternate(&[("w14", &run("Choice run"))], &run("Fallback run")),
        cell_lead = para("Cell lead"),
        block = alternate(
            &[("w14", &para("Choice body paragraph"))],
            &para("Fallback body paragraph")
        ),
    );
    let text = painted(&package(&body));
    assert_draws_once(
        &text,
        &[
            "Lead: Choice run",
            "Cell lead",
            "Choice cell paragraph",
            "Choice body paragraph",
        ],
        &["Fallback"],
    );
}

/// Of two Choices Word understands, the first wins.
#[test]
fn the_first_understood_choice_wins() {
    let body = alternate(
        &[("w15", &para("Choice A")), ("w14", &para("Choice B"))],
        &para("Fallback paragraph"),
    );
    let text = painted(&package(&body));
    assert_draws_once(&text, &["Choice A"], &["Choice B", "Fallback"]);
}

/// A Choice gated on a namespace Word does not know, though the document
/// declares it, is passed over: Word 16 draws the next Choice, else the
/// Fallback. (A prefix the document never declares is no test: Word cannot
/// read such a file, and `jubarte validate` says so.)
#[test]
fn a_choice_on_a_namespace_word_does_not_know_is_passed_over() {
    let body = format!(
        "<w:p>{}</w:p>{}",
        alternate(&[("x1", &run("Unknown run"))], &run("Fallback run")),
        alternate(
            &[
                ("x1", &para("Unknown paragraph")),
                ("w14", &para("Known paragraph"))
            ],
            &para("Fallback paragraph")
        )
    );
    let text = painted(&package(&body));
    assert_draws_once(
        &text,
        &["Fallback run", "Known paragraph"],
        &["Unknown", "Fallback paragraph"],
    );
}

/// Word draws a `w16se:symEx` emoji: a font and a character. The renderer
/// draws no such symbol, so it takes the Fallback, which holds the same
/// character as text, as Word writes it (`a1cfa10c32`'s "☺"). Taking the
/// Choice Word takes would draw nothing.
#[test]
fn an_emoji_symbol_draws_its_fallback_character() {
    let body = format!(
        r#"<w:p><w:r><w:t xml:space="preserve">Cake </w:t></w:r><w:r><w:rPr>{fonts}</w:rPr>{symbol}</w:r></w:p>"#,
        fonts = alternate(
            &[(
                "w16se",
                r#"<w:rFonts w:ascii="Didot" w:hAnsi="Didot" w:hint="cs"/>"#
            )],
            r#"<w:rFonts w:ascii="Apple Color Emoji" w:hAnsi="Apple Color Emoji"/>"#
        ),
        symbol = alternate(
            &[(
                "w16se",
                r#"<w16se:symEx w16se:font="Apple Color Emoji" w16se:char="263A"/>"#
            )],
            "<w:t>\u{263A}</w:t>"
        ),
    );
    let text = painted(&package(&body));
    assert_draws_once(&text, &["Cake \u{263A}"], &[]);
}
