// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M435 — sole pure-D Heading free-mesh into last pure-I keeps live Heading1.
//!
//! Pair: `diff_before16` (Heading1 title + body) × `diff_before19` (short next).
//! Word MIX keeps live `pStyle=Heading1` and a del mark with ins next text plus
//! del title. Sole-del fold used to drop del pPr (empty pPr, LO ~53.9). Adopting
//! the pure-D Heading pPr on that fold restores LO 100.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn diff_before16_x_19_mix_keeps_heading1() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__diff_before16_f518c031.docx");
    let b = src.join("super_editor__diff_before19_97e0f4e6.docx");
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
    let xml = {
        let mut f = zip.by_name("word/document.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };

    // Find MIX with delText "Diffing feature" (base title).
    let mut found = false;
    let mut rest = xml.as_str();
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
        if !(has_ins && has_del) {
            continue;
        }
        if !p.contains("Diffing feature") && !p.contains("image will be added") {
            continue;
        }
        found = true;
        assert!(
            p.contains("Heading1") || p.contains(r#"w:val="Heading1""#),
            "Word keeps live Heading1 on title free-mesh MIX; p was {}",
            &p[..p.len().min(400)]
        );
        // del mark on para rPr (Word shape)
        let ppr = p
            .find("<w:pPr")
            .map(|i| &p[i..])
            .and_then(|s| s.find("</w:pPr>").map(|j| &s[..j + 7]));
        if let Some(ppr) = ppr {
            let live = if let Some(i) = ppr.find("<w:pPrChange") {
                &ppr[..i]
            } else {
                ppr
            };
            assert!(
                live.contains("<w:del") || live.contains("w:del"),
                "Word keeps del mark on MIX pPr/rPr"
            );
        }
        break;
    }
    assert!(
        found,
        "expected MIX free-mesh of image line × Diffing feature title"
    );
}
