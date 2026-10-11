// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte debug diff A B [C …]`: what differs between two or more
//! packages, element by element.

use std::collections::HashMap;
use std::fmt::Write as _;

use super::{Package, WIDTH, clip, clip_from, clip_line, decode_xml, local, story_roles_all};
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

/// `jubarte debug diff` options.
#[derive(Clone, Debug)]
pub struct DiffOptions {
    /// Only parts whose name or role contains this.
    pub part: Option<String>,
    /// Only the style with this name or id (case-insensitive).
    pub style: Option<String>,
    /// Only paragraphs whose text contains this, in any file.
    pub para_text: Option<String>,
    /// Keep what the diff drops by default.
    pub raw: bool,
    /// Print each shown element's common lines too.
    pub full: bool,
    /// Hunks per part (0: all).
    pub limit: usize,
}

impl Default for DiffOptions {
    fn default() -> Self {
        DiffOptions {
            part: None,
            style: None,
            para_text: None,
            raw: false,
            full: false,
            limit: 60,
        }
    }
}

/// The differences between `files` (label, package bytes), first file first.
///
/// Each part becomes a tree whose elements carry a key that names them
/// across documents: a style by type and name, a paragraph by its text, a
/// table by its first text, a row by its text, a cell by its position, a
/// note or comment by its text, a header or footer part by the section role
/// that shows it. Children align by key (a longest common subsequence,
/// file by file); within a gap, unmatched elements of one kind pair in
/// order, so a rewritten paragraph still meets the one it replaced. Each
/// matched element is a list of lines (property items, run segments), and
/// a hunk prints the lines not every file holds.
pub fn diff(files: &[(&str, &[u8])], opts: &DiffOptions) -> Result<String, String> {
    if files.len() < 2 {
        return Err("debug diff needs at least two files".to_string());
    }
    let filter = Filter {
        style: opts.style.as_deref().map(str::to_lowercase),
        para: opts.para_text.clone(),
    };
    let mut trees: Vec<Vec<(String, N)>> = Vec::with_capacity(files.len());
    for (label, bytes) in files {
        let pkg = Package::open(bytes).map_err(|e| format!("{label}: {e}"))?;
        trees.push(package_tree(&pkg, opts, &filter));
    }
    let mut keys: Vec<&str> = Vec::new();
    for t in &trees {
        for (k, _) in t {
            if !keys.contains(&k.as_str()) {
                keys.push(k);
            }
        }
    }
    let labels: Vec<&str> = files.iter().map(|(l, _)| *l).collect();
    let mut out = String::new();
    let _ = writeln!(out, "files: {}", labels.join(" · "));
    let walker = Walker {
        labels: &labels,
        filtered: filter.active(),
        full: opts.full,
    };
    let mut total = 0;
    for key in keys {
        let nodes: Vec<Option<&N>> = trees
            .iter()
            .map(|t| t.iter().find(|(k, _)| k == key).map(|(_, n)| n))
            .collect();
        let mut hunks = Vec::new();
        walker.walk(
            &nodes,
            &vec![true; nodes.len()],
            &mut Vec::new(),
            false,
            &mut hunks,
        );
        // --full prints agreeing elements too; only disagreements count.
        total += hunks.iter().filter(|(_, differs)| *differs).count();
        let shown = if opts.limit == 0 {
            hunks.len()
        } else {
            opts.limit.min(hunks.len())
        };
        for (h, _) in &hunks[..shown] {
            out.push_str(h);
        }
        if shown < hunks.len() {
            let _ = writeln!(
                out,
                "… {} more in {key} (-n 0 prints all)",
                hunks.len() - shown
            );
        }
    }
    match total {
        0 => out.push_str("no differences\n"),
        1 => out.push_str("1 difference\n"),
        n => {
            let _ = writeln!(out, "{n} differences");
        }
    }
    Ok(out)
}

/// What `--style` and `--para-text` keep.
struct Filter {
    /// Lower-cased style name or id.
    style: Option<String>,
    para: Option<String>,
}

impl Filter {
    fn active(&self) -> bool {
        self.style.is_some() || self.para.is_some()
    }
}

/// One element of a part, normalized.
struct N {
    /// Element kind (`p`, `style`, `rPr`, …): unmatched elements of one kind
    /// pair within a gap.
    kind: String,
    /// What names the element across documents.
    key: String,
    /// How a hunk's path shows it.
    label: String,
    /// Property items, run segments, attributes: what the diff compares.
    lines: Vec<String>,
    kids: Vec<N>,
    /// A property block: a missing one is an empty one, not news.
    block: bool,
    /// The filter selects this element (and all below it).
    hit: bool,
    /// The filter selects something below it.
    sub_hit: bool,
}

impl N {
    fn new(kind: &str, key: String, label: String) -> Self {
        N {
            kind: kind.to_string(),
            key,
            label,
            lines: Vec::new(),
            kids: Vec::new(),
            block: false,
            hit: false,
            sub_hit: false,
        }
    }

    fn with_kids(mut self, kids: Vec<N>) -> Self {
        self.sub_hit = kids.iter().any(|k| k.hit || k.sub_hit);
        self.kids = kids;
        self
    }
}

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";

/// Property blocks: their children are items, compared as a set.
const BLOCKS: [&str; 10] = [
    "pPr", "rPr", "tblPr", "trPr", "tcPr", "tblPrEx", "sectPr", "sdtPr", "sdtEndPr", "tblGrid",
];

/// Property-change records: a block holding the recorded old block.
const CHANGES: [&str; 9] = [
    "pPrChange",
    "rPrChange",
    "tblPrChange",
    "tblPrExChange",
    "trPrChange",
    "tcPrChange",
    "sectPrChange",
    "tblGridChange",
    "numberingChange",
];

/// Revision marks and their range markers: their id, author and date are
/// renumbered and restamped by every save.
const REVISION_MARKS: [&str; 18] = [
    "ins",
    "del",
    "moveFrom",
    "moveTo",
    "cellIns",
    "cellDel",
    "cellMerge",
    "moveFromRangeStart",
    "moveFromRangeEnd",
    "moveToRangeStart",
    "moveToRangeEnd",
    "customXmlInsRangeStart",
    "customXmlInsRangeEnd",
    "customXmlDelRangeStart",
    "customXmlDelRangeEnd",
    "customXmlMoveFromRangeStart",
    "customXmlMoveToRangeStart",
    "numberingChange",
];

/// Elements whose `w:id` only links them to a partner Word renumbers.
const LINK_IDS: [&str; 12] = [
    "bookmarkStart",
    "bookmarkEnd",
    "commentRangeStart",
    "commentRangeEnd",
    "commentReference",
    "comment",
    "footnoteReference",
    "endnoteReference",
    "footnote",
    "endnote",
    "permStart",
    "permEnd",
];

/// On/off properties: `w:val="1"` is the bare element.
const TOGGLES: [&str; 48] = [
    "b",
    "bCs",
    "i",
    "iCs",
    "caps",
    "smallCaps",
    "strike",
    "dstrike",
    "outline",
    "shadow",
    "emboss",
    "imprint",
    "noProof",
    "snapToGrid",
    "vanish",
    "webHidden",
    "specVanish",
    "rtl",
    "cs",
    "oMath",
    "keepNext",
    "keepLines",
    "pageBreakBefore",
    "widowControl",
    "suppressLineNumbers",
    "suppressAutoHyphens",
    "kinsoku",
    "wordWrap",
    "overflowPunct",
    "topLinePunct",
    "autoSpaceDE",
    "autoSpaceDN",
    "bidi",
    "adjustRightInd",
    "contextualSpacing",
    "mirrorIndents",
    "suppressOverlap",
    "qFormat",
    "semiHidden",
    "unhideWhenUsed",
    "hidden",
    "locked",
    "cantSplit",
    "tblHeader",
    "noWrap",
    "titlePg",
    "bidiVisual",
    "hideMark",
];

/// Values naming a style by id: shown by the style's name.
const STYLE_REFS: [&str; 6] = ["pStyle", "rStyle", "tblStyle", "basedOn", "next", "link"];

/// Each part of `pkg` the options select, keyed by role (headers and
/// footers) or name, as a tree.
fn package_tree(pkg: &Package, opts: &DiffOptions, filter: &Filter) -> Vec<(String, N)> {
    let roles = story_roles_all(pkg);
    let parse = |data: &[u8]| -> Option<(Dom, NodeId)> {
        let xml = decode_xml(data)?;
        std::panic::catch_unwind(|| {
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&xml);
            let root = dom.root(doc);
            (dom, root)
        })
        .ok()
        .and_then(|(dom, root)| root.map(|r| (dom, r)))
    };
    let style_names: HashMap<String, String> = pkg
        .entries
        .iter()
        .find(|e| e.name == "word/styles.xml")
        .and_then(|e| parse(&e.data))
        .map(|(dom, root)| {
            dom.elements(root, Some(&XNamespace::get(W_NS).name("style")))
                .into_iter()
                .map(|s| {
                    let id = attr(&dom, s, "styleId");
                    let name = dom
                        .element(s, &XNamespace::get(W_NS).name("name"))
                        .map(|n| attr(&dom, n, "val"))
                        .unwrap_or_else(|| id.clone());
                    (id, name)
                })
                .collect()
        })
        .unwrap_or_default();
    let mut out = Vec::new();
    for e in &pkg.entries {
        if e.name.ends_with('/') {
            continue;
        }
        // A part several sections show pairs under each of their roles.
        let keys = roles
            .get(&e.name)
            .cloned()
            .unwrap_or_else(|| vec![e.name.clone()]);
        let xml = e.name.ends_with(".xml") || e.name.ends_with(".rels");
        let targets = if xml {
            rel_targets(pkg, &e.name)
        } else {
            HashMap::new()
        };
        for key in keys {
            if opts
                .part
                .as_ref()
                .is_some_and(|p| !e.name.contains(p.as_str()) && !key.contains(p.as_str()))
            {
                continue;
            }
            // The permissive parser forgives what Word rejects (a
            // mismatched end tag): such a part is reported, not compared.
            let mut malformed = None;
            let parsed = if xml {
                match decode_xml(&e.data).map(|x| crate::xmllinq::parse::validate_xml(&x)) {
                    Some(Err(err)) => {
                        malformed = Some(err);
                        None
                    }
                    _ => parse(&e.data),
                }
            } else {
                None
            };
            let tree = match parsed {
                Some((dom, root)) => {
                    let mut b = Build {
                        dom: &dom,
                        raw: opts.raw,
                        filter,
                        style_names: &style_names,
                        targets: &targets,
                        paras: 0,
                        tables: 0,
                    };
                    let mut n = b.generic(root);
                    n.key = key.clone();
                    n.label = key.clone();
                    n
                }
                None => {
                    let mut n = N::new("bytes", key.clone(), key.clone());
                    n.lines
                        .extend(malformed.map(|err| format!("malformed XML: {err}")));
                    n.lines
                        .push(format!("{} bytes, fnv {:016x}", e.data.len(), fnv(&e.data)));
                    n
                }
            };
            out.push((key, tree));
        }
    }
    out
}

/// What each relationship of `part` points to: an external target as
/// written, an XML part by name, any other part by a hash of its bytes (so
/// a renamed copy of the same image is the same target).
fn rel_targets(pkg: &Package, part: &str) -> HashMap<String, String> {
    let (dir, file) = part.rsplit_once('/').unwrap_or(("", part));
    let rels = if dir.is_empty() {
        format!("_rels/{file}.rels")
    } else {
        format!("{dir}/_rels/{file}.rels")
    };
    let Some(xml) = pkg
        .entries
        .iter()
        .find(|e| e.name == rels)
        .and_then(|e| decode_xml(&e.data))
    else {
        return HashMap::new();
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return HashMap::new();
    };
    dom.elements(root, None)
        .into_iter()
        .map(|r| {
            let target = attr(&dom, r, "Target");
            let shown = if attr(&dom, r, "TargetMode") == "External" {
                target
            } else {
                let joined = match target.strip_prefix('/') {
                    Some(absolute) => absolute.to_string(),
                    None if dir.is_empty() => target.clone(),
                    None => format!("{dir}/{target}"),
                };
                let mut segments: Vec<&str> = Vec::new();
                for s in joined.split('/') {
                    match s {
                        ".." => {
                            segments.pop();
                        }
                        "." | "" => {}
                        _ => segments.push(s),
                    }
                }
                let name = segments.join("/");
                match pkg.entries.iter().find(|e| e.name == name) {
                    Some(t) if !name.ends_with(".xml") => format!("#{:016x}", fnv(&t.data)),
                    _ => name,
                }
            };
            (attr(&dom, r, "Id"), shown)
        })
        .collect()
}

/// FNV-1a: tells binary parts apart.
fn fnv(data: &[u8]) -> u64 {
    data.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// `n`'s attribute with this local name, or "".
fn attr(dom: &Dom, n: NodeId, name: &str) -> String {
    dom.attributes(n)
        .into_iter()
        .find(|(k, _)| k.local_name() == name && !dom.is_namespace_declaration(k))
        .map(|(_, v)| v)
        .unwrap_or_default()
}

/// Builds one part's tree.
struct Build<'a> {
    dom: &'a Dom,
    raw: bool,
    filter: &'a Filter,
    /// Style id → name, from the package's stylesheet.
    style_names: &'a HashMap<String, String>,
    /// Relationship id → what it points to, from the part's rels.
    targets: &'a HashMap<String, String>,
    /// Paragraphs and tables seen so far: their ordinal in labels.
    paras: usize,
    tables: usize,
}

impl Build<'_> {
    fn local(&self, n: NodeId) -> String {
        local(self.dom, n)
    }

    fn elements(&self, n: NodeId) -> Vec<NodeId> {
        self.dom
            .nodes(n)
            .into_iter()
            .filter(|&c| self.dom.is_element(c))
            .collect()
    }

    /// An attribute the diff drops by default.
    fn noise(&self, elem: &str, a: &XName) -> bool {
        if self.dom.is_namespace_declaration(a) {
            return true;
        }
        if self.raw {
            return false;
        }
        let ln = a.local_name();
        let ns = a.namespace().namespace_name();
        ln.starts_with("rsid")
            || (ns == MC_NS && ln == "Ignorable")
            || (ns != W_NS
                && matches!(
                    ln,
                    "paraId" | "textId" | "durableId" | "dateUtc" | "anchorId" | "editId"
                ))
            || (ln == "id"
                && (REVISION_MARKS.contains(&elem)
                    || CHANGES.contains(&elem)
                    || LINK_IDS.contains(&elem)
                    || matches!(elem, "docPr" | "cNvPr")))
            || (matches!(ln, "author" | "date")
                && (REVISION_MARKS.contains(&elem) || CHANGES.contains(&elem)))
            || (elem == "comment" && matches!(ln, "date" | "initials"))
            || (elem == "Relationship" && ln == "Id")
    }

    /// An element the diff drops by default.
    fn noise_element(&self, parent: &str, name: &str) -> bool {
        !self.raw
            && (matches!(
                name,
                "nsid" | "tmpl" | "rsid" | "proofErr" | "lastRenderedPageBreak"
            ) || (parent == "sdtPr" && name == "id")
                // What every save restamps or recounts in docProps.
                || (parent == "coreProperties"
                    && matches!(
                        name,
                        "created" | "modified" | "lastModifiedBy" | "revision" | "lastPrinted"
                    ))
                || (parent == "Properties"
                    && matches!(
                        name,
                        "Application"
                            | "AppVersion"
                            | "TotalTime"
                            | "Pages"
                            | "Words"
                            | "Characters"
                            | "CharactersWithSpaces"
                            | "Lines"
                            | "Paragraphs"
                            | "HeadingPairs"
                            | "TitlesOfParts"
                    )))
    }

    /// `n`'s namespace when it differs from its parent's and says
    /// something: any namespace under --raw, else one outside the Office
    /// vocabularies (Strict and Transitional names compare alike).
    fn namespace_note(&self, n: NodeId) -> Option<String> {
        let ns = |id: NodeId| {
            self.dom
                .name(id)
                .map(|x| x.namespace().namespace_name().to_string())
                .unwrap_or_default()
        };
        let own = ns(n);
        let parent = self
            .dom
            .parent(n)
            .filter(|&p| self.dom.is_element(p))
            .map(ns)
            .unwrap_or_default();
        let office = [
            "http://schemas.openxmlformats.org/",
            "http://purl.oclc.org/ooxml/",
            "http://schemas.microsoft.com/office/",
            "urn:schemas-microsoft-com:",
            "http://www.w3.org/",
            "http://purl.org/dc/",
        ];
        (!own.is_empty()
            && own != parent
            && (self.raw || !office.iter().any(|o| own.starts_with(o))))
        .then_some(own)
    }

    /// The element's attributes, noise dropped, sorted: `k=v`.
    fn attrs(&self, n: NodeId) -> Vec<(String, String)> {
        let elem = self.local(n);
        let mut v: Vec<(String, String)> = self
            .dom
            .attributes(n)
            .into_iter()
            .filter(|(k, _)| !self.noise(&elem, k))
            .filter_map(|(k, v)| {
                if k.namespace().namespace_name() == R_NS {
                    // A relationship id says where it points; the id
                    // itself is renumbered by every save. Header and
                    // footer references pair by section role instead.
                    let target = self.targets.get(&v);
                    let v = match (self.raw, target) {
                        (true, Some(t)) => format!("{v}→{t}"),
                        (true, None) => v,
                        (false, Some(t))
                            if !matches!(elem.as_str(), "headerReference" | "footerReference") =>
                        {
                            format!("→{t}")
                        }
                        (false, _) => return None,
                    };
                    return Some((k.local_name().to_string(), v));
                }
                let v = if !self.raw && elem == "Relationship" && k.local_name() == "Type" {
                    v.rsplit('/').next().unwrap_or(&v).to_string()
                } else if !self.raw
                    && k.local_name() == "val"
                    && STYLE_REFS.contains(&elem.as_str())
                {
                    format!("\"{}\"", self.style_names.get(&v).unwrap_or(&v))
                } else {
                    v
                };
                Some((k.local_name().to_string(), v))
            })
            .collect();
        v.sort();
        v
    }

    /// The element's own attributes as one line, `@(k=v,…)`.
    fn attr_line(&self, n: NodeId) -> Option<String> {
        let attrs = self.attrs(n);
        let a: Vec<String> = attrs.iter().map(|(k, v)| format!("{k}={v}")).collect();
        (!a.is_empty()).then(|| format!("@({})", a.join(",")))
    }

    /// One element as an item: `name`, `name=val`, `name(k=v,…)`, its own
    /// children in braces, its text quoted.
    fn item(&self, c: NodeId) -> String {
        let local = self.local(c);
        let name = match self.namespace_note(c) {
            Some(ns) => format!("{local}<{ns}>"),
            None => local.clone(),
        };
        let attrs = self.attrs(c);
        let mut s = match attrs.as_slice() {
            [] => name.clone(),
            [(k, v)] if k == "val" => match v.as_str() {
                "true" | "on" if !self.raw => name.clone(),
                "1" if !self.raw && TOGGLES.contains(&local.as_str()) => name.clone(),
                "false" | "off" if !self.raw => format!("{name}=0"),
                _ => format!("{name}={v}"),
            },
            _ => {
                let a: Vec<String> = attrs.iter().map(|(k, v)| format!("{k}={v}")).collect();
                format!("{name}({})", a.join(","))
            }
        };
        let kids: Vec<String> = self
            .elements(c)
            .into_iter()
            .filter(|&k| !self.noise_element(&local, &self.local(k)))
            // A text box story is compared as its own node.
            .map(|k| {
                if self.local(k) == "txbxContent" {
                    "txbxContent".to_string()
                } else {
                    self.item(k)
                }
            })
            .collect();
        if !kids.is_empty() {
            s.push('{');
            s.push_str(&kids.join(" "));
            s.push('}');
        } else {
            let text: String = self
                .dom
                .nodes(c)
                .into_iter()
                .filter_map(|t| self.dom.text_value(t))
                .collect();
            if !text.trim().is_empty() {
                let _ = write!(s, " \"{}\"", text.trim());
            }
        }
        s
    }

    /// A child that becomes a node of its own rather than a line.
    fn is_node(&self, c: NodeId) -> bool {
        let l = self.local(c);
        matches!(
            l.as_str(),
            "p" | "tbl"
                | "sdt"
                | "style"
                | "footnote"
                | "endnote"
                | "comment"
                | "body"
                | "txbxContent"
        ) || BLOCKS.contains(&l.as_str())
            || CHANGES.contains(&l.as_str())
            || !self.elements(c).is_empty()
    }

    /// Dispatch on the element's kind.
    fn node(&mut self, e: NodeId) -> Option<N> {
        let l = self.local(e);
        match l.as_str() {
            "style" => Some(self.style(e)),
            "p" => Some(self.paragraph(e)),
            "tbl" => Some(self.table(e)),
            "sdt" => Some(self.sdt(e)),
            "footnote" | "endnote" | "comment" => Some(self.note(e)),
            _ if CHANGES.contains(&l.as_str()) => self.change(e),
            _ if BLOCKS.contains(&l.as_str()) => self.block(e),
            _ => Some(self.generic(e)),
        }
    }

    /// Children as nodes, in order.
    fn content(&mut self, parent: NodeId) -> Vec<N> {
        let pl = self.local(parent);
        let kids: Vec<NodeId> = self
            .elements(parent)
            .into_iter()
            .filter(|&c| !self.noise_element(&pl, &self.local(c)))
            .collect();
        kids.into_iter().filter_map(|c| self.node(c)).collect()
    }

    /// Any element: its attributes and leaf children as lines, the rest as
    /// nodes. Keyed by name and the first identifying attribute.
    fn generic(&mut self, e: NodeId) -> N {
        let name = self.local(e);
        let attrs = self.attrs(e);
        let ident = [
            "abstractNumId",
            "numId",
            "ilvl",
            "type",
            "name",
            "PartName",
            "Extension",
            "uri",
        ]
        .iter()
        .find_map(|k| attrs.iter().find(|(a, _)| a == k))
        .map(|(_, v)| v.clone());
        let label = match &ident {
            Some(v) => format!("{name} {v}"),
            None => name.clone(),
        };
        let mut n = N::new(&name, label.clone(), label);
        n.lines
            .extend(self.namespace_note(e).map(|ns| format!("xmlns={ns}")));
        n.lines.extend(self.attr_line(e));
        let text: String = self
            .dom
            .nodes(e)
            .into_iter()
            .filter_map(|t| self.dom.text_value(t))
            .collect();
        if !text.trim().is_empty() {
            n.lines.push(format!("\"{}\"", text.trim()));
        }
        let mut kids = Vec::new();
        for c in self.elements(e) {
            if self.noise_element(&name, &self.local(c)) {
                continue;
            }
            if self.is_node(c) {
                kids.extend(self.node(c));
            } else {
                n.lines.push(self.item(c));
            }
        }
        n.with_kids(kids)
    }

    /// A property block: each child an item, nested blocks and change
    /// records as nodes. None when empty.
    fn block(&mut self, e: NodeId) -> Option<N> {
        let name = self.local(e);
        let mut n = N::new(&name, name.clone(), name.clone());
        n.block = true;
        n.lines.extend(self.attr_line(e));
        let mut kids = Vec::new();
        for c in self.elements(e) {
            let cl = self.local(c);
            if self.noise_element(&name, &cl) {
                continue;
            }
            let nested = CHANGES.contains(&cl.as_str())
                || (cl != name && matches!(cl.as_str(), "rPr" | "sectPr" | "pPr"));
            if nested {
                kids.extend(self.node(c));
            } else {
                n.lines.push(self.item(c));
            }
        }
        n.lines.sort();
        if n.lines.is_empty() && kids.is_empty() && !self.raw {
            return None;
        }
        Some(n.with_kids(kids))
    }

    /// A change record: the recorded old block's items.
    fn change(&mut self, e: NodeId) -> Option<N> {
        let name = self.local(e);
        let mut n = N::new(&name, name.clone(), name.clone());
        n.block = true;
        n.lines.extend(self.attr_line(e));
        for old in self.elements(e) {
            for c in self.elements(old) {
                n.lines.push(self.item(c));
            }
        }
        n.lines.sort();
        if n.lines.is_empty() {
            n.lines.push("(empty)".to_string());
        }
        Some(n)
    }

    fn style(&mut self, s: NodeId) -> N {
        // An omitted w:type is a paragraph style.
        let ty = match attr(self.dom, s, "type") {
            t if t.is_empty() => "paragraph".to_string(),
            t => t,
        };
        let id = attr(self.dom, s, "styleId");
        let name = self
            .elements(s)
            .into_iter()
            .find(|&c| self.local(c) == "name")
            .map(|c| attr(self.dom, c, "val"))
            .unwrap_or_else(|| id.clone());
        let mut n = N::new(
            "style",
            format!("style {ty} \"{}\"", name.to_lowercase()),
            format!("style {ty} \"{name}\""),
        );
        if matches!(attr(self.dom, s, "default").as_str(), "1" | "true") {
            n.lines.push("default".to_string());
        }
        if matches!(attr(self.dom, s, "customStyle").as_str(), "1" | "true") {
            n.lines.push("customStyle".to_string());
        }
        if self.raw {
            n.lines.push(format!("styleId={id}"));
        }
        let mut kids = Vec::new();
        for c in self.elements(s) {
            let cl = self.local(c);
            match cl.as_str() {
                "name" => {}
                "tblStylePr" => {
                    let t = attr(self.dom, c, "type");
                    let mut k = N::new(
                        "tblStylePr",
                        format!("tblStylePr {t}"),
                        format!("tblStylePr {t}"),
                    );
                    let blocks = self.content(c);
                    k = k.with_kids(blocks);
                    kids.push(k);
                }
                _ if BLOCKS.contains(&cl.as_str()) => kids.extend(self.block(c)),
                _ if !self.noise_element("style", &cl) => n.lines.push(self.item(c)),
                _ => {}
            }
        }
        n.hit = self
            .filter
            .style
            .as_ref()
            .is_some_and(|f| *f == name.to_lowercase() || *f == id.to_lowercase());
        n.with_kids(kids)
    }

    fn paragraph(&mut self, p: NodeId) -> N {
        self.paras += 1;
        let ord = self.paras;
        let mut segs: Vec<Seg> = Vec::new();
        let mut boxes: Vec<NodeId> = Vec::new();
        self.segments(p, "", &mut segs, &mut boxes);
        let text: String = segs
            .iter()
            .filter(|s| s.text_run)
            .map(|s| s.text.as_str())
            .collect();
        let mut n = N::new(
            "p",
            format!("p {text}"),
            format!("p#{ord} \"{}\"", clip(&text, 40)),
        );
        n.lines.extend(self.attr_line(p));
        n.lines.extend(segs.iter().map(Seg::line));
        let mut kids = Vec::new();
        if let Some(ppr) = self
            .elements(p)
            .into_iter()
            .find(|&c| self.local(c) == "pPr")
        {
            kids.extend(self.block(ppr));
        }
        for (i, b) in boxes.into_iter().enumerate() {
            let label = format!("txbx {}", i + 1);
            let content = self.content(b);
            kids.push(N::new("txbx", label.clone(), label).with_kids(content));
        }
        n.hit = self
            .filter
            .para
            .as_ref()
            .is_some_and(|f| text.contains(f.as_str()));
        n.with_kids(kids)
    }

    /// The paragraph content under `n` as run segments, `rev` the revision
    /// mark around it; text box stories go to `boxes`.
    fn segments(&self, n: NodeId, rev: &str, segs: &mut Vec<Seg>, boxes: &mut Vec<NodeId>) {
        for c in self.elements(n) {
            let l = self.local(c);
            match l.as_str() {
                "pPr" => {}
                "r" => self.run(c, rev, segs, boxes),
                "ins" | "del" | "moveFrom" | "moveTo" => {
                    // --raw keeps the mark's id, author and date.
                    let mark = match self.attr_line(c) {
                        Some(a) if self.raw => format!("{l}{a}"),
                        _ => l.clone(),
                    };
                    self.segments(c, &mark, segs, boxes);
                }
                "hyperlink" => {
                    // Where the link leads: its anchor or relationship
                    // target, around the text it covers.
                    let link = match self.attr_line(c) {
                        Some(a) => format!("⟨hyperlink{a}⟩"),
                        None => "⟨hyperlink⟩".to_string(),
                    };
                    segs.push(Seg::marker(rev, link));
                    self.segments(c, rev, segs, boxes);
                    segs.push(Seg::marker(rev, "⟨/hyperlink⟩".to_string()));
                }
                "smartTag" | "customXml" | "sdtContent" | "dir" | "bdo" => {
                    self.segments(c, rev, segs, boxes);
                }
                "sdt" => {
                    for k in self.elements(c) {
                        if self.local(k) == "sdtContent" {
                            self.segments(k, rev, segs, boxes);
                        }
                    }
                }
                "fldSimple" => {
                    Seg::push(
                        segs,
                        rev,
                        "",
                        &format!("{{{}|", attr(self.dom, c, "instr").trim()),
                        true,
                    );
                    self.segments(c, rev, segs, boxes);
                    Seg::push(segs, rev, "", "}", true);
                }
                "oMath" | "oMathPara" => {
                    let t: String = self
                        .dom
                        .descendants(c, None)
                        .into_iter()
                        .filter(|&d| self.local(d) == "t")
                        .map(|d| self.dom.value(d))
                        .collect();
                    Seg::push(segs, rev, "", &format!("[math {t}]"), true);
                }
                "bookmarkStart" => segs.push(Seg::marker(
                    rev,
                    format!("⟨bookmark {}⟩", attr(self.dom, c, "name")),
                )),
                "bookmarkEnd" | "commentRangeStart" | "commentRangeEnd" | "proofErr"
                | "permStart" | "permEnd" | "moveFromRangeStart" | "moveFromRangeEnd"
                | "moveToRangeStart" | "moveToRangeEnd"
                    if !self.raw => {}
                _ => segs.push(Seg::marker(rev, format!("⟨{}⟩", self.item(c)))),
            }
        }
    }

    fn run(&self, r: NodeId, rev: &str, segs: &mut Vec<Seg>, boxes: &mut Vec<NodeId>) {
        let mut fmt = String::new();
        let mut text = String::new();
        for c in self.elements(r) {
            let l = self.local(c);
            match l.as_str() {
                "rPr" => {
                    let mut items = Vec::new();
                    let mut was = None;
                    for k in self.elements(c) {
                        let kl = self.local(k);
                        if kl == "rPrChange" {
                            let old: Vec<String> = self
                                .elements(k)
                                .into_iter()
                                .flat_map(|o| self.elements(o))
                                .map(|o| self.item(o))
                                .collect();
                            was = Some(old.join(" "));
                        } else if !self.noise_element("rPr", &kl) {
                            items.push(self.item(k));
                        }
                    }
                    items.sort();
                    fmt = items.join(" ");
                    if let Some(w) = was {
                        let _ = write!(fmt, "{}was{{{w}}}", if fmt.is_empty() { "" } else { " " });
                    }
                }
                "t" | "delText" => text.push_str(&self.dom.value(c)),
                "instrText" | "delInstrText" => text.push_str(&self.dom.value(c)),
                "fldChar" => text.push_str(match attr(self.dom, c, "fldCharType").as_str() {
                    "begin" => "{",
                    "separate" => "|",
                    "end" => "}",
                    _ => "",
                }),
                "tab" | "ptab" => text.push('⇥'),
                "br" if attr(self.dom, c, "type") == "page" => text.push('⤓'),
                "br" | "cr" => text.push('↵'),
                "noBreakHyphen" => text.push('‑'),
                "softHyphen" => text.push('¬'),
                "lastRenderedPageBreak" if !self.raw => {}
                "drawing" | "pict" | "object" | "AlternateContent" => {
                    // Placement, extent, crop and target are the drawing's
                    // items; its text box stories are nodes of their own.
                    Seg::push(segs, rev, &fmt, &std::mem::take(&mut text), true);
                    segs.push(Seg {
                        rev: rev.to_string(),
                        fmt: fmt.clone(),
                        text: format!("[{}]", self.item(c)),
                        text_run: false,
                    });
                    for d in self.dom.descendants(c, None) {
                        if self.local(d) == "txbxContent"
                            && !self
                                .dom
                                .ancestors(d, None)
                                .into_iter()
                                .take_while(|&a| a != c)
                                .any(|a| {
                                    self.local(a) == "Fallback" || self.local(a) == "txbxContent"
                                })
                        {
                            boxes.push(d);
                        }
                    }
                }
                _ => {
                    let _ = write!(text, "[{}]", self.item(c));
                }
            }
        }
        Seg::push(segs, rev, &fmt, &text, true);
    }

    fn table(&mut self, t: NodeId) -> N {
        self.tables += 1;
        let ord = self.tables;
        let first: String = self
            .dom
            .descendants(t, None)
            .into_iter()
            .filter(|&d| self.local(d) == "t")
            .map(|d| self.dom.value(d))
            .take(8)
            .collect();
        let mut n = N::new(
            "tbl",
            format!("tbl {}", clip(&first, 60)),
            format!("tbl#{ord} \"{}\"", clip(&first, 30)),
        );
        let mut kids = Vec::new();
        let mut row = 0;
        for c in self.elements(t) {
            match self.local(c).as_str() {
                "tr" => {
                    row += 1;
                    kids.push(self.row(c, row));
                }
                _ => kids.extend(self.node(c)),
            }
        }
        n.lines.clear();
        n.with_kids(kids)
    }

    fn row(&mut self, tr: NodeId, i: usize) -> N {
        let text: String = self
            .dom
            .descendants(tr, None)
            .into_iter()
            .filter(|&d| matches!(self.local(d).as_str(), "t" | "delText"))
            .map(|d| self.dom.value(d))
            .collect();
        let n = N::new(
            "tr",
            format!("tr {text}"),
            format!("tr {i} \"{}\"", clip(&text, 30)),
        );
        let mut kids = Vec::new();
        let mut cell = 0;
        for c in self.elements(tr) {
            match self.local(c).as_str() {
                "tc" => {
                    cell += 1;
                    let label = format!("tc {cell}");
                    let content = self.content(c);
                    kids.push(N::new("tc", label.clone(), label).with_kids(content));
                }
                _ => kids.extend(self.node(c)),
            }
        }
        n.with_kids(kids)
    }

    fn sdt(&mut self, s: NodeId) -> N {
        let find = |b: &Self, name: &str| -> String {
            b.dom
                .descendants(s, None)
                .into_iter()
                .find(|&d| b.local(d) == name)
                .map(|d| attr(b.dom, d, "val"))
                .unwrap_or_default()
        };
        let tag = find(self, "tag");
        let tag = if tag.is_empty() {
            find(self, "alias")
        } else {
            tag
        };
        let label = format!("sdt \"{tag}\"");
        let n = N::new("sdt", label.clone(), label);
        let mut kids = Vec::new();
        for c in self.elements(s) {
            if self.local(c) == "sdtContent" {
                kids.extend(self.content(c));
            } else {
                kids.extend(self.node(c));
            }
        }
        n.with_kids(kids)
    }

    /// A footnote, endnote or comment, keyed by its text.
    fn note(&mut self, e: NodeId) -> N {
        let name = self.local(e);
        let text: String = self
            .dom
            .descendants(e, None)
            .into_iter()
            .filter(|&d| matches!(self.local(d).as_str(), "t" | "delText"))
            .map(|d| self.dom.value(d))
            .collect();
        let ty = attr(self.dom, e, "type");
        let label = if ty.is_empty() || ty == "normal" {
            format!("{name} \"{}\"", clip(&text, 30))
        } else {
            format!("{name} {ty}")
        };
        let mut n = N::new(&name, format!("{name} {ty} {text}"), label);
        n.lines.extend(self.attr_line(e));
        let content = self.content(e);
        n.with_kids(content)
    }
}

/// A run of paragraph content: its revision mark, formatting and text.
struct Seg {
    rev: String,
    fmt: String,
    text: String,
    /// Text that joins its neighbours formatted alike (a marker does not).
    text_run: bool,
}

impl Seg {
    fn push(segs: &mut Vec<Seg>, rev: &str, fmt: &str, text: &str, text_run: bool) {
        if text.is_empty() {
            return;
        }
        if let Some(last) = segs.last_mut()
            && last.text_run
            && last.rev == rev
            && last.fmt == fmt
        {
            last.text.push_str(text);
            return;
        }
        segs.push(Seg {
            rev: rev.to_string(),
            fmt: fmt.to_string(),
            text: text.to_string(),
            text_run,
        });
    }

    fn marker(rev: &str, text: String) -> Seg {
        Seg {
            rev: rev.to_string(),
            fmt: String::new(),
            text,
            text_run: false,
        }
    }

    fn line(&self) -> String {
        let mut s = String::new();
        if !self.rev.is_empty() {
            let _ = write!(s, "{}: ", self.rev);
        }
        if self.text_run {
            let _ = write!(s, "\"{}\"", self.text);
        } else {
            s.push_str(&self.text);
        }
        if !self.fmt.is_empty() {
            let _ = write!(s, " [{}]", self.fmt);
        }
        s
    }
}

/// Walks aligned trees and writes hunks.
struct Walker<'a> {
    labels: &'a [&'a str],
    filtered: bool,
    full: bool,
}

impl Walker<'_> {
    /// `nodes`: one element per file (None: absent); `parents`: whether each
    /// file holds the parent (a file without it says nothing here).
    fn walk(
        &self,
        nodes: &[Option<&N>],
        parents: &[bool],
        path: &mut Vec<String>,
        in_scope: bool,
        hunks: &mut Vec<(String, bool)>,
    ) {
        let hit = nodes.iter().flatten().any(|n| n.hit);
        let sub = nodes.iter().flatten().any(|n| n.sub_hit);
        if self.filtered && !in_scope && !hit && !sub {
            return;
        }
        let scope = !self.filtered || in_scope || hit;
        let Some(first) = nodes.iter().flatten().next() else {
            return;
        };
        let pushed = !first.label.is_empty();
        if pushed {
            path.push(first.label.clone());
        }
        if scope && let Some(h) = self.hunk(nodes, parents, path) {
            hunks.push(h);
        }
        let present: Vec<bool> = nodes.iter().map(Option::is_some).collect();
        let lists: Vec<&[N]> = nodes
            .iter()
            .map(|n| n.map_or(&[][..], |n| n.kids.as_slice()))
            .collect();
        for slot in align(&lists) {
            self.walk(&slot, &present, path, scope && self.filtered, hunks);
        }
        if pushed {
            path.pop();
        }
    }

    /// The hunk for one aligned element and whether the files disagree on
    /// it, or None when every file holding its parent agrees on it and
    /// --full is off.
    fn hunk(
        &self,
        nodes: &[Option<&N>],
        parents: &[bool],
        path: &[String],
    ) -> Option<(String, bool)> {
        let block = nodes.iter().flatten().any(|n| n.block);
        let absent: Vec<usize> = (0..nodes.len())
            .filter(|&i| parents[i] && nodes[i].is_none())
            .collect();
        let mut order: Vec<&str> = Vec::new();
        let mut counts: Vec<HashMap<&str, usize>> = vec![HashMap::new(); nodes.len()];
        for (i, n) in nodes.iter().enumerate() {
            for l in n.iter().flat_map(|n| n.lines.iter()) {
                *counts[i].entry(l.as_str()).or_default() += 1;
                if !order.contains(&l.as_str()) {
                    order.push(l);
                }
            }
        }
        // A file without the element holds none of its lines.
        let active: Vec<usize> = (0..nodes.len()).collect();
        let count = |i: usize, l: &str| counts[i].get(l).copied().unwrap_or(0);
        let differs = |l: &str| {
            let c0 = count(active[0], l);
            active.iter().any(|&i| count(i, l) != c0)
        };
        let diff_lines: Vec<&str> = order.iter().copied().filter(|l| differs(l)).collect();
        let show_absent = !block && !absent.is_empty();
        if diff_lines.is_empty() && !show_absent && (!self.full || order.is_empty()) {
            return None;
        }
        let mut out = format!("{}\n", path.join(" › "));
        if show_absent {
            let names: Vec<&str> = absent.iter().map(|&i| self.labels[i]).collect();
            let _ = writeln!(out, "  (absent) [{}]", names.join(" "));
        }
        let rows: Vec<(char, &str)> = order
            .iter()
            .copied()
            .filter(|l| self.full || differs(l))
            .map(|l| {
                let mark = if !differs(l) {
                    ' '
                } else if count(0, l) > 0 {
                    '-'
                } else {
                    '+'
                };
                (mark, l)
            })
            .collect();
        let shown: Vec<String> = rows
            .iter()
            .map(|(m, l)| {
                if *m == ' ' {
                    return clip_line(l);
                }
                // Past the width, show where this line parts from the others.
                let common = diff_lines
                    .iter()
                    .filter(|o| **o != *l)
                    .map(|o| l.chars().zip(o.chars()).take_while(|(a, b)| a == b).count())
                    .max()
                    .unwrap_or(0);
                if common > WIDTH.saturating_sub(60) {
                    clip_line(&clip_from(l, common.saturating_sub(40)))
                } else {
                    clip_line(l)
                }
            })
            .collect();
        let tagged = self.labels.len() > 2;
        let pad = shown
            .iter()
            .map(|s| s.chars().count())
            .max()
            .unwrap_or(0)
            .min(70);
        for ((mark, l), s) in rows.iter().zip(&shown) {
            if tagged && *mark != ' ' {
                let tags: Vec<String> = active
                    .iter()
                    .filter(|&&i| count(i, l) > 0)
                    .map(|&i| match count(i, l) {
                        1 => self.labels[i].to_string(),
                        n => format!("{}×{n}", self.labels[i]),
                    })
                    .collect();
                let w = s.chars().count();
                let _ = writeln!(
                    out,
                    "  {mark} {s}{}  [{}]",
                    " ".repeat(pad.saturating_sub(w)),
                    tags.join(" ")
                );
            } else {
                let _ = writeln!(out, "  {mark} {s}");
            }
        }
        Some((out, !diff_lines.is_empty() || show_absent))
    }
}

/// Content kinds: they align in document order, and within a gap an
/// unmatched one pairs with an unmatched one of its kind (a rewritten
/// paragraph meets the one it replaced). Every other kind is named by its
/// key alone and matches it wherever it stands.
const SEQUENCE: [&str; 9] = [
    "p", "tbl", "tr", "sdt", "footnote", "endnote", "comment", "txbx", "bytes",
];

/// Align each file's children: slots of one element per file.
fn align<'n>(lists: &[&'n [N]]) -> Vec<Vec<Option<&'n N>>> {
    let k = lists.len();
    let is_seq = |n: &N| SEQUENCE.contains(&n.kind.as_str());
    let mut slots: Vec<Vec<Option<&'n N>>> = Vec::new();
    for (f, list) in lists.iter().enumerate() {
        let rep = |s: &Vec<Option<&'n N>>| -> &'n N {
            s.iter()
                .flatten()
                .next()
                .copied()
                .expect("a slot holds an element")
        };
        let fresh = |n: &'n N| {
            let mut s = vec![None; k];
            s[f] = Some(n);
            s
        };
        // Content: an LCS over the content slots, gaps paired by kind.
        let seq_slots: Vec<usize> = (0..slots.len())
            .filter(|&i| is_seq(rep(&slots[i])))
            .collect();
        let seq_items: Vec<&'n N> = list.iter().filter(|n| is_seq(n)).collect();
        let skeys: Vec<&str> = seq_slots
            .iter()
            .map(|&i| rep(&slots[i]).key.as_str())
            .collect();
        let fkeys: Vec<&str> = seq_items.iter().map(|n| n.key.as_str()).collect();
        // New content slots, by the content slot they follow (None: first).
        let mut after: HashMap<Option<usize>, Vec<&'n N>> = HashMap::new();
        let mut last: Option<usize> = None;
        let (mut gap_s, mut gap_f): (Vec<usize>, Vec<&'n N>) = (Vec::new(), Vec::new());
        let mut flush = |gap_s: &mut Vec<usize>,
                         gap_f: &mut Vec<&'n N>,
                         slots: &mut Vec<Vec<Option<&'n N>>>,
                         last: Option<usize>| {
            let mut used = vec![false; gap_f.len()];
            let mut next = 0;
            for &si in gap_s.iter() {
                let kind = rep(&slots[si]).kind.clone();
                if let Some(j) = (next..gap_f.len()).find(|&j| gap_f[j].kind == kind) {
                    slots[si][f] = Some(gap_f[j]);
                    used[j] = true;
                    next = j + 1;
                }
            }
            let tail = gap_s.last().copied().or(last);
            for (j, n) in gap_f.iter().enumerate() {
                if !used[j] {
                    after.entry(tail).or_default().push(n);
                }
            }
            gap_s.clear();
            gap_f.clear();
        };
        for op in lcs_ops(&skeys, &fkeys) {
            match op {
                (Some(i), Some(j)) => {
                    flush(&mut gap_s, &mut gap_f, &mut slots, last);
                    slots[seq_slots[i]][f] = Some(seq_items[j]);
                    last = Some(seq_slots[i]);
                }
                (Some(i), None) => gap_s.push(seq_slots[i]),
                (None, Some(j)) => gap_f.push(seq_items[j]),
                (None, None) => {}
            }
        }
        flush(&mut gap_s, &mut gap_f, &mut slots, last);
        // Everything else: by key, the n-th of a key with the n-th.
        let mut by_key: HashMap<&str, Vec<usize>> = HashMap::new();
        for i in (0..slots.len()).rev() {
            if !is_seq(rep(&slots[i])) && slots[i][f].is_none() {
                by_key
                    .entry(rep(&slots[i]).key.as_str())
                    .or_default()
                    .push(i);
            }
        }
        let mut unmatched = Vec::new();
        for n in list.iter().filter(|n| !is_seq(n)) {
            match by_key.get_mut(n.key.as_str()).and_then(Vec::pop) {
                Some(i) => slots[i][f] = Some(n),
                None => unmatched.push(n),
            }
        }
        let old = std::mem::take(&mut slots);
        slots.extend(
            after
                .remove(&None)
                .unwrap_or_default()
                .into_iter()
                .map(fresh),
        );
        for (i, s) in old.into_iter().enumerate() {
            slots.push(s);
            slots.extend(
                after
                    .remove(&Some(i))
                    .unwrap_or_default()
                    .into_iter()
                    .map(fresh),
            );
        }
        slots.extend(unmatched.into_iter().map(fresh));
    }
    slots
}

/// An edit script between `a` and `b`: `(Some(i), Some(j))` a match,
/// `(Some(i), None)` only in `a`, `(None, Some(j))` only in `b`. A longest
/// common subsequence; past the size an LCS affords, a greedy in-order
/// match by key.
fn lcs_ops(a: &[&str], b: &[&str]) -> Vec<(Option<usize>, Option<usize>)> {
    let pre = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suf = a[pre..]
        .iter()
        .rev()
        .zip(b[pre..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (n, m) = (a.len() - pre - suf, b.len() - pre - suf);
    let mut ops: Vec<(Option<usize>, Option<usize>)> =
        (0..pre).map(|i| (Some(i), Some(i))).collect();
    if n.saturating_mul(m) > 4_000_000 {
        let mut by_key: HashMap<&str, Vec<usize>> = HashMap::new();
        for j in (pre..pre + m).rev() {
            by_key.entry(b[j]).or_default().push(j);
        }
        let mut next_j = pre;
        for (i, key) in a.iter().enumerate().skip(pre).take(n) {
            let hit = by_key.get_mut(key).and_then(|v| {
                while v.last().is_some_and(|&j| j < next_j) {
                    v.pop();
                }
                v.pop()
            });
            match hit {
                Some(j) => {
                    ops.extend((next_j..j).map(|x| (None, Some(x))));
                    ops.push((Some(i), Some(j)));
                    next_j = j + 1;
                }
                None => ops.push((Some(i), None)),
            }
        }
        ops.extend((next_j..pre + m).map(|x| (None, Some(x))));
    } else {
        let w = m + 1;
        let mut lcs = vec![0u32; (n + 1) * w];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                lcs[i * w + j] = if a[pre + i] == b[pre + j] {
                    lcs[(i + 1) * w + j + 1] + 1
                } else {
                    lcs[(i + 1) * w + j].max(lcs[i * w + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < n || j < m {
            if i < n && j < m && a[pre + i] == b[pre + j] {
                ops.push((Some(pre + i), Some(pre + j)));
                i += 1;
                j += 1;
            } else if j == m || (i < n && lcs[(i + 1) * w + j] >= lcs[i * w + j + 1]) {
                ops.push((Some(pre + i), None));
                i += 1;
            } else {
                ops.push((None, Some(pre + j)));
                j += 1;
            }
        }
    }
    let (sa, sb) = (a.len() - suf, b.len() - suf);
    ops.extend((0..suf).map(|x| (Some(sa + x), Some(sb + x))));
    ops
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    const TYPES: &str = r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/></Types>"#;
    const ROOT_RELS: &str = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

    /// A package with `body` (inside `w:body`), a stylesheet holding `styles`
    /// and, when given, a default header part `header` named `hdr_name`.
    fn docx(body: &str, styles: &str, header: Option<(&str, &str)>) -> Vec<u8> {
        let mut rels = String::from(
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
        );
        let mut sect = String::new();
        if let Some((name, _)) = header {
            rels.push_str(&format!(
                r#"<Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="{name}"/>"#
            ));
            sect = r#"<w:sectPr><w:headerReference w:type="default" r:id="rId9"/></w:sectPr>"#
                .to_string();
        }
        rels.push_str("</Relationships>");
        let document = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="{W}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:body>{body}{sect}</w:body></w:document>"#
        );
        let styles = format!(r#"<w:styles xmlns:w="{W}">{styles}</w:styles>"#);
        let mut entries: Vec<(String, String)> = vec![
            ("[Content_Types].xml".into(), TYPES.into()),
            ("_rels/.rels".into(), ROOT_RELS.into()),
            ("word/document.xml".into(), document),
            ("word/_rels/document.xml.rels".into(), rels),
            ("word/styles.xml".into(), styles),
        ];
        if let Some((name, xml)) = header {
            entries.push((
                format!("word/{name}"),
                format!(r#"<w:hdr xmlns:w="{W}">{xml}</w:hdr>"#),
            ));
        }
        let mut buf = Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let opt = zip::write::SimpleFileOptions::default();
            for (name, data) in &entries {
                z.start_file(name.as_str(), opt).unwrap();
                z.write_all(data.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    fn body_text(id: &str, name: &str, rpr: &str) -> String {
        format!(
            r#"<w:style w:type="paragraph" w:styleId="{id}"><w:name w:val="{name}"/><w:basedOn w:val="Normal"/><w:rPr>{rpr}</w:rPr></w:style><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>"#
        )
    }

    fn run(files: &[(&str, &[u8])], opts: &DiffOptions) -> String {
        diff(files, opts).unwrap()
    }

    /// Styles pair by type and name, not by id; each hunk names the style
    /// and the block, and prints only the items that differ.
    #[test]
    fn styles_pair_by_name_and_print_only_their_differing_items() {
        let a = docx(
            "",
            &body_text(
                "BodyText",
                "Body Text",
                r#"<w:rFonts w:ascii="Arial"/><w:b/>"#,
            ),
            None,
        );
        let b = docx(
            "",
            &body_text(
                "Textoindependiente",
                "Body Text",
                r#"<w:b/><w:sz w:val="20"/>"#,
            ),
            None,
        );
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(
            out.contains("word/styles.xml › style paragraph \"Body Text\" › rPr\n"),
            "{out}"
        );
        assert!(out.contains("\n  - rFonts(ascii=Arial)\n"), "{out}");
        assert!(out.contains("\n  + sz=20\n"), "{out}");
        assert!(
            !out.contains("  - b\n") && !out.contains("  + b\n"),
            "{out}"
        );
        assert!(!out.contains("Textoindependiente"), "ids are noise: {out}");
    }

    /// rsids, paragraph ids, revision ids/authors/dates, attribute order, an
    /// explicit on/off `w:val="1"`/`"true"` and empty property shells are
    /// noise; `raw` keeps them.
    #[test]
    fn noise_is_dropped_unless_raw() {
        let a = docx(
            r#"<w:p w14:paraId="11111111" w:rsidR="00AB"><w:pPr/><w:r><w:rPr><w:b/><w:lang w:val="en-US" w:eastAsia="en-US"/></w:rPr><w:t>kept</w:t></w:r><w:ins w:id="3" w:author="X" w:date="2026-01-01T00:00:00Z"><w:r><w:t> more</w:t></w:r></w:ins></w:p>"#,
            "",
            None,
        );
        let b = docx(
            r#"<w:p w14:paraId="22222222" w:rsidR="00CD"><w:r><w:rPr><w:b w:val="1"/><w:lang w:eastAsia="en-US" w:val="en-US"/></w:rPr><w:t>kept</w:t></w:r><w:ins w:id="9" w:author="Y" w:date="2026-02-02T00:00:00Z"><w:r><w:rPr><w:bCs w:val="true"/></w:rPr><w:t> more</w:t></w:r></w:ins></w:p>"#,
            "",
            None,
        );
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(out.contains("\n  + ins: \" more\" [bCs]\n"), "{out}");
        assert!(out.contains("\n  - ins: \" more\"\n"), "{out}");
        assert!(
            !out.contains("  \"kept\""),
            "the first run is identical: {out}"
        );
        assert!(!out.contains("paraId") && !out.contains("rsid"), "{out}");
        assert!(out.ends_with("1 difference\n"), "{out}");

        let same = docx(
            r#"<w:p w14:paraId="33333333"><w:r><w:rPr><w:b w:val="true"/><w:lang w:eastAsia="en-US" w:val="en-US"/></w:rPr><w:t>kept</w:t></w:r><w:ins w:id="1" w:author="Z"><w:r><w:t> more</w:t></w:r></w:ins></w:p>"#,
            "",
            None,
        );
        let out = run(&[("A", &a), ("B", &same)], &DiffOptions::default());
        assert!(out.ends_with("no differences\n"), "{out}");

        let raw = DiffOptions {
            raw: true,
            ..Default::default()
        };
        let out = run(&[("A", &a), ("B", &same)], &raw);
        assert!(out.contains("paraId"), "{out}");
    }

    /// Paragraphs pair by their text: a paragraph inserted before them does
    /// not shift the comparison, and it shows as absent from A.
    #[test]
    fn paragraphs_pair_by_text_not_position() {
        let a = docx(
            r#"<w:p><w:r><w:t>Alpha</w:t></w:r></w:p><w:p><w:pPr><w:jc w:val="left"/></w:pPr><w:r><w:t>Beta</w:t></w:r></w:p>"#,
            "",
            None,
        );
        let b = docx(
            r#"<w:p><w:r><w:t>New</w:t></w:r></w:p><w:p><w:r><w:t>Alpha</w:t></w:r></w:p><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>Beta</w:t></w:r></w:p>"#,
            "",
            None,
        );
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(
            out.contains(
                "word/document.xml › body › p#2 \"Beta\" › pPr\n  - jc=left\n  + jc=center\n"
            ),
            "{out}"
        );
        assert!(
            out.contains("word/document.xml › body › p#1 \"New\"\n  (absent) [A]\n  + \"New\"\n"),
            "{out}"
        );
        assert!(!out.contains("\"Alpha\""), "Alpha is identical: {out}");
    }

    /// A paragraph whose text changed in place still pairs with its
    /// counterpart and shows the text that differs.
    #[test]
    fn a_rewritten_paragraph_pairs_with_the_one_it_replaced() {
        let a = docx(
            r#"<w:p><w:r><w:t>Start</w:t></w:r></w:p><w:p><w:r><w:t>old words</w:t></w:r></w:p><w:p><w:r><w:t>End</w:t></w:r></w:p>"#,
            "",
            None,
        );
        let b = docx(
            r#"<w:p><w:r><w:t>Start</w:t></w:r></w:p><w:p><w:r><w:t>new words</w:t></w:r></w:p><w:p><w:r><w:t>End</w:t></w:r></w:p>"#,
            "",
            None,
        );
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(
            out.contains("› p#2 \"old words\"\n  - \"old words\"\n  + \"new words\"\n"),
            "{out}"
        );
        assert!(!out.contains("(absent)"), "{out}");
    }

    /// With three files each differing line carries the files that hold it.
    #[test]
    fn three_files_tag_each_line_with_the_files_holding_it() {
        let a = docx(
            "",
            &body_text("BodyText", "Body Text", r#"<w:rFonts w:ascii="Arial"/>"#),
            None,
        );
        let b = docx("", &body_text("BodyText", "Body Text", ""), None);
        let c = docx(
            "",
            &body_text("BT", "Body Text", r#"<w:sz w:val="20"/>"#),
            None,
        );
        let out = run(
            &[("A", &a), ("ours_rej", &b), ("word_rej", &c)],
            &DiffOptions::default(),
        );
        assert!(out.starts_with("files: A · ours_rej · word_rej\n"), "{out}");
        let hunk = out
            .split("style paragraph \"Body Text\" › rPr\n")
            .nth(1)
            .expect("rPr hunk");
        let mut lines = hunk.lines();
        let first = lines.next().unwrap();
        assert!(
            first.starts_with("  - rFonts(ascii=Arial)") && first.ends_with("[A]"),
            "{out}"
        );
        let second = lines.next().unwrap();
        assert!(
            second.starts_with("  + sz=20") && second.ends_with("[word_rej]"),
            "{out}"
        );
    }

    /// `style` and `para_text` narrow the report to one element, and `full`
    /// prints its common lines too.
    #[test]
    fn filters_narrow_to_one_element() {
        let styles_a = format!(
            "{}{}",
            body_text("BodyText", "Body Text", r#"<w:b/><w:i/>"#),
            r#"<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:rPr><w:sz w:val="56"/></w:rPr></w:style>"#
        );
        let styles_b = format!(
            "{}{}",
            body_text("BodyText", "Body Text", r#"<w:b/>"#),
            r#"<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:rPr><w:sz w:val="48"/></w:rPr></w:style>"#
        );
        let a = docx(
            r#"<w:p><w:r><w:t>one</w:t></w:r></w:p><w:p><w:r><w:t>two</w:t></w:r></w:p>"#,
            &styles_a,
            None,
        );
        let b = docx(
            r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>one</w:t></w:r></w:p><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>two</w:t></w:r></w:p>"#,
            &styles_b,
            None,
        );
        let files: [(&str, &[u8]); 2] = [("A", &a), ("B", &b)];
        let style = DiffOptions {
            style: Some("body text".into()),
            ..Default::default()
        };
        let out = run(&files, &style);
        assert!(out.contains("\"Body Text\" › rPr\n  - i\n"), "{out}");
        assert!(
            !out.contains("Title") && !out.contains("document.xml"),
            "{out}"
        );

        let para = DiffOptions {
            para_text: Some("two".into()),
            ..Default::default()
        };
        let out = run(&files, &para);
        assert!(out.contains("p#2 \"two\""), "{out}");
        assert!(
            !out.contains("\"one\"") && !out.contains("styles.xml"),
            "{out}"
        );

        let full = DiffOptions {
            style: Some("Body Text".into()),
            full: true,
            ..Default::default()
        };
        let out = run(&files, &full);
        assert!(out.contains("\n    b\n"), "common lines under full: {out}");
    }

    /// Styles match by key whatever their order, and a style one file lacks
    /// never pairs with another style.
    #[test]
    fn styles_match_by_name_in_any_order_and_never_cross_pair() {
        let style = |id: &str, rpr: &str| {
            format!(
                r#"<w:style w:type="paragraph" w:styleId="{id}"><w:name w:val="{id}"/><w:rPr>{rpr}</w:rPr></w:style>"#
            )
        };
        let a = docx(
            "",
            &format!(
                "{}{}",
                style("Title", r#"<w:sz w:val="56"/>"#),
                style("Quote", "<w:i/>")
            ),
            None,
        );
        let b = docx(
            "",
            &format!(
                "{}{}{}",
                style("Heading", r#"<w:sz w:val="32"/>"#),
                style("Quote", "<w:i/>"),
                style("Title", r#"<w:sz w:val="56"/>"#)
            ),
            None,
        );
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(
            out.contains("style paragraph \"Heading\"\n  (absent) [A]\n"),
            "{out}"
        );
        assert!(
            out.contains("style paragraph \"Heading\" › rPr\n  + sz=32\n"),
            "{out}"
        );
        assert!(!out.contains("Title") && !out.contains("Quote"), "{out}");
        assert!(out.ends_with("2 differences\n"), "{out}");
    }

    /// Cells compare by their position in the row, and a field shows as
    /// `{code|result}` in its run segments.
    #[test]
    fn table_cells_pair_by_position_and_fields_show_their_code() {
        let table = |second: &str, w: &str| {
            format!(
                r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tr><w:tc><w:tcPr><w:tcW w:w="{w}" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Name</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>{second}</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#
            )
        };
        let a = docx(&table("Old", "2000"), "", None);
        let b = docx(&table("New", "2400"), "", None);
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(
            out.contains("› tbl#1 \"NameOld\" › tr 1 \"NameOld\" › tc 1 › tcPr\n  - tcW(type=dxa,w=2000)\n  + tcW(type=dxa,w=2400)\n"),
            "{out}"
        );
        assert!(
            out.contains("› tc 2 › p#2 \"Old\"\n  - \"Old\"\n  + \"New\"\n"),
            "{out}"
        );
        assert!(!out.contains("PAGE"), "the field is identical: {out}");
        let c = docx(
            &table("Old", "2000").replace(" PAGE ", " NUMPAGES "),
            "",
            None,
        );
        let out = run(&[("A", &a), ("C", &c)], &DiffOptions::default());
        assert!(
            out.contains("  - \"{ PAGE |1}\"\n  + \"{ NUMPAGES |1}\"\n"),
            "{out}"
        );
    }

    /// Headers pair by the section role that shows them, whatever their
    /// part names.
    #[test]
    fn headers_pair_by_role() {
        let a = docx(
            "",
            "",
            Some(("header1.xml", r#"<w:p><w:r><w:t>Head A</w:t></w:r></w:p>"#)),
        );
        let b = docx(
            "",
            "",
            Some(("header3.xml", r#"<w:p><w:r><w:t>Head B</w:t></w:r></w:p>"#)),
        );
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(
            out.contains("section 1 default header › p#1 \"Head A\"\n"),
            "{out}"
        );
        assert!(!out.contains("(absent)"), "{out}");
    }

    const REL_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

    /// A package of exactly these parts, the content types and root rels
    /// added.
    fn package(parts: &[(&str, &str)]) -> Vec<u8> {
        let mut entries: Vec<(&str, &str)> =
            vec![("[Content_Types].xml", TYPES), ("_rels/.rels", ROOT_RELS)];
        entries.extend_from_slice(parts);
        let mut buf = Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let opt = zip::write::SimpleFileOptions::default();
            for (name, data) in &entries {
                z.start_file(*name, opt).unwrap();
                z.write_all(data.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    fn document(body: &str) -> String {
        format!(
            r#"<w:document xmlns:w="{W}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><w:body>{body}</w:body></w:document>"#
        )
    }

    fn rels(items: &str) -> String {
        format!(r#"<Relationships xmlns="{REL_NS}">{items}</Relationships>"#)
    }

    /// A link that leads elsewhere is a difference, by its anchor or by
    /// the target its relationship names.
    #[test]
    fn hyperlink_destinations_are_compared() {
        let link =
            r#"<w:p><w:hyperlink r:id="rId5"><w:r><w:t>Site</w:t></w:r></w:hyperlink></w:p>"#;
        let to = |url: &str| {
            rels(&format!(
                r#"<Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{url}" TargetMode="External"/>"#
            ))
        };
        let (doc, ra, rb) = (
            document(link),
            to("https://a.example/"),
            to("https://b.example/"),
        );
        let a = package(&[
            ("word/document.xml", &doc),
            ("word/_rels/document.xml.rels", &ra),
        ]);
        let b = package(&[
            ("word/document.xml", &doc),
            ("word/_rels/document.xml.rels", &rb),
        ]);
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(
            out.contains("b.example") && out.contains("p#1 \"Site\""),
            "{out}"
        );

        let anchored = |name: &str| {
            document(&format!(
                r#"<w:p><w:hyperlink w:anchor="{name}"><w:r><w:t>Site</w:t></w:r></w:hyperlink></w:p>"#
            ))
        };
        let (da, db) = (anchored("One"), anchored("Two"));
        let a = package(&[("word/document.xml", &da)]);
        let b = package(&[("word/document.xml", &db)]);
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(out.contains("anchor=Two"), "{out}");
    }

    /// A drawing is its items: a new extent or an embed that now points
    /// at other bytes is a difference.
    #[test]
    fn drawings_compare_their_placement_and_target() {
        let drawing = |cx: &str| {
            document(&format!(
                r#"<w:p><w:r><w:drawing><wp:inline><wp:extent cx="{cx}" cy="100"/><wp:docPr id="1" name="P"/><a:graphic><a:graphicData><a:blip r:embed="rId7"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
            ))
        };
        let image = |target: &str| {
            rels(&format!(
                r#"<Relationship Id="rId7" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="{target}"/>"#
            ))
        };
        let (d100, d200) = (drawing("100"), drawing("200"));
        let (one, two) = (image("media/one.png"), image("media/two.png"));
        let parts = |doc: &str, rels: &str| {
            package(&[
                ("word/document.xml", doc),
                ("word/_rels/document.xml.rels", rels),
                ("word/media/one.png", "AAA"),
                ("word/media/two.png", "BBB"),
            ])
        };
        let a = parts(&d100, &one);
        let out = run(
            &[("A", &a), ("B", &parts(&d200, &one))],
            &DiffOptions::default(),
        );
        assert!(out.contains("cx=200"), "{out}");
        let out = run(
            &[("A", &a), ("B", &parts(&d100, &two))],
            &DiffOptions::default(),
        );
        assert!(out.contains("p#1 \"\"\n  - [drawing{"), "{out}");
        assert!(out.contains("embed=→#"), "{out}");
        let out = run(
            &[("A", &a), ("B", &parts(&d100, &one))],
            &DiffOptions::default(),
        );
        assert!(out.ends_with("no differences\n"), "{out}");
        // The drawing's run keeps its formatting.
        let lang = d100.replace(
            "<w:r><w:drawing>",
            r#"<w:r><w:rPr><w:lang w:val="en-US"/></w:rPr><w:drawing>"#,
        );
        let out = run(
            &[("A", &a), ("B", &parts(&lang, &one))],
            &DiffOptions::default(),
        );
        assert!(out.contains(" [lang=en-US]\n"), "{out}");
    }

    /// A style without w:type is a paragraph style.
    #[test]
    fn an_omitted_style_type_is_paragraph() {
        let a = docx(
            "",
            r#"<w:style w:styleId="X"><w:name w:val="X"/></w:style>"#,
            None,
        );
        let b = docx(
            "",
            r#"<w:style w:type="paragraph" w:styleId="X"><w:name w:val="X"/></w:style>"#,
            None,
        );
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(out.ends_with("no differences\n"), "{out}");
    }

    /// --full prints agreeing elements without counting them.
    #[test]
    fn full_context_is_not_a_difference() {
        let a = docx(
            r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>Same</w:t></w:r></w:p>"#,
            &body_text("BodyText", "Body Text", "<w:b/>"),
            None,
        );
        let full = DiffOptions {
            full: true,
            ..DiffOptions::default()
        };
        let out = run(&[("A", &a), ("B", &a)], &full);
        assert!(
            out.contains("\"Same\""),
            "--full prints the common lines: {out}"
        );
        assert!(out.ends_with("no differences\n"), "{out}");
    }

    /// docProps compare their meaningful fields; the save stamps do not.
    #[test]
    fn document_properties_compare_all_but_save_stamps() {
        let core = |title: &str, modified: &str| {
            format!(
                r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/"><dc:title>{title}</dc:title><dcterms:modified>{modified}</dcterms:modified></cp:coreProperties>"#
            )
        };
        let doc = document("<w:p/>");
        let with =
            |core: &str| package(&[("word/document.xml", &doc), ("docProps/core.xml", core)]);
        let a = with(&core("Lease", "2026-01-01T00:00:00Z"));
        let restamped = with(&core("Lease", "2026-02-02T00:00:00Z"));
        let out = run(&[("A", &a), ("B", &restamped)], &DiffOptions::default());
        assert!(out.ends_with("no differences\n"), "{out}");
        let retitled = with(&core("Sublease", "2026-01-01T00:00:00Z"));
        let out = run(&[("A", &a), ("B", &retitled)], &DiffOptions::default());
        assert!(out.contains("+ title \"Sublease\""), "{out}");
    }

    /// An element that moves to another namespace is a difference.
    #[test]
    fn a_custom_namespace_is_compared() {
        let item = |ns: &str| format!(r#"<root xmlns="{ns}"><v>1</v></root>"#);
        let doc = document("<w:p/>");
        let (ia, ib) = (item("urn:a"), item("urn:b"));
        let a = package(&[("word/document.xml", &doc), ("customXml/item1.xml", &ia)]);
        let b = package(&[("word/document.xml", &doc), ("customXml/item1.xml", &ib)]);
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(out.contains("+ xmlns=urn:b"), "{out}");
        // Office vocabularies say nothing by default.
        let out = run(&[("A", &a), ("B", &a)], &DiffOptions::default());
        assert!(out.ends_with("no differences\n"), "{out}");
    }

    /// A part Word rejects (a mismatched end tag) is reported, not read
    /// as the well-formed part it resembles.
    #[test]
    fn malformed_xml_is_not_compared_as_a_tree() {
        let doc = document("<w:p/>");
        let a = package(&[
            ("word/document.xml", &doc),
            ("customXml/item1.xml", "<root><a>1</a></root>"),
        ]);
        let b = package(&[
            ("word/document.xml", &doc),
            ("customXml/item1.xml", "<root><a>1</b></root>"),
        ]);
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(out.contains("+ malformed XML"), "{out}");
    }

    /// --raw keeps a revision mark's author, date and id.
    #[test]
    fn raw_keeps_revision_metadata() {
        let ins = |author: &str| {
            docx(
                &format!(
                    r#"<w:p><w:ins w:id="1" w:author="{author}" w:date="2026-01-01T00:00:00Z"><w:r><w:t>x</w:t></w:r></w:ins></w:p>"#
                ),
                "",
                None,
            )
        };
        let (a, b) = (ins("Ann"), ins("Bob"));
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(out.ends_with("no differences\n"), "{out}");
        let raw = DiffOptions {
            raw: true,
            ..DiffOptions::default()
        };
        let out = run(&[("A", &a), ("B", &b)], &raw);
        assert!(out.contains("author=Bob"), "{out}");
    }

    /// A header part two sections share pairs under both roles.
    #[test]
    fn a_shared_header_pairs_under_every_role() {
        let head = format!(r#"<w:hdr xmlns:w="{W}"><w:p><w:r><w:t>Head</w:t></w:r></w:p></w:hdr>"#);
        let rel = |id: &str, target: &str| {
            format!(
                r#"<Relationship Id="{id}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="{target}"/>"#
            )
        };
        let body = |last: &str| {
            document(&format!(
                r#"<w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="rId9"/></w:sectPr></w:pPr></w:p><w:sectPr><w:headerReference w:type="default" r:id="{last}"/></w:sectPr>"#
            ))
        };
        let (da, ra) = (body("rId9"), rels(&rel("rId9", "header1.xml")));
        let a = package(&[
            ("word/document.xml", &da),
            ("word/_rels/document.xml.rels", &ra),
            ("word/header1.xml", &head),
        ]);
        let (db, rb) = (
            body("rId10"),
            rels(&format!(
                "{}{}",
                rel("rId9", "header1.xml"),
                rel("rId10", "header2.xml")
            )),
        );
        let b = package(&[
            ("word/document.xml", &db),
            ("word/_rels/document.xml.rels", &rb),
            ("word/header1.xml", &head),
            ("word/header2.xml", &head),
        ]);
        let out = run(&[("A", &a), ("B", &b)], &DiffOptions::default());
        assert!(!out.contains("section 2 default header"), "{out}");
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod boundary_coverage_tests {
    use super::*;

    fn with_build(fragment: &str, raw: bool, f: impl FnOnce(&mut Build<'_>, NodeId)) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(r#"<root xmlns:w="{W_NS}" xmlns:r="{R_NS}" xmlns:mc="{MC_NS}" xmlns:x="urn:custom">{fragment}</root>"#));
        let root = dom.root(doc).unwrap();
        let filter = Filter {
            style: None,
            para: None,
        };
        let names = HashMap::from([("Heading1".into(), "Heading 1".into())]);
        let targets = HashMap::from([("rId1".into(), "https://example.test/".into())]);
        let mut build = Build {
            dom: &dom,
            raw,
            filter: &filter,
            style_names: &names,
            targets: &targets,
            paras: 0,
            tables: 0,
        };
        let node = dom.elements(root, None)[0];
        f(&mut build, node);
    }

    #[test]
    fn toggle_normalization_distinguishes_boolean_values_from_numeric_properties() {
        for (property, value, normalized) in [
            ("b", "true", "b"),
            ("b", "on", "b"),
            ("b", "1", "b"),
            ("b", "false", "b=0"),
            ("b", "off", "b=0"),
            ("sz", "1", "sz=1"),
            ("sz", "12", "sz=12"),
        ] {
            let fragment = format!("<w:{property} w:val=\"{value}\"/>");
            with_build(&fragment, false, |build, node| {
                assert_eq!(build.item(node), normalized);
            });
            with_build(&fragment, true, |build, node| {
                assert_eq!(build.item(node), format!("{property}<{W_NS}>={value}"));
            });
        }
        with_build(
            "<x:custom x:kind=\"1\">text</x:custom>",
            false,
            |build, node| {
                assert_eq!(build.item(node), "custom<urn:custom>(kind=1) \"text\"");
            },
        );
        with_build("<w:b><w:proofErr/></w:b>", false, |build, node| {
            assert_eq!(build.item(node), "b");
        });
    }

    #[test]
    fn relationship_attributes_resolve_targets_without_inventing_missing_targets() {
        for (raw, expected) in [
            (false, "→https://example.test/"),
            (true, "rId1→https://example.test/"),
        ] {
            with_build("<w:hyperlink r:id=\"rId1\"/>", raw, |build, node| {
                assert_eq!(build.attrs(node), [("id".into(), expected.into())]);
            });
        }
        with_build("<w:hyperlink r:id=\"missing\"/>", true, |build, node| {
            assert_eq!(build.attrs(node), [("id".into(), "missing".into())]);
        });
        with_build("<w:hyperlink r:id=\"missing\"/>", false, |build, node| {
            assert!(build.attrs(node).is_empty());
        });
        with_build(
            "<w:headerReference r:id=\"rId1\"/>",
            false,
            |build, node| assert!(build.attrs(node).is_empty()),
        );
        with_build("<w:pStyle w:val=\"Heading1\"/>", false, |build, node| {
            assert_eq!(build.item(node), "pStyle=\"Heading 1\"");
        });
    }

    #[test]
    fn noisy_metadata_drops_only_save_artifacts_and_raw_keeps_it() {
        for (element, namespace, attribute, expected_noise) in [
            ("p", W_NS, "rsidR", true),
            ("p", MC_NS, "Ignorable", true),
            ("p", "urn:custom", "paraId", true),
            ("p", W_NS, "paraId", false),
            ("ins", W_NS, "id", true),
            ("pPrChange", W_NS, "id", true),
            ("bookmarkStart", W_NS, "id", true),
            ("docPr", "", "id", true),
            ("cNvPr", "", "id", true),
            ("ins", W_NS, "author", true),
            ("rPrChange", W_NS, "date", true),
            ("comment", W_NS, "initials", true),
            ("Relationship", "", "Id", true),
            ("p", W_NS, "val", false),
        ] {
            with_build("<w:p/>", false, |build, _| {
                assert_eq!(
                    build.noise(element, &XName::get(attribute, namespace)),
                    expected_noise
                );
            });
            with_build("<w:p/>", true, |build, _| {
                assert!(!build.noise(element, &XName::get(attribute, namespace)));
            });
        }
        for (parent, name, expected) in [
            ("p", "proofErr", true),
            ("sdtPr", "id", true),
            ("pPr", "id", false),
            ("coreProperties", "modified", true),
            ("coreProperties", "title", false),
            ("Properties", "Words", true),
            ("Properties", "custom", false),
        ] {
            with_build("<w:p/>", false, |build, _| {
                assert_eq!(build.noise_element(parent, name), expected);
            });
            with_build("<w:p/>", true, |build, _| {
                assert!(!build.noise_element(parent, name));
            });
        }
    }

    #[test]
    fn run_projection_preserves_field_delimiters_breaks_and_old_formatting() {
        let fragment = "<w:r><w:rPr><w:b/><w:rPrChange><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:fldChar w:fldCharType=\"begin\"/><w:instrText>CODE</w:instrText><w:fldChar w:fldCharType=\"separate\"/><w:t>result</w:t><w:fldChar w:fldCharType=\"end\"/><w:fldChar w:fldCharType=\"unknown\"/><w:tab/><w:ptab/><w:br w:type=\"page\"/><w:br/><w:cr/><w:noBreakHyphen/><w:softHyphen/><w:annotationRef/></w:r>";
        with_build(fragment, false, |build, node| {
            let mut segments = Vec::new();
            build.run(node, "del", &mut segments, &mut Vec::new());
            assert_eq!(segments.len(), 1);
            assert_eq!(segments[0].rev, "del");
            assert_eq!(segments[0].fmt, "b was{i}");
            assert_eq!(segments[0].text, "{CODE|result}⇥⇥⤓↵↵‑¬[annotationRef]");
        });
    }

    #[test]
    fn empty_property_blocks_and_change_records_are_distinct() {
        for raw in [false, true] {
            with_build("<w:pPr/>", raw, |build, node| {
                assert_eq!(build.block(node).is_some(), raw);
            });
            with_build("<w:pPrChange/>", raw, |build, node| {
                assert_eq!(build.change(node).unwrap().lines, ["(empty)"]);
            });
        }
        with_build(
            "<w:sdt><w:sdtPr><w:alias w:val=\"Alias\"/></w:sdtPr><w:sdtContent><w:p/></w:sdtContent></w:sdt>",
            false,
            |build, node| assert_eq!(build.sdt(node).label, "sdt \"Alias\""),
        );
        with_build(
            "<w:footnote w:type=\"separator\"><w:p/></w:footnote>",
            false,
            |build, node| assert_eq!(build.note(node).label, "footnote separator"),
        );
        with_build(
            "<w:comment w:type=\"normal\"><w:p><w:r><w:t>note</w:t></w:r></w:p></w:comment>",
            false,
            |build, node| assert_eq!(build.note(node).label, "comment \"note\""),
        );
    }

    #[test]
    fn hunk_distinguishes_empty_blocks_absence_and_multiplicity() {
        let mut a = N::new("p", "a".into(), "paragraph".into());
        a.lines = vec!["same".into()];
        let mut b = N::new("p", "a".into(), "paragraph".into());
        b.lines = vec!["same".into(), "same".into()];
        let walker = Walker {
            labels: &["A", "B", "C"],
            filtered: false,
            full: false,
        };
        let (output, differs) = walker
            .hunk(
                &[Some(&a), Some(&b), None],
                &[true; 3],
                &["paragraph".into()],
            )
            .unwrap();
        assert!(differs);
        assert!(output.contains("(absent) [C]"));
        assert!(output.contains("B×2"));
        assert!(
            walker
                .hunk(&[Some(&a), Some(&a)], &[true; 2], &[])
                .is_none()
        );
        a.lines.clear();
        a.block = true;
        assert!(walker.hunk(&[Some(&a), None], &[true; 2], &[]).is_none());
        let full = Walker {
            labels: &["A", "B"],
            filtered: false,
            full: true,
        };
        b.lines = vec!["same".into()];
        assert!(!full.hunk(&[Some(&b), Some(&b)], &[true; 2], &[]).unwrap().1);
        let filtered = Walker {
            labels: &["A", "B"],
            filtered: true,
            full: false,
        };
        let mut hunks = Vec::new();
        filtered.walk(
            &[Some(&b), None],
            &[true; 2],
            &mut Vec::new(),
            false,
            &mut hunks,
        );
        assert!(hunks.is_empty());
        b.hit = true;
        filtered.walk(
            &[Some(&b), None],
            &[true; 2],
            &mut Vec::new(),
            false,
            &mut hunks,
        );
        assert_eq!(hunks.len(), 1);
    }

    #[test]
    fn large_greedy_alignment_preserves_every_source_position_and_only_matches_equal_keys() {
        let a = (0..2003)
            .map(|i| if i % 2 == 0 { "left" } else { "shared" })
            .collect::<Vec<_>>();
        let b = (0..2002)
            .map(|i| if i % 2 == 0 { "shared" } else { "right" })
            .collect::<Vec<_>>();
        let operations = lcs_ops(&a, &b);
        assert_eq!(
            operations
                .iter()
                .filter_map(|(i, _)| *i)
                .collect::<Vec<_>>(),
            (0..a.len()).collect::<Vec<_>>()
        );
        assert_eq!(
            operations
                .iter()
                .filter_map(|(_, j)| *j)
                .collect::<Vec<_>>(),
            (0..b.len()).collect::<Vec<_>>()
        );
        let matched = operations
            .iter()
            .filter_map(|(i, j)| Some((*i.as_ref()?, *j.as_ref()?)))
            .collect::<Vec<_>>();
        assert_eq!(matched.len(), 1001);
        assert!(matched.iter().all(|&(i, j)| a[i] == b[j]));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod raw_source_projection_contract_tests {
    use super::*;

    fn with_source(source: &str, raw: bool, check: impl FnOnce(&mut Build<'_>, NodeId)) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{W_NS}"><w:body>{source}</w:body></w:document>"#
        ));
        let before = dom.serialize_document(doc);
        let root = dom.root(doc).unwrap();
        let body = dom.elements(root, None)[0];
        let node = dom.elements(body, None)[0];
        let filter = Filter {
            style: None,
            para: None,
        };
        let names = HashMap::new();
        let targets = HashMap::new();
        let mut build = Build {
            dom: &dom,
            raw,
            filter: &filter,
            style_names: &names,
            targets: &targets,
            paras: 0,
            tables: 0,
        };
        check(&mut build, node);
        assert_eq!(dom.serialize_document(doc), before);
    }

    #[test]
    fn recorded_run_properties_and_saved_page_breaks_keep_exact_raw_source_projection() {
        let source = r#"<w:p><w:r><w:rPr><w:rPrChange w:id="51" w:author="Owner" w:date="2026-10-09T00:00:00Z"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>ação</w:t><w:lastRenderedPageBreak/><w:t>τέλος</w:t></w:r></w:p>"#;
        for raw in [false, true] {
            with_source(source, raw, |build, node| {
                let mut segments = Vec::new();
                let mut boxes = Vec::new();
                let run = build.dom.elements(node, Some(&XName::get("r", W_NS)))[0];
                build.run(run, "del", &mut segments, &mut boxes);
                let actual = segments
                    .into_iter()
                    .map(|s| (s.rev, s.fmt, s.text, s.text_run))
                    .collect::<Vec<_>>();
                let text = if raw {
                    "ação[lastRenderedPageBreak]τέλος"
                } else {
                    "açãoτέλος"
                };
                assert_eq!(
                    actual,
                    vec![("del".into(), "was{i}".into(), text.into(), true)]
                );
                assert!(boxes.is_empty());
            });
        }
    }

    #[test]
    fn inline_content_controls_preserve_each_source_run_format_and_range_marker() {
        let source = r#"<w:p><w:pPr><w:spacing w:after="80"/></w:pPr><w:bookmarkStart w:id="7" w:name="owned"/><w:sdt><w:sdtPr><w:alias w:val="Review"/><w:tag w:val="owner"/></w:sdtPr><w:sdtContent><w:r><w:rPr><w:b/></w:rPr><w:t>ação</w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>τέλος</w:t></w:r></w:sdtContent></w:sdt><w:bookmarkEnd w:id="7"/></w:p>"#;
        for raw in [false, true] {
            with_source(source, raw, |build, node| {
                let mut segments = Vec::new();
                let mut boxes = Vec::new();
                build.segments(node, "", &mut segments, &mut boxes);
                let actual = segments
                    .into_iter()
                    .map(|s| (s.rev, s.fmt, s.text, s.text_run))
                    .collect::<Vec<_>>();
                let mut expected = vec![
                    ("".into(), "".into(), "⟨bookmark owned⟩".into(), false),
                    ("".into(), "b".into(), "ação".into(), true),
                    ("".into(), "i".into(), "τέλος".into(), true),
                ];
                if raw {
                    expected.push(("".into(), "".into(), "⟨bookmarkEnd(id=7)⟩".into(), false));
                }
                assert_eq!(actual, expected);
                assert!(boxes.is_empty());
            });
        }
    }

    #[test]
    fn large_reordered_key_alignment_discards_only_positions_already_crossed() {
        let mut a = vec!["x", "y", "x"];
        let mut b = vec!["y", "x", "x"];
        a.extend(std::iter::repeat_n("left", 2001));
        b.extend(std::iter::repeat_n("right", 2001));
        let mut expected = vec![
            (None, Some(0)),
            (Some(0), Some(1)),
            (Some(1), None),
            (Some(2), Some(2)),
        ];
        expected.extend((3..a.len()).map(|i| (Some(i), None)));
        expected.extend((3..b.len()).map(|j| (None, Some(j))));
        assert_eq!(lcs_ops(&a, &b), expected);
    }
}
