// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M427 — short multi-para table-free base × table-bearing next: pure-I all next
//! then pure-D base (Word order).
//!
//! Exhibit: tab_test × diff_after7 (~54 vs docxodus 99.7). M426 only allowed
//! n1∈[1,2]; tab base has 4 contentful paras so full LCS mid-meshed first base
//! lines into next (D + ID…). Word pure-I's all of next then pure-D all base.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn tab_test_x_diff_after7_pure_i_next_then_d_base() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__tab_test_576c8317.docx");
    let b = src.join("super_editor__diff_after7_b998213e.docx");
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

    let body = xml
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or(&xml);

    let mut saw_table = false;
    let mut pure_d_before_table = false;
    let mut pure_d_after_table = false;
    let mut pure_d_has_tab_tests = false;
    let mut first_contentful_kind = None::<char>;

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
            let mut ins_text = String::new();
            for (tag, end_tag, sink) in [
                ("<w:delText", "</w:delText>", &mut del_text),
                ("<w:t", "</w:t>", &mut ins_text),
            ] {
                let mut r = p;
                while let Some(i) = r.find(tag) {
                    let r2 = &r[i..];
                    let Some(gt) = r2.find('>') else { break };
                    let after_t = &r2[gt + 1..];
                    let Some(end_t) = after_t.find(end_tag) else {
                        break;
                    };
                    sink.push_str(&after_t[..end_t]);
                    r = &after_t[end_t + end_tag.len()..];
                }
            }
            let contentful = !del_text.trim().is_empty() || !ins_text.trim().is_empty() || has_ins || has_del;
            if contentful && first_contentful_kind.is_none() {
                first_contentful_kind = Some(if has_ins && !has_del {
                    'I'
                } else if has_del && !has_ins {
                    'D'
                } else {
                    'M'
                });
            }
            if has_del && !has_ins {
                if !saw_table {
                    pure_d_before_table = true;
                } else {
                    pure_d_after_table = true;
                }
                if del_text.to_ascii_lowercase().contains("tab tests") {
                    pure_d_has_tab_tests = true;
                }
            }
            continue;
        }
        if let Some(gt) = after.find('>') {
            rest = &after[gt + 1..];
        } else {
            break;
        }
    }

    assert!(saw_table, "expected table-bearing next");
    assert_eq!(
        first_contentful_kind,
        Some('I'),
        "Word starts with pure-I of next, not base delete"
    );
    assert!(
        !pure_d_before_table,
        "Word pure-I's all of next before pure-D base; found pure-D before first table"
    );
    assert!(pure_d_after_table, "expected pure-D base after tables");
    assert!(
        pure_d_has_tab_tests,
        "expected pure-D of base 'Tab Tests:' after next content"
    );
}
