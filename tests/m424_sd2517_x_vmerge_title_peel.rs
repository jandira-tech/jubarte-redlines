// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M424 — long multi-table base × short single-table next: pure-I title first.
//!
//! Word pure-I's "SD-2672 plain 3x3" then pure-D all base lorem then pure-I
//! residual cells. Wholesale pure-I/D pure-I'd all cells first (IIII…DDD).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn sd2517_x_vmerge_pure_i_title_first() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__sd_2517_localized_heading_styles_39c2e4a1.docx");
    let b = src.join("behavior__sd_2672_gridbefore_vmerge_7c895dff.docx");
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

    // First contentful revision para must be pure-I title, then pure-D base.
    let mut rest = xml.as_str();
    let mut first_kind = None;
    let mut first_text = String::new();
    let mut saw_pure_d_after_title = false;
    let mut pure_i_before_first_d = 0usize;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        let mut text = String::new();
        for (tag, end_tag) in [("<w:t", "</w:t>"), ("<w:delText", "</w:delText>")] {
            let mut r = p;
            while let Some(i) = r.find(tag) {
                let r2 = &r[i..];
                let Some(gt) = r2.find('>') else { break };
                let after_t = &r2[gt + 1..];
                let Some(end) = after_t.find(end_tag) else {
                    break;
                };
                text.push_str(&after_t[..end]);
                r = &after_t[end + end_tag.len()..];
            }
        }
        let t = text.trim();
        if t.is_empty() && !has_ins && !has_del {
            continue;
        }
        let kind = if has_ins && has_del {
            'M'
        } else if has_ins {
            'I'
        } else if has_del {
            'D'
        } else {
            'E'
        };
        if first_kind.is_none() && !t.is_empty() {
            first_kind = Some(kind);
            first_text = t.to_string();
        }
        if first_kind == Some('I') && kind == 'I' && !saw_pure_d_after_title {
            pure_i_before_first_d += 1;
        }
        if first_kind == Some('I') && kind == 'D' {
            saw_pure_d_after_title = true;
            break;
        }
    }
    assert_eq!(
        first_kind,
        Some('I'),
        "first contentful must be pure-I title"
    );
    assert!(
        first_text.to_ascii_lowercase().contains("sd-2672")
            || first_text.to_ascii_lowercase().contains("plain"),
        "title text unexpected: {first_text}"
    );
    assert!(
        saw_pure_d_after_title,
        "expected pure-D base after title pure-I"
    );
    // Wholesale pure-I/D pure-I'd ~11 cells before any pure-D.
    assert!(
        pure_i_before_first_d <= 3,
        "too many pure-I before first pure-D: {pure_i_before_first_d} (wholesale thrash)"
    );
}
