// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! S4: `scrub` removes who touched a document (authors, rsids, document
//! properties, comments) before it goes out, and the `redact` plan operation
//! replaces text with blocks on both sides of the redline, refusing the plan
//! with `REDACTION_LEAK` when the text survives anywhere in the output.

mod common;

use common::docx::{Part, W_NS, docx, docx_with, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::compare_documents;
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::opc::PartFs;
use jubarte::scrub::{ScrubOptions, leaks, scrub};
use std::io::Read;

fn all_bytes(docx: &[u8]) -> Vec<u8> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut out = Vec::new();
    for i in 0..z.len() {
        z.by_index(i).unwrap().read_to_end(&mut out).unwrap();
    }
    out
}

fn contains(hay: &[u8], needle: &str) -> bool {
    hay.windows(needle.len()).any(|w| w == needle.as_bytes())
}

fn part_names(docx: &[u8]) -> Vec<String> {
    let z = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    z.file_names().map(str::to_string).collect()
}

fn plan(ops: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"A","operations":[{ops}]}}"#
    ))
    .unwrap()
}

#[test]
fn scrub_removes_the_author_and_rsids_everywhere() {
    let red = compare_documents(&docx(&para("a")), &docx(&para("b")), "Jane Secret").unwrap();
    assert!(contains(&all_bytes(&red), "Jane Secret"));
    let out = scrub(
        &red,
        &ScrubOptions {
            author_alias: Some("Reviewer".into()),
            rsids: true,
            docprops: true,
            comments: false,
        },
    )
    .unwrap();
    assert_word_valid_package(&out);
    let bytes = all_bytes(&out);
    assert!(
        !contains(&bytes, "Jane Secret")
            && !contains(&bytes, "w:rsidR=")
            && contains(&bytes, "Reviewer")
    );
}

const W15: &str = "http://schemas.microsoft.com/office/word/2012/wordml";
const SETTINGS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml";
const SETTINGS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings";
const COMMENTS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
const COMMENTS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
const PEOPLE_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.people+xml";
const PEOPLE_REL: &str = "http://schemas.microsoft.com/office/2011/relationships/people";

/// A document that carries everything `scrub` removes: a revision and a
/// comment by "Jane Secret", rsids in the body and settings, `people.xml`
/// with an e-mail, and core, app and custom document properties.
fn identifying_document() -> Vec<u8> {
    let body = r#"<w:p w:rsidR="00AB12CD" w:rsidRDefault="00AB12CD"><w:commentRangeStart w:id="0"/><w:r w:rsidR="00AB12CD"><w:t xml:space="preserve">Kept text. </w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="0"/></w:r><w:ins w:id="1" w:author="Jane Secret" w:date="2026-01-01T00:00:00Z"><w:r><w:t>added</w:t></w:r></w:ins></w:p>"#;
    let settings = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:settings xmlns:w="{W_NS}"><w:zoom w:percent="100"/><w:defaultTabStop w:val="720"/><w:rsids><w:rsidRoot w:val="00AB12CD"/><w:rsid w:val="00AB12CD"/></w:rsids></w:settings>"#
    );
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Jane Secret" w:date="2026-01-01T00:00:00Z" w:initials="JS"><w:p><w:r><w:t>Check with legal.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let people = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w15:people xmlns:w15="{W15}"><w15:person w15:author="Jane Secret"><w15:presenceInfo w15:providerId="AD" w15:userId="jane@secret.example"/></w15:person></w15:people>"#
    );
    let doc = docx_with(
        body,
        &[
            Part {
                name: "word/settings.xml",
                content_type: SETTINGS_CT,
                rel_type: SETTINGS_REL,
                xml: &settings,
            },
            Part {
                name: "word/comments.xml",
                content_type: COMMENTS_CT,
                rel_type: COMMENTS_REL,
                xml: &comments,
            },
            Part {
                name: "word/people.xml",
                content_type: PEOPLE_CT,
                rel_type: PEOPLE_REL,
                xml: &people,
            },
        ],
    );
    let mut pkg = PartFs::open(&doc).unwrap();
    for (name, ct, rel, xml) in [
        (
            "docProps/core.xml",
            "application/vnd.openxmlformats-package.core-properties+xml",
            "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>Deal memo</dc:title><dc:creator>Jane Secret</dc:creator><cp:lastModifiedBy>Jane Secret</cp:lastModifiedBy><cp:revision>7</cp:revision><dcterms:created xsi:type="dcterms:W3CDTF">2026-01-01T00:00:00Z</dcterms:created><dcterms:modified xsi:type="dcterms:W3CDTF">2026-01-02T00:00:00Z</dcterms:modified></cp:coreProperties>"#,
        ),
        (
            "docProps/app.xml",
            "application/vnd.openxmlformats-officedocument.extended-properties+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>Microsoft Office Word</Application><Manager>Jane Secret</Manager><Company>Secret Corp</Company></Properties>"#,
        ),
        (
            "docProps/custom.xml",
            "application/vnd.openxmlformats-officedocument.custom-properties+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"><property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="2" name="Client"><vt:lpwstr>Secret Corp</vt:lpwstr></property></Properties>"#,
        ),
    ] {
        pkg.set_part(name, xml.as_bytes().to_vec());
        pkg.add_content_type_override(&format!("/{name}"), ct);
        pkg.add_package_relationship(rel, name);
    }
    pkg.to_zip().unwrap()
}

#[test]
fn default_scrub_removes_authors_rsids_properties_and_comments() {
    let source = identifying_document();
    assert_word_valid_package(&source);
    let out = scrub(&source, &ScrubOptions::default()).unwrap();
    assert_word_valid_package(&out);
    let bytes = all_bytes(&out);
    for gone in [
        "Jane Secret",
        "JS",
        "jane@secret.example",
        "Secret Corp",
        "rsid",
        "commentReference",
        "commentRangeStart",
        "Check with legal.",
        "2026-01-02T00:00:00Z",
        "<cp:revision>",
    ] {
        assert!(!contains(&bytes, gone), "{gone:?} survives the scrub");
    }
    let names = part_names(&out);
    for part in [
        "word/comments.xml",
        "word/people.xml",
        "docProps/custom.xml",
    ] {
        assert!(!names.iter().any(|n| n == part), "{part} survives");
    }
    // The text and the other party's insertion stay; only who made it goes.
    let text = jubarte::inspect::paragraphs(&out).unwrap()[0].text.clone();
    assert!(text.starts_with("Kept text."), "{text}");
    let changes = jubarte::changes::list_changes(&out).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].author.as_deref(), Some("Author"));
    // Properties that say nothing about a person stay.
    let core = part_string(&out, "docProps/core.xml").unwrap();
    assert!(core.contains("Deal memo"), "{core}");
    let app = part_string(&out, "docProps/app.xml").unwrap();
    assert!(app.contains("Microsoft Office Word"), "{app}");
    let rels = part_string(&out, "_rels/.rels").unwrap();
    assert!(!rels.contains("custom-properties"), "{rels}");
    let types = part_string(&out, "[Content_Types].xml").unwrap();
    assert!(!types.contains("/docProps/custom.xml"), "{types}");
}

#[test]
fn an_alias_alone_renames_comment_authors_and_people_and_keeps_the_rest() {
    let source = identifying_document();
    let out = scrub(
        &source,
        &ScrubOptions {
            author_alias: Some("Counsel".into()),
            rsids: false,
            docprops: false,
            comments: false,
        },
    )
    .unwrap();
    assert_word_valid_package(&out);
    let comments = part_string(&out, "word/comments.xml").unwrap();
    assert!(
        comments.contains(r#"w:author="Counsel""#) && comments.contains(r#"w:initials="C""#),
        "{comments}"
    );
    assert!(comments.contains("Check with legal."), "{comments}");
    let people = part_string(&out, "word/people.xml").unwrap();
    assert!(
        people.contains(r#"w15:author="Counsel""#) && !people.contains("presenceInfo"),
        "{people}"
    );
    let document = part_string(&out, "word/document.xml").unwrap();
    assert!(
        document.contains(r#"w:author="Counsel""#) && document.contains("w:rsidR="),
        "{document}"
    );
    // Untouched options leave their data alone.
    assert!(contains(&all_bytes(&out), "Secret Corp"));
}

#[test]
fn people_with_several_authors_become_one_alias() {
    let people = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w15:people xmlns:w15="{W15}"><w15:person w15:author="Jane Secret"/><w15:person w15:author="Bob Hidden"/></w15:people>"#
    );
    let source = docx_with(
        &para("x"),
        &[Part {
            name: "word/people.xml",
            content_type: PEOPLE_CT,
            rel_type: PEOPLE_REL,
            xml: &people,
        }],
    );
    let out = scrub(
        &source,
        &ScrubOptions {
            author_alias: Some("Author".into()),
            rsids: false,
            docprops: false,
            comments: false,
        },
    )
    .unwrap();
    let people = part_string(&out, "word/people.xml").unwrap();
    assert_eq!(people.matches("<w15:person ").count(), 1, "{people}");
    assert!(!people.contains("Bob Hidden"), "{people}");
}

#[test]
fn scrub_options_parse_from_json_and_refuse_unknown_fields() {
    let options: ScrubOptions =
        serde_json::from_str(r#"{"author_alias":"R","rsids":true}"#).unwrap();
    assert_eq!(
        options,
        ScrubOptions {
            author_alias: Some("R".into()),
            rsids: true,
            docprops: false,
            comments: false,
        }
    );
    assert!(serde_json::from_str::<ScrubOptions>(r#"{"rsid":true}"#).is_err());
    let all = ScrubOptions::default();
    assert_eq!(all.author_alias.as_deref(), Some("Author"));
    assert!(all.rsids && all.docprops && all.comments);
}

#[test]
fn scrub_refuses_bytes_that_are_not_a_document() {
    assert!(scrub(b"not a zip", &ScrubOptions::default()).is_err());
    for alias in [" ", "A\nB"] {
        let options = ScrubOptions {
            author_alias: Some(alias.into()),
            ..ScrubOptions::default()
        };
        let e = scrub(&docx(&para("x")), &options).unwrap_err();
        assert!(e.to_string().contains("author_alias"), "{e}");
    }
}

#[test]
fn redact_leaves_blocks_and_no_copy_of_the_text() {
    let source = docx(&para("Account 12345678 is closed."));
    let plan = EditPlan::from_json(
        r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"redact","paragraph":"body:p:0","find":"12345678"}]}"#,
    )
    .unwrap();
    let out = apply_plan(&source, &plan).unwrap();
    for doc in [&out.clean, &out.redline] {
        assert_word_valid_package(doc);
        assert!(!contains(&all_bytes(doc), "12345678"));
    }
    assert_eq!(
        jubarte::inspect::paragraphs(&out.clean).unwrap()[0].text,
        "Account ████████ is closed."
    );
}

#[test]
fn a_redaction_that_survives_in_a_comment_is_refused() {
    let source = docx(&para("Account 12345678 is closed."));
    let commented = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"comment","paragraph":"body:p:0","text":"12345678 again"}]}"#,
        )
        .unwrap(),
    )
    .unwrap()
    .clean;
    let e = apply_plan(
        &commented,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"redact","paragraph":"body:p:0","find":"12345678"}]}"#,
        )
        .unwrap(),
    )
    .unwrap_err();
    assert_eq!(e.code, "REDACTION_LEAK");
    assert!(e.message.contains("word/comments.xml"));
}

#[test]
fn the_redaction_is_no_tracked_change_and_the_report_never_echoes_the_text() {
    let source = docx(&para("Account 12345678 is closed."));
    let out = apply_plan(
        &source,
        &plan(r#"{"kind":"redact","paragraph":"body:p:0","find":"12345678"}"#),
    )
    .unwrap();
    assert!(
        jubarte::changes::list_changes(&out.redline)
            .unwrap()
            .is_empty()
    );
    assert_eq!(out.report.revisions.total, 0);
    let report = serde_json::to_string(&out.report).unwrap();
    assert!(!report.contains("12345678"), "{report}");
    assert!(report.contains("████████"), "{report}");
}

#[test]
fn a_redaction_beside_another_edit_of_the_paragraph_keeps_both() {
    let source = docx(&para("Account 12345678 is closed."));
    let out = apply_plan(
        &source,
        &plan(
            r#"{"kind":"redact","paragraph":"body:p:0","find":"12345678"},
               {"kind":"replace","paragraph":"body:p:0","find":"closed","replacement":"open"}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.redline);
    assert_eq!(
        jubarte::inspect::paragraphs(&out.clean).unwrap()[0].text,
        "Account ████████ is open."
    );
    let changes = jubarte::changes::list_changes(&out.redline).unwrap();
    // The comparer lists the insertion first; the order is not the point.
    let mut texts: Vec<&str> = changes.iter().map(|c| c.text.as_str()).collect();
    texts.sort_unstable();
    assert_eq!(texts, ["closed", "open"], "{changes:?}");
    assert!(!contains(&all_bytes(&out.redline), "12345678"));
}

#[test]
fn the_same_text_left_in_another_paragraph_is_a_leak() {
    let source = docx(&format!(
        "{}{}",
        para("Account 12345678 is closed."),
        // Split across runs: only the paragraph's joined text shows it.
        r#"<w:p><w:r><w:t>Ref 1234</w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>5678.</w:t></w:r></w:p>"#
    ));
    let e = apply_plan(
        &source,
        &plan(r#"{"kind":"redact","paragraph":"body:p:0","find":"12345678"}"#),
    )
    .unwrap_err();
    assert_eq!(e.code, "REDACTION_LEAK", "{e:?}");
    assert!(e.message.contains("word/document.xml"), "{}", e.message);
    assert!(!e.message.contains("12345678"), "{}", e.message);
    assert_eq!(e.operation.as_deref(), Some("op-1"));
    assert_eq!(e.outcomes[0].status, "failed");
    // Redacting both is accepted.
    let out = apply_plan(
        &docx(&format!(
            "{}{}",
            para("Account 12345678 is closed."),
            para("Ref 12345678.")
        )),
        &plan(
            r#"{"kind":"redact","paragraph":"body:p:0","find":"12345678"},
               {"kind":"redact","paragraph":"body:p:1","find":"12345678"}"#,
        ),
    )
    .unwrap();
    assert!(!contains(&all_bytes(&out.redline), "12345678"));
}

#[test]
fn occurrence_redacts_each_copy_in_one_paragraph() {
    let source = docx(&para("Ref 1234 and again 1234."));
    let one = |ops: &str| apply_plan(&source, &plan(ops));
    // One copy left is a leak, and the refusal does not repeat the text.
    let e = one(r#"{"kind":"redact","paragraph":"body:p:0","find":"1234","occurrence":2}"#)
        .unwrap_err();
    assert_eq!(e.code, "REDACTION_LEAK", "{e:?}");
    assert!(!e.message.contains("1234"), "{}", e.message);
    // Without occurrence the repeated text is ambiguous; the refusal names
    // the count, not the text.
    let e = one(r#"{"kind":"redact","paragraph":"body:p:0","find":"1234"}"#).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR", "{e:?}");
    assert!(!format!("{e:?}").contains("1234"), "{e:?}");
    // Both copies, one op each.
    let out = one(
        r#"{"kind":"redact","paragraph":"body:p:0","find":"1234","occurrence":1},
           {"kind":"redact","paragraph":"body:p:0","find":"1234","occurrence":2}"#,
    )
    .unwrap();
    for doc in [&out.clean, &out.redline] {
        assert_word_valid_package(doc);
        assert!(!contains(&all_bytes(doc), "1234"));
    }
    assert_eq!(
        jubarte::inspect::paragraphs(&out.clean).unwrap()[0].text,
        "Ref \u{2588}\u{2588}\u{2588}\u{2588} and again \u{2588}\u{2588}\u{2588}\u{2588}."
    );
}

#[test]
fn scrub_keeps_a_finding_the_source_already_had() {
    // Text inside a deletion: Word-fatal in the source; scrub did not put
    // it there, so scrub does not refuse it.
    let source = docx(
        r#"<w:p><w:del w:id="1" w:author="Jane" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#,
    );
    let before = jubarte::validate::validate(&source).unwrap();
    assert!(before.iter().any(|f| f.code == "TEXT_INSIDE_DELETION"));
    let out = scrub(&source, &ScrubOptions::default()).unwrap();
    assert_eq!(
        jubarte::validate::validate(&out).unwrap().len(),
        before.len()
    );
}

#[test]
fn leaks_names_each_part_that_holds_the_text() {
    let doc = identifying_document();
    let mut found = leaks(&doc, "Jane Secret");
    found.sort();
    assert_eq!(
        found,
        [
            "docProps/app.xml",
            "docProps/core.xml",
            "word/comments.xml",
            "word/document.xml",
            "word/people.xml"
        ]
    );
    assert!(leaks(&doc, "Kept text.").contains(&"word/document.xml".to_string()));
    assert!(leaks(&doc, "nowhere at all").is_empty());
    assert!(leaks(&doc, "").is_empty());
}

#[test]
fn a_redaction_under_keep_leaves_the_other_partys_changes_and_tracks_nothing_of_its_own() {
    let theirs = compare_documents(
        &docx(&format!(
            "{}{}",
            para("Payment is due in 10 days."),
            para("Account 12345678 is closed.")
        )),
        &docx(&format!(
            "{}{}",
            para("Payment is due in 30 days."),
            para("Account 12345678 is closed.")
        )),
        "Them",
    )
    .unwrap();
    let plan = EditPlan::from_json(
        r#"{"schema_version":1,"author":"Me","existing_revisions":"keep","operations":[
        {"kind":"redact","paragraph":"body:p:1","find":"12345678"},
        {"kind":"replace","paragraph":"body:p:1","find":"closed","replacement":"open"}]}"#,
    )
    .unwrap();
    let out = apply_plan(&theirs, &plan).unwrap();
    for doc in [&out.clean, &out.redline] {
        assert_word_valid_package(doc);
        assert!(!contains(&all_bytes(doc), "12345678"));
    }
    let changes = jubarte::changes::list_changes(&out.redline).unwrap();
    let theirs: Vec<&str> = changes
        .iter()
        .filter(|c| c.author.as_deref() == Some("Them"))
        .map(|c| c.text.as_str())
        .collect();
    let mine: Vec<&str> = changes
        .iter()
        .filter(|c| c.author.as_deref() == Some("Me"))
        .map(|c| c.text.as_str())
        .collect();
    assert_eq!(theirs.len(), 2, "{changes:?}");
    assert_eq!(mine, ["closed", "open"], "{changes:?}");
}

#[test]
fn a_redaction_that_finds_nothing_never_repeats_the_text() {
    let source = docx(&para("Account 12345678 is closed. 87654321 87654321"));
    for find in ["99999999", "87654321"] {
        let e = apply_plan(
            &source,
            &plan(&format!(
                r#"{{"kind":"redact","paragraph":"body:p:0","find":"{find}"}}"#
            )),
        )
        .unwrap_err();
        assert!(
            ["ANCHOR_NOT_FOUND", "AMBIGUOUS_ANCHOR"].contains(&e.code.as_str()),
            "{e:?}"
        );
        let report = format!("{e:?}");
        assert!(!report.contains(find), "{report}");
    }
}
