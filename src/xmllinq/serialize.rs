//! Port of `serializeElement` / `XDocument.toString` from `lib/xml-linq.ts`.
//!
//! Scope-aware namespace serialization: each element inherits its parent
//! namespace scope and may add/override local `xmlns` declarations, so nested
//! namespace scopes are not flattened to the root.

use std::collections::HashMap;

use super::{Dom, NodeId, XName};

const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
const MC_NAMESPACE: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";

/// Conventional OOXML prefixes (URI → prefix) so output matches Word's shape.
fn well_known_prefix(ns: &str) -> Option<&'static str> {
    Some(match ns {
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main" => "w",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships" => "r",
        "http://schemas.openxmlformats.org/markup-compatibility/2006" => "mc",
        "http://schemas.microsoft.com/office/word/2010/wordml" => "w14",
        "http://schemas.microsoft.com/office/word/2012/wordml" => "w15",
        "http://schemas.microsoft.com/office/word/2018/wordml/cex" => "w16cex",
        "http://schemas.microsoft.com/office/word/2016/wordml/cid" => "w16cid",
        "http://schemas.microsoft.com/office/word/2015/wordml/symex" => "w16se",
        "http://schemas.openxmlformats.org/drawingml/2006/main" => "a",
        "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" => "wp",
        "http://schemas.openxmlformats.org/drawingml/2006/picture" => "pic",
        "http://schemas.openxmlformats.org/officeDocument/2006/math" => "m",
        "urn:schemas-microsoft-com:vml" => "v",
        "urn:schemas-microsoft-com:office:office" => "o",
        "urn:schemas-microsoft-com:office:word" => "w10",
        "http://schemas.microsoft.com/office/word/2006/wordml" => "wne",
        // Microsoft drawing/shape extensions. These MUST keep their conventional
        // prefixes: `mc:Choice Requires="wps"` (etc.) is a prefix string evaluated
        // against in-scope xmlns, so renaming wps→nsN dangles Requires and Word
        // rejects the AlternateContent as "unreadable content".
        "http://schemas.microsoft.com/office/word/2010/wordprocessingShape" => "wps",
        "http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing" => "wp14",
        "http://schemas.microsoft.com/office/word/2010/wordprocessingGroup" => "wpg",
        "http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas" => "wpc",
        "http://schemas.microsoft.com/office/word/2010/wordprocessingInk" => "wpi",
        "http://schemas.microsoft.com/office/word/2016/wordml" => "w16",
        "http://schemas.microsoft.com/office/word/2020/wordml/sdtdatahash" => "w16sdtdh",
        "http://schemas.microsoft.com/office/drawing/2010/main" => "a14",
        "http://schemas.microsoft.com/office/drawing/2016/ink" => "aink",
        "http://schemas.microsoft.com/office/drawing/2017/model3d" => "am3d",
        "http://schemas.microsoft.com/office/drawing/2014/chartex" => "cx",
        "http://schemas.microsoft.com/office/drawing/2015/9/8/chartex" => "cx1",
        "http://schemas.microsoft.com/office/drawing/2015/10/21/chartex" => "cx2",
        "http://schemas.microsoft.com/office/drawing/2016/5/9/chartex" => "cx3",
        "http://schemas.microsoft.com/office/drawing/2016/5/10/chartex" => "cx4",
        "http://schemas.microsoft.com/office/drawing/2016/5/11/chartex" => "cx5",
        "http://schemas.microsoft.com/office/drawing/2016/5/12/chartex" => "cx6",
        "http://schemas.microsoft.com/office/drawing/2016/5/13/chartex" => "cx7",
        "http://schemas.microsoft.com/office/drawing/2016/5/14/chartex" => "cx8",
        _ => return None,
    })
}

/// Namespace prefix generator state.
struct State {
    counter: usize,
}

/// A scope in the namespace-prefix stack. Each element inherits its parent
/// scope and may add/override local `xmlns:*`/`xmlns` declarations.
struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    local_uri_to_prefix: HashMap<String, String>,
    local_prefix_to_uri: HashMap<String, String>,
}

impl Scope<'static> {
    fn root() -> Scope<'static> {
        let mut uri_to_prefix = HashMap::new();
        let mut prefix_to_uri = HashMap::new();
        uri_to_prefix.insert(XML_NAMESPACE.to_string(), "xml".to_string());
        prefix_to_uri.insert("xml".to_string(), XML_NAMESPACE.to_string());
        Scope {
            parent: None,
            local_uri_to_prefix: uri_to_prefix,
            local_prefix_to_uri: prefix_to_uri,
        }
    }
}

impl<'a> Scope<'a> {
    fn child(parent: &'a Scope<'a>, dom: &Dom, e: NodeId) -> Scope<'a> {
        let mut uri_to_prefix = HashMap::new();
        let mut prefix_to_uri = HashMap::new();
        // DOM-ITER-01: walk attrs without cloning the attributes() Vec.
        for i in 0..dom.attr_count(e) {
            let (name, value) = dom.attr_at(e, i);
            if !dom.is_namespace_declaration(name) {
                continue;
            }
            let prefix = if name.local_name() == "xmlns" && name.namespace_name().is_empty() {
                ""
            } else {
                name.local_name()
            };
            // Skip reserved / illegal re-declarations; `xml` is always bound.
            if prefix == "xml" || prefix == "xmlns" || value == "http://www.w3.org/2000/xmlns/" {
                continue;
            }
            uri_to_prefix.insert(value.to_string(), prefix.to_string());
            prefix_to_uri.insert(prefix.to_string(), value.to_string());
        }
        Scope {
            parent: Some(parent),
            local_uri_to_prefix: uri_to_prefix,
            local_prefix_to_uri: prefix_to_uri,
        }
    }

    /// Active prefix→URI binding for `prefix` in this scope (local first, then ancestors).
    fn active_prefix_uri(&self, prefix: &str) -> Option<&str> {
        let mut scope = self;
        loop {
            if let Some(uri) = scope.local_prefix_to_uri.get(prefix) {
                return Some(uri);
            }
            match scope.parent {
                Some(parent) => scope = parent,
                None => return None,
            }
        }
    }

    /// Resolve a prefix token to the URI it is bound to in this scope.
    fn uri_for_prefix(&self, prefix: &str) -> Option<&str> {
        self.active_prefix_uri(prefix)
    }

    /// Find the active prefix bound to `uri` in this scope.
    fn prefix_for_uri(&self, uri: &str) -> Option<&str> {
        if uri.is_empty() {
            return Some("");
        }
        let mut scope = self;
        loop {
            if let Some(prefix) = scope.local_uri_to_prefix.get(uri)
                && self.active_prefix_uri(prefix) == Some(uri)
            {
                return Some(prefix);
            }
            match scope.parent {
                Some(parent) => scope = parent,
                None => return None,
            }
        }
    }

    /// Assign (or reuse) a prefix for `uri` in this scope.
    fn assign(&mut self, state: &mut State, uri: &str) -> String {
        if uri.is_empty() {
            return String::new();
        }
        if uri == XML_NAMESPACE {
            return "xml".to_string();
        }
        if let Some(p) = self.prefix_for_uri(uri) {
            return p.to_string();
        }
        let mut p = if let Some(p) = well_known_prefix(uri) {
            if self.active_prefix_uri(p).is_none() {
                p.to_string()
            } else {
                Self::next_generated(state, self)
            }
        } else {
            Self::next_generated(state, self)
        };
        while self.active_prefix_uri(&p).is_some() {
            p = Self::next_generated(state, self);
        }
        self.local_uri_to_prefix.insert(uri.to_string(), p.clone());
        self.local_prefix_to_uri.insert(p.clone(), uri.to_string());
        p
    }

    fn next_generated(state: &mut State, scope: &Scope<'_>) -> String {
        loop {
            let p = format!("ns{}", state.counter);
            state.counter += 1;
            if scope.active_prefix_uri(&p).is_none() {
                return p;
            }
        }
    }
}

/// True for attributes whose value is a list of namespace prefixes that must be
/// rewritten when prefixes are rebound in this scope.
fn is_namespace_prefix_list(name: &XName) -> bool {
    let ns = name.namespace_name();
    let local = name.local_name();
    if ns.is_empty() {
        return local == "Requires";
    }
    ns == MC_NAMESPACE
        && matches!(
            local,
            "Ignorable"
                | "PreserveAttributes"
                | "PreserveElements"
                | "ProcessContent"
                | "MustUnderstand"
        )
}

/// Serialize an element subtree to an XML string (port of `serializeElement`).
pub fn serialize_element(dom: &Dom, el: NodeId) -> String {
    let mut state = State { counter: 0 };
    let root_scope = Scope::root();
    let mut out = String::new();
    emit(dom, el, &root_scope, &mut state, &mut out);
    out
}

fn qname(prefix: &str, name: &XName) -> String {
    if prefix.is_empty() {
        name.local_name().to_string()
    } else {
        format!("{}:{}", prefix, name.local_name())
    }
}

fn emit(dom: &Dom, e: NodeId, parent: &Scope, state: &mut State, out: &mut String) {
    let ename = dom.name(e).expect("emit: non-element node");
    let mut scope = Scope::child(parent, dom, e);

    // First pass: assign prefixes for the element name, all attribute names,
    // and all namespaces referenced by QName-list attribute values.
    // DOM-ITER-01: borrow attr names/values; no attributes() Vec clones.
    scope.assign(state, ename.namespace_name());
    let mut real_attrs: Vec<(&XName, &str)> = Vec::new();
    let mut prefix_list_attrs: Vec<(&XName, &str)> = Vec::new();
    for i in 0..dom.attr_count(e) {
        let (name, value) = dom.attr_at(e, i);
        if dom.is_namespace_declaration(name) {
            continue;
        }
        scope.assign(state, name.namespace_name());
        if is_namespace_prefix_list(name) {
            for token in value.split_whitespace() {
                if let Some(uri) = scope.uri_for_prefix(token).map(|s| s.to_string()) {
                    scope.assign(state, &uri);
                }
            }
            prefix_list_attrs.push((name, value));
            continue;
        }
        real_attrs.push((name, value));
    }

    // Build the attribute string. Namespace declarations come first, then
    // real attributes, then the rewritten QName-list attributes.
    // Sort declarations by prefix so serialization is deterministic.
    let mut attr_str = String::new();
    {
        let mut decls: Vec<(&String, &String)> = scope.local_uri_to_prefix.iter().collect();
        decls.sort_by(|a, b| a.1.cmp(b.1));
        for (uri, prefix) in decls {
            if prefix == "xml" || *uri == XML_NAMESPACE || prefix == "xmlns" {
                continue;
            }
            if uri.is_empty() && !prefix.is_empty() {
                continue;
            }
            if prefix.is_empty() {
                attr_str.push_str(&format!(" xmlns=\"{}\"", escape_attr(uri)));
            } else {
                attr_str.push_str(&format!(" xmlns:{}=\"{}\"", prefix, escape_attr(uri)));
            }
        }
    }

    for (name, value) in real_attrs {
        let prefix = scope
            .prefix_for_uri(name.namespace_name())
            .map(|s| s.to_string())
            .unwrap_or_else(|| scope.assign(state, name.namespace_name()));
        let qn = qname(&prefix, name);
        attr_str.push(' ');
        attr_str.push_str(&qn);
        attr_str.push_str("=\"");
        attr_str.push_str(&escape_attr(value));
        attr_str.push('"');
    }

    for (name, value) in prefix_list_attrs {
        let prefix = scope
            .prefix_for_uri(name.namespace_name())
            .map(|s| s.to_string())
            .unwrap_or_else(|| scope.assign(state, name.namespace_name()));
        let qn = qname(&prefix, name);
        let rewritten = value
            .split_whitespace()
            .map(|token| {
                if let Some(uri) = scope.uri_for_prefix(token) {
                    if let Some(p) = scope.prefix_for_uri(uri) {
                        if p != token {
                            p.to_string()
                        } else {
                            token.to_string()
                        }
                    } else {
                        token.to_string()
                    }
                } else {
                    token.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        attr_str.push(' ');
        attr_str.push_str(&qn);
        attr_str.push_str("=\"");
        attr_str.push_str(&escape_attr(&rewritten));
        attr_str.push('"');
    }

    let tag = {
        let prefix = scope
            .prefix_for_uri(ename.namespace_name())
            .map(|s| s.to_string())
            .unwrap_or_else(|| scope.assign(state, ename.namespace_name()));
        qname(&prefix, &ename)
    };

    // DOM-ITER-01: index children; no nodes() Vec clone per element.
    let n_kids = dom.child_count(e);
    if n_kids == 0 {
        out.push('<');
        out.push_str(&tag);
        out.push_str(&attr_str);
        out.push_str(" />");
        return;
    }

    out.push('<');
    out.push_str(&tag);
    out.push_str(&attr_str);
    out.push('>');
    for i in 0..n_kids {
        let k = dom.child_at(e, i);
        if dom.is_element(k) {
            emit(dom, k, &scope, state, out);
        } else if dom.is_text(k) {
            out.push_str(&escape_text(dom.text_value(k).unwrap_or("")));
        } else if dom.is_comment(k) {
            out.push_str("<!--");
            out.push_str(dom.text_value(k).unwrap_or(""));
            out.push_str("-->");
        } else if dom.is_pi(k) {
            out.push_str("<?");
            out.push_str(dom.pi_target(k).unwrap_or(""));
            if let Some(data) = dom.pi_data(k)
                && !data.is_empty()
            {
                out.push_str(data);
            }
            out.push_str("?>");
        }
    }
    out.push_str("</");
    out.push_str(&tag);
    out.push('>');
}

/// Serialize a whole document (declaration + root), port of `XDocument.toString`.
pub fn serialize_document(dom: &Dom, doc: NodeId) -> String {
    let mut out = String::new();
    if let Some(d) = dom.declaration(doc) {
        out.push_str("<?xml version=\"");
        out.push_str(d.version.as_deref().unwrap_or("1.0"));
        out.push('"');
        if let Some(enc) = &d.encoding {
            out.push_str(&format!(" encoding=\"{enc}\""));
        }
        if let Some(sa) = &d.standalone {
            out.push_str(&format!(" standalone=\"{sa}\""));
        }
        out.push_str("?>");
    }
    for i in 0..dom.child_count(doc) {
        let k = dom.child_at(doc, i);
        if dom.is_element(k) {
            out.push_str(&serialize_element(dom, k));
        } else if dom.is_text(k) {
            out.push_str(&escape_text(dom.text_value(k).unwrap_or("")));
        } else if dom.is_comment(k) {
            out.push_str("<!--");
            out.push_str(dom.text_value(k).unwrap_or(""));
            out.push_str("-->");
        } else if dom.is_pi(k) {
            out.push_str("<?");
            out.push_str(dom.pi_target(k).unwrap_or(""));
            if let Some(data) = dom.pi_data(k)
                && !data.is_empty()
            {
                out.push_str(data);
            }
            out.push_str("?>");
        }
    }
    out
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
