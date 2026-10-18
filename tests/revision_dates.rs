// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! UTC revision dates across the readers and selective resolution.

mod common;

use common::docx::{docx, part_string};
use jubarte::changes::{ChangeFilter, accept_changes, list_changes, reject_changes};
use jubarte::markdown::{MarkdownOptions, docx_to_markdown};
use std::path::Path;
use std::process::Command;

const LOCAL: &str = "2026-10-17T16:59:00Z";
const UTC: &str = "2026-10-17T20:59:00Z";
const REPORTED: &str = "2026-10-17T16:59:00-04:00";
const LEGACY: &str = "2026-10-17T15:00:00Z";

fn revisions(prefix: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p xmlns:{prefix}="http://schemas.microsoft.com/office/word/2023/wordml/word16du" mc:Ignorable="{prefix}">
<w:ins w:id="1" w:author="Editor" w:date="{LOCAL}" {prefix}:dateUtc="{UTC}"><w:r><w:t>outer</w:t></w:r><w:del w:id="2" w:author="Reviewer" w:date="{LOCAL}" {prefix}:dateUtc="{UTC}"><w:r><w:delText>nested</w:delText></w:r></w:del></w:ins>
<w:r><w:t> separator </w:t></w:r>
<w:ins w:id="3" w:author="Editor" w:date="{LEGACY}"><w:r><w:t>legacy</w:t></w:r></w:ins>
<w:r><w:rPr><w:b/><w:rPrChange w:id="4" w:author="Formatter" w:date="{LOCAL}" {prefix}:dateUtc="{UTC}"><w:rPr/></w:rPrChange></w:rPr><w:t>formatted</w:t></w:r>
</w:p>"#
    ))
}

fn view(bytes: &[u8], ids: bool, dates: bool) -> String {
    docx_to_markdown(
        bytes,
        &MarkdownOptions {
            ids,
            dates,
            page_markers: false,
            ..Default::default()
        },
    )
    .unwrap()
    .markdown
}

fn cli(args: &[&str], directory: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(directory)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn revision_date_inventory_prefers_utc_with_any_prefix() {
    for prefix in ["w16du", "utc"] {
        let changes = list_changes(&revisions(prefix)).unwrap();
        let dates: Vec<_> = changes
            .iter()
            .map(|c| (c.id.as_str(), c.date.as_deref()))
            .collect();
        assert_eq!(
            dates,
            [
                ("r1", Some(REPORTED)),
                ("r2", Some(REPORTED)),
                ("r3", Some(LEGACY)),
                ("r4", Some(REPORTED))
            ]
        );
        assert_eq!(changes[1].inside.as_deref(), Some("r1"));
    }
}

#[test]
fn revision_dates_in_md_authors_and_plain_attributions_prefer_utc() {
    for prefix in ["w16du", "utc"] {
        let bytes = revisions(prefix);
        let md = view(&bytes, true, false);
        assert!(md.contains(&format!("latest {REPORTED}")), "{md}");
        assert!(md.contains(&format!("1 revision, {REPORTED}")), "{md}");
        assert!(!md.contains(LOCAL), "{md}");
        let plain = view(&bytes, false, false);
        assert!(plain.contains(&format!("Editor ({REPORTED})")), "{plain}");
        assert!(plain.contains(&format!("Reviewer ({REPORTED})")), "{plain}");
        assert!(plain.contains(&format!("Editor ({LEGACY})")), "{plain}");
        assert!(!plain.contains(LOCAL), "{plain}");
    }
}

#[test]
fn revision_dates_flag_uses_utc_and_legacy_fallback() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("dates.docx"), revisions("utc")).unwrap();
    let md = cli(
        &["read", "dates.docx", "--dates", "--no-page-markers"],
        directory.path(),
    );
    assert!(md.contains(&format!("@E {REPORTED}}}")), "{md}");
    assert!(md.contains(&format!("@E {LEGACY}}}")), "{md}");
    assert!(!md.contains(LOCAL), "{md}");
}

#[test]
fn revision_dates_changes_json_uses_utc_and_the_listing_keeps_its_columns() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("dates.docx"), revisions("utc")).unwrap();
    // The text listing's six tab-separated columns are a contract
    // (docs/javascript.md, `changes_text`): it prints no date, UTC or not.
    let text = cli(&["changes", "dates.docx"], directory.path());
    let rows: Vec<&str> = text.lines().filter(|line| line.starts_with('r')).collect();
    assert_eq!(rows.len(), 4, "{text}");
    for row in rows {
        let columns = row
            .split('\t')
            .filter(|column| !column.starts_with("inside "));
        assert_eq!(columns.count(), 6, "{row:?}");
    }
    for date in [UTC, LEGACY, LOCAL] {
        assert!(!text.contains(date), "{text}");
    }
    let json = cli(&["changes", "dates.docx", "--json"], directory.path());
    let changes: Vec<serde_json::Value> = json
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(changes.len(), 4);
    for change in changes {
        assert_eq!(
            change["date"],
            if change["id"] == "r3" {
                LEGACY
            } else {
                REPORTED
            }
        );
    }
}

#[test]
fn revision_date_selection_accepts_and_rejects_utc_instants() {
    let bytes = revisions("utc");
    let selected: Vec<String> = list_changes(&bytes)
        .unwrap()
        .into_iter()
        .filter(|c| c.date.as_deref().is_some_and(|date| date == REPORTED))
        .map(|c| c.id)
        .collect();
    assert_eq!(selected, ["r1", "r2", "r4"]);
    let filter = ChangeFilter::ids(selected);
    for resolve in [accept_changes, reject_changes] {
        let result = resolve(&bytes, &filter).unwrap();
        let remaining = list_changes(&result).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "r3");
        assert_eq!(remaining[0].date.as_deref(), Some(LEGACY));
        let xml = part_string(&result, "word/document.xml").unwrap();
        assert!(xml.contains(&format!(r#"w:date="{LEGACY}""#)), "{xml}");
    }
}

#[test]
fn revision_date_namespace_and_missing_date_fallbacks() {
    let bytes = docx(&format!(
        r#"<w:p xmlns:utc="http://schemas.microsoft.com/office/word/2023/wordml/word16du" mc:Ignorable="utc">
<w:ins w:id="1" w:author="Editor" utc:dateUtc="{UTC}"><w:r><w:t>utc only</w:t></w:r></w:ins>
<w:r><w:t> separator </w:t></w:r>
<w:ins xmlns:utc="urn:unrelated" w:id="2" w:author="Reviewer" w:date="{LEGACY}" utc:dateUtc="{UTC}"><w:r><w:t>unrelated</w:t></w:r></w:ins>
<w:r><w:t> separator </w:t></w:r>
<w:ins w:id="3" w:author="Undated"><w:r><w:t>undated</w:t></w:r></w:ins>
</w:p>"#
    ));
    let changes = list_changes(&bytes).unwrap();
    assert_eq!(
        changes
            .iter()
            .map(|c| c.date.as_deref())
            .collect::<Vec<_>>(),
        [Some(UTC), Some(LEGACY), None]
    );
    let md = view(&bytes, true, true);
    let reviewer = md.lines().find(|line| line.contains("Reviewer")).unwrap();
    assert!(reviewer.contains(LEGACY), "{md}");
    assert!(!reviewer.contains(UTC), "{md}");
    let editor = md.lines().find(|line| line.contains("Editor")).unwrap();
    assert!(editor.contains(UTC), "{md}");
}

#[test]
fn revision_date_readers_leave_original_attributes_in_unselected_changes() {
    let bytes = revisions("utc");
    for resolve in [accept_changes, reject_changes] {
        let result = resolve(&bytes, &ChangeFilter::ids(["r3"])).unwrap();
        let xml = part_string(&result, "word/document.xml").unwrap();
        assert_eq!(
            xml.matches(&format!(r#"w:date="{LOCAL}""#)).count(),
            3,
            "{xml}"
        );
        assert_eq!(
            xml.matches(&format!(r#"utc:dateUtc="{UTC}""#)).count(),
            3,
            "{xml}"
        );
        assert!(
            list_changes(&result)
                .unwrap()
                .iter()
                .all(|c| c.date.as_deref() == Some(REPORTED))
        );
    }
}

#[test]
fn revision_date_legacy_fallback_keeps_strict_namespace_dates() {
    let bytes = docx(&format!(
        r#"<w:p xmlns:w="http://purl.oclc.org/ooxml/wordprocessingml/main"><w:ins w:id="1" w:author="Editor" w:date="{LEGACY}"><w:r><w:t>legacy</w:t></w:r></w:ins></w:p>"#
    ));
    let md = view(&bytes, true, false);
    assert!(md.contains(&format!("1 revision, {LEGACY}")), "{md}");
    assert!(view(&bytes, false, false).contains(&format!("Editor ({LEGACY})")));
}

#[test]
fn revision_dates_public_revision_list_uses_utc_in_body_notes_and_formatting() {
    use common::docx::{Part, W_NS, docx_with};
    use jubarte::comparer::WmlComparerSettings;
    use jubarte::document_comparer::get_revisions;

    let utc = format!(
        r#"xmlns:utc="http://schemas.microsoft.com/office/word/2023/wordml/word16du" w:date="{LOCAL}" utc:dateUtc="{UTC}""#
    );
    let body = format!(
        r#"<w:p><w:ins w:id="1" w:author="Editor" {utc}><w:r><w:t>inserted</w:t></w:r></w:ins><w:r><w:t> separator </w:t></w:r><w:ins w:id="2" w:author="Legacy" w:date="{LEGACY}"><w:r><w:t>legacy</w:t></w:r></w:ins><w:r><w:rPr><w:b/><w:rPrChange w:id="3" w:author="Formatter" {utc}><w:rPr/></w:rPrChange></w:rPr><w:t>formatted</w:t></w:r></w:p>"#
    );
    let notes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}"><w:footnote w:id="1"><w:p><w:ins w:id="4" w:author="Note Editor" {utc}><w:r><w:t>note</w:t></w:r></w:ins></w:p></w:footnote></w:footnotes>"#
    );
    let bytes = docx_with(
        &body,
        &[Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
            xml: &notes,
        }],
    );
    let revisions = get_revisions(&bytes, &WmlComparerSettings::default()).unwrap();
    for author in ["Editor", "Legacy", "Formatter", "Note Editor"] {
        let revision = revisions
            .iter()
            .find(|r| r.author.as_deref() == Some(author))
            .unwrap();
        assert_eq!(
            revision.date.as_deref(),
            Some(if author == "Legacy" { LEGACY } else { REPORTED }),
            "{author}"
        );
    }
}

#[test]
fn malformed_utc_dates_fall_back_to_stored_date() {
    for utc in ["", " ", "not-a-date", "2026-02-30T00:00:00Z"] {
        let bytes = docx(&format!(
            r#"<w:p xmlns:utc="http://schemas.microsoft.com/office/word/2023/wordml/word16du"><w:ins w:id="1" w:author="Alice" w:date="{LEGACY}" utc:dateUtc="{utc}"><w:r><w:t>text</w:t></w:r></w:ins></w:p>"#
        ));
        assert_eq!(
            list_changes(&bytes).unwrap()[0].date.as_deref(),
            Some(LEGACY),
            "{utc:?}"
        );
    }
}

#[test]
fn reported_date_offset_bounds_and_fractions() {
    for (local, utc, expected) in [
        (
            "2026-09-25T20:04:00Z",
            "2026-09-26T00:04:00Z",
            "2026-09-25T20:04:00-04:00",
        ),
        (
            "2026-09-26T14:04:00Z",
            "2026-09-26T00:04:00Z",
            "2026-09-26T14:04:00+14:00",
        ),
        (
            "2026-09-25T10:04:00Z",
            "2026-09-26T00:04:00Z",
            "2026-09-25T10:04:00-14:00",
        ),
        (
            "2026-09-26T14:05:00Z",
            "2026-09-26T00:04:00Z",
            "2026-09-26T00:04:00Z",
        ),
        (
            "2026-09-25T10:03:00Z",
            "2026-09-26T00:04:00Z",
            "2026-09-26T00:04:00Z",
        ),
        (
            "2026-09-25T20:04:01Z",
            "2026-09-26T00:04:00Z",
            "2026-09-26T00:04:00Z",
        ),
        (
            "2026-09-26T00:04:00Z",
            "2026-09-26T00:04:00Z",
            "2026-09-26T00:04:00Z",
        ),
        ("bad", "2026-09-26T00:04:00Z", "2026-09-26T00:04:00Z"),
        (
            "2026-09-25T20:04:00.5Z",
            "2026-09-26T00:04:00.500Z",
            "2026-09-25T20:04:00.5-04:00",
        ),
        (
            "2026-09-25T20:04:00.1Z",
            "2026-09-26T00:04:00.5Z",
            "2026-09-26T00:04:00.5Z",
        ),
    ] {
        let bytes = docx(&format!(
            r#"<w:p xmlns:utc="http://schemas.microsoft.com/office/word/2023/wordml/word16du"><w:ins w:id="1" w:author="Alice" w:date="{local}" utc:dateUtc="{utc}"><w:r><w:t>text</w:t></w:r></w:ins></w:p>"#
        ));
        assert_eq!(
            list_changes(&bytes).unwrap()[0].date.as_deref(),
            Some(expected),
            "{local} / {utc}"
        );
    }
}

#[test]
fn shared_and_latest_dates_compare_instants_across_offsets() {
    for (second_local, second_utc, expected) in [
        (
            "2026-09-26T02:04:00Z",
            "2026-09-26T00:04:00Z",
            "2 revisions, 2026-09-25T20:04:00-04:00",
        ),
        (
            "2026-09-25T19:05:00Z",
            "2026-09-26T00:05:00Z",
            "latest 2026-09-25T19:05:00-05:00",
        ),
    ] {
        let bytes = docx(&format!(
            r#"<w:p xmlns:utc="http://schemas.microsoft.com/office/word/2023/wordml/word16du"><w:ins w:id="1" w:author="Alice" w:date="2026-09-25T20:04:00Z" utc:dateUtc="2026-09-26T00:04:00Z"><w:r><w:t>first</w:t></w:r></w:ins><w:r><w:t> separator </w:t></w:r><w:ins w:id="2" w:author="Alice" w:date="{second_local}" utc:dateUtc="{second_utc}"><w:r><w:t>second</w:t></w:r></w:ins></w:p>"#
        ));
        let md = view(&bytes, true, true);
        assert!(md.contains(expected), "{md}");
    }
}

#[test]
fn corpus_comment_and_revision_report_the_same_instant() {
    let bytes = std::fs::read("tests/corpus/broken_ones_two/sources/file_170.docx").unwrap();
    let comments = jubarte::comments::list_comments(&bytes).unwrap();
    assert_eq!(
        comments[0].date.as_deref(),
        Some("2026-05-02T14:42:00-04:00")
    );
    let plain = view(&bytes, false, true);
    assert!(
        plain.matches("(2026-05-02T14:42:00-04:00)").count() >= 2,
        "{plain}"
    );
    let agent = view(&bytes, true, true);
    assert!(agent.contains("2026-05-02T14:42:00-04:00"), "{agent}");
}

#[test]
fn comment_instant_wins_latest_over_earlier_revision() {
    use common::docx::{Part, W_NS, docx_with};
    let body = r#"<w:p><w:ins xmlns:utc="http://schemas.microsoft.com/office/word/2023/wordml/word16du" w:id="1" w:author="Alice" w:date="2026-05-02T18:41:00Z" utc:dateUtc="2026-05-02T18:41:00Z"><w:r><w:t>text</w:t></w:r></w:ins><w:commentRangeStart w:id="0"/><w:r><w:t>anchor</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#;
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:comment w:id="0" w:author="Alice" w:date="2026-05-02T14:42:00Z"><w:p w14:paraId="00000001"><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let ids = r#"<w16cid:commentsIds xmlns:w16cid="http://schemas.microsoft.com/office/word/2016/wordml/cid"><w16cid:commentId w16cid:paraId="00000001" w16cid:durableId="00000002"/></w16cid:commentsIds>"#;
    let ext = r#"<w16cex:commentsExtensible xmlns:w16cex="http://schemas.microsoft.com/office/word/2018/wordml/cex"><w16cex:commentExtensible w16cex:durableId="00000002" w16cex:dateUtc="2026-05-02T18:42:00Z"/></w16cex:commentsExtensible>"#;
    let bytes = docx_with(
        body,
        &[
            Part {
                name: "word/comments.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: &comments,
            },
            Part {
                name: "word/commentsIds.xml",
                content_type: "application/vnd.ms-word.commentsIds+xml",
                rel_type: "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds",
                xml: ids,
            },
            Part {
                name: "word/commentsExtensible.xml",
                content_type: "application/vnd.ms-word.commentsExtensible+xml",
                rel_type: "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible",
                xml: ext,
            },
        ],
    );
    let md = view(&bytes, true, true);
    assert!(md.contains("latest 2026-05-02T14:42:00-04:00"), "{md}");
    let plain = view(&bytes, false, true);
    assert!(plain.contains("(2026-05-02T14:42:00-04:00)"), "{plain}");
}

#[test]
fn latest_comment_selection_compares_offsets_as_instants() {
    let bytes = std::fs::read("tests/corpus/broken_ones_two/sources/file_170.docx").unwrap();
    let mut first = jubarte::comments::list_comments(&bytes).unwrap().remove(0);
    first.id = 0;
    first.parent = None;
    first.date = Some("2026-05-02T18:41:00Z".into());
    let mut later = first.clone();
    later.id = 1;
    later.parent = Some(0);
    later.date = Some("2026-05-02T14:42:00-04:00".into());
    let selected = jubarte::comments::select_comments(vec![first, later], None, true);
    assert_eq!(selected[0].id, 1);
}

fn stored_dates(bytes: &[u8]) -> Vec<String> {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = jubarte::xmllinq::Dom::new();
    let doc = dom.parse_xdocument(&xml);
    dom.descendants(dom.root(doc).unwrap(), None)
        .into_iter()
        .filter_map(|n| {
            dom.attribute(n, &jubarte::namespaces::W::date())
                .map(str::to_string)
        })
        .collect()
}

#[test]
fn corpus_round_trip_preserves_all_361_stored_dates() {
    let bytes = std::fs::read(
        "tests/corpus/_to_improve_accepted_changes/_original_unaccepted_docx/791ca61933.docx",
    )
    .unwrap();
    let original = stored_dates(&bytes);
    assert_eq!(original.len(), 361);
    let md = view(&bytes, true, true);
    let output = jubarte::markdown::markdown_to_docx(&md, &Default::default()).unwrap();
    assert_eq!(stored_dates(&output.docx), original);
    assert!(
        part_string(&output.docx, "word/document.xml")
            .unwrap()
            .contains("dateUtc")
    );
}

#[test]
fn offset_view_writer_preserves_wall_clock_for_marks_and_comments() {
    let md = "---\nauthors:\n  A: Alice # 1 revision, 2026-09-25T20:04:00-04:00\n---\n\n{++text++}{r1 @A} {==anchor==}{>>c0 @A 2026-09-25T20:04:00-04:00: note<<}\n";
    let output = jubarte::markdown::markdown_to_docx(md, &Default::default()).unwrap();
    assert_eq!(stored_dates(&output.docx), ["2026-09-25T20:04:00Z"]);
    let comments = part_string(&output.docx, "word/comments.xml").unwrap();
    assert!(
        comments.contains(r#"w:date="2026-09-25T20:04:00Z""#),
        "{comments}"
    );
    assert!(!comments.contains("dateUtc"));
    assert!(part_string(&output.docx, "word/commentsExtensible.xml").is_none());
}

#[test]
fn offset_view_revision_writes_utc_and_root_compatibility() {
    let mut reference = jubarte::opc::PartFs::open(&docx("<w:p/>")).unwrap();
    let source = reference.part_string("word/document.xml").unwrap().replace("<w:document ", "<w:document xmlns:w14=\"http://schemas.microsoft.com/office/word/2010/wordml\" mc:Ignorable=\"w14\" ");
    reference.set_part("word/document.xml", source.into_bytes());
    let reference = reference.to_zip().unwrap();

    for (date, utc) in [
        ("2026-09-25T20:04:00-04:00", "2026-09-26T00:04:00Z"),
        ("2026-01-01T00:15:00.500+05:30", "2025-12-31T18:45:00.5Z"),
        ("2026-09-25T20:04:00+00:00", "2026-09-25T20:04:00Z"),
    ] {
        let md = format!(
            "---\nauthors:\n  A: Alice # 1 revision, {date}\n---\n\n{{++text++}}{{r1 @A}}\n"
        );
        let output = jubarte::markdown::markdown_to_docx(
            &md,
            &jubarte::markdown::DocxOptions {
                reference: Some(&reference),
                ..Default::default()
            },
        )
        .unwrap();
        let xml = part_string(&output.docx, "word/document.xml").unwrap();
        assert!(xml.contains(&format!(r#"w16du:dateUtc="{utc}""#)), "{xml}");
        let mut dom = jubarte::xmllinq::Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        let xmlns = jubarte::xmllinq::XNamespace::xmlns();
        assert_eq!(
            dom.attribute(root, &xmlns.name("w16du")),
            Some("http://schemas.microsoft.com/office/word/2023/wordml/word16du")
        );
        assert_eq!(
            dom.attribute(root, &xmlns.name("mc")),
            Some(jubarte::namespaces::MC::URI)
        );
        assert!(
            dom.attribute(root, &jubarte::namespaces::MC::name("Ignorable"))
                .unwrap()
                .split_whitespace()
                .any(|p| p == "w16du")
        );
        assert!(
            dom.attribute(root, &jubarte::namespaces::MC::name("Ignorable"))
                .unwrap()
                .split_whitespace()
                .any(|p| p == "w14")
        );
        let report = common::validity::check_word_valid_package(&output.docx);
        assert!(report.ok(), "{:?}", report.errors);
    }
}

#[test]
fn z_view_revision_keeps_legacy_attributes() {
    let md =
        "---\nauthors:\n  A: Alice # 1 revision, 2026-09-25T20:04:00Z\n---\n\n{++text++}{r1 @A}\n";
    let output = jubarte::markdown::markdown_to_docx(md, &Default::default()).unwrap();
    let xml = part_string(&output.docx, "word/document.xml").unwrap();
    assert!(xml.contains(r#"w:date="2026-09-25T20:04:00Z""#));
    assert!(!xml.contains("w16du"));
}

#[test]
fn threaded_comment_offset_writes_linked_utc_instant() {
    let md = "---\nauthors:\n  A: Alice # 2026-09-25T20:04:00-04:00\n---\n\n{==anchor==}{>>c0 @A: note<<}{>>c1 @A re c0: reply<<}\n";
    let output = jubarte::markdown::markdown_to_docx(md, &Default::default()).unwrap();
    let ext = part_string(&output.docx, "word/commentsExtensible.xml").unwrap();
    assert!(
        ext.contains(r#"w16cex:dateUtc="2026-09-26T00:04:00Z""#),
        "{ext}"
    );
    let comments = jubarte::comments::list_comments(&output.docx).unwrap();
    assert_eq!(comments.len(), 2);
    for comment in comments {
        assert_eq!(comment.date.as_deref(), Some("2026-09-25T20:04:00-04:00"));
    }
    common::validity::assert_word_valid_package(&output.docx);
}

fn paired_fraction_round_trip(local: &str, utc: &str, reported: &str) {
    let bytes = docx(&format!(
        r#"<w:p xmlns:utc="http://schemas.microsoft.com/office/word/2023/wordml/word16du"><w:ins w:id="1" w:author="Alice" w:date="{local}" utc:dateUtc="{utc}"><w:r><w:t>text</w:t></w:r></w:ins></w:p>"#
    ));
    assert_eq!(
        list_changes(&bytes).unwrap()[0].date.as_deref(),
        Some(reported)
    );
    let md = view(&bytes, true, true);
    let output = jubarte::markdown::markdown_to_docx(&md, &Default::default()).unwrap();
    assert_eq!(stored_dates(&output.docx), [local]);
}

#[test]
fn paired_fraction_precision_survives_markdown_round_trip() {
    paired_fraction_round_trip(
        "2026-09-25T20:04:00.5Z",
        "2026-09-26T00:04:00.500Z",
        "2026-09-25T20:04:00.5-04:00",
    );
}

#[test]
fn zero_offset_pair_preserves_fraction_precision() {
    paired_fraction_round_trip(
        "2026-09-26T00:04:00.5Z",
        "2026-09-26T00:04:00.500Z",
        "2026-09-26T00:04:00.5Z",
    );
}

#[test]
fn shared_instant_keeps_each_wall_clock_in_dated_view_notes() {
    let bytes = docx(
        r#"<w:p xmlns:utc="http://schemas.microsoft.com/office/word/2023/wordml/word16du"><w:ins w:id="1" w:author="Alice" w:date="2026-09-25T20:04:00Z" utc:dateUtc="2026-09-26T00:04:00Z"><w:r><w:t>first</w:t></w:r></w:ins><w:r><w:t> separator </w:t></w:r><w:ins w:id="2" w:author="Alice" w:date="2026-09-26T02:04:00Z" utc:dateUtc="2026-09-26T00:04:00Z"><w:r><w:t>second</w:t></w:r></w:ins></w:p>"#,
    );
    let md = view(&bytes, true, true);
    let output = jubarte::markdown::markdown_to_docx(&md, &Default::default()).unwrap();
    assert_eq!(stored_dates(&output.docx), stored_dates(&bytes), "{md}");
    assert!(
        md.contains("2 revisions, 2026-09-25T20:04:00-04:00"),
        "{md}"
    );
}
