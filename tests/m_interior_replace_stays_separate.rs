// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word's replace-gap grammar (decoded from its Compare output; Docxodus 12
//! renders the same): inside the body, a region whose old and new paragraphs
//! share no words stays whole. The new paragraphs come first under inserted
//! paragraph marks, the old ones after them under deleted marks. A bare
//! paragraph mark, or an empty paragraph found on both sides, is no evidence
//! that two unrelated paragraphs are one edited paragraph. Pairing on it fused
//! "TWO" into "e" and "A" into "a" (list_with_table_break ×
//! broken_complex_list: docxide 13.2 vs Docxodus 93.3).

mod common;

use std::io::{Cursor, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::Dom;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use common::validity::assert_word_valid_package;

fn settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        ..WmlComparerSettings::default()
    }
}

/// An empty paragraph that carries its own spacing.
const SPACED_EMPTY: &str = "<spaced>";
/// A one-cell table.
const TABLE: &str = "<table>";

fn pkg(paras: &[&str]) -> Vec<u8> {
    let body: String = paras
        .iter()
        .map(|t| {
            if t.is_empty() {
                "<w:p/>".to_string()
            } else if *t == TABLE {
                r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Cell text that goes away</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#.to_string()
            } else if *t == SPACED_EMPTY {
                r#"<w:p><w:pPr><w:spacing w:after="0"/></w:pPr></w:p>"#.to_string()
            } else {
                format!(r#"<w:p><w:r><w:t xml:space="preserve">{t}</w:t></w:r></w:p>"#)
            }
        })
        .collect();
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, data) in [
            ("[Content_Types].xml", ct.as_slice()),
            ("_rels/.rels", rels.as_slice()),
            ("word/document.xml", doc.as_bytes()),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(data).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

/// Each body paragraph as `(inserted text, deleted text, plain text)`.
fn paragraphs(out: &[u8]) -> Vec<(String, String, String)> {
    let pkg = PartFs::open(out).expect("open");
    let xml = pkg.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let (ins, del) = (W::ins(), W::del());
    let mut rows = Vec::new();
    for p in dom.descendants(root, Some(&W::p())) {
        let (mut i, mut x, mut plain) = (String::new(), String::new(), String::new());
        for t in dom.descendants(p, None) {
            let is_t = dom.name_is(t, &W::t());
            if !is_t && !dom.name_is(t, &W::name("delText")) {
                continue;
            }
            let v = dom.value_str(t).to_string();
            let mut anc = dom.parent(t);
            let mut side = None;
            while let Some(a) = anc {
                if a == p {
                    break;
                }
                if dom.name_is(a, &ins) {
                    side = Some(true);
                } else if dom.name_is(a, &del) {
                    side = Some(false);
                }
                anc = dom.parent(a);
            }
            match side {
                Some(true) => i.push_str(&v),
                Some(false) => x.push_str(&v),
                None => plain.push_str(&v),
            }
        }
        rows.push((i, x, plain));
    }
    rows
}

fn assert_no_fused_paragraph(a: &[&str], b: &[&str]) -> Vec<(String, String, String)> {
    let out = compare_documents_with_settings(&pkg(a), &pkg(b), &settings()).expect("compare");
    assert_word_valid_package(&out);
    let rows = paragraphs(&out);
    for (i, x, plain) in &rows {
        assert!(
            i.is_empty() || x.is_empty(),
            "paragraph fuses inserted {i:?} with deleted {x:?} (plain {plain:?}): {rows:?}"
        );
    }
    rows
}

/// The inserted paragraphs precede the deleted ones in the gap, in order.
fn gap_order(rows: &[(String, String, String)]) -> Vec<String> {
    rows.iter()
        .filter_map(|(i, x, _)| {
            if !i.is_empty() {
                Some(format!("+{i}"))
            } else if !x.is_empty() {
                Some(format!("-{x}"))
            } else {
                None
            }
        })
        .collect()
}

/// A paragraph mark shared by "TWO" and "e" is not a pairing.
#[test]
fn an_interior_rewrite_keeps_old_and_new_paragraphs_whole() {
    let rows = assert_no_fused_paragraph(
        &["Opening clause stays", "TWO", "Closing clause stays"],
        &["Opening clause stays", "e", "", "a", "Closing clause stays"],
    );
    assert_eq!(gap_order(&rows), ["+e", "+a", "-TWO"]);
}

/// An empty paragraph on both sides, between unrelated paragraphs, is not an
/// anchor: Word's region stays one replace, inserts first.
#[test]
fn an_empty_paragraph_between_unrelated_text_is_no_anchor() {
    let rows = assert_no_fused_paragraph(
        &[
            "Opening clause stays",
            "A",
            "",
            "Old schedule text",
            "Closing clause stays",
        ],
        &[
            "Opening clause stays",
            "",
            "a",
            "",
            "b",
            "Closing clause stays",
        ],
    );
    assert_eq!(gap_order(&rows), ["+a", "+b", "-A", "-Old schedule text"]);
}

/// Both documents end on an empty paragraph: Word pairs those two final
/// marks, so the redline still ends on a live empty paragraph after the
/// deleted one, even behind a long run of insertions.
#[test]
fn the_final_empty_paragraph_survives_a_trailing_deletion() {
    let added: Vec<String> = (1..=11).map(|k| format!("Added item {k}")).collect();
    let mut b = vec!["Opening clause stays"];
    b.extend(added.iter().map(String::as_str));
    b.push(SPACED_EMPTY);
    let rows = assert_no_fused_paragraph(&["Opening clause stays", "TWO", SPACED_EMPTY], &b);
    let order = gap_order(&rows);
    assert_eq!(order.last().map(String::as_str), Some("-TWO"), "{order:?}");
    let out = compare_documents_with_settings(
        &pkg(&["Opening clause stays", "TWO", SPACED_EMPTY]),
        &pkg(&b),
        &settings(),
    )
    .expect("compare");
    let xml = PartFs::open(&out)
        .unwrap()
        .part_string("word/document.xml")
        .unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let last = *dom.descendants(root, Some(&W::p())).last().unwrap();
    let marked = !dom.descendants(last, Some(&W::ins())).is_empty()
        || !dom.descendants(last, Some(&W::del())).is_empty();
    assert!(
        rows.last()
            .is_some_and(|(i, x, p)| i.is_empty() && x.is_empty() && p.is_empty())
            && !marked,
        "the redline ends on the live final empty paragraph: {rows:?}"
    );
}

/// The revised document's last paragraph has text while the original ends on
/// an empty one: that text has no mark of its own before the story's final
/// mark, so Word fuses it into the first deleted paragraph of the story's
/// last gap, even across a deleted table (diff_before16 × diff_before19).
#[test]
fn the_story_tail_text_fuses_into_the_first_deleted_paragraph() {
    let out = compare_documents_with_settings(
        &pkg(&["Diffing feature", TABLE, ""]),
        &pkg(&["Some text", "An image will be added below:"]),
        &settings(),
    )
    .expect("compare");
    assert_word_valid_package(&out);
    let rows = paragraphs(&out);
    assert!(
        rows.iter()
            .any(|(i, x, _)| i == "An image will be added below:" && x == "Diffing feature"),
        "the tail text fuses with the deleted heading: {rows:?}"
    );
}

/// A blank on both sides that closes a region before a table pairs only
/// through Word's pilcrow chain, which "Contract Review" facing a blank
/// cancels: every paragraph keeps its own mark (file_36 × file_37).
#[test]
fn a_wordful_original_facing_a_blank_cancels_the_blank_chain() {
    let rows = assert_no_fused_paragraph(
        &["Opening clause stays", "Contract Review", "", TABLE],
        &[
            "Opening clause stays",
            "HR Onboarding Checklist",
            "",
            "",
            TABLE,
        ],
    );
    let marked_blanks = rows
        .iter()
        .take_while(|(_, x, _)| x != "Cell text that goes away")
        .filter(|(i, x, p)| i.is_empty() && x.is_empty() && p.is_empty())
        .count();
    let out = compare_documents_with_settings(
        &pkg(&["Opening clause stays", "Contract Review", "", TABLE]),
        &pkg(&[
            "Opening clause stays",
            "HR Onboarding Checklist",
            "",
            "",
            TABLE,
        ]),
        &settings(),
    )
    .expect("compare");
    let xml = PartFs::open(&out)
        .unwrap()
        .part_string("word/document.xml")
        .unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.descendants(root, Some(&W::body()))[0];
    let live_blanks = dom
        .elements(body, Some(&W::p()))
        .into_iter()
        .filter(|&p| {
            dom.descendants(p, Some(&W::t())).is_empty()
                && dom.descendants(p, Some(&W::ins())).is_empty()
                && dom.descendants(p, Some(&W::del())).is_empty()
        })
        .count();
    assert_eq!(
        (gap_order(&rows), live_blanks),
        (
            vec![
                "+HR Onboarding Checklist".to_string(),
                "-Contract Review".to_string()
            ],
            0
        ),
        "every blank before the table keeps its own mark ({marked_blanks} unmarked rows): {rows:?}"
    );
}

/// The revised document ends on its only body paragraph while the original
/// ends on blanks, a table and an empty final paragraph: Word fuses that
/// text into the first deleted blank, which keeps its deleted mark, so
/// accepting every change leaves exactly the revised paragraphs
/// (support_tickets_table × support_tickets_summary).
#[test]
fn the_story_tail_text_keeps_the_first_deleted_mark() {
    let out = compare_documents_with_settings(
        &pkg(&["Support Tickets", "", "", TABLE, ""]),
        &pkg(&["Support Tickets Summary", "Ticket ID Issue Priority"]),
        &settings(),
    )
    .expect("compare");
    assert_word_valid_package(&out);
    let xml = PartFs::open(&out)
        .unwrap()
        .part_string("word/document.xml")
        .unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.descendants(root, Some(&W::body()))[0];
    let mark = |p, rev: &jubarte::xmllinq::XName| {
        dom.element(p, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::r_pr()))
            .is_some_and(|rpr| dom.element(rpr, rev).is_some())
    };
    let paras = dom.elements(body, Some(&W::p()));
    let text_para = paras
        .iter()
        .copied()
        .find(|&p| {
            dom.descendants(p, Some(&W::t()))
                .iter()
                .any(|&t| dom.value_str(t).contains("Ticket ID"))
        })
        .expect("the revised text is in the body");
    let inserted_marks = paras.iter().filter(|&&p| mark(p, &W::ins())).count();
    assert!(
        mark(text_para, &W::del()) && inserted_marks == 0,
        "the tail text rides the first deleted blank ({inserted_marks} inserted marks): {:?}",
        paragraphs(&out)
    );
}

/// A revised tail already riding a deleted blank (the original's leading
/// empty paragraph) is Word's fused paragraph: it stays where it is, so
/// rejecting every change still restores that blank
/// (restart_numbering_sub_list × sd_1480_two_col_index).
#[test]
fn an_already_fused_tail_keeps_its_deleted_blank() {
    let a = ["", "ONE", "A", "TWO", "B", ""];
    let b = [
        "Index of defined terms",
        "Page1 Page8",
        "Page2 Page7",
        "Page4 Page5",
    ];
    let out = compare_documents_with_settings(&pkg(&a), &pkg(&b), &settings()).expect("compare");
    assert_word_valid_package(&out);
    let rows = paragraphs(&out);
    assert!(
        rows.iter()
            .all(|(i, x, _)| !(i.contains("Page4") && x.contains("ONE"))),
        "the tail must not re-fuse into ONE: {rows:?}"
    );
}

/// The story's final pilcrow is shared even when the original's last
/// paragraph is deleted text: the revised last paragraph's words still fuse
/// into the first deleted paragraph of the tail (diff_doc2 × numwords).
#[test]
fn the_story_tail_fuses_before_a_deleted_final_paragraph() {
    let rows = assert_no_fused_paragraph_except(
        &[
            "Opening clause stays",
            "It contains three paragraphs",
            TABLE,
            "",
            "Another paragraph",
        ],
        &["Opening clause stays", "", "page 3"],
        "page 3",
    );
    assert!(
        rows.iter()
            .any(|(i, x, _)| i == "page 3" && x == "It contains three paragraphs"),
        "the revised tail opens the first deleted paragraph: {rows:?}"
    );
}

fn assert_no_fused_paragraph_except(
    a: &[&str],
    b: &[&str],
    fused: &str,
) -> Vec<(String, String, String)> {
    let out = compare_documents_with_settings(&pkg(a), &pkg(b), &settings()).expect("compare");
    assert_word_valid_package(&out);
    let rows = paragraphs(&out);
    for (i, x, plain) in &rows {
        assert!(
            i.is_empty() || x.is_empty() || i == fused,
            "paragraph fuses inserted {i:?} with deleted {x:?} (plain {plain:?}): {rows:?}"
        );
    }
    rows
}
