// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The facade in one file: compare two documents, list and settle the
//! changes, check the redline and render it to PDF, all through
//! `jubarte::`.
//! Run: `cargo run --example quickstart [OUT_DIR]`

// ANCHOR: example
use jubarte::changes::{self, ChangeFilter};
use jubarte::convert::{PdfOptions, RevisionStyle, docx_to_pdf_with};
use jubarte::document_comparer::compare_documents;
use jubarte::validate::validate;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/redline");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = std::fs::read(format!("{DIR}/original.docx"))?;
    let modified = std::fs::read(format!("{DIR}/modified.docx"))?;

    let redline = compare_documents(&original, &modified, "Reviewer")?;
    let listed = changes::list_changes(&redline)?;
    println!("{} tracked change(s)", listed.len());

    let word_fatal = validate(&redline)?.iter().filter(|f| f.word_fatal).count();
    println!("{word_fatal} Word-fatal finding(s)");

    let accepted = changes::accept_changes(&redline, &ChangeFilter::default())?;
    println!(
        "after accept all: {} change(s)",
        changes::list_changes(&accepted)?.len()
    );

    let options = PdfOptions {
        revisions: RevisionStyle::Word,
        ..PdfOptions::default()
    };
    let pdf = docx_to_pdf_with(&redline, options)?;
    println!("pdf: {} bytes", pdf.len());

    if let Some(dir) = std::env::args().nth(1) {
        std::fs::write(format!("{dir}/redline.docx"), &redline)?;
        std::fs::write(format!("{dir}/redline.pdf"), &pdf)?;
    }
    Ok(())
}
// ANCHOR_END: example
