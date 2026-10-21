// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A paragraph's own text size rides on its id line (#1474): `size 9pt`
//! when every run of its text states one size its style does not give, or,
//! for a paragraph with no text, when its mark states one. The writer puts
//! it on each run and on the mark. Without it a table of 10pt cells in a
//! 12pt document came back at 12pt, and took a page more than the source.
//! Runs that differ, or that say no more than the style, print nothing.

mod common;

use common::docx::{docx, part_string};
use common::validity::assert_word_valid_package;
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

fn view(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown
    // The source's hash differs for every docx written.
    .lines()
    .filter(|l| !l.starts_with("sha256: "))
    .collect::<Vec<_>>()
    .join("\n")
}

fn write(markdown: &str) -> Vec<u8> {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the Markdown reads")
        .docx;
    assert_word_valid_package(&docx);
    docx
}

fn document(docx: &[u8]) -> String {
    part_string(docx, "word/document.xml").expect("a main document")
}

/// Each paragraph's text-run sizes (`w:sz` of the runs holding `w:t` or
/// `w:delText`, `-` for a run with none) and its mark's, in order: what
/// the reader must give back. The bodies here nest no paragraphs.
fn sizes(xml: &str) -> Vec<(Vec<String>, Option<String>)> {
    let value = |rpr: &str, name: &str| {
        let key = format!("<w:{name} w:val=\"");
        rpr.find(&key).map(|at| {
            let rest = &rpr[at + key.len()..];
            rest[..rest.find('"').unwrap()].to_string()
        })
    };
    xml.split("<w:p>")
        .skip(1)
        .map(|p| {
            let p = &p[..p.find("</w:p>").unwrap()];
            let mark = p
                .find("<w:pPr>")
                .map(|at| &p[at..at + p[at..].find("</w:pPr>").unwrap()])
                .and_then(|ppr| ppr.find("<w:rPr>").map(|at| &ppr[at..]))
                .and_then(|rpr| value(rpr, "sz"));
            let runs = p
                .split("<w:r>")
                .skip(1)
                .map(|r| &r[..r.find("</w:r>").unwrap_or(r.len())])
                .filter(|r| r.contains("<w:t") || r.contains("<w:delText"))
                .map(|r| {
                    let sz = value(r, "sz");
                    // `w:szCs` follows `w:sz`, the same size.
                    assert_eq!(value(r, "szCs"), sz, "{r}");
                    sz.unwrap_or_else(|| "-".to_string())
                })
                .collect();
            (runs, mark)
        })
        .collect()
}

#[test]
fn a_paragraph_whose_runs_share_a_size_names_it() {
    let source = docx(concat!(
        r#"<w:p><w:r><w:rPr><w:sz w:val="18"/><w:szCs w:val="18"/></w:rPr><w:t xml:space="preserve">Nine </w:t></w:r>"#,
        r#"<w:r><w:rPr><w:b/><w:sz w:val="18"/></w:rPr><w:t>points</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:sz w:val="21"/></w:rPr><w:t>Half</w:t></w:r></w:p>"#,
    ));
    let shown = view(&source);
    assert!(
        shown.contains("<!-- p0 size 9pt -->\nNine **points**"),
        "{shown}"
    );
    assert!(
        shown.contains("<!-- p1 center, size 10.5pt -->\nHalf"),
        "{shown}"
    );
    let again = write(&shown);
    assert_eq!(
        sizes(&document(&again)),
        [
            (vec!["18".into(), "18".into()], Some("18".into())),
            (vec!["21".into()], Some("21".into())),
        ]
    );
    assert_eq!(view(&again), shown);
}

#[test]
fn runs_that_differ_or_say_the_styles_size_print_nothing() {
    // The fixture's Normal is 10pt (no `w:sz` anywhere): `20` says no more.
    let source = docx(concat!(
        r#"<w:p><w:r><w:rPr><w:sz w:val="18"/></w:rPr><w:t xml:space="preserve">Nine </w:t></w:r>"#,
        r#"<w:r><w:rPr><w:sz w:val="24"/></w:rPr><w:t>twelve</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:rPr><w:sz w:val="18"/></w:rPr><w:t xml:space="preserve">Nine </w:t></w:r>"#,
        r#"<w:r><w:t>plain</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:rPr><w:sz w:val="20"/></w:rPr><w:t>Ten</w:t></w:r></w:p>"#,
    ));
    let shown = view(&source);
    assert!(!shown.contains("size"), "{shown}");
    assert!(shown.contains("<!-- p0 -->\nNine twelve"), "{shown}");
}

#[test]
fn an_empty_paragraphs_mark_size_keeps_its_own_line() {
    // An empty paragraph is a line of its mark's size: 12pt is taller
    // than the style's 10pt, so it may not merge into a range.
    let source = docx(concat!(
        r#"<w:p><w:r><w:t>Before</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:rPr><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:pPr></w:p>"#,
        r#"<w:p/>"#,
        r#"<w:p><w:r><w:t>After</w:t></w:r></w:p>"#,
    ));
    let shown = view(&source);
    assert!(shown.contains("<!-- p1 empty, size 12pt -->"), "{shown}");
    assert!(shown.contains("<!-- p2 empty -->"), "{shown}");
    let again = write(&shown);
    assert_eq!(sizes(&document(&again))[1], (vec![], Some("24".into())));
    assert_eq!(sizes(&document(&again))[2], (vec![], None));
    assert_eq!(view(&again), shown);
}

#[test]
fn cell_paragraphs_name_their_size_on_the_table_line() {
    let cell = |text: &str, sz: &str| {
        format!(
            r#"<w:tc><w:tcPr><w:tcW w:w="4680" w:type="dxa"/></w:tcPr><w:p><w:r><w:rPr><w:sz w:val="{sz}"/></w:rPr><w:t>{text}</w:t></w:r></w:p></w:tc>"#
        )
    };
    let source = docx(&format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="5000" w:type="pct"/></w:tblPr><w:tblGrid><w:gridCol w:w="4680"/><w:gridCol w:w="4680"/></w:tblGrid><w:tr>{}{}</w:tr><w:tr>{}{}</w:tr></w:tbl><w:p><w:r><w:t>After</w:t></w:r></w:p>"#,
        cell("A", "18"),
        cell("B", "18"),
        cell("C", "18"),
        cell("D", "24"),
    ));
    let shown = view(&source);
    assert!(
        shown.contains("<!-- t0 2x2, cells p0-p3 by row, p0-p2 size 9pt, p3 size 12pt -->"),
        "{shown}"
    );
    let again = write(&shown);
    let got: Vec<Vec<String>> = sizes(&document(&again))
        .into_iter()
        .map(|(runs, _)| runs)
        .collect();
    assert_eq!(got, [["18"], ["18"], ["18"], ["24"], ["-"]]);
    assert_eq!(view(&again), shown);
}

#[test]
fn a_size_clause_written_by_hand_reaches_every_run_and_the_mark() {
    let docx = write(
        "---\nview: tracked\n---\n\n<!-- p0 size 14pt -->\nOne *two* [three](https://example.com)\n\n<!-- p1 -->\nPlain\n",
    );
    let got = sizes(&document(&docx));
    let (runs, mark) = &got[0];
    // `One `, `two`, ` `, `three`: the link's run too.
    assert_eq!(runs.len(), 4, "{runs:?}");
    assert!(runs.iter().all(|sz| sz == "28"), "{runs:?}");
    assert_eq!(mark.as_deref(), Some("28"));
    assert_eq!(got[1], (vec!["-".into()], None));
}

#[test]
fn a_tracked_insertion_and_deletion_keep_their_size() {
    let source = docx(concat!(
        r#"<w:p><w:r><w:rPr><w:sz w:val="18"/></w:rPr><w:t xml:space="preserve">Kept </w:t></w:r>"#,
        r#"<w:ins w:id="1" w:author="Ann Cole" w:date="2026-10-01T00:00:00Z"><w:r><w:rPr><w:sz w:val="18"/></w:rPr><w:t>new</w:t></w:r></w:ins>"#,
        r#"<w:del w:id="2" w:author="Ann Cole" w:date="2026-10-01T00:00:00Z"><w:r><w:rPr><w:sz w:val="18"/></w:rPr><w:delText>old</w:delText></w:r></w:del></w:p>"#,
    ));
    let shown = view(&source);
    assert!(shown.contains("<!-- p0 size 9pt -->"), "{shown}");
    let again = write(&shown);
    let (runs, _) = &sizes(&document(&again))[0];
    assert_eq!(runs, &["18", "18", "18"], "{}", document(&again));
}

#[test]
fn a_size_out_of_range_is_left_out() {
    // `w:sz` is ST_HpsMeasure: 1 to 3276 half-points (1638pt).
    let docx = write(
        "---\nview: tracked\n---\n\n<!-- p0 size 0pt -->\nZero\n\n<!-- p1 size 1700pt -->\nHuge\n\n<!-- p2 size 1638pt -->\nMost\n",
    );
    let got = sizes(&document(&docx));
    assert_eq!(got[0].0, ["-"]);
    assert_eq!(got[1].0, ["-"]);
    assert_eq!(got[2].0, ["3276"]);
}
