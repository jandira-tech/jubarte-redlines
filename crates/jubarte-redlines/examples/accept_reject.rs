// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Accept or reject every tracked change in a redline.
//! Run: `cargo run -p jubarte-redlines --example accept_reject`

// ANCHOR: example
use jubarte_redlines::comparer::WmlComparerSettings;
use jubarte_redlines::document_comparer::{
    accept_revisions, compare_documents, get_revisions, reject_revisions,
};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/redline");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = std::fs::read(format!("{DIR}/original.docx"))?;
    let modified = std::fs::read(format!("{DIR}/modified.docx"))?;
    let redline = compare_documents(&original, &modified, "Reviewer")?;

    // Accept all: the document as the modified side reads.
    let accepted = accept_revisions(&redline)?;
    // Reject all: the document as the original side reads.
    let rejected = reject_revisions(&redline)?;

    let settings = WmlComparerSettings::default();
    println!(
        "redline:  {} revision(s)",
        get_revisions(&redline, &settings)?.len()
    );
    println!(
        "accepted: {} revision(s)",
        get_revisions(&accepted, &settings)?.len()
    );
    println!(
        "rejected: {} revision(s)",
        get_revisions(&rejected, &settings)?.len()
    );
    Ok(())
}
// ANCHOR_END: example
