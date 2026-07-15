//! M4.B — preprocess + block-level hashing. Port of `RemoveExistingPowerToolsMarkup`
//! (:5049), `TestForInvalidContent` (:5037), `CloneForStructureHash` (:5121),
//! the block hash-string builder (inside `HashBlockLevelContent` :867), and
//! (later tasks) `CloneBlockLevelContentForHashing`, `AddSha1HashToBlockLevelContent`,
//! `HashBlockLevelContent`, `PreProcessMarkup`.

use crate::namespaces::{A14, O, PT, R, VML, W, WP};
use crate::util::group_adjacent;
use crate::util::sha1::sha1_hex;
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
    let serialized = dom.serialize_element(clone);
    serialized.replacen(WML_DEFAULT_XMLNS, "", 1)
}

/// M4.B.2 — SHA-1 of the block hash string.
pub fn block_sha1(dom: &Dom, clone: NodeId) -> String {
    sha1_hex(&block_hash_string(dom, clone))
}

/// M4.B.3 — `CloneForStructureHash` (:5121): keep element name + attributes +
/// element nesting, drop ALL text/value nodes. Returns `None` for non-elements.
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
        let clone = clone_block_level_content_for_hashing(dom, d, true, settings, rel_hash);
        let sha = block_sha1(dom, clone);
        dom.set_attribute_value(d, &PT::sha1_hash(), Some(&sha));
        if (name == W::tbl() || name == W::tr())
            && let Some(sc) = clone_for_structure_hash(dom, clone)
        {
            let sha2 = block_sha1(dom, sc);
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
        let clone = clone_block_level_content_for_hashing(dom, b, true, settings, rel_hash);
        // M122: Word-visual correlated hashes ignore whitespace so spacing-
        // stamped related paragraphs (file_175) share a CorrelatedSHA1Hash.
        // Exact pt:SHA1Hash (add_sha1) stays space-sensitive → word LCS after
        // ProcessCorrelatedHashes Unknown pairing can still emit space inserts.
        if settings.merge_replaced_paragraphs {
            strip_whitespace_in_clone_text(dom, clone);
        }
        let sha = block_sha1(dom, clone);
        if let Some(u) = dom.attribute(b, &unid).map(|s| s.to_string())
            && let Some(&src) = source_by_unid.get(&u)
        {
            dom.set_attribute_value(src, &PT::correlated_sha1_hash(), Some(&sha));
        }
    }
    Ok(())
}
