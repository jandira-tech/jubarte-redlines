// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1833 — the revision drops a header or footer that only section breaks
//! the redline deletes reference (the file_N family, 9 of the 23 of
//! #1814). Word's own redline keeps the story's closing mark live, and
//! Word 16.115 crashes opening it when the footer holds a PAGE or NUMPAGES
//! field.
//!
//! - Word mode deletes such a story whole: every paragraph mark goes, the
//!   closing one included, and the paragraphs keep their own properties.
//! - A deleted break that references a page-number footer at all crashed
//!   Word, whatever the story's marks (s176, s472, round 18). So the
//!   footer the final section inherits through deleted breaks moves to the
//!   final section, and no deleted break references it any more. Word
//!   opened all 9 that way. Its Accept All shows nothing; its Reject All
//!   shows the footer on the last section only (docs/WORD_DIFFERENCES.md).
//!   Word's own redlines of the six pairs that crash with every mark
//!   deleted crash too (round 23).
//! - The move needs a deleted table row (Word logs the crash in its table
//!   cache, at a deleted row's end), and no earlier section setting the
//!   slot: otherwise Reject All would show that section's footer on the
//!   moved break's pages. The trust (`24b9811b05_file_22` →
//!   `7226967373_file_23`) keeps its exhibit footers where they are.
//!
//! - s548: `df9ffe9a8c_file_16` → `ec900bd182_file_17`. Three sections;
//!   the first break references header1 and footer1 ("Page PAGE of
//!   NUMPAGES"); the revision has one section and no header or footer.
//! - s176: `3f91c8f0cd_file_9` → `e3d833aef0_file_10`, and s472:
//!   `b175a00954_file_27` → `f32428a03a_file_28`: the same shape, which
//!   still crashed Word with every mark deleted.
//! - s016: `0483e899af_file_143` → `cc82deb093_file_144`. The same footer,
//!   referenced by the live final section, keeps Word's shape.

mod common;

use std::collections::HashMap;
use std::path::PathBuf;

use common::validity::{assert_word_valid_package, check_word_valid_package};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::{R, W};
use jubarte::opc::PartFs;
use jubarte::xmllinq::{Dom, NodeId};

/// A corpus document, by its path under `corpus/word`.
fn fixture(path: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/neurotic_docx_bench/corpus/word")
        .join(path);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The file_N pairs: (sample id, original, revision).
const FAMILY: [(&str, &str, &str); 3] = [
    (
        "s548",
        "clean/docx/df9ffe9a8c_file_16.docx",
        "clean/docx/ec900bd182_file_17.docx",
    ),
    (
        "s176",
        "with_comments_tracking/docx/3f91c8f0cd_file_9.docx",
        "clean/docx/e3d833aef0_file_10.docx",
    ),
    (
        "s472",
        "with_comments_tracking/docx/b175a00954_file_27.docx",
        "clean/docx/f32428a03a_file_28.docx",
    ),
];

fn parse(pkg: &PartFs, part: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string(part).unwrap());
    let root = dom.root(doc).unwrap();
    (dom, root)
}

/// Each section of `pkg`'s body in order, the body's own last, as (its
/// break is deleted, the header/footer parts it references).
fn sections(pkg: &PartFs) -> Vec<(bool, Vec<String>)> {
    let main = "word/document.xml";
    let targets: HashMap<String, String> = pkg
        .read_rels_for(main)
        .unwrap()
        .items
        .iter()
        .map(|r| (r.id.clone(), pkg.resolve_rel_target(main, &r.target)))
        .collect();
    let (dom, root) = parse(pkg, main);
    dom.descendants(root, Some(&W::sect_pr()))
        .into_iter()
        .filter(|&s| {
            dom.parent(s)
                .is_some_and(|p| dom.name_is(p, &W::p_pr()) || dom.name_is(p, &W::body()))
        })
        .map(|sect| {
            let parent = dom.parent(sect).unwrap();
            let deleted = dom
                .element(parent, &W::r_pr())
                .is_some_and(|r| dom.element(r, &W::del()).is_some());
            let mut parts: Vec<String> = dom
                .elements(sect, None)
                .into_iter()
                .filter(|&r| {
                    dom.name_is(r, &W::name("headerReference"))
                        || dom.name_is(r, &W::name("footerReference"))
                })
                .map(|r| targets[dom.attribute(r, &R::name("id")).unwrap()].clone())
                .collect();
            parts.sort();
            (deleted, parts)
        })
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
fn the_family_s_page_footer_leaves_the_deleted_break() {
    for (id, a, b) in FAMILY {
        let (a, b) = (fixture(a), fixture(b));
        let redline = compare_documents_with_settings(&a, &b, &WmlComparerSettings::default())
            .expect("compare");
        // The bases carry findings of their own (s548: commentsExtended/
        // Ids/Extensible without a comments part); compare adds none.
        let mut inherited = check_word_valid_package(&a).errors;
        let mut actual = check_word_valid_package(&redline).errors;
        inherited.sort();
        actual.sort();
        assert_eq!(actual, inherited, "{id}");
        let pkg = PartFs::open(&redline).unwrap();
        assert_eq!(
            sections(&pkg),
            [
                (true, vec!["word/header1.xml".to_string()]),
                (true, vec![]),
                (false, vec!["word/footer1.xml".to_string()]),
            ],
            "{id}: the header stays on its deleted break, the footer goes to the final section"
        );
        for part in ["word/header1.xml", "word/footer1.xml"] {
            assert_eq!(live_text(&pkg, part), "", "{id} {part}: every run deleted");
            let marks = marks(&pkg, part);
            assert!(
                marks.iter().all(|(deleted, _, change)| *deleted && !change),
                "{id} {part}: every mark deleted, own properties kept: {marks:?}"
            );
        }
        assert_eq!(
            marks(&pkg, "word/footer1.xml").last().unwrap().1.as_deref(),
            Some("right"),
            "{id}: the footer keeps its own alignment"
        );
    }
}

#[test]
fn s548_accepts_to_nothing_and_rejects_to_the_original_stories() {
    let (_, a, b) = FAMILY[0];
    let (a, b) = (fixture(a), fixture(b));
    let redline =
        compare_documents_with_settings(&a, &b, &WmlComparerSettings::default()).expect("compare");
    let base = PartFs::open(&a).unwrap();
    let accepted = PartFs::open(&accept_revisions(&redline).unwrap()).unwrap();
    let rejected = PartFs::open(&reject_revisions(&redline).unwrap()).unwrap();
    // Accepted, the one section references the emptied footer, as in Word's
    // Accept All; nothing references the header any more, and it goes.
    assert_eq!(
        sections(&accepted),
        [(false, vec!["word/footer1.xml".to_string()])]
    );
    assert_eq!(live_text(&accepted, "word/footer1.xml"), "");
    assert!(accepted.part_string("word/header1.xml").is_none());
    for part in ["word/header1.xml", "word/footer1.xml"] {
        assert!(
            live_text(&base, part).len() > 10,
            "{part}: the base has text"
        );
        assert_eq!(
            live_text(&rejected, part),
            live_text(&base, part),
            "{part}: rejected, the original story"
        );
    }
    assert_eq!(
        marks(&rejected, "word/footer1.xml"),
        marks(&base, "word/footer1.xml"),
        "rejected, the footer's marks and alignment are the original's"
    );
    // The original references both stories from its first break, and its
    // later sections inherit them. Rejected, the first break keeps the
    // header and the final section references the footer: the footer shows
    // on the last section only, as in Word's Reject All of this shape.
    let header_footer = vec![
        "word/footer1.xml".to_string(),
        "word/header1.xml".to_string(),
    ];
    assert_eq!(
        sections(&base),
        [(false, header_footer), (false, vec![]), (false, vec![])]
    );
    assert_eq!(
        sections(&rejected),
        [
            (false, vec!["word/header1.xml".to_string()]),
            (false, vec![]),
            (false, vec!["word/footer1.xml".to_string()]),
        ]
    );
}

/// The trust `24b9811b05_file_22` → `7226967373_file_23` (full corpus
/// 2fe0dc77f9): 21 sections, each exhibit with its own page-number footer
/// ("Smith Family Trust G-2 - PAGE"). The final section inherits the last
/// deleted break's footer, but an earlier break sets that slot too: moved,
/// Reject All would label G-2's pages "G-1". The reference stays, and the
/// rejected sections reference what the original's do. Word opens it, and
/// Ring-1 reports the 19 breaks as the `DELETED_SECTION_PAGE_FIELD` lead.
#[test]
fn the_trust_s_exhibit_footer_stays_on_its_deleted_break() {
    let a = fixture("clean/docx/24b9811b05_file_22.docx");
    let redline = compare_documents_with_settings(
        &a,
        &fixture("clean/docx/7226967373_file_23.docx"),
        &WmlComparerSettings::default(),
    )
    .expect("compare");
    let lead = |f: &&jubarte::validate::Finding| f.code == "DELETED_SECTION_PAGE_FIELD";
    let mut inherited = check_word_valid_package(&a).errors;
    let report = check_word_valid_package(&redline);
    let mut actual: Vec<&String> = report
        .errors
        .iter()
        .zip(&report.findings)
        .filter(|(_, f)| !lead(f))
        .map(|(e, _)| e)
        .collect();
    inherited.sort();
    actual.sort();
    assert_eq!(actual, inherited.iter().collect::<Vec<_>>());
    let mut leads: Vec<&str> = report
        .findings
        .iter()
        .filter(lead)
        .map(|f| f.part.as_str())
        .collect();
    leads.sort_unstable();
    leads.dedup();
    assert_eq!(leads.len(), 19, "{leads:?}");
    assert!(leads.contains(&"word/footer19.xml"), "{leads:?}");
    let ours = sections(&PartFs::open(&redline).unwrap());
    let (last, earlier) = ours.split_last().unwrap();
    assert!(
        last.1.is_empty(),
        "the final section sets nothing: {last:?}"
    );
    assert!(
        earlier
            .iter()
            .filter(|(_, parts)| parts.iter().any(|p| p.contains("footer")))
            .count()
            > 2,
        "earlier sections set footers of their own: {ours:?}"
    );
    let parts = |pkg: &PartFs| -> Vec<Vec<String>> {
        sections(pkg).into_iter().map(|(_, parts)| parts).collect()
    };
    let rejected = parts(&PartFs::open(&reject_revisions(&redline).unwrap()).unwrap());
    let base = parts(&PartFs::open(&a).unwrap());
    assert_eq!(
        rejected[..rejected.len() - 1],
        base[..base.len() - 1],
        "rejected, every break references the original's stories"
    );
    // The final section is the revision's, which has no footer, and a
    // `sectPrChange` records no references: rejected, it does not get the
    // original's own footer back (before #1833 as well).
    assert_eq!(rejected.last().unwrap(), &Vec::<String>::new());
    assert_eq!(base.last().unwrap(), &vec!["word/footer20.xml".to_string()]);
}

#[test]
fn s016_keeps_word_s_shape_for_a_footer_a_live_section_references() {
    let redline = compare_documents_with_settings(
        &fixture("clean/docx/0483e899af_file_143.docx"),
        &fixture("clean/docx/cc82deb093_file_144.docx"),
        &WmlComparerSettings::default(),
    )
    .expect("compare");
    assert_word_valid_package(&redline);
    let pkg = PartFs::open(&redline).unwrap();
    let footer = "word/footer1.xml";
    let sections = sections(&pkg);
    assert!(
        sections
            .iter()
            .any(|(deleted, parts)| !deleted && parts.iter().any(|p| p == footer)),
        "{sections:?}"
    );
    assert_eq!(live_text(&pkg, footer), "", "every run deleted");
    let marks = marks(&pkg, footer);
    assert!(
        !marks.last().unwrap().0,
        "the closing mark stays live: {marks:?}"
    );
}
