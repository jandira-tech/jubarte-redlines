// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1851 — the revision's words sit in a paragraph whose mark is deleted,
//! right before a table every row of which is deleted. Word's own Compare
//! writes it, and Accept All joins the words to the paragraph after the
//! table. Word 16.115 crashed opening 6 of the 16 release-sample redlines
//! (0.14.0) with that shape, its table cache corrupt at the deleted row's
//! end. The comparer moves the words to the start of the paragraph they
//! join: Word opened w06, w10, w14 and w17 so, and its Accept All and
//! Reject All gave back the revision's and the original's text (fix23
//! probe33, rt30).
//!
//! - w06 (s187): `45c1c5abec_table_vmerge_colspan` → `a94f4c516b_text_box`.
//!   Both tables deleted; the words are the revision's text box anchor.
//! - w10 (s278): `6867e05e76_00b0d42bbea3…` → `1424386e9b_017447de8375…`.
//!   Both tables deleted; the words are the revision's text.

mod common;

use std::path::PathBuf;

use common::validity::check_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::{Dom, NodeId};

/// A corpus document, by its path under `corpus/word`.
fn fixture(path: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/neurotic_docx_bench/corpus/word")
        .join(path);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The pairs: (sample id, original, revision).
const PAIRS: [(&str, &str, &str); 2] = [
    (
        "w06",
        "clean/docx/45c1c5abec_table_vmerge_colspan.docx",
        "clean/docx/a94f4c516b_text_box.docx",
    ),
    (
        "w10",
        "clean/docx/6867e05e76_00b0d42bbea3b9bac7bc95466cf416f6c89a8cda62bbf3d7.docx",
        "clean/docx/1424386e9b_017447de83753a0e3b7afc659df038ef56972925336c6a5b.docx",
    ),
];

fn body(pkg: &PartFs) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (dom, body)
}

/// Each body paragraph's text, outside tables, a drawing shown as `[d]`.
fn paragraphs(bytes: &[u8]) -> Vec<String> {
    let (dom, body) = body(&PartFs::open(bytes).unwrap());
    dom.elements(body, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.descendants(p, None)
                .into_iter()
                .filter_map(|n| {
                    if dom.name_is(n, &W::t()) {
                        Some(dom.value(n))
                    } else if dom.name_is(n, &W::name("drawing")) {
                        Some("[d]".to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

/// The body's blocks: `T` a table every row of which is deleted, `p` a
/// paragraph whose mark is deleted, `P` one that also holds `w:ins`
/// children, `S` a live paragraph starting with `w:ins`, `L` another.
fn blocks(pkg: &PartFs) -> String {
    let (dom, body) = body(pkg);
    dom.elements(body, None)
        .into_iter()
        .filter_map(|block| {
            if dom.name_is(block, &W::tbl()) {
                let rows = dom.elements(block, Some(&W::tr()));
                let deleted = rows.iter().all(|&row| {
                    dom.element(row, &W::tr_pr())
                        .is_some_and(|pr| dom.element(pr, &W::del()).is_some())
                });
                return Some(if deleted { 'T' } else { 't' });
            }
            if !dom.name_is(block, &W::p()) {
                return None;
            }
            let deleted = dom
                .element(block, &W::p_pr())
                .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                .is_some_and(|rpr| dom.element(rpr, &W::del()).is_some());
            let children: Vec<NodeId> = dom
                .elements(block, None)
                .into_iter()
                .filter(|&c| !dom.name_is(c, &W::p_pr()))
                .collect();
            let inserted = children.iter().any(|&c| dom.name_is(c, &W::ins()));
            Some(match (deleted, inserted) {
                (true, true) => 'P',
                (true, false) => 'p',
                _ if children.first().is_some_and(|&c| dom.name_is(c, &W::ins())) => 'S',
                _ => 'L',
            })
        })
        .collect()
}

#[test]
fn the_revision_s_words_start_the_paragraph_after_the_deleted_tables() {
    for (id, a, b) in PAIRS {
        let (a, b) = (fixture(a), fixture(b));
        let redline = compare_documents_with_settings(&a, &b, &WmlComparerSettings::default())
            .expect("compare");
        let report = check_word_valid_package(&redline);
        assert_eq!(report.errors, Vec::<String>::new(), "{id}");
        let pkg = PartFs::open(&redline).unwrap();
        assert_eq!(
            blocks(&pkg),
            "TpTS",
            "{id}: the paragraph between the tables keeps its deleted mark, the words start the last"
        );
        assert_eq!(
            paragraphs(&accept_revisions(&redline).unwrap()),
            paragraphs(&b),
            "{id}: accepted, the revision"
        );
        assert_eq!(
            paragraphs(&reject_revisions(&redline).unwrap()),
            paragraphs(&a),
            "{id}: rejected, the original"
        );
    }
}
