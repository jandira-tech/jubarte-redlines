// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! The formatting changes Word tracks on its notes separators come back
//! through docx → view → docx: the header's `notes: {fmt: footnotes.r0
//! @AS; …}` names them, and the writer puts each on its separator.
//! mix.md (25f1d6ecc3.docx) and ids-table-changes.md (5f0fed8e2a.docx)
//! lost them.

mod common;

use common::docx::{Part, docx_with};
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

const W: &str = common::docx::W_NS;
const AS: &str = r#"w:author="A Smith" w:date="2026-09-26T14:55:00Z""#;

fn view(docx: &[u8]) -> String {
    let options = ReadOptions {
        page_markers: false,
        track_changes: TrackChanges::All,
        ..ReadOptions::default()
    };
    read(docx, &options)
        .expect("the docx reads")
        .markdown
        .lines()
        // The page count is measured, not written (as md_fixed_point).
        .filter(|l| !l.starts_with("sha256: ") && !l.starts_with("body: "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A separator note of `kind` (`footnote`, `endnote`) whose paragraph
/// carries formatting change `id`, or none.
fn separator(kind: &str, ty: &str, number: i32, id: Option<u32>) -> String {
    let change = id.map_or_else(String::new, |id| {
        format!(r#"<w:pPrChange w:id="{id}" {AS}><w:pPr/></w:pPrChange>"#)
    });
    format!(
        r#"<w:{kind} w:type="{ty}" w:id="{number}"><w:p><w:pPr>{change}</w:pPr><w:r><w:{ty}/></w:r></w:p></w:{kind}>"#
    )
}

fn notes(kind: &str, ids: [Option<u32>; 2], extra: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{kind}s xmlns:w="{W}">{}{}{extra}</w:{kind}s>"#,
        separator(kind, "separator", -1, ids[0]),
        separator(kind, "continuationSeparator", 0, ids[1]),
    )
}

fn source(body: &str, footnotes: &str, endnotes: &str) -> Vec<u8> {
    docx_with(
        body,
        &[
            Part {
                name: "word/footnotes.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
                xml: footnotes,
            },
            Part {
                name: "word/endnotes.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes",
                xml: endnotes,
            },
        ],
    )
}

/// The view of `docx`, the docx it writes, and that docx's view.
fn round_trip(docx: &[u8]) -> (String, Vec<u8>, String) {
    let first = view(docx);
    let again = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let second = view(&again);
    (first, again, second)
}

fn part(docx: &[u8], name: &str) -> String {
    common::docx::part_string(docx, name).unwrap_or_else(|| panic!("{name}"))
}

/// The `w:id`s of the formatting changes in `xml`, in order.
fn format_change_ids(xml: &str) -> Vec<String> {
    let mut dom = jubarte::xmllinq::Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).unwrap();
    change_ids_in(&dom, root)
}

/// The `w:id`s of the formatting changes under `node`, in order.
fn change_ids_in(dom: &jubarte::xmllinq::Dom, node: jubarte::xmllinq::NodeId) -> Vec<String> {
    dom.descendants(node, Some(&jubarte::namespaces::W::name("pPrChange")))
        .iter()
        .filter_map(|&node| {
            dom.attribute(node, &jubarte::namespaces::W::id())
                .map(str::to_string)
        })
        .collect()
}

/// The `w:id`s of the formatting changes the `ty` note of `kind`
/// (`footnote`, `endnote`) carries: `separator` and
/// `continuationSeparator` are two notes, so a test that only counts the
/// part's changes cannot say which separator each came back on. This one
/// looks the note up by its `w:type`, so putting both changes on one
/// separator fails.
fn separator_change_ids(xml: &str, kind: &str, ty: &str) -> Vec<String> {
    let mut dom = jubarte::xmllinq::Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).unwrap();
    let note = dom
        .elements(root, Some(&jubarte::namespaces::W::name(kind)))
        .into_iter()
        .find(|&note| dom.attribute(note, &jubarte::namespaces::W::name("type")) == Some(ty))
        .unwrap_or_else(|| panic!("a {ty} {kind} in {xml}"));
    change_ids_in(&dom, note)
}

/// What the view names for each separator, and the `w:id`s each one
/// carries back: `(kind, separator id, continuation id)`.
fn separator_pairs(xml: &str, kind: &str) -> (Vec<String>, Vec<String>) {
    (
        separator_change_ids(xml, kind, "separator"),
        separator_change_ids(xml, kind, "continuationSeparator"),
    )
}

#[test]
fn separator_formatting_changes_come_back_on_their_separators() {
    let docx = source(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &notes("footnote", [Some(0), Some(1)], ""),
        &notes("endnote", [Some(2), Some(3)], ""),
    );
    let (first, again, second) = round_trip(&docx);
    assert!(
        first.contains(
            "notes: {fmt: footnotes.r0 @AS; footnotes.r1 @AS; endnotes.r2 @AS; \
             endnotes.r3 @AS, empty: [footnotes, endnotes]}"
        ),
        "{first}"
    );
    let footnotes = part(&again, "word/footnotes.xml");
    assert_eq!(format_change_ids(&footnotes), ["0", "1"], "{footnotes}");
    assert!(footnotes.contains(r#"w:author="A Smith""#), "{footnotes}");
    // Each change comes back on its own separator: the first on the
    // separator note, the second on the continuation.
    assert_eq!(
        separator_pairs(&footnotes, "footnote"),
        (vec!["0".into()], vec!["1".into()])
    );
    let endnotes = part(&again, "word/endnotes.xml");
    assert_eq!(format_change_ids(&endnotes), ["2", "3"], "{endnotes}");
    assert_eq!(
        separator_pairs(&endnotes, "endnote"),
        (vec!["2".into()], vec!["3".into()])
    );
    assert_eq!(second, first);
}

#[test]
fn a_lone_separator_change_keeps_its_id() {
    let docx = source(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &notes("footnote", [None, Some(7)], ""),
        &notes("endnote", [None, None], ""),
    );
    let (first, again, second) = round_trip(&docx);
    assert!(first.contains("notes: {fmt: footnotes.r7 @AS"), "{first}");
    let footnotes = part(&again, "word/footnotes.xml");
    assert_eq!(format_change_ids(&footnotes), ["7"], "{footnotes}");
    assert_eq!(second, first);
}

#[test]
fn separator_changes_come_back_beside_a_note() {
    let note = r#"<w:footnote w:id="1"><w:p><w:r><w:footnoteRef/></w:r><w:r><w:t xml:space="preserve"> A note.</w:t></w:r></w:p></w:footnote>"#;
    let body =
        r#"<w:p><w:r><w:t>Body.</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#;
    let docx = source(
        body,
        &notes("footnote", [Some(0), Some(1)], note),
        &notes("endnote", [None, None], ""),
    );
    let (first, again, second) = round_trip(&docx);
    assert!(
        first.contains("notes: {fmt: footnotes.r0 @AS; footnotes.r1 @AS"),
        "{first}"
    );
    let footnotes = part(&again, "word/footnotes.xml");
    assert_eq!(format_change_ids(&footnotes), ["0", "1"], "{footnotes}");
    assert!(footnotes.contains("A note."), "{footnotes}");
    assert_eq!(second, first);
}

#[test]
fn separators_without_changes_stay_plain() {
    let docx = source(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &notes("footnote", [None, None], ""),
        &notes("endnote", [None, None], ""),
    );
    let (first, again, second) = round_trip(&docx);
    assert!(!first.contains("fmt:"), "{first}");
    assert!(!part(&again, "word/footnotes.xml").contains("pPrChange"));
    assert_eq!(second, first);
}

#[test]
fn more_changes_than_separators_warn() {
    let view = "---\nview: tracked\n\
                notes: {fmt: footnotes.r0 @AS; footnotes.r1 @AS; footnotes.r2 @AS, \
                empty: [footnotes]}\n\
                ---\n\n<!-- p0 -->\nBody.\n";
    let written = markdown_to_docx(view, &DocxOptions::default()).expect("the view reads");
    assert!(
        written
            .warnings
            .iter()
            .any(|w| w.contains("3 changes for the footnotes' two separators")),
        "{:?}",
        written.warnings
    );
    let footnotes = part(&written.docx, "word/footnotes.xml");
    assert_eq!(format_change_ids(&footnotes), ["0", "1"], "{footnotes}");
}

#[test]
fn single_quoted_separator_tags_preserve_unknown_author_and_date() {
    let markdown = "---\nnotes: {fmt: 'footnotes.r7 @ZZ 2026-09-26T14:55:00Z', empty: [footnotes]}\n---\n\nBody.\n";
    let written = markdown_to_docx(markdown, &DocxOptions::default()).unwrap();
    let xml = part(&written.docx, "word/footnotes.xml");
    assert_eq!(format_change_ids(&xml), ["7"], "{xml}");
    assert!(xml.contains(r#"w:author="ZZ""#), "{xml}");
    assert!(xml.contains(r#"w:date="2026-09-26T14:55:00Z""#), "{xml}");
}

#[test]
fn fresh_note_revisions_cannot_reuse_a_separator_id() {
    let markdown =
        "---\nnotes: {fmt: footnotes.r1 @AS}\n---\n\nBody[^1].\n\n[^1]: {++new note++}\n";
    let written = markdown_to_docx(markdown, &DocxOptions::default()).unwrap();
    let xml = part(&written.docx, "word/footnotes.xml");
    let changes = jubarte::changes::list_changes(&written.docx).unwrap();
    let ids: Vec<_> = changes
        .iter()
        .filter(|c| c.id.starts_with("footnotes."))
        .map(|c| &c.id)
        .collect();
    assert!(ids.len() >= 2, "{xml}");
    assert_eq!(
        ids.iter().collect::<std::collections::HashSet<_>>().len(),
        ids.len(),
        "{xml}"
    );
    common::validity::assert_word_valid_package(&written.docx);
}

#[test]
fn a_reference_separators_changes_follow_the_edited_view() {
    let reference = source(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &notes("footnote", [Some(0), Some(1)], ""),
        &notes("endnote", [None, None], ""),
    );
    for (clause, expected) in [
        ("fmt: footnotes.r9 @ZZ 2026-09-26T14:55:00Z, ", vec!["9"]),
        ("", vec![]),
    ] {
        let markdown = format!("---\nnotes: {{{clause}empty: [footnotes]}}\n---\n\nBody.\n");
        let written = markdown_to_docx(
            &markdown,
            &DocxOptions {
                reference: Some(&reference),
                ..DocxOptions::default()
            },
        )
        .unwrap();
        let xml = part(&written.docx, "word/footnotes.xml");
        assert_eq!(format_change_ids(&xml), expected, "{xml}");
        // The part-wide list above cannot say which separator carries
        // the change: this one names the separator note and the
        // continuation note separately.
        let (separator, continuation) = separator_pairs(&xml, "footnote");
        assert_eq!(continuation, Vec::<String>::new(), "{xml}");
        assert_eq!(
            separator, expected,
            "the change did not land on the separator note\n{xml}"
        );
        common::validity::assert_word_valid_package(&written.docx);
    }
}

#[test]
fn a_formatting_clause_alone_creates_the_notes_part() {
    let markdown = "---\nnotes: {fmt: footnotes.r7 @AS}\n---\n\nBody.\n";
    let written = markdown_to_docx(markdown, &DocxOptions::default()).unwrap();
    let xml = part(&written.docx, "word/footnotes.xml");
    assert_eq!(format_change_ids(&xml), ["7"], "{xml}");
}

#[test]
fn fresh_separator_ids_stay_in_words_integer_range() {
    let markdown =
        "---\nnotes: {fmt: footnotes.r2147483647 @AS; footnotes.noid @AS}\n---\n\nBody.\n";
    let written = markdown_to_docx(markdown, &DocxOptions::default()).unwrap();
    let xml = part(&written.docx, "word/footnotes.xml");
    let ids = format_change_ids(&xml);
    assert_eq!(ids.len(), 2, "{xml}");
    assert_eq!(ids[0], "2147483647");
    assert_ne!(ids[0], ids[1]);
    assert!(ids.iter().all(|id| id.parse::<i32>().is_ok()), "{xml}");
}
