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
use crate::xmllinq::{Dom, NodeId};

/// Package URI for a part being copied during dangling-rel reconcile.
/// Markup parts keep their `word/…` path (with collision uniquify); binary
/// media lands under `word/media/P{sha256}.ext` (extension lowercased, since
/// part names are case-insensitive), named by the SHA-256 of its bytes. The name depends only on the media, never on how many compares ran
/// before it in the process, so one pair compared twice gives identical bytes.
/// Identical media copied twice share one part. A name already held by
/// different bytes (a SHA-256 collision, or a source part that happens to
/// carry the name) gets a `_{n}` suffix, so two different images never share a
/// part.
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
        let digest = crate::inspect::source_sha256(bytes);
        let mut n = 0usize;
        loop {
            let candidate = if n == 0 {
                format!("word/media/P{digest}.{ext_lc}")
            } else {
                format!("word/media/P{digest}_{n}.{ext_lc}")
            };
            match dest.part_bytes(&candidate) {
                None => return candidate,
                Some(existing) if existing == bytes => return candidate,
                Some(_) => n += 1,
            }
        }
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
                        let rel_target = crate::opc::relative_rel_target(&doc_part, &new_uri);
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
                            // drifts (106 vs 107). Images still use media/P{sha256}.ext.
                            let new_uri = dest_uri_for_reconciled_part(dest, &target_part, &bytes);
                            if let Some(ct) = src.content_type_for(&target_part) {
                                dest.add_content_type_override(&new_uri, &ct);
                            }
                            dest.set_part(&new_uri, bytes);
                            // relationship target relative to the document part folder.
                            let rel_target = crate::opc::relative_rel_target(&doc_part, &new_uri);
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

/// The package that supplied a part's content: `(source package, part name there)`.
type PartSource<'a> = (&'a PartFs, String);

/// Where the content of the output part `part` can have come from, in lookup
/// order. The output package starts as the original (A), so A-origin parts
/// already carry A's relationships; a reference left unresolved belongs to
/// content grafted in from the revised document (B). `redlineB_*` parts are
/// B's by construction and never resolve against A.
fn part_sources<'a>(part: &str, a: &'a PartFs, b: &'a PartFs) -> Vec<PartSource<'a>> {
    match revised_part_name(part) {
        Some(orig) => vec![(b, orig)],
        None => vec![(b, part.to_string()), (a, part.to_string())],
    }
}

/// B's own name for a `redlineB_*` output part (`redlineB_{base}`, or
/// `redlineB_{n}_{base}` for the n-th collision); None for any other part.
fn revised_part_name(part: &str) -> Option<String> {
    let (dir, base) = part.rsplit_once('/').unwrap_or(("", part));
    let rest = base.strip_prefix("redlineB_")?;
    let orig = rest
        .split_once('_')
        .filter(|(n, _)| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()))
        .map_or(rest, |(_, tail)| tail);
    Some(if dir.is_empty() {
        orig.to_string()
    } else {
        format!("{dir}/{orig}")
    })
}

/// Every non-main XML part must resolve its own `r:*` references in its own
/// `.rels`: comments, notes, headers/footers and numbering picture bullets
/// copied from the revised document otherwise keep B's ids without B's
/// relationships, and Word refuses the whole package ("document loaded
/// empty"). Per-part counterpart of [`reconcile_dangling_relationships`]
/// (which covers the main document): missing relationships are carried from
/// the part's source (external targets verbatim, internal parts copied), and
/// references no source can resolve lose the attribute.
pub fn reconcile_part_relationships(dest: &mut PartFs, main_part: &str, a: &PartFs, b: &PartFs) {
    let mut parts: Vec<String> = dest
        .parts()
        .into_iter()
        .filter(|p| p != main_part && p.starts_with("word/") && p.ends_with(".xml"))
        .collect();
    parts.sort();
    for part in parts {
        reconcile_one_part(dest, &part, a, b);
    }
}

/// True when `xml` carries an attribute in the relationships namespace — the
/// cheap gate that keeps parts without references from being parsed.
fn has_relationship_attribute(xml: &str) -> bool {
    let uri = format!("=\"{}\"", crate::namespaces::R::URI);
    xml.match_indices(&uri).any(|(at, _)| {
        let decl = &xml[..at];
        decl.rfind("xmlns:").is_some_and(|x| {
            let prefix = &decl[x + "xmlns:".len()..];
            !prefix.is_empty()
                && prefix
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
                && xml
                    .match_indices(&format!("{prefix}:"))
                    .any(|(i, _)| i > 0 && xml.as_bytes()[i - 1].is_ascii_whitespace())
        })
    })
}

/// Carry `src_part`'s relationship `rid` (when its type `fits`) onto `part` of
/// `dest` under a fresh id: external targets verbatim, internal targets copied
/// under a collision-proof name. Images reuse an existing internal relationship
/// when its target has the same bytes and content type. Returns the id, or `None` when the source
/// has no such relationship or its target part.
pub fn carry_relationship(
    dest: &mut PartFs,
    part: &str,
    src: &PartFs,
    src_part: &str,
    rid: &str,
    fits: impl Fn(&str) -> bool,
) -> Option<String> {
    let row = src
        .read_rels_for(src_part)?
        .items
        .iter()
        .find(|r| r.id == rid && fits(&r.rel_type))?
        .clone();
    let external = row.target_mode.as_deref() == Some("External")
        || is_external_relationship(&row.rel_type, &row.target);
    if external {
        return Some(dest.add_document_relationship_external(part, &row.rel_type, &row.target));
    }
    let target_part = src.resolve_rel_target(src_part, &row.target);
    let bytes = src.part_bytes(&target_part)?.to_vec();
    // Distinct picture bullets can share one source image. Reuse the carried
    // relationship before allocating another media part for that image.
    if row.rel_type.ends_with("/image")
        && let Some(existing) = dest.read_rels_for(part).and_then(|rels| {
            rels.items.iter().find(|r| {
                if r.rel_type != row.rel_type
                    || r.target_mode.as_deref() == Some("External")
                    || is_external_relationship(&r.rel_type, &r.target)
                {
                    return false;
                }
                let target = dest.resolve_rel_target(part, &r.target);
                dest.part_bytes(&target) == Some(bytes.as_slice())
                    && dest.content_type_for(&target) == src.content_type_for(&target_part)
            })
        })
    {
        return Some(existing.id.clone());
    }
    let new_uri = dest_uri_for_reconciled_part(dest, &target_part, &bytes);
    if let Some(ct) = src.content_type_for(&target_part) {
        dest.add_content_type_override(&new_uri, &ct);
    }
    dest.set_part(&new_uri, bytes);
    Some(dest.add_document_relationship(
        part,
        &row.rel_type,
        &crate::opc::relative_rel_target(part, &new_uri),
    ))
}

/// Carry every relationship the subtree `root` names (`r:id`, `r:embed`,
/// `r:link`, ...) from `src_part` of `src` onto `dest_part` of `dest` through
/// [`carry_relationship`], and rewrite each attribute to the carried id. A
/// reference `src` cannot resolve loses its attribute. An XML part copied
/// this way (a header, a chart) gets its own relationships carried too.
pub fn carry_part_relationships(
    dest: &mut PartFs,
    dest_part: &str,
    src: &PartFs,
    src_part: &str,
    dom: &mut Dom,
    root: NodeId,
) {
    carry_subtree_relationships(dest, dest_part, src, src_part, dom, root, 0);
}

/// Nesting a carried part may reach: header to chart to embedding is three.
const MAX_CARRY_DEPTH: usize = 8;

fn carry_subtree_relationships(
    dest: &mut PartFs,
    dest_part: &str,
    src: &PartFs,
    src_part: &str,
    dom: &mut Dom,
    root: NodeId,
    depth: usize,
) {
    let mut minted: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();
    for el in dom.descendants_and_self(root, None) {
        for (an, rid) in dom.attributes(el) {
            if !S_RELATIONSHIP_ATTRIBUTE_NAMES.contains(&an) {
                continue;
            }
            let suffix = dom
                .name(el)
                .and_then(|n| required_rel_type_suffix(n.local_name()));
            let key = format!("{rid}\u{0}{}", suffix.unwrap_or(""));
            let new_rid = match minted.get(&key) {
                Some(id) => id.clone(),
                None => {
                    let carried = carry_with_own_relationships(
                        dest, dest_part, src, src_part, &rid, suffix, depth,
                    );
                    minted.insert(key, carried.clone());
                    carried
                }
            };
            dom.set_attribute_value(el, &an, new_rid.as_deref());
        }
    }
}

/// [`carry_relationship`], then the same for the relationships of an XML
/// part it newly copied.
fn carry_with_own_relationships(
    dest: &mut PartFs,
    dest_part: &str,
    src: &PartFs,
    src_part: &str,
    rid: &str,
    suffix: Option<&str>,
    depth: usize,
) -> Option<String> {
    let fits = |ty: &str| suffix.is_none_or(|s| ty.ends_with(s));
    let before: std::collections::HashSet<String> = dest.parts().into_iter().collect();
    let new_rid = carry_relationship(dest, dest_part, src, src_part, rid, fits)?;
    let target = |pkg: &PartFs, part: &str, id: &str| {
        let row = pkg.read_rels_for(part)?.items.iter().find(|r| r.id == id)?;
        (row.target_mode.as_deref() != Some("External"))
            .then(|| pkg.resolve_rel_target(part, &row.target))
    };
    if depth < MAX_CARRY_DEPTH
        && let Some(new_part) = target(dest, dest_part, &new_rid)
        && let Some(src_target) = target(src, src_part, rid)
        && new_part.ends_with(".xml")
        && !before.contains(&new_part)
        && src.read_rels_for(&src_target).is_some()
        && let Some(xml) = dest.part_string(&new_part)
    {
        let mut part_dom = Dom::new();
        let doc = part_dom.parse_xdocument(&xml);
        if let Some(part_root) = part_dom.root(doc) {
            carry_subtree_relationships(
                dest,
                &new_part,
                src,
                &src_target,
                &mut part_dom,
                part_root,
                depth + 1,
            );
            dest.set_part(&new_part, part_dom.serialize_document(doc).into_bytes());
        }
    }
    Some(new_rid)
}

fn reconcile_one_part(dest: &mut PartFs, part: &str, a: &PartFs, b: &PartFs) {
    let Some(xml) = dest.part_string(part) else {
        return;
    };
    if !has_relationship_attribute(&xml) {
        return;
    }
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return;
    };
    let dest_types: std::collections::HashMap<String, String> = dest
        .read_rels_for(part)
        .map(|rels| {
            rels.items
                .iter()
                .map(|r| (r.id.clone(), r.rel_type.clone()))
                .collect()
        })
        .unwrap_or_default();
    let sources = part_sources(part, a, b);
    // One new relationship per source id, however many elements share it.
    let mut minted: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut changed = false;
    for el in dom.descendants_and_self(root, None) {
        for (an, rid) in dom.attributes(el) {
            if !S_RELATIONSHIP_ATTRIBUTE_NAMES.contains(&an) {
                continue;
            }
            let suffix = dom
                .name(el)
                .and_then(|n| required_rel_type_suffix(n.local_name()));
            let fits = |ty: &str| suffix.is_none_or(|s| ty.ends_with(s));
            if dest_types.get(&rid).is_some_and(|t| fits(t)) {
                continue;
            }
            let key = format!("{rid}\u{0}{}", suffix.unwrap_or(""));
            let new_rid = match minted.get(&key) {
                Some(id) => Some(id.clone()),
                None => {
                    let carried = sources.iter().find_map(|(src, src_part)| {
                        carry_relationship(dest, part, src, src_part, &rid, fits)
                    });
                    if let Some(id) = &carried {
                        minted.insert(key, id.clone());
                    }
                    carried
                }
            };
            dom.set_attribute_value(el, &an, new_rid.as_deref());
            changed = true;
        }
    }
    if changed {
        dest.set_part(part, dom.serialize_element(root).into_bytes());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    const PACKAGE: &[u8] = include_bytes!("../../tests/fixtures/relids/image_doc.docx");

    #[test]
    fn media_name_is_the_sha256_of_the_bytes_and_ignores_earlier_calls() {
        let dest = PartFs::open(PACKAGE).unwrap();
        let bytes = b"image bytes";
        let first = dest_uri_for_reconciled_part(&dest, "word/media/image9.PNG", bytes);
        // Unrelated copies in between must not move the name.
        for i in 0..5u8 {
            dest_uri_for_reconciled_part(&dest, "word/media/other.png", &[i]);
        }
        let second = dest_uri_for_reconciled_part(&dest, "word/media/image9.PNG", bytes);
        assert_eq!(first, second);
        assert_eq!(
            first,
            format!("word/media/P{}.png", crate::inspect::source_sha256(bytes))
        );
    }

    #[test]
    fn extension_case_does_not_split_identical_media() {
        // OPC part names are case-insensitive, so P{d}.png and P{d}.PNG would
        // be one name; the generated name always carries a lowercase extension.
        let dest = PartFs::open(PACKAGE).unwrap();
        let bytes = b"image bytes";
        assert_eq!(
            dest_uri_for_reconciled_part(&dest, "word/media/image1.png", bytes),
            dest_uri_for_reconciled_part(&dest, "word/media/image2.PNG", bytes)
        );
    }

    #[test]
    fn a_name_held_by_different_bytes_is_never_reused() {
        let mut dest = PartFs::open(PACKAGE).unwrap();
        let bytes = b"image bytes";
        let base = dest_uri_for_reconciled_part(&dest, "word/media/a.png", bytes);
        dest.set_part(&base, b"a different image".to_vec());
        let next = dest_uri_for_reconciled_part(&dest, "word/media/a.png", bytes);
        assert_ne!(base, next);
        assert!(next.ends_with("_1.png"), "{next}");
        // The same bytes already stored under the name reuse that name.
        dest.set_part(&next, bytes.to_vec());
        assert_eq!(
            dest_uri_for_reconciled_part(&dest, "word/media/a.png", bytes),
            next
        );
    }

    #[test]
    fn revised_part_name_undoes_both_collision_forms() {
        assert_eq!(
            revised_part_name("word/redlineB_header1.xml").as_deref(),
            Some("word/header1.xml")
        );
        assert_eq!(
            revised_part_name("word/redlineB_2_footer3.xml").as_deref(),
            Some("word/footer3.xml")
        );
        // A base name that itself contains an underscore is not a collision index.
        assert_eq!(
            revised_part_name("word/redlineB_my_header.xml").as_deref(),
            Some("word/my_header.xml")
        );
        assert_eq!(revised_part_name("word/header1.xml"), None);
    }

    #[test]
    fn relationship_attribute_gate_sees_any_whitespace_before_the_prefix() {
        // A false positive only costs a parse; a false negative ships a dangling id.
        let ns = crate::namespaces::R::URI;
        assert!(has_relationship_attribute(&format!(
            r#"<w:hdr xmlns:r="{ns}"><w:hyperlink r:id="rId1"/></w:hdr>"#
        )));
        assert!(has_relationship_attribute(&format!(
            "<w:hdr xmlns:rel=\"{ns}\"><a:blip\n\trel:embed=\"rId1\"/></w:hdr>"
        )));
        assert!(!has_relationship_attribute(&format!(
            r#"<w:hdr xmlns:r="{ns}"><w:p><w:t>Number:1</w:t></w:p></w:hdr>"#
        )));
        assert!(!has_relationship_attribute("<w:hdr><w:p/></w:hdr>"));
    }

    #[test]
    fn relationship_uri_schemes_and_malformed_rows_have_explicit_classification() {
        for target in [
            "https://host/path",
            "mailto:a@host",
            "a+b.c-d:opaque",
            "A1:payload",
        ] {
            assert!(
                is_external_relationship("urn:part/image", target),
                "{target}"
            );
        }
        for target in [
            "",
            "media/p.png",
            "/a:part",
            ":missing",
            "1bad:part",
            "a b:part",
            "é:part",
        ] {
            assert!(
                !is_external_relationship("urn:part/image", target),
                "{target}"
            );
            assert!(is_external_relationship("urn:part/hyperlink", target));
        }
        for row in [
            "<Relationship Id=unquoted Type=\"x\" Target=\"y\"/>",
            "<Relationship Id=\"x\" Type=\"y\"/>",
            "<Relationship Id=\"x\" Target=\"y\"/>",
            "<Relationship Type=\"x\" Target=\"y\"/>",
            "<Relationship Id=\"x\" Type=\"y\" Target=\"unterminated/>",
            "<Relationship Id=\"x\" Type=\"y\" Target=\"z\"",
            "<Relationships Id=\"x\" Type=\"y\" Target=\"z\"/>",
        ] {
            assert!(parse_relationship_rows(row).is_empty(), "{row}");
        }
        let rows = parse_relationship_rows(
            "<Relationships><Relationship ExtraId=\"bad\" Id = \"good\" ExtraTarget=\"ignored\" Target = \"media/a&amp;b.png\" Type = \"urn:part/image\"/></Relationships>",
        );
        assert_eq!(
            rows,
            vec![RelationshipRow {
                id: "good".into(),
                rel_type: "urn:part/image".into(),
                target: "media/a&b.png".into(),
                external: false
            }]
        );
    }

    #[test]
    fn carried_images_reuse_only_matching_bytes_content_type_and_internal_type() {
        let source_part = "word/headerGap.xml";
        let dest_part = "word/footerGap.xml";
        for matching_type in [false, true] {
            for matching_bytes in [false, true] {
                let mut src = PartFs::open(PACKAGE).unwrap();
                src.set_part("word/media/source.opaque", b"original image bytes".to_vec());
                src.add_content_type_override("word/media/source.opaque", "image/png");
                let rid = src.add_document_relationship(
                    source_part,
                    "urn:part/image",
                    "media/source.opaque",
                );
                let mut dest = PartFs::open(PACKAGE).unwrap();
                dest.set_part(
                    "word/media/existing.opaque",
                    if matching_bytes {
                        b"original image bytes".to_vec()
                    } else {
                        b"different image bytes".to_vec()
                    },
                );
                dest.add_content_type_override(
                    "word/media/existing.opaque",
                    if matching_type {
                        "image/png"
                    } else {
                        "image/jpeg"
                    },
                );
                let old_id = dest.add_document_relationship(
                    dest_part,
                    "urn:part/image",
                    "media/existing.opaque",
                );
                dest.add_document_relationship_external(
                    dest_part,
                    "urn:part/image",
                    "https://host/image.png",
                );
                dest.add_document_relationship(
                    dest_part,
                    "urn:part/chart",
                    "media/existing.opaque",
                );
                let new_id =
                    carry_relationship(&mut dest, dest_part, &src, source_part, &rid, |ty| {
                        ty.ends_with("/image")
                    })
                    .unwrap();
                assert_eq!(new_id == old_id, matching_type && matching_bytes);
                let rel = dest
                    .read_rels_for(dest_part)
                    .unwrap()
                    .items
                    .iter()
                    .find(|r| r.id == new_id)
                    .unwrap();
                assert_eq!(rel.rel_type, "urn:part/image");
                assert_eq!(rel.target_mode, None);
                let target = dest.resolve_rel_target(dest_part, &rel.target);
                assert_eq!(
                    dest.part_bytes(&target),
                    Some(b"original image bytes".as_slice())
                );
                assert_eq!(dest.content_type_for(&target).as_deref(), Some("image/png"));
                assert_eq!(
                    src.part_bytes("word/media/source.opaque"),
                    Some(b"original image bytes".as_slice())
                );
                assert_eq!(
                    carry_relationship(&mut dest, dest_part, &src, source_part, &rid, |_| true),
                    Some(new_id)
                );
            }
        }
    }

    #[test]
    fn carried_external_links_and_missing_internal_targets_do_not_create_media() {
        let mut src = PartFs::open(PACKAGE).unwrap();
        let mut dest = PartFs::open(PACKAGE).unwrap();
        let before = dest.parts();
        for (ty, target, explicit) in [
            ("urn:part/hyperlink", "https://host/a?x=1&y=2", false),
            ("urn:part/attachment", "custom+scheme:payload", false),
            ("urn:part/attachment", "relative-but-external.bin", true),
        ] {
            let rid = if explicit {
                src.add_document_relationship_external("word/headerGap.xml", ty, target)
            } else {
                src.add_document_relationship("word/headerGap.xml", ty, target)
            };
            let copied = carry_relationship(
                &mut dest,
                "word/footerGap.xml",
                &src,
                "word/headerGap.xml",
                &rid,
                |_| true,
            )
            .unwrap();
            let rel = dest
                .read_rels_for("word/footerGap.xml")
                .unwrap()
                .items
                .iter()
                .find(|r| r.id == copied)
                .unwrap();
            assert_eq!(rel.target, target);
            assert_eq!(rel.target_mode.as_deref(), Some("External"));
            assert_eq!(dest.parts(), before);
        }
        let missing = src.add_document_relationship(
            "word/headerGap.xml",
            "urn:part/image",
            "media/missing.opaque",
        );
        assert_eq!(
            carry_relationship(
                &mut dest,
                "word/footerGap.xml",
                &src,
                "word/headerGap.xml",
                &missing,
                |_| true
            ),
            None
        );
        assert_eq!(
            carry_relationship(
                &mut dest,
                "word/footerGap.xml",
                &src,
                "word/headerGap.xml",
                &missing,
                |_| false
            ),
            None
        );
        assert_eq!(
            carry_relationship(
                &mut dest,
                "word/footerGap.xml",
                &src,
                "word/headerGap.xml",
                "absent",
                |_| true
            ),
            None
        );
        assert_eq!(
            carry_relationship(
                &mut dest,
                "word/footerGap.xml",
                &src,
                "word/noRelationships.xml",
                "absent",
                |_| true
            ),
            None
        );
        assert_eq!(dest.parts(), before);
    }

    #[test]
    fn markup_collision_names_reuse_bytes_without_overwriting_original_parts() {
        for source_name in [
            "word/headerGap.xml",
            "rootGap.xml",
            "word/noextension",
            "folder/with.dot/noextension",
        ] {
            let mut dest = PartFs::open(PACKAGE).unwrap();
            dest.set_part(source_name, b"A".to_vec());
            let first = dest_uri_for_reconciled_part(&dest, source_name, b"B");
            assert_ne!(first, source_name);
            dest.set_part(&first, b"C".to_vec());
            let next = dest_uri_for_reconciled_part(&dest, source_name, b"B");
            assert_ne!(first, next);
            dest.set_part(&next, b"B".to_vec());
            assert_eq!(dest_uri_for_reconciled_part(&dest, source_name, b"B"), next);
            assert_eq!(dest.part_bytes(source_name), Some(b"A".as_slice()));
            assert_eq!(dest.part_bytes(&first), Some(b"C".as_slice()));
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_recursive_part_ownership {
    use super::*;
    use crate::namespaces::R;
    const PACKAGE: &[u8] = include_bytes!("../../tests/fixtures/relids/image_doc.docx");
    const TY: &str = "urn:jandira:document/attachment";

    #[test]
    fn recursive_xml_parts_keep_each_relationship_local_and_each_source_unchanged() {
        for depth in [1usize, 2, 8, 9] {
            for collisions in [false, true] {
                let mut source = PartFs::open(PACKAGE).unwrap();
                let mut destination = PartFs::open(PACKAGE).unwrap();
                let owner = "customXml/owner.xml";
                for index in 1..=depth {
                    let part = format!("customXml/item{index}.xml");
                    let id = if index < depth {
                        Some(source.add_document_relationship(
                            &part,
                            TY,
                            &format!("item{}.xml", index + 1),
                        ))
                    } else {
                        None
                    };
                    let attr = id.map(|id| format!(" r:id=\"{id}\"")).unwrap_or_default();
                    let xml = format!(
                        "<item xmlns=\"urn:jandira:coverage\" xmlns:r=\"{}\" index=\"{index}\"{attr}>payload-{index}</item>",
                        R::URI
                    );
                    source.set_part(&part, xml.into_bytes());
                    source.add_content_type_override(&part, "application/xml");
                    if collisions {
                        destination
                            .set_part(&part, format!("<old index=\"{index}\"/>").into_bytes());
                        destination.add_content_type_override(&part, "application/xml");
                    }
                }
                let rid = source.add_document_relationship(owner, TY, "item1.xml");
                let owner_xml = format!(
                    "<owner xmlns:r=\"{}\" r:id=\"{rid}\"><keep>authored</keep></owner>",
                    R::URI
                );
                source.set_part(owner, owner_xml.as_bytes().to_vec());
                source.add_content_type_override(owner, "application/xml");
                destination.set_part(owner, b"<owner/>".to_vec());
                destination.add_content_type_override(owner, "application/xml");
                let original = source
                    .parts()
                    .into_iter()
                    .map(|part| {
                        let bytes = source.part_bytes(&part).unwrap().to_vec();
                        let content_type = source.content_type_for(&part);
                        (part, bytes, content_type)
                    })
                    .collect::<Vec<_>>();
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&format!(
                    "<owner xmlns:r=\"{}\" r:id=\"{rid}\"><keep>authored</keep></owner>",
                    R::URI
                ));
                let root = dom.root(doc).unwrap();
                carry_part_relationships(&mut destination, owner, &source, owner, &mut dom, root);
                assert_eq!(dom.value(root), "authored");
                let mut current_owner = owner.to_owned();
                let mut current_id = dom.attribute(root, &R::name("id")).unwrap().to_owned();
                for index in 1..=depth {
                    let row = destination
                        .read_rels_for(&current_owner)
                        .unwrap()
                        .items
                        .iter()
                        .find(|row| row.id == current_id)
                        .unwrap()
                        .clone();
                    assert_eq!(row.rel_type, TY);
                    assert_ne!(row.target_mode.as_deref(), Some("External"));
                    let target = destination.resolve_rel_target(&current_owner, &row.target);
                    assert_eq!(
                        destination.content_type_for(&target).as_deref(),
                        Some("application/xml")
                    );
                    if collisions {
                        assert_ne!(target, format!("customXml/item{index}.xml"));
                    }
                    let mut copied = Dom::new();
                    let copied_doc =
                        copied.parse_xdocument(&destination.part_string(&target).unwrap());
                    let item = copied.root(copied_doc).unwrap();
                    assert_eq!(
                        copied.attribute(item, &crate::xmllinq::XName::get("index", "")),
                        Some(index.to_string().as_str())
                    );
                    assert_eq!(copied.value(item), format!("payload-{index}"));
                    if index < depth {
                        current_id = copied.attribute(item, &R::name("id")).unwrap().to_owned();
                        current_owner = target;
                    } else {
                        assert!(copied.attribute(item, &R::name("id")).is_none());
                    }
                }
                assert_eq!(source.part_bytes(owner), Some(owner_xml.as_bytes()));
                for (part, bytes, content_type) in original {
                    assert_eq!(source.part_bytes(&part), Some(bytes.as_slice()));
                    assert_eq!(source.content_type_for(&part), content_type);
                }
                assert_eq!(source.read_rels_for(owner).unwrap().items.len(), 1);
                assert_eq!(source.read_rels_for(owner).unwrap().items[0].rel_type, TY);
                assert_eq!(
                    source
                        .read_rels_for(owner)
                        .unwrap()
                        .items
                        .iter()
                        .find(|row| row.id == rid)
                        .unwrap()
                        .target,
                    "item1.xml"
                );
                for index in 1..depth {
                    let part = format!("customXml/item{index}.xml");
                    let rels = source.read_rels_for(&part).unwrap();
                    assert_eq!(rels.items.len(), 1);
                    assert_eq!(rels.items[0].rel_type, TY);
                    assert_eq!(rels.items[0].target, format!("item{}.xml", index + 1));
                }
            }
        }
    }

    #[test]
    fn repeated_references_share_one_carried_local_relationship_without_merging_owners() {
        for attribute in ["id", "embed", "link"] {
            let mut source = PartFs::open(PACKAGE).unwrap();
            let mut destination = PartFs::open(PACKAGE).unwrap();
            let owner = "customXml/owner.xml";
            let target = "customXml/payload.xml";
            let payload =
                b"<payload xmlns='urn:jandira:coverage' owner='source'>authored payload</payload>";
            source.set_part(target, payload.to_vec());
            source.add_content_type_override(target, "application/xml");
            let source_id = source.add_document_relationship(owner, TY, "payload.xml");
            let xml = format!(
                "<owner xmlns:r='{}'><first r:{attribute}='{source_id}'>first owner</first><second r:{attribute}='{source_id}'>second owner</second></owner>",
                R::URI
            );
            source.set_part(owner, xml.as_bytes().to_vec());
            source.add_content_type_override(owner, "application/xml");
            destination.set_part(owner, b"<owner/>".to_vec());
            destination.add_content_type_override(owner, "application/xml");
            let mut dom = Dom::new();
            let document = dom.parse_xdocument(&xml);
            let root = dom.root(document).unwrap();
            carry_part_relationships(&mut destination, owner, &source, owner, &mut dom, root);
            let children = dom.elements(root, None);
            assert_eq!(children.len(), 2);
            assert_eq!(dom.name(children[0]).unwrap().local_name(), "first");
            assert_eq!(dom.name(children[1]).unwrap().local_name(), "second");
            assert_eq!(dom.value(children[0]), "first owner");
            assert_eq!(dom.value(children[1]), "second owner");
            let new_id = dom.attribute(children[0], &R::name(attribute)).unwrap();
            assert_eq!(
                dom.attribute(children[1], &R::name(attribute)),
                Some(new_id)
            );
            let relationships = destination.read_rels_for(owner).unwrap();
            assert_eq!(
                relationships.items.len(),
                1,
                "a repeated RID has one local carried target"
            );
            let relationship = &relationships.items[0];
            assert_eq!(relationship.id, new_id);
            assert_eq!(relationship.rel_type, TY);
            let carried = destination.resolve_rel_target(owner, &relationship.target);
            assert_eq!(destination.part_bytes(&carried), Some(payload.as_slice()));
            assert_eq!(
                destination.content_type_for(&carried).as_deref(),
                Some("application/xml")
            );
            assert_eq!(source.part_bytes(owner), Some(xml.as_bytes()));
            assert_eq!(source.part_bytes(target), Some(payload.as_slice()));
            assert_eq!(source.read_rels_for(owner).unwrap().items.len(), 1);
            assert_eq!(source.read_rels_for(owner).unwrap().items[0].id, source_id);
            assert_eq!(
                source.read_rels_for(owner).unwrap().items[0].target,
                "payload.xml"
            );
        }
    }

    #[test]
    fn relationship_prefix_gate_accepts_valid_unusual_prefixes_without_text_false_positives() {
        for prefix in ["r", "rel_1", "rel-x", "_relationship"] {
            for whitespace in [" ", "\t", "\n", "\r\n"] {
                assert!(has_relationship_attribute(&format!(
                    "<root xmlns:{prefix}=\"{}\"{whitespace}{prefix}:id=\"rId1\"/>",
                    R::URI
                )));
                assert!(!has_relationship_attribute(&format!(
                    "<root xmlns:{prefix}=\"{}\"><text>{prefix}:id</text></root>",
                    R::URI
                )));
            }
        }
    }
}
