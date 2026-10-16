// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A run's spaces keep the marks that show on a space: underline,
//! highlight and strike. file_170.docx's "yellow-highlighted, " is
//! highlighted and underlined up to the next word; the view moved the
//! space outside `==…==` and `<u>…</u>`, so the round trip broke the
//! highlight and the underline at every space. A space Word left plain
//! between two underlined words stays plain.

mod common;

use common::docx::docx;
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

fn round_trip(docx: &[u8]) -> (String, String, String) {
    let first = view(docx);
    let again = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let document = common::docx::part_string(&again, "word/document.xml").unwrap();
    let second = view(&again);
    (first, document, second)
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

fn run(props: &str, text: &str) -> String {
    format!("<w:r><w:rPr>{props}</w:rPr><w:t xml:space=\"preserve\">{text}</w:t></w:r>")
}

const MARKED: &str = "<w:highlight w:val=\"yellow\"/><w:u w:val=\"single\"/>";

#[test]
fn a_highlighted_runs_spaces_stay_highlighted() {
    let struck = format!("<w:strike/>{MARKED}");
    let body = format!(
        "<w:p>{}{}{}{}</w:p>",
        run("", "Plain "),
        run(MARKED, "yellow-highlighted, "),
        run(&struck, "strikethrough, "),
        run(&struck, "mono.")
    );
    let (first, document, second) = round_trip(&docx(&body));
    assert_eq!(
        marked_text(&document, "<w:highlight"),
        "yellow-highlighted, strikethrough, mono.",
        "{first}\n{document}"
    );
    assert_eq!(
        marked_text(&document, "<w:u "),
        "yellow-highlighted, strikethrough, mono."
    );
    assert_eq!(marked_text(&document, "<w:strike"), "strikethrough, mono.");
    assert_eq!(second, first);
}

#[test]
fn a_plain_space_between_underlined_words_stays_plain() {
    let underline = "<w:u w:val=\"single\"/>";
    let body = format!(
        "<w:p>{}{}{}</w:p>",
        run(underline, "one"),
        run("", " "),
        run(underline, "two")
    );
    let (first, document, second) = round_trip(&docx(&body));
    assert_eq!(
        marked_text(&document, "<w:u "),
        "onetwo",
        "{first}\n{document}"
    );
    assert_eq!(second, first);
}

#[test]
fn a_leading_space_keeps_its_strike() {
    let body = format!(
        "<w:p>{}{}</w:p>",
        run("", "before"),
        run("<w:strike/>", " struck")
    );
    let (first, document, second) = round_trip(&docx(&body));
    assert_eq!(
        marked_text(&document, "<w:strike"),
        " struck",
        "{first}\n{document}"
    );
    assert_eq!(second, first);
}
