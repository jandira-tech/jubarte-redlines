// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M410 — OOXML property demo × short alpha-list pure-I/D (not free-mesh).

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
fn bold_vals_x_complex_list_is_pure_id() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__ooxml_bold_vals_demo_9e688d8f.docx");
    let b = src.join("super_editor__complex_list_def_issue_326369f9.docx");
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
    let mix = paras.iter().any(|(i, d, t)| {
        *i && *d && (t.contains("FOUR") || t.to_ascii_lowercase().contains("st_onoff"))
    });
    assert!(
        !mix,
        "expected pure-I/D, not MIX on FOUR×OOXML: {:?}",
        paras
            .iter()
            .filter(|(i, d, _)| *i && *d)
            .map(|(_, _, t)| t.as_str())
            .collect::<Vec<_>>()
    );
    let pure_i_one = paras.iter().any(|(i, d, t)| *i && !*d && t == "ONE");
    let pure_d_bold = paras
        .iter()
        .any(|(i, d, t)| !*i && *d && t.to_ascii_lowercase().contains("st_onoff"));
    assert!(pure_i_one, "expected pure-I ONE");
    assert!(pure_d_bold, "expected pure-D OOXML bold intro");
}
