// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M430 — title-page pure-I keeps trailing empty pure-I before pure-D base.
//!
//! Exhibit: doc_with_spaces_from_styles × doc_with_spacing (~62.8 vs docxodus 100).
//! Word pure-I's the cover (including ~6 empty pure-I after the date) then
//! pure-D base. fold_whitespace_pure_ins_into_following_pure_del ate those
//! empties into the first pure-D.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn spaces_x_spacing_keeps_trailing_empty_pure_i_before_base_del() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__doc_with_spaces_from_styles_734ca26f.docx");
    let b = src.join("super_editor__doc_with_spacing_e3d47bd7.docx");
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

    // Find March 10, 2040 then count empty pure-I paras before first pure-D
    // of base "ENGAGEMENT".
    let date_pos = xml
        .find("March 10, 2040")
        .expect("expected title-page date pure-I");
    let eng_pos = xml
        .find("ENGAGEMENT")
        .or_else(|| xml.find("Engagement"))
        .expect("expected base ENGAGEMENT del");
    assert!(date_pos < eng_pos, "date should precede base del");
    let between = &xml[date_pos..eng_pos];
    // Count empty pure-I-ish paragraphs: w:p with w:ins and no w:t body text
    // (allow whitespace-only t). Crude but stable.
    let mut empty_pure_i = 0usize;
    let mut rest = between;
    while let Some(rel) = rest.find("<w:p") {
        let after = &rest[rel..];
        if !(after.starts_with("<w:p>") || after.starts_with("<w:p ")) {
            rest = &after[4..];
            continue;
        }
        let end = after.find("</w:p>").map(|j| j + 6).unwrap_or(after.len());
        let p = &after[..end];
        rest = &after[end..];
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        let texts: String = {
            let mut t = String::new();
            let mut r = p;
            while let Some(i) = r.find("<w:t") {
                let r2 = &r[i..];
                let Some(gt) = r2.find('>') else { break };
                let after_t = &r2[gt + 1..];
                let Some(e) = after_t.find("</w:t>") else {
                    break;
                };
                t.push_str(&after_t[..e]);
                r = &after_t[e + 6..];
            }
            t
        };
        if has_ins && !has_del && texts.trim().is_empty() {
            empty_pure_i += 1;
        }
    }
    assert!(
        empty_pure_i >= 3,
        "Word keeps ≥3 trailing empty pure-I after date before base del; got {empty_pure_i}"
    );
}
