// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! List the tracked changes of a redline and settle some of them.
//! Run: `cargo run -p jubarte-doc --example changes`

// ANCHOR: example
use jubarte_doc::changes::{self, ChangeFilter, ChangeKind};
use jubarte_doc::document_comparer::compare_documents;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/redline");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = std::fs::read(format!("{DIR}/original.docx"))?;
    let modified = std::fs::read(format!("{DIR}/modified.docx"))?;
    let redline = compare_documents(&original, &modified, "Reviewer")?;

    // Logical changes: a change Word wrote as several marks is one entry
    // (its id joins them, as in `r2+1`), and a deletion followed by its
    // author's insertion is one replacement.
    let listed = changes::list_logical_changes(&redline)?;
    println!("{} logical change(s)", listed.len());
    for change in listed.iter().take(5) {
        let text: String = change.text.chars().take(40).collect();
        println!("{} {:?} {text:?}…", change.id, change.kind);
    }

    // Accept the insertions only; every other change stays tracked.
    let filter = ChangeFilter {
        kinds: Some(vec![ChangeKind::Insertion]),
        ..ChangeFilter::default()
    };
    let partly = changes::accept_changes(&redline, &filter)?;
    println!(
        "{} change(s) left",
        changes::list_logical_changes(&partly)?.len()
    );

    // Reject one change by its id.
    if let Some(first) = listed.first() {
        let one = changes::reject_changes(&redline, &ChangeFilter::ids([first.id.as_str()]))?;
        println!(
            "{} change(s) left",
            changes::list_logical_changes(&one)?.len()
        );
    }
    Ok(())
}
// ANCHOR_END: example
