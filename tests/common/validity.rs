//! Ring 1 — Rust-native Word-validity package invariants (plan D1).
//!
//! `assert_word_valid_package` fails a test when a produced package would make
//! Word offer repair (dangling rels, duplicate revision ids, orphan comment
//! anchors, …). Intentional broken probes live in `tests/m_validity_ring1.rs`.

use jubarte::namespaces::W;
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
    check_comment_anchors(&pkg, &mut report);
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

/// Every commentReference has matching range start/end and a comments.xml entry.
fn check_comment_anchors(pkg: &PartFs, report: &mut ValidityReport) {
    let Some(main) = pkg.main_document_part().or_else(|| {
        if pkg.part_bytes("word/document.xml").is_some() {
            Some("word/document.xml".into())
        } else {
            None
        }
    }) else {
        return;
    };
    let Some(xml) = pkg.part_string(&main) else {
        return;
    };
    let mut starts = HashSet::new();
    let mut ends = HashSet::new();
    let mut refs = HashSet::new();
    scrape_comment_ids(&xml, "commentRangeStart", &mut starts);
    scrape_comment_ids(&xml, "commentRangeEnd", &mut ends);
    scrape_comment_ids(&xml, "commentReference", &mut refs);

    let comment_ids = if let Some(cx) = pkg.part_string("word/comments.xml") {
        let mut ids = HashSet::new();
        scrape_comment_ids(&cx, "comment", &mut ids);
        ids
    } else {
        HashSet::new()
    };

    for id in &refs {
        if !starts.contains(id) {
            report.fail(format!(
                "commentReference id '{id}' has no commentRangeStart"
            ));
        }
        if !ends.contains(id) {
            report.fail(format!("commentReference id '{id}' has no commentRangeEnd"));
        }
        if !comment_ids.contains(id) {
            report.fail(format!(
                "commentReference id '{id}' has no entry in word/comments.xml"
            ));
        }
    }
    for id in &starts {
        if !ends.contains(id) {
            report.fail(format!(
                "commentRangeStart id '{id}' has no matching commentRangeEnd"
            ));
        }
    }
}

fn scrape_comment_ids(xml: &str, local: &str, out: &mut HashSet<String>) {
    // Match w:local or :local with w:id=
    let tag_patterns = [
        format!("<{local} "),
        format!(": {local} "),
        format!("w:{local} "),
    ];
    // simpler: find local name then nearby id=
    let mut rest = xml;
    let needle = local;
    while let Some(i) = rest.find(needle) {
        // ensure tag-ish context
        let start = rest[..i].rfind('<').unwrap_or(0);
        let region =
            &rest[start..i + needle.len() + 80.min(rest.len().saturating_sub(i + needle.len()))];
        if region.contains(needle)
            && let Some(id_pos) = region.find("id=\"")
        {
            let after = &region[id_pos + 4..];
            if let Some(end) = after.find('"') {
                out.insert(after[..end].to_string());
            }
        }
        rest = &rest[i + needle.len()..];
    }
    let _ = tag_patterns;
}
