// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M425 — short table base × "Num words/chars/pages" next: pure-I Num* first.
//!
//! Word pure-I's all three Num* lines then free-meshes residual. Full LCS
//! free-meshed first "Num words" into base (MIX).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn diff_doc2_x_numwords_pure_i_num_stats() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("doc_api_stories__diff_doc2_bc0da0ce.docx");
    let b = src.join("doc_api_stories__numwords_8be5f783.docx");
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

    let mut rest = xml.as_str();
    let mut pure_i_num = 0usize;
    let mut first_mix_is_num_words = false;
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
        let t = text.trim().to_ascii_lowercase();
        if t.is_empty() && !has_ins && !has_del {
            continue;
        }
        if has_ins && !has_del && t.starts_with("num ") {
            pure_i_num += 1;
        }
        if has_ins && has_del && t.contains("num words") {
            first_mix_is_num_words = true;
        }
        // Stop after first non-Num contentful pure-I/D/MIX with body.
        if pure_i_num >= 3 {
            break;
        }
        if has_del && !t.starts_with("num ") && !t.is_empty() {
            break;
        }
    }
    assert!(
        pure_i_num >= 3,
        "expected ≥3 pure-I Num* stats, got {pure_i_num}; mix_num_words={first_mix_is_num_words}"
    );
    assert!(
        !first_mix_is_num_words,
        "Num words must not free-mesh into base as first MIX"
    );
}
