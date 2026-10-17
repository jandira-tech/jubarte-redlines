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
                ("r1", Some(UTC)),
                ("r2", Some(UTC)),
                ("r3", Some(LEGACY)),
                ("r4", Some(UTC))
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
        assert!(md.contains(&format!("latest {UTC}")), "{md}");
        assert!(md.contains(&format!("1 revision, {UTC}")), "{md}");
        assert!(!md.contains(LOCAL), "{md}");
        let plain = view(&bytes, false, false);
        assert!(plain.contains(&format!("Editor ({UTC})")), "{plain}");
        assert!(plain.contains(&format!("Reviewer ({UTC})")), "{plain}");
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
    assert!(md.contains(&format!("@E {UTC}<<")), "{md}");
    assert!(md.contains(&format!("@E {LEGACY}<<")), "{md}");
    assert!(!md.contains(LOCAL), "{md}");
}

#[test]
fn revision_dates_changes_text_and_json_use_utc() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("dates.docx"), revisions("utc")).unwrap();
    let text = cli(&["changes", "dates.docx"], directory.path());
    assert!(text.contains(UTC), "{text}");
    assert!(text.contains(LEGACY), "{text}");
    assert!(!text.contains(LOCAL), "{text}");
    let json = cli(&["changes", "dates.docx", "--json"], directory.path());
    let changes: Vec<serde_json::Value> = json
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(changes.len(), 4);
    for change in changes {
        assert_eq!(
            change["date"],
            if change["id"] == "r3" { LEGACY } else { UTC }
        );
    }
}

#[test]
fn revision_date_selection_accepts_and_rejects_utc_instants() {
    let bytes = revisions("utc");
    let selected: Vec<String> = list_changes(&bytes)
        .unwrap()
        .into_iter()
        .filter(|c| {
            c.date
                .as_deref()
                .is_some_and(|date| date >= "2026-10-17T19:00:00Z")
        })
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
                .all(|c| c.date.as_deref() == Some(UTC))
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
            Some(if author == "Legacy" { LEGACY } else { UTC }),
            "{author}"
        );
    }
}
