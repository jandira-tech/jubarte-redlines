// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Read-only views of a DOCX for agents and tools: body paragraphs an edit can
//! address, package facts, and a Markdown projection that carries paragraph
//! ids so the reader's coordinates are the editor's coordinates.
//!
//! Paragraph order is body XML order, table cells included; text boxes are
//! separate stories and are omitted from the body (their owner paragraph is
//! flagged). Text is the visible-run projection: `w:del` and `w:moveFrom`
//! content is skipped, tabs stay `\t`, line breaks become `\n`, `w:sym`
//! becomes U+FFFC. Constructs this projection cannot represent are reported per
//! paragraph in `limitations`; they never refuse the document. Malformed
//! packages and XML do.

use std::collections::BTreeSet;
use std::fmt;

use serde::Serialize;

use crate::namespaces::{MC, W};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

/// Wire schema of [`inspect_json`] and of the edit plan that consumes it.
pub const SCHEMA_VERSION: u32 = 1;

/// One paragraph of the body or a story. `index` and `id` are valid for this
/// exact snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Paragraph {
    /// Zero-based order in its story, table-cell paragraphs included.
    pub index: usize,
    /// `body:p:{index}` (or `header1:p:{index}`, ...); the edit plan's
    /// paragraph selector.
    pub id: String,
    /// Visible text (see module docs for the projection rules).
    pub text: String,
    /// `w:pStyle` id when set directly on the paragraph.
    pub style: Option<String>,
    /// Paragraph carries direct `w:numPr` numbering.
    pub numbered: bool,
    /// Paragraph lives in a table cell.
    pub in_table: bool,
    /// Paragraph contains an explicit page break.
    pub page_break: bool,
    /// Direct run formatting over `text`, in char offsets, adjacent equal
    /// formatting merged. Style-inherited formatting is not resolved.
    pub runs: Vec<Span>,
    /// Constructs present but not represented in `text` (`sym`, `field`,
    /// `hyperlink`, `content_control`, `drawing`, `text_box_omitted`,
    /// `alternate_content`, `column_break`, `revision`, `note_reference`,
    /// `unknown:<local-name>`).
    pub limitations: Vec<String>,
}

/// Direct formatting of a run of `text` (`[start, end)` in chars).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Span {
    /// First char (inclusive).
    pub start: usize,
    /// Last char (exclusive).
    pub end: usize,
    /// Direct bold.
    pub bold: bool,
    /// Direct italic.
    pub italic: bool,
    /// Direct underline.
    pub underline: bool,
    /// Direct highlight color name.
    pub highlight: Option<String>,
}

/// Package facts. Counts are XML facts, not rendered-page facts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    /// Body paragraphs, table cells included.
    pub paragraphs: usize,
    /// Body tables, nested included.
    pub tables: usize,
    /// `w:fldSimple` plus complex-field `begin` markers in the body.
    pub fields: usize,
    /// Body section properties, excluding saved revision copies.
    pub sections: usize,
    /// Comment definitions in the related comments part.
    pub comments: usize,
    /// Raw revision carrier elements across XML parts, not coalesced changes.
    pub revisions: usize,
    /// Footnotes excluding separators.
    pub footnotes: usize,
    /// Endnotes excluding separators.
    pub endnotes: usize,
    /// Related header parts.
    pub headers: usize,
    /// Related footer parts.
    pub footers: usize,
    /// Parts whose content type starts with `image/`.
    pub images: usize,
    /// Body or paragraph-style numbering declarations exist.
    pub list_numbering: bool,
    /// Settings enable tracking of new revisions.
    pub track_changes: bool,
}

/// Snapshot serialized by [`inspect_json`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    /// Wire schema version.
    pub schema_version: u32,
    /// SHA-256 of the inspected bytes.
    pub source_sha256: String,
    /// Package facts.
    pub summary: Summary,
    /// Body paragraphs.
    pub paragraphs: Vec<Paragraph>,
    /// Header, footer and note stories, each with its own paragraphs.
    pub stories: Vec<Story>,
}

/// A header, footer or notes part an edit plan can address. Its paragraph
/// ids are `{id}:p:{index}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Story {
    /// The part's file stem: `header1`, `footer2`, `footnotes`, `endnotes`.
    pub id: String,
    /// `header`, `footer`, `footnotes` or `endnotes`.
    pub kind: String,
    /// Package part name, e.g. `word/header1.xml`.
    pub part: String,
    /// The story's paragraphs; separator notes are left out.
    pub paragraphs: Vec<Paragraph>,
}

/// An invalid package or XML part.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InspectError {
    /// OPC/ZIP load failure.
    Package(String),
    /// Missing main document part or body.
    MissingDocument,
    /// Invalid XML, encoding or setting value, including the part name.
    Invalid(String),
    /// Refused before parsing by [`crate::admission`] (a resource budget,
    /// duplicate or unsafe part, or not a Word package).
    Admission(crate::admission::AdmissionError),
}

impl fmt::Display for InspectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Package(message) => write!(f, "opening DOCX: {message}"),
            Self::MissingDocument => f.write_str("DOCX has no main document part or body"),
            Self::Invalid(message) => write!(f, "invalid DOCX content: {message}"),
            Self::Admission(refused) => write!(f, "DOCX refused: {refused}"),
        }
    }
}

impl std::error::Error for InspectError {}

/// SHA-256 of the exact input bytes, lowercase hex: the snapshot guard shared
/// by inspection and edit plans.
pub fn source_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Body paragraphs in body order.
pub fn paragraphs(docx: &[u8]) -> Result<Vec<Paragraph>, InspectError> {
    let opened = Opened::open(docx)?;
    Ok(body_paragraphs(&opened.dom, opened.body))
}

/// Package facts, following OPC relationships for related parts.
pub fn summary(docx: &[u8]) -> Result<Summary, InspectError> {
    let opened = Opened::open(docx)?;
    opened.summary()
}

/// Header, footer and note stories in a stable order (headers, footers,
/// footnotes, endnotes).
pub fn stories(docx: &[u8]) -> Result<Vec<Story>, InspectError> {
    Opened::open(docx)?.stories()
}

/// Paragraphs prefixed with their ids and direct formatting as Markdown
/// marks: `[body:p:12 Heading1] (a) **Confidentiality.** You will ...`.
/// Story paragraphs (`[header1:p:0] ...`) follow the body's.
pub fn markdown(docx: &[u8]) -> Result<String, InspectError> {
    let opened = Opened::open(docx)?;
    let mut all = body_paragraphs(&opened.dom, opened.body);
    for story in opened.stories()? {
        all.extend(story.paragraphs);
    }
    Ok(render_markdown(&all))
}

/// The full snapshot as JSON (`schema_version`, `source_sha256`, `summary`,
/// `paragraphs`).
pub fn inspect_json(docx: &[u8]) -> Result<String, InspectError> {
    let opened = Opened::open(docx)?;
    let snapshot = Snapshot {
        schema_version: SCHEMA_VERSION,
        source_sha256: source_sha256(docx),
        summary: opened.summary()?,
        paragraphs: body_paragraphs(&opened.dom, opened.body),
        stories: opened.stories()?,
    };
    serde_json::to_string(&snapshot).map_err(|e| InspectError::Invalid(e.to_string()))
}

/// Markdown rendering of already inspected paragraphs.
pub fn render_markdown(paragraphs: &[Paragraph]) -> String {
    let mut out = String::new();
    for p in paragraphs {
        out.push('[');
        out.push_str(&p.id);
        if let Some(style) = &p.style {
            out.push(' ');
            out.push_str(style);
        }
        out.push(']');
        let body = markdown_body(p);
        if !body.is_empty() {
            out.push(' ');
            out.push_str(&body);
        }
        out.push_str("\n\n");
    }
    if out.ends_with("\n\n") {
        out.pop();
    }
    out
}

fn markdown_body(p: &Paragraph) -> String {
    let chars: Vec<char> = p.text.chars().collect();
    let mut out = String::new();
    let mut cursor = 0;
    for span in &p.runs {
        let start = span.start.min(chars.len());
        let end = span.end.min(chars.len());
        if start > cursor {
            out.extend(&chars[cursor..start]);
        }
        let piece: String = chars[start..end].iter().collect();
        out.push_str(&emphasize(&piece, span));
        cursor = end;
    }
    if cursor < chars.len() {
        out.extend(&chars[cursor..]);
    }
    out.trim_end().to_string()
}

fn emphasize(piece: &str, span: &Span) -> String {
    let core = piece.trim();
    if core.is_empty() || (!span.bold && !span.italic && span.highlight.is_none()) {
        return piece.to_string();
    }
    let lead = &piece[..piece.len() - piece.trim_start().len()];
    let trail = &piece[piece.trim_end().len()..];
    let mut open = String::new();
    if span.highlight.is_some() {
        open.push_str("==");
    }
    if span.bold {
        open.push_str("**");
    }
    if span.italic {
        open.push('*');
    }
    let close: String = open.chars().rev().collect();
    format!("{lead}{open}{core}{close}{trail}")
}

/// A parsed main document part plus its package.
pub(crate) struct Opened {
    pub(crate) pkg: PartFs,
    pub(crate) main: String,
    pub(crate) dom: Dom,
    pub(crate) document: NodeId,
    pub(crate) body: NodeId,
}

impl Opened {
    pub(crate) fn open(bytes: &[u8]) -> Result<Self, InspectError> {
        crate::admission::admit(bytes, crate::admission::InputLimits::default())
            .map_err(InspectError::Admission)?;
        let normalized = crate::strict_translation::strict_to_transitional_docx(bytes);
        let pkg =
            PartFs::open(&normalized).map_err(|error| InspectError::Package(error.to_string()))?;
        let main = main_part(&pkg)?;
        let (dom, document, root) = parse_part(&pkg, &main)?;
        let body = dom
            .element(root, &W::body())
            .ok_or(InspectError::MissingDocument)?;
        Ok(Self {
            pkg,
            main,
            dom,
            document,
            body,
        })
    }

    fn summary(&self) -> Result<Summary, InspectError> {
        let (dom, body, pkg) = (&self.dom, self.body, &self.pkg);
        let mut summary = Summary {
            paragraphs: body_paragraphs(dom, body).len(),
            tables: dom.descendants(body, Some(&W::tbl())).len(),
            fields: dom.descendants(body, Some(&W::fld_simple())).len()
                + dom
                    .descendants(body, Some(&W::fld_char()))
                    .iter()
                    .filter(|&&node| dom.attribute(node, &W::name("fldCharType")) == Some("begin"))
                    .count(),
            sections: dom
                .descendants(body, Some(&W::sect_pr()))
                .iter()
                .filter(|&&node| {
                    !has_ancestor(dom, node, &W::name("sectPrChange"))
                        && !has_ancestor(dom, node, &W::p_pr_change())
                })
                .count(),
            list_numbering: !dom.descendants(body, Some(&W::num_pr())).is_empty(),
            ..Summary::default()
        };
        for name in pkg.parts() {
            let mime = pkg.content_type_for(&name);
            let mime = mime.as_deref();
            if mime.is_some_and(|mime| mime.starts_with("image/")) {
                summary.images += 1;
            }
            // Revisions live only in WordprocessingML parts; custom XML items
            // and vendor parts are not read, so the checked reader cannot
            // refuse the document over them.
            let wordprocessing = mime.is_some_and(|mime| {
                mime.starts_with("application/vnd.openxmlformats-officedocument.wordprocessingml.")
            });
            if wordprocessing && name != self.main {
                let (part_dom, _, part_root) = parse_part(pkg, &name)?;
                summary.revisions += revision_count(&part_dom, part_root);
            }
        }
        summary.revisions += revision_count(dom, self.body);
        for (kind, targets) in [
            ("header", self.related("header")),
            ("footer", self.related("footer")),
            ("comments", self.related("comments")),
            ("footnotes", self.related("footnotes")),
            ("endnotes", self.related("endnotes")),
            ("settings", self.related("settings")),
            ("styles", self.related("styles")),
        ] {
            for target in targets {
                let (part_dom, _, part_root) = parse_part(pkg, &target)?;
                match kind {
                    "header" => summary.headers += 1,
                    "footer" => summary.footers += 1,
                    "comments" => {
                        summary.comments += part_dom
                            .descendants(part_root, Some(&W::name("comment")))
                            .len();
                    }
                    "footnotes" => {
                        summary.footnotes += note_count(&part_dom, part_root, "footnote");
                    }
                    "endnotes" => summary.endnotes += note_count(&part_dom, part_root, "endnote"),
                    "styles" => {
                        let numbered_style = part_dom
                            .descendants(part_root, Some(&W::name("style")))
                            .into_iter()
                            .any(|style| {
                                part_dom.attribute(style, &W::name("type")) == Some("paragraph")
                                    && !part_dom.descendants(style, Some(&W::num_pr())).is_empty()
                            });
                        summary.list_numbering |= numbered_style;
                    }
                    _ => {
                        if let Some(setting) =
                            part_dom.element(part_root, &W::name("trackRevisions"))
                        {
                            summary.track_changes = match part_dom.attribute(setting, &W::val()) {
                                None | Some("true" | "1" | "on") => true,
                                Some("false" | "0" | "off") => false,
                                Some(value) => {
                                    return Err(InspectError::Invalid(format!(
                                        "{target}: invalid trackRevisions value {value}"
                                    )));
                                }
                            };
                        }
                    }
                }
            }
        }
        Ok(summary)
    }

    /// Internal targets of the main part's relationships of one kind.
    /// Header, footer and note parts: `(story id, kind, part name)`. The id
    /// is the part's file stem; `header2` sorts before `header10`.
    pub(crate) fn story_parts(&self) -> Vec<(String, &'static str, String)> {
        let mut out = Vec::new();
        for kind in ["header", "footer", "footnotes", "endnotes"] {
            let mut parts: Vec<String> = self.related(kind).into_iter().collect();
            parts.sort_by_key(|part| (part.len(), part.clone()));
            for part in parts {
                let stem = part.rsplit('/').next().unwrap_or(&part);
                let id = stem.strip_suffix(".xml").unwrap_or(stem).to_string();
                out.push((id, kind, part));
            }
        }
        out
    }

    fn stories(&self) -> Result<Vec<Story>, InspectError> {
        self.story_parts()
            .into_iter()
            .map(|(id, kind, part)| {
                let (dom, _, root) = parse_part(&self.pkg, &part)?;
                let paragraphs = paragraphs_of(&dom, story_paragraph_nodes(&dom, root), &id);
                Ok(Story {
                    id,
                    kind: kind.to_string(),
                    part,
                    paragraphs,
                })
            })
            .collect()
    }

    pub(crate) fn related(&self, kind: &str) -> BTreeSet<String> {
        self.pkg
            .read_rels_for(&self.main)
            .into_iter()
            .flat_map(|rels| &rels.items)
            .filter(|rel| {
                rel.target_mode.as_deref() != Some("External")
                    && rel.rel_type.rsplit('/').next() == Some(kind)
            })
            .map(|rel| self.pkg.resolve_rel_target(&self.main, &rel.target))
            .collect()
    }
}

fn main_part(pkg: &PartFs) -> Result<String, InspectError> {
    pkg.main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".to_string())
        })
        .ok_or(InspectError::MissingDocument)
}

/// Parse one XML part through the checked reader: `(dom, document, root)`.
pub(crate) fn parse_part(pkg: &PartFs, name: &str) -> Result<(Dom, NodeId, NodeId), InspectError> {
    let bytes = pkg
        .part_bytes(name)
        .ok_or_else(|| InspectError::Invalid(format!("missing related part {name}")))?;
    let xml = std::str::from_utf8(bytes)
        .map_err(|error| InspectError::Invalid(format!("{name}: {error}")))?;
    crate::xmllinq::parse::validate_xml(xml)
        .map_err(|error| InspectError::Invalid(format!("{name}: {error}")))?;
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(xml);
    let root = dom
        .root(document)
        .ok_or_else(|| InspectError::Invalid(format!("{name}: missing XML root")))?;
    Ok((dom, document, root))
}

fn has_ancestor(dom: &Dom, node: NodeId, name: &XName) -> bool {
    !dom.ancestors(node, Some(name)).is_empty()
}

fn note_count(dom: &Dom, root: NodeId, local: &str) -> usize {
    dom.descendants(root, Some(&W::name(local)))
        .iter()
        .filter(|&&node| {
            !matches!(
                dom.attribute(node, &W::name("type")),
                Some("separator" | "continuationSeparator")
            )
        })
        .count()
}

const REVISION_CARRIERS: &[&str] = &[
    "ins",
    "del",
    "moveFrom",
    "moveTo",
    "rPrChange",
    "pPrChange",
    "sectPrChange",
    "tblPrChange",
    "tblGridChange",
    "trPrChange",
    "tcPrChange",
    "numberingChange",
    "cellIns",
    "cellDel",
    "cellMerge",
];

/// Raw revision carrier elements under `root` (self included).
pub(crate) fn revision_count(dom: &Dom, root: NodeId) -> usize {
    dom.descendants_and_self(root, None)
        .iter()
        .filter(|&&node| {
            dom.name(node).is_some_and(|name| {
                name.namespace_name() == W::URI && REVISION_CARRIERS.contains(&name.local_name())
            })
        })
        .count()
}

/// The body paragraphs (`w:p` outside text boxes) in document order.
pub(crate) fn body_paragraph_nodes(dom: &Dom, body: NodeId) -> Vec<NodeId> {
    dom.descendants(body, Some(&W::p()))
        .into_iter()
        .filter(|&p| !has_ancestor(dom, p, &W::txbx_content()))
        .collect()
}

/// A header, footer or notes part's paragraphs, separator notes left out.
pub(crate) fn story_paragraph_nodes(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    body_paragraph_nodes(dom, root)
        .into_iter()
        .filter(|&p| {
            !dom.ancestors(p, None).into_iter().any(|a| {
                (dom.name_is(a, &W::name("footnote")) || dom.name_is(a, &W::name("endnote")))
                    && matches!(
                        dom.attribute(a, &W::name("type")),
                        Some("separator" | "continuationSeparator" | "continuationNotice")
                    )
            })
        })
        .collect()
}

fn body_paragraphs(dom: &Dom, body: NodeId) -> Vec<Paragraph> {
    paragraphs_of(dom, body_paragraph_nodes(dom, body), "body")
}

fn paragraphs_of(dom: &Dom, nodes: Vec<NodeId>, story: &str) -> Vec<Paragraph> {
    nodes
        .into_iter()
        .enumerate()
        .map(|(index, p)| {
            let projection = project_paragraph(dom, p);
            let ppr = dom.element(p, &W::p_pr());
            let style = ppr
                .and_then(|ppr| dom.element(ppr, &W::p_style()))
                .and_then(|s| dom.attribute(s, &W::val()))
                .map(str::to_string);
            let numbered = ppr.is_some_and(|ppr| dom.element(ppr, &W::num_pr()).is_some());
            Paragraph {
                index,
                id: format!("{story}:p:{index}"),
                text: projection.text,
                style,
                numbered,
                in_table: has_ancestor(dom, p, &W::tc()),
                page_break: projection.page_break,
                runs: projection.spans,
                limitations: projection.limitations.into_iter().collect(),
            }
        })
        .collect()
}

/// What one visible piece of paragraph text maps back to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Piece {
    /// A `w:t` element and the `w:r` that owns it.
    Text { t: NodeId, run: NodeId },
    /// A non-text run child projected as one char (`w:tab`, `w:br`, ...).
    Glyph { element: NodeId, run: NodeId },
}

/// One projected segment: `text[start..end]` (bytes) comes from `piece`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Segment {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) piece: Piece,
    /// The run is a direct child of the paragraph (editable position).
    pub(crate) direct: bool,
}

/// Visible projection of one paragraph, with the segment map edits need.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Projection {
    pub(crate) text: String,
    pub(crate) segments: Vec<Segment>,
    pub(crate) spans: Vec<Span>,
    pub(crate) page_break: bool,
    pub(crate) limitations: BTreeSet<String>,
    /// Byte offsets of complex-field `w:fldChar` markers. A field with an
    /// empty result adds no text, so its markers are the only trace of it.
    pub(crate) field_marks: Vec<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Format {
    bold: bool,
    italic: bool,
    underline: bool,
    highlight: Option<String>,
}

/// Project one `w:p` (see module docs for the rules).
pub(crate) fn project_paragraph(dom: &Dom, paragraph: NodeId) -> Projection {
    let mut projection = Projection::default();
    let mut formats: Vec<(usize, usize, Format)> = Vec::new();
    let mut fields = 0;
    walk_container(
        dom,
        paragraph,
        true,
        &mut fields,
        &mut projection,
        &mut formats,
    );
    projection.spans = merge_spans(&projection.text, formats);
    projection
}

/// `fields` counts the complex fields open at this point of the paragraph:
/// their `begin`/`end` runs are siblings of the result runs, so a result run
/// is direct by position yet belongs to the field.
fn walk_container(
    dom: &Dom,
    container: NodeId,
    direct: bool,
    fields: &mut usize,
    out: &mut Projection,
    formats: &mut Vec<(usize, usize, Format)>,
) {
    for child in dom.elements(container, None) {
        let Some(name) = dom.name(child) else {
            continue;
        };
        let ns = name.namespace_name();
        let local = name.local_name();
        if ns == MC::URI {
            out.limitations.insert("alternate_content".into());
            if local == "AlternateContent"
                && let Some(choice) = dom.element(child, &MC::name("Choice"))
            {
                walk_container(dom, choice, false, fields, out, formats);
            }
            continue;
        }
        if ns != W::URI {
            out.limitations.insert(format!("unknown:{local}"));
            continue;
        }
        match local {
            "pPr" | "proofErr" | "bookmarkStart" | "bookmarkEnd" | "commentRangeStart"
            | "commentRangeEnd" | "permStart" | "permEnd" | "moveFromRangeStart"
            | "moveFromRangeEnd" | "moveToRangeStart" | "moveToRangeEnd" => {}
            "del" | "moveFrom" => {
                out.limitations.insert("revision".into());
            }
            "ins" | "moveTo" => {
                out.limitations.insert("revision".into());
                walk_container(dom, child, false, fields, out, formats);
            }
            "r" => walk_run(dom, child, direct, fields, out, formats),
            "hyperlink" => {
                out.limitations.insert("hyperlink".into());
                walk_container(dom, child, false, fields, out, formats);
            }
            "fldSimple" => {
                out.limitations.insert("field".into());
                walk_container(dom, child, false, fields, out, formats);
            }
            "sdt" => {
                out.limitations.insert("content_control".into());
                if let Some(content) = dom.element(child, &W::sdt_content()) {
                    walk_container(dom, content, false, fields, out, formats);
                }
            }
            "smartTag" | "customXml" | "dir" | "bdo" => {
                walk_container(dom, child, false, fields, out, formats);
            }
            _ => {
                out.limitations.insert(format!("unknown:{local}"));
            }
        }
    }
}

fn walk_run(
    dom: &Dom,
    run: NodeId,
    direct: bool,
    fields: &mut usize,
    out: &mut Projection,
    formats: &mut Vec<(usize, usize, Format)>,
) {
    let format = run_format(dom, run);
    let start = out.text.len();
    for child in dom.elements(run, None) {
        let Some(name) = dom.name(child) else {
            continue;
        };
        if name.namespace_name() != W::URI {
            out.limitations
                .insert(format!("unknown:{}", name.local_name()));
            continue;
        }
        let direct = direct && *fields == 0;
        let glyph = |out: &mut Projection, ch: char| {
            let s = out.text.len();
            out.text.push(ch);
            out.segments.push(Segment {
                start: s,
                end: out.text.len(),
                piece: Piece::Glyph {
                    element: child,
                    run,
                },
                direct,
            });
        };
        match name.local_name() {
            "t" => {
                let s = out.text.len();
                let value = dom.value_str(child);
                out.text.push_str(&value);
                out.segments.push(Segment {
                    start: s,
                    end: out.text.len(),
                    piece: Piece::Text { t: child, run },
                    direct,
                });
            }
            "tab" | "ptab" => glyph(out, '\t'),
            "cr" => glyph(out, '\n'),
            "br" => match dom.attribute(child, &W::name("type")) {
                Some("page") => out.page_break = true,
                Some("column") => {
                    out.limitations.insert("column_break".into());
                }
                _ => glyph(out, '\n'),
            },
            "noBreakHyphen" => glyph(out, '\u{2011}'),
            "softHyphen" => glyph(out, '\u{00ad}'),
            "sym" => {
                out.limitations.insert("sym".into());
                glyph(out, '\u{fffc}');
            }
            "fldChar" => {
                out.limitations.insert("field".into());
                out.field_marks.push(out.text.len());
                match dom.attribute(child, &W::name("fldCharType")) {
                    Some("begin") => *fields += 1,
                    Some("end") => *fields = fields.saturating_sub(1),
                    _ => {}
                }
            }
            "drawing" | "pict" | "object" => {
                out.limitations.insert("drawing".into());
                if !dom.descendants(child, Some(&W::txbx_content())).is_empty() {
                    out.limitations.insert("text_box_omitted".into());
                }
            }
            "footnoteReference" | "endnoteReference" => {
                out.limitations.insert("note_reference".into());
            }
            "rPr"
            | "instrText"
            | "delText"
            | "delInstrText"
            | "lastRenderedPageBreak"
            | "commentReference"
            | "annotationRef"
            | "footnoteRef"
            | "endnoteRef"
            | "separator"
            | "continuationSeparator"
            | "dayShort"
            | "dayLong"
            | "monthShort"
            | "monthLong"
            | "yearShort"
            | "yearLong"
            | "pgNum" => {}
            other => {
                out.limitations.insert(format!("unknown:{other}"));
            }
        }
    }
    if out.text.len() > start {
        formats.push((start, out.text.len(), format));
    }
}

fn run_format(dom: &Dom, run: NodeId) -> Format {
    let Some(rpr) = dom.element(run, &W::r_pr()) else {
        return Format::default();
    };
    let on = |local: &str| -> bool {
        dom.element(rpr, &W::name(local))
            .is_some_and(|el| !matches!(dom.attribute(el, &W::val()), Some("false" | "0" | "off")))
    };
    let underline = dom
        .element(rpr, &W::name("u"))
        .is_some_and(|u| dom.attribute(u, &W::val()).is_none_or(|v| v != "none"));
    let highlight = dom
        .element(rpr, &W::name("highlight"))
        .and_then(|h| dom.attribute(h, &W::val()))
        .filter(|v| *v != "none")
        .map(str::to_string);
    Format {
        bold: on("b"),
        italic: on("i"),
        underline,
        highlight,
    }
}

fn merge_spans(text: &str, formats: Vec<(usize, usize, Format)>) -> Vec<Span> {
    let mut merged: Vec<(usize, usize, Format)> = Vec::new();
    for (start, end, format) in formats {
        if let Some(last) = merged.last_mut()
            && last.1 == start
            && last.2 == format
        {
            last.1 = end;
        } else {
            merged.push((start, end, format));
        }
    }
    let char_at = |byte: usize| text[..byte].chars().count();
    merged
        .into_iter()
        .map(|(start, end, format)| Span {
            start: char_at(start),
            end: char_at(end),
            bold: format.bold,
            italic: format.italic,
            underline: format.underline,
            highlight: format.highlight,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_is_lowercase_hex_of_the_bytes() {
        assert_eq!(
            source_sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn markdown_moves_edge_whitespace_outside_emphasis() {
        let span = Span {
            start: 0,
            end: 0,
            bold: true,
            italic: true,
            underline: false,
            highlight: Some("yellow".into()),
        };
        assert_eq!(emphasize("  Title ", &span), "  ==***Title***== ");
        assert_eq!(emphasize("   ", &span), "   ");
        let plain = Span {
            bold: false,
            italic: false,
            highlight: None,
            ..span
        };
        assert_eq!(emphasize(" x ", &plain), " x ");
    }

    #[test]
    fn markdown_of_empty_and_styled_paragraphs() {
        let p = |id: &str, text: &str, style: Option<&str>| Paragraph {
            index: 0,
            id: id.into(),
            text: text.into(),
            style: style.map(str::to_string),
            numbered: false,
            in_table: false,
            page_break: false,
            runs: Vec::new(),
            limitations: Vec::new(),
        };
        let md = render_markdown(&[
            p("body:p:0", "Scope", Some("Heading1")),
            p("body:p:1", "", None),
        ]);
        assert_eq!(md, "[body:p:0 Heading1] Scope\n\n[body:p:1]\n");
        assert_eq!(render_markdown(&[]), "");
    }

    #[test]
    fn spans_merge_only_adjacent_equal_formatting() {
        let bold = Format {
            bold: true,
            ..Format::default()
        };
        let spans = merge_spans(
            "abcdef",
            vec![
                (0, 2, bold.clone()),
                (2, 4, bold.clone()),
                (4, 6, Format::default()),
            ],
        );
        assert_eq!(spans.len(), 2);
        assert_eq!((spans[0].start, spans[0].end, spans[0].bold), (0, 4, true));
        assert_eq!((spans[1].start, spans[1].end, spans[1].bold), (4, 6, false));
    }
}
