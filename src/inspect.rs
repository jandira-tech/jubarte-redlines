// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Read-only views of a DOCX for agents and tools: body paragraphs an edit can
//! address, package facts, and a Markdown projection that carries paragraph
//! ids so the reader's coordinates are the editor's coordinates.
//!
//! Paragraph order is body XML order, table cells included; text boxes are
//! not stories and are omitted from the body (their owner paragraph is
//! flagged). Text is the visible-run projection: `w:del` and `w:moveFrom`
//! content is skipped, tabs stay `\t`, line breaks become `\n`, `w:sym`
//! becomes U+FFFC. Constructs this projection cannot represent are reported per
//! paragraph in `limitations`; they never refuse the document. Malformed
//! packages and XML do.

use std::collections::BTreeSet;
use std::fmt;

use serde::Serialize;

use crate::namespaces::{MC, W, W14, W15, WNE};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

mod tables;
pub(crate) use tables::top_level_table_nodes;

pub use tables::{NestedTable, Table, TableCell};

/// Wire schema version of legacy [`crate::edit`] plans and reports.
pub const SCHEMA_VERSION: u32 = 1;

/// Snapshot wire schema. Version 2 numbers top-level tables only and nests
/// cell tables recursively, with a parent cell path rather than an index.
/// Version 3 lists logical changes (plan 11): a change Word wrote as
/// several marks is one entry with `members`, and kind `replacement`
/// exists.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 3;

/// The story every document has; its paragraph ids are `pN`, and a
/// selector without a `story` searches it.
pub(crate) const BODY_STORY: &str = "body";

/// Kinds of part an edit plan addresses as a story besides the body, in
/// the order [`stories`] lists them. Each is the last segment of the part's
/// relationship type; the story's id is the part's file stem (`header1`,
/// `footer2`, `footnotes`, `endnotes`) and [`Story::kind`] is one of these.
/// Comments and text boxes are not stories. `capabilities` advertises this
/// list after `body`, so the manifest and the parser cannot drift.
pub(crate) const STORY_KINDS: &[&str] = &["header", "footer", "footnotes", "endnotes"];

/// One paragraph of the body or a story. `index` and `id` are valid for this
/// exact snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Paragraph {
    /// Zero-based order in its story, table-cell paragraphs included.
    pub index: usize,
    /// `p{index}` (or `header1`, `header1.p{index}`, ...:
    /// [`crate::ids::paragraph_id`]); the edit plan's paragraph selector.
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
    /// Top-level body tables, numbered as the view's `tN`.
    pub tables: usize,
    /// `w:fldSimple` plus complex-field `begin` markers in the body.
    pub fields: usize,
    /// Body section properties, excluding saved revision copies.
    pub sections: usize,
    /// Comment definitions in the related comments part.
    pub comments: usize,
    /// Tracked changes, as `jubarte changes` lists them: the body, headers,
    /// footers, notes and styles, a move's sides and a text box's two copies
    /// counted once each, and a change Word wrote as several marks once.
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
    /// The input's own validity findings, which `read --json` reports here
    /// (#1309); absent where nothing validated the input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_validity: Option<crate::validate::InputValidity>,
}

/// Snapshot of a package as it stood at inspection time, serialized by
/// [`inspect_json`] and consumed by [`crate::edit::apply_plan_json`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    /// Wire schema version.
    pub schema_version: u32,
    /// SHA-256 of the inspected bytes.
    pub source_sha256: String,
    /// The choice [`settled_snapshot`] read the text under; absent from a
    /// plain snapshot, whose text is the accepted one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_changes: Option<Settle>,
    /// Package facts.
    pub summary: Summary,
    /// Body paragraphs.
    pub paragraphs: Vec<Paragraph>,
    /// Header, footer and note stories, each with its own paragraphs.
    pub stories: Vec<Story>,
    /// Top-level body tables as grids, with nested tables under their cells.
    pub tables: Vec<Table>,
    /// Content controls (`w:sdt`) in the body, in document order.
    pub controls: Vec<ContentControl>,
    /// Tracked changes, as [`crate::changes::list_logical_changes`] lists
    /// them.
    pub changes: Vec<crate::changes::Change>,
    /// Comments, as [`crate::comments::list_comments`] lists them.
    pub comments: Vec<crate::comments::CommentRecord>,
}

/// How [`settled_snapshot`] settles the tracked changes before it reads
/// the text (`read --json --track-changes`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Settle {
    /// Every change accepted: the text a plain snapshot reads.
    Accept,
    /// Every change rejected, as Word's Reject All leaves the text:
    /// insertions gone, deletions back.
    Reject,
}

impl Snapshot {
    /// Keep the body paragraphs numbered in `body_paragraphs` (`N` of
    /// `pN`), and the changes and comments sitting in one of them; a
    /// comment whose range spans a kept paragraph counts. `summary`,
    /// `stories`, `tables` and `controls` stay whole.
    pub fn retain(&mut self, body_paragraphs: &[usize]) {
        self.retain_with(body_paragraphs, &[]);
    }

    /// [`Snapshot::retain`], keeping too the changes and comments in
    /// `stories` (`header1`, `footnotes`): those a `--changed` selection
    /// changed.
    pub(crate) fn retain_with(&mut self, body_paragraphs: &[usize], stories: &[String]) {
        let kept: BTreeSet<usize> = body_paragraphs.iter().copied().collect();
        self.paragraphs
            .retain(|p| body_index(Some(&p.id)).is_some_and(|n| kept.contains(&n)));
        let in_story = |paragraph: Option<&String>| {
            paragraph
                .and_then(|p| crate::ids::paragraph_place(p))
                .is_some_and(|(story, _)| stories.contains(&story))
        };
        self.changes.retain(|c| {
            body_index(c.paragraph.as_ref()).is_some_and(|n| kept.contains(&n))
                || in_story(c.paragraph.as_ref())
        });
        self.comments.retain(|c| {
            let first = body_index(c.paragraph.as_ref());
            let last = body_index(c.last_paragraph.as_ref()).or(first);
            match (first, last) {
                (Some(a), Some(z)) => kept.range(a..=z.max(a)).next().is_some(),
                _ => in_story(c.paragraph.as_ref()),
            }
        });
    }
}

/// `N` of a body paragraph's id `pN`.
fn body_index(id: Option<&String>) -> Option<usize> {
    crate::ids::paragraph_place(id?)
        .filter(|(story, _)| story == "body")
        .map(|(_, n)| n)
}

/// A header, footer or notes part an edit plan can address. Its paragraph
/// ids are `{id}` for the first and `{id}.p{index}` after it
/// ([`crate::ids::paragraph_id`]).
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

/// A content control (`w:sdt`) in the body, in document order. Controls
/// inside text boxes are not listed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ContentControl {
    /// `ccN` for a block control, `pN.ccK` for one in paragraph `pN`.
    pub id: String,
    /// `w:tag`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// `w:alias`, the title Word shows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// `text`, `rich_text`, `drop_down`, `combo_box`, `date`, `checkbox`,
    /// `picture`, `group`, `repeating`, `building_block`, `citation`,
    /// `bibliography`, `equation`, `unknown`. A control whose properties
    /// name no type is `rich_text`, as in Word.
    pub kind: String,
    /// Visible text of the control's content; paragraphs joined by `\n`.
    pub text: String,
    /// Paragraphs the control spans (`pN`); a run-level control lists
    /// the paragraph that holds it.
    pub paragraph_ids: Vec<String>,
    /// `w:lock` is `contentLocked` or `sdtContentLocked`.
    pub locked: bool,
    /// Drop-down and combo-box choices (`w:listItem/@w:value`, or the
    /// display text when an item has no value).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<String>,
    /// Checkbox state (`w14:checked`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    /// `w:showingPlcHdr` is set: the text is placeholder text.
    pub placeholder: bool,
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
///
/// # Errors
///
/// [`InspectError::Admission`] when the input is refused by admission;
/// [`InspectError::Package`] when the package cannot be read;
/// [`InspectError::MissingDocument`] when no main document part is found.
pub fn paragraphs(docx: &[u8]) -> Result<Vec<Paragraph>, InspectError> {
    let opened = Opened::open(docx)?;
    Ok(body_paragraphs(&opened.dom, opened.body))
}

/// Paragraph locations for a comparison rendered under its active budget.
/// One admitted open serves both the body and its related stories.
pub(crate) fn paragraphs_and_stories_within(
    docx: &[u8],
    limits: crate::admission::InputLimits,
) -> Result<(Vec<Paragraph>, Vec<Story>), InspectError> {
    let opened = Opened::open_within(docx, limits)?;
    Ok((
        body_paragraphs(&opened.dom, opened.body),
        opened.stories_within(limits, false)?,
    ))
}

/// Package facts, following OPC relationships for related parts.
///
/// # Errors
///
/// [`InspectError::Admission`] when the input is refused by admission;
/// [`InspectError::Package`] when a part cannot be read;
/// [`InspectError::Invalid`] for a malformed XML part or an invalid setting
/// value; [`InspectError::MissingDocument`] when no main document part is
/// found.
pub fn summary(docx: &[u8]) -> Result<Summary, InspectError> {
    let opened = Opened::open(docx)?;
    opened.summary()
}

/// Header, footer and note stories in a stable order (headers, footers,
/// footnotes, endnotes).
///
/// # Errors
///
/// [`InspectError::Admission`] when the input is refused by admission;
/// [`InspectError::Package`] when the package cannot be read;
/// [`InspectError::Invalid`] for a malformed part;
/// [`InspectError::MissingDocument`] when no main document part is found.
pub fn stories(docx: &[u8]) -> Result<Vec<Story>, InspectError> {
    Opened::open(docx)?.stories()
}

/// Content controls (`w:sdt`) in the body, in document order.
///
/// # Errors
///
/// [`InspectError::Admission`] when the input is refused by admission;
/// [`InspectError::Package`] when the package cannot be read;
/// [`InspectError::MissingDocument`] when no main document part is found.
pub fn controls(docx: &[u8]) -> Result<Vec<ContentControl>, InspectError> {
    let opened = Opened::open(docx)?;
    Ok(collect_controls(&opened.dom, opened.body))
}

/// Paragraphs prefixed with their ids and direct formatting as Markdown
/// marks: `[p12 Heading1] (a) **Confidentiality.** You will ...`.
/// Story paragraphs (`[header1] ...`, `[header1.p1] ...`) follow the body's.
pub fn markdown(docx: &[u8]) -> Result<String, InspectError> {
    let opened = Opened::open(docx)?;
    let mut all = body_paragraphs(&opened.dom, opened.body);
    for story in opened.stories()? {
        all.extend(story.paragraphs);
    }
    Ok(render_markdown(&all))
}

/// Body tables as grids: each cell's text and paragraph ids, the header
/// rows and the grid widths.
///
/// # Errors
///
/// [`InspectError::Admission`] when the input is refused by admission;
/// [`InspectError::Package`] when the package cannot be read;
/// [`InspectError::MissingDocument`] when no main document part is found.
pub fn tables(docx: &[u8]) -> Result<Vec<Table>, InspectError> {
    let opened = Opened::open(docx)?;
    Ok(tables::body_tables(&opened.dom, opened.body))
}

/// The full snapshot: `schema_version`, `source_sha256`, `summary`,
/// `paragraphs`, `stories`, `tables`, `controls`, `changes` and `comments`.
///
/// # Errors
///
/// [`InspectError::Admission`] when the input is refused by admission;
/// [`InspectError::Package`] when the package or comments part cannot be
/// read; [`InspectError::Invalid`] for a malformed part;
/// [`InspectError::MissingDocument`] when no main document part is found.
pub fn snapshot(docx: &[u8]) -> Result<Snapshot, InspectError> {
    // One open serves every field; the comment scan marks up the body's
    // DOM, so it runs last.
    let mut opened = Opened::open(docx)?;
    let summary = opened.summary()?;
    let paragraphs = body_paragraphs(&opened.dom, opened.body);
    let stories = opened.stories()?;
    let tables = tables::body_tables(&opened.dom, opened.body);
    let controls = collect_controls(&opened.dom, opened.body);
    let changes = crate::changes::logical_changes_in(&opened.pkg);
    let comments = crate::comments::comments_in(&mut opened).map_err(|e| match e {
        crate::comments::CommentError::Admission(refused) => InspectError::Admission(refused),
        crate::comments::CommentError::Package(m) => InspectError::Package(m),
    })?;
    Ok(Snapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        source_sha256: source_sha256(docx),
        track_changes: None,
        summary,
        paragraphs,
        stories,
        tables,
        controls,
        changes,
        comments,
    })
}

/// [`snapshot`] with every tracked change settled by `settle`, and
/// `track_changes` saying how. The paragraphs, cells and controls of the
/// body and of every story read the settled text and keep their ids: the
/// paragraphs, rows and cells a choice would remove stay, with the text it
/// leaves them (often none). `summary`, `changes` and `comments` describe
/// the file as stored, every revision still listed: the facts of what
/// `track_changes` settles.
///
/// # Errors
///
/// As [`snapshot`].
pub fn settled_snapshot(docx: &[u8], settle: Settle) -> Result<Snapshot, InspectError> {
    let mut snapshot = snapshot(docx)?;
    snapshot.track_changes = Some(settle);
    if settle == Settle::Reject {
        // The comment scan marks up the DOM it reads, so the rejected text
        // comes from a second open of its own.
        let mut opened = Opened::open(docx)?;
        reject_text(&mut opened.dom, opened.body);
        snapshot.paragraphs = body_paragraphs(&opened.dom, opened.body);
        snapshot.tables = tables::body_tables(&opened.dom, opened.body);
        snapshot.controls = collect_controls(&opened.dom, opened.body);
        snapshot.stories = opened.stories_within(crate::admission::InputLimits::default(), true)?;
    }
    Ok(snapshot)
}

/// [`tables()`], with every change rejected when `settle` says so, each
/// beside the number of its table in `read`'s view: the `tN` of the table
/// outside every cell that is it or holds it, as `read -p tN` picks it.
///
/// # Errors
///
/// As [`tables()`].
pub fn view_tables(
    docx: &[u8],
    settle: Option<Settle>,
) -> Result<Vec<(usize, Table)>, InspectError> {
    let mut opened = Opened::open(docx)?;
    if settle == Some(Settle::Reject) {
        reject_text(&mut opened.dom, opened.body);
    }
    // `body_tables` lists the tables the view numbers, each under its `tN`.
    Ok(tables::body_tables(&opened.dom, opened.body)
        .into_iter()
        .map(|table| (table.index, table))
        .collect())
}

/// Word's Reject All on the text under `root`, in place: inserted and
/// moved-to runs go, deleted and moved-from runs come back, their
/// `w:delText` read as `w:t`, and a formatting change gives back the
/// properties it replaced. Paragraph marks, rows and cells stay (their
/// own revisions hold no text), so every paragraph keeps its id.
fn reject_text(dom: &mut Dom, root: NodeId) {
    // A paragraph's old base properties are in its `w:pPrChange`; its
    // mark's `w:rPr` and its `w:sectPr` are not, and stay. A run's (or a
    // mark's) old properties are the whole `w:rPr` of its `w:rPrChange`.
    for (change, kept) in [
        ("pPrChange", &["rPr", "sectPr"][..]),
        ("rPrChange", &[][..]),
    ] {
        for node in dom.descendants(root, Some(&W::name(change))) {
            let Some(properties) = dom.parent(node) else {
                continue;
            };
            let old = dom
                .elements(node, None)
                .first()
                .map(|&old| {
                    let kids = dom.nodes(old);
                    dom.remove_nodes(old);
                    kids
                })
                .unwrap_or_default();
            let stay: Vec<NodeId> = dom
                .elements(properties, None)
                .into_iter()
                .filter(|&e| kept.iter().any(|k| dom.name_is(e, &W::name(k))))
                .collect();
            dom.remove_nodes(properties);
            for kid in old.into_iter().chain(stay) {
                dom.add(properties, kid);
            }
        }
    }
    for local in ["ins", "moveTo"] {
        for node in dom.descendants(root, Some(&W::name(local))) {
            dom.remove(node);
        }
    }
    for local in ["del", "moveFrom"] {
        for node in dom.descendants(root, Some(&W::name(local))) {
            let kids = dom.nodes(node);
            dom.remove_nodes(node);
            dom.replace_with(node, &kids);
        }
    }
    for (from, to) in [("delText", "t"), ("delInstrText", "instrText")] {
        for node in dom.descendants(root, Some(&W::name(from))) {
            dom.set_name(node, W::name(to));
        }
    }
}

/// [`snapshot`] as JSON.
///
/// # Errors
///
/// As [`snapshot`]; [`InspectError::Invalid`] when the snapshot cannot be
/// serialized.
pub fn inspect_json(docx: &[u8]) -> Result<String, InspectError> {
    serde_json::to_string(&snapshot(docx)?).map_err(|e| InspectError::Invalid(e.to_string()))
}

/// Tables as `read --tables` prints them: a `table N: RxC header_rows=H
/// widths=W` line per table, then a line per row of tab-separated
/// `ids=text` cells (`-` for a cell with no paragraph; `\`, newlines and
/// tabs escaped), or `no tables`. Cell ids are short paragraph ids (`p1`),
/// via [`crate::ids::short_paragraph`]; a cell of several paragraphs joins
/// them with commas (`p3,p4`), a spelling only this listing prints. Every line
/// ends with a newline. Nested tables follow their containing cell row,
/// indented two spaces per level and labelled `nested in CELL_PATH`.
pub fn tables_text(tables: &[Table]) -> String {
    let mut out = String::new();
    if tables.is_empty() {
        out.push_str("no tables\n");
    }
    for table in tables {
        render_grid(
            &mut out,
            &format!("table {}", table.index),
            &table.rows,
            table.header_rows,
            &table.widths_dxa,
            &table.nested,
            0,
        );
    }
    out
}

fn render_grid(
    out: &mut String,
    label: &str,
    rows: &[Vec<TableCell>],
    header_rows: usize,
    widths_dxa: &[u32],
    nested: &[NestedTable],
    depth: usize,
) {
    use std::fmt::Write as _;
    let indent = "  ".repeat(depth);
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    let widths: Vec<String> = widths_dxa.iter().map(u32::to_string).collect();
    let _ = writeln!(
        out,
        "{indent}{label}: {}x{columns} header_rows={header_rows} widths={}",
        rows.len(),
        widths.join(",")
    );
    for (r, row) in rows.iter().enumerate() {
        let cells: Vec<String> = row
            .iter()
            .map(|cell| {
                let ids = if cell.paragraph_ids.is_empty() {
                    "-".to_string()
                } else {
                    cell.paragraph_ids
                        .iter()
                        .map(|id| crate::ids::short_paragraph(id))
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let text = cell
                    .text
                    .replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('\t', "\\t");
                format!("{ids}={text}")
            })
            .collect();
        let _ = writeln!(out, "{indent}{}", cells.join("\t"));
        // The final row/cell segment is this table's own cell, even at
        // several nesting levels. Emit its children directly below the row.
        let row_prefix = format!("{r}.cell");
        for child in nested.iter().filter(|child| {
            child
                .parent
                .rsplit_once(".row")
                .is_some_and(|(_, tail)| tail.starts_with(&row_prefix))
        }) {
            render_grid(
                out,
                &format!("nested in {}", child.parent),
                &child.rows,
                child.header_rows,
                &child.widths_dxa,
                &child.nested,
                depth + 1,
            );
        }
    }
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

// Documents this thread has reopened through `Opened::open` since the last
// `reset_reopened_packages`. Tests that must not pay for a reparse of bytes
// they already hold reset the counter, run the phase under test, and read
// it back.
#[cfg(test)]
thread_local! {
    pub(crate) static REOPENED_PACKAGES: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

/// Add one to this thread's [`REOPENED_PACKAGES`].
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
fn bump_reopened_packages() {
    REOPENED_PACKAGES.with(|count| count.set(count.get() + 1));
}

/// Zero this thread's [`REOPENED_PACKAGES`].
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(crate) fn reset_reopened_packages() {
    REOPENED_PACKAGES.with(|count| count.set(0));
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
        Self::open_within(bytes, crate::admission::InputLimits::default())
    }

    fn open_within(
        bytes: &[u8],
        limits: crate::admission::InputLimits,
    ) -> Result<Self, InspectError> {
        #[cfg(test)]
        bump_reopened_packages();
        crate::admission::admit(bytes, limits).map_err(InspectError::Admission)?;
        let normalized =
            crate::strict_translation::strict_to_transitional_docx_within(bytes, limits)
                .map_err(|error| InspectError::Package(error.to_string()))?;
        let pkg = PartFs::open_within(&normalized, limits)
            .map_err(|error| InspectError::Package(error.to_string()))?;
        let main = main_part(&pkg)?;
        let (dom, document, root) = parse_part_within(&pkg, &main, limits)?;
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
            tables: tables::top_level_table_nodes(dom, body).len(),
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
            // The checked reader refuses a malformed WordprocessingML part;
            // custom XML items and vendor parts are not read, so it cannot
            // refuse the document over them.
            let wordprocessing = mime.is_some_and(|mime| {
                mime.starts_with("application/vnd.openxmlformats-officedocument.wordprocessingml.")
            });
            if wordprocessing && name != self.main {
                parse_part(pkg, &name)?;
            }
        }
        summary.revisions = crate::changes::logical_changes_in(pkg).len();
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
                                part_dom.attribute(style, &W::type_()) == Some("paragraph")
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
        for kind in STORY_KINDS.iter().copied() {
            let mut parts: Vec<String> = self.related(kind).into_iter().collect();
            parts.sort_by_key(|part| (part.len(), part.clone()));
            for part in parts {
                out.push((crate::ids::part_story(&part), kind, part));
            }
        }
        out
    }

    fn stories(&self) -> Result<Vec<Story>, InspectError> {
        self.stories_within(crate::admission::InputLimits::default(), false)
    }

    /// [`Self::stories`] under `limits`, every change rejected when
    /// `reject` is set.
    fn stories_within(
        &self,
        limits: crate::admission::InputLimits,
        reject: bool,
    ) -> Result<Vec<Story>, InspectError> {
        self.story_parts()
            .into_iter()
            .map(|(id, kind, part)| {
                let (mut dom, _, root) = parse_part_within(&self.pkg, &part, limits)?;
                if reject {
                    reject_text(&mut dom, root);
                }
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
    parse_part_within(pkg, name, crate::admission::InputLimits::default())
}

fn parse_part_within(
    pkg: &PartFs,
    name: &str,
    limits: crate::admission::InputLimits,
) -> Result<(Dom, NodeId, NodeId), InspectError> {
    let bytes = pkg
        .part_bytes(name)
        .ok_or_else(|| InspectError::Invalid(format!("missing related part {name}")))?;
    let xml = std::str::from_utf8(bytes)
        .map_err(|error| InspectError::Invalid(format!("{name}: {error}")))?;
    crate::xmllinq::parse::validate_xml_with_limits(xml, limits)
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

/// The two names a text box's content goes by: `w:txbxContent`, and the
/// `wne:txbxContent` a Strict package's shapes hold (the agent view skips
/// both by local name, so every id walk must too).
pub(crate) fn text_box_names() -> [XName; 2] {
    [W::txbx_content(), WNE::name("txbxContent")]
}

/// Whether `node` sits inside a text box.
pub(crate) fn in_text_box(dom: &Dom, node: NodeId) -> bool {
    text_box_names()
        .iter()
        .any(|name| has_ancestor(dom, node, name))
}

/// Whether `node` holds a text box.
fn holds_text_box(dom: &Dom, node: NodeId) -> bool {
    text_box_names()
        .iter()
        .any(|name| !dom.descendants(node, Some(name)).is_empty())
}

fn note_count(dom: &Dom, root: NodeId, local: &str) -> usize {
    dom.descendants(root, Some(&W::name(local)))
        .iter()
        .filter(|&&node| {
            !matches!(
                dom.attribute(node, &W::type_()),
                Some("separator" | "continuationSeparator")
            )
        })
        .count()
}

/// The body paragraphs (`w:p` outside text boxes) in document order.
pub(crate) fn body_paragraph_nodes(dom: &Dom, body: NodeId) -> Vec<NodeId> {
    dom.descendants(body, Some(&W::p()))
        .into_iter()
        .filter(|&p| !in_text_box(dom, p))
        .collect()
}

/// The body's content controls (`w:sdt` outside text boxes) in document
/// order, the order [`collect_controls`] numbers them in.
pub(crate) fn body_control_nodes(dom: &Dom, body: NodeId) -> Vec<NodeId> {
    dom.descendants(body, Some(&W::sdt()))
        .into_iter()
        .filter(|&sdt| !in_text_box(dom, sdt))
        .collect()
}

/// One record per [`body_control_nodes`] entry.
pub(crate) fn collect_controls(dom: &Dom, body: NodeId) -> Vec<ContentControl> {
    let paragraphs = body_paragraph_nodes(dom, body);
    let index: std::collections::HashMap<NodeId, usize> = paragraphs
        .iter()
        .enumerate()
        .map(|(i, &p)| (p, i))
        .collect();
    // A block control is `ccN`, counting the body's block controls; one in
    // a paragraph is `pN.ccK`, counting that paragraph's controls.
    let mut blocks = 0usize;
    let mut inline: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    body_control_nodes(dom, body)
        .into_iter()
        .map(|sdt| {
            let owner = dom
                .ancestors(sdt, Some(&W::p()))
                .first()
                .and_then(|p| index.get(p).copied());
            let id = match owner {
                Some(i) => {
                    let k = inline.entry(i).or_default();
                    *k += 1;
                    format!("{}.cc{}", crate::ids::paragraph_id("body", i), *k - 1)
                }
                None => {
                    blocks += 1;
                    format!("cc{}", blocks - 1)
                }
            };
            control_record(dom, sdt, id, &index)
        })
        .collect()
}

fn control_record(
    dom: &Dom,
    sdt: NodeId,
    id: String,
    index: &std::collections::HashMap<NodeId, usize>,
) -> ContentControl {
    let pr = dom.element(sdt, &W::sdt_pr());
    let child = |name: &XName| pr.and_then(|pr| dom.element(pr, name));
    let val = |name: &XName| {
        child(name)
            .and_then(|e| dom.attribute(e, &W::val()))
            .map(str::to_string)
    };
    let kind = pr.map_or("rich_text", |pr| control_kind(dom, pr));
    let list = child(&W::name("dropDownList")).or_else(|| child(&W::name("comboBox")));
    let choices = list
        .map(|list| {
            dom.elements(list, Some(&W::name("listItem")))
                .into_iter()
                .filter_map(|item| {
                    dom.attribute(item, &W::name("value"))
                        .or_else(|| dom.attribute(item, &W::name("displayText")))
                        .map(str::to_string)
                })
                .collect()
        })
        .unwrap_or_default();
    let checked = child(&W14::name("checkbox")).map(|checkbox| {
        dom.element(checkbox, &W14::name("checked"))
            .and_then(|c| dom.attribute(c, &W14::name("val")))
            .is_some_and(on_off)
    });
    let placeholder = child(&W::name("showingPlcHdr"))
        .is_some_and(|e| dom.attribute(e, &W::val()).is_none_or(on_off));
    let locked = matches!(
        val(&W::name("lock")).as_deref(),
        Some("contentLocked" | "sdtContentLocked")
    );
    let content = dom.element(sdt, &W::sdt_content());
    let owner = dom.ancestors(sdt, Some(&W::p())).first().copied();
    let (paragraph_ids, text) = match (owner, content) {
        (Some(p), _) => {
            let text = content.map_or_else(String::new, |c| project_container(dom, c).text);
            (index.get(&p).map(|&i| vec![i]).unwrap_or_default(), text)
        }
        (None, Some(c)) => {
            let inner: Vec<(usize, NodeId)> = dom
                .descendants(c, Some(&W::p()))
                .into_iter()
                .filter_map(|p| index.get(&p).map(|&i| (i, p)))
                .collect();
            let text = inner
                .iter()
                .map(|&(_, p)| project_paragraph(dom, p).text)
                .collect::<Vec<_>>()
                .join("\n");
            (inner.into_iter().map(|(i, _)| i).collect(), text)
        }
        (None, None) => (Vec::new(), String::new()),
    };
    ContentControl {
        id,
        tag: val(&W::name("tag")),
        alias: val(&W::name("alias")),
        kind: kind.to_string(),
        text,
        paragraph_ids: paragraph_ids
            .into_iter()
            .map(|i| crate::ids::paragraph_id("body", i))
            .collect(),
        locked,
        choices,
        checked,
        placeholder,
    }
}

/// `1`, `true` and `on` are set; anything else is clear.
fn on_off(value: &str) -> bool {
    matches!(value, "1" | "true" | "on")
}

/// `w:sdtPr` children that are properties, not the control's type.
const SDT_PROPERTIES: &[&str] = &[
    "rPr",
    "alias",
    "tag",
    "id",
    "lock",
    "placeholder",
    "temporary",
    "showingPlcHdr",
    "dataBinding",
    "label",
    "tabIndex",
];

/// The control's type from its `w:sdtPr` (see [`ContentControl::kind`]).
fn control_kind(dom: &Dom, pr: NodeId) -> &'static str {
    for child in dom.elements(pr, None) {
        let Some(name) = dom.name(child) else {
            continue;
        };
        let (ns, local) = (name.namespace_name(), name.local_name());
        let kind = if ns == W::URI {
            match local {
                "text" => "text",
                "richText" => "rich_text",
                "dropDownList" => "drop_down",
                "comboBox" => "combo_box",
                "date" => "date",
                "picture" => "picture",
                "group" => "group",
                "docPartObj" | "docPartList" => "building_block",
                "citation" => "citation",
                "bibliography" => "bibliography",
                "equation" => "equation",
                local if SDT_PROPERTIES.contains(&local) => continue,
                _ => "unknown",
            }
        } else if ns == W14::URI {
            match local {
                "checkbox" => "checkbox",
                _ => continue,
            }
        } else if ns == W15::URI {
            match local {
                "repeatingSection" | "repeatingSectionItem" => "repeating",
                _ => continue,
            }
        } else {
            "unknown"
        };
        return kind;
    }
    "rich_text"
}

/// A header, footer or notes part's paragraphs, separator notes left out.
pub(crate) fn story_paragraph_nodes(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    body_paragraph_nodes(dom, root)
        .into_iter()
        .filter(|&p| {
            !dom.ancestors(p, None).into_iter().any(|a| {
                (dom.name_is(a, &W::footnote()) || dom.name_is(a, &W::endnote()))
                    && matches!(
                        dom.attribute(a, &W::type_()),
                        Some("separator" | "continuationSeparator" | "continuationNotice")
                    )
            })
        })
        .collect()
}

fn body_paragraphs(dom: &Dom, body: NodeId) -> Vec<Paragraph> {
    paragraphs_of(dom, body_paragraph_nodes(dom, body), BODY_STORY)
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
                id: crate::ids::paragraph_id(story, index),
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

/// Project the inline content of any container (a run-level `w:sdtContent`)
/// with the paragraph rules.
pub(crate) fn project_container(dom: &Dom, container: NodeId) -> Projection {
    let mut projection = Projection::default();
    let mut formats: Vec<(usize, usize, Format)> = Vec::new();
    let mut fields = 0;
    walk_container(
        dom,
        container,
        false,
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
        // Word's text boxes and shapes sit in a run as `mc:AlternateContent`:
        // the same construct `walk_container` names.
        if name.namespace_name() == MC::URI {
            out.limitations.insert("alternate_content".into());
            if ["drawing", "pict", "object"]
                .iter()
                .any(|local| !dom.descendants(child, Some(&W::name(local))).is_empty())
            {
                out.limitations.insert("drawing".into());
            }
            if holds_text_box(dom, child) {
                out.limitations.insert("text_box_omitted".into());
            }
            continue;
        }
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
            "br" => match dom.attribute(child, &W::type_()) {
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
                if holds_text_box(dom, child) {
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
        let md = render_markdown(&[p("p0", "Scope", Some("Heading1")), p("p1", "", None)]);
        assert_eq!(md, "[p0 Heading1] Scope\n\n[p1]\n");
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
