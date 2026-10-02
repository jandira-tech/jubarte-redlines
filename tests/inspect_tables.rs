// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `inspect` reads tables as grids: each cell's text and the paragraph ids an
//! edit plan addresses, the header rows and the grid widths. The read side of
//! converting a table to CSV.

mod common;

use std::process::Command;

use common::docx::{docx, para};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::{inspect_json, paragraphs, summary, tables};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn cell(paragraphs: &[&str]) -> String {
    let body: String = paragraphs
        .iter()
        .map(|text| format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>"))
        .collect();
    format!("<w:tc><w:tcPr><w:tcW w:w=\"4680\" w:type=\"dxa\"/></w:tcPr>{body}</w:tc>")
}

fn table(header: bool, rows: &[String]) -> String {
    let rows: String = rows
        .iter()
        .enumerate()
        .map(|(i, cells)| {
            let tr_pr = if header && i == 0 {
                "<w:trPr><w:tblHeader/></w:trPr>"
            } else {
                ""
            };
            format!("<w:tr>{tr_pr}{cells}</w:tr>")
        })
        .collect();
    format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"9360\" w:type=\"dxa\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"6000\"/><w:gridCol w:w=\"3360\"/></w:tblGrid>{rows}</w:tbl>"
    )
}

fn two_by_two() -> Vec<u8> {
    let grid = table(
        true,
        &[
            cell(&["Item"]) + &cell(&["Qty"]),
            cell(&["Bolt", "steel"]) + &cell(&["40"]),
        ],
    );
    docx(&(para("Intro.") + &grid + &para("Outro.")))
}

#[test]
fn a_table_reports_cell_text_ids_header_rows_and_widths() {
    let source = two_by_two();
    let found = tables(&source).unwrap();
    assert_eq!(found.len(), 1);
    let t = &found[0];
    assert_eq!(t.index, 0);
    assert_eq!(t.header_rows, 1);
    assert_eq!(t.widths_dxa, [6000, 3360]);
    let grid: Vec<Vec<(Vec<&str>, &str)>> = t
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|c| {
                    (
                        c.paragraph_ids.iter().map(String::as_str).collect(),
                        c.text.as_str(),
                    )
                })
                .collect()
        })
        .collect();
    assert_eq!(
        grid,
        vec![
            vec![(vec!["body:p:1"], "Item"), (vec!["body:p:2"], "Qty")],
            vec![
                (vec!["body:p:3", "body:p:4"], "Bolt\nsteel"),
                (vec!["body:p:5"], "40")
            ],
        ]
    );
    // The ids are the ones paragraphs() reports.
    let paras = paragraphs(&source).unwrap();
    assert_eq!(paras[5].text, "40");
    assert!(paras[5].in_table);
}

#[test]
fn a_cell_id_round_trips_into_an_edit() {
    let source = two_by_two();
    let id = tables(&source).unwrap()[0].rows[1][1].paragraph_ids[0].clone();
    let plan = EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"A","operations":[{{"kind":"replace","paragraph":"{id}","find":"40","replacement":"45"}}]}}"#
    ))
    .unwrap();
    let out = apply_plan(&source, &plan).unwrap();
    assert_eq!(tables(&out.clean).unwrap()[0].rows[1][1].text, "45");
}

#[test]
fn the_snapshot_carries_the_tables() {
    let snapshot: serde_json::Value =
        serde_json::from_str(&inspect_json(&two_by_two()).unwrap()).unwrap();
    let t = &snapshot["tables"][0];
    assert_eq!(t["index"], 0);
    assert_eq!(t["header_rows"], 1);
    assert_eq!(t["widths_dxa"], serde_json::json!([6000, 3360]));
    assert_eq!(t["rows"][0][1]["text"], "Qty");
    assert_eq!(
        t["rows"][1][0]["paragraph_ids"],
        serde_json::json!(["body:p:3", "body:p:4"])
    );
    let plain: serde_json::Value =
        serde_json::from_str(&inspect_json(&docx(&para("No table."))).unwrap()).unwrap();
    assert_eq!(plain["tables"], serde_json::json!([]));
}

#[test]
fn a_nested_table_is_its_own_entry() {
    let inner = table(false, &[cell(&["Inner"]) + &cell(&["Cell"])]);
    let outer_cell = format!(
        "<w:tc><w:tcPr><w:tcW w:w=\"4680\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>Outer</w:t></w:r></w:p>{inner}<w:p/></w:tc>"
    );
    let outer = table(false, &[outer_cell + &cell(&["Right"])]);
    let source = docx(&(outer + &para("After.")));
    let found = tables(&source).unwrap();
    assert_eq!(found.len(), summary(&source).unwrap().tables);
    assert_eq!(found.len(), 2);
    let outer = &found[0];
    assert_eq!(outer.header_rows, 0);
    // The outer cell keeps its own paragraphs; the nested ones are the
    // inner table's.
    assert_eq!(outer.rows[0][0].paragraph_ids, ["body:p:0", "body:p:3"]);
    assert_eq!(outer.rows[0][0].text, "Outer\n");
    assert_eq!(outer.rows[0][1].text, "Right");
    let inner = &found[1];
    assert_eq!(inner.index, 1);
    assert_eq!(inner.rows[0][0].paragraph_ids, ["body:p:1"]);
    assert_eq!(inner.rows[0][1].text, "Cell");
}

#[test]
fn unreadable_grid_widths_read_as_zero() {
    let grid = "<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w=\"wide\"/><w:gridCol/></w:tblGrid>\
                <w:tr><w:tc><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr></w:tbl><w:p/>";
    let found = tables(&docx(grid)).unwrap();
    assert_eq!(found[0].widths_dxa, [0, 0]);
    assert_eq!(found[0].rows[0][0].text, "");
}

#[test]
fn the_cli_prints_each_table_grid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("table.docx");
    std::fs::write(&path, two_by_two()).unwrap();
    let out = Command::new(BIN)
        .args(["inspect", path.to_str().unwrap(), "--tables"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("table 0: 2x2 header_rows=1 widths=6000,3360"),
        "{stdout}"
    );
    assert!(stdout.contains("body:p:1=Item\tbody:p:2=Qty"), "{stdout}");
    assert!(
        stdout.contains("body:p:3,body:p:4=Bolt\\nsteel\tbody:p:5=40"),
        "{stdout}"
    );
}

#[test]
fn the_cli_says_when_there_are_no_tables_and_refuses_json_with_tables() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plain.docx");
    std::fs::write(&path, docx(&para("No table."))).unwrap();
    let out = Command::new(BIN)
        .args(["inspect", path.to_str().unwrap(), "--tables"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "no tables\n");
    let both = Command::new(BIN)
        .args(["inspect", path.to_str().unwrap(), "--tables", "--json"])
        .output()
        .unwrap();
    assert!(!both.status.success(), "{both:?}");
}
