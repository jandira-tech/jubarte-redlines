// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! C5-content / hr_onboarding class: unrelated tables (zero body Jaccard)
//! must not cell-merge. Word pure-dels A table and pure-ins B table.

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

fn doc_body(dom: &mut Dom, inner: &str) -> (NodeId, NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

fn para(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

fn cell(text: &str) -> String {
    format!("<w:tc><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>")
}

fn row(cells: &[&str]) -> String {
    let inner: String = cells.iter().map(|c| cell(c)).collect();
    format!("<w:tr>{inner}</w:tr>")
}

fn table(rows: &[&[&str]]) -> String {
    let body: String = rows.iter().map(|r| row(r)).collect();
    format!("<w:tbl><w:tblPr/><w:tblGrid/>{body}</w:tbl>")
}

#[test]
fn unrelated_checklist_table_does_not_cell_merge_into_prepared_for() {
    let mut dom = Dom::new();
    // Short base: title + 3-col checklist table (hr_onboarding shape).
    let base = [
        para("HR Onboarding Checklist"),
        table(&[
            &["Step", "Task", "Done"],
            &["1", "Sign NDA", "Yes"],
            &["2", "Setup laptop", "No"],
        ]),
    ]
    .concat();
    // Long next: title + ≥3 tables (gate requires multi-table asymmetry;
    // single×single zero-Jaccard still cell-merges like Word project_tasks×q1).
    let next = [
        para("Microsoft Word vs. Google Docs"),
        para("A comprehensive evidence-backed demonstration"),
        table(&[&[
            "Positioning thesis",
            "Word provides real-time collaboration people expect from modern cloud editors",
        ]]),
        table(&[
            &["Prepared for", "Executive decision-makers"],
            &["Prepared by", "Microsoft Word capability team"],
            &["Date", "2026-07-03"],
        ]),
        table(&[&["Bottom line", "Word is built for professional production"]]),
        table(&[&["Source", "Microsoft Support Track Changes"]]),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let tables: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::tbl()))
        .collect();
    // Prepared for must appear in a table that does NOT also hold "Sign NDA".
    let mut found_clean_prepared = false;
    let mut found_mixed = false;
    for &t in &tables {
        let ser = dom.serialize_element(t);
        let has_prep = ser.contains("Prepared for");
        let has_sign = ser.contains("Sign NDA");
        let has_step = ser.contains("Step") && ser.contains("Task");
        if has_prep && (has_sign || has_step) {
            found_mixed = true;
        }
        if has_prep && !has_sign && !has_step {
            found_clean_prepared = true;
        }
    }
    assert!(
        !found_mixed,
        "checklist cells must not fold into Prepared for / Positioning tables"
    );
    assert!(
        found_clean_prepared || tables.len() >= 2,
        "expect intact B tables (Prepared for) separate from A checklist; n_tables={}",
        tables.len()
    );
}

#[test]
fn related_same_slot_tables_still_may_cell_merge() {
    // Control: same skeleton, shared header tokens — cell merge is allowed.
    let mut dom = Dom::new();
    let base = table(&[&["Ticket ID", "Status"], &["T-1", "Open"]]);
    let next = table(&[&["Ticket ID", "Status", "Owner"], &["T-1", "Open", "Ada"]]);
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let tables: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::tbl()))
        .collect();
    assert!(
        !tables.is_empty(),
        "related tables must still produce table output"
    );
    // At least one table should carry Ticket ID (shared content).
    let any_ticket = tables
        .iter()
        .any(|&t| dom.serialize_element(t).contains("Ticket ID"));
    assert!(any_ticket, "shared Ticket ID header should survive");
}
