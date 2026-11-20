// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Compare two Word documents into a tracked-changes redline.
//! Run: `cargo run -p jubarte-redlines --example compare [OUT.docx]`

// ANCHOR: example
use jubarte_redlines::document_comparer::compare_documents;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/redline");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = std::fs::read(format!("{DIR}/original.docx"))?;
    let modified = std::fs::read(format!("{DIR}/modified.docx"))?;

    // Every revision is stamped with this author; the date is a fixed
    // epoch, so the same inputs always give the same bytes.
    let redline = compare_documents(&original, &modified, "Reviewer")?;
    println!("redline: {} bytes", redline.len());

    if let Some(out) = std::env::args().nth(1) {
        std::fs::write(&out, &redline)?;
        println!("wrote {out}");
    }
    Ok(())
}
// ANCHOR_END: example
