// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
const DATE: &str = "2026-01-02T03:04:05Z";
const SECTION: &str = "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:top='720' w:right='900' w:bottom='720' w:left='900'/></w:sectPr>";

fn package(source: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w='{}'><w:body>{source}</w:body></w:document>",
        W::URI
    ));
    let root = dom.root(doc).expect("document");
    (dom, root)
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

const OLD_MARK: &str =
    "<w:del w:id='13' w:author='Original pilcrow owner' w:date='2025-02-03T04:05:06Z'/>";
const NEW_MARK: &str =
    "<w:ins w:id='14' w:author='Revised pilcrow owner' w:date='2025-03-04T05:06:07Z'/>";
const REVISED_FORMAT: &str = "<w:b/><w:color w:val='123456'/>";
const ORIGINAL_FORMAT: &str = "<w:i/><w:color w:val='654321'/>";
const INDEPENDENT: &str = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Independent table payload</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";

fn revision(kind: &str, text: &str, id: u32) -> String {
    let tag = if kind == "del" { "delText" } else { "t" };
    format!(
        "<w:{kind} w:id='{id}' w:author='Body revision owner' w:date='{DATE}'><w:r><w:rPr><w:lang w:val='en-US'/></w:rPr><w:{tag} xml:space='preserve'>{text}</w:{tag}></w:r></w:{kind}>"
    )
}
fn check_complete(dom: &Dom, root: NodeId, expected: &str, label: &str) {
    let (want, wr) = package(expected);
    let actual = semantic(dom, root);
    let oracle = semantic(&want, wr);
    if actual != oracle {
        let first = actual
            .chars()
            .zip(oracle.chars())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| actual.chars().count().min(oracle.chars().count()));
        let context = |v: &str| {
            v.chars()
                .skip(first.saturating_sub(40))
                .take(160)
                .collect::<String>()
        };
        panic!(
            "{label}: entire source carrier differs at {first}: actual {:?}, expected {:?}",
            context(&actual),
            context(&oracle)
        );
    }
}

// Word M86/M88 folds actual inserted spaces into the residual deletion.
// This phase owns the selected pilcrow shell and ordered bodies; full
// pPr history restoration occurs in the surrounding producer pipeline.
// Every legal shell below changes one presence/lifetime operand after
// the whitespace/content/structural gates have already been fulfilled.
#[test]
fn whitespace_fold_preserves_selected_source_mark_shells_and_every_body_owner() {
    let revised = [
        ("absent", String::new()),
        ("empty", "<w:pPr/>".to_string()),
        (
            "format",
            format!("<w:pPr><w:rPr>{REVISED_FORMAT}</w:rPr></w:pPr>"),
        ),
        ("mark", format!("<w:pPr><w:rPr>{NEW_MARK}</w:rPr></w:pPr>")),
        (
            "mark-format",
            format!("<w:pPr><w:rPr>{NEW_MARK}{REVISED_FORMAT}</w:rPr></w:pPr>"),
        ),
    ];
    let original = [
        ("absent", String::new()),
        ("empty", "<w:pPr/>".to_string()),
        (
            "format",
            format!("<w:pPr><w:rPr>{ORIGINAL_FORMAT}</w:rPr></w:pPr>"),
        ),
        (
            "mark-format",
            format!("<w:pPr><w:rPr>{OLD_MARK}{ORIGINAL_FORMAT}</w:rPr></w:pPr>"),
        ),
    ];
    for (revised_kind, revised_props) in &revised {
        for (original_kind, original_props) in &original {
            for whitespace in [" ", "  "] {
                let new_body = revision("ins", whitespace, 11);
                let old_body =
                    revision("del", "Original residual contains five distinct words", 12);
                let input = format!(
                    "<w:p>{revised_props}{new_body}</w:p><w:p>{original_props}{old_body}</w:p>{INDEPENDENT}{SECTION}"
                );
                let (mut dom, root) = package(&input);
                fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
                // Literal source-shell policy: a real original mark is
                // donated; its format accompanies it only when no revised
                // rPr exists. Existing revised format remains authoritative.
                let selected = match (*revised_kind, *original_kind) {
                    ("absent", "mark-format") | ("empty", "mark-format") => {
                        format!("<w:pPr><w:rPr>{OLD_MARK}{ORIGINAL_FORMAT}</w:rPr></w:pPr>")
                    }
                    ("absent", _) => String::new(),
                    ("empty", _) | ("mark", "absent" | "empty" | "format") => {
                        "<w:pPr/>".to_string()
                    }
                    ("format" | "mark-format", "mark-format") => {
                        format!("<w:pPr><w:rPr>{REVISED_FORMAT}{OLD_MARK}</w:rPr></w:pPr>")
                    }
                    ("mark", "mark-format") => {
                        format!("<w:pPr><w:rPr>{OLD_MARK}</w:rPr></w:pPr>")
                    }
                    ("format" | "mark-format", _) => {
                        format!("<w:pPr><w:rPr>{REVISED_FORMAT}</w:rPr></w:pPr>")
                    }
                    _ => unreachable!("enumerated literal source fixtures"),
                };
                let expected =
                    format!("<w:p>{selected}{new_body}{old_body}</w:p>{INDEPENDENT}{SECTION}");
                let label = format!(
                    "M88 source shell revised={revised_kind}/original={original_kind}/spaces={}",
                    whitespace.len()
                );
                check_complete(&dom, root, &expected, &label);
                let once = semantic(&dom, root);
                let nodes = dom.node_count();
                fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
                assert_eq!(semantic(&dom, root), once, "{label}: repeat");
                assert_eq!(dom.node_count(), nodes);
            }
        }
    }
}
