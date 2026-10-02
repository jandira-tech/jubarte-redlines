// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `append::append_documents` with `comments: carry`: the comments B's body
//! anchors come along with fresh ids, their threads, resolution, links and
//! styles; those in notes and headers are dropped and warned. The default
//! (`drop`) is covered in `append_documents.rs` too.

mod common;

use common::docx::{Part, R_NS, W_NS, docx, docx_with, docx_with_sect_pr, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::append::{AppendComments, AppendOptions, SectionBreak, append_documents};
use jubarte::comments::{CommentRecord, list_comments};

const COMMENTS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";

fn carry() -> AppendOptions {
    AppendOptions {
        section_break: SectionBreak::Continuous,
        comments: AppendComments::Carry,
        ..AppendOptions::default()
    }
}

/// Ring-1 clean, and nothing `jubarte validate` calls Word-fatal.
fn checked(docx: &[u8]) {
    assert_word_valid_package(docx);
    let fatal: Vec<String> = jubarte::validate::validate(docx)
        .unwrap()
        .into_iter()
        .filter(|f| f.word_fatal)
        .map(|f| format!("{} {}", f.code, f.message))
        .collect();
    assert!(fatal.is_empty(), "{fatal:#?}");
}

fn edited(base: &[u8], author: &str, operations: &str) -> Vec<u8> {
    let plan = format!(r#"{{"schema_version":1,"author":"{author}","operations":[{operations}]}}"#);
    jubarte::edit::apply_plan(base, &jubarte::edit::EditPlan::from_json(&plan).unwrap())
        .unwrap()
        .clean
}

/// One paragraph of `text` carrying comment `text` by `author`.
fn commented(text: &str, author: &str, comment: &str) -> Vec<u8> {
    edited(
        &docx(&para(text)),
        author,
        &format!(r#"{{"kind":"comment","paragraph":"body:p:0","text":"{comment}"}}"#),
    )
}

fn summary(c: &CommentRecord) -> (String, String, String) {
    (c.author.clone(), c.text.clone(), c.anchor_text.clone())
}

#[test]
fn a_carried_comment_keeps_its_anchor_and_text() {
    let out = append_documents(
        &docx(&para("A.")),
        &commented("B.", "Ann", "check"),
        &carry(),
    )
    .unwrap();
    checked(&out.docx);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let comments = list_comments(&out.docx).unwrap();
    assert_eq!(comments.len(), 1, "{comments:?}");
    assert_eq!(
        summary(&comments[0]),
        ("Ann".into(), "check".into(), "B.".into())
    );
}

#[test]
fn equal_comments_on_both_sides_stay_two_comments() {
    // The same plan on both sides: same comment id, text, author and
    // paraIds. Two documents, so two comments.
    let a = commented("A.", "Ann", "OK");
    let b = commented("B.", "Ann", "OK");
    let a_id = list_comments(&a).unwrap()[0].id;
    let out = append_documents(&a, &b, &carry()).unwrap();
    checked(&out.docx);
    let comments = list_comments(&out.docx).unwrap();
    assert_eq!(
        comments.iter().map(summary).collect::<Vec<_>>(),
        vec![
            ("Ann".into(), "OK".into(), "A.".into()),
            ("Ann".into(), "OK".into(), "B.".into()),
        ]
    );
    assert_eq!(comments[0].id, a_id, "A's comment keeps its id");
    assert_ne!(comments[0].id, comments[1].id);
}

#[test]
fn a_thread_and_its_resolution_come_along() {
    let b = commented("B.", "Ann", "Too low");
    let root = list_comments(&b).unwrap()[0].id;
    let b = edited(
        &b,
        "Bob",
        &format!(
            r#"{{"kind":"reply_comment","comment_id":{root},"text":"Agreed"}},{{"kind":"resolve_comment","comment_id":{root}}}"#
        ),
    );
    let a = commented("A.", "Cy", "Mine");
    let out = append_documents(&a, &b, &carry()).unwrap();
    checked(&out.docx);
    let comments = list_comments(&out.docx).unwrap();
    assert_eq!(comments.len(), 3, "{comments:?}");
    let (ann, bob) = (&comments[1], &comments[2]);
    assert_eq!(
        (ann.text.as_str(), bob.text.as_str()),
        ("Too low", "Agreed")
    );
    assert_eq!(bob.parent, Some(ann.id));
    assert!(ann.done && bob.done);
    assert!(!comments[0].done && comments[0].parent.is_none());
}

/// `docx` with `entries` added or replaced.
fn with_entries(docx: &[u8], entries: &[(&str, &[u8])]) -> Vec<u8> {
    use std::io::Write;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_string();
        if entries.iter().any(|(n, _)| *n == name) {
            continue;
        }
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut data).unwrap();
        zip.start_file(name.as_str(), opts).unwrap();
        zip.write_all(&data).unwrap();
    }
    for (name, bytes) in entries {
        zip.start_file(*name, opts).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

const ANCHORED_B: &str = r#"<w:p><w:commentRangeStart w:id="0"/><w:r><w:t>B.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#;

/// B whose body anchors comment 0, defined by `comment_body` (the
/// `w:comment` content), plus `extras`.
fn b_with_comment(body: &str, comment_body: &str, extras: &[Part<'_>]) -> Vec<u8> {
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{W_NS}" xmlns:r="{R_NS}"><w:comment w:id="0" w:author="Ann" w:date="2026-10-01T00:00:00Z" w:initials="A">{comment_body}</w:comment></w:comments>"#
    );
    let rel = format!("{R_NS}/comments");
    let mut parts = vec![Part {
        name: "word/comments.xml",
        content_type: COMMENTS_CT,
        rel_type: &rel,
        xml: &comments,
    }];
    parts.extend(extras.iter().map(|p| Part { ..*p }));
    docx_with(body, &parts)
}

#[test]
fn a_link_inside_a_comment_keeps_its_target() {
    let b = b_with_comment(
        ANCHORED_B,
        r#"<w:p><w:r><w:annotationRef/></w:r><w:hyperlink r:id="rIdL1"><w:r><w:t>see</w:t></w:r></w:hyperlink></w:p>"#,
        &[],
    );
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdL1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com/terms" TargetMode="External"/></Relationships>"#;
    let b = with_entries(&b, &[("word/_rels/comments.xml.rels", rels.as_bytes())]);
    let out = append_documents(&docx(&para("A.")), &b, &carry()).unwrap();
    checked(&out.docx);
    let comments = part_string(&out.docx, "word/comments.xml").unwrap();
    let at = comments.find(r#"r:id=""#).expect("hyperlink id") + r#"r:id=""#.len();
    let rid = &comments[at..at + comments[at..].find('"').unwrap()];
    let rels = part_string(&out.docx, "word/_rels/comments.xml.rels").unwrap();
    assert!(
        rels.contains(&format!(r#"Id="{rid}""#)) && rels.contains("https://example.com/terms"),
        "{rid}: {rels}"
    );
}

#[test]
fn a_comment_s_paragraph_style_comes_along() {
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:styleId="Kommentartext"><w:name w:val="annotation text"/><w:rPr><w:sz w:val="20"/></w:rPr></w:style></w:styles>"#
    );
    let rel = format!("{R_NS}/styles");
    let b = b_with_comment(
        ANCHORED_B,
        r#"<w:p><w:pPr><w:pStyle w:val="Kommentartext"/></w:pPr><w:r><w:annotationRef/></w:r><w:r><w:t>styled</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: &rel,
            xml: &styles,
        }],
    );
    let out = append_documents(&docx(&para("A.")), &b, &carry()).unwrap();
    checked(&out.docx);
    let styles = part_string(&out.docx, "word/styles.xml").unwrap();
    assert!(
        styles.contains(r#"w:styleId="Kommentartext""#),
        "the style the comment uses is copied: {styles}"
    );
}

#[test]
fn a_comment_inside_a_carried_footnote_is_dropped_with_the_note_kept() {
    let footnotes = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:footnotes xmlns:w="{W_NS}"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p><w:commentRangeStart w:id="0"/><w:r><w:t>Note.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p></w:footnote></w:footnotes>"#
    );
    let rel = format!("{R_NS}/footnotes");
    let b = b_with_comment(
        r#"<w:p><w:r><w:t>B.</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        r#"<w:p><w:r><w:annotationRef/></w:r><w:r><w:t>on the note</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: &rel,
            xml: &footnotes,
        }],
    );
    // `validate` counts comment anchors in the main part only, and Word's
    // handling of a comment anchored in a note is unchecked: dropped, warned.
    let out = append_documents(&commented("A.", "Cy", "mine"), &b, &carry()).unwrap();
    checked(&out.docx);
    let comments = list_comments(&out.docx).unwrap();
    assert_eq!(
        comments.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(),
        vec!["mine"]
    );
    let notes = part_string(&out.docx, "word/footnotes.xml").unwrap();
    assert!(
        notes.contains("Note.") && !notes.contains("comment"),
        "{notes}"
    );
    assert_eq!(
        out.warnings,
        vec!["COMMENTS_DROPPED: 1 comment of B was not carried".to_string()]
    );
}

#[test]
fn a_comment_in_a_carried_header_is_still_dropped() {
    let header_rel = format!("{R_NS}/header");
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}"><w:p><w:commentRangeStart w:id="1"/><w:r><w:t>Head</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p></w:hdr>"#
    );
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Ann"><w:p><w:r><w:t>body</w:t></w:r></w:p></w:comment><w:comment w:id="1" w:author="Ann"><w:p><w:r><w:t>head</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let comments_rel = format!("{R_NS}/comments");
    let b = docx_with_sect_pr(
        ANCHORED_B,
        &[
            Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: &header_rel,
                xml: &header,
            },
            Part {
                name: "word/comments.xml",
                content_type: COMMENTS_CT,
                rel_type: &comments_rel,
                xml: &comments,
            },
        ],
        r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>"#,
    );
    let out = append_documents(
        &docx(&para("A.")),
        &b,
        &AppendOptions {
            keep_sections: true,
            ..carry()
        },
    )
    .unwrap();
    checked(&out.docx);
    let comments = list_comments(&out.docx).unwrap();
    assert_eq!(
        comments.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(),
        vec!["body"]
    );
    assert_eq!(
        out.warnings,
        vec!["COMMENTS_DROPPED: 1 comment of B was not carried".to_string()]
    );
}

#[test]
fn an_anchor_b_does_not_define_is_dropped() {
    let b = docx(ANCHORED_B);
    let out = append_documents(&docx(&para("A.")), &b, &carry()).unwrap();
    checked(&out.docx);
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert!(!doc.contains("comment"), "{doc}");
    assert_eq!(
        out.warnings,
        vec!["COMMENTS_DROPPED: 1 comment of B was not carried".to_string()]
    );
}

#[test]
fn dropping_stays_the_default_and_carry_reads_from_json() {
    assert_eq!(AppendOptions::default().comments, AppendComments::Drop);
    let options: AppendOptions = serde_json::from_str(r#"{"comments":"carry"}"#).unwrap();
    assert_eq!(options.comments, AppendComments::Carry);
    let out = append_documents(
        &docx(&para("A.")),
        &commented("B.", "Ann", "x"),
        &AppendOptions::default(),
    )
    .unwrap();
    assert!(list_comments(&out.docx).unwrap().is_empty());
    assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
}

#[test]
fn cli_carries_comments_on_request() {
    let dir = std::env::temp_dir().join(format!("jubarte-append-comments-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.docx"), docx(&para("A."))).unwrap();
    std::fs::write(dir.join("b.docx"), commented("B.", "Ann", "keep me")).unwrap();
    let run = |extra: &[&str], out: &str| {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .current_dir(&dir)
            .args(["append", "a.docx", "b.docx", "-o", out])
            .args(extra)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        (
            std::fs::read(dir.join(out)).unwrap(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    };
    let (carried, stderr) = run(&["--carry-comments"], "carried.docx");
    checked(&carried);
    assert_eq!(list_comments(&carried).unwrap()[0].text, "keep me");
    assert!(!stderr.contains("COMMENTS_DROPPED"), "{stderr}");
    let (dropped, stderr) = run(&[], "dropped.docx");
    assert!(list_comments(&dropped).unwrap().is_empty());
    assert!(stderr.contains("COMMENTS_DROPPED"), "{stderr}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_comment_s_extension_attributes_stay_ignorable() {
    // Word 2023+ writes `w16du:dateUtc` on `w:comment`; A's comments part
    // must declare the prefix and list it in `mc:Ignorable`, like B's.
    let w16du = "http://schemas.microsoft.com/office/word/2023/wordml/word16du";
    let mc = common::docx::MC_NS;
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{W_NS}" xmlns:mc="{mc}" xmlns:w16du="{w16du}" mc:Ignorable="w16du"><w:comment w:id="0" w:author="Ann" w:date="2026-10-01T00:00:00Z" w16du:dateUtc="2026-10-01T00:00:00Z"><w:p><w:r><w:t>dated</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let rel = format!("{R_NS}/comments");
    let b = docx_with(
        ANCHORED_B,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: &rel,
            xml: &comments,
        }],
    );
    let out = append_documents(&docx(&para("A.")), &b, &carry()).unwrap();
    checked(&out.docx);
    let xml = part_string(&out.docx, "word/comments.xml").unwrap();
    assert!(xml.contains("w16du:dateUtc"), "{xml}");
    let root = &xml[xml.find("<w:comments").unwrap()..];
    let root = &root[..=root.find('>').unwrap()];
    assert!(
        root.contains(&format!(r#"xmlns:w16du="{w16du}""#)),
        "{root}"
    );
    let at = root.find(r#"mc:Ignorable=""#).expect("mc:Ignorable") + r#"mc:Ignorable=""#.len();
    let ignorable = &root[at..at + root[at..].find('"').unwrap()];
    assert!(ignorable.split_whitespace().any(|p| p == "w16du"), "{root}");
}
