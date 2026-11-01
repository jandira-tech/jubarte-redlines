// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! XML parser for the arena DOM, ported from `parseXDocument` in
//! `lib/xml-linq.ts`.
//!
//! The parser is small, non-validating, and namespace-aware — sufficient
//! for OOXML parts, which is all the engine needs. It is transcribed
//! directly from the TypeScript rather than driven by `quick-xml`, so the
//! parse / serialize pair round-trips byte-identically to the TS oracle
//! (the basis of the M1.8 round-trip gate). One deliberate XML 1.0
//! correction over the TS port: raw attribute values and text runs are
//! whitespace / line-end normalized (§3.3.3, §2.11) *before* entity
//! decoding, so the DOM holds the value a conforming reader sees (literal
//! TAB/CR/LF in an attribute are spaces, a literal CR in text is LF;
//! character references keep the characters they name).
//!
//! # Entry points
//!
//! - [`parse_xdocument`] — string → DOM (`Document` node in the supplied
//!   arena). The permissive path used by every code that reads an OOXML
//!   part: tolerates missing closing tags, unquoted / mixed-quote
//!   attributes, and similar WW-isms in real-world .docx files.
//!
//!   It does not enforce input policy. Its input should have passed a
//!   gate first: [`validate_xml_with_limits`], or package admission with
//!   the caller's active limits. A caller holding unchecked XML uses
//!   [`try_parse_xdocument`], which runs [`validate_xml`] before it
//!   builds any node.
//! - [`unescape_xml_text`] — decode XML entities / character references
//!   in attribute / text bodies.
//! - [`validate_xml`] — structural admission for a part *before* the
//!   permissive parser reads it: well-formed nesting, a single root, no
//!   DTD, only the five predefined or numeric entities, and nesting no
//!   deeper than 256. This is not a schema validator.

use std::collections::HashMap;

use crate::admission::InputLimits;

use super::{Dom, NodeId, XDeclaration, XName, XNamespace};

// ── tag policy budgets (N01/N02) ────────────────────────────────────────────
// One policy is shared by admission, strict validation and DOM construction.
// Defaults come from the measured Word corpus, with headroom for extensions.

/// Attributes (namespace declarations included) on one element.
pub(crate) const MAX_ELEMENT_ATTRIBUTES: usize = 1024;
/// Bytes in an element name (the existing element-name budget).
pub(crate) const MAX_NAME_BYTES: usize = 4096;
/// UTF-8 bytes in a qualified attribute name.
pub(crate) const MAX_ATTRIBUTE_NAME_BYTES: usize = 1024;
/// Bytes in one raw or decoded attribute value (64 MiB).
pub(crate) const MAX_ATTRIBUTE_VALUE_BYTES: usize = 64 * 1024 * 1024;
/// Complete start-tag bytes, including whitespace, quotes and delimiters.
pub(crate) const MAX_TAG_ATTRIBUTE_BYTES: usize = 128 * 1024 * 1024;
/// Bytes in one namespace URI.
pub(crate) const MAX_NAMESPACE_URI_BYTES: usize = 8192;
/// Bytes in one namespace prefix.
pub(crate) const MAX_NAMESPACE_PREFIX_BYTES: usize = 256;
/// Bindings live in one lexical scope (all levels, shadowed included).
pub(crate) const MAX_LIVE_NS_BINDINGS: usize = 256;
/// Total prefix+URI bytes declared by xmlns attributes in one document.
pub(crate) const MAX_NS_DECLARED_BYTES: usize = 1024 * 1024;

// N02 mechanism counter: bytes of (prefix, uri) pairs the parser's
// namespace scope has recorded. The scoped-overlay scope only ever records
// what an element declares; a parent-map clone would inflate this by the
// live-set size on every nested declaration. Per thread, so tests running
// side by side in one `cargo test` process do not count each other's work.
#[cfg(test)]
thread_local! {
    pub(crate) static NS_SCOPE_DECLARED_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Add to this thread's [`NS_SCOPE_DECLARED_BYTES`].
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
fn bump_ns_scope_declared_bytes(by: usize) {
    NS_SCOPE_DECLARED_BYTES.with(|c| c.set(c.get() + by));
}

/// Parse an XML string into a Document node allocated into `dom`. The
/// parser is permissive on real-world OOXML inputs (truncated closing
/// tags, missing `?>` terminators, mixed-quote attribute values) — see
/// the module-level docs. A leading UTF-8 BOM is stripped before parsing
/// so a BOM-prefixed input still yields a document with a root.
///
/// This parser does not enforce input policy or panic on policy violations.
/// It recurses once per nesting level: a caller that admitted nothing first
/// bounds the depth with `nests_deeper` and `PERMISSIVE_MAX_DEPTH`.
/// Run [`validate_xml_with_limits`] (or package admission) with the active
/// limits first, or call [`try_parse_xdocument_with_limits`] to receive errors.
/// Attributes retain their order; repeated expanded names keep the first
/// position and the last value. Bulk attribute insertion is linear.
///
/// `xml` may be a complete document (prolog + root + children) or a
/// fragment; the result is wrapped in a fresh Document node.
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

/// Checked parse: enforce [`validate_xml`] (well-formedness plus the tag
/// and namespace budgets) before the permissive parser builds the DOM.
/// Use this on any path where a refusal is meaningful; callers that must
/// stay permissive (the port's contract) keep [`parse_xdocument`].
///
/// # Errors
///
/// Returns the [`validate_xml`] error string unchanged.
pub fn try_parse_xdocument(dom: &mut Dom, xml: &str) -> Result<NodeId, String> {
    try_parse_xdocument_with_limits(dom, xml, InputLimits::default())
}

/// Checked parse under explicit input budgets. No DOM node is built until
/// validation succeeds; only XML-related fields of `limits` are used.
///
/// # Errors
///
/// Returns the same error strings as [`validate_xml_with_limits`].
pub fn try_parse_xdocument_with_limits(
    dom: &mut Dom,
    xml: &str,
    limits: InputLimits,
) -> Result<NodeId, String> {
    validate_xml_with_limits(xml, limits)?;
    Ok(parse_xdocument(dom, xml))
}

/// The deepest nesting a package part may have where nothing admitted it
/// first: the agent view reads that deep, and [`parse_xdocument`] recurses
/// through it safely.
pub(crate) const PERMISSIVE_MAX_DEPTH: usize = 512;

/// Whether `xml` nests deeper than `limit` elements, read the way
/// [`parse_xdocument`] reads it. The scan shares the parser's start-tag
/// reader (names, quoted values) and walks its grammar (comments, CDATA,
/// processing instructions, a `<!x>` inside an element as an element), so
/// markup a strict reader stops at counts as deep as the parser would
/// recurse into it (#1353 review). It keeps a counter, not a stack.
pub(crate) fn nests_deeper(xml: &str, limit: usize) -> bool {
    let xml = xml.strip_prefix('\u{feff}').unwrap_or(xml);
    let mut dom = Dom::new();
    let mut p = Parser {
        dom: &mut dom,
        c: xml.chars().collect(),
        pos: 0,
    };
    p.nests_deeper(limit)
}

struct Parser<'a> {
    dom: &'a mut Dom,
    c: Vec<char>,
    pos: usize,
}

/// In-scope namespace bindings as an overlay stack (N02): one `HashMap`
/// holds the live prefix → URI bindings, and each element's declarations
/// push overrides that `pop` restores — where the old port cloned the
/// entire parent map on every element that declared an `xmlns`, this pays
/// only the declared bytes, and nested rebinding never re-copies ancestors.
/// `pop(mark)` restores exactly the bindings pushed since `mark`, in
/// reverse order, so shadowed outer bindings survive intact.
#[derive(Default)]
struct NsScope {
    bindings: HashMap<String, String>,
    /// (prefix, value it replaced if any, declared bytes of the push) — a
    /// per-element mark is an index into this stack.
    undo: Vec<(String, Option<String>, usize)>,
    /// Prefix+URI bytes of every binding currently stacked (shadowed
    /// bindings still hold memory, so they stay counted until popped).
    live_bytes: usize,
    /// Bindings currently stacked, shadowed ones included.
    live_bindings: usize,
}

impl NsScope {
    fn get(&self, prefix: &str) -> Option<&String> {
        self.bindings.get(prefix)
    }

    fn mark(&self) -> usize {
        self.undo.len()
    }

    /// Record `prefix → uri` as a declaration owned by the current element.
    fn push(&mut self, prefix: &str, uri: &str) {
        let bytes = prefix.len() + uri.len();
        let prior = self.bindings.insert(prefix.to_string(), uri.to_string());
        self.undo.push((prefix.to_string(), prior, bytes));
        self.live_bytes += bytes;
        self.live_bindings += 1;
        #[cfg(test)]
        bump_ns_scope_declared_bytes(bytes);
    }

    /// Restore the scope to `mark`, undoing every push since then.
    fn pop(&mut self, mark: usize) {
        while self.undo.len() > mark {
            let Some((prefix, prior, bytes)) = self.undo.pop() else {
                break;
            };
            match prior {
                Some(old) => {
                    self.bindings.insert(prefix, old);
                }
                None => {
                    self.bindings.remove(&prefix);
                }
            }
            self.live_bytes = self.live_bytes.saturating_sub(bytes);
            self.live_bindings = self.live_bindings.saturating_sub(1);
        }
    }
}

/// The checked gate's per-document tag policy (N01/N02): tag width,
/// attribute counts and sizes, namespace budgets, and expanded-name
/// duplicate detection. Shared by `validate_xml` and the package admission
/// scan; `try_parse_xdocument` gets it through `validate_xml`.
pub(crate) struct TagPolicy {
    limits: InputLimits,
    scope: NsScope,
    /// Undo marks of open elements, parallel to the reader's depth.
    marks: Vec<usize>,
    /// Cumulative declared prefix+URI bytes across the document.
    declared_bytes: usize,
    /// Read attributes with quick-xml's HTML rules, which also accept
    /// unquoted (`w:val=on`) and valueless (`compact`) attributes.
    html_attributes: bool,
}

impl TagPolicy {
    /// The strict gate's policy: every attribute must be well-formed XML,
    /// quoted and with a value.
    pub(crate) fn new(limits: InputLimits) -> Self {
        TagPolicy {
            limits,
            scope: NsScope::default(),
            marks: Vec::new(),
            declared_bytes: 0,
            html_attributes: false,
        }
    }

    /// The package admission scan's policy. The scan admits parts for the
    /// permissive parser, which reads unquoted and valueless attributes, so
    /// those pass here; the budgets and the duplicate checks still apply to
    /// them.
    pub(crate) fn tolerant(limits: InputLimits) -> Self {
        TagPolicy {
            html_attributes: true,
            ..Self::new(limits)
        }
    }

    /// Check one open tag and push its declarations until [`Self::close`].
    ///
    /// # Errors
    ///
    /// A `String` naming the budget or rule the tag violates.
    pub(crate) fn open(&mut self, tag: &quick_xml::events::BytesStart<'_>) -> Result<(), String> {
        self.enter(tag, 2)
    }

    fn enter(
        &mut self,
        tag: &quick_xml::events::BytesStart<'_>,
        delimiters: usize,
    ) -> Result<(), String> {
        let mark = self.scope.mark();
        self.check_tag(tag, delimiters)
            .map_err(|why| format!("element <{}>: {why}", tag.name().as_ref()))
            .inspect_err(|_| self.scope.pop(mark))?;
        self.marks.push(mark);
        Ok(())
    }

    /// Check a self-closing tag; its declarations never outlive the tag.
    ///
    /// # Errors
    ///
    /// As [`Self::open`].
    pub(crate) fn empty(&mut self, tag: &quick_xml::events::BytesStart<'_>) -> Result<(), String> {
        let mark = self.scope.mark();
        let result = self
            .check_tag(tag, 3)
            .map_err(|why| format!("element <{}>: {why}", tag.name().as_ref()));
        self.scope.pop(mark);
        result
    }

    /// Pop the declarations of the innermost open element. Safe to call
    /// once per `End` event the reader reports.
    pub(crate) fn close(&mut self) {
        if let Some(mark) = self.marks.pop() {
            self.scope.pop(mark);
        }
    }

    fn check_tag(
        &mut self,
        tag: &quick_xml::events::BytesStart<'_>,
        delimiters: usize,
    ) -> Result<(), String> {
        let tag_name = tag.name();
        if tag_name.as_ref().len() > MAX_NAME_BYTES {
            return Err(format!("XML name exceeds {MAX_NAME_BYTES} bytes"));
        }
        let mut count = 0usize;
        if tag.len().saturating_add(delimiters) > self.limits.max_start_tag_bytes {
            return Err(format!(
                "start tag exceeds {} bytes",
                self.limits.max_start_tag_bytes
            ));
        }
        // Raw keys for a second pass after the declarations land in scope.
        let mut seen: Vec<String> = Vec::new();
        // Both iterators refuse a repeated raw name (`AttrError::Duplicated`).
        let attributes = if self.html_attributes {
            tag.html_attributes()
        } else {
            tag.attributes()
        };
        for attribute in attributes {
            let attribute = match attribute {
                Ok(attribute) => attribute,
                Err(error @ quick_xml::events::attributes::AttrError::Duplicated(..)) => {
                    return Err(error.to_string());
                }
                Err(error) => return Err(error.to_string()),
            };
            count += 1;
            self.check_attribute(count, attribute.key.as_ref().len(), attribute.value.len())?;
            let key = attribute.key.as_ref().to_string();
            // A valueless attribute normalizes to an empty value.
            let value = attribute
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map_err(|e| e.to_string())?;
            self.check_attribute(count, key.len(), value.len())?;
            if key == "xmlns" {
                self.declare("", &value)?;
            } else if let Some(prefix) = key.strip_prefix("xmlns:") {
                self.declare(prefix, &value)?;
            }
            seen.push(key);
        }
        // Two differently-spelled attributes that expand to the same name
        // collapse in the DOM (and confuse every downstream differ) —
        // malformed XML under Namespaces in XML 1.0 section 6.3.
        let mut expanded = std::collections::HashSet::with_capacity(seen.len());
        for key in &seen {
            if !expanded.insert(self.expand(key)) {
                return Err(format!("attribute {key} repeats an expanded name"));
            }
        }
        Ok(())
    }

    /// Enforce the active width budgets at the checked boundary.
    fn check_attribute(
        &self,
        count: usize,
        name_bytes: usize,
        value_bytes: usize,
    ) -> Result<(), String> {
        if count > self.limits.max_element_attributes {
            return Err(format!(
                "element exceeds {} attributes",
                self.limits.max_element_attributes
            ));
        }
        if name_bytes > self.limits.max_attribute_name_bytes {
            return Err(format!(
                "attribute name exceeds {} bytes",
                self.limits.max_attribute_name_bytes
            ));
        }
        if value_bytes > self.limits.max_attribute_value_bytes {
            return Err(format!(
                "attribute value exceeds {} bytes",
                self.limits.max_attribute_value_bytes
            ));
        }
        Ok(())
    }

    /// Push one `xmlns` declaration under the namespace budgets.
    fn declare(&mut self, prefix: &str, uri: &str) -> Result<(), String> {
        if prefix.len() > MAX_NAMESPACE_PREFIX_BYTES {
            return Err(format!(
                "namespace prefix exceeds {MAX_NAMESPACE_PREFIX_BYTES} bytes"
            ));
        }
        if uri.len() > MAX_NAMESPACE_URI_BYTES {
            return Err(format!(
                "namespace URI exceeds {MAX_NAMESPACE_URI_BYTES} bytes"
            ));
        }
        if self.scope.live_bindings + 1 > MAX_LIVE_NS_BINDINGS {
            return Err(format!(
                "in-scope namespace bindings exceed {MAX_LIVE_NS_BINDINGS}"
            ));
        }
        self.declared_bytes += prefix.len() + uri.len();
        if self.declared_bytes > MAX_NS_DECLARED_BYTES {
            return Err(format!(
                "namespace declarations exceed {MAX_NS_DECLARED_BYTES} bytes"
            ));
        }
        self.scope.push(prefix, uri);
        Ok(())
    }

    /// Expand an attribute name the way the DOM parser's `resolve` does:
    /// `xmlns:*` and `xml:*` map to their reserved URIs, other prefixes go
    /// through the in-scope bindings (unbound → empty), and unprefixed
    /// attributes never take the default namespace.
    fn expand<'a>(&self, key: &'a str) -> (String, &'a str) {
        if let Some(colon) = key.find(':') {
            let (prefix, local) = (&key[..colon], &key[colon + 1..]);
            let uri = match prefix {
                "xmlns" => "http://www.w3.org/2000/xmlns/",
                "xml" => "http://www.w3.org/XML/1998/namespace",
                _ => self.scope.get(prefix).map_or("", String::as_str),
            };
            return (uri.to_string(), local);
        }
        (String::new(), key)
    }
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

    /// True if the buffer at `pos` matches `s` (char-for-char).
    /// PARSE-01b: no per-call `Vec<char>` allocation — walk `s.chars()` directly
    /// against the already-materialized input buffer.
    fn starts_with(&self, s: &str) -> bool {
        for (off, ch) in s.chars().enumerate() {
            let i = self.pos + off;
            if i >= self.len() || self.c[i] != ch {
                return false;
            }
        }
        true
    }

    /// Index of substring `s` at or after `from`, in char units.
    /// PARSE-01b: match without allocating a needle `Vec<char>` each call.
    fn index_of(&self, s: &str, from: usize) -> Option<usize> {
        if s.is_empty() || from > self.len() {
            return None;
        }
        // Call sites use short ASCII needles (`</`, `<!--`, `]]>`, `?>`, …).
        let needle_chars: usize = s.chars().count();
        let end = self.len().saturating_sub(needle_chars);
        for i in from..=end {
            if i + needle_chars > self.len() {
                break;
            }
            let mut ok = true;
            for (off, ch) in s.chars().enumerate() {
                if self.c[i + off] != ch {
                    ok = false;
                    break;
                }
            }
            if ok {
                return Some(i);
            }
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
        let (start, end) = self.name_span();
        self.slice(start, end)
    }

    /// The span [`Self::read_name`] consumes.
    fn name_span(&mut self) -> (usize, usize) {
        let start = self.pos;
        while self.pos < self.len() {
            let ch = self.cur();
            if ch.is_whitespace() || ch == '/' || ch == '>' || ch == '=' {
                break;
            }
            self.pos += 1;
        }
        (start, self.pos)
    }

    /// A start tag from its `<` through `>` or `/>`: the raw name's span,
    /// each attribute's raw name and value spans handed to `attribute` in
    /// order, and whether the tag closed itself. Both the parser and the
    /// depth scan read tags here, so they agree where every tag ends.
    fn start_tag(
        &mut self,
        mut attribute: impl FnMut(&Self, (usize, usize), (usize, usize)),
    ) -> ((usize, usize), bool) {
        self.pos += 1; // consume '<'
        let name = self.name_span();
        loop {
            self.skip_ws();
            if self.pos >= self.len() || self.cur() == '/' || self.cur() == '>' {
                break;
            }
            let aname = self.name_span();
            if aname.0 == aname.1 {
                // Malformed delimiter (`<p a='x' ?>`): the permissive parser
                // must still advance instead of spinning on the same byte.
                break;
            }
            self.skip_ws();
            let mut value = (self.pos, self.pos);
            if self.cur() == '=' {
                self.pos += 1; // '='
                self.skip_ws();
                let quote = self.cur();
                self.pos += 1; // opening quote
                let vstart = self.pos;
                while self.pos < self.len() && self.cur() != quote {
                    self.pos += 1;
                }
                value = (vstart, self.pos);
                self.pos += 1; // closing quote
            }
            attribute(self, aname, value);
        }
        self.skip_ws();
        if self.cur() == '/' {
            self.pos += 2; // '/' '>'
            return (name, true);
        }
        self.pos += 1; // '>'
        (name, false)
    }

    /// [`nests_deeper`]'s walk: [`Self::parse_document`]'s loop at the top
    /// level, [`Self::parse_element`]'s inside an element, with a depth
    /// count where they recurse.
    fn nests_deeper(&mut self, limit: usize) -> bool {
        let len = self.len();
        let mut depth = 0usize;
        while self.pos < len {
            if depth == 0 {
                self.skip_ws();
                if self.starts_with("<?xml") {
                    let after = self.pos + 5;
                    let is_declaration = after >= len
                        || self
                            .c
                            .get(after)
                            .is_none_or(|&ch| ch.is_whitespace() || ch == '?');
                    if is_declaration {
                        let end = self.index_of("?>", self.pos);
                        self.pos = end.map(|e| e + 2).unwrap_or(len);
                        continue;
                    }
                }
                if self.starts_with("<?") {
                    self.pi_spans();
                    continue;
                }
                if self.starts_with("<!--") {
                    self.pos = self.index_of("-->", self.pos).map_or(len, |e| e + 3);
                    continue;
                }
                if self.starts_with("<!") {
                    self.pos = self.index_of(">", self.pos).map_or(len, |e| e + 1);
                    continue;
                }
                if self.cur() != '<' {
                    break;
                }
            } else {
                if self.starts_with("</") {
                    self.pos += 2;
                    self.name_span();
                    self.skip_ws();
                    if self.cur() == '>' {
                        self.pos += 1;
                    }
                    depth -= 1;
                    continue;
                }
                if self.starts_with("<!--") {
                    self.pos = self.index_of("-->", self.pos).map_or(len, |e| e + 3);
                    continue;
                }
                if self.starts_with("<![CDATA[") {
                    self.pos = self.index_of("]]>", self.pos).map_or(len, |e| e + 3);
                    continue;
                }
                if self.starts_with("<?") {
                    self.pi_spans();
                    continue;
                }
                if self.cur() != '<' {
                    while self.pos < len && self.cur() != '<' {
                        self.pos += 1;
                    }
                    continue;
                }
            }
            let (_, closed) = self.start_tag(|_, _, _| {});
            if !closed {
                depth += 1;
                if depth > limit {
                    return true;
                }
            }
        }
        false
    }

    fn parse_document(&mut self) -> NodeId {
        let doc = self.dom.new_document();
        // N02: one overlay stack serves every top-level element; marks are
        // balanced, so it is empty again between roots.
        let mut scope = NsScope::default();
        let len = self.len();
        while self.pos < len {
            self.skip_ws();
            if self.starts_with("<?xml") {
                // Only the real XML declaration looks like `<?xml` followed by
                // whitespace or `?>`. `<?xml-stylesheet` etc. are PIs.
                let after = self.pos + 5;
                let is_declaration = after >= len
                    || self
                        .c
                        .get(after)
                        .is_none_or(|&ch| ch.is_whitespace() || ch == '?');
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
                // DOCTYPE — skip it whole: its internal subset can hold a
                // `>` inside a quoted entity value or between its brackets,
                // and ending at the first `>` would read the rest of the
                // subset as content (#2015).
                self.pos = self.declaration_end(len);
                continue;
            }
            if self.cur() == '<' {
                let el = self.parse_element(&mut scope);
                self.dom.add(doc, el);
                continue;
            }
            break;
        }
        doc
    }

    /// Index just past the `<!…>` declaration at `self.pos`: the first
    /// `>` outside quotes and outside the `[…]` internal subset, or `len`
    /// when it never closes.
    fn declaration_end(&self, len: usize) -> usize {
        let mut quote = None;
        let mut subset = 0usize;
        let mut at = self.pos + 2;
        while at < len {
            let c = self.c[at];
            match (quote, c) {
                (Some(q), _) if c == q => quote = None,
                (Some(_), _) => {}
                (None, '"' | '\'') => quote = Some(c),
                (None, '[') => subset += 1,
                (None, ']') => subset = subset.saturating_sub(1),
                (None, '>') if subset == 0 => return at + 1,
                (None, _) => {}
            }
            at += 1;
        }
        len
    }

    fn parse_element(&mut self, scope: &mut NsScope) -> NodeId {
        // at '<'
        let mut raw_attrs: Vec<(String, String)> = Vec::new();
        let ((name_start, name_end), closed) = self.start_tag(|p, (a0, a1), (v0, v1)| {
            // Normalize the RAW slice before entity decoding: literal
            // whitespace reads as space (§3.3.3) but character references
            // keep the characters they name.
            raw_attrs.push((
                p.slice(a0, a1),
                unescape_xml_text(&normalize_raw_attribute_whitespace(p.slice(v0, v1))),
            ));
        });
        let raw_name = self.slice(name_start, name_end);

        // N02: this element's xmlns / xmlns:* attributes push onto the
        // overlay scope (marks restore them on every return path below) —
        // no parent-map clone, and shadowing is last-declaration-wins, as
        // the cloned HashMap gave before.
        let mark = scope.mark();
        for (name, value) in &raw_attrs {
            if name == "xmlns" {
                scope.push("", value);
            } else if let Some(prefix) = name.strip_prefix("xmlns:") {
                scope.push(prefix, value);
            }
        }

        let el_name = resolve(&raw_name, false, scope);
        let el = self.dom.new_element(el_name);
        // N01: one bulk insert — a hash index of the expanded names — rather
        // than N calls each re-scanning the growing attribute Vec.
        let attrs: Vec<(XName, String)> = raw_attrs
            .iter()
            .map(|(name, value)| (resolve(name, true, scope), value.clone()))
            .collect();
        self.dom.set_attributes(el, attrs);

        if closed {
            scope.pop(mark);
            return el;
        }

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
                // Line-end normalization (§2.11) applies inside CDATA too;
                // entities do not.
                let t = self.dom.new_text(&normalize_raw_text_end_of_line(body));
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
                let child = self.parse_element(scope);
                self.dom.add(el, child);
                continue;
            }
            let tstart = self.pos;
            while self.pos < len && self.cur() != '<' {
                self.pos += 1;
            }
            // Normalize the RAW slice before entity decoding: a literal
            // CR/CRLF reads as LF (§2.11) but a &#13; reference keeps CR.
            let raw = normalize_raw_text_end_of_line(self.slice(tstart, self.pos));
            if !raw.is_empty() {
                let t = self.dom.new_text(&unescape_xml_text(&raw));
                self.dom.add(el, t);
            }
        }
        scope.pop(mark);
        el
    }

    /// Parse a processing instruction (`<?target data?>`) into a `Pi` node.
    /// `self.pos` is on `<?`.
    fn parse_pi_node(&mut self) -> NodeId {
        let (target_start, target_end, data_end) = self.pi_spans();
        let target = self.slice(target_start, target_end);
        let data = self.slice(target_end, data_end);
        self.dom.new_pi(&target, &data)
    }

    /// Consume a processing instruction (`self.pos` on `<?`): where its
    /// target starts and ends, and where its data ends.
    fn pi_spans(&mut self) -> (usize, usize, usize) {
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
        let target_end = self.pos;
        let end = self.index_of("?>", self.pos);
        self.pos = end.map(|e| e + 2).unwrap_or(len);
        (target_start, target_end, end.unwrap_or(len))
    }
}

/// Resolve a qualified name against the in-scope namespaces. Mirrors the TS
/// `resolve` closure exactly.
fn resolve(qn: &str, is_attr: bool, local_scope: &NsScope) -> XName {
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

/// Decode XML entity and character references in attribute and text
/// payloads (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&#nnn;`,
/// `&#xhh;`). Port of `unescapeXmlText`.
///
/// An unknown name reference, a numeric reference that does not decode
/// to a character XML 1.0 allows (§2.2 `Char`: no NUL, no C0 control but
/// tab, LF and CR, no U+FFFE or U+FFFF, no surrogate), or a reference
/// with no terminating `;`
/// is preserved verbatim: the `&` and trailing characters are copied
/// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
/// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
/// literal. Input with no `&` short-circuits to a clone of `s`.
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
        // XML 1.0 §2.2 `Char`: a reference to any other character is no
        // character the document can hold, so it stays as written.
        return char::from_u32(code)
            .filter(|&c| {
                matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}')
                    || c >= '\u{10000}'
            })
            .map(|c| c.to_string());
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

/// XML 1.0 §3.3.3 attribute-value normalization, applied to the RAW value
/// slice *before* [`unescape_xml_text`]: §2.11 end-of-line handling folds a
/// literal CRLF (and a lone CR) to one LF first, and every literal
/// whitespace character then reads as one space — the value a conforming
/// reader (Word) sees. Character references are untouched here: `&#9;`
/// still decodes to a tab afterwards, so referenced whitespace survives.
///
/// Returns the input unchanged when it holds no literal TAB, CR or LF —
/// the allocation-free fast path every ordinary attribute takes.
fn normalize_raw_attribute_whitespace(value: String) -> String {
    if !value.contains(['\t', '\n', '\r']) {
        return value;
    }
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            '\t' | '\n' => out.push(' '),
            // A literal CRLF is one end-of-line — one space, not two.
            '\r' => {
                if chars.as_str().starts_with('\n') {
                    chars.next();
                }
                out.push(' ');
            }
            _ => out.push(c),
        }
    }
    out
}

/// XML 1.0 §2.11 end-of-line handling, applied to the RAW text slice (CDATA
/// bodies included — the processor normalizes line breaks before parsing)
/// *before* [`unescape_xml_text`]: a literal CRLF and a lone CR both read
/// as one LF. TAB and LF stay as they are; a `&#13;` reference still
/// decodes to CR afterwards.
///
/// Returns the input unchanged when it holds no literal CR — the
/// allocation-free fast path every ordinary text run takes.
fn normalize_raw_text_end_of_line(text: String) -> String {
    if !text.contains('\r') {
        return text;
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.as_str().starts_with('\n') {
                chars.next();
            }
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    out
}

/// Structural admission for a part before the permissive DOM parser reads
/// it: well-formed nesting, a single root, no DTD, only the five
/// predefined or numeric entities, and nesting no deeper than 256. This is
/// not an OOXML schema check and not a ZIP resource budget.
///
/// Used to reject malformed parts up-front (so the permissive parser does
/// not silently misrender them) and in `jubarte validate`'s XML-shape
/// checks. `validate_xml` does NOT itself produce a DOM; it only checks
/// structure.
///
/// # Errors
///
/// Returns `Err(String)` describing the first problem found. The
/// possible conditions:
///
/// - a `quick-xml` reader error (invalid UTF-8, attribute parse failure
///   on the structural pass) is forwarded verbatim;
/// - `"multiple XML roots"` when a second top-level start or empty
///   element appears after a root has been opened;
/// - `"XML nesting exceeds {{N}} elements"` when nesting goes beyond 256
///   elements deep;
/// - `"unmatched XML end tag"` when an end tag fires without a matching
///   open (`End` at depth 0);
/// - `"DTD declarations are unsupported"` when a `<!DOCTYPE>` arrives;
/// - `"unsupported XML entity reference: {{name}}"` when an entity
///   reference is seen at the top level or names a value other than the
///   five predefined or a numeric character reference;
/// - `"text outside XML root"` when non-whitespace text appears before
///   the first element;
/// - `"CDATA outside XML root"` when a CDATA section appears before
///   the first element;
/// - `"missing or incomplete XML root"` when no root was opened by EOF
///   or depth was non-zero at EOF.
pub fn validate_xml(xml: &str) -> Result<(), String> {
    validate_xml_with_limits(xml, InputLimits::default())
}

/// Validate XML under explicit XML-related [`InputLimits`].
///
/// # Errors
///
/// Returns the same structural and tag-policy errors as [`validate_xml`].
pub fn validate_xml_with_limits(xml: &str, limits: InputLimits) -> Result<(), String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let max_depth = limits.max_xml_depth;
    let mut reader = Reader::from_str(xml);
    reader.config_mut().check_end_names = true;
    let mut policy = TagPolicy::new(limits);
    let mut depth = 0usize;
    let mut roots = 0usize;
    loop {
        match reader.read_event().map_err(|e| e.to_string())? {
            Event::Start(_) | Event::Empty(_) if depth == 0 && roots > 0 => {
                return Err("multiple XML roots".to_string());
            }
            Event::Start(e) => {
                policy.open(&e)?;
                if depth == 0 {
                    roots += 1;
                }
                depth += 1;
                if depth > max_depth {
                    return Err(format!("XML nesting exceeds {max_depth} elements"));
                }
            }
            Event::Empty(e) => {
                policy.empty(&e)?;
                if depth == 0 {
                    roots += 1;
                }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1).ok_or("unmatched XML end tag")?;
                policy.close();
            }
            Event::DocType(_) => return Err("DTD declarations are unsupported".to_string()),
            Event::GeneralRef(reference) => {
                let name: &str = &reference;
                if depth == 0 || decode_entity(name).is_none() {
                    return Err(format!("unsupported XML entity reference: {name}"));
                }
            }
            Event::Text(text) if depth == 0 && text.bytes().any(|b| !b.is_ascii_whitespace()) => {
                return Err("text outside XML root".to_string());
            }
            Event::CData(_) if depth == 0 => return Err("CDATA outside XML root".to_string()),
            Event::Eof => {
                return if roots == 1 && depth == 0 {
                    Ok(())
                } else {
                    Err("missing or incomplete XML root".to_string())
                };
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod depth_scan_tests {
    use super::{Dom, NodeId, nests_deeper, parse_xdocument};

    /// The deepest element of the tree `parse_xdocument` builds.
    fn built_depth(xml: &str) -> usize {
        let mut dom = Dom::new();
        let doc = parse_xdocument(&mut dom, xml);
        let mut deepest = 0;
        let mut stack: Vec<(NodeId, usize)> = vec![(doc, 0)];
        while let Some((node, depth)) = stack.pop() {
            deepest = deepest.max(depth);
            stack.extend(dom.elements(node, None).into_iter().map(|c| (c, depth + 1)));
        }
        deepest
    }

    /// The deepest nesting the scan reports: the least limit it keeps
    /// within.
    fn scanned_depth(xml: &str) -> usize {
        (0..=xml.len())
            .find(|&limit| !nests_deeper(xml, limit))
            .unwrap_or(usize::MAX)
    }

    /// The scan measures the tree the parser builds, malformed input
    /// included: the two cannot drift apart unnoticed. It counts the
    /// elements open around a point, as the view's reader does, so a
    /// self-closed leaf (`<c/>`) is the one level it does not count.
    #[test]
    fn the_scan_measures_the_tree_the_parser_builds() {
        for xml in [
            "<a><b><c/></b></a>",
            "<a><!x><!y></a>",
            r#"<a b="/>"><c d='>'><e/></c></a>"#,
            "<a><!-- <b><c> --><d/></a>",
            "<a><![CDATA[<b><c>]]><d/></a>",
            "<a><?pi <b> ?><c/></a>",
            "<?xml version='1.0'?><!DOCTYPE x><a><b></a>",
            "\u{feff}<a><b><c>",
            "<a b='x><c>",
            "<a><b c=x d><e>",
            "<a></><b>",
            "<a/><b><c/></b>",
            "<a> text <b>t</b> <c><d/></c></a>",
            "<a><p a='x' ?><b>",
            "<a><b",
        ] {
            let (built, scanned) = (built_depth(xml), scanned_depth(xml));
            assert!(
                scanned <= built && built <= scanned + 1,
                "{xml:?}: the parser nests {built} deep, the scan counts {scanned}"
            );
        }
    }

    /// Inside an element the permissive parser reads `<!x>` as a child
    /// element, so the scan that guards its recursion must count it too
    /// (#1353 review): a strict reader stops at it.
    #[test]
    fn the_scan_counts_markup_the_permissive_parser_nests() {
        let bangs = format!("<a>{}", "<!x>".repeat(600));
        assert!(nests_deeper(&bangs, 512), "bang elements nest");
        let quoted = format!("<a>{}", r#"<b c="/>">"#.repeat(600));
        assert!(nests_deeper(&quoted, 512), "a quoted /> closes nothing");
        assert!(!nests_deeper(
            &format!("<a>{}</a>", "<!x/>".repeat(600)),
            512
        ));
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod checked_xml_tests {
    use super::{Dom, parse_xdocument, validate_xml};

    #[test]
    fn truncated_attribute_input_terminates_in_legacy_parser() {
        for xml in [
            "<p",
            "<p a",
            "<p a=",
            "<p a='",
            "<p a='unfinished",
            "<p a='x' ?>",
        ] {
            let mut dom = Dom::new();
            let document = parse_xdocument(&mut dom, xml);
            assert!(dom.root(document).is_some(), "{xml}");
            assert!(validate_xml(xml).is_err(), "{xml}");
        }
    }

    /// The permissive parser skips a DOCTYPE whole, its internal subset
    /// included, even where the subset holds a `>` inside a quoted entity
    /// value or between its brackets: ending at the first `>` read the rest
    /// of the subset as content (#2015).
    #[test]
    fn a_doctype_is_skipped_whole_whatever_its_subset_holds() {
        for xml in [
            "<!DOCTYPE d><root/>",
            r#"<!DOCTYPE d [<!ENTITY a "<w:p><w:t>SMUGGLED</w:t></w:p>">]><root/>"#,
            "<!DOCTYPE d [<!ENTITY a '>'><!ENTITY b \"]>\">]><root/>",
            "<!DOCTYPE d [<!ELEMENT root ANY><!ATTLIST root x CDATA '>'>]><root/>",
            r#"<?xml version="1.0"?><!DOCTYPE d SYSTEM "x>y.dtd"><root/>"#,
        ] {
            let mut dom = Dom::new();
            let document = parse_xdocument(&mut dom, xml);
            let root = dom.root(document).expect(xml);
            assert!(
                dom.name_is(root, &crate::xmllinq::XName::get("root", "")),
                "{xml}"
            );
            assert!(dom.descendants(root, None).is_empty(), "{xml}");
        }
    }

    /// Package admission tolerates unquoted and valueless attributes so the
    /// permissive parser can read them; the strict gate never did, and the
    /// edit, comments, append and inspect paths still refuse such a part.
    #[test]
    fn checked_xml_keeps_refusing_attributes_that_are_not_xml() {
        for xml in ["<w:b w:val=on/>", "<w:b compact/>", "<p a=1 b='2'></p>"] {
            assert!(validate_xml(xml).is_err(), "{xml}");
            let mut dom = Dom::new();
            let document = parse_xdocument(&mut dom, xml);
            assert!(dom.root(document).is_some(), "{xml}");
        }
    }

    #[test]
    fn checked_xml_rejects_invalid_or_unbounded_structure() {
        for xml in [
            "",
            "<a><b></a>",
            "<a>",
            "<a/><b/>",
            "<a/><b></b>",
            "text<a/>",
            "<a x='1' x='2'/>",
            "<!DOCTYPE a><a/>",
            "<a>&unknown;</a>",
            "&amp;<a/>",
            "<![CDATA[x]]><a/>",
            "</a>",
        ] {
            assert!(validate_xml(xml).is_err(), "{xml}");
        }
        let deep = format!("{}{}", "<a>".repeat(257), "</a>".repeat(257));
        assert!(validate_xml(&deep).is_err());
        let ok_deep = format!("{}{}", "<a>".repeat(256), "</a>".repeat(256));
        assert_eq!(validate_xml(&ok_deep), Ok(()));
    }

    #[test]
    fn checked_xml_accepts_normal_documents_and_entities() {
        for xml in [
            "<?xml version='1.0'?><a x='&amp;'><b/></a>",
            "<a>&lt;&#65;&#x1F600;<![CDATA[raw <>]]></a>",
            "<!--before--><a/> <!--after-->",
            "<a/>",
        ] {
            assert_eq!(validate_xml(xml), Ok(()), "{xml}");
        }
    }
}
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod xml_lexical_boundary_contract_tests {
    use super::*;

    #[test]
    fn undecodable_character_references_remain_literal_and_uppercase_hex_decodes() {
        for (input, expected) in [
            ("&;", "&;"),
            ("&#;", "&#;"),
            ("&#X41;", "A"),
            ("&#x1F433;", "🐳"),
            ("&#1114112;", "&#1114112;"),
            ("&#55296;", "&#55296;"),
            ("&bad1;", "&bad1;"),
            ("&é;", "&é;"),
            ("&unknown;", "&unknown;"),
            ("before &amp", "before &amp"),
            ("A &lt; B &amp; C", "A < B & C"),
        ] {
            assert_eq!(unescape_xml_text(input), expected, "{input}");
        }
    }

    #[test]
    fn literal_attribute_whitespace_reads_as_spaces_references_survive() {
        // XML 1.0 §2.11 + §3.3.3, the value a conforming reader (Word) sees:
        // end-of-line handling folds the literal CRLF to one LF first, then
        // every literal whitespace character in the attribute value reads
        // as one space.
        let mut dom = Dom::new();
        let document = parse_xdocument(&mut dom, "<a b=\"x\ty\nz\r\nw\"/>");
        let root = dom.root(document).unwrap();
        assert_eq!(
            dom.attribute(root, &crate::xmllinq::XName::get("b", "")),
            Some("x y z w")
        );
        // Character references are not literal whitespace: &#9;/&#10;/&#13;
        // still decode to the characters they name.
        let mut dom = Dom::new();
        let document = parse_xdocument(&mut dom, "<a b=\"x&#9;y&#10;z&#13;w\"/>");
        let root = dom.root(document).unwrap();
        assert_eq!(
            dom.attribute(root, &crate::xmllinq::XName::get("b", "")),
            Some("x\ty\nz\rw")
        );
    }

    #[test]
    fn literal_text_end_of_line_reads_as_lf_references_survive() {
        // §2.11: a literal CRLF and a lone CR both read as LF; TAB and LF
        // stay as they are. CDATA content is line-normalized the same way
        // (the processor normalizes before parsing) and builds the same
        // text-node value from raw markup.
        let mut dom = Dom::new();
        let document = parse_xdocument(&mut dom, "<a>t\r\nu\rv</a>");
        let root = dom.root(document).unwrap();
        assert_eq!(dom.value_str(root), "t\nu\nv");
        let mut dom = Dom::new();
        let document = parse_xdocument(&mut dom, "<a><![CDATA[t\r\nu\rv]]></a>");
        let root = dom.root(document).unwrap();
        assert_eq!(dom.value_str(root), "t\nu\nv");
        // A CR reference still decodes to CR.
        let mut dom = Dom::new();
        let document = parse_xdocument(&mut dom, "<a>t&#13;u</a>");
        let root = dom.root(document).unwrap();
        assert_eq!(dom.value_str(root), "t\ru");
    }

    #[test]
    fn literal_whitespace_sources_round_trip_to_the_same_dom_values() {
        for source in [
            "<a b=\"x\ty\nz\r\nw\">t\r\nu\rv</a>",
            "<a b=\"x&#9;y&#10;z&#13;w\">t&#13;u</a>",
        ] {
            let mut dom = Dom::new();
            let document = parse_xdocument(&mut dom, source);
            let root = dom.root(document).unwrap();
            let attr = dom
                .attribute(root, &crate::xmllinq::XName::get("b", ""))
                .unwrap()
                .to_string();
            let text = dom.value(root);
            let serialized = dom.serialize_document(document);
            let mut reparsed = Dom::new();
            let document2 = parse_xdocument(&mut reparsed, &serialized);
            let root2 = reparsed.root(document2).unwrap();
            assert_eq!(
                reparsed.attribute(root2, &crate::xmllinq::XName::get("b", "")),
                Some(attr.as_str()),
                "{source} → {serialized}"
            );
            assert_eq!(reparsed.value(root2), text, "{source} → {serialized}");
        }
    }

    #[test]
    fn legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata() {
        for source in [
            "<?xml",
            "<?xml version=1.0?><p/>",
            "<?xml version='1.0' encoding=bad?><p/>",
        ] {
            let mut dom = Dom::new();
            let document = parse_xdocument(&mut dom, source);
            let declaration = dom.declaration(document).unwrap();
            assert_eq!(declaration.version.as_deref(), Some("1.0"));
            assert_eq!(declaration.encoding, None);
            assert_eq!(declaration.standalone, None);
            if source.ends_with("<p/>") {
                let root = dom.root(document).unwrap();
                assert_eq!(dom.name(root).unwrap().local_name(), "p");
                assert!(dom.elements(root, None).is_empty());
            } else {
                assert!(dom.root(document).is_none());
                assert!(validate_xml(source).is_err());
            }
        }
        let mut dom = Dom::new();
        let document = parse_xdocument(
            &mut dom,
            "<?xml-stylesheet type='text/css'?>\n<p>owned</p><?tail?>",
        );
        assert_eq!(
            dom.nodes(document)
                .iter()
                .filter(|&&node| dom.is_pi(node))
                .count(),
            2
        );
        let root = dom.root(document).unwrap();
        assert_eq!(dom.value(root), "owned");
        assert!(validate_xml("<?xml-stylesheet type='text/css'?>\n<p>owned</p><?tail?>").is_ok());
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tag_policy_tests {
    //! N01/N02: the checked gate (`validate_xml`, `try_parse_xdocument`, and
    //! `admission::check_xml_depth`) bounds tag width and namespace budgets
    //! before the permissive DOM parser ever touches the bytes, and the
    //! parser inserts attributes in bulk (linear) instead of N linear scans.

    use super::*;
    use crate::xmllinq::ATTR_NAME_PROBES;

    fn xml_with_attrs(count: usize) -> String {
        let attrs = (0..count)
            .map(|i| format!(" a{i}='v{i}'"))
            .collect::<String>();
        format!("<a{attrs}/>")
    }

    #[test]
    fn element_attribute_count_is_capped_at_the_boundary() {
        assert_eq!(
            validate_xml(&xml_with_attrs(MAX_ELEMENT_ATTRIBUTES)),
            Ok(())
        );
        for xml in [
            xml_with_attrs(MAX_ELEMENT_ATTRIBUTES + 1),
            "<a b='v' xmlns:p='urn:x' xmlns:q='urn:x' p:c='1' q:c='2'/>".to_string(),
            format!("<a {}='v'/>", "n".repeat(MAX_NAME_BYTES + 1)),
            format!("<a b='{}'/>", "v".repeat(MAX_ATTRIBUTE_VALUE_BYTES + 1)),
        ] {
            assert!(validate_xml(&xml).is_err(), "{xml}");
            let mut dom = Dom::new();
            assert!(
                try_parse_xdocument(&mut dom, &xml).is_err(),
                "try_parse must refuse {xml}"
            );
        }
    }

    #[test]
    fn tag_whole_width_is_bounded() {
        // 256 names × ~600 KiB values exceeds the per-tag attribute-byte
        // budget while every individual cap still passes (each value is
        // far below MAX_ATTRIBUTE_VALUE_BYTES).
        let per_value = 600 * 1024;
        assert!(
            per_value < MAX_ATTRIBUTE_VALUE_BYTES,
            "fixture relies on per_value staying under the value cap"
        );
        let attrs = (0..256)
            .map(|i| format!(" a{i}='{}'", "v".repeat(per_value)))
            .collect::<String>();
        assert!(validate_xml(&format!("<a{attrs}/>")).is_err());
    }

    /// Word writes a VML `o:gfxdata` attribute of up to ~366 KiB inside
    /// `word/header*.xml` parts of its own documents; the checked gate (N01)
    /// must admit the real attribute size Word uses, not a value the engine
    /// happens to like. The base64 alphabet carries no XML-special bytes,
    /// so the value at the threshold is what a body parser sees verbatim.
    #[test]
    fn a_word_gfxdata_attribute_is_admitted() {
        // 400 * 1024 = 409,600 bytes — comfortably above the corpus's
        // largest o:gfxdata (≈366 KiB) and the previous 262,144-byte cap.
        let gfx_value: String = "A".repeat(400 * 1024);
        let header = format!(
            r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office"><w:p><w:r><w:pict><v:shape o:gfxdata="{gfx_value}"/></w:pict></w:r></w:p></w:hdr>"#
        );
        let value_len = gfx_value.len();
        assert_eq!(validate_xml(&header), Ok(()), "{value_len} bytes refused");
        let mut dom = Dom::new();
        let document =
            try_parse_xdocument(&mut dom, &header).expect("try_parse must admit Word's gfxdata");
        // Walk to the <v:shape> so the test mirrors how `body` reads it.
        let w = crate::xmllinq::XNamespace::get(
            "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
        );
        let v = crate::xmllinq::XNamespace::get("urn:schemas-microsoft-com:vml");
        let root = dom.root(document).unwrap();
        let p = dom.element(root, &w.name("p")).expect("w:p");
        let r = dom.element(p, &w.name("r")).expect("w:r");
        let pict = dom.element(r, &w.name("pict")).expect("w:pict");
        let shape = dom
            .element(pict, &v.name("shape"))
            .expect("v:shape must be admitted through the parser");
        let gfx_attr = dom
            .attribute(
                shape,
                &crate::xmllinq::XName::get("gfxdata", "urn:schemas-microsoft-com:office:office"),
            )
            .expect("o:gfxdata must round-trip through the DOM");
        assert_eq!(gfx_attr.len(), value_len);
    }

    /// The exact-boundary contract for the new value cap: a value byte at
    /// `MAX_ATTRIBUTE_VALUE_BYTES` is admitted (it is the inclusive cap);
    /// one byte more is refused with the gate's structured error.
    #[test]
    fn a_value_at_the_attribute_byte_cap_is_admitted_one_more_refused() {
        let at_cap = "v".repeat(MAX_ATTRIBUTE_VALUE_BYTES);
        assert_eq!(
            validate_xml(&format!("<a b='{at_cap}'/>")),
            Ok(()),
            "value at MAX_ATTRIBUTE_VALUE_BYTES must be admitted"
        );
        let over_cap = "v".repeat(MAX_ATTRIBUTE_VALUE_BYTES + 1);
        let over_xml = format!("<a b='{over_cap}'/>");
        let err = validate_xml(&over_xml).expect_err("one byte over MAX_ATTRIBUTE_VALUE_BYTES");
        assert!(
            err.contains(&format!("{MAX_ATTRIBUTE_VALUE_BYTES} bytes")),
            "expected gate error naming the cap, got {err:?}"
        );
        let mut dom = Dom::new();
        assert!(
            try_parse_xdocument(&mut dom, &over_xml).is_err(),
            "try_parse must refuse one byte over MAX_ATTRIBUTE_VALUE_BYTES"
        );
    }

    /// The complete start tag, including spaces, quotes and delimiters,
    /// has an inclusive byte cap independent of each value's cap.
    #[test]
    fn a_tag_at_the_attribute_byte_cap_is_admitted_one_more_refused() {
        let overhead = "<a a0='' a1='' a2=''/>".len();
        let budget = MAX_TAG_ATTRIBUTE_BYTES - overhead;
        let xml = |extra| {
            let attrs: String = (0..3)
                .map(|i| {
                    let length = budget / 3 + if i == 2 { budget % 3 + extra } else { 0 };
                    format!(" a{i}='{}'", "v".repeat(length))
                })
                .collect();
            format!("<a{attrs}/>")
        };
        let at_cap = xml(0);
        assert_eq!(at_cap.len(), MAX_TAG_ATTRIBUTE_BYTES);
        assert_eq!(validate_xml(&at_cap), Ok(()));
        let over = xml(1);
        assert_eq!(over.len(), MAX_TAG_ATTRIBUTE_BYTES + 1);
        let error = validate_xml(&over).expect_err("one start-tag byte over cap must refuse");
        assert!(
            error.contains(&format!("{MAX_TAG_ATTRIBUTE_BYTES}")),
            "{error}"
        );
    }

    #[test]
    fn namespace_declaration_budgets_reject_before_dom_insertion() {
        let uri = "u".repeat(MAX_NAMESPACE_URI_BYTES + 1);
        assert!(validate_xml(&format!("<a xmlns:p='{uri}'/>")).is_err());
        let prefix = "p".repeat(MAX_NAMESPACE_PREFIX_BYTES + 1);
        assert!(validate_xml(&format!("<a xmlns:{prefix}='u'/>")).is_err());
        // Two nested scopes of 128 live bindings each stay under the live
        // binding cap; a single element declaring more than the live cap
        // refuses.
        let many = (0..MAX_LIVE_NS_BINDINGS + 1)
            .map(|i| format!(" xmlns:p{i}='urn:{i}'"))
            .collect::<String>();
        assert!(validate_xml(&format!("<a{many}/>")).is_err());
        // Rebinding the same prefix keeps one live binding per level: nesting
        // MAX_LIVE_NS_BINDINGS-deep with a fresh xmlns:p per level is legal.
        let mut deep = String::new();
        for i in 0..MAX_LIVE_NS_BINDINGS {
            deep.push_str(&format!("<e xmlns:p='urn:{i}'>"));
        }
        for _ in 0..MAX_LIVE_NS_BINDINGS {
            deep.push_str("</e>");
        }
        assert_eq!(validate_xml(&deep), Ok(()));
        // Cumulative declared bytes across the part refuse even when every
        // single declaration is inside the per-element budgets (Empty
        // elements release their live bindings, so only the aggregate byte
        // budget can trip).
        let heavy: String = {
            let mut s = String::from("<a>");
            for i in 0..(MAX_NS_DECLARED_BYTES / 2048 + 8) {
                s.push_str(&format!("<e xmlns:p{i}='{}'/>", "u".repeat(4000)));
            }
            s.push_str("</a>");
            s
        };
        assert!(validate_xml(&heavy).is_err());
    }

    #[test]
    fn the_dom_parser_keeps_every_attribute_and_its_order() {
        let mut dom = Dom::new();
        let document = parse_xdocument(
            &mut dom,
            "<r a='1' b='2' xmlns:x='urn:x' x:c='3' c='4' z='last'/>",
        );
        let root = dom.root(document).unwrap();
        let names: Vec<String> = (0..dom.attr_count(root))
            .map(|i| dom.attr_at(root, i).0.local_name().to_string())
            .collect();
        assert_eq!(names, ["a", "b", "x", "c", "c", "z"]);
    }

    #[test]
    fn parsed_attributes_insert_in_bulk_not_quadratic_scans() {
        // N01 mechanism evidence: inserting N attributes through
        // `set_attribute_value` walks the growing Vec — sum 1..N name
        // comparisons. The bulk path (`set_attributes`, used by the parser)
        // probes a hash index once per attribute.
        const N: usize = 96;
        let xml = xml_with_attrs(N);

        let before = ATTR_NAME_PROBES.with(|c| c.get());
        let mut dom = Dom::new();
        let document = parse_xdocument(&mut dom, &xml);
        let parsed = ATTR_NAME_PROBES.with(|c| c.get()) - before;
        let root = dom.root(document).unwrap();
        assert_eq!(dom.attr_count(root), N);
        assert!(
            parsed <= N,
            "bulk insert probed {parsed} times for {N} attributes"
        );

        // Control: the incremental editing API still costs sum(0..N) probes.
        let before = ATTR_NAME_PROBES.with(|c| c.get());
        let manual = dom.new_element(crate::xmllinq::XName::get("m", ""));
        for i in 0..N {
            dom.set_attribute_value(
                manual,
                &crate::xmllinq::XName::get(&format!("a{i}"), ""),
                Some("v"),
            );
        }
        let incremental = ATTR_NAME_PROBES.with(|c| c.get()) - before;
        assert_eq!(incremental, N * (N - 1) / 2);
    }

    #[test]
    fn duplicate_expanded_names_refuse_only_in_the_checked_parser() {
        let xml = "<r xmlns:p='urn:x' xmlns:q='urn:x' p:a='first' q:a='second'/>";
        assert!(validate_xml(xml).is_err());
        assert!(try_parse_xdocument(&mut Dom::new(), xml).is_err());
        let mut dom = Dom::new();
        let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parse_xdocument(&mut dom, xml)
        }));
        assert!(
            parsed.is_ok(),
            "the infallible parser must return a document"
        );
        let root = dom.root(parsed.unwrap()).unwrap();
        assert_eq!(dom.attr_count(root), 3);
        assert_eq!(
            dom.attribute(root, &XName::get("a", "urn:x")),
            Some("second")
        );
        let names: Vec<_> = (0..3)
            .map(|i| dom.attr_at(root, i).0.local_name().to_string())
            .collect();
        assert_eq!(names, ["p", "q", "a"]);
    }

    #[test]
    fn issue650_infallible_parser_preserves_over_default_count() {
        let xml = xml_with_attrs(1025);
        let mut dom = Dom::new();
        let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parse_xdocument(&mut dom, &xml)
        }));
        assert!(
            parsed.is_ok(),
            "the infallible parser must return all 1025 attributes"
        );
        let root = dom.root(parsed.unwrap()).unwrap();
        assert_eq!(dom.attr_count(root), 1025);
        for i in 0..1025 {
            let (name, value) = dom.attr_at(root, i);
            assert_eq!(name.local_name(), format!("a{i}"));
            assert_eq!(name.namespace_name(), "");
            assert_eq!(value, format!("v{i}"));
        }
    }

    #[test]
    fn issue650_infallible_parser_preserves_over_default_name() {
        let name = "n".repeat(1025);
        let xml = format!("<r {name}='Alice'/>");
        let mut dom = Dom::new();
        let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parse_xdocument(&mut dom, &xml)
        }));
        assert!(
            parsed.is_ok(),
            "the infallible parser must return the 1025-byte name"
        );
        let root = dom.root(parsed.unwrap()).unwrap();
        assert_eq!(dom.attr_count(root), 1);
        assert_eq!(dom.attr_at(root, 0).0.local_name(), name);
        assert_eq!(dom.attr_at(root, 0).1, "Alice");
    }

    #[test]
    fn issue650_wide_tag_preserves_names_values_and_order() {
        let xml = xml_with_attrs(1024);
        assert_eq!(validate_xml(&xml), Ok(()), "1024 attributes must parse");
        let mut dom = Dom::new();
        let doc = try_parse_xdocument(&mut dom, &xml).unwrap();
        let root = dom.root(doc).unwrap();
        assert_eq!(dom.attr_count(root), 1024);
        for i in 0..1024 {
            let (name, value) = dom.attr_at(root, i);
            assert_eq!(name.local_name(), format!("a{i}"));
            assert_eq!(name.namespace_name(), "");
            assert_eq!(value, format!("v{i}"));
        }
        assert!(validate_xml(&xml_with_attrs(1025)).is_err());
    }

    #[test]
    fn issue650_attribute_name_byte_boundary() {
        for (length, accepted) in [(1024, true), (1025, false)] {
            let xml = format!("<r {}='v'/>", "n".repeat(length));
            assert_eq!(validate_xml(&xml).is_ok(), accepted, "name bytes: {length}");
            assert_eq!(try_parse_xdocument(&mut Dom::new(), &xml).is_ok(), accepted);
        }
    }

    #[test]
    fn issue650_checked_dom_parser_enforces_width_limits() {
        for xml in [
            xml_with_attrs(1025),
            format!("<r {}='v'/>", "n".repeat(1025)),
        ] {
            assert!(
                try_parse_xdocument(&mut Dom::new(), &xml).is_err(),
                "checked DOM parser must refuse oversized attributes"
            );
        }
    }

    #[test]
    fn issue650_unprefixed_attribute_has_no_default_namespace() {
        let xml = "<r xmlns='urn:x' xmlns:a='urn:x' x='1' a:x='2'/>";
        let mut dom = Dom::new();
        let doc = try_parse_xdocument(&mut dom, xml).unwrap();
        let root = dom.root(doc).unwrap();
        assert_eq!(dom.attribute(root, &XName::get("x", "")), Some("1"));
        assert_eq!(dom.attribute(root, &XName::get("x", "urn:x")), Some("2"));
    }

    #[test]
    fn issue650_attribute_probes_scale_linearly() {
        let mut counts = Vec::new();
        for n in [64, 256, 1024] {
            let before = ATTR_NAME_PROBES.with(|c| c.get());
            let result = try_parse_xdocument(&mut Dom::new(), &xml_with_attrs(n));
            assert!(result.is_ok(), "{n} attributes must parse: {result:?}");
            counts.push(ATTR_NAME_PROBES.with(|c| c.get()) - before);
        }
        assert!(
            counts[0] > 0,
            "the probe counter must measure bulk insertion: {counts:?}"
        );
        assert!(counts[2] <= 16 * counts[0], "probes: {counts:?}");
        println!("attribute probes at 64, 256, 1024: {counts:?}");
    }

    #[test]
    fn nested_rebinding_shadows_and_restores() {
        let mut dom = Dom::new();
        let document = parse_xdocument(
            &mut dom,
            "<a xmlns:p='urn:outer'><b xmlns:p='urn:inner'><p:x/><c/></b><p:y xmlns=''/></a>",
        );
        let a = dom.root(document).unwrap();
        let b = dom
            .element(a, &crate::xmllinq::XName::get("b", ""))
            .unwrap();
        let x = dom
            .element(b, &crate::xmllinq::XName::get("x", "urn:inner"))
            .unwrap();
        assert_eq!(dom.name(x).unwrap().namespace_name(), "urn:inner");
        // After </b>, p rebinds to urn:outer for siblings of b.
        let y = dom
            .element(a, &crate::xmllinq::XName::get("y", "urn:outer"))
            .unwrap();
        assert_eq!(dom.name(y).unwrap().namespace_name(), "urn:outer");
        // The empty default xmlns disables the default namespace inside y.
        let _ = x;
    }

    #[test]
    fn deep_rebinding_scope_grows_only_with_declared_bytes() {
        // N02 mechanism evidence: the scope tracks only what this element
        // declares — walking down never re-copies ancestor bindings.
        const DEPTH: usize = 64;
        let mut xml = String::new();
        for i in 0..DEPTH {
            xml.push_str(&format!("<e xmlns:p='urn:{i}' xmlns:q='urn:q{i}'>"));
        }
        for _ in 0..DEPTH {
            xml.push_str("</e>");
        }
        let before = NS_SCOPE_DECLARED_BYTES.with(|c| c.get());
        let mut dom = Dom::new();
        let document = parse_xdocument(&mut dom, &xml);
        let inserted = NS_SCOPE_DECLARED_BYTES.with(|c| c.get()) - before;
        // p -> urn:{i} is 1 + 4 + digits bytes; q -> urn:q{i} is
        // 1 + 5 + digits.
        let declared: usize = (0..DEPTH).map(|i| 11 + 2 * i.to_string().len()).sum();
        assert_eq!(
            inserted, declared,
            "scope must record declared bytes only, never cloned ancestor maps"
        );
        let deepest = {
            let mut cur = dom.root(document).unwrap();
            for _ in 0..DEPTH - 1 {
                cur = dom
                    .element(cur, &crate::xmllinq::XName::get("e", ""))
                    .unwrap();
            }
            cur
        };
        assert_eq!(dom.name(deepest).unwrap().local_name(), "e");
    }

    #[test]
    fn normal_documents_parse_unchanged_through_try_parse() {
        let mut dom = Dom::new();
        let document = try_parse_xdocument(
            &mut dom,
            "<w:p xmlns:w='urn:wml' xml:space='preserve'><w:r w:val='1'/></w:p>",
        )
        .expect("normal document parses");
        let root = dom.root(document).unwrap();
        assert_eq!(dom.name(root).unwrap().namespace_name(), "urn:wml");
        assert_eq!(
            dom.attribute(root, &crate::xmllinq::XNamespace::xml().name("space")),
            Some("preserve")
        );
        let r = dom
            .element(root, &crate::xmllinq::XName::get("r", "urn:wml"))
            .unwrap();
        assert_eq!(
            dom.attribute(r, &crate::xmllinq::XName::get("val", "urn:wml")),
            Some("1")
        );
    }

    /// `cargo test` runs tests as threads of one process, so a shared
    /// counter would see every other test's parsing. Work done on another
    /// thread must leave this thread's counters at zero.
    #[test]
    fn mechanism_counters_are_isolated_to_the_calling_thread() {
        let attr_handle = std::thread::spawn(|| {
            let mut dom = Dom::new();
            let el = dom.new_element(crate::xmllinq::XName::get("m", ""));
            for i in 0..100 {
                dom.set_attribute_value(
                    el,
                    &crate::xmllinq::XName::get(&format!("a{i}"), ""),
                    Some("v"),
                );
            }
            ATTR_NAME_PROBES.with(|c| c.get())
        });
        let attr_probed_bg = attr_handle.join().expect("bg attr thread");

        let scope_handle = std::thread::spawn(|| {
            let mut xml = String::new();
            for i in 0..32 {
                xml.push_str(&format!("<e xmlns:p='urn:{i}' xmlns:q='urn:q{i}'>"));
            }
            for _ in 0..32 {
                xml.push_str("</e>");
            }
            let mut dom = Dom::new();
            let _ = parse_xdocument(&mut dom, &xml);
            NS_SCOPE_DECLARED_BYTES.with(|c| c.get())
        });
        let scope_inserted_bg = scope_handle.join().expect("bg scope thread");

        let attr_main = ATTR_NAME_PROBES.with(|c| c.get());
        let scope_main = NS_SCOPE_DECLARED_BYTES.with(|c| c.get());
        assert_eq!(
            attr_main, 0,
            "main-thread ATTR_NAME_PROBES must be untouched by background work, got {attr_main}"
        );
        assert_eq!(
            scope_main, 0,
            "main-thread NS_SCOPE_DECLARED_BYTES must be untouched by background work, got {scope_main}"
        );
        assert!(
            attr_probed_bg > 0,
            "background thread observed {attr_probed_bg} attr probes"
        );
        assert!(
            scope_inserted_bg > 0,
            "background thread observed {scope_inserted_bg} declared bytes"
        );
    }
}

#[cfg(test)]
mod unescape_tests {
    use super::unescape_xml_text;

    #[test]
    fn test_unescape_xml_text() {
        // No ampersand
        assert_eq!(unescape_xml_text("hello world"), "hello world");

        // Basic named entities
        assert_eq!(unescape_xml_text("&lt;hello&gt;"), "<hello>");
        assert_eq!(unescape_xml_text("Tom &amp; Jerry"), "Tom & Jerry");
        assert_eq!(unescape_xml_text("&quot;quote&quot;"), "\"quote\"");
        assert_eq!(unescape_xml_text("&apos;apostrophe&apos;"), "'apostrophe'");

        // Numeric entities - decimal
        assert_eq!(unescape_xml_text("&#65;"), "A");
        assert_eq!(unescape_xml_text("&#97;"), "a");

        // Numeric entities - hexadecimal
        assert_eq!(unescape_xml_text("&#x41;"), "A");
        assert_eq!(unescape_xml_text("&#x61;"), "a");
        assert_eq!(unescape_xml_text("&#X41;"), "A"); // Uppercase X

        // Unicode entities
        assert_eq!(unescape_xml_text("&#x1F433;"), "🐳");
        assert_eq!(unescape_xml_text("&#128051;"), "🐳"); // 1F433 in decimal

        // Invalid/Unknown entities (should be left as is)
        assert_eq!(unescape_xml_text("&unknown;"), "&unknown;");
        assert_eq!(unescape_xml_text("&123;"), "&123;");
        assert_eq!(unescape_xml_text("&#xG;"), "&#xG;"); // Invalid hex
        assert_eq!(unescape_xml_text("&#;"), "&#;"); // Empty numeric
        assert_eq!(unescape_xml_text("&;"), "&;"); // Empty named

        // Missing semicolon (should be left as is)
        assert_eq!(unescape_xml_text("before &amp after"), "before &amp after");

        // Multiple entities
        assert_eq!(
            unescape_xml_text("A &lt; B &amp; C &gt; D"),
            "A < B & C > D"
        );

        // Mixed text and entities
        assert_eq!(
            unescape_xml_text("Start &#x41; middle &amp; end"),
            "Start A middle & end"
        );
    }

    /// XML 1.0 §2.2 `Char`: a reference to a character the production
    /// leaves out (NUL, the C0 controls but tab, LF and CR, U+FFFE and
    /// U+FFFF) is kept verbatim, so it never becomes text the writer then
    /// serializes into a part no XML reader accepts.
    #[test]
    fn a_reference_to_a_character_xml_forbids_stays_literal() {
        for kept in ["&#0;", "&#x0;", "&#x1F;", "&#8;", "&#xFFFE;", "&#xFFFF;"] {
            assert_eq!(unescape_xml_text(kept), kept);
        }
        assert_eq!(unescape_xml_text("&#9;&#xA;&#13;"), "\t\n\r");
        assert_eq!(unescape_xml_text("&#x20;&#xD7FF;"), " \u{D7FF}");
        assert_eq!(unescape_xml_text("&#xE000;&#xFFFD;"), "\u{E000}\u{FFFD}");
        assert_eq!(
            unescape_xml_text("&#x10000;&#x10FFFF;"),
            "\u{10000}\u{10FFFF}"
        );
    }
}
