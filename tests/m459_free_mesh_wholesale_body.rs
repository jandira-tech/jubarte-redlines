// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M459 — wholesale body MIX free-meshes shared word anchors.
//!
//! center_aligned_bold × center_alignment_2: body MIX should not be a single
//! wholesale ins+del; Word free-meshes "is"/"centered" as EQ.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn center_aligned_bold_body_mix_has_eq_anchors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("center_aligned_bold_text_id_paraid_overflow.docx");
    let b = src.join("center_alignment_demo_id_paraid_overflow_2.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    // Find MIX containing "centered" free-mesh.
    let mut rest = xml.as_str();
    let mut found = false;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        if !(after.starts_with("<w:p>") || after.starts_with("<w:p ")) {
            rest = &after[4..];
            continue;
        }
        let end = after.find("</w:p>").map(|j| j + 6).unwrap_or(after.len());
        let p = &after[..end];
        rest = &after[end..];
        if !(p.contains("centered")
            && p.contains("<w:ins")
            && (p.contains("<w:del") || p.contains("delText")))
        {
            continue;
        }
        // Must have bare EQ run with "centered" or "is" (not only inside ins/del).
        // Look for w:t outside ins/del — simple heuristic: count ins wrappers.
        let ins_count = p.matches("<w:ins").count();
        let del_count = p.matches("<w:del").count();
        assert!(
            ins_count >= 2 || del_count >= 2,
            "Word free-meshes wholesale body into multiple rev runs; ins={ins_count} del={del_count} p={p}"
        );
        // Should not be a single wholesale del of entire "This text is both..."
        assert!(
            !p.contains("This text is both centered and bold"),
            "wholesale del of entire A sentence remains; p={p}"
        );
        found = true;
        break;
    }
    assert!(found, "expected body MIX with centered free-mesh");
}
