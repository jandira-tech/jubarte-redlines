// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! LINQ-to-XML-style mutable XML tree, ported from `lib/xml-linq.ts`.
//!
//! Used by every WML/OOXML part the engine reads or writes (the comparer,
//! edit pipeline, accept/reject, markdown projection, etc.). The tree is an
//! arena (`Dom`) of nodes addressed by `NodeId` handles, so in-place
//! mutation is borrow-checker-friendly. Names are interned through a
//! thread-local pool so identical namespace / local-name strings — which
//! recur on millions of elements — share one `Arc<str>` allocation.
//!
//! # Layout
//!
//! - `XNamespace` / `XName` — interned expanded names; equal strings
//!   share storage and are byte-for-byte identical to a fresh `XName`/`XNamespace`.
//! - `NodeId` — handle into the arena.
//! - `XDeclaration` — `<?xml version encoding standalone?>` payload.
//! - `Dom` — the arena; the `parse` / `serialize` entry points are exposed
//!   both as free functions in `parse` / `serialize` and as `Dom` methods
//!   for ergonomics.
//!
//! # CLI
//!
//! The arena backs every XML part in the comparison package and the parts the
//! comparer writes. Nothing in this module is CLI-specific, but
//! `jubarte debug` and the validation passes read trees through it.
//!
//! # Submodules
//!
//! - `parse` — string → DOM (a small, non-validating, namespace-aware
//!   recursive-descent parser; `parse_xdocument` plus `validate_xml`).
//! - `serialize` — DOM → string, with scope-aware namespace prefixes.

pub mod parse;
pub mod serialize;

pub use parse::parse_xdocument;
pub use serialize::{
    serialize_document, serialize_element, serialize_element_sha1_hex,
    serialize_element_structure_sha1_hex,
};

use std::sync::Arc;

thread_local! {
    /// PARSE-01: per-thread pool of interned name strings. `XName`/`XNamespace`
    /// constructors route their `Arc<str>` through this so identical namespace and
    /// local-name strings — which recur on millions of elements/attributes —
    /// share one allocation instead of a fresh `Arc::from` each time. The engine
    /// is single-threaded, so a thread-local pool needs no lock; OOXML's name
    /// vocabulary is bounded, so the pool saturates quickly rather than growing
    /// without bound.
    ///
    /// Hashed with foldhash: a hand-rolled FNV-1a measured a +16% wall
    /// regression — its weak low-bit avalanche clusters under the low-bit
    /// bucket masking for these short, similar names — while SipHash was 11%
    /// of a one-page conversion (samply, fixtures_500 0081ba58). foldhash
    /// mixes the low bits and costs a fraction of SipHash; fixed seed, so
    /// runs stay reproducible.
    static STR_POOL: std::cell::RefCell<std::collections::HashSet<Arc<str>, foldhash::fast::FixedState>> =
        std::cell::RefCell::new(std::collections::HashSet::with_hasher(foldhash::fast::FixedState::default()));
}

/// Cap the intern pool so a pathological host (adversarial inputs) cannot grow
/// the thread-local set without bound (gemini #3584751169, CR #3584798246).
/// OOXML name vocabulary is far smaller; beyond the cap we still return a
/// correct `Arc<str>`, just without interning.
const STR_POOL_MAX: usize = 16_384;

/// Return a shared interned `Arc<str>` for `s`, allocating only on first sight.
/// Equality/hash of the result are byte-for-byte identical to a fresh `Arc::from`;
/// only storage is shared, so interning is behavior-preserving.
fn intern_str(s: &str) -> Arc<str> {
    STR_POOL.with(|pool| {
        {
            // Scope the shared borrow so it is released before `borrow_mut` below.
            let p = pool.borrow();
            if let Some(existing) = p.get(s) {
                return existing.clone();
            }
            if p.len() >= STR_POOL_MAX {
                return Arc::from(s);
            }
        }
        let arc: Arc<str> = Arc::from(s);
        pool.borrow_mut().insert(arc.clone());
        arc
    })
}

/// An XML namespace, identified by its URI string, owned through a
/// thread-local-interned `Arc<str>`. Port of the LINQ-to-XML `XNamespace`.
///
/// Equal strings share storage; semantics of `Eq`,
/// `Hash`, and the borrowed URI view are byte-for-byte identical to a fresh
/// `XNamespace::get(uri)`, only the backing allocation is shared.
#[derive(Clone)]
pub struct XNamespace {
    name: Arc<str>,
}

impl XNamespace {
    /// Construct a namespace, interning `namespace_name` for sharing across
    /// the thread (a no-op once the pool is saturated). The empty URI is the
    /// "no namespace" namespace, equivalent to LINQ-to-XML's `XNamespace.None`.
    pub fn get(namespace_name: &str) -> XNamespace {
        XNamespace {
            name: intern_str(namespace_name),
        }
    }

    /// The empty namespace (no URI). Equal to `XNamespace::get("")`;
    /// the XML namespace in OOXML for unqualified elements.
    pub fn none() -> XNamespace {
        XNamespace::get("")
    }

    /// The reserved `xmlns:` declarations namespace
    /// (`http://www.w3.org/2000/xmlns/`).
    pub fn xmlns() -> XNamespace {
        XNamespace::get("http://www.w3.org/2000/xmlns/")
    }

    /// The reserved `xml:` declarations namespace
    /// (`http://www.w3.org/XML/1998/namespace`).
    pub fn xml() -> XNamespace {
        XNamespace::get("http://www.w3.org/XML/1998/namespace")
    }

    /// Build the [`XName`] `{ns}local` — i.e. an expanded name in this
    /// namespace. Equivalent to LINQ-to-XML's `XNamespace.GetName(local)`.
    pub fn name(&self, local: &str) -> XName {
        XName::get(local, &self.name)
    }

    /// The namespace URI as a borrowed string. Empty for the no-namespace
    /// namespace.
    pub fn namespace_name(&self) -> &str {
        &self.name
    }
}

impl PartialEq for XNamespace {
    fn eq(&self, other: &Self) -> bool {
        // Interned names share an `Arc`, so pointer identity is a fast path for the
        // common equal case; the content compare keeps correctness for any
        // non-interned or pre-warm name. Result is identical to a pure content
        // compare, so `Hash` (content-based) stays consistent.
        Arc::ptr_eq(&self.name, &other.name) || self.name.as_ref() == other.name.as_ref()
    }
}
impl Eq for XNamespace {}
impl std::hash::Hash for XNamespace {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.as_ref().hash(state);
    }
}
impl std::fmt::Debug for XNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XNamespace({:?})", self.name.as_ref())
    }
}

/// An expanded XML name: a local name plus its namespace. Both halves are
/// interned so equal strings share one `Arc<str>` allocation. Port of the
/// LINQ-to-XML `XName`.
#[derive(Clone)]
pub struct XName {
    local: Arc<str>,
    namespace: XNamespace,
}

impl XName {
    /// Construct an expanded name in the given namespace. `local_name` and
    /// `namespace_name` are interned through the thread-local pool
    /// (a no-op once the pool saturates). Pass `""` for the empty
    /// (no-namespace) namespace.
    pub fn get(local_name: &str, namespace_name: &str) -> XName {
        XName {
            local: intern_str(local_name),
            namespace: XNamespace::get(namespace_name),
        }
    }

    /// Parse clark notation: `"{ns}local"` becomes a qualified [`XName`], a
    /// bare `"local"` becomes one in the empty namespace. Equivalent to
    /// LINQ-to-XML's `XName.Get("expanded-name")`.
    ///
    /// # Panics
    ///
    /// Panics when the string starts with `{` and has no closing `}`. An
    /// empty namespace (`{}local`) or local name (`{ns}`) is accepted.
    pub fn from_clark(expanded: &str) -> XName {
        if expanded.starts_with('{') {
            let close = expanded
                .find('}')
                .filter(|&i| i > 0)
                .unwrap_or_else(|| panic!("Invalid expanded name: {expanded}"));
            XName::get(&expanded[close + 1..], &expanded[1..close])
        } else {
            XName::get(expanded, "")
        }
    }

    /// The local part of the name, as a borrowed string.
    pub fn local_name(&self) -> &str {
        &self.local
    }

    /// The namespace half of the expanded name.
    pub fn namespace(&self) -> &XNamespace {
        &self.namespace
    }

    /// The namespace URI as a borrowed string. Empty for the no-namespace
    /// namespace.
    pub fn namespace_name(&self) -> &str {
        self.namespace.namespace_name()
    }

    /// The clark-notation form: `"{ns}local"` for a non-empty namespace,
    /// or just `"local"` for the empty one. Equivalent to LINQ-to-XML's
    /// `XName.ToString()`.
    pub fn clark(&self) -> String {
        if self.namespace.name.is_empty() {
            self.local.to_string()
        } else {
            format!("{{{}}}{}", self.namespace.name, self.local)
        }
    }
}

impl PartialEq for XName {
    fn eq(&self, other: &Self) -> bool {
        // Pointer-identity fast path for interned local names (see `XNamespace`);
        // falls back to a content compare, so equality/hash semantics are unchanged.
        (Arc::ptr_eq(&self.local, &other.local) || self.local.as_ref() == other.local.as_ref())
            && self.namespace == other.namespace
    }
}
impl Eq for XName {}
impl std::hash::Hash for XName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.local.as_ref().hash(state);
        self.namespace.hash(state);
    }
}
impl std::fmt::Debug for XName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XName({:?})", self.clark())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// M1.2: arena DOM — port of XObject/XAttribute/XNode/XText/XComment/
// XProcessingInstruction/XContainer/XElement/XDocument.
//
// The TS uses a pointer tree; Rust uses an arena (`Dom`) with `NodeId` handles so
// in-place mutation is borrow-checker-friendly. Children live in an ordered
// `content` vec (all node kinds); attributes are a separate per-element vec
// (attributes are XObjects but NOT XNodes, exactly as in LINQ-to-XML).
// ─────────────────────────────────────────────────────────────────────────────

use std::any::Any;
use std::collections::HashMap;

// N01 mechanism counter (tests only): attribute-name comparisons the DOM
// performs (bulk index lookups count once per incoming name). Incremental `set_attribute_value` insertion pays one Vec scan
// per name (quadratic in a tag's width); the parser's bulk
// `set_attributes` probes a hash index instead. Per thread, so tests
// running side by side in one `cargo test` process do not count each
// other's work.
#[cfg(test)]
thread_local! {
    pub(crate) static ATTR_NAME_PROBES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Add one to this thread's [`ATTR_NAME_PROBES`].
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
fn bump_attr_name_probes() {
    ATTR_NAME_PROBES.with(|c| c.set(c.get() + 1));
}

/// Handle into a [`Dom`] arena: the slot index of a node, plus a `Debug` /
/// `Copy` so it can travel cheaply through hot loops (compare, atomize).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(#[doc = "Slot index of the node inside the arena `Vec`."] pub u32);

/// An XML declaration (`<?xml version encoding standalone?>`). Port of
/// LINQ-to-XML's `XDeclaration`; the parser populates [`Self::version`] from
/// the declaration body, [`Self::encoding`] and [`Self::standalone`] from the
/// matching pseudo-attributes when present.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct XDeclaration {
    /// The `version` attribute (defaults to `"1.0"` when the parser sees a
    /// declaration that omits or misquotes the value).
    pub version: Option<String>,
    /// The `encoding` attribute, if the declaration carried one.
    pub encoding: Option<String>,
    /// The `standalone` attribute, if the declaration carried one.
    pub standalone: Option<String>,
}

/// An attribute (name + value). Port of `XAttribute`.
#[derive(Clone, Debug)]
struct Attr {
    name: XName,
    value: String,
}

/// Processing-instruction payload. Boxed inside `NodeKind::Pi` so the rare PI
/// variant does not size every node (NODE-KIND-01).
#[derive(Clone, Debug)]
struct PiData {
    target: String,
    data: String,
}

/// Node kind discriminant + kind-specific data.
///
/// The two large-but-rare variants are boxed (NODE-KIND-01): `Pi` (uncommon in
/// docx) and `Document`'s declaration (one node per document). This keeps the
/// enum sized by the common `Element { name: XName }` variant (~32 B) instead of
/// the ~72 B `Document` declaration, shrinking every arena node.
enum NodeKind {
    Element {
        name: XName,
    },
    Text(String),
    Comment(String),
    Pi(Box<PiData>),
    Document {
        declaration: Option<Box<XDeclaration>>,
    },
}

struct NodeData {
    kind: NodeKind,
    parent: Option<NodeId>,
    /// Ordered child nodes (containers only; empty for leaves).
    content: Vec<NodeId>,
    /// Attributes (elements only).
    attrs: Vec<Attr>,
}

impl NodeData {
    fn new(kind: NodeKind) -> Self {
        NodeData {
            kind,
            parent: None,
            content: Vec::new(),
            attrs: Vec::new(),
        }
    }
}

/// The arena holding every XML node the engine constructs while comparing or
/// editing a part. Port of the LINQ-to-XML object graph; the TS pointer tree
/// becomes a `Vec<NodeData>` indexed by [`NodeId`] so in-place edits are
/// borrow-checker-friendly.
///
/// Children of a node live in an ordered `content` vec (all node kinds);
/// attributes are a separate per-element vec (attributes are XObjects but
/// not XNodes, exactly as in LINQ-to-XML).
///
/// Most callers should not allocate `Dom` directly — `WmlDocument` and the
/// per-part helpers in [`crate::opc`] own one. The unit-struct tests at the
/// bottom of the file (`node_data_excludes_annotations_vec`,
/// `node_kind_rare_variants_are_boxed`) pin the in-arena byte counts so a
/// future refactor cannot silently regress the hot per-node struct.
#[derive(Default)]
pub struct Dom {
    nodes: Vec<NodeData>,
    /// Typed annotations, keyed by `NodeId`. Kept off `NodeData` because no
    /// production path stores annotations (only the M1 foundation test), so the
    /// common case is an empty map and every node avoids a 24-byte inline `Vec`.
    annotations: HashMap<NodeId, Vec<Box<dyn Any>>>,
}

impl Dom {
    /// Construct an empty arena. Constructors ([`Self::new_document`],
    /// [`Self::new_element`], …) and the parser entry points allocate into
    /// the arena on demand.
    pub fn new() -> Self {
        Dom {
            nodes: Vec::new(),
            annotations: HashMap::new(),
        }
    }

    fn alloc(&mut self, kind: NodeKind) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(NodeData::new(kind));
        id
    }

    /// Number of nodes currently in the arena. Useful for diagnostics and
    /// for the scratch-scope checks in [`Self::with_scratch`].
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Allocated node-slot capacity of the arena's backing `Vec`.
    ///
    /// [`with_scratch`](Self::with_scratch) reclaims length but not capacity,
    /// so a push against a full arena reallocates the whole buffer to the
    /// next doubling tier and pins it. Tests that want to prove a scratch
    /// build stays inside a dedicated `Dom` compare this against
    /// [`Self::node_count`] (MEM-ATTRIBUTE-01).
    pub fn node_capacity(&self) -> usize {
        self.nodes.capacity()
    }

    /// Shrink the arena's backing `Vec` capacity to its current length so
    /// any subsequent internal push provably reallocates. Test-only helper
    /// that pins `capacity == length` for the MEM-ATTRIBUTE-01 invariant.
    #[cfg(test)]
    #[cfg_attr(all(coverage_nightly, test), coverage(off))]
    pub fn shrink_arena_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
    }

    /// Run `f` and then reclaim every node it allocated by truncating the arena
    /// back to its pre-call length.
    ///
    /// This is the scratch-scope: for work whose output does not reference
    /// the scratch nodes (e.g. building a normalized element only to
    /// serialize it into a `String`). The nodes `f` creates are appended at
    /// the arena's end and dropped on return, freeing their inline child /
    /// attribute `Vec`s and names. Pre-existing nodes keep stable ids.
    ///
    /// # When to use it
    ///
    /// Per-atom format-change normalization builds thousands of distinct
    /// `w:rPr` clones on run-fragmented documents. Without a scratch scope
    /// those accumulate into millions of throwaway nodes and balloon the
    /// arena (MEM-ATTRIBUTE-01: the top single allocation, a multi-GB arena
    /// `Vec`, came from exactly that leak).
    ///
    /// # Panics
    ///
    /// `NodeId`s created inside `f` MUST NOT escape via the return value `R`
    /// — they would dangle or, once the slot is reused, alias a different
    /// node. Borrow pre-existing ids freely; only newly allocated ones are
    /// reclaimed.
    pub fn with_scratch<R>(&mut self, f: impl FnOnce(&mut Dom) -> R) -> R {
        let checkpoint = self.nodes.len();
        let out = f(self);
        self.nodes.truncate(checkpoint);
        // A reused NodeId slot must not inherit a stale annotation. Production
        // never annotates (the map is empty), so this is a no-op there.
        if !self.annotations.is_empty() {
            self.annotations.retain(|k, _| (k.0 as usize) < checkpoint);
        }
        out
    }

    fn data(&self, id: NodeId) -> &NodeData {
        &self.nodes[id.0 as usize]
    }
    fn data_mut(&mut self, id: NodeId) -> &mut NodeData {
        &mut self.nodes[id.0 as usize]
    }

    // ── constructors ──────────────────────────────────────────────────────────
    /// Allocate a Document (root container) node, with no declaration. The
    /// parser ([`Self::parse_xdocument`]) is the usual way to build one;
    /// this constructor is for hand-built DOMs.
    pub fn new_document(&mut self) -> NodeId {
        self.alloc(NodeKind::Document { declaration: None })
    }
    /// Allocate an element node with the given expanded name. No children,
    /// no attributes — use [`Self::add`] / [`Self::set_attribute_value`] to
    /// populate it.
    pub fn new_element(&mut self, name: XName) -> NodeId {
        self.alloc(NodeKind::Element { name })
    }
    /// Allocate a text node carrying `value`. The text is owned as a
    /// `String`; use [`Self::set_text_value`] to overwrite it later.
    pub fn new_text(&mut self, value: &str) -> NodeId {
        self.alloc(NodeKind::Text(value.to_string()))
    }
    /// Allocate a comment node (`<!-- value -->`).
    pub fn new_comment(&mut self, value: &str) -> NodeId {
        self.alloc(NodeKind::Comment(value.to_string()))
    }
    /// Allocate a processing instruction node (`<?target data?>`). The PI
    /// payload is boxed so it does not size every node (NODE-KIND-01).
    pub fn new_pi(&mut self, target: &str, data: &str) -> NodeId {
        self.alloc(NodeKind::Pi(Box::new(PiData {
            target: target.to_string(),
            data: data.to_string(),
        })))
    }

    // ── kind predicates / accessors ────────────────────────────────────────────
    /// True if `id` is an element node. Cheap; the kind discriminant is in
    /// the arena slot.
    pub fn is_element(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Element { .. })
    }
    /// True if `id` is a text node.
    pub fn is_text(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Text(_))
    }
    /// True if `id` is the document container (allocation created with
    /// [`Self::new_document`] or via [`Self::parse_xdocument`]).
    pub fn is_document(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Document { .. })
    }
    /// True if `id` is a comment node.
    pub fn is_comment(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Comment(_))
    }
    /// True if `id` is a processing-instruction node.
    pub fn is_pi(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Pi(_))
    }

    /// The element's expanded name, or `None` for non-element nodes. The
    /// returned [`XName`] is a clone of the stored one (two `Arc` bumps).
    /// For a zero-copy test against a known name, prefer [`Self::name_is`].
    pub fn name(&self, id: NodeId) -> Option<XName> {
        match &self.data(id).kind {
            NodeKind::Element { name } => Some(name.clone()),
            _ => None,
        }
    }
    /// True if `id` is an element whose name equals `name`. No allocations
    /// and no `Arc` bumps (DOM-NAME-REF-01: this is the predicate the hot
    /// comparer loops use; [`Self::name`] clones).
    pub fn name_is(&self, id: NodeId, name: &XName) -> bool {
        matches!(&self.data(id).kind, NodeKind::Element { name: n } if n == name)
    }
    /// Set the element's expanded name. A no-op for non-element nodes.
    pub fn set_name(&mut self, id: NodeId, new_name: XName) {
        if let NodeKind::Element { name } = &mut self.data_mut(id).kind {
            *name = new_name;
        }
    }

    /// The payload of a text or comment node as a borrowed string, or `None`
    /// for any other node kind. The borrow lifetime is tied to `self`.
    pub fn text_value(&self, id: NodeId) -> Option<&str> {
        match &self.data(id).kind {
            NodeKind::Text(v) | NodeKind::Comment(v) => Some(v),
            _ => None,
        }
    }
    /// Replace the payload of a text or comment node. A no-op for any
    /// other node kind.
    pub fn set_text_value(&mut self, id: NodeId, value: &str) {
        match &mut self.data_mut(id).kind {
            NodeKind::Text(v) | NodeKind::Comment(v) => *v = value.to_string(),
            _ => {}
        }
    }

    /// The PI target (the part before the first whitespace inside `<?target?>`),
    /// or `None` for non-PI nodes.
    pub fn pi_target(&self, id: NodeId) -> Option<&str> {
        match &self.data(id).kind {
            NodeKind::Pi(pi) => Some(&pi.target),
            _ => None,
        }
    }
    /// The PI data (everything between the target and `?>`), or `None` for
    /// non-PI nodes.
    pub fn pi_data(&self, id: NodeId) -> Option<&str> {
        match &self.data(id).kind {
            NodeKind::Pi(pi) => Some(&pi.data),
            _ => None,
        }
    }

    /// The XML declaration attached to a Document node, or `None` for any
    /// other node kind. The reference lifetime is tied to `self`.
    pub fn declaration(&self, id: NodeId) -> Option<&XDeclaration> {
        match &self.data(id).kind {
            NodeKind::Document { declaration } => declaration.as_deref(),
            _ => None,
        }
    }
    /// Attach or clear the XML declaration on a Document node. A no-op for
    /// any other node kind. Passing `None` removes the declaration.
    pub fn set_declaration(&mut self, id: NodeId, decl: Option<XDeclaration>) {
        if let NodeKind::Document { declaration } = &mut self.data_mut(id).kind {
            *declaration = decl.map(Box::new);
        }
    }

    // ── tree navigation ────────────────────────────────────────────────────────
    /// The parent of `id`, or `None` for the document root.
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.data(id).parent
    }

    /// Direct children in document order, all node kinds (a clone of the
    /// internal content vec). For non-allocating iteration prefer
    /// [`Self::child_count`] + [`Self::child_at`].
    pub fn nodes(&self, id: NodeId) -> Vec<NodeId> {
        self.data(id).content.clone()
    }

    /// The first direct child of `id`, or `None` if it has none.
    pub fn first_node(&self, id: NodeId) -> Option<NodeId> {
        self.data(id).content.first().copied()
    }
    /// The last direct child of `id`, or `None` if it has none.
    pub fn last_node(&self, id: NodeId) -> Option<NodeId> {
        self.data(id).content.last().copied()
    }

    /// Number of direct children of `id` (all node kinds). O(1) and
    /// allocation-free; pair with [`Self::child_at`] for non-allocating
    /// child iteration on hot paths where [`Self::elements`]' per-call
    /// `Vec` is the cost (atomize). Stable while the caller does not
    /// add or remove children of `id`.
    pub fn child_count(&self, id: NodeId) -> usize {
        self.data(id).content.len()
    }

    /// The `i`-th direct child of `id` (all node kinds).
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds for `id`'s child count. See
    /// [`Self::child_count`].
    pub fn child_at(&self, id: NodeId, i: usize) -> NodeId {
        self.data(id).content[i]
    }

    /// Direct child elements, optionally filtered by expanded name.
    /// `filter == None` returns every direct child element in document
    /// order. For non-allocating iteration, walk [`Self::child_at`]
    /// yourself and call [`Self::is_element`] + [`Self::name_is`].
    pub fn elements(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId> {
        self.data(id)
            .content
            .iter()
            .copied()
            .filter(|&c| match (&self.data(c).kind, filter) {
                (NodeKind::Element { name }, Some(f)) => name == f,
                (NodeKind::Element { .. }, None) => true,
                _ => false,
            })
            .collect()
    }

    /// The first direct child element whose name matches `filter`, or
    /// `None`. Search order is document order.
    pub fn element(&self, id: NodeId, filter: &XName) -> Option<NodeId> {
        self.data(id)
            .content
            .iter()
            .copied()
            .find(|&c| matches!(&self.data(c).kind, NodeKind::Element { name } if name == filter))
    }

    /// Every descendant element of `id`, in pre-order. `filter == None`
    /// visits every element; a name restricts which ones are yielded.
    /// Recursion still enters every element child (LINQ-to-XML
    /// `Descendants` semantics — the filter selects what is yielded, not
    /// where the walk enters).
    pub fn descendants(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId> {
        let mut out = Vec::new();
        self.for_each_descendant_element(id, filter, |c| out.push(c));
        out
    }

    /// Visit every descendant element of `id` in document order without
    /// allocating a result `Vec` (DOM-ITER-03). Same pre-order and
    /// filter rules as [`Self::descendants`].
    pub fn for_each_descendant_element(
        &self,
        id: NodeId,
        filter: Option<&XName>,
        mut visit: impl FnMut(NodeId),
    ) {
        self.find_descendant_element(id, filter, |c| {
            visit(c);
            false
        });
    }

    /// First descendant element (in pre-order) that passes both the name
    /// `filter` and the `pred` predicate; the walk stops there instead of
    /// listing the whole subtree first.
    pub fn find_descendant_element(
        &self,
        id: NodeId,
        filter: Option<&XName>,
        mut pred: impl FnMut(NodeId) -> bool,
    ) -> Option<NodeId> {
        // Iterative stack avoids deep recursion on large bodies.
        let mut stack: Vec<(NodeId, usize)> = vec![(id, 0)];
        while let Some((node, i)) = stack.last_mut() {
            let n = self.child_count(*node);
            if *i >= n {
                stack.pop();
                continue;
            }
            let c = self.child_at(*node, *i);
            *i += 1;
            if !self.is_element(c) {
                continue;
            }
            // DOM-NAME-REF-01: compare the name by reference (`name_is`) instead
            // of `self.name(c)` — the latter clones the `XName` (two `Arc` bumps)
            // for every element visited, and this walk backs the 84-pass finalize
            // mesh + poststeps. `c` is already known to be an element here.
            let matches = match filter {
                None => true,
                Some(f) => self.name_is(c, f),
            };
            if matches && pred(c) {
                return Some(c);
            }
            stack.push((c, 0));
        }
        None
    }

    /// Every descendant node (not just elements), in pre-order: self is
    /// NOT included. For the allocating self + closed walk see
    /// [`Self::descendants_and_self`].
    pub fn descendant_nodes(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        self.walk_descendant_nodes(id, &mut out);
        out
    }
    fn walk_descendant_nodes(&self, id: NodeId, out: &mut Vec<NodeId>) {
        for &c in &self.data(id).content {
            out.push(c);
            if !self.data(c).content.is_empty() {
                self.walk_descendant_nodes(c, out);
            }
        }
    }

    /// `Self` (if it is an element matching `filter`) then its descendants,
    /// in pre-order. Equivalent to LINQ-to-XML `DescendantsAndSelf()`.
    pub fn descendants_and_self(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId> {
        let mut out = Vec::new();
        self.for_each_descendant_and_self(id, filter, |c| out.push(c));
        out
    }

    /// Non-allocating `DescendantsAndSelf` walk (DOM-ITER-03).
    pub fn for_each_descendant_and_self(
        &self,
        id: NodeId,
        filter: Option<&XName>,
        mut visit: impl FnMut(NodeId),
    ) {
        if let NodeKind::Element { name } = &self.data(id).kind
            && filter.is_none_or(|f| name == f)
        {
            visit(id);
        }
        self.for_each_descendant_element(id, filter, visit);
    }

    /// Ancestor elements from the nearest parent up to (but not
    /// including) the document, optionally filtered by name. Order is
    /// nearest → farthest.
    pub fn ancestors(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut p = self.data(id).parent;
        while let Some(pid) = p {
            if let NodeKind::Element { name } = &self.data(pid).kind
                && filter.is_none_or(|f| name == f)
            {
                out.push(pid);
            }
            p = self.data(pid).parent;
        }
        out
    }

    /// `Self` then its ancestor elements, optionally filtered by name.
    /// Walks up element-by-element; non-element ancestors (the document
    /// container) stop it.
    pub fn ancestors_and_self(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut cur = Some(id);
        while let Some(c) = cur {
            match &self.data(c).kind {
                NodeKind::Element { name } => {
                    if filter.is_none_or(|f| name == f) {
                        out.push(c);
                    }
                    cur = match self.data(c).parent {
                        Some(p) if self.is_element(p) => Some(p),
                        _ => None,
                    };
                }
                _ => break,
            }
        }
        out
    }

    /// Walk the parent chain until a Document node is found; returns it,
    /// or `None` if `id` is detached from any document.
    pub fn document(&self, id: NodeId) -> Option<NodeId> {
        let mut cur = Some(id);
        while let Some(c) = cur {
            if self.is_document(c) {
                return Some(c);
            }
            cur = self.data(c).parent;
        }
        None
    }

    /// The first child of `doc` that is an element, or `None` if the
    /// document is empty. `doc` itself must be a Document node.
    pub fn root(&self, doc: NodeId) -> Option<NodeId> {
        self.data(doc)
            .content
            .iter()
            .copied()
            .find(|&c| self.is_element(c))
    }

    fn index_in_parent(&self, id: NodeId) -> Option<(NodeId, usize)> {
        let p = self.data(id).parent?;
        let idx = self.data(p).content.iter().position(|&c| c == id)?;
        Some((p, idx))
    }

    /// Following siblings of `id` (all node kinds), in document order.
    /// Empty if `id` has no parent or is the last child.
    pub fn nodes_after_self(&self, id: NodeId) -> Vec<NodeId> {
        match self.index_in_parent(id) {
            Some((p, idx)) => self.data(p).content[idx + 1..].to_vec(),
            None => Vec::new(),
        }
    }
    /// Preceding siblings of `id` (all node kinds), in document order.
    /// Empty if `id` has no parent or is the first child.
    pub fn nodes_before_self(&self, id: NodeId) -> Vec<NodeId> {
        match self.index_in_parent(id) {
            Some((p, idx)) => self.data(p).content[..idx].to_vec(),
            None => Vec::new(),
        }
    }

    /// First following sibling element, or `None` if there is no element
    /// sibling after `id`. Skips text, comment, and PI siblings.
    pub fn next_element(&self, id: NodeId) -> Option<NodeId> {
        self.nodes_after_self(id)
            .into_iter()
            .find(|&n| self.is_element(n))
    }

    /// True if `id` has at least one direct child element.
    pub fn has_elements(&self, id: NodeId) -> bool {
        self.data(id).content.iter().any(|&c| self.is_element(c))
    }
    /// True if `id` carries at least one attribute. Equivalent to
    /// `attr_count(id) > 0`.
    pub fn has_attributes(&self, id: NodeId) -> bool {
        !self.data(id).attrs.is_empty()
    }

    // ── attributes ────────────────────────────────────────────────────────────
    /// The value of attribute `name` on `id`, or `None` if `id` is not an
    /// element or has no such attribute.
    pub fn attribute(&self, id: NodeId, name: &XName) -> Option<&str> {
        self.data(id)
            .attrs
            .iter()
            .find(|a| &a.name == name)
            .map(|a| a.value.as_str())
    }

    /// All attributes on `id`, in declaration order, as borrowed
    /// `(name, value)` pairs. Allocation-free; prefer it to
    /// [`Self::attributes`] unless the caller needs owned pairs, for example
    /// to change the DOM while it walks them.
    pub fn attributes_iter(
        &self,
        id: NodeId,
    ) -> impl ExactSizeIterator<Item = (&XName, &str)> + '_ {
        self.data(id)
            .attrs
            .iter()
            .map(|a| (&a.name, a.value.as_str()))
    }

    /// All attributes on `id`, in declaration order, as `(name, value)`
    /// owned pairs. For non-allocating iteration, use
    /// [`Self::attributes_iter`].
    pub fn attributes(&self, id: NodeId) -> Vec<(XName, String)> {
        self.data(id)
            .attrs
            .iter()
            .map(|a| (a.name.clone(), a.value.clone()))
            .collect()
    }

    /// Number of attributes on `id`. O(1) and allocation-free. Use
    /// [`Self::attr_at`] to walk them without cloning.
    pub fn attr_count(&self, id: NodeId) -> usize {
        self.data(id).attrs.len()
    }

    /// The `i`-th attribute on `id`, as borrowed `(name, value)` (no
    /// allocations; DOM-ITER-01 backs the serializer hot path).
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds for `id`'s attribute count. See
    /// [`Self::attr_count`].
    pub fn attr_at(&self, id: NodeId, i: usize) -> (&XName, &str) {
        let a = &self.data(id).attrs[i];
        (&a.name, a.value.as_str())
    }

    /// Insert a batch of attributes in declaration order. The parser uses
    /// this: pushing N attributes through [`Self::set_attribute_value`]
    /// re-scans the growing `attrs` Vec for each name (N(N-1)/2
    /// comparisons); here a wide tag costs O(N) — one hash probe per name
    /// against an index built once. Narrow tags take a short linear scan.
    ///
    /// Semantics match the incremental path exactly: a name already
    /// present (existing, or repeated in `attrs` after expansion) keeps
    /// its first declaration-order slot and takes the latest value — what
    /// N `set_attribute_value` calls produce.
    pub(crate) fn set_attributes(&mut self, id: NodeId, attrs: Vec<(XName, String)>) {
        let data = self.data_mut(id);
        if attrs.is_empty() {
            return;
        }
        // Common WordprocessingML tags stay under 16 attributes
        // (`w:rFonts` up to 9, `w:spacing` 8, `w:p` and `w:pgMar` 7). At
        // that width a scan of at most 16 x 16 comparisons beats allocating
        // a hash index; wider tags (a root's namespace declarations, the
        // #650 attack) take the index.
        if attrs.len() <= 16 {
            for (name, value) in attrs {
                if let Some(a) = data.attrs.iter_mut().find(|a| a.name == name) {
                    a.value = value;
                } else {
                    data.attrs.push(Attr { name, value });
                }
            }
            return;
        }
        let mut index: HashMap<XName, usize> =
            HashMap::with_capacity(data.attrs.len() + attrs.len());
        for (i, a) in data.attrs.iter().enumerate() {
            index.entry(a.name.clone()).or_insert(i);
        }
        for (name, value) in attrs {
            #[cfg(test)]
            bump_attr_name_probes();
            match index.get(&name) {
                Some(&i) => data.attrs[i].value = value,
                None => {
                    index.insert(name.clone(), data.attrs.len());
                    data.attrs.push(Attr { name, value });
                }
            }
        }
    }

    /// ⚡ Bolt: Sorts attributes in-place to avoid heavy allocations during canonicalization.
    pub fn sort_attributes(&mut self, id: NodeId) {
        let attrs = &mut self.data_mut(id).attrs;
        if attrs.len() < 2 {
            return;
        }
        attrs.sort_by(|a, b| {
            a.name
                .local_name()
                .cmp(b.name.local_name())
                .then_with(|| a.name.namespace_name().cmp(b.name.namespace_name()))
        });
    }

    /// Add or update an attribute. Passing `Some(value)` inserts the
    /// attribute when missing or replaces its value when present;
    /// `None` removes any matching attribute. Matches the TS
    /// `setAttributeValue(name, null)` removes behavior.
    pub fn set_attribute_value(&mut self, id: NodeId, name: &XName, value: Option<&str>) {
        let attrs = &mut self.data_mut(id).attrs;
        match value {
            None => attrs.retain(|a| &a.name != name),
            Some(v) => {
                #[cfg(test)]
                let found = {
                    let mut hit = None;
                    for (i, a) in attrs.iter().enumerate() {
                        bump_attr_name_probes();
                        if &a.name == name {
                            hit = Some(i);
                            break;
                        }
                    }
                    hit
                };
                #[cfg(not(test))]
                let found = attrs.iter().position(|a| &a.name == name);
                if let Some(i) = found {
                    attrs[i].value = v.to_string();
                } else {
                    attrs.push(Attr {
                        name: name.clone(),
                        value: v.to_string(),
                    });
                }
            }
        }
    }

    /// True if `name` looks like an `xmlns` / `xmlns:*` namespace
    /// declaration: namespace URI `http://www.w3.org/2000/xmlns/`, or no
    /// namespace with local name `xmlns`. Used by the parser and
    /// serializer to recognize (and round-trip) namespace declarations.
    pub fn is_namespace_declaration(&self, name: &XName) -> bool {
        name.namespace_name() == "http://www.w3.org/2000/xmlns/"
            || (name.namespace_name().is_empty() && name.local_name() == "xmlns")
    }

    // ── mutation ──────────────────────────────────────────────────────────────
    fn detach(&mut self, id: NodeId) {
        if let Some((p, idx)) = self.index_in_parent(id) {
            self.data_mut(p).content.remove(idx);
            self.data_mut(id).parent = None;
        }
    }

    /// If `node` already has a parent, deep-clone it (LINQ-to-XML
    /// semantics); otherwise return it as-is. Used before attaching
    /// content so the same subtree can move freely between parents
    /// without aliasing two slots.
    fn materialize(&mut self, node: NodeId) -> NodeId {
        if self.data(node).parent.is_some() {
            self.clone_subtree(node)
        } else {
            node
        }
    }

    /// Centralized validation before a node is attached to a parent:
    /// `parent` must be a container (`Element` or `Document`), `node`
    /// must not be the same as `parent`, and an unparented `node`
    /// must not be an ancestor of `parent` (cycle under descendant).
    fn validate_attachment(&self, parent: NodeId, node: NodeId) {
        if !self.is_element(parent) && !self.is_document(parent) {
            panic!("cannot attach a node to a non-container parent {parent:?}");
        }
        if parent == node {
            panic!("cannot attach a node to itself");
        }
        if self.data(node).parent.is_none() && self.is_ancestor_of(node, parent) {
            panic!("cannot attach an ancestor beneath its own descendant");
        }
    }

    /// True when `ancestor` is strictly above `descendant` on the parent chain.
    fn is_ancestor_of(&self, ancestor: NodeId, descendant: NodeId) -> bool {
        let mut cur = self.data(descendant).parent;
        while let Some(id) = cur {
            if id == ancestor {
                return true;
            }
            cur = self.data(id).parent;
        }
        false
    }

    /// Append `node` as the last child of `parent`. If `node` already
    /// has a parent it is deep-cloned into the arena first; the original
    /// id is left in place and the appended id is the new clone.
    ///
    /// # Panics
    ///
    /// Panics if `parent` is not a container (Element or Document), if
    /// `parent == node`, or if attaching the unparented `node` would
    /// create a cycle (i.e. `node` is an ancestor of `parent`).
    pub fn add(&mut self, parent: NodeId, node: NodeId) {
        let n = self.materialize(node);
        self.validate_attachment(parent, n);
        self.data_mut(n).parent = Some(parent);
        self.data_mut(parent).content.push(n);
    }

    /// Allocate a new text child of `parent` carrying `value`. Returns
    /// the new text node's id.
    ///
    /// # Panics
    ///
    /// Same container / cycle panics as [`Self::add`].
    pub fn add_text(&mut self, parent: NodeId, value: &str) -> NodeId {
        let t = self.new_text(value);
        self.validate_attachment(parent, t);
        self.data_mut(t).parent = Some(parent);
        self.data_mut(parent).content.push(t);
        t
    }

    /// Prepend `node` as the first child of `parent`. Materializes
    /// (deep-clones) `node` first if it is already parented.
    ///
    /// # Panics
    ///
    /// Same container / cycle panics as [`Self::add`].
    pub fn add_first(&mut self, parent: NodeId, node: NodeId) {
        let n = self.materialize(node);
        self.validate_attachment(parent, n);
        self.data_mut(n).parent = Some(parent);
        self.data_mut(parent).content.insert(0, n);
    }

    /// Detach every child of `id` (front matching LINQ-to-XML
    /// `RemoveNodes()`); their parent links are cleared.
    pub fn remove_nodes(&mut self, id: NodeId) {
        let kids = std::mem::take(&mut self.data_mut(id).content);
        for k in kids {
            self.data_mut(k).parent = None;
        }
    }

    /// Detach `id` from its parent. No-op if `id` has no parent.
    pub fn remove(&mut self, id: NodeId) {
        self.detach(id);
    }

    /// Insert `node` as the preceding sibling of `reference`, which must
    /// have a parent. `node` may come from anywhere: one that already has
    /// a parent is cloned first.
    ///
    /// # Panics
    ///
    /// Panics if `reference` has no parent, plus the same
    /// container / cycle panics as [`Self::add`].
    pub fn add_before_self(&mut self, reference: NodeId, node: NodeId) {
        let (p, idx) = self
            .index_in_parent(reference)
            .expect("No parent for AddBeforeSelf");
        let n = self.materialize(node);
        self.validate_attachment(p, n);
        self.data_mut(n).parent = Some(p);
        self.data_mut(p).content.insert(idx, n);
    }

    /// Insert `node` as the following sibling of `reference`. Same
    /// rules as [`Self::add_before_self`].
    ///
    /// # Panics
    ///
    /// Panics if `reference` has no parent, plus the same
    /// container / cycle panics as [`Self::add`].
    pub fn add_after_self(&mut self, reference: NodeId, node: NodeId) {
        let (p, idx) = self
            .index_in_parent(reference)
            .expect("No parent for AddAfterSelf");
        let n = self.materialize(node);
        self.validate_attachment(p, n);
        self.data_mut(n).parent = Some(p);
        self.data_mut(p).content.insert(idx + 1, n);
    }

    /// Replace `reference` with the sequence of `nodes` (in order) at
    /// the same position. `reference` is detached. Each entry is
    /// materialized first if it is already parented.
    ///
    /// # Panics
    ///
    /// Panics if `reference` has no parent, plus the same
    /// container / cycle panics as [`Self::add`] for each entry.
    pub fn replace_with(&mut self, reference: NodeId, nodes: &[NodeId]) {
        let (p, idx) = self
            .index_in_parent(reference)
            .expect("No parent for ReplaceWith");
        let mut materialized = Vec::with_capacity(nodes.len());
        for &node in nodes {
            let n = self.materialize(node);
            self.validate_attachment(p, n);
            self.data_mut(n).parent = Some(p);
            materialized.push(n);
        }
        self.data_mut(reference).parent = None;
        self.data_mut(p).content.splice(idx..=idx, materialized);
    }

    /// Concatenated descendant text of `id` (LINQ-to-XML `element.Value`).
    ///
    /// Always allocates; for the borrowed single-text-child fast path see
    /// [`Self::value_str`] (ATOM-TEXT-01: a `w:t` / atom with one direct
    /// text child skips the recursive walk).
    pub fn value(&self, id: NodeId) -> String {
        match self.value_str(id) {
            std::borrow::Cow::Borrowed(s) => s.to_string(),
            std::borrow::Cow::Owned(s) => s,
        }
    }

    /// Borrowed text when `id` has exactly one direct text child;
    /// otherwise owned concatenated descendant text. Same bytes as
    /// [`Self::value`] in both arms (ATOM-TEXT-01).
    pub fn value_str(&self, id: NodeId) -> std::borrow::Cow<'_, str> {
        let content = &self.data(id).content;
        if content.len() == 1
            && let NodeKind::Text(v) = &self.data(content[0]).kind
        {
            return std::borrow::Cow::Borrowed(v.as_str());
        }
        let mut s = String::new();
        self.collect_text(id, &mut s);
        std::borrow::Cow::Owned(s)
    }

    fn collect_text(&self, id: NodeId, s: &mut String) {
        for &c in &self.data(id).content {
            match &self.data(c).kind {
                NodeKind::Text(v) => s.push_str(v),
                NodeKind::Element { .. } | NodeKind::Document { .. } => self.collect_text(c, s),
                _ => {}
            }
        }
    }

    /// Replace `id`'s children with a single text child carrying `value`
    /// (LINQ-to-XML `element.Value = value`). Comments and PIs are lost.
    pub fn set_value(&mut self, id: NodeId, value: &str) {
        self.remove_nodes(id);
        self.add_text(id, value);
    }

    /// Deep-clone the subtree rooted at `id` into the same arena and return
    /// the new root. The clone has no parent. Port of `XElement.Clone()` /
    /// `XContainer.Clone()`.
    ///
    /// Walks children by index (no temporary `content` Vec clone) and
    /// `reserve_exact`s the destination content capacity so growth is one
    /// shot (CLONE-01).
    pub fn clone_subtree(&mut self, id: NodeId) -> NodeId {
        let new_kind = match &self.data(id).kind {
            NodeKind::Element { name } => NodeKind::Element { name: name.clone() },
            NodeKind::Text(v) => NodeKind::Text(v.clone()),
            NodeKind::Comment(v) => NodeKind::Comment(v.clone()),
            NodeKind::Pi(pi) => NodeKind::Pi(pi.clone()),
            NodeKind::Document { declaration } => NodeKind::Document {
                declaration: declaration.clone(),
            },
        };
        let n_kids = self.child_count(id);
        let attrs = self.data(id).attrs.clone();
        let copy = self.alloc(new_kind);
        {
            let d = self.data_mut(copy);
            d.attrs = attrs;
            d.content.reserve_exact(n_kids);
        }
        for i in 0..n_kids {
            // Re-index each step: recursive clone may reallocate the arena.
            let k = self.child_at(id, i);
            let ck = self.clone_subtree(k);
            self.validate_attachment(copy, ck);
            self.data_mut(ck).parent = Some(copy);
            self.data_mut(copy).content.push(ck);
        }
        copy
    }

    /// Deep-copy the subtree rooted at `id` in another arena `src` into this
    /// one and return the new root, which has no parent. Names, attributes
    /// (namespace declarations included) and child order are kept;
    /// annotations are not.
    pub fn import_subtree(&mut self, src: &Dom, id: NodeId) -> NodeId {
        let new_kind = match &src.data(id).kind {
            NodeKind::Element { name } => NodeKind::Element { name: name.clone() },
            NodeKind::Text(v) => NodeKind::Text(v.clone()),
            NodeKind::Comment(v) => NodeKind::Comment(v.clone()),
            NodeKind::Pi(pi) => NodeKind::Pi(pi.clone()),
            NodeKind::Document { declaration } => NodeKind::Document {
                declaration: declaration.clone(),
            },
        };
        let copy = self.alloc(new_kind);
        self.data_mut(copy).attrs = src.data(id).attrs.clone();
        for &k in &src.data(id).content {
            let ck = self.import_subtree(src, k);
            self.validate_attachment(copy, ck);
            self.data_mut(ck).parent = Some(copy);
            self.data_mut(copy).content.push(ck);
        }
        copy
    }

    // ── annotations ────────────────────────────────────────────────────────────
    // Stored in a `Dom` side table keyed by `NodeId` rather than inline on every
    // `NodeData` (ANN-01): production never annotates, so the map stays empty and
    // the hot per-node struct keeps its 24 bytes. Behavior is identical.
    /// Attach a typed annotation to `id`. Multiple annotations of the same
    /// `T` may be added; [`Self::annotation`] returns the first, and
    /// [`Self::remove_annotations`] removes every one of type `T`.
    ///
    /// Annotations are stored in a side table (ANN-01), so adding them
    /// does not enlarge the per-node struct on the hot path.
    pub fn add_annotation<T: Any + 'static>(&mut self, id: NodeId, annotation: T) {
        self.annotations
            .entry(id)
            .or_default()
            .push(Box::new(annotation));
    }
    /// The first annotation of type `T` attached to `id`, or `None`.
    pub fn annotation<T: Any + 'static>(&self, id: NodeId) -> Option<&T> {
        self.annotations
            .get(&id)?
            .iter()
            .find_map(|a| a.downcast_ref::<T>())
    }
    /// Drop every annotation of type `T` from `id`. Other-type annotations
    /// on the same node are kept; if none remain the side-table entry is
    /// cleared.
    pub fn remove_annotations<T: Any + 'static>(&mut self, id: NodeId) {
        if let Some(v) = self.annotations.get_mut(&id) {
            v.retain(|a| !a.is::<T>());
            if v.is_empty() {
                self.annotations.remove(&id);
            }
        }
    }

    // ── parse / serialize convenience (delegate to the submodules) ──────────────
    /// Parse a full XML string (with optional prolog) into a Document node
    /// allocated into this arena. See [`parse::parse_xdocument`] for the
    /// parsing semantics. A leading UTF-8 BOM is stripped before parsing.
    pub fn parse_xdocument(&mut self, xml: &str) -> NodeId {
        parse::parse_xdocument(self, xml)
    }

    /// Checked variant of [`Self::parse_xdocument`]: refuses input that
    /// fails [`parse::validate_xml`] (including the tag-width and namespace
    /// budgets) before any DOM node is built.
    ///
    /// # Errors
    ///
    /// Returns the [`parse::validate_xml`] error string unchanged.
    pub fn try_parse_xdocument(&mut self, xml: &str) -> Result<NodeId, String> {
        parse::try_parse_xdocument(self, xml)
    }
    /// Serialize an element subtree to an XML string. See
    /// [`serialize::serialize_element`] for namespace-prefix scope rules.
    pub fn serialize_element(&self, el: NodeId) -> String {
        serialize::serialize_element(self, el)
    }

    /// SHA-1 (lowercase hex) of the serialized element with the first
    /// WML default-xmlns declaration stripped from the root open tag.
    /// Byte-identical to hashing
    /// `serialize_element(el).replacen(WML_DEFAULT_XMLNS, "", 1)` without
    /// materializing the full XML string. Used by the atom-hash stream
    /// (HASH-STREAM).
    #[must_use]
    pub fn serialize_element_sha1_hex(&self, el: NodeId) -> String {
        serialize::serialize_element_sha1_hex(self, el)
    }

    /// SHA-1 (lowercase hex) of the structural shape only: attributes and
    /// element nesting, no text / comment / PI children, with the same
    /// first-WML-default-xmlns strip as [`Self::serialize_element_sha1_hex`].
    /// HASH-STREAM-02.
    #[must_use]
    pub fn serialize_element_structure_sha1_hex(&self, el: NodeId) -> String {
        serialize::serialize_element_structure_sha1_hex(self, el)
    }
    /// Serialize a whole document (declaration + children) to a string.
    /// See [`serialize::serialize_document`].
    #[must_use]
    pub fn serialize_document(&self, doc: NodeId) -> String {
        serialize::serialize_document(self, doc)
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod node_layout_tests {
    use super::*;

    /// ANN-01 mechanism counter. `annotations` has no production caller (only the
    /// M1 foundation test), so carrying a `Vec<Box<dyn Any>>` on every node is pure
    /// per-node bloat: 24 bytes × N nodes of arena-realloc memcpy and RSS. After
    /// moving it to a `Dom` side table, `NodeData` must drop by one `Vec` (24 B),
    /// from 152 → 128 on this target. The bound (not an exact equality) guards
    /// against silently regrowing the hot per-node struct.
    #[test]
    fn node_data_excludes_annotations_vec() {
        let sz = std::mem::size_of::<NodeData>();
        assert!(
            sz <= 128,
            "NodeData is {sz} bytes; ANN-01 requires <= 128 (annotations must live \
             in the Dom side table, not inline on every node)"
        );
    }

    /// NODE-KIND-01 mechanism counter. `NodeKind`'s size is set by its largest
    /// variant plus a discriminant. `Document{declaration}` (~72 B, one node per
    /// doc) and `Pi` (~48 B, rare in docx) inflated every node — including the
    /// common `Element`/`Text`. Boxing those two rare variants drops the max
    /// payload to `Element{XName}` (32 B); with 5 data-carrying variants Rust
    /// can't niche the tag, so `NodeKind` = 32 + 8-byte tag = 40 B, and
    /// `NodeData` falls 128 → 96 (originally 152). The bounds guard against
    /// regrowth back toward the fat enum.
    #[test]
    fn node_kind_rare_variants_are_boxed() {
        let kind = std::mem::size_of::<NodeKind>();
        let node = std::mem::size_of::<NodeData>();
        assert!(
            kind <= 40,
            "NodeKind is {kind} bytes; NODE-KIND-01 requires <= 40 (box the rare \
             Document/Pi variants so they don't size every node)"
        );
        assert!(
            node <= 96,
            "NodeData is {node} bytes; NODE-KIND-01 requires <= 96 (was 128 after \
             ANN-01, 152 originally)"
        );
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod intern_tests {
    use super::*;

    /// PARSE-01 mechanism proof. `XName::get`/`XNamespace::get` must intern their
    /// `Arc<str>` so identical namespace/local strings — which recur on millions
    /// of elements and attributes — share one allocation instead of a fresh
    /// `Arc::from` each time. Verified by pointer identity of the backing `Arc`s.
    /// Content-based `Eq`/`Hash` are unchanged, so this is parity-safe.
    #[test]
    fn identical_names_share_interned_arcs() {
        let ns = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
        let a = XName::get("p", ns);
        let b = XName::get("p", ns);
        assert!(
            Arc::ptr_eq(&a.local, &b.local),
            "PARSE-01: identical local names must share one interned Arc"
        );
        assert!(
            Arc::ptr_eq(&a.namespace.name, &b.namespace.name),
            "PARSE-01: identical namespaces must share one interned Arc"
        );
        // Interning must not change equality semantics.
        assert_eq!(a, b);
        assert_ne!(a, XName::get("r", ns));
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod attribute_iter_tests {
    use super::*;

    /// `attributes_iter` walks the same pairs as `attributes`, in the same
    /// declaration order, borrowing instead of cloning.
    #[test]
    fn attributes_iter_borrows_the_pairs_attributes_clones() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(r#"<a x="1" y="two" z=""><b/></a>"#);
        let root = dom.root(doc).unwrap();
        let borrowed: Vec<(XName, String)> = dom
            .attributes_iter(root)
            .map(|(name, value)| (name.clone(), value.to_string()))
            .collect();
        assert_eq!(borrowed, dom.attributes(root));
        assert_eq!(dom.attributes_iter(root).len(), 3);
        let leaf = dom.elements(root, None)[0];
        assert_eq!(dom.attributes_iter(leaf).next(), None);
        assert_eq!(dom.attributes_iter(leaf).len(), 0);
    }

    /// `import_subtree` copies a subtree out of another arena: the copy
    /// serializes as the source does, has no parent, and the source is left
    /// as it was.
    #[test]
    fn import_subtree_copies_across_arenas() {
        let mut src = Dom::new();
        let doc = src.parse_xdocument(
            r#"<w:r xmlns:w="urn:w"><w:rPr><w:sz w:val="21"/><!--c--><?pi x?></w:rPr><w:t xml:space="preserve"> a </w:t></w:r>"#,
        );
        let run = src.root(doc).unwrap();
        let rpr = src.elements(run, None)[0];
        let mut dst = Dom::new();
        dst.parse_xdocument("<x/>");
        let copy = dst.import_subtree(&src, rpr);
        assert_eq!(dst.parent(copy), None);
        assert_eq!(dst.serialize_element(copy), src.serialize_element(rpr));
        let whole = dst.import_subtree(&src, run);
        assert_eq!(dst.serialize_element(whole), src.serialize_element(run));
        assert_eq!(src.parent(rpr), Some(run));
        let other = dst.import_subtree(&src, doc);
        assert_eq!(dst.root(other).map(|r| dst.name(r)), Some(src.name(run)));
    }
}
