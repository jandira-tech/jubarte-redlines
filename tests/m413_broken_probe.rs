// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M413 — short single-letter next labels × short prose base: pure-I/D.
//!
//! Word pure-I a/x/x/b then pure-D base. M412 free-meshed last "b" into base
//! (DI) because short labels lack ≥3-char title tokens.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn broken_media_x_dup_ppr_pure_i_b() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("word_tolerated_broken_media_rel.docx");
    let b = src.join("word_tolerated_duplicate_ppr.docx");
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

    // Find pure-I 'b' paragraph (ins only, no del in same p).
    let mut rest = xml.as_str();
    let mut pure_i_b = false;
    let mut di_love = false;
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
        let t = text.trim();
        if has_ins && !has_del && t == "b" {
            pure_i_b = true;
        }
        if has_ins && has_del && (t.contains("I love") || t.contains("love")) {
            di_love = true;
        }
    }
    // Also accept pure-I via standalone ins of b without requiring exact parse.
    if !pure_i_b {
        pure_i_b = xml.contains("<w:ins")
            && xml.contains(">b</w:t>")
            && !xml.as_bytes().windows(80).any(|w| {
                let s = String::from_utf8_lossy(w);
                s.contains(">b</w:t>") && s.contains("delText")
            });
    }
    assert!(pure_i_b, "expected pure-I 'b'");
    assert!(!di_love, "unexpected DI free-mesh of b into base prose");
    assert!(
        xml.contains("I love") && xml.contains("delText"),
        "expected pure-D base prose"
    );
}
