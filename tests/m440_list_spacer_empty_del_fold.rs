// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M440 — multi pure-I short list labels fold last label into empty pure-D.
//!
//! list_spacer1 × list_with_break_exported_broken (~56 vs docxodus 95): Word
//! free-meshes last pure-I "b" with empty pure-D → MIX del mark (no live
//! numPr). Engine left pure-I "b" + empty pure-D separate (I I I D D).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn list_spacer_last_label_mixes_with_empty_pure_d() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__list_spacer1_06383c66.docx");
    let b = src.join("super_editor__list_with_break_exported_broken_45f7bd19.docx");
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

    let mut kinds = Vec::new();
    let mut rest = xml.as_str();
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        if !(after.starts_with("<w:p>") || after.starts_with("<w:p ")) {
            rest = &after[4..];
            continue;
        }
        let end = after.find("</w:p>").map(|j| j + 6).unwrap_or(after.len());
        let p = &after[..end];
        rest = &after[end..];
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        let texts: String = {
            let mut t = String::new();
            for (tag, end_tag) in [("<w:t", "</w:t>"), ("<w:delText", "</w:delText>")] {
                let mut r = p;
                while let Some(i) = r.find(tag) {
                    let r2 = &r[i..];
                    let Some(gt) = r2.find('>') else { break };
                    let after_t = &r2[gt + 1..];
                    let Some(e) = after_t.find(end_tag) else {
                        break;
                    };
                    t.push_str(&after_t[..e]);
                    r = &after_t[e + end_tag.len()..];
                }
            }
            t
        };
        let kind = if has_ins && has_del {
            'M'
        } else if has_ins {
            'I'
        } else if has_del {
            'D'
        } else {
            'E'
        };
        kinds.push((kind, texts.trim().to_string()));
    }

    // Find MIX with body "b" and no separate pure-I "b".
    let has_mix_b = kinds.iter().any(|(k, t)| *k == 'M' && t == "b");
    let pure_i_b = kinds.iter().filter(|(k, t)| *k == 'I' && t == "b").count();
    assert!(
        has_mix_b,
        "Word MIX-es last list label 'b' with empty pure-D del mark; kinds={kinds:?}"
    );
    assert_eq!(
        pure_i_b, 0,
        "pure-I 'b' should be folded into MIX; kinds={kinds:?}"
    );
    // Leading pure-I stream still present.
    assert!(
        kinds.iter().any(|(k, t)| *k == 'I' && t == "a"),
        "expected pure-I 'a'; kinds={kinds:?}"
    );
}
