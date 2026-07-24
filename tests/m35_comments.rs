// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M35 — comments carryover (word mode). Word's Compare carries comments
//! through the redline (comments_carryover_forensics.md): union of both
//! sides' comment sets — when B's set ⊇ A's, B's four comment parts are
//! emitted byte-identical; when only one side has comments, that side's are
//! carried. Anchors (commentRangeStart/End/commentReference) are re-emitted
//! at the equivalent text positions in the merged body and survive del/ins
//! wrapping. Never an orphaned comments part.

mod common;

use std::collections::HashSet;

use common::validity::assert_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::Dom;

const FRESH: &str = "tests/corpus/fresh_docx_fixtures_and_redlines";
const ORIG: &str = "tests/corpus/_fixtures/original_fixtures";

fn orig_fixtures_present() -> bool {
    if std::path::Path::new(ORIG).is_dir() {
        true
    } else {
        eprintln!("SKIP: _fixtures/original_fixtures corpus not present");
        false
    }
}

/// Self-skip when the local corpus path is absent (clean CI clones).
fn require_path(path: &str) -> bool {
    if std::path::Path::new(path).exists() {
        true
    } else {
        eprintln!("skipping: external fixture not present: {path}");
        false
    }
}

fn word_mode() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    }
}

fn comment_ids(pkg: &PartFs) -> HashSet<String> {
    let Some(xml) = pkg.part_string("word/comments.xml") else {
        return HashSet::new();
    };
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    dom.elements(root, Some(&W::name("comment")))
        .into_iter()
        .filter_map(|c| dom.attribute(c, &W::name("id")).map(str::to_string))
        .collect()
}

/// (rangeStart ids, rangeEnd ids, reference ids) in document order.
fn anchor_ids(pkg: &PartFs) -> (Vec<String>, Vec<String>, Vec<String>) {
    let xml = pkg.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let grab = |name: &str| -> Vec<String> {
        dom.descendants(root, Some(&W::name(name)))
            .into_iter()
            .filter_map(|e| dom.attribute(e, &W::name("id")).map(str::to_string))
            .collect()
    };
    (
        grab("commentRangeStart"),
        grab("commentRangeEnd"),
        grab("commentReference"),
    )
}

fn open_valid_output(out: &[u8]) -> PartFs {
    assert_word_valid_package(out);
    PartFs::open(out).expect("open")
}

// Fresh pair: A has 4 comments (ids 0,1,3,4), B has 6 (superset, +19,20).
// GT (docx_lots_of_comments_addition_redline.docx): B's four comment parts
// byte-identical, 6/6/6 anchors. Ours must carry B's parts and anchor all 6.
//
// A plain comment, not a doc comment: it describes the fixture scenario the
// tests below exercise, not `optional_bench_docx` — which is just the loader.

fn optional_bench_docx(name: &str) -> Option<Vec<u8>> {
    let root = std::env::var_os("BENCH_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let p =
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../neurotic_docx_bench");
            p.is_dir().then_some(p)
        })?;
    std::fs::read(root.join("corpus/word_based/docx_source").join(name)).ok()
}

#[test]
fn w1_superset_carries_revised_parts_byte_identical_with_anchors() {
    if !orig_fixtures_present() {
        return;
    }
    let a_path = format!("{FRESH}/docx_lots_of_comments.docx");
    let b_path = format!("{FRESH}/docx_lots_of_comments_addition.docx");
    if !require_path(&a_path) || !require_path(&b_path) {
        return;
    }
    let a = std::fs::read(&a_path).unwrap();
    let b = std::fs::read(&b_path).unwrap();
    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = open_valid_output(&out);
    let pkg_b = PartFs::open(&b).unwrap();

    // B's four comment parts carried byte-identical
    for part in [
        "word/comments.xml",
        "word/commentsExtended.xml",
        "word/commentsIds.xml",
        "word/commentsExtensible.xml",
    ] {
        assert_eq!(
            pkg.part_bytes(part),
            pkg_b.part_bytes(part),
            "{part} must be byte-identical to the revised (B) document's"
        );
    }

    let want: HashSet<String> = ["0", "1", "3", "4", "19", "20"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(comment_ids(&pkg), want, "6 comments with B's ids");

    let (starts, ends, refs) = anchor_ids(&pkg);
    assert_eq!(starts.len(), 6, "6 commentRangeStart, got {starts:?}");
    assert_eq!(ends.len(), 6, "6 commentRangeEnd, got {ends:?}");
    assert_eq!(refs.len(), 6, "6 commentReference, got {refs:?}");
    let anchored: HashSet<String> = starts.iter().cloned().collect();
    assert_eq!(anchored, want, "every comment id anchored — no orphans");
    assert_eq!(
        ends.iter().cloned().collect::<HashSet<_>>(),
        want,
        "every range closed"
    );
}

/// Single-side: only A (potpourritest, 3 comments ids 0,1,2) has comments;
/// B (product-roadmap suggesting-insertions) has none. GT carries A's
/// comments WITH anchors (they sit inside w:del around the deleted text).
/// Current bug: parts copied but 0 anchors — an orphaned comments part.
#[test]
fn w2_single_side_comments_carried_with_anchors_not_orphaned() {
    if !orig_fixtures_present() {
        return;
    }
    let a_path = format!("{ORIG}/potpourritest.docx");
    let b_path = format!("{ORIG}/product-roadmap-2026.suggesting-insertions.docx");
    if !require_path(&a_path) || !require_path(&b_path) {
        return;
    }
    let a = std::fs::read(&a_path).unwrap();
    let b = std::fs::read(&b_path).unwrap();
    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = open_valid_output(&out);

    let ids = comment_ids(&pkg);
    let (starts, ends, refs) = anchor_ids(&pkg);
    // No orphaned part: whatever comments remain in the part must be anchored.
    assert!(
        !ids.is_empty(),
        "A's comments must be carried (GT keeps all 3)"
    );
    assert_eq!(ids.len(), 3, "all 3 of A's comments carried: {ids:?}");
    assert_eq!(starts.len(), 3, "3 commentRangeStart, got {starts:?}");
    assert_eq!(ends.len(), 3, "3 commentRangeEnd, got {ends:?}");
    assert_eq!(refs.len(), 3, "3 commentReference, got {refs:?}");
    // All three anchor kinds must carry the same id set as comments.xml —
    // count-only checks let a wrong id on end/reference slip through.
    assert_eq!(
        starts.iter().cloned().collect::<HashSet<_>>(),
        ids,
        "commentRangeStart ids match the comment part"
    );
    assert_eq!(
        ends.iter().cloned().collect::<HashSet<_>>(),
        ids,
        "commentRangeEnd ids match the comment part"
    );
    assert_eq!(
        refs.iter().cloned().collect::<HashSet<_>>(),
        ids,
        "commentReference ids match the comment part"
    );
}

/// accept_revisions must not drop comment range markers (nested ends after
/// tables; starts inside w:del). Regression: outer nested ends and del-hoisted
/// starts were lost → comment carry 2/6 on document_100×lots_of_comments.
/// (document_100 × lots_of_comments redline: all 6 B comments should anchor.)
#[test]
fn accept_revisions_preserves_comment_range_markers() {
    let Some(b_path) =
        optional_bench_docx("docx_lots_of_comments_addition_redline_addition_v_removal.docx")
    else {
        eprintln!("skip: missing bench fixture");
        return;
    };
    let b = b_path.clone();
    let list_ids = |bytes: &[u8], tag: &str| -> HashSet<String> {
        let pkg = PartFs::open(bytes).unwrap();
        let xml = pkg.part_string("word/document.xml").unwrap();
        let mut dom = Dom::new();
        let d = dom.parse_xdocument(&xml);
        let root = dom.root(d).unwrap();
        dom.descendants(root, Some(&W::name(tag)))
            .into_iter()
            .filter_map(|e| dom.attribute(e, &W::name("id")).map(str::to_string))
            .collect()
    };
    let before_s = list_ids(&b, "commentRangeStart");
    let before_e = list_ids(&b, "commentRangeEnd");
    let accepted = jubarte::document_comparer::accept_revisions(&b).unwrap();
    let after_s = list_ids(&accepted, "commentRangeStart");
    let after_e = list_ids(&accepted, "commentRangeEnd");
    let dropped_s: Vec<_> = before_s.difference(&after_s).collect();
    let dropped_e: Vec<_> = before_e.difference(&after_e).collect();
    assert_eq!(
        after_s, before_s,
        "accept dropped commentRangeStart ids: {dropped_s:?}"
    );
    assert_eq!(
        after_e, before_e,
        "accept dropped commentRangeEnd ids: {dropped_e:?}"
    );
}

/// document_100 (no comments) × lots_of_comments redline (6 comment *ids* on B,
/// only 4 unique bodies — Complex/Threaded are duplicated). Word redline keeps
/// **4** (one per body). Carry unique bodies with matched anchors (C2).
#[test]
fn document100_vs_lots_of_comments_carries_unique_bodies() {
    let Some(a_path) = optional_bench_docx("document_100_ultimate_demo_id_paraid_overflow.docx")
    else {
        eprintln!("skip: missing bench fixture");
        return;
    };
    let Some(b_path) =
        optional_bench_docx("docx_lots_of_comments_addition_redline_addition_v_removal.docx")
    else {
        eprintln!("skip: missing bench fixture");
        return;
    };
    let a = a_path.clone();
    let b = b_path.clone();
    let pkg_b = PartFs::open(&b).unwrap();
    let b_ids = comment_ids(&pkg_b);
    assert_eq!(b_ids.len(), 6, "fixture must have 6 B comment ids");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = open_valid_output(&out);
    let ids = comment_ids(&pkg);
    let (s, e, r) = anchor_ids(&pkg);
    // Word-oracle parity: 4 unique bodies, not the raw 6-id set.
    assert_eq!(
        ids.len(),
        4,
        "Word keeps one def per unique body text; got {ids:?}"
    );
    assert_eq!(s.len(), 4, "starts={s:?}");
    assert_eq!(e.len(), 4, "ends={e:?}");
    assert_eq!(r.len(), 4, "refs={r:?}");
    // Every remaining anchor resolves to a comment def.
    for id in s.iter().chain(e.iter()).chain(r.iter()) {
        assert!(ids.contains(id), "orphan anchor id {id}");
    }
}

/// Same comment *texts* on A and B under different ids (Word renumbered the
/// set across two redline-derived sources). Union-by-id produces 12 comments;
/// Word keeps one per unique body (4 — fixtures ship Complex/Threaded dups).
/// Prefer B's install path, then body-text dedupe.
#[test]
fn renumbered_same_text_comments_prefer_b_not_double_union() {
    let Some(a_path) = optional_bench_docx("docx_lots_of_comments_addition_redline.docx") else {
        eprintln!("skip: missing bench fixture");
        return;
    };
    let Some(b_path) = optional_bench_docx(
        "docx_lots_of_comments_addition_removal_redline_removal_v_addition.docx",
    ) else {
        eprintln!("skip: missing bench fixture");
        return;
    };
    let a = a_path.clone();
    let b = b_path.clone();
    let pkg_a = PartFs::open(&a).unwrap();
    let pkg_b = PartFs::open(&b).unwrap();
    let a_ids = comment_ids(&pkg_a);
    let b_ids = comment_ids(&pkg_b);
    assert_eq!(a_ids.len(), 6);
    assert_eq!(b_ids.len(), 6);
    assert!(
        a_ids != b_ids,
        "fixture premise: ids differ so bare id-match fails"
    );
    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = open_valid_output(&out);
    let ids = comment_ids(&pkg);
    let (s, e, r) = anchor_ids(&pkg);
    assert_eq!(
        ids.len(),
        4,
        "must not double-union; one def per unique body; got {ids:?}"
    );
    assert_eq!(s.len(), 4, "starts={s:?}");
    assert_eq!(e.len(), 4, "ends={e:?}");
    assert_eq!(r.len(), 4, "refs={r:?}");
    // Surviving ids must be a subset of B's (text-cover path installs B first).
    assert!(
        ids.is_subset(&b_ids),
        "carried ids must come from B: {ids:?} not subset of {b_ids:?}"
    );
}
