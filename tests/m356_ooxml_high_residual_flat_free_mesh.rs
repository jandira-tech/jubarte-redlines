// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M356 — OOXML property demos with high residual overlap keep flat free-mesh.
//!
//! bold_rstyle×vals: residual_j≈0.16, Word meshes the revised opening into
//! the original's second paragraph and matches the sample lines. M346 title
//! peel + finalize fold thrash IDDMD… (−42 vs 27c).
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

    // Word (wr0926 reference): the revised opening "This document
    // demonstrates all valid" lands in the deleted base title, whose mark is
    // deleted, and the rest streams into the original's "A) ST_OnOff values
    // for <w:b> on a run:", which keeps "ST_OnOff values for". Pairing the
    // two titles instead (a title swap) left that paragraph wholly deleted.
    let (first, second) = (&paras[0], &paras[1]);
    assert!(
        first.1.contains("This document demonstrates all valid")
            && first.1.contains("OOXML w:b (bold) tester")
            && !first.1.contains("Each line shows"),
        "the base title should take only the revised opening; seq={seq} first={first:?}"
    );
    assert!(
        second.0 == 'M' && second.1.contains("ST_OnOff values for"),
        "the original's second paragraph should keep its words; seq={seq} second={second:?}"
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
