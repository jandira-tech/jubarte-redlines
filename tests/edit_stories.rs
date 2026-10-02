// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Headers, footers and notes are stories an edit plan can address: inspect
//! lists their paragraphs as `header1:p:0`, `footnotes:p:0`, ...; selectors
//! take those ids or a `story` beside `index`/`starts_with`/`contains`; the
//! comparer tracks the change in the story's own part.

mod common;

use common::docx::{Part, R_NS, W_NS, docx_with_sect, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::capabilities::capabilities;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::{inspect_json, markdown, source_sha256, stories};

const HEADER: &str = "Confidential draft for discussion";
const FOOTER: &str = "Acme Corp. Page footer";
const NOTE: &str = "See the 2024 master agreement.";
const ENDNOTE: &str = "Defined terms follow the master agreement.";

fn letter() -> Vec<u8> {
    letter_docx(false)
}

/// The letter: a header, a footer, footnotes and, on request, endnotes too.
fn letter_docx(with_endnotes: bool) -> Vec<u8> {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>{HEADER}</w:t></w:r></w:p></w:hdr>"#
    );
    let footer = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:ftr xmlns:w="{W_NS}"><w:p><w:r><w:t>{FOOTER}</w:t></w:r></w:p></w:ftr>"#
    );
    let notes = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:footnotes xmlns:w="{W_NS}"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p><w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:footnoteRef/></w:r><w:r><w:t xml:space="preserve"> {NOTE}</w:t></w:r></w:p></w:footnote></w:footnotes>"#
    );
    let endnotes = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:endnotes xmlns:w="{W_NS}"><w:endnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:endnote><w:endnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:endnote><w:endnote w:id="1"><w:p><w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:endnoteRef/></w:r><w:r><w:t xml:space="preserve"> {ENDNOTE}</w:t></w:r></w:p></w:endnote></w:endnotes>"#
    );
    let mut body = para("Body heading")
        + r#"<w:p><w:r><w:t>The parties agree.</w:t></w:r><w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:footnoteReference w:id="1"/></w:r>"#;
    if with_endnotes {
        body += r#"<w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:endnoteReference w:id="1"/></w:r>"#;
    }
    body += "</w:p>";
    let rel = |kind: &str| format!("{R_NS}/{kind}");
    let (h, f, n, e) = (
        rel("header"),
        rel("footer"),
        rel("footnotes"),
        rel("endnotes"),
    );
    let mut parts = vec![
        Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: &h,
            xml: &header,
        },
        Part {
            name: "word/footer1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
            rel_type: &f,
            xml: &footer,
        },
        Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: &n,
            xml: &notes,
        },
    ];
    if with_endnotes {
        parts.push(Part {
            name: "word/endnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml",
            rel_type: &e,
            xml: &endnotes,
        });
    }
    docx_with_sect(
        &body,
        &parts,
        r#"<w:headerReference w:type="default" r:id="rIdX0"/><w:footerReference w:type="default" r:id="rIdX1"/>"#,
    )
}

fn plan(source: &[u8], operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude","date":"2026-09-28T12:00:00Z","operations":{operations}}}"#,
        source_sha256(source)
    ))
    .unwrap()
}

/// Visible text of every paragraph in one part.
fn part_texts(docx: &[u8], part: &str) -> Vec<String> {
    let xml = part_string(docx, part).unwrap();
    let paragraphs: Vec<&str> = xml.split("</w:p>").collect();
    paragraphs[..paragraphs.len() - 1]
        .iter()
        .map(|p| {
            p.split("<w:t")
                .skip(1)
                .filter_map(|t| t.split_once('>').map(|(_, rest)| rest))
                .map(|rest| rest.split('<').next().unwrap_or(""))
                .collect::<String>()
        })
        .filter(|t| !t.is_empty())
        .collect()
}

#[test]
fn inspect_lists_header_footer_and_note_stories() {
    let source = letter();
    let snapshot: serde_json::Value =
        serde_json::from_str(&inspect_json(&source).unwrap()).unwrap();
    let stories: Vec<(String, String, String, Vec<String>)> = snapshot["stories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s["id"].as_str().unwrap().to_string(),
                s["kind"].as_str().unwrap().to_string(),
                s["part"].as_str().unwrap().to_string(),
                s["paragraphs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| {
                        format!(
                            "{} {}",
                            p["id"].as_str().unwrap(),
                            p["text"].as_str().unwrap()
                        )
                    })
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        stories,
        [
            (
                "header1".into(),
                "header".into(),
                "word/header1.xml".into(),
                vec![format!("header1:p:0 {HEADER}")]
            ),
            (
                "footer1".into(),
                "footer".into(),
                "word/footer1.xml".into(),
                vec![format!("footer1:p:0 {FOOTER}")]
            ),
            (
                "footnotes".into(),
                "footnotes".into(),
                "word/footnotes.xml".into(),
                // The separator notes are not text.
                vec![format!("footnotes:p:0  {NOTE}")]
            ),
        ]
    );
    // Body paragraphs keep their ids; the Markdown view lists the stories
    // after the body, each paragraph under its own id.
    assert_eq!(snapshot["paragraphs"].as_array().unwrap().len(), 2);
    let md = markdown(&source).unwrap();
    assert!(md.starts_with("[body:p:0] Body heading"), "{md}");
    assert!(md.contains(&format!("[header1:p:0] {HEADER}")), "{md}");
    assert!(md.contains("[footnotes:p:0]"), "{md}");
}

#[test]
fn edits_in_header_footer_and_footnote_are_tracked_in_their_parts() {
    let source = letter();
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"hdr","kind":"replace","paragraph":"header1:p:0","find":"Confidential","replacement":"Highly confidential"},
                {"id":"ftr","kind":"insert","paragraph":{"story":"footer1","index":0},"position":"end","text":" (rev. 2)"},
                {"id":"note","kind":"replace","paragraph":{"story":"footnotes","contains":"master agreement"},"find":"2024","replacement":"2026","whole":true},
                {"id":"body","kind":"replace","paragraph":{"index":1},"find":"agree","replacement":"hereby agree"}]"#,
        ),
    )
    .unwrap();
    let report = &result.report;
    assert!(report.ok, "{:?}", report.operations);
    let at: Vec<_> = report
        .operations
        .iter()
        .map(|o| o.paragraph.clone().unwrap())
        .collect();
    assert_eq!(
        at,
        ["header1:p:0", "footer1:p:0", "footnotes:p:0", "body:p:1"]
    );
    // One insertion per story; header and footer revisions count too.
    assert!(report.revisions.inserted >= 4, "{:?}", report.revisions);
    assert!(
        report.operations.iter().all(|o| o.message.is_none()),
        "{:?}",
        report.operations
    );

    assert_eq!(
        part_texts(&result.clean, "word/header1.xml"),
        ["Highly confidential draft for discussion"]
    );
    assert_eq!(
        part_texts(&result.clean, "word/footer1.xml"),
        ["Acme Corp. Page footer (rev. 2)"]
    );
    let redline_header = part_string(&result.redline, "word/header1.xml").unwrap();
    assert!(redline_header.contains("<w:ins "), "{redline_header}");
    let redline_footer = part_string(&result.redline, "word/footer1.xml").unwrap();
    assert!(redline_footer.contains("<w:ins "), "{redline_footer}");
    let notes = part_string(&result.redline, "word/footnotes.xml").unwrap();
    assert!(
        notes.contains("2024</w:delText>") && notes.contains(">2026</w:t>"),
        "{notes}"
    );
    assert!(!notes.contains("_jubarte_whole"), "{notes}");

    let accepted = accept_revisions(&result.redline).unwrap();
    let rejected = reject_revisions(&result.redline).unwrap();
    for part in ["word/header1.xml", "word/footer1.xml", "word/footnotes.xml"] {
        assert_eq!(
            part_texts(&accepted, part),
            part_texts(&result.clean, part),
            "{part}"
        );
        assert_eq!(
            part_texts(&rejected, part),
            part_texts(&source, part),
            "{part}"
        );
    }
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
}

/// A header with its own relationship (here a hyperlink; usually a logo).
fn letter_with_linked_header() -> Vec<u8> {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}" xmlns:r="{R_NS}"><w:p><w:r><w:t>{HEADER}</w:t></w:r></w:p><w:p><w:hyperlink r:id="rIdL1"><w:r><w:t>acme.example</w:t></w:r></w:hyperlink></w:p></w:hdr>"#
    );
    let header_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdL1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://acme.example/" TargetMode="External"/></Relationships>"#;
    let h = format!("{R_NS}/header");
    docx_with_sect(
        &para("Body heading"),
        &[
            Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: &h,
                xml: &header,
            },
            Part {
                name: "word/_rels/header1.xml.rels",
                content_type: "application/vnd.openxmlformats-package.relationships+xml",
                rel_type: "",
                xml: header_rels,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

#[test]
fn a_header_with_relationships_is_still_redlined() {
    // The comparer used to skip any header part carrying a relationship, so
    // the edit reached the clean copy but not the redline.
    let source = letter_with_linked_header();
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"replace","paragraph":"header1:p:0","find":"Confidential","replacement":"Privileged"}]"#,
        ),
    )
    .unwrap();
    let redline = part_string(&result.redline, "word/header1.xml").unwrap();
    assert!(
        redline.contains("<w:ins ") && redline.contains("<w:del "),
        "{redline}"
    );
    assert!(redline.contains(r#"r:id="rIdL1""#), "{redline}");
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(
        part_texts(&accepted, "word/header1.xml"),
        ["Privileged draft for discussion", "acme.example"]
    );
    assert_eq!(
        part_texts(
            &reject_revisions(&result.redline).unwrap(),
            "word/header1.xml"
        ),
        [HEADER, "acme.example"]
    );
    assert_word_valid_package(&result.redline);
}

#[test]
fn story_selectors_and_story_limits_are_checked() {
    let source = letter();
    let refuse = |ops: &str| apply_plan(&source, &plan(&source, ops)).unwrap_err();

    let unknown =
        refuse(r#"[{"kind":"delete","paragraph":{"story":"header9","index":0},"find":"x"}]"#);
    assert_eq!(unknown.code, "ANCHOR_NOT_FOUND");
    assert!(
        unknown.message.contains("header1") && unknown.message.contains("footnotes"),
        "{}",
        unknown.message
    );

    let past = refuse(r#"[{"kind":"delete","paragraph":"header1:p:3","find":"x"}]"#);
    assert_eq!(past.code, "ANCHOR_NOT_FOUND");

    // Word cannot anchor a comment in a header or footer.
    let comment = refuse(r#"[{"kind":"comment","paragraph":"header1:p:0","text":"why?"}]"#);
    assert_eq!(comment.code, "COMMENT_NOT_IN_BODY");
    assert!(comment.message.contains("body"), "{}", comment.message);

    // A header keeps at least one paragraph.
    let last = refuse(r#"[{"kind":"delete_paragraph","paragraph":"header1:p:0"}]"#);
    assert_eq!(last.code, "UNSUPPORTED_STRUCTURE");

    // Body selectors never reach into a story.
    let body_only =
        refuse(r#"[{"kind":"delete","paragraph":{"contains":"Confidential"},"find":"x"}]"#);
    assert_eq!(body_only.code, "ANCHOR_NOT_FOUND");

    // A misspelled selector key is refused, not ignored.
    let typo = EditPlan::from_json(
        r#"{"schema_version":1,"author":"A","operations":[{"kind":"delete","paragraph":{"stroy":"header1","index":0},"find":"x"}]}"#,
    )
    .unwrap_err();
    assert_eq!(typo.code, "INVALID_PLAN");

    // New paragraphs in a header copy the anchor's properties.
    let added = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert_paragraph","paragraph":"header1:p:0","position":"after","runs":[{"text":"Privileged"}]}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        part_texts(&added.clean, "word/header1.xml"),
        [HEADER, "Privileged"]
    );
    assert_word_valid_package(&added.redline);
}

/// The capability manifest's `limits.stories` and the stories `inspect`
/// reports and `edit` addresses are one list: every kind the manifest names
/// is a story of a document that has one, named by its id in a selector, and
/// no story kind goes unadvertised.
#[test]
fn capabilities_advertise_exactly_the_story_kinds_edit_accepts() {
    let advertised = capabilities("rust").limits.stories;
    let source = letter_docx(true);
    let found = stories(&source).unwrap();
    let mut kinds = vec!["body".to_string()];
    for story in &found {
        if !kinds.contains(&story.kind) {
            kinds.push(story.kind.clone());
        }
    }
    assert_eq!(advertised, kinds);

    // A selector names each advertised story by the id inspect printed.
    let mut operations = vec![
        r#"{"kind":"insert","paragraph":{"index":0},"position":"end","text":" (rev)"}"#.to_string(),
    ];
    for story in &found {
        operations.push(format!(
            r#"{{"kind":"insert","paragraph":"{}:p:0","position":"end","text":" (rev)"}}"#,
            story.id
        ));
    }
    let result = apply_plan(
        &source,
        &plan(&source, &format!("[{}]", operations.join(","))),
    )
    .unwrap();
    assert!(result.report.ok, "{:?}", result.report.operations);
    let at: Vec<_> = result
        .report
        .operations
        .iter()
        .map(|o| o.paragraph.clone().unwrap())
        .collect();
    assert_eq!(
        at,
        [
            "body:p:0",
            "header1:p:0",
            "footer1:p:0",
            "footnotes:p:0",
            "endnotes:p:0"
        ]
    );
    for story in &found {
        let texts = part_texts(&result.clean, &story.part);
        assert!(
            texts.iter().any(|t| t.ends_with(" (rev)")),
            "{}: {texts:?}",
            story.part
        );
    }
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
}
