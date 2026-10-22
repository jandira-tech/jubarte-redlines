// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The verify-and-repair step of [`super::critic::resolve`] (#1240).
//!
//! `resolve` splices the kept side of every mark back into the source and
//! lets Markdown re-read it. The splice is not compositional under
//! CommonMark's delimiter rules — a kept `_b_` inside `*a…c*` reads as
//! literal underscores, `***a*b*c***` reads as italic plus a stray `**`.
//! The truth is the md→docx reader's own run model: `*a{++_b_++}c*` is
//! the italic runs `a`, `b` (inserted) and `c`, which accepting turns
//! into one italic run `abc`. So for each inline block holding a mark,
//! the spliced bytes' runs (`inline_runs`) are compared with the
//! source's runs resolved under the decision (`resolved_runs`). Equal:
//! the splice's bytes stay. Different: the smallest inline span holding
//! the mark — the seam — is re-emitted from the truth as the
//! common-format interval tree (`**a*b*c**`), the user's delimiter
//! flavour first, every try verified the same way. A block no seam
//! repairs is re-emitted whole, its block syntax kept; when the document
//! still differs, the splice is kept — never worse than before.
//!
//! The reader's model is not a truth in three more shapes this dialect
//! adds, which keep the splice too: a change named as nested inside
//! another (`{~~old~>new~~}{r4+5 Bob re r3 Alice}`), whose text goes
//! either way while the model keeps its old side; a block holding an
//! anchor, which `Item::Bookmark` records without the change it sits in;
//! and a mark holding a line break, which the reader normalizes.

use std::ops::Range;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use super::DocxOptions;
use super::critic::{member_span_ranges, resolve_with_policy, resolved_regions, resolved_slice};
use super::diff;
use super::from_docx::{code_fence, escape_critic, escape_line_starts, escape_markdown};
use super::write::{parser_options, read};
use super::xml::{self, Content, Document, Item, Kind, Link, RunFormat};

/// One inline run of a fragment, as the md→docx reader sees it: text
/// plus every format the reader tracks (emphasis, strong, strike, code,
/// underline, super- and subscript, highlight) and the link target, and
/// the atoms Markdown cannot re-parse (note references, pictures,
/// equations, fields, breaks). `BlockEnd` marks a paragraph boundary so
/// text on either side never compares as one run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InlineRun {
    pub(crate) kind: RunKind,
    pub(crate) format: RunFormat,
    /// `Some((href, title))` while the run sits inside a link.
    pub(crate) link: Option<(String, Option<String>)>,
}

/// What an [`InlineRun`] carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RunKind {
    Text(String),
    /// A code span's CommonMark content.
    Code(String),
    /// `<br>`.
    Break,
    Tab,
    /// A rendered page break: Markdown cannot re-emit one.
    PageBreak,
    /// A footnote or endnote reference.
    NoteRef,
    /// A picture, by its alt text.
    Picture(String),
    /// A `drawing:` payload, by its index: Markdown cannot re-emit one.
    Drawing(usize),
    Math {
        latex: String,
        display: bool,
    },
    /// A body field's instruction (`{PAGE}` reads as `FieldBegin("PAGE")`).
    FieldBegin(String),
    /// `{/}` ending a body field.
    FieldEnd,
    /// `<a id="name"></a>`.
    Bookmark(String),
    /// A paragraph boundary in the model, not a run.
    BlockEnd,
}

/// The runs `markdown` reads to, adjacent runs of equal format merged:
/// the shape a resolved candidate must re-parse to.
#[cfg(test)]
pub(crate) fn inline_runs(markdown: &str) -> Vec<InlineRun> {
    runs_of(&read(markdown, &DocxOptions::default()), None)
}

/// The runs `markdown` reads to once `accept` resolves its marks: the
/// truth a candidate is checked against.
#[cfg(test)]
pub(crate) fn resolved_runs(markdown: &str, accept: bool) -> Vec<InlineRun> {
    runs_of(&read(markdown, &DocxOptions::default()), Some(accept))
}

/// `resolved_runs` with the document's link definitions appended, so
/// a fragment's reference links resolve as they do in the document.
/// `accept` of `None` keeps every change the text still names. `None`
/// when the read leaves a malformed-delimiter warning: the reader's
/// model of a mark it could not hold (a comment or a highlight inside a
/// change splits it) is not a truth to emit from, and the splice stays.
fn fragment_runs(text: &str, defs: &str, accept: Option<bool>) -> Option<Vec<InlineRun>> {
    let source = if defs.is_empty() {
        text.to_string()
    } else {
        format!("{text}\n\n{defs}")
    };
    let document = read(&source, &DocxOptions::default());
    if document
        .warnings
        .iter()
        .any(|warning| warning.ends_with(" is text"))
    {
        return None;
    }
    let mut runs = runs_of(&document, accept);
    // The fragment's own end is not content: emitting it would write a
    // blank line into the block.
    if runs
        .last()
        .is_some_and(|run| matches!(run.kind, RunKind::BlockEnd))
    {
        runs.pop();
    }
    Some(runs)
}

/// The critic delimiters a faithful read keeps as marks, never as run
/// text. One inside a `want`/`expected` text run means the reader could
/// not hold the source's marks — a comment or a highlight inside a
/// change splits it — and its model is no truth to emit from. The
/// splice, already the spec's resolution, stays.
const DELIMITERS: [&str; 11] = [
    "{--", "--}", "{++", "++}", "{~~", "~~}", "~>", "{==", "==}", "{>>", "<<}",
];

fn mangled(runs: &[InlineRun]) -> bool {
    runs.iter().any(|run| {
        matches!(&run.kind, RunKind::Text(text)
            if DELIMITERS.iter().any(|delimiter| text.contains(delimiter)))
    })
}

/// Whether `runs` holds a bookmark: `Item::Bookmark` carries no change,
/// so the model keeps an anchor the decision dropped (the docx writer
/// does not) and cannot be a truth to emit from. A block or seam
/// holding one keeps the splice, as [`mangled`] does.
fn holds_bookmark(runs: &[InlineRun]) -> bool {
    runs.iter()
        .any(|run| matches!(run.kind, RunKind::Bookmark(_)))
}

/// One run on the model: a text run merges into the one before it of
/// the same format and link, as the reader's own merge does.
fn push_run(
    runs: &mut Vec<InlineRun>,
    kind: RunKind,
    format: RunFormat,
    link: Option<(String, Option<String>)>,
) {
    if let Some(InlineRun {
        kind: prev,
        format: before,
        link: before_link,
    }) = runs.last_mut()
        && *before == format
        && *before_link == link
    {
        match (prev, &kind) {
            (RunKind::Text(prev), RunKind::Text(text))
            | (RunKind::Code(prev), RunKind::Code(text)) => {
                prev.push_str(text);
                return;
            }
            _ => {}
        }
    }
    runs.push(InlineRun { kind, format, link });
}

/// The document's body runs, adjacent equal-format texts merged. Runs
/// the decision drops (a deleted run on accept, an inserted one on
/// reject, a whole paragraph's mark the same way) never appear.
fn runs_of(document: &Document, accept: Option<bool>) -> Vec<InlineRun> {
    let kept = |change: Option<(Kind, u32)>| match accept {
        None => true,
        Some(true) => change.is_none_or(|(kind, _)| kind == Kind::Insert),
        Some(false) => change.is_none_or(|(kind, _)| kind == Kind::Delete),
    };
    let mut runs: Vec<InlineRun> = Vec::new();
    let mut link: Option<(String, Option<String>)> = None;
    let mut drop_block = false;
    for item in &document.body {
        match item {
            Item::Paragraph(paragraph) => {
                drop_block = accept.is_some_and(|accept| {
                    paragraph.mark == Some(if accept { Kind::Delete } else { Kind::Insert })
                });
            }
            Item::ParagraphEnd => {
                if !drop_block {
                    push_run(&mut runs, RunKind::BlockEnd, RunFormat::default(), None);
                }
                drop_block = false;
            }
            Item::Run(run) => {
                if drop_block || !kept(run.change) {
                    continue;
                }
                let kind = match &run.content {
                    Content::Text(text) if run.format.code => RunKind::Code(text.clone()),
                    Content::Text(text) => RunKind::Text(text.clone()),
                    Content::Break => RunKind::Break,
                    Content::PageBreak => RunKind::PageBreak,
                    Content::Tab => RunKind::Tab,
                    Content::NoteReference(_) => RunKind::NoteRef,
                    Content::Picture(index) => {
                        RunKind::Picture(document.pictures[*index].alt.clone())
                    }
                    Content::Drawing(index) => RunKind::Drawing(*index),
                    Content::Math { latex, display } => RunKind::Math {
                        latex: latex.clone(),
                        display: *display,
                    },
                    Content::FieldBegin(code) => RunKind::FieldBegin(code.clone()),
                    Content::FieldEnd => RunKind::FieldEnd,
                };
                push_run(&mut runs, kind, run.format, link.clone());
            }
            Item::Bookmark(name) => {
                if !drop_block {
                    push_run(
                        &mut runs,
                        RunKind::Bookmark(name.clone()),
                        RunFormat::default(),
                        link.clone(),
                    );
                }
            }
            Item::LinkStart(target, title) => {
                let href = match target {
                    Link::External(url) => url.clone(),
                    Link::Anchor(name) => format!("#{name}"),
                };
                link = Some((href, title.clone()));
            }
            Item::LinkEnd => link = None,
            _ => {}
        }
    }
    runs
}

/// The inline-content blocks of `source`, innermost only: paragraphs,
/// headings, table cells, and tight list items (whose content the
/// parser wraps in no paragraph). `outer` is the block's whole range;
/// `content` is `outer` past its first line's block marker — the part a
/// last-resort re-emission rewrites while the block syntax stays.
#[derive(Clone, Debug)]
struct InlineBlock {
    outer: Range<usize>,
    content: Range<usize>,
}

fn inline_blocks(source: &str) -> Vec<InlineBlock> {
    let mut ranges = Vec::new();
    for (event, range) in Parser::new_ext(source, parser_options()).into_offset_iter() {
        if matches!(
            event,
            Event::End(TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::TableCell | TagEnd::Item)
        ) {
            ranges.push(range);
        }
    }
    ranges.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));
    ranges.dedup();
    let all = ranges.clone();
    ranges.retain(|inner| {
        !all.iter()
            .any(|outer| *outer != *inner && outer.start <= inner.start && inner.end <= outer.end)
    });
    ranges
        .into_iter()
        .map(|outer| {
            let first = source[outer.clone()].lines().next().unwrap_or_default();
            let marker = diff::marker_len(first).min(outer.end - outer.start);
            InlineBlock {
                content: outer.start + marker..outer.end,
                outer,
            }
        })
        .collect()
}

/// Whether `source[block]` re-reads as the document does: every later
/// line must be a plain continuation — a quote marker, list marker,
/// heading, table row or fence on a later line is block syntax a slice
/// would re-read differently, so such a block is verified only through
/// the whole document, never repaired locally.
fn self_contained(source: &str, block: &Range<usize>) -> bool {
    source[block.clone()].lines().skip(1).all(diff::continues)
}

/// The document's link-definition lines (`[label]: dest "title"`), so a
/// block's or seam's reference links resolve in a fragment as they do
/// in the document.
fn link_definitions(source: &str) -> String {
    let names: Vec<String> = Parser::new_ext(source, parser_options())
        .reference_definitions()
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();
    if names.is_empty() {
        return String::new();
    }
    let mut defs = String::new();
    for line in source.lines() {
        let trimmed = line.trim_end();
        let Some(rest) = trimmed.strip_prefix('[') else {
            continue;
        };
        let Some(end) = rest.find("]:") else {
            continue;
        };
        if names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&rest[..end]))
        {
            defs.push_str(trimmed);
            defs.push('\n');
        }
    }
    defs
}

/// The kinds of verbatim atom a re-emitted seam carries byte-for-byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AtomKind {
    /// `[…](u)`, `[…][ref]`, `<https://…>`: a link atom matches only the
    /// whole group of runs sharing its target.
    Link,
    /// `![…](…)`: carried, or the image is lost.
    Image,
    /// `[^label]`: carried; the label does not survive the run model.
    Note,
    /// `$…$` or `$$…$$`: its source may hold mark delimiters the reader
    /// turns into its own sentinels, which cannot be rebuilt.
    Math,
    /// A code span: carrying it keeps its fence — a rebuilt `` ` `` fence
    /// can merge with a backtick run beside it.
    Code,
}

/// One atom of a seam's slice: its range in the slice, its kind,
/// whether it survives the decision byte-for-byte, and the run pattern
/// the reader gives it under the decision.
struct Atom {
    range: Range<usize>,
    kind: AtomKind,
    carryable: bool,
    pattern: Vec<InlineRun>,
}

fn atoms_of(slice: &str, defs: &str, accept: bool) -> Vec<Atom> {
    let mut atoms: Vec<(Range<usize>, AtomKind)> = Vec::new();
    let mut open: Vec<(usize, AtomKind)> = Vec::new();
    for (event, range) in Parser::new_ext(slice, parser_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::Link { .. }) => open.push((range.start, AtomKind::Link)),
            Event::Start(Tag::Image { .. }) => open.push((range.start, AtomKind::Image)),
            Event::End(TagEnd::Link | TagEnd::Image) => {
                let kind = if matches!(event, Event::End(TagEnd::Link)) {
                    AtomKind::Link
                } else {
                    AtomKind::Image
                };
                if let Some(at) = open.iter().rposition(|(_, k)| *k == kind) {
                    let (start, _) = open.remove(at);
                    atoms.push((start..range.end, kind));
                }
            }
            Event::FootnoteReference(_) => atoms.push((range, AtomKind::Note)),
            Event::InlineMath(_) | Event::DisplayMath(_) => atoms.push((range, AtomKind::Math)),
            Event::Code(_) => atoms.push((range, AtomKind::Code)),
            _ => {}
        }
    }
    atoms.sort_by_key(|(range, _)| range.start);
    atoms
        .into_iter()
        .map(|(range, kind)| {
            let text = &slice[range.clone()];
            let pattern = fragment_runs(text, defs, Some(accept))
                .into_iter()
                .flatten()
                .filter(|run| !matches!(run.kind, RunKind::BlockEnd))
                .collect();
            Atom {
                carryable: resolve_with_policy(text, accept, false) == text,
                range,
                kind,
                pattern,
            }
        })
        .collect()
}

/// Whether `runs` still holds `atom`'s content: its pattern, as
/// [`emit_runs`] matches it. An atom the decision dropped needs no
/// place in the emission.
fn atom_needed(atom: &Atom, runs: &[InlineRun]) -> bool {
    // Links, math and code are rebuilt from the model; only notes and
    // images must carry their slice to appear in the emission at all.
    if !matches!(atom.kind, AtomKind::Note | AtomKind::Image) {
        return false;
    }
    let width = atom.pattern.len();
    width > 0
        && runs.windows(width).any(|span| {
            span.iter()
                .zip(&atom.pattern)
                .all(|(a, b)| a.kind == b.kind && (atom.kind != AtomKind::Link || a.link == b.link))
        })
}

/// The smallest source span holding `region` once resolved, grown over
/// any inline span the region's delimiters or kept body interact with
/// (the outer `*…*` of `*a{++_b_++}c*`) and over the delimiter runs
/// touching it: `****` beside `**p**` re-pairs across the seam once the
/// mark resolves.
fn seam_of(source: &str, region: &Range<usize>, spans: &[Range<usize>]) -> Range<usize> {
    let delimiter = |c: char| matches!(c, '*' | '_' | '~' | '`');
    let mut range = region.clone();
    loop {
        let mut grew = false;
        for span in spans {
            let overlaps = span.start < range.end && range.start < span.end;
            let inside = span.start >= range.start && span.end <= range.end;
            if overlaps && !inside {
                range.start = range.start.min(span.start);
                range.end = range.end.max(span.end);
                grew = true;
            }
        }
        while let Some(c) = source[..range.start].chars().next_back() {
            if !delimiter(c) {
                break;
            }
            range.start -= c.len_utf8();
            grew = true;
        }
        while let Some(c) = source[range.end..].chars().next() {
            if !delimiter(c) {
                break;
            }
            range.end += c.len_utf8();
            grew = true;
        }
        if !grew {
            return range;
        }
    }
}

/// A spelling set for the emitter: emphasis, strong and strikethrough
/// markers in one flavour, each pair `(open, close)`.
struct Flavour {
    em: String,
    strong: String,
    strike: (String, String),
}

/// The user's delimiter characters inside `source`'s seam first, each
/// with `~~` and `<s>` strikes, then the writer's `*` default the same
/// way, then a pure-HTML flavour: `~~` and `*` cannot close beside a
/// word character (`~~v`, `_…_j`), `<s>` and `<i>` can.
fn flavours_of(source: &str, seam: &Range<usize>) -> Vec<Flavour> {
    let mut chars: Vec<char> = Vec::new();
    for c in source[seam.clone()].chars() {
        if matches!(c, '*' | '_') && !chars.contains(&c) {
            chars.push(c);
        }
    }
    for c in ['*', '_'] {
        if !chars.contains(&c) {
            chars.push(c);
        }
    }
    let mut flavours: Vec<Flavour> = chars
        .into_iter()
        .flat_map(|c| {
            [
                Flavour {
                    em: c.to_string(),
                    strong: format!("{c}{c}"),
                    strike: ("~~".to_string(), "~~".to_string()),
                },
                Flavour {
                    em: c.to_string(),
                    strong: format!("{c}{c}"),
                    strike: ("<s>".to_string(), "</s>".to_string()),
                },
            ]
        })
        .collect();
    flavours.push(Flavour {
        em: "<i>".to_string(),
        strong: "<b>".to_string(),
        strike: ("<s>".to_string(), "</s>".to_string()),
    });
    flavours
}

/// An open format level in the emitted tree. Levels carry their
/// delimiter pair so a flavour can spell them `_`/`__`, `*`/`**` or
/// `<i>`/`<b>`.
#[derive(Clone, PartialEq)]
enum Level {
    Strike(String, String),
    Strong(String),
    Em(String),
    Mark(Option<&'static str>),
    Underline,
    Sup,
    Sub,
}

/// The run's formats as open levels, outermost first — the
/// common-format order the interval tree emits (`**a*b*c**`).
fn levels(format: RunFormat, flavour: &Flavour) -> Vec<Level> {
    let mut levels = Vec::new();
    if format.strike {
        levels.push(Level::Strike(
            flavour.strike.0.clone(),
            flavour.strike.1.clone(),
        ));
    }
    if format.bold {
        levels.push(Level::Strong(flavour.strong.clone()));
    }
    if format.italic {
        levels.push(Level::Em(flavour.em.clone()));
    }
    if format.mark {
        levels.push(Level::Mark(format.mark_color));
    }
    if format.underline {
        levels.push(Level::Underline);
    }
    if format.superscript {
        levels.push(Level::Sup);
    }
    if format.subscript {
        levels.push(Level::Sub);
    }
    levels
}

fn open(level: &Level) -> String {
    match level {
        Level::Strike(open, _) => open.clone(),
        Level::Strong(marker) | Level::Em(marker) => marker.clone(),
        Level::Mark(None) => "<mark>".to_string(),
        Level::Mark(Some(colour)) => format!("<mark class=\"hl-{colour}\">"),
        Level::Underline => "<u>".to_string(),
        Level::Sup => "<sup>".to_string(),
        Level::Sub => "<sub>".to_string(),
    }
}

fn close(level: &Level) -> String {
    match level {
        Level::Strike(_, close) => close.clone(),
        Level::Strong(marker) | Level::Em(marker) if marker.starts_with('<') => {
            format!("</{}>", marker.trim_matches(|c| c == '<' || c == '>'))
        }
        Level::Strong(marker) | Level::Em(marker) => marker.clone(),
        Level::Mark(_) => "</mark>".to_string(),
        Level::Underline => "</u>".to_string(),
        Level::Sup => "</sup>".to_string(),
        Level::Sub => "</sub>".to_string(),
    }
}

/// Re-aim `open_levels` at `want`: shared outer levels stay, the rest
/// close innermost first, the missing ones open.
fn adjust(open_levels: &mut Vec<Level>, want: &[Level], out: &mut String) {
    let keep = open_levels
        .iter()
        .zip(want)
        .take_while(|(a, b)| a == b)
        .count();
    for level in open_levels.drain(keep..).rev() {
        out.push_str(&close(&level));
    }
    for level in &want[keep..] {
        out.push_str(&open(level));
    }
    open_levels.extend(want[keep..].iter().cloned());
}

/// `]` and `|` not already escaped get one: a `]` would close the link
/// text it sits in, a `|` splits the table cell it lands in. A run of
/// backslashes counts: an odd run escapes already.
fn escape_brackets(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 4);
    let mut backslashes = 0usize;
    for c in text.chars() {
        if matches!(c, ']' | '|') && backslashes.is_multiple_of(2) {
            out.push('\\');
        }
        out.push(c);
        backslashes = if c == '\\' { backslashes + 1 } else { 0 };
    }
    out
}

/// Text as an emitted run's content: every character CommonMark or
/// CriticMarkup would act on escaped, then `]` and `|` guarded, and a
/// block opener a later line would start with broken.
fn escaped_text(text: &str) -> String {
    let text = escape_brackets(&escape_markdown(text));
    let text = escape_critic(&text);
    escape_line_starts(&text)
}

/// A link destination as `(href)` — `<…>` when it holds spaces or
/// parentheses — plus its optional ` "title"`.
fn link_tail(link: &(String, Option<String>)) -> String {
    let (href, title) = link;
    let href = if href
        .chars()
        .any(|c| c.is_whitespace() || matches!(c, '(' | ')'))
    {
        format!("<{href}>")
    } else {
        href.clone()
    };
    match title {
        Some(title) => format!("({href} \"{}\")", title.replace('"', "\\\"")),
        None => format!("({href})"),
    }
}

/// The truth's runs as Markdown the reader parses back to the same
/// runs, in `flavour`: atoms carried verbatim where the decision left
/// them untouched, text escaped, formats nested as the common-format
/// interval tree. `None` when a run cannot be emitted (a drawing, a
/// page break, a note reference or image with no slice to carry, or an
/// atom the runs cannot place).
fn emit_runs(
    runs: &[InlineRun],
    atoms: &[Atom],
    slice: &str,
    flavour: &Flavour,
    used: &mut Vec<bool>,
    in_link: bool,
) -> Option<String> {
    let mut out = String::new();
    let mut open_levels: Vec<Level> = Vec::new();
    let mut at = 0;
    while at < runs.len() {
        let run = &runs[at];
        if matches!(run.kind, RunKind::BlockEnd) {
            adjust(&mut open_levels, &[], &mut out);
            out.push_str("\n\n");
            at += 1;
            continue;
        }
        // A link wraps the whole group of runs sharing its target; a
        // nested emission (`in_link`) writes its text without re-opening.
        if run.link.is_some() && !in_link {
            let mut end = at + 1;
            while end < runs.len() && runs[end].link == run.link {
                end += 1;
            }
            let group = &runs[at..end];
            // A link atom spanning the whole group carries its slice
            // (`[a][ref]`, `<https://…>`); one the decision touched is
            // re-emitted as an inline link instead of failing.
            let mut carried = false;
            for (index, atom) in atoms.iter().enumerate() {
                if used[index] || atom.kind != AtomKind::Link || atom.pattern.len() != group.len() {
                    continue;
                }
                let matches = group
                    .iter()
                    .zip(&atom.pattern)
                    .all(|(a, b)| a.kind == b.kind && a.link == b.link && a.format == b.format);
                let format = group[0].format;
                if !matches || !group.iter().all(|run| run.format == format) {
                    continue;
                }
                if atom.carryable {
                    adjust(&mut open_levels, &levels(format, flavour), &mut out);
                    out.push_str(&slice[atom.range.clone()]);
                    used[index] = true;
                    carried = true;
                }
                break;
            }
            if carried {
                at = end;
                continue;
            }
            // Enclosing the link would hide its inner formats; they
            // re-open inside the text instead — `[*cp*](u)` not
            // `*[cp](u)*`.
            adjust(&mut open_levels, &[], &mut out);
            let inner = emit_runs(group, atoms, slice, flavour, used, true)?;
            out.push('[');
            out.push_str(&inner);
            out.push(']');
            out.push_str(&link_tail(run.link.as_ref()?));
            at = end;
            continue;
        }
        // A verbatim atom whose pattern sits here carries its slice.
        let mut carried = false;
        for (index, atom) in atoms.iter().enumerate() {
            if used[index] || atom.pattern.is_empty() || (in_link && atom.kind == AtomKind::Link) {
                continue;
            }
            let width = atom.pattern.len();
            let Some(span) = runs.get(at..at + width) else {
                continue;
            };
            // A link atom's runs share its target; other atoms match on
            // content alone, the enclosing format or link beside them.
            let matches = span.iter().zip(&atom.pattern).all(|(a, b)| {
                a.kind == b.kind
                    && (atom.kind != AtomKind::Link || (a.link == b.link && a.format == b.format))
            });
            if !matches {
                continue;
            }
            let format = span[0].format;
            if !span.iter().all(|run| run.format == format) {
                continue;
            }
            if atom.kind == AtomKind::Link
                && ((at > 0 && runs[at - 1].link == span[0].link)
                    || runs
                        .get(at + width)
                        .is_some_and(|next| next.link == span[0].link))
            {
                continue;
            }
            if !atom.carryable {
                // Math and code rebuild from the model; every other
                // kind must carry or the seam cannot be emitted.
                if matches!(atom.kind, AtomKind::Math | AtomKind::Code) {
                    continue;
                }
                return None;
            }
            adjust(&mut open_levels, &levels(format, flavour), &mut out);
            out.push_str(&slice[atom.range.clone()]);
            used[index] = true;
            at += width;
            carried = true;
            break;
        }
        if carried {
            continue;
        }
        match &run.kind {
            RunKind::Text(text) => {
                adjust(&mut open_levels, &levels(run.format, flavour), &mut out);
                out.push_str(&escaped_text(text));
            }
            RunKind::Code(text) => {
                adjust(&mut open_levels, &levels(run.format, flavour), &mut out);
                let (open, close) = code_fence(text);
                out.push_str(&format!("{open}{text}{close}"));
            }
            RunKind::Break => {
                adjust(&mut open_levels, &levels(run.format, flavour), &mut out);
                out.push_str("<br>");
            }
            RunKind::Tab => {
                adjust(&mut open_levels, &levels(run.format, flavour), &mut out);
                out.push('\t');
            }
            RunKind::Math { latex, display } => {
                adjust(&mut open_levels, &levels(run.format, flavour), &mut out);
                let fence = if *display { "$$" } else { "$" };
                out.push_str(&format!("{fence}{latex}{fence}"));
            }
            RunKind::FieldBegin(code) => {
                adjust(&mut open_levels, &[], &mut out);
                out.push_str(&format!("{{{code}}}"));
            }
            RunKind::FieldEnd => {
                adjust(&mut open_levels, &[], &mut out);
                out.push_str("{/}");
            }
            RunKind::Bookmark(name) => {
                adjust(&mut open_levels, &[], &mut out);
                // The reader decodes a bookmark's entities, so the name
                // can hold a quote: the docx→md writer escapes it at the
                // same sink, and a repaired seam must not write an
                // attribute the user did not type.
                out.push_str(&format!("<a id=\"{}\"></a>", xml::escape(name)));
            }
            // Atoms the model cannot rebuild: they must have matched a
            // carried slice above.
            RunKind::PageBreak | RunKind::NoteRef | RunKind::Picture(_) | RunKind::Drawing(_) => {
                return None;
            }
            RunKind::BlockEnd => unreachable!("handled at the loop's head"),
        }
        at += 1;
    }
    adjust(&mut open_levels, &[], &mut out);
    Some(out)
}

/// `source`'s resolved output verified: `candidate` when the reader
/// already gives the spliced bytes the truth's runs, else the smallest
/// repairs that verify — seam by seam, then block by block — and the
/// splice untouched when nothing verifies. A block or seam whose truth
/// is not the reader's model (a nested change, an anchor, [`mangled`]
/// text, a mark holding a line break, a mark crossing the block's edge)
/// is left to the splice, as is the whole document when it still reads
/// differently from the model.
pub(super) fn repair(source: &str, candidate: &str, accept: bool) -> String {
    let regions = resolved_regions(source);
    if regions.is_empty() {
        return candidate.to_string();
    }
    let defs = link_definitions(source);
    let blocks = inline_blocks(source);
    if blocks.len() != inline_blocks(candidate).len() {
        return candidate.to_string();
    }
    let span_ranges = member_span_ranges(source);
    let nested = super::critic::nested_change_regions(source);
    let mut work = source.to_string();
    for block in blocks.iter().rev() {
        // A mark crossing the block's edge, or a block whose later lines
        // carry block syntax, is outside local verification.
        let crossing = regions.iter().any(|region| {
            region.start < block.outer.start && block.outer.start < region.end
                || region.start < block.outer.end && block.outer.end < region.end
        });
        // The reader normalizes whitespace inside a mark: which side of
        // it a kept `\n` lands on is not a truth it holds, and the
        // splice keeps the user's lines exactly.
        let newline = regions.iter().any(|region| {
            block.outer.start <= region.start
                && region.end <= block.outer.end
                && source[region.clone()].contains('\n')
        });
        // A change named as nested inside another (`{… re r3}`) goes
        // either way, a rule the reader's run model does not hold: the
        // model would keep the inner old side and the block would be
        // re-emitted to it. Keep the splice instead.
        let parented = nested
            .iter()
            .any(|region| block.outer.start <= region.start && region.end <= block.outer.end);
        if crossing || newline || parented || !self_contained(source, &block.outer) {
            continue;
        }
        // Both sides of every comparison end in the fragment's own
        // `BlockEnd`, so it cancels out.
        let Some(expected) = fragment_runs(&source[block.outer.clone()], &defs, Some(accept))
        else {
            continue;
        };
        if mangled(&expected) || holds_bookmark(&expected) {
            continue;
        }
        // The block's position in `work` drifts as repairs replace text:
        // `outer` and `content` track it there.
        let mut outer = block.outer.clone();
        let mut content = block.content.clone();
        let block_runs = |work: &str, outer: &Range<usize>| {
            fragment_runs(&resolved_slice(&work[outer.clone()], accept), &defs, None)
        };
        if block_runs(&work, &outer).as_deref() == Some(expected.as_slice()) {
            continue;
        }
        // The block's marks, seam by seam, rightmost first: each seam
        // re-emits from the truth in the user's flavour, then the
        // writer's, and verifies before the next seam.
        let mut seams: Vec<Range<usize>> = regions
            .iter()
            .filter(|region| block.outer.start <= region.start && region.end <= block.outer.end)
            .map(|region| seam_of(source, region, &span_ranges))
            .collect();
        // Overlapping seams share their resolution: one re-emission
        // must cover both.
        seams.sort_by_key(|range| range.start);
        let mut merged: Vec<Range<usize>> = Vec::new();
        for seam in seams {
            match merged.last_mut() {
                Some(last) if seam.start <= last.end => last.end = last.end.max(seam.end),
                _ => merged.push(seam),
            }
        }
        merged.sort_by_key(|range| std::cmp::Reverse(range.start));
        for seam in merged {
            let slice = source[seam.clone()].to_string();
            let Some(want) = fragment_runs(&slice, &defs, Some(accept)) else {
                continue;
            };
            if mangled(&want) || holds_bookmark(&want) {
                continue;
            }
            // The seam still holds its source bytes: edits so far land
            // right of it.
            debug_assert_eq!(work[seam.clone()], slice);
            // The mark's context decides its splice: resolving the seam
            // alone can verify where the document's does not, so emit.
            let atoms = atoms_of(&slice, &defs, accept);
            for flavour in flavours_of(source, &seam) {
                let mut used = vec![false; atoms.len()];
                let Some(emitted) = emit_runs(&want, &atoms, &slice, &flavour, &mut used, false)
                else {
                    continue;
                };
                if atoms
                    .iter()
                    .enumerate()
                    .any(|(index, atom)| !used[index] && atom_needed(atom, &want))
                {
                    break;
                }
                work.replace_range(seam.clone(), &emitted);
                let now = seam.start..seam.start + emitted.len();
                let got = fragment_runs(&resolved_slice(&work[now.clone()], accept), &defs, None);
                if got.as_deref() == Some(want.as_slice()) {
                    let delta = emitted.len() as isize - slice.len() as isize;
                    outer.end = (outer.end as isize + delta) as usize;
                    content.end = (content.end as isize + delta) as usize;
                    break;
                }
                work.replace_range(now, &slice);
            }
        }
        // Last resort: the block's whole inline content, its block
        // syntax kept.
        if block_runs(&work, &outer).as_deref() != Some(expected.as_slice()) {
            work.replace_range(outer.clone(), &source[block.outer.clone()]);
            outer.end = outer.start + (block.outer.end - block.outer.start);
            content.end = content.start + (block.content.end - block.content.start);
            let slice = source[block.content.clone()].to_string();
            let atoms = atoms_of(&slice, &defs, accept);
            for flavour in flavours_of(source, &block.outer) {
                let mut used = vec![false; atoms.len()];
                let Some(emitted) =
                    emit_runs(&expected, &atoms, &slice, &flavour, &mut used, false)
                else {
                    break;
                };
                if atoms
                    .iter()
                    .enumerate()
                    .any(|(index, atom)| !used[index] && atom_needed(atom, &expected))
                {
                    break;
                }
                work.replace_range(content.clone(), &emitted);
                let delta = emitted.len() as isize - slice.len() as isize;
                let now = content.start..content.start + emitted.len();
                let shifted = outer.start..(outer.end as isize + delta) as usize;
                if block_runs(&work, &shifted).as_deref() == Some(expected.as_slice()) {
                    break;
                }
                work.replace_range(now, &slice);
            }
        }
    }
    let resolved = resolved_slice(&work, accept);
    match (
        fragment_runs(source, &defs, Some(accept)),
        fragment_runs(&resolved, &defs, None),
    ) {
        (Some(want), Some(got)) if want == got && !mangled(&want) && !holds_bookmark(&want) => {
            resolved
        }
        _ => candidate.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `resolve_critic`'s public entry: `true` accepts.
    fn resolve(markdown: &str, accept: bool) -> String {
        crate::markdown::critic::resolve(markdown, accept)
    }

    #[test]
    fn inline_runs_reads_format_link_and_code() {
        let runs = inline_runs("*b* **c** ~~d~~ `e` [f](u)\n");
        fn text(runs: &[InlineRun], n: usize) -> &str {
            match &runs[n].kind {
                RunKind::Text(text) => text.as_str(),
                kind => panic!("run {n} is {kind:?}, not text"),
            }
        }
        assert_eq!(text(&runs, 0), "b");
        assert!(runs[0].format.italic && !runs[0].format.bold);
        assert_eq!(text(&runs, 2), "c");
        assert!(runs[2].format.bold && !runs[2].format.italic);
        assert_eq!(text(&runs, 4), "d");
        assert!(runs[4].format.strike);
        assert_eq!(runs[6].kind, RunKind::Code("e".to_string()));
        assert_eq!(text(&runs, 8), "f");
        assert_eq!(runs[8].link, Some(("u".to_string(), None)));
        assert_eq!(runs[9].kind, RunKind::BlockEnd);
    }

    #[test]
    fn inline_runs_merges_adjacent_runs_of_equal_format() {
        // Same-format, same-target runs fold into one, as the reader's
        // own merge does.
        let runs = inline_runs("[a](u)[b](u)\n");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].kind, RunKind::Text("ab".to_string()));
        assert_eq!(runs[0].link, Some(("u".to_string(), None)));
    }

    #[test]
    fn resolved_runs_applies_the_decision() {
        let source = "a{++b++} {--c--}d\n";
        let accepted = resolved_runs(source, true);
        let rejected = resolved_runs(source, false);
        assert_eq!(accepted[0].kind, RunKind::Text("ab d".to_string()));
        assert_eq!(rejected[0].kind, RunKind::Text("a cd".to_string()));
    }

    #[test]
    fn a_candidate_that_verifies_is_emitted_byte_identical() {
        // The splice already reads correctly: repair must not touch it.
        let source = "Say *a*{++b++}*c* now.\n";
        for accept in [true, false] {
            let merged = crate::markdown::critic::resolved_slice(source, accept);
            assert_eq!(resolve(source, accept), merged);
        }
    }

    #[test]
    fn a_repaired_seam_keeps_block_syntax_and_neighbours() {
        assert_eq!(
            resolve("Say *a{++*b*++}c* now.\n", true),
            "Say *abc* now.\n"
        );
        assert_eq!(
            resolve("- item *a{++*b*++}c*\n- next\n", true),
            "- item *abc*\n- next\n"
        );
        assert_eq!(
            resolve("# head *a{++*b*++}c*\n\nbody\n", true),
            "# head *abc*\n\nbody\n"
        );
    }

    #[test]
    fn a_block_without_a_mark_is_byte_identical() {
        let source = "Say plain text now.\n\n- a\n- b\n\n| x | y |\n| - | - |\n| 1 | 2 |\n";
        for accept in [true, false] {
            assert_eq!(resolve(source, accept), source);
        }
    }

    /// A repair writes a bookmark name the md→docx reader decoded, so
    /// it escapes it exactly as the docx→md writer does at the same
    /// sink: `x" onclick="alert(1)"` stays one attribute's value
    /// (review gid=81).
    #[test]
    fn a_repaired_seam_escapes_a_bookmark_name() {
        let runs = vec![InlineRun {
            kind: RunKind::Bookmark("x\" onclick=\"alert(1)".to_string()),
            format: RunFormat::default(),
            link: None,
        }];
        let emitted = emit_runs(
            &runs,
            &[],
            "",
            &flavours_of("a*b*", &(0..1))[0],
            &mut Vec::new(),
            false,
        )
        .expect("a bookmark is emittable");
        assert_eq!(
            emitted, "<a id=\"x&quot; onclick=&quot;alert(1)\"></a>",
            "the name is written back as live attributes"
        );
    }

    /// The reader's model holds no bookmark's change membership, so a
    /// block with an anchor keeps the splice: accepting a deleted
    /// anchor drops it, as the splice does, and no repair may write it
    /// back from a model that cannot tell.
    #[test]
    fn a_block_holding_a_bookmark_keeps_the_splice() {
        let anchor = "<a id=\"mark\"></a>";
        let source = format!("Say **a{{--{anchor}b--}}c** now\n");
        assert_eq!(
            resolve(&source, true),
            "Say **ac** now\n",
            "accepting a deleted anchor drops it"
        );
        for accept in [true, false] {
            assert_eq!(
                resolve(&source, accept),
                crate::markdown::critic::resolved_slice(&source, accept),
                "accept={accept}"
            );
        }
    }

    /// A change named as nested inside another (`{… re r3}`) resolves
    /// to nothing on either decision — a rule the reader's run model
    /// does not hold — so its block keeps the splice byte-for-byte
    /// (`tests/md_review_1142.rs` pins the resolution itself).
    #[test]
    fn a_change_nested_in_another_keeps_the_splice() {
        let source = "Before {~~old~>new~~}{r4+5 Bob re r3 Alice} after";
        for accept in [true, false] {
            assert_eq!(
                resolve(source, accept),
                crate::markdown::critic::resolved_slice(source, accept),
                "accept={accept}"
            );
        }
    }
}
