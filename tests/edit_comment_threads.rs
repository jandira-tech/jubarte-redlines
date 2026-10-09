// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comment threads: add, list, reply, resolve, edit, delete; every output
//! is Word-valid and survives Accept All.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::comments::list_comments;
use jubarte::document_comparer::accept_revisions;
use jubarte::edit::{EditPlan, apply_plan};

fn plan(json: &str) -> EditPlan {
    EditPlan::from_json(json).unwrap()
}

#[test]
fn reply_and_resolve_round_trip() {
    let source = docx(&para("The cap is 10."));
    let first = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"body:p:0","find":"cap","text":"Too low"}]}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&first.clean);
    let listed = list_comments(&first.clean).unwrap();
    assert_eq!(listed.len(), 1);
    let root = &listed[0];
    assert_eq!(
        (
            root.author.as_str(),
            root.text.as_str(),
            root.done,
            root.parent
        ),
        ("Ann", "Too low", false, None)
    );
    assert_eq!(root.anchor_text, "cap");
    assert_eq!(root.before, "The ");
    assert_eq!(root.after, " is 10.");
    assert_eq!(root.paragraph.as_deref(), Some("body:p:0"));

    let second = apply_plan(
        &first.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Bob","operations":[
        {{"kind":"reply_comment","comment_id":{id},"text":"Agreed"}},
        {{"kind":"resolve_comment","comment_id":{id}}}]}}"#,
            id = root.id
        )),
    )
    .unwrap();
    assert_word_valid_package(&second.clean);
    assert_word_valid_package(&second.redline);
    let thread = list_comments(&second.clean).unwrap();
    assert_eq!(thread.len(), 2);
    assert_eq!(thread[1].parent, Some(root.id));
    assert_eq!(thread[1].author, "Bob");
    assert_eq!(thread[1].anchor_text, "cap");
    assert!(thread[0].done && thread[1].done);
    // The thread rides the redline and Accept All.
    let redline = list_comments(&second.redline).unwrap();
    assert_eq!(redline.len(), 2);
    assert_eq!(redline[1].parent, Some(redline[0].id));
    assert!(redline[0].done && redline[1].done);
    assert_eq!(
        list_comments(&accept_revisions(&second.redline).unwrap())
            .unwrap()
            .len(),
        2
    );
    assert_eq!(second.report.comments_added, 1);
}

#[test]
fn edit_and_delete_leave_a_valid_package() {
    let source = docx(&(para("One.") + &para("Two.")));
    let with = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"body:p:0","through":"body:p:1","text":"Both paragraphs"}]}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&with.clean);
    let listed = list_comments(&with.clean).unwrap();
    let id = listed[0].id;
    assert_eq!(listed[0].anchor_text, "One.\nTwo.");
    assert_eq!(listed[0].paragraph.as_deref(), Some("body:p:0"));

    let edited = apply_plan(
        &with.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Ann","operations":[
        {{"kind":"edit_comment","comment_id":{id},"text":"Both, reworded"}}]}}"#
        )),
    )
    .unwrap();
    assert_word_valid_package(&edited.clean);
    assert_word_valid_package(&edited.redline);
    assert_eq!(
        list_comments(&edited.clean).unwrap()[0].text,
        "Both, reworded"
    );
    // The redline shows the edited comment once, not the old and new ones.
    let red = list_comments(&edited.redline).unwrap();
    assert_eq!(red.len(), 1, "{red:?}");
    assert_eq!(red[0].text, "Both, reworded");

    let deleted = apply_plan(
        &edited.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Ann","operations":[
        {{"kind":"delete_comment","comment_id":{id}}}]}}"#
        )),
    )
    .unwrap();
    assert_word_valid_package(&deleted.clean);
    assert_word_valid_package(&deleted.redline);
    assert!(list_comments(&deleted.clean).unwrap().is_empty());
    let xml = part_string(&deleted.clean, "word/document.xml").unwrap();
    assert!(!xml.contains("commentRangeStart") && !xml.contains("commentReference"));
    // The deleted comment does not come back through the redline.
    assert!(list_comments(&deleted.redline).unwrap().is_empty());
}

#[test]
fn unknown_ids_and_header_anchors_are_refused() {
    let source = docx(&para("x"));
    for op in [
        r#"{"kind":"reply_comment","comment_id":7,"text":"?"}"#,
        r#"{"kind":"resolve_comment","comment_id":7}"#,
        r#"{"kind":"edit_comment","comment_id":7,"text":"?"}"#,
        r#"{"kind":"delete_comment","comment_id":7}"#,
    ] {
        let e = apply_plan(
            &source,
            &plan(&format!(
                r#"{{"schema_version":1,"author":"A","operations":[{op}]}}"#
            )),
        )
        .unwrap_err();
        assert_eq!(e.code, "UNKNOWN_COMMENT", "{op}");
    }
    // `through` must not run backwards.
    let two = docx(&(para("a") + &para("b")));
    let e = apply_plan(
        &two,
        &plan(
            r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"comment","paragraph":"body:p:1","through":"body:p:0","text":"?"}]}"#,
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "INVALID_EDIT");
}

#[test]
fn a_new_comment_writes_the_whole_family_with_para_ids() {
    let source = docx(&para("Alpha beta."));
    let out = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"body:p:0","find":"beta","text":"one\ntwo"}]}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let comments = part_string(&out.clean, "word/comments.xml").unwrap();
    assert!(comments.contains("w14:paraId="), "{comments}");
    for part in [
        "word/commentsExtended.xml",
        "word/commentsIds.xml",
        "word/commentsExtensible.xml",
    ] {
        assert!(part_string(&out.clean, part).is_some(), "{part} missing");
        assert!(part_string(&out.redline, part).is_some(), "{part} missing");
    }
    assert_eq!(list_comments(&out.clean).unwrap()[0].text, "one\ntwo");
}

#[test]
fn through_places_start_in_the_first_and_end_in_the_last_paragraph() {
    let source = docx(&(para("One.") + &para("Mid.") + &para("Two.")));
    let out = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"body:p:0","through":"body:p:2","text":"span"}]}"#,
        ),
    )
    .unwrap();
    let xml = part_string(&out.clean, "word/document.xml").unwrap();
    let paras: Vec<&str> = xml.split("</w:p>").collect();
    assert!(paras[0].contains("commentRangeStart"), "{xml}");
    assert!(!paras[0].contains("commentRangeEnd"), "{xml}");
    assert!(!paras[1].contains("comment"), "{xml}");
    assert!(paras[2].contains("commentRangeEnd"), "{xml}");
    assert!(paras[2].contains("commentReference"), "{xml}");
    assert_eq!(
        list_comments(&out.clean).unwrap()[0].anchor_text,
        "One.\nMid.\nTwo."
    );
}

/// A Word-shaped package: comments with paraIds and the three extended
/// parts already present. A reply keeps the existing rows and adds its own.
#[test]
fn existing_extended_rows_survive_a_reply() {
    use common::docx::{Part, docx_with};
    const W14: &str = "http://schemas.microsoft.com/office/word/2010/wordml";
    const W15: &str = "http://schemas.microsoft.com/office/word/2012/wordml";
    const CID: &str = "http://schemas.microsoft.com/office/word/2016/wordml/cid";
    const CEX: &str = "http://schemas.microsoft.com/office/word/2018/wordml/cex";
    let w = common::docx::W_NS;
    let comments = format!(
        r#"<w:comments xmlns:w="{w}" xmlns:w14="{W14}"><w:comment w:id="0" w:author="Ann" w:date="2026-01-01T00:00:00Z" w:initials="A"><w:p w14:paraId="0A0B0C0D"><w:r><w:annotationRef/></w:r><w:r><w:t>Root</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let ext = format!(
        r#"<w15:commentsEx xmlns:w15="{W15}"><w15:commentEx w15:paraId="0A0B0C0D" w15:done="0"/></w15:commentsEx>"#
    );
    let ids = format!(
        r#"<w16cid:commentsIds xmlns:w16cid="{CID}"><w16cid:commentId w16cid:paraId="0A0B0C0D" w16cid:durableId="12345678"/></w16cid:commentsIds>"#
    );
    let cex = format!(
        r#"<w16cex:commentsExtensible xmlns:w16cex="{CEX}"><w16cex:commentExtensible w16cex:durableId="12345678" w16cex:dateUtc="2026-01-01T00:00:00Z"/></w16cex:commentsExtensible>"#
    );
    let body = r#"<w:p><w:r><w:t xml:space="preserve">Hello </w:t></w:r><w:commentRangeStart w:id="0"/><w:r><w:t>world</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#;
    let source = docx_with(
        body,
        &[
            Part {
                name: "word/comments.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: &comments,
            },
            Part {
                name: "word/commentsExtended.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml",
                rel_type: "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
                xml: &ext,
            },
            Part {
                name: "word/commentsIds.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsIds+xml",
                rel_type: "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds",
                xml: &ids,
            },
            Part {
                name: "word/commentsExtensible.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtensible+xml",
                rel_type: "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible",
                xml: &cex,
            },
        ],
    );
    assert_word_valid_package(&source);
    let listed = list_comments(&source).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].anchor_text, "world");
    assert_eq!(listed[0].before, "Hello ");

    let out = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Bob","operations":[
        {"kind":"reply_comment","comment_id":0,"text":"Yes"}]}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let ids = part_string(&out.clean, "word/commentsIds.xml").unwrap();
    assert!(ids.contains(r#"durableId="12345678""#), "{ids}");
    let cex = part_string(&out.clean, "word/commentsExtensible.xml").unwrap();
    assert!(cex.contains("2026-01-01T00:00:00Z"), "{cex}");
    let ext = part_string(&out.clean, "word/commentsExtended.xml").unwrap();
    assert!(ext.contains(r#"paraIdParent="0A0B0C0D""#), "{ext}");
    let thread = list_comments(&out.clean).unwrap();
    assert_eq!(thread.len(), 2);
    assert_eq!(thread[1].parent, Some(0));
}

#[test]
fn replying_to_a_reply_threads_under_the_root_and_delete_takes_replies() {
    let source = docx(&para("The cap is 10."));
    let first = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"body:p:0","find":"cap","text":"Too low"}]}"#,
        ),
    )
    .unwrap();
    let root = list_comments(&first.clean).unwrap()[0].id;
    let second = apply_plan(
        &first.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Bob","operations":[
        {{"kind":"reply_comment","comment_id":{root},"text":"Why?"}}]}}"#
        )),
    )
    .unwrap();
    let reply = list_comments(&second.clean).unwrap()[1].id;
    let third = apply_plan(
        &second.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Ann","operations":[
        {{"kind":"reply_comment","comment_id":{reply},"text":"Market"}}]}}"#
        )),
    )
    .unwrap();
    assert_word_valid_package(&third.clean);
    let thread = list_comments(&third.clean).unwrap();
    assert_eq!(thread.len(), 3);
    assert_eq!(thread[2].parent, Some(root));

    // Reopen: resolve with done=false clears the flag.
    let reopened = apply_plan(
        &third.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Ann","operations":[
        {{"kind":"resolve_comment","comment_id":{root},"done":true}}]}}"#
        )),
    )
    .unwrap();
    assert!(
        list_comments(&reopened.clean)
            .unwrap()
            .iter()
            .all(|c| c.done)
    );
    let reopened = apply_plan(
        &reopened.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Ann","operations":[
        {{"kind":"resolve_comment","comment_id":{root},"done":false}}]}}"#
        )),
    )
    .unwrap();
    assert!(
        list_comments(&reopened.clean)
            .unwrap()
            .iter()
            .all(|c| !c.done)
    );

    let deleted = apply_plan(
        &third.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Ann","operations":[
        {{"kind":"delete_comment","comment_id":{root}}}]}}"#
        )),
    )
    .unwrap();
    assert_word_valid_package(&deleted.clean);
    assert_word_valid_package(&deleted.redline);
    assert!(list_comments(&deleted.clean).unwrap().is_empty());
    for part in [
        "word/comments.xml",
        "word/commentsExtended.xml",
        "word/commentsIds.xml",
        "word/commentsExtensible.xml",
    ] {
        assert!(part_string(&deleted.clean, part).is_none(), "{part} kept");
    }
}

#[test]
fn select_comments_filters_by_author_and_latest_per_thread() {
    use jubarte::comments::select_comments;
    let source = docx(&(para("The cap is 10.") + &para("Other.")));
    let first = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"body:p:0","find":"cap","text":"Too low"},
        {"kind":"comment","paragraph":"body:p:1","text":"Fine"}]}"#,
        ),
    )
    .unwrap();
    let root = list_comments(&first.clean).unwrap()[0].id;
    let second = apply_plan(
        &first.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Bob","operations":[
        {{"kind":"reply_comment","comment_id":{root},"text":"Agreed"}}]}}"#
        )),
    )
    .unwrap();
    let all = list_comments(&second.clean).unwrap();
    assert_eq!(all.len(), 3);
    let bob = select_comments(all.clone(), Some("Bob"), false);
    assert_eq!(bob.len(), 1);
    assert_eq!(bob[0].text, "Agreed");
    let latest = select_comments(all.clone(), None, true);
    let texts: Vec<&str> = latest.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(texts, ["Agreed", "Fine"]);
    let none = select_comments(all, Some("Zed"), false);
    assert!(none.is_empty());
}

#[test]
fn list_comments_on_a_document_without_comments_is_empty() {
    assert!(list_comments(&docx(&para("x"))).unwrap().is_empty());
    assert!(list_comments(b"not a zip").is_err());
}

/// A plan without a date stamps the comments and replies it writes with the
/// time it wrote them, in UTC, as `w:date` and as the `dateUtc` Word 365
/// reads. Its tracked changes keep the pinned date.
#[test]
fn undated_plans_stamp_comments_and_replies_with_now() {
    let source = docx(&format!("{}{}", para("The cap is 10."), para("Drop this.")));
    let before = jubarte::convert::utc_now_iso8601();
    let first = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"body:p:0","find":"cap","text":"Too low"},
        {"kind":"replace","paragraph":"body:p:0","find":"10","replacement":"12"},
        {"kind":"delete_paragraph","paragraph":"body:p:1","comment":"Not needed"}]}"#,
        ),
    )
    .unwrap();
    let root = list_comments(&first.clean).unwrap()[0].id;
    let second = apply_plan(
        &first.redline,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Bob","existing_revisions":"keep","operations":[
        {{"kind":"reply_comment","comment_id":{root},"text":"Agreed"}}]}}"#
        )),
    )
    .unwrap();
    let after = jubarte::convert::utc_now_iso8601();
    for package in [&first.clean, &first.redline, &second.clean, &second.redline] {
        let comments = list_comments(package).unwrap();
        assert!(!comments.is_empty());
        let extensible = part_string(package, "word/commentsExtensible.xml").unwrap_or_default();
        for comment in comments {
            let date = comment.date.expect("dated");
            assert!(
                date.len() == 20
                    && date.ends_with('Z')
                    && (before.as_str()..=after.as_str()).contains(&date.as_str()),
                "{date} not in {before}..={after}: {} {:?}",
                comment.text,
                comment.parent
            );
            assert!(
                extensible.contains(&format!(r#"w16cex:dateUtc="{date}""#)),
                "{extensible}"
            );
        }
    }
    assert_eq!(list_comments(&second.clean).unwrap().len(), 3);
    for change in jubarte::changes::list_changes(&second.redline).unwrap() {
        assert_eq!(
            change.date.as_deref(),
            Some(jubarte::document_comparer::DEFAULT_DATE)
        );
    }
    // A plan's own date still dates everything it writes.
    let dated = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","date":"2026-01-02T03:04:05Z","operations":[
        {"kind":"comment","paragraph":"body:p:0","find":"cap","text":"Too low"}]}"#,
        ),
    )
    .unwrap();
    assert_eq!(
        list_comments(&dated.clean).unwrap()[0].date.as_deref(),
        Some("2026-01-02T03:04:05Z")
    );
}
