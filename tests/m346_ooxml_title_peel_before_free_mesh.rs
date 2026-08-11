// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M346 — OOXML property demos: peel differing titles pure-I/D before free-mesh.
//!
//! bold_vals×color: Word IIMMMMM… (pure-I color title then MIX sample lines).
//! Flat free-mesh confetti-MIX-ed the color title (MIIII…, pagefair ~43).
//! Peel leading pure-I titles when first significant tokens differ.

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
fn bold_vals_x_color_title_pure_i() {
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
    assert!(!paras.is_empty(), "expected body paras");
    // First contentful next title must be pure-I (Word IIMMMMM…), not MIX.
    let first = paras
        .iter()
        .find(|(_, t)| t.to_ascii_lowercase().contains("color tester") || t.contains("w:color"))
        .expect("color title para");
    assert_eq!(
        first.0, 'I',
        "color title must be pure-I (not confetti MIX); got {paras:?}"
    );
    let seq: String = paras.iter().map(|(c, _)| *c).collect();
    assert!(
        seq.starts_with('I'),
        "Word-shaped II… / IID… not MIX-first; got {seq}"
    );
}
