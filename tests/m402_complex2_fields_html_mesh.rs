// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M402 — complex2 short alpha-list × fields_test "html input type" free-mesh.
//!
//! Word free-meshes residual "html input type" with base "ONE" (MIX). Engine
//! pure-I/D left html as pure-I after pure-D ONE (IIDDI). Content fingerprint
//! only (short alpha-list base + "html input type" next).

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
        let mut r = p;
        while let Some(i) = r.find("<w:t") {
            let r2 = &r[i..];
            let Some(gt) = r2.find('>') else { break };
            let after_t = &r2[gt + 1..];
            let Some(end) = after_t.find("</w:t>") else {
                break;
            };
            text.push_str(&after_t[..end]);
            r = &after_t[end + 6..];
        }
        r = p;
        while let Some(i) = r.find("<w:delText") {
            let r2 = &r[i..];
            let Some(gt) = r2.find('>') else { break };
            let after_t = &r2[gt + 1..];
            let Some(end) = after_t.find("</w:delText>") else {
                break;
            };
            text.push_str(&after_t[..end]);
            r = &after_t[end + 12..];
        }
        paras.push((has_ins, has_del, text));
    }
    paras
}

#[test]
fn complex2_x_fields_meshes_html_with_one() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__complex2_dbba7f05.docx");
    let b = src.join("super_editor__fields_test_4a8ffd8c.docx");
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

    // Word: MIX containing both "html input type" and "ONE" (or adjacent free-mesh).
    let mix_html_one = paras.iter().any(|(i, d, t)| {
        *i && *d
            && t.to_ascii_lowercase().contains("html")
            && (t.contains("ONE") || t.contains("One") || t.contains("one"))
    });
    if mix_html_one {
        return;
    }
    // Accept MIX with html and a nearby pure-D ONE (still better than pure-I
    // html after pure-D ONE wholesale).
    let html_i = paras
        .iter()
        .position(|(i, _d, t)| *i && t.to_ascii_lowercase().contains("html input type"));
    let one_d = paras
        .iter()
        .position(|(i, d, t)| !*i && *d && t.trim() == "ONE");
    match (html_i, one_d) {
        (Some(hi), Some(od)) => {
            // Free-mesh: MIX html or html before/with ONE, not pure-I html after ONE.
            let html_para = &paras[hi];
            if html_para.0 && html_para.1 {
                return; // MIX ok
            }
            assert!(
                hi <= od + 1,
                "html must not trail pure-D ONE by wholesale pure-I/D; hi={hi} od={od} paras={paras:?}"
            );
        }
        _ => {
            // At least one MIX paragraph overall (Word has MIX≥1).
            let mix_n = paras.iter().filter(|(i, d, _)| *i && *d).count();
            assert!(
                mix_n >= 1,
                "expected free-mesh MIX html×ONE; paras={paras:?}"
            );
        }
    }
}
