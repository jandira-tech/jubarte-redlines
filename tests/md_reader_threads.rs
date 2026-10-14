// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The reader brings back the comment threads the agent view writes: a
//! reply (`c6 @AS re c5`) hangs off its comment in `commentsExtended.xml`,
//! a resolved comment (`c7 @AC resolved`) is `w15:done`, and each comment
//! keeps the number the view gave it as its `w:id`.

mod common;

use common::docx::part_string;
use common::validity::assert_word_valid_package;
use jubarte::comments::{CommentRecord, list_comments};
use jubarte::markdown::{DocxOptions, markdown_to_docx};

const HEADER: &str = "---\nauthors:\n  AC: Ann Counsel                  # 2026-10-01T09:00:00Z\n  \
                      AS: Arthur Souza Rodrigues       # 2026-10-09T16:13:00Z\n---\n\n";

fn comments(markdown: &str) -> (Vec<u8>, Vec<CommentRecord>) {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    assert_word_valid_package(&docx);
    let records = list_comments(&docx).expect("comments list");
    (docx, records)
}

fn by_id(records: &[CommentRecord], id: u32) -> &CommentRecord {
    records
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("no comment {id} in {records:?}"))
}

#[test]
fn a_reply_hangs_off_its_comment_and_both_keep_their_numbers() {
    let md = format!(
        "{HEADER}Late amounts {{==accrue interest==}}{{>>c5 @AC: Is one percent the cap?<<}}\
         {{>>c6 @AS re c5: Disagree.<<}} at one percent.\n"
    );
    let (docx, records) = comments(&md);
    assert_eq!(records.len(), 2, "{records:?}");
    let first = by_id(&records, 5);
    let reply = by_id(&records, 6);
    assert_eq!(first.author, "Ann Counsel");
    assert_eq!(first.initials.as_deref(), Some("AC"));
    assert_eq!(first.parent, None);
    assert_eq!(reply.author, "Arthur Souza Rodrigues");
    assert_eq!(reply.parent, Some(5));
    assert!(!first.done && !reply.done);
    let extended = part_string(&docx, "word/commentsExtended.xml").expect("commentsExtended.xml");
    assert!(extended.contains("w15:paraIdParent="), "{extended}");
    let comments_xml = part_string(&docx, "word/comments.xml").expect("comments.xml");
    assert!(comments_xml.contains("w14:paraId="), "{comments_xml}");
}

#[test]
fn a_resolved_comment_is_done_with_its_replies() {
    let md = format!(
        "{HEADER}{{==Payment==}}{{>>c7 @AC resolved: Fixed?<<}}{{>>c8 @AS re c7: Yes.<<}} \
         is due.\n\nOther {{==text==}}{{>>c9 @AC: Open.<<}} here.\n"
    );
    let (_, records) = comments(&md);
    assert!(by_id(&records, 7).done);
    assert!(by_id(&records, 8).done, "Word resolves a thread whole");
    assert_eq!(by_id(&records, 8).parent, Some(7));
    assert!(!by_id(&records, 9).done);
}

#[test]
fn a_comment_without_a_number_takes_the_lowest_free_one() {
    let md = format!(
        "{HEADER}{{==One==}}{{>>A note with no head.<<}} and {{==two==}}{{>>c0 @AC: Numbered.<<}} \
         and {{==three==}}{{>>c0 @AS: Number taken.<<}}.\n"
    );
    let (_, records) = comments(&md);
    let mut ids: Vec<u32> = records.iter().map(|c| c.id).collect();
    ids.sort_unstable();
    assert_eq!(ids, [0, 1, 2], "{records:?}");
    assert_eq!(by_id(&records, 0).text, "Numbered.");
}

#[test]
fn no_thread_writes_no_extended_part() {
    let md = format!("{HEADER}{{==One==}}{{>>c3 @AC: Plain.<<}} here.\n");
    let (docx, records) = comments(&md);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, 3);
    assert!(part_string(&docx, "word/commentsExtended.xml").is_none());
}
