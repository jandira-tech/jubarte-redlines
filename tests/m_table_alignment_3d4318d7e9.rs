// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Task 22 (`3d4318d7e9`): Word's table alignment on mismatched grids.
//!
//! Sources `aeb0ace263` (2 tables: 3x4, 19x6) vs `1000537d17` (3 tables:
//! 21x8, 11x8, 47x9) share no identical rows. Word still pairs rows by
//! shared words, splits deleted runs into their own tables, and inserts
//! whole new tables:
//!
//! | Word table | live gridCols | rows | TR-INS | TR-DEL |
//! |---|---|---|---|---|
//! | 1 (3 old x 21 new) | 10 | 21 | 18 | 0 |
//! | 2 (19 old x 11 new, fragment) | 16 | 5 | 1 | 0 |
//! | 3 (deleted run, 6 rows) | 6 | 6 | 0 | 6 |
//! | 4 (fragment) | 21 | 4 | 1 | 0 |
//! | 5 (deleted run, 1 row) | 6 | 1 | 0 | 1 |
//! | 6 (fragment) | 9 | 2 | 0 | 0 |
//! | 7 (deleted run, 3 rows) | 6 | 3 | 0 | 3 |
//! | 8 (47 new rows, inserted whole) | 9 | 47 | 47 | 0 |
//!
//! Four `tblPrChange` / four `tblGridChange`, one per mixed fragment.
//! The `our_compare_matches_word_table_shape` test is the Task 22 target:
//! it is red until the comparer learns the row alignment and the split.

mod common;

use common::validity::assert_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

const A: &str = "tests/corpus/neurotic_docx_bench/corpus/word/clean/docx/aeb0ace263_793bb9c35c342cd532221e60f7f70c0411de141b896a63b2.docx";
const B: &str = "tests/corpus/neurotic_docx_bench/corpus/word/clean/docx/1000537d17_8d9cfd6b8768cf88164c1b8c735e97b737193f3e92fd18ee.docx";
const WORD: &str = "tests/corpus/neurotic_docx_bench/corpus/word/tracking_without_comments/docx/aeb0ace263__vs__1000537d17_redline_3d4318d7e9.docx";

/// (live gridCol count, row count, TR-INS rows, TR-DEL rows) per table.
type TableShape = (usize, usize, usize, usize);

const WORD_SHAPES: [TableShape; 8] = [
    (10, 21, 18, 0),
    (16, 5, 1, 0),
    (6, 6, 0, 6),
    (21, 4, 1, 0),
    (6, 1, 0, 1),
    (9, 2, 0, 0),
    (6, 3, 0, 3),
    (9, 47, 47, 0),
];

fn word_mode() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        merge_replaced_paragraphs: true,
        detect_format_changes: true,
        ..WmlComparerSettings::default()
    }
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).unwrap();
    let mut s = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("word/document.xml").unwrap(), &mut s).unwrap();
    s
}

/// Byte offset of the next `<tag>` / `<tag ...>` open (not `<tagX`).
fn open_at(xml: &str, tag: &str) -> Option<usize> {
    let mut from = 0;
    loop {
        let i = xml[from..].find(tag)? + from;
        let after = xml[i + tag.len()..].chars().next().unwrap_or('>');
        if after == '>'
            || after == ' '
            || after == '\t'
            || after == '\r'
            || after == '\n'
            || after == '/'
        {
            return Some(i);
        }
        from = i + tag.len();
    }
}

/// Split `hay` into top-level `<tag>…</tag>` elements (nesting-aware).
fn elements<'a>(hay: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = hay;
    while let Some(start) = open_at(rest, &open) {
        let mut depth = 0;
        let mut i = start;
        let mut end = None;
        loop {
            let no = open_at(&rest[i..], &open).map(|o| i + o);
            let nc = rest[i..].find(&close).map(|c| i + c);
            match (no, nc) {
                (Some(o), Some(c)) if o < c => {
                    depth += 1;
                    i = o + open.len();
                }
                (_, Some(c)) => {
                    depth -= 1;
                    i = c + close.len();
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
                _ => break,
            }
        }
        let Some(end) = end else { break };
        out.push(&rest[start..end]);
        rest = &rest[end..];
    }
    out
}

/// Split `word/document.xml` into its top-level `w:tbl` elements.
fn tables(xml: &str) -> Vec<&str> {
    let body = xml
        .split("<w:body>")
        .nth(1)
        .unwrap_or("")
        .split("</w:body>")
        .next()
        .unwrap_or("");
    elements(body, "w:tbl")
}

fn live_grid_cols(tbl: &str) -> usize {
    let grid = tbl
        .split("<w:tblGrid>")
        .nth(1)
        .unwrap_or("")
        .split("</w:tblGrid>")
        .next()
        .unwrap_or("");
    // A fragment's tblGrid also carries the w:tblGridChange record (which
    // nests a full copy of the old grid): cut it off, then count live cols.
    let live = grid.split("<w:tblGridChange").next().unwrap_or(grid);
    live.matches("<w:gridCol").count()
}

/// True when `xml` opens `<tag>` / `<tag …>` (not `<tagX`, not `<tagY…>`).
fn has_open(xml: &str, tag: &str) -> bool {
    open_at(xml, &format!("<{tag}")).is_some()
}

/// (TR-INS rows, TR-DEL rows): marks live in `w:trPr`, ahead of the cells.
fn row_marks(tbl: &str) -> (usize, usize) {
    let mut ins = 0;
    let mut del = 0;
    for row in elements(tbl, "w:tr") {
        let head = row.split("<w:tc").next().unwrap_or(row);
        if has_open(head, "w:ins") {
            ins += 1;
        } else if has_open(head, "w:del") {
            del += 1;
        }
    }
    (ins, del)
}

fn shape_of(tbl: &str) -> TableShape {
    let (ins, del) = row_marks(tbl);
    (live_grid_cols(tbl), elements(tbl, "w:tr").len(), ins, del)
}

/// Pins Word's own redline shape, so the target cannot drift silently.
#[test]
fn word_redline_has_expected_table_shape() {
    let xml = document_xml(&std::fs::read(WORD).unwrap());
    let tbls = tables(&xml);
    assert_eq!(tbls.len(), WORD_SHAPES.len(), "Word table count");
    for (i, (tbl, want)) in tbls.iter().zip(WORD_SHAPES.iter()).enumerate() {
        assert_eq!(shape_of(tbl), *want, "Word table {}", i + 1);
    }
    assert_eq!(
        xml.matches("<w:tblPrChange").count(),
        4,
        "Word tblPrChange count"
    );
    assert_eq!(
        xml.matches("<w:tblGridChange").count(),
        4,
        "Word tblGridChange count"
    );
}

/// Task 22 target: our compare reaches Word's table shape.
#[test]
#[ignore = "KNOWN_ISSUES.md #11: Task 22 target, the comparer does not split tables yet"]
fn our_compare_matches_word_table_shape() {
    let a = std::fs::read(A).unwrap();
    let b = std::fs::read(B).unwrap();
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    assert_word_valid_package(&out);
    let xml = document_xml(&out);
    let tbls = tables(&xml);
    assert_eq!(tbls.len(), WORD_SHAPES.len(), "table count");
    for (i, (tbl, want)) in tbls.iter().zip(WORD_SHAPES.iter()).enumerate() {
        assert_eq!(shape_of(tbl), *want, "table {}", i + 1);
    }
    assert_eq!(
        xml.matches("<w:tblPrChange").count(),
        4,
        "tblPrChange count"
    );
    assert_eq!(
        xml.matches("<w:tblGridChange").count(),
        4,
        "tblGridChange count"
    );
}

/// Word pairs old rows with new rows even when nothing matches exactly:
/// the old header's `Rhythmia` deletion shares a merged row with the new
/// header's insertion, and old patient 1's row merges into the
/// `Health Website` row — never a whole deleted table beside a whole
/// inserted one. Index-agnostic so later fragmentation cannot break it.
#[test]
fn our_compare_pairs_unmatched_rows_into_merged_rows() {
    let a = std::fs::read(A).unwrap();
    let b = std::fs::read(B).unwrap();
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    assert_word_valid_package(&out);
    let xml = document_xml(&out);
    let tbls = tables(&xml);
    let merged_with = |del: &str, ins: &str| {
        tbls.iter().any(|tbl| {
            elements(tbl, "w:tr").into_iter().any(|row| {
                let head = row.split("<w:tc").next().unwrap_or(row);
                let paired = !has_open(head, "w:ins") && !has_open(head, "w:del");
                paired && row.contains(del) && row.contains(ins)
            })
        })
    };
    assert!(
        merged_with("Rhythmia", "Group"),
        "old header row merges with the new header row"
    );
    assert!(
        merged_with("Patient number", "Resource")
            || merged_with("Small reentry in the posterior", "Health Website"),
        "an old patient row merges with a new resource row"
    );
}
