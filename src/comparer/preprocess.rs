// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.B — preprocess + block-level hashing. Port of `RemoveExistingPowerToolsMarkup`
//! (:5049), `TestForInvalidContent` (:5037), `CloneForStructureHash` (:5121),
//! the block hash-string builder (inside `HashBlockLevelContent` :867), and
//! (later tasks) `CloneBlockLevelContentForHashing`, `AddSha1HashToBlockLevelContent`,
//! `HashBlockLevelContent`, `PreProcessMarkup`.

use crate::namespaces::{A14, O, PT, R, VML, W, WP};
use crate::util::group_adjacent;

use crate::xmllinq::{Dom, NodeId, XName};

use super::WmlComparerSettings;
use super::tables::{
    ATTRIBUTES_TO_TRIM_WHEN_CLONING, S_ELEMENTS_WITH_RELATIONSHIP_IDS,
    S_RELATIONSHIP_ATTRIBUTE_NAMES,
};

/// A resolver `rId -> Some(replacement)|None` used by the rel-id clone branch:
/// `Some(v)` replaces the attribute value (part content-hash / hyperlink URI /
/// external URI / "NULL Relationship"); `None` drops the attribute (xml part).
pub type RelHashResolver<'a> = dyn Fn(&str) -> Option<String> + 'a;

/// Default resolver when no OPC package is available: every rId is treated as
/// unresolvable → "NULL Relationship" (the TS fallback). Real part-hashing is
/// wired in M4.B.6/M4.I via a package-backed resolver.
pub fn null_rel_resolver(_r_id: &str) -> Option<String> {
    Some("NULL Relationship".to_string())
}

fn is_rsid_attr(name: &XName) -> bool {
    name.namespace_name() == W::URI
        && matches!(
            name.local_name(),
            "rsid"
                | "rsidDel"
                | "rsidP"
                | "rsidR"
                | "rsidRDefault"
                | "rsidRPr"
                | "rsidSect"
                | "rsidTr"
        )
}

fn is_pt(name: &XName) -> bool {
    name.namespace_name() == PT::URI
}

/// `w14:paraId` / `w14:textId` are volatile per-paragraph ids Word regenerates;
/// they must not affect content correlation (the golden correlates paragraphs
/// with differing paraIds → del=0 on pure-additive docs), so strip them from the
/// block hash.
fn is_volatile_para_attr(name: &XName) -> bool {
    name.namespace_name() == crate::namespaces::W14::URI
        && matches!(name.local_name(), "paraId" | "textId")
}

/// Whitespace-invariant form for **correlated** block hashes only (Word-visual).
/// Spacing-stamped variants (file_175×file_176) share letters but not exact
/// spaces; strip so ProcessCorrelatedHashes can pair them as Unknown and word
/// LCS can emit space inserts. Exact `pt:SHA1Hash` stays space-sensitive.
fn whitespace_invariant_for_hash(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn apply_text_transform(text: &str, settings: &WmlComparerSettings) -> String {
    let mut t = text.to_string();
    if settings.case_insensitive {
        t = t.to_uppercase();
    }
    if settings.conflate_breaking_and_nonbreaking_spaces {
        t = t.replace(' ', "\u{00A0}"); // space → NBSP (faithful, :5225/:5533)
    }
    t
}

/// Replace every text node under `root` with whitespace-stripped form.
fn strip_whitespace_in_clone_text(dom: &mut Dom, root: NodeId) {
    let nodes: Vec<NodeId> = dom.descendant_nodes(root);
    for n in nodes {
        if !dom.is_text(n) {
            continue;
        }
        let raw = dom.text_value(n).unwrap_or("").to_string();
        let stripped = whitespace_invariant_for_hash(&raw);
        if stripped != raw {
            dom.set_text_value(n, &stripped);
        }
    }
}

/// M4.B.4 — `CloneBlockLevelContentForHashing` (:5142): clone for hashing, then
/// strip every namespace-declaration attribute from the result subtree.
pub fn clone_block_level_content_for_hashing(
    dom: &mut Dom,
    node: NodeId,
    include_related_parts: bool,
    settings: &WmlComparerSettings,
    rel_hash: &RelHashResolver,
) -> NodeId {
    let cloned = clone_internal(dom, node, include_related_parts, settings, rel_hash);
    let root = cloned
        .into_iter()
        .next()
        .unwrap_or_else(|| dom.new_element(dom.name(node).unwrap_or_else(|| W::name("p"))));
    // remove all namespace-declaration attributes across the result
    // (index walk — no descendants_and_self / attributes() Vec per node)
    fn strip_nsdecls(dom: &mut Dom, id: NodeId) {
        if dom.is_element(id) {
            let n = dom.attr_count(id);
            let mut to_drop = Vec::new();
            for i in 0..n {
                let (name, _) = dom.attr_at(id, i);
                if dom.is_namespace_declaration(name) {
                    to_drop.push(name.clone());
                }
            }
            for a in &to_drop {
                dom.set_attribute_value(id, a, None);
            }
            let kids = dom.child_count(id);
            for i in 0..kids {
                let c = dom.child_at(id, i);
                strip_nsdecls(dom, c);
            }
        }
    }
    strip_nsdecls(dom, root);
    root
}

/// Build a new element named `name`, copying `src`'s attributes except those for
/// which `drop(name)` is true, then appending `children`.
///
/// DOM-ITER-02: walk attributes by index (no `attributes()` Vec alloc).
fn new_with_filtered_attrs(
    dom: &mut Dom,
    name: XName,
    src: NodeId,
    drop: impl Fn(&XName) -> bool,
    children: Vec<NodeId>,
) -> NodeId {
    let ne = dom.new_element(name);
    // Collect first: attr_at borrows dom; set_attribute_value needs &mut.
    let keep: Vec<(XName, String)> = (0..dom.attr_count(src))
        .filter_map(|i| {
            let (an, av) = dom.attr_at(src, i);
            if drop(an) {
                None
            } else {
                Some((an.clone(), av.to_string()))
            }
        })
        .collect();
    for (an, av) in &keep {
        dom.set_attribute_value(ne, an, Some(av));
    }
    for c in children {
        dom.add(ne, c);
    }
    ne
}

/// Recurse-clone all child nodes, flattening fragment results.
///
/// DOM-ITER-02: index walk (no `nodes()` Vec alloc per element).
fn clone_children(
    dom: &mut Dom,
    node: NodeId,
    include_related_parts: bool,
    settings: &WmlComparerSettings,
    rel_hash: &RelHashResolver,
) -> Vec<NodeId> {
    let mut out = Vec::new();
    let n = dom.child_count(node);
    for i in 0..n {
        let c = dom.child_at(node, i);
        out.extend(clone_internal(
            dom,
            c,
            include_related_parts,
            settings,
            rel_hash,
        ));
    }
    out
}

/// Port of `CloneBlockLevelContentForHashingInternal` (:5160). Returns 0..n nodes
/// (drop = empty, `w:r` = a fragment of single-child runs).
///
/// M-MOVE S1: `pt:PreDelete`-stamped elements (word-mode flattened
/// pre-existing deletions) get a salt attribute on the clone so their
/// block/structure hash can NEVER equal the hash of identical unstamped
/// (live) content — otherwise the LCS correlates them Equal and both the
/// deletion history and doc B's real insertions vanish (fresh-p4). Only
/// PreDelete: `pt:PreIns` carries REQUIRE Equal correlation with B's live
/// copy (D1 / m32 w18). Unstamped content is untouched (byte-identical hash).
fn clone_internal(
    dom: &mut Dom,
    node: NodeId,
    include_related_parts: bool,
    settings: &WmlComparerSettings,
    rel_hash: &RelHashResolver,
) -> Vec<NodeId> {
    let out = clone_internal_unsalted(dom, node, include_related_parts, settings, rel_hash);
    if dom.is_element(node)
        && dom.attribute(node, &PT::name("PreDelete")) == Some(super::PREDELETE_STAMP_ORIG)
    {
        for &c in &out {
            if dom.is_element(c) {
                dom.set_attribute_value(c, &PT::name("PreDeleteSalt"), Some("1"));
            }
        }
    }
    out
}

fn clone_internal_unsalted(
    dom: &mut Dom,
    node: NodeId,
    include_related_parts: bool,
    settings: &WmlComparerSettings,
    rel_hash: &RelHashResolver,
) -> Vec<NodeId> {
    if !dom.is_element(node) {
        // text-node transform (B.4a)
        if dom.is_text(node) {
            let raw = dom.text_value(node).unwrap_or("");
            let t =
                if settings.case_insensitive || settings.conflate_breaking_and_nonbreaking_spaces {
                    apply_text_transform(raw, settings)
                } else {
                    raw.to_string()
                };
            return vec![dom.new_text(&t)];
        }
        // comment/PI: still need a fresh leaf (no shared ownership)
        return vec![dom.clone_subtree(node)];
    }
    let name = dom.name(node).unwrap();

    // ── B.4a: drops + text/run/para ───────────────────────────────────────────
    if name == W::bookmark_start()
        || name == W::bookmark_end()
        || name == W::p_pr()
        || name == W::r_pr()
    {
        return vec![];
    }
    if name.namespace_name() == A14::URI {
        return vec![];
    }
    // footnote/endnote references → bare empty element (drops w:id). First branch
    // (:5180) shadows the dead :5491 branch.
    if name == W::name("footnoteReference") || name == W::name("endnoteReference") {
        return vec![dom.new_element(name)];
    }

    if name == W::p() {
        // clone children first
        let cloned_children = clone_children(dom, node, include_related_parts, settings, rel_hash);
        let element_children: Vec<NodeId> = cloned_children
            .into_iter()
            .filter(|&c| dom.is_element(c))
            .collect();
        // group adjacent runs that are a single w:t-run — salted (PreDelete)
        // and unsalted runs must never merge into one hash run, and the merge
        // must carry the salt (it rebuilds fresh elements).
        let salt_name = PT::name("PreDeleteSalt");
        let grouped = group_adjacent(element_children.iter().copied(), |&e| {
            if !is_single_t_run(dom, e) {
                0u8
            } else if dom.attribute(e, &salt_name).is_some() {
                2
            } else {
                1
            }
        });
        let new_p = new_with_filtered_attrs(
            dom,
            W::p(),
            node,
            |a| is_rsid_attr(a) || is_pt(a) || is_volatile_para_attr(a),
            vec![],
        );
        for (kind, group) in grouped {
            if kind != 0 {
                // ATOM-TEXT-01: borrow single-text-child leaves when grouping runs.
                let text: String = group
                    .iter()
                    .map(|&e| dom.value_str(e).into_owned())
                    .collect();
                let text = apply_text_transform(&text, settings);
                let r = dom.new_element(W::r());
                if kind == 2 {
                    dom.set_attribute_value(r, &salt_name, Some("1"));
                }
                let t = dom.new_element(W::t());
                dom.add_text(t, &text);
                dom.add(r, t);
                dom.add(new_p, r);
            } else {
                for e in group {
                    dom.add(new_p, e);
                }
            }
        }
        return vec![new_p];
    }

    if name == W::r() {
        // fragment: each non-rPr child element wrapped in its own fresh w:r
        // DOM-ITER-02: index walk (no elements() Vec).
        let mut runs = Vec::new();
        let n = dom.child_count(node);
        for i in 0..n {
            let rc = dom.child_at(node, i);
            if !dom.is_element(rc) {
                continue;
            }
            if dom.name(rc).unwrap() == W::r_pr() {
                continue;
            }
            let inner = clone_internal(dom, rc, include_related_parts, settings, rel_hash);
            let r = dom.new_element(W::r());
            for n in inner {
                dom.add(r, n);
            }
            runs.push(r);
        }
        return runs;
    }

    // ── B.4b: table cases ─────────────────────────────────────────────────────
    // DOM-ITER-02: filter child elements by local name without elements() Vec.
    if name == W::tbl() {
        let tr_name = W::tr();
        let children: Vec<NodeId> = element_children_named(dom, node, &tr_name)
            .into_iter()
            .flat_map(|tr| clone_internal(dom, tr, include_related_parts, settings, rel_hash))
            .collect();
        let tbl = dom.new_element(W::tbl());
        for c in children {
            dom.add(tbl, c);
        }
        return vec![tbl];
    }
    if name == W::tr() {
        let tc_name = W::tc();
        let children: Vec<NodeId> = element_children_named(dom, node, &tc_name)
            .into_iter()
            .flat_map(|tc| clone_internal(dom, tc, include_related_parts, settings, rel_hash))
            .collect();
        let tr = dom.new_element(W::tr());
        for c in children {
            dom.add(tr, c);
        }
        return vec![tr];
    }
    if name == W::tc() {
        let children =
            clone_children_elements(dom, node, include_related_parts, settings, rel_hash);
        let tc = dom.new_element(W::tc());
        for c in children {
            dom.add(tc, c);
        }
        return vec![tc];
    }
    if name == W::tc_pr() {
        let gs_name = W::grid_span();
        let children: Vec<NodeId> = element_children_named(dom, node, &gs_name)
            .into_iter()
            .flat_map(|gs| clone_internal(dom, gs, include_related_parts, settings, rel_hash))
            .collect();
        let tcpr = dom.new_element(W::tc_pr());
        for c in children {
            dom.add(tcpr, c);
        }
        return vec![tcpr];
    }
    if name == W::grid_span() {
        let val = dom.attribute(node, &W::val()).unwrap_or("").to_string();
        let gs = dom.new_element(W::grid_span());
        dom.set_attribute_value(gs, &XName::get("val", ""), Some(&val));
        return vec![gs];
    }
    if name == W::txbx_content() {
        let children =
            clone_children_elements(dom, node, include_related_parts, settings, rel_hash);
        let tb = dom.new_element(W::txbx_content());
        for c in children {
            dom.add(tb, c);
        }
        return vec![tb];
    }

    // ── B.4c: relationship-id branch ──────────────────────────────────────────
    if include_related_parts && S_ELEMENTS_WITH_RELATIONSHIP_IDS.contains(&name) {
        let ne = dom.new_element(name.clone());
        // DOM-ITER-02: snapshot attrs (index walk), then mutate.
        let attrs: Vec<(XName, String)> = (0..dom.attr_count(node))
            .map(|i| {
                let (an, av) = dom.attr_at(node, i);
                (an.clone(), av.to_string())
            })
            .collect();
        for (an, av) in &attrs {
            if is_pt(an) || ATTRIBUTES_TO_TRIM_WHEN_CLONING.contains(an) {
                continue;
            }
            if S_RELATIONSHIP_ATTRIBUTE_NAMES.contains(an) {
                match rel_hash(av) {
                    Some(v) => dom.set_attribute_value(ne, an, Some(&v)),
                    None => { /* xml part → drop attribute */ }
                }
            } else {
                dom.set_attribute_value(ne, an, Some(av));
            }
        }
        for c in clone_children(dom, node, include_related_parts, settings, rel_hash) {
            dom.add(ne, c);
        }
        return vec![ne];
    }

    // ── B.4d: VML / OLE / object / docPr / default ────────────────────────────
    if name == VML::name("shape") {
        let children = clone_children(dom, node, include_related_parts, settings, rel_hash);
        return vec![new_with_filtered_attrs(
            dom,
            name,
            node,
            |a| {
                is_pt(a)
                    || *a == XName::get("style", "")
                    || *a == XName::get("id", "")
                    || *a == XName::get("type", "")
            },
            children,
        )];
    }
    if name == O::name("OLEObject") {
        let children = clone_children(dom, node, include_related_parts, settings, rel_hash);
        return vec![new_with_filtered_attrs(
            dom,
            name,
            node,
            |a| is_pt(a) || *a == XName::get("ObjectID", "") || *a == R::name("id"),
            children,
        )];
    }
    if name == W::object() {
        let children = clone_children(dom, node, include_related_parts, settings, rel_hash);
        return vec![new_with_filtered_attrs(dom, name, node, is_pt, children)];
    }
    if name == WP::name("docPr") {
        let children = clone_children(dom, node, include_related_parts, settings, rel_hash);
        return vec![new_with_filtered_attrs(
            dom,
            name,
            node,
            |a| is_pt(a) || *a == XName::get("id", ""),
            children,
        )];
    }

    // default
    let children = clone_children(dom, node, include_related_parts, settings, rel_hash);
    vec![new_with_filtered_attrs(
        dom,
        name,
        node,
        |a| is_pt(a) || is_volatile_para_attr(a) || ATTRIBUTES_TO_TRIM_WHEN_CLONING.contains(a),
        children,
    )]
}

/// DOM-ITER-02: direct-child element ids (no `elements()` Vec).
fn element_children(dom: &Dom, node: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    let n = dom.child_count(node);
    for i in 0..n {
        let c = dom.child_at(node, i);
        if dom.is_element(c) {
            out.push(c);
        }
    }
    out
}

/// DOM-ITER-02: direct-child elements matching `filter` (no `elements()` Vec).
fn element_children_named(dom: &Dom, node: NodeId, filter: &XName) -> Vec<NodeId> {
    let mut out = Vec::new();
    let n = dom.child_count(node);
    for i in 0..n {
        let c = dom.child_at(node, i);
        if dom.is_element(c) && dom.name(c).as_ref() == Some(filter) {
            out.push(c);
        }
    }
    out
}

fn clone_children_elements(
    dom: &mut Dom,
    node: NodeId,
    include_related_parts: bool,
    settings: &WmlComparerSettings,
    rel_hash: &RelHashResolver,
) -> Vec<NodeId> {
    element_children(dom, node)
        .into_iter()
        .flat_map(|c| clone_internal(dom, c, include_related_parts, settings, rel_hash))
        .collect()
}

fn is_single_t_run(dom: &Dom, e: NodeId) -> bool {
    if dom.name(e) != Some(W::r()) {
        return false;
    }
    // DOM-ITER-02: count element children without allocating.
    let mut el_count = 0usize;
    let mut has_t = false;
    let n = dom.child_count(e);
    for i in 0..n {
        let c = dom.child_at(e, i);
        if !dom.is_element(c) {
            continue;
        }
        el_count += 1;
        if dom.name(c) == Some(W::t()) {
            has_t = true;
        }
    }
    el_count == 1 && has_t
}

const WML_DEFAULT_XMLNS: &str =
    " xmlns=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"";

/// M4.B.1 — `RemoveExistingPowerToolsMarkup` (:5049): drop every `pt:*` attribute
/// EXCEPT `pt:Unid`, across `root` and all descendants.
pub fn remove_existing_powertools_markup(dom: &mut Dom, root: NodeId) {
    let unid = PT::unid();
    // DOM-ITER-03: collect elements first (visit needs &Dom; mutate after).
    let mut els = Vec::new();
    dom.for_each_descendant_and_self(root, None, |el| els.push(el));
    for el in els {
        // DOM-ITER-02: index walk attrs (no attributes() Vec).
        let mut pt_attrs = Vec::new();
        let n = dom.attr_count(el);
        for i in 0..n {
            let (name, _) = dom.attr_at(el, i);
            if name.namespace_name() == PT::URI && *name != unid {
                pt_attrs.push(name.clone());
            }
        }
        for a in &pt_attrs {
            dom.set_attribute_value(el, a, None);
        }
    }
}

/// M4.B.1 — `TestForInvalidContent` (:5037): the *preprocess* guard, which rejects
/// only `w:altChunk`, `w:subDoc`, `w:contentPart` (a narrower set than the
/// atomizer's `VerifyNoInvalidContent`).
pub fn test_for_invalid_content(dom: &Dom, root: NodeId) -> Result<(), String> {
    let invalid = [
        W::name("altChunk"),
        W::name("subDoc"),
        W::name("contentPart"),
    ];
    // DOM-ITER-03: stop at first hit without allocating the full descendant list.
    let mut bad: Option<String> = None;
    dom.for_each_descendant_element(root, None, |d| {
        if bad.is_some() {
            return;
        }
        if let Some(name) = dom.name(d)
            && invalid.contains(&name)
        {
            bad = Some(format!("Document contains {}", name.local_name()));
        }
    });
    match bad {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// M4.B.2 — the block hash *string*: serialize the clone, then strip the single
/// re-emitted wordprocessingml default-xmlns declaration (`replacen(.., 1)` —
/// JS string `.replace` removes only the first occurrence; the wrapper has
/// already stripped all xmlns decls so exactly one is re-emitted on the root).
pub fn block_hash_string(dom: &Dom, clone: NodeId) -> String {
    let mut serialized = dom.serialize_element(clone);
    // In-place strip (same result as `replacen(.., 1)` without a second full alloc).
    if let Some(i) = serialized.find(WML_DEFAULT_XMLNS) {
        serialized.drain(i..i + WML_DEFAULT_XMLNS.len());
    }
    serialized
}

/// M4.B.2 — SHA-1 of the block hash string.
///
/// HASH-STREAM-01 lite: stream serialize into SHA-1 (no full XML `String` for
/// the digest path). Digest must match `sha1_hex(block_hash_string(...))`.
pub fn block_sha1(dom: &Dom, clone: NodeId) -> String {
    // HASH-STREAM-01 lite: stream serialize into SHA-1 (no full intermediate
    // hash string required beyond the serialize sink).
    dom.serialize_element_sha1_hex(clone)
}

/// HASH-STREAM-03/04: content SHA-1 of a **source** block node, preferring a
/// no-clone stream for simple `w:p` and simple `w:tbl`/`w:tr`.
/// Falls back to `clone_block_level_content_for_hashing` + [`block_sha1`].
///
/// When `correlated_ws` is true (Word-visual correlated hashes), text is
/// whitespace-stripped like `strip_whitespace_in_clone_text` after clone.
///
/// Digest-identical to the clone oracle for every accepted simple shape and
/// for all complex nodes (fallback path).
pub fn block_sha1_from_source(
    dom: &mut Dom,
    node: NodeId,
    include_related_parts: bool,
    settings: &WmlComparerSettings,
    rel_hash: &RelHashResolver,
    correlated_ws: bool,
) -> String {
    if let Some(hex) = try_stream_hash_simple_paragraph(dom, node, settings, correlated_ws) {
        return hex;
    }
    // HASH-STREAM-04: simple tables/rows stream content without hash-clone DOM.
    if let Some((content, _)) =
        try_stream_hash_simple_table_or_tr(dom, node, settings, correlated_ws)
    {
        return content;
    }
    let clone =
        clone_block_level_content_for_hashing(dom, node, include_related_parts, settings, rel_hash);
    if correlated_ws {
        strip_whitespace_in_clone_text(dom, clone);
    }
    block_sha1(dom, clone)
}

/// HASH-STREAM-03/05: stream-hash a simple paragraph without a hash-clone
/// subtree. Accepts `w:t` and empty leaf run children (`w:br`/`w:tab`/…):
/// multi-child runs expand like `clone_internal` r-fragments; adjacent text
/// fragments merge. Returns `None` for drawings, footnotes, nested content.
pub fn try_stream_hash_simple_paragraph(
    dom: &Dom,
    node: NodeId,
    settings: &WmlComparerSettings,
    correlated_ws: bool,
) -> Option<String> {
    if dom.name(node) != Some(W::p()) {
        return None;
    }
    // PreDelete salt changes clone attrs — stream only unsalted clean trees
    // (same gate as try_stream_hash_simple_table_or_tr). Without this, A-only
    // flattened pre-dels hash Equal to B's live copy and both history + B
    // inserts vanish (m36 S1 / fresh-p4).
    if element_or_desc_has_predelete_orig(dom, node) {
        return None;
    }
    let attr_xml = filtered_p_attr_xml(dom, node)?;
    let frags = collect_simple_p_fragments(dom, node, settings, correlated_ws)?;
    let mut xml = String::with_capacity(128);
    xml.push_str("<w:p xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"");
    xml.push_str(&attr_xml);
    if frags.is_empty() {
        xml.push_str(" />");
    } else {
        xml.push('>');
        emit_merged_run_fragments(&frags, &mut xml, /*structure_only*/ false);
        xml.push_str("</w:p>");
    }
    if let Some(i) = xml.find(WML_DEFAULT_XMLNS) {
        xml.drain(i..i + WML_DEFAULT_XMLNS.len());
    }
    Some(crate::util::sha1::sha1_hex(&xml))
}

/// Fragment after r-expansion: text (mergeable) or an empty leaf element.
enum RunFrag {
    Text(String),
    /// Self-closing `w:{local}` with pre-rendered attribute string (may be empty).
    Leaf {
        local: String,
        attrs: String,
    },
}

fn is_streamable_empty_run_leaf(local: &str) -> bool {
    matches!(
        local,
        "br" | "tab" | "cr" | "noBreakHyphen" | "softHyphen" | "lastRenderedPageBreak"
    )
}

/// Paragraph attrs surviving rsid/pt/volatile filter (w: or empty ns only).
fn filtered_p_attr_xml(dom: &Dom, node: NodeId) -> Option<String> {
    let mut attr_xml = String::new();
    for i in 0..dom.attr_count(node) {
        let (an, av) = dom.attr_at(node, i);
        if dom.is_namespace_declaration(an) {
            continue;
        }
        if is_rsid_attr(an) || is_pt(an) || is_volatile_para_attr(an) {
            continue;
        }
        let ns = an.namespace_name();
        let local = an.local_name();
        if ns == W::URI {
            attr_xml.push(' ');
            attr_xml.push_str("w:");
            attr_xml.push_str(local);
            attr_xml.push_str("=\"");
            attr_xml.push_str(&escape_xml_attr(av));
            attr_xml.push('"');
        } else if ns.is_empty() {
            attr_xml.push(' ');
            attr_xml.push_str(local);
            attr_xml.push_str("=\"");
            attr_xml.push_str(&escape_xml_attr(av));
            attr_xml.push('"');
        } else {
            return None;
        }
    }
    Some(attr_xml)
}

/// Expand runs under `w:p` to fragments; empty runs skipped; complex → None.
fn collect_simple_p_fragments(
    dom: &Dom,
    node: NodeId,
    settings: &WmlComparerSettings,
    correlated_ws: bool,
) -> Option<Vec<RunFrag>> {
    let mut frags: Vec<RunFrag> = Vec::new();
    let n = dom.child_count(node);
    for i in 0..n {
        let c = dom.child_at(node, i);
        if !dom.is_element(c) {
            continue;
        }
        let name = dom.name(c)?;
        if name == W::p_pr() || name == W::bookmark_start() || name == W::bookmark_end() {
            continue;
        }
        if name != W::r() {
            return None;
        }
        // Expand run: each non-rPr child is a fragment (clone_internal r path).
        let mut saw = false;
        for j in 0..dom.child_count(c) {
            let cc = dom.child_at(c, j);
            if !dom.is_element(cc) {
                continue;
            }
            let cn = dom.name(cc)?;
            if cn == W::r_pr() {
                continue;
            }
            if cn == W::t() {
                saw = true;
                let mut t = apply_text_transform(&dom.value_str(cc), settings);
                if correlated_ws {
                    t = whitespace_invariant_for_hash(&t);
                }
                frags.push(RunFrag::Text(t));
                continue;
            }
            // HASH-STREAM-05: empty leaf run children (br/tab/…).
            if cn.namespace_name() == W::URI
                && is_streamable_empty_run_leaf(cn.local_name())
                && !has_element_child(dom, cc)
            {
                let attrs = filtered_leaf_attr_xml(dom, cc)?;
                saw = true;
                frags.push(RunFrag::Leaf {
                    local: cn.local_name().to_string(),
                    attrs,
                });
                continue;
            }
            return None; // drawing, footnoteRef, nested content, …
        }
        let _ = saw; // empty run (only rPr) contributes nothing — like clone
    }
    Some(frags)
}

fn has_element_child(dom: &Dom, node: NodeId) -> bool {
    let n = dom.child_count(node);
    for i in 0..n {
        if dom.is_element(dom.child_at(node, i)) {
            return true;
        }
    }
    false
}

/// Attrs on empty leaf after clone's default filter (pt / volatile / trim set).
fn filtered_leaf_attr_xml(dom: &Dom, node: NodeId) -> Option<String> {
    let mut attr_xml = String::new();
    for i in 0..dom.attr_count(node) {
        let (an, av) = dom.attr_at(node, i);
        if dom.is_namespace_declaration(an) {
            continue;
        }
        if is_pt(an) || is_volatile_para_attr(an) || ATTRIBUTES_TO_TRIM_WHEN_CLONING.contains(an) {
            continue;
        }
        let ns = an.namespace_name();
        let local = an.local_name();
        if ns == W::URI {
            attr_xml.push(' ');
            attr_xml.push_str("w:");
            attr_xml.push_str(local);
            attr_xml.push_str("=\"");
            attr_xml.push_str(&escape_xml_attr(av));
            attr_xml.push('"');
        } else if ns.is_empty() {
            attr_xml.push(' ');
            attr_xml.push_str(local);
            attr_xml.push_str("=\"");
            attr_xml.push_str(&escape_xml_attr(av));
            attr_xml.push('"');
        } else {
            return None;
        }
    }
    Some(attr_xml)
}

/// Merge adjacent text fragments, emit `<w:r>…</w:r>` sequence into `out`.
fn emit_merged_run_fragments(frags: &[RunFrag], out: &mut String, structure_only: bool) {
    let mut i = 0;
    while i < frags.len() {
        match &frags[i] {
            RunFrag::Text(t0) => {
                let mut merged = t0.clone();
                i += 1;
                while i < frags.len() {
                    if let RunFrag::Text(t) = &frags[i] {
                        merged.push_str(t);
                        i += 1;
                    } else {
                        break;
                    }
                }
                // structure_only always drops text; empty merged also emits bare t.
                if structure_only || merged.is_empty() {
                    out.push_str("<w:r><w:t /></w:r>");
                } else {
                    out.push_str("<w:r><w:t>");
                    out.push_str(&escape_xml_text(&merged));
                    out.push_str("</w:t></w:r>");
                }
            }
            RunFrag::Leaf { local, attrs } => {
                out.push_str("<w:r><w:");
                out.push_str(local);
                out.push_str(attrs);
                out.push_str(" /></w:r>");
                i += 1;
            }
        }
    }
}

/// HASH-STREAM-04/06: stream content + structure digests for a simple `w:tbl`,
/// `w:tr`, or `w:tc` without materializing a hash-clone subtree.
///
/// Accepted shapes match `clone_internal` projection for tables:
/// - `w:tbl` keeps only `w:tr` children (drops `tblPr`/`tblGrid`/…)
/// - `w:tr` keeps only `w:tc` children (drops `trPr`)
/// - `w:tc` keeps element children: simple paragraphs, `tcPr` (gridSpan only)
/// - paragraphs only when streamable as simple-p (incl. empty leaf runs)
///
/// Returns `(content_sha1_hex, structure_sha1_hex)` digest-identical to
/// `block_sha1(clone)` / `structure_sha1(clone)`. For `w:tc`, structure is
/// still returned (callers that only need content may ignore it). `None` →
/// use clone fallback.
pub fn try_stream_hash_simple_table_or_tr(
    dom: &Dom,
    node: NodeId,
    settings: &WmlComparerSettings,
    correlated_ws: bool,
) -> Option<(String, String)> {
    let name = dom.name(node)?;
    if name != W::tbl() && name != W::tr() && name != W::tc() {
        return None;
    }
    // PreDelete salt changes clone attrs — stream only unsalted clean trees.
    if element_or_desc_has_predelete_orig(dom, node) {
        return None;
    }
    let mut content = String::with_capacity(256);
    let mut structure = String::with_capacity(256);
    if name == W::tbl() {
        emit_open_root(&mut content, "tbl");
        emit_open_root(&mut structure, "tbl");
        stream_tbl_body(
            dom,
            node,
            settings,
            correlated_ws,
            &mut content,
            &mut structure,
        )?;
        content.push_str("</w:tbl>");
        structure.push_str("</w:tbl>");
    } else if name == W::tr() {
        emit_open_root(&mut content, "tr");
        emit_open_root(&mut structure, "tr");
        stream_tr_body(
            dom,
            node,
            settings,
            correlated_ws,
            &mut content,
            &mut structure,
        )?;
        content.push_str("</w:tr>");
        structure.push_str("</w:tr>");
    } else {
        // HASH-STREAM-06: simple table cell root.
        emit_open_root(&mut content, "tc");
        emit_open_root(&mut structure, "tc");
        stream_tc_body(
            dom,
            node,
            settings,
            correlated_ws,
            &mut content,
            &mut structure,
        )?;
        content.push_str("</w:tc>");
        structure.push_str("</w:tc>");
    }
    // xmlns:w form never contains the default xmlns strip target; keep parity.
    if let Some(i) = content.find(WML_DEFAULT_XMLNS) {
        content.drain(i..i + WML_DEFAULT_XMLNS.len());
    }
    if let Some(i) = structure.find(WML_DEFAULT_XMLNS) {
        structure.drain(i..i + WML_DEFAULT_XMLNS.len());
    }
    Some((
        crate::util::sha1::sha1_hex(&content),
        crate::util::sha1::sha1_hex(&structure),
    ))
}

fn emit_open_root(out: &mut String, local: &str) {
    out.push_str("<w:");
    out.push_str(local);
    out.push_str(" xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">");
}

fn element_or_desc_has_predelete_orig(dom: &Dom, node: NodeId) -> bool {
    let pre = PT::name("PreDelete");
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if !dom.is_element(n) {
            continue;
        }
        if dom.attribute(n, &pre) == Some(super::PREDELETE_STAMP_ORIG) {
            return true;
        }
        let c = dom.child_count(n);
        for i in 0..c {
            stack.push(dom.child_at(n, i));
        }
    }
    false
}

/// Stream all `w:tr` children of a table (non-tr children ignored like clone).
fn stream_tbl_body(
    dom: &Dom,
    tbl: NodeId,
    settings: &WmlComparerSettings,
    correlated_ws: bool,
    content: &mut String,
    structure: &mut String,
) -> Option<()> {
    let tr_name = W::tr();
    let n = dom.child_count(tbl);
    for i in 0..n {
        let c = dom.child_at(tbl, i);
        if !dom.is_element(c) || dom.name(c).as_ref() != Some(&tr_name) {
            continue;
        }
        content.push_str("<w:tr>");
        structure.push_str("<w:tr>");
        stream_tr_body(dom, c, settings, correlated_ws, content, structure)?;
        content.push_str("</w:tr>");
        structure.push_str("</w:tr>");
    }
    Some(())
}

/// Stream all `w:tc` children of a row (non-tc children ignored like clone).
fn stream_tr_body(
    dom: &Dom,
    tr: NodeId,
    settings: &WmlComparerSettings,
    correlated_ws: bool,
    content: &mut String,
    structure: &mut String,
) -> Option<()> {
    let tc_name = W::tc();
    let n = dom.child_count(tr);
    for i in 0..n {
        let c = dom.child_at(tr, i);
        if !dom.is_element(c) || dom.name(c).as_ref() != Some(&tc_name) {
            continue;
        }
        content.push_str("<w:tc>");
        structure.push_str("<w:tc>");
        stream_tc_body(dom, c, settings, correlated_ws, content, structure)?;
        content.push_str("</w:tc>");
        structure.push_str("</w:tc>");
    }
    Some(())
}

/// Stream cell body: simple paragraphs + tcPr (gridSpan only). Bookmarks/pPr
/// drops mirror clone_internal. Anything else → None (fallback).
fn stream_tc_body(
    dom: &Dom,
    tc: NodeId,
    settings: &WmlComparerSettings,
    correlated_ws: bool,
    content: &mut String,
    structure: &mut String,
) -> Option<()> {
    let n = dom.child_count(tc);
    for i in 0..n {
        let c = dom.child_at(tc, i);
        if !dom.is_element(c) {
            continue;
        }
        let name = dom.name(c)?;
        // Drops identical to clone_internal for these tags.
        if name == W::bookmark_start()
            || name == W::bookmark_end()
            || name == W::p_pr()
            || name == W::r_pr()
        {
            continue;
        }
        if name.namespace_name() == A14::URI {
            continue;
        }
        if name == W::tc_pr() {
            stream_tc_pr(dom, c, content, structure);
            continue;
        }
        if name == W::p() {
            stream_simple_p_fragment(dom, c, settings, correlated_ws, content, structure)?;
            continue;
        }
        // Nested simple table under cell.
        if name == W::tbl() {
            content.push_str("<w:tbl>");
            structure.push_str("<w:tbl>");
            stream_tbl_body(dom, c, settings, correlated_ws, content, structure)?;
            content.push_str("</w:tbl>");
            structure.push_str("</w:tbl>");
            continue;
        }
        // Complex cell content (drawing, br-only via non-simple p, sdt, …).
        return None;
    }
    Some(())
}

/// `tcPr` clone keeps only `w:gridSpan` children; empty → `<w:tcPr />`.
/// gridSpan attrs: clone rewrites `w:val` → bare `val="..."`.
fn stream_tc_pr(dom: &Dom, tc_pr: NodeId, content: &mut String, structure: &mut String) {
    let gs_name = W::grid_span();
    let mut spans: Vec<String> = Vec::new();
    let n = dom.child_count(tc_pr);
    for i in 0..n {
        let c = dom.child_at(tc_pr, i);
        if !dom.is_element(c) || dom.name(c).as_ref() != Some(&gs_name) {
            continue;
        }
        let val = dom.attribute(c, &W::val()).unwrap_or("").to_string();
        spans.push(val);
    }
    if spans.is_empty() {
        content.push_str("<w:tcPr />");
        structure.push_str("<w:tcPr />");
        return;
    }
    content.push_str("<w:tcPr>");
    structure.push_str("<w:tcPr>");
    for val in &spans {
        // Clone uses empty-namespace `val` attribute (not w:val).
        let frag = format!("<w:gridSpan val=\"{}\" />", escape_xml_attr(val));
        content.push_str(&frag);
        structure.push_str(&frag);
    }
    content.push_str("</w:tcPr>");
    structure.push_str("</w:tcPr>");
}

/// Emit simple-paragraph projection as a fragment (no root xmlns).
/// Shared fragment rules with [`try_stream_hash_simple_paragraph`] (HASH-STREAM-05).
fn stream_simple_p_fragment(
    dom: &Dom,
    node: NodeId,
    settings: &WmlComparerSettings,
    correlated_ws: bool,
    content: &mut String,
    structure: &mut String,
) -> Option<()> {
    if dom.name(node) != Some(W::p()) {
        return None;
    }
    let attr_xml = filtered_p_attr_xml(dom, node)?;
    let frags = collect_simple_p_fragments(dom, node, settings, correlated_ws)?;
    if frags.is_empty() {
        content.push_str("<w:p");
        content.push_str(&attr_xml);
        content.push_str(" />");
        structure.push_str("<w:p");
        structure.push_str(&attr_xml);
        structure.push_str(" />");
        return Some(());
    }
    content.push_str("<w:p");
    content.push_str(&attr_xml);
    content.push('>');
    emit_merged_run_fragments(&frags, content, false);
    content.push_str("</w:p>");
    structure.push_str("<w:p");
    structure.push_str(&attr_xml);
    structure.push('>');
    emit_merged_run_fragments(&frags, structure, true);
    structure.push_str("</w:p>");
    Some(())
}

fn escape_xml_attr(s: &str) -> String {
    // Hot-path: most attr values need no escaping (CR #3642397970).
    if !s.bytes().any(|b| matches!(b, b'&' | b'<' | b'>' | b'"')) {
        return s.to_string();
    }
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_xml_text(s: &str) -> String {
    if !s.bytes().any(|b| matches!(b, b'&' | b'<' | b'>')) {
        return s.to_string();
    }
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// HASH-STREAM-02: SHA-1 of the structure projection of `node` (elements + attrs
/// only). Digest-identical to `block_sha1(clone_for_structure_hash(...))`.
pub fn structure_sha1(dom: &Dom, node: NodeId) -> String {
    dom.serialize_element_structure_sha1_hex(node)
}

/// M4.B.3 — `CloneForStructureHash` (:5121): keep element name + attributes +
/// element nesting, drop ALL text/value nodes. Returns `None` for non-elements.
/// Kept as the oracle for HASH-STREAM-02 tests; production uses [`structure_sha1`].
pub fn clone_for_structure_hash(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    if !dom.is_element(node) {
        return None;
    }
    let name = dom.name(node).unwrap();
    let ne = dom.new_element(name);
    // DOM-ITER-02: snapshot attrs then children by index (no attributes()/nodes() Vecs).
    let attrs: Vec<(XName, String)> = (0..dom.attr_count(node))
        .map(|i| {
            let (an, av) = dom.attr_at(node, i);
            (an.clone(), av.to_string())
        })
        .collect();
    for (an, av) in &attrs {
        dom.set_attribute_value(ne, an, Some(av));
    }
    let n = dom.child_count(node);
    for i in 0..n {
        let c = dom.child_at(node, i);
        if let Some(child) = clone_for_structure_hash(dom, c) {
            dom.add(ne, child);
        }
    }
    Some(ne)
}

use super::tables::ELEMENTS_TO_HAVE_SHA1;
use std::collections::HashMap;

/// M4.B.5 — `AddSha1HashToBlockLevelContent` (:5080): stamp `pt:SHA1Hash` on every
/// `ElementsToHaveSha1Hash` descendant; additionally `pt:StructureSHA1Hash` on
/// `w:tbl`/`w:tr` (computed from the structure-clone of the hashing-clone).
pub fn add_sha1_hash_to_block_level_content(
    dom: &mut Dom,
    content_parent: NodeId,
    settings: &WmlComparerSettings,
    rel_hash: &RelHashResolver,
) {
    // DOM-ITER-03: collect targets without a full descendants() Vec of every node.
    let mut targets = Vec::new();
    dom.for_each_descendant_element(content_parent, None, |d| {
        if dom
            .name(d)
            .is_some_and(|n| ELEMENTS_TO_HAVE_SHA1.contains(&n))
        {
            targets.push(d);
        }
    });
    for d in targets {
        let name = dom.name(d).unwrap();
        // HASH-STREAM-03: simple paragraphs skip hash-clone DOM materialization.
        if name == W::p() {
            // Content hash is space-sensitive (correlated_ws = false).
            let sha = block_sha1_from_source(dom, d, true, settings, rel_hash, false);
            dom.set_attribute_value(d, &PT::sha1_hash(), Some(&sha));
            continue;
        }
        // HASH-STREAM-04/06: simple tbl/tr/tc stream without clone.
        if (name == W::tbl() || name == W::tr() || name == W::tc())
            && let Some((sha, sha2)) = try_stream_hash_simple_table_or_tr(dom, d, settings, false)
        {
            dom.set_attribute_value(d, &PT::sha1_hash(), Some(&sha));
            // Structure digests only for tbl/tr (PowerTools ElementsToHaveSha1 extras).
            if name == W::tbl() || name == W::tr() {
                dom.set_attribute_value(d, &PT::structure_sha1_hash(), Some(&sha2));
            }
            continue;
        }
        let clone = clone_block_level_content_for_hashing(dom, d, true, settings, rel_hash);
        let sha = block_sha1(dom, clone);
        dom.set_attribute_value(d, &PT::sha1_hash(), Some(&sha));
        if name == W::tbl() || name == W::tr() {
            // HASH-STREAM-02: structure digest without allocating a structure-clone DOM.
            let sha2 = structure_sha1(dom, clone);
            dom.set_attribute_value(d, &PT::structure_sha1_hash(), Some(&sha2));
        }
    }
}

/// M4.B.6 — `HashBlockLevelContent` (:832): hash each block of the `after_proc`
/// projection and store it as `pt:CorrelatedSHA1Hash` back onto the corresponding
/// ORIGINAL `source` element (matched by surviving `pt:Unid`). Blocks whose Unid
/// is absent from `source` (coalesced away by accept/reject) get no hash.
/// Operates on `w:p`/`w:tbl`/`w:tr`. Returns `Err` on a duplicate Unid in source.
pub fn hash_block_level_content(
    dom: &mut Dom,
    source_root: NodeId,
    after_proc_root: NodeId,
    settings: &WmlComparerSettings,
    rel_hash: &RelHashResolver,
) -> Result<(), String> {
    let block = |n: &XName| *n == W::p() || *n == W::tbl() || *n == W::tr();
    let unid = PT::unid();

    // sourceUnidDict: Unid -> source element (duplicate Unid is an error).
    // DOM-ITER-03: non-allocating descendant walk.
    let mut source_by_unid: HashMap<String, NodeId> = HashMap::new();
    let mut dup: Option<String> = None;
    dom.for_each_descendant_element(source_root, None, |d| {
        if dup.is_some() {
            return;
        }
        if dom.name(d).is_some_and(|n| block(&n))
            && let Some(u) = dom.attribute(d, &unid)
            && source_by_unid.insert(u.to_string(), d).is_some()
        {
            dup = Some(u.to_string());
        }
    });
    if let Some(u) = dup {
        return Err(format!("duplicate Unid in source: {u}"));
    }

    let mut after_blocks = Vec::new();
    dom.for_each_descendant_element(after_proc_root, None, |d| {
        if dom.name(d).is_some_and(|n| block(&n)) {
            after_blocks.push(d);
        }
    });
    for b in after_blocks {
        // HASH-STREAM-03: simple paragraphs stream without clone; M122
        // whitespace-invariant form is applied inside the stream/fallback path.
        let sha = block_sha1_from_source(
            dom,
            b,
            true,
            settings,
            rel_hash,
            settings.merge_replaced_paragraphs,
        );
        if let Some(u) = dom.attribute(b, &unid).map(|s| s.to_string())
            && let Some(&src) = source_by_unid.get(&u)
        {
            dom.set_attribute_value(src, &PT::correlated_sha1_hash(), Some(&sha));
        }
    }
    Ok(())
}

#[cfg(test)]
mod escape_xml_tests {
    use super::{escape_xml_attr, escape_xml_text};

    #[test]
    fn plain_text_roundtrips_without_entities() {
        assert_eq!(escape_xml_text("hello world"), "hello world");
        assert_eq!(escape_xml_attr("id-42"), "id-42");
    }

    #[test]
    fn specials_are_escaped() {
        assert_eq!(escape_xml_text("a&b<c>d"), "a&amp;b&lt;c&gt;d");
        assert_eq!(escape_xml_attr(r#"say "hi""#), "say &quot;hi&quot;");
    }
}
