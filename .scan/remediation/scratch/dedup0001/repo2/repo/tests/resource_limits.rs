// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Resource budgets on the public render path (security-scan dedup-0001):
//! attacker-chosen numbers inside an admitted package — table spans, a
//! renamed deeply nested part, a huge tab stop — must render within a
//! bounded allocation, not size one.

mod common;

use common::docx::{Part, docx, docx_with};
use jubarte::convert::{docx_to_pdf, pdf_page_count};

/// One `w:gridSpan` of a billion in a one-cell table: the span parsed as
/// bare `usize` used to size the column vector (~4 GiB for the `f32`s)
/// before layout began. The span now clamps to Word's reachability bound
/// and the tiny table renders.
#[test]
fn a_huge_gridspan_renders_within_budget() {
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"4675\"/></w:tblGrid>\
                <w:tr><w:tc><w:tcPr>\
                <w:gridSpan w:val=\"1000000000\"/>\
                </w:tcPr><w:p/></w:tc></w:tr></w:tbl>";
    let pdf = docx_to_pdf(&docx(body)).expect("a clamped span renders");
    assert_eq!(pdf_page_count(&pdf), 1);
}

/// `w:gridBefore` takes the same placeholder-span path
/// (`row_grid_skip` → `grid_skip_cell`): a billion empty leading columns
/// must not become a billion-column grid.
#[test]
fn a_huge_grid_before_renders_within_budget() {
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"4675\"/></w:tblGrid>\
                <w:tr><w:trPr><w:gridBefore w:val=\"2000000000\"/></w:trPr>\
                <w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
    let pdf = docx_to_pdf(&docx(body)).expect("a clamped skip renders");
    assert_eq!(pdf_page_count(&pdf), 1);
}

/// A deeply nested theme stored under a non-XML name: admission's depth
/// scan only reads `.xml`/`.rels` names, and rendering selects the part by
/// relationship. The DOM construction cap keeps the parse bounded, where
/// the permissive parser used to recurse to the input's depth.
#[test]
fn a_renamed_deeply_nested_theme_part_renders_bounded() {
    let deep = format!(
        "<a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">{}\
         <a:clrScheme name=\"x\"><a:dk1><a:sysClr val=\"window\"/></a:dk1></a:clrScheme>\
         </a:theme>",
        "<a:e>".repeat(300_000)
    );
    let bytes = docx_with(
        "<w:p><w:r><w:t>T</w:t></w:r></w:p>",
        &[Part {
            name: "word/theme1.bin",
            content_type: "application/vnd.openxmlformats-officedocument.theme+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme",
            xml: &deep,
        }],
    );
    let pdf = docx_to_pdf(&bytes).expect("a bounded theme parses");
    assert_eq!(pdf_page_count(&pdf), 1);
}

/// A left tab stop at `1e22` twips with a dot leader: the leader interval
/// is attacker paper (`repeat` sized by the stop's coordinate, saturating
/// `as usize` to `usize::MAX`). Nonfinite or beyond-page intervals now
/// paint nothing or clamp to the line's extent.
#[test]
fn a_huge_tab_leader_renders_bounded() {
    let body = "<w:p><w:pPr><w:tabs>\
                <w:tab w:val=\"left\" w:leader=\"dot\" w:pos=\"1e22\"/>\
                </w:tabs></w:pPr>\
                <w:r><w:tab/></w:r><w:r><w:t>x</w:t></w:r></w:p>";
    let pdf = docx_to_pdf(&docx(body)).expect("a clamped leader renders");
    assert_eq!(pdf_page_count(&pdf), 1);
}
