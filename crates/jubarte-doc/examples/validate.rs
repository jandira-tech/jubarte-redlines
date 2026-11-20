// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Check that Word will open a package (Ring-1), and repair what can be
//! repaired.
//! Run: `cargo run -p jubarte-doc --example validate`

// ANCHOR: example
use jubarte_doc::validate::{findings_text, repair, validate};

const DOCX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/1686-word-compare.docx"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docx = std::fs::read(DOCX)?;

    let findings = validate(&docx)?;
    if findings.is_empty() {
        println!("no findings");
    } else {
        print!("{}", findings_text(&findings));
    }
    let fatal = findings.iter().filter(|f| f.word_fatal).count();
    println!("{fatal} finding(s) Word refuses or repairs");

    // repair() fixes every finding it can deterministically and reports
    // the ones left; parts it does not touch keep their bytes.
    let repaired = repair(&docx)?;
    println!(
        "repaired {}, remaining {}",
        repaired.repaired.len(),
        repaired.remaining.len()
    );
    Ok(())
}
// ANCHOR_END: example
