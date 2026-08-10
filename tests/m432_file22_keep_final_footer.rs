// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M432 — multi-section unique footers: keep body-final footerReference.
//!
//! file_22 has 20 mid-section footers (distinct rIds) + final footer. Word's
//! redline keeps the final footerReference. Pre-M432 strip treated any mid
//! (footer, default) slot as inherited and deleted the final ref.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn file_22_x_file_23_keeps_final_section_footer_ref() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join(
        "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized",
    );
    let a = src.join("file_22.docx");
    let b = src.join("file_23.docx");
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
    let xml = {
        let mut f = zip.by_name("word/document.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };

    // Body-final sectPr is the last </w:sectPr> before </w:body> (direct child).
    let body = xml
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or(&xml);
    // Find last sectPr that is NOT nested inside another element incorrectly —
    // take the last occurrence of <w:sectPr … </w:sectPr>.
    let mut last_sect = None;
    let mut rest = body;
    while let Some(start) = rest.find("<w:sectPr") {
        let after = &rest[start..];
        let end = after
            .find("</w:sectPr>")
            .map(|j| j + "</w:sectPr>".len())
            .unwrap_or(after.len());
        last_sect = Some(&after[..end]);
        rest = &after[end..];
    }
    let last = last_sect.expect("body must have a final sectPr");
    assert!(
        last.contains("footerReference") || last.contains("w:footerReference"),
        "Word keeps final-section footerReference on multi-section unique footers; got: {}",
        &last[..last.len().min(400)]
    );
    // Package should still carry footer parts from the multi-section base.
    let footer_parts: Vec<_> = (0..zip.len())
        .filter_map(|i| {
            let name = zip.by_index(i).ok()?.name().to_string();
            if name.contains("footer") && name.ends_with(".xml") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    assert!(
        footer_parts.len() >= 15,
        "expected many footer parts from file_22, got {}",
        footer_parts.len()
    );
}
