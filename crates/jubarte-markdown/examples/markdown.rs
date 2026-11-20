// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Markdown in and out of Word: CriticMarkup becomes tracked changes and
//! comments, and a .docx reads back as Markdown.
//! Run: `cargo run -p jubarte-markdown --example markdown [OUT.docx]`

// ANCHOR: example
use jubarte_markdown::markdown::{
    DocxOptions, MarkdownOptions, diff_markdown, docx_to_markdown, markdown_to_docx,
};

const SOURCE: &str = "# Services Agreement

Payment is due in {~~thirty~>forty-five~~} days.

The supplier {++promptly ++}delivers the goods.{>>Should this be a deadline?<<}
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Markdown to .docx: the substitution and insertion become tracked
    // changes by the author, the comment a Word comment.
    let options = DocxOptions {
        author: "Reviewer".into(),
        ..DocxOptions::default()
    };
    let written = markdown_to_docx(SOURCE, &options)?;
    println!(
        "docx: {} bytes, {} warning(s)",
        written.docx.len(),
        written.warnings.len()
    );
    if let Some(out) = std::env::args().nth(1) {
        std::fs::write(&out, &written.docx)?;
    }

    // .docx to Markdown: tracked changes and comments come back as
    // CriticMarkup.
    let read = docx_to_markdown(&written.docx, &MarkdownOptions::default())?;
    println!("{}", read.markdown);

    // Two Markdown texts diffed as CriticMarkup.
    print!(
        "{}",
        diff_markdown("Payment in 30 days.\n", "Payment in 45 days.\n")
    );
    Ok(())
}
// ANCHOR_END: example
