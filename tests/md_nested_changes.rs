// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! A change inside a change (`w:ins` holding a `w:del`, and back): the view
//! splits the outer change around the inner one, each piece keeping the
//! outer change's id, author and date, so the writer nests the marks again
//! and both directions are fixed points (issue #517).

mod common;

use common::docx::{docx, part_string};
use common::validity::check_word_valid_package;
use jubarte::markdown::{
    DocxOptions, MarkdownOptions, ReadOptions, docx_to_markdown, markdown_to_docx, read,
};

/// `w:ins` by Ana holding a `w:del` by Ben between two runs.
fn del_in_ins() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:r><w:t xml:space="preserve">one </w:t></w:r>\
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:delText xml:space="preserve">two</w:delText></w:r></w:del>\
           <w:r><w:t xml:space="preserve"> three</w:t></w:r></w:ins></w:p>"#,
    )
}

/// `w:del` by Ben holding a `w:ins` by Ana between two runs.
fn ins_in_del() -> Vec<u8> {
    docx(
        r#"<w:p><w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:delText xml:space="preserve">one </w:delText></w:r>\
           <w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:r><w:t xml:space="preserve">two</w:t></w:r></w:ins>\
           <w:r><w:delText xml:space="preserve"> three</w:delText></w:r></w:del></w:p>"#,
    )
}

/// `w:ins` by Ana whose only content is a `w:del` by Ben: what Word
/// writes when one author inserts text and another deletes it.
fn sole_del_in_ins() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:delText xml:space="preserve">two</w:delText></w:r></w:del></w:ins></w:p>"#,
    )
}

/// `w:del` by Ben whose only content is a `w:ins` by Ana.
fn sole_ins_in_del() -> Vec<u8> {
    docx(
        r#"<w:p><w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:r><w:t xml:space="preserve">two</w:t></w:r></w:ins></w:del></w:p>"#,
    )
}

/// `w:ins` by Ana whose only content is a `w:moveFrom` by Ben: the same
/// shape through the move's deletion projection.
fn sole_move_from_in_ins() -> Vec<u8> {
    docx(
        r#"<w:p><w:moveFromRangeStart w:id="7" w:name="move"/>\
           <w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:moveFrom w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:delText xml:space="preserve">two</w:delText></w:r></w:moveFrom></w:ins>\
           <w:moveFromRangeEnd w:id="7"/></w:p>"#,
    )
}

/// An insertion right before a sole nest: the folded change must not
/// pair with it as a substitution.
fn ins_then_sole_nest() -> Vec<u8> {
    docx(
        r#"<w:p><w:r><w:t xml:space="preserve">pre </w:t></w:r>\
           <w:ins w:id="6" w:author="Cy Dee" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:t xml:space="preserve">one</w:t></w:r></w:ins>\
           <w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:delText xml:space="preserve">a</w:delText></w:r></w:del></w:ins></w:p>"#,
    )
}

/// Two sole nests side by side, each keeping its own outer change.
fn two_sole_nests() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:delText xml:space="preserve">a</w:delText></w:r></w:del></w:ins>\
           <w:ins w:id="2" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:del w:id="4" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:delText xml:space="preserve">b</w:delText></w:r></w:del></w:ins></w:p>"#,
    )
}

/// The review's split-piece comment probe (#547 defect 1): Word's own
/// shape, the comment on the inner deletion.
fn split_with_comment() -> Vec<u8> {
    with_comment(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:r><w:t>one </w:t></w:r><w:commentRangeStart w:id="7"/>
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
           <w:r><w:delText>two</w:delText></w:r></w:del><w:commentRangeEnd w:id="7"/>
           <w:r><w:commentReference w:id="7"/></w:r>
           <w:r><w:t xml:space="preserve"> three</w:t></w:r></w:ins></w:p>"#,
    )
}

/// The review's sole-nest comment probe (#547 defect 2): the comment's
/// range and reference sit inside the outer, around the sole `w:del`.
fn sole_nest_with_comment() -> Vec<u8> {
    with_comment(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:commentRangeStart w:id="7"/>
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
           <w:r><w:delText>two</w:delText></w:r></w:del><w:commentRangeEnd w:id="7"/>
           <w:r><w:commentReference w:id="7"/></w:r></w:ins></w:p>"#,
    )
}

/// Two inner changes with no outer text of its own (#547 defect 3).
fn two_inner_no_text() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
           <w:r><w:delText>a</w:delText></w:r></w:del>
           <w:del w:id="4" w:author="Cy Dee" w:date="2026-09-30T10:00:00Z">
           <w:r><w:delText>b</w:delText></w:r></w:del></w:ins></w:p>"#,
    )
}

/// Two inner changes with outer text between them (#547 defect 4).
fn two_inner_with_text() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:r><w:t xml:space="preserve">pre </w:t></w:r>
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
           <w:r><w:delText>a</w:delText></w:r></w:del>
           <w:r><w:t xml:space="preserve"> mid </w:t></w:r>
           <w:del w:id="4" w:author="Cy Dee" w:date="2026-09-30T10:00:00Z">
           <w:r><w:delText>b</w:delText></w:r></w:del>
           <w:r><w:t> post</w:t></w:r></w:ins></w:p>"#,
    )
}

/// The outer `w:ins` continued into the next paragraph under the same
/// `w:id` (#547 defect 5): how Word writes one revision across paragraphs.
fn outer_into_next_paragraph() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:r><w:t>one</w:t></w:r></w:ins></w:p>
           <w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
           <w:r><w:delText>two</w:delText></w:r></w:del></w:ins></w:p>"#,
    )
}

/// A cell's own `w:tcPr/w:ins` around a sole nest (#547 defect 6).
fn carried_cell(outer: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:tbl><w:tr>{outer}<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
           <w:r><w:delText>two</w:delText></w:r></w:del></w:ins></w:p>
           </w:tc></w:tr></w:tbl>"#
    ))
}

/// The outer's author holds parentheses (#547 defect 7).
fn paren_author_outer() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima (consultant)" w:date="2026-09-29T14:05:00Z">
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
           <w:r><w:delText>two</w:delText></w:r></w:del></w:ins></w:p>"#,
    )
}

/// The inner's author holds ` on rN ` (#547 defect 8).
fn separator_author_inner() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:del w:id="3" w:author="Bo on r1 Kim" w:date="2026-09-30T09:00:00Z">
           <w:r><w:delText>two</w:delText></w:r></w:del></w:ins></w:p>"#,
    )
}

/// A three-level alternating nest (#547 defect 10).
fn three_levels() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
           <w:ins w:id="5" w:author="Cy Dee" w:date="2026-09-30T11:00:00Z">
           <w:r><w:t>two</w:t></w:r></w:ins></w:del></w:ins></w:p>"#,
    )
}

/// A docx with Zoe Stone's comment 7 ("Check.") beside `body`.
fn with_comment(body: &str) -> Vec<u8> {
    use common::docx::{R_NS, W_NS};
    const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="{W_NS}" xmlns:r="{R_NS}"><w:comment w:id="7" w:author="Zoe Stone" w:date="2026-10-01T10:00:00Z" w:initials="ZS"><w:p><w:r><w:t>Check.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let parts = [common::docx::Part {
        name: "word/comments.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        rel_type: &format!("{RELS}/comments"),
        xml: &comments,
    }];
    common::docx::docx_with(body, &parts)
}

/// The plain view (`--commonmark`) of `docx`.
fn plain(docx: &[u8]) -> String {
    docx_to_markdown(docx, &MarkdownOptions::default())
        .unwrap()
        .markdown
}

/// The agent view (`jubarte read`) of `docx`, without page markers.
fn agent(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .unwrap()
    .markdown
}

/// Every change resolved the way Word's Accept All (`true`) or Reject
/// All (`false`) resolves them, as the plain text that remains.
fn resolved(docx: &[u8], accept: bool) -> String {
    use jubarte::changes::ChangeFilter;
    let filter = ChangeFilter::default();
    let out = if accept {
        jubarte::changes::accept_changes(docx, &filter).unwrap()
    } else {
        jubarte::changes::reject_changes(docx, &filter).unwrap()
    };
    plain(&out)
}

/// `markdown` written to Word and read back, each way.
fn written(markdown: &str) -> Vec<u8> {
    markdown_to_docx(markdown, &DocxOptions::default())
        .unwrap()
        .docx
}

#[test]
fn a_deletion_that_is_an_insertions_only_content_names_it_in_the_note() {
    assert_eq!(
        plain(&sole_del_in_ins()),
        "{--two--}{>>r3 Ben Ode (2026-09-30T09:00:00Z) \
         on r1 Ana Lima (2026-09-29T14:05:00Z)<<}\n"
    );
    let view = agent(&sole_del_in_ins());
    assert!(view.contains("{--two--}{>>r3 @BO on r1 @AL<<}"), "{view}");
}

#[test]
fn an_insertion_that_is_a_deletions_only_content_names_it_in_the_note() {
    assert_eq!(
        plain(&sole_ins_in_del()),
        "{++two++}{>>r1 Ana Lima (2026-09-29T14:05:00Z) \
         on r3 Ben Ode (2026-09-30T09:00:00Z)<<}\n"
    );
    let view = agent(&sole_ins_in_del());
    assert!(view.contains("{++two++}{>>r1 @AL on r3 @BO<<}"), "{view}");
}

#[test]
fn a_move_from_that_is_an_insertions_only_content_names_it_in_the_note() {
    assert_eq!(
        plain(&sole_move_from_in_ins()),
        "{--two--}{>>r3 Ben Ode (2026-09-30T09:00:00Z) \
         on r1 Ana Lima (2026-09-29T14:05:00Z)<<}\n"
    );
}

#[test]
fn a_sole_deletion_writes_back_inside_its_insertion() {
    let document = part_string(&written(&plain(&sole_del_in_ins())), "word/document.xml").unwrap();
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    let ins = document.split("<w:ins ").nth(1).unwrap();
    assert!(
        ins.starts_with(r#"w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z""#),
        "{document}"
    );
    // The deletion sits inside the insertion, and is the only thing in it.
    assert!(
        ins.starts_with(concat!(
            r#"w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">"#,
            r#"<w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">"#
        )),
        "{document}"
    );
    assert!(
        ins.contains(r#"<w:delText xml:space="preserve">two</w:delText></w:r></w:del></w:ins>"#),
        "{document}"
    );
    // The note was an attribution: no comment, no literal delimiters, and
    // no empty mark the ids left behind.
    assert!(!document.contains("commentReference"), "{document}");
    assert!(!document.contains("{--"), "{document}");
    for name in ["<w:ins ", "<w:del "] {
        for (at, _) in document.match_indices(name) {
            let tag = &document[at..];
            let end = tag.find('>').unwrap();
            assert!(!tag[..end].ends_with('/'), "{document}");
        }
    }
}

#[test]
fn a_sole_insertion_writes_back_inside_its_deletion() {
    let document = part_string(&written(&plain(&sole_ins_in_del())), "word/document.xml").unwrap();
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    let del = document.split("<w:del ").nth(1).unwrap();
    assert!(
        del.starts_with(concat!(
            r#"w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">"#,
            r#"<w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">"#
        )),
        "{document}"
    );
    // The inner text is live: rejecting the deletion removes it, accepting
    // the insertion keeps it.
    assert!(
        del.contains(r#"<w:t xml:space="preserve">two</w:t></w:r></w:ins></w:del>"#),
        "{document}"
    );
}

#[test]
fn the_agent_view_writes_the_nested_marks_back_too() {
    for source in [sole_del_in_ins(), sole_ins_in_del()] {
        let document = part_string(&written(&agent(&source)), "word/document.xml").unwrap();
        assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
        assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
        assert!(!document.contains("commentReference"), "{document}");
    }
}

#[test]
fn sole_nested_changes_round_trip_in_both_views() {
    for source in [
        sole_del_in_ins(),
        sole_ins_in_del(),
        sole_move_from_in_ins(),
    ] {
        let markdown = plain(&source);
        assert_eq!(plain(&written(&markdown)), markdown, "{markdown}");
        let docx = written(&markdown);
        assert!(
            check_word_valid_package(&docx).ok(),
            "{markdown}: {}",
            check_word_valid_package(&docx).errors.join("\n")
        );
    }
}

#[test]
fn resolving_sole_nested_changes_matches_the_source() {
    for source in [sole_del_in_ins(), sole_ins_in_del()] {
        let round = written(&plain(&source));
        // Rejecting the insertion removes what it holds, as it does in the
        // source; accepting the deletion removes the text either way.
        assert_eq!(
            resolved(&round, false),
            resolved(&source, false),
            "reject: {}",
            plain(&source)
        );
        assert_eq!(
            resolved(&round, true),
            resolved(&source, true),
            "accept: {}",
            plain(&source)
        );
    }
}

#[test]
fn a_deletion_inside_an_insertion_splits_the_insertion() {
    assert_eq!(
        plain(&del_in_ins()),
        "{++one ++}{>>r1 Ana Lima (2026-09-29T14:05:00Z)<<}\
         {--two--}{>>r3 Ben Ode (2026-09-30T09:00:00Z)<<}\
         {++ three++}{>>r1 Ana Lima (2026-09-29T14:05:00Z)<<}\n"
    );
    let view = agent(&del_in_ins());
    assert!(
        view.contains("{++one ++}{>>r1 @AL<<}{--two--}{>>r3 @BO<<}{++ three++}{>>r1 @AL<<}"),
        "{view}"
    );
}

#[test]
fn an_insertion_inside_a_deletion_splits_the_deletion() {
    assert_eq!(
        plain(&ins_in_del()),
        "{--one --}{>>r3 Ben Ode (2026-09-30T09:00:00Z)<<}\
         {++two++}{>>r1 Ana Lima (2026-09-29T14:05:00Z)<<}\
         {-- three--}{>>r3 Ben Ode (2026-09-30T09:00:00Z)<<}\n"
    );
}

#[test]
fn the_split_writes_the_nested_marks_back() {
    let document = part_string(&written(&plain(&del_in_ins())), "word/document.xml").unwrap();
    // One `w:ins`, one `w:del` inside it, each with its id, author and date.
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    let ins = document.split("<w:ins ").nth(1).unwrap();
    assert!(
        ins.starts_with(r#"w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z""#),
        "{document}"
    );
    assert!(
        ins.contains(r#"<w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">"#),
        "{document}"
    );
    // The note was an attribution: no comment and no literal delimiters.
    assert!(!document.contains("commentReference"), "{document}");
    assert!(!document.contains("{--"), "{document}");
}

#[test]
fn a_deleted_insertion_writes_the_nested_marks_back() {
    let document = part_string(&written(&plain(&ins_in_del())), "word/document.xml").unwrap();
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    let del = document.split("<w:del ").nth(1).unwrap();
    assert!(
        del.starts_with(r#"w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z""#),
        "{document}"
    );
    assert!(
        del.contains(r#"<w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">"#),
        "{document}"
    );
}

#[test]
fn nested_changes_round_trip_in_both_views() {
    for source in [del_in_ins(), ins_in_del()] {
        let markdown = plain(&source);
        assert_eq!(plain(&written(&markdown)), markdown, "{markdown}");
        let docx = written(&markdown);
        assert!(
            check_word_valid_package(&docx).ok(),
            "{markdown}: {}",
            check_word_valid_package(&docx).errors.join("\n")
        );
    }
}

#[test]
fn a_folded_change_never_pairs_as_a_substitution() {
    // The deletion names the insertion it sits inside, so it is no
    // substitution's old side: the pair would lose the nest.
    let markdown = plain(&ins_then_sole_nest());
    assert_eq!(
        markdown,
        "pre {++one++}{>>r6 Cy Dee (2026-09-30T09:00:00Z)<<}\
         {--a--}{>>r3 Ben Ode (2026-09-30T09:00:00Z) \
         on r1 Ana Lima (2026-09-29T14:05:00Z)<<}\n"
    );
    assert_eq!(plain(&written(&markdown)), markdown, "{markdown}");
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert!(
        document.contains(concat!(
            r#"<w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">"#,
            r#"<w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">"#
        )),
        "{document}"
    );
}

#[test]
fn two_sole_nests_side_by_side_keep_their_own_changes() {
    let markdown = plain(&two_sole_nests());
    assert_eq!(
        markdown,
        "{--a--}{>>r3 Ben Ode (2026-09-30T09:00:00Z) \
         on r1 Ana Lima (2026-09-29T14:05:00Z)<<}\
         {--b--}{>>r4 Ben Ode (2026-09-30T09:00:00Z) \
         on r2 Ana Lima (2026-09-29T14:05:00Z)<<}\n"
    );
    assert_eq!(plain(&written(&markdown)), markdown, "{markdown}");
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert_eq!(document.matches("<w:ins ").count(), 2, "{document}");
    assert_eq!(document.matches("<w:del ").count(), 2, "{document}");
    assert!(document.contains(r#"<w:ins w:id="2""#), "{document}");
    assert!(document.contains(r#"<w:del w:id="4""#), "{document}");
}

#[test]
fn a_note_naming_two_outers_keeps_the_file_word_valid() {
    // Word refuses a revision nested in its own kind (`ins > del > ins`),
    // so a longer chain rebuilds only the first outer it names.
    let markdown = "{++two++}{>>r5 @CD on r3 @BO on r1 @AL<<}\n";
    let docx = written(markdown);
    assert!(
        check_word_valid_package(&docx).ok(),
        "{}",
        check_word_valid_package(&docx).errors.join("\n")
    );
    let document = part_string(&docx, "word/document.xml").unwrap();
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    assert!(
        document.contains(r#"<w:del w:id="3" w:author="BO""#)
            && document.contains(r#"<w:ins w:id="5" w:author="CD""#),
        "{document}"
    );
}

#[test]
fn nested_changes_keep_both_authors_and_dates() {
    use jubarte::changes::{ChangeKind, list_changes};
    for (source, kinds) in [
        (del_in_ins(), [ChangeKind::Insertion, ChangeKind::Deletion]),
        (ins_in_del(), [ChangeKind::Deletion, ChangeKind::Insertion]),
        (
            sole_del_in_ins(),
            [ChangeKind::Insertion, ChangeKind::Deletion],
        ),
        (
            sole_ins_in_del(),
            [ChangeKind::Deletion, ChangeKind::Insertion],
        ),
    ] {
        let changes = list_changes(&written(&plain(&source))).unwrap();
        assert_eq!(
            changes.iter().map(|c| c.kind).collect::<Vec<_>>(),
            kinds,
            "{changes:?}"
        );
        let by_author = |name: &str| {
            changes
                .iter()
                .find(|c| c.author.as_deref() == Some(name))
                .unwrap_or_else(|| panic!("no change by {name}: {changes:?}"))
                .date
                .clone()
        };
        assert_eq!(
            by_author("Ana Lima").as_deref(),
            Some("2026-09-29T14:05:00Z")
        );
        assert_eq!(
            by_author("Ben Ode").as_deref(),
            Some("2026-09-30T09:00:00Z")
        );
    }
}

/// The body of an agent view, its front matter gone, a comment's number
/// normalised and the writer's own empty trailing paragraphs dropped: the
/// front matter names the bytes, a comment's number and the paragraph a
/// story that ends in a table gets are the writer's, so none of them
/// survives a round trip to compare.
fn body(view: &str) -> String {
    let body = view.split("\n---\n").nth(1).unwrap_or(view).to_string();
    let ended = body.ends_with('\n');
    let mut body = body
        .lines()
        .filter(|line| {
            !line.trim_start_matches("<!-- ").starts_with("p")
                || !line.trim().ends_with(" empty -->")
        })
        .collect::<Vec<_>>()
        .join("\n");
    if ended && !body.is_empty() {
        body.push('\n');
    }
    while body.ends_with("\n\n") {
        body.pop();
    }
    let mut out = String::with_capacity(body.len());
    let mut rest = body.as_str();
    while let Some(at) = rest.find('c') {
        let digits = rest[at + 1..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        out.push_str(&rest[..at + 1]);
        if digits > 0 {
            out.push('#');
        }
        rest = &rest[at + 1 + digits..];
    }
    out.push_str(rest);
    out
}

/// The docx `markdown` writes, its plain view the same markdown again,
/// Word-valid, and resolving it the way the source resolves.
fn round_trips(source: &[u8], markdown: &str, view: &str) {
    let docx = written(markdown);
    assert_eq!(plain(&docx), markdown, "{view}: plain fixed point");
    keeps_the_source(source, &docx, view);
}

/// The docx the agent view of `source` writes, its agent view (body only,
/// see [`body`]) the same again, Word-valid, and resolving it the way the
/// source resolves.
fn agent_round_trips(source: &[u8], view: &str) {
    let markdown = agent(source);
    let docx = written(&markdown);
    assert_eq!(
        body(&agent(&docx)),
        body(&markdown),
        "{view}: agent fixed point"
    );
    keeps_the_source(source, &docx, view);
}

/// The round-tripped `docx` is Word-valid, its agent view matches the
/// source's, and resolving it matches the source, accept and reject.
fn keeps_the_source(source: &[u8], docx: &[u8], view: &str) {
    assert_eq!(
        body(&agent(docx)),
        body(&agent(source)),
        "{view}: agent view of the round docx"
    );
    assert!(
        check_word_valid_package(docx).ok(),
        "{view}: {}",
        check_word_valid_package(docx).errors.join("\n")
    );
    for accept in [false, true] {
        assert_eq!(
            resolved(docx, accept),
            resolved(source, accept),
            "{view}: {}",
            if accept { "accept" } else { "reject" }
        );
    }
}

#[test]
fn a_comment_in_a_split_piece_stays_inside_the_outer_insertion() {
    let source = split_with_comment();
    let markdown = plain(&source);
    // One `w:ins` holds the `w:del`, the comment and the piece's text: the
    // comment's note switch must not open a second insertion inside it.
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert!(document.contains("<w:del "), "{document}");
    let ins = document.split("<w:ins ").nth(1).unwrap();
    assert!(ins.contains(r#"<w:del w:id="3""#), "{document}");
    assert!(ins.contains("commentReference"), "{document}");
    assert!(ins.contains("> three"), "{document}");
    round_trips(&source, &markdown, "defect 1 plain");
    agent_round_trips(&source, "defect 1 agent");
}

#[test]
fn a_comment_inside_a_sole_nests_outer_keeps_the_outer() {
    let source = sole_nest_with_comment();
    let markdown = plain(&source);
    // The comment's position alone must not decide whether the outer
    // revision survives: the note still names it.
    assert!(
        markdown.contains("{>>r3 Ben Ode (2026-09-30T09:00:00Z) on r1 Ana Lima"),
        "{markdown}"
    );
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    let ins = document.split("<w:ins ").nth(1).unwrap();
    assert!(ins.starts_with(r#"w:id="1""#), "{document}");
    assert!(ins.contains(r#"<w:del w:id="3""#), "{document}");
    assert!(document.contains("commentReference"), "{document}");
    round_trips(&source, &markdown, "defect 2 plain");
    agent_round_trips(&source, "defect 2 agent");
}

#[test]
fn two_inner_changes_without_outer_text_keep_the_outer() {
    let source = two_inner_no_text();
    let markdown = plain(&source);
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    // One insertion holds both deletions, as the source does.
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    let ins = document.split("<w:ins ").nth(1).unwrap();
    assert!(ins.contains(r#"<w:del w:id="3""#), "{document}");
    assert!(ins.contains(r#"<w:del w:id="4""#), "{document}");
    assert!(ins.contains("<w:delText"), "{document}");
    round_trips(&source, &markdown, "defect 3 plain");
    agent_round_trips(&source, "defect 3 agent");
}

#[test]
fn two_inner_changes_with_outer_text_between_stay_nested() {
    let source = two_inner_with_text();
    let markdown = plain(&source);
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    // The outer wraps text, both deletions and text again: no fresh id for
    // the trailing piece, no deletion fallen out of the insertion.
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert_eq!(document.matches("<w:del ").count(), 2, "{document}");
    assert!(document.contains("<w:ins "), "{document}");
    round_trips(&source, &markdown, "defect 4 plain");
    agent_round_trips(&source, "defect 4 agent");
}

#[test]
fn an_outer_continued_into_the_next_paragraph_keeps_its_id() {
    let source = outer_into_next_paragraph();
    let markdown = plain(&source);
    assert!(
        markdown.contains(" on r1 Ana Lima (2026-09-29T14:05:00Z)<<}"),
        "{markdown}"
    );
    let docx = written(&markdown);
    let document = part_string(&docx, "word/document.xml").unwrap();
    // Two paragraphs, the second's nest naming the id the first spent: one
    // revision across the paragraph mark, as Word writes it — the same
    // `w:id` twice, which the source carries too (Ring-1 warns of it and
    // nothing else; a revision nested in its own kind is what the gate
    // refuses).
    assert_eq!(document.matches("<w:p>").count(), 2, "{document}");
    assert!(
        document.contains(r#"<w:ins w:id="1""#) && !document.contains(r#"<w:ins w:id="4""#),
        "{document}"
    );
    assert_eq!(
        document.matches(r#"<w:ins w:id="1""#).count(),
        2,
        "{document}"
    );
    let findings = check_word_valid_package(&docx);
    assert!(
        findings.findings.is_empty()
            || findings
                .findings
                .iter()
                .all(|f| f.code == "DUPLICATE_REVISION_ID"),
        "defect 5: {}",
        findings.errors.join("\n")
    );
    for accept in [false, true] {
        assert_eq!(
            resolved(&docx, accept),
            resolved(&source, accept),
            "defect 5: {}",
            if accept { "accept" } else { "reject" }
        );
    }
    // Both views are fixed points, the agent view agreeing with the
    // source's.
    assert_eq!(plain(&docx), markdown, "defect 5: plain fixed point");
    let view = agent(&source);
    let agent_docx = written(&view);
    assert_eq!(body(&agent(&agent_docx)), body(&view), "defect 5: agent");
}

#[test]
fn a_carried_cell_or_row_keeps_the_paragraphs_outer_change() {
    let cell = r#"<w:tc><w:tcPr><w:ins w:id="9" w:author="Row Ann" w:date="2026-09-28T08:00:00Z"/></w:tcPr>"#;
    let row = r#"<w:trPr><w:ins w:id="9" w:author="Row Ann" w:date="2026-09-28T08:00:00Z"/></w:trPr><w:tc><w:tcPr/>"#;
    for (name, outer) in [("cell", cell), ("row", row)] {
        let source = carried_cell(outer);
        let markdown = plain(&source);
        // Both views keep the paragraph's own insertion: the container's
        // mark must not swallow it, and the fold still names the outer.
        assert!(
            markdown.contains(" on r1 Ana Lima (2026-09-29T14:05:00Z)<<}"),
            "{name}: {markdown}"
        );
        let view = agent(&source);
        assert!(view.contains(" on r1 @AL<<}"), "{name}: {view}");
        assert!(view.contains("rev r9 @RA in t0.row0"), "{name}: {view}");
        // The agent view carries the container's own mark on its table line
        // (`rev r9 @RA in t0.row0.cell0`), so its round trip is a full one.
        agent_round_trips(&source, &format!("defect 6 {name} agent"));
        // The plain view has no line to carry it on, so its round trip
        // keeps the nest but not the container's insertion: a plain channel
        // for a container's own mark is a design this fix does not make.
        // What it writes comes back as written, Word-valid.
        let docx = written(&markdown);
        assert_eq!(plain(&docx), markdown, "{name}: plain fixed point");
        assert!(
            check_word_valid_package(&docx).ok(),
            "{name}: {}",
            check_word_valid_package(&docx).errors.join("\n")
        );
        let document = part_string(&docx, "word/document.xml").unwrap();
        // No deleted row, no paragraph-mark deletion: the container stays
        // as it was around the nest.
        assert!(!document.contains("<w:trPr><w:del"), "{name}: {document}");
        assert!(
            document.contains(r#"<w:del w:id="3""#),
            "{name}: {document}"
        );
    }
}

#[test]
fn an_outer_author_with_parentheses_round_trips() {
    let source = paren_author_outer();
    let markdown = plain(&source);
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert!(
        document.contains(r#"w:author="Ana Lima (consultant)""#),
        "{markdown}: {document}"
    );
    assert!(!document.contains("{>>"), "{document}");
    round_trips(&source, &markdown, "defect 7 plain");
}

#[test]
fn an_inner_author_holding_the_separator_round_trips() {
    let source = separator_author_inner();
    let markdown = plain(&source);
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert!(
        document.contains(r#"w:author="Bo on r1 Kim""#),
        "{markdown}: {document}"
    );
    assert!(
        document.contains(r#"w:author="Ana Lima""#),
        "{markdown}: {document}"
    );
    round_trips(&source, &markdown, "defect 8 plain");
}

#[test]
fn resolving_a_fold_note_matches_the_source() {
    use jubarte::markdown::{TrackChanges, resolve_critic};
    // Exactly what `convert --commonmark` writes for the sole nest.
    let markdown = "{--two--}{>>r3 Ben Ode (2026-09-30T09:00:00Z) \
                    on r1 Ana Lima (2026-09-29T14:05:00Z)<<}\n";
    assert_eq!(resolve_critic(markdown, TrackChanges::Reject), "\n");
    assert_eq!(resolve_critic(markdown, TrackChanges::Accept), "\n");
    // The agent view's note names the same nest.
    let agent_markdown = "{--two--}{>>r3 @BO on r1 @AL<<}\n";
    assert_eq!(resolve_critic(agent_markdown, TrackChanges::Reject), "\n");
    assert_eq!(resolve_critic(agent_markdown, TrackChanges::Accept), "\n");
}

#[test]
fn a_three_level_nest_keeps_the_innermost_text_live() {
    let source = three_levels();
    let markdown = plain(&source);
    // The innermost run is live text inside Cy Dee's insertion, not
    // deleted text: the note chains every level it names.
    assert!(
        markdown.starts_with(
            "{++two++}{>>r5 Cy Dee (2026-09-30T11:00:00Z) \
             on r3 Ben Ode (2026-09-30T09:00:00Z) \
             on r1 Ana Lima (2026-09-29T14:05:00Z)<<}"
        ),
        "{markdown}"
    );
    let view = agent(&source);
    assert!(
        view.contains("{++two++}{>>r5 @CD on r3 @BO on r1 @AL<<}"),
        "{view}"
    );
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert!(
        document.contains(r#"<w:t xml:space="preserve">two</w:t>"#),
        "{markdown}: {document}"
    );
    assert!(
        document.contains(r#"<w:ins w:id="5" w:author="Cy Dee""#),
        "{markdown}: {document}"
    );
    assert!(
        check_word_valid_package(&written(&markdown)).ok(),
        "{}",
        check_word_valid_package(&written(&markdown))
            .errors
            .join("\n")
    );
    for accept in [false, true] {
        assert_eq!(
            resolved(&written(&markdown), accept),
            resolved(&source, accept),
            "{}",
            if accept { "accept" } else { "reject" }
        );
    }
    // Word refuses a revision nested in its own kind, so the writer
    // rebuilds only the first outer the chain names; from there on the
    // view is a fixed point.
    let round = written(&markdown);
    let second = plain(&round);
    assert_eq!(plain(&written(&second)), second, "{second}");
}
