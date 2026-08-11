// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M414 — short alpha-list next × long base: pure-I/D (not free-mesh last label).
//!
//! Word pure-I ONE/A/TWO/A/B/C then pure-D pageref body. Engine free-meshed
//! last "C" into first TOC line (DI).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_paras(xml: &str) -> Vec<(bool, bool, String)> {
    let mut rest = xml;
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut paras = Vec::new();
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
        paras.push((has_ins, has_del, text));
    }
    paras
}

#[test]
fn pageref_x_restart_list_pure_i_c() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__pageref_standalone_uppercase_h_7701e07f.docx");
    let b = src.join("super_editor__restart_numbering_sub_list_85ddcb79.docx");
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
    let paras = body_paras(&xml);

    let pure_i_c = paras.iter().any(|(i, d, t)| *i && !*d && t.trim() == "C");
    assert!(
        pure_i_c,
        "expected pure-I C; got {:?}",
        paras
            .iter()
            .take(12)
            .map(|(i, d, t)| format!(
                "{}{} {:?}",
                if *i { "I" } else { "" },
                if *d { "D" } else { "" },
                t.chars().take(40).collect::<String>()
            ))
            .collect::<Vec<_>>()
    );
    let di_c = paras
        .iter()
        .any(|(i, d, t)| *i && *d && (t.contains('C') || t.contains("Introduction")));
    assert!(!di_c, "unexpected DI free-mesh of C into base");
}
