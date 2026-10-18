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
use jubarte::inspect::{inspect_json, paragraphs, summary, tables, tables_text};

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
            vec![(vec!["p1"], "Item"), (vec!["p2"], "Qty")],
            vec![(vec!["p3", "p4"], "Bolt\nsteel"), (vec!["p5"], "40")],
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
        serde_json::json!(["p3", "p4"])
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
    assert_eq!(outer.rows[0][0].paragraph_ids, ["p0", "p3"]);
    assert_eq!(outer.rows[0][0].text, "Outer\n");
    assert_eq!(outer.rows[0][1].text, "Right");
    let inner = &found[1];
    assert_eq!(inner.index, 1);
    assert_eq!(inner.rows[0][0].paragraph_ids, ["p1"]);
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
fn tables_text_prints_a_grid_line_and_escaped_cells() {
    let text = tables_text(&tables(&two_by_two()).unwrap());
    assert!(
        text.starts_with("table 0: 2x2 header_rows=1 widths=6000,3360\n"),
        "{text}"
    );
    assert!(text.ends_with('\n'), "{text}");
    assert_eq!(tables_text(&[]), "no tables\n");
}

#[test]
fn the_cli_prints_each_table_grid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("table.docx");
    std::fs::write(&path, two_by_two()).unwrap();
    let out = Command::new(BIN)
        .args(["read", path.to_str().unwrap(), "--tables"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    // inspect --tables prints the same grid until inspect retires.
    let inspect = Command::new(BIN)
        .args(["inspect", path.to_str().unwrap(), "--tables"])
        .output()
        .unwrap();
    assert_eq!(inspect.stdout, out.stdout);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, tables_text(&tables(&two_by_two()).unwrap()));
    assert!(
        stdout.contains("table 0: 2x2 header_rows=1 widths=6000,3360"),
        "{stdout}"
    );
    assert!(stdout.contains("p1=Item\tp2=Qty"), "{stdout}");
    assert!(stdout.contains("p3,p4=Bolt\\nsteel\tp5=40"), "{stdout}");
}

#[test]
fn the_cli_says_when_there_are_no_tables_and_refuses_json_with_tables() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plain.docx");
    std::fs::write(&path, docx(&para("No table."))).unwrap();
    let out = Command::new(BIN)
        .args(["read", path.to_str().unwrap(), "--tables"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "no tables\n");
    for task in ["inspect", "read"] {
        let both = Command::new(BIN)
            .args([task, path.to_str().unwrap(), "--tables", "--json"])
            .output()
            .unwrap();
        assert_eq!(both.status.code(), Some(2), "{task}: {both:?}");
    }
}

#[test]
fn read_json_of_a_cell_paragraph_holds_that_paragraph_only() {
    // The view prints a table whole, but `read -p p5 --json` names one cell
    // paragraph: its JSON must hold p5, not the table's first cell.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("table.docx");
    std::fs::write(&path, two_by_two()).unwrap();
    let ids = |pick: &str| -> Vec<String> {
        let out = Command::new(BIN)
            .args(["read", path.to_str().unwrap(), "-p", pick, "--json"])
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
        let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        json["paragraphs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["id"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(ids("p5"), ["p5"]);
    assert_eq!(ids("p2-p3"), ["p2", "p3"]);
    assert_eq!(ids("p0-p1"), ["p0", "p1"]);
    assert_eq!(ids("t0"), ["p1", "p2", "p3", "p4", "p5"]);
}

// CLI integration tests: real temporary packages and subprocesses exercise
// selection, exit status and the exact inventory wire format together.
const TABLE_ENTRIES: [&str; 3] = [
    "table 0: 1x2 header_rows=1 widths=6000,3360\np1=Alice\tp2=One\n",
    "table 1: 1x2 header_rows=1 widths=6000,3360\np4=Bob\tp5=Two\n",
    "table 2: 1x2 header_rows=1 widths=6000,3360\np7=Carol Diaz\tp8=Three\n",
];

fn three_tables() -> Vec<u8> {
    docx(
        &(para("Before.")
            + &table(true, &[cell(&["Alice"]) + &cell(&["One"])])
            + &para("Between.")
            + &table(true, &[cell(&["Bob"]) + &cell(&["Two"])])
            + &para("Between again.")
            + &table(true, &[cell(&["Carol Diaz"]) + &cell(&["Three"])])
            + &para("After.")),
    )
}

fn read_table_fixture(source: &[u8], shorthand: bool, flags: &[&str]) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("selection.docx");
    std::fs::write(&path, source).unwrap();
    let mut command = Command::new(BIN);
    if !shorthand {
        command.arg("read");
    }
    command.arg(path).args(flags).output().unwrap()
}

fn selected_inventory(spec: &str, indices: &[usize]) {
    let source = three_tables();
    let full = read_table_fixture(&source, false, &["--tables"]);
    assert!(full.status.success(), "{full:?}");
    assert_eq!(full.stdout, TABLE_ENTRIES.concat().as_bytes());
    let out = read_table_fixture(&source, false, &["--tables", "-p", spec]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{spec}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let expected: String = indices.iter().map(|&i| TABLE_ENTRIES[i]).collect();
    assert_summary_and_view(&source, spec, &expected, &out);
    assert!(out.stderr.is_empty(), "{out:?}");
}

fn assert_summary_and_view(source: &[u8], spec: &str, summary: &str, out: &std::process::Output) {
    let view = read_table_fixture(source, false, &["-p", spec]);
    assert!(view.status.success(), "{view:?}");
    let expected = [summary.as_bytes(), b"\n", &view.stdout].concat();
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&expected),
        "{spec}: selected grid, blank line, and unchanged view"
    );
}

#[test]
fn table_inventory_selects_one_document_index() {
    selected_inventory("t0", &[0]);
    selected_inventory("t1", &[1]);
}

#[test]
fn table_inventory_selects_a_list() {
    selected_inventory("t0,t2", &[0, 2]);
}

#[test]
fn table_inventory_sorts_and_deduplicates_a_list() {
    selected_inventory("t2,t0,t2", &[0, 2]);
    selected_inventory("t2,t0", &[0, 2]);
}

#[test]
fn table_inventory_selects_a_cell_paragraph() {
    selected_inventory("p4", &[1]);
}

#[test]
fn table_inventory_selects_a_paragraph_range() {
    selected_inventory("p4-p5", &[1]);
}

#[test]
fn table_inventory_selects_paragraphs_from_several_tables() {
    selected_inventory("p8,p1,p4", &[0, 1, 2]);
}

fn refused_inventory(spec: &str, message: &str) {
    let out = read_table_fixture(&three_tables(), false, &["--tables", "-p", spec]);
    assert_eq!(out.status.code(), Some(2), "{spec}: {out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains(message), "{spec}: {stderr}");
}

#[test]
fn table_inventory_refuses_a_non_table_paragraph() {
    refused_inventory(
        "p3",
        "p3 is not in a table; --tables -p takes tN, or a pN inside a table",
    );
}

#[test]
fn table_inventory_refuses_mixed_table_and_non_table_items() {
    refused_inventory(
        "t0,p3",
        "p3 is not in a table; --tables -p takes tN, or a pN inside a table",
    );
}

#[test]
fn table_inventory_refuses_the_first_non_table_paragraph_in_a_range() {
    refused_inventory(
        "p4-p8",
        "p6 is not in a table; --tables -p takes tN, or a pN inside a table",
    );
}

#[test]
fn table_inventory_refuses_missing_tables_with_the_view_diagnosis() {
    let view = read_table_fixture(&three_tables(), false, &["-p", "t9"]);
    let stderr = String::from_utf8(view.stderr).unwrap();
    let diagnosis = stderr.trim().strip_prefix("error: ").unwrap();
    refused_inventory("t9", diagnosis);
}

#[test]
fn table_inventory_refuses_invalid_selection_grammar() {
    for spec in ["t0-t2", "p5-p4", "Alice", "p99", ","] {
        let view = read_table_fixture(&three_tables(), false, &["-p", spec]);
        let stderr = String::from_utf8(view.stderr).unwrap();
        let diagnosis = stderr.trim().strip_prefix("error: ").unwrap();
        refused_inventory(spec, diagnosis);
    }
}

#[test]
fn table_inventory_shorthand_selects_the_same_entry() {
    let source = three_tables();
    let flags = ["--tables", "-p", "t0"];
    let explicit = read_table_fixture(&source, false, &flags);
    let shorthand = read_table_fixture(&source, true, &flags);
    assert_eq!(
        shorthand.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&shorthand.stderr)
    );
    assert_summary_and_view(&source, "t0", TABLE_ENTRIES[0], &shorthand);
    assert_eq!(shorthand.stdout, explicit.stdout);
}

fn hard_tables() -> Vec<u8> {
    include_bytes!("fixtures/from-docx/critic/synthetic/hard-tables.docx").to_vec()
}

#[test]
fn table_inventory_selects_a_top_level_table_and_its_nested_tables() {
    let source = hard_tables();
    let full = read_table_fixture(&source, false, &["--tables"]);
    assert!(full.status.success(), "{full:?}");
    let text = String::from_utf8_lossy(&full.stdout);
    assert!(text.contains("table 0: 3x3"), "{text}");
    assert!(text.contains("table 1:"), "{text}");
    let out = read_table_fixture(&source, false, &["--tables", "-p", "t0"]);
    assert!(out.status.success(), "{out:?}");
    assert_summary_and_view(
        &source,
        "t0",
        &String::from_utf8(full.stdout).unwrap(),
        &out,
    );
}

#[test]
fn table_inventory_selects_a_nested_tables_own_paragraph() {
    let source = hard_tables();
    let full = read_table_fixture(&source, false, &["--tables"]);
    let out = read_table_fixture(&source, false, &["--tables", "-p", "p9"]);
    assert!(out.status.success(), "{out:?}");
    assert_summary_and_view(
        &source,
        "p9",
        &String::from_utf8(full.stdout).unwrap(),
        &out,
    );
}

#[test]
fn table_inventory_refuses_a_nested_inventory_index_as_a_view_table() {
    let source = hard_tables();
    let view = read_table_fixture(&source, false, &["-p", "t1"]);
    assert_eq!(view.status.code(), Some(1), "{view:?}");
    let out = read_table_fixture(&source, false, &["--tables", "-p", "t1"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(out.stderr, view.stderr);
    assert!(out.stdout.is_empty());
}

fn deeply_nested_tables() -> Vec<u8> {
    let deepest = table(false, &[cell(&["Carol Diaz"]) + &cell(&["Three"])]);
    let middle_cell = format!("<w:tc>{}{deepest}<w:p/></w:tc>", para("Bob"));
    let middle = table(false, &[middle_cell + &cell(&["Two"])]);
    let outer_cell = format!("<w:tc>{}{middle}<w:p/></w:tc>", para("Alice"));
    let outer = table(false, &[outer_cell + &cell(&["One"])]);
    docx(&(outer + &para("Between.") + &table(false, &[cell(&["Alice"]) + &cell(&["After"])])))
}

#[test]
fn table_inventory_selection_follows_top_level_indices_at_any_depth() {
    let source = deeply_nested_tables();
    let found = tables(&source).unwrap();
    assert_eq!(found.len(), 4);
    for spec in ["t0", "p2"] {
        let out = read_table_fixture(&source, false, &["--tables", "-p", spec]);
        assert!(out.status.success(), "{out:?}");
        assert_summary_and_view(&source, spec, &tables_text(&found[..3]), &out);
    }
    let out = read_table_fixture(&source, false, &["--tables", "-p", "t1"]);
    assert!(out.status.success(), "{out:?}");
    assert_summary_and_view(&source, "t1", &tables_text(&found[3..]), &out);
    let view = read_table_fixture(&source, false, &["-p", "t1"]);
    assert!(view.status.success(), "{view:?}");
    assert!(String::from_utf8_lossy(&view.stdout).contains("|Alice|After|"));
}

#[test]
fn table_inventory_without_selection_and_the_table_view_are_unchanged() {
    let source = three_tables();
    let full = read_table_fixture(&source, false, &["--tables"]);
    assert_eq!(full.status.code(), Some(0));
    assert_eq!(full.stdout, TABLE_ENTRIES.concat().as_bytes());
    let view = read_table_fixture(&source, false, &["-p", "t1"]);
    assert_eq!(view.status.code(), Some(0), "{view:?}");
    let stdout = String::from_utf8(view.stdout).unwrap();
    assert!(stdout.contains("range: t1 of p0-p9\n"), "{stdout}");
    assert!(
        stdout.ends_with(
            "<!-- t1 1x2, cells p4-p5 by row, header row repeats -->\n|Bob|Two|\n|-|-|\n"
        ),
        "{stdout}"
    );
}

#[test]
fn table_inventory_still_refuses_other_selection_flags() {
    for flags in [
        vec!["--tables", "--head", "3"],
        vec!["--tables", "--tail", "3"],
        vec!["--tables", "--changed"],
        vec!["--tables", "--by", "Alice"],
    ] {
        let out = read_table_fixture(&three_tables(), false, &flags);
        assert_eq!(out.status.code(), Some(2), "{flags:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{out:?}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(
            stderr.contains("Usage: jubarte read [OPTIONS] <FILE>"),
            "{stderr}"
        );
    }
}
