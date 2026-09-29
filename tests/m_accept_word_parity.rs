// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Accept All as Microsoft Word does it. Each rule was read off Word's own
//! Accept All of Word's redlines (`_to_improve_accepted_changes`, 2026-09-28)
//! and is pinned here with a synthetic package.

mod common;

use common::docx::{Part, docx, docx_with, part_string};
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
