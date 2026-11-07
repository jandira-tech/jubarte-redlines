// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comment marks that sit inside a revision, but not at the start of the
//! paragraph, come back inside that same revision. No new revision id is
//! allocated, and rejecting the round trip removes the same comments as
//! rejecting the source.

mod common;

use common::docx::{Part, R_NS, W_NS, docx_with, part_string};
use jubarte::changes::{ChangeFilter, list_changes, reject_changes};
use jubarte::comments::list_comments;
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

const REF: &str = r#"<w:rPr><w:rStyle w:val="CommentReference"/></w:rPr>"#;
const WHEN: &str = r#"w:author="Ann Counsel" w:date="2026-10-01T10:00:00Z""#;

fn comment_ref(id: u32) -> String {
    format!(r#"<w:r>{REF}<w:commentReference w:id="{id}"/></w:r>"#)
}

fn package(body: &str, notes: &[(u32, &str, &str)]) -> Vec<u8> {
    let entries: String = notes
        .iter()
        .map(|(id, author, text)| {
            format!(
                r#"<w:comment w:id="{id}" w:author="{author}" w:date="2026-10-01T10:00:00Z" w:initials="A"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#
            )
        })
        .collect();
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="{W_NS}" xmlns:r="{R_NS}">{entries}</w:comments>"#
    );
    let rels = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    docx_with(
        body,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: &format!("{rels}/comments"),
            xml: &comments,
        }],
    )
}

/// `w:del 70` holds only the range start. The end and the reference follow
/// the kept text.
fn probe_c1() -> Vec<u8> {
    package(
        &format!(
            r#"<w:p><w:del w:id="70" {WHEN}><w:commentRangeStart w:id="0"/></w:del><w:r><w:t>kept</w:t></w:r><w:commentRangeEnd w:id="0"/>{}</w:p>"#,
            comment_ref(0)
        ),
        &[(0, "Zoe Stone", "Check.")],
    )
}

/// `w:del 71` holds both threads' ends and references, after the visible text.
fn probe_c2() -> Vec<u8> {
    package(
        &format!(
            r#"<w:p><w:commentRangeStart w:id="0"/><w:commentRangeStart w:id="1"/><w:r><w:t>kept</w:t></w:r><w:del w:id="71" {WHEN}><w:commentRangeEnd w:id="0"/>{}<w:commentRangeEnd w:id="1"/>{}</w:del></w:p>"#,
            comment_ref(0),
            comment_ref(1)
        ),
        &[(0, "Zoe Stone", "Check."), (1, "Bob Hale", "Reply.")],
    )
}

/// `w:del 70` holds the deleted text, then the range end and the reference.
/// The range start is before the deletion.
fn probe_p3() -> Vec<u8> {
    package(
        &format!(
            r#"<w:p><w:commentRangeStart w:id="0"/><w:del w:id="70" {WHEN}><w:r><w:delText>gone</w:delText></w:r><w:commentRangeEnd w:id="0"/>{}</w:del></w:p>"#,
            comment_ref(0)
        ),
        &[(0, "Zoe Stone", "Check.")],
    )
}

/// `w:ins 51` holds the inserted text and a trailing reference.
fn probe_p1b() -> Vec<u8> {
    package(
        &format!(
            r#"<w:p><w:commentRangeStart w:id="0"/><w:ins w:id="51" {WHEN}><w:r><w:t>added</w:t></w:r><w:commentRangeEnd w:id="0"/>{}</w:ins></w:p>"#,
            comment_ref(0)
        ),
        &[(0, "Zoe Stone", "Check.")],
    )
}

fn events(docx: &[u8]) -> Vec<String> {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let mut out = Vec::new();
    walk(&dom, doc, &mut out);
    out
}

fn walk(dom: &Dom, id: NodeId, out: &mut Vec<String>) {
    if let Some(name) = dom.name(id) {
        let local = name.local_name();
        if matches!(
            local,
            "commentRangeStart" | "commentRangeEnd" | "commentReference"
        ) {
            let cid = dom.attribute(id, &W::id()).unwrap_or("?");
            let kind = match local {
                "commentRangeStart" => "start",
                "commentRangeEnd" => "end",
                _ => "ref",
            };
            out.push(format!("{kind} {cid} {}", owner(dom, id)));
        }
    }
    for child in dom.elements(id, None) {
        walk(dom, child, out);
    }
}

fn owner(dom: &Dom, mut id: NodeId) -> String {
    while let Some(parent) = dom.parent(id) {
        if let Some(name) = dom.name(parent) {
            let local = name.local_name();
            if matches!(local, "ins" | "del") {
                let wid = dom.attribute(parent, &W::id()).unwrap_or("?");
                return format!("in {local} {wid}");
            }
        }
        id = parent;
    }
    "outside".to_string()
}

fn revision_ids(docx: &[u8]) -> Vec<String> {
    list_changes(docx)
        .unwrap()
        .into_iter()
        .map(|change| change.id)
        .collect()
}

fn rejected(docx: &[u8]) -> (Vec<String>, Vec<String>) {
    let out = reject_changes(docx, &ChangeFilter::default()).unwrap();
    let comments = list_comments(&out)
        .unwrap()
        .into_iter()
        .map(|comment| comment.r#ref)
        .collect();
    (events(&out), comments)
}

fn keeps(source: &[u8]) {
    let options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    let view = read(source, &options).unwrap().markdown;
    let back = markdown_to_docx(&view, &DocxOptions::default()).unwrap();
    let xml = part_string(&back.docx, "word/document.xml").unwrap_or_default();
    assert_eq!(
        events(&back.docx),
        events(source),
        "view:\n{view}\nback:\n{xml}"
    );
    assert_eq!(
        revision_ids(&back.docx),
        revision_ids(source),
        "{view}\n{xml}"
    );
    assert_eq!(rejected(&back.docx), rejected(source), "{view}\n{xml}");
    assert!(back.warnings.is_empty(), "{:?}\n{view}", back.warnings);
}

#[test]
fn held_comment_probe_c1_start_stays_in_its_deletion() {
    keeps(&probe_c1());
}

#[test]
fn held_comment_probe_c2_ends_stay_in_one_deletion() {
    keeps(&probe_c2());
}

#[test]
fn held_comment_probe_p3_end_stays_in_its_deletion() {
    keeps(&probe_p3());
}

#[test]
fn held_comment_probe_p1b_reference_stays_in_its_insertion() {
    keeps(&probe_p1b());
}
