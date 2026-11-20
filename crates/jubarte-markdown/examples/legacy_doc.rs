// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Read a Word 97-2003 .doc: its text, headings and tables, as Markdown or
//! as a new .docx.
//! Run: `cargo run -p jubarte-markdown --example legacy_doc`

// ANCHOR: example
use jubarte_markdown::legacy_doc::{doc_to_docx, doc_to_markdown, is_compound_file};

const DOC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/legacy/services.doc"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read(DOC)?;
    // A .doc is an OLE compound file; a .docx is a ZIP.
    println!("compound file: {}", is_compound_file(&bytes));
    println!("{}", doc_to_markdown(&bytes)?);
    let docx = doc_to_docx(&bytes)?;
    println!("docx: {} bytes", docx.len());
    Ok(())
}
// ANCHOR_END: example
