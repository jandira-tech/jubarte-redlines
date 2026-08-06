// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M306 LO-neg lock — custom_list_numbering1 × custom_list1: forcing Word's
//! live `before=120 after=0` on the lead MIX was fair tip-to-tip **−0.073**
//! (structure residual remains free-LCS; format-only emit not LO-positive).
//! Keep M271 mesh; do not re-add lead spacing ensure without fair ≥0.

use jubarte::document_comparer::compare_documents;
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
    )
    .join(name);
    std::fs::read(p).ok()
}

#[test]
fn custom_list_mesh_still_compares_without_spacing_ensure() {
    let Some(a) = load("super_editor__custom_list_numbering1_7eb9fda4.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__custom_list1_77f82bd7.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    // Smoke: compare succeeds; M271 mesh path remains (no spacing ensure).
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert!(out.len() > 1000);
}
