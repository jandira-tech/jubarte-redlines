// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
const DATE: &str = "2026-01-02T03:04:05Z";
const FORMAT: &str = "<w:rPr><w:b/><w:color w:val='123456'/><w:lang w:val='en-US'/></w:rPr>";
const TABLE: &str = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Independent table payload</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";

fn package(body: &str) -> (Dom, NodeId, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w='{}' xmlns:pt='{}'><w:body>{body}</w:body></w:document>",
        W::URI,
        PT::URI
    ));
    let root = dom.root(doc).expect("document");
    let body = dom.element(root, &W::body()).expect("body");
    (dom, root, body)
}
fn semantic(dom: &Dom, node: NodeId) -> String {
    if !dom.is_element(node) {
        return format!("text:{:?}", dom.text_value(node));
    }
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .map(|(name, value)| {
            (
                name.namespace_name().to_owned(),
                name.local_name().to_owned(),
                value,
            )
        })
        .collect::<Vec<_>>();
    attrs.sort();
    format!(
        "{:?}{attrs:?}[{}]",
        dom.name(node),
        dom.nodes(node)
            .into_iter()
            .map(|child| semantic(dom, child))
            .collect::<String>()
    )
}
fn run(kind: &str, text: &str, author: &str, date: &str) -> String {
    let tag = if kind == "del" { "delText" } else { "t" };
    let run = format!("<w:r>{FORMAT}<w:{tag} xml:space='preserve'>{text}</w:{tag}></w:r>");
    if kind == "live" {
        run
    } else {
        format!("<w:{kind} w:id='17' w:author='{author}' w:date='{date}'>{run}</w:{kind}>")
    }
}
fn assert_body(dom: &Dom, body: NodeId, expected: &str, label: &str) {
    let (want, _, wb) = package(expected);
    let actual = semantic(dom, body);
    let expected = semantic(&want, wb);
    if actual != expected {
        let first = actual
            .chars()
            .zip(expected.chars())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| actual.chars().count().min(expected.chars().count()));
        let context = |x: &str| {
            x.chars()
                .skip(first.saturating_sub(48))
                .take(200)
                .collect::<String>()
        };
        panic!(
            "{label}: full source tree differs at {first}; actual {:?}, expected {:?}; lengths {}/{}",
            context(&actual),
            context(&expected),
            actual.len(),
            expected.len()
        );
    }
}
fn mark(kind: &str, id: u32, author: &str, date: Option<&str>) -> String {
    let date = date.map(|v| format!(" w:date='{v}'")).unwrap_or_default();
    format!("<w:{kind} w:id='{id}' w:author='{author}'{date}/>")
}
fn history() -> String {
    format!(
        "<w:pPrChange w:id='29' w:author='Prior layout editor' w:date='{DATE}'><w:pPr><w:spacing w:after='50'/><w:jc w:val='center'/></w:pPr></w:pPrChange>"
    )
}
fn fresh_history() -> String {
    format!(
        "<w:pPrChange w:id='40' w:author='Boundary editor' w:date='{DATE}'><w:pPr/></w:pPrChange>"
    )
}
fn settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Boundary editor".to_string(),
        date_time_for_revisions: DATE.to_string(),
        ..WmlComparerSettings::default()
    }
}

// Word M221 moves a MIX's spacing to the authored following blank.
// An already-authored pilcrow lifetime is authoritative: the metadata
// gate must not synthesize a different lifetime or rewrite its owner.
#[test]
fn moving_mixed_spacing_keeps_existing_inserted_and_deleted_mark_owners() {
    let spacing = "<w:spacing w:before='400' w:after='120'/>";
    let format = "<w:b/><w:color w:val='AABBCC'/>";
    for existing in ["none", "ins", "del"] {
        for prior_history in [false, true] {
            for dated in [false, true] {
                let old_mark = if existing == "none" {
                    String::new()
                } else {
                    mark(
                        existing,
                        18,
                        "Original pilcrow editor",
                        dated.then_some("2025-04-05T06:07:08Z"),
                    )
                };
                let mix_body = run("ins", "Revised heading payload", "Boundary editor", DATE)
                    + &run("del", "Original heading payload", "Boundary editor", DATE);
                let mixed = format!(
                    "<w:p><w:pPr><w:keepNext/>{spacing}<w:rPr>{old_mark}{format}</w:rPr></w:pPr>{mix_body}</w:p>"
                );
                let original_history = if prior_history {
                    history()
                } else {
                    String::new()
                };
                let empty =
                    format!("<w:p><w:pPr><w:rPr><w:i/></w:rPr>{original_history}</w:pPr></w:p>");
                let (mut dom, root, body) = package(&(TABLE.to_string() + &mixed + &empty));
                let mut id = 40;
                mixed_spacing_to_following_empty(&mut dom, root, &settings(), &mut id);
                let added_history = if prior_history {
                    original_history
                } else {
                    fresh_history()
                };
                let expected_mark = if existing == "none" {
                    mark(
                        "del",
                        if prior_history { 40 } else { 41 },
                        "Boundary editor",
                        Some(DATE),
                    )
                } else {
                    old_mark
                };
                let expected_format = if existing == "none" {
                    format!("{format}{expected_mark}")
                } else {
                    format!("{expected_mark}{format}")
                };
                let expected = format!(
                    "{TABLE}<w:p><w:pPr><w:keepNext/><w:rPr>{expected_format}</w:rPr></w:pPr>{mix_body}</w:p><w:p><w:pPr>{spacing}<w:rPr><w:i/></w:rPr>{added_history}</w:pPr></w:p>"
                );
                assert_body(
                    &dom,
                    body,
                    &expected,
                    &format!("M221/{existing}/history={prior_history}/date={dated}"),
                );
                assert_eq!(
                    id,
                    40 + u32::from(!prior_history) + u32::from(existing == "none")
                );
                let once = semantic(&dom, root);
                let next = id;
                mixed_spacing_to_following_empty(&mut dom, root, &settings(), &mut id);
                assert_eq!(semantic(&dom, root), once);
                assert_eq!(id, next);
            }
        }
    }
}

// M221's other observed Word shape parks heading spacing on the final
// deleted residual. Preserve existing mixed-mark and old-history owners.
#[test]
fn parking_heading_spacing_keeps_existing_lifetime_and_history_provenance() {
    let spacing = "<w:spacing w:before='400' w:after='120'/>";
    let format = "<w:b/><w:color w:val='AABBCC'/>";
    for existing in ["none", "ins", "del"] {
        for prior_history in [false, true] {
            let old_mark = if existing == "none" {
                String::new()
            } else {
                mark(
                    existing,
                    18,
                    "Original pilcrow editor",
                    Some("2025-04-05T06:07:08Z"),
                )
            };
            let mixed_body = run("ins", "Revised heading payload", "Boundary editor", DATE)
                + &run("del", "Original heading payload", "Boundary editor", DATE);
            let mixed = format!(
                "<w:p><w:pPr><w:keepNext/>{spacing}<w:rPr>{old_mark}{format}</w:rPr></w:pPr>{mixed_body}</w:p>"
            );
            let first = format!(
                "<w:p><w:pPr><w:keepLines/></w:pPr>{}</w:p>",
                run("del", "Original first residual", "Boundary editor", DATE)
            );
            let original_history = if prior_history {
                history()
            } else {
                String::new()
            };
            let last_mark = mark(
                "del",
                20,
                "Original residual editor",
                Some("2025-04-05T06:07:08Z"),
            );
            let last_body = run("del", "Original final residual", "Boundary editor", DATE);
            let last = format!(
                "<w:p><w:pPr><w:rPr>{last_mark}</w:rPr>{original_history}</w:pPr>{last_body}</w:p>"
            );
            let (mut dom, root, body) = package(&(TABLE.to_string() + &mixed + &first + &last));
            let mut id = 40;
            park_mixed_spacing_onto_trailing_pure_del(&mut dom, root, &settings(), &mut id);
            let expected_mark = if existing == "none" {
                mark(
                    "del",
                    if prior_history { 40 } else { 41 },
                    "Boundary editor",
                    Some(DATE),
                )
            } else {
                old_mark
            };
            let expected_history = if prior_history {
                original_history
            } else {
                fresh_history()
            };
            let expected_format = if existing == "none" {
                format!("{format}{expected_mark}")
            } else {
                format!("{expected_mark}{format}")
            };
            let expected = format!(
                "{TABLE}<w:p><w:pPr><w:keepNext/><w:rPr>{expected_format}</w:rPr></w:pPr>{mixed_body}</w:p>{first}<w:p><w:pPr>{spacing}<w:rPr>{last_mark}</w:rPr>{expected_history}</w:pPr>{last_body}</w:p>"
            );
            assert_body(
                &dom,
                body,
                &expected,
                &format!("M221 residual/{existing}/history={prior_history}"),
            );
            assert_eq!(
                id,
                40 + u32::from(!prior_history) + u32::from(existing == "none")
            );
            let once = semantic(&dom, root);
            let next = id;
            park_mixed_spacing_onto_trailing_pure_del(&mut dom, root, &settings(), &mut id);
            assert_eq!(semantic(&dom, root), once);
            assert_eq!(id, next);
        }
    }
}

#[test]
fn alignment_donor_search_crosses_default_property_paragraphs_without_claiming_their_formats() {
    for default_count in [1, 2] {
        for table_before_defaults in [false, true] {
            let donor = format!(
                "<w:p><w:pPr><w:spacing w:after='80'/><w:jc w:val='center'/></w:pPr>{}</w:p>",
                run(
                    "live",
                    "Independent alignment donor",
                    "Boundary editor",
                    DATE
                )
            );
            let defaults = (0..default_count)
                .map(|n| {
                    format!(
                        "<w:p>{}</w:p>",
                        run(
                            "live",
                            &format!("Independent default source paragraph {n}"),
                            "Boundary editor",
                            DATE
                        )
                    )
                })
                .collect::<String>();
            let table = if table_before_defaults { TABLE } else { "" };
            let old = history();
            let mark = mark(
                "del",
                20,
                "Original residual editor",
                Some("2025-04-05T06:07:08Z"),
            );
            let body_text = run(
                "del",
                "Original line spacing residual",
                "Boundary editor",
                DATE,
            );
            let last = format!("<w:p><w:pPr><w:rPr>{mark}</w:rPr>{old}</w:pPr>{body_text}</w:p>");
            let (mut dom, root, body) = package(&(donor.clone() + table + &defaults + &last));
            last_pure_del_inherit_prev_jc(&mut dom, root);
            let expected = donor
                + table
                + &defaults
                + &format!(
                    "<w:p><w:pPr><w:rPr>{mark}</w:rPr><w:jc w:val='center'/>{old}</w:pPr>{body_text}</w:p>"
                );
            assert_body(
                &dom,
                body,
                &expected,
                &format!("default donors {default_count}/table={table_before_defaults}"),
            );
            let once = semantic(&dom, root);
            last_pure_del_inherit_prev_jc(&mut dom, root);
            assert_eq!(semantic(&dom, root), once);
        }
    }
}
