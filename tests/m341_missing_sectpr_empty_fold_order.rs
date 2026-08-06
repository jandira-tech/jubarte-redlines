// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M341 — missing_sectpr × missing_separator: Word IIIIM (pure-I "something"
//! then MIX empty+del base title). Fold whitespace pure-I into pure-D must
//! run before stripping empties before pure-Ds.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_para_classes(xml: &str) -> Vec<(char, String, String)> {
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
        let mut dt = String::new();
        // crude text extract
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
        s = p;
        while let Some(i) = s.find("<w:delText") {
            let s2 = &s[i..];
            if let Some(j) = s2.find('>') {
                let s3 = &s2[j + 1..];
                if let Some(k) = s3.find("</w:delText>") {
                    dt.push_str(&s3[..k]);
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
        out.push((c, t, dt));
    }
    out
}

#[test]
fn missing_sectpr_x_separator_iiim() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__missing_sectpr_967a402d.docx");
    let b = src.join("super_editor__missing_separator_41c823b9.docx");
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
    // Ignore trailing EQ empties (sectPr neighbors).
    let content: Vec<_> = paras
        .iter()
        .filter(|(c, t, dt)| *c != 'E' || !t.is_empty() || !dt.is_empty())
        .cloned()
        .collect();
    let seq: String = content.iter().map(|(c, _, _)| *c).collect();
    assert!(
        seq.starts_with("IIIIM") || seq == "IIIIM",
        "Word IIIIM; got {seq} full={paras:?}"
    );
    let something = content.iter().find(|(_, t, _)| t == "something");
    assert!(
        something.is_some_and(|(c, _, _)| *c == 'I'),
        "something must stay pure-I; got {content:?}"
    );
    let mix = content.iter().find(|(c, _, _)| *c == 'M').expect("MIX");
    assert!(
        mix.2.contains("sectPr") || mix.2.contains("Document"),
        "MIX holds del base title; got {mix:?}"
    );
}
