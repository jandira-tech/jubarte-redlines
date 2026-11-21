// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Read a document the way an edit plan addresses it: paragraph ids,
//! styles and text, and the package summary.
//! Run: `cargo run -p jubarte-doc --example inspect`

// ANCHOR: example
use jubarte_doc::inspect::{paragraphs, summary};

const DOCX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/redline/original.docx"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docx = std::fs::read(DOCX)?;
    for p in paragraphs(&docx)? {
        let style = p.style.as_deref().unwrap_or("Normal");
        println!("{:>4} [{style}] {}", p.id, p.text);
    }
    println!("{:#?}", summary(&docx)?);
    Ok(())
}
// ANCHOR_END: example
