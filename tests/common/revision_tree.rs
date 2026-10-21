// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//! Revision identity and containment across all Word story parts.
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Revision {
    kind: String,
    author: String,
    parent: Option<String>,
    text: String,
    date: Option<String>,
}

pub fn revisions(bytes: &[u8], view: &str) -> BTreeMap<(String, String), Revision> {
    fn walk(
        dom: &Dom,
        node: NodeId,
        parent: Option<&str>,
        part: &str,
        view: &str,
        out: &mut BTreeMap<(String, String), Revision>,
    ) {
        let mut parent = parent.map(str::to_string);
        if dom.is_element(node)
            && dom
                .name(node)
                .is_some_and(|name| name.namespace_name() == W::URI)
            && dom.name(node).is_some_and(|name| {
                matches!(name.local_name(), "ins" | "del" | "moveFrom" | "moveTo")
            })
        {
            let id = dom.attribute(node, &W::name("id")).unwrap().to_string();
            let date = dom
                .attribute(node, &W::name("date"))
                .filter(|date| view.contains(*date))
                .map(str::to_string);
            let text = dom
                .descendants(node, None)
                .into_iter()
                .filter(|&n| {
                    dom.is_element(n)
                        && dom.name(n).is_some_and(|name| {
                            name.namespace_name() == W::URI
                                && matches!(name.local_name(), "t" | "delText")
                        })
                })
                .map(|n| dom.value(n))
                .collect::<String>();
            let revision = Revision {
                kind: dom.name(node).unwrap().local_name().to_string(),
                author: dom
                    .attribute(node, &W::name("author"))
                    .unwrap_or("")
                    .to_string(),
                parent: parent.clone(),
                text,
                date,
            };
            assert!(
                out.insert((part.to_string(), id.clone()), revision)
                    .is_none(),
                "duplicate revision {part}:{id}"
            );
            parent = Some(id);
        }
        for child in dom.elements(node, None) {
            walk(dom, child, parent.as_deref(), part, view, out);
        }
    }
    let mut out = BTreeMap::new();
    for (part, xml) in super::read_parts(bytes) {
        if part.starts_with("word/") && part.ends_with(".xml") {
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&String::from_utf8(xml).unwrap());
            walk(&dom, dom.root(doc).unwrap(), None, &part, view, &mut out);
        }
    }
    out
}

/// Compare dates only for ids whose original date the view actually carries.
/// An author's latest date cannot recover every earlier revision's date.
#[track_caller]
pub fn assert_same(original: &[u8], rewritten: &[u8], view: &str) {
    let original = revisions(original, view);
    let mut rewritten = revisions(rewritten, view);
    for (id, revision) in &original {
        if revision.date.is_none()
            && let Some(back) = rewritten.get_mut(id)
        {
            back.date = None;
        }
    }
    assert_eq!(
        original, rewritten,
        "revision kind, author, parent, text and carried dates"
    );
}
