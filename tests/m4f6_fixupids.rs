// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.F.6 — FixUp*Ids tests.

use jubarte::comparer::fixups::{
    fix_up_doc_pr_ids, fix_up_group_ids, fix_up_shape_ids, fix_up_shape_type_ids,
};
use jubarte::namespaces::{O, VML, W, WP};
use jubarte::xmllinq::{Dom, XName};

fn id() -> XName {
    XName::get("id", "")
}

#[test]
fn m4_f6_docpr_ids() {
    let mut d = Dom::new();
    let body = d.new_element(W::body());
    for old in ["7", "7", "99"] {
        let dp = d.new_element(WP::name("docPr"));
        d.set_attribute_value(dp, &id(), Some(old));
        d.add(body, dp);
    }
    fix_up_doc_pr_ids(&mut d, body);
    let ids: Vec<_> = d
        .descendants(body, Some(&WP::name("docPr")))
        .iter()
        .map(|&e| d.attribute(e, &id()).unwrap().to_string())
        .collect();
    assert_eq!(ids, vec!["1", "2", "3"]);
}

#[test]
fn m4_f6_shape_ids_relink_ole() {
    let mut d = Dom::new();
    let pict = d.new_element(W::name("pict"));
    let shape = d.new_element(VML::name("shape"));
    d.set_attribute_value(shape, &id(), Some("_x0000_s1026"));
    let ole = d.new_element(O::name("OLEObject"));
    d.set_attribute_value(ole, &XName::get("ShapeID", ""), Some("_x0000_s1026"));
    d.add(pict, shape);
    d.add(pict, ole);
    fix_up_shape_ids(&mut d, pict);
    let new_shape_id = d.attribute(shape, &id()).unwrap().to_string();
    assert_eq!(new_shape_id, "1");
    assert_eq!(
        d.attribute(ole, &XName::get("ShapeID", "")).unwrap(),
        "1",
        "OLEObject ShapeID relinked"
    );
}

#[test]
fn m4_f6_shapetype_relink() {
    let mut d = Dom::new();
    let pict = d.new_element(W::name("pict"));
    let st = d.new_element(VML::name("shapetype"));
    d.set_attribute_value(st, &id(), Some("t75"));
    let shape = d.new_element(VML::name("shape"));
    d.set_attribute_value(shape, &XName::get("type", ""), Some("#t75"));
    d.add(pict, st);
    d.add(pict, shape);
    fix_up_shape_type_ids(&mut d, pict);
    assert_eq!(d.attribute(st, &id()).unwrap(), "1");
    assert_eq!(
        d.attribute(shape, &XName::get("type", "")).unwrap(),
        "#1",
        "shape type relinked"
    );
}

#[test]
fn m4_f6_group_ids() {
    // `fix_up_group_ids` renumbers `v:group/@id` from 1 (FixUpGroupIds :5986). It is
    // the consolidate-path helper (not wired into compare/produce — see
    // comparer/mod.rs), so cover it directly here to keep it from rotting.
    let mut d = Dom::new();
    let pict = d.new_element(W::name("pict"));
    for old in ["5", "9", "5"] {
        let g = d.new_element(VML::name("group"));
        d.set_attribute_value(g, &id(), Some(old));
        d.add(pict, g);
    }
    fix_up_group_ids(&mut d, pict);
    let ids: Vec<_> = d
        .descendants(pict, Some(&VML::name("group")))
        .iter()
        .map(|&e| d.attribute(e, &id()).unwrap().to_string())
        .collect();
    assert_eq!(ids, vec!["1", "2", "3"]);
}

/// End-of-produce FixUpDocPrIds: drawing-heavy pairs must not emit duplicate
/// wp:docPr/@id after Word-mode merge/wrap (parity S-dup-docpr-id).
#[test]
fn m4_f6_docpr_ids_unique_after_full_compare() {
    use jubarte::document_comparer::compare_documents;
    use std::collections::HashMap;
    use std::io::Read;
    use zip::ZipArchive;

    let corpus = {
        let root = std::env::var("BENCH_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../neurotic_docx_bench"));
        let c = root.join("corpus/word_based/docx_source");
        if !c.is_dir() {
            eprintln!("skip: bench corpus missing");
            return;
        }
        c
    };
    let corpus = corpus.to_str().unwrap();
    let a = std::fs::read(format!("{corpus}/strict01_sdt_controls.docx"))
        .expect("strict01_sdt_controls fixture");
    let b = std::fs::read(format!("{corpus}/strict01.docx")).expect("strict01 fixture");
    let out = compare_documents(&a, &b, "Test").expect("compare");
    let mut zip = ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    // Namespace-agnostic id harvest (mirrors parity_ladder S-dup-docpr-id).
    let mut ids = Vec::new();
    let mut rest = xml.as_str();
    while let Some(i) = rest.find("docPr") {
        let slice = &rest[i..];
        if let Some(id_at) = slice.find("id=\"") {
            let after = &slice[id_at + 4..];
            if let Some(end) = after.find('"') {
                ids.push(after[..end].to_string());
            }
        }
        rest = &rest[i + 4..];
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    for id in &ids {
        *counts.entry(id.clone()).or_default() += 1;
    }
    let dups: Vec<_> = counts
        .iter()
        .filter(|(_, c)| **c > 1)
        .map(|(k, _)| k.clone())
        .collect();
    assert!(
        dups.is_empty(),
        "duplicate wp:docPr/@id after compare: {dups:?} (all={ids:?})"
    );
}
