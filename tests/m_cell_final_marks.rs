// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A table cell's two final marks pair, as Word's Compare pairs them: a
//! paragraph added at the end of a cell, a cell's paragraph replaced by
//! two, or every paragraph of a cell deleted leaves the cell's final mark
//! unrevised, and the inserted or deleted mark is the one before it. A
//! revision on a cell's final mark hangs Word's Accept All or Reject All
//! (#1669; Word 16 oracles of 2026-10-23 on the issue).

mod common;

use common::docx::{docx, para, part_string};
use common::validity::check_word_valid_package;
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};
use jubarte::inspect::paragraphs;

/// A table between two paragraphs: its first cell holds "Full.", its
/// second `cell` (a row whose every cell changes is #1672's).
fn table(cell: &str) -> Vec<u8> {
    let tc = |inner: &str| {
        format!("<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{inner}</w:tc>")
    };
    docx(&format!(
        "{}<w:tbl><w:tblPr><w:tblW w:w=\"4000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{}{}</w:tr></w:tbl>{}",
        para("Before."),
        tc(&para("Full.")),
        tc(cell),
        para("After.")
    ))
}

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

/// The second cell's paragraphs, each as its mark's revision (`+`, `-` or none)
/// and its text, inserted words after `+`, deleted ones after `-`.
fn cell(bytes: &[u8]) -> Vec<(&'static str, String)> {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let start = xml.rfind("<w:tc>").unwrap();
    let end = start + xml[start..].find("</w:tc>").unwrap();
    let paragraph = regex::Regex::new(r"(?s)<w:p(?: [^>]*)?(?:/>|>.*?</w:p>)").unwrap();
    let mark = regex::Regex::new(r"(?s)<w:pPr>.*?<w:rPr>(.*?)</w:rPr>.*?</w:pPr>").unwrap();
    let token =
        regex::Regex::new(r"<w:(ins|del) |</w:(ins|del)>|<w:(?:t|delText)(?: [^>]*)?>([^<]*)</w:")
            .unwrap();
    paragraph
        .find_iter(&xml[start..end])
        .map(|p| {
            let p = p.as_str();
            let revision = match mark.captures(p).map(|c| c[1].to_string()) {
                Some(rpr) if rpr.contains("<w:ins ") => "+",
                Some(rpr) if rpr.contains("<w:del ") => "-",
                _ => "",
            };
            let body = p.find("</w:pPr>").map_or(p, |e| &p[e..]);
            let (mut side, mut text) = ("", String::new());
            for c in token.captures_iter(body) {
                if let Some(open) = c.get(1) {
                    side = if open.as_str() == "ins" { "+" } else { "-" };
                } else if c.get(2).is_some() {
                    side = "";
                } else {
                    text.push_str(&format!("{side}{}", &c[3]));
                }
            }
            (revision, text)
        })
        .collect()
}

/// Compare `a` with `b`: the redline is Word-valid (no revision on a cell's
/// final mark), accepts to `b` and rejects to `a`; its cell is `expected`.
fn check(a: &[u8], b: &[u8], expected: &[(&str, &str)], what: &str) -> Vec<u8> {
    let out = check_valid(a, b, what);
    let expected: Vec<(&str, String)> = expected.iter().map(|(m, t)| (*m, t.to_string())).collect();
    assert_eq!(cell(&out), expected, "{what}");
    out
}

#[test]
fn a_paragraph_appended_to_a_cell_inserts_the_mark_before_the_final_one() {
    check(
        &table(&para("Old.")),
        &table(&format!("{}{}", para("Old."), para("New."))),
        &[("+", "Old."), ("", "+New.")],
        "append",
    );
    check(
        &table(&para("Old.")),
        &table(&format!("{}{}{}", para("Old."), para("One."), para("Two."))),
        &[("+", "Old."), ("+", "+One."), ("", "+Two.")],
        "append two",
    );
}

#[test]
fn a_cells_paragraph_replaced_by_two_inserts_the_mark_before_the_final_one() {
    let out = check(
        &table(&para("Old.")),
        &table(&format!("{}{}", para("One."), para("Two."))),
        &[("+", "+One-Old."), ("", "+Two.")],
        "replace by two",
    );
    // The first paragraph's words are the two sides' first words, in
    // either order; the mark after them is the inserted one.
    assert!(cell(&out)[0].1.contains("+One") && cell(&out)[0].1.contains("-Old"));
}

#[test]
fn deleting_every_paragraph_of_a_cell_deletes_the_mark_before_the_final_one() {
    check(
        &table(&format!("{}{}", para("One."), para("Two."))),
        &table("<w:p/>"),
        &[("-", "-One."), ("", "-Two.")],
        "delete all",
    );
}

/// The final mark records the original's properties when the revised last
/// paragraph is formatted otherwise: Reject All gives the original its
/// properties back, Accept All keeps the revision's.
#[test]
fn an_appended_cell_paragraph_records_the_original_final_properties() {
    let centered = r#"<w:p><w:pPr><w:jc w:val="center"/><w:rPr><w:b/></w:rPr></w:pPr><w:r><w:t>Old.</w:t></w:r></w:p>"#;
    let a = table(centered);
    let b = table(&format!("{centered}{}", para("New.")));
    let out = check(&a, &b, &[("+", "Old."), ("", "+New.")], "formatted");
    let properties = |bytes: &[u8]| -> Vec<String> {
        let xml = part_string(bytes, "word/document.xml").unwrap();
        let start = xml.rfind("<w:tc>").unwrap();
        let end = start + xml[start..].find("</w:tc>").unwrap();
        let ppr = regex::Regex::new(r"(?s)<w:p(?: [^>]*)?(?:/>|>(.*?)</w:p>)").unwrap();
        ppr.captures_iter(&xml[start..end])
            .map(|c| {
                let p = c.get(1).map_or("", |m| m.as_str()).replace(" />", "/>");
                let jc = p.contains(r#"<w:jc w:val="center"/>"#);
                let bold = p.contains("<w:b/>");
                format!("jc={jc} b={bold}")
            })
            .collect()
    };
    assert_eq!(
        properties(&accept_revisions(&out).unwrap()),
        properties(&b),
        "Accept All"
    );
    assert_eq!(
        properties(&reject_revisions(&out).unwrap()),
        properties(&a),
        "Reject All"
    );
}

/// A cell's paragraphs replaced by others that share no word: whichever
/// order the comparer writes the two sides in, the closing marks pair and
/// the cell's final mark stays unrevised.
#[test]
fn a_cells_paragraphs_replaced_whole_keep_the_final_mark() {
    for (old, new) in [
        (para("Alpha beta gamma."), para("Delta epsilon zeta.")),
        (
            format!("{}{}", para("Alpha beta."), para("Gamma delta.")),
            para("Epsilon zeta eta theta."),
        ),
        (
            para("Alpha beta gamma delta."),
            format!("{}{}", para("Epsilon zeta."), para("Eta theta.")),
        ),
    ] {
        let a = table(&old);
        let b = table(&new);
        let out = check_valid(&a, &b, "replaced whole");
        assert_eq!(cell(&out).last().unwrap().0, "", "{:?}", cell(&out));
    }
}

/// A cell holding a nested table between two paragraphs, replaced by an
/// empty paragraph: the outer cell's closing marks pair over the deleted
/// table, as Word writes it.
#[test]
fn a_cells_tail_replaced_over_a_nested_table_keeps_the_final_mark() {
    let nested = format!(
        r#"{}<w:tbl><w:tblPr><w:tblW w:w="1800" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="1800"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="1800" w:type="dxa"/></w:tcPr>{}</w:tc></w:tr></w:tbl>{}"#,
        para("Lead."),
        para("Inner."),
        para("Tail.")
    );
    check_valid(&table(&nested), &table("<w:p/>"), "nested tail");
}

/// A row gaining a cell. Here the comparer rewrites the row (the old one
/// deleted, the new one inserted, every mark with its row); where it keeps
/// the row and adds the cell, the new cell's mark stays unrevised and only
/// its words are inserted (Word 16, ff42b4a7a3's "Role" cell:
/// `observed_m348_rewritten_cells_keep_revised_properties_and_complete_original_history`).
/// Either way no cell's final mark carries a revision its row does not.
#[test]
fn a_row_gaining_a_cell_resolves_both_ways() {
    let row = |cells: &[&str]| {
        let tc = |text: &str| {
            format!(
                "<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{}</w:tc>",
                para(text)
            )
        };
        let grid: String = cells.iter().map(|_| "<w:gridCol w:w=\"2000\"/>").collect();
        docx(&format!(
            "{}<w:tbl><w:tblPr><w:tblW w:w=\"6000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr>{}</w:tr></w:tbl>{}",
            para("Before."),
            cells.iter().map(|c| tc(c)).collect::<String>(),
            para("After.")
        ))
    };
    let a = row(&["Name", "Department"]);
    let b = row(&["Name", "Department", "Role"]);
    check_valid(&a, &b, "added cell");
    check_valid(&b, &a, "removed cell");
}

/// The redline of `a` against `b` is Word-valid, accepts to `b` and rejects
/// to `a`.
fn check_valid(a: &[u8], b: &[u8], what: &str) -> Vec<u8> {
    let out = compare_documents(a, b, "Redline").expect("compare");
    let report = check_word_valid_package(&out);
    assert!(
        report.ok(),
        "{what}: {:?}\n{}",
        report.errors,
        part_string(&out, "word/document.xml").unwrap()
    );
    assert_eq!(
        texts(&accept_revisions(&out).unwrap()),
        texts(b),
        "{what}: Accept All"
    );
    assert_eq!(
        texts(&reject_revisions(&out).unwrap()),
        texts(a),
        "{what}: Reject All"
    );
    out
}
