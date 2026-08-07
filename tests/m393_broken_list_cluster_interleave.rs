// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M393 — broken_list_missing × broken_list: Word list-cluster interleave.
//!
//! Word: pure-I first next item ("ONE"), pure-D first base cluster through
//! nested subs (Item1…Sub b), pure-I rest next (TWO/a), pure-D rest base
//! ("First shown"…). Free word-LCS free-meshes "a"×"Item 1" into MIX.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn broken_list_missing_x_broken_list_cluster_interleave() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__broken_list_missing_items_36b4199e.docx");
    let b = src.join("super_editor__broken_list_7e9b9bf7.docx");
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
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut paras: Vec<(bool, bool, String)> = Vec::new();
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
        let mut r = p;
        while let Some(i) = r.find("<w:t") {
            let r2 = &r[i..];
            let Some(gt) = r2.find('>') else { break };
            let after_t = &r2[gt + 1..];
            let Some(end) = after_t.find("</w:t>") else {
                break;
            };
            text.push_str(&after_t[..end]);
            r = &after_t[end + 6..];
        }
        r = p;
        while let Some(i) = r.find("<w:delText") {
            let r2 = &r[i..];
            let Some(gt) = r2.find('>') else { break };
            let after_t = &r2[gt + 1..];
            let Some(end) = after_t.find("</w:delText>") else {
                break;
            };
            text.push_str(&after_t[..end]);
            r = &after_t[end + 12..];
        }
        paras.push((has_ins, has_del, text));
    }

    // First pure-I is "ONE"
    let one_i = paras
        .iter()
        .position(|(i, d, t)| *i && !*d && t.contains("ONE"));
    let Some(oi) = one_i else {
        panic!("expected pure-I ONE; paras={paras:?}");
    };
    assert_eq!(oi, 0, "Word pure-I first next item at body start");

    // Immediately after ONE: pure-D "Item 1"
    assert!(oi + 1 < paras.len());
    let (i1, d1, t1) = &paras[oi + 1];
    assert!(
        !*i1 && *d1 && t1.contains("Item 1"),
        "Word pure-D first base cluster after ONE; got {i1}/{d1}/{t1:?}"
    );

    // No MIX free-mesh of "a"+"Item"
    for (i, d, t) in &paras {
        if *i && *d {
            panic!("Word has no MIX free-mesh on this pair; got MIX {t:?}");
        }
    }

    // "TWO" is pure-I after the first del cluster
    let two_i = paras
        .iter()
        .position(|(i, d, t)| *i && !*d && t.trim() == "TWO");
    let Some(ti) = two_i else {
        panic!("expected pure-I TWO");
    };
    assert!(ti > oi + 1, "TWO comes after first del cluster");

    // "First shown" pure-D after TWO block
    let first_shown = paras
        .iter()
        .position(|(i, d, t)| !*i && *d && t.contains("First shown"));
    let Some(fi) = first_shown else {
        panic!("expected pure-D First shown");
    };
    assert!(fi > ti, "First shown pure-D after pure-I TWO block");
}
