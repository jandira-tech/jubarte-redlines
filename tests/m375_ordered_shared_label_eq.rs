// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M375 — ordered×sublist residual free-meshes shared trailing label as EQ.
//!
//! Word MIX for "a"/"Lvl 1 – a" is `del "Lvl 1 – " + EQ "a" + ins " "` (not
//! `ins "a " + del "Lvl 1 – a"`). Whole-label insert left double-"a" thrash
//! (−1.9 residual after M374 spacing).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn ordered_x_sublist_shared_label_is_eq_not_ins() {
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

    let mut eq_label_count = 0usize;
    let mut full_del_label = 0usize;
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
        if !p.contains("Lvl 1") || !p.contains("<w:del") {
            continue;
        }
        // Full delText still holding trailing label = peel miss.
        if p.contains(">Lvl 1 – a<") || p.contains(">Lvl 1 – b<") {
            full_del_label += 1;
        }
        // Word shape: EQ label between </w:del> and optional <w:ins.
        if let Some(di) = p.find("</w:del>") {
            let tail = &p[di..];
            let before_ins = tail.split("<w:ins").next().unwrap_or(tail);
            if before_ins.contains(">a</w:t>") || before_ins.contains(">b</w:t>") {
                eq_label_count += 1;
            }
        }
    }
    assert!(
        eq_label_count >= 2,
        "expected ≥2 MIX with EQ shared label after del (Word free-mesh), got {eq_label_count}"
    );
    assert_eq!(
        full_del_label, 0,
        "delText must not keep full 'Lvl 1 – a/b' after shared-label EQ peel"
    );
}
