// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Admit a .docx under a budget, open it as an OPC package, list its parts
//! and read the main document part.
//! Run: `cargo run -p jubarte-opc --example open_package`

// ANCHOR: example
use jubarte_opc::admission::{InputLimits, admit};
use jubarte_opc::opc::PartFs;

const DOCX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/redline/original.docx"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read(DOCX)?;

    // Admission reads the ZIP directory and refuses an oversized or hostile
    // package before anything inflates it.
    let admitted = admit(&bytes, InputLimits::default())?;
    println!(
        "{} entries, {} inflated bytes, main part {}",
        admitted.entries, admitted.inflated_bytes, admitted.main_part
    );

    let package = PartFs::open(&bytes)?;
    for part in package.parts() {
        println!("  {part}");
    }

    let main = package
        .main_document_part()
        .ok_or("no main document part")?;
    let xml = package.part_string(&main).ok_or("main part is not text")?;
    println!("{main}: {} characters of XML", xml.len());

    // Writing the package back gives a .docx again.
    let round_trip = package.to_zip()?;
    println!("re-zipped: {} bytes", round_trip.len());
    Ok(())
}
// ANCHOR_END: example
