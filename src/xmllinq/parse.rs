//! Port of `parseXDocument` from `lib/xml-linq.ts` — a small, non-validating,
//! namespace-aware recursive-descent XML parser sufficient for OOXML parts.
//!
//! Transcribed directly (char-by-char) from the TS rather than driven by
//! quick-xml, so the parse/serialize pair round-trips byte-identically to the
//! TypeScript oracle (the basis of the M1.8 round-trip gate).

use std::collections::HashMap;

use super::{Dom, NodeId, XDeclaration, XName, XNamespace};

/// Parse a full XML document (with optional prolog) into a Document node in `dom`.
pub fn parse_xdocument(dom: &mut Dom, xml: &str) -> NodeId {
    // Strip a leading UTF-8 BOM (U+FEFF): char::is_whitespace() excludes it,
    // so skip_ws() cannot, and an unstripped BOM leaves the document with no root.
    let xml = xml.strip_prefix('\u{feff}').unwrap_or(xml);
    let chars: Vec<char> = xml.chars().collect();
    let mut p = Parser {
        dom,
        c: chars,
        pos: 0,
    };
    p.parse_document()
}

struct Parser<'a> {
    dom: &'a mut Dom,
    c: Vec<char>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn len(&self) -> usize {
        self.c.len()
    }

    fn cur(&self) -> char {
        if self.pos < self.len() {
            self.c[self.pos]
        } else {
            '\0'
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        let sc: Vec<char> = s.chars().collect();
        if self.pos + sc.len() > self.len() {
            return false;
        }
        for (k, ch) in sc.iter().enumerate() {
            if self.c[self.pos + k] != *ch {
                return false;
            }
        }
        true
    }

    /// Index of substring `s` at or after `from`, in char units.
    fn index_of(&self, s: &str, from: usize) -> Option<usize> {
        let sc: Vec<char> = s.chars().collect();
        if sc.is_empty() || from > self.len() {
            return None;
        }
        let mut i = from;
        while i + sc.len() <= self.len() {
            if self.c[i..i + sc.len()] == sc[..] {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    fn slice(&self, start: usize, end: usize) -> String {
        let end = end.min(self.len());
        if start >= end {
            return String::new();
        }
        self.c[start..end].iter().collect()
    }

    fn skip_ws(&mut self) {
        while self.pos < self.len() && self.cur().is_whitespace() {
            self.pos += 1;
        }
    }

    /// readName — consume until whitespace, '/', '>', or '='.
    fn read_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.len() {
            let ch = self.cur();
            if ch.is_whitespace() || ch == '/' || ch == '>' || ch == '=' {
                break;
            }
            self.pos += 1;
        }
        self.slice(start, self.pos)
    }

    fn parse_document(&mut self) -> NodeId {
        let doc = self.dom.new_document();
        let len = self.len();
        while self.pos < len {
            self.skip_ws();
            if self.starts_with("<?xml") {
                // Only the real XML declaration looks like `<?xml` followed by
                // whitespace or `?>`. `<?xml-stylesheet` etc. are PIs.
                let after = self.pos + 5;
                let is_declaration = after >= len
                    || self.c.get(after).map_or(true, |&ch| ch.is_whitespace() || ch == '?');
                if is_declaration {
                    let end = self.index_of("?>", self.pos);
                    let decl_str = self.slice(self.pos, end.unwrap_or(len));
                    let decl = parse_declaration(&decl_str);
                    self.dom.set_declaration(doc, Some(decl));
                    self.pos = end.map(|e| e + 2).unwrap_or(len);
                    continue;
                }
            }
            if self.starts_with("<?") {
                let pi = self.parse_pi_node();
                self.dom.add(doc, pi);
                continue;
            }
            if self.starts_with("<!--") {
                let end = self.index_of("-->", self.pos);
                let body = self.slice(self.pos + 4, end.unwrap_or(len));
                let cm = self.dom.new_comment(&body);
                self.dom.add(doc, cm);
                self.pos = end.map(|e| e + 3).unwrap_or(len);
                continue;
            }
            if self.starts_with("<!") {
                // DOCTYPE — skip
                let end = self.index_of(">", self.pos);
                self.pos = end.map(|e| e + 1).unwrap_or(len);
                continue;
            }
            if self.cur() == '<' {
                let scope: HashMap<String, String> = HashMap::new();
                let el = self.parse_element(&scope);
                self.dom.add(doc, el);
                continue;
            }
            break;
        }
        doc
    }

    fn parse_element(&mut self, ns_scope: &HashMap<String, String>) -> NodeId {
        // at '<'
        self.pos += 1; // consume '<'
        let raw_name = self.read_name();
        let mut raw_attrs: Vec<(String, String)> = Vec::new();
        loop {
            self.skip_ws();
            if self.cur() == '/' || self.cur() == '>' {
                break;
            }
            let aname = self.read_name();
            self.skip_ws();
            let mut avalue = String::new();
            if self.cur() == '=' {
                self.pos += 1; // '='
                self.skip_ws();
                let quote = self.cur();
                self.pos += 1; // opening quote
                let vstart = self.pos;
                while self.pos < self.len() && self.cur() != quote {
                    self.pos += 1;
                }
                avalue = self.slice(vstart, self.pos);
                self.pos += 1; // closing quote
            }
            raw_attrs.push((aname, unescape_xml_text(&avalue)));
        }

        // Build local namespace scope.
        let mut local_scope = ns_scope.clone();
        for (name, value) in &raw_attrs {
            if name == "xmlns" {
                local_scope.insert(String::new(), value.clone());
            } else if let Some(prefix) = name.strip_prefix("xmlns:") {
                local_scope.insert(prefix.to_string(), value.clone());
            }
        }

        let el_name = resolve(&raw_name, false, &local_scope);
        let el = self.dom.new_element(el_name);
        for (name, value) in &raw_attrs {
            let an = resolve(name, true, &local_scope);
            self.dom.set_attribute_value(el, &an, Some(value));
        }

        self.skip_ws();
        if self.cur() == '/' {
            self.pos += 2; // '/' '>'
            return el;
        }
        self.pos += 1; // '>'

        let len = self.len();
        while self.pos < len {
            if self.starts_with("</") {
                self.pos += 2;
                self.read_name();
                self.skip_ws();
                if self.cur() == '>' {
                    self.pos += 1;
                }
                break;
            }
            if self.starts_with("<!--") {
                let end = self.index_of("-->", self.pos);
                let body = self.slice(self.pos + 4, end.unwrap_or(len));
                let cm = self.dom.new_comment(&body);
                self.dom.add(el, cm);
                self.pos = end.map(|e| e + 3).unwrap_or(len);
                continue;
            }
            if self.starts_with("<![CDATA[") {
                let end = self.index_of("]]>", self.pos);
                let body = self.slice(self.pos + 9, end.unwrap_or(len));
                let t = self.dom.new_text(&body);
                self.dom.add(el, t);
                self.pos = end.map(|e| e + 3).unwrap_or(len);
                continue;
            }
            if self.starts_with("<?") {
                let pi = self.parse_pi_node();
                self.dom.add(el, pi);
                continue;
            }
            if self.cur() == '<' {
                let child = self.parse_element(&local_scope);
                self.dom.add(el, child);
                continue;
            }
            let tstart = self.pos;
            while self.pos < len && self.cur() != '<' {
                self.pos += 1;
            }
            let raw = self.slice(tstart, self.pos);
            if !raw.is_empty() {
                let t = self.dom.new_text(&unescape_xml_text(&raw));
                self.dom.add(el, t);
            }
        }
        el
    }

    /// Parse a processing instruction (`<?target data?>`) into a `Pi` node.
    /// `self.pos` is on `<?`.
    fn parse_pi_node(&mut self) -> NodeId {
        let len = self.len();
        self.pos += 2; // consume "<?"
        let target_start = self.pos;
        while self.pos < len {
            let ch = self.cur();
            if ch.is_whitespace() || ch == '?' || ch == '>' {
                break;
            }
            self.pos += 1;
        }
        let target = self.slice(target_start, self.pos);
        let end = self.index_of("?>", self.pos);
        let data = self.slice(self.pos, end.unwrap_or(len));
        self.pos = end.map(|e| e + 2).unwrap_or(len);
        self.dom.new_pi(&target, &data)
    }
}

/// Resolve a qualified name against the in-scope namespaces. Mirrors the TS
/// `resolve` closure exactly.
fn resolve(qn: &str, is_attr: bool, local_scope: &HashMap<String, String>) -> XName {
    if let Some(colon) = qn.find(':') {
        let prefix = &qn[..colon];
        let local = &qn[colon + 1..];
        if prefix == "xmlns" {
            return XNamespace::xmlns().name(local);
        }
        if prefix == "xml" {
            return XNamespace::xml().name(local);
        }
        return match local_scope.get(prefix) {
            Some(ns) => XNamespace::get(ns).name(local),
            None => XName::get(local, ""),
        };
    }
    if qn == "xmlns" {
        return XName::get("xmlns", "");
    }
    // Attributes default to no namespace; elements use the default (xmlns="…").
    let def_ns: Option<&String> = if is_attr { None } else { local_scope.get("") };
    match def_ns {
        Some(ns) if !ns.is_empty() => XNamespace::get(ns).name(qn),
        _ => XName::get(qn, ""),
    }
}

/// Parse the `<?xml ...?>` declaration body.
fn parse_declaration(decl: &str) -> XDeclaration {
    XDeclaration {
        version: Some(extract_pseudo_attr(decl, "version").unwrap_or_else(|| "1.0".to_string())),
        encoding: extract_pseudo_attr(decl, "encoding"),
        standalone: extract_pseudo_attr(decl, "standalone"),
    }
}

/// Extract `key="value"` / `key='value'` from a declaration string.
fn extract_pseudo_attr(s: &str, key: &str) -> Option<String> {
    let idx = s.find(key)?;
    let rest = &s[idx + key.len()..];
    let eq = rest.find('=')?;
    let after = rest[eq + 1..].trim_start();
    let mut chars = after.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = after[1..].find(quote)?;
    Some(after[1..1 + end].to_string())
}

/// Port of `unescapeXmlText` — decode entity / character references.
pub fn unescape_xml_text(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' {
            // find ';'
            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
                let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
                if let Some(decoded) = decode_entity(&body) {
                    out.push_str(&decoded);
                    i = i + 1 + semi_rel + 1;
                    continue;
                }
            }
            out.push('&');
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn decode_entity(body: &str) -> Option<String> {
    let b: Vec<char> = body.chars().collect();
    if b.is_empty() {
        return None;
    }
    if b[0] == '#' {
        let code = if b.len() > 1 && (b[1] == 'x' || b[1] == 'X') {
            u32::from_str_radix(&body[2..], 16).ok()?
        } else {
            body[1..].parse::<u32>().ok()?
        };
        return char::from_u32(code).map(|c| c.to_string());
    }
    // named entity — must be all ASCII letters to match the TS regex `[a-zA-Z]+`
    if !b.iter().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    match body {
        "lt" => Some("<".to_string()),
        "gt" => Some(">".to_string()),
        "amp" => Some("&".to_string()),
        "quot" => Some("\"".to_string()),
        "apos" => Some("'".to_string()),
        _ => None,
    }
}
