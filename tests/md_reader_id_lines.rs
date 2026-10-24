// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The reader brings back what the agent view's id lines say about a
//! paragraph (`<!-- p2-p4 empty -->`, `<!-- p5 TitlePage center -->`), so a
//! view read into a docx and read again prints the same ids.

mod common;

use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

use common::docx::part_string;
use common::validity::assert_word_valid_package;
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

/// `markdown` under the view's YAML header (`view: tracked`), which makes
/// the reader read its id lines.
fn as_view(markdown: &str) -> String {
    match markdown.strip_prefix("---\n") {
        Some(rest) => format!("---\nview: tracked\n{rest}"),
        None => format!("---\nview: tracked\n---\n\n{markdown}"),
    }
}

/// The view of the docx `markdown` (read as a view) reads into, without
/// its YAML header.
fn round_trip(markdown: &str) -> String {
    let docx = markdown_to_docx(&as_view(markdown), &DocxOptions::default())
        .expect("the view reads")
        .docx;
    assert_word_valid_package(&docx);
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    let body = view.split_once("\n---\n").map_or(view.as_str(), |(_, b)| b);
    body.trim().to_string()
}

#[test]
fn empty_paragraphs_come_back_from_their_id_lines() {
    let md = "<!-- p0 -->\nOne\n\n<!-- p1 empty -->\n\n<!-- p2 -->\nTwo\n\n\
              <!-- p3-p5 empty -->\n\n<!-- p6 -->\nThree\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn an_id_line_alone_makes_no_paragraph() {
    // Only `empty` stands for paragraphs: a plain id line names the text
    // after it.
    let md = "<!-- p0 -->\nOne\n\n<!-- p1 -->\nTwo\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn an_id_line_style_comes_back_on_its_paragraph() {
    let md = "<!-- p0 DocumentTitle -->\nLast Will\n\n<!-- p1 -->\nBody text.\n\n\
              <!-- p2 TitlePage -->\nOf Ann Counsel\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn id_line_alignment_and_indents_come_back() {
    let md = "<!-- p0 BodyText center, first-line 0.5in, left 0.25in, right 1in -->\nCentered\n\n\
              <!-- p1 justify, hanging 0.3in, left 0.3in -->\nJustified\n\n\
              <!-- p2 right -->\nRight\n\n\
              <!-- p3 left -->\nLeft\n\n\
              <!-- p4 left -0.25in -->\nOutdented\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_heading_keeps_its_level_under_the_style_its_id_line_names() {
    let md = "<!-- p0 DocumentTitle -->\n# Last Will\n\n<!-- p1 TextHeading2 center -->\n## Article One\n\n\
              <!-- p2 -->\nBody text.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_paragraph_without_text_comes_back_from_its_id_line() {
    // (A textless paragraph with nothing but a style prints `empty`.)
    let md = "<!-- p0 -->\nOne\n\n<!-- p1 page-break -->\n\n<!-- p2 empty -->\n\n\
              <!-- p3 -->\nTwo\n\n<!-- p4 page-break -->\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_page_break_comes_back_after_its_text() {
    let md = "<!-- p0 page-break -->\nBefore the break\n\n<!-- p1 -->\nAfter it\n";
    assert_eq!(round_trip(md), md.trim());
}

const AUTHORS: &str =
    "---\nauthors:\n  AC: Ann Counsel                  # 2026-10-01T09:00:00Z\n---\n\n";

#[test]
fn revisions_keep_the_numbers_their_tags_name() {
    let body = "<!-- p0 -->\nPay {~~thirty~>forty~~}{>>r7+9 @AC<<} days {++from today++}{>>r12 @AC<<}.\n\n\
                <!-- p1 -->\n{--Gone.--}{>>r3 @AC<<} Kept.\n";
    assert_eq!(round_trip(&format!("{AUTHORS}{body}")), body.trim());
}

#[test]
fn a_revision_number_is_written_as_often_as_the_view_names_it() {
    // Two separately named changes keep the number both name (`r5` twice,
    // as Word's own files repeat an id). A change written as several marks
    // (two runs) keeps its number on the first; the rest take fresh ones,
    // so a number the view names once is on no two marks.
    let body = "<!-- p0 -->\n{++**Bold** plain++}{>>r4 @AC<<} and {++one++}{>>r5 @AC<<} \
                {++two++}{>>r5 @AC<<}.\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{AUTHORS}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    // The one duplicate is the twice-named `r5`, as in Word's own files;
    // nothing else may be invalid.
    let report = common::validity::check_word_valid_package(&docx);
    let repeated: Vec<&jubarte::validate::Finding> = report
        .findings
        .iter()
        .filter(|finding| finding.code == "DUPLICATE_REVISION_ID")
        .collect();
    assert_eq!(report.findings.len(), repeated.len(), "{report:#?}");
    assert!(
        repeated.len() == 1 && repeated[0].message.contains("'5'"),
        "{report:#?}"
    );
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    assert!(
        document.matches("w:ins w:id=\"5\"").count() == 2,
        "r5 on both changes: {document}"
    );
    assert!(
        document.matches("w:ins w:id=\"4\"").count() == 1,
        "r4 on the first mark only: {document}"
    );
}

#[test]
fn a_paragraph_mark_change_comes_back_with_its_number() {
    let body = "<!-- p0 break-ins r4 @AC -->\nThe break after this text is new.\n\n\
                <!-- p1 -->\nNext.\n\n\
                <!-- p2 empty, break-del r6 @AC -->\n\n\
                <!-- p3 BodyText center, break-ins r8 @AC -->\nStyled.\n\n<!-- p4 -->\nLast.\n";
    assert_eq!(round_trip(&format!("{AUTHORS}{body}")), body.trim());
}

#[test]
fn a_text_box_stays_in_its_anchor_paragraph() {
    // The view prints a text box's content as blocks with no id lines,
    // after the paragraph whose `rev` clause names the anchor's change,
    // and the header's `unplaced:` names the box's own carriers. Written
    // back, the box is anchored in that paragraph — no paragraph of its
    // own, so no stamp shifts — and its carriers read back as `unplaced:`
    // again.
    let body = "<!-- p0 -->\nKept text.\n\n\
                <!-- p1 break-del r2 @AC, rev r3 @AC -->\n{--Gone. --}{>>r4 @AC<<}\n\n\
                {++Boxed.++}{>>r3 @AC<<}\n\n\
                <!-- p2 -->\nAfter.\n";
    let header = "---\nauthors:\n  AC: Ann Counsel                  # 2026-10-01T09:00:00Z\n\
unplaced: [ins r5 @AC in p1, ins r6 @AC in p1]\n---\n\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{header}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    let anchor = document
        .find("<w:drawing>")
        .and_then(|at| {
            document[at..]
                .find("</w:drawing>")
                .and_then(|end| document[..at + end].rfind("<w:ins "))
        })
        .unwrap_or(0);
    let text = document.find("Gone.").unwrap_or(0);
    assert!(anchor < text, "the drawing before the text: {document}");
    let box_at = document.find("<w:txbxContent>").unwrap_or(0);
    let boxed = &document[box_at..document.find("</w:txbxContent>").unwrap_or(0)];
    assert!(boxed.contains("Boxed."), "the text in the box: {document}");
    assert!(
        document.contains("w:ins w:id=\"5\""),
        "the box's mark: {document}"
    );
    assert!(
        document.contains("w:ins w:id=\"6\""),
        "the box's text: {document}"
    );
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    assert!(
        view.contains("unplaced: [ins r5 @AC in p1, ins r6 @AC in p1]"),
        "{view}"
    );
    assert!(
        view.contains("<!-- p2 -->\nAfter."),
        "no stamp shift: {view}"
    );
}

#[test]
fn an_unshown_deletion_holds_the_comment_marks_it_wraps() {
    // A paragraph whose text prints only comment marks, inside a deletion
    // its id line names (`rev-del`): the deletion wraps the marks, as in
    // Word, and so counts as a change when read back — an empty `w:del`
    // is hollow, and the `revisions:` header would lose it.
    let body = "<!-- p0 -->\nText.\n\n\
                <!-- p1 break-del r2 @AC, rev-del r3 @AC -->\n{>>c0 @AC: Note.<<}\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{AUTHORS}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    let del = document
        .find("<w:del ")
        .and_then(|at| {
            document[at..]
                .find("</w:del>")
                .map(|end| &document[at..at + end])
        })
        .unwrap_or_default();
    assert!(del.contains("w:commentReference"), "{document}");
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    assert!(
        view.lines().any(|line| line.contains("1 deletion")),
        "{view}"
    );
}

#[test]
fn an_id_line_without_a_break_clause_leaves_the_mark_alone() {
    // Text inserted whole, its paragraph mark not: the id line says so by
    // naming no `break-ins`.
    let body = "<!-- p0 -->\n{++All new text.++}{>>r3 @AC<<}\n\n<!-- p1 -->\nNext.\n";
    assert_eq!(round_trip(&format!("{AUTHORS}{body}")), body.trim());
}

#[test]
fn a_numbered_paragraph_keeps_its_label() {
    // As the view prints them: a list item under its label, numbered as its
    // list counts it; a heading with its label before its text.
    let md = "<!-- p0 num \"Article One\" -->\n# Article One The Trust\n\n\
              <!-- p1 num \"Section One.05\" -->\n## Section One.05 Deep\n\n\
              <!-- p2 num \"(a)\" -->\n1. First.\n\n<!-- p3 num \"(b)\" -->\n2. Second.\n\n\
              <!-- p4 justify, num \"\\\"Q\\\" 1.\" -->\n1. Quoted label.\n\n\
              <!-- p5 bullet -->\n- A bullet.\n\n<!-- p6 -->\nPlain.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn an_empty_id_line_keeps_its_style_and_label() {
    let md =
        "<!-- p0 -->\nOne\n\n<!-- p1 ListParagraph empty, num \"1.\" -->\n\n<!-- p2 -->\nTwo\n";
    assert_eq!(round_trip(md), md.trim());
}

/// The paragraph id lines of the view of `markdown`'s docx.
fn paragraphs(markdown: &str) -> Vec<String> {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the Markdown reads")
        .docx;
    let view = read(&docx, &ReadOptions::default())
        .expect("the docx reads")
        .markdown;
    view.lines()
        .map(str::trim_start)
        .filter(|line| {
            line.strip_prefix("<!-- p")
                .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()))
        })
        .map(str::to_string)
        .collect()
}

#[test]
fn html_comments_in_plain_markdown_write_nothing() {
    // Without the view's header an HTML comment is the author's own: it
    // makes no paragraph and gives none a style or an alignment.
    let plain = "Hello\n\nWorld\n";
    let expected = paragraphs(plain);
    for markdown in [
        "Hello\n\n<!-- p1 empty -->\n\nWorld\n",
        "Hello\n\n<!-- p1 center -->\nWorld\n",
        "Hello\n\n<div>\n<!-- p1 -->\n</div>\n\nWorld\n",
    ] {
        assert_eq!(paragraphs(markdown), expected, "{markdown}");
    }
}

#[test]
fn a_comment_inside_other_html_is_no_id_line_even_in_a_view() {
    let md = as_view("<!-- p0 -->\nOne\n\n<div>\n<!-- p1 center -->\n</div>\n\n<!-- p1 -->\nTwo\n");
    let lines = paragraphs(&md);
    assert_eq!(lines, ["<!-- p0 -->", "<!-- p1 -->"], "{lines:?}");
}

#[test]
fn a_number_word_cannot_read_takes_a_fresh_one() {
    // `w:id` is an `int`: 2147483648 does not fit, so the change and the
    // comment that name it are numbered afresh.
    let body = "<!-- p0 -->\n{++Added++}{>>r2147483648 @AC<<} and \
                {==this==}{>>c2147483648 @AC: Note.<<}.\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{AUTHORS}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    assert!(!document.contains("2147483648"), "{document}");
}

#[test]
fn fresh_numbers_stay_within_what_word_reads() {
    let body = "<!-- p0 -->\n{++One++}{>>r2147483647 @AC<<} and {++two++}{>>noid @AC<<}.\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{AUTHORS}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    assert!(document.contains("w:id=\"2147483647\""), "{document}");
    assert!(!document.contains("2147483648"), "{document}");
}

#[test]
fn an_id_line_before_a_footnote_definition_keeps_its_paragraph() {
    let md = "<!-- p0 -->\nText.[^1]\n\n<!-- p1 page-break -->\n\n[^1]: The note.\n";
    let lines = paragraphs(&as_view(md));
    assert_eq!(
        lines,
        ["<!-- p0 -->", "<!-- p1 page-break -->"],
        "{lines:?}"
    );
}

#[test]
fn a_cell_keeps_its_paragraphs() {
    // The table line counts each row's paragraphs: when a row's cells and
    // `<br>`s add up to that count, every `<br>` there ends a paragraph.
    let md = "<!-- p0 -->\nBefore.\n\n\
              <!-- t0 2x2, cells t0.row0 p1-p3 t0.row1 p4-p5 -->\n\
              |A<br>B|C|\n|-|-|\n|D|E|\n\n<!-- p6 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_line_break_in_a_cell_stays_a_line_break() {
    // Two cells, one `<br>`, two paragraphs: the `<br>` is a line break,
    // which the view reads back as `<br/>`, and `<br/>` stays one.
    let md = "<!-- t0 1x2, cells p0-p1 by row -->\n|A<br>B|C|\n|-|-|\n\n<!-- p2 -->\nAfter.\n";
    let again = md.replace("A<br>B", "A<br/>B");
    assert_eq!(round_trip(md), again.trim());
    assert_eq!(round_trip(&again), again.trim());
}

#[test]
fn a_cell_paragraph_keeps_the_mark_its_table_line_names() {
    let body = "<!-- p0 -->\nBefore.\n\n\
                <!-- t0 2x2, cells t0.row0 p1-p3 t0.row1 p4-p5, break-ins r5 @AC in p1 -->\n\
                |{++A++}{>>r6 @AC<<}{++<br>B++}{>>r7 @AC<<}|C|\n|-|-|\n|D|E|\n\n<!-- p6 -->\nAfter.\n";
    assert_eq!(round_trip(&format!("{AUTHORS}{body}")), body.trim());
}

#[test]
fn a_cell_spanning_columns_comes_back_as_one_cell() {
    // The view prints a span's other columns as empty cells: they are no
    // cells of their own, so the paragraphs after the table keep their ids.
    let md = "<!-- t0 2x2, cells p0-p2 by row, merged t0.row0.cell0 across 2 -->\n\
              |Head||\n|-|-|\n|a|b|\n\n<!-- p3 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_vertical_merge_comes_back_over_its_rows() {
    let md = "<!-- t0 3x2, cells p0-p5 by row, merged t0.row0.cell0 down 3 -->\n\
              |A|B|\n|-|-|\n||C|\n||D|\n\n<!-- p6 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_spanning_merge_comes_back_on_every_row_it_covers() {
    let md = "<!-- t0 2x3, cells p0-p3 by row, merged t0.row0.cell0 across 2 down 2; \
              t0.row1.cell0 across 2 -->\n|A||B|\n|-|-|-|\n|||C|\n\n<!-- p4 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn empty_trailing_columns_come_back() {
    // The view drops the columns no row writes in; the table line still
    // counts them and their paragraphs.
    let md = "<!-- t0 2x3, cells p0-p5 by row -->\n|A|B|\n|-|-|\n|C|D|\n\n<!-- p6 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_cells_empty_paragraphs_come_back() {
    let md =
        "<!-- t0 1x2, cells t0.row0 p0-p4 -->\n|a<br><br>b|c<br>|\n|-|-|\n\n<!-- p5 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_tables_alignment_comes_back() {
    for md in [
        "<!-- t0 center 1x2, cells p0-p1 by row -->\n|a|b|\n|-|-|\n\n<!-- p2 -->\nAfter.\n",
        "<!-- t0 right 1x2, cells p0-p1 by row -->\n|a|b|\n|-|-|\n\n<!-- p2 -->\nAfter.\n",
    ] {
        assert_eq!(round_trip(md), md.trim());
    }
}

#[test]
fn an_inserted_row_stays_one_change_when_a_cell_holds_paragraphs() {
    // The row's insertion tags every cell's text, across the `<br>` that
    // ends a cell paragraph too.
    let body = "<!-- p0 -->\nBefore.\n\n\
                <!-- t0 2x2, cells t0.row0 p1-p3 t0.row1 p4-p5, break-ins r5 @AC in p1 -->\n\
                |{++A<br>B++}{>>r2 @AC<<}|{++C++}{>>r2 @AC<<}|\n|-|-|\n|D|E|\n\n<!-- p6 -->\nAfter.\n";
    assert_eq!(round_trip(&format!("{AUTHORS}{body}")), body.trim());
}

#[test]
fn a_nested_list_item_keeps_its_level_from_its_id_line() {
    // The 0.12 view printed the level on the id line (`level 1`) and no
    // indentation; that Markdown still reads, and comes back nested by
    // indentation, which then round-trips as it is.
    let flat = "<!-- p0 ListParagraph num \"1)\" -->\n1. One.\n\n\
                <!-- p1 ListParagraph num \"a)\", level 1 -->\n1. Under one.\n\n\
                <!-- p2 ListParagraph num \"i)\", level 2 -->\n1. Deeper.\n\n\
                <!-- p3 ListParagraph num \"b)\", level 1 -->\n2. Under one again.\n\n\
                <!-- p4 ListParagraph num \"2)\" -->\n2. Two.\n\n\
                <!-- p5 bullet, level 1 -->\n- A bullet under two.\n\n\
                <!-- p6 -->\nAfter.\n";
    let nested = "<!-- p0 ListParagraph num \"1)\" -->\n1. One.\n\n\
                  \x20  <!-- p1 ListParagraph num \"a)\" -->\n   1. Under one.\n\n\
                  \x20     <!-- p2 ListParagraph num \"i)\" -->\n      1. Deeper.\n\n\
                  \x20  <!-- p3 ListParagraph num \"b)\" -->\n   2. Under one again.\n\n\
                  <!-- p4 ListParagraph num \"2)\" -->\n2. Two.\n\n\
                  \x20  <!-- p5 bullet -->\n   - A bullet under two.\n\n\
                  <!-- p6 -->\nAfter.\n";
    assert_eq!(round_trip(flat), nested.trim());
    assert_eq!(round_trip(nested), nested.trim());
}

#[test]
fn sections_and_their_pages_come_back() {
    let header = "---\npage: A4 landscape, margins 0.5in, header/footer 0.49in\nsections:\n  \
                  2: {p1-p2, page: Letter portrait, margins top 1in, right 1.25in, bottom 1in, \
                  left 1.25in, header/footer 0.5in}\n---\n\n";
    let body = "<!-- p0 section-break -->\nOne.\n\n<!-- p1 -->\nTwo.\n\n<!-- p2 -->\nThree.\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{header}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    for line in [
        "page: A4 landscape, margins 0.5in, header/footer 0.49in",
        "  2: {p1-p2, page: Letter portrait, margins top 1in, right 1.25in, bottom 1in, left 1.25in, \
         header/footer 0.5in}",
    ] {
        assert!(view.lines().any(|l| l.starts_with(line)), "{line}\n{view}");
    }
    assert!(view.ends_with(body), "{view}");
}

#[test]
fn columns_and_their_gap_come_back() {
    // `columns N` and `column gap` are page clauses: the first section's
    // come back, and a later section's own page names its count.
    let header = "---\npage: Letter portrait, margins 1in, header/footer 0.5in, columns 2, \
                  column gap 0.25in\nsections:\n  \
                  2: {p1-p2, page: Letter portrait, margins 1in, header/footer 0.5in}\n---\n\n";
    let body = "<!-- p0 section-break -->\nOne.\n\n<!-- p1 -->\nTwo.\n\n<!-- p2 -->\nThree.\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{header}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    let xml = common::docx::part_string(&docx, "word/document.xml").unwrap();
    assert!(
        xml.contains("<w:cols w:space=\"360\" w:num=\"2\"/>"),
        "{xml}"
    );
    assert!(xml.contains("<w:cols w:space=\"720\"/>"), "{xml}");
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    for line in [
        "page: Letter portrait, margins 1in, header/footer 0.5in, columns 2, column gap 0.25in",
        "  2: {p1-p2, page: Letter portrait, margins 1in, header/footer 0.5in}",
    ] {
        assert!(view.lines().any(|l| l.starts_with(line)), "{line}\n{view}");
    }
}

/// The view of `header` and `body` read into a docx and read again.
fn view_round_trip(header: &str, body: &str) -> String {
    let docx = markdown_to_docx(
        &as_view(&format!("{header}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown
}

#[test]
fn headers_and_footers_come_back_with_their_ids() {
    let header = "---\nheaders:\n  \
                  first: {id: header3, text: \"{PAGE}1{/}\"}  # page 1 only (different first page)\n  \
                  default: {id: header2.p1, text: Smith & Co, center}\n  \
                  even: {id: header1, text: Even, inactive}  # defined, but even/odd headers are off\n\
                  footers:\n  default: {id: footer2, text: \"Page {PAGE}1{/}\", right}\n\
                  sections:\n  2: {p1-p1, footers: {default: {id: footer5, text: Annex}}}\n---\n\n";
    let body = "<!-- p0 section-break -->\nOne.\n\n<!-- p1 -->\nTwo.\n";
    let view = view_round_trip(header, body);
    for line in [
        "  first: {id: header3, text: \"{PAGE}1{/}\"}",
        "  default: {id: header2.p1, text: Smith & Co, center}",
        "  even: {id: header1, text: Even, inactive}",
        "  default: {id: footer2, text: \"Page {PAGE}1{/}\", right}",
        "  2: {p1-p1, footers: {default: {id: footer5, text: Annex}}}",
    ] {
        assert!(view.lines().any(|l| l.starts_with(line)), "{line}\n{view}");
    }
    assert!(view.ends_with(body), "{view}");
}

#[test]
fn a_substitution_by_two_authors_keeps_both_tags() {
    // The view tags a substitution's sides one by one when two authors
    // made them: the second tag is the insertion's, not a comment.
    let header = "---\nauthors:\n  AC: Ann Counsel                  # 2026-10-01T09:00:00Z\n  \
                  ASR: Arthur Souza Rodrigues      # 2026-10-09T16:13:00Z\n---\n\n";
    let body =
        "<!-- p0 -->\nThe {~~old~>new~~}{>>r0 @AC<<}{>>r1 @ASR<<} term {~~a~>b~~}{>>r2+3 @AC<<}.\n";
    assert_eq!(round_trip(&format!("{header}{body}")), body.trim());
}

#[test]
fn the_styles_legend_comes_back() {
    // The legend defines the default style and the headings', spacing,
    // keep-next and alignment included.
    let header = "---\nstyles:\n  \
                  Normal: Georgia 11pt, after 8pt, line 1.08, justify  # default; unannotated paragraphs use it\n  \
                  \"#\": Heading1, Arial bold 16pt, before 12pt, keep-next, center\n  \
                  \"##\": Heading2, Arial italic 13pt, before 2pt, line 12pt exact\n---\n\n";
    let body = "<!-- p0 -->\n# Title\n\n<!-- p1 -->\n## Part\n\n<!-- p2 -->\nText.\n";
    let view = view_round_trip(header, body);
    for line in [
        "  Normal: Georgia 11pt, after 8pt, line 1.08, justify",
        "  \"#\": Heading1, Arial bold 16pt, before 12pt, keep-next, center",
        "  \"##\": Heading2, Arial italic 13pt, before 2pt, line 12pt exact",
    ] {
        assert!(view.lines().any(|l| l.starts_with(line)), "{line}\n{view}");
    }
    assert!(view.ends_with(body), "{view}");
}

#[test]
fn a_heading_takes_the_style_the_legend_names_for_its_level() {
    let header = "---\nstyles:\n  \
                  \"1\": Arial 12pt, left            # default; unannotated paragraphs use it\n  \
                  \"#\": 2, Calibri bold 16pt, before 12pt, after 6pt\n---\n\n";
    // The printer names a heading style other than HeadingN on the id
    // line; the legend gives the level its style when the line does not.
    let body = "<!-- p0 -->\n# Title\n\n<!-- p1 -->\nText.\n";
    let view = view_round_trip(header, body);
    let body = "<!-- p0 2 -->\n# Title\n\n<!-- p1 -->\nText.\n";
    for line in [
        "  \"1\": Arial 12pt, left",
        "  \"#\": 2, Calibri bold 16pt, before 12pt, after 6pt",
    ] {
        assert!(view.lines().any(|l| l.starts_with(line)), "{line}\n{view}");
    }
    assert!(view.ends_with(body), "{view}");
}

#[test]
fn a_title_page_belongs_to_its_own_section() {
    // Word reads w:titlePg per section: section 1's does not reach 2.
    let header = "---\nheaders:\n  first: none\n  default: {id: header2, text: X}\n\
                  sections:\n  2: {p1-p1}\n---\n\n";
    let body = "<!-- p0 section-break -->\nOne.\n\n<!-- p1 -->\nTwo.\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{header}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    let document = common::docx::part_string(&docx, "word/document.xml").expect("a body");
    assert_eq!(document.matches("<w:titlePg/>").count(), 1, "{document}");
}

#[test]
fn a_later_sections_title_page_comes_back() {
    let header = "---\nheaders:\n  default: {id: header2, text: X}\n\
                  sections:\n  2: {p1-p1, headers: {first: none}}\n---\n\n";
    let body = "<!-- p0 section-break -->\nOne.\n\n<!-- p1 -->\nTwo.\n";
    let view = view_round_trip(header, body);
    assert!(
        view.lines().any(|l| {
            l.starts_with("  2: {p1-p1, headers: {first: none}, footers: {first: none}}")
        }),
        "{view}"
    );
}

#[test]
fn a_later_sections_first_story_is_per_kind() {
    // w:titlePg names each kind's first-page story: a first footer leaves
    // the first header `none`, and a first header leaves the footer's.
    let body = "<!-- p0 section-break -->\nOne.\n\n<!-- p1 -->\nTwo.\n";
    let header = "---\nheaders:\n  default: {id: header2, text: X}\n\
                  sections:\n  2: {p1-p1, footers: {first: {id: footer3, text: F}}}\n---\n\n";
    let view = view_round_trip(header, body);
    assert!(
        view.lines().any(|l| l.starts_with(
            "  2: {p1-p1, headers: {first: none}, footers: {first: {id: footer3, text: F}}}"
        )),
        "{view}"
    );
    let header = "---\nheaders:\n  default: {id: header2, text: X}\n\
                  sections:\n  2: {p1-p1, headers: {first: {id: header3, text: H}}}\n---\n\n";
    let view = view_round_trip(header, body);
    assert!(
        view.lines().any(|l| l.starts_with(
            "  2: {p1-p1, headers: {first: {id: header3, text: H}}, footers: {first: none}}"
        )),
        "{view}"
    );
}

#[test]
fn a_hash_inside_quotes_is_no_yaml_comment() {
    let header = "---\nheaders:\n  default: {id: header2, text: \"Totals  #3\", center}\n---\n\n";
    let view = view_round_trip(header, "<!-- p0 -->\nOne.\n");
    assert!(
        view.lines()
            .any(|l| l.starts_with("  default: {id: header2, text: \"Totals  #3\", center}")),
        "{view}"
    );
}

#[test]
fn a_span_cut_at_the_right_edge_keeps_the_cells_after_it() {
    // The view drops a span's empty columns at the right too; the cells
    // past the span still come back.
    let md = "<!-- t0 2x5, cells p0-p7 by row, merged t0.row0.cell0 across 3 -->\n\
              |A|\n|-|\n|B|\n\n<!-- p8 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn table_column_alignment_comes_back() {
    // The delimiter row's colons align a column's cells; Word holds them as
    // each cell paragraph's `w:jc`, and the reader gives them back. A
    // column with no alignment stays `-`.
    let view = round_trip("| L | C | R | N |\n| :--- | :---: | ---: | --- |\n| a | b | c | d |\n");
    assert!(view.contains("|:-|:-:|-:|-|"), "{view}");
}

#[test]
fn a_cell_aligned_against_its_column_is_named_on_the_table_line() {
    // Most of a column decides its marker; a cell paragraph that differs
    // is an exception on the table line (`p4 center`), and it comes back.
    let source = "<!-- t0 3x2, cells p0-p5 by row, p4 center -->\n\
                  |A|B|\n|:-|-:|\n|a|b|\n|c|d|\n";
    let view = round_trip(source);
    assert!(view.contains("|:-|-:|"), "{view}");
    assert!(view.contains("by row, p4 center"), "{view}");
    // A tie prints `-`, and the cell that is not left is named; left and
    // no alignment look alike, so neither is an exception to the other.
    let tie = round_trip("<!-- t0 2x1, cells p0-p1 by row, p1 center -->\n|A|\n|:-|\n|a|\n");
    assert!(tie.contains("|-|"), "{tie}");
    assert!(tie.contains("by row, p1 center"), "{tie}");
}

/// The `w:ins` / `w:del` marks of `document.xml`, in order: `ins1 del2`.
fn marks(docx: &[u8]) -> Vec<String> {
    let document = common::docx::part_string(docx, "word/document.xml").expect("document.xml");
    let mut out = Vec::new();
    for (at, _) in document.match_indices("<w:") {
        let rest = &document[at + 3..];
        for kind in ["ins", "del"] {
            if let Some(attrs) = rest.strip_prefix(kind).filter(|r| r.starts_with(' ')) {
                let id = attrs
                    .split_once("w:id=\"")
                    .and_then(|(_, r)| r.split_once('"'))
                    .map_or("", |(id, _)| id);
                out.push(format!("{kind}{id}"));
            }
        }
    }
    out
}

#[test]
fn a_substitution_keeps_each_id_on_its_side_and_the_insertion_first() {
    // Word numbers marks in document order and writes the insertion first,
    // so the tag's deleted ids (`3+4`) run until the numbers drop back to
    // the inserted ones (`1+2`); a side whose text is one run splits to
    // take them all. Links on either side move with their text.
    for (body, want) in [
        (
            "{~~CC DD~>AA BB ~~}{>>r3+4+1+2 @AC<<}",
            &["ins1", "ins2", "del3", "del4"][..],
        ),
        ("{~~BBCC~>AA~~}{>>r2+3+1 @AC<<}", &["ins1", "del2", "del3"]),
        (
            "{~~[old](https://example.com/)~>[new](https://example.com/)~~}{>>r2+1 @AC<<}",
            &["ins1", "del2"],
        ),
    ] {
        let view = format!("{AUTHORS}<!-- p0 -->\nA {body} z.\n");
        let docx = markdown_to_docx(&as_view(&view), &DocxOptions::default())
            .expect("the view reads")
            .docx;
        assert_word_valid_package(&docx);
        assert_eq!(marks(&docx), want, "{body}");
        let again = format!("<!-- p0 -->\nA {body} z.");
        assert_eq!(round_trip(&view), again, "{body}");
    }
}

#[test]
fn a_bookmark_between_two_changes_stays_between_them() {
    // `<a id>` between an insertion and a deletion: two changes, and the
    // bookmark is in neither mark.
    let body = "<!-- p0 -->\nK {++new++}{>>r1 @AC<<}<a id=\"mark\"></a>{--old--}{>>r2 @AC<<} \
                [go](#mark)";
    assert_eq!(round_trip(&format!("{AUTHORS}{body}\n")), body);
}

#[test]
fn a_dated_range_puts_its_last_day_on_the_mark_written_last() {
    // `first..last` comes back as the first mark's first day and the last
    // mark's last day; a substitution writes its insertion first.
    let view = "---\nauthors:\n  AL: Ann Lee                      # 2 revisions, 2026-03-02..2026-03-04\n---\n\n\
                <!-- p0 -->\nA {~~old~>new~~}{>>r2+1 @AL<<} z.\n";
    let docx = markdown_to_docx(&as_view(view), &DocxOptions::default())
        .expect("the view reads")
        .docx;
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    for mark in [
        "<w:ins w:id=\"1\" w:author=\"Ann Lee\" w:date=\"2026-03-02T00:00:00Z\"",
        "<w:del w:id=\"2\" w:author=\"Ann Lee\" w:date=\"2026-03-04T23:59:59Z\"",
    ] {
        assert!(document.contains(mark), "{mark} in {document}");
    }
}

#[test]
fn a_note_inserted_or_deleted_with_its_reference_keeps_words_shape() {
    // Word marks such a note's paragraph mark (`footnotes.r11`) and wraps
    // its reference mark and text in one change (`footnotes.r12`); the
    // view prints the note in its reference's change and names both.
    let body = "<!-- p0 -->\nAdded{++[^1]++}{>>r10 @AC<<} and cut{--[^2]--}{>>r20 @AC<<}.\n\n\
                <!-- footnotes.p0 -->\n\
                {++[^1]: An inserted note.++}{>>r10 @AC<<} <!-- rev footnotes.r11 @AC; footnotes.r12 @AC -->\n\
                <!-- footnotes.p1 -->\n\
                {--[^2]: A deleted note.--}{>>r20 @AC<<} <!-- rev-del footnotes.r21 @AC; footnotes.r22 @AC -->";
    let docx = markdown_to_docx(
        &as_view(&format!("{AUTHORS}{body}\n")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    let notes = common::docx::part_string(&docx, "word/footnotes.xml").expect("footnotes.xml");
    for (kind, mark, run, text) in [
        ("ins", 11, 12, "<w:t xml:space=\"preserve\">"),
        ("del", 21, 22, "<w:delText xml:space=\"preserve\">"),
    ] {
        let paragraph_mark = format!("<w:rPr><w:{kind} w:id=\"{mark}\" w:author=\"Ann Counsel\"");
        assert!(
            notes.contains(&paragraph_mark),
            "{paragraph_mark} in {notes}"
        );
        let wrap = notes
            .split_once(&format!("<w:{kind} w:id=\"{run}\" "))
            .map(|(_, rest)| {
                rest.split_once(&format!("</w:{kind}>"))
                    .map_or(rest, |(inside, _)| inside)
            })
            .unwrap_or_else(|| panic!("{kind} {run} in {notes}"));
        assert!(
            wrap.contains("Ref/>"),
            "the reference mark in {kind} {run}: {wrap}"
        );
        assert!(
            wrap.contains(text) && wrap.contains("note."),
            "the text in {kind} {run}: {wrap}"
        );
    }
    assert_eq!(round_trip(&format!("{AUTHORS}{body}\n")), body);
}

#[test]
fn a_tracked_note_reads_with_one_space_after_its_label() {
    // Word's text after the reference mark starts with a space (` An
    // inserted note.`): the label's own space stands for it, as in an
    // untracked note.
    let docx = std::fs::read("tests/fixtures/md-roundtrip/edges/edge-notes-tracked.docx")
        .expect("the edge fixture");
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    for line in [
        "{++[^1]: An inserted note.++}{>>r10 @AL<<} <!-- rev footnotes.r11 @AL; footnotes.r12 @AL -->",
        "{--[^2]: A deleted note.--}{>>r20 @AL<<} <!-- rev-del footnotes.r21 @AL; footnotes.r22 @AL -->",
    ] {
        assert!(view.contains(line), "{line} in {view}");
    }
}

#[test]
fn a_note_of_two_paragraphs_keeps_them() {
    // The view joins a note's paragraphs with `<br>` when its id line
    // counts them; a note in its reference's change names every mark of its
    // own after it, paragraph marks too.
    let docx = std::fs::read("tests/fixtures/md-roundtrip/edges/edge-notes-tracked.docx")
        .expect("the edge fixture");
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    let line = "<!-- footnotes.p2-p3 -->\n{++[^3]: Two paragraphs,<br>inserted.++}{>>r30 @AL<<} \
                <!-- rev footnotes.r31 @AL; footnotes.r32 @AL; footnotes.r33 @AL; footnotes.r34 @AL -->";
    assert!(view.contains(line), "{line} in {view}");
    for body in [
        "<!-- p0 -->\nSee[^1].\n\n<!-- footnotes.p0-p1 -->\n[^1]: One,<br>two.",
        "<!-- p0 -->\nSee[^1].\n\n<!-- footnotes.p0 -->\n[^1]: One,<br/>broken.",
    ] {
        let docx = markdown_to_docx(
            &as_view(&format!("{AUTHORS}{body}\n")),
            &DocxOptions::default(),
        )
        .expect("the view reads")
        .docx;
        let notes = common::docx::part_string(&docx, "word/footnotes.xml").expect("footnotes.xml");
        // The note is the part's last, after Word's separators.
        let note = notes
            .rsplit_once("<w:footnote ")
            .map_or("", |(_, note)| note);
        let paragraphs = note.matches("<w:p>").count() + note.matches("<w:p ").count();
        let want = if body.contains("p0-p1") { 2 } else { 1 };
        assert_eq!(paragraphs, want, "{body}: {notes}");
        assert_eq!(round_trip(&format!("{AUTHORS}{body}\n")), body);
    }
}

#[test]
fn gaps_and_rows_past_the_grid_come_back_as_the_table_line_says() {
    // A row's gaps are written as the table line counts them, even where
    // the view shows no empty cell for them; an empty cell past the grid's
    // columns is the view's padding, not a cell.
    for table in [
        // A full row with a grid column after it.
        "<!-- t0 2x2, cells p1-p4 by row, gaps t0.row1 after 1 -->\n|a|b|\n|-|-|\n|c|d|",
        // Rows wider and narrower than the grid.
        "<!-- t0 3x3, cells p1-p10 by row, gaps t0.row1 after 1 -->\n|a|b|c|||\n|-|-|-|-|-|\n|d|e||||\n|f|g|h|i|j|",
        "<!-- t0 2x2, cells p1-p5 by row, gaps t0.row1 before 1 -->\n|a|b|||\n|-|-|-|-|\n||x|y|z|",
        // Empty rows: a span and a gap the view has no cell for.
        "<!-- t0 3x3, cells p1-p6 by row, merged t0.row1.cell1 across 2, gaps t0.row2 before 2 -->\n||\n|-|\n||\n||",
    ] {
        let body = format!(
            "<!-- p0 -->\nBefore.\n\n{table}\n\n<!-- p{} -->\nAfter.",
            last_paragraph(table) + 1
        );
        assert_eq!(round_trip(&format!("{body}\n")), body, "{table}");
    }
}

/// The last paragraph a table line's `cells pA-pB` counts.
fn last_paragraph(table: &str) -> usize {
    let cells = table.split_once("cells p").map_or("", |(_, rest)| rest);
    let range = cells.split_once(' ').map_or(cells, |(range, _)| range);
    range
        .rsplit_once("-p")
        .and_then(|(_, last)| last.trim_end_matches(',').parse().ok())
        .unwrap_or(0)
}

#[test]
fn a_note_whose_own_marks_the_view_does_not_name_keeps_its_text_unmarked() {
    // A reference deleted with no mark on its note's text (`{--[^1]: …--}`
    // with no `rev` after it): the note keeps its text, and the body's mark
    // keeps the reference's id.
    let body = "<!-- p0 -->\nBefore{--[^1]--}{>>r1 @AC<<} after.\n\n<!-- footnotes.p0 -->\n\
                {--[^1]: Still here.--}{>>r1 @AC<<}";
    let docx = markdown_to_docx(
        &as_view(&format!("{AUTHORS}{body}\n")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    let notes = common::docx::part_string(&docx, "word/footnotes.xml").expect("footnotes.xml");
    assert!(!notes.contains("<w:del "), "{notes}");
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    assert!(document.contains("<w:del w:id=\"1\" "), "{document}");
    assert_eq!(round_trip(&format!("{AUTHORS}{body}\n")), body);
}

#[test]
fn a_note_referenced_twice_is_one_note() {
    let body = "<!-- p0 -->\nSee[^1] and again[^1].\n\n<!-- footnotes.p0 -->\n[^1]: Once.";
    let docx = markdown_to_docx(&as_view(&format!("{body}\n")), &DocxOptions::default())
        .expect("the view reads")
        .docx;
    let notes = common::docx::part_string(&docx, "word/footnotes.xml").expect("footnotes.xml");
    assert_eq!(notes.matches("Once.").count(), 1, "{notes}");
    assert_eq!(round_trip(&format!("{body}\n")), body);
}

#[test]
fn a_table_in_a_note_keeps_its_text() {
    // The view has no table in a note: its cells' paragraphs are the
    // note's, joined as its other paragraphs are, and its id line counts
    // them all.
    let docx = std::fs::read("tests/fixtures/md-roundtrip/edges/edge-notes-table.docx")
        .expect("the edge fixture");
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    let line = "<!-- footnotes.p0-p5 -->\n[^1]: Note<br>fn-a<br>fn-b<br>fn-c<br>fn-d<br>After.";
    assert!(view.contains(line), "{line} in {view}");
}

#[test]
fn a_dated_range_survives_on_comments_and_paragraph_marks() {
    // The range needs two dated things to tell its ends apart, whatever
    // they are: two comments, or two paragraph marks.
    for (authors, body) in [
        (
            "AC: Ann Counsel                  # 2 comments, 2026-03-02..2026-03-04",
            "<!-- p0 -->\n{==One==}{>>c0 @AC: first<<} and {==two==}{>>c1 @AC: second<<}.",
        ),
        (
            "AC: Ann Counsel                  # 2 revisions, 2026-03-02..2026-03-04",
            "<!-- p0 break-ins r1 @AC -->\nOne.\n\n<!-- p1 break-ins r2 @AC -->\nTwo.\n\n<!-- p2 -->\nThree.",
        ),
    ] {
        let view = format!("---\nauthors:\n  {authors}\n---\n\n{body}\n");
        let docx = markdown_to_docx(&as_view(&view), &DocxOptions::default())
            .expect("the view reads")
            .docx;
        let again = read(
            &docx,
            &ReadOptions {
                page_markers: false,
                ..ReadOptions::default()
            },
        )
        .expect("the docx reads")
        .markdown;
        assert!(
            again.contains("2026-03-02..2026-03-04"),
            "{authors}: {again}"
        );
    }
}

#[test]
fn a_comment_inside_a_substitution_side_keeps_the_pair() {
    // The comment's range sits in the old text: the writer still writes the
    // insertion first, and the view pairs the two sides again.
    let body = "<!-- p0 -->\nA {~~aa {==bb==}{>>c0 @AC: Inside.<<} cc~>NEW~~}{>>r3+1 @AC<<} z.";
    let view = format!("{AUTHORS}{body}\n");
    let docx = markdown_to_docx(&as_view(&view), &DocxOptions::default())
        .expect("the view reads")
        .docx;
    assert_word_valid_package(&docx);
    let ids = marks(&docx);
    assert_eq!(ids.first().map(String::as_str), Some("ins1"), "{ids:?}");
    let again = round_trip(&view);
    assert!(again.contains("{~~aa {==bb==}"), "{again}");
    assert!(again.contains("~>NEW~~}"), "{again}");
}

#[test]
fn a_heading_slug_bookmark_between_marks_keeps_the_pair_and_its_count() {
    // The heading's own bookmark prints no anchor, so the text pairs the
    // marks around it; the `revisions:` count must say the same.
    let mark = |kind: &str, id: u32, text: &str| {
        let t = if kind == "del" { "delText" } else { "t" };
        format!(
            "<w:{kind} w:id=\"{id}\" w:author=\"Ann Lee\" w:date=\"2026-03-02T00:00:00Z\">\
             <w:r><w:{t}>{text}</w:{t}></w:r></w:{kind}>"
        )
    };
    let body = format!(
        "<w:p><w:pPr><w:outlineLvl w:val=\"0\"/></w:pPr>{}\
         <w:bookmarkStart w:id=\"0\" w:name=\"hello\"/><w:bookmarkEnd w:id=\"0\"/>{}</w:p>\
         <w:p><w:hyperlink w:anchor=\"hello\"><w:r><w:t>the heading</w:t></w:r></w:hyperlink></w:p>",
        mark("ins", 54, "Hello"),
        mark("del", 55, "Old"),
    );
    let view = read(
        &common::docx::docx(&body),
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    assert!(
        view.contains("# {~~Old~>Hello~~}{>>r55+54 @AL<<}"),
        "{view}"
    );
    assert!(view.contains("1 change: 1 substitution"), "{view}");
}

#[test]
fn a_run_that_prints_nothing_keeps_no_two_marks_apart() {
    // An empty run, a vanished run and a page break between an insertion
    // and a deletion: the text pairs them, and so does the count.
    let mark = |kind: &str, id: u32, text: &str| {
        let t = if kind == "del" { "delText" } else { "t" };
        format!(
            "<w:{kind} w:id=\"{id}\" w:author=\"Ann Lee\" w:date=\"2026-03-02T00:00:00Z\">\
             <w:r><w:{t}>{text}</w:{t}></w:r></w:{kind}>"
        )
    };
    let between = [
        "<w:r><w:rPr><w:b/></w:rPr></w:r>",
        "<w:r><w:rPr><w:vanish/></w:rPr><w:t>HID</w:t></w:r>",
        "<w:r><w:br w:type=\"page\"/></w:r>",
    ];
    let body: String = between
        .iter()
        .enumerate()
        .map(|(n, run)| {
            let id = 10 + 2 * n as u32;
            format!(
                "<w:p><w:r><w:t xml:space=\"preserve\">A </w:t></w:r>{}{run}{}</w:p>",
                mark("ins", id, "new"),
                mark("del", id + 1, "old"),
            )
        })
        .collect();
    let view = read(
        &common::docx::docx(&body),
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    assert_eq!(view.matches("{~~old~>new~~}").count(), 3, "{view}");
    assert!(
        view.contains("6 marks shown as 3 changes: 3 substitutions"),
        "{view}"
    );
}

#[test]
fn a_header_substitution_pairs_and_keeps_each_id_on_its_side() {
    // As in the body: Word's insertion then deletion is one substitution,
    // marks of one kind by one author join, and writing it back puts every
    // id on the side it came from.
    let mark = |kind: &str, id: u32, text: &str| {
        let t = if kind == "del" { "delText" } else { "t" };
        format!(
            "<w:{kind} w:id=\"{id}\" w:author=\"Ann Lee\" w:date=\"2026-03-02T00:00:00Z\">\
             <w:r><w:{t}>{text}</w:{t}></w:r></w:{kind}>"
        )
    };
    let text = |t: &str| format!("<w:r><w:t xml:space=\"preserve\">{t}</w:t></w:r>");
    let header = format!(
        "<w:hdr xmlns:w=\"{}\"><w:p>{}{}{}{}</w:p><w:p>{}{}{}{}{}{}</w:p></w:hdr>",
        common::docx::W_NS,
        text("Sub "),
        mark("ins", 90, "new"),
        mark("del", 91, "old"),
        text("."),
        text("Runs "),
        mark("ins", 92, "A"),
        mark("ins", 93, "B"),
        mark("del", 94, "C"),
        mark("del", 95, "D"),
        text("."),
    );
    let docx = common::docx::docx_with_sect(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[common::docx::Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    let view = read(&docx, &options).expect("the docx reads").markdown;
    for want in [
        "Sub {~~old~>new~~}{>>header1.r91+90 @AL<<}.",
        "Runs {~~CD~>AB~~}{>>header1.r94+95+92+93 @AL<<}.",
    ] {
        assert!(view.contains(want), "{want} in {view}");
    }
    let written = markdown_to_docx(&view, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    assert_word_valid_package(&written);
    let part = common::docx::part_string(&written, "word/header1.xml").expect("header1.xml");
    let order: Vec<&str> = ["\"90\"", "\"91\"", "\"92\"", "\"93\"", "\"94\"", "\"95\""]
        .into_iter()
        .filter(|id| part.contains(&format!("w:id={id}")))
        .collect();
    assert_eq!(order.len(), 6, "{part}");
    for (ins, del) in [("90", "91"), ("93", "94")] {
        let at = |id: &str| part.find(&format!("w:id=\"{id}\"")).expect("the id");
        assert!(
            at(ins) < at(del),
            "insertion {ins} before deletion {del} in {part}"
        );
    }
    let again = read(&written, &options).expect("the docx reads").markdown;
    let headers = |v: &str| {
        v.lines()
            .filter(|l| l.contains("header1."))
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    assert_eq!(headers(&again), headers(&view));
}

/// The whole view `markdown` reads into, read again, without its
/// `source:` and `sha256:` lines; and the docx it wrote.
fn full_trip(markdown: &str) -> (String, Vec<u8>) {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    assert_word_valid_package(&docx);
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    let kept: Vec<&str> = view
        .lines()
        .filter(|l| !l.starts_with("source: ") && !l.starts_with("sha256: "))
        .collect();
    (kept.join("\n"), docx)
}

#[test]
fn a_footer_keeps_its_paragraph_marks_its_fields_and_its_literal_text() {
    // A mark on a trailing empty paragraph, two marks on one paragraph
    // mark, a deleted field, text that reads like a field code, and text
    // that reads like a mark.
    let view = "---\nview: tracked\nauthors:\n  AL: Ann Lee                      # 2026-03-02T10:00:00Z\n  \
                BC: Bo Chen                      # 2026-03-02T10:00:00Z\n\
                headers:\n  default: {id: header1, text: \"Text with {\\\\++literal++\\\\} braces and \\\\{PAGE} too\"}\n\
                footers:\n  default: {id: footer1, text: [\"Page {--{PAGE}1{/}--}{>>footer1.r31 @AL<<}\", \"\"], \
                rev: break-ins footer1.r60 @AL in footer1; break-del footer1.r61 @BC in footer1; \
                break-del footer1.r40 @BC in footer1.p1}\n---\n\n<!-- p0 -->\nBody.\n";
    let (again, docx) = full_trip(view);
    for line in [
        "  default: {id: header1, text: \"Text with {\\\\++literal++\\\\} braces and \\\\{PAGE} too\"}",
        "  default: {id: footer1, text: [\"Page {--{PAGE}1{/}--}{>>footer1.r31 @AL<<}\", \"\"], \
         rev: break-ins footer1.r60 @AL in footer1; break-del footer1.r61 @BC in footer1; \
         break-del footer1.r40 @BC in footer1.p1}",
    ] {
        assert!(again.contains(line), "{line} in {again}");
    }
    let header = common::docx::part_string(&docx, "word/header1.xml").expect("header1.xml");
    assert!(
        header.contains(">Text with {++literal++} braces and {PAGE} too<"),
        "{header}"
    );
    assert!(!header.contains("fldSimple"), "{header}");
}

#[test]
fn a_footer_field_without_a_result_comes_back_without_one() {
    // kitchen-sink.md: `{PAGE}` came back `{PAGE}1{/}`, the writer's
    // made-up result. Word computes both on every page ("Page 1 of 2").
    let view = "---\nview: tracked\n\
                footers:\n  default: {id: footer1, text: \"Page {PAGE} of {NUMPAGES}\"}\n\
                ---\n\n<!-- p0 -->\nBody.\n";
    let (again, docx) = full_trip(view);
    assert!(
        again.contains("  default: {id: footer1, text: \"Page {PAGE} of {NUMPAGES}\"}"),
        "{again}"
    );
    let footer = common::docx::part_string(&docx, "word/footer1.xml").expect("footer1.xml");
    assert!(
        footer.contains(r#"<w:fldSimple w:instr=" PAGE "/>"#),
        "{footer}"
    );
    assert!(!footer.contains("<w:t>1</w:t>"), "{footer}");
}

#[test]
fn a_mark_with_no_author_keeps_its_id_and_takes_no_comment() {
    // `@??`: the reader's handle for a mark without an author's name. It
    // is a tag, not a comment, in the body and in a header.
    let view = "---\nview: tracked\nheaders:\n  default: {id: header1, text: \"Keep {++new++}{>>header1.r3 @??<<}\"}\n\
                ---\n\n<!-- p0 -->\nKeep {++new++}{>>r5 @??<<} it.\n";
    let (again, docx) = full_trip(view);
    assert!(again.contains("Keep {++new++}{>>r5 @??<<} it."), "{again}");
    assert!(
        again.contains("text: \"Keep {++new++}{>>header1.r3 @??<<}\""),
        "{again}"
    );
    assert!(again.contains("comments: 0"), "{again}");
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    assert!(
        document.contains("<w:ins w:id=\"5\" w:author=\"\""),
        "{document}"
    );
}

#[test]
fn a_table_style_whose_borders_vary_warns_that_they_are_not_written() {
    let view = "---\nview: tracked\nstyles:\n  table: Ledger, borders vary\n---\n\n\
                <!-- t0 1x2, cells p0-p1 by row -->\n|a|b|\n|-|-|\n";
    let out = markdown_to_docx(view, &DocxOptions::default()).expect("the view reads");
    assert!(
        out.warnings
            .iter()
            .any(|w| w.starts_with("table: Ledger, borders vary: the view does not say")),
        "{:?}",
        out.warnings
    );
    let plain = "---\nview: tracked\nstyles:\n  table: Ledger\n---\n\n\
                 <!-- t0 1x2, cells p0-p1 by row -->\n|a|b|\n|-|-|\n";
    let out = markdown_to_docx(plain, &DocxOptions::default()).expect("the view reads");
    assert!(
        !out.warnings.iter().any(|w| w.contains("borders vary")),
        "{:?}",
        out.warnings
    );
}

/// The view a note `[^1]` with `text` and id line `ids` writes and reads.
fn note_trip(ids: &str, text: &str) -> String {
    let view = format!(
        "---\nview: tracked\nauthors:\n  AL: Ann Lee                      # 2026-03-02T10:00:00Z\n---\n\n\
         <!-- p0 -->\nSee[^1].\n\n<!-- {ids} -->\n[^1]: {text}\n"
    );
    full_trip(&view).0
}

#[test]
fn a_note_keeps_its_line_break_apart_from_its_inserted_paragraph() {
    // `<br/>` is a w:br in the paragraph; `<br>` ends it. The inserted
    // paragraph is one change, its mark untracked.
    let again = note_trip(
        "footnotes.p0-p1",
        "Keep<br/>this<br>{++added++}{>>footnotes.r2 @AL<<}",
    );
    assert!(
        again
            .lines()
            .any(|l| l == "[^1]: Keep<br/>this<br>{++added++}{>>footnotes.r2 @AL<<}"),
        "{again}"
    );
    assert!(again.contains("<!-- footnotes.p0-p1 -->"), "{again}");
}

#[test]
fn a_note_whose_first_paragraph_holds_only_its_reference_keeps_it() {
    let again = note_trip("footnotes.p0-p2", "<br>Body of the note.<br>Second line.");
    assert!(
        again.contains("[^1]:<br>Body of the note.<br>Second line."),
        "{again}"
    );
}

#[test]
fn a_deleted_reference_leaves_its_live_note_its_own_marks() {
    let view = "---\nview: tracked\nauthors:\n  AL: Ann Lee                      # 2026-03-02T10:00:00Z\n---\n\n\
                <!-- p0 -->\nCut{--[^1]--}{>>r1 @AL<<}.\n\n<!-- footnotes.p0 -->\n\
                [^1]: Still {++here++}{>>footnotes.r5 @AL<<}.\n";
    let (again, _) = full_trip(view);
    assert!(again.contains("Cut{--[^1]--}{>>r1 @AL<<}."), "{again}");
    assert!(
        again.contains("[^1]: Still {++here++}{>>footnotes.r5 @AL<<}."),
        "{again}"
    );
}

#[test]
fn a_note_whose_breaks_disagree_with_its_id_line_warns() {
    let view = "---\nview: tracked\n---\n\n<!-- p0 -->\nSee[^1].\n\n\
                <!-- footnotes.p0-p2 -->\n[^1]: One<br>two.\n";
    let out = markdown_to_docx(view, &DocxOptions::default()).expect("the view reads");
    assert!(
        out.warnings.iter().any(|w| w
            == "[^1]: its id line counts 3 paragraphs and its text 2 by its <br>s: \
                the <br>s are written as line breaks"),
        "{:?}",
        out.warnings
    );
}

#[test]
fn a_footnote_and_an_endnote_sharing_a_label_stay_two_notes() {
    // The label's first reference names its first definition; the next
    // reference the next one, which takes a number of its own.
    let view = "---\nview: tracked\n---\n\n<!-- p0 -->\nFoot[^1] and end[^1].\n\n\
                <!-- footnotes.p0 -->\n[^1]: foot text\n\n<!-- endnotes.p0 -->\n[^1]: end text\n";
    let (again, _) = full_trip(view);
    for line in [
        "Foot[^1] and end[^2].",
        "<!-- footnotes.p0 -->",
        "[^1]: foot text",
        "<!-- endnotes.p0 -->",
        "[^2]: end text",
    ] {
        assert!(again.lines().any(|l| l == line), "no {line:?} in\n{again}");
    }
    // Written the other way round, the endnote is referenced first.
    let view = "---\nview: tracked\n---\n\n<!-- p0 -->\nEnd[^1] and foot[^1].\n\n\
                <!-- endnotes.p0 -->\n[^1]: end text\n\n<!-- footnotes.p0 -->\n[^1]: foot text\n";
    let (again, _) = full_trip(view);
    assert!(
        again.contains("<!-- endnotes.p0 -->\n[^1]: end text"),
        "{again}"
    );
    assert!(
        again.contains("<!-- footnotes.p0 -->\n[^2]: foot text"),
        "{again}"
    );
}

#[test]
fn a_label_defined_once_and_referenced_twice_is_one_note() {
    let view = "---\nview: tracked\n---\n\n<!-- p0 -->\nOne[^1] and again[^1].\n\n\
                <!-- footnotes.p0 -->\n[^1]: the note\n";
    let (again, _) = full_trip(view);
    assert!(again.contains("One[^1] and again[^1]."), "{again}");
    assert!(!again.contains("[^2]"), "{again}");
}

/// The plain Markdown `markdown` written and read back (no view), with
/// the writer's warnings.
fn plain_round_trip(markdown: &str) -> (String, Vec<String>) {
    let written = markdown_to_docx(markdown, &DocxOptions::default()).expect("writes");
    assert_word_valid_package(&written.docx);
    let view = read(
        &written.docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    (view, written.warnings)
}

/// A footnote label names its definition whatever its case, as the
/// Markdown parser matches them (round-7 notes review, F1).
#[test]
fn a_note_label_names_its_definition_whatever_its_case() {
    let (view, _) = plain_round_trip("Text[^Note] here.\n\n[^note]: the real definition\n");
    assert!(view.contains("the real definition"), "{view}");
    let (view, warnings) =
        plain_round_trip("A[^One] B[^ONE] C[^one].\n\n[^one]: first\n\n[^One]: second\n");
    let notes: Vec<&str> = view.lines().filter(|l| l.starts_with("[^")).collect();
    assert_eq!(notes, ["[^1]: first", "[^2]: second"], "{view}");
    assert!(view.contains("A[^1] B[^2] C[^1]."), "{view}");
    assert!(
        warnings.iter().any(|w| w.contains("is defined 2 times")),
        "{warnings:?}"
    );
}

/// A reference inside a note's text is text: Word holds no note inside a
/// note, and it takes none of the label's definitions (F2).
#[test]
fn a_reference_inside_a_note_is_text_and_takes_no_definition() {
    let (view, warnings) =
        plain_round_trip("[^1]: first, see [^1]\n\n[^1]: second\n\nA[^1] and B[^1].\n");
    assert!(view.contains("A[^1] and B[^2]."), "{view}");
    assert!(view.contains("[^1]: first, see"), "{view}");
    assert!(view.contains("[^2]: second"), "{view}");
    assert!(
        warnings.iter().any(|w| w.contains("a note holds no note")),
        "{warnings:?}"
    );
}

/// A notes part (`footnotes.xml`, `endnotes.xml`) holding only Word's
/// separators: the `notes:` line names it, and the writer makes it again.
fn notes_part<'a>(kind: &str, notes: &str, xml: &'a mut String) -> common::docx::Part<'a> {
    *xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{kind}s xmlns:w="{}"><w:{kind} w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:{kind}><w:{kind} w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:{kind}>{notes}</w:{kind}s>"#,
        common::docx::W_NS
    );
    let (content_type, rel_type) = if kind == "footnote" {
        (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
        )
    } else {
        (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes",
        )
    };
    common::docx::Part {
        name: if kind == "footnote" {
            "word/footnotes.xml"
        } else {
            "word/endnotes.xml"
        },
        content_type,
        rel_type,
        xml: xml.as_str(),
    }
}

/// The view of `docx`, the docx that view writes, and that docx's view;
/// both views without their `sha256:` line, which names the bytes.
fn docx_round_trip(docx: &[u8]) -> (String, Vec<u8>, String) {
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
    assert_word_valid_package(&again);
    let view_again = read(&again, &options).expect("the docx reads").markdown;
    (without_sha(view), again, without_sha(view_again))
}

fn has_part(docx: &[u8], name: &str) -> bool {
    zip::ZipArchive::new(std::io::Cursor::new(docx))
        .unwrap()
        .by_name(name)
        .is_ok()
}

#[test]
fn a_notes_part_that_holds_only_separators_comes_back() {
    let (mut foot, mut end) = (String::new(), String::new());
    let parts = [
        notes_part("footnote", "", &mut foot),
        notes_part("endnote", "", &mut end),
    ];
    let docx = common::docx::docx_with(&common::docx::para("Body."), &parts);
    let (view, again, view_again) = docx_round_trip(&docx);
    assert!(
        view.contains("\nnotes: {empty: [footnotes, endnotes]}"),
        "{view}"
    );
    assert!(has_part(&again, "word/footnotes.xml"));
    assert!(has_part(&again, "word/endnotes.xml"));
    assert_eq!(view_again, view);

    // A note in the body makes its part; only the other is named.
    let note = r#"<w:footnote w:id="1"><w:p><w:r><w:t>A note.</w:t></w:r></w:p></w:footnote>"#;
    let parts = [
        notes_part("footnote", note, &mut foot),
        notes_part("endnote", "", &mut end),
    ];
    let body = r#"<w:p><w:r><w:t>Body.</w:t></w:r><w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="1"/></w:r></w:p>"#;
    let docx = common::docx::docx_with(body, &parts);
    let (view, again, view_again) = docx_round_trip(&docx);
    assert!(view.contains("\nnotes: {empty: [endnotes]}"), "{view}");
    assert!(has_part(&again, "word/footnotes.xml"));
    assert!(has_part(&again, "word/endnotes.xml"));
    assert_eq!(view_again, view);

    // A document without notes parts gains none.
    let docx = common::docx::docx(&common::docx::para("Body."));
    let (view, again, _) = docx_round_trip(&docx);
    assert!(!view.contains("\nnotes:"), "{view}");
    assert!(!has_part(&again, "word/footnotes.xml"));
    assert!(!has_part(&again, "word/endnotes.xml"));
}

#[test]
fn a_notes_part_with_an_unreferenced_note_is_not_empty() {
    // No reference reaches footnote 1: the view does not show it, but the
    // part holds it, so the view must not call the part empty.
    let (mut foot, mut end) = (String::new(), String::new());
    let orphan = r#"<w:footnote w:id="1"><w:p><w:r><w:t>Orphan.</w:t></w:r></w:p></w:footnote>"#;
    let parts = [
        notes_part("footnote", orphan, &mut foot),
        notes_part("endnote", "", &mut end),
    ];
    let docx = common::docx::docx_with(&common::docx::para("Body."), &parts);
    let (view, _, _) = docx_round_trip(&docx);
    assert!(view.contains("\nnotes: {empty: [endnotes]}"), "{view}");
}

#[test]
fn a_reference_notes_part_left_with_no_note_takes_the_separators() {
    // A reference whose footnotes part holds one note and no separators:
    // the writer drops the note, so it puts Word's separators back.
    let mut full = String::new();
    let note = r#"<w:footnote w:id="1"><w:p><w:r><w:t>Old.</w:t></w:r></w:p></w:footnote>"#;
    let part = notes_part("footnote", note, &mut full);
    let (content_type, rel_type) = (part.content_type.to_string(), part.rel_type.to_string());
    let bare = full
        .replace(r#"<w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>"#, "")
        .replace(r#"<w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>"#, "");
    assert!(!bare.contains("w:type="), "{bare}");
    let reference = common::docx::docx_with(
        &common::docx::para("Reference."),
        &[common::docx::Part {
            name: "word/footnotes.xml",
            content_type: &content_type,
            rel_type: &rel_type,
            xml: &bare,
        }],
    );
    let markdown = "---\nnotes: {empty: [footnotes]}\n---\n\nBody.\n";
    let docx = markdown_to_docx(
        markdown,
        &DocxOptions {
            reference: Some(&reference),
            ..DocxOptions::default()
        },
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    let notes = common::docx::part_string(&docx, "word/footnotes.xml").expect("the part");
    assert!(!notes.contains("Old."), "{notes}");
    assert!(notes.contains(r#"w:type="separator" w:id="-1""#), "{notes}");
    assert!(
        notes.contains(r#"w:type="continuationSeparator" w:id="0""#),
        "{notes}"
    );
}

#[test]
fn a_notes_clause_that_does_not_read_is_a_warning() {
    let markdown = "---\nnotes: {empty: [sidenotes]}\n---\n\nBody.\n";
    let out = markdown_to_docx(markdown, &DocxOptions::default()).expect("the view reads");
    assert!(
        out.warnings
            .iter()
            .any(|w| w.starts_with("notes: empty: names sidenotes")),
        "{:?}",
        out.warnings
    );
    assert!(!has_part(&out.docx, "word/footnotes.xml"));
}

/// Indent, label text and font of the numbering level one list paragraph uses.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ListLevel {
    format: String,
    text: String,
    left: String,
    hanging: String,
    /// `w:rFonts` `ascii` and `hAnsi`, when the level sets a font.
    font: Option<(String, String)>,
}

fn fixture(path: &str) -> String {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    std::fs::read_to_string(&file).unwrap_or_else(|err| panic!("{}: {err}", file.display()))
}

/// A without id lines, then D written from the view of that docx.
fn list_round_trip(markdown: &str) -> (Vec<u8>, Vec<u8>) {
    let from_markdown = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the Markdown reads")
        .docx;
    assert_word_valid_package(&from_markdown);
    let view = read(
        &from_markdown,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    let from_view = markdown_to_docx(&view, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    assert_word_valid_package(&from_view);
    (from_markdown, from_view)
}

fn attr(dom: &Dom, node: NodeId, name: &str) -> String {
    dom.attribute(node, &W::name(name))
        .unwrap_or("")
        .to_string()
}

fn level_of(dom: &Dom, lvl: NodeId) -> ListLevel {
    let ind = dom
        .element(lvl, &W::p_pr())
        .and_then(|props| dom.element(props, &W::name("ind")));
    let font = dom
        .element(lvl, &W::r_pr())
        .and_then(|props| dom.element(props, &W::name("rFonts")))
        .map(|fonts| (attr(dom, fonts, "ascii"), attr(dom, fonts, "hAnsi")));
    ListLevel {
        format: dom
            .element(lvl, &W::name("numFmt"))
            .map(|node| attr(dom, node, "val"))
            .unwrap_or_default(),
        text: dom
            .element(lvl, &W::name("lvlText"))
            .map(|node| attr(dom, node, "val"))
            .unwrap_or_default(),
        left: ind.map(|node| attr(dom, node, "left")).unwrap_or_default(),
        hanging: ind
            .map(|node| attr(dom, node, "hanging"))
            .unwrap_or_default(),
        font,
    }
}

/// The numbering level of each list paragraph, in document order.
fn list_levels(docx: &[u8]) -> Vec<ListLevel> {
    let document = part_string(docx, "word/document.xml").expect("document.xml");
    let numbering = part_string(docx, "word/numbering.xml").expect("numbering.xml");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&document);
    let nums = dom.parse_xdocument(&numbering);
    let doc_root = dom.root(doc).expect("document root");
    let num_root = dom.root(nums).expect("numbering root");
    let mut by_abstract: HashMap<(String, String), NodeId> = HashMap::new();
    for abs in dom.descendants(num_root, Some(&W::name("abstractNum"))) {
        let Some(id) = dom
            .attribute(abs, &W::name("abstractNumId"))
            .map(str::to_string)
        else {
            continue;
        };
        for lvl in dom.elements(abs, Some(&W::name("lvl"))) {
            let Some(ilvl) = dom.attribute(lvl, &W::name("ilvl")).map(str::to_string) else {
                continue;
            };
            by_abstract.insert((id.clone(), ilvl), lvl);
        }
    }
    let mut abstract_of: HashMap<String, String> = HashMap::new();
    let mut overridden: HashMap<(String, String), NodeId> = HashMap::new();
    for num in dom.elements(num_root, Some(&W::name("num"))) {
        let Some(id) = dom.attribute(num, &W::name("numId")).map(str::to_string) else {
            continue;
        };
        let Some(abs) = dom
            .element(num, &W::name("abstractNumId"))
            .and_then(|node| dom.attribute(node, &W::val()))
            .map(str::to_string)
        else {
            continue;
        };
        abstract_of.insert(id.clone(), abs);
        for over in dom.elements(num, Some(&W::name("lvlOverride"))) {
            let Some(ilvl) = dom.attribute(over, &W::name("ilvl")).map(str::to_string) else {
                continue;
            };
            if let Some(lvl) = dom.element(over, &W::name("lvl")) {
                overridden.insert((id.clone(), ilvl), lvl);
            }
        }
    }
    let mut out = Vec::new();
    for paragraph in dom.descendants(doc_root, Some(&W::p())) {
        let Some(props) = dom.element(paragraph, &W::p_pr()) else {
            continue;
        };
        let Some(numpr) = dom.element(props, &W::num_pr()) else {
            continue;
        };
        let ilvl = dom
            .element(numpr, &W::name("ilvl"))
            .and_then(|node| dom.attribute(node, &W::val()))
            .unwrap_or("0")
            .to_string();
        let num_id = dom
            .element(numpr, &W::name("numId"))
            .and_then(|node| dom.attribute(node, &W::val()))
            .unwrap_or("0")
            .to_string();
        if num_id == "0" {
            continue;
        }
        let lvl = overridden
            .get(&(num_id.clone(), ilvl.clone()))
            .copied()
            .or_else(|| {
                abstract_of
                    .get(&num_id)
                    .and_then(|abs| by_abstract.get(&(abs.clone(), ilvl.clone())).copied())
            })
            .unwrap_or_else(|| panic!("num {num_id} level {ilvl} has no definition"));
        out.push(level_of(&dom, lvl));
    }
    out
}

/// A level's `(left, hanging, lvlText)` as Word's list definition gives it.
type Indent<'a> = (&'a str, &'a str, &'a str);

#[test]
fn bullet_and_numbered_levels_match_after_the_id_line_round_trip() {
    let cases: &[(&str, &[Indent<'_>])] = &[
        (
            "tests/fixtures/legacy/services.md",
            &[
                ("720", "360", "\u{2022}"),
                ("1440", "360", "\u{25E6}"),
                ("720", "360", "%1."),
            ],
        ),
        (
            "tests/fixtures/from-docx/critic/synthetic/hard-lists.accept.md",
            &[
                ("720", "360", "%1."),
                ("1440", "360", "%2."),
                ("720", "360", "\u{2022}"),
            ],
        ),
        (
            "tests/fixtures/from-docx/critic/synthetic/hard-lists.critic.md",
            &[("720", "360", "%1."), ("1440", "360", "%2.")],
        ),
        (
            "tests/fixtures/from-docx/critic/synthetic/hard-lists.reject.md",
            &[("720", "360", "%1."), ("1440", "360", "%2.")],
        ),
    ];
    for (path, expect) in cases {
        let (from_markdown, from_view) = list_round_trip(&fixture(path));
        let plain = list_levels(&from_markdown);
        let viewed = list_levels(&from_view);
        assert_eq!(viewed, plain, "{path}");
        assert!(!plain.is_empty(), "{path}");
        for (left, hanging, text) in *expect {
            assert!(
                plain.iter().any(|level| {
                    level.left == *left && level.hanging == *hanging && level.text == *text
                }),
                "{path}: missing left {left} hanging {hanging} text {text:?} in {plain:?}"
            );
        }
    }
}

/// Body paragraph texts, deletions left out, a line break as `\n`.
fn paragraph_texts(docx: &[u8]) -> Vec<String> {
    jubarte::inspect::paragraphs(docx)
        .expect("paragraphs")
        .into_iter()
        .map(|paragraph| paragraph.text)
        .collect()
}

/// `read` of `docx` with one pinned source and no page markers.
fn agent_view(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            source: Some("a.md".into()),
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown
}

/// The view without its `sha256:` line. Two docx of one view differ there.
fn without_sha(markdown: &str) -> String {
    let mut text = markdown
        .lines()
        .filter(|line| !line.starts_with("sha256:"))
        .collect::<Vec<_>>()
        .join("\n");
    if markdown.ends_with('\n') {
        text.push('\n');
    }
    text
}

/// A to B to C to D. The space is in B; C has to carry it or D loses it.
fn chain(markdown: &str) -> (Vec<String>, String, Vec<String>, String) {
    let b = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("A converts")
        .docx;
    assert_word_valid_package(&b);
    let c = agent_view(&b);
    let d = markdown_to_docx(&c, &DocxOptions::default())
        .expect("C converts")
        .docx;
    assert_word_valid_package(&d);
    (paragraph_texts(&b), c, paragraph_texts(&d), agent_view(&d))
}

#[test]
fn a_leading_space_after_a_tracked_break_round_trips() {
    // The space is not at the start of A's line (the closer is), so the
    // writer stores it as the next paragraph's first run. The view has to
    // say so: a literal leading space would not come back.
    let cases = [
        (
            "packaging{++\n\n++}{>>Bo Chen (2026-09-01T00:07:00Z)<<} but excludes.\n",
            " but excludes.",
            "&#32;but excludes.",
        ),
        (
            "rate.{--\n\n--}{>>Bo Chen (2026-09-01T01:38:00Z)<<} The price.\n",
            " The price.",
            "&#32;The price.",
        ),
    ];
    for (markdown, run, shown) in cases {
        let (b, c, d, again) = chain(markdown);
        assert!(
            b.iter().any(|text| text == run),
            "B lost the leading space {run:?}: {b:?}"
        );
        assert!(
            c.contains(shown),
            "the view dropped the leading space {shown:?}:\n{c}"
        );
        assert_eq!(d, b, "D dropped the leading space\n{c}");
        assert_eq!(without_sha(&again), without_sha(&c), "read(D):\n{again}");
    }
}

#[test]
fn a_leading_space_after_a_line_break_round_trips() {
    // `<br>&#32;` is a `w:br` and a run of one space. A literal space on
    // the next Markdown line is indentation and never reaches the run.
    let (b, c, d, again) = chain("alpha<br>&#32;beta\n");
    assert_eq!(b, ["alpha\n beta".to_string()]);
    assert!(
        c.contains("alpha\\\n&#32;beta"),
        "the view dropped the space after the break:\n{c}"
    );
    assert_eq!(d, b, "D dropped the space after the break\n{c}");
    assert_eq!(without_sha(&again), without_sha(&c), "read(D):\n{again}");
}

/// The `w:style` element whose `w:styleId` is `id`.
fn style_element(styles: &str, id: &str) -> String {
    let key = format!("w:styleId=\"{id}\"");
    let at = styles
        .find(&key)
        .unwrap_or_else(|| panic!("no {id} in {styles}"));
    let start = styles[..at].rfind("<w:style ").unwrap_or(at);
    let end = styles[at..]
        .find("</w:style>")
        .map(|i| at + i + "</w:style>".len())
        .unwrap_or(styles.len());
    styles[start..end].to_string()
}

/// A → B → C → D for id-less Markdown. D's list style keeps its indent and
/// contextual spacing, its heading keeps keep-lines, and D's Normal carries
/// no `rPr` or `pPr` that B's lacks.
fn keeps_list_indent_keep_lines_and_a_bare_normal(markdown: &str) {
    let first = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("A reads")
        .docx;
    let view = read(
        &first,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("B reads")
    .markdown;
    let second = markdown_to_docx(&view, &DocxOptions::default())
        .expect("C reads")
        .docx;
    let before = common::docx::part_string(&first, "word/styles.xml").expect("B styles");
    let after = common::docx::part_string(&second, "word/styles.xml").expect("D styles");
    let list = style_element(&after, "ListParagraph");
    assert!(list.contains("w:left=\"720\""), "{list}");
    assert!(list.contains("contextualSpacing"), "{list}");
    let heading = style_element(&after, "Heading1");
    assert!(heading.contains("keepLines"), "{heading}");
    let normal_b = style_element(&before, "Normal");
    let normal_d = style_element(&after, "Normal");
    for tag in ["<w:rPr", "<w:pPr"] {
        if normal_d.contains(tag) {
            assert!(
                normal_b.contains(tag),
                "D's Normal has {tag} and B's does not\nB: {normal_b}\nD: {normal_d}"
            );
        }
    }
}

#[test]
fn an_idless_file_keeps_list_indent_keep_lines_and_a_bare_normal() {
    keeps_list_indent_keep_lines_and_a_bare_normal(include_str!(
        "fixtures/from-docx/critic/synthetic/all.accept.md"
    ));
    keeps_list_indent_keep_lines_and_a_bare_normal(include_str!(
        "fixtures/from-docx/critic/synthetic/hard-lists.accept.md"
    ));
    keeps_list_indent_keep_lines_and_a_bare_normal(include_str!(
        "fixtures/from-docx/critic/synthetic/hard-lists.critic.md"
    ));
    keeps_list_indent_keep_lines_and_a_bare_normal(include_str!(
        "fixtures/from-docx/critic/synthetic/hard-lists.reject.md"
    ));
}

/// `markdown` with its `sha256:` line dropped. The line names the docx
/// bytes, which a second write does not repeat.
fn without_sha256(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    for line in markdown.split_inclusive('\n') {
        if !line.starts_with("sha256:") {
            out.push_str(line);
        }
    }
    out
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).expect("a docx zip");
    let mut file = zip.by_name("word/document.xml").expect("word/document.xml");
    let mut xml = String::new();
    file.read_to_string(&mut xml).expect("document xml");
    xml
}

/// Plain Markdown with no id lines, through docx and back, then written
/// once more. The second read matches the first apart from `sha256:`.
fn idless_chain(markdown: &str) -> (String, Vec<u8>, Vec<u8>) {
    let read_options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    let b = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("B")
        .docx;
    assert_word_valid_package(&b);
    let c = read(&b, &read_options).expect("C").markdown;
    let d = markdown_to_docx(&c, &DocxOptions::default())
        .expect("D")
        .docx;
    assert_word_valid_package(&d);
    let again = read(&d, &read_options).expect("read D").markdown;
    assert_eq!(without_sha256(&again), without_sha256(&c), "read(D) != C");
    (c, b, d)
}

/// `$x_{--}$` is text that looks like an equation: the escapes the view
/// writes keep it text through the chain, with no math built from it.
#[test]
fn literal_math_delimiters_keep_their_page_text() {
    let markdown = include_str!("fixtures/from-docx/critic/synthetic/hard-delimiters.accept.md");
    let (c, b, d) = idless_chain(markdown);
    assert!(
        c.lines()
            .any(|line| line == r"Math-like text $x\_{\--\}$ and $y\^{\++\}$ stays."),
        "{c}"
    );
    // The paragraph's text, whatever runs carry it: the delimiters are
    // page text, `--` and `++` included.
    for (name, docx) in [("B", &b), ("D", &d)] {
        let xml = document_xml(docx);
        let paragraph = xml
            .split("<w:p>")
            .find(|p| p.contains("Math-like"))
            .unwrap_or_else(|| panic!("{name} lost the paragraph: {xml}"));
        let text: String = paragraph
            .split("<w:t")
            .skip(1)
            .filter_map(|t| t.split_once('>').and_then(|(_, t)| t.split_once("</w:t>")))
            .map(|(t, _)| t)
            .collect();
        assert_eq!(
            text, "Math-like text $x_{--}$ and $y^{++}$ stays.",
            "{name}: {xml}"
        );
        assert!(!xml.contains("<m:oMath"), "{name} built math from text");
    }
}

/// Bold closed against bold is `****`, which comes back as asterisks.
#[test]
fn bold_around_italic_does_not_print_empty_emphasis() {
    let markdown = "__Monthly distribution of fishing effort and common dolphin (_Delphinus delphis_) captures.__\n";
    let (c, _b, d) = idless_chain(markdown);
    let line = "**Monthly distribution of fishing effort and common dolphin (**<b><i>Delphinus delphis</i></b>**) captures.**";
    assert!(c.lines().any(|l| l == line), "{c}");
    assert!(!c.contains("****"), "{c}");
    let xml = document_xml(&d);
    assert!(xml.contains("delphis"), "{xml}");
    assert!(xml.contains(") captures."), "{xml}");
    assert!(!xml.contains("****"), "{xml}");
}

/// `=  ` is two spaces: collapsing them onto one moves the line Word draws.
/// A line break in a math run reads as a space, since a break in the view
/// would end the paragraph's line. The break follows `λ`, not `\lambda`:
/// LaTeX ends a control word on the space after it.
#[test]
fn math_run_spaces_survive_the_view_and_a_break_reads_as_a_space() {
    let markdown = "$$\\textrm{logit}(\\pi_i) =  λ\n\\label{eq:one}$$\n";
    let (c, b, d) = idless_chain(markdown);
    let equation = "$$\\text{logit}(π_i) =  λ \\backslash label\\lbrace eq:one\\rbrace$$";
    assert!(c.contains(equation), "{c}");
    for (name, docx) in [("B", &b), ("D", &d)] {
        let xml = document_xml(docx);
        assert!(
            xml.contains(" =  "),
            "{name} collapsed the spaces after '=': {xml}"
        );
    }
}

#[test]
fn section_references_come_in_words_order() {
    // Every section Word wrote in the bench corpus (30 with three or more
    // references) lists them even, default header, even, default footer,
    // then the first header and footer.
    let header = "---\nheaders:\n  \
                  first: {id: header3, text: F}\n  \
                  default: {id: header2, text: D}\n  \
                  even: {id: header1, text: E}\n\
                  footers:\n  \
                  first: {id: footer3, text: f}\n  \
                  default: {id: footer2, text: d}\n  \
                  even: {id: footer1, text: e}\n---\n\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{header}<!-- p0 -->\nOne.\n")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    let document = part_string(&docx, "word/document.xml").expect("a body");
    let order: Vec<String> = document
        .split("Reference w:type=\"")
        .skip(1)
        .zip(document.match_indices("<w:").filter_map(|(at, _)| {
            let rest = &document[at + 3..];
            ["headerReference", "footerReference"]
                .into_iter()
                .find(|k| rest.starts_with(k))
                .map(|k| &k[..1])
        }))
        .map(|(rest, kind)| format!("{kind}{}", &rest[..rest.find('"').unwrap()]))
        .collect();
    assert_eq!(
        order,
        ["heven", "hdefault", "feven", "fdefault", "hfirst", "ffirst"],
        "{document}"
    );
}

#[test]
fn a_cell_paragraphs_style_comes_back_from_the_table_line() {
    // `pN Style` names a cell paragraph's style; consecutive paragraphs in
    // one style share a range (`p2-p3 Caption`), as the 18 List Paragraph
    // cell lines of 3c920d5996_rejected_tracking.docx need.
    let md = "<!-- t0 2x2, cells p0-p3 by row, p1 center, p1 ListParagraph, p2-p3 Caption -->\n\
              |A|B|\n|-|-|\n|C|D|\n\n<!-- p4 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_style_id_that_is_no_plain_word_is_quoted() {
    // pi on #490: a style id spelled like a clause word (`center`), with a
    // space, or holding `, ` was read back as an alignment, a cut style or
    // a style that never existed. Such an id prints quoted, `\` and `"`
    // escaped, on the table line and on a paragraph's id line alike.
    let md = "<!-- p0 \"center\" -->\nCentred by style\n\n\
              <!-- p1 \"My Style\" justify -->\nSpaced\n\n\
              <!-- t0 2x2, cells p2-p5 by row, p3 \"center\", p4 \"A, B\", p5 \"say \\\"hi\\\"\" -->\n\
              |A|B|\n|-|-|\n|C|D|\n\n<!-- p6 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
    let docx = markdown_to_docx(&as_view(md), &DocxOptions::default())
        .expect("the view reads")
        .docx;
    let document = part_string(&docx, "word/document.xml").expect("a body");
    for style in ["center", "My Style", "A, B", "say &quot;hi&quot;"] {
        assert!(
            document.contains(&format!("<w:pStyle w:val=\"{style}\"/>")),
            "{style}: {document}"
        );
    }
    assert!(!document.contains("<w:jc w:val=\"center\"/>"), "{document}");
}

#[test]
fn a_nested_tables_paragraph_style_comes_back() {
    // pi on #490: the cells range numbers a nested table's paragraphs; the
    // style clause names them too.
    let p = |text: &str| {
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="Caption"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#
        )
    };
    let cell = |inner: &str| {
        format!(r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>{inner}</w:tc>"#)
    };
    let nested = format!(
        r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="1500"/></w:tblGrid><w:tr>{}</w:tr></w:tbl><w:p/>"#,
        cell(&p("N"))
    );
    let body = format!(
        r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid><w:tr>{}{}</w:tr></w:tbl><w:p><w:r><w:t>After</w:t></w:r></w:p>"#,
        cell("<w:p><w:r><w:t>A</w:t></w:r></w:p>"),
        cell(&format!("{}{nested}", p("B")))
    );
    let source = common::docx::docx(&body);
    let view = read(
        &source,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    let docx = markdown_to_docx(&view, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    let document = part_string(&docx, "word/document.xml").expect("a body");
    assert_eq!(
        document.matches("<w:pStyle w:val=\"Caption\"/>").count(),
        2,
        "{view}\n{document}"
    );
}

/// `A` written, read as a view (`C`), written again (`D`) and read again.
/// The two views, without `sha256:` (the packages differ), then `B` and `D`.
fn idless_views(markdown: &str) -> (String, String, Vec<u8>, Vec<u8>) {
    let options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    let b = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("A reads")
        .docx;
    assert_word_valid_package(&b);
    let c = read(&b, &options).expect("B reads").markdown;
    let d = markdown_to_docx(&c, &DocxOptions::default())
        .expect("C reads")
        .docx;
    assert_word_valid_package(&d);
    let again = read(&d, &options).expect("D reads").markdown;
    (without_sha(&c), without_sha(&again), b, d)
}

fn xml_decode(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn tag_attr(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')? + start;
    Some(xml_decode(&tag[start..end]))
}

/// Visible text of every `w:t` in `xml`. `<w:tab` is not one.
fn run_texts(xml: &str) -> String {
    let mut out = String::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<w:t") {
        let from = &rest[start..];
        let boundary = from.as_bytes().get(4).copied();
        if !matches!(boundary, Some(b'>' | b' ' | b'/')) {
            rest = &from[4..];
            continue;
        }
        let Some(open_end) = from.find('>') else {
            break;
        };
        if from.as_bytes().get(open_end.wrapping_sub(1)) == Some(&b'/') {
            rest = &from[open_end + 1..];
            continue;
        }
        let after = &from[open_end + 1..];
        let Some(close) = after.find("</w:t>") else {
            break;
        };
        out.push_str(&xml_decode(&after[..close]));
        rest = &after[close + "</w:t>".len()..];
    }
    out
}

/// `(visible text, relationship target)` of each external hyperlink.
fn hyperlinks(docx: &[u8]) -> Vec<(String, String)> {
    let document = common::docx::part_string(docx, "word/document.xml").expect("document.xml");
    let rels =
        common::docx::part_string(docx, "word/_rels/document.xml.rels").expect("document.xml.rels");
    let mut targets = Vec::new();
    let mut rel_rest = rels.as_str();
    while let Some(start) = rel_rest.find("<Relationship ") {
        let from = &rel_rest[start..];
        let end = from.find("/>").unwrap_or(from.len());
        let tag = &from[..end];
        if tag_attr(tag, "Type").is_some_and(|ty| ty.ends_with("/hyperlink"))
            && let (Some(id), Some(target)) = (tag_attr(tag, "Id"), tag_attr(tag, "Target"))
        {
            targets.push((id, target));
        }
        rel_rest = &from[end..];
    }
    let mut out = Vec::new();
    let mut rest = document.as_str();
    while let Some(start) = rest.find("<w:hyperlink ") {
        let from = &rest[start..];
        let open_end = from.find('>').expect("hyperlink opens");
        let open = &from[..=open_end];
        let after = &from[open_end + 1..];
        let close = after.find("</w:hyperlink>").expect("hyperlink closes");
        if let Some(id) = tag_attr(open, "r:id") {
            let target = targets
                .iter()
                .find(|(rel, _)| rel == &id)
                .map(|(_, target)| target.clone())
                .unwrap_or_default();
            out.push((run_texts(&after[..close]), target));
        }
        rest = &after[close + "</w:hyperlink>".len()..];
    }
    out
}

/// Text of each `VerbatimChar` run, in document order.
fn verbatim_runs(docx: &[u8]) -> Vec<String> {
    let document = common::docx::part_string(docx, "word/document.xml").expect("document.xml");
    let mut out = Vec::new();
    let mut rest = document.as_str();
    while let Some(start) = rest.find("<w:r>") {
        let from = &rest[start + "<w:r>".len()..];
        let Some(close) = from.find("</w:r>") else {
            break;
        };
        let run = &from[..close];
        if run.contains("w:val=\"VerbatimChar\"") {
            out.push(run_texts(run));
        }
        rest = &from[close + "</w:r>".len()..];
    }
    out
}

/// A link whose URL holds parentheses comes back as that link.
/// `S1074-7613(00)80619-6`: a bare destination treats the final `)` as the
/// link's closer, and percent-encoding only `)` leaves `(` unmatched, so
/// the link comes back as literal text.
#[test]
fn a_link_whose_url_holds_parentheses_comes_back_as_that_link() {
    const URL: &str = concat!(
        "http://elife.elifesciences.org/lookup/external-ref",
        "?access_num=10.1016/S1074-7613(00)80619-6&link_type=DOI"
    );
    let (c, again, b, d) = idless_views(&format!("See [CrossRef]({URL}) now.\n"));
    let expected = vec![("CrossRef".to_string(), URL.to_string())];
    assert_eq!(hyperlinks(&b), expected, "{c}");
    assert_eq!(hyperlinks(&d), expected, "{again}");
    assert!(c.contains(&format!("[CrossRef](<{URL}>)")), "{c}");
    assert_eq!(again, c);
}

/// An email autolink keeps the address as its text. `<addr>` reads back as
/// the address; `<mailto:addr>` reads back with `mailto:` in the text.
#[test]
fn an_email_autolink_keeps_the_address_as_its_text() {
    let (c, again, b, d) =
        idless_views("[karthik.ram@berkeley.edu](mailto:karthik.ram@berkeley.edu)\n");
    let expected = vec![(
        "karthik.ram@berkeley.edu".to_string(),
        "mailto:karthik.ram@berkeley.edu".to_string(),
    )];
    assert_eq!(hyperlinks(&b), expected, "{c}");
    assert_eq!(hyperlinks(&d), expected, "{again}");
    assert!(c.contains("<karthik.ram@berkeley.edu>"), "{c}");
    assert!(!c.contains("<mailto:"), "{c}");
    assert_eq!(again, c);
}

/// A plain space between two code spans keeps both spans. Folding the
/// space into the mark makes one `VerbatimChar` run of both.
#[test]
fn a_plain_space_between_code_spans_keeps_both_spans() {
    let (c, again, b, d) = idless_views(include_str!("fixtures/math/SOURCES.md"));
    let left = verbatim_runs(&b);
    let right = verbatim_runs(&d);
    assert_eq!(left.len(), 31, "{left:?}");
    assert_eq!(right, left);
    assert!(
        left.windows(2)
            .any(|pair| pair[0] == "docProps/app.xml" && pair[1] == "Application"),
        "{left:?}"
    );
    assert!(c.contains("`docProps/app.xml` `Application`"), "{c}");
    assert!(!c.contains("`docProps/app.xml Application`"), "{c}");
    assert_eq!(again, c, "{c}");
}

/// A styles part with `Normal` as the default paragraph style.
fn default_normal_styles() -> common::docx::Part<'static> {
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="BodyText"><w:name w:val="Body Text"/><w:basedOn w:val="Normal"/></w:style></w:styles>"#,
        W_NS = common::docx::W_NS
    );
    common::docx::Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: Box::leak(styles.into_boxed_str()),
    }
}

/// A paragraph that states the default style in its own `w:pPr` keeps the
/// statement: the view names `Normal` only where the document says it
/// (ids-authors, ids-comments, #528).
#[test]
fn an_explicit_default_style_is_named_and_comes_back() {
    let stated =
        r#"<w:p><w:pPr><w:pStyle w:val="Normal"/></w:pPr><w:r><w:t>Stated.</w:t></w:r></w:p>"#;
    let unstated = r#"<w:p><w:r><w:t>Unstated.</w:t></w:r></w:p>"#;
    let cell = r#"<w:tbl><w:tr><w:tc><w:p><w:pPr><w:pStyle w:val="Normal"/></w:pPr></w:p></w:tc></w:tr></w:tbl>"#;
    let end = r#"<w:p><w:r><w:t>End.</w:t></w:r></w:p>"#;
    let body = format!("{stated}{unstated}{cell}{end}");
    let parts = [default_normal_styles()];
    let docx = common::docx::docx_with(&body, &parts);
    let (view, again, once) = docx_round_trip(&docx);
    assert!(view.contains("<!-- p0 Normal -->"), "{view}");
    assert!(view.contains("<!-- p1 -->"), "{view}");
    assert!(view.contains("p2 Normal"), "{view}");
    let document = part_string(&again, "word/document.xml").unwrap();
    assert_eq!(
        document.matches(r#"<w:pStyle w:val="Normal"/>"#).count(),
        2,
        "{document}"
    );
    assert_eq!(view, once);
}

/// An empty paragraph keeps the style it states: a bare `<!-- p9 empty -->`
/// stands only for paragraphs with no `w:pStyle`, and a heading style is
/// named too, with no `#` to imply it (ids-paragraphs, ids-font-sizes,
/// #528).
#[test]
fn an_empty_paragraph_keeps_its_style() {
    let stated = r#"<w:p><w:pPr><w:pStyle w:val="TitlePage"/></w:pPr></w:p>"#;
    let heading = r#"<w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr></w:p>"#;
    let bare = r"<w:p/>";
    let body = format!("{stated}{stated}{bare}{heading}<w:p><w:r><w:t>Text.</w:t></w:r></w:p>");
    let parts = [default_normal_styles()];
    let docx = common::docx::docx_with(&body, &parts);
    let (view, again, once) = docx_round_trip(&docx);
    assert!(view.contains("<!-- p0 TitlePage empty -->"), "{view}");
    assert!(view.contains("<!-- p1 TitlePage empty -->"), "{view}");
    assert!(view.contains("<!-- p2 empty -->"), "{view}");
    assert!(view.contains("<!-- p3 Heading2 empty -->"), "{view}");
    let document = part_string(&again, "word/document.xml").unwrap();
    assert_eq!(
        document.matches(r#"<w:pStyle w:val="TitlePage"/>"#).count(),
        2,
        "{document}"
    );
    assert_eq!(
        document.matches(r#"<w:pStyle w:val="Heading2"/>"#).count(),
        1
    );
    assert_eq!(view, once);
}

/// A range of styled empty paragraphs (`<!-- p0-p1 TitlePage empty -->`)
/// stands for every paragraph it numbers, not only the first.
#[test]
fn a_range_of_styled_empty_paragraphs_stands_for_all_of_them() {
    let md = "<!-- p0-p1 TitlePage empty -->\n\n<!-- p2 -->\nText.\n";
    let docx = markdown_to_docx(&as_view(md), &DocxOptions::default())
        .expect("the view reads")
        .docx;
    assert_word_valid_package(&docx);
    let document = part_string(&docx, "word/document.xml").unwrap();
    assert_eq!(
        document.matches(r#"<w:pStyle w:val="TitlePage"/>"#).count(),
        2,
        "{document}"
    );
}

/// A heading that states its own outline level, with no style, keeps the
/// statement: `# Text` alone means the writer's Heading1, so the id line
/// says the level is the paragraph's own (`outline 2`, ids-font-sizes,
/// #528).
#[test]
fn a_heading_that_states_its_outline_writes_no_style() {
    let outlined =
        r#"<w:p><w:pPr><w:outlineLvl w:val="1"/></w:pPr><w:r><w:t>Own level.</w:t></w:r></w:p>"#;
    let body = format!("{outlined}<w:p><w:r><w:t>Body.</w:t></w:r></w:p>");
    let parts = [default_normal_styles()];
    let docx = common::docx::docx_with(&body, &parts);
    let (view, again, once) = docx_round_trip(&docx);
    assert!(view.contains("## Own level."), "{view}");
    assert!(view.contains("outline 2"), "{view}");
    let document = part_string(&again, "word/document.xml").unwrap();
    assert!(
        document.contains(r#"<w:outlineLvl w:val="1"/>"#),
        "{document}"
    );
    assert!(!document.contains("w:pStyle"), "{document}");
    assert_eq!(view, once);
}

/// Every outline level 0-8 survives the round trip: the `#` count stops
/// at the writer's heading styles, so a level past them (7-9) is body
/// text whose id line alone carries it (`outline 7` to `outline 9`),
/// and the writer writes the `w:outlineLvl` it says (#549).
#[test]
fn outline_levels_past_the_heading_styles_survive() {
    let mut body = String::new();
    for level in 0..=8 {
        body.push_str(&format!(
            r#"<w:p><w:pPr><w:outlineLvl w:val="{level}"/></w:pPr><w:r><w:t>L{level}.</w:t></w:r></w:p>"#
        ));
    }
    let parts = [default_normal_styles()];
    let docx = common::docx::docx_with(&body, &parts);
    let (view, again, once) = docx_round_trip(&docx);
    for level in 0..=5 {
        assert!(
            view.contains(&format!("{} L{level}.", "#".repeat(level + 1))),
            "{view}"
        );
    }
    // Past the heading styles there is no `#` to print: the clause alone
    // says the level, and the text stays a plain paragraph.
    for level in 6..=8 {
        assert!(
            view.contains(&format!("outline {} -->\nL{level}.", level + 1)),
            "{view}"
        );
    }
    assert!(!view.contains("###### L6."), "{view}");
    assert!(!view.contains("###### L7."), "{view}");
    assert!(!view.contains("###### L8."), "{view}");
    let document = part_string(&again, "word/document.xml").unwrap();
    assert_eq!(document.matches("<w:outlineLvl").count(), 9, "{document}");
    for level in 0..=8 {
        assert!(
            document.contains(&format!(r#"<w:outlineLvl w:val="{level}"/>"#)),
            "{document}"
        );
    }
    assert!(!document.contains("w:pStyle"), "{document}");
    assert_eq!(view, once);
}

/// An empty paragraph that states its own outline level keeps it: the
/// pooling `<!-- p1-p2 empty -->` is for paragraphs with nothing to say,
/// and a `w:outlineLvl` is something to say (#549).
#[test]
fn an_empty_paragraph_keeps_its_own_outline_level() {
    let one = r#"<w:p><w:pPr><w:outlineLvl w:val="1"/></w:pPr></w:p>"#;
    let two = r#"<w:p><w:pPr><w:outlineLvl w:val="2"/></w:pPr></w:p>"#;
    let body =
        format!("<w:p><w:r><w:t>S.</w:t></w:r></w:p>{one}{two}<w:p><w:r><w:t>E.</w:t></w:r></w:p>");
    let parts = [default_normal_styles()];
    let docx = common::docx::docx_with(&body, &parts);
    let (view, again, once) = docx_round_trip(&docx);
    assert!(view.contains("<!-- p1 outline 2, empty -->"), "{view}");
    assert!(view.contains("<!-- p2 outline 3, empty -->"), "{view}");
    assert!(!view.contains("p1-p2"), "{view}");
    let document = part_string(&again, "word/document.xml").unwrap();
    assert_eq!(
        document.matches(r#"<w:outlineLvl w:val="1"/>"#).count(),
        1,
        "{document}"
    );
    assert_eq!(
        document.matches(r#"<w:outlineLvl w:val="2"/>"#).count(),
        1,
        "{document}"
    );
    assert_eq!(view, once);
}
