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
           <w:r><w:delText xml:space="preserve">two</w:delText></w:r></w:ins>\
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
           <w:r><w:delText xml:space="preserve">two</w:delText></w:r></w:ins></w:del></w:p>"#,
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

/// The inner's author holds ` re rN ` (#547 defect 8).
fn separator_author_inner() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:del w:id="3" w:author="Bo re r1 Kim" w:date="2026-09-30T09:00:00Z">
           <w:r><w:delText>two</w:delText></w:r></w:del></w:ins></w:p>"#,
    )
}

/// A three-level alternating nest (#547 defect 10).
fn three_levels() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
           <w:ins w:id="5" w:author="Cy Dee" w:date="2026-09-30T11:00:00Z">
           <w:r><w:delText>two</w:delText></w:r></w:ins></w:del></w:ins></w:p>"#,
    )
}

/// A docx with Zoe Stone's comment 7 ("Check.") beside `body`.
fn with_comment(body: &str) -> Vec<u8> {
    with_comments(body, &[7])
}

fn with_comments(body: &str, ids: &[u32]) -> Vec<u8> {
    use common::docx::{R_NS, W_NS};
    let entries = ids.iter().map(|id| format!(
        r#"<w:comment w:id="{id}" w:author="Zoe Stone" w:date="2026-10-01T10:00:00Z" w:initials="ZS"><w:p><w:r><w:t>Check.</w:t></w:r></w:p></w:comment>"#
    )).collect::<String>();
    const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="{W_NS}" xmlns:r="{R_NS}">{entries}</w:comments>"#
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
        "{--two--}{r3 Ben Ode (2026-09-30T09:00:00Z) \
         re r1 Ana Lima (2026-09-29T14:05:00Z)}\n"
    );
    let view = agent(&sole_del_in_ins());
    assert!(view.contains("{--two--}{r3 @BO re r1 @AL}"), "{view}");
}

#[test]
fn an_insertion_that_is_a_deletions_only_content_names_it_in_the_note() {
    assert_eq!(
        plain(&sole_ins_in_del()),
        "{++two++}{r1 Ana Lima (2026-09-29T14:05:00Z) \
         re r3 Ben Ode (2026-09-30T09:00:00Z)}\n"
    );
    let view = agent(&sole_ins_in_del());
    assert!(view.contains("{++two++}{r1 @AL re r3 @BO}"), "{view}");
}

#[test]
fn a_move_from_that_is_an_insertions_only_content_names_it_in_the_note() {
    assert_eq!(
        plain(&sole_move_from_in_ins()),
        "{--two--}{r3 Ben Ode (2026-09-30T09:00:00Z) \
         re r1 Ana Lima (2026-09-29T14:05:00Z)}\n"
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
    // It belongs to the inner insertion, but Word requires delText under
    // the enclosing deletion (#1134).
    assert!(
        del.contains(r#"<w:delText xml:space="preserve">two</w:delText></w:r></w:ins></w:del>"#),
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
        "{++one ++}{r1 Ana Lima (2026-09-29T14:05:00Z)}\
         {--two--}{r3 Ben Ode (2026-09-30T09:00:00Z) re r1 Ana Lima (2026-09-29T14:05:00Z)}\
         {++ three++}{r1 Ana Lima (2026-09-29T14:05:00Z)}\n"
    );
    let view = agent(&del_in_ins());
    assert!(
        view.contains("{++one ++}{r1 @AL}{--two--}{r3 @BO re r1 @AL}{++ three++}{r1 @AL}"),
        "{view}"
    );
}

#[test]
fn an_insertion_inside_a_deletion_splits_the_deletion() {
    assert_eq!(
        plain(&ins_in_del()),
        "{--one --}{r3 Ben Ode (2026-09-30T09:00:00Z)}\
         {++two++}{r1 Ana Lima (2026-09-29T14:05:00Z) re r3 Ben Ode (2026-09-30T09:00:00Z)}\
         {-- three--}{r3 Ben Ode (2026-09-30T09:00:00Z)}\n"
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
        "pre {++one++}{r6 Cy Dee (2026-09-30T09:00:00Z)}\
         {--a--}{r3 Ben Ode (2026-09-30T09:00:00Z) \
         re r1 Ana Lima (2026-09-29T14:05:00Z)}\n"
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
    // Ana's two insertions are one change, and so are Ben's deletions in
    // them; the change's two ids give each nest its own insertion back.
    let markdown = plain(&two_sole_nests());
    assert_eq!(
        markdown,
        "{--a--}{r3+4 Ben Ode (2026-09-30T09:00:00Z) \
         re r1+2 Ana Lima (2026-09-29T14:05:00Z)}\
         {--b--}{r3+4 Ben Ode (2026-09-30T09:00:00Z) \
         re r1+2 Ana Lima (2026-09-29T14:05:00Z)}\n"
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
    // #1134: Word refuses w:t under a deletion. It opens ins > del > ins
    // with delText but misreads reject-all. Pending the chain-writing
    // decision, the body still rebuilds only the first outer it names.
    let markdown = "{++two++}{r5 @CD re r3 @BO re r1 @AL}\n";
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
        markdown.contains("{r3 Ben Ode (2026-09-30T09:00:00Z) re r1 Ana Lima"),
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
fn an_outer_continued_into_the_next_paragraph_gets_a_fresh_id() {
    let source = outer_into_next_paragraph();
    let markdown = plain(&source);
    assert!(
        markdown.contains(" re r1 Ana Lima (2026-09-29T14:05:00Z)}"),
        "{markdown}"
    );
    let docx = written(&markdown);
    let document = part_string(&docx, "word/document.xml").unwrap();
    // One XML element per paragraph: the second outer receives a fresh
    // id instead of repeating the first paragraph's id (#1137).
    assert_eq!(document.matches("<w:p>").count(), 2, "{document}");
    assert_eq!(
        document.matches(r#"<w:ins w:id="1""#).count(),
        1,
        "{document}"
    );
    assert_eq!(
        document.matches(r#"<w:ins w:id="4""#).count(),
        1,
        "{document}"
    );
    let findings = check_word_valid_package(&docx);
    assert!(findings.ok(), "defect 5: {:?}", findings.errors);
    for accept in [false, true] {
        assert_eq!(
            resolved(&docx, accept),
            resolved(&source, accept),
            "defect 5: {}",
            if accept { "accept" } else { "reject" }
        );
    }
    // Both views retain the fresh id on subsequent round trips.
    let renamed = markdown.replace("re r1 Ana Lima", "re r4 Ana Lima");
    assert_eq!(plain(&docx), renamed, "defect 5: renamed plain view");
    assert_eq!(
        plain(&written(&renamed)),
        renamed,
        "defect 5: plain fixed point"
    );
    let view = agent(&source);
    let agent_docx = written(&view);
    assert_eq!(
        body(&agent(&agent_docx)),
        body(&view.replace("re r1 @AL", "re r4 @AL")),
        "defect 5: agent"
    );
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
            markdown.contains(" re r1 Ana Lima (2026-09-29T14:05:00Z)}"),
            "{name}: {markdown}"
        );
        let view = agent(&source);
        assert!(view.contains(" re r1 @AL}"), "{name}: {view}");
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
        document.contains(r#"w:author="Bo re r1 Kim""#),
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
    let markdown = "{--two--}{r3 Ben Ode (2026-09-30T09:00:00Z) \
                    re r1 Ana Lima (2026-09-29T14:05:00Z)}\n";
    assert_eq!(resolve_critic(markdown, TrackChanges::Reject), "\n");
    assert_eq!(resolve_critic(markdown, TrackChanges::Accept), "\n");
    // The agent view's note names the same nest.
    let agent_markdown = "{--two--}{r3 @BO re r1 @AL}\n";
    assert_eq!(resolve_critic(agent_markdown, TrackChanges::Reject), "\n");
    assert_eq!(resolve_critic(agent_markdown, TrackChanges::Accept), "\n");
}

#[test]
fn a_three_level_nest_keeps_the_innermost_text_live() {
    let source = three_levels();
    let markdown = plain(&source);
    // The innermost run belongs to the insertion even though its XML text
    // element is delText under the deletion (#1134); the note names each level.
    assert!(
        markdown.starts_with(
            "{++two++}{r5 Cy Dee (2026-09-30T11:00:00Z) \
             re r3 Ben Ode (2026-09-30T09:00:00Z) \
             re r1 Ana Lima (2026-09-29T14:05:00Z)}"
        ),
        "{markdown}"
    );
    let view = agent(&source);
    assert!(
        view.contains("{++two++}{r5 @CD re r3 @BO re r1 @AL}"),
        "{view}"
    );
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert!(
        document.contains(r#"<w:delText xml:space="preserve">two</w:delText>"#),
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
    // #1134: Word refuses w:t under a deletion; it opens ins > del > ins
    // with delText but misreads reject-all. The body still rebuilds only
    // the first outer pending that decision; its view is a fixed point.
    let round = written(&markdown);
    let second = plain(&round);
    assert_eq!(plain(&written(&second)), second, "{second}");
}

/// Live text beside deletions inside one insertion (issue #1111).
fn live_insertion_with_deletions(shape: &str) -> Vec<u8> {
    let live = r#"<w:r><w:t xml:space="preserve"> and the bank</w:t></w:r>"#;
    let deletion = r#"<w:del w:id="4" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z"><w:r><w:delText xml:space="preserve"> and the broker</w:delText></w:r></w:del>"#;
    let second = r#"<w:del w:id="5" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z"><w:r><w:delText xml:space="preserve"> and the seller</w:delText></w:r></w:del>"#;
    let children = match shape {
        "tail" => format!("{live}{deletion}"),
        "head" => format!("{deletion}{live}"),
        "middle" => format!("{live}{deletion}{live}"),
        "two" => format!("{live}{deletion}{live}{second}{live}"),
        _ => panic!("unknown shape: {shape}"),
    };
    docx(&format!(
        r#"<w:p><w:r><w:t>Notify the buyer</w:t></w:r><w:ins w:id="3" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">{children}</w:ins><w:r><w:t>.</w:t></w:r></w:p>"#
    ))
}

fn live_insertion_keeps_nested_deletions(shape: &str) {
    let source = live_insertion_with_deletions(shape);
    let view = agent(&source);
    let count = if shape == "two" { 2 } else { 1 };
    for id in 4..4 + count {
        assert!(
            view.contains(&format!("{{r{id} @BO re r3 @AL}}")),
            "{shape}: {view}"
        );
    }
    assert!(view.contains("{++ and the bank++}{r3 @AL}"), "{view}");
    for markdown in [view, plain(&source)] {
        let round = written(&markdown);
        assert_eq!(
            body(&agent(&round)),
            body(&agent(&source)),
            "{shape}: {markdown}"
        );
        assert_eq!(plain(&round), plain(&source), "{shape}: {markdown}");
        let document = part_string(&round, "word/document.xml").unwrap();
        assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
        let insertion = document.split("<w:ins ").nth(1).unwrap();
        let insertion = insertion.split("</w:ins>").next().unwrap();
        assert_eq!(document.matches("<w:del ").count(), count, "{document}");
        assert_eq!(insertion.matches("<w:del ").count(), count, "{document}");
        assert!(check_word_valid_package(&round).ok(), "{document}");
        let rejected =
            jubarte::changes::reject_changes(&round, &jubarte::changes::ChangeFilter::ids(["r4"]))
                .unwrap();
        let document = part_string(&rejected, "word/document.xml").unwrap();
        let insertion = document.split("<w:ins ").nth(1).unwrap();
        let insertion = insertion.split("</w:ins>").next().unwrap();
        assert!(insertion.contains(" and the broker</w:t>"), "{document}");
        let original_rejected =
            jubarte::changes::reject_changes(&source, &jubarte::changes::ChangeFilter::ids(["r4"]))
                .unwrap();
        assert_eq!(
            body(&agent(&rejected)),
            body(&agent(&original_rejected)),
            "{shape}"
        );
    }
}

#[test]
fn a_nested_deletion_after_live_text_names_its_insertion() {
    live_insertion_keeps_nested_deletions("tail");
}

#[test]
fn a_nested_deletion_before_live_text_names_its_insertion() {
    live_insertion_keeps_nested_deletions("head");
}

#[test]
fn a_nested_deletion_between_live_runs_names_its_insertion() {
    live_insertion_keeps_nested_deletions("middle");
}

#[test]
fn two_nested_deletions_among_live_runs_name_their_insertion() {
    live_insertion_keeps_nested_deletions("two");
}

#[test]
fn a_note_on_a_live_insertion_rebuilds_the_nested_deletion() {
    for shape in ["tail", "head", "middle", "two"] {
        let source = live_insertion_with_deletions(shape);
        let view = agent(&source)
            .replace("{r4 @BO}", "{r4 @BO re r3 @AL}")
            .replace("{r5 @BO}", "{r5 @BO re r3 @AL}");
        let document = part_string(&written(&view), "word/document.xml").unwrap();
        assert_eq!(
            document.matches("<w:ins ").count(),
            1,
            "{shape}: {document}"
        );
        let mut expected = vec![("w:ins", "3", None), ("w:del", "4", Some("3"))];
        if shape == "two" {
            expected.push(("w:del", "5", Some("3")));
        }
        assert_revision_tree(
            &document,
            &expected,
            &revision_attributes(&part_string(&source, "word/document.xml").unwrap()),
        );
    }
}

#[test]
fn review_1111_paragraph_wrapping_insertion_keeps_text() {
    let source = docx(
        r#"<w:ins w:id="3" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
     <w:p><w:r><w:t>Notify the buyer</w:t></w:r>
       <w:del w:id="4" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
         <w:r><w:delText> and the broker</w:delText></w:r></w:del></w:p>
   </w:ins>"#,
    );
    for view in [agent(&source), plain(&source)] {
        assert!(view.contains("Notify the buyer"), "{view}");
        assert!(view.contains(" and the broker"), "{view}");
        let round = written(&view);
        assert!(plain(&round).contains("Notify the buyer"));
        assert!(plain(&round).contains(" and the broker"));
    }
}

fn assert_revision_tree(
    xml: &str,
    expected: &[(&str, &str, Option<&str>)],
    attributes: &[(String, Option<String>)],
) {
    use quick_xml::{Reader, events::Event};
    let mut reader = Reader::from_str(xml);
    let mut stack = Vec::new();
    let mut found = Vec::new();
    loop {
        match reader.read_event().unwrap() {
            Event::Start(element) => {
                let name = element.name().as_ref().to_string();
                if matches!(name.as_str(), "w:ins" | "w:del") {
                    let id = element
                        .attributes()
                        .map(Result::unwrap)
                        .find(|attr| attr.key.as_ref() == "w:id")
                        .unwrap();
                    let id = id.value.into_owned();
                    let parent = stack
                        .iter()
                        .rev()
                        .find_map(|entry: &Option<String>| entry.clone());
                    found.push((name, id.clone(), parent));
                    stack.push(Some(id));
                } else {
                    stack.push(None);
                }
            }
            Event::End(_) => {
                stack.pop().unwrap();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    let expected = expected
        .iter()
        .map(|(name, id, parent)| (name.to_string(), id.to_string(), parent.map(str::to_string)))
        .collect::<Vec<_>>();
    assert_eq!(found, expected, "{xml}");
    let ids = found
        .iter()
        .map(|(_, id, _)| id)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(ids.len(), found.len(), "duplicate revision id: {xml}");
    assert_eq!(
        revision_attributes(xml),
        attributes,
        "revision author/date: {xml}"
    );
}

fn assert_nested_round_trip(
    source: &[u8],
    notes: &[&str],
    expected: &[(&str, &str, Option<&str>)],
) {
    let view = agent(source);
    for note in notes {
        assert!(view.contains(note), "missing {note}: {view}");
    }
    for markdown in [view, plain(source)] {
        let round = written(&markdown);
        let document = part_string(&round, "word/document.xml").unwrap();
        assert_revision_tree(
            &document,
            expected,
            &revision_attributes(&part_string(source, "word/document.xml").unwrap()),
        );
        assert_eq!(body(&agent(&round)), body(&agent(source)), "{markdown}");
        assert_eq!(plain(&round), plain(source), "{markdown}");
    }
}

#[test]
fn review_1111_comment_scope_names_both_deletions() {
    let source = with_comment(
        r#"<w:p><w:r><w:t>base </w:t></w:r>
     <w:ins w:id="3" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
       <w:r><w:t>live1 </w:t></w:r><w:commentRangeStart w:id="7"/>
       <w:del w:id="4" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
         <w:r><w:delText>d1</w:delText></w:r></w:del>
       <w:r><w:t>live2 </w:t></w:r>
       <w:del w:id="5" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
         <w:r><w:delText>d2</w:delText></w:r></w:del>
       <w:commentRangeEnd w:id="7"/><w:r><w:commentReference w:id="7"/></w:r>
     </w:ins></w:p>"#,
    );
    assert_nested_round_trip(
        &source,
        &["{r4 @BO re r3 @AL}", "{r5 @BO re r3 @AL}"],
        &[
            ("w:ins", "3", None),
            ("w:del", "4", Some("3")),
            ("w:del", "5", Some("3")),
        ],
    );
}

#[test]
fn review_1111_nested_comment_scopes_name_the_deletion() {
    let source = with_comments(
        r#"<w:p><w:r><w:t>base </w:t></w:r>
     <w:ins w:id="3" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
       <w:r><w:t>live1 </w:t></w:r><w:commentRangeStart w:id="7"/><w:commentRangeStart w:id="8"/>
       <w:del w:id="4" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
         <w:r><w:delText>d1</w:delText></w:r></w:del>
       <w:commentRangeEnd w:id="8"/><w:commentRangeEnd w:id="7"/>
       <w:r><w:commentReference w:id="8"/></w:r><w:r><w:commentReference w:id="7"/></w:r>
     </w:ins></w:p>"#,
        &[7, 8],
    );
    assert_nested_round_trip(
        &source,
        &["{r4 @BO re r3 @AL}"],
        &[("w:ins", "3", None), ("w:del", "4", Some("3"))],
    );
}

#[test]
fn review_1111_split_names_every_full_parent_chain() {
    let source = docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
     <w:r><w:t>liveA </w:t></w:r>
     <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
       <w:ins w:id="5" w:author="Cy Dee" w:date="2026-09-30T11:00:00Z"><w:r><w:delText>x</w:delText></w:r></w:ins>
       <w:r><w:delText> mid </w:delText></w:r>
       <w:ins w:id="7" w:author="Cy Dee" w:date="2026-09-30T11:00:00Z"><w:r><w:delText>y</w:delText></w:r></w:ins>
     </w:del>
     <w:r><w:t>tail</w:t></w:r></w:ins></w:p>"#,
    );
    assert_nested_round_trip(
        &source,
        &[
            "{r5 @CD re r3 @BO re r1 @AL}",
            "{r3 @BO re r1 @AL}",
            "{r7 @CD re r3 @BO re r1 @AL}",
        ],
        &[
            ("w:ins", "1", None),
            ("w:del", "3", Some("1")),
            ("w:ins", "5", Some("3")),
            ("w:ins", "7", Some("3")),
        ],
    );
}

#[test]
fn split_revision_parent_choice_is_deterministic() {
    // Revision 3 has two named parents and one piece with none. Vary the
    // bare piece's position to exercise both preceding and smallest serials.
    let pieces = [
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">
        <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
        <w:r><w:delText>a</w:delText></w:r></w:del></w:ins></w:p>"#,
        r#"<w:p><w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
        <w:r><w:delText>b</w:delText></w:r></w:del></w:p>"#,
        r#"<w:p><w:ins w:id="2" w:author="Cy Dee" w:date="2026-09-30T11:00:00Z">
        <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">
        <w:r><w:delText>c</w:delText></w:r></w:del></w:ins></w:p>"#,
    ];
    for (name, order, outer_ids, delete_ids, bare_parent) in [
        (
            "between",
            [0, 1, 2],
            ["1", "4", "2"],
            ["3", "5", "6"],
            "Ana Lima",
        ),
        (
            "before",
            [1, 0, 2],
            ["1", "4", "2"],
            ["3", "5", "6"],
            "Ana Lima",
        ),
        (
            "after",
            [0, 2, 1],
            ["1", "2", "5"],
            ["3", "4", "6"],
            "Cy Dee",
        ),
    ] {
        let source = docx(&order.map(|at| pieces[at]).concat());
        let view = agent(&source);
        let first = written(&view);
        let checked = check_word_valid_package(&first);
        assert!(checked.ok(), "{name}: {:?}", checked.errors);
        for attempt in 1..64 {
            let next = written(&view);
            assert!(
                next == first,
                "{name}: split revision parent choice changed on write {attempt}:\nfirst: {}\nnext: {}",
                part_string(&first, "word/document.xml").unwrap(),
                part_string(&next, "word/document.xml").unwrap(),
            );
        }
        let document = part_string(&first, "word/document.xml").unwrap();
        let paragraphs = document
            .split("<w:p>")
            .skip(1)
            .map(|part| part.split("</w:p>").next().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(paragraphs.len(), 3, "{name}: {document}");
        for (((paragraph, outer_id), deleted_id), piece) in
            paragraphs.iter().zip(outer_ids).zip(delete_ids).zip(order)
        {
            // Every paragraph keeps the deterministic parent attribution,
            // with separate OOXML ids for separate elements.
            let parent = match piece {
                0 => "Ana Lima",
                1 => bare_parent,
                _ => "Cy Dee",
            };
            let parent_piece = if piece == 1 {
                if name == "after" { 2 } else { 0 }
            } else {
                piece
            };
            assert_revision_tree(
                &format!("<w:p>{paragraph}</w:p>"),
                &[
                    ("w:ins", outer_id, None),
                    ("w:del", deleted_id, Some(outer_id)),
                ],
                &revision_attributes(pieces[parent_piece]),
            );
            assert!(
                paragraph.contains(&format!(r#"w:author="{parent}""#)),
                "{name}: {paragraph}"
            );
        }
    }
}

#[test]
fn live_insertion_keeps_a_same_author_deletion() {
    let source = live_insertion_with_deletions("tail");
    let document = part_string(&source, "word/document.xml")
        .unwrap()
        .replace("Ben Ode", "Ana Lima")
        .replace("2026-09-30T09:00:00Z", "2026-09-29T14:05:00Z");
    let source = common::docx::replace_entry(&source, "word/document.xml", document.as_bytes());
    assert_nested_round_trip(
        &source,
        &["{r4 @AL re r3 @AL}"],
        &[("w:ins", "3", None), ("w:del", "4", Some("3"))],
    );
}

#[test]
fn live_insertion_keeps_a_move_from() {
    let source = live_insertion_with_deletions("tail");
    let document = part_string(&source, "word/document.xml")
        .unwrap()
        .replace("<w:del ", "<w:moveFrom ")
        .replace("</w:del>", "</w:moveFrom>")
        .replace(
            "<w:p>",
            r#"<w:p><w:moveFromRangeStart w:id="7" w:name="move"/>"#,
        )
        .replace("</w:p>", r#"<w:moveFromRangeEnd w:id="7"/></w:p>"#);
    let source = common::docx::replace_entry(&source, "word/document.xml", document.as_bytes());
    assert_nested_round_trip(
        &source,
        &["{r4 @BO re r3 @AL}"],
        &[("w:ins", "3", None), ("w:del", "4", Some("3"))],
    );
}

#[test]
fn whitespace_between_nested_deletions_keeps_their_parent() {
    let source = live_insertion_with_deletions("two");
    let document = part_string(&source, "word/document.xml")
        .unwrap()
        .replace(" and the bank", " ");
    let source = common::docx::replace_entry(&source, "word/document.xml", document.as_bytes());
    assert!(agent(&source).contains("{++ ++}{r3 @AL}"));
    assert_nested_round_trip(
        &source,
        &["{r4 @BO re r3 @AL}", "{r5 @BO re r3 @AL}"],
        &[
            ("w:ins", "3", None),
            ("w:del", "4", Some("3")),
            ("w:del", "5", Some("3")),
        ],
    );
}

#[test]
fn a_footnote_live_insertion_keeps_its_nested_deletion() {
    let source = live_insertion_with_deletions("tail");
    let document = part_string(&source, "word/document.xml").unwrap();
    let paragraph = document
        .split("<w:body>")
        .nth(1)
        .unwrap()
        .split("<w:sectPr>")
        .next()
        .unwrap();
    let paragraph = paragraph.replace(
        "<w:p>",
        r#"<w:p><w:pPr><w:pStyle w:val="FootnoteText"/></w:pPr>"#,
    );
    let notes = format!(
        r#"<w:footnotes xmlns:w="{}"><w:footnote w:id="1">{paragraph}</w:footnote></w:footnotes>"#,
        common::docx::W_NS
    );
    let parts = [common::docx::Part {
        name: "word/footnotes.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
        xml: &notes,
    }];
    let source = common::docx::docx_with(
        r#"<w:p><w:r><w:t>Body.</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &parts,
    );
    let view = agent(&source);
    let note = "{footnotes.r4 @BO re footnotes.r3 @AL}";
    assert!(view.contains(note), "{view}");
    for markdown in [view, plain(&source)] {
        let round = written(&markdown);
        let notes = part_string(&round, "word/footnotes.xml").unwrap();
        assert_revision_tree(
            &notes,
            &[("w:ins", "3", None), ("w:del", "4", Some("3"))],
            &revision_attributes(&part_string(&source, "word/footnotes.xml").unwrap()),
        );
        let second = agent(&round);
        assert!(second.contains(note), "{second}");
        assert_eq!(body(&second), body(&agent(&source)), "{markdown}");
        assert_eq!(plain(&round), plain(&source), "{markdown}");
    }
}

/// Move the body's probe paragraph into a single-paragraph header or footer.
fn in_story(source: &[u8], kind: &str) -> Vec<u8> {
    let document = part_string(source, "word/document.xml").unwrap();
    let paragraph = document
        .split("<w:body>")
        .nth(1)
        .unwrap()
        .split("<w:sectPr>")
        .next()
        .unwrap();
    let root = if kind == "header" { "hdr" } else { "ftr" };
    let xml = format!(
        r#"<w:{root} xmlns:w="{}">{paragraph}</w:{root}>"#,
        common::docx::W_NS
    );
    common::docx::docx_with_sect(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &[common::docx::Part {
            name: &format!("word/{kind}1.xml"),
            content_type: &format!(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml"
            ),
            rel_type: &format!(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}"
            ),
            xml: &xml,
        }],
        &format!(r#"<w:{kind}Reference w:type="default" r:id="rIdX0"/>"#),
    )
}

/// The whole story map, independent of the package hash and body summary.
fn story_block(view: &str, kind: &str) -> String {
    view.split_once(&format!("\n{kind}s:\n"))
        .unwrap()
        .1
        .lines()
        .take_while(|line| line.starts_with(' '))
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_story_nested_round_trip(
    source: &[u8],
    kind: &str,
    notes: &[&str],
    expected: &[(&str, &str, Option<&str>)],
) {
    let source = in_story(source, kind);
    let view = agent(&source);
    for note in notes {
        assert!(view.contains(note), "missing {note}: {view}");
    }
    let round = written(&view);
    let part = part_string(&round, &format!("word/{kind}1.xml")).unwrap();
    assert_revision_tree(
        &part,
        expected,
        &revision_attributes(&part_string(&source, &format!("word/{kind}1.xml")).unwrap()),
    );
    assert!(check_word_valid_package(&round).ok(), "{part}");
    let second = agent(&round);
    for note in notes {
        assert!(second.contains(note), "missing {note}: {second}");
    }
    assert_eq!(body(&second), body(&agent(&source)), "{view}");
    assert_eq!(plain(&round), plain(&source), "{view}");
    assert_eq!(
        story_block(&second, kind),
        story_block(&agent(&source), kind)
    );
    assert_eq!(
        story_block(&agent(&written(&second)), kind),
        story_block(&second, kind)
    );
    assert_eq!(plain(&written(&plain(&round))), plain(&round));
    // CommonMark omits header/footer front matter; its body remains a fixed point.
    let commonmark = plain(&source);
    assert_eq!(plain(&written(&commonmark)), commonmark);
}

fn story_live_insertion_keeps_nested_deletions(kind: &str, shape: &str) {
    let mut notes = vec![format!("{{{kind}1.r4 @BO re {kind}1.r3 @AL}}")];
    let mut expected = vec![("w:ins", "3", None), ("w:del", "4", Some("3"))];
    if shape == "two" {
        notes.push(format!("{{{kind}1.r5 @BO re {kind}1.r3 @AL}}"));
        expected.push(("w:del", "5", Some("3")));
    }
    assert_story_nested_round_trip(
        &live_insertion_with_deletions(shape),
        kind,
        &notes.iter().map(String::as_str).collect::<Vec<_>>(),
        &expected,
    );
}

#[test]
fn a_header_live_insertion_keeps_its_nested_deletion() {
    story_live_insertion_keeps_nested_deletions("header", "tail");
}

#[test]
fn a_footer_live_insertion_keeps_its_nested_deletion() {
    story_live_insertion_keeps_nested_deletions("footer", "tail");
}

#[test]
fn a_header_live_insertion_keeps_a_leading_deletion() {
    story_live_insertion_keeps_nested_deletions("header", "head");
}

#[test]
fn a_header_live_insertion_keeps_a_middle_deletion() {
    story_live_insertion_keeps_nested_deletions("header", "middle");
}

#[test]
fn a_header_live_insertion_keeps_two_deletions() {
    story_live_insertion_keeps_nested_deletions("header", "two");
}

#[test]
fn a_header_deletion_keeps_its_nested_insertion() {
    assert_story_nested_round_trip(
        &ins_in_del(),
        "header",
        &["{header1.r1 @AL re header1.r3 @BO}"],
        &[("w:del", "3", None), ("w:ins", "1", Some("3"))],
    );
}

#[test]
fn review_1126_day_range_story_reuses_one_open_tag() {
    for kind in ["header", "footer"] {
        for text in [
            "{++a++}{PART.r3 @AL}{--x--}{PART.r4 @BO re PART.r3 @AL}",
            "{--x--}{PART.r4 @BO re PART.r3 @AL}{++a++}{PART.r3 @AL}",
        ] {
            let text = text.replace("PART", &format!("{kind}1"));
            let view = format!(
                "---\nauthors:\n  AL: Alice # 1 revision, 2026-09-29..2026-09-30\n  BO: Bob # 1 revision, 2026-09-30T09:00:00Z\n{kind}s:\n  default: {{id: {kind}1, text: \"{text}\", style: Normal}}\n---\nBody.\n"
            );
            let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
            println!("{kind}: {xml}");
            assert_eq!(xml.matches("<w:ins ").count(), 1, "{xml}");
            assert_revision_tree(
                &xml,
                &[("w:ins", "3", None), ("w:del", "4", Some("3"))],
                &[
                    revision_by("Alice", Some("2026-09-29T00:00:00Z")),
                    revision_by("Bob", Some("2026-09-30T09:00:00Z")),
                ],
            );
        }
    }
}

#[test]
fn review_1126_day_range_body_has_one_open_tag() {
    let view = "---\nauthors:\n  AL: Alice # 1 revision, 2026-09-29..2026-09-30\n  BO: Bob # 1 revision, 2026-09-30T09:00:00Z\n---\nNotify the buyer{++ and the bank++}{r3 @AL}{-- and the broker--}{r4 @BO re r3 @AL}.\n";
    let xml = part_string(&written(view), "word/document.xml").unwrap();
    println!("body: {xml}");
    assert_revision_tree(
        &xml,
        &[("w:ins", "3", None), ("w:del", "4", Some("3"))],
        &[
            revision_by("Alice", Some("2026-09-30T23:59:59Z")),
            revision_by("Bob", Some("2026-09-30T09:00:00Z")),
        ],
    );
}

#[test]
fn review_1126_noid_parent_is_one_fresh_revision() {
    for kind in ["header", "footer"] {
        for text in [
            "{++a++}{PART.noid @AL}{++b++}{PART.noid @AL}{--x--}{PART.r4 @BO re PART.noid @AL}{++c++}{PART.noid @AL}",
            "{++a++}{PART.noid @AL}{--x--}{PART.r4 @BO re PART.noid @AL}{++b++}{PART.noid @AL}",
            "{--x--}{PART.r4 @BO re PART.noid @AL}{++ab++}{PART.noid @AL}",
            "{++ab++}{PART.noid @AL}{--x--}{PART.r4 @BO re PART.noid @AL}",
        ] {
            let text = text.replace("PART", &format!("{kind}1"));
            let view = format!(
                "---\nauthors:\n  AL: Alice # 1 revision, 2026-09-29T14:05:00Z\n  BO: Bob # 1 revision, 2026-09-30T09:00:00Z\n{kind}s:\n  default: {{id: {kind}1, text: \"{text}\", style: Normal}}\n---\nBody.\n"
            );
            let round = written(&view);
            let xml = part_string(&round, &format!("word/{kind}1.xml")).unwrap();
            assert_eq!(xml.matches("<w:ins ").count(), 1, "{xml}");
            let id = xml
                .split("<w:ins w:id=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap();
            assert_ne!(id, "4", "fresh parent id collides with child: {xml}");
            assert_revision_tree(
                &xml,
                &[("w:ins", id, None), ("w:del", "4", Some(id))],
                &[
                    revision_by("Alice", Some("2026-09-29T14:05:00Z")),
                    revision_by("Bob", Some("2026-09-30T09:00:00Z")),
                ],
            );
            assert!(check_word_valid_package(&round).ok(), "{xml}");
        }
    }
}

#[test]
fn review_1126_noid_ids_stop_at_nest_boundaries() {
    let nest = "{++a++}{header1.noid @AL}{--x--}{header1.r4 @BO re header1.noid @AL}{++b++}{header1.noid @AL}";
    for text in [
        "{++a++}{header1.noid @AL}{++b++}{header1.noid @AL}".to_string(),
        format!("{nest} gap {}", nest.replace("r4", "r5")),
        format!("{nest}{{PAGE}}{}", nest.replace("r4", "r5")),
        format!(
            "{nest}{}",
            nest.replace("@AL", "@AL 2026-09-30T12:00:00Z")
                .replace("r4", "r5")
        ),
        format!("{nest}{}", nest.replace("@AL", "@BO").replace("r4", "r5")),
    ] {
        let view = format!(
            "---\nauthors:\n  AL: Alice # 2 revisions, 2026-09-29T14:05:00Z\n  BO: Bob # 2 revisions, 2026-09-30T09:00:00Z\nheaders:\n  default: {{id: header1, text: \"{text}\", style: Normal}}\n---\nBody.\n"
        );
        let xml = part_string(&written(&view), "word/header1.xml").unwrap();
        let ids = xml
            .split("<w:ins w:id=\"")
            .skip(1)
            .map(|tail| tail.split('"').next().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 2, "{xml}");
        assert_ne!(
            ids[0], ids[1],
            "separate anonymous revisions share an id: {xml}"
        );
    }
}

#[test]
fn review_1126_body_puts_a_substitution_parent_on_the_old_side_only() {
    let xml = part_string(
        &written("{~~old~>new~~}{r5+6 @BO re r3 @AL}"),
        "word/document.xml",
    )
    .unwrap();
    assert_revision_tree(
        &xml,
        &[
            ("w:ins", "3", None),
            ("w:del", "5", Some("3")),
            ("w:ins", "6", None),
        ],
        &[
            revision_by("AL", Some("1970-01-01T00:00:00Z")),
            revision_by("BO", Some("1970-01-01T00:00:00Z")),
            revision_by("BO", Some("1970-01-01T00:00:00Z")),
        ],
    );
    assert!(
        xml.contains("<w:delText xml:space=\"preserve\">old</w:delText>"),
        "{xml}"
    );
    assert!(xml.contains("</w:del></w:ins><w:ins w:id=\"6\""), "{xml}");
    assert!(xml.contains(">new</w:t>"), "{xml}");
}

#[test]
fn review_1126_story_puts_a_substitution_parent_on_the_old_side_only() {
    for kind in ["header", "footer"] {
        for paragraph in [0, 1] {
            let note = format!("{kind}1.r5+6 @BO re {kind}1.r3 @AL");
            let text = format!("{{~~old~>new~~}}{{{note}}}");
            let text = if paragraph == 0 {
                format!("\"{text}\"")
            } else {
                format!("[plain, \"{text}\"]")
            };
            let view =
                format!("---\n{kind}s:\n  default: {{id: {kind}1, text: {text}}}\n---\nBody.\n");
            let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
            assert_revision_tree(
                &xml,
                &[
                    ("w:ins", "3", None),
                    ("w:del", "5", Some("3")),
                    ("w:ins", "6", None),
                ],
                &[
                    revision_by("AL", Some("1970-01-01T00:00:00Z")),
                    revision_by("BO", Some("1970-01-01T00:00:00Z")),
                    revision_by("BO", Some("1970-01-01T00:00:00Z")),
                ],
            );
            assert!(xml.contains("</w:del></w:ins><w:ins w:id=\"6\""), "{xml}");
            assert!(
                xml.contains(">old</w:delText>") && xml.contains(">new</w:t>"),
                "{xml}"
            );
        }
    }
}

fn revision_by(author: &str, date: Option<&str>) -> (String, Option<String>) {
    (author.to_string(), date.map(str::to_string))
}

/// Expected attributions come from the source package, independently of
/// the writer; move marks become insertions/deletions on the round trip.
fn revision_attributes(xml: &str) -> Vec<(String, Option<String>)> {
    use quick_xml::{Reader, events::Event};
    let mut reader = Reader::from_str(xml);
    let mut attributes = Vec::new();
    loop {
        match reader.read_event().unwrap() {
            Event::Start(element)
                if matches!(
                    element.name().as_ref(),
                    "w:ins" | "w:del" | "w:moveFrom" | "w:moveTo"
                ) =>
            {
                let get = |name: &str| {
                    element
                        .attributes()
                        .map(Result::unwrap)
                        .find(|attr| attr.key.as_ref() == name)
                        .map(|attr| attr.value.into_owned())
                };
                attributes.push((get("w:author").unwrap(), get("w:date")));
            }
            Event::Eof => break,
            _ => {}
        }
    }
    attributes
}

#[test]
fn review_1126_tree_check_rejects_wrong_author_and_date() {
    let original = r#"<w:ins w:id="3" w:author="Alice" w:date="2026-09-29T14:05:00Z"><w:r><w:t>live</w:t></w:r></w:ins>"#;
    let expected = [("w:ins", "3", None)];
    let attributes = revision_attributes(original);
    for changed in [
        original.replace("Alice", "Bob"),
        original.replace("2026-09-29", "2026-09-30"),
    ] {
        assert!(
            std::panic::catch_unwind(|| assert_revision_tree(&changed, &expected, &attributes))
                .is_err(),
            "tree assertion accepted changed metadata: {changed}"
        );
    }
}

#[test]
fn a_footer_live_insertion_keeps_a_leading_deletion() {
    story_live_insertion_keeps_nested_deletions("footer", "head");
}

#[test]
fn a_footer_live_insertion_keeps_a_middle_deletion() {
    story_live_insertion_keeps_nested_deletions("footer", "middle");
}

/// A nested story probe with Alice's live runs around Bob's payload.
fn review_1126_story_payload(payload: &str, beside: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:ins w:id="3" w:author="Alice" w:date="2026-09-29T14:05:00Z">
      <w:r><w:t>live</w:t></w:r>
      <w:del w:id="4" w:author="Bob" w:date="2026-09-30T09:00:00Z">{payload}</w:del>
      <w:r><w:t>tail</w:t></w:r></w:ins>{beside}</w:p>"#
    ))
}

#[test]
fn review_1126_nested_story_keeps_a_page_field() {
    let source = review_1126_story_payload(
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>
      <w:r><w:delInstrText xml:space="preserve"> PAGE </w:delInstrText></w:r>
      <w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        "",
    );
    for kind in ["header", "footer"] {
        let story = in_story(&source, kind);
        let view = agent(&story);
        assert!(view.contains("{PAGE}"), "{view}");
        let note = format!("{{{kind}1.r4 @B re {kind}1.r3 @A}}");
        assert_story_nested_round_trip(
            &source,
            kind,
            &[&note],
            &[("w:ins", "3", None), ("w:del", "4", Some("3"))],
        );
        let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
        let deletion = xml
            .split("<w:del ")
            .nth(1)
            .unwrap()
            .split("</w:del>")
            .next()
            .unwrap();
        assert!(deletion.contains("w:delInstrText"), "{xml}");
        assert!(deletion.contains(" PAGE "), "{xml}");
        assert_eq!(
            deletion.matches("w:fldCharType=\"begin\"").count(),
            1,
            "{xml}"
        );
        assert_eq!(
            deletion.matches("w:fldCharType=\"end\"").count(),
            1,
            "{xml}"
        );
    }
}

#[test]
fn review_1126_story_keeps_separate_authors_beside_a_nest() {
    let source = review_1126_story_payload(
        r#"<w:r><w:delText>x</w:delText></w:r>"#,
        r#"<w:ins w:id="6" w:author="Alice" w:date="2026-09-29T14:05:00Z"><w:r><w:t>new</w:t></w:r></w:ins>
      <w:del w:id="7" w:author="Bob" w:date="2026-09-30T09:00:00Z"><w:r><w:delText>old</w:delText></w:r></w:del>"#,
    );
    for kind in ["header", "footer"] {
        let view = agent(&in_story(&source, kind));
        // Alice's two insertions are one change (`r3+6`) on each of its
        // pieces, the nest's `re` too; Bob's deletion beside them is his.
        assert!(
            view.contains(&format!(
                "{{++new++}}{{{kind}1.r3+6 @A}}{{--old--}}{{{kind}1.r7 @B}}"
            )),
            "{view}"
        );
        let note = format!("{{{kind}1.r4 @B re {kind}1.r3+6 @A}}");
        assert_story_nested_round_trip(
            &source,
            kind,
            &[&note],
            &[
                ("w:ins", "3", None),
                ("w:del", "4", Some("3")),
                ("w:ins", "6", None),
                ("w:del", "7", None),
            ],
        );
    }
}

#[test]
fn review_1126_open_tag_identity_is_local_to_each_story_part() {
    let view = "---\nauthors:\n  AL: Alice # 1 revision, 2026-09-29T14:05:00Z\n  BO: Bob # 1 revision, 2026-09-30T09:00:00Z\nheaders:\n  default: {id: header1, text: \"{++a++}{header1.r3 @AL}\"}\nfooters:\n  default: {id: footer1, text: \"{++b++}{footer1.r3 @BO}\"}\n---\nBody.\n";
    let round = written(view);
    for (part, author, date) in [
        ("header1", "Alice", "2026-09-29T14:05:00Z"),
        ("footer1", "Bob", "2026-09-30T09:00:00Z"),
    ] {
        let xml = part_string(&round, &format!("word/{part}.xml")).unwrap();
        assert_revision_tree(
            &xml,
            &[("w:ins", "3", None)],
            &[revision_by(author, Some(date))],
        );
    }
}

/// In a header as in the body (#1261), a deletion and its author's
/// insertion are one change (`r5+6`). A substitution nested in the
/// insertion names it with `re`: the writer gives the deletion 5 and the
/// insertion around the nest 6, and the nest's own sides 7 and 8.
#[test]
fn story_replacement_around_a_nest_gives_each_side_its_own_id() {
    let view = "---\nauthors:\n  AL: Alice # 2 revisions, 2026-09-29T14:05:00Z\n  BO: Bob # 2 revisions, 2026-09-30T09:00:00Z\nheaders:\n  default: {id: header1, text: \"{--old--}{header1.r5+6 @AL}{~~x~>y~~}{header1.r7+8 @BO re header1.r5+6 @AL}.\"}\n---\nBody.\n";
    let xml = part_string(&written(view), "word/header1.xml").unwrap();
    let alice = revision_by("Alice", Some("2026-09-29T14:05:00Z"));
    let bob = revision_by("Bob", Some("2026-09-30T09:00:00Z"));
    assert_revision_tree(
        &xml,
        &[
            ("w:del", "5", None),
            ("w:ins", "6", None),
            ("w:del", "7", Some("6")),
            ("w:ins", "8", None),
        ],
        &[alice.clone(), alice, bob.clone(), bob],
    );
}

/// A nest closing one mark of a change, then that change's next mark
/// (#1354): `r7+9` names two marks, so ` gamma.` is mark 9, not more of
/// mark 7, and the copy reads as the source does. Headers and footers
/// pool their pieces the same way.
#[test]
fn a_nest_then_the_next_mark_of_its_change_writes_each_mark_once() {
    let paragraph = r#"<w:p><w:r><w:t xml:space="preserve">Start </w:t></w:r>
      <w:ins w:id="7" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:r><w:t xml:space="preserve">alpha </w:t></w:r>
      <w:del w:id="8" w:author="Alice" w:date="2026-10-02T10:00:00Z"><w:r><w:delText>beta</w:delText></w:r></w:del></w:ins>
      <w:ins w:id="9" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:r><w:t xml:space="preserve"> gamma.</w:t></w:r></w:ins></w:p>"#;
    let source = docx(paragraph);
    let tree = [
        ("w:ins", "7", None),
        ("w:del", "8", Some("7")),
        ("w:ins", "9", None),
    ];
    assert_nested_round_trip(&source, &["{r8 @A re r7+9 @B}"], &tree);
    for kind in ["header", "footer"] {
        let note = format!("{{{kind}1.r8 @A re {kind}1.r7+9 @B}}");
        assert_story_nested_round_trip(&source, kind, &[&note], &tree);
    }
}

/// Two nests in one mark, then the change's next mark with a nest of its
/// own (#1353 review, F1): `r7+8` has two ids, and the piece after the
/// nests takes the second, so `x` and `y` stay in mark 7 with `a` and
/// mark 8 holds `b` and `z`, as the source does. The spare id goes to the
/// piece after a nest before it splits two nests apart.
#[test]
fn two_nests_then_the_next_mark_keep_their_own_marks() {
    let paragraph = r#"<w:p><w:r><w:t xml:space="preserve">K </w:t></w:r>
      <w:ins w:id="7" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:r><w:t>a</w:t></w:r>
      <w:del w:id="9" w:author="Alice" w:date="2026-10-02T10:00:00Z"><w:r><w:delText>x</w:delText></w:r></w:del>
      <w:del w:id="10" w:author="Alice" w:date="2026-10-02T10:00:00Z"><w:r><w:delText>y</w:delText></w:r></w:del></w:ins>
      <w:ins w:id="8" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:r><w:t>b</w:t></w:r>
      <w:del w:id="11" w:author="Alice" w:date="2026-10-02T10:00:00Z"><w:r><w:delText>z</w:delText></w:r></w:del></w:ins>
      <w:r><w:t xml:space="preserve"> T.</w:t></w:r></w:p>"#;
    let source = docx(paragraph);
    let tree = [
        ("w:ins", "7", None),
        ("w:del", "9", Some("7")),
        ("w:del", "10", Some("7")),
        ("w:ins", "8", None),
        ("w:del", "11", Some("8")),
    ];
    assert_nested_round_trip(
        &source,
        &["{--y--}{r9+10+11 @A re r7+8 @B}{++b++}{r7+8 @B}"],
        &tree,
    );
    for kind in ["header", "footer"] {
        let note = format!("{{--z--}}{{{kind}1.r9+10+11 @A re {kind}1.r7+8 @B}}");
        assert_story_nested_round_trip(&source, kind, &[&note], &tree);
    }
}

/// A nest in one mark, then the next mark with two nests (#1353 review,
/// F1): mark 8 starts at `b` and holds both `y` and `z`.
#[test]
fn a_nest_then_a_mark_with_two_nests_keep_their_own_marks() {
    let paragraph = r#"<w:p><w:r><w:t xml:space="preserve">K </w:t></w:r>
      <w:ins w:id="7" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:r><w:t>a</w:t></w:r>
      <w:del w:id="9" w:author="Alice" w:date="2026-10-02T10:00:00Z"><w:r><w:delText>x</w:delText></w:r></w:del></w:ins>
      <w:ins w:id="8" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:r><w:t>b</w:t></w:r>
      <w:del w:id="10" w:author="Alice" w:date="2026-10-02T10:00:00Z"><w:r><w:delText>y</w:delText></w:r></w:del>
      <w:del w:id="11" w:author="Alice" w:date="2026-10-02T10:00:00Z"><w:r><w:delText>z</w:delText></w:r></w:del></w:ins>
      <w:r><w:t xml:space="preserve"> T.</w:t></w:r></w:p>"#;
    let source = docx(paragraph);
    let tree = [
        ("w:ins", "7", None),
        ("w:del", "9", Some("7")),
        ("w:ins", "8", None),
        ("w:del", "10", Some("8")),
        ("w:del", "11", Some("8")),
    ];
    assert_nested_round_trip(
        &source,
        &["{--x--}{r9+10+11 @A re r7+8 @B}{++b++}{r7+8 @B}"],
        &tree,
    );
    for kind in ["header", "footer"] {
        let note = format!("{{--z--}}{{{kind}1.r9+10+11 @A re {kind}1.r7+8 @B}}");
        assert_story_nested_round_trip(&source, kind, &[&note], &tree);
    }
}

/// The body and the footnotes number their marks apart (#1355): the
/// body's `r7+8` and a footnote's `footnotes.r7+8` are two changes, each
/// of its own two marks.
#[test]
fn a_footnote_change_with_the_body_changes_ids_keeps_its_own_marks() {
    let view = "---\nauthors:\n  B: Bob # 4 revisions, 2026-10-01T10:00:00Z\n---\nA{++x++}{r7+8 @B}{++y++}{r7+8 @B}[^1]\n\n[^1]: note {++m++}{footnotes.r7+8 @B}{++n++}{footnotes.r7+8 @B}\n";
    let round = written(view);
    let bob = revision_by("Bob", Some("2026-10-01T10:00:00Z"));
    for part in ["document", "footnotes"] {
        let xml = part_string(&round, &format!("word/{part}.xml")).unwrap();
        assert_revision_tree(
            &xml,
            &[("w:ins", "7", None), ("w:ins", "8", None)],
            &[bob.clone(), bob.clone()],
        );
    }
    let back = agent(&round);
    assert!(
        back.contains("A{++x++}{r7+8 @B}{++y++}{r7+8 @B}[^1]"),
        "{back}"
    );
    assert!(
        back.contains("[^1]: note {++m++}{footnotes.r7+8 @B}{++n++}{footnotes.r7+8 @B}\n"),
        "{back}"
    );
}

/// A story substitution retains each side's metadata and uses consecutive notes.
#[test]
fn review_1128_story_substitution_keeps_both_authors_and_dates() {
    let source = docx(
        r#"<w:p><w:ins w:id="1" w:author="Bob" w:date="2026-09-30T09:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins>
        <w:del w:id="2" w:author="Alice" w:date="2026-09-29T14:05:00Z"><w:r><w:delText>old</w:delText></w:r></w:del></w:p>"#,
    );
    for kind in ["header", "footer"] {
        let source = in_story(&source, kind);
        let view = agent(&source);
        let round = written(&view);
        let part = format!("word/{kind}1.xml");
        assert_eq!(
            revision_attributes(&part_string(&round, &part).unwrap()),
            revision_attributes(&part_string(&source, &part).unwrap()),
            "{view}"
        );
        assert!(
            view.contains(&format!(
                "{{++new++}}{{{kind}1.r1 @B}}{{--old--}}{{{kind}1.r2 @A}}"
            )),
            "{view}"
        );
    }
}

/// A same-author replacement in the body keeps each side's date: the settled
/// dialect pairs carriers only when their dates agree (#1142). Insertion
/// first, it is one change (#1359) whose pieces print in document order,
/// each with its own date, and write back as the two marks.
#[test]
fn review_1128_body_same_author_replacement_keeps_both_dates() {
    let source = docx(
        r#"<w:p><w:ins w:id="1" w:author="Alice" w:date="2026-09-30T09:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins>
        <w:del w:id="2" w:author="Alice" w:date="2026-09-29T14:05:00Z"><w:r><w:delText>old</w:delText></w:r></w:del></w:p>"#,
    );
    let view = read(
        &source,
        &ReadOptions {
            dates: true,
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .unwrap()
    .markdown;
    assert!(
        view.contains(
            "{++new++}{r1+2 @A 2026-09-30T09:00:00Z}{--old--}{r1+2 @A 2026-09-29T14:05:00Z}"
        ),
        "{view}"
    );
    let round = written(&view);
    assert_eq!(
        revision_attributes(&part_string(&round, "word/document.xml").unwrap()),
        revision_attributes(&part_string(&source, "word/document.xml").unwrap()),
        "{view}"
    );
}

#[test]
fn review_1128_nested_story_notes_are_not_dropped_comments() {
    for kind in ["header", "footer"] {
        let nested = format!("{{--old--}}{{{kind}1.r4 @BO re {kind}1.r3 @AL}}");
        let comment = "{>>c0 @AL: Please check this.<<}";
        for (text, warns) in [
            (comment.to_string(), true),
            (format!("{nested}{comment}"), true),
            (nested, false),
        ] {
            let view = format!(
                "---\nauthors:\n  AL: Alice # 1 revision, 2026-09-29T14:05:00Z\n  BO: Bob # 1 revision, 2026-09-30T09:00:00Z\n{kind}s:\n  default: {{id: {kind}1, text: \"{text}\"}}\n---\nBody.\n"
            );
            let result = markdown_to_docx(&view, &DocxOptions::default()).unwrap();
            assert_eq!(
                result
                    .warnings
                    .iter()
                    .any(|warning| warning.contains("comment in")),
                warns,
                "{kind}: {text}: {:?}",
                result.warnings
            );
        }
    }
}

fn story_keeps_its_three_level_chain(kind: &str) {
    let source = docx(
        r#"<w:p><w:ins w:id="1" w:author="Alice" w:date="2026-09-29T14:05:00Z">
     <w:r><w:t>liveA </w:t></w:r>
     <w:del w:id="3" w:author="Bob" w:date="2026-09-30T09:00:00Z">
       <w:ins w:id="5" w:author="Alice" w:date="2026-09-29T14:05:00Z"><w:r><w:delText>x</w:delText></w:r></w:ins>
       <w:r><w:delText> mid </w:delText></w:r>
       <w:ins w:id="7" w:author="Alice" w:date="2026-09-29T14:05:00Z"><w:r><w:delText>y</w:delText></w:r></w:ins>
     </w:del>
     <w:r><w:t>tail</w:t></w:r></w:ins></w:p>"#,
    );
    // Feed the note grammar directly to pin the writer's metadata; the
    // reader's side of the same chain is `story_reads_its_three_level_chain`.
    let text = "{++liveA ++}{PART.r1 @A}{++x++}{PART.r5 @A re PART.r3 @B re PART.r1 @A}{-- mid --}{PART.r3 @B re PART.r1 @A}{++y++}{PART.r7 @A re PART.r3 @B re PART.r1 @A}{++tail++}{PART.r1 @A}".replace("PART", &format!("{kind}1"));
    let view = format!(
        "---\nauthors:\n  A: Alice # 3 revisions, 2026-09-29T14:05:00Z\n  B: Bob # 1 revision, 2026-09-30T09:00:00Z\n{kind}s:\n  default: {{id: {kind}1, text: \"{text}\"}}\n---\nBody.\n"
    );
    let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
    assert_revision_tree(
        &xml,
        &[
            ("w:ins", "1", None),
            ("w:del", "3", Some("1")),
            ("w:ins", "5", Some("3")),
            ("w:ins", "7", Some("3")),
        ],
        &revision_attributes(&part_string(&source, "word/document.xml").unwrap()),
    );
}

#[test]
fn a_header_keeps_its_three_level_chain() {
    story_keeps_its_three_level_chain("header");
}

#[test]
fn a_footer_keeps_its_three_level_chain() {
    story_keeps_its_three_level_chain("footer");
}

// #1134: the inner insertions hold `w:delText`, the shape Word opens. The
// story reader must keep the enclosing deletion while it visits them.
fn story_reads_its_three_level_chain(kind: &str) {
    let source = docx(
        r#"<w:p><w:ins w:id="1" w:author="Alpha Lambda" w:date="2026-09-29T14:05:00Z">
     <w:r><w:t>liveA </w:t></w:r>
     <w:del w:id="3" w:author="Beta Omicron" w:date="2026-09-30T09:00:00Z">
       <w:ins w:id="5" w:author="Chi Delta" w:date="2026-09-30T11:00:00Z"><w:r><w:delText>x</w:delText></w:r></w:ins>
       <w:r><w:delText> mid </w:delText></w:r>
       <w:ins w:id="7" w:author="Chi Delta" w:date="2026-09-30T11:00:00Z"><w:r><w:delText>y</w:delText></w:r></w:ins>
     </w:del>
     <w:r><w:t>tail</w:t></w:r></w:ins></w:p>"#,
    );
    let notes = [
        format!("{{{kind}1.r5 @CD re {kind}1.r3 @BO re {kind}1.r1 @AL}}"),
        format!("{{{kind}1.r3 @BO re {kind}1.r1 @AL}}"),
        format!("{{{kind}1.r7 @CD re {kind}1.r3 @BO re {kind}1.r1 @AL}}"),
    ];
    assert_story_nested_round_trip(
        &source,
        kind,
        &notes.iter().map(String::as_str).collect::<Vec<_>>(),
        &[
            ("w:ins", "1", None),
            ("w:del", "3", Some("1")),
            ("w:ins", "5", Some("3")),
            ("w:ins", "7", Some("3")),
        ],
    );
}

#[test]
fn a_header_reads_its_three_level_chain() {
    story_reads_its_three_level_chain("header");
}

#[test]
fn a_footer_reads_its_three_level_chain() {
    story_reads_its_three_level_chain("footer");
}

fn review_1129_story_view(kind: &str, text: &str) -> String {
    format!(
        "---\nauthors:\n  A: Alice # 2 revisions, 2026-09-29..2026-09-30\n  B: Bob # 2 revisions, 2026-09-29..2026-09-30\n{kind}s:\n  default: {{id: {kind}1, text: \"{}\", style: Normal}}\n---\nBody.\n",
        text.replace("PART", &format!("{kind}1"))
    )
}

#[test]
fn review_1129_anonymous_parents_at_every_level() {
    for kind in ["header", "footer"] {
        let view = review_1129_story_view(
            kind,
            "{++live++}{PART.noid @A}{--mid--}{PART.noid @B re PART.noid @A}{++x++}{PART.r5 @A re PART.noid @B re PART.noid @A}{--tail--}{PART.noid @B re PART.noid @A}{++end++}{PART.noid @A}",
        );
        let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
        assert_eq!(xml.matches("<w:ins ").count(), 2, "{xml}");
        assert_eq!(xml.matches("<w:del ").count(), 1, "{xml}");
        let ids = xml
            .split("w:id=\"")
            .skip(1)
            .map(|s| s.split('"').next().unwrap())
            .collect::<Vec<_>>();
        assert_revision_tree(
            &xml,
            &[
                ("w:ins", ids[0], None),
                ("w:del", ids[1], Some(ids[0])),
                ("w:ins", "5", Some(ids[1])),
            ],
            &[
                revision_by("Alice", Some("2026-09-29T00:00:00Z")),
                revision_by("Bob", Some("2026-09-29T00:00:00Z")),
                revision_by("Alice", Some("2026-09-30T23:59:59Z")),
            ],
        );
    }
}

#[test]
fn review_1129_day_range_reuses_opens_at_every_level() {
    for kind in ["header", "footer"] {
        let view = review_1129_story_view(
            kind,
            "{++live++}{PART.r1 @A}{--mid--}{PART.r3 @B re PART.r1 @A}{++x++}{PART.r5 @A re PART.r3 @B re PART.r1 @A}{--tail--}{PART.r3 @B re PART.r1 @A}{++end++}{PART.r1 @A}",
        );
        let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
        assert_revision_tree(
            &xml,
            &[
                ("w:ins", "1", None),
                ("w:del", "3", Some("1")),
                ("w:ins", "5", Some("3")),
            ],
            &[
                revision_by("Alice", Some("2026-09-29T00:00:00Z")),
                revision_by("Bob", Some("2026-09-29T00:00:00Z")),
                revision_by("Alice", Some("2026-09-30T23:59:59Z")),
            ],
        );
    }
}

#[test]
fn review_1129_repeated_parents_stop_the_chain() {
    for kind in ["header", "footer"] {
        for (note, expected) in [
            (
                "PART.r5 @A re PART.r3 @B re PART.r3 @B",
                vec![("w:del", "3", None), ("w:ins", "5", Some("3"))],
            ),
            ("PART.r5 @A re PART.r5 @A", vec![("w:ins", "5", None)]),
            (
                "PART.r5 @A re PART.r3 @B re PART.r1 @A re PART.r3 @B",
                vec![
                    ("w:ins", "1", None),
                    ("w:del", "3", Some("1")),
                    ("w:ins", "5", Some("3")),
                ],
            ),
            (
                "PART.r5+6 @A re PART.r3 @B re PART.r6 @A",
                vec![
                    ("w:del", "3", None),
                    ("w:ins", "5", Some("3")),
                    ("w:ins", "6", Some("3")),
                ],
            ),
        ] {
            let view = review_1129_story_view(kind, &format!("{{++xy++}}{{{note}}}"))
                .replace("2026-09-29..2026-09-30", "2026-09-29T14:05:00Z");
            let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
            assert!(
                !xml.contains("<w:ins w:id=\"3\""),
                "repeated parent changed kind: {xml}"
            );
            let attributes = expected
                .iter()
                .map(|(_, id, _)| {
                    revision_by(
                        if *id == "3" { "Bob" } else { "Alice" },
                        Some("2026-09-29T14:05:00Z"),
                    )
                })
                .collect::<Vec<_>>();
            assert_revision_tree(&xml, &expected, &attributes);
        }
    }
}

fn review_1129_fixed_story_view(kind: &str, text: &str) -> String {
    review_1129_story_view(kind, text).replace("2026-09-29..2026-09-30", "2026-09-29T14:05:00Z")
}

fn review_1129_four_level_source() -> &'static str {
    r#"<w:ins w:id="1" w:author="Alice" w:date="2026-09-29T14:05:00Z"><w:r><w:t>live</w:t></w:r><w:del w:id="3" w:author="Bob" w:date="2026-09-29T14:05:00Z"><w:ins w:id="5" w:author="Alice" w:date="2026-09-29T14:05:00Z"><w:r><w:delText>head</w:delText></w:r><w:del w:id="7" w:author="Bob" w:date="2026-09-29T14:05:00Z"><w:r><w:delText>x</w:delText></w:r></w:del><w:r><w:delText>tail</w:delText></w:r></w:ins></w:del><w:r><w:t>end</w:t></w:r></w:ins>"#
}

#[test]
fn review_1129_four_level_chain_keeps_metadata() {
    for kind in ["header", "footer"] {
        let view = review_1129_fixed_story_view(
            kind,
            "{++live++}{PART.r1 @A}{++head++}{PART.r5 @A re PART.r3 @B re PART.r1 @A}{--x--}{PART.r7 @B re PART.r5 @A re PART.r3 @B re PART.r1 @A}{++tail++}{PART.r5 @A re PART.r3 @B re PART.r1 @A}{++end++}{PART.r1 @A}",
        );
        let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
        assert_revision_tree(
            &xml,
            &[
                ("w:ins", "1", None),
                ("w:del", "3", Some("1")),
                ("w:ins", "5", Some("3")),
                ("w:del", "7", Some("5")),
            ],
            &revision_attributes(review_1129_four_level_source()),
        );
        for payload in [">live<", ">head<", ">x<", ">tail<", ">end<"] {
            assert_eq!(xml.matches(payload).count(), 1, "{xml}");
        }
    }
}

#[test]
fn review_1129_tree_checks_metadata_at_every_level() {
    let source = review_1129_four_level_source();
    let expected = [
        ("w:ins", "1", None),
        ("w:del", "3", Some("1")),
        ("w:ins", "5", Some("3")),
        ("w:del", "7", Some("5")),
    ];
    let attributes = revision_attributes(source);
    assert_revision_tree(source, &expected, &attributes);
    for id in ["1", "3", "5", "7"] {
        for (attribute, wrong) in [("w:author", "changed"), ("w:date", "2026-09-30T14:05:00Z")] {
            let prefix = format!("w:id=\"{id}\"");
            let start = source.find(&prefix).unwrap();
            let target = format!("{attribute}=\"");
            let value_start = start + source[start..].find(&target).unwrap() + target.len();
            let value_end = value_start + source[value_start..].find('"').unwrap();
            let mut changed = source.to_string();
            changed.replace_range(value_start..value_end, wrong);
            assert!(
                std::panic::catch_unwind(|| assert_revision_tree(&changed, &expected, &attributes))
                    .is_err(),
                "tree check accepted {attribute} corruption at r{id}"
            );
        }
    }
}

#[test]
fn issue_1134_pins_body_and_story_sole_intermediate_difference() {
    // #1134 owns the depth decision. The same note grammar currently rebuilds
    // only the nearest parent in the body and the whole chain in a story.
    let text = "{++x++}{PART.r5 @A re PART.r3 @B re PART.r1 @A}{++tail++}{PART.r1 @A}";
    let story_tree = [
        ("w:ins", "1", None),
        ("w:del", "3", Some("1")),
        ("w:ins", "5", Some("3")),
    ];
    let body_tree = [
        ("w:del", "3", None),
        ("w:ins", "5", Some("3")),
        ("w:ins", "1", None),
    ];
    let a = revision_by("Alice", Some("2026-09-29T14:05:00Z"));
    let b = revision_by("Bob", Some("2026-09-29T14:05:00Z"));
    for kind in ["header", "footer"] {
        let view = review_1129_fixed_story_view(kind, text);
        let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
        assert_revision_tree(&xml, &story_tree, &[a.clone(), b.clone(), a.clone()]);
    }
    let view = format!(
        "---\nauthors:\n  A: Alice # 2 revisions, 2026-09-29T14:05:00Z\n  B: Bob # 1 revision, 2026-09-29T14:05:00Z\n---\n{}\n",
        text.replace("PART.", "")
    );
    let xml = part_string(&written(&view), "word/document.xml").unwrap();
    assert_revision_tree(&xml, &body_tree, &[b, a.clone(), a]);
}

#[test]
fn review_1129_field_and_substitution_between_chains() {
    for kind in ["header", "footer"] {
        for separator in ["{PAGE}1{/}", "{~~old~>new~~}{PART.r10 @B}{PART.r11 @A}"] {
            let first = "{--x--}{PART.r7 @B re PART.r5 @A re PART.r3 @B re PART.r1 @A}";
            let second = first
                .replace("r7", "r27")
                .replace("r5", "r25")
                .replace("r3", "r23")
                .replace("r1", "r21");
            let view = review_1129_fixed_story_view(kind, &format!("{first}{separator}{second}"));
            let xml = part_string(&written(&view), &format!("word/{kind}1.xml")).unwrap();
            let mut expected = vec![
                ("w:ins", "1", None),
                ("w:del", "3", Some("1")),
                ("w:ins", "5", Some("3")),
                ("w:del", "7", Some("5")),
            ];
            let mut attributes = revision_attributes(review_1129_four_level_source());
            if separator.starts_with("{~~") {
                expected.extend([("w:del", "10", None), ("w:ins", "11", None)]);
                attributes.extend([
                    revision_by("Bob", Some("2026-09-29T14:05:00Z")),
                    revision_by("Alice", Some("2026-09-29T14:05:00Z")),
                ]);
                assert!(xml.contains(">new<") && xml.contains(">old<"), "{xml}");
                let end = xml.find("</w:ins>").unwrap();
                assert!(xml[end..].contains("<w:ins w:id=\"11\""), "{xml}");
            } else {
                assert!(xml.contains("<w:fldSimple w:instr=\" PAGE \""), "{xml}");
                assert!(
                    xml.find("</w:ins>").unwrap() < xml.find("<w:fldSimple").unwrap(),
                    "{xml}"
                );
                assert!(
                    xml.find("</w:fldSimple>").unwrap() < xml.find("<w:ins w:id=\"21\"").unwrap(),
                    "{xml}"
                );
            }
            expected.extend([
                ("w:ins", "21", None),
                ("w:del", "23", Some("21")),
                ("w:ins", "25", Some("23")),
                ("w:del", "27", Some("25")),
            ]);
            attributes.extend(revision_attributes(review_1129_four_level_source()));
            assert_revision_tree(&xml, &expected, &attributes);
        }
    }
}

#[test]
fn review_1129_substitution_keeps_parent_chains_on_the_old_side_only() {
    for parents in ["PART.r3 @B", "PART.r3 @B re PART.r1 @A"] {
        let text = format!("{{~~old~>new~~}}{{PART.r5+6 @A re {parents}}}");
        for kind in ["header", "footer", "body"] {
            let view = if kind == "body" {
                format!(
                    "---\nauthors:\n  A: Alice # 2 revisions, 2026-09-29T14:05:00Z\n  B: Bob # 1 revision, 2026-09-29T14:05:00Z\n---\n{}",
                    text.replace("PART.", "")
                )
            } else {
                review_1129_fixed_story_view(kind, &text)
            };
            let part = if kind == "body" {
                "word/document.xml".to_string()
            } else {
                format!("word/{kind}1.xml")
            };
            let xml = part_string(&written(&view), &part).unwrap();
            let mut tree = Vec::new();
            let mut authors = Vec::new();
            // #1134 retains only the nearest body parent; stories keep
            // the complete alternating chain.
            let full_chain = kind != "body" && parents.contains("r1");
            if full_chain {
                tree.push(("w:del", "1", None));
                authors.push(revision_by("Alice", Some("2026-09-29T14:05:00Z")));
            }
            tree.extend([
                ("w:ins", "3", full_chain.then_some("1")),
                ("w:del", "5", Some("3")),
                ("w:ins", "6", None),
            ]);
            authors.extend([
                revision_by("Bob", Some("2026-09-29T14:05:00Z")),
                revision_by("Alice", Some("2026-09-29T14:05:00Z")),
                revision_by("Alice", Some("2026-09-29T14:05:00Z")),
            ]);
            assert_revision_tree(&xml, &tree, &authors);
            assert!(
                xml.contains(">old</w:delText>") && xml.contains(">new</w:t>"),
                "{xml}"
            );
            let insertion = xml
                .split("<w:ins w:id=\"6\"")
                .nth(1)
                .unwrap()
                .split("</w:ins>")
                .next()
                .unwrap();
            assert!(
                !insertion.contains("<w:del") && !insertion.contains("<w:ins"),
                "{xml}"
            );
        }
    }
}

// #1134: Word refuses live text anywhere below a deletion (2026-10-17).
fn deletion_text_probe(kind: &str, content: &str) {
    let tag = if kind == "body" {
        String::new()
    } else {
        format!("{kind}1.")
    };
    let marked = format!("Lead {{++{content}++}}{{{tag}r5 @CD re {tag}r3 @BO}} tail.");
    let markdown = if kind == "body" {
        marked
    } else {
        agent(&in_story(
            &docx("<w:p><w:r><w:t>Lead x tail.</w:t></w:r></w:p>"),
            kind,
        ))
        .replace("Lead x tail.", &format!("\"{marked}\""))
    };
    let output = written(&markdown);
    let part = if kind == "body" {
        "word/document.xml".to_string()
    } else {
        format!("word/{kind}1.xml")
    };
    let xml = part_string(&output, &part).unwrap();
    if content.contains("{PAGE}") {
        assert!(xml.contains("<w:delInstrText"), "{xml}");
        assert!(!xml.contains("<w:instrText"), "{xml}");
    }
    assert!(xml.contains("<w:delText"), "{xml}");
    assert!(
        !xml.contains("<w:t xml:space=\"preserve\">x</w:t>"),
        "{xml}"
    );
    let checked = check_word_valid_package(&output);
    assert!(checked.ok(), "{:?}", checked.errors);
}

#[test]
fn issue_1134_body_deletion_text() {
    deletion_text_probe("body", "x");
}
#[test]
fn issue_1134_header_deletion_text() {
    deletion_text_probe("header", "x");
}
#[test]
fn issue_1134_footer_deletion_text() {
    deletion_text_probe("footer", "x");
}
#[test]
fn issue_1134_body_deletion_field() {
    deletion_text_probe("body", "{PAGE}x{/}");
}
#[test]
fn issue_1134_header_deletion_field() {
    deletion_text_probe("header", "{PAGE}x{/}");
}
#[test]
fn issue_1134_footer_deletion_field() {
    deletion_text_probe("footer", "{PAGE}x{/}");
}

// #1137: one view id can name several marks, but each written element
// needs an OOXML id unique within its element kind. These are synthetic,
// in-memory packages.
fn issue_1137_view(body: &str) -> String {
    format!("---\nauthors:\n  A: Alice\n  B: Bob\n---\n{body}\n")
}

fn issue_1137_checked(
    markdown: &str,
    part: &str,
    ins: usize,
    del: usize,
) -> jubarte::markdown::WrittenDocx {
    let output = markdown_to_docx(markdown, &DocxOptions::default()).unwrap();
    let checked = check_word_valid_package(&output.docx);
    assert!(checked.ok(), "{:?}", checked.errors);
    let package = jubarte::opc::PartFs::open(&output.docx).unwrap();
    for name in package
        .parts()
        .into_iter()
        .filter(|name| name.ends_with(".xml"))
    {
        let xml = package.part_string(&name).unwrap();
        let mut dom = jubarte::xmllinq::Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        let mut ids = std::collections::HashSet::new();
        for node in dom.descendants(root, None) {
            let Some(name) = dom.name(node) else { continue };
            if name.namespace_name() == common::docx::W_NS
                && matches!(
                    name.local_name(),
                    "ins"
                        | "del"
                        | "moveFrom"
                        | "moveTo"
                        | "rPrChange"
                        | "pPrChange"
                        | "sectPrChange"
                        | "tblPrChange"
                        | "tblGridChange"
                        | "trPrChange"
                        | "tcPrChange"
                        | "cellIns"
                        | "cellDel"
                        | "cellMerge"
                        | "numberingChange"
                )
            {
                let id = dom
                    .attribute(node, &jubarte::namespaces::W::name("id"))
                    .unwrap();
                assert!(
                    ids.insert((name, id.to_string())),
                    "duplicate {id} in {xml}"
                );
            }
        }
    }
    let xml = part_string(&output.docx, part).unwrap();
    assert_eq!(xml.matches("<w:ins ").count(), ins, "{xml}");
    assert_eq!(xml.matches("<w:del ").count(), del, "{xml}");
    output
}

#[test]
fn issue_1137_adjacent_deletions_merge_and_round_trip() {
    let out = issue_1137_checked(
        &issue_1137_view("{--a--}{r1 @A}{--b--}{r1 @A}"),
        "word/document.xml",
        0,
        1,
    );
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let view = plain(&out.docx);
    assert!(view.contains("{--ab--}{r1 Alice"), "{view}");
    assert_eq!(plain(&written(&view)), view);
}

#[test]
fn issue_1137_adjacent_insertions_merge_across_formatting() {
    let out = issue_1137_checked(
        &issue_1137_view("{++a **bold**++}{r1 @A}{++ b++}{r1 @A}"),
        "word/document.xml",
        1,
        0,
    );
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let xml = part_string(&out.docx, "word/document.xml").unwrap();
    assert!(xml.contains("<w:b/>") && xml.contains(">bold<"), "{xml}");
    let view = plain(&out.docx);
    assert_eq!(view.matches("{r1 ").count(), 1, "{view}");
    assert_eq!(plain(&written(&view)), view);
}

#[test]
fn issue_1137_nested_deletion_keeps_one_outer_insertion() {
    let out = issue_1137_checked(
        &issue_1137_view("{++a++}{r1 @A}{--x--}{r3 @B re r1 @A}{++b++}{r1 @A}"),
        "word/document.xml",
        1,
        1,
    );
    let xml = part_string(&out.docx, "word/document.xml").unwrap();
    assert!(
        xml.find("<w:ins ").unwrap() < xml.find("<w:del ").unwrap(),
        "{xml}"
    );
    assert!(
        xml.find("</w:del>").unwrap() < xml.find("</w:ins>").unwrap(),
        "{xml}"
    );
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let view = plain(&out.docx);
    assert_eq!(plain(&written(&view)), view);
}

#[test]
fn issue_1137_separate_deletions_rename_and_round_trip() {
    let out = issue_1137_checked(
        &issue_1137_view("{--a--}{r1 @A} x {--b--}{r1 @A}"),
        "word/document.xml",
        0,
        2,
    );
    assert_eq!(
        out.warnings,
        ["r1 names two separate changes on line 6; the second is written as r2"]
    );
    let view = plain(&out.docx);
    assert!(
        view.contains("{--a--}{r1 Alice") && view.contains("{--b--}{r2 Alice"),
        "{view}"
    );
    assert_eq!(plain(&written(&view)), view);
}

#[test]
fn issue_1137_separate_insertions_reserve_later_ids_and_report_lines() {
    let out = issue_1137_checked(
        &issue_1137_view("{++a++}{r1 @A}\n x {++b++}{r1 @A} {++c++}{r2 @B}\n y {++d++}{r1 @A}"),
        "word/document.xml",
        4,
        0,
    );
    assert_eq!(
        out.warnings,
        [
            "r1 names two separate changes on line 7; the second is written as r3",
            "r1 names two separate changes on line 8; the second is written as r4"
        ]
    );
    let view = plain(&out.docx);
    for id in [1, 2, 3, 4] {
        assert!(view.contains(&format!("{{r{id} ")), "{view}");
    }
    assert_eq!(plain(&written(&view)), view);
}

#[test]
fn issue_1137_same_id_with_different_author_stays_separate() {
    let out = issue_1137_checked(
        &issue_1137_view("{++a++}{r1 @A}{++b++}{r1 @B}"),
        "word/document.xml",
        2,
        0,
    );
    assert_eq!(
        revision_attributes(&part_string(&out.docx, "word/document.xml").unwrap())
            .iter()
            .map(|(author, _)| author.as_str())
            .collect::<Vec<_>>(),
        ["Alice", "Bob"]
    );
    assert_eq!(out.warnings.len(), 1);
}

#[test]
fn issue_1137_same_id_with_different_date_stays_separate() {
    let out = issue_1137_checked(
        &issue_1137_view("{++a++}{r1 @A 2026-10-01T09:00:00Z}{++b++}{r1 @A 2026-10-02T09:00:00Z}"),
        "word/document.xml",
        2,
        0,
    );
    let xml = part_string(&out.docx, "word/document.xml").unwrap();
    assert_eq!(
        revision_attributes(&xml)
            .iter()
            .map(|(_, date)| date.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["2026-10-01T09:00:00Z", "2026-10-02T09:00:00Z"]
    );
    assert_eq!(out.warnings.len(), 1);
}

#[test]
fn issue_1137_same_id_with_different_kind_stays_separate() {
    let out = issue_1137_checked(
        &issue_1137_view("{--a--}{r1 @A}{++b++}{r1 @A}"),
        "word/document.xml",
        1,
        1,
    );
    assert_eq!(out.warnings.len(), 1);
}

#[test]
fn issue_1137_same_id_in_separate_paragraphs_is_renamed() {
    let out = issue_1137_checked(
        &issue_1137_view("{++a++}{r1 @A}\n\n{++b++}{r1 @A}"),
        "word/document.xml",
        2,
        0,
    );
    assert_eq!(
        out.warnings,
        ["r1 names two separate changes on line 8; the second is written as r2"]
    );
}

#[test]
fn issue_1137_header_adjacent_marks_merge() {
    let view = review_1129_fixed_story_view("header", "{++a++}{PART.r1 @A}{++b++}{PART.r1 @A}");
    let out = issue_1137_checked(&view, "word/header1.xml", 1, 0);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
}

#[test]
fn issue_1137_footer_separate_marks_rename_and_round_trip() {
    let view = review_1129_fixed_story_view("footer", "{--a--}{PART.r1 @A} x {--b--}{PART.r1 @A}");
    let out = issue_1137_checked(&view, "word/footer1.xml", 0, 2);
    assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
    assert!(
        out.warnings[0].contains("footer1.r1")
            && out.warnings[0].contains("footer1.r2")
            && out.warnings[0].contains("line"),
        "{:?}",
        out.warnings
    );
    let round = agent(&out.docx);
    assert!(round.contains("footer1.r2"), "{round}");
    let reread = written(&round);
    assert!(check_word_valid_package(&reread).ok());
    let without_hash = |view: &str| {
        view.lines()
            .filter(|line| !line.starts_with("sha256:"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(without_hash(&agent(&reread)), without_hash(&round));
}

#[test]
fn issue_1137_header_same_id_keeps_distinct_authors() {
    let view = review_1129_fixed_story_view("header", "{++a++}{PART.r1 @A}{++b++}{PART.r1 @B}");
    let out = issue_1137_checked(&view, "word/header1.xml", 2, 0);
    let xml = part_string(&out.docx, "word/header1.xml").unwrap();
    assert_eq!(
        revision_attributes(&xml)
            .iter()
            .map(|(author, _)| author.as_str())
            .collect::<Vec<_>>(),
        ["Alice", "Bob"]
    );
    assert_eq!(out.warnings.len(), 1);
}

#[test]
fn issue_1137_footnote_rename_names_its_story() {
    let view =
        issue_1137_view("Lead[^1].\n\n[^1]: {++a++}{footnotes.r1 @A} x {++b++}{footnotes.r1 @A}");
    let out = issue_1137_checked(&view, "word/footnotes.xml", 2, 0);
    assert_eq!(
        out.warnings,
        [
            "footnotes.r1 names two separate changes on line 8; the second is written as footnotes.r2"
        ]
    );
}

#[test]
fn issue_1137_adjacent_pieces_compare_the_written_dates() {
    let view = "---\nauthors:\n  A: Alice # 3 revisions, 2026-10-01..2026-10-02\n---\n{++earlier++}{r2 @A} x {++a++}{r1 @A}{++b++}{r1 @A 2026-10-02T23:59:59Z}\n";
    let out = issue_1137_checked(view, "word/document.xml", 2, 0);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let xml = part_string(&out.docx, "word/document.xml").unwrap();
    assert_eq!(
        revision_attributes(&xml),
        [
            revision_by("Alice", Some("2026-10-01T00:00:00Z")),
            revision_by("Alice", Some("2026-10-02T23:59:59Z"))
        ]
    );
    let view = plain(&out.docx);
    assert_eq!(plain(&written(&view)), view);
}

fn issue_1137_style_view(marks: &str) -> String {
    format!(
        "---\nview: tracked\nauthors:\n  A: Alice\n  B: Bob\nstyle_changes: {{Normal: {marks}}}\n---\nBody.\n"
    )
}

fn issue_1137_style_records(bytes: &[u8]) -> Vec<(String, String, String)> {
    use jubarte::{namespaces::W, xmllinq::Dom};
    let xml = part_string(bytes, "word/styles.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, None)
        .into_iter()
        .filter_map(|node| {
            let name = dom.name(node)?;
            matches!(name.local_name(), "pPrChange" | "rPrChange").then(|| {
                (
                    name.local_name().to_string(),
                    dom.attribute(node, &W::name("id")).unwrap().to_string(),
                    dom.attribute(node, &W::name("author")).unwrap().to_string(),
                )
            })
        })
        .collect()
}

fn issue_1137_agent_without_hash(bytes: &[u8]) -> String {
    agent(bytes)
        .lines()
        .filter(|line| !line.starts_with("sha256:"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn issue_1137_distinct_style_tags_name_distinct_property_records() {
    let out = issue_1137_checked(
        &issue_1137_style_view("styles.r7 @A; styles.r9 @B"),
        "word/styles.xml",
        0,
        0,
    );
    let records = issue_1137_style_records(&out.docx);
    assert_eq!(records.len(), 2, "{records:?}");
    assert_eq!(
        records,
        [
            (
                "pPrChange".to_string(),
                "7".to_string(),
                "Alice".to_string()
            ),
            ("rPrChange".to_string(), "9".to_string(), "Bob".to_string())
        ]
    );
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let view = issue_1137_agent_without_hash(&out.docx);
    assert_eq!(issue_1137_agent_without_hash(&written(&view)), view);
}

#[test]
fn issue_1137_paired_style_records_reach_a_fixed_point() {
    let first = issue_1137_checked(
        &issue_1137_style_view("styles.r1 @A"),
        "word/styles.xml",
        0,
        0,
    );
    assert_eq!(issue_1137_style_records(&first.docx).len(), 2);
    assert!(first.warnings.is_empty(), "{:?}", first.warnings);
    assert_eq!(
        issue_1137_style_records(&first.docx)
            .iter()
            .map(|(_, id, _)| id.as_str())
            .collect::<Vec<_>>(),
        ["1", "1"]
    );
    let view = issue_1137_agent_without_hash(&first.docx);
    let second = issue_1137_checked(&view, "word/styles.xml", 0, 0);
    let records = issue_1137_style_records(&second.docx);
    assert_eq!(records.len(), 2, "{records:?}");
    assert!(second.warnings.is_empty(), "{:?}", second.warnings);
    assert_eq!(issue_1137_agent_without_hash(&second.docx), view);
}

#[test]
fn issue_1137_adjacent_hyperlink_pieces_merge() {
    for mark in ["++", "--"] {
        let view = issue_1137_view(&format!(
            "{{{mark}a{mark}}}{{r1 @A}}{{{mark}[b](https://example.com/){mark}}}{{r1 @A}}{{{mark}c{mark}}}{{r1 @A}}"
        ));
        let (ins, del) = if mark == "++" { (1, 0) } else { (0, 1) };
        let out = issue_1137_checked(&view, "word/document.xml", ins, del);
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        let canonical = plain(&out.docx);
        assert_eq!(plain(&written(&canonical)), canonical);
    }
}

// Existing Word-saved fixture: both EvidenceQuote and Decision have paired ids.
const ISSUE_1137_WORD_STYLES: &[u8] = include_bytes!(
    "corpus/neurotic_docx_bench/corpus/word_based/docx_source/docx_lots_of_comments_addition_redline_addition_v_removal.docx"
);

#[test]
fn issue_1137_reference_keeps_word_style_records_byte_identical() {
    let source = part_string(ISSUE_1137_WORD_STYLES, "word/styles.xml").unwrap();
    let out = markdown_to_docx(
        "Body.",
        &DocxOptions {
            reference: Some(ISSUE_1137_WORD_STYLES),
            ..DocxOptions::default()
        },
    )
    .unwrap();
    assert!(
        part_string(&out.docx, "word/styles.xml").unwrap() == source,
        "reference stylesheet bytes changed"
    );
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
}

#[test]
fn issue_1137_word_style_records_keep_their_shape() {
    let expected = issue_1137_style_records(ISSUE_1137_WORD_STYLES);
    assert_eq!(expected.len(), 4);
    assert_eq!(
        expected
            .iter()
            .map(|(_, id, _)| id.as_str())
            .collect::<Vec<_>>(),
        ["0", "0", "1", "1"]
    );
    let view = agent(ISSUE_1137_WORD_STYLES);
    let out = markdown_to_docx(&view, &DocxOptions::default()).unwrap();
    assert_eq!(issue_1137_style_records(&out.docx), expected);
    assert!(
        !out.warnings.iter().any(|w| w.contains("styles.r")),
        "{:?}",
        out.warnings
    );
}

/// A deletion inside one mark of an insertion never pairs with the
/// insertion's next mark: the substitution's new side would be the
/// change its old side sits in (`r3+4 re r3+4`). The two print apart,
/// the deletion naming its insertion, and write back as they were.
#[test]
fn a_deletion_in_an_insertion_never_pairs_with_more_of_that_insertion() {
    let paragraph = r#"<w:p><w:r><w:t xml:space="preserve">A </w:t></w:r>
      <w:ins w:id="3" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:del w:id="5" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:r><w:delText xml:space="preserve">old </w:delText></w:r></w:del></w:ins>
      <w:ins w:id="4" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#;
    let source = docx(paragraph);
    let view = agent(&source);
    assert!(
        view.contains("A {--old --}{r5 @B re r3+4 @B}{++new++}{r3+4 @B}"),
        "{view}"
    );
    let tree = [
        ("w:ins", "3", None),
        ("w:del", "5", Some("3")),
        ("w:ins", "4", None),
    ];
    assert_nested_round_trip(&source, &["{r5 @B re r3+4 @B}"], &tree);
    for kind in ["header", "footer"] {
        let note = format!("{{{kind}1.r5 @B re {kind}1.r3+4 @B}}");
        assert_story_nested_round_trip(&source, kind, &[&note], &tree);
    }
}

/// A mark of a change can hold nothing but another author's deletion
/// (Bob typed "x" in a session of its own, Zed deleted it): its id goes
/// to the insertion around the nest, so each of the change's four marks
/// writes back with its own id, the paragraph mark's too.
#[test]
fn a_mark_that_holds_only_a_nest_keeps_its_id() {
    use jubarte::changes::list_changes;
    let at = r#"w:author="Bob" w:date="2026-10-01T10:00:00Z""#;
    let paragraph = format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="13" {at}/></w:rPr></w:pPr><w:r><w:t xml:space="preserve">Start </w:t></w:r>
      <w:ins w:id="7" {at}><w:r><w:t xml:space="preserve">a </w:t></w:r></w:ins>
      <w:ins w:id="9" {at}><w:del w:id="8" w:author="Zed" w:date="2026-10-02T10:00:00Z"><w:r><w:delText>x</w:delText></w:r></w:del></w:ins>
      <w:ins w:id="11" {at}><w:r><w:t xml:space="preserve"> b</w:t></w:r></w:ins></w:p>"#
    );
    let source = docx(&paragraph);
    let tree = [
        ("w:ins", "7", None),
        ("w:ins", "9", None),
        ("w:del", "8", Some("9")),
        ("w:ins", "11", None),
    ];
    assert_nested_round_trip(&source, &["{r8 @Z re r7+9+11+13 @B}"], &tree);
    let marks = |doc: &[u8]| -> Vec<(String, String)> {
        list_changes(doc)
            .unwrap()
            .into_iter()
            .map(|c| (c.id, c.text))
            .collect()
    };
    assert_eq!(marks(&written(&agent(&source))), marks(&source));
}
