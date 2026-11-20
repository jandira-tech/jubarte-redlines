// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Tune a comparison with `WmlComparerSettings`, then list the revisions
//! the redline holds.
//! Run: `cargo run -p jubarte-redlines --example compare_settings`

// ANCHOR: example
use jubarte_redlines::comparer::{CompareMode, WmlComparerSettings};
use jubarte_redlines::document_comparer::{
    compare_documents_with_settings, get_revisions, revisions_to_json,
};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/redline");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = std::fs::read(format!("{DIR}/original.docx"))?;
    let modified = std::fs::read(format!("{DIR}/modified.docx"))?;

    // Start from a preset, then change only the safe scalars.
    let settings = WmlComparerSettings::new(CompareMode::Word)
        .with_author("Legal")
        .with_date("2026-01-15T09:00:00Z");
    let redline = compare_documents_with_settings(&original, &modified, &settings)?;

    let revisions = get_revisions(&redline, &settings)?;
    println!("{} revision(s)", revisions.len());
    for r in &revisions {
        let text: String = r.text.as_deref().unwrap_or("").chars().take(40).collect();
        println!(
            "{:?} by {}: {text:?}…",
            r.revision_type,
            r.author.as_deref().unwrap_or("?")
        );
    }
    // The same list in the JSON shape the Python and WASM bindings return.
    let json = revisions_to_json(&revisions);
    println!("JSON: {} bytes", json.len());

    // The PowerTools-faithful preset: no Word-visual passes, no moves.
    let classic = WmlComparerSettings::new(CompareMode::PowerTools).with_author("Legal");
    let classic_redline = compare_documents_with_settings(&original, &modified, &classic)?;
    println!("powertools redline: {} bytes", classic_redline.len());
    Ok(())
}
// ANCHOR_END: example
