// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A symbol run between two letters stays in the view and comes back as the
//! same `w:sym`.

mod common;

use common::docx::{docx, part_string};
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

const SPAN: &str = r#"<sym font="Wingdings" char="F0FC"/>"#;

fn view_of(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown
    .lines()
    .filter(|line| !line.starts_with("sha256: ") && !line.starts_with("body: "))
    .collect::<Vec<_>>()
    .join("\n")
}

fn symbol_paragraph(inner: &str) -> Vec<u8> {
    docx(&format!("<w:p>{inner}</w:p>"))
}

fn keeps_the_symbol(docx: &[u8]) {
    let first = view_of(docx);
    assert!(first.contains(SPAN), "the view prints the symbol:\n{first}");
    let copy = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view writes")
        .docx;
    let second = view_of(&copy);
    assert_eq!(second, first, "the view comes back\n{first}\n---\n{second}");
    let xml = part_string(&copy, "word/document.xml").expect("document.xml");
    assert_eq!(
        xml.matches("<w:sym ").count(),
        1,
        "one symbol comes back\n{xml}"
    );
    assert!(
        xml.contains(r#"w:font="Wingdings""#) && xml.contains(r#"w:char="F0FC""#),
        "the font and the character come back\n{xml}"
    );
}

#[test]
fn a_symbol_between_letters_round_trips() {
    keeps_the_symbol(&symbol_paragraph(
        r#"<w:r><w:t>a</w:t></w:r><w:r><w:sym w:font="Wingdings" w:char="F0FC"/></w:r><w:r><w:t>b</w:t></w:r>"#,
    ));
}

#[test]
fn a_symbol_inside_an_insertion_round_trips() {
    keeps_the_symbol(&symbol_paragraph(
        r#"<w:r><w:t>a</w:t></w:r><w:ins w:id="1" w:author="A" w:date="2026-10-01T09:00:00Z"><w:r><w:sym w:font="Wingdings" w:char="F0FC"/></w:r></w:ins><w:r><w:t>b</w:t></w:r>"#,
    ));
}

/// cursor 4499761867: a paragraph that holds only the symbol starts with
/// the tag, which CommonMark reads as an HTML block; the symbol still comes
/// back.
#[test]
fn a_paragraph_of_only_a_symbol_round_trips() {
    keeps_the_symbol(&symbol_paragraph(
        r#"<w:r><w:sym w:font="Wingdings" w:char="F0FC"/></w:r>"#,
    ));
}

/// cursor 4499778454, jules 4499916208: a font name with quotes, markup or
/// CriticMarkup delimiters stays the font name. It ends neither the
/// attribute nor the tag, and it forges no change.
#[test]
fn a_font_name_with_markup_stays_the_font_name() {
    let font = r#"A"B&lt;i&gt; {++x++} \`*_[|]~"#;
    let docx = symbol_paragraph(&format!(
        r#"<w:r><w:t>a</w:t></w:r><w:r><w:sym w:font="{}" w:char="F0FC"/></w:r><w:r><w:t>b</w:t></w:r>"#,
        font.replace('"', "&quot;")
    ));
    let first = view_of(&docx);
    assert!(!first.contains("{++"), "no change is forged:\n{first}");
    let copy = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view writes")
        .docx;
    let second = view_of(&copy);
    assert_eq!(second, first, "the view comes back\n{first}\n---\n{second}");
    let xml = part_string(&copy, "word/document.xml").expect("document.xml");
    assert_eq!(xml.matches("<w:sym ").count(), 1, "{xml}");
    assert!(
        xml.contains(r#"w:font="A&quot;B&lt;i&gt; {++x++} \`*_[|]~""#),
        "the font name comes back whole\n{xml}"
    );
    assert!(!xml.contains("<w:ins "), "no change is forged\n{xml}");
    assert!(xml.contains(">a<") && xml.contains(">b<"), "{xml}");
}

/// cursor 4517245568: a page break with only a symbol after it prints in
/// place, before the symbol, and comes back there: on the id line it came
/// back after the symbol.
#[test]
fn a_page_break_before_a_symbol_stays_before_it() {
    let docx = symbol_paragraph(
        r#"<w:r><w:t>a</w:t></w:r><w:r><w:br w:type="page"/></w:r><w:r><w:sym w:font="Wingdings" w:char="F0FC"/></w:r>"#,
    );
    let first = view_of(&docx);
    assert!(
        first.contains(&format!(r#"a<br type="page"/>{SPAN}"#)),
        "the break prints in place:\n{first}"
    );
    let copy = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view writes")
        .docx;
    let xml = part_string(&copy, "word/document.xml").expect("document.xml");
    let br = xml.find(r#"<w:br w:type="page"/>"#).expect("the break");
    let sym = xml.find("<w:sym ").expect("the symbol");
    assert!(br < sym, "the break comes back before the symbol\n{xml}");
    assert_eq!(view_of(&copy), first);
}
