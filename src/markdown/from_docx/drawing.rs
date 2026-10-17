// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Drawings, in the three readable forms the view gives them. Every one
//! keeps a `drawing:` payload — URL-safe base64 of a JSON envelope holding
//! the element's XML, its relationships, and every package part they
//! reach, recursively — as a reference definition or a backing comment
//! at the view's end, so the body text stays readable past it.
//!
//! - A drawing the readable forms cannot say — a shape, text box,
//!   connector, chart, VML picture, AlternateContent — is `![label][dN]`
//!   where an image would go, `dN` its number in reading order; a
//!   reference definition holds the payload: `[dN]: drawing:PAYLOAD`.
//! - A raster (PNG, JPEG, GIF, BMP, TIFF) is `![alt][imageK]` in the
//!   Google Docs form: the `data:` definition `[imageK]: <data:mime;
//!   base64,…>` holds the bytes, `imageK` numbered by first use and one
//!   definition shared by every use of the same payload. A backing
//!   comment `<!-- imageK drawing:PAYLOAD -->` rides with it only when
//!   the payload is not the `wp:inline` the picture writer regenerates
//!   from bytes, alt text and size alone; then the definition is all the
//!   round trip needs. A use of a shared payload whose own drawing
//!   differs from the first keeps its `dN` definition instead.
//! - An `asvg:svgBlip` a paragraph holds alone is a ` ```svg ` fence with
//!   its source; `<!-- dN drawing:PAYLOAD -->` backs it, PNG fallback
//!   inside. Anywhere else an SVG is a `data:` raster like the others.
//!
//! `label` is the `wp:docPr` description, title or name (empty is fine);
//! a raster's `alt` stops at `title` — the `name` Word gives a picture
//! is the selection pane's, not the alt text. The writer splices the
//! payload's XML back and gives each stored part and relationship the id
//! the new package assigns it, so the whole drawing is lossless even
//! where the Markdown shows only its readable form.
//!
//! A plain inline raster collected under `--media-dir` still extracts to
//! a file; everything else the payload needs travels inside the view, so
//! `jubarte read` — which extracts no media — keeps the drawing whole.
//!
//! Reading is two-phase: the walk queues a [`Pending`] per element and
//! prints the `![label][dN]` reference; once the package can be borrowed
//! again, [`fill`] captures each element, picks its form and appends the
//! definitions.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use super::ooxml::{Element, Node, Package, Rels, local, parse_xml};
use crate::opc::{PartFs, relative_rel_target};

/// The relationship namespaces (`r`), transitional and Strict: an attribute
/// prefixed by a prefix bound to one holds a relationship id of the part the
/// element sits in.
const REL_NS: [&str; 2] = [
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    "http://purl.oclc.org/ooxml/officeDocument/relationships",
];

/// The most dependent parts a `drawing:` payload holds.
const MAX_DEPS: usize = 64;
/// The most relationships one part of a payload holds.
const MAX_DEP_RELS: usize = 64;
/// The most bytes one dependent part holds.
const MAX_DEP_BYTES: u64 = 16 * 1024 * 1024;
/// The most `drawing:` payloads one document queues.
const MAX_PENDING: usize = 4096;

/// What `![label](drawing:PAYLOAD)` decodes to: the element's XML with its
/// relationship ids symbolized (`r1`, `d3.2` — part 3's second relationship),
/// and every part they resolve to.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Drawing {
    /// The captured element, verbatim but for symbolized relationship ids,
    /// `wp:docPr` ids normalized to `0`, and the `xmlns:*` declarations
    /// injected on its root.
    xml: String,
    /// Splice point: the element is run content (`w:drawing`, `w:pict`,
    /// `w:object`, run-level AlternateContent) and goes inside a `w:r`
    /// (`true`), or it sits where `w:r` may (`false`).
    #[serde(default)]
    wrap: bool,
    /// Relationships of the part the element was read from.
    #[serde(default)]
    rels: Vec<PayRel>,
    /// Package parts the relationships reach, in capture order.
    #[serde(default)]
    deps: Vec<Dep>,
    /// The `deps` index whose bytes the readable layer carries: a raster's
    /// `data:` definition or an `svg` fence's source. Its `b64` is empty in
    /// a backing payload; the writer fills it from the readable form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    slot: Option<usize>,
    /// SHA-256 of the ` ```mermaid ` fence's text the view printed for a
    /// chart, hex. An unchanged fence's hash matches and the drawing
    /// writes back byte for byte; an edited one rebuilds its chart part.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mhash: Option<String>,
    /// The Mermaid text `mhash` hashes, for the fence `fill` prints.
    /// Never in the payload: the fence is the text's home.
    #[serde(skip)]
    mermaid: Option<String>,
}

/// One relationship of the element's part or of a dependent part.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct PayRel {
    /// The symbol `xml` names it by (`r1`, `d0.2`).
    sym: String,
    /// The relationship type, verbatim.
    #[serde(rename = "t")]
    kind: String,
    /// External target (`TargetMode="External"`), e.g. an `r:link` URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    u: Option<String>,
    /// Internal target: an index into [`Drawing::deps`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    part: Option<usize>,
    /// Internal target's resolved part name kept raw: the part does not
    /// exist, so it travels as a relationship target string only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    raw: Option<String>,
}

/// One package part a drawing's relationships reach.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Dep {
    /// The part name it had, used again when it is free.
    name: String,
    /// Its content type (an override is written for it, always).
    #[serde(rename = "t", default)]
    kind: String,
    /// Raw bytes, base64, when nothing in the part needs a relationship id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    b64: String,
    /// The part's XML when it names relationships of its own: serialized
    /// with them symbolized (`d{this}.{n}`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    xml: String,
    /// The part's own relationships, in the order `xml` names them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    rels: Vec<PayRel>,
}

/// `xmlns` bindings in scope at an element: `(prefix, uri)` where `""` is
/// the default namespace.
type Scope = Vec<(String, String)>;

/// `xmlns` scopes for the drawings of one part's element tree, by element
/// address. Elements not in the map — clones, like a comment's `w:comment` —
/// take the part's root scope, where the common prefixes are declared.
#[derive(Clone)]
pub(crate) struct ScopeMap {
    root: Scope,
    by_ptr: HashMap<*const Element, Scope>,
}

impl ScopeMap {
    /// The scope map of a part's tree. `drawing`, `pict`, `object` and
    /// `AlternateContent` elements may be captured, so each gets an entry.
    pub(crate) fn of(root: &Element) -> Self {
        let root_scope = scope_of(root);
        let mut by_ptr = HashMap::new();
        fn walk(element: &Element, scope: &Scope, map: &mut HashMap<*const Element, Scope>) {
            if matches!(
                element.local(),
                "drawing" | "pict" | "object" | "AlternateContent"
            ) {
                map.insert(element as *const Element, scope.clone());
            }
            let own = merged(scope, &scope_of(element));
            for child in element.elements() {
                walk(child, &own, map);
            }
        }
        for child in root.elements() {
            walk(child, &root_scope, &mut by_ptr);
        }
        Self {
            root: root_scope,
            by_ptr,
        }
    }

    /// The scope frozen at `element`: its ancestors' declarations, then its
    /// own.
    pub(crate) fn scope(&self, element: &Element) -> Scope {
        merged(
            self.by_ptr
                .get(&(element as *const Element))
                .unwrap_or(&self.root),
            &scope_of(element),
        )
    }
}

/// A drawing queued while a story was read: its element cloned, where it
/// sits, and the scope frozen where it was found.
pub(crate) struct Pending {
    element: Element,
    /// The part whose relationships it names.
    part: String,
    scope: Scope,
    /// Inside a `w:r` (see [`Drawing::wrap`]).
    wrap: bool,
}

impl Pending {
    /// Queue `element`, printing its `![label][dN]` reference. `label` is
    /// the drawing's `wp:docPr` description, title or name.
    pub(crate) fn push(
        list: &mut Vec<Pending>,
        element: &Element,
        part: &str,
        scope: Scope,
        wrap: bool,
    ) -> String {
        let label = label(element);
        if list.len() >= MAX_PENDING {
            return image(&label, "");
        }
        let index = list.len();
        list.push(Pending {
            element: element.clone(),
            part: part.to_string(),
            scope,
            wrap,
        });
        image(&label, &format!("d{}", index + 1))
    }
}

/// `![label][name]` with the label made a Markdown alt text: one line,
/// brackets escaped. An empty `name` is `![label]()` — an empty image the
/// writer reads as alt text.
fn image(label: &str, name: &str) -> String {
    if name.is_empty() {
        format!("![{label}]()")
    } else {
        format!("![{label}][{name}]")
    }
}

/// The `wp:docPr` description, title or name as the token's label.
fn label(element: &Element) -> String {
    let alt = alt(element);
    if !alt.is_empty() {
        return alt;
    }
    element
        .find("docPr")
        .and_then(|doc_pr| doc_pr.attr("name"))
        .map(alt_text)
        .unwrap_or_default()
}

/// A raster's alt text: the `wp:docPr` description or title. The `name`
/// Word gives a picture (`Picture 1`) is the selection pane's, not alt
/// text — printing it would invent one.
fn alt(element: &Element) -> String {
    let Some(doc_pr) = element.find("docPr") else {
        return String::new();
    };
    alt_text(
        doc_pr
            .attr("descr")
            .filter(|d| !d.trim().is_empty())
            .or_else(|| doc_pr.attr("title"))
            .unwrap_or(""),
    )
}

/// `text` as a Markdown alt text: one line, brackets escaped.
fn alt_text(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

/// The `xmlns`/`xmlns:p` attributes `element` declares, `(prefix, uri)`.
pub(crate) fn scope_of(element: &Element) -> Scope {
    element
        .attrs
        .iter()
        .filter_map(|(name, uri)| {
            name.strip_prefix("xmlns").map(|prefix| {
                (
                    prefix.strip_prefix(':').unwrap_or(prefix).to_string(),
                    uri.clone(),
                )
            })
        })
        .collect()
}

/// `later` wins over `scope` on the same prefix.
fn merge(scope: &mut Scope, later: &Scope) {
    for (prefix, uri) in later {
        match scope.iter_mut().find(|(p, _)| p == prefix) {
            Some((_, u)) => *u = uri.clone(),
            None => scope.push((prefix.clone(), uri.clone())),
        }
    }
}

fn merged(scope: &Scope, later: &Scope) -> Scope {
    let mut out = scope.clone();
    merge(&mut out, later);
    out
}

/// The union of every `xmlns` scope in the tree: a dependent part is read
/// whole, so its own declarations are enough.
fn tree_scope(root: &Element) -> Scope {
    let mut scope = Vec::new();
    fn collect(element: &Element, scope: &mut Scope) {
        merge(scope, &scope_of(element));
        for child in element.elements() {
            collect(child, scope);
        }
    }
    collect(root, &mut scope);
    scope
}

/// `name` is `prefix:local` whose prefix binds to `REL_NS`.
fn is_rel_name(name: &str, scope: &Scope) -> bool {
    let Some((prefix, _)) = name.split_once(':') else {
        return false;
    };
    scope
        .iter()
        .any(|(p, uri)| p == prefix && REL_NS.contains(&uri.as_str()))
}

/// Declare on `root` every prefix the tree uses that no element inside it
/// declares (and every prefix `mc:Requires` names), so the spliced element
/// is self-contained.
fn declare(root: &mut Element, scope: &Scope) {
    let mut declared: HashSet<String> = HashSet::new();
    let mut used: HashSet<String> = HashSet::new();
    fn visit(element: &Element, declared: &mut HashSet<String>, used: &mut HashSet<String>) {
        if let Some((prefix, _)) = element.name.split_once(':') {
            used.insert(prefix.to_string());
        }
        for (name, value) in &element.attrs {
            if name == "xmlns" {
                declared.insert(String::new());
            } else if let Some(prefix) = name.strip_prefix("xmlns:") {
                declared.insert(prefix.to_string());
            } else if let Some((prefix, _)) = name.split_once(':') {
                used.insert(prefix.to_string());
            }
            if name
                .split_once(':')
                .is_some_and(|(p, l)| p == "mc" && l == "Requires")
            {
                used.extend(value.split_whitespace().map(str::to_string));
            }
        }
        for child in element.elements() {
            visit(child, declared, used);
        }
    }
    visit(root, &mut declared, &mut used);
    // Sorted, so the payload reads the same every time.
    let mut used: Vec<String> = used.into_iter().collect();
    used.sort();
    for prefix in used {
        if declared.contains(&prefix) {
            continue;
        }
        if let Some((_, uri)) = scope.iter().find(|(p, _)| *p == prefix) {
            root.attrs.push((format!("xmlns:{prefix}"), uri.clone()));
        }
    }
}

/// `element` as XML: self-closing when childless, attributes and text
/// escaped. `Element` keeps its names' prefixes, so this is verbatim where
/// the source was.
pub(crate) fn serialize(element: &Element) -> String {
    let mut out = String::new();
    fn put(element: &Element, out: &mut String) {
        let _ = write!(out, "<{}", element.name);
        for (name, value) in &element.attrs {
            let _ = write!(out, " {name}=\"{}\"", crate::markdown::xml::escape(value));
        }
        if element.children.is_empty() {
            out.push_str("/>");
            return;
        }
        out.push('>');
        for child in &element.children {
            match child {
                Node::Element(inner) => put(inner, out),
                Node::Text(text) => out.push_str(&crate::markdown::xml::escape(text)),
            }
        }
        let _ = write!(out, "</{}>", element.name);
    }
    put(element, &mut out);
    out
}

/// `wp:docPr` drawing ids are the writer's to assign: capture normalizes
/// them to `0` so the payload does not change when the written file gives
/// them fresh numbers.
fn normalize_ids(element: &mut Element) {
    if local(&element.name) == "docPr" {
        for (name, value) in &mut element.attrs {
            if name == "id" {
                *value = "0".to_string();
            }
        }
    }
    for child in &mut element.children {
        if let Node::Element(inner) = child {
            normalize_ids(inner);
        }
    }
}

/// Scratch for the capture in flight: the parts collected and why it
/// stopped.
struct Deps<'a> {
    types: &'a HashMap<String, String>,
    parts: Vec<Dep>,
    index: HashMap<String, usize>,
    failed: Option<String>,
}

impl Deps<'_> {
    /// The index of `target`'s part in `parts`, reading and capturing it on
    /// first use; `None` when the part is absent.
    fn dep_for(&mut self, package: &mut Package<'_>, target: &str) -> Option<usize> {
        if let Some(&index) = self.index.get(target) {
            return Some(index);
        }
        if self.parts.len() >= MAX_DEPS {
            self.failed = Some(format!("more than {MAX_DEPS} dependent parts"));
            return None;
        }
        let bytes = match package.read(target) {
            Ok(Some(bytes)) if bytes.len() as u64 <= MAX_DEP_BYTES => bytes,
            Ok(Some(_)) => {
                self.failed = Some(format!("{target} exceeds {MAX_DEP_BYTES} bytes"));
                return None;
            }
            _ => return None,
        };
        let index = self.parts.len();
        self.parts.push(Dep::default());
        self.index.insert(target.to_string(), index);
        let rels = package.rels(target).unwrap_or_default();
        let mut pay_rels = Vec::new();
        let mut xml = String::new();
        if !rels.is_empty() {
            if rels.len() > MAX_DEP_RELS {
                self.failed = Some(format!(
                    "{target} has more than {MAX_DEP_RELS} relationships"
                ));
            } else if let Ok(mut root) = parse_xml(&bytes) {
                let scope = tree_scope(&root);
                let sym = format!("d{index}.");
                symbolize(package, self, &mut root, &scope, &rels, &sym, &mut pay_rels);
                normalize_ids(&mut root);
                declare(&mut root, &scope);
                xml = serialize(&root);
            } else {
                self.failed = Some(format!("{target} is not readable XML"));
            }
        }
        let kind = self
            .types
            .get(&format!("/{target}"))
            .cloned()
            .or_else(|| {
                target
                    .rsplit('.')
                    .next()
                    .filter(|ext| *ext != target)
                    .and_then(|ext| self.types.get(&format!(".{ext}")).cloned())
            })
            .unwrap_or_else(|| "application/octet-stream".to_string());
        let dep = &mut self.parts[index];
        dep.name = target.to_string();
        dep.kind = kind;
        if xml.is_empty() {
            dep.b64 = base64_encode(&bytes);
        } else {
            dep.xml = xml;
        }
        dep.rels = pay_rels;
        Some(index)
    }
}

/// Symbolize every relationship id `element` and its descendants name:
/// attributes whose prefix binds to `REL_NS` in `scope`. Their values
/// become `sym{n}` on the element and `PayRel`s in `out`.
fn symbolize(
    package: &mut Package<'_>,
    deps: &mut Deps<'_>,
    element: &mut Element,
    scope: &Scope,
    rels: &Rels,
    sym: &str,
    out: &mut Vec<PayRel>,
) {
    for (name, value) in &mut element.attrs {
        if !is_rel_name(name, scope) {
            continue;
        }
        let Some(rel) = rels.get(value.as_str()) else {
            deps.failed
                .get_or_insert_with(|| format!("{name}=\"{value}\" names no relationship"));
            continue;
        };
        let n = out.len() + 1;
        let mut pay = PayRel {
            sym: format!("{sym}{n}"),
            kind: rel.kind.clone(),
            ..PayRel::default()
        };
        if rel.external {
            pay.u = Some(rel.target.clone());
        } else if let Some(index) = deps.dep_for(package, &rel.target) {
            pay.part = Some(index);
        } else {
            pay.raw = Some(rel.target.clone());
        }
        *value = pay.sym.clone();
        out.push(pay);
    }
    let own = merged(scope, &scope_of(element));
    for child in &mut element.children {
        if let Node::Element(inner) = child {
            symbolize(package, deps, inner, &own, rels, sym, out);
        }
    }
}

/// Capture `pending`: its element verbatim but for the relationship ids it
/// symbolizes, with every part they reach. `None` when something failed.
pub(crate) fn resolve(
    package: &mut Package<'_>,
    types: &HashMap<String, String>,
    pending: &Pending,
) -> Option<Drawing> {
    let rels = package.rels(&pending.part).ok()?;
    let mut deps = Deps {
        types,
        parts: Vec::new(),
        index: HashMap::new(),
        failed: None,
    };
    let mut root = pending.element.clone();
    let mut out = Vec::new();
    symbolize(
        package,
        &mut deps,
        &mut root,
        &pending.scope,
        &rels,
        "r",
        &mut out,
    );
    if deps.failed.is_some() {
        return None;
    }
    let slot = media_slot(&root, &out, &deps);
    let mermaid = chart_mermaid(&deps);
    let mhash = mermaid.as_deref().map(mermaid_hash);
    normalize_ids(&mut root);
    declare(&mut root, &pending.scope);
    Some(Drawing {
        xml: serialize(&root),
        wrap: pending.wrap,
        rels: out,
        deps: deps.parts,
        slot,
        mhash,
        mermaid,
    })
}

/// The chart part's Mermaid text when the payload holds a `chartSpace`
/// `pie`/`xychart-beta` can say: a standalone drawing prints it as a
/// ` ```mermaid ` fence and `mhash` says whether the text came back
/// unchanged. Charts that hold relationships keep their fences too —
/// [`Drawing::rebuild_mermaid`] patches their `xml` in place.
fn chart_mermaid(deps: &Deps<'_>) -> Option<String> {
    let dep = deps
        .parts
        .iter()
        .find(|dep| dep.kind.ends_with("chart+xml"))?;
    let xml = if dep.xml.is_empty() {
        String::from_utf8(base64_decode(&dep.b64)?).ok()?
    } else {
        dep.xml.clone()
    };
    super::mermaid::chart(&xml)
}

/// SHA-256 of `text`, hex: the hash `mhash` carries for a fence's text.
fn mermaid_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(text.trim().as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The image part the readable layer carries for `root`, as a `deps` index:
/// an `asvg:svgBlip`'s SVG source, else the `a:blip`'s PNG, JPEG, GIF, BMP
/// or TIFF raster. `None` when the element is a picture of nothing the
/// readable forms name (EMF, WMF, an external or missing target) or holds
/// content the payload must keep — a text box's story, a chart's frame.
fn media_slot(root: &Element, rels: &[PayRel], deps: &Deps<'_>) -> Option<usize> {
    if root.find("txbxContent").is_some() || root.find("graphicFrame").is_some() {
        return None;
    }
    // The dependency a relationship symbol `rN` names.
    let dep_of = |element: &Element| {
        element.attrs.iter().find_map(|(name, sym)| {
            if !name.contains(':') {
                return None;
            }
            let rel = rels.iter().find(|rel| rel.sym == *sym)?;
            let index = match rel.part {
                Some(index) => index,
                None => *deps.index.get(rel.raw.as_deref()?)?,
            };
            (index < deps.parts.len()).then_some(index)
        })
    };
    // The SVG form names its source, not the PNG fallback `a:blip` beside it.
    if let Some(blip) = root.find("svgBlip")
        && let Some(index) = dep_of(blip)
        && deps.parts[index].kind.starts_with("image/")
    {
        return Some(index);
    }
    const RASTER: [&str; 5] = [
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/bmp",
        "image/tiff",
    ];
    if let Some(blip) = root.find("blip")
        && let Some(index) = dep_of(blip)
        && RASTER.contains(&deps.parts[index].kind.as_str())
    {
        return Some(index);
    }
    None
}

/// The `[Content_Types]` entries of `package`: `/part` for each override,
/// `.ext` for each default.
pub(crate) fn content_types(package: &mut Package<'_>) -> HashMap<String, String> {
    let mut types = HashMap::new();
    if let Ok(Some(root)) = package.xml("[Content_Types].xml") {
        for child in root.elements() {
            match child.local() {
                "Override" => {
                    if let (Some(name), Some(kind)) =
                        (child.attr("PartName"), child.attr("ContentType"))
                    {
                        types.insert(name.to_string(), kind.to_string());
                    }
                }
                "Default" => {
                    if let (Some(ext), Some(kind)) =
                        (child.attr("Extension"), child.attr("ContentType"))
                    {
                        types.insert(format!(".{ext}"), kind.to_string());
                    }
                }
                _ => {}
            }
        }
    }
    types
}

/// `markdown` with its `![label][dN]` references given their readable
/// form and their payloads appended at the view's end. A raster the
/// layer reads as bytes is `![alt][imageK]` with a `data:` definition
/// shared by every use of the payload (`alt` the description or title —
/// Word's picture `name` is not alt text); a drawing that is not the
/// `wp:inline` the picture writer rebuilds gets a `drawing:` backing
/// comment, and a use of a shared payload whose drawing differs keeps
/// its `dN` definition. An SVG the paragraph holds alone is a ` ```svg `
/// fence backed by a comment; every other drawing keeps the `dN`
/// reference to a `drawing:` definition. A pending that failed to
/// capture loses its reference: `![label]()` stays readable and the
/// writer writes it as alt text.
pub(crate) fn fill(
    markdown: &str,
    package: &mut Package<'_>,
    types: &HashMap<String, String>,
    pendings: &[Pending],
) -> String {
    if pendings.is_empty() {
        return markdown.to_string();
    }
    let mut out = markdown.to_string();
    // A token's place is found in the whole text — a header's `dN` names
    // its line in the YAML summary, printed before the body's pendings —
    // except inside the spans a fence just took: `][dN]` there is source.
    let mut fences: Vec<(usize, usize)> = Vec::new();
    // `token`'s first occurrence outside every `fences` span.
    let find_token = |out: &str, token: &str, fences: &[(usize, usize)]| -> Option<usize> {
        let mut from = 0;
        while let Some(pos) = out[from..].find(token) {
            let pos = from + pos;
            if fences.iter().all(|&(s, e)| pos < s || pos >= e) {
                return Some(pos);
            }
            from = pos + 1;
        }
        None
    };
    let mut appendix = String::new();
    let mut media = 0usize;
    // Media payload (`mime`, base64) → (`imageK` name, first drawing's
    // canonical form): later uses of the same bytes share the one
    // definition while their drawing is the same.
    let mut names: HashMap<(String, String), (String, String)> = HashMap::new();
    for (index, pending) in pendings.iter().enumerate() {
        let d = format!("d{}", index + 1);
        let token = format!("][{d}]");
        let Some(drawing) = resolve(package, types, pending) else {
            // `![label][dN]` is the one shape push prints; only a capture
            // failure turns it into `![label]()`.
            if let Some(pos) = find_token(&out, &token, &fences) {
                out.replace_range(pos..pos + token.len(), "]()");
            }
            continue;
        };
        let Some(pos) = find_token(&out, &token, &fences) else {
            continue;
        };
        let end = pos + token.len();
        // A chart its paragraph holds alone prints its Mermaid: a
        // ` ```mermaid ` fence the payload comment backs. Anywhere else —
        // run content — the fence could not sit, so it keeps `[dN]`.
        if let Some(text) = drawing.mermaid.clone()
            && let Some((start, text_end)) = standalone(&out, pos, end)
        {
            let label = label(&pending.element);
            let info = if label.is_empty() {
                format!("mermaid {d}")
            } else {
                format!("mermaid {d} {label}")
            };
            let fence = fenced(&info, &text);
            out.replace_range(start..text_end, &fence);
            fences.push((start, start + fence.len()));
            let _ = writeln!(appendix, "<!-- {d} drawing:{} -->", encode(&drawing));
            continue;
        }
        let Some(slot) = drawing.slot else {
            let _ = writeln!(appendix, "[{d}]: drawing:{}", encode(&drawing));
            continue;
        };
        let dep = &drawing.deps[slot];
        let is_svg = dep.kind == "image/svg+xml" && !dep.b64.is_empty();
        if is_svg
            && let Some((start, text_end)) = standalone(&out, pos, end)
            && let Some(svg) = base64_decode(&dep.b64)
            && let Ok(svg) = str::from_utf8(&svg)
        {
            let label = label(&pending.element);
            let info = if label.is_empty() {
                format!("svg {d}")
            } else {
                format!("svg {d} {label}")
            };
            let fence = fenced(&info, svg.trim());
            out.replace_range(start..text_end, &fence);
            fences.push((start, start + fence.len()));
            let _ = writeln!(
                appendix,
                "<!-- {d} drawing:{} -->",
                encode(&drawing.without_slot_bytes())
            );
            continue;
        }
        // A raster, or an SVG run content cannot leave a fence: its
        // `data:` definition — unless the media bytes never read, a
        // `dN` definition like every other payload.
        if dep.b64.is_empty() {
            let _ = writeln!(appendix, "[{d}]: drawing:{}", encode(&drawing));
            continue;
        }
        let key = (dep.kind.clone(), dep.b64.clone());
        let shared = match names.get(&key) {
            Some((name, seen)) if *seen == canonical(&drawing) => Some(name.clone()),
            Some(_) => None,
            None => {
                media += 1;
                let name = format!("image{media}");
                names.insert(key, (name.clone(), canonical(&drawing)));
                let _ = writeln!(appendix, "[{name}]: <data:{};base64,{}>", dep.kind, dep.b64);
                if !regenerable(&drawing) {
                    let _ = writeln!(
                        appendix,
                        "<!-- {name} drawing:{} -->",
                        encode(&drawing.without_slot_bytes())
                    );
                }
                Some(name)
            }
        };
        match shared {
            // A use of a shared payload whose own drawing differs
            // keeps its `dN` definition: the `imageK` name's one
            // backing cannot be both drawings.
            None => {
                let _ = writeln!(appendix, "[{d}]: drawing:{}", encode(&drawing));
            }
            Some(name) => {
                let shift = relabel(&mut out, pos, &alt(&pending.element));
                let pos = pos.wrapping_add_signed(shift);
                out.replace_range(pos + 2..pos + token.len() - 1, &name);
            }
        }
    }
    if !appendix.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        let _ = write!(out, "\n{appendix}");
    }
    out
}

/// Rewrite the label of the `![label][dN]` token ending at `pos`: the
/// printable alt text a `docPr` name would have invented. The byte
/// shift the token's end moved by.
fn relabel(out: &mut String, pos: usize, label: &str) -> isize {
    let Some(start) = out[..pos].rfind("![") else {
        return 0;
    };
    let Some((name, _)) = at(&out[start..]) else {
        return 0;
    };
    if !(name.starts_with('d') || name.starts_with("image")) {
        return 0;
    }
    let old = pos - (start + 2);
    out.replace_range(start + 2..pos, label);
    label.len() as isize - old as isize
}

/// The payload minus what use of it may differ in — `docPr` and
/// `cNvPr` alt text and generated names, which the readable layer
/// carries or the writer renumbers — so two drawings of one media
/// payload share its `imageK` definition only while they splice the
/// same XML.
fn canonical(drawing: &Drawing) -> String {
    let mut share = drawing.clone();
    let Ok(mut root) = parse_xml(drawing.xml.as_bytes()) else {
        return drawing.xml.clone();
    };
    fn names(element: &mut Element) {
        if matches!(local(&element.name), "docPr" | "cNvPr") {
            for (name, value) in &mut element.attrs {
                let generated = name == "name" && generated_name(value);
                if matches!(name.as_str(), "descr" | "title") || generated {
                    value.clear();
                }
            }
        }
        for child in &mut element.children {
            if let Node::Element(inner) = child {
                names(inner);
            }
        }
    }
    names(&mut root);
    share.xml = serialize(&root);
    for (index, dep) in share.deps.iter_mut().enumerate() {
        if drawing.slot == Some(index) {
            dep.b64.clear();
            dep.xml.clear();
        } else {
            *dep = Dep::default();
        }
    }
    for rel in &mut share.rels {
        if rel.part.is_some() && rel.part != drawing.slot {
            rel.part = Some(usize::MAX);
        }
    }
    encode(&share)
}

/// A `docPr`/`cNvPr` name the writer gives the picture it numbers
/// (`Picture 3`): no content to keep.
fn generated_name(name: &str) -> bool {
    name.strip_prefix("Picture ")
        .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether the picture writer rebuilds `drawing` from its `data:`
/// bytes, its alt text and the raster's size alone: a `wp:inline` of
/// exactly the shape `drawing_xml` writes, at the size `read_picture`
/// computes. Then the `data:` definition is all the round trip needs
/// and the view carries no `drawing:` backing comment.
fn regenerable(drawing: &Drawing) -> bool {
    let Some(slot) = drawing.slot else {
        return false;
    };
    if !drawing.wrap || drawing.rels.len() != 1 || drawing.rels[0].part != Some(slot) {
        return false;
    }
    let Ok(root) = parse_xml(drawing.xml.as_bytes()) else {
        return false;
    };
    if !root.is("drawing") {
        return false;
    }
    let kids: Vec<&Element> = root.elements().collect();
    let [inline] = kids.as_slice() else {
        return false;
    };
    if !inline.is("inline")
        || !attrs_ok(inline, |name, value| {
            matches!(local(name), "distT" | "distB" | "distL" | "distR") && value == "0"
        })
    {
        return false;
    }
    let kids: Vec<&Element> = inline.elements().collect();
    let [extent, effect, doc_pr, frame, graphic] = kids.as_slice() else {
        return false;
    };
    if !extent.is("extent")
        || extent.attr("cx").is_none()
        || extent.attr("cy").is_none()
        || !attrs_ok(extent, |name, _| matches!(local(name), "cx" | "cy"))
    {
        return false;
    }
    if !effect.is("effectExtent")
        || !attrs_ok(effect, |name, value| {
            matches!(local(name), "l" | "t" | "r" | "b") && value == "0"
        })
    {
        return false;
    }
    if !doc_pr.is("docPr")
        || !attrs_ok(doc_pr, |name, value| {
            (name == "id" && value == "0")
                || (name == "name" && generated_name(value))
                || matches!(name, "descr" | "title")
        })
        || doc_pr.elements().next().is_some()
    {
        return false;
    }
    if !frame.is("cNvGraphicFramePr") {
        return false;
    }
    let Some(locks) = frame.child("graphicFrameLocks") else {
        return false;
    };
    if !attrs_ok(locks, |name, value| {
        name == "noChangeAspect" && value == "1"
    }) || locks.elements().next().is_some()
        || frame.elements().nth(1).is_some()
    {
        return false;
    }
    let Some(data) = graphic.child("graphicData") else {
        return false;
    };
    if data.attr("uri") != Some("http://schemas.openxmlformats.org/drawingml/2006/picture")
        || !attrs_ok(data, |name, _| local(name) == "uri")
    {
        return false;
    }
    let Some(pic) = data.child("pic") else {
        return false;
    };
    let kids: Vec<&Element> = pic.elements().collect();
    let [nv_pic_pr, blip_fill, sp_pr] = kids.as_slice() else {
        return false;
    };
    if !nv_pic_pr.is("nvPicPr") || !blip_fill.is("blipFill") || !sp_pr.is("spPr") {
        return false;
    }
    let Some(c_nv_pr) = nv_pic_pr.child("cNvPr") else {
        return false;
    };
    if !attrs_ok(c_nv_pr, |name, value| {
        (name == "id" && value == "0")
            || (name == "name" && generated_name(value))
            || matches!(name, "descr" | "title")
    }) || c_nv_pr.elements().next().is_some()
    {
        return false;
    }
    if !nv_pic_pr
        .elements()
        .nth(1)
        .is_some_and(|e| e.is("cNvPicPr") && e.elements().next().is_none() && e.attrs.is_empty())
        || nv_pic_pr.elements().nth(2).is_some()
    {
        return false;
    }
    let kids: Vec<&Element> = blip_fill.elements().collect();
    let [blip, stretch] = kids.as_slice() else {
        return false;
    };
    if !blip.is("blip")
        || blip.elements().next().is_some()
        || blip.rel_attr("embed").is_none()
        || !attrs_ok(blip, |name, _| name.contains(':') && local(name) == "embed")
    {
        return false;
    }
    let kids: Vec<&Element> = stretch.elements().collect();
    let [fill_rect] = kids.as_slice() else {
        return false;
    };
    if !fill_rect.is("fillRect") || !attrs_ok(stretch, |_, _| false) {
        return false;
    }
    let kids: Vec<&Element> = sp_pr.elements().collect();
    let [xfrm, prst] = kids.as_slice() else {
        return false;
    };
    if !xfrm.is("xfrm") || !attrs_ok(sp_pr, |_, _| false) {
        return false;
    }
    let kids: Vec<&Element> = xfrm.elements().collect();
    let [off, ext] = kids.as_slice() else {
        return false;
    };
    if !off.is("off")
        || off.attr("x") != Some("0")
        || off.attr("y") != Some("0")
        || !attrs_ok(off, |name, _| matches!(local(name), "x" | "y"))
        || !ext.is("ext")
        || !attrs_ok(ext, |name, _| matches!(local(name), "cx" | "cy"))
    {
        return false;
    }
    if !prst.is("prstGeom")
        || prst.attr("prst") != Some("rect")
        || !attrs_ok(prst, |name, _| local(name) == "prst")
    {
        return false;
    }
    let Some(av_lst) = prst.child("avLst") else {
        return false;
    };
    if !av_lst.attrs.is_empty()
        || av_lst.elements().next().is_some()
        || prst.elements().nth(1).is_some()
    {
        return false;
    }
    // The sizes the writer puts on a `wp:inline` of these bytes: the
    // extent and the picture's `a:ext` are the picture's own.
    let Some(bytes) = base64_decode(&drawing.deps[slot].b64) else {
        return false;
    };
    let Some(picture) = crate::markdown::write::read_picture(bytes, "") else {
        return false;
    };
    extent.attr("cx") == Some(picture.width.to_string().as_str())
        && extent.attr("cy") == Some(picture.height.to_string().as_str())
        && ext.attr("cx") == Some(picture.width.to_string().as_str())
        && ext.attr("cy") == Some(picture.height.to_string().as_str())
}

/// `element`'s non-`xmlns` attributes all pass `ok`.
fn attrs_ok(element: &Element, ok: impl Fn(&str, &str) -> bool) -> bool {
    element
        .attrs
        .iter()
        .all(|(name, value)| name.starts_with("xmlns") || ok(name, value))
}

/// The span of a `![label][dN]` token that is a whole line: its `![`
/// opening and its line's end. `None` when anything else shares the line
/// — a run of text, a change mark — where a fence could not sit.
fn standalone(out: &str, pos: usize, end: usize) -> Option<(usize, usize)> {
    let start = out[..pos].rfind("![")?;
    let (name, len) = at(&out[start..])?;
    if !(name.starts_with('d')
        && name.len() > 1
        && name.bytes().skip(1).all(|b| b.is_ascii_digit())
        && start + len == end)
    {
        return None;
    }
    let line_start = out[..start].rfind('\n').map_or(0, |n| n + 1);
    let line_end = out[end..].find('\n').map_or(out.len(), |n| end + n);
    (out[line_start..start].trim().is_empty() && out[end..line_end].trim().is_empty())
        .then_some((start, line_end))
}

/// `text` as a fenced code block carrying `info`, the fence one backtick
/// wider than any run inside.
fn fenced(info: &str, text: &str) -> String {
    let mut widest = 0usize;
    let mut run = 0usize;
    for b in text.bytes() {
        if b == b'`' {
            run += 1;
            widest = widest.max(run);
        } else {
            run = 0;
        }
    }
    let ticks = "`".repeat((widest + 1).max(3));
    format!("{ticks} {info}\n{text}\n{ticks}")
}

/// The Markdown drawing token at `text`'s start, if any —
/// `![label][dN]`, or a `![label](drawing:PAYLOAD)` of a view an older
/// build wrote. What the token resolves by: the `dN` reference name or the
/// `drawing:` destination, and the token's whole length.
pub(crate) fn at(text: &str) -> Option<(&str, usize)> {
    let rest = text.strip_prefix("![")?;
    let label_end = rest.find(']')?;
    let after = &rest[label_end + 1..];
    if let Some(inner) = after.strip_prefix('(') {
        let end = inner.find(')')?;
        let dest = &inner[..end];
        let payload = dest.strip_prefix("drawing:")?;
        return (!payload.is_empty()
            && payload
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'=')))
        .then(|| (dest, 2 + label_end + 2 + dest.len() + 1));
    }
    let name = after
        .strip_prefix('[')
        .and_then(|inner| inner.split_once(']'))
        .map(|(name, _)| name)?;
    let digits = name
        .strip_prefix('d')
        .or_else(|| name.strip_prefix("image"))?;
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())).then(|| {
        let start = 2 + label_end + 2;
        (&text[start..start + name.len()], start + name.len() + 1)
    })
}

/// Whether cutting `text` at `cut` splits a `![…](drawing:…)` token.
pub(crate) fn cuts_token(text: &str, cut: usize) -> bool {
    let Some(open) = text[..cut].rfind("![") else {
        return false;
    };
    at(&text[open..]).is_some_and(|(_, len)| len > cut - open)
}

/// Decode the payload of `![label](drawing:PAYLOAD)`; `dest` is what
/// [`at`] returns (`drawing:…`).
pub(crate) fn decode(dest: &str) -> Result<Drawing, String> {
    let encoded = dest
        .strip_prefix("drawing:")
        .ok_or_else(|| "not a drawing: destination".to_string())?;
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .or_else(|_| {
            base64::engine::general_purpose::URL_SAFE.decode(encoded.trim_end_matches('='))
        })
        .map_err(|e| format!("drawing payload is not base64: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("drawing payload is not its JSON: {e}"))
}

pub(crate) fn encode(drawing: &Drawing) -> String {
    use base64::Engine;
    let json = serde_json::to_vec(drawing).unwrap_or_default();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub(crate) fn base64_decode(encoded: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()
}

/// A `data:` destination's `(mime, base64)` parts: `data:mime;base64,…`.
/// `data:` with no `;base64` marker reads as no media bytes.
pub(crate) fn data_uri(dest: &str) -> Option<(&str, &str)> {
    let rest = dest.strip_prefix("data:")?;
    let (meta, data) = rest.split_once(',')?;
    let mime = meta.strip_suffix(";base64")?;
    (!mime.is_empty() && !data.is_empty()).then_some((mime, data))
}

/// The `<!-- NAME drawing:PAYLOAD -->` backing comments of a view's
/// drawings appendix, `NAME` → the `drawing:` destination.
pub(crate) fn backing_comments(source: &str) -> HashMap<String, String> {
    source
        .lines()
        .filter_map(|line| {
            let inner = line
                .trim()
                .strip_prefix("<!--")?
                .strip_suffix("-->")?
                .trim();
            let (name, dest) = inner.split_once(' ')?;
            (dest.starts_with("drawing:") && !name.is_empty())
                .then(|| (name.to_string(), dest.to_string()))
        })
        .collect()
}

/// The `Drawing` a reference name or destination in the view resolves to:
/// a `drawing:` destination or `dN` definition as it is; an `imageK` name
/// through its backing comment, the `data:` definition's bytes filling
/// the slot the readable layer carries. `None` when no form resolves.
pub(crate) fn lookup(
    key_or_dest: &str,
    defs: &HashMap<String, String>,
    backings: &HashMap<String, String>,
) -> Option<Drawing> {
    if let Some(dest) = key_or_dest
        .starts_with("drawing:")
        .then_some(key_or_dest)
        .or_else(|| defs.get(key_or_dest).map(String::as_str))
    {
        if dest.starts_with("drawing:") {
            return decode(dest).ok();
        }
        if let Some((mime, b64)) = data_uri(dest) {
            let mut drawing = decode(backings.get(key_or_dest)?).ok()?;
            return drawing.fill_slot_data(b64, mime).then_some(drawing);
        }
    }
    None
}

/// Whether `element` or a descendant holds a drawing: `w:drawing`, `w:pict`
/// or `w:object` anywhere below `AlternateContent`'s `Choice`/`Fallback`.
pub(crate) fn holds_drawing(element: &Element) -> bool {
    matches!(element.local(), "drawing" | "pict" | "object")
        || element.elements().any(holds_drawing)
}

/// Whether `element` is a plain inline picture the `![alt](name)` syntax
/// keeps when its raster is extracted: `wp:inline` holding an `a:blip`, no
/// floating anchor, no text box the payload would carry with it.
pub(crate) fn is_plain_inline(element: &Element) -> bool {
    element.is("drawing")
        && element.find("anchor").is_none()
        && element.find("inline").is_some()
        && element.find("blip").is_some()
        && element.find("txbxContent").is_none()
}

/// `element` holds paragraph-level content (`w:p`, `w:tbl`, `w:sdt`)
/// outside a drawing, picture or text box — where it is encapsulated and
/// stays inside the run whatever it holds. Such an element under an
/// AlternateContent branch makes the branch block-level, unspliceable
/// where a run sits.
fn holds_block_content(element: &Element) -> bool {
    element.elements().any(|e| {
        if matches!(e.local(), "drawing" | "pict" | "object" | "txbxContent") {
            false
        } else {
            matches!(e.local(), "p" | "tbl" | "sdt" | "customXml") || holds_block_content(e)
        }
    })
}

/// Whether a paragraph-level `mc:AlternateContent` may be captured whole:
/// one with a drawing and only run-level content below its `Choice`.
pub(crate) fn capturable_alternate(element: &Element) -> bool {
    if !holds_drawing(element) {
        return false;
    }
    element.elements().all(|branch| {
        !matches!(branch.local(), "Choice" | "Fallback") || !holds_block_content(branch)
    })
}

/// Write `drawing` into `package` as content of `part`: the XML with fresh
/// relationship ids, and every dependent part (its own relationships and
/// parts too). `existing` and `used` keep names apart from parts already
/// there; `next_doc_pr` gives each `wp:docPr` a fresh id. The `xml` to
/// splice, `None` when a dependency failed to write.
impl Drawing {
    /// The element goes inside a `w:r` (`w:drawing`, `w:pict`, run-level
    /// AlternateContent), not where a `w:r` may sit (paragraph-level
    /// AlternateContent).
    pub(crate) fn wraps_run(&self) -> bool {
        self.wrap
    }

    /// `self` with the readable layer's bytes out of the payload: the
    /// backing comment of a `data:` definition or an `svg` fence.
    fn without_slot_bytes(&self) -> Drawing {
        let mut backing = self.clone();
        if let Some(slot) = backing.slot
            && let Some(dep) = backing.deps.get_mut(slot)
        {
            dep.b64.clear();
            dep.xml.clear();
        }
        backing
    }

    /// Fill the slot with `b64` bytes and, when `kind` differs, its
    /// content type; `false` when the payload carries no slot.
    pub(crate) fn fill_slot_data(&mut self, b64: &str, kind: &str) -> bool {
        let Some(dep) = self.slot.and_then(|slot| self.deps.get_mut(slot)) else {
            return false;
        };
        dep.b64 = b64.to_string();
        dep.xml.clear();
        if dep.kind != kind {
            dep.kind = kind.to_string();
        }
        true
    }

    /// `text` is the ` ```mermaid ` fence this drawing printed: its hash
    /// is `mhash`'s, so the drawing writes back byte for byte.
    pub(crate) fn mermaid_unchanged(&self, text: &str) -> bool {
        self.mhash.as_deref() == Some(mermaid_hash(text).as_str())
    }

    /// The fence's text edited: patch the chart part it names in place,
    /// keeping everything the Mermaid cannot say. `false` when the
    /// payload names no chart part the patch can reach.
    pub(crate) fn rebuild_mermaid(&mut self, text: &str) -> bool {
        let Some(dep) = self
            .deps
            .iter_mut()
            .find(|dep| dep.kind.ends_with("chart+xml"))
        else {
            return false;
        };
        let xml = if dep.xml.is_empty() {
            base64_decode(&dep.b64).and_then(|b| String::from_utf8(b).ok())
        } else {
            Some(dep.xml.clone())
        };
        let Some(patched) = xml.and_then(|xml| super::mermaid::patch(&xml, text)) else {
            return false;
        };
        dep.xml = patched;
        dep.b64.clear();
        self.mhash = Some(mermaid_hash(text));
        true
    }
}

pub(crate) fn materialize(
    drawing: &Drawing,
    package: &mut PartFs,
    part: &str,
    existing: &HashSet<String>,
    used: &mut HashSet<String>,
    next_doc_pr: &mut u32,
) -> Option<String> {
    let mut xml = drawing.xml.clone();
    for rel in &drawing.rels {
        let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
            (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
            (_, Some(index), _) if index < drawing.deps.len() => {
                let name = dep_part(drawing, index, package, existing, used, next_doc_pr)?;
                (rel.kind.clone(), relative_rel_target(part, &name), false)
            }
            (_, _, Some(raw)) => (rel.kind.clone(), relative_rel_target(part, raw), false),
            _ => return None,
        };
        let rid = relate(package, part, &kind, &target, external);
        xml = xml.replace(&format!("\"{}\"", rel.sym), &format!("\"{rid}\""));
    }
    renumber_doc_pr(&mut xml, next_doc_pr);
    Some(xml)
}

/// A relationship `part` → `target` of `kind`, reusing one already
/// written when there is one: two drawings that share a media part
/// share its relationship too.
fn relate(package: &mut PartFs, part: &str, kind: &str, target: &str, external: bool) -> String {
    if let Some(rels) = package.read_rels_for(part)
        && let Some(rel) = rels
            .items
            .iter()
            .find(|r| r.rel_type == kind && r.target == target)
    {
        return rel.id.clone();
    }
    if external {
        package.add_document_relationship_external(part, kind, target)
    } else {
        package.add_document_relationship(part, kind, target)
    }
}

/// Give every `docPr id` in `xml` a fresh number: the stored `0`s collide
/// with each other and with pictures the writer numbers.
fn renumber_doc_pr(xml: &mut String, next: &mut u32) {
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml.as_str();
    while let Some(at) = rest.find("docPr") {
        out.push_str(&rest[..at + "docPr".len()]);
        rest = &rest[at + "docPr".len()..];
        let Some(id_at) = rest.find("id=\"") else {
            continue;
        };
        let Some(gt) = rest.find('>') else { break };
        if id_at > gt {
            continue;
        }
        out.push_str(&rest[..id_at + 4]);
        rest = &rest[id_at + 4..];
        let digits = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let _ = write!(out, "{next}");
        *next += 1;
        rest = &rest[digits..];
    }
    out.push_str(rest);
    *xml = out;
}

/// Write dependent part `index` and its own dependency closure; its new
/// part name.
fn dep_part(
    drawing: &Drawing,
    index: usize,
    package: &mut PartFs,
    existing: &HashSet<String>,
    used: &mut HashSet<String>,
    next_doc_pr: &mut u32,
) -> Option<String> {
    let dep = drawing.deps.get(index)?;
    let name = name_for(&dep.name, existing, used);
    used.insert(name.clone());
    // The part's own relationships name the parts the recursion is writing;
    // their ids land in a rewrite of its `d{index}.{n}` symbols.
    let mut xml = dep.xml.clone();
    for (n, rel) in dep.rels.iter().enumerate() {
        let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
            (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
            (_, Some(inner), _) if inner < drawing.deps.len() => {
                let inner_name = dep_part(drawing, inner, package, existing, used, next_doc_pr)?;
                (
                    rel.kind.clone(),
                    relative_rel_target(&name, &inner_name),
                    false,
                )
            }
            (_, _, Some(raw)) => (rel.kind.clone(), relative_rel_target(&name, raw), false),
            _ => return None,
        };
        let rid = relate(package, &name, &kind, &target, external);
        if !xml.is_empty() {
            xml = xml.replace(&format!("\"d{index}.{}\"", n + 1), &format!("\"{rid}\""));
        }
    }
    let bytes = if xml.is_empty() {
        base64_decode(&dep.b64)?
    } else {
        renumber_doc_pr(&mut xml, next_doc_pr);
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n{xml}").into_bytes()
    };
    // Two payloads may depend on the one part (two drawings share an
    // image): a part already written byte for byte is the same one, not
    // a collision to rename around.
    if package
        .part_bytes(&dep.name)
        .is_some_and(|written| written == bytes.as_slice())
    {
        used.insert(dep.name.clone());
        return Some(dep.name.clone());
    }
    package.set_part(&name, bytes);
    package.add_content_type_override(&format!("/{name}"), &dep.kind);
    Some(name)
}

/// `want` when nothing holds it, else `want` with `-j2`, `-j3`, … before
/// its extension. `used` names the parts this write already took.
fn name_for(want: &str, existing: &HashSet<String>, used: &HashSet<String>) -> String {
    let free = |name: &str| !existing.contains(name) && !used.contains(name);
    if free(want) {
        return want.to_string();
    }
    let (stem, ext) = match want.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem, format!(".{ext}")),
        _ => (want, String::new()),
    };
    for n in 2..1000 {
        let candidate = format!("{stem}-j{n}{ext}");
        if free(&candidate) {
            return candidate;
        }
    }
    format!("{stem}-j{}{ext}", usize::MAX)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn element(xml: &str) -> Element {
        parse_xml(xml.as_bytes()).unwrap()
    }

    #[test]
    fn xml_round_trips_verbatim() {
        let xml = r#"<wp:anchor xmlns:wp="wp" wp14:anchorId="AB"><wp:docPr id="2" name="P"/><x:y a="1&quot;2"><t>x &amp; y</t></x:y></wp:anchor>"#;
        let e = element(xml);
        assert_eq!(serialize(&e), xml);
    }

    #[test]
    fn declares_prefixes_the_tree_uses() {
        let mut e = element(
            r#"<w:drawing><wp:anchor wp14:anchorId="X"><wp:docPr/></wp:anchor></w:drawing>"#,
        );
        let scope: Scope = vec![
            ("w".into(), "W".into()),
            ("wp".into(), "WP".into()),
            ("wp14".into(), "WP14".into()),
        ];
        declare(&mut e, &scope);
        let xml = serialize(&e);
        assert!(xml.contains("xmlns:wp=\"WP\""), "{xml}");
        assert!(xml.contains("xmlns:wp14=\"WP14\""), "{xml}");
        assert!(!xml.contains("xmlns:w"), "{xml}");
    }

    #[test]
    fn rel_names_match_only_the_relationship_namespace() {
        let scope: Scope = vec![
            ("r".into(), REL_NS[0].into()),
            ("v".into(), "urn:vml".into()),
        ];
        assert!(is_rel_name("r:embed", &scope));
        // The Strict relationship namespace names one too.
        let strict: Scope = vec![("rel".into(), REL_NS[1].into())];
        assert!(is_rel_name("rel:embed", &strict));
        assert!(!is_rel_name("v:id", &scope));
        assert!(!is_rel_name("embed", &scope));
    }

    #[test]
    fn token_parses_destination_and_length() {
        let text = "![Text Box 2](drawing:eyJ4) next";
        let (dest, len) = at(text).unwrap();
        assert_eq!(dest, "drawing:eyJ4");
        assert_eq!(&text[..len], "![Text Box 2](drawing:eyJ4)");
        assert!(at("![a](media/x.png)").is_none());
        assert!(at("x![a](drawing:Zg)").is_none());
        assert!(at("![a](drawing:has space)").is_none());
        let text = "![Text Box 2][d12] next";
        let (dest, len) = at(text).unwrap();
        assert_eq!(dest, "d12");
        assert_eq!(&text[..len], "![Text Box 2][d12]");
        assert!(at("![a][d]").is_none());
        assert!(at("![a][dx]").is_none());
        assert!(at("![a][other]").is_none());
        assert!(at("![a][d1").is_none());
    }

    #[test]
    fn doc_pr_ids_normalize_and_renumber() {
        let mut e = element(r#"<w:drawing><wp:inline><wp:docPr id="7"/></wp:inline></w:drawing>"#);
        normalize_ids(&mut e);
        assert_eq!(
            serialize(&e),
            r#"<w:drawing><wp:inline><wp:docPr id="0"/></wp:inline></w:drawing>"#
        );
        let mut xml = serialize(&e);
        let mut next = 41;
        renumber_doc_pr(&mut xml, &mut next);
        assert_eq!(
            xml,
            r#"<w:drawing><wp:inline><wp:docPr id="41"/></wp:inline></w:drawing>"#
        );
        assert_eq!(next, 42);
    }
}
