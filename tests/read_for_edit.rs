// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `read --for-edit` (V2-10): exact working hashes, paired target IDs and
//! range-specific readiness on the working document template/apply derive.

mod common;

use common::docx::{Part, docx, docx_with, para};
use jubarte::edit::ExistingRevisions;
use jubarte::edit::for_edit::{Correspondence, ForEditRequest, for_edit};
use jubarte::inspect::source_sha256;

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
    assert!(
        row.blockers
            .iter()
            .all(|b| !(b.range[0] < 23 && 23 < b.range[1]) || b.range == ins.range),
        "nothing else blocks the span around 'is 2%'"
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
