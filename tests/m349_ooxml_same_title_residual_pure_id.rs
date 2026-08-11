// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M349 — OOXML property demos with shared first title token (OOXML×OOXML):
//! free-mesh first 2 contentful (title + section header), pure-I/D residual
//! body samples.
//!
//! italic×rFonts: Word MMIIIIIIIIIIDDDD… (titles MIX, body pure-I then pure-D).
//! Flat free-mesh over-meshes body samples (MMMMMMMMM…, pagefair thrash).

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
fn italic_x_rfonts_title_mesh_body_pure_i() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__ooxml_italic_rstyle_combos_demo_90894ac1.docx");
    let b = src.join("super_editor__ooxml_rFonts_rstyle_linked_combos_dem_213298de.docx");
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

    // First two contentful (title + "A) Direct…") should MIX (Word MM…).
    let contentful: Vec<_> = paras.iter().filter(|(_, t)| !t.trim().is_empty()).collect();
    assert!(
        contentful.len() >= 4,
        "need title+header+samples; got {contentful:?}"
    );
    assert_eq!(
        contentful[0].0, 'M',
        "title should MIX (shared OOXML token); got {:?}",
        contentful[0]
    );
    assert_eq!(
        contentful[1].0, 'M',
        "section A header should MIX; got {:?}",
        contentful[1]
    );

    // First sample line after headers must be pure-I (Word MMIIII…), not MIX.
    // Over-mesh was MMMMMMMMM… on sample lines.
    let first_sample = contentful
        .iter()
        .skip(2)
        .find(|(_, t)| {
            t.contains("Times New Roman") || t.contains("quick brown") || t.contains("ascii/hAnsi")
        })
        .expect("rFonts sample line");
    assert_eq!(
        first_sample.0,
        'I',
        "body sample must be pure-I (not free-mesh MIX); got seq={}",
        paras.iter().map(|(c, _)| *c).collect::<String>()
    );

    let seq: String = paras.iter().map(|(c, _)| *c).collect();
    let early: String = seq.chars().take(12).collect();
    // Word MMIIIIIIIIIID… — at least one pure-I in the first 12, and not all-M.
    assert!(
        early.contains('I') && !early.chars().all(|c| c == 'M'),
        "Word-shaped MMIIII… not MMMMM…; got {early} full={seq}"
    );
}
