// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! What the Markdown round trip lost (tests/md_fixed_point.rs, run of
//! 2026-10-14), one shape each: the view written to Word and read back
//! prints the same line.

use std::path::Path;
use std::process::Command;

mod common;

use common::docx::{Part, R_NS, docx_with, para, part_string, replace_entry};

fn jubarte(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

const HEADER: &str = "---\n\
view: tracked\n\
authors:\n  SW: S Wheeler                    # 2026-09-25T20:04:00Z\n\
---\n\n";

/// `body` under a view header, written to Word and read back: the body
/// lines of the second view, and the docx.
fn round_trip(body: &str) -> (String, Vec<u8>) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.md"), format!("{HEADER}{body}")).unwrap();
    jubarte(
        dir.path(),
        &[
            "convert",
            "in.md",
            "-o",
            "out.docx",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    let view = jubarte(dir.path(), &["read", "out.docx", "--no-page-markers"]);
    let body = view
        .splitn(3, "---\n")
        .nth(2)
        .unwrap_or_default()
        .to_string();
    (body, std::fs::read(dir.path().join("out.docx")).unwrap())
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut text = String::new();
    std::io::Read::read_to_string(&mut zip.by_name(name).unwrap(), &mut text).unwrap();
    text
}

/// A link inside one tracked change is one change: Word's own shape, a
/// HYPERLINK field inside the one `w:del`, not a `w:hyperlink` that parts
/// the mark in two (`r213` read back as `r213+361`).
#[test]
fn a_link_inside_a_change_stays_one_change() {
    let line =
        "{--Title: [Ida Greaves](https://eur.on.worldcat.org/oclc/1381732921)--}{>>r213 @SW<<}";
    let (body, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
    assert!(body.contains(&format!("{line}\n")), "{body}");
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    assert!(
        document.contains("HYPERLINK &quot;https://eur.on.worldcat.org/oclc/1381732921&quot;")
            || document.contains("HYPERLINK \"https://eur.on.worldcat.org/oclc/1381732921\""),
        "{document}"
    );
    assert!(document.contains("<w:delInstrText"), "{document}");
    // An inserted link the same; a plain link stays a hyperlink element.
    let inserted = "{++See [the site](https://example.com) now++}{>>r4 @SW<<}";
    let (body, docx) = round_trip(&format!(
        "<!-- p0 -->\n{inserted}\n\n<!-- p1 -->\n[plain](https://example.org)\n"
    ));
    assert!(body.contains(&format!("{inserted}\n")), "{body}");
    assert!(body.contains("[plain](https://example.org)\n"), "{body}");
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert!(document.contains("<w:instrText"), "{document}");
    assert_eq!(document.matches("<w:hyperlink ").count(), 1, "{document}");
}

/// A heading in a bulleted list prints its bullet before its text
/// (`## - Economics`); the bullet is the list's, not text to write.
#[test]
fn a_bulleted_heading_keeps_one_bullet() {
    let body = "<!-- p0 bullet -->\n## - {--Economics & business--}{>>r211 @SW<<}\n";
    let (got, docx) = round_trip(body);
    assert!(
        got.contains("## - {--Economics & business--}{>>r211 @SW<<}\n"),
        "{got}"
    );
    assert!(!got.contains("## - - "), "{got}");
    assert!(!part(&docx, "word/document.xml").contains(">- <"));
}

/// The definition of the table style `id` in the styles part.
fn table_style(docx: &[u8], id: &str) -> String {
    let styles = part(docx, "word/styles.xml");
    let open = format!("w:styleId=\"{id}\"");
    let at = styles
        .find(&open)
        .unwrap_or_else(|| panic!("no {id}: {styles}"));
    styles[at..].split("</w:style>").next().unwrap().to_string()
}

/// A table under one of Word's built-in table styles (`Grid Table 4 Accent
/// 1`) keeps its look: the view names it by id, and the writer writes
/// Word's own definition, its colours from the document's theme (Word's
/// default theme here), not a bare style with the id for a name.
#[test]
fn a_built_in_table_style_keeps_its_definition() {
    let body = "<!-- t0 2x2, cells p0-p3 by row, style GridTable4-Accent1 -->\n|a|b|\n|-|-|\n|c|d|\n\n\
                <!-- t1 2x2, cells p4-p7 by row, style GridTable6Colorful -->\n|a|b|\n|-|-|\n|c|d|\n";
    let (got, docx) = round_trip(body);
    assert!(got.contains("style GridTable4-Accent1 -->"), "{got}");
    let style = table_style(&docx, "GridTable4-Accent1");
    for want in [
        "<w:name w:val=\"Grid Table 4 Accent 1\"/>",
        "<w:tblStylePr w:type=\"firstRow\">",
        // The header row: accent 1 itself; the bands its 20% tint.
        "w:fill=\"156082\" w:themeFill=\"accent1\"/>",
        "w:fill=\"C1E4F5\" w:themeFill=\"accent1\" w:themeFillTint=\"33\"/>",
        "w:color=\"45B0E1\" w:themeColor=\"accent1\" w:themeTint=\"99\"/>",
    ] {
        assert!(style.contains(want), "{want}: {style}");
    }
    assert!(
        !style.contains("\"@\"") && !style.contains("{C}"),
        "{style}"
    );
    let plain = table_style(&docx, "GridTable6Colorful");
    assert!(
        plain.contains("<w:name w:val=\"Grid Table 6 Colorful\"/>"),
        "{plain}"
    );
    assert!(
        plain.contains("w:color=\"666666\" w:themeColor=\"text1\" w:themeTint=\"99\"/>"),
        "{plain}"
    );
    // A name Word has no definition for stays the bare style it was.
    let (_, docx) = round_trip("<!-- t0 1x1, cells p0-p0 by row, style Ledger -->\n|a|\n|-|\n");
    assert!(table_style(&docx, "Ledger").contains("<w:name w:val=\"Ledger\"/>"));
}

/// A built-in table style's borders are Word's own: the writer no longer
/// warns that it writes the style without them.
#[test]
fn a_built_in_table_style_writes_its_borders_unwarned() {
    let dir = tempfile::tempdir().unwrap();
    let header = "---\nview: tracked\nstyles:\n  Normal: Calibri 11pt  # default\n  table: GridTable4-Accent5, borders vary\n---\n\n";
    let body = "<!-- t0 1x1, cells p0-p0 by row -->\n|a|\n|-|\n";
    for (style, warned) in [("GridTable4-Accent5", false), ("Ledger", true)] {
        let md = format!("{header}{body}").replace("GridTable4-Accent5", style);
        std::fs::write(dir.path().join("t.md"), md).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(["convert", "t.md", "-o", "t.docx", "--force"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(out.status.success());
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(stderr.contains("borders vary"), warned, "{style}: {stderr}");
    }
}

/// A field whose result is whole paragraphs (a table of contents) opens at
/// the start of its first paragraph and closes with `{/}` in a later one:
/// its paragraphs between are its result, and the field survives the round
/// trip, not just the fields inside it.
#[test]
fn a_field_across_paragraphs_keeps_its_code() {
    let body = "<!-- p0 -->\n{TOC \\o \"1-3\" \\h \\z}Intro\t{PAGEREF _Toc1 \\h}1{/}\n\n\
                <!-- p1 -->\nTerms\t{PAGEREF _Toc2 \\h}2{/}\n\n\
                <!-- p2 -->\n{/}\n\n\
                <!-- p3 -->\nBody text.\n";
    let (got, docx) = round_trip(body);
    assert!(got.contains(body), "{got}");
    let document = part(&docx, "word/document.xml");
    assert_eq!(
        document.matches("w:fldCharType=\"begin\"").count(),
        3,
        "{document}"
    );
    assert_eq!(
        document.matches("w:fldCharType=\"end\"").count(),
        3,
        "{document}"
    );
    let toc = document.find(" TOC ").expect("the TOC code");
    let paragraphs: Vec<_> = document
        .match_indices("<w:p>")
        .chain(document.match_indices("<w:p "))
        .map(|(at, _)| at)
        .collect();
    let last_end = document.rfind("w:fldCharType=\"end\"").unwrap();
    let opened_in = paragraphs.iter().filter(|&&at| at < toc).count();
    let closed_in = paragraphs.iter().filter(|&&at| at < last_end).count();
    assert_eq!(closed_in - opened_in, 2, "{document}");
    // An unclosed code mid-paragraph stays text, as before.
    let (got, _) = round_trip("<!-- p0 -->\nSee {PAGE} here\n\n<!-- p1 -->\nnow{/}\n");
    assert!(
        got.contains("See \\{PAGE} here") || got.contains("See {PAGE} here"),
        "{got}"
    );
}

/// Each `w:ins`/`w:del` of `document` holds as many field begins as ends:
/// no mark parts a tracked link's field.
fn fields_whole_in_marks(document: &str) {
    for tag in ["<w:ins ", "<w:del "] {
        let close = if tag == "<w:ins " {
            "</w:ins>"
        } else {
            "</w:del>"
        };
        for (at, _) in document.match_indices(tag) {
            let mark = &document[at..at + document[at..].find(close).unwrap()];
            assert_eq!(
                mark.matches("fldCharType=\"begin\"").count(),
                mark.matches("fldCharType=\"end\"").count(),
                "{mark}"
            );
        }
    }
}

/// A change written as two marks (`r213+361`) keeps its link's field in
/// one of them; the second mark goes after the field.
#[test]
fn a_change_of_two_marks_keeps_its_link_field_in_one() {
    let line =
        "{--Title: [Ida Greaves](https://example.org/ida) and more words--}{>>r213+361 @SW<<}";
    let (body, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:del ").count(), 2, "{document}");
    fields_whole_in_marks(&document);
    assert!(body.contains(&format!("{line}\n")), "{body}");
}

/// A comment range or bookmark that ends a tracked link stays in the
/// link's mark, which goes on to the field's end.
#[test]
fn a_range_ending_a_tracked_link_stays_in_its_mark() {
    let line = "{--See [the {==site==}{>>c0 @SW: check<<}](https://example.org)--}{>>r7 @SW<<}";
    let (body, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}\n{body}");
    fields_whole_in_marks(&document);
}

/// A tracked link's title and target are field arguments: a quote is
/// `\"` and a backslash `\\` in the code, and both read back as written.
#[test]
fn a_tracked_link_escapes_its_field_arguments() {
    let line = "{++See [the site](https://example.org/a \"say \\\"hi\\\"\") now++}{>>r4 @SW<<}";
    let (body, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
    let document = part(&docx, "word/document.xml");
    assert!(
        document.contains("\\o &quot;say \\&quot;hi\\&quot;&quot;")
            || document.contains("\\o \"say \\\"hi\\\"\""),
        "{document}"
    );
    assert!(body.contains("say"), "{body}");
    let plain = "[the site](https://example.org/a \"say \\\"hi\\\"\")";
    let (plain_body, _) = round_trip(&format!("<!-- p0 -->\n{plain}\n"));
    // The tracked link reads back as the plain one does.
    let shown = plain_body.lines().find(|l| l.contains("the site")).unwrap();
    assert!(body.contains(shown), "{body}\n{plain_body}");
}

/// A bulleted heading whose own text begins with `- ` keeps it: only the
/// list's bullet is taken off, once.
#[test]
fn a_bulleted_heading_whose_text_begins_with_a_hyphen_keeps_it() {
    let (got, docx) = round_trip("<!-- p0 bullet -->\n## - - Economics\n");
    let document = part(&docx, "word/document.xml");
    assert!(document.contains(">- Economics<"), "{document}");
    assert!(got.contains("Economics\n"), "{got}");
    let line = got.lines().find(|l| l.contains("Economics")).unwrap();
    assert!(
        line == "## - - Economics" || line == "## - \\- Economics",
        "{got}"
    );
}

/// A comment whose range ends between two blocks, its note opening the
/// next paragraph, ends before that paragraph: the paragraph is not in the
/// comment (no `in c0`), as Word wrote it. A paragraph that says `in c0`
/// keeps the end inside it, before the note.
#[test]
fn a_range_ending_before_its_notes_paragraph_stays_out_of_it() {
    for (p1, inside) in [("<!-- p1 -->", false), ("<!-- p1 in c0 -->", true)] {
        let body =
            format!("<!-- p0 in c0 -->\n{{==Some text==}}\n\n{p1}\n{{>>c0 @SW: Hi.<<}}After.\n");
        let (got, docx) = round_trip(&body);
        assert!(got.contains(&body), "{got}");
        let document = part(&docx, "word/document.xml");
        let end = document.find("<w:commentRangeEnd").unwrap();
        let second = document.match_indices("<w:p>").nth(1).unwrap().0;
        assert_eq!(end > second, inside, "{document}");
    }
    // Two notes, neither of whose ranges runs into the paragraph their
    // notes open; and a note opening a paragraph another range runs into.
    for body in [
        "<!-- p0 in c0 c1 -->\n{==Some text==}\n\n\
         <!-- p1 -->\n{>>c0 @SW: One.<<}{>>c1 @SW: Two.<<}After.\n",
        "<!-- p0 in c0 c1 -->\n{==Some text==}\n\n\
         <!-- p1 in c1 -->\n{==After.==}{>>c0 @SW: One.<<}{>>c1 @SW: Two.<<}\n",
    ] {
        let (got, docx) = round_trip(body);
        assert!(got.contains(body), "{got}");
        let document = part(&docx, "word/document.xml");
        let second = document.match_indices("<w:p>").nth(1).unwrap().0;
        let end = document.find("<w:commentRangeEnd w:id=\"0\"/>").unwrap();
        assert!(end < second, "{document}");
    }
    // The paragraph keeps its own mark when the range end goes before it.
    let body = "<!-- p0 in c0 -->\n{==Some text==}\n\n\
                <!-- p1 break-ins r6 @SW -->\n{>>c0 @SW: Hi.<<}{++After.++}{>>r5 @SW<<}\n\n\
                <!-- p2 -->\nEnd.\n";
    let (got, _) = round_trip(body);
    assert!(got.contains(body), "{got}");
    // Through a table, as in ids-table-changes: the table says `in c0`.
    let body = "<!-- p0 in c0 -->\n{==Some text==}\n\n\
                <!-- t0 1x1, cells p1-p1 by row, in c0 -->\n|{==a==}|\n|-|\n\n\
                <!-- p2 -->\n{>>c0 @SW: Hi.<<}{>>c1 @SW re c0: Yes.<<}\n";
    let (got, _) = round_trip(body);
    assert!(got.contains(body), "{got}");
}

/// A comment note inside a tracked change (a reply at the start of an
/// insertion, docx_lots_of_comments r125) keeps its reference in the
/// change's mark, and the change keeps its text.
#[test]
fn a_note_inside_a_change_stays_in_its_mark() {
    // A note numbered apart from its place (c7, the first comment) too.
    let (got, _) = round_trip("<!-- p0 -->\nx {++{>>c7 @SW: New.<<}cap++}{>>r5 @SW<<}\n");
    assert!(
        got.contains("x {++{>>c7 @SW: New.<<}cap++}{>>r5 @SW<<}\n"),
        "{got}"
    );
    let line =
        "{==Plan==}{>>c0 @SW: Root.<<} and {++{>>c1 @SW re c0: Reply.<<}more text++}{>>r4 @SW<<}";
    let (got, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
    assert!(got.contains(&format!("{line}\n")), "{got}");
    let document = part(&docx, "word/document.xml");
    let ins = document.find("<w:ins ").unwrap();
    let reference = document.rfind("<w:commentReference").unwrap();
    assert!(reference > ins, "{document}");
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
}

/// A change holding a comment whose text contains `~>`, `++}`, `--}`,
/// `~~}`, or `==}` reaches a fixed point: the change and the comment
/// survive the first DOCX, and a second md round trip keeps them.
#[test]
fn a_comment_with_a_closer_or_separator_inside_a_change_reaches_a_fixed_point() {
    let payloads = ["~>", "++}", "--}", "~~}", "==}"];
    let shapes = [
        ("ins", "{++a{{>>{p}<<}}b++}"),
        ("del", "{--a{{>>{p}<<}}b--}"),
        ("sub-old", "{~~a{{>>{p}<<}} b~>c~~}"),
        ("sub-new", "{~~a~>b{{>>{p}<<}} c~~}"),
    ];
    for payload in &payloads {
        for (name, template) in &shapes {
            let raw = template.replace("{p}", payload);
            let (md1, docx1) = round_trip(&format!("<!-- p0 -->\n{raw}\n"));
            let (md2, _) = round_trip(&format!("<!-- p0 -->\n{md1}"));
            // The change line (the only non-empty paragraph) survives a
            // second round trip unchanged. The id-line names may shift
            // between md1 and md2 because paragraph mark ids depend on
            // the first read-back, so compare the change lines only.
            let change_line = |md: &str| -> String {
                md.lines()
                    .find(|line| line.contains('{') && line.contains('}'))
                    .unwrap_or_default()
                    .to_string()
            };
            assert_eq!(
                change_line(&md1),
                change_line(&md2),
                "{name} {payload:?}: not a fixed point\n{md1}\n{md2}",
            );
            let document = part(&docx1, "word/document.xml");
            match *name {
                "ins" => assert!(
                    document.matches("<w:ins ").count() >= 1,
                    "{name} {payload:?}: docx1 has ins in {document}"
                ),
                "del" => assert!(
                    document.matches("<w:del ").count() >= 1,
                    "{name} {payload:?}: docx1 has del in {document}"
                ),
                "sub-old" | "sub-new" => {
                    assert!(
                        document.matches("<w:ins ").count() >= 1,
                        "{name} {payload:?}: docx1 has ins in {document}"
                    );
                    assert!(
                        document.matches("<w:del ").count() >= 1,
                        "{name} {payload:?}: docx1 has del in {document}"
                    );
                }
                _ => unreachable!(),
            }
            assert_eq!(
                part(&docx1, "word/comments.xml")
                    .matches("<w:comment ")
                    .count(),
                1,
                "{name} {payload:?}: one comment in document",
            );
        }
    }
}

/// Bold or italic text whose delimiters CommonMark would not read (a
/// bold `*` right after `0.081`, an italic letter inside a word) prints as
/// `<b>`/`<i>`, which reads back the same; `**`/`_` stay wherever they
/// read.
#[test]
fn emphasis_prints_as_a_tag_where_its_delimiters_would_not_read() {
    for line in [
        "{++0.081<b>\\*</b>++}{>>r1 @SW<<}",
        "word<b>.</b> and x<b>,</b>y",
        "in<i>side</i>out and **bold** and _it_.",
        // `**` beside `_` is beside punctuation: it needs space or
        // punctuation outside.
        "in<b><i>side</i></b>out and a <b><i>b</i></b>c and **_both_** here.",
    ] {
        let (got, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
        assert!(got.contains(&format!("{line}\n")), "{line}\n{got}");
        let document = part(&docx, "word/document.xml");
        assert!(
            document.contains("<w:b/>") || document.contains("<w:i/>"),
            "{document}"
        );
    }
}

/// A raised link Word splits into runs (`[\[2\]]`, the escapes parting
/// its text) stays one link inside one `<sup>`.
#[test]
fn a_raised_link_of_several_runs_stays_one_link() {
    for line in [
        "x <sup>[\\[2\\]](#footnote-2)</sup>",
        "x <sup>[a\\]](#f)</sup> and <sub>2</sub><sup>3</sup>",
    ] {
        let (got, _) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
        assert!(got.contains(&format!("{line}\n")), "{line}\n{got}");
    }
}

/// A comment inside a change in a footnote moves to the note's reference
/// mark without the change: the body has no insertion of its own.
#[test]
fn a_comment_moved_out_of_a_note_leaves_its_change_there() {
    let (_, docx) = round_trip(
        "<!-- p0 -->\nText[^1]\n\n[^1]: {++{==note text==}{>>c0 @SW: hi.<<}++}{>>r1 @SW<<}\n",
    );
    let document = part(&docx, "word/document.xml");
    assert!(document.contains("<w:commentReference"), "{document}");
    assert!(!document.contains("<w:ins "), "{document}");
    let notes = part(&docx, "word/footnotes.xml");
    assert!(notes.contains("<w:ins "), "{notes}");
}

/// An item an id line parts from the list before it nests by its
/// indentation, as a hand-written view nests it (`   - X` under `2. Y`);
/// a level the id line names wins. Only three spaces can say it: after an
/// id line, four are an indented code block.
#[test]
fn an_indented_item_after_an_id_line_nests_under_the_item_before() {
    let body = "<!-- p0 ListParagraph num \"1.\" -->\n1. One.\n\n\
                <!-- p1 ListParagraph bullet -->\n   - Sub.\n\n\
                <!-- p2 ListParagraph bullet -->\n   - Sub two.\n\n\
                <!-- p3 ListParagraph bullet, level 3 -->\n   - Named.\n\n\
                <!-- p4 ListParagraph bullet -->\n- Out.\n";
    let (got, docx) = round_trip(body);
    for line in [
        "   <!-- p1 ListParagraph bullet -->\n   - Sub.\n",
        "   <!-- p2 ListParagraph bullet -->\n   - Sub two.\n",
        "     <!-- p3 ListParagraph bullet, level 3 -->\n     - Named.\n",
        "\n<!-- p4 ListParagraph bullet -->\n- Out.\n",
    ] {
        assert!(got.contains(line), "{line}\n{got}");
    }
    let document = part(&docx, "word/document.xml");
    let levels: Vec<&str> = document
        .match_indices("<w:ilvl w:val=\"")
        .map(|(at, found)| &document[at + found.len()..at + found.len() + 1])
        .collect();
    assert_eq!(levels, ["0", "1", "1", "3", "0"], "{document}");
    // A list's own nesting is untouched.
    let (got, _) = round_trip("1. One\n2. Two\n   - Sub\n3. Three\n");
    assert!(
        got.contains("\n   <!-- p2 ListParagraph bullet -->\n   - Sub\n"),
        "{got}"
    );
    assert!(got.contains("num \"3.\" -->\n3. Three\n"), "{got}");
}

/// The view nests a list item under the one before by indentation, its id
/// line with it, so the Markdown is a nested list at any depth. A level the
/// indentation cannot say (a skipped one, or a list that starts deep) stays
/// on the id line, and a skipped level adds no indentation: four columns
/// past an item's content would start a code block.
#[test]
fn the_view_indents_a_nested_item_and_its_id_line() {
    for (body, deepest) in [
        (
            "<!-- p0 ListParagraph num \"1.\" -->\n1. One.\n\n\
             \x20  <!-- p1 ListParagraph bullet -->\n   - Sub.\n\n\
             \x20    <!-- p2 ListParagraph bullet -->\n     - Deeper.\n\n\
             <!-- p3 ListParagraph num \"2.\" -->\n2. Two.\n",
            "2",
        ),
        (
            "<!-- p0 ListParagraph bullet -->\n- Top.\n\n\
             \x20 <!-- p1 ListParagraph bullet, level 3 -->\n  - Two skipped.\n",
            "3",
        ),
        (
            "<!-- p0 ListParagraph bullet, level 2 -->\n- Starts deep.\n",
            "2",
        ),
    ] {
        let (got, docx) = round_trip(body);
        assert!(got.contains(body), "{body}\n{got}");
        assert!(!got.contains("level 1"), "{got}");
        let document = part(&docx, "word/document.xml");
        let levels: Vec<&str> = document
            .match_indices("<w:ilvl w:val=\"")
            .map(|(at, found)| &document[at + found.len()..at + found.len() + 1])
            .collect();
        assert_eq!(levels.iter().max(), Some(&deepest), "{document}");
    }
}

const PNG_1X1: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0x0D, b'I', b'H', b'D', b'R', 0, 0, 0,
    1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1F, 0x15, 0xC4, 0x89, 0, 0, 0, 0x0A, b'I', b'D', b'A', b'T',
    0x78, 0x9C, 0x63, 0, 1, 0, 0, 5, 0, 1, 0x0D, 0x0A, 0x2D, 0xB4, 0, 0, 0, 0, b'I', b'E', b'N',
    b'D', 0xAE, 0x42, 0x60, 0x82,
];

/// `docx` bytes read to a view, the view written to Word again: the view,
/// and the second document.
fn docx_round_trip(docx: &[u8]) -> (String, Vec<u8>) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), docx).unwrap();
    let view = jubarte(dir.path(), &["read", "in.docx", "--no-page-markers"]);
    std::fs::write(dir.path().join("view.md"), &view).unwrap();
    jubarte(dir.path(), &["convert", "view.md", "-o", "out.docx"]);
    let out = std::fs::read(dir.path().join("out.docx")).unwrap();
    (view, out)
}

/// A floating image — a `wp:anchor` drawing — is not a plain inline raster:
/// the view carries it whole as a `drawing:` payload, and writing the view
/// back puts the anchor — its wrap, position, extent and `docPr` — into
/// `document.xml` again, with a live image relationship.
#[test]
fn a_floating_image_keeps_its_anchor() {
    let anchor = r#"<w:p><w:r><w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" distT="0" distB="0" distL="114300" distR="114300" simplePos="0" relativeHeight="251658240" behindDoc="0" locked="0" layoutInCell="1" allowOverlap="1"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="margin"><wp:posOffset>1828800</wp:posOffset></wp:positionH><wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV><wp:extent cx="914400" cy="914400"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="7" name="Picture 7" descr="logo"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="logo.png"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rIdX0"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p>"#;
    let image_rel = format!("{R_NS}/image");
    let doc = replace_entry(
        &docx_with(
            &(para("Before.") + anchor),
            &[Part {
                name: "word/media/logo.png",
                content_type: "image/png",
                rel_type: &image_rel,
                xml: "",
            }],
        ),
        "word/media/logo.png",
        PNG_1X1,
    );
    let (view, out) = docx_round_trip(&doc);
    assert!(view.contains("](drawing:"), "{view}");
    let document = part_string(&out, "word/document.xml").expect("document.xml");
    for want in [
        "<wp:anchor",
        "<wp:wrapNone/>",
        "<wp:posOffset>1828800</wp:posOffset>",
        "name=\"Picture 7\"",
        "<pic:pic",
    ] {
        assert!(document.contains(want), "{want}: {document}");
    }
    let rels = part_string(&out, "word/_rels/document.xml.rels").expect("document rels");
    assert!(rels.contains("media/"), "{rels}");
}
