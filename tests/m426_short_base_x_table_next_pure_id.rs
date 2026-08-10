// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M426 — short table-free base × long table-bearing next: pure-I all next then
//! pure-D base (Word order).
//!
//! Exhibit: text_color_highlight × sd_2672_nested_table (~52 vs docxodus 100).
//! M315 only fires when **both** sides are table-free, so this pair fell through
//! to full LCS which pure-I's the next title, pure-D's base mid-stream, then
//! pure-I's the tables — Word pure-I's the entire next doc first, then pure-D's
//! the base technicolor line at the end.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn text_color_x_nested_table_pure_i_all_next_then_d_base() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__text_color_highlight_36cb4c90.docx");
    let b = src.join("behavior__sd_2672_nested_table_dfac08bb.docx");
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

    // Body top-level: pure-I stream of next (title + tables + cell notes), then
    // pure-D base technicolor text — never a pure-D before the first table.
    let body = xml
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or(&xml);

    let mut saw_table = false;
    let mut pure_d_before_table = false;
    let mut pure_d_after_table = false;
    let mut pure_d_has_technicolor = false;

    let mut rest = body;
    while let Some(start) = rest.find('<') {
        let after = &rest[start..];
        if after.starts_with("<w:tbl") {
            let end = after
                .find("</w:tbl>")
                .map(|j| j + "</w:tbl>".len())
                .unwrap_or(after.len());
            saw_table = true;
            rest = &after[end..];
            continue;
        }
        if after.starts_with("<w:p") {
            let end = after
                .find("</w:p>")
                .map(|j| j + "</w:p>".len())
                .or_else(|| after.find("/>").map(|j| j + 2))
                .unwrap_or(after.len());
            let p = &after[..end];
            rest = &after[end..];
            let has_ins = p.contains("<w:ins");
            let has_del = p.contains("<w:del") || p.contains("<w:delText");
            let mut del_text = String::new();
            let mut r = p;
            while let Some(i) = r.find("<w:delText") {
                let r2 = &r[i..];
                let Some(gt) = r2.find('>') else { break };
                let after_t = &r2[gt + 1..];
                let Some(end_t) = after_t.find("</w:delText>") else {
                    break;
                };
                del_text.push_str(&after_t[..end_t]);
                r = &after_t[end_t + "</w:delText>".len()..];
            }
            if has_del && !has_ins {
                if !saw_table {
                    pure_d_before_table = true;
                } else {
                    pure_d_after_table = true;
                }
                if del_text.to_ascii_lowercase().contains("technicolor") {
                    pure_d_has_technicolor = true;
                }
            }
            continue;
        }
        // Skip other tags
        if let Some(gt) = after.find('>') {
            rest = &after[gt + 1..];
        } else {
            break;
        }
    }

    assert!(
        saw_table,
        "expected nested-table next to emit at least one w:tbl"
    );
    assert!(
        !pure_d_before_table,
        "Word pure-I's all of next (including tables) before pure-D base; \
         found pure-D before first table"
    );
    assert!(
        pure_d_after_table,
        "expected pure-D base after tables (Word order)"
    );
    assert!(
        pure_d_has_technicolor,
        "expected pure-D of base technicolor line after next content"
    );
}
