// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2024-2026 SylphxAI
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A real-world Word sample read as Markdown.
//!
//! `fixtures/from-docx/equations.docx` comes from microsoft/markitdown
//! (packages/markitdown/tests/test_files, MIT licence, Copyright (c) Microsoft
//! Corporation), by way of anymd's tests.

use jubarte::markdown::{MarkdownOptions, docx_to_markdown};

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/from-docx")
            .join(name),
    )
    .unwrap()
}

#[test]
fn word_equations_become_latex() {
    let markdown = docx_to_markdown(&fixture("equations.docx"), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert!(markdown.starts_with("For $m=1$,\n\n$$"), "{markdown}");
    assert!(markdown.contains(r"\frac{mλ}{a}"), "{markdown}");
    assert!(markdown.contains(r"{10}^{-6}"), "{markdown}");
}

#[test]
fn corrupted_samples_never_panic() {
    let original = fixture("equations.docx");
    for cut in (0..original.len()).step_by(997) {
        let _ = docx_to_markdown(&original[..cut], &MarkdownOptions::default());
        let mut flipped = original.clone();
        flipped[cut] ^= 0x5a;
        let _ = docx_to_markdown(&flipped, &MarkdownOptions::default());
    }
}
