// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M438 — title-page pure-I ends with e×6 DD E (trailing bare empty).
//!
//! Exhibit: doc_with_spaces_from_styles × doc_with_spacing (~67 vs docxodus 100).
//! Word keeps six empty pure-I after the date, pure-D base, then a bare trailing
//! empty. Engine kept all seven next empties as pure-I and no trailing E.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn spaces_x_spacing_trailing_bare_empty_after_pure_d() {
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

    // Classify body paragraphs after body open.
    let body = xml.find("<w:body").map(|i| &xml[i..]).unwrap_or(&xml);
    let mut kinds = Vec::new();
    let mut rest = body;
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
            let mut r = p;
            while let Some(i) = r.find("<w:delText") {
                let r2 = &r[i..];
                let Some(gt) = r2.find('>') else { break };
                let after_t = &r2[gt + 1..];
                let Some(e) = after_t.find("</w:delText>") else {
                    break;
                };
                t.push_str(&after_t[..e]);
                r = &after_t[e + 12..];
            }
            t
        };
        let empty = texts.trim().is_empty();
        let kind = if has_ins && has_del {
            'M'
        } else if has_ins {
            'I'
        } else if has_del {
            'D'
        } else {
            'E'
        };
        kinds.push((kind, empty, texts.trim().to_string()));
    }

    // Find date pure-I index and first ENGAGEMENT pure-D.
    let date_i = kinds
        .iter()
        .position(|(_, _, t)| t.contains("March 10, 2040"))
        .expect("date pure-I");
    let eng_i = kinds
        .iter()
        .position(|(_, _, t)| t.to_ascii_uppercase().contains("ENGAGEMENT"))
        .expect("ENGAGEMENT pure-D");
    assert!(date_i < eng_i, "date before ENGAGEMENT; kinds={kinds:?}");

    // Empty pure-I between date and ENGAGEMENT.
    let empty_i = kinds[date_i + 1..eng_i]
        .iter()
        .filter(|(k, e, _)| *k == 'I' && *e)
        .count();
    assert!(
        (5..=6).contains(&empty_i),
        "Word keeps e×6 empty pure-I after date (not all 7 next empties); got {empty_i}; kinds={kinds:?}"
    );

    // Trailing bare empty after last pure-D.
    let last_d = kinds
        .iter()
        .rposition(|(k, _, _)| *k == 'D')
        .expect("pure-D residual");
    assert!(
        last_d + 1 < kinds.len(),
        "Word keeps trailing bare empty after pure-D; kinds={kinds:?}"
    );
    let (k, empty, _) = &kinds[last_d + 1];
    assert!(
        *k == 'E' && *empty,
        "trailing after pure-D must be bare empty E; got kinds={kinds:?}"
    );
}
