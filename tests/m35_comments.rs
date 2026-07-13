//! M35 — comments carryover (word mode). Word's Compare carries comments
//! through the redline (comments_carryover_forensics.md): union of both
//! sides' comment sets — when B's set ⊇ A's, B's four comment parts are
//! emitted byte-identical; when only one side has comments, that side's are
//! carried. Anchors (commentRangeStart/End/commentReference) are re-emitted
//! at the equivalent text positions in the merged body and survive del/ins
//! wrapping. Never an orphaned comments part.

use std::collections::HashSet;

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

/// Fresh pair: A has 4 comments (ids 0,1,3,4), B has 6 (superset, +19,20).
/// GT (docx_lots_of_comments_addition_redline.docx): B's four comment parts
/// byte-identical, 6/6/6 anchors. Ours must carry B's parts and anchor all 6.
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
    let pkg = PartFs::open(&out).unwrap();
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
    let pkg = PartFs::open(&out).unwrap();

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
