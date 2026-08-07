// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M356 — OOXML property demos with high residual overlap keep flat free-mesh.
//!
//! bold_rstyle×vals: residual_j≈0.16, Word DXMDMD… (mesh title×section, match
//! sample lines). M346 title peel + finalize fold thrash IDDMD… (−42 vs 27c).
//! Only peel titles when residual vocab is sparse (vals×color residual_j≈0.04).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_para_classes(xml: &str) -> Vec<(char, String)> {
    let mut out = Vec::new();
    let mut rest = xml;
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
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
        let has_del = p.contains("<w:del");
        let mut t = String::new();
        let mut s = p;
        while let Some(i) = s.find("<w:t") {
            let s2 = &s[i..];
            if let Some(j) = s2.find('>') {
                let s3 = &s2[j + 1..];
                if let Some(k) = s3.find("</w:t>") {
                    t.push_str(&s3[..k]);
                    s = &s3[k..];
                    continue;
                }
            }
            break;
        }
        // Also harvest delText so pure-D paras are contentful.
        let mut s = p;
        while let Some(i) = s.find("<w:delText") {
            let s2 = &s[i..];
            if let Some(j) = s2.find('>') {
                let s3 = &s2[j + 1..];
                if let Some(k) = s3.find("</w:delText>") {
                    t.push_str(&s3[..k]);
                    s = &s3[k..];
                    continue;
                }
            }
            break;
        }
        let c = match (has_ins, has_del) {
            (true, true) => 'M',
            (true, false) => 'I',
            (false, true) => 'D',
            (false, false) => 'E',
        };
        out.push((c, t));
    }
    out
}

#[test]
fn bold_rstyle_x_vals_flat_free_mesh_not_title_peel() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__ooxml_bold_rstyle_linked_combos_demo_90819822.docx");
    let b = src.join("super_editor__ooxml_bold_vals_demo_9e688d8f.docx");
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
    let paras = body_para_classes(&xml);
    assert!(!paras.is_empty(), "expected body paras");
    let seq: String = paras.iter().map(|(c, _)| *c).collect();

    // Word/27c: first contentful is pure-D (base title) then MIX, not a single
    // ID title-swap of both full titles (M346 peel + finalize fold thrash).
    let first = &paras[0];
    assert_eq!(
        first.0, 'D',
        "first para must be pure-D base title (Word DX…), not ID title-swap; seq={seq}"
    );
    assert!(
        first.1.to_ascii_lowercase().contains("ooxml")
            || first.1.to_ascii_lowercase().contains("bold"),
        "first pure-D should be base rstyle title; got {:?}",
        first.1
    );
    // Must not open with ID (ins full next title + del full base title).
    assert!(
        !seq.starts_with("MD") && !seq.starts_with('M') || seq.starts_with('D'),
        "must not start with title MIX/ID swap; seq={seq}"
    );
    // Sample lines mesh as MD/E (delete leading dash, match body).
    let n_md_or_e = paras
        .iter()
        .filter(|(c, t)| {
            (*c == 'D' || *c == 'E' || *c == 'M') && t.to_ascii_lowercase().contains("sample text")
        })
        .count();
    assert!(
        n_md_or_e >= 4,
        "sample lines should mesh (MD/E/M with Sample text); seq={seq} n={n_md_or_e}"
    );
}

#[test]
fn bold_vals_x_color_still_peels_title() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__ooxml_bold_vals_demo_9e688d8f.docx");
    let b = src.join("super_editor__ooxml_color_rstyle_linked_combos_demo_23e43bed.docx");
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
    let paras = body_para_classes(&xml);
    let first = paras
        .iter()
        .find(|(_, t)| t.to_ascii_lowercase().contains("color tester") || t.contains("w:color"))
        .expect("color title para");
    assert_eq!(
        first.0, 'I',
        "color title must stay pure-I (M346 peel for low residual_j); got {paras:?}"
    );
}
