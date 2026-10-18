// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Accept All as Microsoft Word does it. Each rule was read off Word's own
//! Accept All of Word's redlines (`_to_improve_accepted_changes`, 2026-09-28)
//! and is pinned here with a synthetic package.

mod common;

use common::docx::{Part, docx, docx_with, docx_with_sect, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const REV: &str = r#"w:author="a" w:date="2026-01-01T00:00:00Z""#;

/// Body paragraphs of `document.xml` (tables included, in order), each as
/// its visible text.
fn paragraphs(pkg: &[u8]) -> Vec<String> {
    let xml = common::docx::part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect::<String>()
        })
        .collect()
}

fn deleted_mark(id: u32) -> String {
    format!(r#"<w:pPr><w:rPr><w:del w:id="{id}" {REV}/></w:rPr></w:pPr>"#)
}

/// R2: a run of deleted paragraph marks joins the paragraph after a table
/// whose rows are all deleted; the table does not end the run. Word's accept
/// of 5 redlines (bc0135eaa1, d8b0c2ae01, a52462ea98, ece42865e7, 25f1d6ecc3)
/// keeps no empty paragraph after the vanished table.
#[test]
fn a_deleted_mark_run_joins_across_a_wholly_deleted_table() {
    let body = format!(
        r#"<w:p><w:r><w:t>Keep</w:t></w:r></w:p>
<w:p>{m1}<w:ins w:id="2" {REV}><w:r><w:t>Fresh</w:t></w:r></w:ins></w:p>
<w:tbl><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid>
<w:tr><w:trPr><w:del w:id="3" {REV}/></w:trPr><w:tc><w:p>{m4}<w:del w:id="5" {REV}><w:r><w:delText>Old cell</w:delText></w:r></w:del></w:p></w:tc></w:tr>
</w:tbl>
<w:p>{m6}</w:p>
<w:p/>"#,
        m1 = deleted_mark(1),
        m4 = deleted_mark(4),
        m6 = deleted_mark(6),
    );
    let accepted = accept_revisions(&docx(&body)).unwrap();
    assert_eq!(paragraphs(&accepted), ["Keep", "Fresh"]);
}

/// R2's boundary: a table that keeps a row still ends the run, as before.
#[test]
fn a_surviving_table_still_ends_a_deleted_mark_run() {
    let body = format!(
        r#"<w:p><w:r><w:t>Keep</w:t></w:r></w:p>
<w:p>{m1}<w:ins w:id="2" {REV}><w:r><w:t>Fresh</w:t></w:r></w:ins></w:p>
<w:tbl><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid>
<w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl>
<w:p/>"#,
        m1 = deleted_mark(1),
    );
    let accepted = accept_revisions(&docx(&body)).unwrap();
    assert_eq!(paragraphs(&accepted), ["Keep", "Fresh", "Cell", ""]);
}

fn moved_mark(id: u32) -> String {
    format!(r#"<w:pPr><w:rPr><w:moveFrom w:id="{id}" {REV}/></w:rPr></w:pPr>"#)
}

/// A run of deleted marks goes on through moved-from marks: Word's accept of
/// 3866f441cc joins every paragraph from the first deleted mark through the
/// inserted "Subject" paragraph, leaving no blank paragraph where the
/// moved-from ones were.
#[test]
fn moved_from_marks_join_a_deleted_mark_run() {
    let body = format!(
        r#"<w:p><w:r><w:t>Section</w:t></w:r></w:p>
<w:p>{m1}<w:del w:id="2" {REV}><w:r><w:delText>Gone</w:delText></w:r></w:del></w:p>
<w:p>{mv3}<w:moveFromRangeStart w:id="4" {REV} w:name="move1"/><w:moveFrom w:id="5" {REV}><w:r><w:t xml:space="preserve"> </w:t></w:r></w:moveFrom></w:p>
<w:p>{m6}<w:moveFrom w:id="7" {REV}><w:r><w:t xml:space="preserve">The </w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="4"/><w:del w:id="8" {REV}><w:r><w:delText>old</w:delText></w:r></w:del></w:p>
<w:p>{mv9}<w:moveFrom w:id="10" {REV}><w:r><w:t xml:space="preserve"> </w:t></w:r></w:moveFrom></w:p>
<w:p><w:pPr><w:rPr><w:ins w:id="11" {REV}/></w:rPr></w:pPr><w:ins w:id="12" {REV}><w:r><w:t>Subject</w:t></w:r></w:ins></w:p>
<w:p><w:moveToRangeStart w:id="13" {REV} w:name="move1"/><w:moveTo w:id="14" {REV}><w:r><w:t xml:space="preserve">The </w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="13"/><w:r><w:t>end</w:t></w:r></w:p>"#,
        m1 = deleted_mark(1),
        mv3 = moved_mark(3),
        m6 = deleted_mark(6),
        mv9 = moved_mark(9),
    );
    let accepted = accept_revisions(&docx(&body)).unwrap();
    assert_eq!(paragraphs(&accepted), ["Section", "Subject", "The end"]);
}

/// A table whose rows are all deleted vanishes after a moved-from mark too,
/// when no paragraph follows it. Emptying its cells and then dropping the
/// emptied rows (the moveFrom row pass) left a `w:tbl` with no row, which
/// Word's Compare then read as a table of its own: accepted,
/// docx_lots_of_comments_addition_removal_redline_removal_v_addition kept
/// two, and our redline marked 52 changes where Word found none.
#[test]
fn a_wholly_deleted_table_after_a_moved_from_mark_leaves_no_table() {
    let body = format!(
        r#"<w:p><w:r><w:t>Start</w:t></w:r></w:p>
<w:p>{mv1}<w:moveFrom w:id="2" {REV}><w:r><w:t>Heading</w:t></w:r></w:moveFrom></w:p>
<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid>
<w:tr><w:trPr><w:del w:id="3" {REV}/></w:trPr><w:tc><w:p>{m4}<w:del w:id="5" {REV}><w:r><w:delText>Old cell</w:delText></w:r></w:del></w:p></w:tc></w:tr>
</w:tbl>"#,
        mv1 = moved_mark(1),
        m4 = deleted_mark(4),
    );
    let accepted = accept_revisions(&docx(&body)).unwrap();
    let xml = part_string(&accepted, "word/document.xml").unwrap();
    assert!(!xml.contains("<w:tbl>"), "{xml}");
    assert_eq!(paragraphs(&accepted), ["Start"]);
    assert_word_valid_package(&accepted);
}

const W14_NS: &str = "http://schemas.microsoft.com/office/word/2010/wordml";
const W15_NS: &str = "http://schemas.microsoft.com/office/word/2012/wordml";
const CID_NS: &str = "http://schemas.microsoft.com/office/word/2016/wordml/cid";
const CEX_NS: &str = "http://schemas.microsoft.com/office/word/2018/wordml/cex";

/// A comment with id `id`, its one paragraph `paraId` = `para`.
fn comment(id: u32, author: &str, para: &str, text: &str) -> String {
    format!(
        r#"<w:comment w:id="{id}" w:author="{author}" w:initials="X"><w:p w14:paraId="{para}"><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#
    )
}

/// Body + the four comment parts + people.xml for comments `(id, author,
/// paraId, durableId)`.
fn commented_docx(body: &str, comments: &[(u32, &str, &str, &str)], people: &[&str]) -> Vec<u8> {
    let w = common::docx::W_NS;
    let comments_xml = format!(
        r#"<w:comments xmlns:w="{w}" xmlns:w14="{W14_NS}">{}</w:comments>"#,
        comments
            .iter()
            .map(|(id, a, para, _)| comment(*id, a, para, &format!("note {id}")))
            .collect::<String>()
    );
    let ext = format!(
        r#"<w15:commentsEx xmlns:w15="{W15_NS}">{}</w15:commentsEx>"#,
        comments
            .iter()
            .map(|(_, _, para, _)| format!(r#"<w15:commentEx w15:paraId="{para}" w15:done="0"/>"#))
            .collect::<String>()
    );
    let ids = format!(
        r#"<w16cid:commentsIds xmlns:w16cid="{CID_NS}">{}</w16cid:commentsIds>"#,
        comments
            .iter()
            .map(|(_, _, para, dur)| {
                format!(r#"<w16cid:commentId w16cid:paraId="{para}" w16cid:durableId="{dur}"/>"#)
            })
            .collect::<String>()
    );
    let cex = format!(
        r#"<w16cex:commentsExtensible xmlns:w16cex="{CEX_NS}">{}</w16cex:commentsExtensible>"#,
        comments
            .iter()
            .map(|(_, _, _, dur)| {
                format!(r#"<w16cex:commentExtensible w16cex:durableId="{dur}" w16cex:dateUtc="2026-01-01T00:00:00Z"/>"#)
            })
            .collect::<String>()
    );
    let people_xml = format!(
        r#"<w15:people xmlns:w15="{W15_NS}">{}</w15:people>"#,
        people
            .iter()
            .map(|a| format!(r#"<w15:person w15:author="{a}"><w15:presenceInfo w15:providerId="None" w15:userId="{a}"/></w15:person>"#))
            .collect::<String>()
    );
    let ct = "application/vnd.openxmlformats-officedocument.wordprocessingml.";
    docx_with(
        body,
        &[
            Part {
                name: "word/comments.xml",
                content_type: &format!("{ct}comments+xml"),
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: &comments_xml,
            },
            Part {
                name: "word/commentsExtended.xml",
                content_type: &format!("{ct}commentsExtended+xml"),
                rel_type: "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
                xml: &ext,
            },
            Part {
                name: "word/commentsIds.xml",
                content_type: &format!("{ct}commentsIds+xml"),
                rel_type: "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds",
                xml: &ids,
            },
            Part {
                name: "word/commentsExtensible.xml",
                content_type: &format!("{ct}commentsExtensible+xml"),
                rel_type: "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible",
                xml: &cex,
            },
            Part {
                name: "word/people.xml",
                content_type: &format!("{ct}people+xml"),
                rel_type: "http://schemas.microsoft.com/office/2011/relationships/people",
                xml: &people_xml,
            },
        ],
    )
}

/// A commented run: range start, text, range end, reference.
fn commented(id: u32, text: &str) -> String {
    format!(
        r#"<w:commentRangeStart w:id="{id}"/><w:r><w:t>{text}</w:t></w:r><w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r>"#
    )
}

/// The same inside a deletion.
fn commented_deleted(id: u32, text: &str) -> String {
    format!(
        r#"<w:commentRangeStart w:id="{id}"/><w:del w:id="9{id}" {REV}><w:r><w:delText>{text}</w:delText></w:r></w:del><w:commentRangeEnd w:id="{id}"/><w:del w:id="8{id}" {REV}><w:r><w:commentReference w:id="{id}"/></w:r></w:del>"#
    )
}

fn count(xml: &str, needle: &str) -> usize {
    xml.matches(needle).count()
}

/// R1: a comment whose reference is deleted goes with it — its anchors, its
/// comments.xml entry and its commentsExtended/Ids/Extensible entries
/// (29e3872eed, 145b9e67e2, 92075b7449, 01f3deda92). The comments left are
/// numbered 0, 1, … in document order (R5).
#[test]
fn accept_drops_comments_in_deleted_text_and_renumbers_the_rest() {
    let body = format!(
        "<w:p>{}<w:r><w:t> and </w:t></w:r>{}</w:p><w:p>{}</w:p>",
        commented(7, "kept"),
        commented_deleted(3, "gone"),
        commented(5, "also kept"),
    );
    let pkg = commented_docx(
        &body,
        &[
            (3, "Ann", "00000003", "10000003"),
            (5, "Bo", "00000005", "10000005"),
            (7, "Cy", "00000007", "10000007"),
        ],
        &["Ann", "Bo"],
    );
    let accepted = accept_revisions(&pkg).unwrap();
    assert_word_valid_package(&accepted);
    let doc = part_string(&accepted, "word/document.xml").unwrap();
    for (tag, n) in [
        ("commentRangeStart", 2),
        ("commentRangeEnd", 2),
        ("commentReference", 2),
    ] {
        assert_eq!(count(&doc, &format!("<w:{tag} ")), n, "{tag} in {doc}");
    }
    // Document order is 7 then 5: old 7 → 0, old 5 → 1.
    let first = doc
        .find(r#"w:commentReference w:id="0""#)
        .expect("7 renumbered to 0");
    let second = doc
        .find(r#"w:commentReference w:id="1""#)
        .expect("5 renumbered to 1");
    assert!(first < second, "{doc}");
    let comments = part_string(&accepted, "word/comments.xml").unwrap();
    assert!(comments.contains(r#"w:id="0" w:author="Cy""#), "{comments}");
    let comments = part_string(&accepted, "word/comments.xml").unwrap();
    assert_eq!(count(&comments, "<w:comment "), 2, "{comments}");
    assert!(
        comments.contains("note 5") && comments.contains("note 7") && !comments.contains("note 3")
    );
    for (part, gone) in [
        ("word/commentsExtended.xml", "00000003"),
        ("word/commentsIds.xml", "00000003"),
        ("word/commentsExtensible.xml", "10000003"),
    ] {
        let xml = part_string(&accepted, part).unwrap();
        assert!(!xml.contains(gone), "{part}: {xml}");
        assert!(
            xml.contains("00000005") || xml.contains("10000005"),
            "{part}: {xml}"
        );
    }
    // Ann authored only the dropped comment; Bo's stays.
    let people = part_string(&accepted, "word/people.xml").unwrap();
    assert!(!people.contains("Ann") && people.contains("Bo"), "{people}");
}

/// R1: when no comment is left, Word writes no comment parts and no
/// people.xml (their relationships and content types go too).
#[test]
fn accept_drops_the_comment_parts_when_no_comment_is_left() {
    let body = format!(
        "<w:p><w:r><w:t>text </w:t></w:r>{}</w:p>",
        commented_deleted(0, "gone")
    );
    let pkg = commented_docx(&body, &[(0, "Ann", "00000001", "10000001")], &["Ann"]);
    let accepted = accept_revisions(&pkg).unwrap();
    assert_word_valid_package(&accepted);
    for part in [
        "word/comments.xml",
        "word/commentsExtended.xml",
        "word/commentsIds.xml",
        "word/commentsExtensible.xml",
        "word/people.xml",
    ] {
        assert!(part_string(&accepted, part).is_none(), "{part} left behind");
    }
    let rels = part_string(&accepted, "word/_rels/document.xml.rels").unwrap();
    assert!(
        !rels.contains("comments") && !rels.contains("people"),
        "{rels}"
    );
    let types = part_string(&accepted, "[Content_Types].xml").unwrap();
    assert!(
        !types.contains("comments") && !types.contains("people"),
        "{types}"
    );
    let doc = part_string(&accepted, "word/document.xml").unwrap();
    assert!(!doc.contains("comment"), "{doc}");
}

/// Reject All mirrors R1: a comment whose reference sits in rejected
/// (inserted) text goes, and the rest are renumbered.
#[test]
fn reject_drops_comments_in_inserted_text() {
    let inserted = r#"<w:commentRangeStart w:id="4"/><w:ins w:id="94" w:author="a" w:date="2026-01-01T00:00:00Z"><w:r><w:t>new</w:t></w:r><w:r><w:commentReference w:id="4"/></w:r></w:ins><w:commentRangeEnd w:id="4"/>"#;
    let body = format!("<w:p>{}{inserted}</w:p>", commented(9, "old"));
    let pkg = commented_docx(
        &body,
        &[
            (4, "Ann", "00000004", "10000004"),
            (9, "Bo", "00000009", "10000009"),
        ],
        &["Ann", "Bo"],
    );
    let rejected = reject_revisions(&pkg).unwrap();
    assert_word_valid_package(&rejected);
    let doc = part_string(&rejected, "word/document.xml").unwrap();
    assert_eq!(count(&doc, "<w:commentReference "), 1, "{doc}");
    assert!(
        doc.contains(r#"w:commentReference w:id="0""#),
        "9 renumbered to 0: {doc}"
    );
    let comments = part_string(&rejected, "word/comments.xml").unwrap();
    assert!(
        comments.contains("note 9") && !comments.contains("note 4"),
        "{comments}"
    );
    let people = part_string(&rejected, "word/people.xml").unwrap();
    assert!(!people.contains("Ann") && people.contains("Bo"), "{people}");
}

/// The `w:name` of every bookmark start in `document.xml`, with its id.
fn bookmarks(pkg: &[u8]) -> Vec<(String, String)> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let ends: Vec<String> = dom
        .descendants(root, Some(&W::name("bookmarkEnd")))
        .into_iter()
        .filter_map(|e| dom.attribute(e, &W::id()).map(str::to_string))
        .collect();
    dom.descendants(root, Some(&W::name("bookmarkStart")))
        .into_iter()
        .map(|b| {
            let id = dom.attribute(b, &W::id()).unwrap_or("").to_string();
            assert!(ends.contains(&id), "bookmark {id} has no end: {xml}");
            (
                dom.attribute(b, &W::name("name")).unwrap_or("").to_string(),
                id,
            )
        })
        .collect()
}

/// R4: an empty bookmark inside deleted text survives Accept All, where the
/// deletion was (ece42865e7: two zero-length bookmarks in a deleted run).
#[test]
fn an_empty_bookmark_inside_deleted_text_survives_accept() {
    let body = format!(
        r#"<w:p><w:r><w:t>Before </w:t></w:r><w:del w:id="1" {REV}><w:r><w:delText>old </w:delText></w:r><w:bookmarkStart w:id="40" w:name="Anchor"/><w:bookmarkEnd w:id="40"/><w:r><w:delText>text</w:delText></w:r></w:del><w:r><w:t>after</w:t></w:r></w:p>"#
    );
    let accepted = accept_revisions(&docx(&body)).unwrap();
    assert_word_valid_package(&accepted);
    assert_eq!(
        bookmarks(&accepted),
        [("Anchor".to_string(), "0".to_string())]
    );
    let doc = part_string(&accepted, "word/document.xml").unwrap();
    let (b, a) = (
        doc.find("bookmarkStart").unwrap(),
        doc.find("after").unwrap(),
    );
    assert!(doc.find("Before").unwrap() < b && b < a, "{doc}");
}

/// R4: a bookmark whose whole span was deleted goes with it, both markers
/// (221577c35b `_Toc0`: start before the deletion, end inside it; 6fb9bbdb49
/// `bmTitle`: deleted text and deleted paragraph marks).
#[test]
fn a_bookmark_over_wholly_deleted_text_is_dropped() {
    let body = format!(
        r#"<w:p><w:r><w:t>Keep</w:t></w:r></w:p>
<w:p>{m1}<w:bookmarkStart w:id="71" w:name="_Toc0"/><w:del w:id="2" {REV}><w:r><w:delText>Heading gone</w:delText></w:r><w:bookmarkEnd w:id="71"/></w:del></w:p>
<w:p>{m3}<w:bookmarkStart w:id="72" w:name="Span"/><w:del w:id="4" {REV}><w:r><w:delText>one</w:delText></w:r></w:del></w:p>
<w:p>{m5}<w:del w:id="6" {REV}><w:r><w:delText>two</w:delText></w:r></w:del><w:bookmarkEnd w:id="72"/></w:p>
<w:p><w:r><w:t>Tail</w:t></w:r></w:p>"#,
        m1 = deleted_mark(1),
        m3 = deleted_mark(3),
        m5 = deleted_mark(5),
    );
    let accepted = accept_revisions(&docx(&body)).unwrap();
    assert_word_valid_package(&accepted);
    assert_eq!(bookmarks(&accepted), []);
    let doc = part_string(&accepted, "word/document.xml").unwrap();
    assert!(!doc.contains("bookmark"), "{doc}");
    assert_eq!(paragraphs(&accepted), ["Keep", "Tail"]);
}

/// R4's boundary: a bookmark that keeps any character keeps both markers.
#[test]
fn a_partly_deleted_bookmark_keeps_both_markers() {
    let body = format!(
        r#"<w:p><w:bookmarkStart w:id="9" w:name="Part"/><w:r><w:t>kept </w:t></w:r><w:del w:id="1" {REV}><w:r><w:delText>gone</w:delText></w:r><w:bookmarkEnd w:id="9"/></w:del></w:p>"#
    );
    let accepted = accept_revisions(&docx(&body)).unwrap();
    assert_word_valid_package(&accepted);
    assert_eq!(
        bookmarks(&accepted),
        [("Part".to_string(), "0".to_string())]
    );
}

/// R5: after Accept All, bookmarks and comments share one id counter, dense
/// and in document order (2288f27be1: comments 0 1, bookmarks 2–6, comments
/// 7 8; d6b1d609c1 interleaves them).
#[test]
fn accept_numbers_bookmarks_and_comments_in_one_document_order_counter() {
    let body = format!(
        r#"<w:p>{}<w:bookmarkStart w:id="84" w:name="First"/><w:r><w:t>x</w:t></w:r><w:bookmarkEnd w:id="84"/><w:ins w:id="1" {REV}><w:r><w:t>new</w:t></w:r></w:ins></w:p><w:p><w:bookmarkStart w:id="12" w:name="Second"/><w:bookmarkEnd w:id="12"/>{}</w:p>"#,
        commented(38, "a"),
        commented(2, "b"),
    );
    let pkg = commented_docx(
        &body,
        &[
            (2, "Bo", "00000002", "10000002"),
            (38, "Cy", "00000038", "10000038"),
        ],
        &["Bo", "Cy"],
    );
    let accepted = accept_revisions(&pkg).unwrap();
    assert_word_valid_package(&accepted);
    assert_eq!(
        bookmarks(&accepted),
        [
            ("First".to_string(), "1".to_string()),
            ("Second".to_string(), "2".to_string())
        ]
    );
    let doc = part_string(&accepted, "word/document.xml").unwrap();
    for (old, new) in [("a", "0"), ("b", "3")] {
        let at = doc.find(&format!("<w:t>{old}</w:t>")).unwrap();
        let start = doc[..at].rfind("commentRangeStart").unwrap();
        assert!(
            doc[start..at].contains(&format!(r#"w:id="{new}""#)),
            "{old} → {new}: {doc}"
        );
    }
    let comments = part_string(&accepted, "word/comments.xml").unwrap();
    assert!(
        comments.contains(r#"w:id="0" w:author="Cy""#)
            && comments.contains(r#"w:id="3" w:author="Bo""#),
        "{comments}"
    );
}

/// Reject All mirrors R4: a bookmark over wholly inserted text goes; an
/// empty one inside it stays.
#[test]
fn reject_drops_a_bookmark_over_wholly_inserted_text() {
    let body = format!(
        r#"<w:p><w:r><w:t>Old</w:t></w:r><w:bookmarkStart w:id="5" w:name="New"/><w:ins w:id="1" {REV}><w:r><w:t> added</w:t></w:r><w:bookmarkStart w:id="6" w:name="Empty"/><w:bookmarkEnd w:id="6"/></w:ins><w:bookmarkEnd w:id="5"/></w:p>"#
    );
    let rejected = reject_revisions(&docx(&body)).unwrap();
    assert_word_valid_package(&rejected);
    assert_eq!(
        bookmarks(&rejected),
        [("Empty".to_string(), "0".to_string())]
    );
}

/// R3: Word reads a `w:pStyle` that names a character style as no style at
/// all, so its Accept All writes the paragraph without it (Word's own
/// redlines carry `HeaderChar`, `FooterChar`, `HTMLPreformattedChar` there:
/// 6fb9bbdb49, bc0135eaa1, 73105518ef, c8db65e1fd, ece42865e7, ac6cd8d92f).
#[test]
fn accept_drops_a_paragraph_style_that_names_a_character_style() {
    let w = common::docx::W_NS;
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Header"><w:name w:val="header"/></w:style><w:style w:type="character" w:customStyle="1" w:styleId="HeaderChar"><w:name w:val="Header Char"/></w:style></w:styles>"#
    );
    let body = format!(
        r#"<w:p><w:pPr><w:pStyle w:val="HeaderChar"/><w:jc w:val="center"/></w:pPr><w:ins w:id="1" {REV}><w:r><w:t>new</w:t></w:r></w:ins></w:p><w:p><w:pPr><w:pStyle w:val="Header"/></w:pPr><w:r><w:t>kept</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="HeaderChar"/></w:pPr></w:p>"#
    );
    let pkg = docx_with(
        &body,
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: &styles,
        }],
    );
    for out in [
        accept_revisions(&pkg).unwrap(),
        reject_revisions(&pkg).unwrap(),
    ] {
        assert_word_valid_package(&out);
        let doc = part_string(&out, "word/document.xml").unwrap();
        assert!(!doc.contains("HeaderChar"), "{doc}");
        assert!(doc.contains(r#"<w:pStyle w:val="Header""#), "{doc}");
        assert!(doc.contains(r#"<w:jc w:val="center""#), "{doc}");
    }
}

/// Word writes no empty `w:rPr` / `w:pPr`: accepting a paragraph mark's
/// `w:ins` leaves none behind (ff27140d0a, 485b916ef9, d8b0c2ae01).
#[test]
fn accept_leaves_no_empty_property_elements() {
    let body = format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="1" {REV}/></w:rPr></w:pPr><w:ins w:id="2" {REV}><w:r><w:rPr><w:rPrChange w:id="3" {REV}><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr><w:t>new</w:t></w:r></w:ins></w:p><w:p><w:r><w:t>old</w:t></w:r></w:p>"#
    );
    let accepted = accept_revisions(&docx(&body)).unwrap();
    assert_word_valid_package(&accepted);
    let doc = part_string(&accepted, "word/document.xml").unwrap();
    for empty in [
        "<w:rPr/>",
        "<w:rPr />",
        "<w:rPr></w:rPr>",
        "<w:pPr/>",
        "<w:pPr />",
        "<w:pPr></w:pPr>",
    ] {
        assert!(!doc.contains(empty), "{empty} in {doc}");
    }
    assert_eq!(paragraphs(&accepted), ["new", "old"]);
}

/// R9: a section without a header reference shows its predecessor's ("link
/// to previous"). Accepting a deleted section break removes the predecessor,
/// so Word writes the header the last section showed into it
/// (`_to_improve_accepted_changes` 29e3872eed, 4eff11f045) and keeps the part.
#[test]
fn accept_of_a_deleted_section_break_carries_its_header_on() {
    let body = format!(
        r#"<w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/></w:sectPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>One</w:t></w:r></w:p><w:p><w:r><w:t>Two</w:t></w:r></w:p>"#
    );
    let header = Part {
        name: "word/header1.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        xml: r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>Header</w:t></w:r></w:p></w:hdr>"#,
    };
    let accepted = accept_revisions(&docx_with_sect(&body, &[header], "")).unwrap();
    assert_word_valid_package(&accepted);
    assert_eq!(paragraphs(&accepted), ["OneTwo"]);
    let xml = part_string(&accepted, "word/document.xml").unwrap();
    assert_eq!(xml.matches("<w:sectPr>").count(), 1, "{xml}");
    assert!(
        xml.contains(r#"<w:headerReference w:type="default" r:id="rIdX0""#),
        "{xml}"
    );
    assert!(
        part_string(&accepted, "word/header1.xml").is_some(),
        "header lost"
    );
}

#[path = "common/dead_comments.rs"]
mod dead_comments;

#[path = "common/comment_revisions.rs"]
mod comment_revisions;

/// #1141(a): accepting a surrounding insertion must not keep a comment
/// whose entire range disappears in a nested deletion.
#[test]
fn accept_dead_comment_probe_a_nested_deletion() {
    let source = dead_comments::probe('a');
    let accepted = accept_revisions(&source).unwrap();
    dead_comments::assert_state(&accepted, false, &[]);
    assert_eq!(paragraphs(&accepted), ["alpha NEW gamma"]);
    let rejected = reject_revisions(&source).unwrap();
    dead_comments::assert_state(&rejected, false, &[]);
    assert_eq!(paragraphs(&rejected), [""]);
}

#[test]
fn accept_dead_comment_probe_b_deleted_reference() {
    let source = dead_comments::probe('b');
    let accepted = accept_revisions(&source).unwrap();
    dead_comments::assert_state(&accepted, false, &[]);
    assert_eq!(paragraphs(&accepted), [""]);
    let rejected = reject_revisions(&source).unwrap();
    dead_comments::assert_state(&rejected, false, &[(0, "Existing range", None)]);
    assert_eq!(paragraphs(&rejected), ["beta"]);
}

#[test]
fn accept_dead_comment_probe_c_kept_character() {
    let source = dead_comments::probe('c');
    let accepted = accept_revisions(&source).unwrap();
    dead_comments::assert_state(&accepted, false, &[(0, "Existing range", None)]);
    assert_eq!(paragraphs(&accepted), ["x"]);
    let rejected = reject_revisions(&source).unwrap();
    dead_comments::assert_state(&rejected, false, &[(0, "Existing range", None)]);
    assert_eq!(paragraphs(&rejected), ["betax"]);
}

#[test]
fn accept_dead_comment_probe_e_removes_thread_metadata() {
    let source = dead_comments::probe('e');
    let accepted = accept_revisions(&source).unwrap();
    dead_comments::assert_state(&accepted, true, &[(0, "Kept range", None)]);
    assert_eq!(paragraphs(&accepted), ["", "kept"]);
}

#[test]
fn accept_dead_comment_probe_f_deleted_paragraph_mark() {
    let source = dead_comments::probe('f');
    let accepted = accept_revisions(&source).unwrap();
    dead_comments::assert_state(&accepted, false, &[]);
    assert_eq!(paragraphs(&accepted), [""]);
    let rejected = reject_revisions(&source).unwrap();
    dead_comments::assert_state(&rejected, false, &[(0, "Existing range", None)]);
    assert_eq!(paragraphs(&rejected), ["beta", "gamma"]);
}

/// Existing point comments have no characters to lose. Real inline content
/// and an undeleted paragraph mark keep a nonempty range alive.
#[test]
fn accept_dead_comment_controls_keep_empty_and_nontext_ranges() {
    for atom in [
        "",
        "<w:r><w:tab/></w:r>",
        "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r>",
        "</w:p><w:p>",
    ] {
        let deleted = if atom.is_empty() {
            String::new()
        } else {
            dead_comments::deleted("gone", 7)
        };
        let body = format!(
            "<w:p>{}{deleted}{atom}{}{}</w:p>",
            dead_comments::start(0),
            dead_comments::end(0),
            dead_comments::reference(0)
        );
        let source = dead_comments::package(&body, false);
        let accepted = accept_revisions(&source).unwrap();
        dead_comments::assert_state(&accepted, false, &[(0, "Existing range", None)]);
    }
}

/// #1155: the issue's two paragraphs, optionally with a comment spanning A to B.
fn inserted_final_mark_source(with_comment: bool) -> Vec<u8> {
    let (start, end) = if with_comment {
        (
            dead_comments::start(0),
            format!("{}{}", dead_comments::end(0), dead_comments::reference(0)),
        )
    } else {
        (String::new(), String::new())
    };
    let body = format!(
        r#"<w:p>{start}<w:r><w:t>A</w:t></w:r></w:p>
<w:p><w:pPr><w:rPr><w:ins w:id="9" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>B</w:t></w:r>{end}</w:p>"#
    );
    if with_comment {
        dead_comments::package(&body, false)
    } else {
        docx(&body)
    }
}

fn assert_inserted_final_mark_resolved(settled: &[u8], with_comment: bool) {
    assert_eq!(paragraphs(settled), ["A", "B"]);
    let xml = part_string(settled, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    assert!(
        !jubarte::revision_processor::element_has_tracked_revisions(&dom, root),
        "#1155: final paragraph mark must be resolved"
    );
    if with_comment {
        dead_comments::assert_state(settled, false, &[(0, "Existing range", None)]);
    } else {
        assert_word_valid_package(settled);
    }
}

fn accept_inserted_final_mark_with_cli(with_comment: bool) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("input.docx"),
        inserted_final_mark_source(with_comment),
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args(["accept", "input.docx", "-o", "settled.docx"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let settled = std::fs::read(dir.path().join("settled.docx")).unwrap();
    assert_inserted_final_mark_resolved(&settled, with_comment);
}

/// A16 (#1155): differs from Word on purpose; resolve the final inserted mark.
#[test]
fn accept_resolves_inserted_final_mark() {
    let settled = accept_revisions(&inserted_final_mark_source(false)).unwrap();
    assert_inserted_final_mark_resolved(&settled, false);
}

/// A16 (#1155): differs from Word on purpose, also with a comment ending on B.
#[test]
fn accept_resolves_inserted_final_mark_with_comment() {
    let settled = accept_revisions(&inserted_final_mark_source(true)).unwrap();
    assert_inserted_final_mark_resolved(&settled, true);
}

/// A16 (#1155): CLI differs from Word on purpose; leave no final-mark revision.
#[test]
fn cli_accept_resolves_inserted_final_mark() {
    accept_inserted_final_mark_with_cli(false);
}

/// A16 (#1155): CLI differs from Word on purpose, also with a comment ending on B.
#[test]
fn cli_accept_resolves_inserted_final_mark_with_comment() {
    accept_inserted_final_mark_with_cli(true);
}

/// #1223: Word's Accept All settles revisions inside comment bodies too
/// (its probe keeps `ZEBRAQUOKKA `, drops `OKAPIWOMBAT `, and leaves no
/// `w:ins`/`w:del` in `comments.xml`).
fn comment_body_probe() -> Vec<u8> {
    let body = format!(
        "<w:p>{}<w:r><w:t xml:space=\"preserve\"> tail</w:t></w:r><w:ins w:id=\"10\" w:author=\"Alice\" w:date=\"2026-10-18T00:00:00Z\"><w:r><w:t>new</w:t></w:r></w:ins></w:p>",
        comment_revisions::anchor(0, "anchor")
    );
    let comment = comment_revisions::comment(
        0,
        "Alice",
        r#"<w:p w14:paraId="00000001"><w:ins w:id="20" w:author="Bob" w:date="2026-10-18T00:00:00Z"><w:r><w:t xml:space="preserve">ZEBRAQUOKKA </w:t></w:r></w:ins><w:del w:id="21" w:author="Bob" w:date="2026-10-18T00:00:00Z"><w:r><w:delText xml:space="preserve">OKAPIWOMBAT </w:delText></w:r></w:del><w:r><w:annotationRef/></w:r><w:r><w:t xml:space="preserve">Just like that!</w:t></w:r></w:p>"#,
    );
    comment_revisions::docx(&body, &[comment])
}

/// The text of comment `id` in `word/comments.xml` (w:t runs only).
fn comment_text(pkg: &[u8], id: u32) -> String {
    let xml = part_string(pkg, "word/comments.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let id = id.to_string();
    let comment = dom
        .descendants(root, Some(&W::name("comment")))
        .into_iter()
        .find(|&c| dom.attribute(c, &W::id()) == Some(id.as_str()))
        .unwrap_or_else(|| panic!("no comment {id}: {xml}"));
    dom.descendants(comment, Some(&W::t()))
        .into_iter()
        .map(|t| dom.value(t))
        .collect()
}

/// #1223: Accept All resolves a comment body's insertion and deletion.
#[test]
fn accept_resolves_comment_body_revisions() {
    let accepted = accept_revisions(&comment_body_probe()).unwrap();
    assert_word_valid_package(&accepted);
    let xml = part_string(&accepted, "word/comments.xml").unwrap();
    for mark in ["<w:ins", "<w:del", "delText"] {
        assert!(!xml.contains(mark), "{mark} left in comments.xml: {xml}");
    }
    assert_eq!(comment_text(&accepted, 0), "ZEBRAQUOKKA Just like that!");
    // The body revision resolved too, and the comment kept its anchor.
    let doc = part_string(&accepted, "word/document.xml").unwrap();
    assert!(doc.contains(r#"<w:commentReference w:id="0""#), "{doc}");
}

/// #1223: a deleted paragraph mark merges inside its own comment, never
/// across `w:comment` boundaries; a comment whose whole content went keeps
/// one empty paragraph (a `w:comment` without one is not Word-valid).
#[test]
fn accept_comment_revisions_never_cross_the_comment_boundary() {
    let body = format!(
        "<w:p>{}{}</w:p>",
        comment_revisions::anchor(0, "first"),
        comment_revisions::anchor(1, "second")
    );
    let first = comment_revisions::comment(
        0,
        "Alice",
        r#"<w:p w14:paraId="00000001"><w:pPr><w:rPr><w:del w:id="30" w:author="Bob" w:date="2026-10-18T00:00:00Z"/></w:rPr></w:pPr><w:del w:id="31" w:author="Bob" w:date="2026-10-18T00:00:00Z"><w:r><w:delText>gone </w:delText></w:r></w:del></w:p><w:p w14:paraId="00000002"><w:r><w:t>rest</w:t></w:r></w:p>"#,
    );
    let second = comment_revisions::comment(
        1,
        "Alice",
        r#"<w:p w14:paraId="00000003"><w:r><w:t>neighbour</w:t></w:r></w:p>"#,
    );
    let source = comment_revisions::docx(&body, &[first, second]);
    let accepted = accept_revisions(&source).unwrap();
    assert_word_valid_package(&accepted);
    // The deletion merged the comment's second paragraph into its first.
    assert_eq!(comment_text(&accepted, 0), "rest");
    assert_eq!(comment_text(&accepted, 1), "neighbour");
    let xml = part_string(&accepted, "word/comments.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let comments = dom.descendants(root, Some(&W::name("comment")));
    assert_eq!(comments.len(), 2, "{xml}");
    // Comment 0's merge dropped the emptied first paragraph: one `w:p`
    // holds "rest"; the neighbouring comment keeps its own.
    assert_eq!(
        dom.descendants(comments[0], Some(&W::p())).len(),
        1,
        "{xml}"
    );
    assert_eq!(
        dom.descendants(comments[1], Some(&W::p())).len(),
        1,
        "{xml}"
    );
}

/// #1223: accepting a deletion that empties a comment's only paragraph
/// leaves a valid `w:comment` — Word writes an empty `w:p` there.
#[test]
fn accept_an_emptied_comment_keeps_a_paragraph() {
    let body = format!("<w:p>{}</w:p>", comment_revisions::anchor(0, "anchor"));
    let comment = comment_revisions::comment(
        0,
        "Alice",
        r#"<w:p w14:paraId="00000001"><w:del w:id="40" w:author="Bob" w:date="2026-10-18T00:00:00Z"><w:r><w:delText>whole</w:delText></w:r></w:del></w:p>"#,
    );
    let source = comment_revisions::docx(&body, &[comment]);
    let accepted = accept_revisions(&source).unwrap();
    assert_word_valid_package(&accepted);
    let xml = part_string(&accepted, "word/comments.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let comment = dom
        .descendants(root, Some(&W::name("comment")))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no comment: {xml}"));
    assert_eq!(
        dom.descendants(comment, Some(&W::p())).len(),
        1,
        "emptied comment keeps one paragraph: {xml}"
    );
    assert_eq!(comment_text(&accepted, 0), "");
}

// =========================================================================
// #1224: inserted final paragraph mark across stories and shapes.
// =========================================================================

/// Paragraphs of `part` in `pkg`, each as its visible text.
fn part_paragraphs(pkg: &[u8], part: &str) -> Vec<String> {
    let xml = part_string(pkg, part).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect::<String>()
        })
        .collect()
}

/// Resolves the inserted final mark in `part`: paragraphs read `expected`,
/// no tracked revision left, package is Word-valid.
fn assert_part_resolved(pkg: &[u8], part: &str, expected: &[&str]) {
    assert_eq!(part_paragraphs(pkg, part), expected);
    let xml = part_string(pkg, part).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    assert!(
        !jubarte::revision_processor::element_has_tracked_revisions(&dom, root),
        "{part} still has tracked revisions"
    );
    assert_word_valid_package(pkg);
}

fn header_part(xml: &str) -> Part<'_> {
    Part {
        name: "word/header1.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        xml,
    }
}

fn footer_part(xml: &str) -> Part<'_> {
    Part {
        name: "word/footer1.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
        xml,
    }
}

fn footnotes_part(xml: &str) -> Part<'_> {
    Part {
        name: "word/footnotes.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
        xml,
    }
}

/// A header with two paragraphs (`XA` plain, `XB` with an inserted mark).
fn inserted_final_mark_header(ins_id: u32) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>XA</w:t></w:r></w:p><w:p><w:pPr><w:rPr><w:ins w:id="{ins_id}" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>XB</w:t></w:r></w:p></w:hdr>"#
    )
}

/// A footer with two paragraphs (`XA` plain, `XB` with an inserted mark).
fn inserted_final_mark_footer(ins_id: u32) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>XA</w:t></w:r></w:p><w:p><w:pPr><w:rPr><w:ins w:id="{ins_id}" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>XB</w:t></w:r></w:p></w:ftr>"#
    )
}

/// A footnotes part with two separators and a footnote whose final
/// paragraph carries an inserted mark.
fn inserted_final_mark_footnotes(ins_id: u32) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p><w:r><w:t>XA</w:t></w:r></w:p><w:p><w:pPr><w:rPr><w:ins w:id="{ins_id}" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>XB</w:t></w:r></w:p></w:footnote></w:footnotes>"#
    )
}

/// #1224 (A16): an inserted final mark inside a header. Word resolves
/// the mark, as jubarte does. Accept All keeps `XA | XB`.
#[test]
fn accept_resolves_inserted_final_mark_in_header() {
    let body = r#"<w:p><w:r><w:t>Body</w:t></w:r></w:p>"#;
    let pkg = docx_with_sect(
        body,
        &[header_part(&inserted_final_mark_header(11))],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let settled = accept_revisions(&pkg).unwrap();
    assert_part_resolved(&settled, "word/header1.xml", &["XA", "XB"]);
    assert_eq!(paragraphs(&settled), ["Body"]);
}

/// #1224 (A16): an inserted final mark inside a footer. Word resolves
/// the mark, as jubarte does. Accept All keeps `XA | XB`.
#[test]
fn accept_resolves_inserted_final_mark_in_footer() {
    let body = r#"<w:p><w:r><w:t>Body</w:t></w:r></w:p>"#;
    let pkg = docx_with_sect(
        body,
        &[footer_part(&inserted_final_mark_footer(12))],
        r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let settled = accept_revisions(&pkg).unwrap();
    assert_part_resolved(&settled, "word/footer1.xml", &["XA", "XB"]);
    assert_eq!(paragraphs(&settled), ["Body"]);
}

/// #1224 (A16): an inserted final mark inside a footnote. Word resolves
/// the mark, as jubarte does. Accept All keeps `XA | XB`.
#[test]
fn accept_resolves_inserted_final_mark_in_footnote() {
    let body = r#"<w:p><w:r><w:t>Body</w:t></w:r><w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:footnoteReference w:id="1"/></w:r></w:p>"#;
    let pkg = docx_with(body, &[footnotes_part(&inserted_final_mark_footnotes(13))]);
    let settled = accept_revisions(&pkg).unwrap();
    assert_part_resolved(&settled, "word/footnotes.xml", &["", "", "XA", "XB"]);
    assert_eq!(paragraphs(&settled), ["Body"]);
}

/// #1224: a document whose ONLY paragraph carries the inserted mark.
/// The transform resolves the mark without needing a predecessor.
#[test]
fn accept_resolves_inserted_final_mark_when_only_paragraph() {
    let body = r#"<w:p><w:pPr><w:rPr><w:ins w:id="9" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>B</w:t></w:r></w:p>"#;
    let pkg = docx(body);
    let settled = accept_revisions(&pkg).unwrap();
    assert_part_resolved(&settled, "word/document.xml", &["B"]);
}

/// #1224: a section-break paragraph (its `w:pPr` holds a `w:sectPr`)
/// whose mark is inserted. Accept resolves the mark, keeps the section
/// break.
#[test]
fn accept_resolves_inserted_final_mark_on_section_break_paragraph() {
    let body = r#"<w:p><w:r><w:t>Keep</w:t></w:r></w:p>
<w:p><w:pPr><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr><w:rPr><w:ins w:id="9" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>B</w:t></w:r></w:p>"#;
    let pkg = docx(body);
    let settled = accept_revisions(&pkg).unwrap();
    assert_part_resolved(&settled, "word/document.xml", &["Keep", "B"]);
    let xml = part_string(&settled, "word/document.xml").unwrap();
    assert!(
        xml.contains(r#"<w:sectPr><w:pgSz w:w="12240""#),
        "section break lost: {xml}"
    );
}

/// #1224: a `w:moveTo` final mark (`<w:rPr><w:moveTo …/></w:rPr>` in the
/// final paragraph's `w:pPr`, with the matching move range in the body).
/// Accept uses `accept_move_from_move_to_transform`.
#[test]
fn accept_resolves_move_to_final_mark() {
    let body = r#"<w:p><w:r><w:t>Keep</w:t></w:r></w:p>
<w:moveToRangeStart w:id="2" w:name="m1"/>
<w:p><w:pPr><w:rPr><w:moveTo w:id="1" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>B</w:t></w:r></w:p>
<w:moveToRangeEnd w:id="2"/>"#;
    let pkg = docx(body);
    let settled = accept_revisions(&pkg).unwrap();
    assert_part_resolved(&settled, "word/document.xml", &["Keep", "B"]);
}

/// #1224: a `w:pPrChange` on the same final paragraph as the inserted
/// mark. Accept drops both: the property history and the inserted mark.
#[test]
fn accept_resolves_inserted_final_mark_alongside_p_pr_change() {
    let body = r#"<w:p><w:r><w:t>Keep</w:t></w:r></w:p>
<w:p><w:pPr><w:pPrChange w:id="3" w:author="Alice" w:date="2026-10-18T11:40:32Z"><w:pPr/></w:pPrChange><w:rPr><w:ins w:id="9" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>B</w:t></w:r></w:p>"#;
    let pkg = docx(body);
    let settled = accept_revisions(&pkg).unwrap();
    assert_part_resolved(&settled, "word/document.xml", &["Keep", "B"]);
}

/// #1224: an inserted final mark inside `w:tc` (last paragraph of a
/// cell). Accept resolves the mark.
#[test]
fn accept_resolves_inserted_final_mark_inside_tc() {
    let body = r#"<w:p><w:r><w:t>Keep</w:t></w:r></w:p>
<w:tbl><w:tblPr><w:tblW w:type="auto" w:w="0"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid>
<w:tr><w:tc><w:tcPr><w:tcW w:type="dxa" w:w="4000"/></w:tcPr><w:p><w:pPr><w:rPr><w:ins w:id="9" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let pkg = docx(body);
    let settled = accept_revisions(&pkg).unwrap();
    let xml = part_string(&settled, "word/document.xml").unwrap();
    assert_eq!(paragraphs(&settled), ["Keep", "B"]);
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    assert!(
        !jubarte::revision_processor::element_has_tracked_revisions(&dom, root),
        "{xml}"
    );
    assert_word_valid_package(&settled);
}

/// #1224: an inserted final mark inside `w:txbxContent`. Accept
/// resolves the mark.
#[test]
fn accept_resolves_inserted_final_mark_inside_txbx_content() {
    let body = r#"<w:p><w:r><w:t>Keep</w:t></w:r></w:p>
<w:p><w:r>
<w:drawing xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="1000000" cy="1000000"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:docPr id="1" name="Box"/><wp:cNvGraphicFramePr/>
<a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<wps:wsp><wps:cNvSpPr/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="1000000" cy="1000000"/></a:xfrm></wps:spPr><wps:txbx><w:txbxContent><w:p><w:pPr><w:rPr><w:ins w:id="9" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>B</w:t></w:r></w:p></w:txbxContent></wps:txbx></wps:wsp>
</a:graphicData></a:graphic></wp:inline></w:drawing>
</w:r></w:p>"#;
    let pkg = docx(body);
    let settled = accept_revisions(&pkg).unwrap();
    let xml = part_string(&settled, "word/document.xml").unwrap();
    assert!(
        paragraphs(&settled).contains(&"B".to_string()),
        "the text-box paragraph carries B: {xml}"
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    assert!(
        !jubarte::revision_processor::element_has_tracked_revisions(&dom, root),
        "{xml}"
    );
    assert!(!xml.contains("<w:ins"), "the inserted mark survived: {xml}");
    assert_word_valid_package(&settled);
}

/// #1224 finding 4: `A` plain, `B` inside `w:ins`, its mark inserted.
/// Accept keeps the inserted B; the marks resolve.
#[test]
fn accept_resolves_finding_4_inserted_content_and_mark() {
    let body = r#"<w:p><w:r><w:t>A</w:t></w:r></w:p>
<w:p><w:pPr><w:rPr><w:ins w:id="9" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:ins w:id="10" w:author="Alice" w:date="2026-10-18T11:40:32Z"><w:r><w:t>B</w:t></w:r></w:ins></w:p>"#;
    let pkg = docx(body);
    let settled = accept_revisions(&pkg).unwrap();
    assert_part_resolved(&settled, "word/document.xml", &["A", "B"]);
}

/// #1224 (finding 3): selective CLI case. Accept only `r7` (a body run
/// insertion); the inserted final mark `r9` stays tracked.
#[test]
fn cli_accept_selects_one_revision_and_leaves_inserted_final_mark() {
    let body = r#"<w:p><w:r><w:t>A</w:t></w:r><w:ins w:id="7" w:author="Alice" w:date="2026-10-18T11:40:32Z"><w:r><w:t>extra</w:t></w:r></w:ins></w:p>
<w:p><w:pPr><w:rPr><w:ins w:id="9" w:author="Alice" w:date="2026-10-18T11:40:32Z"/></w:rPr></w:pPr><w:r><w:t>B</w:t></w:r></w:p>"#;
    let pkg = docx(body);
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("input.docx"), &pkg).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args(["accept", "input.docx", "--id", "r7", "-o", "settled.docx"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let settled = std::fs::read(dir.path().join("settled.docx")).unwrap();
    assert_word_valid_package(&settled);
    assert_eq!(paragraphs(&settled), ["Aextra", "B"]);
    let xml = part_string(&settled, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    assert!(
        jubarte::revision_processor::element_has_tracked_revisions(&dom, root),
        "r9 should still be tracked: {xml}"
    );
    assert!(xml.contains(r#"<w:ins w:id="9""#), "r9 lost: {xml}");
    assert!(
        !xml.contains(r#"<w:ins w:id="7""#),
        "r7 was resolved: {xml}"
    );
}
