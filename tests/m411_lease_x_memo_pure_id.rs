// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M411 — lease×memo pure-I all next then pure-D all base.
//!
//! Word: continuous pure-I memo then pure-D lease. legal_mid_splice_cut was
//! cutting memo at "1. Business Operations" and interleaving pure-D mid-doc.

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
fn lease_x_memo_is_pure_i_then_pure_d() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("evals__lease_agreement_7081191d.docx");
    let b = src.join("evals__memorandum_258c774a.docx");
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

    // Seq must be I…I D…D (no D then I again mid-stream).
    let mut saw_d = false;
    let mut saw_i_after_d = false;
    for (i, d, _) in &paras {
        if *d && !*i {
            saw_d = true;
        }
        if saw_d && *i && !*d {
            saw_i_after_d = true;
            break;
        }
    }
    assert!(
        !saw_i_after_d,
        "expected pure-I-all then pure-D-all, got mid pure-I after pure-D: {:?}",
        paras
            .iter()
            .map(|(i, d, t)| format!(
                "{}{} {:?}",
                if *i { "I" } else { "" },
                if *d { "D" } else { "" },
                t.chars().take(25).collect::<String>()
            ))
            .collect::<Vec<_>>()
    );

    let pure_i_memo = paras
        .iter()
        .any(|(i, d, t)| *i && !*d && t.trim() == "MEMORANDUM");
    let pure_d_lease = paras
        .iter()
        .any(|(i, d, t)| !*i && *d && t.to_ascii_lowercase().contains("commercial lease"));
    assert!(pure_i_memo, "expected pure-I MEMORANDUM");
    assert!(pure_d_lease, "expected pure-D Commercial Lease");
}
