//! Port of `serializeElement` / `XDocument.toString` from `lib/xml-linq.ts`.
//!
//! Collects every namespace used in the subtree, assigns a prefix (conventional
//! OOXML prefix when known, else `nsN`), and emits the matching `xmlns:`
//! declarations on the root, so the output is namespace-valid XML.

use std::collections::{HashMap, HashSet};

use super::{Dom, NodeId, XName};

const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

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

/// Ordered namespace→prefix registry (insertion order preserved for emission).
struct PrefixMap {
    order: Vec<String>, // namespace URIs, in registration order
    by_ns: HashMap<String, String>,
    used: HashSet<String>,
    counter: usize,
}

impl PrefixMap {
    fn new() -> Self {
        PrefixMap {
            order: Vec::new(),
            by_ns: HashMap::new(),
            used: HashSet::new(),
            counter: 0,
        }
    }

    /// Seed a fixed prefix from an existing `xmlns:` declaration.
    fn seed(&mut self, ns: &str, prefix: &str) {
        if prefix.is_empty() || ns.is_empty() || ns == XML_NAMESPACE {
            return;
        }
        if self.used.contains(prefix) || self.by_ns.contains_key(ns) {
            return;
        }
        self.by_ns.insert(ns.to_string(), prefix.to_string());
        self.used.insert(prefix.to_string());
        self.order.push(ns.to_string());
    }

    /// Assign (or look up) a prefix for `ns`.
    fn assign(&mut self, ns: &str) -> String {
        if ns.is_empty() {
            return String::new();
        }
        if ns == XML_NAMESPACE {
            return "xml".to_string();
        }
        if let Some(p) = self.by_ns.get(ns) {
            return p.clone();
        }
        let mut p = match well_known_prefix(ns) {
            Some(p) if !self.used.contains(p) => p.to_string(),
            _ => {
                let mut g;
                loop {
                    g = format!("ns{}", self.counter);
                    self.counter += 1;
                    if !self.used.contains(&g) {
                        break;
                    }
                }
                g
            }
        };
        // (p may be a well-known or generated prefix)
        if self.used.contains(&p) {
            // extremely defensive; should not happen
            loop {
                p = format!("ns{}", self.counter);
                self.counter += 1;
                if !self.used.contains(&p) {
                    break;
                }
            }
        }
        self.by_ns.insert(ns.to_string(), p.clone());
        self.used.insert(p.clone());
        self.order.push(ns.to_string());
        p
    }
}

/// Serialize an element subtree to an XML string (port of `serializeElement`).
pub fn serialize_element(dom: &Dom, el: NodeId) -> String {
    let mut pm = PrefixMap::new();

    // 1. Seed prefixes from EVERY xmlns declaration in the subtree (root-first,
    //    iterative DFS, children pushed in reverse to keep left-to-right order).
    let mut stack = vec![el];
    while let Some(e) = stack.pop() {
        for (name, value) in dom.attributes(e) {
            if !dom.is_namespace_declaration(&name) {
                continue;
            }
            let prefix = if name.local_name() == "xmlns" {
                String::new()
            } else {
                name.local_name().to_string()
            };
            pm.seed(&value, &prefix);
        }
        let kids = dom.nodes(e);
        for &k in kids.iter().rev() {
            if dom.is_element(k) {
                stack.push(k);
            }
        }
    }

    // 2. Pre-walk to register every used namespace (declared on root).
    register(dom, el, &mut pm);

    // 3. Emit.
    let mut out = String::new();
    emit(dom, el, true, &mut pm, &mut out);
    out
}

fn register(dom: &Dom, e: NodeId, pm: &mut PrefixMap) {
    if let Some(name) = dom.name(e) {
        pm.assign(name.namespace_name());
    }
    for (name, _value) in dom.attributes(e) {
        if dom.is_namespace_declaration(&name) {
            continue;
        }
        pm.assign(name.namespace_name());
    }
    for k in dom.nodes(e) {
        if dom.is_element(k) {
            register(dom, k, pm);
        }
    }
}

fn qname(pm: &mut PrefixMap, name: &XName) -> String {
    let ns = name.namespace_name();
    if ns.is_empty() {
        name.local_name().to_string()
    } else {
        format!("{}:{}", pm.assign(ns), name.local_name())
    }
}

fn emit(dom: &Dom, e: NodeId, is_root: bool, pm: &mut PrefixMap, out: &mut String) {
    let ename = dom.name(e).expect("emit: non-element node");
    let tag = qname(pm, &ename);

    // Real attributes (skip stored xmlns declarations; regenerated on root).
    let mut attrs = String::new();
    for (name, value) in dom.attributes(e) {
        if dom.is_namespace_declaration(&name) {
            continue;
        }
        let qn = qname(pm, &name);
        attrs.push(' ');
        attrs.push_str(&qn);
        attrs.push_str("=\"");
        attrs.push_str(&escape_attr(&value));
        attrs.push('"');
    }

    // Declare all collected namespaces on the root.
    if is_root {
        for ns in pm.order.clone() {
            if ns.is_empty() || ns == XML_NAMESPACE {
                continue;
            }
            let prefix = pm.by_ns[&ns].clone();
            attrs.push_str(&format!(" xmlns:{}=\"{}\"", prefix, escape_attr(&ns)));
        }
    }

    let kids = dom.nodes(e);
    if kids.is_empty() {
        out.push('<');
        out.push_str(&tag);
        out.push_str(&attrs);
        out.push_str(" />");
        return;
    }

    out.push('<');
    out.push_str(&tag);
    out.push_str(&attrs);
    out.push('>');
    for k in kids {
        if dom.is_element(k) {
            emit(dom, k, false, pm, out);
        } else if dom.is_text(k) {
            out.push_str(&escape_text(dom.text_value(k).unwrap_or("")));
        } else if dom.is_comment(k) {
            out.push_str("<!--");
            out.push_str(dom.text_value(k).unwrap_or(""));
            out.push_str("-->");
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
    for k in dom.nodes(doc) {
        if dom.is_element(k) {
            out.push_str(&serialize_element(dom, k));
        } else if dom.is_text(k) {
            out.push_str(&escape_text(dom.text_value(k).unwrap_or("")));
        } else if dom.is_comment(k) {
            out.push_str("<!--");
            out.push_str(dom.text_value(k).unwrap_or(""));
            out.push_str("-->");
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
