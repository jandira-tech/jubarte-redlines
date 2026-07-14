//! Port of `lib/xml-linq.ts` — a LINQ-to-XML-style mutable XML tree.
//!
//! M1.1: `XName` / `XNamespace` (interned expanded names).
//! M1.2: arena DOM (`Dom`, `NodeId`, …).
//! M1.3: `parse` — string → DOM.
//! M1.4: `serialize` — DOM → string.

pub mod parse;
pub mod serialize;

pub use parse::parse_xdocument;
pub use serialize::{serialize_document, serialize_element};

use std::sync::Arc;

/// An XML namespace (just its URI), owned via `Arc<str>`. Port of `XNamespace`.
#[derive(Clone)]
pub struct XNamespace {
    name: Arc<str>,
}

impl XNamespace {
    /// `XNamespace.get(namespaceName)`.
    pub fn get(namespace_name: &str) -> XNamespace {
        XNamespace {
            name: Arc::from(namespace_name),
        }
    }

    /// `XNamespace.None` — the empty namespace.
    pub fn none() -> XNamespace {
        XNamespace::get("")
    }

    /// `XNamespace.Xmlns`.
    pub fn xmlns() -> XNamespace {
        XNamespace::get("http://www.w3.org/2000/xmlns/")
    }

    /// `XNamespace.Xml`.
    pub fn xml() -> XNamespace {
        XNamespace::get("http://www.w3.org/XML/1998/namespace")
    }

    /// `ns.getName(local)` → `XName`.
    pub fn name(&self, local: &str) -> XName {
        XName::get(local, &self.name)
    }

    /// `ns.NamespaceName`.
    pub fn namespace_name(&self) -> &str {
        &self.name
    }
}

impl PartialEq for XNamespace {
    fn eq(&self, other: &Self) -> bool {
        self.name.as_ref() == other.name.as_ref()
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

/// An expanded XML name (namespace + local name), owned via `Arc<str>`. Port of `XName`.
#[derive(Clone)]
pub struct XName {
    local: Arc<str>,
    namespace: XNamespace,
}

impl XName {
    /// `XName.get(localName, namespaceName = "")`.
    pub fn get(local_name: &str, namespace_name: &str) -> XName {
        XName {
            local: Arc::from(local_name),
            namespace: XNamespace::get(namespace_name),
        }
    }

    /// `XName.fromExpanded(expanded)` — parse clark notation `"{ns}local"` or
    /// bare `"local"`. Renamed `from_clark` to match the plan's API.
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

    /// `name.LocalName`.
    pub fn local_name(&self) -> &str {
        &self.local
    }

    /// `name.Namespace`.
    pub fn namespace(&self) -> &XNamespace {
        &self.namespace
    }

    /// `name.NamespaceName`.
    pub fn namespace_name(&self) -> &str {
        self.namespace.namespace_name()
    }

    /// `name.toString()` — clark notation.
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
        self.local.as_ref() == other.local.as_ref() && self.namespace == other.namespace
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

/// Handle into the `Dom` arena.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u32);

/// An XML declaration (`<?xml version encoding standalone?>`). Port of `XDeclaration`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct XDeclaration {
    pub version: Option<String>,
    pub encoding: Option<String>,
    pub standalone: Option<String>,
}

/// An attribute (name + value). Port of `XAttribute`.
#[derive(Clone, Debug)]
struct Attr {
    name: XName,
    value: String,
}

/// Node kind discriminant + kind-specific data.
enum NodeKind {
    Element { name: XName },
    Text(String),
    Comment(String),
    Pi { target: String, data: String },
    Document { declaration: Option<XDeclaration> },
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

/// The arena holding all nodes. Port of the LINQ-to-XML object graph.
#[derive(Default)]
pub struct Dom {
    nodes: Vec<NodeData>,
    /// Typed annotations, keyed by `NodeId`. Kept off `NodeData` because no
    /// production path stores annotations (only the M1 foundation test), so the
    /// common case is an empty map and every node avoids a 24-byte inline `Vec`.
    annotations: HashMap<NodeId, Vec<Box<dyn Any>>>,
}

impl Dom {
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

    fn data(&self, id: NodeId) -> &NodeData {
        &self.nodes[id.0 as usize]
    }
    fn data_mut(&mut self, id: NodeId) -> &mut NodeData {
        &mut self.nodes[id.0 as usize]
    }

    // ── constructors ────────────────────────────────────────────────────────
    pub fn new_document(&mut self) -> NodeId {
        self.alloc(NodeKind::Document { declaration: None })
    }
    pub fn new_element(&mut self, name: XName) -> NodeId {
        self.alloc(NodeKind::Element { name })
    }
    pub fn new_text(&mut self, value: &str) -> NodeId {
        self.alloc(NodeKind::Text(value.to_string()))
    }
    pub fn new_comment(&mut self, value: &str) -> NodeId {
        self.alloc(NodeKind::Comment(value.to_string()))
    }
    pub fn new_pi(&mut self, target: &str, data: &str) -> NodeId {
        self.alloc(NodeKind::Pi {
            target: target.to_string(),
            data: data.to_string(),
        })
    }

    // ── kind predicates / accessors ───────────────────────────────────────────
    pub fn is_element(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Element { .. })
    }
    pub fn is_text(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Text(_))
    }
    pub fn is_document(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Document { .. })
    }
    pub fn is_comment(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Comment(_))
    }
    pub fn is_pi(&self, id: NodeId) -> bool {
        matches!(self.data(id).kind, NodeKind::Pi { .. })
    }

    /// `element.Name` — element name, or None for non-elements.
    pub fn name(&self, id: NodeId) -> Option<XName> {
        match &self.data(id).kind {
            NodeKind::Element { name } => Some(name.clone()),
            _ => None,
        }
    }
    /// `element.Name = n`.
    pub fn set_name(&mut self, id: NodeId, new_name: XName) {
        if let NodeKind::Element { name } = &mut self.data_mut(id).kind {
            *name = new_name;
        }
    }

    /// Text/Comment value, or None for other kinds.
    pub fn text_value(&self, id: NodeId) -> Option<&str> {
        match &self.data(id).kind {
            NodeKind::Text(v) | NodeKind::Comment(v) => Some(v),
            _ => None,
        }
    }
    pub fn set_text_value(&mut self, id: NodeId, value: &str) {
        match &mut self.data_mut(id).kind {
            NodeKind::Text(v) | NodeKind::Comment(v) => *v = value.to_string(),
            _ => {}
        }
    }

    pub fn pi_target(&self, id: NodeId) -> Option<&str> {
        match &self.data(id).kind {
            NodeKind::Pi { target, .. } => Some(target),
            _ => None,
        }
    }
    pub fn pi_data(&self, id: NodeId) -> Option<&str> {
        match &self.data(id).kind {
            NodeKind::Pi { data, .. } => Some(data),
            _ => None,
        }
    }

    pub fn declaration(&self, id: NodeId) -> Option<&XDeclaration> {
        match &self.data(id).kind {
            NodeKind::Document { declaration } => declaration.as_ref(),
            _ => None,
        }
    }
    pub fn set_declaration(&mut self, id: NodeId, decl: Option<XDeclaration>) {
        if let NodeKind::Document { declaration } = &mut self.data_mut(id).kind {
            *declaration = decl;
        }
    }

    // ── tree navigation ───────────────────────────────────────────────────────
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.data(id).parent
    }

    /// `Nodes()` — all child nodes in document order (clone of the content vec).
    pub fn nodes(&self, id: NodeId) -> Vec<NodeId> {
        self.data(id).content.clone()
    }

    /// `FirstNode`.
    pub fn first_node(&self, id: NodeId) -> Option<NodeId> {
        self.data(id).content.first().copied()
    }
    /// `LastNode`.
    pub fn last_node(&self, id: NodeId) -> Option<NodeId> {
        self.data(id).content.last().copied()
    }

    /// `Elements()` / `Elements(name)` — child elements, optionally filtered.
    /// Number of direct children of `id` (all node kinds). Cheap O(1) index —
    /// paired with [`child_at`](Self::child_at) for non-allocating child
    /// iteration on hot paths where [`elements`](Self::elements)' per-call
    /// `Vec` is the cost (atomize). Re-read the count each loop step: it is
    /// stable while the caller does not add/remove children of `id`.
    pub fn child_count(&self, id: NodeId) -> usize {
        self.data(id).content.len()
    }

    /// The `i`-th direct child of `id` (all node kinds). Panics out of bounds.
    /// See [`child_count`](Self::child_count).
    pub fn child_at(&self, id: NodeId, i: usize) -> NodeId {
        self.data(id).content[i]
    }

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

    /// `Element(name)` — first matching child element.
    pub fn element(&self, id: NodeId, filter: &XName) -> Option<NodeId> {
        self.data(id)
            .content
            .iter()
            .copied()
            .find(|&c| matches!(&self.data(c).kind, NodeKind::Element { name } if name == filter))
    }

    /// `Descendants()` / `Descendants(name)` — all descendant elements (pre-order).
    pub fn descendants(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId> {
        let mut out = Vec::new();
        self.walk_descendant_elements(id, filter, &mut out);
        out
    }
    fn walk_descendant_elements(&self, id: NodeId, filter: Option<&XName>, out: &mut Vec<NodeId>) {
        for &c in &self.data(id).content {
            if let NodeKind::Element { name } = &self.data(c).kind {
                if filter.is_none_or(|f| name == f) {
                    out.push(c);
                }
                self.walk_descendant_elements(c, filter, out);
            }
        }
    }

    /// `DescendantNodes()` — all descendant nodes (not just elements), pre-order.
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

    /// `DescendantsAndSelf()` — self (if element & matches) then descendants.
    pub fn descendants_and_self(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId> {
        let mut out = Vec::new();
        if let NodeKind::Element { name } = &self.data(id).kind
            && filter.is_none_or(|f| name == f)
        {
            out.push(id);
        }
        self.walk_descendant_elements(id, filter, &mut out);
        out
    }

    /// `Ancestors()` — parents from nearest to root (elements only).
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

    /// `AncestorsAndSelf()` — self then ancestors (elements only).
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

    /// `Document` — the owning XDocument, if any.
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

    /// `Root` — the document's root element.
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

    /// `NodesAfterSelf()`.
    pub fn nodes_after_self(&self, id: NodeId) -> Vec<NodeId> {
        match self.index_in_parent(id) {
            Some((p, idx)) => self.data(p).content[idx + 1..].to_vec(),
            None => Vec::new(),
        }
    }
    /// `NodesBeforeSelf()`.
    pub fn nodes_before_self(&self, id: NodeId) -> Vec<NodeId> {
        match self.index_in_parent(id) {
            Some((p, idx)) => self.data(p).content[..idx].to_vec(),
            None => Vec::new(),
        }
    }

    /// `NextElement` — first following sibling element.
    pub fn next_element(&self, id: NodeId) -> Option<NodeId> {
        self.nodes_after_self(id)
            .into_iter()
            .find(|&n| self.is_element(n))
    }

    pub fn has_elements(&self, id: NodeId) -> bool {
        self.data(id).content.iter().any(|&c| self.is_element(c))
    }
    pub fn has_attributes(&self, id: NodeId) -> bool {
        !self.data(id).attrs.is_empty()
    }

    // ── attributes ────────────────────────────────────────────────────────────
    /// `Attribute(name)` value.
    pub fn attribute(&self, id: NodeId, name: &XName) -> Option<&str> {
        self.data(id)
            .attrs
            .iter()
            .find(|a| &a.name == name)
            .map(|a| a.value.as_str())
    }

    /// `Attributes()` — (name, value) pairs in order.
    pub fn attributes(&self, id: NodeId) -> Vec<(XName, String)> {
        self.data(id)
            .attrs
            .iter()
            .map(|a| (a.name.clone(), a.value.clone()))
            .collect()
    }

    /// `SetAttributeValue(name, value)` — add/update; `None` removes (matches the
    /// TS where passing `null` removes the attribute).
    pub fn set_attribute_value(&mut self, id: NodeId, name: &XName, value: Option<&str>) {
        let attrs = &mut self.data_mut(id).attrs;
        match value {
            None => attrs.retain(|a| &a.name != name),
            Some(v) => {
                if let Some(a) = attrs.iter_mut().find(|a| &a.name == name) {
                    a.value = v.to_string();
                } else {
                    attrs.push(Attr {
                        name: name.clone(),
                        value: v.to_string(),
                    });
                }
            }
        }
    }

    /// True if `attr` is a namespace declaration (`xmlns` / `xmlns:*`).
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

    /// If `node` already has a parent, deep-clone it (LINQ-to-XML semantics);
    /// otherwise return it as-is. Used before attaching content.
    fn materialize(&mut self, node: NodeId) -> NodeId {
        if self.data(node).parent.is_some() {
            self.clone_subtree(node)
        } else {
            node
        }
    }

    /// Centralized validation before a node is attached to a parent.
    /// - `parent` must be a container (`Element` or `Document`).
    /// - `node` must not be the same as `parent` (self-attachment).
    /// - When `node` is unparented it must not be an ancestor of `parent`,
    ///   which would create a cycle under its own descendant.
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

    /// `Add(node)` — append a single node (cloning if already parented).
    pub fn add(&mut self, parent: NodeId, node: NodeId) {
        let n = self.materialize(node);
        self.validate_attachment(parent, n);
        self.data_mut(n).parent = Some(parent);
        self.data_mut(parent).content.push(n);
    }

    /// `Add(text)` — append a fresh text node.
    pub fn add_text(&mut self, parent: NodeId, value: &str) -> NodeId {
        let t = self.new_text(value);
        self.validate_attachment(parent, t);
        self.data_mut(t).parent = Some(parent);
        self.data_mut(parent).content.push(t);
        t
    }

    /// `AddFirst(node)` — prepend.
    pub fn add_first(&mut self, parent: NodeId, node: NodeId) {
        let n = self.materialize(node);
        self.validate_attachment(parent, n);
        self.data_mut(n).parent = Some(parent);
        self.data_mut(parent).content.insert(0, n);
    }

    /// `RemoveNodes()` — detach all children.
    pub fn remove_nodes(&mut self, id: NodeId) {
        let kids = std::mem::take(&mut self.data_mut(id).content);
        for k in kids {
            self.data_mut(k).parent = None;
        }
    }

    /// `Remove()` — detach this node from its parent.
    pub fn remove(&mut self, id: NodeId) {
        self.detach(id);
    }

    /// `AddBeforeSelf(node)`.
    pub fn add_before_self(&mut self, reference: NodeId, node: NodeId) {
        let (p, idx) = self
            .index_in_parent(reference)
            .expect("No parent for AddBeforeSelf");
        let n = self.materialize(node);
        self.validate_attachment(p, n);
        self.data_mut(n).parent = Some(p);
        self.data_mut(p).content.insert(idx, n);
    }

    /// `AddAfterSelf(node)`.
    pub fn add_after_self(&mut self, reference: NodeId, node: NodeId) {
        let (p, idx) = self
            .index_in_parent(reference)
            .expect("No parent for AddAfterSelf");
        let n = self.materialize(node);
        self.validate_attachment(p, n);
        self.data_mut(n).parent = Some(p);
        self.data_mut(p).content.insert(idx + 1, n);
    }

    /// `ReplaceWith(nodes)` — replace this node with the given content.
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

    /// `element.Value` getter — concatenated descendant text.
    pub fn value(&self, id: NodeId) -> String {
        let mut s = String::new();
        self.collect_text(id, &mut s);
        s
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

    /// `element.Value = v` — clear children, add a single text node.
    pub fn set_value(&mut self, id: NodeId, value: &str) {
        self.remove_nodes(id);
        self.add_text(id, value);
    }

    /// Deep-clone a subtree into the same arena, returning the new root. The
    /// clone has no parent. Port of `XElement.clone()` / `XContainer.clone()`.
    pub fn clone_subtree(&mut self, id: NodeId) -> NodeId {
        let new_kind = match &self.data(id).kind {
            NodeKind::Element { name } => NodeKind::Element { name: name.clone() },
            NodeKind::Text(v) => NodeKind::Text(v.clone()),
            NodeKind::Comment(v) => NodeKind::Comment(v.clone()),
            NodeKind::Pi { target, data } => NodeKind::Pi {
                target: target.clone(),
                data: data.clone(),
            },
            NodeKind::Document { declaration } => NodeKind::Document {
                declaration: declaration.clone(),
            },
        };
        let copy = self.alloc(new_kind);
        // attributes
        let attrs = self.data(id).attrs.clone();
        self.data_mut(copy).attrs = attrs;
        // children (recursive)
        let kids = self.data(id).content.clone();
        for k in kids {
            let ck = self.clone_subtree(k);
            self.validate_attachment(copy, ck);
            self.data_mut(ck).parent = Some(copy);
            self.data_mut(copy).content.push(ck);
        }
        copy
    }

    // ── annotations ───────────────────────────────────────────────────────────
    // Stored in a `Dom` side table keyed by `NodeId` rather than inline on every
    // `NodeData` (ANN-01): production never annotates, so the map stays empty and
    // the hot per-node struct keeps its 24 bytes. Behavior is identical.
    /// `AddAnnotation(obj)`.
    pub fn add_annotation<T: Any + 'static>(&mut self, id: NodeId, annotation: T) {
        self.annotations
            .entry(id)
            .or_default()
            .push(Box::new(annotation));
    }
    /// `Annotation<T>()` — first annotation of type `T`.
    pub fn annotation<T: Any + 'static>(&self, id: NodeId) -> Option<&T> {
        self.annotations
            .get(&id)?
            .iter()
            .find_map(|a| a.downcast_ref::<T>())
    }
    /// `RemoveAnnotations<T>()`.
    pub fn remove_annotations<T: Any + 'static>(&mut self, id: NodeId) {
        if let Some(v) = self.annotations.get_mut(&id) {
            v.retain(|a| !a.is::<T>());
            if v.is_empty() {
                self.annotations.remove(&id);
            }
        }
    }

    // ── parse / serialize convenience (delegate to the submodules) ─────────────
    /// Parse an XML string into a Document node (M1.3).
    pub fn parse_xdocument(&mut self, xml: &str) -> NodeId {
        parse::parse_xdocument(self, xml)
    }
    /// Serialize an element subtree to XML (M1.4).
    pub fn serialize_element(&self, el: NodeId) -> String {
        serialize::serialize_element(self, el)
    }
    /// Serialize a whole document (declaration + root) (M1.4).
    pub fn serialize_document(&self, doc: NodeId) -> String {
        serialize::serialize_document(self, doc)
    }
}

#[cfg(test)]
mod ann01_tests {
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
}
