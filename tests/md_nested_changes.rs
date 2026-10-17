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
