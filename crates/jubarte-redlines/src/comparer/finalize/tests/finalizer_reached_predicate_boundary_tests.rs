// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
const DATE: &str = "2026-01-02T03:04:05Z";
const PROPS: &str =
    "<w:pStyle w:val='BodyText'/><w:spacing w:before='120' w:after='240'/><w:ind w:left='360'/>";
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
fn para(kind: &str, text: &str, props: &str) -> String {
    format!(
        "<w:p><w:pPr>{props}</w:pPr>{}</w:p>",
        run(kind, text, "Boundary editor", DATE)
    )
}
fn marked(
    kind: &str,
    text: &str,
    props: &str,
    mark_author: &str,
    mark_date: &str,
    body_author: &str,
    body_date: &str,
) -> String {
    format!(
        "<w:p><w:pPr>{props}<w:rPr><w:{kind} w:id='18' w:author='{mark_author}' w:date='{mark_date}'/></w:rPr></w:pPr>{}</w:p>",
        run(kind, text, body_author, body_date)
    )
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
        let context = |text: &str| {
            text.chars()
                .skip(first.saturating_sub(64))
                .take(240)
                .collect::<String>()
        };
        panic!(
            "{label}: complete source tree differs at character {first}; actual length {}, expected length {}; actual context {:?}; expected context {:?}",
            actual.len(),
            expected.len(),
            context(&actual),
            context(&expected)
        );
    }
}
// Midstory unrelated replacements keep their complete source carriers.
// A live source bridge prevents D→I reordering from hiding the later I.
// The earlier story-end/relatedness guard already owns these decisions:
// changing the long-intro M145 threshold cannot bypass that boundary.
// Related text folds with the documented bare mixed mark instead.
#[test]
fn a_later_revised_block_protects_the_long_intro_checklist_boundary() {
    for (label, text, original, later_inserted, count, keep) in [
        (
            "documented baseline",
            "Revised contractual responsibilities owners",
            "HR Onboarding Checklist",
            true,
            2,
            true,
        ),
        (
            "intro one char below",
            "Revised contractual responsibilities owner",
            "HR Onboarding Checklist",
            true,
            2,
            true,
        ),
        (
            "no later revised block",
            "Revised contractual responsibilities owners",
            "HR Onboarding Checklist",
            false,
            2,
            true,
        ),
        (
            "intro count above",
            "Revised contractual responsibilities owners",
            "HR Onboarding Checklist",
            true,
            4,
            true,
        ),
        (
            "original title above five tokens",
            "Revised contractual responsibilities owners",
            "Original Human Resources Employee Onboarding Policy Checklist",
            true,
            2,
            true,
        ),
        (
            "original title related",
            "Revised contractual responsibilities owners",
            "HR revised checklist",
            true,
            2,
            false,
        ),
        (
            "short checklist cell remains independent",
            "Revised contractual responsibilities owners",
            "Checklist",
            true,
            2,
            true,
        ),
    ] {
        if label == "documented baseline" {
            assert_eq!(text.chars().filter(|c| c.is_alphanumeric()).count(), 40);
        }
        if label == "intro one char below" {
            assert_eq!(text.chars().filter(|c| c.is_alphanumeric()).count(), 39);
        }
        let lead = (0..count - 1)
            .map(|n| {
                marked(
                    "ins",
                    &format!("Revised introductory paragraph {n}"),
                    PROPS,
                    "Boundary editor",
                    DATE,
                    "Boundary editor",
                    DATE,
                )
            })
            .collect::<String>();
        let intro = marked(
            "ins",
            text,
            PROPS,
            "Boundary editor",
            DATE,
            "Boundary editor",
            DATE,
        );
        let first = marked(
            "del",
            original,
            PROPS,
            "Boundary editor",
            DATE,
            "Boundary editor",
            DATE,
        );
        let rest = marked(
            "del",
            "Original independent responsibilities remain fully attributed",
            PROPS,
            "Boundary editor",
            DATE,
            "Boundary editor",
            DATE,
        );
        let bridge = para("live", "Independently authored live source bridge", PROPS);
        let after = if later_inserted {
            marked(
                "ins",
                "Revised later independently authored continuation",
                PROPS,
                "Boundary editor",
                DATE,
                "Boundary editor",
                DATE,
            )
        } else {
            para(
                "live",
                "Independently authored original continuation",
                PROPS,
            )
        };
        let input = lead.clone() + &intro + &first + &rest + &bridge + &after + TABLE;
        let (mut dom, root, body) = package(&input);
        merge_replaced_in_container(&mut dom, body, "Boundary editor", true);
        let expected = if keep {
            input
        } else {
            let merged = format!(
                "<w:p><w:pPr>{PROPS}</w:pPr>{}{}</w:p>",
                run("ins", text, "Boundary editor", DATE),
                run("del", original, "Boundary editor", DATE)
            );
            lead + &merged + &rest + &bridge + &after + TABLE
        };
        assert_body(&dom, body, &expected, label);
        let once = semantic(&dom, root);
        merge_replaced_in_container(&mut dom, body, "Boundary editor", true);
        assert_eq!(semantic(&dom, root), once, "repeat {label}");
    }
}
// The original bookmark prose deliberately includes the inserted label.
// Its relatedness defeats the earlier M325 gate, so M365's actual short
// label and prose/heading boundary is reached rather than masked.
#[test]
fn a_short_residual_label_does_not_steal_long_bookmark_or_heading_properties() {
    for (label, inserted, original, heading, keep) in [
        (
            "eight-token bookmark prose",
            "a",
            "This document demonstrates a source bookmark paragraph continuation",
            false,
            true,
        ),
        (
            "seven-token prose below",
            "a",
            "This document demonstrates a source bookmark paragraph",
            false,
            false,
        ),
        (
            "seven-token heading",
            "a",
            "This document demonstrates a source bookmark paragraph",
            true,
            true,
        ),
        (
            "label above two chars",
            "abc",
            "This document demonstrates abc source bookmark paragraph continuation",
            false,
            false,
        ),
        ("heading length below ten", "a", "Heading", true, false),
        ("short heading source", "a", "Generalities", true, true),
    ] {
        let first = marked(
            "ins",
            "ONE",
            PROPS,
            "Boundary editor",
            DATE,
            "Boundary editor",
            DATE,
        );
        let second = marked(
            "ins",
            "TWO",
            PROPS,
            "Boundary editor",
            DATE,
            "Boundary editor",
            DATE,
        );
        let last = marked(
            "ins",
            inserted,
            PROPS,
            "Boundary editor",
            DATE,
            "Boundary editor",
            DATE,
        );
        let old_props = if heading {
            "<w:pStyle w:val='Heading1'/><w:spacing w:after='240'/>"
        } else {
            PROPS
        };
        let old = marked(
            "del",
            original,
            old_props,
            "Boundary editor",
            DATE,
            "Boundary editor",
            DATE,
        );
        let tail = marked(
            "del",
            "Original independently owned concluding paragraph",
            PROPS,
            "Boundary editor",
            DATE,
            "Boundary editor",
            DATE,
        );
        let section = "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>";
        let input = first.clone() + &second + &last + &old + &tail + section;
        let (mut dom, root, body) = package(&input);
        merge_replaced_in_container(&mut dom, body, "Boundary editor", true);
        let expected = if keep {
            input
        } else {
            let properties = if heading {
                old[old.find("<w:pPr>").unwrap()..old.find("</w:pPr>").unwrap() + "</w:pPr>".len()]
                    .to_string()
            } else {
                format!("<w:pPr>{PROPS}</w:pPr>")
            };
            first
                + &second
                + &format!(
                    "<w:p>{properties}{}{}</w:p>",
                    run("ins", inserted, "Boundary editor", DATE),
                    run("del", original, "Boundary editor", DATE)
                )
                + &tail
                + section
        };
        assert_body(&dom, body, &expected, label);
        let once = semantic(&dom, root);
        merge_replaced_in_container(&mut dom, body, "Boundary editor", true);
        assert_eq!(semantic(&dom, root), once, "repeat {label}");
    }
}
