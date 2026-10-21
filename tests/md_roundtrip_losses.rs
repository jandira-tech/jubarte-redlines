// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! What the Markdown round trip lost (tests/md_fixed_point.rs, run of
//! 2026-10-14), one shape each: the view written to Word and read back
//! prints the same line.

use std::path::Path;
use std::process::Command;

mod common;

use common::docx::{Part, R_NS, W_NS, docx_with, para, part_string, replace_entry};

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
    let line = "{--Title: [Ida Greaves](https://eur.on.worldcat.org/oclc/1381732921)--}{r213 @SW}";
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
    let inserted = "{++See [the site](https://example.com) now++}{r4 @SW}";
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
    let body = "<!-- p0 bullet -->\n## - {--Economics & business--}{r211 @SW}\n";
    let (got, docx) = round_trip(body);
    assert!(
        got.contains("## - {--Economics & business--}{r211 @SW}\n"),
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

/// Distinct deletion ids stay separate marks and keep the link's field
/// whole; the second mark goes after the field, and both pieces read back
/// as the one change they are.
#[test]
fn a_change_of_two_marks_keeps_its_link_field_in_one() {
    let line = "{--Title: [Ida Greaves](https://example.org/ida) and more words--}{r213+361 @SW}";
    let (body, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:del ").count(), 2, "{document}");
    fields_whole_in_marks(&document);
    assert!(
        body.contains("{--Title: [Ida Greaves](https://example.org/ida)--}{r213+361 @SW}{-- and more words--}{r213+361 @SW}\n"),
        "{body}"
    );
}

/// A comment range or bookmark that ends a tracked link stays in the
/// link's mark, which goes on to the field's end.
#[test]
fn a_range_ending_a_tracked_link_stays_in_its_mark() {
    let line = "{--See [the {==site==}{>>c0 @SW: check<<}](https://example.org)--}{r7 @SW}";
    let (body, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}\n{body}");
    fields_whole_in_marks(&document);
}

/// A tracked link's title and target are field arguments: a quote is
/// `\"` and a backslash `\\` in the code, and both read back as written.
#[test]
fn a_tracked_link_escapes_its_field_arguments() {
    let line = "{++See [the site](https://example.org/a \"say \\\"hi\\\"\") now++}{r4 @SW}";
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
                <!-- p1 -->\n{>>c0 @SW: Hi.<<}{++After.++}{r5+6 @SW}¶{r5+6 @SW}\n\n\
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
    let (got, _) = round_trip("<!-- p0 -->\nx {++{>>c7 @SW: New.<<}cap++}{r5 @SW}\n");
    assert!(
        got.contains("x {++{>>c7 @SW: New.<<}cap++}{r5 @SW}\n"),
        "{got}"
    );
    let line =
        "{==Plan==}{>>c0 @SW: Root.<<} and {++{>>c1 @SW re c0: Reply.<<}more text++}{r4 @SW}";
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
        "{++0.081<b>\\*</b>++}{r1 @SW}",
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
        "<!-- p0 -->\nText[^1]\n\n[^1]: {++{==note text==}{>>c0 @SW: hi.<<}++}{r1 @SW}\n",
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

/// Word keeps the body's spacing in `docDefaults`, and a table style's own
/// spacing wins over that in its cells. Read and written back, the spacing
/// stays the document's default: on `Normal` it would beat the table
/// style, every cell would take the body's spacing, and the table would
/// spill onto a second page (#1420).
#[test]
fn the_documents_default_spacing_stays_its_default() {
    let thin = "w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>";
    let borders: String = ["top", "left", "bottom", "right", "insideH", "insideV"]
        .iter()
        .map(|side| format!("<w:{side} {thin}"))
        .collect();
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/><w:sz w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="200" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style><w:style w:type="table" w:styleId="TableGrid"><w:name w:val="Table Grid"/><w:basedOn w:val="TableNormal"/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:tblPr><w:tblBorders>{borders}</w:tblBorders></w:tblPr></w:style></w:styles>"#
    );
    let rows: String = (0..34)
        .map(|i| {
            format!(
                "<w:tr><w:tc><w:tcPr><w:tcW w:w=\"9360\" w:type=\"dxa\"/></w:tcPr>{}</w:tc></w:tr>",
                para(&format!("Row {i}"))
            )
        })
        .collect();
    let body = format!(
        "{}<w:tbl><w:tblPr><w:tblStyle w:val=\"TableGrid\"/><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"9360\"/></w:tblGrid>{rows}</w:tbl>{}",
        para("Before."),
        para("After.")
    );
    let styles_rel = format!("{R_NS}/styles");
    let original = docx_with(
        &body,
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: &styles_rel,
            xml: &styles,
        }],
    );
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.docx"), &original).unwrap();
    jubarte(dir.path(), &["read", "a.docx", "-o", "a.md"]);
    jubarte(dir.path(), &["convert", "a.md", "-o", "b.docx"]);
    let written = part(
        &std::fs::read(dir.path().join("b.docx")).unwrap(),
        "word/styles.xml",
    );
    assert!(
        written.contains(
            "<w:pPrDefault><w:pPr><w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>\
             </w:pPr></w:pPrDefault>"
        ),
        "{written}"
    );
    let normal = written
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"Normal\""))
        .unwrap_or_else(|| panic!("no Normal: {written}"));
    assert!(!normal.contains("<w:spacing"), "{normal}");
    let pages = |name: &str| {
        jubarte(
            dir.path(),
            &["debug", name, "--check", "render", "--part", "(layout)"],
        )
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("pages "))
        .map(str::to_string)
    };
    assert_eq!(pages("a.docx").as_deref(), Some("pages 1"));
    assert_eq!(pages("b.docx"), pages("a.docx"));
}

/// A docx with a floating image — a `wp:anchor` drawing with one PNG —
/// as `docx_with` builds it.
fn floating_image_docx() -> Vec<u8> {
    let anchor = r#"<w:p><w:r><w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" distT="0" distB="0" distL="114300" distR="114300" simplePos="0" relativeHeight="251658240" behindDoc="0" locked="0" layoutInCell="1" allowOverlap="1"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="margin"><wp:posOffset>1828800</wp:posOffset></wp:positionH><wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV><wp:extent cx="914400" cy="914400"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="7" name="Picture 7" descr="logo"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="logo.png"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rIdX0"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p>"#;
    let image_rel = format!("{R_NS}/image");
    replace_entry(
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
    )
}

/// A floating image — a `wp:anchor` drawing — is not a plain inline raster:
/// the view carries it whole as a `drawing:` payload, and writing the view
/// back puts the anchor — its wrap, position, extent and `docPr` — into
/// `document.xml` again, with a live image relationship.
#[test]
fn a_floating_image_keeps_its_anchor() {
    let (_view, out) = docx_round_trip(&floating_image_docx());
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

/// A drawing no image form carries — here a VML rectangle — leaves the
/// body text: the paragraph shows only `![descr][d1]`, and the payload
/// sits in a `[d1]: drawing:…` reference definition at the view's end.
/// The written document reads back to the same view.
#[test]
fn a_drawing_payload_is_a_reference_definition() {
    let shape = "<w:p><w:r><w:pict>\
        <v:rect xmlns:v=\"urn:schemas-microsoft-com:vml\" \
        style=\"width:100pt;height:50pt\" fill=\"true\" stroke=\"true\"/>\
        </w:pict></w:r></w:p>";
    let (view, out) = docx_round_trip(&docx_with(&(para("Before.") + shape), &[]));
    let reference = "![][d1]";
    let body = view
        .find(reference)
        .unwrap_or_else(|| panic!("{reference} not in:\n{view}"));
    let definition = view
        .find("[d1]: drawing:")
        .unwrap_or_else(|| panic!("d1 definition not in:\n{view}"));
    assert!(body < definition, "{view}");
    assert!(!view.contains("](drawing:"), "{view}");
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("out.docx"), &out).unwrap();
    let second = jubarte(dir.path(), &["read", "out.docx", "--no-page-markers"]);
    // The lines that name the file differ; everything else is the same view.
    let unnamed = |view: &str| {
        view.lines()
            .filter(|l| !l.starts_with("source: ") && !l.starts_with("sha256: "))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(unnamed(&view), unnamed(&second));
}

/// A text box's words are document text, not payload (#1101 A): the
/// view prints them as the paragraphs after the anchor, and the payload
/// holds the box's frame only. A docx whose middle paragraph anchors a
/// plain text box reading `Added box`.
fn text_box_docx() -> Vec<u8> {
    text_box_docx_with("", &[])
}

/// [`text_box_docx`] whose box paragraph has the properties `ppr` (a
/// `w:pPr`, or nothing), with the extra parts `extras`.
fn text_box_docx_with(ppr: &str, extras: &[Part<'_>]) -> Vec<u8> {
    let box_run = "<w:r><w:drawing><wp:anchor \
        xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
        distT=\"0\" distB=\"0\" distL=\"114300\" distR=\"114300\" simplePos=\"0\" \
        relativeHeight=\"251658240\" behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" \
        allowOverlap=\"1\"><wp:simplePos x=\"0\" y=\"0\"/>\
        <wp:positionH relativeFrom=\"column\"><wp:posOffset>0</wp:posOffset>\
        </wp:positionH><wp:positionV relativeFrom=\"paragraph\">\
        <wp:posOffset>0</wp:posOffset></wp:positionV>\
        <wp:extent cx=\"2374265\" cy=\"457200\"/><wp:effectExtent l=\"0\" t=\"0\" \
        r=\"0\" b=\"0\"/><wp:wrapSquare wrapText=\"bothSides\"/>\
        <wp:docPr id=\"7\" name=\"Text Box 7\"/><wp:cNvGraphicFramePr/>\
        <a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
        <a:graphicData \
        uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
        <wps:wsp \
        xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
        <wps:cNvSpPr txBox=\"1\"/><wps:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/>\
        <a:ext cx=\"2374265\" cy=\"457200\"/></a:xfrm>\
        <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></wps:spPr>\
        <wps:txbx><w:txbxContent>\
        <w:p>{ppr}<w:r><w:t xml:space=\"preserve\">Added box</w:t></w:r></w:p>\
        </w:txbxContent></wps:txbx><wps:bodyPr rot=\"0\" vert=\"horz\" \
        wrap=\"square\" lIns=\"91440\" tIns=\"45720\" rIns=\"91440\" bIns=\"45720\" \
        anchor=\"t\"/></wps:wsp></a:graphicData></a:graphic></wp:anchor>\
        </w:drawing></w:r>"
        .replace("{ppr}", ppr);
    docx_with(
        &(para("Before.") + "<w:p>" + &box_run + "</w:p>" + &para("After.")),
        extras,
    )
}

/// The view of a text box holds its words as the paragraphs after the
/// anchor, never inside an HTML comment: a reader sees them and an edit
/// has somewhere to land (#1101 A).
#[test]
fn a_text_boxs_words_print_as_paragraphs_not_comments() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), text_box_docx()).unwrap();
    let view = jubarte(dir.path(), &["read", "in.docx", "--no-page-markers"]);
    let body = view.splitn(3, "---\n").nth(2).unwrap_or_default();
    assert!(body.contains("\nAdded box\n"), "{body}");
    assert!(!body.contains("<!-- Added box"), "{body}");
}

/// Editing a text box's words in the view writes the new words into the
/// box (`w:txbxContent`) of the document written back; the old words are
/// gone (#1101 A).
#[test]
fn editing_a_text_boxs_words_writes_them_into_the_box() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), text_box_docx()).unwrap();
    let view = jubarte(dir.path(), &["read", "in.docx", "--no-page-markers"]);
    let edited = view.replace("Added box", "Edited words");
    assert_ne!(edited, view);
    std::fs::write(dir.path().join("edit.md"), edited).unwrap();
    jubarte(dir.path(), &["convert", "edit.md", "-o", "out.docx"]);
    let document = std::fs::read(dir.path().join("out.docx").canonicalize().unwrap()).unwrap();
    let document = part(&document, "word/document.xml");
    let box_at = document
        .find("txbxContent")
        .unwrap_or_else(|| panic!("no text box:\n{document}"));
    let box_xml = document[box_at - 3..]
        .split("</w:txbxContent>")
        .next()
        .unwrap();
    assert!(box_xml.contains("Edited words"), "{document}");
    assert!(!document.contains("Added box"), "{document}");
}

/// An unedited round trip restores the original drawing: the payload's
/// frame plus the words the view printed give the box back, and the
/// second view is the first again (#1101 A).
#[test]
fn an_unedited_text_box_round_trips_to_the_same_drawing() {
    let (view, out) = docx_round_trip(&text_box_docx());
    let document = part(&out, "word/document.xml");
    let original = part(&text_box_docx(), "word/document.xml");
    assert_eq!(
        xml_element(&document, "wp:anchor"),
        xml_element(&original, "wp:anchor").replace("id=\"7\"", "id=\"1\"")
    );
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("out.docx"), &out).unwrap();
    let second = jubarte(dir.path(), &["read", "out.docx", "--no-page-markers"]);
    let unnamed = |view: &str| {
        view.lines()
            .filter(|l| !l.starts_with("source: ") && !l.starts_with("sha256: "))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(unnamed(&view), unnamed(&second));
}

fn xml_element<'a>(xml: &'a str, name: &str) -> &'a str {
    let start = xml.find(&format!("<{name}")).unwrap();
    let close = format!("</{name}>");
    let end = start + xml[start..].find(&close).unwrap() + close.len();
    &xml[start..end]
}

#[test]
fn plain_markdown_edits_stay_inside_the_text_box() {
    use jubarte::markdown::{DocxOptions, MarkdownOptions, docx_to_markdown, markdown_to_docx};
    let view = docx_to_markdown(&text_box_docx(), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert!(view.contains("\nAdded box\n"), "{view}");
    let edited = view.replace("Added box", "Edited **box**\n\nSecond paragraph");
    let out = markdown_to_docx(&edited, &DocxOptions::default())
        .unwrap()
        .docx;
    let xml = part(&out, "word/document.xml");
    assert!(
        xml.contains("<w:txbxContent>"),
        "box content is empty: {xml}"
    );
    let boxed = xml_element(&xml, "w:txbxContent");
    assert!(boxed.contains("Edited "), "{xml}");
    assert!(boxed.contains("Second paragraph"), "{xml}");
    assert!(!boxed.contains("After."), "{xml}");
    assert_eq!(xml.matches("Second paragraph").count(), 1, "{xml}");
    assert!(xml.contains("Before.") && xml.contains("After."), "{xml}");
}

#[test]
fn an_unedited_box_keeps_unprinted_run_properties_exactly() {
    let original = text_box_docx();
    let xml = part(&original, "word/document.xml").replace(
        "<w:t xml:space=\"preserve\">Added box",
        "<w:rPr><w:lang w:val=\"pt-BR\"/></w:rPr><w:t xml:space=\"preserve\">Added box",
    );
    let original = replace_entry(&original, "word/document.xml", xml.as_bytes());
    let (_, out) = docx_round_trip(&original);
    let actual = part(&out, "word/document.xml");
    assert_eq!(
        xml_element(&actual, "wp:anchor"),
        xml_element(&xml, "wp:anchor").replace("id=\"7\"", "id=\"1\"")
    );
}

#[test]
fn a_box_with_its_own_change_remains_editable() {
    let original = text_box_docx();
    let xml = part(&original, "word/document.xml").replace(
        "<w:p><w:r><w:t xml:space=\"preserve\">Added box</w:t></w:r></w:p>",
        "<w:p><w:ins w:id=\"90\" w:author=\"A\" w:date=\"2026-01-01T00:00:00Z\"><w:r><w:t>Added box</w:t></w:r></w:ins></w:p>",
    );
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("in.docx"),
        replace_entry(&original, "word/document.xml", xml.as_bytes()),
    )
    .unwrap();
    let view = jubarte(dir.path(), &["read", "in.docx", "--no-page-markers"]);
    assert!(view.contains("Added box"), "{view}");
    std::fs::write(
        dir.path().join("edit.md"),
        view.replace("Added box", "Edited words"),
    )
    .unwrap();
    jubarte(dir.path(), &["convert", "edit.md", "-o", "out.docx"]);
    let out = std::fs::read(dir.path().join("out.docx")).unwrap();
    let xml = part(&out, "word/document.xml");
    let boxed = xml_element(&xml, "w:txbxContent");
    assert!(boxed.contains("Edited words"), "{xml}");
    assert!(!xml.contains("Added box"), "{xml}");
}

#[test]
fn two_boxes_at_one_anchor_keep_separate_stories() {
    use jubarte::markdown::{DocxOptions, MarkdownOptions, docx_to_markdown, markdown_to_docx};
    let input = text_box_docx();
    let xml = part(&input, "word/document.xml");
    let drawing = xml_element(&xml, "w:drawing");
    let xml = xml.replace(
        drawing,
        &format!(
            "{drawing}</w:r><w:r>{}",
            drawing
                .replace("Added box", "Other box")
                .replace("id=\"7\"", "id=\"8\"")
        ),
    );
    let input = replace_entry(&input, "word/document.xml", xml.as_bytes());
    let view = docx_to_markdown(&input, &MarkdownOptions::default())
        .unwrap()
        .markdown;
    let out = markdown_to_docx(
        &view.replace("Other box", "Second edited"),
        &DocxOptions::default(),
    )
    .unwrap()
    .docx;
    let xml = part(&out, "word/document.xml");
    let first = xml_element(&xml, "w:txbxContent");
    assert!(
        first.contains("Added box") && !first.contains("Second edited"),
        "{xml}"
    );
    let rest = &xml[xml.find(first).unwrap() + first.len()..];
    let second = xml_element(rest, "w:txbxContent");
    assert!(
        second.contains("Second edited") && !second.contains("Added box"),
        "{xml}"
    );
    assert_eq!(xml.matches("Second edited").count(), 1, "{xml}");
}

#[test]
fn deleting_all_box_words_leaves_a_valid_empty_paragraph() {
    use jubarte::markdown::{DocxOptions, MarkdownOptions, docx_to_markdown, markdown_to_docx};
    let view = docx_to_markdown(&text_box_docx(), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    let out = markdown_to_docx(&view.replace("Added box", ""), &DocxOptions::default())
        .unwrap()
        .docx;
    let xml = part(&out, "word/document.xml");
    assert!(
        xml.contains("<w:txbxContent><w:p/></w:txbxContent>"),
        "{xml}"
    );
    assert!(!xml.contains("Added box"), "{xml}");
}

#[test]
fn a_box_in_a_table_cell_keeps_edited_words_in_the_box() {
    use jubarte::markdown::{DocxOptions, MarkdownOptions, docx_to_markdown, markdown_to_docx};
    let input = text_box_docx();
    let xml = part(&input, "word/document.xml");
    let drawing = xml_element(&xml, "w:drawing");
    let body = format!(
        "<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t>{drawing}</w:r></w:p></w:tc></w:tr></w:tbl>"
    );
    let input = docx_with(&body, &[]);
    let view = docx_to_markdown(&input, &MarkdownOptions::default())
        .unwrap()
        .markdown;
    let out = markdown_to_docx(
        &view.replace("Added box", "Cell box edited"),
        &DocxOptions::default(),
    )
    .unwrap()
    .docx;
    let xml = part(&out, "word/document.xml");
    assert!(
        xml_element(&xml, "w:txbxContent").contains("Cell box edited"),
        "{view}\n{xml}"
    );
    assert_eq!(xml.matches("Cell box edited").count(), 1, "{xml}");
}

#[test]
fn a_box_payload_carries_no_document_words() {
    use base64::Engine as _;
    use jubarte::markdown::{MarkdownOptions, docx_to_markdown};
    let view = docx_to_markdown(&text_box_docx(), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    let payload = view
        .lines()
        .find_map(|line| line.strip_prefix("[d1]: drawing:"))
        .unwrap();
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
    assert!(!json["xml"].as_str().unwrap().contains("Added box"));
    assert_eq!(view.matches("Added box").count(), 1, "{view}");
}

/// A raster drawing — here a floating anchor — is image bytes to the
/// readable layer: `![descr][imageN]` backed by a `data:` definition and a
/// `drawing:` comment that keeps the anchor XML. Written back, the
/// package holds the anchor and the PNG again.
#[test]
fn a_floating_raster_is_a_backed_data_uri() {
    let (view, out) = docx_round_trip(&floating_image_docx());
    let reference = "![logo][image1]";
    let body = view
        .find(reference)
        .unwrap_or_else(|| panic!("{reference} not in:\n{view}"));
    let definition = view
        .find("[image1]: <data:image/png;base64,")
        .unwrap_or_else(|| panic!("image1 definition not in:\n{view}"));
    let backing = view
        .find("<!-- image1 drawing:")
        .unwrap_or_else(|| panic!("image1 backing not in:\n{view}"));
    assert!(body < definition && definition < backing, "{view}");
    assert!(!view.contains("[d1]"), "{view}");
    let document = part(&out, "word/document.xml");
    assert!(document.contains("<wp:anchor"), "{document}");
    let rels = part(&out, "word/_rels/document.xml.rels");
    assert!(rels.contains("media/logo"), "{rels}");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
    let mut media = Vec::new();
    std::io::Read::read_to_end(
        &mut zip.by_name("word/media/logo.png").expect("logo.png"),
        &mut media,
    )
    .unwrap();
    assert_eq!(media, PNG_1X1);
}

const SVG_1X1: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"><rect width="1" height="1"/></svg>"#;

/// A docx whose one paragraph is an `asvg:svgBlip` drawing: the SVG part
/// and the PNG fallback Word stores beside it.
fn svg_blip_docx() -> Vec<u8> {
    let drawing = "<w:p><w:r><w:drawing><wp:inline \
        xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
        distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
        <wp:extent cx=\"914400\" cy=\"914400\"/>\
        <wp:docPr id=\"7\" name=\"Icon 1\" descr=\"icon\"/>\
        <a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
        <a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
        <pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
        <pic:nvPicPr><pic:cNvPr id=\"0\" name=\"icon.svg\"/><pic:cNvPicPr/></pic:nvPicPr>\
        <pic:blipFill><a:blip r:embed=\"rIdX1\"><a:extLst>\
        <a:ext uri=\"{96DAC541-7B7A-43D3-8B79-37D633B846F1}\">\
        <asvg:svgBlip \
        xmlns:asvg=\"http://schemas.microsoft.com/office/drawing/2016/SVG/main\" \
        r:embed=\"rIdX0\"/></a:ext></a:extLst></a:blip>\
        <a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
        <pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"914400\" cy=\"914440\"/></a:xfrm>\
        <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic>\
        </a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>";
    let image_rel = format!("{R_NS}/image");
    replace_entry(
        &docx_with(
            &(para("Before.") + drawing),
            &[
                Part {
                    name: "word/media/icon.svg",
                    content_type: "image/svg+xml",
                    rel_type: &image_rel,
                    xml: SVG_1X1,
                },
                Part {
                    name: "word/media/icon.png",
                    content_type: "image/png",
                    rel_type: &image_rel,
                    xml: "",
                },
            ],
        ),
        "word/media/icon.png",
        PNG_1X1,
    )
}

/// An `asvg:svgBlip` the paragraph holds alone reads as its code: a
/// ` ```svg ` fence in the body, the drawing's payload — its PNG fallback
/// with it — in a backing comment at the view's end. Written back, the
/// package holds the blip, the SVG and the fallback again.
#[test]
fn a_standalone_svg_blip_is_a_fenced_source() {
    let (view, out) = docx_round_trip(&svg_blip_docx());
    let fence = view
        .find("``` svg d1 icon\n")
        .unwrap_or_else(|| panic!("svg fence not in:\n{view}"));
    let source = view
        .find(SVG_1X1)
        .unwrap_or_else(|| panic!("svg source not in:\n{view}"));
    let backing = view
        .find("<!-- d1 drawing:")
        .unwrap_or_else(|| panic!("d1 backing not in:\n{view}"));
    assert!(fence < source && source < backing, "{view}");
    assert!(!view.contains("![icon][d1]"), "{view}");
    let document = part(&out, "word/document.xml");
    assert!(document.contains("svgBlip"), "{document}");
    assert_eq!(
        part_string(&out, "word/media/icon.svg").as_deref(),
        Some(SVG_1X1)
    );
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
    let mut png = Vec::new();
    std::io::Read::read_to_end(
        &mut zip.by_name("word/media/icon.png").expect("icon.png"),
        &mut png,
    )
    .unwrap();
    assert_eq!(png, PNG_1X1);
}

/// A bar chart part with two series over four categories.
const BAR_CHART: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
    <c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\">\
    <c:chart><c:title><c:tx><c:rich><a:p xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
    <a:r><a:t>Quarterly Revenue</a:t></a:r></a:p></c:rich></c:tx></c:title>\
    <c:plotArea><c:layout/><c:barChart><c:ser>\
    <c:tx><c:strRef><c:strCache><c:ptCount val=\"1\"/><c:pt idx=\"0\"><c:v>Revenue</c:v></c:pt></c:strCache></c:strRef></c:tx>\
    <c:cat><c:strRef><c:strCache><c:ptCount val=\"4\"/>\
    <c:pt idx=\"0\"><c:v>Q1</c:v></c:pt><c:pt idx=\"1\"><c:v>Q2</c:v></c:pt>\
    <c:pt idx=\"2\"><c:v>Q3</c:v></c:pt><c:pt idx=\"3\"><c:v>Q4</c:v></c:pt>\
    </c:strCache></c:strRef></c:cat>\
    <c:val><c:numRef><c:numCache><c:ptCount val=\"4\"/>\
    <c:pt idx=\"0\"><c:v>12</c:v></c:pt><c:pt idx=\"1\"><c:v>18</c:v></c:pt>\
    <c:pt idx=\"2\"><c:v>15</c:v></c:pt><c:pt idx=\"3\"><c:v>22</c:v></c:pt>\
    </c:numCache></c:numRef></c:val></c:ser><c:ser>\
    <c:tx><c:strRef><c:strCache><c:ptCount val=\"1\"/><c:pt idx=\"0\"><c:v>Costs</c:v></c:pt></c:strCache></c:strRef></c:tx>\
    <c:cat><c:strRef><c:strCache><c:ptCount val=\"4\"/>\
    <c:pt idx=\"0\"><c:v>Q1</c:v></c:pt><c:pt idx=\"1\"><c:v>Q2</c:v></c:pt>\
    <c:pt idx=\"2\"><c:v>Q3</c:v></c:pt><c:pt idx=\"3\"><c:v>Q4</c:v></c:pt>\
    </c:strCache></c:strRef></c:cat>\
    <c:val><c:numRef><c:numCache><c:ptCount val=\"4\"/>\
    <c:pt idx=\"0\"><c:v>8</c:v></c:pt><c:pt idx=\"1\"><c:v>11</c:v></c:pt>\
    <c:pt idx=\"2\"><c:v>9</c:v></c:pt><c:pt idx=\"3\"><c:v>13</c:v></c:pt>\
    </c:numCache></c:numRef></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>";

/// A pie chart part: one series, three slices.
const PIE_CHART: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
    <c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\">\
    <c:chart><c:title><c:tx><c:rich><a:p xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
    <a:r><a:t>Market Share</a:t></a:r></a:p></c:rich></c:tx></c:title>\
    <c:plotArea><c:layout/><c:pieChart><c:ser>\
    <c:cat><c:strRef><c:strCache><c:ptCount val=\"3\"/>\
    <c:pt idx=\"0\"><c:v>Alpha</c:v></c:pt><c:pt idx=\"1\"><c:v>Beta</c:v></c:pt>\
    <c:pt idx=\"2\"><c:v>Gamma</c:v></c:pt></c:strCache></c:strRef></c:cat>\
    <c:val><c:numRef><c:numCache><c:ptCount val=\"3\"/>\
    <c:pt idx=\"0\"><c:v>45</c:v></c:pt><c:pt idx=\"1\"><c:v>30</c:v></c:pt>\
    <c:pt idx=\"2\"><c:v>25</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser>\
    </c:pieChart></c:plotArea></c:chart></c:chartSpace>";

/// A `w:drawing` holding an inline chart reference to `rid`.
fn chart_drawing(rid: &str) -> String {
    format!(
        "<w:p><w:r><w:drawing><wp:inline \
        xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
        distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
        <wp:extent cx=\"4572000\" cy=\"2743200\"/><wp:docPr id=\"0\" name=\"\"/>\
        <a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
        <a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\">\
        <c:chart xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" \
        xmlns:r=\"{R_NS}\" r:id=\"{rid}\"/>\
        </a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"
    )
}

/// A docx whose body is a bar chart and a pie chart, each a paragraph's
/// only drawing, `word/charts/chart{1,2}.xml` the parts.
fn charts_docx() -> Vec<u8> {
    let chart_type = "application/vnd.openxmlformats-officedocument.drawingml.chart+xml";
    let chart_rel = format!("{R_NS}/chart");
    docx_with(
        &(para("Before.") + &chart_drawing("rIdX0") + &chart_drawing("rIdX1")),
        &[
            Part {
                name: "word/charts/chart1.xml",
                content_type: chart_type,
                rel_type: &chart_rel,
                xml: BAR_CHART,
            },
            Part {
                name: "word/charts/chart2.xml",
                content_type: chart_type,
                rel_type: &chart_rel,
                xml: PIE_CHART,
            },
        ],
    )
}

/// A chart a paragraph holds alone prints its Mermaid — `xychart-beta`
/// for a bar chart, `pie` for a pie — under ` ```mermaid ` fences the
/// `<!-- dN drawing:… -->` backings keep whole. An unchanged fence writes
/// the chart part back byte for byte.
#[test]
fn a_standalone_chart_prints_mermaid_and_round_trips_exactly() {
    let (view, out) = docx_round_trip(&charts_docx());
    for want in [
        "``` mermaid d1\nxychart-beta\n",
        "title \"Quarterly Revenue\"",
        "x-axis [\"Q1\", \"Q2\", \"Q3\", \"Q4\"]",
        "bar [12, 18, 15, 22]",
        "bar [8, 11, 9, 13]",
        "``` mermaid d2\npie title Market Share\n",
        "\"Alpha\" : 45",
    ] {
        assert!(view.contains(want), "{want} not in:\n{view}");
    }
    for backing in ["<!-- d1 drawing:", "<!-- d2 drawing:"] {
        assert!(view.contains(backing), "{backing} not in:\n{view}");
    }
    assert_eq!(
        part_string(&out, "word/charts/chart1.xml").as_deref(),
        Some(BAR_CHART)
    );
    assert_eq!(
        part_string(&out, "word/charts/chart2.xml").as_deref(),
        Some(PIE_CHART)
    );
}

/// A fence the user edited rebuilds the chart part: the changed values
/// reach `c:v` and the rest — series, counts, styling — stays.
#[test]
fn an_edited_mermaid_fence_rewrites_the_chart() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), charts_docx()).unwrap();
    let view = jubarte(dir.path(), &["read", "in.docx", "--no-page-markers"]);
    let edited = view
        .replace("bar [12, 18, 15, 22]", "bar [99, 18, 15, 22]")
        .replace("\"Beta\" : 30", "\"Beta\" : 31");
    std::fs::write(dir.path().join("view.md"), &edited).unwrap();
    jubarte(dir.path(), &["convert", "view.md", "-o", "out.docx"]);
    jubarte(dir.path(), &["validate", "out.docx"]);
    let out = std::fs::read(dir.path().join("out.docx")).unwrap();
    let bar = part_string(&out, "word/charts/chart1.xml").expect("chart1.xml");
    assert!(bar.contains("<c:v>99</c:v>"), "{bar}");
    assert!(bar.contains("<c:v>Costs</c:v>"), "{bar}");
    let pie = part_string(&out, "word/charts/chart2.xml").expect("chart2.xml");
    assert!(pie.contains("<c:v>31</c:v>"), "{pie}");
    assert!(pie.contains("<c:v>Gamma</c:v>"), "{pie}");
}

/// A small PNG, `size` pixels square.
fn png(size: u32) -> Vec<u8> {
    use image::ImageEncoder as _;
    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new(&mut out)
        .write_image(
            &vec![0u8; (size * size) as usize],
            size,
            size,
            image::ExtendedColorType::L8,
        )
        .unwrap();
    out
}

/// Google Docs' image syntax — `![][imageN]` uses, one `<data:…>`
/// definition a payload at the end — keeps every byte through the round
/// trip: an empty alt stays empty (the `wp:docPr` name Word gives a
/// picture is not its alt text), each distinct payload is one media part
/// and one image relationship however often it is used, and the view
/// prints one definition a payload, `imageN` in order of first use.
/// (`tests/fixtures/md-roundtrip/docx-libraries-comparison.md`, #556.)
#[test]
fn google_docs_images_share_one_definition_per_payload() {
    use base64::Engine as _;
    let b64 = |bytes: &[u8]| base64::engine::general_purpose::STANDARD.encode(bytes);
    let one = b64(&png(2));
    let two = b64(&png(3));
    let body = format!(
        "a ![][image1] b ![][image2] c ![][image1]\n\n\
         [image1]: <data:image/png;base64,{one}>\n\
         [image2]: <data:image/png;base64,{two}>\n"
    );
    let (view, docx) = round_trip(&body);
    assert!(
        view.contains("a ![][image1] b ![][image2] c ![][image1]"),
        "{view}"
    );
    assert_eq!(view.matches("![][image1]").count(), 2, "{view}");
    assert_eq!(view.matches("![][image2]").count(), 1, "{view}");
    assert_eq!(
        view.matches("[image1]: <data:image/png;base64,").count(),
        1,
        "{view}"
    );
    assert_eq!(
        view.matches("[image2]: <data:image/png;base64,").count(),
        1,
        "{view}"
    );
    assert!(!view.contains("[image3]"), "{view}");
    assert!(!view.contains("![Picture"), "{view}");
    assert!(
        view.contains(&format!("[image1]: <data:image/png;base64,{one}>")),
        "{view}"
    );
    assert!(
        view.contains(&format!("[image2]: <data:image/png;base64,{two}>")),
        "{view}"
    );
    // Two payloads: two media parts and two image relationships between
    // them, shared by the three blips the document.xml holds.
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&docx)).unwrap();
    let media: Vec<String> = (0..zip.len())
        .map(|i| {
            zip.by_index(i)
                .unwrap()
                .name()
                .expect("decode ZIP fixture name")
                .into_owned()
        })
        .filter(|name| name.starts_with("word/media/"))
        .collect();
    assert_eq!(media.len(), 2, "{media:?}");
    let rels = part(&docx, "word/_rels/document.xml.rels");
    assert_eq!(rels.matches("/image\"").count(), 2, "{rels}");
    assert_eq!(
        part(&docx, "word/document.xml").matches("r:embed").count(),
        3
    );
}

fn box_view(input: &[u8]) -> String {
    jubarte::markdown::docx_to_markdown(input, &jubarte::markdown::MarkdownOptions::default())
        .unwrap()
        .markdown
}

fn write_box_view(view: &str) -> Result<Vec<u8>, jubarte::markdown::MarkdownError> {
    jubarte::markdown::markdown_to_docx(view, &jubarte::markdown::DocxOptions::default())
        .map(|written| written.docx)
}

fn refuses_box(view: &str, name: &str) {
    let result = write_box_view(view);
    assert!(result.is_err(), "must refuse ambiguous text-box {name}");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains(&format!("text-box {name}:"))
    );
}

fn alternate_boxes(nested: bool) -> Vec<u8> {
    let input = text_box_docx();
    let xml = part(&input, "word/document.xml");
    let drawing = xml_element(&xml, "w:drawing");
    let second = drawing.replace("Added box", "Other box");
    let choice = if nested {
        drawing.replace(
            "</w:txbxContent>",
            &format!("<w:p><w:r>{second}</w:r></w:p></w:txbxContent>"),
        )
    } else {
        format!("{drawing}{second}")
    };
    let body = format!(
        r#"<w:p><w:r><mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><mc:Choice Requires="wps">{choice}</mc:Choice><mc:Fallback>{choice}</mc:Fallback></mc:AlternateContent></w:r></w:p>"#
    );
    docx_with(&body, &[])
}

#[test]
fn missing_or_annotated_box_markers_refuse_conversion() {
    let view = box_view(&text_box_docx());
    for broken in [
        view.replace("<!-- text-box d1.0 -->", "")
            .replace("<!-- /text-box -->", ""),
        view.replace("<!-- text-box d1.0 -->", "<!-- text-box d1.0 was here -->"),
        view.replace("<!-- /text-box -->", ""),
        view.replace("<!-- text-box d1.0 -->", "<!-- text-box renamed -->"),
    ] {
        refuses_box(&broken, "d1.0");
    }
}

#[test]
fn a_missing_second_box_never_borrows_the_first_story() {
    let view = box_view(&alternate_boxes(false));
    assert!(view.contains("<!-- text-box d1.1 -->"), "{view}");
    let start = view.find("<!-- text-box d1.1 -->").unwrap();
    let end =
        start + view[start..].find("<!-- /text-box -->").unwrap() + "<!-- /text-box -->".len();
    refuses_box(&format!("{}{}", &view[..start], &view[end..]), "d1.1");
}

#[test]
fn duplicate_and_earlier_box_openers_refuse_conversion() {
    let view = box_view(&text_box_docx());
    let start = view.find("<!-- text-box d1.0 -->").unwrap();
    let end =
        start + view[start..].find("<!-- /text-box -->").unwrap() + "<!-- /text-box -->".len();
    refuses_box(&format!("{}\n\n{view}", &view[start..end]), "d1.0");
    refuses_box(
        &format!("<!-- text-box d1.0 -->\n\nBody note\n\n{view}"),
        "d1.0",
    );
}

#[test]
fn literal_box_markers_do_not_steal_the_real_region() {
    let input = text_box_docx();
    let xml = part(&input, "word/document.xml")
        .replace("Added box", "Literal &lt;!-- /text-box --&gt; words");
    let view = box_view(&replace_entry(&input, "word/document.xml", xml.as_bytes()));
    let out = write_box_view(&view).unwrap();
    assert_eq!(
        xml_element(&part(&out, "word/document.xml"), "wp:anchor"),
        xml_element(&xml, "wp:anchor").replace("id=\"7\"", "id=\"1\"")
    );
    for note in [
        "`<!-- text-box d1.0 -->`",
        "```\n<!-- text-box d1.0 -->\n<!-- /text-box -->\n```",
    ] {
        let view = format!("{note}\n\n{}", box_view(&input));
        let out = write_box_view(&view).unwrap();
        assert!(
            xml_element(&part(&out, "word/document.xml"), "w:txbxContent").contains("Added box")
        );
    }
}

#[test]
fn an_unescaped_closer_ends_the_box_at_the_parser_boundary() {
    let view =
        box_view(&text_box_docx()).replace("Added box", "First\n\n<!-- /text-box -->\n\nOutside");
    let out = write_box_view(&view).unwrap();
    let xml = part(&out, "word/document.xml");
    let boxed = xml_element(&xml, "w:txbxContent");
    assert!(
        boxed.contains("First") && !boxed.contains("Outside"),
        "{xml}"
    );
    assert_eq!(xml.matches("Outside").count(), 1);
}

#[test]
fn editing_an_outer_box_keeps_its_nested_shape() {
    let view = box_view(&alternate_boxes(true));
    println!(
        "NESTED VIEW:\n{}",
        view.lines()
            .filter(|line| !line.contains(": drawing:"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(view.contains("Added box"), "{view}");
    let out = write_box_view(&view.replace("Added box", "Edited outer"));
    assert!(
        out.is_ok(),
        "outer words must be editable: {}",
        out.unwrap_err()
    );
    let xml = part(&out.unwrap(), "word/document.xml");
    assert_eq!(xml.matches("Other box").count(), 2, "{xml}");
    assert_eq!(xml.matches("Edited outer").count(), 2, "{xml}");
}

#[test]
fn an_edited_alternate_box_updates_both_branches_and_is_a_fixed_point() {
    let view = box_view(&alternate_boxes(false))
        .replace("Added box", "Edited first")
        .replace("Other box", "Edited second");
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    let payload = view.split_once("[d1]: drawing:").unwrap().1.trim();
    let drawing: serde_json::Value =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap();
    assert_eq!(drawing["boxes"].as_array().unwrap().len(), 4);
    let frame = drawing["xml"].as_str().unwrap();
    assert_eq!(frame.matches("<w:txbxContent>").count(), 4);
    assert!(!frame.contains("Added box") && !frame.contains("Other box"));
    let out = write_box_view(&view).unwrap();
    let xml = part(&out, "word/document.xml");
    for branch in ["mc:Choice", "mc:Fallback"] {
        let branch = xml_element(&xml, branch);
        assert_eq!(branch.matches("Edited first").count(), 1, "{xml}");
        assert_eq!(branch.matches("Edited second").count(), 1, "{xml}");
        assert!(!branch.contains("Added box") && !branch.contains("Other box"));
    }
    let second = box_view(&out);
    assert_eq!(box_view(&write_box_view(&second).unwrap()), second);
}

#[test]
fn rebuilt_box_stories_end_in_a_paragraph_after_a_table() {
    let input = text_box_docx();
    let xml = part(&input, "word/document.xml");
    let old = xml_element(&xml, "w:txbxContent");
    let content = r#"<w:txbxContent><w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Heading</w:t></w:r></w:p><w:p><w:r><w:t>Item</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:txbxContent>"#;
    let input = replace_entry(
        &input,
        "word/document.xml",
        xml.replace(old, content).as_bytes(),
    );
    let out = write_box_view(&box_view(&input).replace("Heading", "Head")).unwrap();
    let xml = part(&out, "word/document.xml");
    let boxed = xml_element(&xml, "w:txbxContent");
    assert!(
        boxed.ends_with("<w:p/></w:txbxContent>") || boxed.ends_with("</w:p></w:txbxContent>"),
        "{boxed}"
    );
}

#[test]
fn crlf_and_trailing_spaces_restore_unprinted_box_properties() {
    let input = text_box_docx();
    let xml = part(&input, "word/document.xml").replace(
        "<w:t xml:space=\"preserve\">Added box",
        "<w:rPr><w:lang w:val=\"pt-BR\"/></w:rPr><w:t xml:space=\"preserve\">Added box",
    );
    let xml = xml.replace(
        "</w:txbxContent>",
        "<w:p><w:r><w:t>Second line</w:t></w:r></w:p></w:txbxContent>",
    );
    let input = replace_entry(&input, "word/document.xml", xml.as_bytes());
    let view = box_view(&input);
    for saved in [
        view.replace('\n', "\r\n"),
        view.replace("Added box\n", "Added box   \n"),
    ] {
        let out = write_box_view(&saved).unwrap();
        assert_eq!(
            xml_element(&part(&out, "word/document.xml"), "wp:anchor"),
            xml_element(&xml, "wp:anchor").replace("id=\"7\"", "id=\"1\"")
        );
    }
}

#[test]
fn cli_box_refusal_is_nonzero_and_names_the_box() {
    let dir = tempfile::tempdir().unwrap();
    let view = box_view(&text_box_docx())
        .replace("<!-- text-box d1.0 -->", "")
        .replace("<!-- /text-box -->", "");
    std::fs::write(dir.path().join("box.md"), view).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["convert", "box.md", "-o", "box.docx"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("text-box d1.0:"));
    assert!(!dir.path().join("box.docx").exists());
}

#[test]
fn legacy_payload_words_need_no_marker_region() {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    let view = box_view(&text_box_docx());
    let mut drawing: serde_json::Value = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(view.split_once("[d1]: drawing:").unwrap().1.trim())
            .unwrap(),
    )
    .unwrap();
    let xml = drawing["xml"].as_str().unwrap().replace(
        "<w:t xml:space=\"preserve\"/>",
        "<w:t xml:space=\"preserve\">Added box</w:t>",
    );
    drawing["xml"] = xml.into();
    drawing.as_object_mut().unwrap().remove("boxes");
    let legacy = format!(
        "![Text Box][d1]\n\n[d1]: drawing:{}\n",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&drawing).unwrap())
    );
    let out = write_box_view(&legacy).unwrap();
    assert!(xml_element(&part(&out, "word/document.xml"), "w:txbxContent").contains("Added box"));
}

#[test]
fn header_and_footer_boxes_keep_their_payload_words_without_regions() {
    let xml = part(&text_box_docx(), "word/document.xml");
    let drawing = xml_element(&xml, "w:drawing");
    for (kind, root) in [("header", "hdr"), ("footer", "ftr")] {
        let name = format!("word/{kind}1.xml");
        let content_type =
            format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml");
        let rel_type = format!("{R_NS}/{kind}");
        let story = format!(
            r#"<w:{root} xmlns:w="{}"><w:p><w:r>{drawing}</w:r></w:p></w:{root}>"#,
            common::docx::W_NS
        );
        let input = common::docx::docx_with_sect(
            &para("Body"),
            &[Part {
                name: &name,
                content_type: &content_type,
                rel_type: &rel_type,
                xml: &story,
            }],
            &format!(r#"<w:{kind}Reference w:type="default" r:id="rIdX0"/>"#),
        );
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("in.docx"), &input).unwrap();
        let view = jubarte(dir.path(), &["read", "in.docx", "--no-page-markers"]);
        assert!(!view.contains("<!-- text-box"), "{view}");
        let out = write_box_view(&view).unwrap();
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().unwrap().into_owned())
            .filter(|n| n.starts_with(&format!("word/{kind}")) && n.ends_with(".xml"))
            .collect();
        assert!(!names.is_empty());
        assert!(
            names
                .iter()
                .any(|name| part(&out, name).contains("Added box"))
        );
    }
}

#[test]
fn unedited_nested_box_keeps_the_nested_shape_and_words() {
    let view = box_view(&alternate_boxes(true));
    let out = write_box_view(&view).unwrap();
    let xml = part(&out, "word/document.xml");
    for branch in ["mc:Choice", "mc:Fallback"] {
        assert!(xml_element(&xml, branch).contains("Other box"), "{xml}");
        assert_eq!(
            xml_element(&xml, branch).matches("<w:drawing").count(),
            2,
            "{xml}"
        );
    }
}

// The nested story uses Word's run-level shape, in both outer branches.
fn nested_box_docx(mismatch: bool) -> Vec<u8> {
    let base = text_box_docx();
    let xml = part(&base, "word/document.xml");
    let drawing = xml_element(&xml, "w:drawing");
    let inner = drawing
        .replace("Added box", "Bob")
        .replace("Text Box 7", "Text Box 9")
        .replace("id=\"7\"", "id=\"9\"")
        .replace("2374265", "1500000")
        .replace("<wp:posOffset>0", "<wp:posOffset>12345");
    let vml = |words: &str, width: &str| {
        format!(
            r#"<w:pict xmlns:v="urn:schemas-microsoft-com:vml"><v:shape id="Inner" style="position:absolute;width:{width}pt;height:36pt"><v:textbox><w:txbxContent><w:p><w:r><w:t>{words}</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict>"#
        )
    };
    let inner_fallback = vml("Bob", "118");
    let inner_ac = format!(
        r#"<mc:AlternateContent><mc:Choice Requires="wps">{inner}</mc:Choice><mc:Fallback>{inner_fallback}</mc:Fallback></mc:AlternateContent>"#
    );
    let story = |nested: &str| {
        format!(
            "<w:txbxContent><w:p><w:r><w:t>Alice</w:t></w:r><w:r>{nested}</w:r><w:r><w:t>Carol Diaz</w:t></w:r></w:p></w:txbxContent>"
        )
    };
    let original = "<w:txbxContent><w:p><w:r><w:t xml:space=\"preserve\">Added box</w:t></w:r></w:p></w:txbxContent>";
    let outer = drawing
        .replace(original, &story(&inner_ac))
        .replace("cy=\"457200\"", "cy=\"3000000\"");
    let fallback = vml("Alice", "400");
    let fallback = fallback.replace(
        "<w:txbxContent><w:p><w:r><w:t>Alice</w:t></w:r></w:p></w:txbxContent>",
        &story(&vml("Bob", if mismatch { "119" } else { "118" })),
    );
    docx_with(
        &format!(
            r#"<w:p><w:r><mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><mc:Choice Requires="wps">{outer}</mc:Choice><mc:Fallback>{fallback}</mc:Fallback></mc:AlternateContent></w:r></w:p>"#
        ),
        &[],
    )
}

fn nested_warning_run(args: &[&str], probe: char) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    let bytes = if probe == 'B' {
        text_box_docx()
    } else {
        nested_box_docx(false)
    };
    let bytes = if probe == 'D' {
        use jubarte::{
            namespaces::{MC, W},
            xmllinq::Dom,
        };
        let mut dom = Dom::new();
        let root = dom.parse_xdocument(&part(&bytes, "word/document.xml"));
        let outer = dom.descendants(root, Some(&W::name("txbxContent")))[0];
        let inner = dom.descendants(outer, Some(&MC::name("AlternateContent")))[0];
        dom.remove(inner);
        replace_entry(
            &bytes,
            "word/document.xml",
            dom.serialize_document(root).as_bytes(),
        )
    } else {
        bytes
    };
    let xml = part(&bytes, "word/document.xml").replace(
        "<w:body>",
        "<w:body><w:p><w:r><w:t>Alice and Carol Diaz</w:t></w:r></w:p>",
    );
    std::fs::write(
        dir.path().join("in.docx"),
        replace_entry(&bytes, "word/document.xml", xml.as_bytes()),
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir.path())
        .output()
        .unwrap()
}

fn assert_nested_warning(args: &[&str], code: i32) {
    let output = nested_warning_run(args, 'A');
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(code), "{args:?}: {stderr}");
    let warnings = stderr
        .lines()
        .filter(|line| line.contains("Word cannot open this document"))
        .collect::<Vec<_>>();
    assert_eq!(
        warnings,
        [
            "warning: text box Text Box 7 holds another text box (Text Box 9); Word cannot open this document"
        ],
        "{args:?}: {stderr}"
    );
}

#[test]
fn nested_box_read_warns_once() {
    assert_nested_warning(&["read", "in.docx", "--no-page-markers"], 0);
}

#[test]
fn nested_box_convert_to_md_warns_once() {
    assert_nested_warning(
        &["convert", "in.docx", "--to", "md", "--no-page-markers"],
        0,
    );
}

#[test]
fn nested_box_convert_to_commonmark_warns_once() {
    assert_nested_warning(
        &[
            "convert",
            "in.docx",
            "--to",
            "md",
            "--commonmark",
            "--no-page-markers",
        ],
        0,
    );
}

#[test]
fn nested_box_edit_warns_once() {
    assert_nested_warning(
        &[
            "edit",
            "in.docx",
            "-p",
            "p0",
            "--content",
            "Bob",
            "--author",
            "Alice",
        ],
        0,
    );
}

#[test]
fn nested_box_refused_edit_warns_once() {
    assert_nested_warning(
        &[
            "edit",
            "in.docx",
            "-p",
            "p999",
            "--content",
            "Bob",
            "--author",
            "Alice",
        ],
        3,
    );
}

#[test]
fn nested_box_dry_run_warns_once() {
    assert_nested_warning(
        &[
            "edit",
            "in.docx",
            "-p",
            "p0",
            "--content",
            "Bob",
            "--author",
            "Alice",
            "--dry-run",
        ],
        0,
    );
}

#[test]
fn nested_box_typed_edit_warns_once() {
    assert_nested_warning(
        &[
            "edit",
            "replace",
            "in.docx",
            "--find",
            "Alice and Carol Diaz",
            "--text",
            "Bob",
            "--track",
            "off",
            "-o",
            "out.docx",
        ],
        0,
    );
}

#[test]
fn nested_box_edit_template_warns_once() {
    assert_nested_warning(&["edit", "template", "in.docx", "--dry-run", "--jsonl"], 0);
}

#[test]
fn nested_box_read_output_warns_once() {
    assert_nested_warning(&["read", "in.docx", "--no-page-markers", "-o", "out.md"], 0);
}

#[test]
fn nested_box_read_for_edit_warns_once() {
    assert_nested_warning(&["read", "in.docx", "--for-edit"], 0);
}

#[test]
fn nested_box_read_tables_warns_once() {
    assert_nested_warning(&["read", "in.docx", "--tables"], 0);
}

#[test]
fn single_box_commands_do_not_warn_about_nesting() {
    for args in [
        vec!["read", "in.docx", "--no-page-markers"],
        vec!["convert", "in.docx", "--to", "md", "--no-page-markers"],
        vec![
            "edit",
            "in.docx",
            "-p",
            "p0",
            "--content",
            "Bob",
            "--author",
            "Alice",
            "--dry-run",
        ],
    ] {
        let output = nested_warning_run(&args, 'B');
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("Word cannot open this document")
        );
    }
}

fn directly_owned_box_words(xml: &str) -> Vec<String> {
    use jubarte::{namespaces::W, xmllinq::Dom};
    let mut dom = Dom::new();
    let root = dom.parse_xdocument(xml);
    dom.descendants(root, Some(&W::name("txbxContent")))
        .iter()
        .map(|&story| {
            dom.descendants(story, Some(&W::t()))
                .into_iter()
                .filter(|&t| {
                    dom.ancestors_and_self(t, Some(&W::name("txbxContent")))
                        .first()
                        == Some(&story)
                })
                .map(|t| dom.value(t))
                .collect()
        })
        .collect()
}

#[test]
fn nested_box_tokens_and_flat_regions_are_visible_at_the_run_position() {
    let view = box_view(&nested_box_docx(false));
    let visible = view
        .lines()
        .filter(|line| !line.contains(": drawing:"))
        .collect::<Vec<_>>()
        .join("\n");
    println!("NESTED VIEW:\n{visible}");
    assert!(
        view.contains("Alice![Text Box 9][d2]Carol Diaz"),
        "{visible}"
    );
    assert!(
        view.contains("<!-- text-box d2.0 -->\n\nBob\n"),
        "{visible}"
    );
    let outer_close = view.find("<!-- /text-box -->").unwrap();
    assert!(outer_close < view.find("<!-- text-box d2.0 -->").unwrap());
}

#[test]
fn nested_box_outer_words_can_be_edited() {
    let view = box_view(&nested_box_docx(false)).replace("Alice", "Alice edited");
    let out = write_box_view(&view);
    assert!(out.is_ok(), "{}", out.unwrap_err());
    let xml = part(&out.unwrap(), "word/document.xml");
    assert_eq!(
        directly_owned_box_words(&xml),
        [
            "Alice editedCarol Diaz",
            "Bob",
            "Bob",
            "Alice editedCarol Diaz",
            "Bob"
        ]
    );
    assert_eq!(xml.matches("<wp:anchor ").count(), 2, "{xml}");
}

#[test]
fn nested_box_inner_words_can_be_edited() {
    let view = box_view(&nested_box_docx(false));
    assert!(view.contains("\nBob\n"), "nested words must be editable");
    let edited = view.replace("\nBob\n", "\nBob edited\n");
    let out = write_box_view(&edited).unwrap();
    let xml = part(&out, "word/document.xml");
    assert_eq!(
        directly_owned_box_words(&xml),
        [
            "AliceCarol Diaz",
            "Bob edited",
            "Bob edited",
            "AliceCarol Diaz",
            "Bob edited"
        ]
    );
}

#[test]
fn nested_box_combined_edits_are_an_exact_markdown_fixed_point() {
    let view = box_view(&nested_box_docx(false));
    assert!(view.contains("\nBob\n"), "nested words must be editable");
    let view = view
        .replace("Alice", "Alice edited")
        .replace("\nBob\n", "\nBob edited\n");
    let out = write_box_view(&view).unwrap();
    let second = box_view(&out);
    let visible = |view: &str| {
        view.lines()
            .filter(|line| !line.contains(": drawing:"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(visible(&view), visible(&second));
    assert_eq!(box_view(&write_box_view(&second).unwrap()), second);
    assert_eq!(
        directly_owned_box_words(&part(&out, "word/document.xml")),
        [
            "Alice editedCarol Diaz",
            "Bob edited",
            "Bob edited",
            "Alice editedCarol Diaz",
            "Bob edited"
        ]
    );
}

#[test]
fn nested_box_unedited_round_trip_keeps_both_anchors_and_nesting() {
    let original = nested_box_docx(false);
    let view = box_view(&original);
    assert!(view.contains("[d2]"), "inner token missing");
    let out = write_box_view(&view).unwrap();
    assert_eq!(box_view(&out), view);
    // Compare complete anchors with only generated docPr ids and namespace
    // declaration placement normalized; all drawing properties and text stay.
    let anchors = |bytes: &[u8]| {
        use jubarte::xmllinq::Dom;
        let mut dom = Dom::new();
        let root = dom.parse_xdocument(&part(bytes, "word/document.xml"));
        let anchors = dom
            .descendants(root, None)
            .into_iter()
            .filter(|&n| {
                dom.name(n)
                    .is_some_and(|name| name.local_name() == "anchor")
            })
            .collect::<Vec<_>>();
        for n in dom.descendants(root, None) {
            for (name, _) in dom.attributes(n) {
                if dom.is_namespace_declaration(&name) {
                    dom.set_attribute_value(n, &name, None);
                }
                if dom.name(n).is_some_and(|name| name.local_name() == "docPr")
                    && name.local_name() == "id"
                {
                    dom.set_attribute_value(n, &name, Some("0"));
                }
            }
        }
        anchors
            .into_iter()
            .map(|n| dom.serialize_element(n))
            .collect::<Vec<_>>()
    };
    let actual = anchors(&out);
    let expected = anchors(&original);
    assert_eq!(actual.len(), 2);
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        common::assert_xml_structurally_eq(actual, expected, "nested anchor");
    }
}

#[test]
fn nested_box_token_deletion_refuses_with_the_inner_box_name() {
    let view = box_view(&nested_box_docx(false));
    assert!(view.contains("![Text Box 9][d2]"), "inner token missing");
    let result = write_box_view(&view.replace("![Text Box 9][d2]", ""));
    assert!(result.is_err(), "deleting d2 must refuse");
    let error = result.unwrap_err().to_string();
    assert!(error.contains("d2") && error.contains("token"), "{error}");
}

#[test]
fn nested_box_mismatched_fallback_stays_readable_and_refuses_edits() {
    let view = box_view(&nested_box_docx(true));
    assert!(view.contains("Bob"), "nested words must remain readable");
    assert!(!view.contains("[d2]"));
    let result = write_box_view(&view.replace("Alice", "Alice edited"));
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("d1.0") && error.contains("Choice/Fallback"),
        "{error}"
    );
}

#[test]
fn nested_box_regions_reject_undefined_tokens_and_out_of_range_ordinals() {
    let view = box_view(&text_box_docx());
    for name in ["d9.0", "d1.99"] {
        let broken = format!("{view}\n\n<!-- text-box {name} -->\n\nBob\n\n<!-- /text-box -->\n");
        let result = write_box_view(&broken);
        assert!(result.is_err(), "region {name} must refuse");
        assert!(result.unwrap_err().to_string().contains(name));
    }
}

#[test]
fn nested_box_duplicate_token_refuses() {
    let view = box_view(&nested_box_docx(false));
    assert!(view.contains("![Text Box 9][d2]"), "inner token missing");
    let result =
        write_box_view(&view.replace("![Text Box 9][d2]", "![Text Box 9][d2]![Text Box 9][d2]"));
    assert!(result.is_err(), "a nested token has exactly one owner");
    assert!(result.unwrap_err().to_string().contains("d2"));
}

#[test]
fn fallback_only_box_commands_do_not_warn_about_nesting() {
    for args in [
        vec!["read", "in.docx", "--no-page-markers"],
        vec!["convert", "in.docx", "--to", "md", "--no-page-markers"],
        vec![
            "edit",
            "in.docx",
            "-p",
            "p0",
            "--content",
            "Bob",
            "--author",
            "Alice",
            "--dry-run",
        ],
    ] {
        let output = nested_warning_run(&args, 'D');
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("Word cannot open this document")
        );
    }
}

/// A paragraph style used only inside a text box is the legend's too: the
/// written docx defines it as the original did, name and spacing, so the
/// box's paragraph does not fall back to Normal (#1423).
#[test]
fn a_text_box_s_paragraph_style_is_defined_in_the_written_docx() {
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:style><w:style w:type="paragraph" w:styleId="NoSpacing"><w:name w:val="No Spacing"/><w:uiPriority w:val="1"/><w:qFormat/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr></w:style></w:styles>"#
    );
    let styles_rel = format!("{R_NS}/styles");
    let original = text_box_docx_with(
        "<w:pPr><w:pStyle w:val=\"NoSpacing\"/></w:pPr>",
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: &styles_rel,
            xml: &styles,
        }],
    );
    let (view, out) = docx_round_trip(&original);
    assert!(view.contains("\n  NoSpacing: "), "{view}");
    let written = part(&out, "word/styles.xml");
    let no_spacing = written
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"NoSpacing\""))
        .unwrap_or_else(|| panic!("NoSpacing is not defined: {written}"));
    assert!(
        no_spacing.contains("<w:name w:val=\"No Spacing\"/>"),
        "{no_spacing}"
    );
    assert!(no_spacing.contains("w:after=\"0\""), "{no_spacing}");
    assert!(part(&out, "word/document.xml").contains("<w:pStyle w:val=\"NoSpacing\"/>"));
}

/// The cover page's Abstract: a content control bound to the custom XML
/// store `{55AF091B-…}` (CoverPageProperties).
const ABSTRACT_STORE: &str = "{55AF091B-3C7A-41E3-B477-F2FDAA23CFDA}";
const COVER_NS: &str = "http://schemas.microsoft.com/office/2006/coverPageProps";

/// [`text_box_docx`] whose box holds a content control bound to the cover
/// page's Abstract, reading `My abstract`, with its custom XML store.
fn bound_text_box_docx() -> Vec<u8> {
    bound_text_box_docx_with("/ns0:CoverPageProperties[1]/ns0:Abstract[1]", "<w:text/>")
}

/// [`bound_text_box_docx`] whose control binds `xpath` and is of the type
/// `control` names (`<w:text/>`, a check box, …).
fn bound_text_box_docx_with(xpath: &str, control: &str) -> Vec<u8> {
    let sdt = format!(
        "<w:sdt><w:sdtPr><w:alias w:val=\"Abstract\"/><w:id w:val=\"7\"/>\
         <w:dataBinding w:prefixMappings=\"xmlns:ns0='{COVER_NS}'\" \
         w:xpath=\"{xpath}\" w:storeItemID=\"{ABSTRACT_STORE}\"/>\
         {control}</w:sdtPr><w:sdtContent><w:r><w:t>My abstract</w:t></w:r></w:sdtContent></w:sdt>"
    );
    let item = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?><CoverPageProperties xmlns=\"{COVER_NS}\"><PublishDate/><Abstract>My abstract</Abstract></CoverPageProperties>"
    );
    let props = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?><ds:datastoreItem ds:itemID=\"{ABSTRACT_STORE}\" xmlns:ds=\"http://schemas.openxmlformats.org/officeDocument/2006/customXml\"><ds:schemaRefs><ds:schemaRef ds:uri=\"{COVER_NS}\"/></ds:schemaRefs></ds:datastoreItem>"
    );
    let item_rels = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"{R_NS}/customXmlProps\" Target=\"itemProps1.xml\"/></Relationships>"
    );
    let docx = text_box_docx_with(
        "",
        &[
            Part {
                name: "customXml/item1.xml",
                content_type: "application/xml",
                rel_type: "",
                xml: &item,
            },
            Part {
                name: "customXml/itemProps1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.customXmlProperties+xml",
                rel_type: "",
                xml: &props,
            },
            Part {
                name: "customXml/_rels/item1.xml.rels",
                content_type: "application/vnd.openxmlformats-package.relationships+xml",
                rel_type: "",
                xml: &item_rels,
            },
        ],
    );
    let document = part(&docx, "word/document.xml").replace(
        "<w:r><w:t xml:space=\"preserve\">Added box</w:t></w:r>",
        &sdt,
    );
    assert!(document.contains(ABSTRACT_STORE), "{document}");
    let docx = replace_entry(&docx, "word/document.xml", document.as_bytes());
    let rels = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rIdC1\" Type=\"{R_NS}/customXml\" Target=\"../customXml/item1.xml\"/></Relationships>"
    );
    replace_entry(&docx, "word/_rels/document.xml.rels", rels.as_bytes())
}

/// A content control in a text box bound to a custom XML store keeps its
/// text in Word: Word fills a bound control from its store when it opens
/// the file, so the docx written back holds the store, the bound node
/// reading what the control shows (#1427).
#[test]
fn a_bound_content_control_s_store_is_written_back() {
    let original = bound_text_box_docx();
    let (view, out) = docx_round_trip(&original);
    assert!(view.contains("My abstract"), "{view}");
    let document = part(&out, "word/document.xml");
    assert!(
        document.contains(ABSTRACT_STORE),
        "the binding is kept: {document}"
    );
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
    let names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().unwrap().to_string())
        .collect();
    let mut read = |name: &str| {
        let mut text = String::new();
        std::io::Read::read_to_string(&mut zip.by_name(name).unwrap(), &mut text).unwrap();
        text
    };
    let props: Vec<String> = names
        .iter()
        .filter(|n| n.starts_with("customXml/itemProps"))
        .map(|n| read(n))
        .collect();
    assert!(
        props
            .iter()
            .any(|p| p.contains(&format!("ds:itemID=\"{ABSTRACT_STORE}\""))),
        "no store {ABSTRACT_STORE} among {names:?}: {props:?}"
    );
    let items: Vec<String> = names
        .iter()
        .filter(|n| n.starts_with("customXml/item") && !n.contains("Props") && n.ends_with(".xml"))
        .map(|n| read(n))
        .collect();
    assert!(
        items
            .iter()
            .any(|i| i.contains(COVER_NS) && i.contains(">My abstract</")),
        "{items:?}"
    );
    let rels = read("word/_rels/document.xml.rels");
    assert!(rels.contains("relationships/customXml\""), "{rels}");
    let types = read("[Content_Types].xml");
    assert!(types.contains("customXmlProperties+xml"), "{types}");
    common::validity::assert_word_valid_package(&out);
}

/// A binding no store can be built for (a path to an attribute, a check
/// box, whose store value is not its text) loses its `w:dataBinding`, so
/// Word keeps the text the control shows instead of emptying it for a
/// store the package lacks (#1427).
#[test]
fn a_binding_with_no_store_to_build_leaves_its_control_s_text() {
    for (xpath, control) in [
        ("/ns0:CoverPageProperties[1]/@ns0:Abstract", "<w:text/>"),
        (
            "/ns0:CoverPageProperties[1]/ns0:Abstract[1]",
            "<w14:checkbox xmlns:w14=\"http://schemas.microsoft.com/office/word/2010/wordml\">\
             <w14:checked w14:val=\"0\"/></w14:checkbox>",
        ),
    ] {
        let (view, out) = docx_round_trip(&bound_text_box_docx_with(xpath, control));
        assert!(view.contains("My abstract"), "{view}");
        let document = part(&out, "word/document.xml");
        assert!(!document.contains("dataBinding"), "{xpath}: {document}");
        assert!(
            xml_element(&document, "w:txbxContent").contains("My abstract"),
            "{document}"
        );
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().unwrap().to_string())
            .collect();
        assert!(
            names.iter().all(|n| !n.starts_with("customXml/")),
            "{xpath}: {names:?}"
        );
        common::validity::assert_word_valid_package(&out);
    }
}

/// An edit of a bound control's text in the view shows in Word: the box
/// is written with the edited words (#1101 A), and no store is left
/// holding the old text for Word to fill a control with (#1427).
#[test]
fn an_edited_bound_control_shows_its_edit() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), bound_text_box_docx()).unwrap();
    let view = jubarte(dir.path(), &["read", "in.docx", "--no-page-markers"]);
    let edited = view.replace("My abstract", "Edited abstract");
    assert_ne!(edited, view);
    std::fs::write(dir.path().join("edit.md"), edited).unwrap();
    jubarte(dir.path(), &["convert", "edit.md", "-o", "out.docx"]);
    let out = std::fs::read(dir.path().join("out.docx")).unwrap();
    let document = part(&out, "word/document.xml");
    let boxed = xml_element(&document, "w:txbxContent");
    assert!(boxed.contains("Edited abstract"), "{document}");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).unwrap();
        let mut text = String::new();
        if std::io::Read::read_to_string(&mut entry, &mut text).is_ok() {
            assert!(
                !text.contains("My abstract"),
                "{}: {text}",
                entry.name().unwrap()
            );
        }
    }
    drop(zip);
    common::validity::assert_word_valid_package(&out);
}
