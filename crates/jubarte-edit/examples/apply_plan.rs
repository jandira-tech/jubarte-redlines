// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Apply an edit plan as tracked changes: build a small contract in
//! memory, find the paragraph to edit, and get back the clean copy and
//! the redline.
//! Run: `cargo run -p jubarte-edit --example apply_plan [OUT_DIR]`

// ANCHOR: example
use jubarte_edit::edit::{apply_plan_json, dispatched_operation_kinds};
use jubarte_edit::inspect::paragraphs;
use jubarte_edit::markdown::{DocxOptions, markdown_to_docx};

const CONTRACT: &str = "# Services Agreement

The Supplier shall deliver the Goods within thirty (30) days of the order.

Either party may terminate this Agreement on written notice.
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = markdown_to_docx(CONTRACT, &DocxOptions::default())?.docx;

    // An operation names its paragraph by id and quotes text that occurs
    // there exactly once.
    let target = paragraphs(&source)?
        .into_iter()
        .find(|p| p.text.contains("thirty (30) days"))
        .ok_or("paragraph not found")?;

    let plan = format!(
        r#"{{"schema_version": 1, "author": "Reviewer", "operations": [
            {{"kind": "replace", "paragraph": "{}",
              "find": "thirty (30) days", "replacement": "sixty (60) days"}},
            {{"kind": "comment", "paragraph": "{}",
              "find": "Supplier", "text": "Confirm the supplier's legal name."}}
        ]}}"#,
        target.id, target.id
    );
    let edited = apply_plan_json(&source, &plan)?;
    println!(
        "clean: {} bytes, redline: {} bytes",
        edited.clean.len(),
        edited.redline.len()
    );
    println!(
        "report ok: {}, author {}",
        edited.report.ok, edited.report.author
    );

    // Every operation kind a plan may use.
    println!("kinds: {}", dispatched_operation_kinds().join(", "));

    if let Some(dir) = std::env::args().nth(1) {
        std::fs::write(format!("{dir}/contract_clean.docx"), &edited.clean)?;
        std::fs::write(format!("{dir}/contract_redline.docx"), &edited.redline)?;
    }
    Ok(())
}
// ANCHOR_END: example
