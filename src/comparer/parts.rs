// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.H.1 — relationship parsing helpers for footnotes/related-part relocation.
//! Port of ParseRelationshipRows (:6945), DecodeXmlAttribute (:6967),
//! RequiredRelTypeSuffix (:6914), IsExternalRelationship (:6678). The rel-name
//! tables (s_RelationshipAttributeNames, s_ElementsWithRelationshipIds,
//! AttributesToTrimWhenCloning) live in `tables.rs`.

/// A parsed `<Relationship>` row from a `.rels` part.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationshipRow {
    /// `id`.
    pub id: String,
    /// `rel_type`.
    pub rel_type: String,
    /// `target`.
    pub target: String,
    /// `external`.
    pub external: bool,
}

/// `DecodeXmlAttribute` (:6967) — undo entity escaping (order: quot, gt, lt, amp).
pub fn decode_xml_attribute(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

fn attr_of(tag: &str, name: &str) -> Option<String> {
    // find `name="..."` (allowing whitespace around =), simple scan.
    let key = name.to_string();
    let mut search_from = 0;
    while let Some(pos) = tag[search_from..].find(&key) {
        let abs = search_from + pos;
        // ensure it's a whole attribute name (preceded by space/start)
        let before_ok = abs == 0 || tag.as_bytes()[abs - 1].is_ascii_whitespace();
        let after = &tag[abs + key.len()..];
        let after_trim = after.trim_start();
        if before_ok && after_trim.starts_with('=') {
            let rest = after_trim[1..].trim_start();
            if let Some(stripped) = rest.strip_prefix('"')
                && let Some(end) = stripped.find('"')
            {
                return Some(stripped[..end].to_string());
            }
        }
        search_from = abs + key.len();
    }
    None
}

/// `ParseRelationshipRows` (:6945) — extract `<Relationship …>` rows.
pub fn parse_relationship_rows(rels_xml: &str) -> Vec<RelationshipRow> {
    let mut rows = Vec::new();
    let mut from = 0;
    while let Some(start) = rels_xml[from..].find("<Relationship") {
        let abs = from + start;
        // ensure word boundary after "<Relationship"
        let after = rels_xml[abs + "<Relationship".len()..].chars().next();
        if !matches!(after, Some(c) if c.is_ascii_whitespace()) {
            from = abs + "<Relationship".len();
            continue;
        }
        let end = match rels_xml[abs..].find('>') {
            Some(e) => abs + e + 1,
            None => break,
        };
        let tag = &rels_xml[abs..end];
        from = end;
        let id = attr_of(tag, "Id");
        let rel_type = attr_of(tag, "Type");
        let target = attr_of(tag, "Target");
        let mode = attr_of(tag, "TargetMode");
        if let (Some(id), Some(rel_type), Some(target)) = (id, rel_type, target) {
            let target = decode_xml_attribute(&target);
            // OPC marks external rels with TargetMode="External", but a hyperlink rel
            // type or an absolute-URI target is external regardless — there is no part
            // to copy. Honor both so such rows aren't misclassified as internal and
            // dropped during reconcile (the `else` branch would try to copy a part for
            // an `https://…` target, fail, and orphan the rId). `IsExternalRelationship`
            // (:6678).
            let external =
                mode.as_deref() == Some("External") || is_external_relationship(&rel_type, &target);
            rows.push(RelationshipRow {
                id,
                rel_type,
                target,
                external,
            });
        }
    }
    rows
}

/// `RequiredRelTypeSuffix` (:6914) — the rel-type suffix an element's rId must
/// resolve to (by local name), or None.
pub fn required_rel_type_suffix(local_name: &str) -> Option<&'static str> {
    match local_name {
        "hyperlink" | "hlinkClick" => Some("/hyperlink"),
        "chart" => Some("/chart"),
        "headerReference" => Some("/header"),
        "footerReference" => Some("/footer"),
        "blip" | "imagedata" => Some("/image"),
        "OLEObject" => Some("/oleObject"),
        _ => None,
    }
}

/// `IsExternalRelationship` (:6678) — hyperlink rel type, or an absolute-URI
/// target (has a scheme and doesn't start with `/`).
pub fn is_external_relationship(rel_type: &str, target: &str) -> bool {
    if rel_type.ends_with("/hyperlink") {
        return true;
    }
    if target.starts_with('/') {
        return false;
    }
    if let Some(i) = target.find(':') {
        let scheme = &target[..i];
        if !scheme.is_empty()
            && scheme.chars().next().unwrap().is_ascii_alphabetic()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+.-".contains(c))
        {
            return true;
        }
    }
    false
}

use super::tables::S_RELATIONSHIP_ATTRIBUTE_NAMES;
use crate::opc::PartFs;
use crate::unid::generate_unid;
use crate::xmllinq::{Dom, NodeId};

/// Package URI for a part being copied during dangling-rel reconcile.
/// Markup parts keep their `word/…` path (with collision uniquify); binary
/// media still lands under `word/media/P{unid}.ext`.
fn dest_uri_for_reconciled_part(dest: &PartFs, target_part: &str, bytes: &[u8]) -> String {
    let ext = target_part
        .rsplit('.')
        .next()
        .filter(|e| !e.contains('/'))
        .unwrap_or("bin");
    let ext_lc = ext.to_ascii_lowercase();
    let is_image = matches!(
        ext_lc.as_str(),
        "png" | "jpeg" | "jpg" | "gif" | "tiff" | "tif" | "bmp" | "svg" | "ico" | "emf" | "wmf"
    );
    if target_part.starts_with("word/media/") || is_image {
        return format!("word/media/P{}.{ext}", generate_unid());
    }
    // Preserve conventional word/* paths for footers/headers/numbering/notes.
    match dest.part_bytes(target_part) {
        None => target_part.to_string(),
        Some(existing) if existing == bytes => target_part.to_string(),
        Some(_) => {
            let (dir, base) = target_part
                .rsplit_once('/')
                .unwrap_or(("word", target_part));
            let mut n = 0usize;
            loop {
                let candidate = if n == 0 {
                    format!("{dir}/redlineB_{base}")
                } else {
                    format!("{dir}/redlineB_{n}_{base}")
                };
                match dest.part_bytes(&candidate) {
                    None => return candidate,
                    Some(existing) if existing == bytes => return candidate,
                    Some(_) => n += 1,
                }
            }
        }
    }
}

/// True when `el` sits inside inserted / moved-in content (a `w:ins` or
/// `w:moveTo` ancestor). Such a reference's rId belongs to the REVISED
/// document, so on an rId collision with the base package it must resolve to
/// the revised part — not silently render the base one (image_inline_and_block
/// × image_doc: both docs' first image is rId4; our inserted drawing rendered
/// BASE's image because reconcile saw rId4 already resolved in the destination).
fn ref_from_inserted_content(dom: &Dom, mut el: NodeId) -> bool {
    while let Some(p) = dom.parent(el) {
        if let Some(n) = dom.name(p)
            && n.namespace_name() == crate::namespaces::W::URI
            && matches!(n.local_name(), "ins" | "moveTo")
        {
            return true;
        }
        el = p;
    }
    false
}

/// Bytes of the part the destination's `rid` currently resolves to (via the
/// main-document rels), or None if unresolved.
fn dest_rid_part_bytes(dest: &PartFs, doc_part: &str, rid: &str) -> Option<Vec<u8>> {
    let rels = dest.read_rels_for(doc_part)?;
    let target = rels
        .items
        .iter()
        .find(|i| i.id == rid)
        .map(|i| i.target.clone())?;
    let part = dest.resolve_rel_target(doc_part, &target);
    dest.part_bytes(&part).map(|b| b.to_vec())
}

/// M4.H.3 — `ReconcileDanglingRelationships` (:6711), conservative form: ensure
/// every rId referenced by the result document resolves in the destination
/// package. rIds already present are left as-is; rIds missing from the
/// destination but found in a source package are carried over (internal media
/// copied + a fresh rel minted, the referencing attribute rewired); rIds found
/// nowhere have their attribute dropped (which is what prevents Word's
/// "unreadable content" repair). Text documents reference no rIds → no-op.
pub fn reconcile_dangling_relationships(
    dom: &mut Dom,
    root: NodeId,
    dest: &mut PartFs,
    sources: &[&PartFs],
) {
    let doc_part = dest
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());

    // 1. collect referenced (element, attr_name, rId, from_inserted_content).
    let mut refs: Vec<(NodeId, crate::xmllinq::XName, String, bool)> = Vec::new();
    for el in dom.descendants_and_self(root, None) {
        for (an, av) in dom.attributes(el) {
            if S_RELATIONSHIP_ATTRIBUTE_NAMES.contains(&an) {
                let inserted = ref_from_inserted_content(dom, el);
                refs.push((el, an, av, inserted));
            }
        }
    }
    if refs.is_empty() {
        return;
    }

    // 2. ids already in the destination's document rels. NOTE: the OPC layer
    //    parses `.rels` into `Relationships` (NOT exposed as a part), so we must
    //    read them via `read_rels_for` — `part_bytes("…/_rels/….rels")` always
    //    returns None, which previously left this set empty and made EVERY rId
    //    look like an orphan (silently dropping image/hyperlink references).
    // id -> rel_type, so a referenced rId only counts as "already resolved" when
    // its destination relationship is of the TYPE the reference requires (an rId
    // string can collide across the two source docs for different part types).
    let dest_rels: std::collections::HashMap<String, String> = dest
        .read_rels_for(&doc_part)
        .map(|rels| {
            rels.items
                .iter()
                .map(|r| (r.id.clone(), r.rel_type.clone()))
                .collect()
        })
        .unwrap_or_default();

    // 3. source rows by id (first source wins) + which package they came from.
    //    (search each source's main-document rels)
    let mut source_rows: Vec<(usize, RelationshipRow, String)> = Vec::new();
    for (si, src) in sources.iter().enumerate() {
        let src_doc = src
            .main_document_part()
            .unwrap_or_else(|| "word/document.xml".to_string());
        if let Some(rels) = src.read_rels_for(&src_doc) {
            for r in &rels.items {
                source_rows.push((
                    si,
                    RelationshipRow {
                        id: r.id.clone(),
                        rel_type: r.rel_type.clone(),
                        target: r.target.clone(),
                        // Same rule as parse_relationship_rows: TargetMode OR
                        // hyperlink/absolute-URI classification.
                        external: r.target_mode.as_deref() == Some("External")
                            || is_external_relationship(&r.rel_type, &r.target),
                    },
                    src_doc.clone(),
                ));
            }
        }
    }
    // Type-aware source lookup: prefer a source relationship of the required type
    // (resolves rId collisions, e.g. B's rId1=/header vs A's rId1=/endnotes).
    let find_source = |rid: &str, suffix: Option<&str>| {
        source_rows
            .iter()
            .find(|(_, r, _)| r.id == rid && suffix.is_none_or(|s| r.rel_type.ends_with(s)))
    };

    // 4. reconcile each referenced rId not already in the destination.
    for (el, an, rid, inserted) in refs {
        let suffix = dom
            .name(el)
            .and_then(|n| required_rel_type_suffix(n.local_name()));
        // A destination rId resolves the reference only if its type matches.
        let dest_ok = dest_rels
            .get(&rid)
            .is_some_and(|t| suffix.is_none_or(|s| t.ends_with(s)));
        if dest_ok {
            // rId collision: an INSERTED ref keeps the revised doc's rId, which
            // may numerically match a base rId of DIFFERENT content. Left as-is
            // it silently renders the base part. When the revised source has the
            // same rId pointing to distinct bytes, carry that part over under a
            // fresh id and repoint (image_inline_and_block × image_doc). Rare
            // path (only inserted refs whose rId already resolves in dest); the
            // common shared-image case (identical bytes) is left untouched.
            if inserted
                && let Some((si, row, src_doc)) = source_rows
                    .iter()
                    .rev()
                    .find(|(_, r, _)| r.id == rid && suffix.is_none_or(|s| r.rel_type.ends_with(s)))
                && !row.external
            {
                let src = sources[*si];
                let modf_part = src.resolve_rel_target(src_doc, &row.target);
                if let Some(modf_bytes) = src.part_bytes(&modf_part) {
                    let modf_bytes = modf_bytes.to_vec();
                    if dest_rid_part_bytes(dest, &doc_part, &rid).as_deref() != Some(&modf_bytes) {
                        let new_uri = dest_uri_for_reconciled_part(dest, &modf_part, &modf_bytes);
                        if let Some(ct) = src.content_type_for(&modf_part) {
                            dest.add_content_type_override(&new_uri, &ct);
                        }
                        dest.set_part(&new_uri, modf_bytes);
                        let rel_target = new_uri
                            .strip_prefix("word/")
                            .unwrap_or(&new_uri)
                            .to_string();
                        let new_rid =
                            dest.add_document_relationship(&doc_part, &row.rel_type, &rel_target);
                        dom.set_attribute_value(el, &an, Some(&new_rid));
                    }
                }
            }
            continue;
        }
        match find_source(&rid, suffix) {
            None => {
                // orphan everywhere → drop the dangling attribute.
                dom.set_attribute_value(el, &an, None);
            }
            Some((si, row, src_doc)) => {
                if row.external {
                    // Preserve TargetMode="External": absolute URIs are
                    // illegal for the (default) Internal mode and Word's
                    // packaging layer rejects the file (repair prompt).
                    let new_rid = dest.add_document_relationship_external(
                        &doc_part,
                        &row.rel_type,
                        &row.target,
                    );
                    dom.set_attribute_value(el, &an, Some(&new_rid));
                } else {
                    let src = sources[*si];
                    let target_part = src.resolve_rel_target(src_doc, &row.target);
                    match src.part_bytes(&target_part) {
                        Some(bytes) => {
                            let bytes = bytes.to_vec();
                            // Headers/footers/numbering/notes must keep OPC-conventional
                            // paths (word/footerN.xml). Dumping them into word/media/P*.xml
                            // (legacy always-media rewrite) left file_21 with 0 renderable
                            // footers while Word's redline carries all 20+ — LO page geometry
                            // drifts (106 vs 107). Images still use media/P{unid}.ext.
                            let new_uri = dest_uri_for_reconciled_part(dest, &target_part, &bytes);
                            if let Some(ct) = src.content_type_for(&target_part) {
                                dest.add_content_type_override(&new_uri, &ct);
                            }
                            dest.set_part(&new_uri, bytes);
                            // relationship target relative to the document part folder.
                            let rel_target = new_uri
                                .strip_prefix("word/")
                                .unwrap_or(&new_uri)
                                .to_string();
                            let new_rid = dest.add_document_relationship(
                                &doc_part,
                                &row.rel_type,
                                &rel_target,
                            );
                            dom.set_attribute_value(el, &an, Some(&new_rid));
                        }
                        None => {
                            dom.set_attribute_value(el, &an, None);
                        }
                    }
                }
            }
        }
    }
}
