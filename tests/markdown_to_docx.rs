// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::markdown::markdown_to_docx`: Markdown blocks become Word
//! paragraphs, lists, tables and notes; CriticMarkup becomes tracked changes
//! and comments that accept and reject as the Markdown says.

mod common;

use std::io::Cursor;

use image::ImageEncoder as _;

use common::docx::part_string;
use common::validity::assert_word_valid_package;
use jubarte::comparer::{WmlComparerRevisionType, WmlComparerSettings};
use jubarte::document_comparer::{accept_revisions, get_revisions, reject_revisions};
use jubarte::inspect;
use jubarte::markdown::{DocxOptions, TrackChanges, markdown_to_docx};

fn write(markdown: &str) -> Vec<u8> {
    write_with(markdown, &DocxOptions::default())
}

fn write_with(markdown: &str, options: &DocxOptions<'_>) -> Vec<u8> {
    let docx = markdown_to_docx(markdown, options).unwrap().docx;
    assert_word_valid_package(&docx);
    ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(
        docx.clone(),
    ))
    .expect("the OOXML SDK reads the package");
    docx
}

/// Body paragraph texts.
fn texts(docx: &[u8]) -> Vec<String> {
    inspect::paragraphs(docx)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn accepted(docx: &[u8]) -> Vec<String> {
    texts(&accept_revisions(docx).unwrap())
}

fn rejected(docx: &[u8]) -> Vec<String> {
    texts(&reject_revisions(docx).unwrap())
}

fn revisions(docx: &[u8]) -> Vec<(WmlComparerRevisionType, String)> {
    get_revisions(docx, &WmlComparerSettings::default())
        .unwrap()
        .into_iter()
        .map(|r| (r.revision_type, r.text.unwrap_or_default()))
        .collect()
}

fn document_xml(docx: &[u8]) -> String {
    part_string(docx, "word/document.xml").unwrap()
}

#[test]
fn every_block_kind_is_written() {
    let docx = write(
        "---\ntitle: The Terms\nauthor: Jane Roe\n---\n\n\
         # Terms\n\n\
         Plain, **bold**, *italic*, ~~struck~~, `code` and a [link](https://example.com/a?b=1&c=2).\n\n\
         ## Lists\n\n\
         - one\n- two\n  - nested\n\n\
         3. three\n4. four\n\n\
         - [x] done\n- [ ] open\n\n\
         > quoted\n\n\
         ```\nlet x = 1;\n\tindented\n```\n\n\
         | Left | Right |\n|:-----|------:|\n| a | b |\n\n\
         A note.[^n]\n\n\
         ---\n\n\
         Line one  \nline two\n\n\
         [^n]: The note's text.\n",
    );
    let paragraphs = inspect::paragraphs(&docx).unwrap();
    let find = |text: &str| {
        paragraphs
            .iter()
            .find(|p| p.text == text)
            .unwrap_or_else(|| panic!("no paragraph {text:?} in {paragraphs:#?}"))
    };
    assert_eq!(find("Terms").style.as_deref(), Some("Heading1"));
    assert_eq!(find("Lists").style.as_deref(), Some("Heading2"));
    let formatted = find("Plain, bold, italic, struck, code and a link.");
    assert!(formatted.runs.iter().any(|r| r.bold));
    assert!(formatted.runs.iter().any(|r| r.italic));
    for item in ["one", "two", "nested", "three", "four"] {
        let p = find(item);
        assert!(p.numbered, "{item}");
        assert_eq!(p.style.as_deref(), Some("ListParagraph"), "{item}");
    }
    find("\u{2612} done");
    find("\u{2610} open");
    assert_eq!(find("quoted").style.as_deref(), Some("Quote"));
    assert_eq!(
        find("let x = 1;\n\tindented").style.as_deref(),
        Some("SourceCode")
    );
    assert!(find("a").in_table && find("b").in_table && find("Right").in_table);
    find("A note.");
    find("Line one\nline two");

    let xml = document_xml(&docx);
    assert!(xml.contains("<w:strike/>"));
    assert!(xml.contains("<w:rStyle w:val=\"VerbatimChar\"/>"));
    assert!(xml.contains("<w:hyperlink r:id="));
    assert!(xml.contains("<w:jc w:val=\"right\"/>"));
    assert!(xml.contains("<w:tblHeader/>"));
    assert!(xml.contains("<w:pBdr><w:bottom"));
    let rels = part_string(&docx, "word/_rels/document.xml.rels").unwrap();
    assert!(
        rels.contains("Target=\"https://example.com/a?b=1&amp;c=2\""),
        "{rels}"
    );
    assert!(rels.contains("TargetMode=\"External\""));
    let notes = part_string(&docx, "word/footnotes.xml").unwrap();
    assert!(notes.contains("The note's text."));
    assert!(notes.contains("<w:footnoteRef/>"));
    let numbering = part_string(&docx, "word/numbering.xml").unwrap();
    assert!(
        numbering.contains("<w:startOverride w:val=\"3\"/>"),
        "{numbering}"
    );
    let core = part_string(&docx, "docProps/core.xml").unwrap();
    assert!(core.contains("<dc:title>The Terms</dc:title>"));
    assert!(core.contains("<dc:creator>Jane Roe</dc:creator>"));
    let styles = part_string(&docx, "word/styles.xml").unwrap();
    for id in [
        "Heading1",
        "Heading2",
        "ListParagraph",
        "Quote",
        "SourceCode",
        "TableGrid",
    ] {
        assert!(styles.contains(&format!("w:styleId=\"{id}\"")), "{id}");
    }
}

#[test]
fn critic_markup_becomes_revisions_and_comments() {
    let docx = write(
        "The fee is {~~ten~>twelve~~} dollars{++ per month++}{--, payable yearly--}.\n\n\
         See {==clause 4==}{>>Is this the right clause?<<} and {++the annex++}{>>Added at the client's request.<<}.\n\n\
         A point comment.{>>Check.<<}\n",
    );
    let found = revisions(&docx);
    let inserted: Vec<&str> = found
        .iter()
        .filter(|(t, _)| *t == WmlComparerRevisionType::Inserted)
        .map(|(_, text)| text.as_str())
        .collect();
    let deleted: Vec<&str> = found
        .iter()
        .filter(|(t, _)| *t == WmlComparerRevisionType::Deleted)
        .map(|(_, text)| text.as_str())
        .collect();
    assert_eq!(inserted, ["twelve", " per month", "the annex"]);
    assert_eq!(deleted, ["ten", ", payable yearly"]);
    assert_eq!(
        accepted(&docx),
        [
            "The fee is twelve dollars per month.",
            "See clause 4 and the annex.",
            "A point comment."
        ]
    );
    assert_eq!(
        rejected(&docx),
        [
            "The fee is ten dollars, payable yearly.",
            "See clause 4 and .",
            "A point comment."
        ]
    );
    let comments = part_string(&docx, "word/comments.xml").unwrap();
    for text in [
        "Is this the right clause?",
        "Added at the client's request.",
        "Check.",
    ] {
        assert!(comments.contains(text), "{text} in {comments}");
    }
    let xml = document_xml(&docx);
    assert_eq!(xml.matches("<w:commentRangeStart ").count(), 3);
    assert_eq!(xml.matches("<w:commentReference ").count(), 3);
    assert!(xml.contains("w:author=\"Redline\""));
    assert!(
        !xml.contains("<w:highlight"),
        "a commented highlight is the comment's range"
    );
}

#[test]
fn a_highlight_without_a_comment_is_highlighted_text() {
    let xml = document_xml(&write("A {==marked==} word.\n"));
    assert!(xml.contains("<w:highlight w:val=\"yellow\"/>"));
    assert!(!xml.contains("commentRangeStart"));
}

#[test]
fn a_change_across_a_paragraph_break_holds_the_break() {
    let docx = write("A{++\n\nB++}\n\nC\n");
    assert_eq!(accepted(&docx), ["A", "B", "C"]);
    assert_eq!(rejected(&docx), ["A", "C"]);

    let docx = write("A{--\n\nB--}\n\nC\n");
    assert_eq!(accepted(&docx), ["A", "C"]);
    assert_eq!(rejected(&docx), ["A", "B", "C"]);
}

#[test]
fn a_paragraph_that_is_one_change_whole_is_added_or_removed_whole() {
    let docx = write("A\n\n{++New paragraph.++}\n\nC\n");
    assert_eq!(accepted(&docx), ["A", "New paragraph.", "C"]);
    assert_eq!(rejected(&docx), ["A", "C"]);

    let docx = write("A\n\n{--Old paragraph.--}\n\nC\n");
    assert_eq!(accepted(&docx), ["A", "C"]);
    assert_eq!(rejected(&docx), ["A", "Old paragraph.", "C"]);

    let docx = write("# {++New heading++}\n\nText\n");
    assert_eq!(accepted(&docx), ["New heading", "Text"]);
    assert_eq!(rejected(&docx), ["Text"]);

    // Part of a paragraph, or a substitution, leaves the paragraph in place.
    let docx = write("A\n\n{++Half++} a paragraph.\n\n{~~Old~>New~~}\n\nC\n");
    assert_eq!(rejected(&docx), ["A", " a paragraph.", "Old", "C"]);
}

#[test]
fn the_last_paragraph_added_or_removed_whole_moves_its_change_to_the_mark_before() {
    let docx = write("A\n\n{++B++}\n");
    assert_eq!(accepted(&docx), ["A", "B"]);
    assert_eq!(rejected(&docx), ["A"]);

    let docx = write("A\n\n{--B--}\n");
    assert_eq!(accepted(&docx), ["A"]);
    assert_eq!(rejected(&docx), ["A", "B"]);

    // A document that is one inserted paragraph keeps an empty paragraph.
    let docx = write("{++Only++}\n");
    assert_eq!(accepted(&docx), ["Only"]);
    assert_eq!(rejected(&docx), [""]);
}

#[test]
fn a_change_crosses_emphasis_and_links() {
    let docx = write("Keep {++**bold** and [a link](https://example.com)++} here.\n");
    assert_eq!(accepted(&docx), ["Keep bold and a link here."]);
    assert_eq!(rejected(&docx), ["Keep  here."]);
    let xml = document_xml(&docx);
    // The change is re-opened inside the hyperlink: w:ins cannot hold one.
    assert!(xml.contains("<w:hyperlink r:id=\"rId"));
    assert!(!xml.contains("00:00:00Z\"><w:hyperlink"), "{xml}");
    assert!(
        xml.contains("<w:hyperlink r:id=\"rId3\" w:history=\"1\"><w:ins "),
        "{xml}"
    );
}

#[test]
fn a_footnote_whose_reference_is_changed_is_changed_with_it() {
    let docx = write("Text{++ with a note[^1]++}.\n\n[^1]: Added note.\n");
    let notes = part_string(&docx, "word/footnotes.xml").unwrap();
    assert!(notes.contains("<w:ins "), "{notes}");
    let found = revisions(&docx);
    assert!(
        found.iter().any(
            |(t, text)| *t == WmlComparerRevisionType::Inserted && text.contains("Added note.")
        ),
        "{found:?}"
    );
    let accept = accept_revisions(&docx).unwrap();
    assert!(
        part_string(&accept, "word/footnotes.xml")
            .unwrap()
            .contains("Added note.")
    );
    let reject = reject_revisions(&docx).unwrap();
    assert!(!document_xml(&reject).contains("footnoteReference"));
}

#[test]
fn accept_and_reject_write_clean_documents() {
    let markdown = "The fee is {~~ten~>twelve~~} dollars.{>>Why?<<}\n";
    for (choice, text) in [
        (TrackChanges::Accept, "The fee is twelve dollars."),
        (TrackChanges::Reject, "The fee is ten dollars."),
    ] {
        let docx = write_with(
            markdown,
            &DocxOptions {
                track_changes: choice,
                ..DocxOptions::default()
            },
        );
        assert_eq!(texts(&docx), [text], "{choice:?}");
        let xml = document_xml(&docx);
        assert!(
            !xml.contains("<w:ins ") && !xml.contains("<w:del "),
            "{choice:?}"
        );
    }
}

#[test]
fn without_critic_the_delimiters_are_text() {
    let docx = write_with(
        "Literal {++braces++} and {~~a~>b~~}.\n",
        &DocxOptions {
            critic: false,
            ..DocxOptions::default()
        },
    );
    assert_eq!(texts(&docx), ["Literal {++braces++} and {~~a~>b~~}."]);
    assert!(revisions(&docx).is_empty());
}

#[test]
fn escaped_or_unpaired_delimiters_are_text() {
    let docx = write("An \\{++escaped++} one, a {++ lone opener.\n");
    assert_eq!(texts(&docx), ["An {++escaped++} one, a {++ lone opener."]);
    assert!(revisions(&docx).is_empty());
}

#[test]
fn a_reference_document_lends_its_styles_page_and_headers() {
    let reference = std::fs::read("tests/fixtures/redline-inpi/original-new.docx").unwrap();
    let docx = write_with(
        "# Heading\n\nBody with a [link](https://example.com) and a note.[^a]\n\n- item\n\n[^a]: Note.\n",
        &DocxOptions {
            reference: Some(&reference),
            ..DocxOptions::default()
        },
    );
    assert_eq!(
        texts(&docx),
        ["Heading", "Body with a link and a note.", "item"]
    );
    // Headers, footers, theme and numbering come along; the reference's
    // text, links and notes do not.
    for part in [
        "word/header1.xml",
        "word/footer1.xml",
        "word/theme/theme1.xml",
    ] {
        assert!(part_string(&docx, part).is_some(), "{part}");
    }
    let xml = document_xml(&docx);
    assert!(
        xml.contains("<w:headerReference"),
        "the reference's section is kept"
    );
    let reference_styles = part_string(&reference, "word/styles.xml").unwrap();
    assert!(
        part_string(&docx, "word/styles.xml")
            .unwrap()
            .starts_with(&reference_styles[..200])
    );
    let rels = part_string(&docx, "word/_rels/document.xml.rels").unwrap();
    assert_eq!(rels.matches("/hyperlink\"").count(), 1, "{rels}");
    let notes = part_string(&docx, "word/footnotes.xml").unwrap();
    assert!(notes.contains("Note."));
    assert!(
        !notes.contains("Target"),
        "only the separators and the new note"
    );
    let numbering = part_string(&docx, "word/numbering.xml").unwrap();
    let reference_numbering = part_string(&reference, "word/numbering.xml").unwrap();
    assert!(numbering.len() > reference_numbering.len());
}

#[test]
fn images_are_embedded_through_the_loader() {
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&[0u8; 200 * 100], 200, 100, image::ExtendedColorType::L8)
        .unwrap();
    let loader = |path: &str| (path == "figure.png").then(|| png.clone());
    let written = markdown_to_docx(
        "![A figure](figure.png) and ![missing](nowhere.png)\n",
        &DocxOptions {
            images: Some(&loader),
            ..DocxOptions::default()
        },
    )
    .unwrap();
    assert_word_valid_package(&written.docx);
    assert_eq!(
        written.warnings,
        ["image 'nowhere.png' was written as its alt text"]
    );
    let xml = document_xml(&written.docx);
    assert!(
        xml.contains("<wp:extent cx=\"1905000\" cy=\"952500\"/>"),
        "{xml}"
    );
    assert!(xml.contains("descr=\"A figure\""));
    assert!(xml.contains("missing"));
    let types = part_string(&written.docx, "[Content_Types].xml").unwrap();
    assert!(types.contains("Extension=\"png\""));
    assert!(
        part_string(&written.docx, "word/media/image1.png").is_some() || {
            let zip = zip::ZipArchive::new(Cursor::new(&written.docx)).unwrap();
            zip.file_names().any(|n| n == "word/media/image1.png")
        }
    );
}

#[test]
fn inline_html_quotes_code_in_lists_and_anchors() {
    let docx = write(
        "H<sub>2</sub>O and E=mc<sup>2</sup>, <u>underlined</u>, a<br>break, <span>kept</span>.\n\n\
         > > deeper\n\n\
         - item\n\n  ```\n  code in a list\n  ```\n\n\
         [back to top](#top)\n",
    );
    let xml = document_xml(&docx);
    assert!(xml.contains("<w:vertAlign w:val=\"subscript\"/>"));
    assert!(xml.contains("<w:vertAlign w:val=\"superscript\"/>"));
    assert!(xml.contains("<w:u w:val=\"single\"/>"));
    assert!(xml.contains("<w:hyperlink w:anchor=\"top\""));
    let paragraphs = inspect::paragraphs(&docx).unwrap();
    let find = |text: &str| paragraphs.iter().find(|p| p.text == text).unwrap();
    assert_eq!(
        find("H2O and E=mc2, underlined, a\nbreak, kept.").style,
        None
    );
    assert_eq!(find("deeper").style.as_deref(), Some("Quote"));
    assert_eq!(find("code in a list").style.as_deref(), Some("SourceCode"));
    assert!(
        xml.contains("<w:ind w:left=\"1440\"/>"),
        "a nested quote is indented"
    );
    assert!(
        xml.contains("<w:ind w:left=\"720\"/>"),
        "code in a list is indented"
    );
}

#[test]
fn comments_take_breaks_and_note_references_as_text() {
    let docx = write("Text{>>first line<br>second  \nthird [^n] end<<}.\n\n[^n]: Note.\n");
    let comments = part_string(&docx, "word/comments.xml").unwrap();
    for line in ["first line", "second", "third [^n] end"] {
        assert!(comments.contains(line), "{line} in {comments}");
    }
    assert_eq!(comments.matches("<w:p>").count(), 3, "{comments}");
    assert_eq!(texts(&docx), ["Text."]);
}

#[test]
fn front_matter_without_values_and_a_second_definition_are_ignored() {
    let docx = write(
        "---\ntitle:\nnot a pair\nauthor: 'Quoted Name'\n---\n\nText.[^a]\n\n[^a]: First.\n\n[^a]: Second.\n",
    );
    let core = part_string(&docx, "docProps/core.xml").unwrap();
    assert!(
        core.contains("<dc:creator>Quoted Name</dc:creator>"),
        "{core}"
    );
    assert!(!core.contains("<dc:title>"), "{core}");
    let notes = part_string(&docx, "word/footnotes.xml").unwrap();
    assert!(
        notes.contains("First.") && !notes.contains("Second."),
        "{notes}"
    );
}

#[test]
fn an_image_the_loader_cannot_read_as_a_picture_is_its_alt_text() {
    let loader = |_: &str| Some(b"not an image".to_vec());
    let written = markdown_to_docx(
        "![Chart](chart.svg) and ![](empty.png)\n",
        &DocxOptions {
            images: Some(&loader),
            ..DocxOptions::default()
        },
    )
    .unwrap();
    assert_eq!(texts(&written.docx), ["Chart and "]);
    assert_eq!(written.warnings.len(), 2);
    // Without a loader, alt text is written and nothing is reported.
    let written = markdown_to_docx("![Chart](chart.png)\n", &DocxOptions::default()).unwrap();
    assert_eq!(texts(&written.docx), ["Chart"]);
    assert!(written.warnings.is_empty());
}

#[test]
fn a_wide_picture_is_scaled_to_the_text_width() {
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &vec![0u8; 1000 * 10],
            1000,
            10,
            image::ExtendedColorType::L8,
        )
        .unwrap();
    let loader = |_: &str| Some(png.clone());
    let written = markdown_to_docx(
        "![wide](w.png)\n",
        &DocxOptions {
            images: Some(&loader),
            ..DocxOptions::default()
        },
    )
    .unwrap();
    let xml = document_xml(&written.docx);
    assert!(
        xml.contains("<wp:extent cx=\"5943600\" cy=\"59436\"/>"),
        "{xml}"
    );
}

#[test]
fn a_reference_that_is_not_a_docx_is_refused() {
    let err = markdown_to_docx(
        "Text.\n",
        &DocxOptions {
            reference: Some(b"not a zip"),
            ..DocxOptions::default()
        },
    )
    .unwrap_err();
    assert!(err.to_string().starts_with("reference document:"), "{err}");
}
