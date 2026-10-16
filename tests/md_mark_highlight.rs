// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! The view prints a Word highlight as `<mark>…</mark>`, so `{==…==}` only
//! ever marks a comment's scope. With `==` for both, a highlight at the
//! edge of a scope printed `{====lit==…==}` and Word got the literal text
//! `==lit==`; text ending in `=` beside a highlight printed `a===b==`.
//! Hand-written `==x==` still reads as a highlight.

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

fn write(markdown: &str) -> Vec<u8> {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the Markdown reads")
        .docx;
    common::validity::assert_word_valid_package(&docx);
    docx
}

fn document(docx: &[u8]) -> String {
    common::docx::part_string(docx, "word/document.xml").unwrap()
}

/// The text of `document`'s runs that hold a highlight, in order.
fn highlighted(document: &str) -> String {
    document
        .split("</w:r>")
        .filter_map(|run| {
            let run = &run[run.rfind("<w:r>").or_else(|| run.rfind("<w:r "))?..];
            run.contains("<w:highlight").then(|| {
                run.split("<w:t")
                    .skip(1)
                    .filter_map(|t| t.split_once('>').map(|(_, rest)| rest))
                    .filter_map(|t| t.split_once("</w:t>").map(|(text, _)| text))
                    .collect::<String>()
            })
        })
        .collect()
}

/// All of `document`'s text.
fn text(document: &str) -> String {
    document
        .split("<w:t")
        .skip(1)
        .filter_map(|t| t.split_once('>').map(|(_, rest)| rest))
        .filter_map(|t| t.split_once("</w:t>").map(|(text, _)| text))
        .collect()
}

#[test]
fn a_highlight_prints_as_mark() {
    let body = "<w:p><w:r><w:t xml:space=\"preserve\">Plain </w:t></w:r>\
                <w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr><w:t>lit</w:t></w:r></w:p>";
    let first = view(&docx(body));
    assert!(first.contains("Plain <mark>lit</mark>"), "{first}");
    assert!(!first.contains("=="), "{first}");
}

#[test]
fn a_highlight_at_a_comment_scopes_edge_comes_back_highlighted() {
    let source = write("A {==<mark>lit</mark> and plain==}{>>note<<} b.\n");
    let first = view(&source);
    assert!(
        first.contains("{==<mark>lit</mark> and plain==}"),
        "{first}"
    );
    let again = document(&write(&first));
    assert_eq!(highlighted(&again), "lit", "{first}\n{again}");
    assert!(!text(&again).contains("=="), "{again}");
    assert_eq!(view(&write(&first)), first);
}

#[test]
fn an_equals_sign_beside_a_highlight_stays_text() {
    let body = "<w:p><w:r><w:t>a=</w:t></w:r>\
                <w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr><w:t>b</w:t></w:r>\
                <w:r><w:t>=c</w:t></w:r></w:p>";
    let first = view(&docx(body));
    let again = document(&write(&first));
    assert_eq!(highlighted(&again), "b", "{first}\n{again}");
    assert_eq!(text(&again), "a=b=c", "{first}");
}

#[test]
fn hand_written_equals_marks_still_highlight() {
    let again = document(&write("Some ==marked== text.\n"));
    assert_eq!(highlighted(&again), "marked");
}

#[test]
fn a_non_yellow_highlight_prints_with_its_colour_class() {
    // Spec §3: `<mark>` is the highlight tag; its `class="hl-<colour>"`
    // says which colour Word holds (`hl-yellow` is the default and the
    // tag may omit it). The view grammar prints the colour so the
    // round-trip keeps it.
    let body = "<w:p><w:r><w:t xml:space=\"preserve\">Cyan </w:t></w:r>\
                <w:r><w:rPr><w:highlight w:val=\"cyan\"/></w:rPr><w:t>lit</w:t></w:r>\
                <w:r><w:t xml:space=\"preserve\">. </w:t></w:r>\
                <w:r><w:rPr><w:highlight w:val=\"green\"/></w:rPr><w:t>lit</w:t></w:r>\
                <w:r><w:t xml:space=\"preserve\">. </w:t></w:r>\
                <w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr><w:t>lit</w:t></w:r></w:p>";
    let first = view(&docx(body));
    assert!(
        first.contains("<mark class=\"hl-cyan\">lit</mark>"),
        "{first}"
    );
    assert!(
        first.contains("<mark class=\"hl-green\">lit</mark>"),
        "{first}"
    );
    assert!(first.contains("<mark>lit</mark>"), "{first}");
}

#[test]
fn a_non_yellow_highlight_round_trips_its_colour() {
    // Spec §3: the colour a Word highlight holds survives docx → md → docx;
    // each paragraph run keeps `<w:highlight w:val="<colour>"/>` with the
    // colour it had on the way in.
    let body = "<w:p><w:r><w:t xml:space=\"preserve\">. </w:t></w:r>\
                <w:r><w:rPr><w:highlight w:val=\"cyan\"/></w:rPr><w:t>lit</w:t></w:r>\
                <w:r><w:t xml:space=\"preserve\">. </w:t></w:r>\
                <w:r><w:rPr><w:highlight w:val=\"green\"/></w:rPr><w:t>lit</w:t></w:r>\
                <w:r><w:t xml:space=\"preserve\">. </w:t></w:r></w:p>";
    let first = view(&docx(body));
    let again = document(&write(&first));
    let colours = highlight_colours(&again);
    assert_eq!(colours, vec!["cyan", "green"], "{first}\n{again}");
}

#[test]
fn a_yellow_highlight_keeps_its_single_run() {
    // Spec §3: a highlight's text is one run on the page; pulldown-cmark's
    // split on `_` does not split the run because adjacent runs with the
    // same format merge before they leave the writer.
    let body =
        "<w:p><w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr><w:t>file_</w:t></w:r></w:p>";
    let first = view(&docx(body));
    assert!(first.contains("<mark>file\\_</mark>"), "{first}");
    let again = document(&write(&first));
    let runs = highlight_runs(&again);
    assert_eq!(runs, vec!["file_"], "{first}\n{again}");
}

#[test]
fn an_unclosed_mark_does_not_colour_a_later_equals_mark() {
    // A tag left open closes with its block; the `==` marks of a later
    // paragraph are the default yellow, not the colour the unclosed tag
    // pushed (spec §3).
    let source = "<mark class=\"hl-cyan\">cyan\n\ntext ==yellow==\n";
    let again = document(&write(source));
    assert_eq!(highlight_colours(&again), vec!["cyan", "yellow"], "{again}");
}

#[test]
fn a_colour_word_does_not_name_prints_as_yellow() {
    // `w:val` outside Word's 16 highlight names (schema-invalid, but a
    // file can hold one) keeps the highlight: it prints and comes back
    // as the default yellow.
    let body = "<w:p><w:r><w:rPr><w:highlight w:val=\"sunny\"/></w:rPr><w:t>lit</w:t></w:r></w:p>";
    let first = view(&docx(body));
    assert!(first.contains("<mark>lit</mark>"), "{first}");
    let again = document(&write(&first));
    assert_eq!(
        highlight_colours(&again),
        vec!["yellow"],
        "{first}\n{again}"
    );
}

/// The `<w:t>` text inside `document`'s `<w:tag>…</w:tag>` marks, in order.
fn marked_text(document: &str, tag: &str) -> String {
    document
        .split(&format!("</w:{tag}>"))
        .filter_map(|part| {
            // `<w:del ` or `<w:del>`, never `<w:delText`.
            part.rfind(&format!("<w:{tag} "))
                .or_else(|| part.rfind(&format!("<w:{tag}>")))
                .map(|at| &part[at..])
        })
        .flat_map(|part| {
            part.split("<w:t")
                .skip(1)
                .chain(part.split("<w:delText").skip(1))
        })
        .filter_map(|t| t.split_once('>').map(|(_, rest)| rest))
        .filter_map(|t| {
            t.split_once("</w:t>")
                .or_else(|| t.split_once("</w:delText>"))
                .map(|(text, _)| text)
        })
        .collect()
}

#[test]
fn a_change_after_a_task_box_holds_only_its_own_text() {
    // The task box and the plain text after it are the paragraph's own
    // runs: the writer's merge of equal runs must stop at the box (a
    // prefix run the paragraph's first change claims), or the box and
    // the plain text ride along into the revision.
    let again = document(&write("- [ ] plain {++inserted++}\n"));
    assert_eq!(marked_text(&again, "ins"), "inserted", "{again}");
    assert_eq!(text(&again), "\u{2610} plain inserted", "{again}");
    let again = document(&write("- [x] plain {--gone--}\n"));
    assert_eq!(marked_text(&again, "del"), "gone", "{again}");
    assert!(text(&again).starts_with("\u{2612} plain "), "{again}");
    let again = document(&write("- [ ] plain {~~old~>new~~}\n"));
    assert_eq!(marked_text(&again, "ins"), "new", "{again}");
    assert_eq!(marked_text(&again, "del"), "old", "{again}");
    assert!(text(&again).starts_with("\u{2610} plain "), "{again}");
}

#[test]
fn a_colour_class_reads_without_regard_to_case() {
    // A hand-edited `hl-Cyan` still names cyan (MARKDOWN.md: "attributes
    // and case do not matter"), not the default yellow.
    let again = document(&write("x <mark class=\"hl-Cyan\">lit</mark> y\n"));
    assert_eq!(highlight_colours(&again), vec!["cyan"], "{again}");
    let again = document(&write("x <mark class=\"hl-DARKBLUE\">lit</mark> y\n"));
    assert_eq!(highlight_colours(&again), vec!["darkBlue"], "{again}");
}

/// The text of every run with highlight that the writer wrote, in order.
fn highlight_runs(document: &str) -> Vec<String> {
    document
        .split("</w:r>")
        .filter_map(|run| {
            let run = &run[run.rfind("<w:r>").or_else(|| run.rfind("<w:r "))?..];
            run.contains("<w:highlight").then(|| {
                run.split("<w:t")
                    .skip(1)
                    .filter_map(|t| t.split_once('>').map(|(_, rest)| rest))
                    .filter_map(|t| t.split_once("</w:t>").map(|(text, _)| text))
                    .collect::<String>()
            })
        })
        .collect()
}

/// The colour of every run with a highlight, in order.
fn highlight_colours(document: &str) -> Vec<String> {
    let mut colours = Vec::new();
    let mut rest = document;
    while let Some(at) = rest.find("<w:highlight") {
        let after = &rest[at..];
        let end = after.find('>').unwrap();
        let tag = &after[..end + 1];
        let val = tag
            .split_once("w:val=\"")
            .and_then(|(_, rest)| rest.split_once('"').map(|(v, _)| v.to_string()))
            .unwrap_or_default();
        colours.push(val);
        rest = &after[end + 1..];
    }
    colours
}
