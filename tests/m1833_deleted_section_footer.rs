// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1833 — a header or footer the revision drops, which only section
//! breaks the redline deletes reference, is deleted as a whole story in
//! Word mode: every paragraph mark goes, the closing one included, and the
//! paragraphs keep their own properties. Word's own redline keeps the
//! closing mark live, and Word 16.115 crashes opening it when the footer
//! holds a PAGE or NUMPAGES field (the file_N family, 9 of the 23 of
//! #1814). With the marks deleted Word opens 7 of the 9 (round 18), and
//! its Accept All and Reject All give an empty story and the original one.
//! s176 and s472 still crash: a deleted break references the footer at
//! all, whatever its marks, and `jubarte validate` says so
//! (`DELETED_SECTION_PAGE_FIELD`).
//!
//! - s548: `df9ffe9a8c_file_16` → `ec900bd182_file_17`. Three sections;
//!   the first break references header1 and footer1 ("Page PAGE of
//!   NUMPAGES"); the revision has one section and no header or footer.
//! - s016: `0483e899af_file_143` → `cc82deb093_file_144`. The same footer,
//!   referenced by the live final section, keeps Word's shape.

mod common;

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use common::validity::{assert_word_valid_package, check_word_valid_package};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::{R, W};
use jubarte::opc::PartFs;
use jubarte::xmllinq::{Dom, NodeId};

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/neurotic_docx_bench/corpus/word/clean/docx")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn parse(pkg: &PartFs, part: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string(part).unwrap());
    let root = dom.root(doc).unwrap();
    (dom, root)
}

/// The header and footer parts the redline's section breaks reference, as
/// (only deleted breaks reference it, part name).
fn story_parts(pkg: &PartFs) -> Vec<(bool, String)> {
    let main = "word/document.xml";
    let targets: HashMap<String, String> = pkg
        .read_rels_for(main)
        .unwrap()
        .items
        .iter()
        .map(|r| (r.id.clone(), pkg.resolve_rel_target(main, &r.target)))
        .collect();
    let (dom, root) = parse(pkg, main);
    let (mut dead, mut live) = (BTreeSet::new(), BTreeSet::new());
    for sect in dom.descendants(root, Some(&W::sect_pr())) {
        let Some(parent) = dom.parent(sect) else {
            continue;
        };
        if dom.name_is(parent, &W::name("sectPrChange")) {
            continue;
        }
        let deleted = dom.name_is(parent, &W::p_pr())
            && dom
                .element(parent, &W::r_pr())
                .is_some_and(|r| dom.element(r, &W::del()).is_some());
        for r in dom.elements(sect, None) {
            if !(dom.name_is(r, &W::name("headerReference"))
                || dom.name_is(r, &W::name("footerReference")))
            {
                continue;
            }
            let part = targets[dom.attribute(r, &R::name("id")).unwrap()].clone();
            if deleted {
                dead.insert(part);
            } else {
                live.insert(part);
            }
        }
    }
    dead.iter()
        .chain(live.iter())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|p| (!live.contains(p), p.clone()))
        .collect()
}

/// Each paragraph of a story: (its mark deleted, its own `w:jc`, a
/// `pPrChange` present).
fn marks(pkg: &PartFs, part: &str) -> Vec<(bool, Option<String>, bool)> {
    let (dom, root) = parse(pkg, part);
    dom.descendants(root, Some(&W::p()))
        .into_iter()
        .map(|p| {
            let ppr = dom.element(p, &W::p_pr());
            let deleted = ppr
                .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                .is_some_and(|r| dom.element(r, &W::del()).is_some());
            let jc = ppr
                .and_then(|ppr| dom.element(ppr, &W::name("jc")))
                .and_then(|jc| dom.attribute(jc, &W::val()).map(str::to_string));
            let change = ppr.is_some_and(|ppr| dom.element(ppr, &W::name("pPrChange")).is_some());
            (deleted, jc, change)
        })
        .collect()
}

/// A story's live text and field codes, in order.
fn live_text(pkg: &PartFs, part: &str) -> String {
    let (dom, root) = parse(pkg, part);
    dom.descendants(root, None)
        .into_iter()
        .filter(|&n| dom.name_is(n, &W::t()) || dom.name_is(n, &W::name("instrText")))
        .map(|n| dom.value(n))
        .collect()
}

#[test]
fn s548_deletes_every_mark_of_the_stories_only_deleted_breaks_reference() {
    let (a, b) = (
        fixture("df9ffe9a8c_file_16.docx"),
        fixture("ec900bd182_file_17.docx"),
    );
    let redline =
        compare_documents_with_settings(&a, &b, &WmlComparerSettings::default()).expect("compare");
    // The base carries commentsExtended/Ids/Extensible without a comments
    // part; the redline keeps them. Compare adds only the lead that the
    // deleted break still references the page-number footer: Word opens
    // s548's redline, but this shape still crashed it on s176 and s472.
    let mut inherited = check_word_valid_package(&a).errors;
    let report = check_word_valid_package(&redline);
    let (leads, others): (Vec<_>, Vec<_>) = report
        .findings
        .iter()
        .partition(|f| f.code == "DELETED_SECTION_PAGE_FIELD");
    let mut actual: Vec<String> = others
        .iter()
        .map(|f| format!("{} [{}#{}]", f.message, f.part, f.location))
        .collect();
    inherited.sort();
    actual.sort();
    assert_eq!(actual, inherited);
    assert_eq!(
        leads
            .iter()
            .map(|f| (f.part.as_str(), f.location.as_str()))
            .collect::<Vec<_>>(),
        [("word/footer1.xml", "w:p[0]")]
    );
    let pkg = PartFs::open(&redline).unwrap();
    let dead: Vec<String> = story_parts(&pkg)
        .into_iter()
        .filter_map(|(dead, part)| dead.then_some(part))
        .collect();
    assert_eq!(dead, ["word/footer1.xml", "word/header1.xml"]);
    for part in &dead {
        assert_eq!(live_text(&pkg, part), "", "{part}: every run deleted");
        let marks = marks(&pkg, part);
        assert!(
            marks.iter().all(|(deleted, _, change)| *deleted && !change),
            "{part}: every mark deleted, own properties kept: {marks:?}"
        );
    }
    assert_eq!(
        marks(&pkg, "word/footer1.xml").last().unwrap().1.as_deref(),
        Some("right"),
        "the footer keeps its own alignment"
    );

    let base = PartFs::open(&a).unwrap();
    let accepted = PartFs::open(&accept_revisions(&redline).unwrap()).unwrap();
    let rejected = PartFs::open(&reject_revisions(&redline).unwrap()).unwrap();
    for part in &dead {
        assert_eq!(
            live_text(&accepted, part),
            "",
            "{part}: accepted, nothing shows"
        );
        assert_eq!(
            live_text(&rejected, part),
            live_text(&base, part),
            "{part}: rejected, the original story"
        );
        assert!(
            live_text(&base, part).len() > 10,
            "{part}: the base has text"
        );
    }
    assert_eq!(
        marks(&rejected, "word/footer1.xml"),
        marks(&base, "word/footer1.xml"),
        "rejected, the footer's marks and alignment are the original's"
    );
}

#[test]
fn s016_keeps_word_s_shape_for_a_footer_a_live_section_references() {
    let redline = compare_documents_with_settings(
        &fixture("0483e899af_file_143.docx"),
        &fixture("cc82deb093_file_144.docx"),
        &WmlComparerSettings::default(),
    )
    .expect("compare");
    assert_word_valid_package(&redline);
    let pkg = PartFs::open(&redline).unwrap();
    let parts = story_parts(&pkg);
    assert!(
        !parts.is_empty() && parts.iter().all(|(dead, _)| !dead),
        "{parts:?}"
    );
    let footer = "word/footer1.xml";
    assert!(parts.iter().any(|(_, p)| p == footer), "{parts:?}");
    assert_eq!(live_text(&pkg, footer), "", "every run deleted");
    let marks = marks(&pkg, footer);
    assert!(
        !marks.last().unwrap().0,
        "the closing mark stays live: {marks:?}"
    );
}
