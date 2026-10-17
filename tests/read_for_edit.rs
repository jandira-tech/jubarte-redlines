// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `read --for-edit` (V2-10): exact working hashes, paired target IDs and
//! range-specific readiness on the working document template/apply derive.

mod common;

use common::docx::{Part, docx, docx_with, docx_with_sect, para};
use jubarte::edit::ExistingRevisions;
use jubarte::edit::for_edit::{Correspondence, ForEditRequest, for_edit};
use jubarte::inspect::source_sha256;
use serde_json::{Value, json};

/// The acceptance fixture of the template plan §6: a pending insertion
/// changes "30" to "45"; the rest of the paragraph is plain text.
const PENDING: &str = r#"<w:p><w:r><w:t xml:space="preserve">Payment is due within </w:t></w:r><w:del w:id="8" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:delText>30</w:delText></w:r></w:del><w:ins w:id="7" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>45</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> days. Late interest is 2%.</w:t></w:r></w:p>"#;

fn source() -> Vec<u8> {
    docx(&(PENDING.to_string() + &para("Governing law: Delaware.")))
}

fn status<'a>(row: &'a jubarte::edit::for_edit::ForEditRow, operation: &str) -> &'a str {
    &row.readiness
        .iter()
        .find(|r| r.operation == operation)
        .unwrap_or_else(|| panic!("no readiness row for {operation}"))
        .status
}

#[test]
fn pending_insertion_only_blocks_its_changed_span() {
    let result = for_edit(&source(), &ForEditRequest::default()).unwrap();
    assert_eq!(result.paragraphs.len(), 2);
    let row = &result.paragraphs[0];
    assert_eq!(row.target.short_id, "p0");

    // The working text is the final logical view: "45" in, "30" out.
    let working = row.working.as_ref().expect("a working view under keep");
    let text = working.value.text.as_deref().expect("complete text");
    assert_eq!(text, "Payment is due within 45 days. Late interest is 2%.");
    assert_eq!(
        working.before_sha256,
        Some(source_sha256(text.as_bytes())),
        "before_sha256 hashes the exact UTF-8 working text"
    );

    // The insertion blocks exactly its own span, and nothing else.
    let ins = row
        .blockers
        .iter()
        .find(|b| b.kind == "tracked_insertion")
        .expect("the pending insertion is a blocker");
    let at = text.find("45").unwrap();
    assert_eq!(ins.range, [at, at + 2]);
    assert_eq!(ins.decision, "conditional");
    let revision = ins.revision.as_ref().expect("the blocking revision pair");
    assert_eq!(revision.short_id, "r7");
    assert_eq!(
        revision.long_id, "document.w:ins.w:id.7",
        "the revision's native w:id is its long key"
    );
    // The blockers cover exactly the insertion's span: nothing else in
    // the paragraph is blocked, on any position.
    let mut ranges: Vec<[usize; 2]> = row.blockers.iter().map(|b| b.range).collect();
    ranges.sort_unstable();
    let mut merged: Vec<[usize; 2]> = Vec::new();
    for [start, end] in ranges {
        match merged.last_mut() {
            Some([_, last_end]) if start <= *last_end => *last_end = (*last_end).max(end),
            _ => merged.push([start, end]),
        }
    }
    assert_eq!(
        merged,
        vec![ins.range],
        "the union of blocker ranges is exactly the tracked insertion's range"
    );

    // Readiness is range-specific: the paragraph is not uneditable, the
    // changed span alone is conditional on a decision about r7.
    assert_eq!(status(row, "replace"), "conditional");
    assert_eq!(status(row, "rewrite"), "conditional");
    assert_eq!(status(row, "comment"), "available");
    // The clean paragraph carries no restriction.
    let clean = &result.paragraphs[1];
    assert_eq!(status(clean, "replace"), "available");
    assert_eq!(status(clean, "rewrite"), "available");
    assert!(clean.blockers.is_empty());
}

#[test]
fn selection_and_source_context_remain_bound_to_original() {
    // --find sees S's final text; rejecting the insertion removes the
    // match from W, yet the frozen selection still names p0.
    let request = ForEditRequest {
        find: Some("45".to_string()),
        reject: vec!["r7".to_string()],
        context: 1,
        ..ForEditRequest::default()
    };
    let result = for_edit(&source(), &request).unwrap();
    assert_eq!(
        result.paragraphs.len(),
        1,
        "the frozen selection keeps the paragraph whose match rejection removed"
    );
    let row = &result.paragraphs[0];
    assert_eq!(row.target.short_id, "p0");
    assert_eq!(
        row.matched_by,
        ["find"],
        "the original match is labelled, not silently dropped"
    );
    assert_eq!(
        row.original.text.as_deref(),
        Some("Payment is due within 45 days. Late interest is 2%."),
        "the original observation keeps S's final text"
    );
    let working = row.working.as_ref().expect("a working view");
    assert_eq!(
        working.value.text.as_deref(),
        Some("Payment is due within  days. Late interest is 2%."),
        "working text reflects the rejection"
    );
    assert_eq!(
        working.before_sha256,
        Some(source_sha256(
            "Payment is due within  days. Late interest is 2%.".as_bytes()
        ))
    );
    assert_eq!(
        result.settled.rejected,
        ["r7"],
        "the read reports which decisions shaped W"
    );
    // --context emits the neighbour as a read-only original observation.
    assert_eq!(result.source_context.len(), 1);
    let context = &result.source_context[0];
    assert_eq!(context.target.short_id, "p1");
    assert_eq!(context.kind, "paragraph");
    assert_eq!(
        context.value.text.as_deref(),
        Some("Governing law: Delaware.")
    );
    assert!(
        context.reason_codes.contains(&"context".to_string()),
        "context rows never become rewrite rows"
    );
}

#[test]
fn refuse_policy_returns_blockers_not_a_usable_hash() {
    let request = ForEditRequest {
        existing_revisions: ExistingRevisions::Refuse,
        ..ForEditRequest::default()
    };
    let error = for_edit(&source(), &request).unwrap_err();
    assert_eq!(error.code, "EXISTING_REVISIONS");
}

#[test]
fn accept_change_reshapes_the_working_text_and_hash() {
    let request = ForEditRequest {
        accept: vec!["r7".to_string()],
        ..ForEditRequest::default()
    };
    let result = for_edit(&source(), &request).unwrap();
    let row = &result.paragraphs[0];
    let working = row.working.as_ref().expect("a working view");
    assert_eq!(
        working.value.text.as_deref(),
        Some("Payment is due within 45 days. Late interest is 2%.")
    );
    assert!(
        row.blockers.iter().all(|b| b.kind != "tracked_insertion"),
        "an accepted insertion is no longer a pending blocker"
    );
    assert_eq!(status(row, "replace"), "available");
    assert_eq!(result.settled.accepted, ["r7"]);
}

#[test]
fn bounded_previews_never_pose_as_complete_payloads() {
    let request = ForEditRequest {
        limit: Some(1),
        max_chars: Some(20),
        ..ForEditRequest::default()
    };
    let result = for_edit(&source(), &request).unwrap();
    assert!(result.coverage.truncated);
    let row = &result.paragraphs[0];
    let working = row.working.as_ref().expect("a working view");
    assert!(!working.value.complete);
    assert!(
        working.value.text.is_none(),
        "a truncated row has no copyable complete text"
    );
    assert_eq!(
        working.value.text_preview.as_deref(),
        Some("Payment is due within")
    );
    // The full-text hash is still shown, but the preview is no payload.
    assert_eq!(
        working.before_sha256,
        Some(source_sha256(
            "Payment is due within 45 days. Late interest is 2%.".as_bytes()
        ))
    );
    let cursor = result.next_cursor.expect("a bounded page continues");

    // The same basis continues the cursor; a changed basis cannot.
    let page_two = for_edit(
        &source(),
        &ForEditRequest {
            cursor: Some(cursor.clone()),
            ..request.clone()
        },
    )
    .unwrap();
    assert!(page_two.coverage.paragraphs_returned >= 1);
    let changed = for_edit(
        &source(),
        &ForEditRequest {
            cursor: Some(cursor),
            track: false,
            ..request
        },
    )
    .unwrap_err();
    assert_eq!(changed.code, "CURSOR_BASIS_CHANGED");
}

#[test]
fn limit_pages_bind_selection_and_emit_a_cursor() {
    let doc = docx(&(para("one.") + &para("two.") + &para("three.")));
    let request = ForEditRequest {
        limit: Some(2),
        ..ForEditRequest::default()
    };
    let first = for_edit(&doc, &request).unwrap();
    assert_eq!(first.coverage.paragraphs_selected, 3);
    assert_eq!(first.coverage.paragraphs_returned, 2);
    assert!(!first.coverage.selection_complete);
    let cursor = first.next_cursor.clone().expect("page one continues");
    let second = for_edit(
        &doc,
        &ForEditRequest {
            cursor: Some(cursor.clone()),
            ..request.clone()
        },
    )
    .unwrap();
    assert_eq!(second.paragraphs.len(), 1);
    assert_eq!(second.paragraphs[0].target.short_id, "p2");
    assert!(second.coverage.selection_complete);
    assert!(second.next_cursor.is_none());
    // A different document cannot continue the cursor.
    let other = for_edit(
        &docx(&para("other.")),
        &ForEditRequest {
            cursor: Some(cursor),
            ..request
        },
    )
    .unwrap_err();
    assert_eq!(other.code, "CURSOR_BASIS_CHANGED");
}

#[test]
fn paired_ids_take_the_native_paraid_when_word_gave_one() {
    let doc = docx(
        r#"<w:p w14:paraId="3F2A1B7C" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:r><w:t>x</w:t></w:r></w:p>"#,
    );
    let result = for_edit(&doc, &ForEditRequest::default()).unwrap();
    assert_eq!(result.paragraphs[0].target.short_id, "p0");
    assert_eq!(
        result.paragraphs[0].target.long_id,
        "document.w:p.w14:paraId.3F2A1B7C"
    );
    // Without a usable native key the long id is the j:short form.
    let plain = for_edit(&docx(&para("x")), &ForEditRequest::default()).unwrap();
    assert_eq!(
        plain.paragraphs[0].target.long_id,
        "document.w:p.j:short.p0"
    );
}

#[test]
fn merged_and_removed_targets_are_read_only_context() {
    // Accepting removes p1 with its text: its mark and its content are
    // both deleted, so nothing of it survives settlement.
    let doc = docx(&format!(
        "{}{}{}",
        para("start"),
        r#"<w:p><w:pPr><w:rPr><w:del w:id="8" w:author="Other" w:date="2026-09-01T00:00:00Z"/></w:rPr></w:pPr><w:del w:id="9" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:delText>gone</w:delText></w:r></w:del></w:p>"#,
        para("end")
    ));
    let result = for_edit(
        &doc,
        &ForEditRequest {
            existing_revisions: ExistingRevisions::Accept,
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    let row = &result.paragraphs[1];
    assert_eq!(row.target.short_id, "p1");
    assert_eq!(row.correspondence, Correspondence::Removed);
    assert!(row.working.is_none());
    assert_eq!(status(row, "rewrite"), "unavailable");
    assert_eq!(status(row, "replace"), "unavailable");
    // The neighbours keep their exact correspondence and text.
    assert_eq!(
        result.paragraphs[0].correspondence,
        Correspondence::OneToOne
    );
    assert_eq!(
        result.paragraphs[2]
            .working
            .as_ref()
            .unwrap()
            .value
            .text
            .as_deref(),
        Some("end")
    );
}

#[test]
fn comments_intersecting_the_paragraph_are_listed() {
    let comments = Part {
        name: "word/comments.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        xml: r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="5" w:author="AC" w:date="2026-09-01T00:00:00Z"><w:p><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#,
    };
    let body = r#"<w:p><w:commentRangeStart w:id="5"/><w:r><w:t>covered</w:t></w:r><w:commentRangeEnd w:id="5"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="5"/></w:r></w:p>"#.to_string()
        + &para("plain");
    let result = for_edit(&docx_with(&body, &[comments]), &ForEditRequest::default()).unwrap();
    let row = &result.paragraphs[0];
    assert_eq!(row.comments.len(), 1);
    assert_eq!(row.comments[0].short_id, "c5");
    assert_eq!(row.comments[0].long_id, "comments.w:comment.w:id.5");
    // A comment does not make the paragraph conditional.
    assert_eq!(status(row, "replace"), "available");
}

#[test]
fn changed_selects_blocks_intersecting_revisions_and_comments() {
    let comments = Part {
        name: "word/comments.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        xml: r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="5" w:author="AC" w:date="2026-09-01T00:00:00Z"><w:p><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#,
    };
    let body = para("plain")
        + PENDING
        + r#"<w:p><w:commentRangeStart w:id="5"/><w:r><w:t>noted</w:t></w:r><w:commentRangeEnd w:id="5"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="5"/></w:r></w:p>"#;
    let doc = docx_with(&body, &[comments]);
    let result = for_edit(
        &doc,
        &ForEditRequest {
            changed: true,
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    let ids: Vec<&str> = result
        .paragraphs
        .iter()
        .map(|r| r.target.short_id.as_str())
        .collect();
    assert_eq!(ids, ["p1", "p2"], "--changed uses the original marks");
    let by_other = for_edit(
        &doc,
        &ForEditRequest {
            changed: true,
            by: Some("Other".to_string()),
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    let ids: Vec<&str> = by_other
        .paragraphs
        .iter()
        .map(|r| r.target.short_id.as_str())
        .collect();
    assert_eq!(ids, ["p1"], "--by filters on the mark's author");
}

#[test]
fn decision_flags_do_not_modify_the_source() {
    let doc = source();
    let before = source_sha256(&doc);
    let result = for_edit(
        &doc,
        &ForEditRequest {
            reject: vec!["r7".to_string(), "r8".to_string()],
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    assert_eq!(result.source_sha256, before);
    // The working basis carries the rejection; the source bytes are intact.
    assert_ne!(result.basis.working_sha256, result.source_sha256);
}

fn comments_part(xml: &str) -> Part<'_> {
    Part {
        name: "word/comments.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        xml,
    }
}

fn one_comment(id: u32, attrs: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="{id}" {attrs}><w:p><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#
    )
}

#[test]
fn cross_story_comment_ranges_label_their_endpoint_paragraphs() {
    let comment_xml = one_comment(5, r#"w:author="AC" w:date="2026-09-01T00:00:00Z""#);
    let comments = comments_part(&comment_xml);
    let header = Part {
        name: "word/header1.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        xml: r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>head</w:t></w:r><w:commentRangeEnd w:id="5"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="5"/></w:r></w:p></w:hdr>"#,
    };
    let body = r#"<w:p><w:commentRangeStart w:id="5"/><w:r><w:t>covered</w:t></w:r></w:p>"#;
    let doc = docx_with_sect(
        body,
        &[comments, header],
        r#"<w:headerReference w:type="default" r:id="rIdX1"/>"#,
    );
    let result = for_edit(&doc, &ForEditRequest::default()).unwrap();
    let row_of = |story: &str| {
        result
            .paragraphs
            .iter()
            .find(|r| r.story == story)
            .unwrap_or_else(|| panic!("a row for the {story} story"))
    };
    for row in [row_of("body"), row_of("header1")] {
        assert!(
            row.comments.iter().any(|c| c.short_id == "c5"),
            "the {} endpoint carries the comment its cross-story range anchors",
            row.story
        );
        let value = serde_json::to_value(row).unwrap();
        assert_eq!(
            value["comments"][0]["limitation"], "cross_story_range",
            "the malformed range is labelled, not silent"
        );
    }
    let changed = for_edit(
        &doc,
        &ForEditRequest {
            changed: true,
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    let stories: Vec<&str> = changed
        .paragraphs
        .iter()
        .map(|r| r.story.as_str())
        .collect();
    assert_eq!(
        stories,
        ["body", "header1"],
        "--changed keeps both endpoints"
    );
}

#[test]
fn unanchored_comments_are_inventoried() {
    let comment_xml = one_comment(7, r#"w:author="AC" w:date="2026-09-01T00:00:00Z""#);
    let comments = comments_part(&comment_xml);
    let doc = docx_with(&para("plain"), &[comments]);
    let result = for_edit(&doc, &ForEditRequest::default()).unwrap();
    let value = serde_json::to_value(&result).unwrap();
    let inventory = value["unanchored_comments"]
        .as_array()
        .expect("a document-level inventory of comments with no anchor");
    assert_eq!(inventory.len(), 1);
    assert_eq!(inventory[0]["short_id"], "c7");
    assert_eq!(inventory[0]["long_id"], "comments.w:comment.w:id.7");
    assert!(
        result.paragraphs.iter().all(|r| r.comments.is_empty()),
        "an unanchored comment claims no paragraph"
    );
}

#[test]
fn context_rows_state_their_view_codec_limitations_and_revisions() {
    let doc = docx(&format!(
        "{}{}",
        para("anchor"),
        r#"<w:p><w:ins w:id="9" w:author="AC" w:date="2026-09-01T00:00:00Z"><w:r><w:t>inserted</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> plain</w:t></w:r><w:r><w:sym w:font="Wingdings" w:char="F0E0"/></w:r></w:p>"#
    ));
    let result = for_edit(
        &doc,
        &ForEditRequest {
            find: Some("anchor".to_string()),
            context: 1,
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    assert_eq!(result.source_context.len(), 1);
    let context = &result.source_context[0];
    assert_eq!(context.target.short_id, "p1");
    assert_eq!(
        context.value.text.as_deref(),
        Some("inserted plain\u{FFFC}"),
        "context text is the final view, exactly as the row would read it"
    );
    let value = serde_json::to_value(context).unwrap();
    assert_eq!(value["text_view"], "final", "the projection is labelled");
    assert_eq!(
        value["text_codec"], "unsupported",
        "a U+FFFC neighbour is no false promise of editability"
    );
    let limitations = value["limitations"]
        .as_array()
        .expect("the codec limitations reach the context record");
    assert!(
        limitations.iter().any(|l| l == "sym"),
        "limitations: {limitations:?}"
    );
    let revisions = value["revisions"]
        .as_array()
        .expect("the context row carries its revision marks");
    assert!(
        revisions.iter().any(|r| r["target"]["short_id"] == "r9"
            && r["kind"] == "insertion"
            && r["basis"] == "original"),
        "inserted context text is labelled as a pending insertion"
    );
}

#[test]
fn a_fully_deleted_context_neighbour_carries_its_revision_marks() {
    let doc = docx(&format!(
        "{}{}",
        para("kept"),
        r#"<w:p><w:del w:id="10" w:author="AC" w:date="2026-09-01T00:00:00Z"><w:r><w:delText>gone</w:delText></w:r></w:del></w:p>"#
    ));
    let result = for_edit(
        &doc,
        &ForEditRequest {
            find: Some("kept".to_string()),
            context: 1,
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    assert_eq!(result.source_context.len(), 1);
    let context = &result.source_context[0];
    assert_eq!(context.value.text.as_deref(), Some(""));
    assert!(context.value.complete);
    let value = serde_json::to_value(context).unwrap();
    assert_eq!(value["text_view"], "final");
    let revisions = value["revisions"]
        .as_array()
        .expect("an empty final view still names the deletion that emptied it");
    assert!(
        revisions
            .iter()
            .any(|r| r["target"]["short_id"] == "r10" && r["kind"] == "deletion"),
        "the neighbour's emptiness is explained, not silent"
    );
}

#[test]
fn reversed_comment_endpoints_span_their_paragraphs() {
    let comment_xml = one_comment(5, r#"w:author="AC" w:date="2026-09-01T00:00:00Z""#);
    let comments = comments_part(&comment_xml);
    let body = r#"<w:p><w:r><w:t>AA</w:t></w:r><w:commentRangeStart w:id="5"/><w:r><w:t>BB</w:t></w:r></w:p>"#
        .to_string()
        + &para("middle")
        + r#"<w:p><w:r><w:t>CC</w:t></w:r><w:commentRangeEnd w:id="5"/><w:r><w:t>DD</w:t></w:r><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="5"/></w:r></w:p>"#;
    let doc = docx_with(&body, &[comments]);
    let result = for_edit(&doc, &ForEditRequest::default()).unwrap();
    for id in ["p0", "p1", "p2"] {
        let row = result
            .paragraphs
            .iter()
            .find(|r| r.target.short_id == id)
            .unwrap();
        assert!(
            row.comments.iter().any(|c| c.short_id == "c5"),
            "{id} carries the comment its reversed range spans"
        );
    }
    let changed = for_edit(
        &doc,
        &ForEditRequest {
            changed: true,
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    let ids: Vec<&str> = changed
        .paragraphs
        .iter()
        .map(|r| r.target.short_id.as_str())
        .collect();
    assert_eq!(ids, ["p0", "p1", "p2"]);
}

#[test]
fn filters_narrow_and_empty_results_name_their_filters() {
    let doc = docx(&(para("alpha foo") + &para("beta bar") + &para("gamma foo")));
    // The filters intersect: a paragraph is selected only when every
    // filter given matches it.
    let both = for_edit(
        &doc,
        &ForEditRequest {
            select: Some("p0-p2".to_string()),
            find: Some("foo".to_string()),
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    let ids: Vec<&str> = both
        .paragraphs
        .iter()
        .map(|r| r.target.short_id.as_str())
        .collect();
    assert_eq!(ids, ["p0", "p2"], "select and find narrow together");
    let empty = for_edit(
        &doc,
        &ForEditRequest {
            select: Some("p1".to_string()),
            find: Some("foo".to_string()),
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    assert!(empty.paragraphs.is_empty());
    assert_eq!(empty.coverage.paragraphs_selected, 0);
    assert!(
        empty
            .coverage
            .omissions
            .iter()
            .any(|o| o.contains("ids") && o.contains("find")),
        "an empty result names the filters applied: {:?}",
        empty.coverage.omissions
    );
}

#[test]
fn decision_order_does_not_change_the_cursor_basis() {
    let doc = source();
    let first = for_edit(
        &doc,
        &ForEditRequest {
            accept: vec!["r7".to_string(), "r8".to_string()],
            limit: Some(1),
            ..ForEditRequest::default()
        },
    )
    .unwrap();
    let cursor = first.next_cursor.clone().expect("page one continues");
    let second = for_edit(
        &doc,
        &ForEditRequest {
            accept: vec!["r8".to_string(), "r7".to_string()],
            limit: Some(1),
            cursor: Some(cursor),
            ..ForEditRequest::default()
        },
    )
    .expect("decisions are a set: their order is no part of the basis");
    assert_eq!(second.basis.digest, first.basis.digest);
    assert_eq!(second.coverage.paragraphs_returned, 1);
}

fn v2_plan(bytes: &[u8]) -> Value {
    json!({"schema_version":2, "source":{"sha256":source_sha256(bytes)},
        "policy":{"track":"on","existing_revisions":"keep","text_view":"final"},
        "resolve_revisions":{"accept":[],"reject":[]}, "author":"Me","date":"2026-10-17T00:00:00Z",
        "scope":{"kind":"all"},"operations":[]})
}

#[test]
fn accept_keeps_the_source_intact_and_agrees_with_apply() {
    use jubarte::edit::v2::PlanV2;
    use jubarte::edit::v2_apply::apply_plan;
    let doc = source();
    let before = source_sha256(&doc);
    for (accept, ids, text) in [
        (
            true,
            json!([
                {"short_id":"r7","long_id":"document.w:ins.w:id.7"},
                {"short_id":"r8","long_id":"document.w:del.w:id.8"}
            ]),
            "Payment is due within 45 days. Late interest is 2%.",
        ),
        (
            false,
            json!([
                {"short_id":"r7","long_id":"document.w:ins.w:id.7"},
                {"short_id":"r8","long_id":"document.w:del.w:id.8"}
            ]),
            "Payment is due within 30 days. Late interest is 2%.",
        ),
    ] {
        let request = if accept {
            ForEditRequest {
                accept: vec!["r7".to_string(), "r8".to_string()],
                ..ForEditRequest::default()
            }
        } else {
            ForEditRequest {
                reject: vec!["r7".to_string(), "r8".to_string()],
                ..ForEditRequest::default()
            }
        };
        let result = for_edit(&doc, &request).unwrap();
        assert_eq!(
            result.source_sha256, before,
            "decision flags never modify the source file"
        );
        assert_ne!(result.basis.working_sha256, before);
        let working = result.paragraphs[0]
            .working
            .as_ref()
            .expect("a working view");
        assert_eq!(working.value.text.as_deref(), Some(text));
        // The apply half: a rewrite bound to for-edit's before_sha256
        // applies under the same decisions (read hashes are apply hashes).
        let mut plan = v2_plan(&doc);
        plan["resolve_revisions"][if accept { "accept" } else { "reject" }] = ids;
        plan["operations"] = json!([{
            "kind": "rewrite",
            "target": {"short_id": "p0", "long_id": "document.w:p.j:short.p0"},
            "before_sha256": working.before_sha256.clone().expect("a supported codec"),
            "text": text
        }]);
        apply_plan(&doc, &PlanV2::from_json(&plan.to_string()).unwrap())
            .unwrap_or_else(|e| panic!("apply must accept for-edit's working basis: {e}"));
    }
}

#[test]
fn by_matches_authors_and_initials_exactly() {
    let comment_xml = one_comment(
        5,
        r#"w:author="Alice Cooper" w:initials="AC" w:date="2026-09-01T00:00:00Z""#,
    );
    let comments = comments_part(&comment_xml);
    let body = r#"<w:p><w:ins w:id="9" w:author="Alice Cooper" w:date="2026-09-01T00:00:00Z"><w:r><w:t>change</w:t></w:r></w:ins></w:p>"#
        .to_string()
        + r#"<w:p><w:commentRangeStart w:id="5"/><w:r><w:t>noted</w:t></w:r><w:commentRangeEnd w:id="5"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="5"/></w:r></w:p>"#;
    let doc = docx_with(&body, &[comments]);
    let ids = |by: &str| -> Vec<String> {
        for_edit(
            &doc,
            &ForEditRequest {
                changed: true,
                by: Some(by.to_string()),
                ..ForEditRequest::default()
            },
        )
        .unwrap()
        .paragraphs
        .iter()
        .map(|r| r.target.short_id.clone())
        .collect()
    };
    assert_eq!(
        ids("Alice Cooper"),
        ["p0", "p1"],
        "full names match both inventories exactly"
    );
    assert_eq!(
        ids("AC"),
        ["p1"],
        "a handle matches a comment's explicit w:initials, never initials derived from a name"
    );
}

// V2-12 review rows 2–3: `read --for-edit` keeps the selection grammar its
// shipped help documents (open-ended `p17-` / `-p3` spellings, transitionally)
// and refuses a selector past the end as `USAGE`, not `ANCHOR_NOT_FOUND`.
mod selection_grammar {
    use super::*;
    use serde_json::Value;

    fn three() -> Vec<u8> {
        docx(&(para("One") + &para("Two") + &para("Three")))
    }

    fn selected(spec: &str) -> Result<Vec<String>, jubarte::edit::EditError> {
        for_edit(
            &three(),
            &ForEditRequest {
                select: Some(spec.to_string()),
                ..ForEditRequest::default()
            },
        )
        .map(|view| {
            view.paragraphs
                .into_iter()
                .map(|row| row.target.short_id)
                .collect()
        })
    }

    #[test]
    fn open_ended_spellings_still_parse() {
        assert_eq!(selected("p1-").unwrap(), ["p1", "p2"]);
        assert_eq!(selected("-p1").unwrap(), ["p0", "p1"]);
        assert_eq!(selected("p0-p2").unwrap(), ["p0", "p1", "p2"]);
        assert_eq!(selected("p1,p2").unwrap(), ["p1", "p2"]);
    }

    #[test]
    fn a_selector_past_the_end_is_usage_not_a_missing_anchor() {
        for spec in ["p3", "p1-p3", "p3-", "-p3"] {
            let error = selected(spec).unwrap_err();
            assert_eq!(error.code, "USAGE", "{spec}: {error}");
            assert!(
                error.message.contains("past the last paragraph"),
                "{spec}: {error}"
            );
        }
    }

    #[test]
    fn the_cli_still_accepts_the_open_ended_spellings_its_help_documents() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("three.docx");
        std::fs::write(&input, three()).unwrap();
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(["read", input.to_str().unwrap(), "--for-edit", "-p", "p1-"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "stdout={} stderr={}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let view: Value = serde_json::from_slice(&out.stdout).unwrap();
        let ids: Vec<&str> = view["paragraphs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["target"]["short_id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, ["p1", "p2"]);
    }
}

mod cursor_basis {
    use super::*;

    fn document() -> Vec<u8> {
        docx(&format!(
            "{}{}{}",
            PENDING,
            PENDING
                .replace("w:id=\"8\"", "w:id=\"10\"")
                .replace("w:id=\"7\"", "w:id=\"9\""),
            para("Governing law: Delaware.")
        ))
    }

    fn request() -> ForEditRequest {
        ForEditRequest {
            select: Some("p0-p1".to_string()),
            limit: Some(1),
            ..ForEditRequest::default()
        }
    }

    fn refuses_changed_basis(original: &ForEditRequest, changed: &ForEditRequest) {
        let doc = document();
        let first = for_edit(&doc, original).unwrap();
        let cursor = first.next_cursor.expect("the first page continues");
        let same = for_edit(
            &doc,
            &ForEditRequest {
                cursor: Some(cursor.clone()),
                ..original.clone()
            },
        )
        .expect("an unchanged basis continues");
        assert_eq!(same.paragraphs[0].target.short_id, "p1");
        let error = for_edit(
            &doc,
            &ForEditRequest {
                cursor: Some(cursor),
                ..changed.clone()
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "CURSOR_BASIS_CHANGED");
    }

    #[test]
    fn accept_decisions_bind_the_cursor() {
        let original = request();
        let mut changed = original.clone();
        changed.accept = vec!["r7".to_string()];
        refuses_changed_basis(&original, &changed);
    }

    #[test]
    fn reject_decisions_bind_the_cursor() {
        let original = request();
        let mut changed = original.clone();
        changed.reject = vec!["r7".to_string()];
        refuses_changed_basis(&original, &changed);
    }

    #[test]
    fn existing_revision_policy_binds_the_cursor() {
        let original = request();
        for policy in [ExistingRevisions::Accept, ExistingRevisions::Reject] {
            let mut changed = original.clone();
            changed.existing_revisions = policy;
            refuses_changed_basis(&original, &changed);
        }
    }

    #[test]
    fn selection_binds_the_cursor() {
        let original = request();
        let mut changed = original.clone();
        changed.select = Some("p1-p2".to_string());
        refuses_changed_basis(&original, &changed);
    }

    fn same_membership(original: &ForEditRequest, changed: &ForEditRequest) {
        let doc = document();
        let ids = |request: &ForEditRequest| {
            for_edit(
                &doc,
                &ForEditRequest {
                    all: true,
                    ..request.clone()
                },
            )
            .unwrap()
            .paragraphs
            .into_iter()
            .map(|row| row.target.short_id)
            .collect::<Vec<_>>()
        };
        assert_eq!(
            ids(original),
            ids(changed),
            "only the filter changes, not membership"
        );
        refuses_changed_basis(original, changed);
    }

    macro_rules! filter_test {
        ($name:ident, $field:ident, $value:expr $(, $base_field:ident, $base_value:expr)?) => {
            #[test]
            fn $name() {
                let original = ForEditRequest {
                    $($base_field: $base_value,)?
                    ..request()
                };
                let changed = ForEditRequest { $field: $value, ..original.clone() };
                same_membership(&original, &changed);
            }
        };
    }

    filter_test!(
        selector_spelling_binds_the_cursor,
        select,
        Some("p0,p1".to_string())
    );
    filter_test!(story_binds_the_cursor, story, Some("body".to_string()));
    filter_test!(head_binds_the_cursor, head, Some(2));
    filter_test!(tail_binds_the_cursor, tail, Some(3));
    filter_test!(find_binds_the_cursor, find, Some("Payment".to_string()));
    filter_test!(regex_binds_the_cursor, regex, Some("Payment".to_string()));
    filter_test!(
        ignore_case_binds_the_cursor,
        ignore_case,
        true,
        find,
        Some("Payment".to_string())
    );
    filter_test!(changed_binds_the_cursor, changed, true);
    filter_test!(
        by_binds_the_cursor,
        by,
        Some("Other".to_string()),
        changed,
        true
    );
    filter_test!(
        by_any_binds_the_cursor,
        by_any,
        vec!["Other".to_string()],
        changed,
        true
    );
    filter_test!(context_binds_the_cursor, context, 1);
}

mod cli_policy {
    use super::*;
    use std::process::{Command, Output};

    fn read(input: &std::path::Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .arg("read")
            .arg(input)
            .arg("--for-edit")
            .args(args)
            .output()
            .unwrap()
    }

    fn view(output: &Output) -> Value {
        assert!(
            output.status.success(),
            "stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }

    fn with_source(test: impl FnOnce(&std::path::Path, &[u8])) {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("pending.docx");
        let bytes = source();
        std::fs::write(&input, &bytes).unwrap();
        test(&input, &bytes);
        assert_eq!(
            std::fs::read(&input).unwrap(),
            bytes,
            "read leaves the source intact"
        );
    }

    fn agrees_with_library(actual: &Value, bytes: &[u8], request: &ForEditRequest) {
        assert_eq!(
            *actual,
            serde_json::to_value(for_edit(bytes, request).unwrap()).unwrap()
        );
    }

    fn settled_ids<'a>(view: &'a Value, decision: &str) -> Vec<&'a str> {
        let mut ids: Vec<&str> = view["settled"][decision]
            .as_array()
            .into_iter()
            .flatten()
            .map(|id| id.as_str().unwrap())
            .collect();
        ids.sort_unstable();
        ids
    }

    #[test]
    fn track_flag_reports_the_declared_basis_with_its_rows() {
        with_source(|input, bytes| {
            for (mode, track) in [("on", true), ("off", false)] {
                let actual = view(&read(input, &["--track", mode]));
                assert_eq!(actual["basis"]["track"], mode);
                assert_eq!(
                    actual["paragraphs"][0]["working"]["text"],
                    "Payment is due within 45 days. Late interest is 2%."
                );
                assert_eq!(
                    actual["paragraphs"][0]["blockers"][0]["kind"],
                    "tracked_insertion"
                );
                agrees_with_library(
                    &actual,
                    bytes,
                    &ForEditRequest {
                        track,
                        ..ForEditRequest::default()
                    },
                );
            }
        });
    }

    #[test]
    fn existing_revision_flag_settles_the_returned_rows() {
        with_source(|input, bytes| {
            for (mode, policy, number, revisions) in [
                ("keep", ExistingRevisions::Keep, "45", 2),
                ("accept", ExistingRevisions::Accept, "45", 0),
                ("reject", ExistingRevisions::Reject, "30", 0),
            ] {
                let actual = view(&read(input, &["--existing-revisions", mode]));
                assert_eq!(actual["basis"]["existing_revisions"], mode);
                let row = &actual["paragraphs"][0];
                assert_eq!(
                    row["working"]["text"],
                    format!("Payment is due within {number} days. Late interest is 2%.")
                );
                assert_eq!(row["revisions"].as_array().map_or(0, Vec::len), revisions);
                agrees_with_library(
                    &actual,
                    bytes,
                    &ForEditRequest {
                        existing_revisions: policy,
                        ..ForEditRequest::default()
                    },
                );
            }
            let refused = read(input, &["--existing-revisions", "refuse"]);
            assert_eq!(refused.status.code(), Some(3));
            assert!(refused.stdout.is_empty());
            assert!(String::from_utf8_lossy(&refused.stderr).contains("EXISTING_REVISIONS"));
        });
    }

    #[test]
    fn repeated_accept_change_flags_keep_every_decision() {
        with_source(|input, bytes| {
            let actual = view(&read(
                input,
                &["--accept-change", "r7", "--accept-change", "r8"],
            ));
            assert_eq!(settled_ids(&actual, "accepted"), ["r7", "r8"]);
            let row = &actual["paragraphs"][0];
            assert_eq!(
                row["working"]["text"],
                "Payment is due within 45 days. Late interest is 2%."
            );
            assert!(row["blockers"].as_array().is_none_or(Vec::is_empty));
            assert!(row["revisions"].as_array().is_none_or(Vec::is_empty));
            agrees_with_library(
                &actual,
                bytes,
                &ForEditRequest {
                    accept: vec!["r7".to_string(), "r8".to_string()],
                    ..ForEditRequest::default()
                },
            );
        });
    }

    #[test]
    fn repeated_reject_change_flags_keep_every_decision() {
        with_source(|input, bytes| {
            let actual = view(&read(
                input,
                &["--reject-change", "r7", "--reject-change", "r8"],
            ));
            assert_eq!(settled_ids(&actual, "rejected"), ["r7", "r8"]);
            let row = &actual["paragraphs"][0];
            assert_eq!(
                row["working"]["text"],
                "Payment is due within 30 days. Late interest is 2%."
            );
            assert!(row["blockers"].as_array().is_none_or(Vec::is_empty));
            agrees_with_library(
                &actual,
                bytes,
                &ForEditRequest {
                    reject: vec!["r7".to_string(), "r8".to_string()],
                    ..ForEditRequest::default()
                },
            );
        });
    }

    #[test]
    fn stale_cursor_refuses_changed_decisions_and_policies() {
        with_source(|input, _| {
            let first = view(&read(input, &["--limit", "1"]));
            let cursor = first["next_cursor"].as_str().expect("page one continues");
            let same = view(&read(input, &["--limit", "1", "--cursor", cursor]));
            assert_eq!(same["paragraphs"][0]["target"]["short_id"], "p1");
            for flags in [
                ["--accept-change", "r7"],
                ["--reject-change", "r7"],
                ["--existing-revisions", "accept"],
                ["--existing-revisions", "reject"],
                ["--track", "off"],
            ] {
                let refused = read(
                    input,
                    &["--limit", "1", "--cursor", cursor, flags[0], flags[1]],
                );
                assert_eq!(refused.status.code(), Some(3), "{flags:?}");
                assert!(refused.stdout.is_empty(), "{flags:?}");
                assert!(
                    String::from_utf8_lossy(&refused.stderr).contains("CURSOR_BASIS_CHANGED"),
                    "{flags:?}"
                );
            }
        });
    }
}
