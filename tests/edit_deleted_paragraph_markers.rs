// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1676: deleting a paragraph must leave its bookmarks and range markers in
//! Word's shape. Accept All of that redline and the clean copy hold the same
//! markers. Word's compare (`tests/fixtures/1676-word.docx`) puts the end of a
//! nonempty range inside the deletion, and leaves an empty bookmark outside it.

mod common;

use common::docx::{Part, W_NS, docx, docx_with, para, part_string};
use common::validity::check_word_valid_package;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};

const WORD: &[u8] = include_bytes!("fixtures/1676-word.docx");
const MODES: [&str; 2] = ["", r#","existing_revisions":"keep""#];
const KEEP: &str = MODES[1];

fn plan(mode: &str, paragraph: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-19T12:00:00Z"{mode},"operations":[{{"kind":"delete_paragraph","paragraph":"{paragraph}"}}]}}"#
    ))
    .unwrap()
}

fn cell(inner: &str) -> String {
    format!(r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>{inner}</w:tc>"#)
}

fn table(inner: &str) -> String {
    format!(
        "{}<w:tbl><w:tblPr><w:tblW w:w=\"4000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{}{}</w:tr></w:tbl>{}",
        para("Before."),
        cell(&para("Full.")),
        cell(inner),
        para("After.")
    )
}

fn with_comments(body: &str, ids: &[(u32, &str)]) -> Vec<u8> {
    let comments: String = ids
        .iter()
        .map(|(id, text)| {
            format!(
                r#"<w:comment w:id="{id}" w:author="Ann Counsel" w:date="2026-10-19T12:00:00Z" w:initials="AC"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#
            )
        })
        .collect();
    let xml = format!(r#"<w:comments xmlns:w="{W_NS}">{comments}</w:comments>"#);
    docx_with(
        body,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &xml,
        }],
    )
}

fn normalized(xml: &str) -> String {
    xml.replace(" />", "/>").replace("  />", "/>")
}

/// Marker tags, in order, from one part. `w:id` is the comparer's renumbering,
/// not which marker survived.
fn marker_tags(xml: &str) -> Vec<String> {
    let xml = normalized(xml);
    let mut out = Vec::new();
    let mut rest = xml.as_str();
    while let Some(at) = rest.find("<w:") {
        rest = &rest[at..];
        let end = rest.find('>').expect("element");
        let tag = &rest[..=end];
        if tag.contains("bookmark")
            || tag.contains("comment")
            || tag.contains("permStart")
            || tag.contains("permEnd")
        {
            out.push(strip_id(tag));
        }
        rest = &rest[end + 1..];
    }
    out
}

fn strip_id(tag: &str) -> String {
    let mut out = String::new();
    let mut rest = tag;
    while let Some(at) = rest.find(" w:id=\"") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 7..];
        let Some(end) = after.find('"') else {
            out.push_str(rest);
            return out;
        };
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

fn paragraph_text(paragraph: &str) -> String {
    let mut text = String::new();
    for leaf in ["<w:t", "<w:delText"] {
        let mut rest = paragraph;
        while let Some(at) = rest.find(leaf) {
            rest = &rest[at..];
            let Some(close) = rest.find('>') else { break };
            rest = &rest[close + 1..];
            let Some(end) = rest.find("</w:") else { break };
            text.push_str(&rest[..end]);
            rest = &rest[end..];
        }
    }
    text
}

/// Each marker with the text of the paragraph that holds it. Clean and
/// Accept All renumber ids independently; the paragraph is the placement.
fn markers(bytes: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(xml) = part_string(bytes, "word/document.xml") {
        let xml = normalized(&xml);
        let mut rest = xml.as_str();
        while let Some(at) = rest.find("<w:p") {
            let next = rest.as_bytes().get(at + 4).copied();
            if next == Some(b'/') || (next != Some(b'>') && next != Some(b' ')) {
                rest = &rest[at + 4..];
                continue;
            }
            let Some(end) = rest[at..].find("</w:p>") else {
                break;
            };
            let paragraph = &rest[at..at + end + 6];
            let text = paragraph_text(paragraph);
            for tag in marker_tags(paragraph) {
                out.push(format!("word/document.xml {text} {tag}"));
            }
            rest = &rest[at + end + 6..];
        }
    }
    if let Some(xml) = part_string(bytes, "word/comments.xml") {
        for tag in marker_tags(&xml) {
            out.push(format!("word/comments.xml {tag}"));
        }
    }
    out
}

fn paired(bytes: &[u8], start: &str, end: &str) {
    let xml = part_string(bytes, "word/document.xml").unwrap_or_default();
    let ids = |kind: &str| -> Vec<String> {
        xml.match_indices(&format!("<w:{kind}"))
            .map(|(i, _)| {
                let tag = &xml[i..i + xml[i..].find('>').unwrap()];
                tag.split("w:id=\"")
                    .nth(1)
                    .and_then(|v| v.split('"').next())
                    .unwrap_or("")
                    .to_string()
            })
            .collect()
    };
    let mut starts = ids(start);
    let mut ends = ids(end);
    starts.sort();
    ends.sort();
    assert_eq!(starts, ends, "{start}/{end} unpaired in {xml}");
}

fn assert_outputs_agree(source: &[u8], paragraph: &str, what: &str) {
    for mode in MODES {
        let result = apply_plan(source, &plan(mode, paragraph))
            .unwrap_or_else(|e| panic!("{what} {mode}: {e:?}"));
        let accepted = accept_revisions(&result.redline).expect("accept");
        for (bytes, output) in [(&result.clean, "clean"), (&accepted, "Accept All")] {
            let report = check_word_valid_package(bytes);
            assert!(report.ok(), "{what} {mode} {output}: {:?}", report.errors);
            paired(bytes, "bookmarkStart", "bookmarkEnd");
            paired(bytes, "commentRangeStart", "commentRangeEnd");
            paired(bytes, "permStart", "permEnd");
        }
        assert_eq!(
            markers(&result.clean),
            markers(&accepted),
            "{what} {mode}: clean markers differ from Accept All"
        );
    }
}

/// The paragraph element in `xml` that contains `needle`.
fn paragraph_holding(xml: &str, needle: &str) -> String {
    let xml = normalized(xml);
    let at = xml
        .find(needle)
        .unwrap_or_else(|| panic!("no {needle} in {xml}"));
    let start = xml[..at]
        .rfind("<w:p>")
        .or_else(|| xml[..at].rfind("<w:p "))
        .unwrap();
    let end = at + xml[at..].find("</w:p>").unwrap() + 6;
    xml[start..end].to_string()
}

/// Whether `needle` sits inside the paragraph's content `w:del` (not the
/// mark's `w:del` in `w:pPr`).
fn inside_content_del(paragraph: &str, needle: &str) -> bool {
    let body = paragraph
        .find("</w:pPr>")
        .map_or(paragraph, |i| &paragraph[i + 8..]);
    let Some(del) = body.find("<w:del ") else {
        return false;
    };
    let Some(end) = body[del..].find("</w:del>") else {
        return false;
    };
    body[del..del + end].contains(needle)
}

fn bookmark(name: &str, id: &str, text: &str) -> String {
    format!(
        r#"<w:p><w:bookmarkStart w:id="{id}" w:name="{name}"/><w:r><w:t xml:space="preserve">{text}</w:t></w:r><w:bookmarkEnd w:id="{id}"/></w:p>"#
    )
}

fn comment(id: &str, text: &str) -> String {
    format!(
        r#"<w:p><w:commentRangeStart w:id="{id}"/><w:r><w:t xml:space="preserve">{text}</w:t></w:r><w:commentRangeEnd w:id="{id}"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="{id}"/></w:r></w:p>"#
    )
}

fn perm(id: &str, text: &str) -> String {
    format!(
        r#"<w:p><w:permStart w:id="{id}" w:edGrp="everyone"/><w:r><w:t xml:space="preserve">{text}</w:t></w:r><w:permEnd w:id="{id}"/></w:p>"#
    )
}

/// Word's compare of a deleted paragraph: a nonempty bookmark end is inside
/// the deletion when the mark goes, and after it when the cell keeps the mark.
#[test]
fn word_puts_a_deleted_paragraphs_range_ends_in_the_deletion() {
    let xml = part_string(WORD, "word/document.xml").unwrap();
    let mid = paragraph_holding(&xml, r#"w:name="mid""#);
    assert!(
        inside_content_del(&mid, "bookmarkEnd"),
        "Word mid bookmark end is outside the deletion: {mid}"
    );
    assert!(inside_content_del(&mid, "permEnd"), "{mid}");
    assert!(inside_content_del(&mid, "commentRangeEnd"), "{mid}");
    let cell = paragraph_holding(&xml, r#"w:name="cell""#);
    assert!(
        !inside_content_del(&cell, "bookmarkEnd"),
        "Word keeps the cell bookmark end with the surviving mark: {cell}"
    );
    assert!(inside_content_del(&cell, "permEnd"), "{cell}");
    assert!(inside_content_del(&cell, "commentRangeEnd"), "{cell}");
    let empty = paragraph_holding(&xml, r#"w:name="empty""#);
    assert!(
        !inside_content_del(&empty, r#"w:name="empty""#),
        "Word leaves an empty bookmark outside the deletion: {empty}"
    );
}

#[test]
fn deleted_paragraph_markers_match_word_on_accept_all() {
    assert_outputs_agree(
        &docx(&format!(
            "{}{}{}",
            para("Keep."),
            bookmark("mid", "3", "Gone."),
            para("After.")
        )),
        "p1",
        "mid bookmark",
    );
    assert_outputs_agree(
        &docx(&table(&bookmark("cell", "4", "Old."))),
        "p2",
        "cell bookmark",
    );
    assert_outputs_agree(
        &with_comments(
            &format!(
                "{}{}{}",
                para("Keep."),
                comment("0", "Gone."),
                para("After.")
            ),
            &[(0, "Mid.")],
        ),
        "p1",
        "mid comment",
    );
    assert_outputs_agree(
        &with_comments(&table(&comment("1", "Old.")), &[(1, "Cell.")]),
        "p2",
        "cell comment",
    );
    assert_outputs_agree(
        &docx(&format!(
            "{}{}{}",
            para("Keep."),
            perm("7", "Gone."),
            para("After.")
        )),
        "p1",
        "mid permission",
    );
    assert_outputs_agree(&docx(&table(&perm("8", "Old."))), "p2", "cell permission");

    // An empty bookmark has no characters to lose. Word keeps it, and so
    // does Accept All; the clean copy has to keep it in that same place.
    let empty = |text: &str| {
        format!(
            r#"<w:p><w:bookmarkStart w:id="9" w:name="empty"/><w:bookmarkEnd w:id="9"/><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#
        )
    };
    assert_outputs_agree(
        &docx(&format!(
            "{}{}{}",
            para("Keep."),
            empty("Gone."),
            para("After.")
        )),
        "p1",
        "mid empty bookmark",
    );
    assert_outputs_agree(&docx(&table(&empty("Old."))), "p2", "cell empty bookmark");

    let kept = apply_plan(
        &docx(&format!(
            "{}{}{}",
            para("Keep."),
            bookmark("mid", "3", "Gone."),
            para("After.")
        )),
        &plan(KEEP, "p1"),
    )
    .unwrap();
    let mid = paragraph_holding(
        &part_string(&kept.redline, "word/document.xml").unwrap(),
        "Gone.",
    );
    assert!(
        inside_content_del(&mid, "bookmarkEnd"),
        "keep redline should put the mid bookmark end inside the deletion, as Word does: {mid}"
    );
    let report = check_word_valid_package(&kept.redline);
    assert!(report.ok(), "keep redline: {:?}", report.errors);
    let rejected = reject_revisions(&kept.redline).unwrap();
    assert!(
        part_string(&rejected, "word/document.xml")
            .unwrap()
            .contains(r#"w:name="mid""#),
        "reject restores the bookmark"
    );
}

/// #2104 review: a deleted last paragraph has no following paragraph to
/// carry its empty bookmark. Accept All keeps the bookmark, so the clean
/// copy has to keep it too — in the last body paragraph and in the last
/// paragraph of a cell that keeps another.
#[test]
fn a_deleted_last_paragraph_keeps_its_empty_bookmark() {
    let empty = |text: &str| {
        format!(
            r#"<w:p><w:bookmarkStart w:id="9" w:name="empty"/><w:bookmarkEnd w:id="9"/><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#
        )
    };
    let body = docx(&format!("{}{}", para("Keep."), empty("Gone.")));
    assert_outputs_agree(&body, "p1", "last body empty bookmark");
    let clean = apply_plan(&body, &plan("", "p1")).unwrap().clean;
    assert!(
        part_string(&clean, "word/document.xml")
            .unwrap()
            .contains(r#"w:name="empty""#),
        "clean copy dropped the last paragraph's empty bookmark"
    );

    let in_cell = docx(&table(&format!("{}{}", para("Stay."), empty("Old."))));
    assert_outputs_agree(&in_cell, "p3", "last cell empty bookmark");
    let clean = apply_plan(&in_cell, &plan("", "p3")).unwrap().clean;
    assert!(
        part_string(&clean, "word/document.xml")
            .unwrap()
            .contains(r#"w:name="empty""#),
        "clean copy dropped the cell's last empty bookmark"
    );
}
