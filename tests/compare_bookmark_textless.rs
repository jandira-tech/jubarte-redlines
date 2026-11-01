// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1880: a bookmark that starts in a paragraph with no text (a TOC
//! heading's tab, a paragraph holding only a picture) starts there in the
//! redline too. The comparer drops every bookmark and puts it back by text,
//! and a paragraph with no text has none of its own: the start went to the
//! next text, so a paragraph inserted after it took the start inside its
//! `w:ins` (`BOOKMARK_CROSSES_REVISION`), and a `PAGEREF` to it pointed at
//! another paragraph.
//!
//! #1710 is the same projection on the deleted side: B deletes the
//! paragraph that holds only a bookmark (or comment anchor). The start
//! went into the next paragraph's `w:del`, so Reject All gave an empty
//! paragraph where the bookmark was, and the bookmark glued to the next
//! paragraph's text. The `paragraph_summaries` tests below pin the
//! restore: Reject All gives the original back.

mod common;

use common::docx::{Part, docx, docx_with, para, part_string, replace_entry};
use common::validity::assert_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

/// A tab, and no text.
const TAB: &str = "<w:r><w:tab/></w:r>";

/// A VML rectangle, and no text.
const PICTURE: &str =
    r#"<w:r><w:pict><v:rect style="width:20pt;height:10pt" fillcolor="red"/></w:pict></w:r>"#;

fn modes() -> [(&'static str, WmlComparerSettings); 2] {
    [
        ("word", WmlComparerSettings::default()),
        ("powertools", WmlComparerSettings::powertools_faithful()),
    ]
}

/// `before`, the paragraph `content` with `_Toc1` starting in it, `after`,
/// then "Heading text" (where `_Toc1` ends) and "Body text.".
fn package(content: &str, before: &str, after: &str, bookmark: bool) -> Vec<u8> {
    let (start, end) = if bookmark {
        (
            r#"<w:bookmarkStart w:id="0" w:name="_Toc1"/>"#,
            r#"<w:bookmarkEnd w:id="0"/>"#,
        )
    } else {
        ("", "")
    };
    let built = docx(&format!(
        "{before}<w:p>{start}{content}</w:p>{after}{}{end}{}",
        para("Heading text"),
        para("Body text.")
    ));
    let document = part_string(&built, "word/document.xml").unwrap().replacen(
        "<w:document ",
        r#"<w:document xmlns:v="urn:schemas-microsoft-com:vml" "#,
        1,
    );
    replace_entry(&built, "word/document.xml", document.as_bytes())
}

/// The paragraph `_Toc1` starts in holds `content`'s marker (`w:tab` or
/// `w:pict`) and no text, and the start sits in no revision.
fn assert_starts_in(docx: &[u8], marker: &str, label: &str) {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let start = dom
        .descendants(root, Some(&W::name("bookmarkStart")))
        .into_iter()
        .find(|&s| dom.attribute(s, &W::name("name")) == Some("_Toc1"))
        .unwrap_or_else(|| panic!("{label}: no _Toc1: {xml}"));
    let p = dom.ancestors(start, Some(&W::p()))[0];
    let text: String = dom
        .descendants(p, Some(&W::t()))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    assert_eq!(text, "", "{label}: {}", dom.serialize_element(p));
    assert!(
        !dom.descendants(p, Some(&W::name(marker))).is_empty(),
        "{label}: {}",
        dom.serialize_element(p)
    );
    for revision in ["ins", "del", "moveFrom", "moveTo"] {
        assert!(
            dom.ancestors(start, Some(&W::name(revision))).is_empty(),
            "{label}: start in a w:{revision}"
        );
    }
}

/// Compare `original` with `revised` in both modes: the redline, Accept All
/// and Reject All each start `_Toc1` in the paragraph without text.
fn assert_kept(original: &[u8], revised: &[u8], marker: &str, case: &str) {
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(original, revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        assert!(
            part_string(&redline, "word/document.xml")
                .unwrap()
                .contains("<w:ins "),
            "{case}, {mode}: the edit is marked"
        );
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            assert_starts_in(&docx, marker, &format!("{case}, {mode}, {view}"));
        }
    }
}

/// The issue's pair: a paragraph inserted right after the one without
/// text took its bookmark's start.
#[test]
fn a_paragraph_inserted_after_keeps_the_start_out() {
    let original = package(TAB, "", "", true);
    let revised = package(TAB, "", &para("Probe paragraph."), true);
    assert_kept(&original, &revised, "tab", "inserted after");
}

/// A paragraph inserted before the one without text leaves the start
/// where it was, not on "Heading text".
#[test]
fn a_paragraph_inserted_before_leaves_the_start_in_place() {
    let original = package(TAB, "", "", true);
    let revised = package(TAB, &para("Probe paragraph."), "", true);
    assert_kept(&original, &revised, "tab", "inserted before");
}

/// An edit elsewhere leaves the start in its paragraph.
#[test]
fn an_edit_elsewhere_leaves_the_start_in_place() {
    let original = package(TAB, &para("Intro."), "", true);
    let revised = package(TAB, &para("Intro, changed."), "", true);
    assert_kept(&original, &revised, "tab", "edit elsewhere");
}

/// A paragraph whose only content is a picture keeps its bookmark.
#[test]
fn a_picture_paragraph_keeps_its_start() {
    let original = package(PICTURE, "", "", true);
    let revised = package(PICTURE, "", &para("Probe paragraph."), true);
    assert_kept(&original, &revised, "pict", "picture");
}

/// A bookmark only the original holds goes back on the original's side.
#[test]
fn an_original_only_bookmark_keeps_its_start() {
    let original = package(TAB, "", "", true);
    let revised = package(TAB, "", &para("Probe paragraph."), false);
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        assert_starts_in(&redline, "tab", &format!("original only, {mode}, redline"));
        assert_starts_in(
            &reject_revisions(&redline).unwrap(),
            "tab",
            &format!("original only, {mode}, reject"),
        );
    }
}

/// The text of the paragraph `name` starts in, and whether its end follows
/// the start in that paragraph.
fn start_paragraph(docx: &[u8], name: &str) -> (String, bool) {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let start = dom
        .descendants(root, Some(&W::name("bookmarkStart")))
        .into_iter()
        .find(|&s| dom.attribute(s, &W::name("name")) == Some(name))
        .unwrap_or_else(|| panic!("no {name}: {xml}"));
    let id = dom.attribute(start, &W::name("id")).unwrap().to_string();
    let p = dom.ancestors(start, Some(&W::p()))[0];
    let text = dom
        .descendants(p, Some(&W::t()))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    let order = dom.descendants(p, None);
    let at = |n| order.iter().position(|&o| o == n);
    let ends_after = dom
        .descendants(p, Some(&W::name("bookmarkEnd")))
        .into_iter()
        .any(|e| dom.attribute(e, &W::name("id")) == Some(id.as_str()) && at(e) > at(start));
    (text, ends_after)
}

/// A bookmark with no extent in the paragraph without text keeps its end
/// right after its start there.
#[test]
fn an_empty_bookmark_stays_whole_in_its_paragraph() {
    let build = |after: &str| {
        docx(&format!(
            r#"<w:p><w:bookmarkStart w:id="0" w:name="_Toc1"/><w:bookmarkEnd w:id="0"/>{TAB}</w:p>{after}{}{}"#,
            para("Heading text"),
            para("Body text.")
        ))
    };
    let (original, revised) = (build(""), build(&para("Probe paragraph.")));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            assert_eq!(
                start_paragraph(&docx, "_Toc1"),
                (String::new(), true),
                "{mode}, {view}"
            );
        }
    }
}

/// Bookmarks on text stay on their text: one leading a paragraph with
/// text, one starting inside a paragraph's text.
#[test]
fn a_bookmark_on_text_stays_on_its_text() {
    let build = |after: &str| {
        docx(&format!(
            r#"<w:p>{TAB}</w:p>{after}<w:p><w:bookmarkStart w:id="1" w:name="_Ref2"/><w:r><w:t>Heading text</w:t></w:r><w:bookmarkEnd w:id="1"/></w:p><w:p><w:r><w:t xml:space="preserve">Body </w:t></w:r><w:bookmarkStart w:id="2" w:name="_Ref3"/><w:r><w:t>text.</w:t></w:r><w:bookmarkEnd w:id="2"/></w:p>"#
        ))
    };
    let (original, revised) = (build(""), build(&para("Probe paragraph.")));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            assert_eq!(
                start_paragraph(&docx, "_Ref2").0,
                "Heading text",
                "{mode}, {view}"
            );
            assert_eq!(
                start_paragraph(&docx, "_Ref3").0,
                "Body text.",
                "{mode}, {view}"
            );
        }
    }
}

/// Every `w:p` in document order as (its text, whether it holds `marker`
/// — a `w:bookmarkStart` or `w:commentRangeStart`). A bookmark inside a
/// deleted run still counts: it is in the paragraph, wherever in it.
fn paragraph_summaries(docx: &[u8], marker: &str) -> Vec<(String, bool)> {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    dom.descendants(dom.root(doc).unwrap(), Some(&W::p()))
        .iter()
        .map(|&p| {
            let text: String = dom
                .descendants(p, Some(&W::t()))
                .iter()
                .map(|&t| dom.value(t))
                .collect();
            let marked = !dom.descendants(p, Some(&W::name(marker))).is_empty();
            (text, marked)
        })
        .collect()
}

/// A paragraph that holds only `marker` (a bookmark or comment anchor):
/// "Ka." before it, "Zz." after. `tail` also drops "Zz.".
fn marker_document(marker_xml: &str, tail: bool) -> (Vec<u8>, Vec<u8>) {
    let original = docx(&format!("{}{}{}", para("Ka."), marker_xml, para("Zz.")));
    let revised = if tail {
        docx(&para("Ka."))
    } else {
        docx(&format!("{}{}", para("Ka."), para("Zz.")))
    };
    (original, revised)
}

const BOOKMARK_P: &str =
    r#"<w:p><w:bookmarkStart w:id="4" w:name="b"/><w:bookmarkEnd w:id="4"/></w:p>"#;

/// #1710: B deletes the paragraph that held only the bookmark. The
/// redline keeps that paragraph, its mark deleted, the bookmark a direct
/// child of it — not inside the next paragraph's `w:del`. Reject All
/// gives A back: the bookmark in its own restored paragraph, no stray
/// empty paragraph. Accept All gives B's paragraphs.
#[test]
fn a_deleted_bookmark_paragraph_comes_back_whole() {
    for tail in [false, true] {
        let (original, revised) = marker_document(BOOKMARK_P, tail);
        for (mode, settings) in modes() {
            let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
            assert_word_valid_package(&redline);
            // A deleted paragraph's text is `w:delText`, not `w:t`.
            let zz = if tail { "" } else { "Zz." };
            assert_eq!(
                paragraph_summaries(&redline, "bookmarkStart"),
                [
                    ("Ka.".to_string(), false),
                    (String::new(), true),
                    (zz.to_string(), false)
                ],
                "{mode}, tail={tail}, redline: the bookmark stays in the deleted paragraph"
            );
            let rejected = reject_revisions(&redline).unwrap();
            assert_word_valid_package(&rejected);
            assert_eq!(
                paragraph_summaries(&rejected, "bookmarkStart"),
                [
                    ("Ka.".to_string(), false),
                    (String::new(), true),
                    ("Zz.".to_string(), false)
                ],
                "{mode}, tail={tail}, reject: A is restored"
            );
            let accepted = accept_revisions(&redline).unwrap();
            assert_word_valid_package(&accepted);
            let paragraphs = paragraph_summaries(&accepted, "bookmarkStart");
            let texts: Vec<String> = paragraphs.iter().map(|(t, _)| t.clone()).collect();
            assert_eq!(
                texts,
                if tail {
                    vec!["Ka.".to_string()]
                } else {
                    vec!["Ka.".to_string(), "Zz.".to_string()]
                },
                "{mode}, tail={tail}, accept: B's paragraphs"
            );
            // B never carried the bookmark, so a mode may drop it on
            // accept when its whole paragraph went; when it stays, it
            // rides a paragraph that survives.
            if !tail {
                assert!(
                    paragraphs.iter().any(|(_, b)| *b),
                    "{mode}, tail={tail}, accept: the bookmark survives"
                );
            }
        }
    }
}

/// The same deletion inside a table cell.
#[test]
fn a_deleted_bookmark_paragraph_in_a_cell_comes_back_whole() {
    let cell = |inner: String| docx(&format!("<w:tbl><w:tr><w:tc>{inner}</w:tc></w:tr></w:tbl>"));
    let original = cell(format!("{}{}{}", para("Ka."), BOOKMARK_P, para("Zz.")));
    let revised = cell(para("Ka."));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        let rejected = reject_revisions(&redline).unwrap();
        assert_word_valid_package(&rejected);
        assert_eq!(
            paragraph_summaries(&rejected, "bookmarkStart"),
            [
                ("Ka.".to_string(), false),
                (String::new(), true),
                ("Zz.".to_string(), false)
            ],
            "{mode}, cell reject: A is restored"
        );
        let accepted = accept_revisions(&redline).unwrap();
        assert_word_valid_package(&accepted);
        let paragraphs = paragraph_summaries(&accepted, "bookmarkStart");
        assert_eq!(
            paragraphs
                .iter()
                .map(|(t, _)| t.as_str())
                .collect::<Vec<_>>(),
            ["Ka."],
            "{mode}, cell accept: B's paragraphs"
        );
        assert!(
            paragraphs.iter().any(|(_, b)| *b),
            "{mode}, cell accept: the bookmark survives"
        );
    }
}

/// A paragraph holding only a comment's anchor: its `commentRangeStart`
/// and `commentRangeEnd` were once dropped from the redline entirely, so
/// Reject All lost the anchor. They stay with the deleted paragraph.
#[test]
fn a_deleted_comment_anchor_paragraph_comes_back_whole() {
    const ANCHOR_P: &str = r#"<w:p><w:commentRangeStart w:id="0"/><w:commentRangeEnd w:id="0"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="0"/></w:r></w:p>"#;
    let comments = format!(
        r#"<w:comments xmlns:w="{}"><w:comment w:id="0" w:author="T"><w:p><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#,
        common::docx::W_NS
    );
    let with_comments = |body: String| {
        docx_with(
            &body,
            &[Part {
                name: "word/comments.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: &comments,
            }],
        )
    };
    let original = with_comments(format!("{}{}{}", para("Ka."), ANCHOR_P, para("Zz.")));
    let revised = with_comments(para("Ka."));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        assert!(
            part_string(&redline, "word/document.xml")
                .unwrap()
                .contains("<w:commentRangeStart"),
            "{mode}, redline: the anchor is carried"
        );
        let rejected = reject_revisions(&redline).unwrap();
        assert_word_valid_package(&rejected);
        let paragraphs = paragraph_summaries(&rejected, "commentRangeStart");
        assert_eq!(paragraphs.len(), 3, "{mode}, reject: A's three paragraphs");
        assert_eq!(paragraphs[0].0, "Ka.", "{mode}, reject");
        assert!(
            paragraphs[1].1 && paragraphs[1].0.is_empty(),
            "{mode}, reject: the anchor is back in its own paragraph"
        );
        assert_eq!(paragraphs[2].0, "Zz.", "{mode}, reject");
    }
}

/// `body` as a document that declares the VML namespace.
fn vml_docx(body: &str) -> Vec<u8> {
    let built = docx(body);
    let document = part_string(&built, "word/document.xml").unwrap().replacen(
        "<w:document ",
        r#"<w:document xmlns:v="urn:schemas-microsoft-com:vml" "#,
        1,
    );
    replace_entry(&built, "word/document.xml", document.as_bytes())
}

/// A paragraph without text after the last text keeps its bookmark: no
/// later text holds an offset to place it by, so the start must not fall
/// back to the end of the text before it.
#[test]
fn a_trailing_picture_paragraph_keeps_its_start() {
    let build = |intro: &str| {
        vml_docx(&format!(
            r#"{}{}<w:p><w:bookmarkStart w:id="0" w:name="_Toc1"/>{PICTURE}<w:bookmarkEnd w:id="0"/></w:p>"#,
            para(intro),
            para("Heading text")
        ))
    };
    assert_kept(
        &build("Intro."),
        &build("Intro, changed."),
        "pict",
        "trailing picture",
    );
}

/// A text box drawn in the paragraph without text, ahead of the bookmark.
const TEXT_BOX: &str = r#"<w:r><w:pict><v:shape style="width:100pt;height:20pt"><v:textbox><w:txbxContent><w:p><w:r><w:t>Box text</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r>"#;

/// The text box's own paragraph does not count as one of the paragraphs
/// between the start and its next text: the start stays in the paragraph
/// that draws the box, not in the box.
#[test]
fn a_text_box_before_the_start_does_not_take_it() {
    let build = |after: &str| {
        vml_docx(&format!(
            r#"<w:p>{TEXT_BOX}<w:bookmarkStart w:id="0" w:name="_Toc1"/>{TAB}</w:p>{after}{}<w:bookmarkEnd w:id="0"/>{}"#,
            para("Heading text"),
            para("Body text.")
        ))
    };
    let (original, revised) = (build(""), build(&para("Probe paragraph.")));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            let xml = part_string(&docx, "word/document.xml").unwrap();
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&xml);
            let root = dom.root(doc).unwrap();
            let start = dom
                .descendants(root, Some(&W::name("bookmarkStart")))
                .into_iter()
                .find(|&s| dom.attribute(s, &W::name("name")) == Some("_Toc1"))
                .unwrap_or_else(|| panic!("{mode}, {view}: no _Toc1: {xml}"));
            assert!(
                dom.ancestors(start, Some(&W::name("txbxContent")))
                    .is_empty(),
                "{mode}, {view}: the start went into the text box: {xml}"
            );
            let p = dom.ancestors(start, Some(&W::p()))[0];
            assert!(
                !dom.descendants(p, Some(&W::name("txbxContent"))).is_empty(),
                "{mode}, {view}: the start left the box's paragraph: {}",
                dom.serialize_element(p)
            );
        }
    }
}

/// With no text after the start, the last text may sit in a text box the
/// start's own paragraph draws: the start stays in that paragraph, not in
/// the box.
#[test]
fn a_last_paragraph_drawing_a_text_box_keeps_its_start() {
    let build = |intro: &str| {
        vml_docx(&format!(
            r#"{}<w:p>{TEXT_BOX}<w:bookmarkStart w:id="0" w:name="_Toc1"/>{TAB}<w:bookmarkEnd w:id="0"/></w:p>"#,
            para(intro)
        ))
    };
    let (original, revised) = (build("Intro."), build("Intro, changed."));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            let xml = part_string(&docx, "word/document.xml").unwrap();
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&xml);
            let root = dom.root(doc).unwrap();
            let start = dom
                .descendants(root, Some(&W::name("bookmarkStart")))
                .into_iter()
                .find(|&s| dom.attribute(s, &W::name("name")) == Some("_Toc1"))
                .unwrap_or_else(|| panic!("{mode}, {view}: no _Toc1: {xml}"));
            assert!(
                dom.ancestors(start, Some(&W::name("txbxContent")))
                    .is_empty(),
                "{mode}, {view}: the start went into the text box: {xml}"
            );
            let p = dom.ancestors(start, Some(&W::p()))[0];
            assert!(
                !dom.descendants(p, Some(&W::name("txbxContent"))).is_empty(),
                "{mode}, {view}: the start left the box's paragraph: {}",
                dom.serialize_element(p)
            );
        }
    }
}

/// Cursor Bugbot on #1921: a paragraph inserted between the last text and a
/// trailing paragraph without text does not take that paragraph's start.
#[test]
fn a_paragraph_inserted_before_a_trailing_picture_leaves_its_start() {
    let build = |inserted: &str| {
        vml_docx(&format!(
            r#"{}{}{inserted}<w:p><w:bookmarkStart w:id="0" w:name="_Toc1"/>{PICTURE}<w:bookmarkEnd w:id="0"/></w:p>"#,
            para("Intro."),
            para("Heading text")
        ))
    };
    assert_kept(
        &build(""),
        &build(&para("Probe paragraph.")),
        "pict",
        "inserted before a trailing picture",
    );
    // An inserted paragraph with no text of its own either.
    let original = build("");
    let revised = build(&format!("<w:p>{TAB}</w:p>"));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            assert_starts_in(&docx, "pict", &format!("textless insert, {mode}, {view}"));
        }
    }
}

/// The bookmark only the original holds. A paragraph inserted ahead of a
/// textless one that is not the document's first is not a paragraph of the
/// original's view, so the start stays in the textless paragraph.
#[test]
fn an_original_only_start_stays_out_of_a_paragraph_inserted_before_it() {
    let build = |inserted: &str, bookmark: bool| {
        let (start, end) = if bookmark {
            (
                r#"<w:bookmarkStart w:id="0" w:name="_Toc1"/>"#,
                r#"<w:bookmarkEnd w:id="0"/>"#,
            )
        } else {
            ("", "")
        };
        docx(&format!(
            r#"{}{inserted}<w:p>{start}{TAB}{end}</w:p>{}{}"#,
            para("Intro."),
            para("Heading text"),
            para("Body text.")
        ))
    };
    let original = build("", true);
    let revised = build(&para("Probe paragraph."), false);
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            assert_starts_in(
                &docx,
                "tab",
                &format!("original only before, {mode}, {view}"),
            );
        }
    }
}

/// The bookmark only the original holds. A paragraph the revised side
/// inserts between the last text and the trailing picture is not a
/// paragraph of the original's view, so the start stays in the picture.
#[test]
fn an_original_only_start_stays_out_of_a_paragraph_inserted_before_the_picture() {
    let build = |inserted: &str, bookmark: bool| {
        let (start, end) = if bookmark {
            (
                r#"<w:bookmarkStart w:id="0" w:name="_Toc1"/>"#,
                r#"<w:bookmarkEnd w:id="0"/>"#,
            )
        } else {
            ("", "")
        };
        vml_docx(&format!(
            r#"{}{}{inserted}<w:p>{start}{PICTURE}{end}</w:p>"#,
            para("Intro."),
            para("Heading text")
        ))
    };
    let original = build("", true);
    let revised = build(&para("Probe paragraph."), false);
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            assert_starts_in(
                &docx,
                "pict",
                &format!("original only insert, {mode}, {view}"),
            );
        }
    }
}

/// Both documents bookmark the trailing picture. The original has one more
/// paragraph before it, which the revised side deletes. The start is placed
/// from the revised view and stays in the picture.
#[test]
fn a_paragraph_deleted_before_a_trailing_picture_leaves_its_start() {
    let build = |extra: &str| {
        vml_docx(&format!(
            r#"{}{}{extra}<w:p><w:bookmarkStart w:id="0" w:name="_Toc1"/>{PICTURE}<w:bookmarkEnd w:id="0"/></w:p>"#,
            para("Intro."),
            para("Heading text")
        ))
    };
    let (original, revised) = (build(&para("Gone paragraph.")), build(""));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        assert!(
            part_string(&redline, "word/document.xml")
                .unwrap()
                .contains("<w:del "),
            "{mode}: the deletion is marked"
        );
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            assert_starts_in(
                &docx,
                "pict",
                &format!("deleted before picture, {mode}, {view}"),
            );
        }
    }
}

/// Both documents bookmark the tab. The original has one more paragraph
/// ahead of it, which the revised side deletes. The start is placed from
/// the revised view and stays on the tab.
#[test]
fn a_paragraph_deleted_before_a_textless_tab_leaves_its_start() {
    let original = package(
        TAB,
        &format!("{}{}", para("Intro."), para("Gone paragraph.")),
        "",
        true,
    );
    let revised = package(TAB, &para("Intro."), "", true);
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        assert!(
            part_string(&redline, "word/document.xml")
                .unwrap()
                .contains("<w:del "),
            "{mode}: the deletion is marked"
        );
        for (view, docx) in [
            ("redline", redline.clone()),
            ("accept", accept_revisions(&redline).unwrap()),
            ("reject", reject_revisions(&redline).unwrap()),
        ] {
            assert_starts_in(&docx, "tab", &format!("deleted before tab, {mode}, {view}"));
        }
    }
}
