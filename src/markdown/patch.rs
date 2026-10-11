// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A Markdown edit of a Word document, applied to the document itself.
//!
//! The Markdown is read as the document's text after an edit: however it
//! was made (pandoc, `jubarte text`, a converter, an agent), its blocks are
//! aligned with the body's paragraphs by their text, and only what differs
//! is edited, through an [edit plan](crate::edit): a paragraph whose text
//! changed is rewritten word by word, so its runs, formatting, fields and
//! properties stay; a block the document lacks becomes a paragraph next to
//! its neighbours, taking the properties of a neighbour of its kind; a
//! paragraph the Markdown lacks is deleted. Empty paragraphs, which Markdown
//! cannot hold, are left alone, and typed numbering (`1.`, `(a)`) that the
//! Markdown writes as a list marker is kept.

use std::collections::HashSet;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use similar::{Algorithm, DiffTag, capture_diff_slices};

use super::MarkdownError;
use super::diff::similarity;
use crate::edit::{
    EditPlan, ExistingRevisions, Operation, OperationKind, RunSpec, Selector, Side, apply_plan,
};

/// Paragraphs that share less than this share of their words pair only when
/// nothing better is left between two pairs.
const PAIR_SIMILARITY: f64 = 0.3;

/// Pairing compares every paragraph of a changed stretch with every block
/// of it; above this many comparisons they pair by position.
const MAX_PAIRINGS: usize = 2_500;

/// A document edited as Markdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patched {
    /// The edited `.docx`, without tracked changes.
    pub docx: Vec<u8>,
    /// What the Markdown changed that the document could not take.
    pub warnings: Vec<String>,
}

/// The kind of a block or paragraph, for pairing and for the properties a
/// new paragraph copies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Heading(u8),
    List,
    Cell,
    Plain,
}

/// A Markdown block: its kind, its plain text and its runs.
#[derive(Clone, Debug)]
struct Block {
    kind: Kind,
    runs: Vec<RunSpec>,
}

impl Block {
    fn text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }
}

/// A body paragraph: its index in the body, text and kind.
#[derive(Clone, Debug)]
struct Paragraph {
    index: usize,
    text: String,
    kind: Kind,
}

/// `docx` edited to read as `markdown`. A document with tracked changes is
/// read with them accepted.
///
/// ```
/// use jubarte::markdown::{DocxOptions, apply_markdown, markdown_to_docx};
///
/// let original = markdown_to_docx("Pay in 30 days.\n\nSigned.\n", &DocxOptions::default())
///     .unwrap()
///     .docx;
/// let edited = apply_markdown(&original, "Pay in 45 days.\n\nSigned.\n").unwrap();
/// assert!(edited.warnings.is_empty());
/// ```
pub fn apply_markdown(docx: &[u8], markdown: &str) -> Result<Patched, MarkdownError> {
    let invalid = |e: &dyn std::fmt::Display| MarkdownError::Package(e.to_string());
    let summary = crate::inspect::summary(docx).map_err(|e| invalid(&e))?;
    // A plan that accepts existing revisions addresses the accepted text.
    let base = if summary.revisions > 0 {
        crate::document_comparer::accept_revisions(docx).map_err(|e| invalid(&e))?
    } else {
        docx.to_vec()
    };
    let paragraphs: Vec<Paragraph> = crate::inspect::paragraphs(&base)
        .map_err(|e| invalid(&e))?
        .into_iter()
        .filter(|p| !p.text.trim().is_empty())
        .map(|p| Paragraph {
            index: p.index,
            kind: match p.style.as_deref().and_then(heading_level) {
                Some(level) => Kind::Heading(level),
                None if p.in_table => Kind::Cell,
                None if p.numbered => Kind::List,
                None => Kind::Plain,
            },
            text: p.text,
        })
        .collect();
    let (blocks, notes) = blocks(markdown);
    let mut warnings = Vec::new();
    if notes {
        warnings.push("footnote text is not applied: only the body is edited".to_string());
    }
    let edits = align(&paragraphs, &blocks);
    let styles = paragraph_styles(&base);
    for edit in &edits {
        if let Edit::Insert(b, _) = edit
            && blocks[*b].kind == Kind::Cell
        {
            warnings.push(format!(
                "table cell {:?} not applied: rows and columns are not added",
                blocks[*b].text()
            ));
        }
    }

    // Rewrites the engine refuses (text inside a link or a field) become
    // whole-paragraph replacements, and what still fails is left out.
    let mut whole: HashSet<usize> = HashSet::new();
    let mut dropped: HashSet<usize> = HashSet::new();
    for _ in 0..4 {
        let operations = operations(&paragraphs, &blocks, &edits, &styles, &whole, &dropped);
        if operations.is_empty() {
            return Ok(Patched {
                docx: base,
                warnings,
            });
        }
        let plan = EditPlan {
            schema_version: crate::inspect::SCHEMA_VERSION,
            source_sha256: None,
            author: "jubarte".to_string(),
            date: None,
            initials: None,
            existing_revisions: ExistingRevisions::Accept,
            resolve_revisions: None,
            operations,
            update_fields: false,
        };
        match apply_plan(docx, &plan) {
            Ok(result) => {
                return Ok(Patched {
                    docx: result.clean,
                    warnings,
                });
            }
            Err(error) => {
                // Each failed edit once, with the first reason given.
                let failed: std::collections::BTreeMap<usize, String> = error
                    .outcomes
                    .iter()
                    .filter(|o| o.status == "failed")
                    .chain(
                        // A conflict names its operation but leaves it "ok".
                        error
                            .operation
                            .iter()
                            .filter_map(|id| error.outcomes.iter().find(|o| &o.id == id)),
                    )
                    .filter_map(|o| {
                        let entry = o.id.split(':').nth(1)?.parse::<usize>().ok()?;
                        let why = o.message.clone().unwrap_or_else(|| error.message.clone());
                        Some((entry, why))
                    })
                    .rev()
                    .collect();
                if failed.is_empty() {
                    return Err(MarkdownError::Package(error.to_string()));
                }
                for (entry, why) in failed {
                    let rewrite = matches!(edits[entry], Edit::Pair(..));
                    if rewrite && whole.insert(entry) {
                        continue;
                    }
                    if dropped.insert(entry) {
                        warnings.push(format!(
                            "{} not applied: {why}",
                            describe(&edits[entry], &paragraphs, &blocks)
                        ));
                    }
                }
            }
        }
    }
    Err(MarkdownError::Package(
        "the Markdown's changes could not be applied".to_string(),
    ))
}

/// The paragraph styles a document defines, by lowercase name.
fn paragraph_styles(docx: &[u8]) -> HashSet<String> {
    let Ok(package) = crate::opc::PartFs::open(docx) else {
        return HashSet::new();
    };
    let Some(xml) = package
        .main_document_part()
        .and_then(|main| {
            let rels = package.read_rels_for(&main)?;
            let rel = rels
                .items
                .iter()
                .find(|r| r.rel_type.ends_with("/styles"))?;
            Some(package.resolve_rel_target(&main, &rel.target))
        })
        .and_then(|part| package.part_string(&part))
    else {
        return HashSet::new();
    };
    let mut dom = crate::xmllinq::Dom::new();
    let document = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(document) else {
        return HashSet::new();
    };
    use crate::namespaces::W;
    dom.elements(root, Some(&W::name("style")))
        .into_iter()
        .filter(|&style| dom.attribute(style, &W::name("type")) == Some("paragraph"))
        .filter_map(|style| {
            let name = dom.element(style, &W::name("name"))?;
            Some(dom.attribute(name, &W::val())?.to_lowercase())
        })
        .collect()
}

/// One step of the alignment of paragraphs (by position in `paragraphs`)
/// and blocks (by position in `blocks`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edit {
    /// A paragraph and the block it became; rewritten when they differ.
    Pair(usize, usize),
    /// A block the document lacks, inserted after paragraph `after` (or
    /// before the first when `None`).
    Insert(usize, Option<usize>),
    /// A paragraph the Markdown lacks.
    Delete(usize),
}

fn describe(edit: &Edit, paragraphs: &[Paragraph], blocks: &[Block]) -> String {
    let excerpt = |text: &str| {
        let short: String = text.chars().take(40).collect();
        if short.len() < text.len() {
            format!("\"{short}…\"")
        } else {
            format!("\"{short}\"")
        }
    };
    match *edit {
        Edit::Pair(p, b) => format!(
            "rewriting paragraph {} as {}",
            paragraphs[p].index,
            excerpt(&blocks[b].text())
        ),
        Edit::Insert(b, _) => format!("inserting {}", excerpt(&blocks[b].text())),
        Edit::Delete(p) => format!(
            "deleting paragraph {} {}",
            paragraphs[p].index,
            excerpt(&paragraphs[p].text)
        ),
    }
}

/// Text compared for alignment: whitespace collapsed, symbols and leading
/// typed numbering left out.
fn key(text: &str) -> String {
    let spaced = text.replace('\u{FFFC}', " ");
    let collapsed = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed[enumerator_len(&collapsed)..].to_string()
}

/// The length of typed numbering at the start of `text` with the whitespace
/// after it: `1.`, `1.2`, `(a)`, `b)`, `iv.`, `A.`. Zero when there is none.
fn enumerator_len(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut at = 0;
    let open = bytes.first() == Some(&b'(');
    if open {
        at += 1;
    }
    let body_start = at;
    let digits = |from: usize| {
        bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let mut inner_dot = false;
    if digits(at) > 0 {
        at += digits(at);
        while bytes.get(at) == Some(&b'.') && digits(at + 1) > 0 {
            inner_dot = true;
            at += 1 + digits(at + 1);
        }
    } else {
        let letters = bytes[at..]
            .iter()
            .take_while(|b| b.is_ascii_alphabetic())
            .count();
        let roman = bytes[at..at + letters]
            .iter()
            .all(|b| b"ivxlcdmIVXLCDM".contains(b));
        if letters == 1 || (roman && (1..=6).contains(&letters)) {
            at += letters;
        } else {
            return 0;
        }
    }
    if at == body_start || at > 12 {
        return 0;
    }
    let close = match bytes.get(at) {
        Some(b')') => {
            at += 1;
            true
        }
        Some(b'.') if !open => {
            at += 1;
            true
        }
        _ => false,
    };
    if !(close || (inner_dot && !open)) || (open && bytes.get(at - 1) != Some(&b')')) {
        return 0;
    }
    let space = bytes[at..]
        .iter()
        .take_while(|b| b.is_ascii_whitespace())
        .count();
    if space == 0 || at + space >= bytes.len() {
        return 0;
    }
    at + space
}

fn heading_level(style: &str) -> Option<u8> {
    let level = style.strip_prefix("Heading")?.parse::<u8>().ok()?;
    (1..=9).contains(&level).then_some(level)
}

/// Whether a paragraph and a block can be the same: cells pair with cells.
fn compatible(paragraph: &Paragraph, block: &Block) -> bool {
    (paragraph.kind == Kind::Cell) == (block.kind == Kind::Cell)
}

/// Aligns paragraphs and blocks: equal text first, then, in each stretch
/// that differs, the most similar compatible pairs in order, then leftovers
/// by position.
fn align(paragraphs: &[Paragraph], blocks: &[Block]) -> Vec<Edit> {
    let old_keys: Vec<String> = paragraphs.iter().map(|p| key(&p.text)).collect();
    let new_keys: Vec<String> = blocks.iter().map(|b| key(&b.text())).collect();
    let mut edits = Vec::new();
    let mut last: Option<usize> = None;
    for op in capture_diff_slices(Algorithm::Patience, &old_keys, &new_keys) {
        let (tag, olds, news) = op.as_tag_tuple();
        if tag == DiffTag::Equal {
            for (p, b) in olds.zip(news) {
                edits.push(Edit::Pair(p, b));
                last = Some(p);
            }
            continue;
        }
        let olds: Vec<usize> = olds.collect();
        let news: Vec<usize> = news.collect();
        let matched = pair(&olds, &news, |p, b| {
            if !compatible(&paragraphs[p], &blocks[b]) {
                return None;
            }
            let s = similarity(&old_keys[p], &new_keys[b]);
            (s >= PAIR_SIMILARITY).then_some(s)
        });
        let (mut i, mut j) = (0, 0);
        for (mi, mj) in matched
            .into_iter()
            .chain(std::iter::once((olds.len(), news.len())))
        {
            while i < mi && j < mj && compatible(&paragraphs[olds[i]], &blocks[news[j]]) {
                edits.push(Edit::Pair(olds[i], news[j]));
                last = Some(olds[i]);
                i += 1;
                j += 1;
            }
            for &p in &olds[i..mi] {
                edits.push(Edit::Delete(p));
            }
            for &b in &news[j..mj] {
                edits.push(Edit::Insert(b, last));
            }
            if mi < olds.len() && mj < news.len() {
                edits.push(Edit::Pair(olds[mi], news[mj]));
                last = Some(olds[mi]);
            }
            i = mi + 1;
            j = mj + 1;
        }
    }
    edits
}

/// Weighted longest common subsequence of `olds` and `news` under `score`.
fn pair(
    olds: &[usize],
    news: &[usize],
    score: impl Fn(usize, usize) -> Option<f64>,
) -> Vec<(usize, usize)> {
    let (k, m) = (olds.len(), news.len());
    if k == 0 || m == 0 || k * m > MAX_PAIRINGS {
        return Vec::new();
    }
    let scores: Vec<Vec<Option<f64>>> = olds
        .iter()
        .map(|&p| news.iter().map(|&b| score(p, b)).collect())
        .collect();
    let mut best = vec![vec![0.0f64; m + 1]; k + 1];
    for i in (0..k).rev() {
        for j in (0..m).rev() {
            let take = scores[i][j].map_or(f64::MIN, |s| s + best[i + 1][j + 1]);
            best[i][j] = take.max(best[i + 1][j]).max(best[i][j + 1]);
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < k && j < m {
        if let Some(s) = scores[i][j]
            && (best[i][j] - (s + best[i + 1][j + 1])).abs() < 1e-9
        {
            out.push((i, j));
            i += 1;
            j += 1;
        } else if (best[i][j] - best[i + 1][j]).abs() < 1e-9 {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

/// The paragraph's new text: the block's, after the paragraph's own typed
/// numbering when it has some.
fn rewritten(paragraph: &Paragraph, block: &Block) -> String {
    let text = block.text();
    let own = enumerator_len(&paragraph.text);
    if own == 0 {
        return text;
    }
    let theirs = enumerator_len(&text);
    format!("{}{}", &paragraph.text[..own], &text[theirs..])
}

/// The plan's operations. Operation ids are `edit:N`, N the edit's index.
fn operations(
    paragraphs: &[Paragraph],
    blocks: &[Block],
    edits: &[Edit],
    styles: &HashSet<String>,
    whole: &HashSet<usize>,
    dropped: &HashSet<usize>,
) -> Vec<Operation> {
    let at = |p: usize| Selector::Index {
        index: paragraphs[p].index,
        story: None,
    };
    // Paragraphs the plan deletes: a new paragraph cannot anchor on them.
    let gone: HashSet<usize> = edits
        .iter()
        .enumerate()
        .filter(|(entry, _)| !dropped.contains(entry))
        .filter_map(|(entry, edit)| match *edit {
            Edit::Delete(p) if paragraphs[p].kind != Kind::Cell => Some(p),
            Edit::Pair(p, _) if whole.contains(&entry) => Some(p),
            _ => None,
        })
        .collect();
    let stays = |p: &usize| !gone.contains(p);
    // A paragraph for `block` between paragraph positions `after` and
    // `next`: placed next to a staying neighbour, with the properties of the
    // nearest staying paragraph of the block's kind.
    let insert = |block: &Block, after: Option<usize>, next: Option<usize>| {
        let after = after
            .filter(stays)
            .or_else(|| after.and_then(|a| (0..a).rev().find(|p| stays(p))));
        let next = next
            .filter(stays)
            .or_else(|| next.and_then(|n| (n + 1..paragraphs.len()).find(|p| stays(p))));
        let (anchor, side) = match (after, next) {
            (Some(a), _) => (a, Side::After),
            (None, Some(n)) => (n, Side::Before),
            (None, None) => return None,
        };
        let nearest = |fits: &dyn Fn(Kind) -> bool| {
            (0..paragraphs.len())
                .filter(|p| stays(p) && fits(paragraphs[*p].kind))
                .min_by_key(|p| p.abs_diff(anchor))
        };
        let same = nearest(&|kind| kind == block.kind);
        let (like, style) = match (same, block.kind) {
            (Some(like), _) => (like, None),
            (None, Kind::Heading(level)) => {
                let name = format!("heading {level}");
                let style = styles.contains(&name).then_some(name);
                let like = nearest(&|kind| matches!(kind, Kind::Heading(_)))
                    .filter(|_| style.is_some())
                    .or_else(|| nearest(&|kind| kind == Kind::Plain))
                    .unwrap_or(anchor);
                (like, style)
            }
            (None, _) => (nearest(&|kind| kind == Kind::Plain).unwrap_or(anchor), None),
        };
        Some(OperationKind::InsertParagraph {
            paragraph: at(anchor),
            position: side,
            runs: block.runs.clone(),
            like: (like != anchor).then(|| at(like)),
            style,
            comment: None,
        })
    };
    let mut operations = Vec::new();
    for (entry, edit) in edits.iter().enumerate() {
        if dropped.contains(&entry) {
            continue;
        }
        let id = Some(format!("edit:{entry}"));
        match *edit {
            Edit::Pair(p, b) => {
                let paragraph = &paragraphs[p];
                let block = &blocks[b];
                if key(&paragraph.text) == key(&block.text()) {
                    continue;
                }
                if !whole.contains(&entry) {
                    operations.push(Operation {
                        id,
                        kind: OperationKind::Rewrite {
                            paragraph: at(p),
                            text: rewritten(paragraph, block),
                        },
                    });
                    continue;
                }
                // The paragraph replaced whole: the new one next to a
                // neighbour, then the old one deleted.
                let replaced = Block {
                    kind: paragraph.kind,
                    runs: block.runs.clone(),
                };
                let before = p.checked_sub(1);
                let after = (p + 1 < paragraphs.len()).then_some(p + 1);
                let Some(kind) = insert(&replaced, before, after) else {
                    continue;
                };
                operations.push(Operation {
                    id: Some(format!("edit:{entry}:new")),
                    kind,
                });
                operations.push(Operation {
                    id,
                    kind: OperationKind::DeleteParagraph {
                        paragraph: at(p),
                        comment: None,
                    },
                });
            }
            Edit::Delete(p) => {
                let kind = if paragraphs[p].kind == Kind::Cell {
                    // A cell keeps its paragraph; its text goes.
                    OperationKind::Rewrite {
                        paragraph: at(p),
                        text: String::new(),
                    }
                } else {
                    OperationKind::DeleteParagraph {
                        paragraph: at(p),
                        comment: None,
                    }
                };
                operations.push(Operation { id, kind });
            }
            Edit::Insert(b, after) => {
                let block = &blocks[b];
                if block.kind == Kind::Cell {
                    continue;
                }
                let next = edits[entry..].iter().find_map(|e| match e {
                    Edit::Pair(p, _) | Edit::Delete(p) => Some(*p),
                    Edit::Insert(..) => None,
                });
                if let Some(kind) = insert(block, after, next) {
                    operations.push(Operation { id, kind });
                }
            }
        }
    }
    operations
}

/// The Markdown's blocks in order, and whether it has footnotes.
fn blocks(markdown: &str) -> (Vec<Block>, bool) {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
    let mut out: Vec<Block> = Vec::new();
    let mut open: Option<Block> = None;
    let mut notes = false;
    let mut skip = 0u32;
    let mut lists = 0u32;
    let mut heading: Option<u8> = None;
    let mut cell = false;
    let mut code = false;
    let (mut bold, mut italic) = (0u32, 0u32);
    let close = |open: &mut Option<Block>, out: &mut Vec<Block>| {
        if let Some(block) = open.take()
            && !block.text().trim().is_empty()
        {
            out.push(block);
        }
    };
    for event in Parser::new_ext(markdown, options) {
        if skip > 0 {
            match event {
                Event::Start(
                    Tag::FootnoteDefinition(_) | Tag::MetadataBlock(_) | Tag::Image { .. },
                ) => {
                    skip += 1;
                }
                Event::End(
                    TagEnd::FootnoteDefinition | TagEnd::MetadataBlock(_) | TagEnd::Image,
                ) => {
                    skip -= 1;
                }
                _ => {}
            }
            continue;
        }
        let kind = || match heading {
            Some(level) => Kind::Heading(level),
            None if cell => Kind::Cell,
            None if lists > 0 => Kind::List,
            None => Kind::Plain,
        };
        match event {
            Event::Start(Tag::FootnoteDefinition(_)) => {
                close(&mut open, &mut out);
                notes = true;
                skip = 1;
            }
            Event::Start(Tag::MetadataBlock(_) | Tag::Image { .. }) => skip = 1,
            Event::Start(Tag::Heading { level, .. }) => {
                close(&mut open, &mut out);
                heading = Some(match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                });
            }
            Event::End(TagEnd::Heading(_)) => {
                close(&mut open, &mut out);
                heading = None;
            }
            Event::Start(Tag::List(_)) => {
                close(&mut open, &mut out);
                lists += 1;
            }
            Event::End(TagEnd::List(_)) => {
                close(&mut open, &mut out);
                lists = lists.saturating_sub(1);
            }
            Event::Start(Tag::TableCell) => {
                close(&mut open, &mut out);
                cell = true;
                open = Some(Block {
                    kind: Kind::Cell,
                    runs: Vec::new(),
                });
            }
            Event::End(TagEnd::TableCell) => {
                close(&mut open, &mut out);
                cell = false;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                close(&mut open, &mut out);
                code = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                close(&mut open, &mut out);
                code = false;
            }
            Event::Start(
                Tag::Paragraph | Tag::Item | Tag::BlockQuote(_) | Tag::Table(_) | Tag::HtmlBlock,
            )
            | Event::End(
                TagEnd::Paragraph
                | TagEnd::Item
                | TagEnd::BlockQuote(_)
                | TagEnd::Table
                | TagEnd::HtmlBlock,
            )
            | Event::Rule => close(&mut open, &mut out),
            Event::Start(Tag::Strong) => bold += 1,
            Event::End(TagEnd::Strong) => bold = bold.saturating_sub(1),
            Event::Start(Tag::Emphasis) => italic += 1,
            Event::End(TagEnd::Emphasis) => italic = italic.saturating_sub(1),
            Event::Text(text) | Event::Code(text) => {
                let pieces: Vec<&str> = if code {
                    text.split('\n').collect()
                } else {
                    vec![&text]
                };
                for (n, piece) in pieces.iter().enumerate() {
                    if code && n > 0 {
                        close(&mut open, &mut out);
                    }
                    let block = open.get_or_insert_with(|| Block {
                        kind: kind(),
                        runs: Vec::new(),
                    });
                    push_run(
                        block,
                        &piece.replace(['\t', '\r'], " "),
                        bold > 0,
                        italic > 0,
                    );
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(block) = &mut open {
                    push_run(block, " ", bold > 0, italic > 0);
                }
            }
            Event::InlineHtml(html) if html.to_ascii_lowercase().starts_with("<br") => {
                if let Some(block) = &mut open {
                    push_run(block, " ", bold > 0, italic > 0);
                }
            }
            _ => {}
        }
    }
    close(&mut open, &mut out);
    (out, notes)
}

/// Appends text to the block, joining the last run when its formatting is
/// the same.
fn push_run(block: &mut Block, text: &str, bold: bool, italic: bool) {
    if text.is_empty() {
        return;
    }
    let (bold, italic) = (bold.then_some(true), italic.then_some(true));
    match block.runs.last_mut() {
        Some(last) if last.bold == bold && last.italic == italic => last.text.push_str(text),
        _ => block.runs.push(RunSpec {
            text: text.to_string(),
            bold,
            italic,
            ..RunSpec::default()
        }),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn typed_numbering_is_found() {
        for (text, number) in [
            ("1. Definitions", "1. "),
            ("1.\tDefinitions", "1.\t"),
            ("1.2 Scope", "1.2 "),
            ("(a) the Buyer", "(a) "),
            ("b) the Seller", "b) "),
            ("iv. Remedies", "iv. "),
            ("IV. Remedies", "IV. "),
            ("A. Parties", "A. "),
            ("A cat sat.", ""),
            ("I am here.", ""),
            ("2024 was a year.", ""),
            ("(see below) text", ""),
            ("1.", ""),
            ("Intro", ""),
        ] {
            assert_eq!(&text[..enumerator_len(text)], number, "{text}");
        }
    }

    #[test]
    fn keys_ignore_numbering_spacing_and_symbols() {
        assert_eq!(key("1.\tDefinitions  apply"), "Definitions apply");
        assert_eq!(key("x \u{FFFC} y"), "x y");
    }

    #[test]
    fn blocks_have_kinds_and_runs() {
        let (blocks, notes) = blocks(
            "# Title\n\nSome **bold** text.[^1]\n\n- item\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n```\nx\ny\n```\n\n![alt](i.png)\n\n[^1]: note\n",
        );
        let summary: Vec<(Kind, String)> = blocks.iter().map(|b| (b.kind, b.text())).collect();
        assert_eq!(
            summary,
            [
                (Kind::Heading(1), "Title".to_string()),
                (Kind::Plain, "Some bold text.".to_string()),
                (Kind::List, "item".to_string()),
                (Kind::Cell, "a".to_string()),
                (Kind::Cell, "b".to_string()),
                (Kind::Cell, "1".to_string()),
                (Kind::Cell, "2".to_string()),
                (Kind::Plain, "x".to_string()),
                (Kind::Plain, "y".to_string()),
            ]
        );
        assert_eq!(blocks[1].runs[1].bold, Some(true));
        assert!(notes);
    }

    #[test]
    fn a_rewrite_keeps_the_paragraphs_own_numbering() {
        let paragraph = Paragraph {
            index: 0,
            text: "3.\tPayment terms".to_string(),
            kind: Kind::Plain,
        };
        let block = |text: &str| Block {
            kind: Kind::List,
            runs: vec![RunSpec {
                text: text.to_string(),
                ..RunSpec::default()
            }],
        };
        assert_eq!(
            rewritten(&paragraph, &block("Payment and terms")),
            "3.\tPayment and terms"
        );
        assert_eq!(rewritten(&paragraph, &block("4. Payment")), "3.\tPayment");
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod markdown_pairing_boundary_tests {
    use super::*;

    fn block(text: &str, kind: Kind) -> Block {
        Block {
            kind,
            runs: vec![RunSpec {
                text: text.to_string(),
                ..RunSpec::default()
            }],
        }
    }

    fn edit_view(edits: &[Edit]) -> Vec<(char, usize, Option<usize>)> {
        edits
            .iter()
            .map(|e| match e {
                Edit::Pair(p, b) => ('P', *p, Some(*b)),
                Edit::Insert(b, after) => ('I', *b, *after),
                Edit::Delete(p) => ('D', *p, None),
            })
            .collect()
    }

    #[test]
    fn ordered_pairing_boundaries_keep_every_authored_index_once() {
        let a = (0..50)
            .map(|i| Paragraph {
                index: i,
                text: format!("old-{i}"),
                kind: Kind::Plain,
            })
            .collect::<Vec<_>>();
        for count in [0, 49, 50, 51] {
            let b = (0..count)
                .map(|i| block(&format!("new-{i}"), Kind::Plain))
                .collect::<Vec<_>>();
            let edits = align(&a, &b);
            let mut expected = (0..count.min(50))
                .map(|i| ('P', i, Some(i)))
                .collect::<Vec<_>>();
            if count < 50 {
                expected.extend((count..50).map(|i| ('D', i, None)));
            }
            if count > 50 {
                expected.push(('I', 50, Some(49)));
            }
            assert_eq!(edit_view(&edits), expected, "count {count}");
            assert_eq!(
                a.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
                (0..50)
                    .map(|i| format!("old-{i}"))
                    .collect::<Vec<_>>()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(
            edit_view(&align(&[], &[block("new", Kind::Plain)])),
            vec![('I', 0, None)]
        );
        let paragraphs = vec![
            Paragraph {
                index: 0,
                text: "old cell".to_string(),
                kind: Kind::Cell,
            },
            Paragraph {
                index: 1,
                text: "anchor".to_string(),
                kind: Kind::Plain,
            },
        ];
        assert_eq!(
            edit_view(&align(
                &paragraphs,
                &[block("new body", Kind::Plain), block("anchor", Kind::Plain)]
            )),
            vec![('D', 0, None), ('I', 0, None), ('P', 1, Some(1))]
        );
    }

    #[test]
    fn weighted_pairing_skips_weaker_conflicts_without_crossing_source_owners() {
        let indices = [0, 1, 2];
        let score = |a: usize, b: usize| match (a, b) {
            (0, 0) => Some(0.4),
            (0, 1) => Some(0.9),
            (1, 1) => Some(0.2),
            (2, 2) => Some(1.0),
            _ => None,
        };
        assert_eq!(pair(&indices, &indices, score), vec![(0, 1), (2, 2)]);
        assert_eq!(
            pair(&indices, &indices, |a, b| if a == 1 && b == 0 {
                Some(1.0)
            } else {
                None
            }),
            vec![(1, 0)]
        );
        assert_eq!(
            pair(&indices, &indices, |_, _| None),
            Vec::<(usize, usize)>::new()
        );
        assert_eq!(pair(&[], &indices, score), Vec::<(usize, usize)>::new());
        assert_eq!(pair(&indices, &[], score), Vec::<(usize, usize)>::new());
    }

    #[test]
    fn malformed_numbering_remains_literal_and_normalized_line_breaks_keep_run_formatting() {
        for text in [
            "1234567890123. text",
            "1.x",
            "1. ",
            "(a. text",
            "(1.2 text",
            "vvvvvvv. text",
        ] {
            assert_eq!(enumerator_len(text), 0, "{text}");
            let p = Paragraph {
                index: 0,
                text: text.to_string(),
                kind: Kind::Plain,
            };
            assert_eq!(
                rewritten(&p, &block("replacement", Kind::Plain)),
                "replacement"
            );
        }
        for (source, expected) in [
            ("left  \nright", "left right"),
            ("left<br>right", "left right"),
            ("**left<br>right**", "left right"),
        ] {
            let (parsed, notes) = blocks(source);
            assert!(!notes);
            assert_eq!(parsed.len(), 1);
            assert_eq!(parsed[0].kind, Kind::Plain);
            assert_eq!(parsed[0].text(), expected);
            if source.starts_with("**") {
                assert!(parsed[0].runs.iter().all(|run| run.bold == Some(true)));
            }
        }
    }
}
