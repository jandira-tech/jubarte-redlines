// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Ring 1 — Rust-native Word-validity package invariants (plan D1).
//!
//! `assert_word_valid_package` fails a test when a produced package would make
//! Word offer repair (dangling rels, duplicate revision ids, orphan comment
//! anchors, …). Intentional broken probes live in `tests/m_validity_ring1.rs`.

use jubarte::namespaces::{MC, W, W14};
use jubarte::opc::PartFs;
use jubarte::xmllinq::{Dom, NodeId};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::{HashMap, HashSet};

/// Failures collected by the Ring-1 checks.
#[derive(Debug, Default)]
pub struct ValidityReport {
    pub errors: Vec<String>,
}

impl ValidityReport {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }

    fn fail(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }
}

/// Assert every Ring-1 invariant; panics with the full error list on failure.
pub fn assert_word_valid_package(bytes: &[u8]) {
    let report = check_word_valid_package(bytes);
    assert!(
        report.ok(),
        "Ring-1 Word-validity failed:\n  - {}",
        report.errors.join("\n  - ")
    );
}

/// Run all Ring-1 checks without panicking (for probe tests).
pub fn check_word_valid_package(bytes: &[u8]) -> ValidityReport {
    let mut report = ValidityReport::default();
    let Ok(pkg) = PartFs::open(bytes) else {
        report.fail("package is not a readable OPC zip");
        return report;
    };
    check_content_types_and_xml(&pkg, &mut report);
    check_relationship_integrity(&pkg, &mut report);
    check_revision_and_drawing_ids(&pkg, &mut report);
    check_para_text_id_bounds(&pkg, &mut report);
    check_del_text_under_del(&pkg, &mut report);
    check_comment_graph(&pkg, &mut report);
    report
}

fn check_content_types_and_xml(pkg: &PartFs, report: &mut ValidityReport) {
    for name in pkg.parts() {
        // Every part should have a content type (default or override).
        if pkg.content_type_for(&name).is_none() {
            // package rels and content types themselves are ok without override
            if name != "[Content_Types].xml" {
                report.fail(format!("part '{name}' has no content type"));
            }
        }
        // XML-ish parts must parse.
        if name.ends_with(".xml") || name.ends_with(".rels") {
            let Some(xml) = pkg.part_string(&name) else {
                report.fail(format!("part '{name}' unreadable as string"));
                continue;
            };
            let mut reader = Reader::from_str(&xml);
            reader.config_mut().trim_text(false);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Eof) => break,
                    Ok(_) => {}
                    Err(e) => {
                        report.fail(format!("part '{name}' is not well-formed XML: {e}"));
                        break;
                    }
                }
                buf.clear();
            }
        }
    }
}

/// Every `r:id` / `r:embed` / `r:link` in a part resolves in that part's `.rels`;
/// no duplicate rIds; no dangling internal targets.
fn check_relationship_integrity(pkg: &PartFs, report: &mut ValidityReport) {
    for name in pkg.parts() {
        if !name.ends_with(".xml") || name.ends_with(".rels") {
            continue;
        }
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        let rels = pkg.read_rels_for(&name);
        let mut ids: HashSet<String> = HashSet::new();
        let mut targets: HashMap<String, (String, bool)> = HashMap::new();
        if let Some(r) = rels {
            let mut seen_ids = HashSet::new();
            for item in &r.items {
                if !seen_ids.insert(item.id.clone()) {
                    report.fail(format!(
                        "duplicate rId '{}' in relationships of '{name}'",
                        item.id
                    ));
                }
                ids.insert(item.id.clone());
                let external = item.target_mode.as_deref() == Some("External");
                targets.insert(item.id.clone(), (item.target.clone(), external));
            }
        }
        // Scan for r:id / r:embed / r:link attributes (namespace-agnostic local).
        for attr in ["r:id=\"", " r:id=\"", "r:embed=\"", "r:link=\""] {
            let mut rest = xml.as_str();
            while let Some(i) = rest.find(attr) {
                let after = &rest[i + attr.len()..];
                if let Some(end) = after.find('"') {
                    let rid = &after[..end];
                    if rid.is_empty() {
                        rest = &after[end + 1..];
                        continue;
                    }
                    if !ids.contains(rid) {
                        report.fail(format!(
                            "dangling relationship id '{rid}' referenced from '{name}'"
                        ));
                    } else if let Some((target, external)) = targets.get(rid)
                        && !external
                    {
                        let resolved = pkg.resolve_rel_target(&name, target);
                        if pkg.part_bytes(&resolved).is_none()
                            && pkg.part_bytes(target.trim_start_matches('/')).is_none()
                        {
                            // External-looking absolute targets without External mode
                            // are still flagged only when the target is clearly a package
                            // path that is missing. Skip http(s) and mailto.
                            let t = target.as_str();
                            if !t.starts_with("http://")
                                && !t.starts_with("https://")
                                && !t.starts_with("mailto:")
                            {
                                report.fail(format!(
                                    "relationship '{rid}' on '{name}' targets missing part '{target}' (resolved '{resolved}')"
                                ));
                            }
                        }
                    }
                    rest = &after[end + 1..];
                } else {
                    break;
                }
            }
        }
    }
}

fn check_revision_and_drawing_ids(pkg: &PartFs, report: &mut ValidityReport) {
    for name in pkg.parts() {
        if !name.ends_with(".xml") {
            continue;
        }
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else {
            continue;
        };
        let mut rev_ids: HashSet<String> = HashSet::new();
        let mut docpr_ids: HashSet<String> = HashSet::new();
        collect_ids(&dom, root, &mut rev_ids, &mut docpr_ids, report, &name);
    }
}

fn collect_ids(
    dom: &Dom,
    root: NodeId,
    rev_ids: &mut HashSet<String>,
    docpr_ids: &mut HashSet<String>,
    report: &mut ValidityReport,
    part: &str,
) {
    let rev_locals = [
        "ins",
        "del",
        "moveFrom",
        "moveTo",
        "moveFromRangeStart",
        "moveToRangeStart",
        "comment",
        "commentRangeStart",
        "commentRangeEnd",
        "commentReference",
    ];
    for e in dom.descendants(root, None) {
        let Some(name) = dom.name(e) else {
            continue;
        };
        let local = name.local_name();
        if rev_locals.contains(&local)
            && let Some(id) = dom.attribute(e, &W::name("id"))
        {
            // comment* share id space with each other; ins/del share another.
            // Ring-1: uniqueness of (local_kind_group, id) — use full local for strictness
            // on the same element type within the part.
            let key = format!("{local}:{id}");
            // Only enforce for ins/del/move* (comment ids are intentionally shared
            // across start/end/ref/comment entry).
            if matches!(
                local,
                "ins" | "del" | "moveFrom" | "moveTo" | "moveFromRangeStart" | "moveToRangeStart"
            ) && !rev_ids.insert(format!("rev:{id}"))
            {
                report.fail(format!(
                    "duplicate w:id '{id}' on revision markup in '{part}' ({key})"
                ));
            }
        }
        if local == "docPr" {
            let wp_id = jubarte::xmllinq::XName::get(
                "id",
                "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing",
            );
            let id = dom.attribute(e, &wp_id).map(str::to_string).or_else(|| {
                dom.attributes(e)
                    .into_iter()
                    .find(|(n, _)| n.local_name() == "id")
                    .map(|(_, v)| v)
            });
            if let Some(id) = id
                && !docpr_ids.insert(id.clone())
            {
                report.fail(format!("duplicate wp:docPr id '{id}' in '{part}'"));
            }
        }
    }
}

fn check_para_text_id_bounds(pkg: &PartFs, report: &mut ValidityReport) {
    let w14 = "http://schemas.microsoft.com/office/word/2010/wordml";
    for name in pkg.parts() {
        if !name.ends_with(".xml") {
            continue;
        }
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        for attr in ["w14:paraId=\"", "w14:textId=\""] {
            let mut rest = xml.as_str();
            while let Some(i) = rest.find(attr) {
                let after = &rest[i + attr.len()..];
                if let Some(end) = after.find('"') {
                    let val = &after[..end];
                    if let Ok(n) = u32::from_str_radix(val, 16)
                        && n >= 0x8000_0000
                    {
                        report.fail(format!(
                            "{attr} value '{val}' >= 0x80000000 in '{name}' (id-paraid-overflow)"
                        ));
                    }
                    rest = &after[end + 1..];
                } else {
                    break;
                }
            }
        }
        let _ = w14;
    }
}

/// `w:del` must carry `w:delText` (never `w:t`).
/// `w:moveFrom` must carry `w:t` (never `w:delText`) — Word-required contract
/// settled by Ring-3 probe 2026-07-16 (delText-under-moveFrom failed open).
fn check_del_text_under_del(pkg: &PartFs, report: &mut ValidityReport) {
    for name in pkg.parts() {
        if !name.ends_with(".xml") {
            continue;
        }
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else {
            continue;
        };
        for del in dom.descendants(root, Some(&W::del())) {
            for t in dom.descendants(del, Some(&W::t())) {
                if ancestor_has(&dom, t, del, "ins") {
                    continue;
                }
                report.fail(format!("w:t under w:del in '{name}' (must be w:delText)"));
            }
        }
        let move_from = W::name("moveFrom");
        for mf in dom.descendants(root, Some(&move_from)) {
            for dt in dom.descendants(mf, Some(&W::name("delText"))) {
                if ancestor_has(&dom, dt, mf, "ins") {
                    continue;
                }
                report.fail(format!(
                    "w:delText under w:moveFrom in '{name}' (Word requires w:t)"
                ));
            }
        }
    }
}

fn ancestor_has(dom: &Dom, node: NodeId, stop: NodeId, local: &str) -> bool {
    let mut cur = dom.parent(node);
    while let Some(p) = cur {
        if p == stop {
            break;
        }
        if let Some(n) = dom.name(p)
            && n.local_name() == local
        {
            return true;
        }
        cur = dom.parent(p);
    }
    false
}

const COMMENT_FAMILY: [(&str, &str, &str); 4] = [
    (
        "word/comments.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
    ),
    (
        "word/commentsExtended.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml",
        "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
    ),
    (
        "word/commentsIds.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsIds+xml",
        "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds",
    ),
    (
        "word/commentsExtensible.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtensible+xml",
        "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible",
    ),
];

fn check_comment_graph(pkg: &PartFs, report: &mut ValidityReport) {
    let Some(main) = pkg.main_document_part().or_else(|| {
        if pkg.part_bytes("word/document.xml").is_some() {
            Some("word/document.xml".into())
        } else {
            None
        }
    }) else {
        return;
    };
    let Some(main_xml) = pkg.part_string(&main) else {
        return;
    };
    let mut main_dom = Dom::new();
    let main_doc = main_dom.parse_xdocument(&main_xml);
    let Some(main_root) = main_dom.root(main_doc) else {
        return;
    };
    let starts = comment_id_counts(&main_dom, main_root, "commentRangeStart");
    let ends = comment_id_counts(&main_dom, main_root, "commentRangeEnd");
    let references = comment_id_counts(&main_dom, main_root, "commentReference");

    let Some(comments_xml) = pkg.part_string("word/comments.xml") else {
        for (kind, counts) in [
            ("commentRangeStart", &starts),
            ("commentRangeEnd", &ends),
            ("commentReference", &references),
        ] {
            for id in counts.keys() {
                report.fail(format!(
                    "{kind} id '{id}' has no entry in word/comments.xml"
                ));
            }
        }
        for (part, _, _) in &COMMENT_FAMILY[1..] {
            if pkg.part_bytes(part).is_some() {
                report.fail(format!("'{part}' exists without word/comments.xml"));
            }
        }
        return;
    };

    let mut comments_dom = Dom::new();
    let comments_doc = comments_dom.parse_xdocument(&comments_xml);
    let Some(comments_root) = comments_dom.root(comments_doc) else {
        return;
    };
    let definitions = comment_id_counts(&comments_dom, comments_root, "comment");
    for (kind, counts) in [
        ("comment definition", &definitions),
        ("commentRangeStart", &starts),
        ("commentRangeEnd", &ends),
        ("commentReference", &references),
    ] {
        check_unique_comment_ids(kind, counts, report);
    }
    let definition_ids: HashSet<String> = definitions.keys().cloned().collect();
    for (kind, counts) in [
        ("commentRangeStart", &starts),
        ("commentRangeEnd", &ends),
        ("commentReference", &references),
    ] {
        let ids: HashSet<String> = counts.keys().cloned().collect();
        for id in definition_ids.difference(&ids) {
            report.fail(format!(
                "comment definition id '{id}' has no matching {kind}"
            ));
        }
        for id in ids.difference(&definition_ids) {
            report.fail(format!(
                "{kind} id '{id}' has no entry in word/comments.xml"
            ));
        }
    }

    check_comment_family_packaging(pkg, &main, report);
    for (part, _, _) in COMMENT_FAMILY {
        if pkg.part_bytes(part).is_some() {
            check_namespace_qname_context(pkg, part, report);
        }
    }

    let aux_present = COMMENT_FAMILY[1..]
        .iter()
        .any(|(part, _, _)| pkg.part_bytes(part).is_some());
    let mut all_para_ids = HashSet::new();
    let mut last_para_ids = HashSet::new();
    for comment in comments_dom.elements(comments_root, Some(&W::name("comment"))) {
        let comment_id = comments_dom
            .attribute(comment, &W::id())
            .unwrap_or("<missing>");
        let mut comment_para_ids = Vec::new();
        for paragraph in comments_dom.descendants(comment, Some(&W::p())) {
            let Some(para_id) = comments_dom.attribute(paragraph, &W14::name("paraId")) else {
                continue;
            };
            let key = para_id.to_ascii_uppercase();
            check_hex_id("paraId", para_id, true, report);
            if !all_para_ids.insert(key.clone()) {
                report.fail(format!(
                    "duplicate comment paraId '{para_id}' in word/comments.xml"
                ));
            }
            comment_para_ids.push(key);
        }
        if aux_present && comment_para_ids.is_empty() {
            report.fail(format!(
                "comment id '{comment_id}' has no w14:paraId for its auxiliary metadata"
            ));
        }
        if let Some(last) = comment_para_ids.last() {
            last_para_ids.insert(last.clone());
        }
    }

    let extended_para_ids = check_comments_extended(pkg, &last_para_ids, report);
    let durable_ids = check_comments_ids(pkg, &last_para_ids, report);
    check_comments_extensible(pkg, durable_ids.as_ref(), report);
    if let Some(extended) = extended_para_ids {
        for parent in extended.parents.values() {
            if !extended.keys.contains(parent) {
                report.fail(format!(
                    "commentsExtended paraIdParent '{parent}' does not resolve"
                ));
            }
        }
        check_parent_cycles(&extended.parents, report);
    }
}

fn comment_id_counts(dom: &Dom, root: NodeId, local: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for element in dom.descendants(root, Some(&W::name(local))) {
        if let Some(id) = dom.attribute(element, &W::id()) {
            *counts.entry(id.to_string()).or_default() += 1;
        }
    }
    counts
}

fn check_unique_comment_ids(
    kind: &str,
    counts: &HashMap<String, usize>,
    report: &mut ValidityReport,
) {
    for (id, count) in counts {
        if *count != 1 {
            report.fail(format!("{kind} id '{id}' occurs {count} times"));
        }
    }
}

fn check_comment_family_packaging(pkg: &PartFs, main: &str, report: &mut ValidityReport) {
    let rels = pkg.read_rels_for(main);
    for (part, content_type, relationship_type) in COMMENT_FAMILY {
        let part_present = pkg.part_bytes(part).is_some();
        let matching: Vec<_> = rels
            .into_iter()
            .flat_map(|relationships| &relationships.items)
            .filter(|relationship| relationship.rel_type == relationship_type)
            .collect();
        if part_present {
            if pkg.content_type_for(part).as_deref() != Some(content_type) {
                report.fail(format!(
                    "'{part}' has the wrong content type (expected '{content_type}')"
                ));
            }
            if matching.len() != 1 {
                report.fail(format!(
                    "'{main}' needs exactly one relationship to '{part}', found {}",
                    matching.len()
                ));
            }
            for relationship in matching {
                let resolved = pkg
                    .resolve_rel_target(main, &relationship.target)
                    .trim_start_matches('/')
                    .to_string();
                if relationship.target_mode.as_deref() == Some("External") || resolved != part {
                    report.fail(format!(
                        "comment relationship '{}' on '{main}' resolves to '{}' instead of '{part}'",
                        relationship.id, relationship.target
                    ));
                }
            }
        } else if !matching.is_empty() {
            report.fail(format!(
                "'{main}' has a relationship for missing comment part '{part}'"
            ));
        }
    }
}

fn check_hex_id(label: &str, value: &str, word_para_bound: bool, report: &mut ValidityReport) {
    let parsed = if value.len() == 8 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        u32::from_str_radix(value, 16).ok()
    } else {
        None
    };
    let Some(parsed) = parsed else {
        report.fail(format!(
            "{label} '{value}' is not an 8-digit hexadecimal id"
        ));
        return;
    };
    if word_para_bound && parsed >= 0x8000_0000 {
        report.fail(format!("{label} '{value}' is outside Word's paraId range"));
    }
}

struct ExtendedGraph {
    keys: HashSet<String>,
    parents: HashMap<String, String>,
}

fn check_comments_extended(
    pkg: &PartFs,
    last_para_ids: &HashSet<String>,
    report: &mut ValidityReport,
) -> Option<ExtendedGraph> {
    let xml = pkg.part_string("word/commentsExtended.xml")?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc)?;
    let mut keys = HashSet::new();
    let mut parents = HashMap::new();
    for entry in dom.elements(root, None) {
        let Some(para_id) = attribute_by_local(&dom, entry, "paraId") else {
            report.fail("commentsExtended entry has no paraId");
            continue;
        };
        let key = para_id.to_ascii_uppercase();
        check_hex_id("commentsExtended paraId", para_id, true, report);
        if !keys.insert(key.clone()) {
            report.fail(format!("duplicate commentsExtended paraId '{para_id}'"));
        }
        if let Some(parent) = attribute_by_local(&dom, entry, "paraIdParent") {
            let parent = parent.to_ascii_uppercase();
            check_hex_id("commentsExtended paraIdParent", &parent, true, report);
            if parent == key {
                report.fail(format!("commentsExtended paraId '{key}' is its own parent"));
            }
            parents.insert(key, parent);
        }
    }
    check_exact_key_set("commentsExtended paraId", &keys, last_para_ids, report);
    Some(ExtendedGraph { keys, parents })
}

fn check_comments_ids(
    pkg: &PartFs,
    last_para_ids: &HashSet<String>,
    report: &mut ValidityReport,
) -> Option<HashSet<String>> {
    let xml = pkg.part_string("word/commentsIds.xml")?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc)?;
    let mut para_ids = HashSet::new();
    let mut durable_ids = HashSet::new();
    for entry in dom.elements(root, None) {
        let Some(para_id) = attribute_by_local(&dom, entry, "paraId") else {
            report.fail("commentsIds entry has no paraId");
            continue;
        };
        let para_key = para_id.to_ascii_uppercase();
        check_hex_id("commentsIds paraId", para_id, true, report);
        if !para_ids.insert(para_key) {
            report.fail(format!("duplicate commentsIds paraId '{para_id}'"));
        }
        let Some(durable_id) = attribute_by_local(&dom, entry, "durableId") else {
            report.fail(format!("commentsIds paraId '{para_id}' has no durableId"));
            continue;
        };
        let durable_key = durable_id.to_ascii_uppercase();
        check_hex_id("commentsIds durableId", durable_id, false, report);
        if !durable_ids.insert(durable_key) {
            report.fail(format!("duplicate commentsIds durableId '{durable_id}'"));
        }
    }
    check_exact_key_set("commentsIds paraId", &para_ids, last_para_ids, report);
    Some(durable_ids)
}

fn check_comments_extensible(
    pkg: &PartFs,
    expected_durable_ids: Option<&HashSet<String>>,
    report: &mut ValidityReport,
) {
    let Some(xml) = pkg.part_string("word/commentsExtensible.xml") else {
        return;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else { return };
    let mut durable_ids = HashSet::new();
    for entry in dom.elements(root, None) {
        let Some(durable_id) = attribute_by_local(&dom, entry, "durableId") else {
            report.fail("commentsExtensible entry has no durableId");
            continue;
        };
        let key = durable_id.to_ascii_uppercase();
        check_hex_id("commentsExtensible durableId", durable_id, false, report);
        if !durable_ids.insert(key) {
            report.fail(format!(
                "duplicate commentsExtensible durableId '{durable_id}'"
            ));
        }
    }
    match expected_durable_ids {
        Some(expected) => check_exact_key_set(
            "commentsExtensible durableId",
            &durable_ids,
            expected,
            report,
        ),
        None => report.fail("commentsExtensible exists without commentsIds"),
    }
}

fn attribute_by_local<'a>(dom: &'a Dom, element: NodeId, local: &str) -> Option<&'a str> {
    for index in 0..dom.attr_count(element) {
        let (name, value) = dom.attr_at(element, index);
        if name.local_name() == local {
            return Some(value);
        }
    }
    None
}

fn check_exact_key_set(
    label: &str,
    actual: &HashSet<String>,
    expected: &HashSet<String>,
    report: &mut ValidityReport,
) {
    for key in expected.difference(actual) {
        report.fail(format!("{label} is missing '{key}'"));
    }
    for key in actual.difference(expected) {
        report.fail(format!("{label} '{key}' has no matching comment paragraph"));
    }
}

fn check_parent_cycles(parents: &HashMap<String, String>, report: &mut ValidityReport) {
    for start in parents.keys() {
        let mut seen = HashSet::new();
        let mut current = start;
        while let Some(parent) = parents.get(current) {
            if !seen.insert(current.clone()) {
                report.fail(format!(
                    "commentsExtended paraIdParent cycle contains '{current}'"
                ));
                break;
            }
            current = parent;
        }
    }
}

fn is_namespace_qname_list(name: &jubarte::xmllinq::XName) -> bool {
    (name.namespace_name().is_empty() && name.local_name() == "Requires")
        || (name.namespace_name() == MC::URI
            && matches!(
                name.local_name(),
                "Ignorable"
                    | "PreserveAttributes"
                    | "PreserveElements"
                    | "ProcessContent"
                    | "MustUnderstand"
            ))
}

fn check_namespace_qname_context(pkg: &PartFs, part: &str, report: &mut ValidityReport) {
    let Some(xml) = pkg.part_string(part) else {
        return;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else { return };
    for element in dom.descendants_and_self(root, None) {
        for (name, value) in dom.attributes(element) {
            if !is_namespace_qname_list(&name) {
                continue;
            }
            for token in value.split_whitespace() {
                let prefix = token.split_once(':').map_or(token, |(prefix, _)| prefix);
                if prefix != "xml" && namespace_in_scope(&dom, element, prefix).is_none() {
                    report.fail(format!(
                        "unresolved namespace prefix '{prefix}' in {}='{}' in '{part}'",
                        name.local_name(),
                        value
                    ));
                }
            }
        }
    }
}

fn namespace_in_scope<'a>(dom: &'a Dom, element: NodeId, prefix: &str) -> Option<&'a str> {
    let mut current = Some(element);
    while let Some(node) = current {
        for index in 0..dom.attr_count(node) {
            let (name, value) = dom.attr_at(node, index);
            if dom.is_namespace_declaration(name) && name.local_name() == prefix {
                return Some(value);
            }
        }
        current = dom.parent(node);
    }
    None
}
