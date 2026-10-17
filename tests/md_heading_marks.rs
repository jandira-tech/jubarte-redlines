// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A heading's runs keep the marks that show on their text: underline,
//! highlight and strike print inside the heading's text, as they do in a
//! body paragraph, and come back through the round trip.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

fn view(docx: &[u8]) -> String {
    let options = ReadOptions {
        page_markers: false,
        track_changes: TrackChanges::All,
        ..ReadOptions::default()
    };
    read(docx, &options)
        .expect("the docx reads")
        .markdown
        .lines()
        .filter(|l| !l.starts_with("sha256: ") && !l.starts_with("body: "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text of `document`'s runs whose properties hold `mark`, in order.
fn marked_text(document: &str, mark: &str) -> String {
    document
        .split("</w:r>")
        .filter_map(|run| {
            let run = &run[run.rfind("<w:r>").or_else(|| run.rfind("<w:r "))?..];
            let props = run
                .split_once("<w:rPr>")
                .and_then(|(_, rest)| rest.split_once("</w:rPr>"))
                .map_or("", |(props, _)| props);
            props.contains(mark).then(|| {
                run.split("<w:t")
                    .skip(1)
                    .filter_map(|t| t.split_once('>').map(|(_, rest)| rest))
                    .filter_map(|t| t.split_once("</w:t>").map(|(text, _)| text))
                    .collect::<String>()
            })
        })
        .collect()
}

/// A level-1 heading whose `lit` run carries `props` after a plain `word `,
/// in a document the writer itself seeded: its styles part already holds
/// the heading style, so the round trip's two views print the same legend.
fn heading_docx(props: &str) -> Vec<u8> {
    let seed = markdown_to_docx("# word lit\n", &DocxOptions::default())
        .expect("the seed reads")
        .docx;
    let document = common::docx::part_string(&seed, "word/document.xml").unwrap();
    let at = document.find("<w:r>").expect("the seed holds one run");
    let end = document[at..].find("</w:r>").expect("the run ends") + at;
    let runs = format!(
        "<w:r><w:t xml:space=\"preserve\">word </w:t></w:r>\
         <w:r><w:rPr>{props}</w:rPr><w:t xml:space=\"preserve\">lit</w:t></w:r>"
    );
    let spliced = format!("{}{runs}{}", &document[..at], &document[end + 6..]);
    common::docx::replace_entry(&seed, "word/document.xml", spliced.as_bytes())
}

/// The heading round-tripped: the view, the rebuilt document and the view
/// again.
fn round_trip(props: &str) -> (String, String, String) {
    let source = heading_docx(props);
    let first = view(&source);
    let again = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let document = common::docx::part_string(&again, "word/document.xml").unwrap();
    let second = view(&again);
    (first, document, second)
}

#[test]
fn an_underlined_run_in_a_heading_keeps_its_underline() {
    let (first, document, second) = round_trip("<w:u w:val=\"single\"/>");
    assert!(first.contains("# word <u>lit</u>"), "{first}");
    assert_eq!(
        marked_text(&document, "<w:u "),
        "lit",
        "{first}\n{document}"
    );
    assert_eq!(second, first);
}

#[test]
fn a_highlighted_run_in_a_heading_keeps_its_highlight() {
    let (first, document, second) = round_trip("<w:highlight w:val=\"yellow\"/>");
    assert!(first.contains("# word <mark>lit</mark>"), "{first}");
    assert_eq!(
        marked_text(&document, "<w:highlight"),
        "lit",
        "{first}\n{document}"
    );
    assert_eq!(second, first);
}

#[test]
fn a_struck_run_in_a_heading_keeps_its_strike() {
    let (first, document, second) = round_trip("<w:strike/>");
    assert!(first.contains("# word ~~lit~~"), "{first}");
    assert_eq!(
        marked_text(&document, "<w:strike"),
        "lit",
        "{first}\n{document}"
    );
    assert_eq!(second, first);
}
