// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A table cell replaced whole must not take the content after its table
//! with it. The bench corpus pair docx_lots_of_comments_addition ×
//! …_redline holds a cell whose revised side is a tracked insertion of a
//! HYPERLINK field around the same words; the paragraph verdict replaces
//! the cell (no unit matches across the field), and the heading and the
//! 48-row table after it were then deleted and re-inserted whole (as moves)
//! while Word and the run resolvers keep them unchanged.

mod common;

use common::docx::{docx, esc, para};
use jubarte::changes::list_changes;
use jubarte::document_comparer::compare_documents;

const REV: &str = r#"w:author="A" w:date="2026-07-03T18:56:00Z""#;

fn cell(p: &str) -> String {
    format!(r#"<w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr>{p}</w:tc>"#)
}

fn table(rows: &[Vec<String>]) -> String {
    let cols = rows[0].len();
    let grid: String = (0..cols).map(|_| r#"<w:gridCol w:w="4000"/>"#).collect();
    let body: String = rows
        .iter()
        .map(|r| {
            format!(
                "<w:tr>{}</w:tr>",
                r.iter().map(|p| cell(p)).collect::<String>()
            )
        })
        .collect();
    format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid>{grid}</w:tblGrid>{body}</w:tbl>"#
    )
}

/// A paragraph that is one tracked insertion of a HYPERLINK field around
/// `text`, its mark inserted too (what Word writes for a pasted link).
fn inserted_link_para(text: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="40" {REV}/></w:rPr></w:pPr><w:ins w:id="41" {REV}><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve">HYPERLINK "https://example.test/a" \h</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>{}</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:ins></w:p>"#,
        esc(text)
    )
}

/// A paragraph that is one tracked insertion of `text`, its mark inserted
/// too (a cell of a table pasted with Track Changes on).
fn inserted_para(text: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="42" {REV}/></w:rPr></w:pPr><w:ins w:id="43" {REV}><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:ins></w:p>"#,
        esc(text)
    )
}

#[test]
fn a_replaced_cell_leaves_the_next_table_in_place() {
    let heading = "3. Capability matrix: Word has all the expected basics";
    let a = format!(
        "{}{}{}",
        table(&[
            vec![para("Source"), para("Link")],
            vec![para("Microsoft Support"), para("Open source")],
        ]),
        para(heading),
        table(&[
            vec![para("Capability"), para("Word advantage")],
            vec![
                para("Real-time coauthoring"),
                para("Collaboration with desktop")
            ],
        ]),
    );
    let b = format!(
        "{}{}{}",
        table(&[
            vec![para("Source"), para("Link")],
            vec![para("Microsoft Support"), inserted_link_para("Open source")],
        ]),
        para(heading),
        table(&[
            vec![inserted_para("Capability"), inserted_para("Word advantage")],
            vec![
                inserted_para("Real-time coauthoring"),
                inserted_para("Collaboration with desktop")
            ],
        ]),
    );
    let ours = compare_documents(&docx(&a), &docx(&b), "Comparison").expect("compare");
    let changes: Vec<String> = list_changes(&ours)
        .expect("list changes")
        .into_iter()
        .map(|c| format!("{:?} {} {}", c.kind, c.target, c.text))
        .collect();
    for kept in [
        heading,
        "Capability",
        "Word advantage",
        "Real-time coauthoring",
    ] {
        assert!(
            !changes
                .iter()
                .any(|c| c.starts_with("Deletion") && c.contains(kept)),
            "{kept:?} must stay in place after the replaced cell: {changes:#?}"
        );
        assert!(
            !changes
                .iter()
                .any(|c| c.contains("Move") && c.contains(kept)),
            "{kept:?} must not become a move: {changes:#?}"
        );
    }
}
