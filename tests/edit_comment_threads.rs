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
        {"kind":"comment","paragraph":"p0","find":"cap","text":"Too low"}]}"#,
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
    assert_eq!(root.paragraph.as_deref(), Some("p0"));

    // Bob resolves Ann's thread on purpose (#1846).
    let second = apply_plan(
        &first.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Bob","rewrite_others_comments":true,"operations":[
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
        {"kind":"comment","paragraph":"p0","through":"p1","text":"Both paragraphs"}]}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&with.clean);
    let listed = list_comments(&with.clean).unwrap();
    let id = listed[0].id;
    assert_eq!(listed[0].anchor_text, "One.\nTwo.");
    assert_eq!(listed[0].paragraph.as_deref(), Some("p0"));

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
        {"kind":"comment","paragraph":"p1","through":"p0","text":"?"}]}"#,
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
        {"kind":"comment","paragraph":"p0","find":"beta","text":"one\ntwo"}]}"#,
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
        {"kind":"comment","paragraph":"p0","through":"p2","text":"span"}]}"#,
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
        {"kind":"comment","paragraph":"p0","find":"cap","text":"Too low"}]}"#,
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

    // Bob's reply goes with Ann's thread on purpose (#1846).
    let deleted = apply_plan(
        &third.clean,
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Ann","rewrite_others_comments":true,"operations":[
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
        {"kind":"comment","paragraph":"p0","find":"cap","text":"Too low"},
        {"kind":"comment","paragraph":"p1","text":"Fine"}]}"#,
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

/// A plan without a date dates everything it writes (tracked changes,
/// comments, replies) with the time it is applied, in UTC, as `w:date` and as
/// the `dateUtc` Word 365 reads. One plan writes one instant.
#[test]
fn undated_plans_date_everything_they_write_now() {
    let source = docx(&format!("{}{}", para("The cap is 10."), para("Drop this.")));
    let before = jubarte::convert::utc_now_iso8601();
    let first = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"comment","paragraph":"p0","find":"cap","text":"Too low"},
        {"kind":"replace","paragraph":"p0","find":"10","replacement":"12"},
        {"kind":"delete_paragraph","paragraph":"p1","comment":"Not needed"}]}"#,
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
    let changes = jubarte::changes::list_changes(&second.redline).unwrap();
    assert!(!changes.is_empty());
    for change in changes {
        let date = change.date.expect("dated");
        assert!(
            (before.as_str()..=after.as_str()).contains(&date.as_str()),
            "{date} not in {before}..={after}"
        );
    }
    // The clean copy's comment and the redline's changes share one instant.
    let comment_date = list_comments(&first.clean).unwrap()[0].date.clone();
    for change in jubarte::changes::list_changes(&first.redline).unwrap() {
        assert_eq!(change.date, comment_date);
    }
    // A plan's own date still dates everything it writes.
    let dated = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","date":"2026-01-02T03:04:05Z","operations":[
        {"kind":"comment","paragraph":"p0","find":"cap","text":"Too low"}]}"#,
        ),
    )
    .unwrap();
    assert_eq!(
        list_comments(&dated.clean).unwrap()[0].date.as_deref(),
        Some("2026-01-02T03:04:05Z")
    );
}

#[test]
fn a_comment_the_plans_settling_removes_is_refused_with_the_reason() {
    // Ann deletes a paragraph and says why: her comment sits on the deleted
    // text of the redline.
    let deleted = apply_plan(
        &docx(&(para("Drop this.") + &para("Kept."))),
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
        {"kind":"delete_paragraph","paragraph":"p0","comment":"Not needed"}]}"#,
        ),
    )
    .unwrap();
    let id = list_comments(&deleted.redline).unwrap()[0].id;
    let reply = |settle: &str, id: u32| {
        apply_plan(
            &deleted.redline,
            &plan(&format!(
                r#"{{"schema_version":1,"author":"Bob"{settle},"operations":[
        {{"kind":"reply_comment","comment_id":{id},"text":"Agreed"}}]}}"#
            )),
        )
    };
    // Accepting the deletion first takes the comment with it, so a reply
    // cannot land; the refusal says why rather than "no comment".
    let e = reply(r#","existing_revisions":"accept""#, id).unwrap_err();
    assert_eq!(e.code, "UNKNOWN_COMMENT");
    assert!(
        e.message.contains(&format!(
            "c{id} is gone once the plan's revisions are settled"
        )) && e.message.contains("\"removed_anchor_comments\": \"keep\""),
        "{e}"
    );
    // Kept, the reply lands.
    let replied = reply(r#","existing_revisions":"keep""#, id).unwrap();
    assert_word_valid_package(&replied.redline);
    assert_eq!(list_comments(&replied.redline).unwrap().len(), 2);
    // A comment that never existed keeps the plain refusal.
    let e = reply(r#","existing_revisions":"accept""#, 99).unwrap_err();
    assert!(e.message.starts_with("no comment c99"), "{e}");
}

// #1159: all sources are synthetic in-memory packages; process probes live in
// edit_refusal_wording. No wall clock is consulted by these plans.
fn anchor_plan(extra: serde_json::Value, operations: serde_json::Value) -> EditPlan {
    let mut value = serde_json::json!({"schema_version":1,"author":"Alice",
        "date":"2026-10-18T12:05:00Z","existing_revisions":"keep","operations":[]});
    value["operations"] = operations;
    let serde_json::Value::Object(extra) = extra else {
        panic!("plan fields must be an object")
    };
    value.as_object_mut().unwrap().extend(extra);
    {
        let parsed = EditPlan::from_json(&value.to_string());
        assert!(parsed.is_ok(), "{parsed:?}");
        parsed.unwrap()
    }
}

fn inserted_comment() -> Vec<u8> {
    let base = docx(
        &(para("Services")
            + &para("The Provider shall perform the Services.")
            + &para("Next paragraph.")),
    );
    let bob = plan(
        r#"{"schema_version":1,"author":"Bob","date":"2026-10-18T12:00:00Z","operations":[{"kind":"insert","paragraph":"p1","position":"end","text":" Bob added this sentence.","comment":"Bob's reason."}]}"#,
    );
    apply_plan(&base, &bob).unwrap().redline
}

fn anchor_reply() -> serde_json::Value {
    serde_json::json!([{"kind":"reply_comment","comment_id":"c0","text":"Alice disagrees."}])
}
fn anchor_settle(mode: &str, decision: &str) -> serde_json::Value {
    let mut v = serde_json::json!({"removed_anchor_comments":mode,"resolve_revisions":{}});
    v["resolve_revisions"][decision] = serde_json::json!({"ids":["r1"]});
    v
}
fn assert_empty_thread(bytes: &[u8]) {
    assert_word_valid_package(bytes);
    let listed = list_comments(bytes).unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[1].parent, Some(listed[0].id));
    assert!(
        listed.iter().all(|c| c.anchor_text.is_empty()),
        "{listed:?}"
    );
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = jubarte::xmllinq::Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let markers: Vec<_> = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&n| {
            dom.name(n).is_some_and(|name| {
                ["commentRangeStart", "commentRangeEnd", "commentReference"]
                    .contains(&name.local_name())
            })
        })
        .collect();
    assert_eq!(markers.len(), 6, "{xml}");
    let names: Vec<_> = markers
        .iter()
        .map(|&n| dom.name(n).unwrap().local_name().to_string())
        .collect();
    assert_eq!(
        names,
        [
            "commentRangeStart",
            "commentRangeStart",
            "commentRangeEnd",
            "commentRangeEnd",
            "commentReference",
            "commentReference"
        ]
    );
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let parents: Vec<_> = markers
        .iter()
        .map(|&n| {
            let p = dom.parent(n).unwrap();
            if dom.name(p).unwrap().local_name() == "r" {
                dom.parent(p).unwrap()
            } else {
                p
            }
        })
        .collect();
    assert!(parents.iter().all(|&p| p == parents[0]));
    assert_eq!(
        dom.name(parents[0]),
        Some(jubarte::xmllinq::XName::get("p", w))
    );
    let children = dom.elements(parents[0], None);
    let first = children.iter().position(|&n| n == markers[0]).unwrap();
    let adjacent: Vec<_> = children[first..first + 6]
        .iter()
        .map(|&n| dom.name(n).unwrap().local_name().to_string())
        .collect();
    assert_eq!(
        adjacent,
        [
            "commentRangeStart",
            "commentRangeStart",
            "commentRangeEnd",
            "commentRangeEnd",
            "r",
            "r"
        ]
    );
}

#[test]
fn removed_anchor_keep_reply_and_reject_one_plan() {
    let out = apply_plan(
        &inserted_comment(),
        &anchor_plan(anchor_settle("keep", "reject"), anchor_reply()),
    )
    .unwrap();
    assert_empty_thread(&out.redline);
    let report = serde_json::to_value(&out.report).unwrap();
    assert_eq!(
        report["kept_comments"],
        serde_json::json!([{"id":"c0","replies":["c1"],"anchor":"empty"}])
    );
}
#[test]
fn removed_anchor_keep_two_plans() {
    let a = apply_plan(
        &inserted_comment(),
        &anchor_plan(serde_json::json!({}), anchor_reply()),
    )
    .unwrap();
    let b = apply_plan(
        &a.redline,
        &anchor_plan(anchor_settle("keep", "reject"), serde_json::json!([])),
    )
    .unwrap();
    assert_empty_thread(&b.redline);
    assert_eq!(
        serde_json::to_value(&b.report).unwrap()["kept_comments"],
        serde_json::json!([{"id":"c0","replies":["c1"],"anchor":"empty"}])
    );
    for part in [
        "word/comments.xml",
        "word/commentsExtended.xml",
        "word/commentsIds.xml",
        "word/commentsExtensible.xml",
        "word/people.xml",
    ] {
        if let Some(before) = part_string(&a.redline, part) {
            assert_eq!(
                part_string(&b.redline, part).as_deref(),
                Some(before.as_str()),
                "{part}"
            );
        }
    }
}
#[test]
fn removed_anchor_default_reports_thread_and_still_removes_it() {
    let a = apply_plan(
        &inserted_comment(),
        &anchor_plan(serde_json::json!({}), anchor_reply()),
    )
    .unwrap();
    let out = apply_plan(
        &a.redline,
        &anchor_plan(
            serde_json::json!({"resolve_revisions":{"reject":{"ids":["r1"]}}}),
            serde_json::json!([]),
        ),
    )
    .unwrap();
    // Plan 10 adds why the thread went (#1159).
    let expected = serde_json::json!([{"id":"c0","author":"Bob","replies":["c1"],
        "why":"anchor removed by rejecting r1"}]);
    assert_eq!(
        serde_json::to_value(&out.report).unwrap()["removed_comments"],
        expected
    );
    let summary: serde_json::Value =
        serde_json::from_str(out.report.to_jsonl().lines().last().unwrap()).unwrap();
    assert_eq!(summary["removed_comments"], expected);
    assert!(list_comments(&out.redline).unwrap().is_empty());
    assert_word_valid_package(&out.redline);
}
#[test]
fn removed_anchor_unknown_comment_advice_preserves_reply_intent() {
    let e = apply_plan(
        &inserted_comment(),
        &anchor_plan(
            serde_json::json!({"resolve_revisions":{"reject":{"ids":["r1"]}}}),
            anchor_reply(),
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "UNKNOWN_COMMENT");
    assert!(
        e.message.contains("c0")
            && e.message.contains("r1")
            && e.message.contains("Word")
            && e.message.contains("replies")
            && e.message.contains("\"removed_anchor_comments\": \"keep\""),
        "{}",
        e.message
    );
    assert!(!e.message.contains("plan of its own"));
}
#[test]
fn removed_anchor_unknown_policy_is_invalid_plan() {
    let e = EditPlan::from_json(r#"{"schema_version":1,"author":"Alice","removed_anchor_comments":"retain","operations":[]}"#).unwrap_err();
    assert_eq!(e.code, "INVALID_PLAN");
    assert!(
        e.message.contains("retain") && e.message.contains("keep"),
        "{}",
        e.message
    );
}

fn anchor_variant(kind: &str, partial: bool) -> Vec<u8> {
    let a = apply_plan(
        &inserted_comment(),
        &anchor_plan(serde_json::json!({}), anchor_reply()),
    )
    .unwrap();
    let mut pkg = jubarte::opc::PartFs::open(&a.redline).unwrap();
    let tag = if kind == "del" { "delText" } else { "t" };
    let start = r#"<w:commentRangeStart w:id="0"/><w:commentRangeStart w:id="1"/>"#;
    let end = r#"<w:commentRangeEnd w:id="0"/><w:commentRangeEnd w:id="1"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="0"/></w:r><w:r><w:commentReference w:id="1"/></w:r>"#;
    let left = if partial { start } else { "" };
    let inner = if partial { "" } else { start };
    let body = format!(
        r#"<w:p><w:r><w:t>Services</w:t></w:r></w:p><w:p>{left}<w:r><w:t>Before.</w:t></w:r><w:{kind} w:id="1" w:author="Bob" w:date="2026-10-18T12:00:00Z">{inner}<w:r><w:{tag}> Bob added this sentence.</w:{tag}></w:r>{end}</w:{kind}><w:r><w:t> After.</w:t></w:r></w:p>"#
    );
    let xml = format!(
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr/></w:body></w:document>"#
    );
    pkg.set_part("word/document.xml", xml.into_bytes());
    pkg.to_zip().unwrap()
}
#[test]
fn removed_anchor_keep_accept_deletion_and_mid_paragraph() {
    for (kind, decision) in [("del", "accept"), ("ins", "reject")] {
        let out = apply_plan(
            &anchor_variant(kind, false),
            &anchor_plan(anchor_settle("keep", decision), serde_json::json!([])),
        )
        .unwrap();
        assert_empty_thread(&out.redline);
        let xml = part_string(&out.redline, "word/document.xml").unwrap();
        assert!(xml.find("Before.").unwrap() < xml.find("commentRangeStart").unwrap());
        assert!(xml.find("commentReference").unwrap() < xml.find(" After.").unwrap());
    }
}
#[test]
fn removed_anchor_keep_partial_overlap_matches_remove() {
    let source = anchor_variant("ins", true);
    let remove = apply_plan(
        &source,
        &anchor_plan(anchor_settle("remove", "reject"), serde_json::json!([])),
    )
    .unwrap();
    let keep = apply_plan(
        &source,
        &anchor_plan(anchor_settle("keep", "reject"), serde_json::json!([])),
    )
    .unwrap();
    assert_eq!(
        part_string(&remove.redline, "word/document.xml"),
        part_string(&keep.redline, "word/document.xml")
    );
    assert_eq!(
        list_comments(&remove.redline).unwrap(),
        list_comments(&keep.redline).unwrap()
    );
}
#[test]
fn removed_anchor_typed_v2_keep_and_remove() {
    let source = inserted_comment();
    let mut value = serde_json::json!({"schema_version":2,"source":{"sha256":jubarte::inspect::source_sha256(&source)},"policy":{"track":"on","existing_revisions":"keep","text_view":"final"},"resolve_revisions":{"accept":[],"reject":[{"short_id":"r1","long_id":"document.w:ins.w:id.1"}]},"author":"Alice","date":"2026-10-18T12:06:00Z","scope":{"kind":"all"},"operations":[],"removed_anchor_comments":"keep"});
    let typed = jubarte::edit::v2::PlanV2::from_json(&value.to_string());
    assert!(typed.is_ok(), "{typed:?}");
    let typed = typed.unwrap();
    let result = jubarte::edit::v2_apply::apply_plan(&source, &typed).unwrap();
    assert_eq!(list_comments(&result.redline).unwrap().len(), 1);
    assert_eq!(list_comments(&result.redline).unwrap()[0].anchor_text, "");
    assert_eq!(
        serde_json::to_value(&result.report).unwrap()["kept_comments"][0]["id"],
        "c0"
    );
    value["removed_anchor_comments"] = serde_json::json!("remove");
    let typed = jubarte::edit::v2::PlanV2::from_json(&value.to_string());
    assert!(typed.is_ok(), "{typed:?}");
    let typed = typed.unwrap();
    let result = jubarte::edit::v2_apply::apply_plan(&source, &typed).unwrap();
    assert_eq!(
        serde_json::to_value(&result.report).unwrap()["removed_comments"][0]["id"],
        "c0"
    );
}

#[test]
fn removed_anchor_keep_outside_markers_have_one_empty_point() {
    let source = anchor_variant("del", false);
    let mut pkg = jubarte::opc::PartFs::open(&source).unwrap();
    let xml = pkg.part_string("word/document.xml").unwrap();
    let start = r#"<w:commentRangeStart w:id="0"/><w:commentRangeStart w:id="1"/>"#;
    let xml = xml
        .replace(start, "")
        .replace("<w:del w:id=", &format!("{start}<w:del w:id="));
    pkg.set_part("word/document.xml", xml.into_bytes());
    let out = apply_plan(
        &pkg.to_zip().unwrap(),
        &anchor_plan(anchor_settle("keep", "accept"), serde_json::json!([])),
    )
    .unwrap();
    assert_empty_thread(&out.redline);
}
#[test]
fn removed_anchor_keep_moves_and_typed_reply() {
    for (kind, decision) in [("moveFrom", "accept"), ("moveTo", "reject")] {
        let out = apply_plan(
            &anchor_variant(kind, false),
            &anchor_plan(anchor_settle("keep", decision), serde_json::json!([])),
        )
        .unwrap();
        assert_empty_thread(&out.redline);
    }
    let source = inserted_comment();
    let value = serde_json::json!({"schema_version":2,"source":{"sha256":jubarte::inspect::source_sha256(&source)},"policy":{"track":"on","existing_revisions":"keep","text_view":"final"},"resolve_revisions":{"accept":[],"reject":[{"short_id":"r1","long_id":"document.w:ins.w:id.1"}]},"author":"Alice","date":"2026-10-18T12:06:00Z","scope":{"kind":"all"},"operations":[{"kind":"reply_comment","target":{"short_id":"c0","long_id":"comments.w:comment.w:id.0"},"text":"Alice disagrees."}],"removed_anchor_comments":"keep"});
    let typed = jubarte::edit::v2::PlanV2::from_json(&value.to_string()).unwrap();
    let result = jubarte::edit::v2_apply::apply_plan(&source, &typed).unwrap();
    assert_empty_thread(&result.redline);
}

// #1159: c2 must still name the source comment after c0 disappears.
fn settlement_identity_source() -> Vec<u8> {
    use common::docx::{Part, W_NS, docx_with};
    let mut body = String::new();
    let mut definitions = String::new();
    for (id, text) in ["Removed", "First survivor", "Target", "Other"]
        .iter()
        .enumerate()
    {
        let anchor = format!(
            r#"<w:commentRangeStart w:id="{id}"/><w:r><w:t>{text}</w:t></w:r><w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r>"#
        );
        let content = if id == 0 {
            format!(
                r#"<w:ins w:id="1" w:author="Bob" w:date="2026-10-18T12:00:00Z">{anchor}</w:ins>"#
            )
        } else {
            anchor
        };
        body.push_str(&format!("<w:p>{content}</w:p>"));
        definitions.push_str(&format!(r#"<w:comment w:id="{id}" w:author="Carol Diaz" w:date="2026-10-18T12:00:00Z"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#));
    }
    body.push_str(r#"<w:p><w:bookmarkStart w:id="42" w:name="surviving_bookmark"/><w:r><w:t>Bookmarked</w:t></w:r><w:bookmarkEnd w:id="42"/></w:p>"#);
    let comments = format!(r#"<w:comments xmlns:w="{W_NS}">{definitions}</w:comments>"#);
    docx_with(
        &body,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    )
}

fn settlement_identity_plan(operations: &serde_json::Value) -> EditPlan {
    EditPlan::from_json(&serde_json::json!({"schema_version":1,"author":"Alice","date":"2026-10-18T12:06:00Z","existing_revisions":"keep","resolve_revisions":{"reject":{"ids":["r1"]}},"operations":operations}).to_string()).unwrap()
}

#[test]
fn settlement_then_edit_addresses_source_c2() {
    // Carol wrote c2; Alice rewrites it on purpose (#1264).
    let mut plan = settlement_identity_plan(&serde_json::json!([
        {"kind":"edit_comment","comment_id":"c2","text":"Edited target"}
    ]));
    plan.rewrite_others_comments = true;
    let out = apply_plan(&settlement_identity_source(), &plan).unwrap();
    for bytes in [&out.clean, &out.redline] {
        let comments = list_comments(bytes).unwrap();
        let target = comments.iter().find(|c| c.anchor_text == "Target").unwrap();
        assert_eq!(
            target.text, "Edited target",
            "settlement must address the original c2: {comments:?}"
        );
        assert_eq!(
            comments
                .iter()
                .find(|c| c.anchor_text == "Other")
                .unwrap()
                .text,
            "Other"
        );
    }
}

#[test]
fn settlement_then_reply_addresses_source_c2() {
    let out = apply_plan(
        &settlement_identity_source(),
        &settlement_identity_plan(&serde_json::json!([
            {"kind":"reply_comment","comment_id":"c2","text":"Reply to target"}
        ])),
    )
    .unwrap();
    for bytes in [&out.clean, &out.redline] {
        let comments = list_comments(bytes).unwrap();
        let target = comments.iter().find(|c| c.text == "Target").unwrap();
        let reply = comments
            .iter()
            .find(|c| c.text == "Reply to target")
            .unwrap();
        assert_eq!(
            reply.parent,
            Some(target.id),
            "reply must attach to the original c2: {comments:?}"
        );
    }
}

/// The source's ids stay (#1264): c0 goes with r1 unless it is kept, a
/// reply takes the next id after the source's highest (c3), and the
/// bookmark keeps 42.
fn assert_settled_annotation_ids(bytes: &[u8], mode: &str, replies: u32) {
    let mut ids: Vec<_> = list_comments(bytes).unwrap().iter().map(|c| c.id).collect();
    ids.sort_unstable();
    let first = if mode == "keep" { 0 } else { 1 };
    assert_eq!(
        ids,
        (first..4 + replies).collect::<Vec<_>>(),
        "final comment ids"
    );
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = jubarte::xmllinq::Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let bookmarks = dom.descendants(root, Some(&jubarte::namespaces::W::name("bookmarkStart")));
    assert_eq!(bookmarks.len(), 1);
    assert_eq!(
        dom.attribute(bookmarks[0], &jubarte::namespaces::W::id()),
        Some("42"),
        "final bookmark id"
    );
    assert_word_valid_package(bytes);
}

fn final_annotation_probe(mode: &str, typed: bool) {
    let source = settlement_identity_source();
    let (clean, redline) = if typed {
        let value = serde_json::json!({"schema_version":2,"source":{"sha256":jubarte::inspect::source_sha256(&source)},"policy":{"track":"on","existing_revisions":"keep","text_view":"final"},"resolve_revisions":{"accept":[],"reject":[{"short_id":"r1","long_id":"document.w:ins.w:id.1"}]},"author":"Alice","date":"2026-10-18T12:06:00Z","scope":{"kind":"all"},"operations":[],"removed_anchor_comments":mode});
        let plan = jubarte::edit::v2::PlanV2::from_json(&value.to_string()).unwrap();
        let out = jubarte::edit::v2_apply::apply_plan(&source, &plan).unwrap();
        (out.clean, out.redline)
    } else {
        let mut plan = settlement_identity_plan(&serde_json::json!([]));
        plan.removed_anchor_comments = if mode == "keep" {
            jubarte::edit::RemovedAnchorComments::Keep
        } else {
            jubarte::edit::RemovedAnchorComments::Remove
        };
        let out = apply_plan(&source, &plan).unwrap();
        (out.clean, out.redline)
    };
    assert_settled_annotation_ids(&clean, mode, 0);
    assert_settled_annotation_ids(&redline, mode, 0);
}

#[test]
fn settlement_keeps_final_annotation_ids_remove() {
    final_annotation_probe("remove", false);
}
#[test]
fn settlement_keeps_final_annotation_ids_keep() {
    final_annotation_probe("keep", false);
}
#[test]
fn settlement_keeps_final_annotation_ids_v2_remove() {
    final_annotation_probe("remove", true);
}
#[test]
fn settlement_keeps_final_annotation_ids_v2_keep() {
    final_annotation_probe("keep", true);
}

#[test]
fn settlement_keeps_ids_after_a_reply_and_reports_its_id() {
    let out = apply_plan(
        &settlement_identity_source(),
        &settlement_identity_plan(&serde_json::json!([
            {"kind":"reply_comment","comment_id":"c2","text":"Reply to target"}
        ])),
    )
    .unwrap();
    assert_settled_annotation_ids(&out.redline, "remove", 1);
    let comments = list_comments(&out.redline).unwrap();
    let reply = comments
        .iter()
        .find(|c| c.text == "Reply to target")
        .unwrap();
    assert_eq!(out.report.operations[0].comment_id, Some(reply.id));
    assert_eq!(out.report.removed_comments[0].id, "c0");
}

#[test]
fn settlement_preserves_annotation_ids_with_revisions_left() {
    let mut pkg = jubarte::opc::PartFs::open(&settlement_identity_source()).unwrap();
    let xml = pkg.part_string("word/document.xml").unwrap().replace(
        "<w:t>Bookmarked</w:t>",
        r#"<w:t>Bookmarked</w:t></w:r><w:ins w:id="2" w:author="Bob" w:date="2026-10-18T12:00:00Z"><w:r><w:t>pending</w:t></w:r></w:ins><w:r>"#,
    );
    pkg.set_part("word/document.xml", xml.into_bytes());
    let out = apply_plan(
        &pkg.to_zip().unwrap(),
        &settlement_identity_plan(&serde_json::json!([])),
    )
    .unwrap();
    assert_eq!(
        jubarte::changes::list_changes(&out.redline).unwrap()[0].id,
        "r2"
    );
    let ids: Vec<_> = list_comments(&out.redline)
        .unwrap()
        .iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(ids, [1, 2, 3]);
    let xml = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(xml.contains(r#"bookmarkStart w:id="42""#));
    assert_word_valid_package(&out.redline);
}

// #1193: container rescue and receipts use the published document's identities.
fn thread_in_body(body: &str) -> Vec<u8> {
    let mut pkg = jubarte::opc::PartFs::open(&anchor_variant("ins", false)).unwrap();
    pkg.set_part(
        "word/document.xml",
        format!(
            r#"<w:document xmlns:w="{}"><w:body>{body}<w:sectPr/></w:body></w:document>"#,
            common::docx::W_NS
        )
        .into_bytes(),
    );
    pkg.to_zip().unwrap()
}

fn revised_thread(kind: &str, id: u32) -> String {
    let text = if kind == "del" { "delText" } else { "t" };
    format!(
        r#"<w:p><w:{kind} w:id="{id}" w:author="Bob" w:date="2026-10-18T12:00:00Z"><w:commentRangeStart w:id="0"/><w:commentRangeStart w:id="1"/><w:r><w:{text}>Bob added this sentence.</w:{text}></w:r><w:commentRangeEnd w:id="0"/><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="0"/></w:r><w:r><w:commentReference w:id="1"/></w:r></w:{kind}></w:p>"#
    )
}

fn container_thread(kind: &str, container: &str, before: bool) -> Vec<u8> {
    let stamp = r#"w:id="2" w:author="Bob" w:date="2026-10-18T12:00:00Z""#;
    let (row_pr, cell_pr) = if container == "cell" {
        (
            String::new(),
            format!(r#"<w:tcPr><w:cellDel {stamp}/></w:tcPr>"#),
        )
    } else {
        (
            format!("<w:trPr><w:{kind} {stamp}/></w:trPr>"),
            "<w:tcPr/>".into(),
        )
    };
    let previous_row = if container == "row" {
        format!(
            "<w:tr><w:tc><w:tcPr/>{}{}</w:tc></w:tr>",
            para("Previous row first."),
            para("Previous row last.")
        )
    } else {
        String::new()
    };
    let surviving_cell = if container == "cell" {
        format!("<w:tc><w:tcPr/>{}</w:tc>", para("Surviving cell."))
    } else {
        String::new()
    };
    let table = format!(
        r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid>{previous_row}<w:tr>{row_pr}<w:tc>{cell_pr}{}</w:tc>{surviving_cell}</w:tr></w:tbl>"#,
        revised_thread(kind, 1)
    );
    thread_in_body(&format!(
        "{}{table}{}",
        if before {
            para("Before table.")
        } else {
            String::new()
        },
        para("After table.")
    ))
}

// CLI invocations and temp files here are integration tests; library probes stay in memory.
fn run_thread_cli(source: &[u8], plan: &EditPlan, editing: bool) -> (String, Vec<u8>) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("source.docx");
    let request = dir.path().join("plan.json");
    let output = dir.path().join("result.docx");
    std::fs::write(&input, source).unwrap();
    std::fs::write(&request, plan.to_json()).unwrap();
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"));
    command
        .arg("edit")
        .arg(&input)
        .arg("--plan")
        .arg(&request)
        .arg("-o")
        .arg(&output);
    if editing {
        command.arg("--editing-mode");
    }
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let bytes = std::fs::read(output).unwrap();
    (text, bytes)
}

fn container_rescue_probe(kind: &str, container: &str, before: bool) {
    let decision = if kind == "del" { "accept" } else { "reject" };
    let mut extra = anchor_settle("keep", decision);
    extra["resolve_revisions"][decision]["ids"] = serde_json::json!(["r1", "r2"]);
    let source = container_thread(kind, container, before);
    let plan = anchor_plan(extra, serde_json::json!([]));
    let (text, published) = run_thread_cli(&source, &plan, false);
    let listed = list_comments(&published).unwrap();
    assert_eq!(
        listed.len(),
        2,
        "both comments must survive {container} settlement; {text}"
    );
    let expected = if container == "row" {
        "Previous row last."
    } else if before {
        "Before table."
    } else {
        "After table."
    };
    let paragraphs = jubarte::inspect::paragraphs(&published).unwrap();
    let paragraph = paragraphs.iter().find(|p| p.text == expected).unwrap();
    assert!(
        listed
            .iter()
            .all(|c| c.paragraph.as_deref() == Some(paragraph.id.as_str())),
        "anchor must be in {expected}: {listed:?}"
    );
    let out = apply_plan(&source, &plan).unwrap();
    assert!(
        out.report.removed_comments.is_empty(),
        "a kept id must never be reported removed"
    );
    assert_eq!(out.report.kept_comments.len(), 1);
    assert_empty_thread(&published);
    assert_empty_thread(&out.clean);
    let xml = part_string(&published, "word/document.xml").unwrap();
    if before || container == "row" {
        assert!(
            xml.find(expected).unwrap() < xml.find("commentRangeStart").unwrap(),
            "anchor goes at the previous paragraph's end"
        );
    } else {
        assert!(
            xml.find("commentReference").unwrap() < xml.find(expected).unwrap(),
            "anchor goes at the next paragraph's start"
        );
    }
    if let Ok(dir) = std::env::var("JUBARTE_WORD_OUTPUT") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        let name = format!("{container}-{decision}");
        let inputs = dir.parent().unwrap().join("smoke-inputs");
        std::fs::create_dir_all(&inputs).unwrap();
        std::fs::write(inputs.join(format!("{name}.docx")), &source).unwrap();
        std::fs::write(inputs.join(format!("{name}.json")), plan.to_json()).unwrap();
        std::fs::write(dir.join(format!("{name}.docx")), &published).unwrap();
        std::fs::write(
            dir.join(format!("{name}.txt")),
            format!("both balloons, reply threaded; empty anchor in {expected}\n"),
        )
        .unwrap();
    }
}

#[test]
fn rescue_1193_rejected_row() {
    container_rescue_probe("ins", "row", true);
}
#[test]
fn rescue_1193_accepted_row() {
    container_rescue_probe("del", "row", true);
}
#[test]
fn rescue_1193_whole_table() {
    container_rescue_probe("ins", "table", true);
}
#[test]
fn rescue_1193_table_without_preceding_paragraph() {
    container_rescue_probe("ins", "table", false);
}
#[test]
fn rescue_1193_deleted_cell() {
    container_rescue_probe("del", "cell", true);
}

#[test]
fn receipt_1193_comparer_reply_id_matches_both_outputs() {
    let mut plan = settlement_identity_plan(&serde_json::json!([
        {"kind":"reply_comment","comment_id":"c2","text":"Alice replies."},
        {"kind":"insert","paragraph":"p4","position":"end","text":" New text."}
    ]));
    plan.existing_revisions = jubarte::edit::ExistingRevisions::Reject;
    let source = settlement_identity_source();
    let out = apply_plan(&source, &plan).unwrap();
    let (_, published) = run_thread_cli(&source, &plan, true);
    for bytes in [&out.clean, &out.redline, &published] {
        let listed = list_comments(bytes).unwrap();
        let reply = listed.iter().find(|c| c.text == "Alice replies.").unwrap();
        assert_eq!(
            out.report.operations[0].comment_id,
            Some(reply.id),
            "reported reply id must name the reply in every published representation"
        );
        let target = listed.iter().find(|c| c.text == "Target").unwrap();
        assert_eq!(reply.parent, Some(target.id));
        assert_word_valid_package(bytes);
    }
}

#[test]
fn receipt_1193_blanket_reject_says_rejection() {
    let plan = anchor_plan(
        serde_json::json!({"existing_revisions":"reject"}),
        serde_json::json!([]),
    );
    let (text, _) = run_thread_cli(&inserted_comment(), &plan, false);
    assert!(text.contains("anchor removed by rejecting r"), "{text}");
}

#[test]
fn receipt_1193_mixed_decisions_name_each_removing_pass() {
    let mut pkg = jubarte::opc::PartFs::open(&settlement_identity_source()).unwrap();
    let xml = pkg.part_string("word/document.xml").unwrap().replace(
        r#"<w:commentRangeStart w:id="1"/>"#,
        r#"<w:del w:id="2" w:author="Bob" w:date="2026-10-18T12:00:00Z"><w:commentRangeStart w:id="1"/>"#,
    ).replace(r#"<w:t>First survivor</w:t>"#, r#"<w:delText>First survivor</w:delText>"#)
     .replace(r#"<w:commentReference w:id="1"/></w:r>"#, r#"<w:commentReference w:id="1"/></w:r></w:del>"#);
    pkg.set_part("word/document.xml", xml.into_bytes());
    let plan = anchor_plan(
        serde_json::json!({"resolve_revisions":{"accept":{"ids":["r2"]},"reject":{"ids":["r1"]}}}),
        serde_json::json!([]),
    );
    let (text, _) = run_thread_cli(&pkg.to_zip().unwrap(), &plan, false);
    // Each line names the pass that removed the thread, and its change.
    for (id, why) in [
        ("c0", "anchor removed by rejecting r1"),
        ("c1", "anchor removed by accepting r2"),
    ] {
        let line = text
            .lines()
            .find(|l| l.contains(&format!("removed {id} ")))
            .unwrap();
        assert!(line.ends_with(why), "{line}");
    }
}

#[test]
fn receipt_1193_kept_paragraph_after_ids_shift() {
    let mut pkg = jubarte::opc::PartFs::open(&settlement_identity_source()).unwrap();
    let xml = pkg.part_string("word/document.xml").unwrap().replace(
        r#"<w:commentRangeStart w:id="2"/>"#,
        r#"<w:ins w:id="2" w:author="Bob" w:date="2026-10-18T12:00:00Z"><w:commentRangeStart w:id="2"/>"#,
    ).replace(r#"<w:commentReference w:id="2"/></w:r>"#, r#"<w:commentReference w:id="2"/></w:r></w:ins>"#);
    pkg.set_part("word/document.xml", xml.into_bytes());
    let source = pkg.to_zip().unwrap();
    // Carol wrote c0; Alice deletes it on purpose (#1846).
    let plan = anchor_plan(
        serde_json::json!({"existing_revisions":"reject","removed_anchor_comments":"keep","rewrite_others_comments":true,"resolve_revisions":{"reject":{"ids":["r2"]}}}),
        serde_json::json!([
            {"kind":"delete_comment","comment_id":"c0"}
        ]),
    );
    let (text, bytes) = run_thread_cli(&source, &plan, false);
    let comments = list_comments(&bytes).unwrap();
    let kept = comments.iter().find(|c| c.text == "Target").unwrap();
    let paragraph = kept.paragraph.as_deref().unwrap();
    assert!(
        text.lines()
            .any(|l| l == format!("kept c2 on an empty anchor in {paragraph}")),
        "kept receipt must name published {paragraph}: {text}"
    );
}

#[test]
fn receipt_1193_tracked_reply_id_matches_both_outputs() {
    let source = settlement_identity_source();
    let plan = settlement_identity_plan(&serde_json::json!([
        {"kind":"reply_comment","comment_id":"c2","text":"Alice replies."},
        {"kind":"insert","paragraph":"p4","position":"end","text":" New text."}
    ]));
    let out = apply_plan(&source, &plan).unwrap();
    for bytes in [&out.clean, &out.redline] {
        let comments = list_comments(bytes).unwrap();
        let reply = comments
            .iter()
            .find(|c| c.text == "Alice replies.")
            .unwrap();
        assert_eq!(
            out.report.operations[0].comment_id,
            Some(reply.id),
            "one annotation map must serve clean and tracked replay"
        );
        assert_word_valid_package(bytes);
    }
}

#[test]
fn refusal_1191_lists_settled_revisions_only_when_the_plan_names_them() {
    let named = apply_plan(
        &inserted_comment(),
        &anchor_plan(
            serde_json::json!({"resolve_revisions":{"reject":{"ids":["r1"]}}}),
            anchor_reply(),
        ),
    )
    .unwrap_err();
    assert_eq!(named.code, "UNKNOWN_COMMENT");
    assert!(
        named
            .message
            .contains("c0 is gone once the plan's revisions are settled (r1): its anchor"),
        "{}",
        named.message
    );
    // A blanket settlement names no revision, so no parenthesis repeats
    // "the plan's revisions".
    let blanket = apply_plan(
        &inserted_comment(),
        &anchor_plan(
            serde_json::json!({"existing_revisions":"reject"}),
            anchor_reply(),
        ),
    )
    .unwrap_err();
    assert_eq!(blanket.code, "UNKNOWN_COMMENT");
    assert!(
        blanket
            .message
            .contains("c0 is gone once the plan's revisions are settled: its anchor"),
        "{}",
        blanket.message
    );
    assert!(
        !blanket.message.contains("(the plan's revisions)"),
        "{}",
        blanket.message
    );
}

/// The kept receipt names the paragraph of the file the edit publishes:
/// the clean copy under `--editing-mode`, where a paragraph the plan deletes
/// is gone, and the redline otherwise, where it is still there, deleted.
fn kept_receipt_after_a_deleted_paragraph(editing: bool) {
    let mut pkg = jubarte::opc::PartFs::open(&settlement_identity_source()).unwrap();
    let xml = pkg
        .part_string("word/document.xml")
        .unwrap()
        .replace(
            r#"<w:commentRangeStart w:id="2"/>"#,
            r#"<w:ins w:id="2" w:author="Bob" w:date="2026-10-18T12:00:00Z"><w:commentRangeStart w:id="2"/>"#,
        )
        .replace(
            r#"<w:commentReference w:id="2"/></w:r>"#,
            r#"<w:commentReference w:id="2"/></w:r></w:ins>"#,
        )
        .replacen(
            "<w:body>",
            "<w:body><w:p><w:r><w:t>Lead</w:t></w:r></w:p>",
            1,
        );
    pkg.set_part("word/document.xml", xml.into_bytes());
    // Carol wrote c0; Alice deletes it on purpose (#1846).
    let plan = anchor_plan(
        serde_json::json!({"existing_revisions":"reject","removed_anchor_comments":"keep","rewrite_others_comments":true}),
        serde_json::json!([
            {"kind":"delete_comment","comment_id":"c0"},
            {"kind":"delete_paragraph","paragraph":"p0"}
        ]),
    );
    let (text, bytes) = run_thread_cli(&pkg.to_zip().unwrap(), &plan, editing);
    let comments = list_comments(&bytes).unwrap();
    let kept = comments.iter().find(|c| c.text == "Target").unwrap();
    let paragraph = kept.paragraph.as_deref().unwrap();
    assert!(
        text.lines()
            .any(|l| l == format!("kept c2 on an empty anchor in {paragraph}")),
        "kept receipt must name published {paragraph} (editing: {editing}): {text}"
    );
}

#[test]
fn receipt_1191_kept_paragraph_names_the_clean_copy_in_editing_mode() {
    kept_receipt_after_a_deleted_paragraph(true);
}

#[test]
fn receipt_1191_kept_paragraph_names_the_redline_in_suggesting_mode() {
    kept_receipt_after_a_deleted_paragraph(false);
}

/// A comment the settlement keeps on an empty anchor and the plan then
/// deletes is gone from the output: no receipt may call it kept.
fn deleted_kept_comment_receipt(editing: bool) {
    let source = settlement_identity_source();
    // Carol wrote c0; Alice deletes it on purpose (#1846).
    let plan = anchor_plan(
        serde_json::json!({"existing_revisions":"reject","removed_anchor_comments":"keep","rewrite_others_comments":true}),
        serde_json::json!([{"kind":"delete_comment","comment_id":"c0"}]),
    );
    let (text, bytes) = run_thread_cli(&source, &plan, editing);
    assert!(
        list_comments(&bytes)
            .unwrap()
            .iter()
            .all(|c| c.text != "Removed"),
        "the plan deleted c0"
    );
    assert!(
        !text.lines().any(|l| l.starts_with("kept c0")),
        "a deleted comment is not kept (editing: {editing}): {text}"
    );
}

#[test]
fn receipt_kept_then_deleted_comment_is_not_called_kept_in_suggesting_mode() {
    deleted_kept_comment_receipt(false);
}

#[test]
fn receipt_kept_then_deleted_comment_is_not_called_kept_in_editing_mode() {
    deleted_kept_comment_receipt(true);
}

// #1206: settlement's provisional rescue must agree with the plan's deletions.
// Bob wrote c0; Alice deletes it on purpose (#1846).
fn kept_deletion_plan(source: &[u8], id: u32, track: Option<&str>) -> serde_json::Value {
    if let Some(track) = track {
        serde_json::json!({
            "schema_version": 2,
            "source": {"sha256": jubarte::inspect::source_sha256(source)},
            "policy": {"track": track, "existing_revisions": "reject", "text_view": "final"},
            "resolve_revisions": {"accept": [], "reject": []},
            "author": "Alice", "date": "2026-10-18T12:06:00Z",
            "scope": {"kind": "all"}, "removed_anchor_comments": "keep",
            "rewrite_others_comments": true,
            "operations": [{"kind": "delete_comment", "include_replies": true,
                "target": {"short_id": format!("c{id}"), "long_id": format!("comments.w:comment.w:id.{id}")}}]
        })
    } else {
        serde_json::json!({
            "schema_version": 1, "author": "Alice", "date": "2026-10-18T12:06:00Z",
            "existing_revisions": "reject", "removed_anchor_comments": "keep",
            "rewrite_others_comments": true,
            "operations": [{"kind": "delete_comment", "comment_id": format!("c{id}")}]
        })
    }
}

fn kept_deletion_report(id: u32, track: Option<&str>) {
    let source = anchor_variant("ins", false);
    let value = kept_deletion_plan(&source, id, track);
    let (clean, redline, report) = if track.is_some() {
        let plan = jubarte::edit::v2::PlanV2::from_json(&value.to_string()).unwrap();
        let out = jubarte::edit::v2_apply::apply_plan(&source, &plan).unwrap();
        (out.clean, out.redline, out.report)
    } else {
        let out = apply_plan(&source, &EditPlan::from_json(&value.to_string()).unwrap()).unwrap();
        (out.clean, out.redline, out.report)
    };
    for bytes in [&clean, &redline] {
        assert_word_valid_package(bytes);
        let comments = list_comments(bytes).unwrap();
        assert_eq!(comments.len(), usize::from(id == 1));
        if id == 1 {
            assert_eq!(comments[0].text, "Bob's reason.");
            assert!(comments[0].parent.is_none());
            assert!(comments[0].anchor_text.is_empty());
        }
    }
    let json = serde_json::to_value(&report).unwrap();
    let summary: serde_json::Value =
        serde_json::from_str(report.to_jsonl().lines().last().unwrap()).unwrap();
    assert_eq!(summary["ev"], "summary");
    let expected = if id == 0 {
        (serde_json::Value::Null, serde_json::json!([]))
    } else {
        let kept = serde_json::json!([{"id":"c0","replies":[],"anchor":"empty"}]);
        (kept.clone(), kept)
    };
    assert_eq!(
        (
            json["kept_comments"].clone(),
            summary["kept_comments"].clone()
        ),
        expected,
        "JSON report and JSONL summary must describe only surviving kept comments"
    );
}

#[test]
fn report_1206_v1_kept_root_then_deleted() {
    kept_deletion_report(0, None);
}

#[test]
fn report_1206_v1_kept_root_with_deleted_reply() {
    kept_deletion_report(1, None);
}

#[test]
fn report_1206_v2_kept_root_then_deleted() {
    kept_deletion_report(0, Some("on"));
}

#[test]
fn report_1206_v2_kept_root_with_deleted_reply() {
    kept_deletion_report(1, Some("on"));
}

#[test]
fn report_1206_v2_untracked_kept_root_then_deleted() {
    kept_deletion_report(0, Some("off"));
}

#[test]
fn report_1206_v2_untracked_kept_root_with_deleted_reply() {
    kept_deletion_report(1, Some("off"));
}

fn typed_deleted_kept_receipt(track: &str) {
    let source = anchor_variant("ins", false);
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("source.docx");
    let request = dir.path().join("plan.json");
    let output = dir.path().join("result.docx");
    std::fs::write(&input, &source).unwrap();
    std::fs::write(
        &request,
        kept_deletion_plan(&source, 0, Some(track)).to_string(),
    )
    .unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["edit", "apply"])
        .arg(&input)
        .arg("--plan")
        .arg(&request)
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.status.success(), "{text}");
    assert!(
        list_comments(&std::fs::read(output).unwrap())
            .unwrap()
            .is_empty()
    );
    assert!(
        !text.lines().any(|line| line.starts_with("kept c0")),
        "a deleted comment is not kept (track: {track}): {text}"
    );
}

#[test]
fn receipt_1206_typed_kept_then_deleted_tracked() {
    typed_deleted_kept_receipt("on");
}

#[test]
fn receipt_1206_typed_kept_then_deleted_untracked() {
    typed_deleted_kept_receipt("off");
}
