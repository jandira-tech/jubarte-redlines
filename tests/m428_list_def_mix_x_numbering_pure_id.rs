// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M428 — mixed-label list base × short uniform-token list next: pure-I next
//! then pure-D base (Word order).
//!
//! Exhibit: list_def_mix × list_numbering_reimport (~52.6 vs docxodus 90.1).
//! Word pure-I's all four "test" items (last ID with first base label) then
//! pure-D residual base. Full LCS interleaves mid-base deletes between next
//! inserts (I DDDDD III DDD).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn list_def_mix_x_numbering_reimport_pure_i_then_d() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__list_def_mix_d7cec092.docx");
    let b = src.join("super_editor__list_numbering_reimport_d788d573.docx");
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

    // Collect contentful para kinds in order.
    let mut kinds = Vec::new();
    let mut rest = xml.as_str();
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
        if text.trim().is_empty() && !has_ins && !has_del {
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
        kinds.push((kind, text.trim().to_string()));
    }

    // Word: leading pure-I stream of next "test" items before any pure-D of
    // base labels (Num/Letter). Allow trailing ID on last test×Num1.
    let first_d = kinds.iter().position(|(k, _)| *k == 'D' || *k == 'M');
    let pure_i_prefix = kinds
        .iter()
        .take_while(|(k, _)| *k == 'I')
        .count();
    assert!(
        pure_i_prefix >= 3,
        "Word pure-I's ≥3 next 'test' items first; got pure_i_prefix={pure_i_prefix} kinds={kinds:?}"
    );
    // No pure-D of base before we've seen ≥3 pure-I next.
    if let Some(i) = first_d {
        assert!(
            i >= 3,
            "pure-D/MIX too early at {i}; kinds={kinds:?}"
        );
    }
    // Residual pure-D should include base labels.
    let del_joined: String = kinds
        .iter()
        .filter(|(k, _)| *k == 'D' || *k == 'M')
        .map(|(_, t)| t.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        del_joined.to_ascii_lowercase().contains("num")
            || del_joined.to_ascii_lowercase().contains("letter"),
        "expected pure-D of base list labels; got {del_joined:?}"
    );
    // Word junction: last pure-I "test" MIX-es with first pure-D "Num 1" (ID).
    let has_id_junction = kinds.iter().any(|(k, t)| {
        *k == 'M' && t.to_ascii_lowercase().contains("test") && t.to_ascii_lowercase().contains("num")
    });
    assert!(
        has_id_junction,
        "Word MIX-es last test into Num 1 (ID junction); kinds={kinds:?}"
    );
}
