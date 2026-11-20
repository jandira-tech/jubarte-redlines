// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Remove authors and metadata before sending, and append one document
//! after another.
//! Run: `cargo run -p jubarte-doc --example scrub_append`

// ANCHOR: scrub
use jubarte_doc::append::{AppendOptions, SectionBreak, append_documents};
use jubarte_doc::document_comparer::compare_documents;
use jubarte_doc::scrub::{ScrubOptions, leaks, scrub};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/redline");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = std::fs::read(format!("{DIR}/original.docx"))?;
    let modified = std::fs::read(format!("{DIR}/modified.docx"))?;
    let redline = compare_documents(&original, &modified, "Jane Counsel")?;

    // The default turns everything on: authors become "Author", rsids,
    // document-property people and dates, and comments are removed.
    let options = ScrubOptions {
        author_alias: Some("Counsel".into()),
        ..ScrubOptions::default()
    };
    let clean = scrub(&redline, &options)?;
    // leaks() names every part that still holds the text.
    println!("before: {:?}", leaks(&redline, "Jane Counsel"));
    println!("after:  {:?}", leaks(&clean, "Jane Counsel"));
    // ANCHOR_END: scrub

    // ANCHOR: append
    let options = AppendOptions {
        section_break: SectionBreak::NextPage,
        ..AppendOptions::default()
    };
    let joined = append_documents(&original, &modified, &options)?;
    println!("appended: {} bytes", joined.docx.len());
    for warning in &joined.warnings {
        println!("warning: {warning}");
    }
    // ANCHOR_END: append
    Ok(())
}
