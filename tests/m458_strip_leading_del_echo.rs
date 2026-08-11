// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M458 — strip leading del echoing previous pure-I first token.
//!
//! right_aligned×right_alignment_2: body MIX must not lead with del "This"
//! after pure-I "This document…".

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn right_align_body_mix_no_leading_del_this() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("right_aligned_italic_demo_id_paraid_overflow.docx");
    let b = src.join("right_alignment_demo_id_paraid_overflow_2.docx");
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

    let mut rest = xml.as_str();
    let mut checked = false;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        if !(after.starts_with("<w:p>") || after.starts_with("<w:p ")) {
            rest = &after[4..];
            continue;
        }
        let end = after.find("</w:p>").map(|j| j + 6).unwrap_or(after.len());
        let p = &after[..end];
        rest = &after[end..];
        if !(p.contains("aligned")
            && p.contains("<w:ins")
            && (p.contains("<w:del") || p.contains("delText")))
        {
            continue;
        }
        // First delText should not be This.
        if let Some(i) = p.find("<w:delText") {
            let slice = &p[i..];
            let end_tag = slice.find('>').unwrap_or(0);
            let after_open = &slice[end_tag + 1..];
            let text_end = after_open.find('<').unwrap_or(after_open.len());
            let text = &after_open[..text_end];
            assert!(
                !text.trim().eq_ignore_ascii_case("This")
                    && !text.trim().to_ascii_lowercase().starts_with("this "),
                "leading del must not echo prev pure-I This; delText={text:?} p={p}"
            );
        }
        checked = true;
        break;
    }
    assert!(checked, "expected body MIX with aligned free-mesh");
}
