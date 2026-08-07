// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M369 — ordered_list × sublist: residual short labels zip into Lvl pure-D.
//!
//! Word free-meshes pure-I "a"/"b"/"3" with pure-D "Lvl 1 – a" / "Lvl 1 – b" /
//! "Lvl 2 – i" as MIX. Wholesale pure-I/D left them separate (−3.5). Residual
//! short-label zip folds pure-I into the matching pure-D carrier.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn ordered_x_sublist_short_labels_mix_with_lvl() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__simple_ordered_list_8288421a.docx");
    let b = src.join("super_editor__sublist_issue_66a1800a.docx");
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

    // Expect MIX containing both short label ins and Lvl delText.
    let mut mix_lvl = 0usize;
    let mut pure_i_short_a = 0usize;
    let mut rest = xml.as_str();
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
        let has_lvl = p.contains("Lvl");
        if has_ins && has_del && has_lvl {
            mix_lvl += 1;
        }
        // Standalone pure-I residual "a" (not MIX) should be gone.
        if has_ins
            && !has_del
            && !has_lvl
            && (p.contains(">a</w:t>") || p.contains(">a </w:t>") || p.contains(">a\n"))
            && p.contains("numPr")
        {
            pure_i_short_a += 1;
        }
    }
    assert!(
        mix_lvl >= 3,
        "expected ≥3 MIX paras with Lvl del + short-label ins (a/b/3), got {mix_lvl}"
    );
    assert_eq!(
        pure_i_short_a, 0,
        "short pure-I 'a' residual should zip into Lvl pure-D"
    );
    // M372: Word keeps Inserted live numPr (B's list) + pPrChange(Deleted old).
    // At least one MIX with Lvl should carry pPrChange (A ListParagraph parked).
    assert!(
        xml.contains("pPrChange") && xml.contains("Lvl"),
        "Word MIX parks Deleted list pPr under pPrChange"
    );
}
