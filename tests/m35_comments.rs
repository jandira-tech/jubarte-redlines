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
    // The corpus copy moved; the fixtures folder keeps the originals.
    [
        "corpus/word_based/docx_source",
        "grok_run/_fixtures/fresh_docx_fixtures_and_redlines",
    ]
    .iter()
    .find_map(|dir| std::fs::read(root.join(dir).join(name)).ok())
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

/// accept_revisions keeps every comment whose reference survives, with its
/// range markers (nested ends after tables; starts inside w:del), and drops
/// the ones whose reference goes with deleted text, as Word's Accept All
/// does. Regression: outer nested ends and del-hoisted starts were lost.
/// The lots_of_comments redline anchors comments 9/10 in deleted text and
/// 2/3/66/67 in live text; the survivors are renumbered, as Word does.
#[test]
fn accept_revisions_preserves_comment_range_markers() {
    let Some(b) =
        optional_bench_docx("docx_lots_of_comments_addition_redline_addition_v_removal.docx")
    else {
        eprintln!("skip: missing bench fixture");
        return;
    };
    let accepted = jubarte::document_comparer::accept_revisions(&b).unwrap();
    let pkg = PartFs::open(&accepted).unwrap();
    let ids = comment_ids(&pkg);
    let (starts, ends, refs) = anchor_ids(&pkg);
    assert_eq!(ids.len(), 4, "the four live comments stay: {ids:?}");
    for (tag, found) in [("start", starts), ("end", ends), ("reference", refs)] {
        assert_eq!(found.len(), 4, "one {tag} each: {found:?}");
        assert_eq!(found.into_iter().collect::<HashSet<_>>(), ids, "{tag} ids");
    }
    let xml = pkg.part_string("word/comments.xml").unwrap();
    for body in [
        "Comment on table.",
        "Threaded comment on table.",
        "Complex comment.",
        "Threaded over complex comment.",
    ] {
        assert!(xml.contains(body), "{body} kept");
    }
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
/// Word's own redline of this pair keeps 6 with 6 anchors: B's set (with
/// Word's renumbered ids), where the
/// Complex/Threaded bodies appear twice because a copied section carries its
/// own pair. A range maps to the copy at its own place, so the two pairs are
/// not collapsed as duplicates (this test once asserted 4, which was that
/// collapse, not Word).
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
    // Word's redline of this pair (corpus/word_based/docx_redlines_word)
    // keeps B's six comments renumbered in document order: 0 1 3 4 10 11.
    let word: HashSet<String> = ["0", "1", "3", "4", "10", "11"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        ids, word,
        "must not double-union: B's six comments with Word's ids"
    );
    assert_eq!(s.len(), 6, "starts={s:?}");
    assert_eq!(e.len(), 6, "ends={e:?}");
    assert_eq!(r.len(), 6, "refs={r:?}");
}

/// A document whose first paragraph carries comment 1 as a point comment: a
/// `w:commentReference` with no `commentRangeStart`/`commentRangeEnd`.
fn point_comment_docx(text: &str) -> Vec<u8> {
    use std::io::Write;
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="{w}"><w:body><w:p><w:r><w:t>{text}</w:t></w:r><w:r><w:commentReference w:id="1"/></w:r></w:p><w:p><w:r><w:t>Tail paragraph.</w:t></w:r></w:p><w:sectPr/></w:body></w:document>"#
    );
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{w}"><w:comment w:id="1" w:author="R" w:initials="R"><w:p><w:r><w:t>Note</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let parts = [
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/comments.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml"/></Types>"#.to_string(),
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.to_string(),
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/></Relationships>"#.to_string(),
        ),
        ("word/document.xml", doc),
        ("word/comments.xml", comments),
    ];
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        for (name, body) in parts {
            z.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            z.write_all(body.as_bytes()).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

/// A point comment survives the redline right after the text it follows, as
/// the empty range Word's redline writes for it (comments.docx comment 2 in
/// Word's clear_formatting × comments redline). It had no range to map, so
/// the carryover dropped it and the whole comments part with it.
#[test]
fn point_comments_are_carried_as_empty_ranges() {
    let a = point_comment_docx("Hello world");
    let b = point_comment_docx("Hello there world");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).unwrap();
    let pkg = open_valid_output(&out);
    assert_eq!(comment_ids(&pkg), HashSet::from(["1".to_string()]));
    let (s, e, r) = anchor_ids(&pkg);
    let one = vec!["1".to_string()];
    assert_eq!((s, e, r), (one.clone(), one.clone(), one));
    let xml = pkg.part_string("word/document.xml").unwrap();
    let first = xml.split("</w:p>").next().unwrap();
    let text_end = first.rfind("world<").expect("text in the first paragraph");
    let start = first
        .find("<w:commentRangeStart")
        .expect("start in the first paragraph");
    let end = first.find("<w:commentRangeEnd").unwrap();
    let reference = first.find("<w:commentReference").unwrap();
    assert!(
        text_end < start && start < end && end < reference,
        "empty range after the text, then the reference: {first}"
    );
}

#[test]
fn multiple_point_comments_inside_one_run_preserve_unicode_text_order() {
    let mut pkg = PartFs::open(&point_comment_docx("unused")).unwrap();
    let body = r#"<w:p><w:r><w:t>ação 🐋</w:t><w:commentReference w:id="1"/><w:t>東京</w:t><w:commentReference w:id="2"/><w:t>tail</w:t></w:r></w:p>"#;
    pkg.set_part(
        "word/document.xml",
        format!(
            r#"<w:document xmlns:w="{}"><w:body>{body}<w:sectPr/></w:body></w:document>"#,
            W::URI
        )
        .into_bytes(),
    );
    let comments = pkg.part_string("word/comments.xml").unwrap().replace(
        "</w:comments>",
        r#"<w:comment w:id="2" w:author="R" w:initials="R"><w:p><w:r><w:t>Second note</w:t></w:r></w:p></w:comment></w:comments>"#,
    );
    pkg.set_part("word/comments.xml", comments.into_bytes());
    let input = pkg.to_zip().unwrap();
    // Change the tail so the comparer projects anchors instead of returning
    // an identical document with its original point-comment representation.
    let revised_xml = pkg
        .part_string("word/document.xml")
        .unwrap()
        .replace("<w:t>tail</w:t>", "<w:t>new tail</w:t>");
    pkg.set_part("word/document.xml", revised_xml.into_bytes());
    let revised = pkg.to_zip().unwrap();
    let out = compare_documents_with_settings(&input, &revised, &word_mode()).unwrap();
    let pkg = open_valid_output(&out);
    let (starts, ends, references) = anchor_ids(&pkg);
    assert_eq!(starts.len(), 2);
    assert_eq!(starts, ends);
    assert_eq!(starts, references);
    assert_eq!(
        starts.iter().cloned().collect::<HashSet<_>>(),
        comment_ids(&pkg)
    );
    // Ids may be remapped by the comparer; the definition must stay at the
    // text position belonging to that comment, regardless of its numeric id.
    let mut comments_dom = Dom::new();
    let comments_doc = comments_dom.parse_xdocument(&pkg.part_string("word/comments.xml").unwrap());
    let comments_root = comments_dom.root(comments_doc).unwrap();
    let notes: std::collections::HashMap<String, String> = comments_dom
        .descendants(comments_root, Some(&W::name("comment")))
        .into_iter()
        .map(|c| {
            (
                comments_dom
                    .attribute(c, &W::name("id"))
                    .unwrap()
                    .to_string(),
                comments_dom.value(c),
            )
        })
        .collect();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let root = dom.root(doc).unwrap();
    let mut text = String::new();
    let mut positions = Vec::new();
    for node in dom.descendants(root, None) {
        if dom.name_is(node, &W::t()) {
            text.push_str(&dom.value(node));
        } else if dom.name_is(node, &W::name("commentReference")) {
            let id = dom.attribute(node, &W::name("id")).unwrap();
            positions.push((notes[id].as_str(), text.clone()));
        }
    }
    assert_eq!(text, "ação 🐋東京new tail");
    assert_eq!(
        positions,
        [
            ("Note", "ação 🐋".to_string()),
            ("Second note", "ação 🐋東京".to_string())
        ]
    );
}
