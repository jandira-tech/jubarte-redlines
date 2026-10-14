// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The reader brings back what the agent view's id lines say about a
//! paragraph (`<!-- p2-p4 empty -->`, `<!-- p5 TitlePage center -->`), so a
//! view read into a docx and read again prints the same ids.

mod common;

use common::validity::assert_word_valid_package;
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

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
fn a_revision_number_is_written_once() {
    // A change written as several marks (two runs, or a view that names one
    // number for two changes) keeps its number on the first mark; the rest
    // take fresh ones, so no two marks share a `w:id`.
    let body = "<!-- p0 -->\n{++**Bold** plain++}{>>r4 @AC<<} and {++one++}{>>r5 @AC<<} \
                {++two++}{>>r5 @AC<<}.\n";
    let docx = markdown_to_docx(
        &as_view(&format!("{AUTHORS}{body}")),
        &DocxOptions::default(),
    )
    .expect("the view reads")
    .docx;
    assert_word_valid_package(&docx);
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    for id in ["4", "5"] {
        assert!(
            document.contains(&format!("w:ins w:id=\"{id}\"")),
            "w:id {id} kept: {document}"
        );
    }
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
    // Two cells, one `<br>`, two paragraphs: the `<br>` is a line break.
    let md = "<!-- t0 1x2, cells p0-p1 by row -->\n|A<br>B|C|\n|-|-|\n\n<!-- p2 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
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
    // Each id line ends the Markdown list, so the view prints the level
    // Word gives the paragraph (`ilvl`) on its id line, not as indentation
    // (which past one level would read as code).
    let md = "<!-- p0 ListParagraph num \"1)\" -->\n1. One.\n\n\
              <!-- p1 ListParagraph num \"a)\", level 1 -->\n1. Under one.\n\n\
              <!-- p2 ListParagraph num \"i)\", level 2 -->\n1. Deeper.\n\n\
              <!-- p3 ListParagraph num \"b)\", level 1 -->\n2. Under one again.\n\n\
              <!-- p4 ListParagraph num \"2)\" -->\n2. Two.\n\n\
              <!-- p5 bullet, level 1 -->\n- A bullet under two.\n\n\
              <!-- p6 -->\nAfter.\n";
    assert_eq!(round_trip(md), md.trim());
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
