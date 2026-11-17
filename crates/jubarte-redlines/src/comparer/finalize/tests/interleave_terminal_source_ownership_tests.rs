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
// Empty text is an authored blank paragraph with a revision on its
// pilcrow, rather than an empty text node masquerading as a list item.
fn owned(kind: &str, text: &str, level: u32, id: u32, history: bool) -> String {
    let mark = format!("<w:{kind} w:id='{id}' w:author='Source lifetime owner' w:date='{DATE}'/>");
    let old = if history {
        format!(
            "<w:pPrChange w:id='{}' w:author='Earlier format owner' w:date='2025-02-03T04:05:06Z'><w:pPr><w:spacing w:after='60'/></w:pPr></w:pPrChange>",
            id + 100
        )
    } else {
        String::new()
    };
    let body = if text.is_empty() {
        String::new()
    } else {
        let tag = if kind == "del" { "delText" } else { "t" };
        format!(
            "<w:{kind} w:id='{}' w:author='Source content owner' w:date='{DATE}'><w:r><w:rPr><w:b/><w:color w:val='123456'/><w:lang w:val='en-US'/></w:rPr><w:{tag} xml:space='preserve'>{text}</w:{tag}></w:r></w:{kind}>",
            id + 200
        )
    };
    format!(
        "<w:p><w:pPr><w:pStyle w:val='ListParagraph'/><w:numPr><w:ilvl w:val='{level}'/><w:numId w:val='7'/></w:numPr><w:spacing w:after='80'/><w:rPr>{mark}<w:i/></w:rPr>{old}</w:pPr>{body}</w:p>"
    )
}
fn remains_source_owned(blocks: &[String], label: &str) {
    let input = blocks.concat() + SECTION;
    let (mut dom, root) = package(&input);
    let original = semantic(&dom, root);
    let nodes = dom.node_count();
    interleave_list_cluster_after_coalesce(&mut dom, root);
    let actual = semantic(&dom, root);
    if actual != original {
        let first = actual
            .chars()
            .zip(original.chars())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| actual.chars().count().min(original.chars().count()));
        let context = |value: &str| {
            value
                .chars()
                .skip(first.saturating_sub(40))
                .take(160)
                .collect::<String>()
        };
        panic!(
            "{label}: complete source differs at {first}, actual {:?}, expected {:?}",
            context(&actual),
            context(&original)
        );
    }
    assert_eq!(
        dom.node_count(),
        nodes,
        "{label}: declining a peel allocates no replacement source nodes"
    );
    interleave_list_cluster_after_coalesce(&mut dom, root);
    assert_eq!(
        semantic(&dom, root),
        original,
        "{label}: repeated normalization preserves every property, revision owner and paragraph position"
    );
    assert_eq!(dom.node_count(), nodes);
}

#[test]
fn terminal_revision_runs_and_incomplete_interleave_keep_all_source_positions() {
    for history in [false, true] {
        // These legal terminal streams cannot be peeled into a nested
        // cluster. The whole source tree, including section and history,
        // is the independent conservative contract.
        for (label, kinds) in [
            ("inserted terminal run", "IIIIII"),
            ("deleted terminal run after heading", "IDDDDD"),
            ("incomplete interleave without deleted remainder", "IDDIII"),
        ] {
            let blocks: Vec<String> = kinds
                .chars()
                .enumerate()
                .map(|(index, kind)| {
                    owned(
                        if kind == 'I' { "ins" } else { "del" },
                        &format!("Item {index}"),
                        u32::from(index == 2),
                        index as u32 + 1,
                        history,
                    )
                })
                .collect();
            remains_source_owned(&blocks, label);
        }
    }
}

#[test]
fn an_authored_inserted_blank_is_not_a_short_list_label() {
    for history in [false, true] {
        let blocks = vec![
            owned("ins", "Revised heading", 0, 1, history),
            owned("ins", "a", 0, 2, history),
            owned("ins", "", 0, 3, history),
            owned("del", "Original first item", 0, 4, history),
            owned("del", "Original nested item", 1, 5, history),
            owned("del", "Original final item", 0, 6, history),
        ];
        remains_source_owned(
            &blocks,
            "authored inserted blank owns a pilcrow, not a word label",
        );
    }
}

#[test]
fn a_deleted_blank_numbered_paragraph_does_not_claim_a_nested_content_cluster() {
    for history in [false, true] {
        let blocks = vec![
            owned("ins", "Revised heading", 0, 1, history),
            owned("ins", "a", 0, 2, history),
            owned("ins", "b", 0, 3, history),
            owned("del", "Original first item", 0, 4, history),
            owned("del", "", 1, 5, history),
            owned("del", "Original final item", 0, 6, history),
        ];
        remains_source_owned(
            &blocks,
            "empty numbered source preserves its own geometry and lifetime without moving other owners",
        );
    }
}
#[test]
fn split_inserted_title_fragments_do_not_take_another_authors_pending_revision() {
    // The short shared-word title is otherwise an eligible Word mesh.
    // Its second inserted fragment belongs to a different pending edit;
    // changing that owner is forbidden even when all formats agree.
    for source_author in ["Earlier title editor", "Imported pending editor"] {
        for different_fragment in [0usize, 1] {
            let first_author = if different_fragment == 0 {
                source_author
            } else {
                "Comparer editor"
            };
            let second_author = if different_fragment == 1 {
                source_author
            } else {
                "Comparer editor"
            };
            let fragment = |kind: &str, text: &str, id: u32, author: &str| {
                let tag = if kind == "del" { "delText" } else { "t" };
                format!(
                    "<w:{kind} w:id='{id}' w:author='{author}' w:date='{DATE}'><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:{tag} xml:space='preserve'>{text}</w:{tag}></w:r></w:{kind}>"
                )
            };
            let input = format!(
                "<w:p><w:pPr><w:pStyle w:val='Title'/><w:spacing w:after='120'/><w:jc w:val='center'/><w:pPrChange w:id='70' w:author='Original layout editor' w:date='2025-02-03T04:05:06Z'><w:pPr><w:spacing w:after='80'/></w:pPr></w:pPrChange></w:pPr>{}{}{}</w:p>{SECTION}",
                fragment("ins", "New ", 41, first_author),
                fragment("ins", "document", 42, second_author),
                fragment("del", "Old document", 43, "Comparer editor")
            );
            let (mut dom, root) = package(&input);
            let source = semantic(&dom, root);
            let nodes = dom.node_count();
            free_mesh_shared_title_token_in_mix(&mut dom, root);
            assert_eq!(
                semantic(&dom, root),
                source,
                "each title revision must retain its original pending owner and exact source format"
            );
            assert_eq!(dom.node_count(), nodes);
            free_mesh_shared_title_token_in_mix(&mut dom, root);
            assert_eq!(semantic(&dom, root), source);
            assert_eq!(dom.node_count(), nodes);
        }
    }
}
