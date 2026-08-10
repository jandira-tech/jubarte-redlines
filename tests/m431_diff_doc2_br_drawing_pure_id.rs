// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M431 — after pure-I Num* stats, residual br-only must stay pure-I and
//! base drawing-only pure-D (not one MIX with both).
//!
//! Word: III (Num*) + pure-I br + pure-D drawing + pure-D empty + MIX…
//! Pre-M431 residual free-mesh mid-spliced br×drawing → IIIMD…

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn diff_doc2_x_numwords_br_not_mixed_with_drawing() {
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

    // Walk body paragraphs in document order. Nested table cells may appear
    // after the body paras of interest; we stop at first contentful MIX/I/D
    // after the Num*/br/drawing prefix, so table noise does not matter.
    let mut paras: Vec<&str> = Vec::new();
    let mut rest = xml.as_str();
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        paras.push(&after[..end_rel]);
        rest = &after[end_rel..];
    }

    // After ≥3 pure-I Num*, next contentful-or-br/drawing paras:
    // must see pure-I with br (no del) before any pure-D with drawing.
    let mut pure_i_num = 0usize;
    let mut saw_pure_i_br = false;
    let mut saw_pure_d_drawing = false;
    let mut mix_br_and_drawing = false;

    for p in &paras {
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        let has_br = p.contains("<w:br") || p.contains("<w:br/");
        let has_drawing = p.contains("<w:drawing") || p.contains("<w:pict");
        let mut text = String::new();
        for (tag, end_tag) in [("<w:t", "</w:t>"), ("<w:delText", "</w:delText>")] {
            let mut r = *p;
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

        if has_ins && !has_del && t.starts_with("num ") {
            pure_i_num += 1;
            continue;
        }
        if pure_i_num < 3 {
            continue;
        }

        if has_ins && has_del && has_br && has_drawing {
            mix_br_and_drawing = true;
        }
        if has_ins && !has_del && has_br && !has_drawing {
            saw_pure_i_br = true;
        }
        if has_del && !has_ins && has_drawing {
            saw_pure_d_drawing = true;
        }
        // Stop once we've left the empty/br/drawing prefix.
        if !t.is_empty() && (has_ins || has_del) {
            break;
        }
    }

    assert!(
        pure_i_num >= 3,
        "expected ≥3 pure-I Num* (M425), got {pure_i_num}"
    );
    assert!(
        !mix_br_and_drawing,
        "br-only must not MIX with drawing-only (Word pure-I then pure-D)"
    );
    assert!(
        saw_pure_i_br,
        "expected pure-I page-break (w:br) after Num* stats"
    );
    assert!(
        saw_pure_d_drawing,
        "expected pure-D drawing paragraph after pure-I br"
    );
}
