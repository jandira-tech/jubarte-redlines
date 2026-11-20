// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Audit a document for accessibility, style and structure, and refresh its
//! fields. Both need the layout pass, so both need the `render` feature.
//! Run: `cargo run -p jubarte-doc --features render --example audit_fields`

const DOCX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/agent-view/received.docx"
);

// ANCHOR: example
#[cfg(feature = "render")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use jubarte_doc::audit::audit_report;
    use jubarte_doc::fields::update_fields;

    let docx = std::fs::read(DOCX)?;

    // Every rule (an empty list), or sets and codes: "a11y", "style",
    // "structure", "HEADING_SKIP", ...
    let report = audit_report(&docx, &[])?;
    println!("rules run: {}", report.rules.join(", "));
    for f in &report.findings {
        println!(
            "{} {} {} {}: {}",
            f.severity, f.rule_set, f.code, f.location, f.message
        );
    }

    // Recompute TOC, PAGEREF, REF, NUMPAGES and SEQ results from jubarte's
    // own layout.
    let updated = update_fields(&docx)?;
    println!(
        "{} page(s), {} field(s) rewritten",
        updated.page_count,
        updated.fields.len()
    );
    Ok(())
}
// ANCHOR_END: example

#[cfg(not(feature = "render"))]
fn main() {
    let _ = DOCX;
    eprintln!("run with --features render: the audit and field updates need the layout pass");
}
