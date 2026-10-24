// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Accepting or rejecting some tracked changes and leaving the rest tracked,
//! as Word's Accept/Reject This Change does.

mod common;

use common::docx::{Part, docx, docx_with_sect, part_string};
use common::validity::{assert_word_valid_package, check_word_valid_package};
use jubarte::changes::{
    Change, ChangeError, ChangeFilter, ChangeKind, MoveSide, accept_changes, accept_elements,
    list_changes, list_logical_changes, reject_changes, reject_elements,
};
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use std::collections::HashSet;

const A: &str = r#"w:author="Ann" w:date="2026-01-01T00:00:00Z""#;
const B: &str = r#"w:author="Bob" w:date="2026-01-02T00:00:00Z""#;

/// One insertion by Ann, one deletion by Bob, one paragraph formatting
/// change by Ann.
fn three_changes() -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">Keep </w:t></w:r><w:ins w:id="1" {A}><w:r><w:t>new</w:t></w:r></w:ins><w:del w:id="2" {B}><w:r><w:delText>old</w:delText></w:r></w:del></w:p><w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="3" {A}><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>Centred</w:t></w:r></w:p>"#
    ))
}

fn ids(pkg: &[u8]) -> Vec<String> {
    list_changes(pkg)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect()
}

fn filter_ids(ids: &[&str]) -> ChangeFilter {
    ChangeFilter::ids(ids.iter().copied())
}

#[test]
fn list_changes_names_each_change_by_story_and_id() {
    let changes = list_changes(&three_changes()).unwrap();
    let got: Vec<_> = changes
        .iter()
        .map(|c| (c.id.as_str(), c.kind, c.author.as_deref(), c.text.as_str()))
        .collect();
    assert_eq!(
        got,
        [
            ("r1", ChangeKind::Insertion, Some("Ann"), "new"),
            ("r2", ChangeKind::Deletion, Some("Bob"), "old"),
            ("r3", ChangeKind::Formatting, Some("Ann"), "Centred"),
        ]
    );
}

#[test]
fn accepting_one_insertion_leaves_the_others_tracked() {
    let out = accept_changes(&three_changes(), &filter_ids(&["r1"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["r2", "r3"]);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(
        xml.contains(">new</w:t>") && !xml.contains(r#"<w:ins w:id="1""#),
        "{xml}"
    );
    assert!(xml.contains("<w:delText>old</w:delText>"), "{xml}");
}

#[test]
fn rejecting_by_author_leaves_the_other_authors_changes_tracked() {
    let filter = ChangeFilter {
        authors: Some(vec!["Bob".into()]),
        ..Default::default()
    };
    let out = reject_changes(&three_changes(), &filter).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["r1", "r3"]);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(
        xml.contains(">old</w:t>") && !xml.contains("delText"),
        "{xml}"
    );
}

#[test]
fn a_kind_filter_resolves_only_that_kind() {
    let filter = ChangeFilter {
        kinds: Some(vec![ChangeKind::Formatting]),
        ..Default::default()
    };
    let out = reject_changes(&three_changes(), &filter).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["r1", "r2"]);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(!xml.contains("<w:jc"), "{xml}");
}

#[test]
fn filters_combine_as_and() {
    let filter = ChangeFilter {
        authors: Some(vec!["Ann".into()]),
        kinds: Some(vec![ChangeKind::Insertion]),
        ..Default::default()
    };
    let out = accept_changes(&three_changes(), &filter).unwrap();
    assert_eq!(ids(&out), ["r2", "r3"]);
}

#[test]
fn selecting_every_change_is_accept_or_reject_all() {
    let pkg = three_changes();
    let all = ChangeFilter::default();
    assert_eq!(
        accept_changes(&pkg, &all).unwrap(),
        accept_revisions(&pkg).unwrap()
    );
    assert_eq!(
        reject_changes(&pkg, &all).unwrap(),
        reject_revisions(&pkg).unwrap()
    );
}

#[test]
fn an_unknown_id_is_refused() {
    let err = accept_changes(&three_changes(), &filter_ids(&["r9"])).unwrap_err();
    assert!(
        matches!(&err, ChangeError::UnknownChange(id) if id == "r9"),
        "{err:?}"
    );
}

/// Word accepts or rejects a move as one change: selecting either side
/// resolves both, range markers included.
#[test]
fn a_move_resolves_both_sides() {
    let pkg = docx(&format!(
        r#"<w:p><w:moveFromRangeStart w:id="10" w:name="move1" {A}/><w:moveFrom w:id="11" {A}><w:r><w:t>Moved</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="10"/></w:p><w:p><w:r><w:t>Stays</w:t></w:r></w:p><w:p><w:moveToRangeStart w:id="12" w:name="move1" {A}/><w:moveTo w:id="13" {A}><w:r><w:t>Moved</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="12"/><w:ins w:id="14" {B}><w:r><w:t xml:space="preserve"> too</w:t></w:r></w:ins></w:p>"#
    ));
    let listed = list_changes(&pkg).unwrap();
    assert_eq!(
        listed.iter().map(|c| c.kind).collect::<Vec<_>>(),
        [ChangeKind::Move, ChangeKind::Move, ChangeKind::Insertion]
    );
    assert_eq!(listed[0].move_name.as_deref(), Some("move1"));
    assert_eq!(
        listed.iter().map(|c| c.move_side).collect::<Vec<_>>(),
        [Some(MoveSide::From), Some(MoveSide::To), None]
    );
    for side in ["r11", "r13"] {
        let out = accept_changes(&pkg, &filter_ids(&[side])).unwrap();
        assert_word_valid_package(&out);
        assert_eq!(ids(&out), ["r14"], "accepting {side}");
        let xml = part_string(&out, "word/document.xml").unwrap();
        assert_eq!(xml.matches(">Moved<").count(), 1, "{xml}");
        assert!(
            !xml.contains("moveFrom") && !xml.contains("moveTo"),
            "{xml}"
        );
    }
    let out = reject_changes(&pkg, &filter_ids(&["r13"])).unwrap();
    assert_eq!(ids(&out), ["r14"]);
}

/// A paragraph mark's own insertion is an element of its own, and one
/// change with the paragraph's inserted text (plan 11): settling the text
/// alone takes the element API.
#[test]
fn an_inserted_paragraph_mark_is_its_own_element_of_its_paragraphs_change() {
    let pkg = docx(&format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="5" {A}/></w:rPr></w:pPr><w:ins w:id="6" {A}><w:r><w:t>First</w:t></w:r></w:ins></w:p><w:p><w:r><w:t>Second</w:t></w:r></w:p>"#
    ));
    let listed = list_changes(&pkg).unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].target, "paragraph_mark");
    assert_eq!(listed[1].target, "text");
    let logical = list_logical_changes(&pkg).unwrap();
    assert_eq!(logical.len(), 1, "{logical:#?}");
    assert_eq!(logical[0].id, "r6+5");
    assert!(matches!(
        reject_changes(&pkg, &filter_ids(&["r6"])),
        Err(ChangeError::PartOfChange { ref whole, .. }) if whole == "r6+5"
    ));
    let out = reject_elements(&pkg, &filter_ids(&["r6"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["r5"]);
    let whole = reject_changes(&pkg, &filter_ids(&["r6+5"])).unwrap();
    assert_word_valid_package(&whole);
    assert!(ids(&whole).is_empty());
}

#[test]
fn header_changes_carry_their_story() {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:ins w:id="7" {B}><w:r><w:t>Head</w:t></w:r></w:ins></w:p></w:hdr>"#
    );
    let header: &'static str = Box::leak(header.into_boxed_str());
    let pkg = docx_with_sect(
        &format!(r#"<w:p><w:ins w:id="8" {A}><w:r><w:t>Body</w:t></w:r></w:ins></w:p>"#),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    assert_eq!(ids(&pkg), ["r8", "header1.r7"]);
    let out = reject_changes(&pkg, &filter_ids(&["header1.r7"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["r8"]);
    let hx = part_string(&out, "word/header1.xml").unwrap();
    assert!(!hx.contains("Head"), "{hx}");
}

#[test]
fn a_filter_reads_from_json() {
    let f: ChangeFilter =
        serde_json::from_str(r#"{"ids":["r1"],"authors":["Ann"],"kinds":["insertion","move"]}"#)
            .unwrap();
    assert_eq!(f.kinds, Some(vec![ChangeKind::Insertion, ChangeKind::Move]));
    assert!(serde_json::from_str::<ChangeFilter>(r#"{"author":"Ann"}"#).is_err());
}

/// A list given empty selects nothing (an agent's empty id list must not
/// resolve every change); `{}` selects every change.
#[test]
fn an_empty_list_selects_nothing() {
    let pkg = three_changes();
    for filter in [r#"{"ids":[]}"#, r#"{"authors":[]}"#, r#"{"kinds":[]}"#] {
        let f: ChangeFilter = serde_json::from_str(filter).unwrap();
        assert_eq!(accept_changes(&pkg, &f).unwrap(), pkg, "{filter}");
        assert_eq!(reject_changes(&pkg, &f).unwrap(), pkg, "{filter}");
    }
    let all: ChangeFilter = serde_json::from_str("{}").unwrap();
    assert!(ids(&accept_changes(&pkg, &all).unwrap()).is_empty());
}

/// The bench fixtures copied into this repository.
fn bench() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/neurotic_docx_bench")
}

/// Owner, record type, id and payload (without id), in stylesheet order.
fn style_histories(pkg: &[u8]) -> Vec<(String, String, String, String)> {
    use jubarte::namespaces::W;
    use jubarte::xmllinq::Dom;
    let Some(xml) = part_string(pkg, "word/styles.xml") else {
        return Vec::new();
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let mut out = Vec::new();
    for style in dom.elements(root, Some(&W::name("style"))) {
        let owner = dom
            .attribute(style, &W::name("styleId"))
            .unwrap()
            .to_string();
        for node in dom.descendants(style, None) {
            let Some(name) = dom.name(node) else { continue };
            if name.namespace_name() != W::URI
                || !matches!(name.local_name(), "rPrChange" | "pPrChange")
            {
                continue;
            }
            let id = dom.attribute(node, &W::id()).unwrap().to_string();
            let copy = dom.clone_subtree(node);
            dom.set_attribute_value(copy, &W::id(), None);
            out.push((
                owner.clone(),
                name.local_name().to_string(),
                id,
                dom.serialize_element(copy),
            ));
        }
    }
    out
}

/// A copied history is a new change; it must be fresh, preserve a retained
/// source history's payload, and leave that source history's id intact.
fn fresh_style_copies(before: &[u8], after: &[u8], selected: &ChangeFilter) -> HashSet<String> {
    let old = style_histories(before);
    let new = style_histories(after);
    let max_id = list_changes(before)
        .unwrap()
        .iter()
        .map(|c| c.id.rsplit('r').next().unwrap().parse::<u64>().unwrap())
        .max()
        .unwrap_or(0);
    let selected_ids: HashSet<String> = list_changes(before)
        .unwrap()
        .into_iter()
        .filter(|c| selected.matches(c))
        .map(|c| c.id)
        .collect();
    let mut copies = HashSet::new();
    for (owner, kind, id, payload) in &new {
        if old
            .iter()
            .any(|r| &r.0 == owner && &r.1 == kind && &r.2 == id)
        {
            continue;
        }
        assert!(
            id.parse::<u64>().unwrap() > max_id,
            "copied {owner}/{kind}:{id} must be package-fresh (> {max_id})"
        );
        assert!(
            old.iter().any(|source| {
                &source.1 == kind
                    && &source.3 == payload
                    && !selected_ids.contains(&format!("styles.r{}", source.2))
                    && new.contains(source)
            }),
            "copied {owner}/{kind}:{id} must keep a retained source history intact"
        );
        copies.insert(format!("styles.r{id}"));
    }
    copies
}

#[test]
fn rejecting_alternate_changes_allocates_fresh_style_history_ids() {
    // Smallest of the 11 failing redlines in the 200-input selection: 23,013 bytes.
    let redline = std::fs::read(bench().join("corpus/word/tracking_without_comments/docx/83799a0610__vs__333bfe069a_redline_2a068b16e5.docx")).unwrap();
    let assert_unique = |pkg: &[u8]| {
        let records = style_histories(pkg);
        let mut seen = HashSet::new();
        for (owner, kind, id, _) in records {
            assert!(
                seen.insert((kind.clone(), id.clone())),
                "duplicate {kind}:{id} in word/styles.xml at {owner}"
            );
        }
    };
    assert_unique(&redline);
    let listed = list_changes(&redline).unwrap();
    let half = ChangeFilter::ids(listed.iter().step_by(2).map(|c| c.id.clone()));
    let partial = reject_elements(&redline, &half).unwrap();
    assert_unique(&partial);
    let copies = fresh_style_copies(&redline, &partial, &half);
    assert_eq!(
        copies.len(),
        1,
        "Heading7Char inherits Normal's kept run history"
    );
    let after = style_histories(&partial);
    for record in style_histories(&redline) {
        if !half.matches(
            listed
                .iter()
                .find(|c| c.id == format!("styles.r{}", record.2))
                .unwrap(),
        ) {
            assert!(
                after.contains(&record),
                "untouched history changed: {record:?}"
            );
        }
    }
    assert_word_valid_package(&partial);
    let finished = reject_changes(&partial, &ChangeFilter::default()).unwrap();
    assert!(list_changes(&finished).unwrap().is_empty());
    let texts = |pkg: &[u8]| {
        jubarte::inspect::paragraphs(pkg)
            .unwrap()
            .into_iter()
            .map(|p| p.text)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        texts(&finished),
        texts(&reject_revisions(&redline).unwrap())
    );
}

#[test]
fn half_way_style_reject_ignores_div_and_bookmark_ids() {
    use jubarte::opc::PartFs;
    let redline = std::fs::read(bench().join("corpus/word/tracking_without_comments/docx/83799a0610__vs__333bfe069a_redline_2a068b16e5.docx")).unwrap();
    let mut pkg = PartFs::open(&redline).unwrap();
    pkg.set_part("word/webSettings.xml", br#"<w:webSettings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:divs><w:div w:id="2147483647"/></w:divs></w:webSettings>"#.to_vec());
    let mut xml = pkg.part_string("word/document.xml").unwrap();
    let first_p = xml.find("<w:p ").or_else(|| xml.find("<w:p>")).unwrap();
    let at = first_p + xml[first_p..].find('>').unwrap() + 1;
    xml.insert_str(
        at,
        r#"<w:bookmarkStart w:id="900000" w:name="Alice"/><w:bookmarkEnd w:id="900000"/>"#,
    );
    pkg.set_part("word/document.xml", xml.into_bytes());
    let redline = pkg.to_zip().unwrap();
    let listed = list_changes(&redline).unwrap();
    let max_id = listed
        .iter()
        .map(|c| c.id.rsplit('r').next().unwrap().parse::<u64>().unwrap())
        .max()
        .unwrap();
    let half = ChangeFilter::ids(listed.iter().step_by(2).map(|c| c.id.clone()));
    let partial = reject_elements(&redline, &half).unwrap();
    let old = style_histories(&redline);
    let after = style_histories(&partial);
    let copies: Vec<_> = after.iter().filter(|r| !old.contains(r)).collect();
    assert_eq!(copies.len(), 1, "Heading7Char inherits Normal's history");
    let bound = max_id + copies.len() as u64;
    for (_, _, id, _) in &after {
        let id = id.parse::<u64>().unwrap();
        assert!(
            id <= bound && id <= 2147483647,
            "style revision id {id} exceeds input revision max {max_id} + {} new records (bound {bound}) or signed 32-bit range",
            copies.len()
        );
    }
    assert_eq!(fresh_style_copies(&redline, &partial, &half).len(), 1);
    assert_word_valid_package(&partial);
}

/// The last signed id can be allocated once; subsequent copies use the
/// lowest unused revision ids, reserving records in other parts as well.
#[test]
fn style_history_id_overflow_uses_lowest_unused_package_ids() {
    check_style_history_id_limit(2147483647);
}

#[test]
fn style_history_id_allocation_crosses_signed_limit_safely() {
    check_style_history_id_limit(2147483646);
}

fn check_style_history_id_limit(max_id: u64) {
    let styles = format!(
        r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:b/><w:rPrChange w:id="{max_id}" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:rPr/></w:rPrChange></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading7"><w:name w:val="heading 7"/><w:basedOn w:val="Normal"/><w:link w:val="AliceChar"/><w:rPr><w:i/><w:rPrChange w:id="1" w:author="Bob" w:date="2026-01-01T00:00:00Z"><w:rPr/></w:rPrChange></w:rPr></w:style>
<w:style w:type="character" w:styleId="AliceChar"><w:name w:val="Alice"/><w:link w:val="Heading7"/></w:style>
<w:style w:type="paragraph" w:styleId="CarolPara"><w:name w:val="Carol Diaz"/><w:basedOn w:val="Heading7"/><w:link w:val="CarolChar"/></w:style>
<w:style w:type="character" w:styleId="CarolChar"><w:name w:val="Carol Diaz Char"/><w:basedOn w:val="AliceChar"/><w:link w:val="CarolPara"/></w:style>
</w:styles>"#
    );
    let source = docx_with_sect(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading7"/></w:pPr><w:r><w:t>Alice and Bob</w:t></w:r></w:p>"#,
        &[
            Part {
                name: "word/styles.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
                xml: &styles,
            },
            Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:ins w:id="0" w:author="Carol Diaz" w:date="2026-01-01T00:00:00Z"><w:r><w:t>Header</w:t></w:r></w:ins></w:p></w:hdr>"#,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX1"/>"#,
    );
    let partial = reject_changes(&source, &filter_ids(&["styles.r1"])).unwrap();
    let before = style_histories(&source);
    let after = style_histories(&partial);
    let mut copies: Vec<_> = after
        .iter()
        .filter(|r| !before.contains(r))
        .map(|r| r.2.parse::<u64>().unwrap())
        .collect();
    copies.sort_unstable();
    assert!(
        copies.iter().all(|&id| id <= 2147483647),
        "copied revision ids {copies:?} exceed signed 32-bit range with input max {max_id}"
    );
    let expected = if max_id == 2147483647 {
        vec![2, 3]
    } else {
        vec![2, 2147483647]
    };
    assert_eq!(
        copies, expected,
        "use max + 1 until exhausted, then the lowest unused ids"
    );
    assert!(
        after
            .iter()
            .any(|r| r.0 == "Normal" && r.2 == max_id.to_string()),
        "retained history id must not change"
    );
    assert!(
        part_string(&partial, "word/header1.xml")
            .unwrap()
            .contains(r#"w:id="0""#)
    );
    assert_word_valid_package(&partial);
}

/// On Word's own redlines (the bench's `rejected_tracking` set): resolving every other change first and the rest after ends
/// where resolving them all at once does, and the half-way document is
/// Word-valid with every unselected id still listed.
#[test]
fn resolving_in_two_halves_ends_where_resolving_all_does() {
    use jubarte::debug::{Check, Options, report};
    let bench = bench();
    let selection =
        std::fs::read_to_string(bench.join("corpus/word/notices/rejected_tracking_selection.csv"))
            .unwrap();
    let mut rows = selection.lines();
    let header: Vec<&str> = rows.next().unwrap().split(',').collect();
    let docx_col = header.iter().position(|h| *h == "docx").unwrap();
    let opts = Options {
        checks: vec![Check::Text],
        ..Default::default()
    };
    let (mut pairs, mut differ) = (0, Vec::new());
    // A malformed row or a missing redline fails; nothing listed is skipped.
    for row in rows.filter(|r| !r.trim().is_empty()) {
        let Some(docx) = row.split(',').nth(docx_col) else {
            differ.push(format!("malformed row: {row}"));
            continue;
        };
        let Ok(redline) = std::fs::read(bench.join("corpus/word").join(docx)) else {
            differ.push(format!("{docx}: missing"));
            continue;
        };
        let listed = list_changes(&redline).unwrap();
        // What Word's own redline already breaks is not the resolution's.
        let inherited: HashSet<String> = check_word_valid_package(&redline)
            .errors
            .into_iter()
            .collect();
        let half: Vec<String> = listed.iter().step_by(2).map(|c| c.id.clone()).collect();
        let first = ChangeFilter::ids(half);
        for (name, one, all) in [
            (
                "accept",
                accept_elements as fn(&[u8], &ChangeFilter) -> Result<Vec<u8>, ChangeError>,
                accept_revisions as fn(&[u8]) -> Result<Vec<u8>, _>,
            ),
            ("reject", reject_elements, reject_revisions),
        ] {
            pairs += 1;
            let partial = one(&redline, &first).unwrap();
            let broken: Vec<String> = check_word_valid_package(&partial)
                .errors
                .into_iter()
                .filter(|e| !inherited.contains(e))
                .collect();
            if !broken.is_empty() {
                differ.push(format!("{docx} [{name}]: invalid half-way: {broken:?}"));
                continue;
            }
            let left: HashSet<String> = ids(&partial).into_iter().collect();
            let selected = |c: &Change| {
                first.matches(c)
                    || c.move_name.as_ref().is_some_and(|m| {
                        listed
                            .iter()
                            .any(|o| o.move_name.as_ref() == Some(m) && first.matches(o))
                    })
            };
            // Resolving a change so its content goes takes what it holds.
            let removes = |c: &Change| match (name, c.kind) {
                ("accept", ChangeKind::Deletion) | ("reject", ChangeKind::Insertion) => true,
                (_, ChangeKind::Move) => {
                    (c.move_side == Some(MoveSide::From)) == (name == "accept")
                }
                _ => false,
            };
            let by_id: std::collections::HashMap<&str, &Change> =
                listed.iter().map(|c| (c.id.as_str(), c)).collect();
            let swept = |c: &Change| {
                let mut at = c.inside.as_deref();
                while let Some(id) = at {
                    let holder = by_id[id];
                    if selected(holder) && removes(holder) {
                        return true;
                    }
                    at = holder.inside.as_deref();
                }
                false
            };
            let expected: HashSet<String> = listed
                .iter()
                .filter(|c| !selected(c) && !swept(c))
                .map(|c| c.id.clone())
                .chain(fresh_style_copies(&redline, &partial, &first))
                .collect();
            // A note goes with a reference its resolved change removed, a
            // header or footer with the section a resolved mark ended.
            let story_went = |id: &str| {
                let story = id.split(':').next().unwrap_or("");
                matches!(story, "footnotes" | "endnotes")
                    || story != "body"
                        && part_string(&partial, &format!("word/{story}.xml")).is_none()
            };
            // A cell's final mark inserted or deleted with its row goes
            // with the row (#1669): the two resolve together.
            let with_row = |id: &str| {
                let Some(c) = by_id.get(id) else {
                    return false;
                };
                c.target == "paragraph_mark"
                    && c.inside
                        .as_deref()
                        .and_then(|h| by_id.get(h))
                        .is_some_and(|holder| {
                            matches!(holder.target, "table_row" | "table_cell")
                                && holder.kind == c.kind
                                && selected(holder)
                        })
            };
            if !left.is_subset(&expected)
                || expected
                    .difference(&left)
                    .any(|id| !story_went(id) && !with_row(id))
            {
                let mut lost: Vec<_> = expected.difference(&left).collect();
                let mut stayed: Vec<_> = left.difference(&expected).collect();
                lost.sort();
                stayed.sort();
                differ.push(format!("{docx} [{name}]: lost {lost:?}, stayed {stayed:?}"));
                continue;
            }
            let finished = one(&partial, &ChangeFilter::default()).unwrap();
            let out = report(&all(&redline).unwrap(), Some(&finished), &opts).unwrap();
            // A paragraph mark resolved on its own before a table (or at a
            // story's end) cannot merge away and stays, as in Word; resolving
            // the table later leaves the paragraph empty or split where
            // resolving both at once joins it (A.5a). The text may not differ.
            let side = |sign: &str| -> String {
                out.lines()
                    .filter(|l| l.starts_with(sign))
                    .filter_map(|l| l.split_once('¶'))
                    .map(|(_, t)| t.strip_prefix("  ").unwrap_or(t))
                    .collect()
            };
            let only_breaks = out.lines().all(|l| {
                l.ends_with("differ")
                    || l.ends_with("differs")
                    || l.trim() == "~"
                    || l.starts_with("-A   ¶")
                    || l.starts_with("+B   ¶")
            }) && side("-A") == side("+B");
            if out != "text identical\n" && !only_breaks {
                differ.push(format!("{docx} [{name}]:\n{out}"));
            }
        }
    }
    assert!(pairs > 0);
    assert!(
        differ.is_empty(),
        "{} of {pairs} differ:\n{}",
        differ.len(),
        differ.join("\n")
    );
}

/// Rejecting an inserted row that holds a comment's start, while the text
/// holding its end and reference stays tracked, keeps the comment: its
/// range collapses onto the surviving end instead of losing its start
/// (Word-invalid; bench 7787357743).
#[test]
fn a_comment_cut_by_a_resolved_row_keeps_its_range() {
    let comments = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="0" w:author="Ann" w:date="2026-01-01T00:00:00Z" w:initials="A"><w:p><w:r><w:t>Note</w:t></w:r></w:p></w:comment></w:comments>"#;
    let pkg = common::docx::docx_with(
        &format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:trPr><w:ins w:id="1" {A}/></w:trPr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:commentRangeStart w:id="0"/><w:ins w:id="3" {A}><w:r><w:t>Cell</w:t></w:r></w:ins></w:p></w:tc></w:tr></w:tbl><w:p><w:ins w:id="2" {B}><w:r><w:t>Kept</w:t></w:r></w:ins><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#
        ),
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: comments,
        }],
    );
    let out = reject_changes(&pkg, &filter_ids(&["r1"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["r2"]);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(!xml.contains("Cell"), "{xml}");
    assert!(
        xml.contains(r#"<w:commentRangeStart w:id="0" /><w:commentRangeEnd w:id="0" />"#),
        "{xml}"
    );
}

/// Two adjacent tables alike in every whole-table property, the second
/// carrying a table grid change. Word's save holds such tables as one, but a
/// grid change left tracked keeps its own table.
#[test]
fn a_kept_grid_change_keeps_its_table() {
    let cell = |t: &str| {
        format!(r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p>{t}</w:p></w:tc>"#)
    };
    let tbl = |grid_change: &str, t: &str| {
        format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/>{grid_change}</w:tblGrid><w:tr>{}</w:tr></w:tbl>"#,
            cell(t)
        )
    };
    let pkg = docx(&format!(
        r#"{}{}<w:p/>"#,
        tbl(
            "",
            &format!(r#"<w:ins w:id="1" {A}><w:r><w:t>One</w:t></w:r></w:ins>"#)
        ),
        tbl(
            r#"<w:tblGridChange w:id="2"><w:tblGrid><w:gridCol w:w="1500"/></w:tblGrid></w:tblGridChange>"#,
            "<w:r><w:t>Two</w:t></w:r>"
        ),
    ));
    for out in [
        accept_changes(&pkg, &filter_ids(&["r1"])).unwrap(),
        reject_changes(&pkg, &filter_ids(&["r1"])).unwrap(),
    ] {
        assert_word_valid_package(&out);
        assert_eq!(ids(&out), ["r2"]);
    }
}

/// A moved paragraph's mark sits in its `pPr`, before the range start Word
/// writes after it: the mark is part of the move and resolves with it.
#[test]
fn a_moved_paragraph_mark_resolves_with_its_move() {
    let mark = |side: &str, id: u32| {
        format!(r#"<w:pPr><w:rPr><w:{side} w:id="{id}" {A}/></w:rPr></w:pPr>"#)
    };
    let pkg = docx(&format!(
        r#"<w:p>{}<w:moveFromRangeStart w:id="10" w:name="move1" {A}/><w:moveFrom w:id="11" {A}><w:r><w:t>Moved</w:t></w:r></w:moveFrom></w:p><w:p><w:moveFromRangeEnd w:id="10"/><w:r><w:t>Stays</w:t></w:r></w:p><w:p>{}<w:moveToRangeStart w:id="12" w:name="move1" {A}/><w:moveTo w:id="13" {A}><w:r><w:t>Moved</w:t></w:r></w:moveTo></w:p><w:p><w:moveToRangeEnd w:id="12"/><w:ins w:id="14" {B}><w:r><w:t>Tail</w:t></w:r></w:ins></w:p>"#,
        mark("moveFrom", 20),
        mark("moveTo", 21),
    ));
    let listed = list_changes(&pkg).unwrap();
    assert!(
        listed
            .iter()
            .filter(|c| c.kind == ChangeKind::Move)
            .all(|c| c.move_name.as_deref() == Some("move1")),
        "{listed:?}"
    );
    for resolve in [accept_changes, reject_changes] {
        for side in ["r20", "r21", "r13"] {
            let out = resolve(&pkg, &filter_ids(&[side])).unwrap();
            assert_word_valid_package(&out);
            assert_eq!(ids(&out), ["r14"], "resolving {side}");
        }
    }
}

/// A revision mark inside a formatting change's recorded properties is
/// history, not a change of its own; the paragraph's formatting changes go
/// with its mark when resolving the mark removes it.
#[test]
fn recorded_marks_are_history_and_mark_formatting_goes_with_the_mark() {
    let pkg = docx(&format!(
        r#"<w:p><w:pPr><w:jc w:val="both"/><w:rPr><w:del w:id="1" {A}/><w:sz w:val="22"/><w:rPrChange w:id="2" {A}><w:rPr><w:del w:id="3" {A}/><w:sz w:val="28"/></w:rPr></w:rPrChange></w:rPr><w:pPrChange w:id="4" {A}><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>Joined</w:t></w:r></w:p><w:p><w:r><w:t>Next</w:t></w:r></w:p>"#
    ));
    let listed = list_changes(&pkg).unwrap();
    let got: Vec<_> = listed
        .iter()
        .map(|c| (c.id.as_str(), c.inside.as_deref()))
        .collect();
    assert_eq!(got, [("r1", None), ("r2", Some("r1")), ("r4", Some("r1")),]);
    let out = accept_changes(&pkg, &filter_ids(&["r1"])).unwrap();
    assert_word_valid_package(&out);
    assert!(ids(&out).is_empty(), "{:?}", ids(&out));
    let out = reject_changes(&pkg, &filter_ids(&["r2"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["r1", "r4"]);
}

/// Word records a deleted cell's properties with their own `w:cellDel` in
/// its `w:tcPrChange`: history, like a recorded mark. Resolving another
/// change keeps the cell, its deletion and its formatting change tracked
/// (the bench's 32328b55db and 3890d0a960 redlines lost three cells).
#[test]
fn a_cell_deletion_recorded_in_its_formatting_history_stays_tracked() {
    let cell = |props: &str, text: &str| {
        format!(
            r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/>{props}</w:tcPr><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>"#
        )
    };
    let pkg = docx(&format!(
        r#"<w:p><w:ins w:id="1" {A}><w:r><w:t>Elsewhere</w:t></w:r></w:ins></w:p><w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid><w:tr>{}{}</w:tr></w:tbl><w:p/>"#,
        cell("", "Kept"),
        cell(
            &format!(
                r#"<w:cellDel w:id="2" {A}/><w:tcPrChange w:id="3" {A}><w:tcPr><w:tcW w:w="1500" w:type="dxa"/><w:cellDel w:id="4" {A}/></w:tcPr></w:tcPrChange>"#
            ),
            "Gone"
        ),
    ));
    let listed: Vec<String> = list_changes(&pkg)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(listed, ["r1", "r2", "r3"]);
    for resolve in [accept_changes, reject_changes] {
        let out = resolve(&pkg, &filter_ids(&["r1"])).unwrap();
        assert_word_valid_package(&out);
        assert_eq!(ids(&out), ["r2", "r3"]);
        let xml = part_string(&out, "word/document.xml").unwrap();
        assert_eq!(xml.matches("<w:tc>").count(), 2, "{xml}");
        assert!(xml.contains("Gone"), "{xml}");
        assert!(xml.contains(r#"<w:cellDel w:id="4""#), "{xml}");
    }
}

/// Formatting history recorded inside a kept change is part of it: a
/// list's `w:numberingChange` saved in a `w:pPrChange`, and a recorded
/// vertical merge in a `w:tcPrChange`, stay as recorded when another change
/// is accepted or rejected.
#[test]
fn formatting_history_inside_a_kept_change_stays_as_recorded() {
    let pkg = docx(&format!(
        r#"<w:p><w:ins w:id="1" {A}><w:r><w:t>Elsewhere</w:t></w:r></w:ins></w:p><w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="3" {A}><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/><w:numberingChange w:id="4" {A} w:original="%1."/></w:numPr></w:pPr></w:pPrChange></w:pPr><w:r><w:t>Listed</w:t></w:r></w:p><w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/><w:tcPrChange w:id="5" {A}><w:tcPr><w:tcW w:w="1500" w:type="dxa"/><w:cellMerge w:id="6" {A} w:vMerge="rest"/></w:tcPr></w:tcPrChange></w:tcPr><w:p><w:r><w:t>Top</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/><w:tcPrChange w:id="7" {A}><w:tcPr><w:tcW w:w="1500" w:type="dxa"/><w:cellMerge w:id="8" {A} w:vMerge="cont"/></w:tcPr></w:tcPrChange></w:tcPr><w:p><w:r><w:t>Below</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>"#
    ));
    let listed: Vec<String> = list_changes(&pkg)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(listed, ["r1", "r3", "r5", "r7"]);
    for resolve in [accept_changes, reject_changes] {
        let out = resolve(&pkg, &filter_ids(&["r1"])).unwrap();
        assert_word_valid_package(&out);
        assert_eq!(ids(&out), ["r3", "r5", "r7"]);
        let xml = part_string(&out, "word/document.xml").unwrap();
        assert!(xml.contains(r#"<w:numberingChange w:id="4""#), "{xml}");
        assert!(xml.contains(r#"w:original="%1.""#), "{xml}");
        assert_eq!(xml.matches("<w:cellMerge ").count(), 2, "{xml}");
        assert!(!xml.contains("<w:vMerge"), "{xml}");
    }
}

/// The short id and the long id the JSON prints select the same change;
/// expired spellings are refused with the settled one (Arthur, 2026-10-13).
#[test]
fn every_spelling_of_a_change_id_selects_the_same_change() {
    let pkg = three_changes();
    let want = accept_changes(&pkg, &filter_ids(&["r2"])).unwrap();
    for spelling in ["r2", "r2"] {
        assert_eq!(
            accept_changes(&pkg, &filter_ids(&[spelling])).unwrap(),
            want,
            "{spelling}"
        );
    }
    let both = accept_changes(&pkg, &filter_ids(&["r1", "r2"])).unwrap();
    assert_eq!(accept_changes(&pkg, &filter_ids(&["r1+2"])).unwrap(), both);
    for (expired, settled) in [("2", "r2"), ("#2", "r2"), ("1+2", "r1+2"), ("#1+2", "r1+2")] {
        let err = accept_changes(&pkg, &filter_ids(&[expired])).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!(
                "{expired}: expected a change id such as r12, r1+2 or header1.r1; \
                 the settled spelling is {settled}"
            )
        );
    }
    let err = accept_changes(&pkg, &filter_ids(&["body.r2"])).unwrap_err();
    assert!(matches!(err, ChangeError::InvalidId(_)), "{err:?}");
    let err = accept_changes(&pkg, &filter_ids(&["c2"])).unwrap_err();
    assert_eq!(
        err.to_string(),
        "c2: expected a change id such as r12, r1+2 or header1.r1"
    );
    assert!(matches!(err, ChangeError::InvalidId(_)), "{err:?}");
    let err = accept_changes(&pkg, &filter_ids(&["r9"])).unwrap_err();
    assert!(
        matches!(&err, ChangeError::UnknownChange(id) if id == "r9"),
        "{err:?}"
    );
    assert_eq!(err.to_string(), "no tracked change r9");
}

/// Listings carry the short id and the paragraph each change sits in.
#[test]
fn changes_carry_their_short_id_and_paragraph() {
    let changes = list_changes(&three_changes()).unwrap();
    let got: Vec<_> = changes
        .iter()
        .map(|c| (c.r#ref.as_str(), c.paragraph.as_deref()))
        .collect();
    assert_eq!(
        got,
        [("r1", Some("p0")), ("r2", Some("p0")), ("r3", Some("p1")),]
    );
    let json = serde_json::to_string(&changes[0]).unwrap();
    assert!(json.starts_with(r#"{"id":"r1","ref":"r1","#), "{json}");
    assert!(json.contains(r#""paragraph":"p0""#), "{json}");
}

#[test]
fn changes_text_is_the_listing() {
    let changes = list_changes(&three_changes()).unwrap();
    assert_eq!(
        jubarte::changes::changes_text(&changes),
        "r1\tinsertion\ttext\tAnn\tp0\t\"new\"\n\
         r2\tdeletion\ttext\tBob\tp0\t\"old\"\n\
         r3\tformatting\tproperties\tAnn\tp1\t\"Centred\"\n\
         3 change(s)\n"
    );
}

/// A duplicate `w:ins` id names two changes: resolving it is refused, and
/// the message lists both and names the repair.
#[test]
fn a_duplicate_insertion_id_is_ambiguous() {
    let pkg = docx(&format!(
        r#"<w:p><w:ins w:id="4" {A}><w:r><w:t>one</w:t></w:r></w:ins><w:ins w:id="4" {A}><w:r><w:t>two</w:t></w:r></w:ins></w:p>"#
    ));
    for spelling in ["r4", "r4"] {
        let err = accept_changes(&pkg, &filter_ids(&[spelling])).unwrap_err();
        assert!(matches!(&err, ChangeError::Ambiguous { candidates, .. } if candidates.len() == 2));
        assert_eq!(
            err.to_string(),
            "r4 names 2 changes (duplicate w:id; validate --repair renumbers them): r4 insertion \"one\" in p0; r4 insertion \"two\" in p0"
        );
    }
}

/// Two paragraphs' formatting changes sharing an id are ambiguous too, but
/// `validate --repair` does not renumber them, so the message does not
/// promise it.
#[test]
fn a_shared_formatting_id_is_ambiguous_without_the_repair_hint() {
    let pkg = docx(&format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="3" {A}><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>One</w:t></w:r></w:p><w:p><w:pPr><w:jc w:val="right"/><w:pPrChange w:id="3" {A}><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>Two</w:t></w:r></w:p>"#
    ));
    let err = reject_changes(&pkg, &filter_ids(&["r3"])).unwrap_err();
    assert_eq!(
        err.to_string(),
        "r3 names 2 changes: r3 formatting \"One\" in p0; r3 formatting \"Two\" in p1"
    );
}

/// Word gives a style's paragraph and run formatting changes one `w:id`:
/// one change, listed once as `styles.rN`, resolved whole.
#[test]
fn a_styles_one_id_pair_is_one_change() {
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="21" {A}><w:pPr/></w:pPrChange></w:pPr><w:rPr><w:i/><w:rPrChange w:id="21" {A}><w:rPr/></w:rPrChange></w:rPr></w:style></w:styles>"#
    );
    let styles: &'static str = Box::leak(styles.into_boxed_str());
    let pkg = common::docx::docx_with(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: styles,
        }],
    );
    let listed = list_changes(&pkg).unwrap();
    assert_eq!(listed.len(), 1, "{listed:?}");
    assert_eq!(listed[0].r#ref, "styles.r21");
    assert_eq!(listed[0].paragraph, None);
    let out = reject_changes(&pkg, &filter_ids(&["styles.r21"])).unwrap();
    assert_word_valid_package(&out);
    assert!(ids(&out).is_empty());
    let xml = part_string(&out, "word/styles.xml").unwrap();
    assert!(
        !xml.contains("PrChange") && !xml.contains("<w:i/>"),
        "{xml}"
    );
}

/// Only a style's own `w:pPr`/`w:rPr` records pair: a table style's
/// conditional (`w:tblStylePr`) record sharing the id is another change, so
/// the id is ambiguous instead of one record vanishing from the list.
#[test]
fn a_conditional_style_record_does_not_pair() {
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="table" w:styleId="Grid"><w:name w:val="Grid"/><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="9" {A}><w:pPr/></w:pPrChange></w:pPr><w:tblStylePr w:type="firstRow"><w:rPr><w:b/><w:rPrChange w:id="9" {B}><w:rPr/></w:rPrChange></w:rPr></w:tblStylePr></w:style></w:styles>"#
    );
    let styles: &'static str = Box::leak(styles.into_boxed_str());
    let pkg = common::docx::docx_with(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: styles,
        }],
    );
    let listed = list_changes(&pkg).unwrap();
    assert_eq!(listed.len(), 2, "{listed:?}");
    assert!(matches!(
        reject_changes(&pkg, &filter_ids(&["styles.r9"])),
        Err(ChangeError::Ambiguous { .. })
    ));
}

/// A header change prints `header1.rN` and resolves by it.
#[test]
fn a_header_change_resolves_by_its_short_id() {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:ins w:id="7" {B}><w:r><w:t>Head</w:t></w:r></w:ins></w:p></w:hdr>"#
    );
    let header: &'static str = Box::leak(header.into_boxed_str());
    let pkg = docx_with_sect(
        &format!(r#"<w:p><w:ins w:id="7" {A}><w:r><w:t>Body</w:t></w:r></w:ins></w:p>"#),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let refs: Vec<String> = list_changes(&pkg)
        .unwrap()
        .into_iter()
        .map(|c| format!("{} {}", c.r#ref, c.paragraph.unwrap_or_default()))
        .collect();
    assert_eq!(refs, ["r7 p0", "header1.r7 header1"]);
    let out = reject_changes(&pkg, &filter_ids(&["header1.r7"])).unwrap();
    assert_eq!(ids(&out), ["r7"]);
}

/// A text box Word stores twice: the `mc:Choice` copy and its `mc:Fallback`
/// twin, `extra` adding a Fallback-only insertion.
fn boxed(extra: &str) -> Vec<u8> {
    let copy = |id: u32, more: &str| {
        format!(
            r#"<w:txbxContent><w:p><w:ins w:id="{id}" {A}><w:r><w:t>Boxed</w:t></w:r></w:ins>{more}</w:p></w:txbxContent>"#
        )
    };
    docx(&format!(
        r#"<w:p><w:r><w:t>Lead</w:t></w:r><w:r><mc:AlternateContent><mc:Choice Requires="wps">{}</mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent></w:r></w:p>"#,
        copy(5, ""),
        copy(6, extra)
    ))
}

/// Word gives the Fallback copy of a text box change its own consecutive
/// `w:id`: the change lists once (the Choice copy, the one the view shows)
/// and resolves in both copies. A Fallback carrier without a Choice partner
/// stays listed.
#[test]
fn a_text_box_change_lists_once_and_resolves_in_both_copies() {
    assert_eq!(ids(&boxed("")), ["r5"]);
    let accepted = accept_changes(&boxed(""), &filter_ids(&["r5"])).unwrap();
    let xml = part_string(&accepted, "word/document.xml").unwrap();
    assert!(!xml.contains("<w:ins"), "{xml}");
    assert_eq!(xml.matches(">Boxed<").count(), 2, "{xml}");
    let rejected = reject_changes(&boxed(""), &filter_ids(&["r5"])).unwrap();
    let xml = part_string(&rejected, "word/document.xml").unwrap();
    assert!(!xml.contains("Boxed"), "{xml}");

    let extra = format!(r#"<w:ins w:id="7" {A}><w:r><w:t>Extra</w:t></w:r></w:ins>"#);
    assert_eq!(ids(&boxed(&extra)), ["r5", "r7"]);
}

/// Word's own redline of a text box: the Choice copy's deletions are
/// r9-r10 and the Fallback's r11-r12. Each lists once; rejecting r10 also
/// restores its Fallback twin r12, and the deletion around the box stays.
#[test]
fn a_word_redlines_text_box_twins_list_once() {
    let word = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/corpus/neurotic_docx_bench/corpus/word/tracking_without_comments/docx/4640e71ddd__vs__1424386e9b_redline_32d0abcfc9.docx"
    ))
    .unwrap();
    let listed = ids(&word);
    for twin in ["r11", "r12"] {
        assert!(!listed.iter().any(|id| id == twin), "{twin}: {listed:?}");
    }
    for own in ["r8", "r9", "r10"] {
        assert!(listed.iter().any(|id| id == own), "{own}: {listed:?}");
    }
    let rejected = reject_changes(&word, &filter_ids(&["r10"])).unwrap();
    let xml = part_string(&rejected, "word/document.xml").unwrap();
    for gone in [r#"w:id="10""#, r#"w:id="12""#] {
        assert!(!xml.contains(gone), "{gone}");
    }
    for kept in [r#"w:id="8""#, r#"w:id="9""#, r#"w:id="11""#] {
        assert!(xml.contains(kept), "{kept}");
    }
}

fn alternate(choice: &str, fallback: &str) -> String {
    format!(
        r#"<mc:AlternateContent><mc:Choice Requires="wps">{choice}</mc:Choice><mc:Fallback>{fallback}</mc:Fallback></mc:AlternateContent>"#
    )
}

fn boxed_text(inner: &str) -> String {
    format!(r#"<w:txbxContent><w:p>{inner}</w:p></w:txbxContent>"#)
}

/// A Fallback-only change between two twins is skipped, not paired with
/// the next Choice change: each twin still finds its own partner.
#[test]
fn a_fallback_only_change_between_twins_does_not_shift_the_pairs() {
    let ins = |id: u32, text: &str| {
        format!(r#"<w:ins w:id="{id}" {A}><w:r><w:t>{text}</w:t></w:r></w:ins>"#)
    };
    let del = |id: u32, text: &str| {
        format!(r#"<w:del w:id="{id}" {A}><w:r><w:delText>{text}</w:delText></w:r></w:del>"#)
    };
    let body = format!(
        r#"<w:p><w:r>{}</w:r></w:p>"#,
        alternate(
            &boxed_text(&format!("{}{}", ins(5, "One"), del(6, "Two"))),
            &boxed_text(&format!(
                "{}{}{}",
                ins(7, "One"),
                ins(8, "Extra"),
                del(9, "Two")
            )),
        )
    );
    let pkg = docx(&body);
    assert_eq!(ids(&pkg), ["r5", "r6", "r8"]);
    let rejected = reject_changes(&pkg, &filter_ids(&["r6"])).unwrap();
    let xml = part_string(&rejected, "word/document.xml").unwrap();
    assert!(!xml.contains("<w:del "), "both copies of r6 resolve: {xml}");
}

/// A group shape nests one `mc:AlternateContent` in another's Choice: the
/// inner Fallback twin and the outer Fallback copy both join the change.
#[test]
fn nested_alternate_content_copies_join_one_change() {
    let ins = |id: u32| format!(r#"<w:ins w:id="{id}" {A}><w:r><w:t>Grouped</w:t></w:r></w:ins>"#);
    let inner = alternate(&boxed_text(&ins(5)), &boxed_text(&ins(6)));
    let body = format!(
        r#"<w:p><w:r>{}</w:r></w:p>"#,
        alternate(&format!("<w:r>{inner}</w:r>"), &boxed_text(&ins(7)))
    );
    let pkg = docx(&body);
    assert_eq!(ids(&pkg), ["r5"]);
    let accepted = accept_changes(&pkg, &filter_ids(&["r5"])).unwrap();
    let xml = part_string(&accepted, "word/document.xml").unwrap();
    assert!(!xml.contains("<w:ins "), "every copy resolves: {xml}");
    assert_eq!(xml.matches(">Grouped<").count(), 3, "{xml}");
}

// ───────────────────────── #1223: revisions inside comment bodies ──────────

#[path = "common/comment_revisions.rs"]
mod comment_revisions;

/// One body insertion by Ann, and one comment holding an insertion and a
/// deletion by Bob.
fn commented_body() -> Vec<u8> {
    let body = format!(
        r#"<w:p>{}<w:ins w:id="1" {A}><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#,
        comment_revisions::anchor(0, "anchor")
    );
    let comment = comment_revisions::comment(
        0,
        "Alice",
        r#"<w:p w14:paraId="00000001"><w:ins w:id="20" w:author="Bob" w:date="2026-10-18T00:00:00Z"><w:r><w:t xml:space="preserve">in </w:t></w:r></w:ins><w:del w:id="21" w:author="Bob" w:date="2026-10-18T00:00:00Z"><w:r><w:delText xml:space="preserve">out </w:delText></w:r></w:del><w:r><w:t>rest</w:t></w:r></w:p>"#,
    );
    comment_revisions::docx(&body, &[comment])
}

/// `word/comments.xml` as text (panics when the part is gone).
fn comments_xml(pkg: &[u8]) -> String {
    part_string(pkg, "word/comments.xml").unwrap()
}

/// #1223: `jubarte changes` (and the view) still names no revision inside a
/// comment body; accept/reject all settles them anyway.
#[test]
fn comment_body_changes_stay_unlisted() {
    assert_eq!(ids(&commented_body()), ["r1"]);
    let accepted = accept_changes(&commented_body(), &ChangeFilter::default()).unwrap();
    let xml = comments_xml(&accepted);
    assert!(!xml.contains("<w:ins") && !xml.contains("<w:del"), "{xml}");
}

/// #1223: `--id` cannot name a comment-body revision; selecting a body
/// change leaves the comment's revisions tracked.
#[test]
fn an_id_selection_leaves_comment_body_changes() {
    let accepted = accept_changes(&commented_body(), &filter_ids(&["r1"])).unwrap();
    assert_word_valid_package(&accepted);
    let xml = comments_xml(&accepted);
    assert!(
        xml.contains(r#"<w:ins w:id="20""#) && xml.contains(r#"<w:del w:id="21""#),
        "comment revisions left tracked: {xml}"
    );
    // A comment-story id names no listed change.
    let err = accept_changes(&commented_body(), &filter_ids(&["comments.r20"])).unwrap_err();
    assert!(
        matches!(&err, ChangeError::UnknownChange(id) if id == "comments.r20"),
        "{err:?}"
    );
}

/// #1223: an author or kind filter reaches comment bodies — "every change
/// by Bob" includes the ones inside his comments.
#[test]
fn author_and_kind_filters_resolve_comment_body_changes() {
    let bob = ChangeFilter {
        authors: Some(vec!["Bob".into()]),
        ..Default::default()
    };
    let accepted = accept_changes(&commented_body(), &bob).unwrap();
    assert_word_valid_package(&accepted);
    let xml = comments_xml(&accepted);
    assert_eq!(xml.matches("<w:ins ").count(), 0, "{xml}");
    assert_eq!(xml.matches("<w:del ").count(), 0, "{xml}");
    assert!(xml.contains(">in ") && !xml.contains("out"), "{xml}");
    // Ann's body insertion was not selected.
    let doc = part_string(&accepted, "word/document.xml").unwrap();
    assert!(doc.contains(r#"<w:ins w:id="1""#), "{doc}");

    let deletions = ChangeFilter {
        kinds: Some(vec![ChangeKind::Deletion]),
        ..Default::default()
    };
    let rejected = reject_changes(&commented_body(), &deletions).unwrap();
    let xml = comments_xml(&rejected);
    assert!(
        !xml.contains(r#"<w:del w:id="21""#) && xml.contains(">out "),
        "comment deletion restored: {xml}"
    );
    assert!(xml.contains(r#"<w:ins w:id="20""#), "insertion kept: {xml}");
}

/// #1223: the CLI says how many comment-body revisions a selection left.
#[test]
fn cli_id_selection_reports_comment_body_changes_left() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("input.docx"), commented_body()).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args(["accept", "input.docx", "--id", "r1", "-o", "settled.docx"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("2 tracked changes inside comments were left"),
        "{stderr}"
    );
}

/// A table whose second row the revision inserts (`inserted`) or deletes:
/// the row's revision is `w:id="1"`, its two cells' final marks `2` and
/// `4`, their words `3` and `5`, as Word writes a row added or removed
/// whole.
fn revised_row(inserted: bool) -> Vec<u8> {
    let (kind, text) = if inserted {
        ("ins", "t")
    } else {
        ("del", "delText")
    };
    let cell = |mark: u32, words: u32, value: &str| {
        format!(
            r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:pPr><w:rPr><w:{kind} w:id="{mark}" {A}/></w:rPr></w:pPr><w:{kind} w:id="{words}" {A}><w:r><w:{text}>{value}</w:{text}></w:r></w:{kind}></w:p></w:tc>"#
        )
    };
    let plain = |value: &str| {
        format!(
            r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>{value}</w:t></w:r></w:p></w:tc>"#
        )
    };
    docx(&format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/><w:gridCol w:w="2000"/></w:tblGrid><w:tr>{}{}</w:tr><w:tr><w:trPr><w:{kind} w:id="1" {A}/></w:trPr>{}{}</w:tr></w:tbl><w:p/>"#,
        plain("Kept."),
        plain("Also kept."),
        cell(2, 3, "Left."),
        cell(4, 5, "Right.")
    ))
}

/// Resolving only a row's insertion or deletion so the row stays resolves
/// the same revision on its cells' final marks, which Word writes only
/// with the row's: left on them, Word's Accept All or Reject All never
/// returns (#1679, #1669). The cells' words stay tracked.
#[test]
fn resolving_a_row_alone_takes_its_cells_final_marks_along() {
    for (inserted, name, resolve) in [
        (
            true,
            "accept the inserted row",
            accept_elements as fn(&[u8], &ChangeFilter) -> Result<Vec<u8>, ChangeError>,
        ),
        (false, "reject the deleted row", reject_elements),
    ] {
        let source = revised_row(inserted);
        assert!(check_word_valid_package(&source).ok(), "{name}: source");
        let partial = resolve(&source, &filter_ids(&["r1"])).unwrap();
        let report = check_word_valid_package(&partial);
        assert!(
            report.ok(),
            "{name}: {:?}\n{}",
            report.errors,
            part_string(&partial, "word/document.xml").unwrap()
        );
        let mut left = ids(&partial);
        left.sort();
        assert_eq!(left, ["r3", "r5"], "{name}: the words stay tracked");
    }
}

/// The same with a nested table in the row's cell whose own row carries
/// no revision, as Open-Xml-PowerTools writes it: the outer row's revision
/// covers the nested cell's final mark, so resolving the row alone takes
/// that mark along too (#1684, G2 of #1680's round 9).
#[test]
fn resolving_a_row_alone_takes_its_nested_cells_final_marks_along() {
    for (inserted, name, resolve) in [
        (
            true,
            "accept the inserted row",
            accept_elements as fn(&[u8], &ChangeFilter) -> Result<Vec<u8>, ChangeError>,
        ),
        (false, "reject the deleted row", reject_elements),
    ] {
        let (kind, text) = if inserted {
            ("ins", "t")
        } else {
            ("del", "delText")
        };
        let marked = |mark: u32, words: u32, value: &str| {
            format!(
                r#"<w:p><w:pPr><w:rPr><w:{kind} w:id="{mark}" {A}/></w:rPr></w:pPr><w:{kind} w:id="{words}" {A}><w:r><w:{text}>{value}</w:{text}></w:r></w:{kind}></w:p>"#
            )
        };
        let nested = format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="1800" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="1800"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="1800" w:type="dxa"/></w:tcPr>{}</w:tc></w:tr></w:tbl>"#,
            marked(6, 7, "Inner.")
        );
        let source = docx(&format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="2000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Kept.</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:trPr><w:{kind} w:id="1" {A}/></w:trPr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>{nested}{}</w:tc></w:tr></w:tbl><w:p/>"#,
            marked(2, 3, "Outer.")
        ));
        assert!(check_word_valid_package(&source).ok(), "{name}: source");
        let partial = resolve(&source, &filter_ids(&["r1"])).unwrap();
        let report = check_word_valid_package(&partial);
        assert!(
            report.ok(),
            "{name}: {:?}\n{}",
            report.errors,
            part_string(&partial, "word/document.xml").unwrap()
        );
        let mut left = ids(&partial);
        left.sort();
        assert_eq!(left, ["r3", "r7"], "{name}: the words stay tracked");
    }
}

/// A nested row that carries the outer row's revision itself covers its
/// own cells: resolving the outer row alone leaves that row, its cell's
/// final mark and its words tracked (CodeRabbit on #1680).
#[test]
fn a_nested_row_revised_on_its_own_stays_when_its_outer_row_resolves() {
    for (inserted, name, resolve) in [
        (
            true,
            "accept the inserted row",
            accept_elements as fn(&[u8], &ChangeFilter) -> Result<Vec<u8>, ChangeError>,
        ),
        (false, "reject the deleted row", reject_elements),
    ] {
        let (kind, text) = if inserted {
            ("ins", "t")
        } else {
            ("del", "delText")
        };
        let marked = |mark: u32, words: u32, value: &str| {
            format!(
                r#"<w:p><w:pPr><w:rPr><w:{kind} w:id="{mark}" {A}/></w:rPr></w:pPr><w:{kind} w:id="{words}" {A}><w:r><w:{text}>{value}</w:{text}></w:r></w:{kind}></w:p>"#
            )
        };
        let nested = format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="1800" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="1800"/></w:tblGrid><w:tr><w:trPr><w:{kind} w:id="8" {A}/></w:trPr><w:tc><w:tcPr><w:tcW w:w="1800" w:type="dxa"/></w:tcPr>{}</w:tc></w:tr></w:tbl>"#,
            marked(6, 7, "Inner.")
        );
        let source = docx(&format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="2000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Kept.</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:trPr><w:{kind} w:id="1" {A}/></w:trPr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>{nested}{}</w:tc></w:tr></w:tbl><w:p/>"#,
            marked(2, 3, "Outer.")
        ));
        assert!(check_word_valid_package(&source).ok(), "{name}: source");
        let partial = resolve(&source, &filter_ids(&["r1"])).unwrap();
        let report = check_word_valid_package(&partial);
        assert!(
            report.ok(),
            "{name}: {:?}\n{}",
            report.errors,
            part_string(&partial, "word/document.xml").unwrap()
        );
        let mut left = ids(&partial);
        left.sort();
        assert_eq!(
            left,
            ["r3", "r6", "r7", "r8"],
            "{name}: the nested row stays"
        );
    }
}
