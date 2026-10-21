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
use jubarte::markdown::{
    DocxOptions, MarkdownOptions, TrackChanges, docx_to_markdown, markdown_to_docx,
};

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
    ] {
        assert!(styles.contains(&format!("w:styleId=\"{id}\"")), "{id}");
    }
    // A plain Markdown table takes Word's thin grid (Arthur, 2026-10-15).
    assert!(styles.contains("w:styleId=\"TableGrid\""), "{styles}");
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

/// The Markdown parser must skip complete nested comments when looking for
/// a change's closer or a substitution's `~>` separator, otherwise the
/// `~>`, `++}`, `--}`, `~~}`, or `==}` inside the comment steals the
/// change. Cover every change kind and every payload, with both
/// substitution halves.
#[test]
fn a_complete_comment_inside_a_change_keeps_the_change_and_the_comment() {
    let payloads = ["~>", "++}", "--}", "~~}", "==}"];
    for payload in &payloads {
        let expected = |sub: &str| match sub {
            "ins" => format!("{{++a{{>>{payload}<<}}b++}}"),
            "del" => format!("{{--a{{>>{payload}<<}}b--}}"),
            "sub-old" => format!("{{~~a{{>>{payload}<<}} b~>c~~}}"),
            "sub-new" => format!("{{~~a~>b{{>>{payload}<<}} c~~}}"),
            "hi" => format!("{{==a{{>>{payload}<<}}b==}}"),
            _ => unreachable!(),
        };
        let accepted_text = |sub: &str| -> Vec<&'static str> {
            match sub {
                "ins" => vec!["ab"],
                "del" => vec![""],
                "sub-old" => vec!["c"],
                "sub-new" => vec!["b c"],
                "hi" => vec!["ab"],
                _ => unreachable!(),
            }
        };
        let rejected_text = |sub: &str| -> Vec<&'static str> {
            match sub {
                "ins" => vec![""],
                "del" => vec!["ab"],
                "sub-old" => vec!["a b"],
                "sub-new" => vec!["a"],
                "hi" => vec!["ab"],
                _ => unreachable!(),
            }
        };
        let revision = |sub: &str| -> Vec<(String, String)> {
            match sub {
                "ins" => vec![("Inserted".to_string(), "ab".to_string())],
                "del" => vec![("Deleted".to_string(), "ab".to_string())],
                "sub-old" => vec![
                    ("Deleted".to_string(), "a b".to_string()),
                    ("Inserted".to_string(), "c".to_string()),
                ],
                "sub-new" => vec![
                    ("Deleted".to_string(), "a".to_string()),
                    ("Inserted".to_string(), "b c".to_string()),
                ],
                "hi" => vec![],
                _ => unreachable!(),
            }
        };

        for sub in ["ins", "del", "sub-old", "sub-new", "hi"] {
            let markdown = expected(sub);
            let docx = write(&markdown);
            assert_eq!(
                accepted(&docx),
                accepted_text(sub),
                "{sub} {payload:?}: accepted for {markdown:?}",
            );
            assert_eq!(
                rejected(&docx),
                rejected_text(sub),
                "{sub} {payload:?}: rejected for {markdown:?}",
            );
            let revs: Vec<(String, String)> = revisions(&docx)
                .iter()
                .map(|(t, text)| {
                    let label = match t {
                        jubarte::comparer::WmlComparerRevisionType::Inserted => "Inserted",
                        jubarte::comparer::WmlComparerRevisionType::Deleted => "Deleted",
                        _ => "Other",
                    };
                    (label.to_string(), text.clone())
                })
                .collect();
            let mut revs_sorted = revs.clone();
            revs_sorted.sort();
            assert_eq!(
                revs_sorted,
                revision(sub),
                "{sub} {payload:?}: revisions for {markdown:?}",
            );

            // One comment, and its XML body contains the payload verbatim.
            let comments = part_string(&docx, "word/comments.xml").unwrap();
            assert!(
                comments.matches("<w:comment ").count() == 1,
                "{sub} {payload:?}: exactly one comment in {comments}",
            );
            // The payload is in the comment body. Compare against the
            // decoded text since the writer stores `>` as `&gt;`, `~` as
            // `~`, etc.
            let decoded = decode_xml_entities(&comments);
            assert!(
                decoded.contains(payload),
                "{sub} {payload:?}: comment XML should contain payload {decoded}",
            );

            // Range-start, range-end, and reference share the same id, and
            // that id appears inside the intended revision (insertion or
            // deletion).
            let xml = document_xml(&docx);
            let ids: std::collections::HashSet<u32> = (0..16)
                .filter(|i| xml.contains(&format!(r#"<w:commentRangeStart w:id="{i}"/>"#)))
                .collect();
            assert_eq!(ids.len(), 1, "{sub} {payload:?}: one id in {xml}");
            let id = ids.into_iter().next().unwrap();
            assert!(
                xml.contains(&format!(r#"<w:commentRangeEnd w:id="{id}"/>"#)),
                "{sub} {payload:?}: range-end id {id} in {xml}",
            );
            assert!(
                xml.contains(&format!(r#"<w:commentReference w:id="{id}"/>"#)),
                "{sub} {payload:?}: reference id {id} in {xml}",
            );
            let start = xml
                .find(&format!(r#"<w:commentRangeStart w:id="{id}"/>"#))
                .unwrap();
            let end = xml
                .find(&format!(r#"<w:commentRangeEnd w:id="{id}"/>"#))
                .unwrap();
            let inside_rev = match sub {
                "ins" | "sub-new" => {
                    xml[..start].matches("<w:ins ").count() == xml[..end].matches("<w:ins ").count()
                }
                "del" | "sub-old" => {
                    xml[..start].matches("<w:del ").count() == xml[..end].matches("<w:del ").count()
                }
                "hi" => xml.contains("<w:highlight "),
                _ => unreachable!(),
            };
            assert!(
                inside_rev,
                "{sub} {payload:?}: comment range inside the intended revision in {xml}",
            );
        }
    }
}

/// Decode the XML entities the writer produces (`>`, `<`, `&`) so comment
/// text matches the source it came from. Used to compare comment bodies
/// without quoting each special character.
fn decode_xml_entities(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut chars = xml.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' {
            let mut tail = String::new();
            while let Some(&next) = chars.peek() {
                if next == ';' {
                    chars.next();
                    break;
                }
                tail.push(next);
                chars.next();
            }
            match tail.as_str() {
                "gt" => out.push('>'),
                "lt" => out.push('<'),
                "amp" => out.push('&'),
                "quot" => out.push('"'),
                "apos" => out.push('\''),
                other => {
                    out.push('&');
                    out.push_str(other);
                    out.push(';');
                }
            }
        } else {
            out.push(c);
        }
    }
    out
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
    // A link wholly inside the change is a HYPERLINK field in its one mark,
    // not a w:hyperlink that parts the change in two.
    assert_eq!(xml.matches("<w:ins ").count(), 1, "{xml}");
    assert!(!xml.contains("<w:hyperlink"), "{xml}");
    assert!(
        xml.contains("HYPERLINK &quot;https://example.com&quot;")
            || xml.contains("HYPERLINK \"https://example.com\""),
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
fn math_in_an_image_description_stays_in_the_alt_text() {
    // pulldown-cmark parses the description of `![…]` as inline events, so
    // `$…$` there is InlineMath with the image open: it joins the alt text,
    // never a body run before the picture.
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&[0u8; 200 * 100], 200, 100, image::ExtendedColorType::L8)
        .unwrap();
    let loader = |_: &str| Some(png.clone());
    let written = markdown_to_docx(
        "![area $x^2$](a.png)\n",
        &DocxOptions {
            images: Some(&loader),
            ..DocxOptions::default()
        },
    )
    .unwrap();
    let xml = document_xml(&written.docx);
    assert!(xml.contains("descr=\"area $x^2$\""), "{xml}");
    assert_eq!(texts(&written.docx), [""]);
    // Without a loader, the alt text — math included — is the body text.
    let written = markdown_to_docx("![area $x^2$](a.png)\n", &DocxOptions::default()).unwrap();
    assert_eq!(texts(&written.docx), ["area $x^2$"]);
}

#[test]
fn caret_and_single_tilde_mark_superscript_and_subscript() {
    // `^x^` and `~x~` at word edges, as pulldown-cmark reads them; two
    // tildes stay strikethrough and `{~~a~>b~~}` stays a change. Inside a
    // word (`H~2~O`) they stay text: pulldown-cmark opens neither there.
    let docx = write("water H ~2~ O, E = mc ^2^, H~2~O, ~~struck~~ and {~~a~>b~~}.\n");
    let xml = document_xml(&docx);
    assert_eq!(
        xml.matches("<w:vertAlign w:val=\"subscript\"/>").count(),
        1,
        "{xml}"
    );
    assert_eq!(
        xml.matches("<w:vertAlign w:val=\"superscript\"/>").count(),
        1,
        "{xml}"
    );
    assert!(xml.contains("<w:strike/>"), "{xml}");
    assert_eq!(
        texts(&docx),
        ["water H 2 O, E = mc 2, H~2~O, struck and b."]
    );
    assert_eq!(
        rejected(&docx),
        ["water H 2 O, E = mc 2, H~2~O, struck and a."]
    );
}

#[test]
fn a_link_to_a_heading_lands_on_its_bookmark() {
    // `[x](#slug)` names a heading the way GitHub slugs it; the heading
    // gets the bookmark Word's `w:anchor` jumps to. A second heading with
    // the same text is `-1`; a heading no link names gets no bookmark.
    let docx = write(
        "# The Intro: part 1\n\nText.\n\n## Unlinked\n\n# The Intro: part 1\n\n\
         See [the intro](#the-intro-part-1) and [again](#the-intro-part-1-1).\n",
    );
    let xml = document_xml(&docx);
    let starts: Vec<&str> = xml
        .match_indices("<w:bookmarkStart")
        .map(|(at, _)| &xml[at..at + xml[at..].find("/>").unwrap() + 2])
        .collect();
    assert_eq!(
        starts,
        [
            "<w:bookmarkStart w:id=\"0\" w:name=\"the-intro-part-1\"/>",
            "<w:bookmarkStart w:id=\"1\" w:name=\"the-intro-part-1-1\"/>",
        ],
        "{xml}"
    );
    assert!(
        xml.contains("<w:t xml:space=\"preserve\">The Intro: part 1</w:t></w:r><w:bookmarkEnd w:id=\"0\"/></w:p>"),
        "{xml}"
    );
    assert_eq!(xml.matches("<w:bookmarkEnd").count(), 2, "{xml}");
}

#[test]
fn heading_slugs_follow_github_slugger() {
    // github-slugger: a repeat retries its suffixed name against every name
    // given out; combining marks stay; a no-break space and an em dash go;
    // only a plain space becomes `-`. An empty heading gets no bookmark.
    let docx = write(
        "# Intro\n\n# Intro\n\n# Intro 1\n\n# e\u{301}cole math\n\n\
         # Foo\u{a0}Bar \u{2014} em\u{2014}dash\n\n#\n\n\
         [a](#intro) [b](#intro-1) [c](#intro-1-1) [d](#e\u{301}cole-math) \
         [e](#foobar--emdash) [f](#)\n",
    );
    let xml = document_xml(&docx);
    let names: Vec<&str> = xml
        .split("<w:bookmarkStart ")
        .skip(1)
        .filter_map(|tag| tag.split("w:name=\"").nth(1)?.split('"').next())
        .collect();
    assert_eq!(
        names,
        [
            "intro",
            "intro-1",
            "intro-1-1",
            "e\u{301}cole-math",
            "foobar--emdash"
        ],
        "{xml}"
    );
}

#[test]
fn a_bookmark_that_would_start_with_a_digit_starts_with_an_underscore() {
    // Word drops a bookmark whose name starts with no letter; `_` keeps it,
    // hidden like Word's own `_Toc` ones, and the link follows.
    let docx = write("# 2026 plan: Q&A\n\n[plan](#2026-plan-qa)\n");
    let xml = document_xml(&docx);
    assert!(
        xml.contains("<w:bookmarkStart w:id=\"0\" w:name=\"_2026-plan-qa\"/>"),
        "{xml}"
    );
    assert!(
        xml.contains("<w:hyperlink w:anchor=\"_2026-plan-qa\""),
        "{xml}"
    );
}

#[test]
fn a_bookmark_name_keeps_to_forty_characters() {
    // Word's schema caps a bookmark name at 40 characters: a longer slug
    // is cut, a cut that repeats another gets `-1`, and the links follow.
    let long = "a-rather-long-heading-that-runs-well-past-forty";
    let docx = write(&format!(
        "# A rather long heading that runs well past forty\n\n\
         # A rather long heading that runs well past forty-two\n\n\
         [one](#{long}) [two](#{long}-two)\n"
    ));
    let xml = document_xml(&docx);
    let cut = &long[..40];
    let cut_again = format!("{}-1", &long[..38]);
    assert!(
        xml.contains(&format!("<w:bookmarkStart w:id=\"0\" w:name=\"{cut}\"/>")),
        "{xml}"
    );
    assert!(
        xml.contains(&format!(
            "<w:bookmarkStart w:id=\"1\" w:name=\"{cut_again}\"/>"
        )),
        "{xml}"
    );
    assert!(
        xml.contains(&format!("<w:hyperlink w:anchor=\"{cut}\"")),
        "{xml}"
    );
    assert!(
        xml.contains(&format!("<w:hyperlink w:anchor=\"{cut_again}\"")),
        "{xml}"
    );
}

#[test]
fn inline_html_bold_italic_strike_code_and_mark() {
    // The common inline tags format their text as their Markdown siblings
    // do; a tag's attributes and case do not matter. `<mark>` is Word's
    // yellow highlight, alone.
    let docx = write(
        "<b>b</b> <strong>s</strong> <i>i</i> <em>e</em> <s>x</s> <del>d</del> \
         <code>c</code> <mark>m</mark> <B CLASS=\"k\">attr</B>.\n",
    );
    let xml = document_xml(&docx);
    for (element, count) in [
        ("<w:b/>", 3),
        ("<w:i/>", 2),
        ("<w:strike/>", 2),
        ("<w:rStyle w:val=\"VerbatimChar\"/>", 1),
        ("<w:highlight w:val=\"yellow\"/>", 1),
    ] {
        assert_eq!(xml.matches(element).count(), count, "{element}\n{xml}");
    }
    assert!(!xml.contains("w:em "), "{xml}");
    assert_eq!(texts(&docx), ["b s i e x d c m attr."]);
}

#[test]
fn a_braced_code_and_its_result_are_a_field() {
    // Spec §3: `{CODE}result{/}` is a body field, the code then the result
    // the reader sees. Braces that do not close with `{/}` in their
    // paragraph, and escaped ones, are text.
    let docx = write(
        "Begins {DATE \\@ \"MMMM d, yyyy\"}March 1, 2026{/} as {REF _Ref1 \\h}Section **2**{/} says.\n\n\
         Dear {Client Name}, and \\{PAGE}1{/}.\n",
    );
    let xml = document_xml(&docx);
    assert_eq!(
        xml.matches(r#"<w:fldChar w:fldCharType="begin"/>"#).count(),
        2,
        "{xml}"
    );
    assert_eq!(
        xml.matches(r#"<w:fldChar w:fldCharType="separate"/>"#)
            .count(),
        2,
        "{xml}"
    );
    assert_eq!(
        xml.matches(r#"<w:fldChar w:fldCharType="end"/>"#).count(),
        2,
        "{xml}"
    );
    assert!(
        xml.contains(
            r#"<w:instrText xml:space="preserve"> DATE \@ &quot;MMMM d, yyyy&quot; </w:instrText>"#
        ),
        "{xml}"
    );
    assert!(
        xml.contains(r#"<w:instrText xml:space="preserve"> REF _Ref1 \h </w:instrText>"#),
        "{xml}"
    );
    assert_eq!(
        texts(&docx),
        [
            "Begins March 1, 2026 as Section 2 says.",
            "Dear {Client Name}, and {PAGE}1{/}."
        ]
    );
}

#[test]
fn code_inside_a_link_keeps_its_face() {
    // A run has one character style: inside a link it is `Hyperlink`, so
    // code there takes `VerbatimChar`'s face directly and reads back as code.
    let xml = document_xml(&write("See [`open()`](https://a.example/b) now.\n"));
    assert!(
        xml.contains(r#"<w:rStyle w:val="Hyperlink"/><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:cs="Consolas"/>"#),
        "{xml}"
    );
}

#[test]
fn double_equals_is_a_highlight() {
    // `==x==` is a highlight (spec §3), the run property `<mark>` writes; it
    // may hold emphasis. An `==` with spaces on both sides, one inside code
    // or one escaped is text, and `{==x==}` stays CriticMarkup.
    let docx = write("Pay ==**now**== and ==later==, a == b, `c==d`, e=\\=f.\n");
    let xml = document_xml(&docx);
    assert_eq!(
        xml.matches("<w:highlight w:val=\"yellow\"/>").count(),
        2,
        "{xml}"
    );
    assert!(xml.contains("<w:b/><w:bCs/><w:highlight"), "{xml}");
    assert_eq!(texts(&docx), ["Pay now and later, a == b, c==d, e==f."]);
    let comment = document_xml(&write("A {==range==}{>>note<<} here.\n"));
    assert!(!comment.contains("w:highlight"), "{comment}");
    assert!(comment.contains("commentRangeStart"), "{comment}");
}

#[test]
fn an_unclosed_highlight_is_text() {
    // An `==` without its partner in the paragraph is text, as an unclosed
    // `~~` is.
    let docx = write("Only ==one here.\n\nNext ==para== ok.\n");
    let xml = document_xml(&docx);
    assert_eq!(xml.matches("<w:highlight").count(), 1, "{xml}");
    assert_eq!(texts(&docx), ["Only ==one here.", "Next para ok."]);
}

#[test]
fn an_unclosed_inline_tag_ends_with_its_block() {
    // `<b>` without `</b>` formats the rest of its paragraph, cell or
    // heading, never the blocks after it.
    let docx = write("# <u>head\n\n<b>open and **strong\n\n|<i>a|b|\n|-|-|\n|c|d|\n\nplain\n");
    let xml = document_xml(&docx);
    assert_eq!(xml.matches("<w:u ").count(), 1, "{xml}");
    assert_eq!(xml.matches("<w:i/>").count(), 1, "{xml}");
    let plain = xml.split("plain").next().unwrap();
    let last = &plain[plain.rfind("<w:r>").unwrap()..];
    assert!(!last.contains("<w:b/>"), "{xml}");
}

#[test]
fn inline_html_the_writer_drops_is_named_once_in_a_warning() {
    // A tag the writer does not know keeps its text and loses its markup;
    // the warning names each such tag once. Comments are no tags.
    let written = markdown_to_docx(
        "<span>kept</span> and <kbd>K</kbd> and <span class=\"x\">again</span> <!-- note -->.\n",
        &DocxOptions::default(),
    )
    .unwrap();
    assert_eq!(texts(&written.docx), ["kept and K and again ."]);
    let html: Vec<&String> = written
        .warnings
        .iter()
        .filter(|w| w.contains("HTML"))
        .collect();
    assert_eq!(html.len(), 2, "{:?}", written.warnings);
    assert!(html[0].contains("<span>"), "{html:?}");
    assert!(html[1].contains("<kbd>"), "{html:?}");
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

#[test]
fn a_whole_block_after_a_changed_break_keeps_its_own_mark() {
    // The inserted break is the only paragraph mark the markup adds: B's
    // own mark was A's, so rejecting leaves A and C apart.
    for markdown in [
        "A{++\n\n++}{++B++}\n\nC",
        "A{++\n\n++}{>>Ana<<}{++B++}\n\nC",
    ] {
        let docx = write(markdown);
        assert_eq!(accepted(&docx), ["A", "B", "C"], "{markdown}");
        assert_eq!(rejected(&docx), ["A", "C"], "{markdown}");
    }
    let docx = write("A{--\n\n--}{--B--}\n\nC");
    assert_eq!(accepted(&docx), ["A", "C"]);
    assert_eq!(rejected(&docx), ["A", "B", "C"]);
}

#[test]
fn a_change_before_a_footnote_label_changes_the_note() {
    // How a note inserted with its reference is read from Word.
    let docx = write("A claim{++[^1]++} here.\n\n{++[^1]: Added later.++}\n");
    let xml = part_string(&docx, "word/footnotes.xml").unwrap_or_default();
    assert!(xml.contains("Added later."), "{xml}");
    assert!(!xml.contains("[^1]"), "{xml}");
    assert_eq!(accepted(&docx), ["A claim here."]);
    assert!(!document_xml(&docx).contains("[^1]"));
}

#[test]
fn a_block_added_after_a_paragraph_leaves_that_paragraph_as_it_was() {
    use jubarte::markdown::{MarkdownOptions, docx_to_markdown};
    let read = |docx: &[u8]| {
        docx_to_markdown(docx, &MarkdownOptions::default())
            .unwrap()
            .markdown
    };
    // Word records a paragraph added after A as its own mark inserted, so
    // rejecting it leaves A with A's properties, not the new heading's.
    let docx = write("A.{++\n\n## New heading++}\n\nC");
    assert_eq!(read(&reject_revisions(&docx).unwrap()), "A.\n\nC\n");
    assert_eq!(
        read(&accept_revisions(&docx).unwrap()),
        "A.\n\n## New heading\n\nC\n"
    );
    let docx = write("## Title{--\n\nGone.\n\nGone too.--}\n\nC");
    assert_eq!(read(&accept_revisions(&docx).unwrap()), "## Title\n\nC\n");
    assert_eq!(
        read(&reject_revisions(&docx).unwrap()),
        "## Title\n\nGone.\n\nGone too.\n\nC\n"
    );
}

#[test]
fn attributions_name_the_author_and_date_of_the_change_before_them() {
    let docx = write(
        "A{++b++}{>>Ana Lima (2026-01-02T03:04:00Z)<<} c{~~d~>e~~}{>>Bo (2026-02-03T04:05:06Z)<<}\
         {>>Cy (2026-03-04T05:06:07Z)<<} f{--g--}{>>(2026-04-05T06:07:08+01:00)<<}.\n\n\
         {==h==}{>>Dee (2026-05-06T07:08:09Z): Why?<<} i{++j++}{>>Ana Lima<<}\n",
    );
    let xml = document_xml(&docx);
    let changes: Vec<&str> = xml
        .split("<w:")
        .filter(|tag| tag.starts_with("ins ") || tag.starts_with("del "))
        .map(|tag| &tag[tag.find("w:author").unwrap()..tag.find('>').unwrap()])
        .collect();
    assert_eq!(
        changes,
        [
            r#"w:author="Ana Lima" w:date="2026-01-02T03:04:00Z""#,
            // A substitution is written as Word writes one: the inserted
            // side (Cy's) before the deleted side (Bo's).
            r#"w:author="Cy" w:date="2026-03-04T05:06:07Z""#,
            r#"w:author="Bo" w:date="2026-02-03T04:05:06Z""#,
            r#"w:author="Redline" w:date="2026-04-05T06:07:08+01:00""#,
            r#"w:author="Redline" w:date="1970-01-01T00:00:00Z""#,
        ]
    );
    // Two real comments: one named, and one whose text has no date.
    let comments = part_string(&docx, "word/comments.xml").unwrap();
    assert_eq!(comments.matches("<w:comment ").count(), 2, "{comments}");
    assert!(comments.contains(r#"w:author="Dee" w:date="2026-05-06T07:08:09Z" w:initials="D""#));
    assert!(comments.contains(">Why?<"), "{comments}");
    assert!(comments.contains(">Ana Lima<"), "{comments}");
    assert_eq!(accepted(&docx), ["Ab ce f.", "h ij"]);
    assert_eq!(rejected(&docx), ["A cd fg.", "h i"]);
}

/// A closer inside an HTML comment or an image's alt text still closes
/// the span that opened before it: CriticMarkup is read before Markdown,
/// so the constructs the writer drops or captures cannot hide a token.
/// A span complete inside such a construct is dropped with it, and a
/// stray closer in a comment of the author's own stays dropped.
#[test]
fn a_closer_inside_dropped_html_or_alt_text_still_closes_its_span() {
    for (markdown, kept, dropped) in [
        (
            "{++text <!-- ++} --> tail\n",
            vec!["text  tail"],
            vec![" tail"],
        ),
        (
            "{++text ![alt ++}](url) tail\n",
            vec!["text alt tail"],
            vec![" tail"],
        ),
        // The change holds the paragraph break it spans.
        (
            "{++text\n\n<!-- ++} -->\n\n tail\n",
            vec!["text", "tail"],
            vec!["tail"],
        ),
    ] {
        let docx = write(markdown);
        let ins: Vec<String> = revisions(&docx)
            .into_iter()
            .filter(|(kind, _)| *kind == WmlComparerRevisionType::Inserted)
            .map(|(_, text)| text)
            .collect();
        assert!(
            ins.iter().all(|text| !text.contains("tail")),
            "{markdown:?}: the insertion swallowed 'tail': {ins:?}"
        );
        assert_eq!(accepted(&docx), kept, "{markdown:?}");
        assert_eq!(rejected(&docx), dropped, "{markdown:?}");
    }
    // Spans and markers complete inside dropped text stay dropped with it.
    for (markdown, kept) in [
        ("a <!-- {++x++} --> b\n", "a  b"),
        ("a <!-- x ++} --> b\n", "a  b"),
        // The image falls back to its alt text, span dropped with it.
        ("a ![x {++y++} z](u) b\n", "a x y z b"),
    ] {
        let docx = write(markdown);
        assert!(revisions(&docx).is_empty(), "{markdown:?}");
        assert_eq!(accepted(&docx), [kept], "{markdown:?}");
    }
}

/// A blank line inside `{>>…<<}` is a paragraph break in the comment's
/// text, never a break in the paragraph it is anchored on.
#[test]
fn a_blank_line_inside_a_comment_stays_in_the_comment() {
    let docx = write("before {>>first\n\nsecond<<} after\n");
    assert_eq!(texts(&docx), ["before  after"]);
    let comments = part_string(&docx, "word/comments.xml").unwrap();
    assert_eq!(comments.matches("<w:comment ").count(), 1, "{comments}");
    assert_eq!(comments.matches("<w:p>").count(), 2, "{comments}");
    for line in ["first", "second"] {
        assert!(comments.contains(line), "{line} in {comments}");
    }
    let xml = document_xml(&docx);
    assert_eq!(xml.matches("<w:commentRangeStart ").count(), 1, "{xml}");
    assert_eq!(xml.matches("<w:commentRangeEnd ").count(), 1, "{xml}");
    assert_eq!(xml.matches("<w:commentReference ").count(), 1, "{xml}");
    // One newline is a line of the same comment paragraph, and runs of
    // blank or whitespace-only lines collapse to one paragraph break with
    // no comment text leaking into the body.
    let docx = write("a {>>x\ny<<} b\n");
    assert_eq!(texts(&docx), ["a  b"]);
    let comments = part_string(&docx, "word/comments.xml").unwrap();
    assert_eq!(comments.matches("<w:p>").count(), 1, "{comments}");
    let docx = write("a {>>x\n\n\n   \n\ny<<} b\n");
    assert_eq!(texts(&docx), ["a  b"]);
    let comments = part_string(&docx, "word/comments.xml").unwrap();
    assert_eq!(comments.matches("<w:p>").count(), 2, "{comments}");
}

/// Malformed CriticMarkup comes out text with a warning: an unclosed
/// opener, a closer that closed nothing, a substitution with no `~>`.
#[test]
fn malformed_critic_markup_warns_and_stays_text() {
    for (markdown, warning) in [
        ("a {++b\n", "'{++'"),
        ("a ++} b\n", "'++}'"),
        ("a {~~b~~}\n", "'{~~'"),
        ("a {++b--}\n", "'--}'"),
    ] {
        let written = markdown_to_docx(markdown, &DocxOptions::default()).unwrap();
        assert!(
            written.warnings.iter().any(|w| w.contains(warning)),
            "{markdown:?}: {:?}",
            written.warnings
        );
        assert!(revisions(&written.docx).is_empty(), "{markdown:?}");
    }
}

/// A substitution whose start and separator sit in text the writer drops
/// or captures, with its end outside, still splits: the text after the
/// drop is the new half, not a deletion.
#[test]
fn a_substitution_split_by_dropped_text_inserts_its_new_half() {
    for markdown in [
        "a <!--{~~x~> -->b~~} c\n",
        "a ![{~~x~>b](none.png) c~~} d\n",
    ] {
        let docx = write(markdown);
        let revs = revisions(&docx);
        let inserted: Vec<&str> = revs
            .iter()
            .filter(|(kind, _)| *kind == WmlComparerRevisionType::Inserted)
            .map(|(_, text)| text.as_str())
            .collect();
        let deleted: Vec<&str> = revs
            .iter()
            .filter(|(kind, _)| *kind == WmlComparerRevisionType::Deleted)
            .map(|(_, text)| text.as_str())
            .collect();
        assert!(
            inserted
                .iter()
                .any(|text| text.contains('b') || text.contains(" c")),
            "{markdown:?}: new half not inserted: {revs:?}"
        );
        assert!(deleted.is_empty(), "{markdown:?}: {deleted:?}");
    }
}

/// A numeric character reference that names a CriticMarkup stand-in (or
/// the escape) is literal text wherever the Markdown parser decodes it:
/// paragraph text, a comment's body, an HTML block's visible text. The
/// delimiters written directly in an HTML block are real CriticMarkup:
/// the change applies, and no stand-in character reaches the document.
#[test]
fn character_references_naming_stand_ins_are_text_and_html_blocks_apply_changes() {
    // Paragraph text: no revision, the decoded characters are text.
    let docx = write("a &#x2E0E;x&#x2E0F; b\n");
    assert!(revisions(&docx).is_empty());
    assert_eq!(texts(&docx), [format!("a {}x{} b", '\u{2E0E}', '\u{2E0F}')]);
    // A separator reference opens no phantom insertion.
    let docx = write("a &#x2E13; b\n");
    assert!(revisions(&docx).is_empty());
    // An escape reference kills no change that follows it.
    let docx = write("a &#xF0000;{++x++} b\n");
    let inserted: Vec<String> = revisions(&docx)
        .into_iter()
        .filter(|(kind, _)| *kind == WmlComparerRevisionType::Inserted)
        .map(|(_, text)| text)
        .collect();
    assert_eq!(inserted, ["x"], "the change after an escape reference");
    // A comment keeps its whole body: no early end, no body leak.
    let docx = write("{>>a &#x2E19; b<<}\n");
    assert!(texts(&docx).iter().all(|text| text.trim().is_empty()));
    let comments = part_string(&docx, "word/comments.xml").unwrap();
    assert_eq!(comments.matches("<w:comment ").count(), 1, "{comments}");
    assert_eq!(comments.matches("<w:p>").count(), 1, "{comments}");
    // No comment starts from references either.
    let docx = write("a &#x2E18;n&#x2E19; b\n");
    assert!(revisions(&docx).is_empty());
    assert!(!document_xml(&docx).contains("commentRangeStart"));

    // An HTML block's visible text: written delimiters are changes and
    // leave no stand-in in the document; references are literal text
    // (the characters they name, and no revision).
    for markdown in ["<div>{++x++}</div>\n", "<p>a {--b--} c</p>\n"] {
        let docx = write(markdown);
        assert!(!revisions(&docx).is_empty(), "{markdown:?}");
        let xml = document_xml(&docx);
        for code in 0x2E0E..=0x2E2F {
            let stand_in = char::from_u32(code).unwrap();
            assert!(
                !xml.contains(stand_in),
                "{markdown:?}: stand-in U+{code:X} reached the document"
            );
        }
        assert!(
            !xml.contains('\u{F0000}'),
            "{markdown:?}: escape reached the document"
        );
    }
    let docx = write("<div>&#x2E0E;x&#x2E0F;</div>\n");
    assert!(revisions(&docx).is_empty());
    assert_eq!(texts(&docx), [format!("{}x{}", '\u{2E0E}', '\u{2E0F}')]);
    assert!(!document_xml(&docx).contains('\u{F0000}'));
}

#[test]
fn a_range_split_into_highlights_is_one_comment() {
    // How a comment range that crosses paragraphs and changes is read from
    // Word: pieces, with the comment after the last.
    let docx = write(
        "Start {==of a range.==}\n\n{==The middle.==}\n\n{==The{++ new++}{>>Ana (2026-01-02T03:04:00Z)<<} end==}{>>Bo (2026-01-02T03:05:00Z): Note<<} after.\n\n\
         {==apart==} text {==b==}{>>n<<}\n",
    );
    let xml = document_xml(&docx);
    assert_eq!(xml.matches("<w:commentRangeStart ").count(), 2, "{xml}");
    let first = xml.find("<w:commentRangeStart w:id=\"0\"/>").unwrap();
    assert!(first < xml.find(">of a range.<").unwrap());
    assert!(xml.find(">Start <").unwrap() < first);
    // Only the highlight with text between it and the comment stays one.
    assert_eq!(xml.matches("<w:highlight ").count(), 1, "{xml}");
    // A comment after an attribution is on its own point, not the change.
    let docx =
        write("a{--b--}{>>Bo (2026-01-02T03:04:00Z)<<}{>>Cy (2026-01-02T03:05:00Z): why<<} c\n");
    let xml = document_xml(&docx);
    let start = xml.find("<w:commentRangeStart").unwrap();
    assert!(xml.find("</w:del>").unwrap() < start, "{xml}");
}

#[test]
fn comments_as_read_from_word_keep_their_ranges_and_authors() {
    let docx = write(
        "An {==empty comment==}{>>Émile Zola-Ng (2026-10-01T00:20:00Z)<<} and \
         {++{==inserted text==}++}{>>Ana (2026-10-01T00:45:00Z)<<}{>>Carla (2026-10-01T00:50:00Z): Good.<<}.\n",
    );
    let comments = part_string(&docx, "word/comments.xml").unwrap();
    assert!(
        comments.contains(r#"w:author="Émile Zola-Ng" w:date="2026-10-01T00:20:00Z""#),
        "{comments}"
    );
    assert!(!comments.contains("Émile Zola-Ng ("), "{comments}");
    let xml = document_xml(&docx);
    // Carla's comment covers the inserted text, not a point after it.
    let start = xml.find("<w:commentRangeStart w:id=\"1\"/>").unwrap();
    let end = xml.find("<w:commentRangeEnd w:id=\"1\"/>").unwrap();
    assert!(
        start < xml.find(">inserted text<").unwrap() && xml.find(">inserted text<").unwrap() < end,
        "{xml}"
    );
    assert!(!xml.contains("<w:highlight "), "{xml}");
}

fn tracks(docx: &[u8]) -> bool {
    part_string(docx, "word/settings.xml")
        .unwrap()
        .contains("<w:trackRevisions")
}

#[test]
fn track_changes_key_writes_track_revisions() {
    let on = write("---\ntrack_changes: on\n---\n\nText.\n");
    assert!(tracks(&on), "on");
    assert_word_valid_package(&on);
    assert!(
        !tracks(&write("---\ntrack_changes: off\n---\n\nText.\n")),
        "off"
    );
    // Revisions alone never turn tracking on.
    assert!(!tracks(&write("Due in {~~30~>45~~} days.\n")), "no key");
}

#[test]
fn track_changes_key_overrides_reference_settings() {
    let tracked = write("---\ntrack_changes: on\n---\n\nText.\n");
    let with = |markdown: &str| {
        write_with(
            markdown,
            &DocxOptions {
                reference: Some(&tracked),
                ..DocxOptions::default()
            },
        )
    };
    assert!(tracks(&with("Text.\n")), "the reference's setting is kept");
    assert!(
        !tracks(&with("---\ntrack_changes: off\n---\n\nText.\n")),
        "off"
    );
}

#[test]
fn the_tracking_option_overrides_the_key() {
    let with = |markdown: &str, tracking: bool| {
        write_with(
            markdown,
            &DocxOptions {
                tracking: Some(tracking),
                ..DocxOptions::default()
            },
        )
    };
    assert!(!tracks(&with(
        "---\ntrack_changes: on\n---\n\nText.\n",
        false
    )));
    assert!(tracks(&with(
        "---\ntrack_changes: off\n---\n\nText.\n",
        true
    )));
    assert!(tracks(&with("Text.\n", true)));
}

#[test]
fn a_table_takes_the_style_the_view_names_and_otherwise_table_grid() {
    // No style named: Word's thin grid, as Insert Table makes it.
    let plain = document_xml(&write("|a|b|\n|-|-|\n|c|d|\n"));
    assert!(
        plain.contains(r#"<w:tblStyle w:val="TableGrid"/>"#),
        "{plain}"
    );
    // The legend's `table:` styles every table.
    let legend = markdown_to_docx(
        "---\nview: tracked\nstyles:\n  Normal: Calibri 11pt  # default\n  table: TableGrid, all borders 0.5pt\n---\n\n\
         <!-- t0 1x2, cells p0-p1 by row -->\n|a|b|\n|-|-|\n",
        &DocxOptions::default(),
    )
    .unwrap();
    let xml = document_xml(&legend.docx);
    assert!(xml.contains(r#"<w:tblStyle w:val="TableGrid"/>"#), "{xml}");
    let styles = part_string(&legend.docx, "word/styles.xml").unwrap();
    assert!(styles.contains(r#"w:styleId="TableGrid""#), "{styles}");
    // Tables of several styles name theirs on their lines; one naming
    // none takes Table Grid. A style Word does not know is defined by its name.
    let mixed = markdown_to_docx(
        "---\nview: tracked\nstyles:\n  Normal: Calibri 11pt  # default\n  tables: TableGrid, Ledger\n---\n\n\
         <!-- t0 1x1, cells p0-p0 by row -->\n|a|\n|-|\n\n\
         <!-- t1 1x1, cells p1-p1 by row, style TableGrid -->\n|b|\n|-|\n\n\
         <!-- t2 1x1, cells p2-p2 by row, style Ledger -->\n|c|\n|-|\n",
        &DocxOptions::default(),
    )
    .unwrap();
    let xml = document_xml(&mixed.docx);
    let styles_of: Vec<&str> = xml
        .split("<w:tbl>")
        .skip(1)
        .map(|t| {
            t.split_once(r#"<w:tblStyle w:val=""#)
                .filter(|(before, _)| !before.contains("</w:tblPr>"))
                .map_or("", |(_, rest)| rest.split('"').next().unwrap())
        })
        .collect();
    assert_eq!(styles_of, ["TableGrid", "TableGrid", "Ledger"], "{xml}");
    let styles = part_string(&mixed.docx, "word/styles.xml").unwrap();
    assert!(
        styles.contains(r#"<w:style w:type="table" w:styleId="Ledger"><w:name w:val="Ledger"/>"#),
        "{styles}"
    );
    assert_word_valid_package(&mixed.docx);
}

/// GFM reads `\\|` in a table row as an escaped backslash then the cell
/// separator; pulldown-cmark shields the pipe behind any backslash, so
/// the escaped rows the reader writes (`|\\|\\\\|`) and hand-written
/// ones come back as the cells they name, and the row is a fixed point.
#[test]
fn backslash_runs_before_a_cell_pipe_split_the_cell() {
    let docx = write("| a \\| b | c \\\\| d |\n|---|---|---|\n| \\\\|\\\\\\\\| z |\n");
    assert_eq!(texts(&docx), ["a | b", "c \\", "d", "\\", "\\\\", "z", ""]);
    let markdown = docx_to_markdown(&docx, &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert!(markdown.contains("|\\\\|\\\\\\\\|z|\n"), "{markdown}");
    let back = markdown_to_docx(&markdown, &DocxOptions::default())
        .unwrap()
        .docx;
    assert_eq!(texts(&back), texts(&docx));
    assert_eq!(
        docx_to_markdown(&back, &MarkdownOptions::default())
            .unwrap()
            .markdown,
        markdown
    );
}

#[test]
fn the_document_owner_is_written_where_the_view_read_it() {
    // `document_owner:` names dc:creator, or cp:lastModifiedBy when its
    // note says the file had no creator; `none` writes neither, and its
    // `no docProps/core.xml` note writes no core properties at all.
    let core = |owner: &str| {
        let written = markdown_to_docx(
            &format!("---\nview: tracked\nauthors:\n  document_owner: {owner}\n---\n\nText.\n"),
            &DocxOptions::default(),
        )
        .unwrap();
        part_string(&written.docx, "docProps/core.xml")
    };
    let creator = core("\"Jones, SA\"        # dc:creator").unwrap();
    assert!(
        creator.contains("<dc:creator>Jones, SA</dc:creator>"),
        "{creator}"
    );
    assert!(!creator.contains("lastModifiedBy"), "{creator}");
    let plain = core("Ann Lee").unwrap();
    assert!(
        plain.contains("<dc:creator>Ann Lee</dc:creator>"),
        "{plain}"
    );
    let editor = core("Last Editor      # cp:lastModifiedBy; no dc:creator").unwrap();
    assert!(
        editor.contains("<cp:lastModifiedBy>Last Editor</cp:lastModifiedBy>"),
        "{editor}"
    );
    assert!(!editor.contains("dc:creator"), "{editor}");
    let nobody =
        core("none             # docProps/core.xml names no dc:creator or cp:lastModifiedBy")
            .unwrap();
    assert!(
        !nobody.contains("dc:creator") && !nobody.contains("lastModifiedBy"),
        "{nobody}"
    );
    let written = markdown_to_docx(
        "---\nview: tracked\nauthors:\n  document_owner: none             # no docProps/core.xml\n---\n\nText.\n",
        &DocxOptions::default(),
    )
    .unwrap();
    assert!(part_string(&written.docx, "docProps/core.xml").is_none());
    let types = part_string(&written.docx, "[Content_Types].xml").unwrap();
    assert!(!types.contains("core.xml"), "{types}");
    let rels = part_string(&written.docx, "_rels/.rels").unwrap();
    assert!(!rels.contains("core-properties"), "{rels}");
    assert_word_valid_package(&written.docx);
}

/// The text of every `<{name}>` element in `core`, outside comments and
/// CDATA, unescaped.
fn core_values(core: &str, name: &str) -> Vec<String> {
    let mut reader = quick_xml::Reader::from_str(core);
    let mut values = Vec::new();
    let mut inside: Option<String> = None;
    loop {
        match reader.read_event().unwrap() {
            quick_xml::events::Event::Start(e) if e.name().as_ref() == name => {
                inside = Some(String::new());
            }
            quick_xml::events::Event::Empty(e) if e.name().as_ref() == name => {
                values.push(String::new());
            }
            quick_xml::events::Event::Text(t) => {
                if let Some(text) = inside.as_mut() {
                    text.push_str(&t.xml10_content());
                }
            }
            quick_xml::events::Event::GeneralRef(r) => {
                if let Some(text) = inside.as_mut() {
                    let entity: &str = &r;
                    text.push_str(match entity {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        other => panic!("entity {other}"),
                    });
                }
            }
            quick_xml::events::Event::End(e) if e.name().as_ref() == name => {
                values.extend(inside.take());
            }
            quick_xml::events::Event::Eof => return values,
            _ => {}
        }
    }
}

#[test]
fn the_document_owner_replaces_whatever_the_reference_names() {
    // The owner is written into the reference's core properties however
    // they spell the element, and only into the element itself: never into
    // a comment or CDATA that mentions it. A reference with no core
    // properties gets them.
    const NAME: &str = r#"Ada & Bo <C> "Q""#;
    let view = "---\nview: tracked\nauthors:\n  document_owner: \"Ada & Bo <C> \\\"Q\\\"\"  # dc:creator\n---\n\nText.\n";
    let core = |inner: &str| {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">{inner}</cp:coreProperties>"#
        )
    };
    let template = write("Text.\n");
    for inner in [
        "<dc:creator>Someone Else</dc:creator>",
        r#"<dc:creator xml:space="preserve">Someone Else</dc:creator>"#,
        r#"<dc:creator xmlns:dc="http://purl.org/dc/elements/1.1/">Someone Else</dc:creator>"#,
        "<!-- <dc:creator>decoy</dc:creator> --><dc:creator>Someone Else</dc:creator>",
        "<dc:description><![CDATA[<dc:creator>decoy</dc:creator>]]></dc:description><dc:creator>Someone Else</dc:creator>",
        "<dc:description><![CDATA[</dc:creator>]]></dc:description><dc:creator>Someone Else</dc:creator>",
        "<dc:creator /><cp:lastModifiedBy>Someone Else</cp:lastModifiedBy>",
        "<dc:creator />",
    ] {
        let reference =
            common::docx::replace_entry(&template, "docProps/core.xml", core(inner).as_bytes());
        let written = write_with(
            view,
            &DocxOptions {
                reference: Some(&reference),
                ..DocxOptions::default()
            },
        );
        let out = part_string(&written, "docProps/core.xml").unwrap();
        assert_eq!(core_values(&out, "dc:creator"), [NAME], "{inner}\n{out}");
        assert!(!out.contains("Someone Else"), "{inner}\n{out}");
        assert!(core_values(&out, "cp:lastModifiedBy").is_empty(), "{out}");
        if inner.contains("decoy") {
            assert!(out.contains("decoy"), "{out}");
        }
    }
    // No core properties in the reference: the owner makes them.
    let bare = common::docx::docx(&common::docx::para("Text."));
    let written = write_with(
        view,
        &DocxOptions {
            reference: Some(&bare),
            ..DocxOptions::default()
        },
    );
    let out = part_string(&written, "docProps/core.xml").expect("core properties written");
    assert_eq!(core_values(&out, "dc:creator"), [NAME], "{out}");
    let rels = part_string(&written, "_rels/.rels").unwrap();
    assert!(rels.contains("core-properties"), "{rels}");
    let types = part_string(&written, "[Content_Types].xml").unwrap();
    assert!(types.contains("/docProps/core.xml"), "{types}");
}

#[test]
fn an_owner_named_none_is_a_name_and_the_owner_line_outranks_author() {
    let core = |front: &str| {
        let written = write(&format!("---\nview: tracked\n{front}---\n\nText.\n"));
        part_string(&written, "docProps/core.xml").unwrap_or_default()
    };
    // The note says a creator holds the name: `none` is that name.
    let named = core("authors:\n  document_owner: none             # dc:creator\n");
    assert_eq!(core_values(&named, "dc:creator"), ["none"], "{named}");
    let quoted = core("authors:\n  document_owner: \"none\"\n");
    assert_eq!(core_values(&quoted, "dc:creator"), ["none"], "{quoted}");
    let editor = core("authors:\n  document_owner: none   # cp:lastModifiedBy; no dc:creator\n");
    assert_eq!(
        core_values(&editor, "cp:lastModifiedBy"),
        ["none"],
        "{editor}"
    );
    assert!(core_values(&editor, "dc:creator").is_empty(), "{editor}");
    // `document_owner:` says who the file names; `author:` does not undo it.
    let both = core(
        "author: Override\nauthors:\n  document_owner: Last Editor  # cp:lastModifiedBy; no dc:creator\n",
    );
    assert!(core_values(&both, "dc:creator").is_empty(), "{both}");
    assert_eq!(
        core_values(&both, "cp:lastModifiedBy"),
        ["Last Editor"],
        "{both}"
    );
    // A quoted title keeps its quotes inside.
    let title = core("title: \"Quarterly & <Report> \\\"Q\\\"\"\n");
    assert_eq!(
        core_values(&title, "dc:title"),
        [r#"Quarterly & <Report> "Q""#],
        "{title}"
    );
}

#[test]
fn no_core_properties_with_a_title_keeps_the_title_and_names_nobody() {
    let template = write("Text.\n");
    let reference = common::docx::replace_entry(
        &template,
        "docProps/core.xml",
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:creator>Someone Else</dc:creator><cp:lastModifiedBy>Other Editor</cp:lastModifiedBy></cp:coreProperties>"#,
    );
    let written = write_with(
        "---\ntitle: Quarterly\nauthors:\n  document_owner: none             # no docProps/core.xml\n---\n\nText.\n",
        &DocxOptions {
            reference: Some(&reference),
            ..DocxOptions::default()
        },
    );
    let out = part_string(&written, "docProps/core.xml").unwrap();
    assert_eq!(core_values(&out, "dc:title"), ["Quarterly"], "{out}");
    assert!(core_values(&out, "dc:creator").is_empty(), "{out}");
    assert!(core_values(&out, "cp:lastModifiedBy").is_empty(), "{out}");
}

#[test]
fn an_anchor_tag_is_the_bookmark_a_link_names() {
    // `<a id="name"></a>` is a bookmark where it stands (the view prints
    // the bookmarks links and reference fields name); a link to it is no
    // missing heading, and no heading takes its name.
    let written = markdown_to_docx(
        "# Up\n\n<a id=\"exec_summary\"></a>Summary and <a name=\"_Ref9\"></a>more.\n\n\
         Back [up](#exec_summary \"Go up\") and [there](#_Ref9).\n",
        &DocxOptions::default(),
    )
    .unwrap();
    let xml = document_xml(&written.docx);
    assert!(
        xml.contains(
            r#"<w:bookmarkStart w:id="0" w:name="exec_summary"/><w:bookmarkEnd w:id="0"/>"#
        ),
        "{xml}"
    );
    assert!(
        xml.contains(r#"<w:bookmarkStart w:id="1" w:name="_Ref9"/><w:bookmarkEnd w:id="1"/>"#),
        "{xml}"
    );
    assert!(
        xml.contains(r#"<w:hyperlink w:anchor="exec_summary" w:tooltip="Go up""#),
        "{xml}"
    );
    assert!(xml.contains(r#"w:anchor="_Ref9""#), "{xml}");
    assert!(
        !written.warnings.iter().any(|w| w.contains("link")),
        "{:?}",
        written.warnings
    );
    let view = jubarte::markdown::docx_to_markdown(
        &written.docx,
        &jubarte::markdown::MarkdownOptions::default(),
    )
    .unwrap()
    .markdown;
    assert!(
        view.contains("<a id=\"exec_summary\"></a>Summary and <a id=\"_Ref9\"></a>more."),
        "{view}"
    );
    assert!(
        view.contains("Back [up](#exec_summary \"Go up\") and [there](#_Ref9)."),
        "{view}"
    );
}

#[test]
fn a_link_to_no_heading_is_written_and_warned_once() {
    let written = markdown_to_docx(
        "# Intro\n\n[a](#intro) [b](#nowhere) [c](#nowhere) [d](#gone)\n\n{--\n# Gone\n--}\n",
        &DocxOptions::default(),
    )
    .unwrap();
    let xml = document_xml(&written.docx);
    assert!(xml.contains("w:anchor=\"nowhere\""), "the link is kept");
    let links: Vec<&String> = written
        .warnings
        .iter()
        .filter(|w| w.contains("link"))
        .collect();
    assert_eq!(links.len(), 2, "{:?}", written.warnings);
    assert!(links[0].contains("#gone"), "{links:?}");
    assert!(links[1].contains("#nowhere"), "{links:?}");
}

#[test]
fn a_self_closing_tag_formats_nothing() {
    let xml = document_xml(&write("plain <b/> after <mark/> more <u/> end\n"));
    assert!(!xml.contains("<w:b/>"), "{xml}");
    assert!(!xml.contains("<w:highlight"), "{xml}");
    assert!(!xml.contains("<w:u "), "{xml}");
}

#[test]
fn a_hyphenated_tag_is_its_own_name() {
    let written = markdown_to_docx(
        "A <i-am-not>x</i-am-not> and <code-of-conduct>y</code-of-conduct>.\n",
        &DocxOptions::default(),
    )
    .unwrap();
    let xml = document_xml(&written.docx);
    assert!(!xml.contains("<w:i/>"), "{xml}");
    assert!(!xml.contains("VerbatimChar"), "{xml}");
    let html: Vec<&String> = written
        .warnings
        .iter()
        .filter(|w| w.contains("HTML"))
        .collect();
    assert_eq!(html.len(), 2, "{:?}", written.warnings);
    assert!(html[0].contains("<i-am-not>"), "{html:?}");
    assert!(html[1].contains("<code-of-conduct>"), "{html:?}");
}

#[test]
fn a_tag_inside_a_comment_never_formats_the_body() {
    let docx = write("before {>>a comment with <b>bold words<<} after\n");
    let xml = document_xml(&docx);
    assert!(!xml.contains("<w:b/>"), "{xml}");
}

#[test]
fn only_a_top_level_track_changes_key_counts() {
    let docx = write("---\ntitle: N\noptions:\n  track_changes: on\n---\n\nText.\n");
    assert!(!tracks(&docx));
    let written = markdown_to_docx(
        "---\ntrack_changes:\n---\n\nText.\n",
        &DocxOptions::default(),
    )
    .unwrap();
    assert!(
        written.warnings.iter().any(|w| w.contains("track_changes")),
        "{:?}",
        written.warnings
    );
}

#[test]
fn a_comment_in_a_footnote_moves_to_its_reference() {
    // Word shows a comment only from the body: the note's comment anchors
    // on the note's reference mark, and the writer says so.
    let written = markdown_to_docx(
        "Text[^1] and again[^1].\n\n[^1]: note {>>a comment<<} here\n",
        &DocxOptions::default(),
    )
    .unwrap();
    let notes = part_string(&written.docx, "word/footnotes.xml").unwrap();
    assert!(!notes.contains("comment"), "{notes}");
    let body = document_xml(&written.docx);
    assert_eq!(body.matches("<w:commentReference").count(), 1, "{body}");
    let start = body.find("<w:commentRangeStart").unwrap();
    let reference = body.find("<w:footnoteReference").unwrap();
    let end = body.find("<w:commentRangeEnd").unwrap();
    assert!(start < reference && reference < end, "{body}");
    assert!(
        written.warnings.iter().any(|w| w.contains("footnote")),
        "{:?}",
        written.warnings
    );
    assert_word_valid_package(&written.docx);
}

#[test]
fn a_comment_in_a_header_is_dropped_with_its_text_kept() {
    let written = markdown_to_docx(
        "---\nsections:\n  1: {p0-p0, headers: {default: {id: header1, text: \"Head {>>hc<<}\"}}}\n---\n\nBody.\n",
        &DocxOptions::default(),
    )
    .unwrap();
    let header = part_string(&written.docx, "word/header1.xml").unwrap();
    assert!(header.contains(">Head <"), "{header}");
    assert!(!header.contains("hc"), "{header}");
    assert!(
        written.warnings.iter().any(|w| w.contains("header")),
        "{:?}",
        written.warnings
    );
}

#[test]
fn code_and_link_destinations_open_no_highlight_field_or_change() {
    // Markdown reads a code span and a link destination literally, so
    // `==`, `{CODE}…{/}` and CriticMarkup open nothing inside them. A fenced
    // block keeps tracked changes (the reader prints them there) but not
    // highlights or fields, which it never prints there.
    let docx = write(
        "See `==x==`, `{PAGE}1{/}` and `{++y++}` here.\n\n\
         Run `==a` and `b==` now.\n\n\
         Get [t](http://e.example/?q=ab==) then ==hl== end.\n\n\
         ```\nkey ==x== {PAGE}1{/} {++z++}\n```\n",
    );
    let xml = document_xml(&docx);
    assert_eq!(xml.matches("<w:highlight ").count(), 1, "{xml}");
    assert!(!xml.contains("<w:fldChar"), "{xml}");
    assert_eq!(xml.matches("<w:ins ").count(), 1, "{xml}");
    let rels = part_string(&docx, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains("http://e.example/?q=ab=="), "{rels}");
    assert_eq!(
        accepted(&docx),
        [
            "See ==x==, {PAGE}1{/} and {++y++} here.",
            "Run ==a and b== now.",
            "Get t then hl end.",
            "key ==x== {PAGE}1{/} z",
        ]
    );
}

#[test]
fn an_unclosed_backtick_autolinks_and_parenthesised_urls_keep_highlights_right() {
    let docx = write(
        "An `unclosed tick ==hl== and <https://a.example/==x==> ==two==.\n\n\
         [p](https://a.example/(x==)) ==y==\n\n\
         ~~~ text\n==z==\n~~~\n",
    );
    let xml = document_xml(&docx);
    assert_eq!(xml.matches("<w:highlight ").count(), 3, "{xml}");
    let rels = part_string(&docx, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains("https://a.example/(x==)"), "{rels}");
    assert_eq!(
        accepted(&docx),
        [
            "An `unclosed tick hl and https://a.example/==x== two.",
            "p y",
            "==z==",
        ]
    );
}

#[test]
fn an_html_block_keeps_its_text() {
    // A raw HTML block (`<div>` up to a blank line) is not Markdown, but its
    // text is the author's: it stays, a paragraph per block element, with
    // a warning that the markup is gone. Script and style bodies are not text.
    let out = markdown_to_docx(
        "<div>\nDiv &amp; text\nhere.<!-- note -->\n<p>Second</p>\n<script>x()</script>\n</div>\n\nAfter.\n",
        &DocxOptions::default(),
    )
    .unwrap();
    assert_eq!(texts(&out.docx), ["Div & text here.", "Second", "After."]);
    assert!(
        out.warnings.iter().any(|w| w.contains("HTML block <div>")),
        "{:?}",
        out.warnings
    );
}

#[test]
fn an_html_anchor_with_href_is_a_link() {
    // `<a href>` is a link like `[t](u)`; one without `href` is text, and
    // one left open closes with its block.
    let docx = write(
        "See <a href=\"https://a.example/x?a=1&amp;b=2\">anchor <b>tag</b></a> and \
         <a name=\"x\">no href</a> now.\n\nOpen <a href='https://b.example/'>unclosed\n\nPlain.\n",
    );
    let xml = document_xml(&docx);
    assert_eq!(xml.matches("<w:hyperlink ").count(), 2, "{xml}");
    let rels = part_string(&docx, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains("https://a.example/x?a=1&amp;b=2"), "{rels}");
    assert!(rels.contains("https://b.example/"), "{rels}");
    assert_eq!(
        texts(&docx),
        ["See anchor tag and no href now.", "Open unclosed", "Plain."]
    );
}

#[test]
fn a_link_title_is_its_tooltip() {
    let docx = write(
        "# Intro\n\n[titled](https://a.example/t \"The & Title\") and [back](#intro 'Go up').\n",
    );
    let xml = document_xml(&docx);
    assert!(xml.contains(r#"w:tooltip="The &amp; Title""#), "{xml}");
    assert!(xml.contains(r#"w:tooltip="Go up""#), "{xml}");
}

#[test]
fn a_brace_inside_a_quoted_field_argument_is_part_of_the_code() {
    let docx = write("A {QUOTE \"a}b\"}X{/} and {QUOTE \"c{d\"}Y{/} B.\n");
    let xml = document_xml(&docx);
    assert_eq!(xml.matches(r#"w:fldCharType="begin""#).count(), 2, "{xml}");
    assert!(
        xml.contains(r#"<w:instrText xml:space="preserve"> QUOTE &quot;a}b&quot; </w:instrText>"#),
        "{xml}"
    );
    assert!(
        xml.contains(r#"<w:instrText xml:space="preserve"> QUOTE &quot;c{d&quot; </w:instrText>"#),
        "{xml}"
    );
    assert_eq!(texts(&docx), ["A X and Y B."]);
}

#[test]
fn a_reference_definitions_destination_is_read_literally() {
    // Like an inline destination, a definition's destination and title
    // open no highlight or field; a footnote definition still does.
    let docx = write(
        "See [x] and [y] then ==hl== end.\n\n\
         [x]: http://e.example/u==v==\n\
         [y]: <http://e.example/{PAGE}1{/}> \"t ==z==\"\n\n\
         A [^1].\n\n\
         [^1]: Note ==n== here.\n\n\
         A paragraph line\n[z]: is no ==definition==.\n",
    );
    let rels = part_string(&docx, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains("http://e.example/u==v=="), "{rels}");
    assert!(rels.contains("http://e.example/{PAGE}1{/}"), "{rels}");
    let xml = document_xml(&docx);
    assert_eq!(xml.matches("<w:highlight ").count(), 2, "{xml}");
    assert!(!xml.contains("<w:fldChar"), "{xml}");
    let notes = part_string(&docx, "word/footnotes.xml").unwrap();
    assert_eq!(notes.matches("<w:highlight ").count(), 1, "{notes}");
}

#[test]
fn a_front_matter_story_that_does_not_read_warns() {
    // YAML reads no `{` in a plain scalar: the footer is dropped, and says
    // so. A single-quoted scalar keeps the field's backslash.
    let written = markdown_to_docx(
        "---\nfooters:\n  default: {id: footer1, text: Page {PAGE} of {NUMPAGES}}\n\
         headers:\n  default: {id: header1, text: '{REF _R \\h}hi{/}'}\n  \
         first: {id: header2, text: \"{REF _R}hi{/}\"}\n---\n\nBody.\n",
        &DocxOptions::default(),
    )
    .unwrap();
    assert!(part_string(&written.docx, "word/footer1.xml").is_none());
    assert!(
        written
            .warnings
            .iter()
            .any(|w| w.contains("footers: default")),
        "{:?}",
        written.warnings
    );
    assert!(
        !written.warnings.iter().any(|w| w.contains("headers")),
        "{:?}",
        written.warnings
    );
    // The front matter is no Markdown: a field in it is not encoded, so
    // its code keeps its characters and is written as the field it names.
    let header = part_string(&written.docx, "word/header1.xml").unwrap();
    assert!(
        header.contains(r#"<w:fldSimple w:instr=" REF _R \h "><w:r><w:t xml:space="preserve">hi</w:t></w:r></w:fldSimple>"#),
        "{header}"
    );
    let first = part_string(&written.docx, "word/header2.xml").unwrap();
    assert!(
        first.contains(r#"<w:fldSimple w:instr=" REF _R ">"#),
        "{first}"
    );
}

/// The paragraphs of `docx`, each with whether it ends in a page break.
fn breaks(docx: &[u8]) -> Vec<(String, bool)> {
    let xml = document_xml(docx);
    xml.split("<w:p>")
        .chain(xml.split("<w:p ").skip(1))
        .skip(1)
        .filter_map(|p| p.split("</w:p>").next())
        .map(|p| {
            let text: String = p
                .split("<w:t")
                .skip(1)
                .filter_map(|t| {
                    t.split_once('>')
                        .and_then(|(_, rest)| rest.split("</w:t>").next())
                })
                .collect();
            (text, p.contains(r#"<w:br w:type="page"/>"#))
        })
        .collect()
}

/// LaTeX's page commands, as Pandoc manuscripts write them on a line of
/// their own, are page breaks: the paragraph before them holds the break,
/// as Word puts one (2026-10-14, the git_manuscript fixture).
#[test]
fn latex_page_commands_are_page_breaks() {
    for command in [
        r"\newpage",
        r"\pagebreak",
        r"\clearpage",
        r"\cleardoublepage",
    ] {
        // Right under a line of text, and alone between blank lines.
        for gap in ["\n", "\n\n"] {
            let markdown = format!("Keywords line.{gap}{command}\n\n# Findings\n\nText.\n");
            let docx = write(&markdown);
            let paragraphs = breaks(&docx);
            assert!(
                paragraphs.contains(&("Keywords line.".to_string(), true)),
                "{command:?} {gap:?}: {paragraphs:?}"
            );
            assert!(
                !paragraphs.iter().any(|(text, _)| text.contains(command)),
                "{command:?} {gap:?}: {paragraphs:?}"
            );
            assert_eq!(texts(&docx), ["Keywords line.", "Findings", "Text."]);
        }
    }
    // A document that opens with one: an empty paragraph holds the break.
    let docx = write("\\newpage\n\nAfter.\n");
    assert_eq!(
        breaks(&docx),
        [(String::new(), true), ("After.".to_string(), false)]
    );
    // Code keeps the command as text.
    let docx = write("```\n\\newpage\n```\n");
    assert!(document_xml(&docx).contains(r"\newpage"));
    assert!(!document_xml(&docx).contains(r#"w:type="page""#));
}

/// The plain Markdown writer says a page break in the view's own words, a
/// `<!-- page-break -->` line after the paragraph that holds it, and reads
/// it back as the break.
#[test]
fn a_page_break_round_trips_through_plain_markdown() {
    let docx = write("Keywords line.\n\\newpage\n\n# Findings\n");
    let markdown =
        jubarte::markdown::docx_to_markdown(&docx, &jubarte::markdown::MarkdownOptions::default())
            .unwrap()
            .markdown;
    assert!(
        markdown.contains("Keywords line.\n\n<!-- page-break -->\n\n# Findings"),
        "{markdown}"
    );
    let again = write(&markdown);
    assert_eq!(breaks(&again), breaks(&docx));
}

fn scholar(markdown: &str) -> Vec<u8> {
    write_with(
        markdown,
        &DocxOptions {
            scholar: true,
            ..DocxOptions::default()
        },
    )
}

/// The style definition of `id` in the styles part.
fn style(docx: &[u8], id: &str) -> String {
    let styles = part_string(docx, "word/styles.xml").unwrap();
    let open = format!("w:styleId=\"{id}\"");
    let at = styles.find(&open).unwrap_or_else(|| panic!("no {id}"));
    styles[at..].split("</w:style>").next().unwrap().to_string()
}

/// `scholar` sets a manuscript as LaTeX's article class does: Latin Modern
/// Roman at 10pt on 12pt lines, justified, 6pt between paragraphs, bold
/// headings at 14.4, 12 and 10pt, and the page number centered in the
/// footer from page 2 (the title page has none).
#[test]
fn scholar_sets_a_manuscript_as_latex_does() {
    let docx = scholar("# Introduction\n\nText.\n\\newpage\n\n## Methods\n\nMore.\n");
    let normal = style(&docx, "Normal");
    for expected in [
        r#"w:ascii="Latin Modern Roman""#,
        r#"<w:sz w:val="20"/>"#,
        r#"w:line="240" w:lineRule="exact""#,
        r#"w:after="120""#,
        r#"<w:jc w:val="both"/>"#,
    ] {
        assert!(normal.contains(expected), "{expected}: {normal}");
    }
    // A heading takes Latin Modern Roman from Normal, which it is based
    // on, so it names no font of its own. Heading 3 is 10pt like Normal,
    // so it names no size either.
    for (id, size) in [
        ("Heading1", Some("29")),
        ("Heading2", Some("24")),
        ("Heading3", None),
    ] {
        let heading = style(&docx, id);
        assert!(!heading.contains("<w:rFonts"), "{id}: {heading}");
        match size {
            Some(size) => {
                let sz = format!(r#"<w:sz w:val="{size}"/>"#);
                assert!(heading.contains(&sz), "{id} {sz}: {heading}");
            }
            None => assert!(!heading.contains("<w:sz "), "{id}: {heading}"),
        }
        for expected in [
            r#"<w:basedOn w:val="Normal"/>"#.to_string(),
            "<w:b/>".to_string(),
            r#"w:lineRule="auto""#.to_string(),
            r#"<w:jc w:val="left"/>"#.to_string(),
            "<w:keepNext/>".to_string(),
        ] {
            assert!(heading.contains(&expected), "{id} {expected}: {heading}");
        }
    }
    let body = document_xml(&docx);
    assert!(body.contains("<w:titlePg/>"), "{body}");
    assert!(
        body.contains(r#"<w:footerReference w:type="default""#),
        "{body}"
    );
    let footer = part_string(&docx, "word/footer1.xml").unwrap();
    assert!(footer.contains(" PAGE "), "{footer}");
    assert!(footer.contains(r#"<w:jc w:val="center"/>"#), "{footer}");
    // A line that ends in a line break (an author block, an address) is
    // not stretched to the margin, as LaTeX's `\\` is not.
    let settings = part_string(&docx, "word/settings.xml").unwrap();
    assert!(
        settings.contains("<w:compat><w:doNotExpandShiftReturn /><w:compatSetting"),
        "{settings}"
    );
    // Without it, none of this.
    let plain = write("# Introduction\n\nText.\n");
    assert!(
        !part_string(&plain, "word/styles.xml")
            .unwrap()
            .contains("Latin Modern")
    );
    assert!(!document_xml(&plain).contains("<w:footerReference"));
    assert!(
        !part_string(&plain, "word/settings.xml")
            .unwrap()
            .contains("doNotExpandShiftReturn")
    );
}

/// What the front matter says wins: its own styles and footers stay, and
/// `scholar` fills in only the styles it does not name.
#[test]
fn scholar_keeps_what_the_front_matter_says() {
    let docx = scholar(
        "---\nstyles:\n  Normal: Arial 11pt\nfooters:\n  default: {id: footer1, text: Draft}\n---\n\n# Title\n\nText.\n",
    );
    let normal = style(&docx, "Normal");
    assert!(normal.contains(r#"w:ascii="Arial""#), "{normal}");
    assert!(!normal.contains("Latin Modern"), "{normal}");
    assert!(style(&docx, "Heading1").contains("Latin Modern Roman"));
    let footer = part_string(&docx, "word/footer1.xml").unwrap();
    assert!(
        footer.contains("Draft") && !footer.contains("PAGE"),
        "{footer}"
    );
    assert!(!document_xml(&docx).contains("<w:titlePg/>"));
    // A page number in a header is the page number: no footer is added.
    let docx =
        scholar("---\nheaders:\n  default: {id: header1, text: \"{PAGE}\", right}\n---\n\nText.\n");
    assert!(!document_xml(&docx).contains("<w:footerReference"));
}

/// Only a header or footer story that holds `{PAGE}` is the page number
/// (sourcery review of #472): the words elsewhere in the front matter are
/// not, and an empty `headers:` block is no header for page 1. A reference
/// document's own headers and footers win, as the front matter's do.
#[test]
fn scholar_numbers_the_pages_unless_a_story_holds_the_number() {
    let numbered =
        |docx: &[u8]| part_string(docx, "word/footer1.xml").is_some_and(|f| f.contains(" PAGE "));
    let docx = scholar("---\ntitle: \"{PAGE} turners\"\n---\n\nText.\n");
    assert!(numbered(&docx));
    assert!(document_xml(&docx).contains("<w:titlePg/>"));
    let docx = scholar("---\nheaders:\n---\n\nText.\n");
    assert!(numbered(&docx));
    assert!(
        document_xml(&docx).contains("<w:titlePg/>"),
        "an empty headers: block"
    );
    let reference =
        write("---\nfooters:\n  default: {id: footer1, text: Template footer}\n---\n\nX\n");
    let docx = write_with(
        "Text.\n",
        &DocxOptions {
            scholar: true,
            reference: Some(&reference),
            ..DocxOptions::default()
        },
    );
    let footers: String = (1..4)
        .filter_map(|n| part_string(&docx, &format!("word/footer{n}.xml")))
        .collect();
    assert!(footers.contains("Template footer"), "{footers}");
    assert!(!footers.contains(" PAGE "), "{footers}");
}

/// md -> docx is a function of its input (#534): undated marks of an author
/// dated `first..last` stamp their first `last` day on whichever mark the
/// `row_marks`/`cell_marks`/`row_fmt`/`cell_fmt` maps hand to `format_mark`
/// first, so a `HashMap`'s random order must not reach the bytes.
#[test]
fn converting_the_same_markdown_is_deterministic() {
    let source = std::fs::read_to_string("tests/fixtures/md-roundtrip/potpourri.md")
        .expect("the committed fixture");
    let first = markdown_to_docx(&source, &DocxOptions::default())
        .unwrap()
        .docx;
    for _ in 0..40 {
        let docx = markdown_to_docx(&source, &DocxOptions::default())
            .unwrap()
            .docx;
        assert_eq!(first, docx, "the same Markdown wrote a different docx");
    }
}
