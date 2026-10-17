// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The changes between two Markdown documents as CriticMarkup, as pandiff
//! writes them.
//!
//! Lines are aligned first, then words within the lines that pair up. On
//! lines both versions share, a list, heading or quote marker stays before
//! the change (`# {~~Old~>New~~} title`); a marker that itself changed
//! takes the whole block with it into one change, and a block one version
//! alone has goes into one change with its marker and its line breaks,
//! which
//! [`markdown_to_docx`](super::markdown_to_docx) writes as a block added or
//! removed whole. A line added or removed inside a paragraph takes the
//! line break with it (`first line{++\nnew line++}`).

use std::borrow::Cow;

use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};
use similar::{Algorithm, DiffTag, capture_diff_slices};

use crate::util::word_tokens as tokens;

/// Lines that share less than this share of their words are replaced whole
/// rather than word by word.
const WORD_DIFF_SIMILARITY: f64 = 0.3;

/// A word shared by more old lines than this does not suggest a pairing;
/// candidates come from the rarer words a line has. The work of pairing a
/// hunk is then bounded by the tokens in it, not by cells of a matrix.
const RARE_LINE_SHARE: usize = 8;

/// At most this many candidate old lines are scored against one new line.
const MAX_CANDIDATES: usize = 64;

/// A reflow whose two sides hold more than this many bytes is left to the
/// line pairing: Myers on the tokens is O((n+m)·d), and a rewrap this
/// large can have a distance to match.
const REFLOW_MAX_BYTES: usize = 400_000;

/// The two text snapshots, `old` and `new`, as one Markdown document
/// whose CriticMarkup holds the changes: accepting them all gives `new`,
/// rejecting them all gives `old`. Critically, the block markers
/// (heading `#`, list `-`, quote `>`, table `|`, footnote `[^…]:`, …)
/// stay outside the changes on lines both versions share; when the marker
/// itself changed — a renumbered item, `#` becoming `##`, a task box
/// ticked — or a whole block came and went, the block joins the change so
/// the marked-up document rebuilds either version exactly and the Word
/// writer reads it as a block changed whole.
///
/// # How changes are scoped
///
/// Lines are aligned first, then the words within the lines that pair up.
/// On lines both versions share, the marker stays before the change
/// (`- {~~old~>new~~}`); a marker that itself changed takes the whole
/// block with it — both versions of the line in one substitution — and a
/// block one version alone has goes into one change with its marker and
/// its line breaks, so accepting and rejecting rebuild the documents
/// exactly and [`markdown_to_docx`](super::markdown_to_docx) writes the
/// change as Word structure: a deleted heading is a deleted Heading
/// paragraph. A line added or removed inside a paragraph takes the line
/// break with it (`first line{++\nnew line++}`). Numbers (`$5,000,000`,
/// `04/20/26`, `3.5%`), emphasis and strikethrough delimiters, and link
/// syntax are read whole; a code span, an image, an equation, an autolink
/// and an HTML comment are one atom, so a change that touches any of them
/// covers all of it (`*Buyer*` → `Buyer` is one change, not two).
/// CriticMarkup delimiters already in the text are escaped — outside what
/// the reader reads verbatim — so they stay text.
///
/// # Inputs
///
/// - `old`, `new` — line-oriented Markdown (`\n` line ends, no carriage
///   returns — CRLF is normalized to LF).
///
/// # Examples
///
/// ```
/// use jubarte::markdown::diff_markdown;
///
/// assert_eq!(
///     diff_markdown("Payment in 30 days.\n", "Payment in 45 days.\n"),
///     "Payment in {~~30~>45~~} days.\n"
/// );
/// ```
pub fn diff_markdown(old: &str, new: &str) -> String {
    let old = old.replace("\r\n", "\n");
    let new = new.replace("\r\n", "\n");
    let old_lines = lines(&old);
    let new_lines = lines(&new);
    let mut out = Output::default();
    let ops = capture_diff_slices(Algorithm::Patience, &old_lines, &new_lines);
    for op in &ops {
        let (tag, olds, news) = op.as_tag_tuple();
        match tag {
            DiffTag::Equal => {
                for line in &new_lines[news] {
                    out.line(&escape(line));
                }
            }
            DiffTag::Delete | DiffTag::Insert | DiffTag::Replace => {
                hunk(&mut out, &old_lines[olds], &new_lines[news]);
            }
        }
    }
    out.finish(old.ends_with('\n'), new.ends_with('\n'))
}

fn lines(text: &str) -> Vec<&str> {
    let text = text.strip_suffix('\n').unwrap_or(text);
    if text.is_empty() {
        return Vec::new();
    }
    opaque_units(text)
}

/// The physical lines of `text`, except that a code span, equation, image,
/// autolink or HTML comment that spans a line break keeps its lines in one
/// unit: [`atoms`] then reads the whole construct, not a line of it.
fn opaque_units(text: &str) -> Vec<&str> {
    let mut starts: Vec<usize> = vec![0];
    starts.extend(
        text.match_indices('\n')
            .map(|(at, _)| at + 1)
            .filter(|&at| at < text.len()),
    );
    let line_of = |at: usize| starts.partition_point(|&start| start <= at) - 1;
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_MATH;
    for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
        let opaque = match event {
            Event::Code(_) => true,
            // Two amounts in a paragraph (`$250,000 … $5,000,000`) pair as
            // one equation to the parser; the amounts are text, so the
            // lines between them stay separate.
            Event::InlineMath(_) | Event::DisplayMath(_) => !opens_amount(text, range.start),
            Event::Html(_) | Event::InlineHtml(_) => true,
            Event::Start(Tag::Image { .. }) => true,
            Event::Start(Tag::Link { link_type, .. }) => {
                matches!(link_type, LinkType::Autolink | LinkType::Email)
            }
            _ => false,
        };
        if opaque && range.end > range.start && text[range.clone()].contains('\n') {
            spans.push((line_of(range.start), line_of(range.end - 1)));
        }
    }
    spans.sort_unstable();
    // Merge overlapping line ranges.
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (first, last) in spans {
        match merged.last_mut() {
            Some((_, end)) if first <= *end + 1 => *end = (*end).max(last),
            _ => merged.push((first, last)),
        }
    }
    // A line ends before its line break; a joined unit keeps the breaks
    // inside it but not the last one.
    let end_of = |line: usize| match starts.get(line + 1) {
        Some(&next_start) => next_start - 1,
        None => text.len(),
    };
    let mut units: Vec<&str> = Vec::new();
    let mut line = 0;
    for (first, last) in merged {
        while line < first {
            units.push(&text[starts[line]..end_of(line)]);
            line += 1;
        }
        units.push(&text[starts[first]..end_of(last)]);
        line = last + 1;
    }
    while line < starts.len() {
        units.push(&text[starts[line]..end_of(line)]);
        line += 1;
    }
    units
}

/// The length of a line's block markers: indentation, `>` quote markers, a
/// list marker with its task box, or a heading's `#`s. A change starts
/// after them.
pub(super) fn marker_len(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut at = bytes
        .iter()
        .take_while(|b| **b == b' ' || **b == b'\t')
        .count();
    while bytes.get(at) == Some(&b'>') {
        at += 1;
        if bytes.get(at) == Some(&b' ') {
            at += 1;
        }
    }
    let rest = &line[at..];
    // A footnote definition, `[^label]: `.
    if rest.starts_with("[^")
        && let Some(end) = rest.find("]: ")
        && !rest[2..end].contains([' ', ']'])
    {
        return at + end + 3;
    }
    let hashes = rest.bytes().take_while(|b| *b == b'#').count();
    if (1..=6).contains(&hashes) && matches!(rest.as_bytes().get(hashes), None | Some(b' ')) {
        return at + hashes + usize::from(rest.len() > hashes);
    }
    let list = match rest.as_bytes().first() {
        Some(b'-' | b'*' | b'+') => 1,
        Some(b'0'..=b'9') => {
            let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
            if digits <= 9 && matches!(rest.as_bytes().get(digits), Some(b'.' | b')')) {
                digits + 1
            } else {
                0
            }
        }
        _ => 0,
    };
    if list > 0 && matches!(rest.as_bytes().get(list), Some(b' ' | b'\t')) {
        at += list + 1;
        let rest = &line[at..];
        for task in ["[ ] ", "[x] ", "[X] "] {
            if rest.starts_with(task) {
                at += task.len();
            }
        }
    }
    at
}

/// Whether a line continues the paragraph of a non-blank line before it:
/// it has text and no block marker, and is not a table row or code fence.
pub(super) fn continues(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.is_empty() && marker_len(line) == 0 && !trimmed.starts_with("```") && !table_row(line)
}

/// Whether text may continue after a line: not blank, not a heading, a
/// table row or a code fence.
pub(super) fn open_paragraph(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.is_empty()
        && !trimmed.starts_with('#')
        && !trimmed.starts_with("```")
        && !table_row(line)
}

/// A table row: the line starts with a pipe.
pub(super) fn table_row(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

/// A table row added or removed whole: each cell's text is one change, so
/// the row keeps its cells (a table drops text outside them).
pub(super) fn table_row_change(line: &str, open: &str, close: &str) -> String {
    let mut out = String::new();
    let mut cell = String::new();
    let mut escaped = false;
    let flush = |out: &mut String, cell: &mut String| {
        let text = cell.trim();
        if text.is_empty() || text.chars().all(|c| matches!(c, '-' | ':')) {
            out.push_str(cell);
        } else {
            let start = cell.len() - cell.trim_start().len();
            let end = cell.trim_end().len();
            out.push_str(&cell[..start]);
            out.push_str(open);
            out.push_str(&escape(&cell[start..end]));
            out.push_str(close);
            out.push_str(&cell[end..]);
        }
        cell.clear();
    };
    for c in line.chars() {
        if c == '|' && !escaped {
            flush(&mut out, &mut cell);
            out.push('|');
        } else {
            cell.push(c);
        }
        escaped = c == '\\' && !escaped;
    }
    flush(&mut out, &mut cell);
    out
}

/// CriticMarkup delimiters in text, escaped so they stay text.
///
/// A delimiter behind an odd number of backslashes is already escaped, as
/// [`super::unified`] counts them, and one the reader passes over
/// verbatim — inside a code span, a link destination, an autolink or an
/// HTML comment — is literal: escaping it would change the text the
/// reader gives back.
fn escape(text: &str) -> Cow<'_, str> {
    if !text_has_delimiter(text) {
        return Cow::Borrowed(text);
    }
    let points = escape_points(text);
    if points.is_empty() {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + points.len());
    let mut from = 0;
    for at in points {
        out.push_str(&text[from..at]);
        out.push('\\');
        from = at;
    }
    out.push_str(&text[from..]);
    Cow::Owned(out)
}

/// Openers and the separator take the backslash before their first
/// character; closers take it before `}`, which is how the marks are
/// written today (`\{++`, `++\}`, `~\>`).
const DELIMITERS: [(&str, usize); 11] = [
    ("{++", 0),
    ("{--", 0),
    ("{~~", 0),
    ("{==", 0),
    ("{>>", 0),
    ("++}", 2),
    ("--}", 2),
    ("~~}", 2),
    ("==}", 2),
    ("<<}", 2),
    ("~>", 1),
];

fn text_has_delimiter(text: &str) -> bool {
    DELIMITERS.iter().any(|(d, _)| text.contains(d))
}

/// Byte positions where a backslash must be inserted.
fn escape_points(text: &str) -> Vec<usize> {
    let spans = super::critic::verbatim_ranges(text);
    let bytes = text.as_bytes();
    let mut points = Vec::new();
    // The next verbatim span, and how far the scan has consumed it.
    let (mut span, mut end) = (0, 0);
    for i in 0..bytes.len() {
        if i < end {
            continue;
        }
        if spans.get(span).is_some_and(|range| range.start <= i) {
            end = spans[span].end;
            span += 1;
            continue;
        }
        for (delimiter, insert) in DELIMITERS {
            if bytes[i..].starts_with(delimiter.as_bytes()) && !escaped_at(text, i) {
                points.push(i + insert);
                break;
            }
        }
    }
    points.sort_unstable();
    points.dedup();
    points
}

/// Whether `at` follows an odd number of backslashes.
fn escaped_at(text: &str, at: usize) -> bool {
    text[..at].bytes().rev().take_while(|b| *b == b'\\').count() % 2 == 1
}

/// The marked-up document being written, line by line.
#[derive(Default)]
struct Output {
    text: String,
    started: bool,
    /// A closing delimiter the next line starts with: a change that ends
    /// with a line break inside a paragraph.
    carry: Option<&'static str>,
    /// The last line written.
    last: String,
}

impl Output {
    fn line(&mut self, line: &str) {
        if self.started {
            self.text.push('\n');
        }
        self.started = true;
        if let Some(close) = self.carry.take() {
            self.text.push_str(close);
        }
        self.text.push_str(line);
        self.last = line.to_string();
    }

    /// Appends to the last line without a line break.
    fn append(&mut self, text: &str) {
        self.started = true;
        self.text.push_str(text);
        self.last.push_str(text);
    }

    fn finish(mut self, old_trailing: bool, new_trailing: bool) -> String {
        if let Some(close) = self.carry.take() {
            // The change that carried its closer holds the document's final
            // line break when its side of the documents has one.
            let trailing = if close == "--}" {
                old_trailing
            } else {
                new_trailing
            };
            if trailing {
                self.text.push('\n');
            }
            self.text.push_str(close);
            return self.text;
        }
        if new_trailing && old_trailing {
            if self.started {
                self.text.push('\n');
            }
        } else if new_trailing != old_trailing {
            self.add_trailing(new_trailing);
        }
        self.text
    }

    /// The final line break changed sides. It joins the last change when
    /// the text ends with that side's closer, rather than opening a change
    /// of its own next to it.
    fn add_trailing(&mut self, inserted: bool) {
        let close = if inserted { "++}" } else { "--}" };
        if self.text.ends_with(close) {
            let at = self.text.len() - close.len();
            self.text.insert(at, '\n');
        } else if self.text.ends_with("~~}") {
            // A substitution: the break goes into the side it belongs to.
            let at = if inserted {
                self.text.len() - 3
            } else {
                self.text.rfind("~>").map_or(self.text.len(), |at| at + 2)
            };
            self.text.insert(at, '\n');
        } else {
            let open = if inserted { "{++" } else { "{--" };
            self.text.push_str(&format!("{open}\n{close}"));
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Delete,
    Insert,
}

impl Side {
    fn delimiters(self) -> (&'static str, &'static str) {
        match self {
            Self::Delete => ("{--", "--}"),
            Self::Insert => ("{++", "++}"),
        }
    }
}

/// One line of a changed hunk: paired, or only in one version.
enum Entry<'a> {
    Pair(&'a str, &'a str),
    Only(Side, &'a str),
}

fn hunk(out: &mut Output, olds: &[&str], news: &[&str]) {
    if let Some(body) = reflow(olds, news) {
        for line in body.split('\n') {
            out.line(line);
        }
        return;
    }
    let entries = pair(olds, news);
    let mut at = 0;
    while at < entries.len() {
        match entries[at] {
            Entry::Pair(old, new) => {
                paired(out, old, new);
                at += 1;
            }
            Entry::Only(..) => {
                let end = entries[at..]
                    .iter()
                    .position(|e| matches!(e, Entry::Pair(..)))
                    .map_or(entries.len(), |i| at + i);
                let only = |side: Side| -> Vec<&str> {
                    entries[at..end]
                        .iter()
                        .filter_map(|e| match e {
                            Entry::Only(s, line) if *s == side => Some(*line),
                            _ => None,
                        })
                        .collect()
                };
                let deleted = only(Side::Delete);
                let inserted = only(Side::Insert);
                group(out, Side::Delete, &deleted);
                group(out, Side::Insert, &inserted);
                at = end;
            }
        }
    }
}

/// A hard-wrapped paragraph whose lines all moved, as one word diff.
///
/// Line pairing rewrites each wrapped word into the next line (`{--runs--}`
/// then `{++runs ++}`). When the two sides are one plain paragraph and
/// joining them on spaces is a small change, the words are diffed across
/// the line breaks instead. Independent lines (a list of items, each
/// changed) are left to [`pair`]: their joined text is not a small change.
fn reflow(olds: &[&str], news: &[&str]) -> Option<String> {
    if !plain_paragraph(olds) || !plain_paragraph(news) || (olds.len() < 2 && news.len() < 2) {
        return None;
    }
    let (old, new) = (olds.join("\n"), news.join("\n"));
    if old.len() + new.len() > REFLOW_MAX_BYTES {
        return None;
    }
    let flat_old = old.replace('\n', " ");
    let flat_new = new.replace('\n', " ");
    if similarity(&flat_old, &flat_new) < 0.8 {
        return None;
    }
    let flat = words_diff(&flat_old, &flat_new);
    let marks =
        flat.matches("{--").count() + flat.matches("{++").count() + flat.matches("{~~").count();
    // More than a couple of word changes is a block of independent lines,
    // not a wrap that shifted.
    if marks > 2 {
        return None;
    }
    // Keep the line breaks in the diff so accepting and rejecting still
    // give the two wrappings. Do not join changes across a line break:
    // that pulls a word that only moved to the next line into the change.
    Some(reflow_body(&old, &new))
}

/// Word diff of a reflowed paragraph. A word match outranks a whitespace
/// match, so a word that only moved onto the next line stays put; the line
/// breaks still come along, and accepting or rejecting gives that wrapping.
/// The alignment is Myers over the tokens with each word repeated by its
/// weight, which stands in for a weighted subsequence: O((n+m)·d) work and
/// no score matrix.
fn reflow_body(old: &str, new: &str) -> String {
    let a = tokens(old);
    let b = tokens(new);
    fn weigh<'a>(tokens: &[&'a str]) -> Vec<&'a str> {
        tokens
            .iter()
            .flat_map(|token| std::iter::repeat_n(*token, weight(token) as usize))
            .collect()
    }
    fn owner(tokens: &[&str]) -> Vec<usize> {
        tokens
            .iter()
            .enumerate()
            .flat_map(|(i, token)| std::iter::repeat_n(i, weight(token) as usize))
            .collect()
    }
    let (wa, wb) = (weigh(&a), weigh(&b));
    let (owners_a, owners_b) = (owner(&a), owner(&b));
    let mut segments: Vec<(bool, String, String)> = Vec::new();
    for op in capture_diff_slices(Algorithm::Myers, &wa, &wb) {
        let (tag, olds, news) = op.as_tag_tuple();
        let owned = |range: std::ops::Range<usize>, owners: &[usize]| match (
            range.start,
            range.end.checked_sub(1),
        ) {
            (start, Some(last)) if start <= last => owners[start]..owners[last] + 1,
            _ => 0..0,
        };
        let old_range = owned(olds, &owners_a);
        let new_range = owned(news, &owners_b);
        let old_text = a[old_range].concat();
        let new_text = b[new_range].concat();
        let equal = tag == DiffTag::Equal;
        match segments.last_mut() {
            Some((same, o, n)) if *same == equal => {
                o.push_str(&old_text);
                n.push_str(&new_text);
            }
            _ => segments.push((equal, old_text, new_text)),
        }
    }
    widen(segments, old, new)
        .into_iter()
        .map(|(equal, old_text, new_text)| {
            if equal {
                escape(&new_text).into_owned()
            } else {
                change(&old_text, &new_text)
            }
        })
        .collect()
}

/// A word is worth more than the space or line break it moved across.
fn weight(token: &str) -> i32 {
    if token.chars().any(char::is_alphanumeric) {
        3
    } else {
        1
    }
}

fn plain_paragraph(lines: &[&str]) -> bool {
    let Some(first) = lines.first() else {
        return false;
    };
    marker_len(first) == 0
        && !first.trim().is_empty()
        && !first.starts_with("```")
        && lines[1..].iter().all(|line| continues(line))
}

/// Aligns the lines of a hunk by content: the similar lines with the same
/// marker shapes pair up, in order; the rest are marked block by block.
fn pair<'a>(olds: &[&'a str], news: &[&'a str]) -> Vec<Entry<'a>> {
    // One line against one line is a replacement: there is nothing to
    // misalign, and a word diff reads better than two blocks. Otherwise
    // lines pair only by content, never by position: pairing by position
    // rewrote each line of a shifted block into its neighbour (#522).
    let matched = if olds.len() == 1 && news.len() == 1 {
        vec![(0, 0)]
    } else {
        pair_by_similarity(olds, news)
    };
    let mut entries = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (mi, mj) in matched
        .into_iter()
        .chain(std::iter::once((olds.len(), news.len())))
    {
        entries.extend(olds[i..mi].iter().map(|l| Entry::Only(Side::Delete, l)));
        entries.extend(news[j..mj].iter().map(|l| Entry::Only(Side::Insert, l)));
        if mi < olds.len() && mj < news.len() {
            entries.push(Entry::Pair(olds[mi], news[mj]));
        }
        i = mi + 1;
        j = mj + 1;
    }
    entries
}

/// The similar-line pairs of a hunk, in order. Candidates come from an
/// inverted index over the old lines' words: a new line is scored only
/// against old lines that share one of its rarer words, so the work is
/// bounded by the tokens in the hunk, not by the cells of a matrix. Each
/// score is the Dice coefficient of the two lines' sorted word ids, which
/// costs one merge over them. The kept pairs maximize the score sum in
/// order: a weighted longest common subsequence over the sparse set.
fn pair_by_similarity(olds: &[&str], news: &[&str]) -> Vec<(usize, usize)> {
    let mut ids = std::collections::HashMap::new();
    let mut interned: Vec<Vec<u32>> = Vec::with_capacity(olds.len() + news.len());
    for line in olds.iter().chain(news.iter()) {
        interned.push(
            body_words(line)
                .into_iter()
                .map(|word| intern(&mut ids, word))
                .collect(),
        );
    }
    let (old_words, new_words) = interned.split_at(olds.len());
    let sorted = |words: &[u32]| -> Vec<u32> {
        let mut out = words.to_vec();
        out.sort_unstable();
        out
    };
    let old_multiset: Vec<Vec<u32>> = old_words.iter().map(|w| sorted(w)).collect();
    let new_multiset: Vec<Vec<u32>> = new_words.iter().map(|w| sorted(w)).collect();
    let mut old_distinct: Vec<Vec<u32>> = old_multiset.clone();
    for words in &mut old_distinct {
        words.dedup();
    }
    let mut new_distinct: Vec<Vec<u32>> = new_multiset.clone();
    for words in &mut new_distinct {
        words.dedup();
    }
    // The old lines each word occurs in.
    let mut index: std::collections::HashMap<u32, Vec<usize>> = std::collections::HashMap::new();
    for (i, words) in old_distinct.iter().enumerate() {
        for &word in words {
            index.entry(word).or_default().push(i);
        }
    }
    let shape =
        |lines: &[&str]| -> Vec<String> { lines.iter().map(|line| marker_shape(line)).collect() };
    let (old_shape, new_shape) = (shape(olds), shape(news));
    // The candidate old lines of each new line: the lines its rarer words
    // occur in (at least its rarest word, however common that is).
    let mut candidates: Vec<Vec<usize>> = Vec::with_capacity(news.len());
    for words in &new_distinct {
        let mut by_rarity: Vec<(usize, u32)> = words
            .iter()
            .filter_map(|&word| index.get(&word).map(|lines| (lines.len(), word)))
            .collect();
        by_rarity.sort_unstable();
        let mut lines = Vec::new();
        if let Some(&(least, _)) = by_rarity.first() {
            let limit = least.max(RARE_LINE_SHARE);
            lines = by_rarity
                .iter()
                .take_while(|&&(count, _)| count <= limit)
                .flat_map(|&(_, word)| index[&word].iter().copied())
                .collect();
            lines.sort_unstable();
            lines.dedup();
            lines.truncate(MAX_CANDIDATES);
        }
        candidates.push(lines);
    }
    let mut pairs: Vec<(usize, usize, f64)> = Vec::new();
    for (j, lines) in candidates.iter().enumerate() {
        for &i in lines {
            let score = dice(&old_multiset[i], &new_multiset[j]);
            if score >= WORD_DIFF_SIMILARITY && old_shape[i] == new_shape[j] {
                pairs.push((i, j, score));
            }
        }
    }
    weighted_pairs(&pairs, news.len())
}

/// Dice coefficient of two sorted id (or word) multisets: twice the size of
/// their intersection, counting repeats, over the sizes summed.
fn dice<T: Ord>(a: &[T], b: &[T]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let best = 2.0 * a.len().min(b.len()) as f64 / (a.len() + b.len()) as f64;
    if best < WORD_DIFF_SIMILARITY {
        return 0.0;
    }
    let (mut i, mut j, mut common) = (0, 0, 0usize);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                let (word, mut more_i, mut more_j) = (&a[i], 0, 0);
                while a.get(i + more_i) == Some(word) {
                    more_i += 1;
                }
                while b.get(j + more_j) == Some(word) {
                    more_j += 1;
                }
                common += more_i.min(more_j);
                i += more_i;
                j += more_j;
            }
        }
    }
    2.0 * common as f64 / (a.len() + b.len()) as f64
}

/// The chain of `pairs` (old index, new index, score) with the greatest
/// score sum whose indices strictly increase. A Fenwick tree over the new
/// index holds, for each position, the best sum that ended before it; the
/// old lines are processed in order so a line never chains to itself.
fn weighted_pairs(pairs: &[(usize, usize, f64)], new_len: usize) -> Vec<(usize, usize)> {
    let mut order: Vec<usize> = (0..pairs.len()).collect();
    order.sort_by_key(|&at| (pairs[at].0, pairs[at].1));
    let mut best = vec![0.0f64; pairs.len()];
    let mut from = vec![usize::MAX; pairs.len()];
    let mut tree = vec![(0.0f64, usize::MAX); new_len + 1];
    let mut at = 0;
    while at < order.len() {
        let same_old = order[at..]
            .iter()
            .position(|&later| pairs[later].0 != pairs[order[at]].0)
            .map_or(order.len(), |offset| at + offset);
        for &candidate in &order[at..same_old] {
            let (mut sum, mut parent) = (0.0f64, usize::MAX);
            let mut node = pairs[candidate].1;
            while node > 0 {
                if tree[node].0 > sum {
                    sum = tree[node].0;
                    parent = tree[node].1;
                }
                node -= node & node.wrapping_neg();
            }
            best[candidate] = sum + pairs[candidate].2;
            from[candidate] = parent;
        }
        for &candidate in &order[at..same_old] {
            let mut node = pairs[candidate].1 + 1;
            while node < tree.len() {
                if best[candidate] > tree[node].0 {
                    tree[node] = (best[candidate], candidate);
                }
                node += node & node.wrapping_neg();
            }
        }
        at = same_old;
    }
    let mut tip = usize::MAX;
    for candidate in 0..pairs.len() {
        if tip == usize::MAX || best[candidate] > best[tip] {
            tip = candidate;
        }
    }
    let mut matched = Vec::new();
    if tip != usize::MAX && best[tip] > 1e-9 {
        let mut at = tip;
        while at != usize::MAX {
            matched.push((pairs[at].0, pairs[at].1));
            at = from[at];
        }
    }
    matched.reverse();
    matched
}

fn intern<'a>(ids: &mut std::collections::HashMap<&'a str, u32>, word: &'a str) -> u32 {
    let next = ids.len() as u32;
    *ids.entry(word).or_insert(next)
}

/// Words of a line after its block marker. List numbers live in the marker,
/// so they cannot pull a pairing toward the same number.
fn body_words(line: &str) -> Vec<&str> {
    words(&line[marker_len(line)..])
}

fn marker_shape(line: &str) -> String {
    let marker = &line[..marker_len(line)];
    let mut out = String::with_capacity(marker.len());
    for c in marker.chars() {
        if c.is_ascii_digit() {
            if !out.ends_with('0') {
                out.push('0');
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The share of words two lines have in common (0 to 1): twice the
/// longest common subsequence of their words over the sizes summed. The
/// subsequence comes from Myers on the words — O((n+m)·d) — not from a
/// score matrix.
pub(super) fn similarity(old: &str, new: &str) -> f64 {
    let (a, b) = (words(old), words(new));
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let best = 2.0 * a.len().min(b.len()) as f64 / (a.len() + b.len()) as f64;
    if best < WORD_DIFF_SIMILARITY {
        return 0.0;
    }
    let common = capture_diff_slices(Algorithm::Myers, &a, &b)
        .iter()
        .filter(|op| matches!(op.as_tag_tuple().0, DiffTag::Equal))
        .map(|op| op.as_tag_tuple().1.len())
        .sum::<usize>();
    2.0 * common as f64 / (a.len() + b.len()) as f64
}

fn words(line: &str) -> Vec<&str> {
    tokens(line)
        .into_iter()
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .collect()
}

/// A line in both versions: its markers, then its text word by word. A
/// marker that changed textually (a renumbered list item, `#` becoming
/// `##`, a task box ticked) takes the whole block with it: rejecting must
/// bring the old marker back, and the Word reader reads a block inside a
/// change as a block changed whole (`{~~2. b~>3. b~~}` is a deleted `2. b`
/// item and an inserted `3. b` one), never as the marker text.
fn paired(out: &mut Output, old: &str, new: &str) {
    let (old_marker, new_marker) = (&old[..marker_len(old)], &new[..marker_len(new)]);
    if old_marker != new_marker {
        out.line(&change(old, new));
        return;
    }
    let (old_text, new_text) = (&old[old_marker.len()..], &new[new_marker.len()..]);
    let mut line = escape(new_marker).into_owned();
    if similarity(old_text, new_text) < WORD_DIFF_SIMILARITY {
        line.push_str(&change(old_text, new_text));
    } else {
        line.push_str(&words_diff(old_text, new_text));
    }
    out.line(&line);
}

/// A replacement, deletion or insertion of `old` by `new`.
fn change(old: &str, new: &str) -> String {
    match (old.is_empty(), new.is_empty()) {
        (true, true) => String::new(),
        (true, false) => format!("{{++{}++}}", escape(new)),
        (false, true) => format!("{{--{}--}}", escape(old)),
        (false, false) => format!("{{~~{}~>{}~~}}", escape(old), escape(new)),
    }
}

/// `old` against `new` word by word. Changes separated only by whitespace
/// merge, so a rewritten phrase is one change rather than one per word.
fn words_diff(old: &str, new: &str) -> String {
    let a = tokens(old);
    let b = tokens(new);
    // (equal, old text, new text) segments.
    let mut segments: Vec<(bool, String, String)> = Vec::new();
    for op in capture_diff_slices(Algorithm::Myers, &a, &b) {
        let (tag, olds, news) = op.as_tag_tuple();
        let old_text = a[olds].concat();
        let new_text = b[news].concat();
        let equal = tag == DiffTag::Equal;
        match segments.last_mut() {
            Some((false, o, n)) if !equal => {
                o.push_str(&old_text);
                n.push_str(&new_text);
            }
            _ => segments.push((equal, old_text, new_text)),
        }
    }
    // Whitespace alone between two changes joins them.
    let mut merged: Vec<(bool, String, String)> = Vec::new();
    let mut index = 0;
    while index < segments.len() {
        let (equal, old_text, new_text) = segments[index].clone();
        let bridge = equal
            && old_text.trim().is_empty()
            && index > 0
            && index + 1 < segments.len()
            && matches!(merged.last(), Some((false, ..)));
        if bridge && let Some((false, o, n)) = merged.last_mut() {
            o.push_str(&old_text);
            n.push_str(&new_text);
            let (_, next_old, next_new) = &segments[index + 1];
            o.push_str(next_old);
            n.push_str(next_new);
            index += 2;
            continue;
        }
        merged.push((equal, old_text, new_text));
        index += 1;
    }
    widen(merged, old, new)
        .into_iter()
        .map(|(equal, old_text, new_text)| {
            if equal {
                escape(&new_text).into_owned()
            } else {
                change(&old_text, &new_text)
            }
        })
        .collect()
}

/// A formatting span [`atoms`] is reading: where it started, the range of
/// its text so far, and whether a change anywhere in it covers all of it
/// (an image or an autolink, whose text is as verbatim as its syntax).
struct OpenSpan {
    start: usize,
    inner: Option<(usize, usize)>,
    whole: bool,
}

/// A span that is read whole: a change that touches its `triggers` covers
/// all of `range`.
struct Atom {
    range: (usize, usize),
    triggers: Vec<(usize, usize)>,
}

/// Numbers (`$5,000,000`, `04/20/26`, `3.5%`), the delimiters of emphasis
/// and strikethrough, link syntax (all but its text), and a whole code
/// span, image or equation. The reader treats those three as literal, so a
/// mark inside one would not accept or reject.
fn atoms(text: &str) -> Vec<Atom> {
    scan_atoms(text, 0, true)
        .into_iter()
        .chain(numbers(text))
        .collect()
}

/// The parser's spans of `text`, a slice of a line at `offset`, with
/// equations read (`math`) or not. A pair of amounts the parser reads as
/// one equation (`$250,000 … **$5,000,000**`) is prose: its inside is
/// scanned again without math, so the formatting spans it hid still widen
/// changes ([`numbers`] already holds each amount whole).
fn scan_atoms(text: &str, offset: usize, math: bool) -> Vec<Atom> {
    let shift = |(start, end): (usize, usize)| (offset + start, offset + end);
    let mut out = Vec::new();
    // (start, inner range so far, whole-range trigger): the last for an
    // image or an autolink, whose text is as verbatim as its syntax.
    let mut open: Vec<OpenSpan> = Vec::new();
    let mut options = Options::ENABLE_STRIKETHROUGH;
    if math {
        options |= Options::ENABLE_MATH;
    }
    for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
        match event {
            Event::Start(tag) => match tag {
                Tag::Emphasis | Tag::Strong | Tag::Strikethrough => {
                    open.push(OpenSpan {
                        start: range.start,
                        inner: None,
                        whole: false,
                    });
                }
                Tag::Link { link_type, .. } => {
                    open.push(OpenSpan {
                        start: range.start,
                        inner: None,
                        whole: matches!(link_type, LinkType::Autolink | LinkType::Email),
                    });
                }
                Tag::Image { .. } => {
                    open.push(OpenSpan {
                        start: range.start,
                        inner: None,
                        whole: true,
                    });
                }
                _ => extend(&mut open, (range.start, range.end)),
            },
            Event::End(end) if is_atom_end(&end) => {
                if let Some(OpenSpan {
                    start,
                    inner,
                    whole,
                }) = open.pop()
                {
                    let (inner_start, inner_end) = inner.unwrap_or((range.end, range.end));
                    // An image's alt is part of the atom, not text inside
                    // it: a changed alt or target wraps the whole image. A
                    // link's text stays editable; only its syntax widens.
                    // An autolink has no editable text at all.
                    let triggers = if whole {
                        vec![shift((start, range.end))]
                    } else {
                        vec![shift((start, inner_start)), shift((inner_end, range.end))]
                    };
                    out.push(Atom {
                        range: shift((start, range.end)),
                        triggers,
                    });
                    extend(&mut open, (start, range.end));
                }
            }
            Event::Code(_) => {
                out.push(Atom {
                    range: shift((range.start, range.end)),
                    triggers: vec![shift((range.start, range.end))],
                });
                extend(&mut open, (range.start, range.end));
            }
            Event::DisplayMath(_) if math => {
                out.push(Atom {
                    range: shift((range.start, range.end)),
                    triggers: vec![shift((range.start, range.end))],
                });
                extend(&mut open, (range.start, range.end));
            }
            Event::InlineMath(_) if math && !opens_amount(text, range.start) => {
                out.push(Atom {
                    range: shift((range.start, range.end)),
                    triggers: vec![shift((range.start, range.end))],
                });
                extend(&mut open, (range.start, range.end));
            }
            Event::InlineMath(_) if math => {
                let inside = &text[range.start + 1..range.end - 1];
                out.extend(scan_atoms(inside, offset + range.start + 1, false));
                extend(&mut open, (range.start, range.end));
            }
            // An HTML comment: the reader drops it whole, so any change
            // touching it wraps it.
            Event::Html(_) | Event::InlineHtml(_) => {
                out.push(Atom {
                    range: shift((range.start, range.end)),
                    triggers: vec![shift((range.start, range.end))],
                });
                extend(&mut open, (range.start, range.end));
            }
            _ => extend(&mut open, (range.start, range.end)),
        }
    }
    out
}

/// Whether `end` closes a span [`atoms`] reads whole.
fn is_atom_end(end: &TagEnd) -> bool {
    matches!(
        end,
        TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link | TagEnd::Image
    )
}

/// Widens the innermost open span's text to cover `range`.
fn extend(open: &mut [OpenSpan], range: (usize, usize)) {
    if let Some(OpenSpan { inner, .. }) = open.last_mut() {
        *inner = Some(match inner {
            Some((start, end)) => ((*start).min(range.0), (*end).max(range.1)),
            None => range,
        });
    }
}

/// Whether `at` is the `$` of a currency amount, with a digit right after
/// it: [`numbers`] reads the sign as part of the amount, so this `$` opens
/// no equation — even where the parser pairs it with a later `$` as math
/// (`$250,000 … exceeds **$5,000,000**`).
fn opens_amount(text: &str, at: usize) -> bool {
    text.as_bytes().get(at) == Some(&b'$')
        && text.as_bytes().get(at + 1).is_some_and(u8::is_ascii_digit)
}

/// Runs of digits joined by `,` `.` `/` `:` `-` between digits, with a
/// currency sign before and a `%` after.
fn numbers(text: &str) -> Vec<Atom> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let currency = matches!(chars[at].1, '$' | '€' | '£' | '¥');
        let first = if currency { at + 1 } else { at };
        if !chars.get(first).is_some_and(|(_, c)| c.is_ascii_digit()) {
            at += 1;
            continue;
        }
        let mut end = first;
        while end < chars.len() {
            let c = chars[end].1;
            let joins = matches!(c, ',' | '.' | '/' | ':' | '-')
                && chars.get(end + 1).is_some_and(|(_, n)| n.is_ascii_digit())
                && end > first;
            if c.is_ascii_digit() || joins {
                end += 1;
            } else {
                break;
            }
        }
        if chars.get(end).is_some_and(|(_, c)| *c == '%') {
            end += 1;
        }
        let range = (chars[at].0, chars.get(end).map_or(text.len(), |(i, _)| *i));
        out.push(Atom {
            range,
            triggers: vec![range],
        });
        at = end;
    }
    out
}

/// Whether a change over `range` (empty: a point) touches `span`.
fn touches(range: (usize, usize), span: (usize, usize)) -> bool {
    if range.0 == range.1 {
        span.0 < range.0 && range.0 < span.1
    } else {
        range.0 < span.1 && span.0 < range.1
    }
}

/// Changes that touch a number, a formatting delimiter or a link's syntax,
/// widened to all of it (in both versions), so `$5,000,000` is replaced
/// whole and `*Buyer*` becoming `Buyer` reads as one change.
fn widen(
    mut segments: Vec<(bool, String, String)>,
    old: &str,
    new: &str,
) -> Vec<(bool, String, String)> {
    let (old_atoms, new_atoms) = (atoms(old), atoms(new));
    let mut index = 0;
    while index < segments.len() {
        if segments[index].0 {
            index += 1;
            continue;
        }
        loop {
            let (mut o, mut n) = (0, 0);
            for segment in &segments[..index] {
                o += segment.1.len();
                n += segment.2.len();
            }
            let (_, old_text, new_text) = &segments[index];
            let old_range = (o, o + old_text.len());
            let new_range = (n, n + new_text.len());
            let cover = |range: (usize, usize), atoms: &[Atom]| {
                atoms
                    .iter()
                    .filter(|a| a.triggers.iter().any(|t| touches(range, *t)))
                    .fold(range, |(s, e), a| (s.min(a.range.0), e.max(a.range.1)))
            };
            let (old_to, new_to) = (cover(old_range, &old_atoms), cover(new_range, &new_atoms));
            let left = (old_range.0 - old_to.0).max(new_range.0 - new_to.0);
            let right = (old_to.1 - old_range.1).max(new_to.1 - new_range.1);
            if left == 0 && right == 0 {
                break;
            }
            // Atoms lie inside the text, so there is a segment on each side
            // to take from; were there not, the widening stops.
            let before = left > 0 && index > 0;
            let after = right > 0 && index + 1 < segments.len();
            if !before && !after {
                break;
            }
            // A segment taken whole is removed: the change moves back one.
            if before && take(&mut segments, index - 1, left, true) {
                index -= 1;
            }
            if after {
                take(&mut segments, index + 1, right, false);
            }
        }
        index += 1;
    }
    segments
}

/// Moves `length` bytes (or all) of the segment at `at` into the change
/// next to it: its end into the change after it (`into_next`), or its start
/// into the change before it. A change is taken whole. Returns whether the
/// segment was taken whole (and so removed).
fn take(
    segments: &mut Vec<(bool, String, String)>,
    at: usize,
    length: usize,
    into_next: bool,
) -> bool {
    let (equal, old_text, new_text) = segments[at].clone();
    let whole = !equal || length >= old_text.len();
    let (moved_old, moved_new, kept) = if whole {
        (old_text, new_text, None)
    } else if into_next {
        let cut = old_text.floor_char_boundary(old_text.len() - length);
        (
            old_text[cut..].to_string(),
            old_text[cut..].to_string(),
            Some(old_text[..cut].to_string()),
        )
    } else {
        let cut = old_text.ceil_char_boundary(length);
        (
            old_text[..cut].to_string(),
            old_text[..cut].to_string(),
            Some(old_text[cut..].to_string()),
        )
    };
    let change = if into_next { at + 1 } else { at - 1 };
    {
        let (_, o, n) = &mut segments[change];
        if into_next {
            o.insert_str(0, &moved_old);
            n.insert_str(0, &moved_new);
        } else {
            o.push_str(&moved_old);
            n.push_str(&moved_new);
        }
    }
    match kept {
        Some(kept) => {
            segments[at] = (true, kept.clone(), kept);
            false
        }
        None => {
            segments.remove(at);
            true
        }
    }
}

/// Lines only in one version. A run of them goes into one change that takes
/// the lines and the line break before them with it — the opener rides at
/// the end of the line before the run (or starts the document's first
/// line), and the closer at the end of the run's last line — so accepting
/// and rejecting rebuild the two documents byte for byte. Lines that
/// continue the paragraph written just before take their line breaks the
/// same way, glued to that paragraph.
///
/// A run of inserted table rows marks its cells, so the rows stay rows of
/// their table; the Word reader reads the blocks inside any such change as
/// blocks changed whole ([`super::write`]).
fn group(out: &mut Output, side: Side, lines: &[&str]) {
    if lines.is_empty() {
        return;
    }
    let (open, close) = side.delimiters();
    let mut rest = lines;
    // A list item, heading or quote is not the paragraph a following
    // plain line continues: gluing them writes the new paragraph onto the
    // deleted item (`- {--hello--}{++\nhello++}`).
    if out.started && open_paragraph(&out.last) && marker_len(&out.last) == 0 && continues(rest[0])
    {
        let length = rest.iter().take_while(|l| continues(l)).count();
        let joined: Vec<Cow<'_, str>> = rest[..length].iter().map(|l| escape(l)).collect();
        out.append(&format!("{open}\n{}{close}", joined.join("\n")));
        rest = &rest[length..];
    }
    if rest.is_empty() {
        return;
    }
    // Inserted table rows keep their cells: each cell's text is one change,
    // so the row stays a row (a table drops text outside its cells).
    if side == Side::Insert && rest.iter().all(|l| table_row(l)) {
        for line in rest {
            out.line(&table_row_change(line, open, close));
        }
        return;
    }
    let run = rest;
    let doc_start = !out.started;
    // The opener rides at the end of the line before the run, or starts
    // the run's first line when that line is blank before it (a blank
    // line must stay empty: the reader splits paragraphs on it) or when
    // the document starts here, after any closer a change before it
    // carried there.
    let prev_blank = out.started && out.last.trim().is_empty();
    let mut prefix = String::new();
    if doc_start || prev_blank {
        prefix.push_str(open);
    } else if let Some(pending) = out.carry.take() {
        prefix.push_str(pending);
        prefix.push_str(open);
    } else {
        out.append(open);
    }
    // A run whose opener starts its own line — the document's first, or
    // one after a blank the change must leave empty — carries its closer
    // onto the line after it: the line breaks around such a run belong to
    // its side alone (the other document has no line break where the run
    // begins or ends), and a blank line that ends the run stays empty.
    // Every other run closes on its last line: a closer on the next line
    // would take that line's break, which the documents share.
    let carry_closer = doc_start || prev_blank;
    let last = run.len() - 1;
    for (at, line) in run.iter().enumerate() {
        let mut text = if at == 0 && !prefix.is_empty() {
            format!("{prefix}{}", escape(line))
        } else {
            escape(line).into_owned()
        };
        if at == last && !carry_closer {
            text.push_str(close);
        }
        out.line(&text);
    }
    if carry_closer {
        out.carry = Some(close);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn markers_are_found() {
        for (line, marker) in [
            ("# Title", "# "),
            ("###### Six", "###### "),
            ("####### Seven", ""),
            ("- item", "- "),
            ("  * nested", "  * "),
            ("12. twelfth", "12. "),
            ("3) third", "3) "),
            ("- [x] done", "- [x] "),
            ("> quoted", "> "),
            ("> > - deep", "> > - "),
            ("plain", ""),
            ("-5 degrees", ""),
            ("#hashtag", ""),
            ("#", "#"),
            ("[^note]: text", "[^note]: "),
            ("[^a b]: text", ""),
        ] {
            assert_eq!(&line[..marker_len(line)], marker, "{line}");
        }
    }

    #[test]
    fn a_rewritten_phrase_is_one_change() {
        assert_eq!(
            words_diff("the quick brown fox", "the slow red fox"),
            "the {~~quick brown~>slow red~~} fox"
        );
        assert_eq!(words_diff("a b", "a b c"), "a b{++ c++}");
        assert_eq!(words_diff("a b c", "a c"), "a {--b --}c");
    }

    #[test]
    fn a_decomposed_accent_is_part_of_the_word_replaced() {
        // "cafe" + U+0301 and "the" + U+0301: one word replaced, not two
        // words whose shared combining mark is left outside the change.
        let (old, new) = ("Le cafe\u{301} est chaud.\n", "Le the\u{301} est chaud.\n");
        assert_eq!(
            diff_markdown(old, new),
            "Le {~~cafe\u{301}~>the\u{301}~~} est chaud.\n"
        );
        // The same change in precomposed text reads alike.
        assert_eq!(
            diff_markdown("Le caf\u{e9} est chaud.\n", "Le th\u{e9} est chaud.\n"),
            "Le {~~caf\u{e9}~>th\u{e9}~~} est chaud.\n"
        );
        // Only an accent added: the whole word changes, in either form.
        assert_eq!(
            words_diff("un cafe est", "un cafe\u{301} est"),
            "un {~~cafe~>cafe\u{301}~~} est"
        );
    }

    #[test]
    fn a_change_inside_a_number_is_the_whole_number() {
        for (old, new, expected) in [
            (
                "exceeds **$250,000** now",
                "exceeds **$150,000** now",
                "exceeds **{~~$250,000~>$150,000~~}** now",
            ),
            (
                "on 04/20/26.",
                "on 10/30/26.",
                "on {~~04/20/26~>10/30/26~~}.",
            ),
            ("grew 3.5% a", "grew 4.25% a", "grew {~~3.5%~>4.25%~~} a"),
            (
                "cap of $5,000,000 total",
                "cap of $7,500,000 total",
                "cap of {~~$5,000,000~>$7,500,000~~} total",
            ),
            ("5 days", "5 business days", "5 {++business ++}days"),
            // Next to a number, not in it.
            ("5 days", "5, days", "5{++,++} days"),
        ] {
            assert_eq!(words_diff(old, new), expected, "{old}");
        }
    }

    #[test]
    fn a_formatting_change_is_the_whole_span() {
        for (old, new, expected) in [
            (
                "harmless *Buyer*, its",
                "harmless Buyer, its",
                "harmless {~~*Buyer*~>Buyer~~}, its",
            ),
            (
                "the \"Basket\")",
                "the \"**Basket**\")",
                "the \"{~~Basket~>**Basket**~~}\")",
            ),
            // Text changed inside emphasis stays inside it.
            (
                "*provided, however*, that",
                "*provided, further*, that",
                "*provided, {~~however~>further~~}*, that",
            ),
        ] {
            assert_eq!(words_diff(old, new), expected, "{old}");
        }
    }

    #[test]
    fn a_changed_link_target_is_the_whole_link() {
        assert_eq!(
            words_diff(
                "in [Section 7.4](#section-7-4).",
                "in [Section 7.5](#section-7-5)."
            ),
            "in {~~[Section 7.4](#section-7-4)~>[Section 7.5](#section-7-5)~~}."
        );
        // Only the link text: the change stays in it.
        assert_eq!(
            words_diff("see [the old terms](x)", "see [the new terms](x)"),
            "see [the {~~old~>new~~} terms](x)"
        );
    }

    #[test]
    fn delimiters_in_the_text_are_escaped() {
        assert_eq!(escape("a {++ b ++} ~> c"), "a \\{++ b ++\\} ~\\> c");
        assert_eq!(
            diff_markdown("x {++y++}\n", "x {++z++}\n"),
            "x \\{++{~~y~>z~~}++\\}\n"
        );
    }

    #[test]
    fn identical_documents_come_back_unchanged() {
        let text = "# Title\n\n- a\n- b\n\nText.\n";
        assert_eq!(diff_markdown(text, text), text);
        assert_eq!(diff_markdown("", ""), "");
    }

    #[test]
    fn blocks_added_or_removed_keep_their_markers_outside() {
        assert_eq!(
            diff_markdown("- a\n- c\n", "- a\n- b\n- c\n"),
            "- a{++\n- b++}\n- c\n"
        );
        assert_eq!(
            diff_markdown("A\n\nOld paragraph.\n\nC\n", "A\n\nC\n"),
            "A\n\n{--Old paragraph.\n\n--}C\n"
        );
        assert_eq!(
            diff_markdown("# One\n", "# One\n\n## Two\n"),
            "# One{++\n\n## Two++}\n"
        );
    }

    #[test]
    fn a_line_added_inside_a_paragraph_takes_its_line_break() {
        assert_eq!(
            diff_markdown("first\nthird\n", "first\nsecond\nthird\n"),
            "first{++\nsecond++}\nthird\n"
        );
        assert_eq!(diff_markdown("x\ny\n", "y\n"), "{--x\n--}y\n");
    }

    #[test]
    fn a_changed_marker_replaces_the_block() {
        // The marker is the change, and it takes the whole block with it:
        // rejecting brings `# Title` back, accepting gives `## Title`, and
        // the Word reader reads each side as a heading of its own level.
        assert_eq!(
            diff_markdown("# Title\n", "## Title\n"),
            "{~~# Title~>## Title~~}\n"
        );
        round_trip("# Title\n", "## Title\n");
        // A task box ticked, a renumbered item: the same whole-block change.
        assert_eq!(
            diff_markdown("- [ ] write\n", "- [x] write\n"),
            "{~~- [ ] write~>- [x] write~~}\n"
        );
        round_trip("- [ ] write\n", "- [x] write\n");
        // A list item turned into a paragraph: the item, marker and all,
        // against the paragraph.
        assert_eq!(
            diff_markdown("- hello world\n", "hello world\n"),
            "{~~- hello world~>hello world~~}\n"
        );
        round_trip("- hello world\n", "hello world\n");
    }

    #[test]
    fn a_table_row_changes_cell_by_cell() {
        assert_eq!(
            diff_markdown(
                "| a |\n|---|\n| 1 |\n",
                "| a |\n|---|\n| 1 |\n| 2 \\| 3 |\n"
            ),
            "| a |\n|---|\n| 1 |\n| {++2 \\| 3++} |\n"
        );
        assert_eq!(table_row_change("|---|:-:|", "{++", "++}"), "|---|:-:|");
    }

    #[test]
    fn unrelated_lines_are_replaced_whole() {
        assert_eq!(
            diff_markdown("Lorem ipsum dolor.\n", "Completely other words here.\n"),
            "{~~Lorem ipsum dolor.~>Completely other words here.~~}\n"
        );
    }

    fn round_trip(old: &str, new: &str) {
        let diff = diff_markdown(old, new);
        assert_eq!(
            crate::markdown::resolve_critic(&diff, crate::markdown::TrackChanges::Accept),
            new,
            "accept {diff:?}"
        );
        assert_eq!(
            crate::markdown::resolve_critic(&diff, crate::markdown::TrackChanges::Reject),
            old,
            "reject {diff:?}"
        );
    }

    #[test]
    fn a_changed_code_span_is_wrapped_whole() {
        let (old, new) = ("Call `foo()` now.\n", "Call `bar()` now.\n");
        assert_eq!(
            diff_markdown(old, new),
            "Call {~~`foo()`~>`bar()`~~} now.\n"
        );
        // Delimiters inside the span stay literal: escaping them would
        // change the code the reader gives back.
        let (old, new) = ("use `{++}` here.\n", "use `{--}` here.\n");
        assert_eq!(diff_markdown(old, new), "use {~~`{++}`~>`{--}`~~} here.\n");
        round_trip(old, new);
        round_trip("Call `foo()` now.\n", "Call `bar()` now.\n");
    }

    #[test]
    fn a_changed_image_is_wrapped_whole() {
        let target = (
            "See ![cat](old.png) now.\n",
            "See ![cat](new.png) now.\n",
            "See {~~![cat](old.png)~>![cat](new.png)~~} now.\n",
        );
        let alt = (
            "See ![old](a.png) now.\n",
            "See ![new](a.png) now.\n",
            "See {~~![old](a.png)~>![new](a.png)~~} now.\n",
        );
        for (old, new, expected) in [target, alt] {
            assert_eq!(diff_markdown(old, new), expected, "{old}");
            round_trip(old, new);
        }
    }

    #[test]
    fn a_changed_equation_is_wrapped_whole() {
        for (old, new, expected) in [
            (
                "Area is $a+b$ now.\n",
                "Area is $a+c$ now.\n",
                "Area is {~~$a+b$~>$a+c$~~} now.\n",
            ),
            (
                "Area is $$a+b$$ now.\n",
                "Area is $$a+c$$ now.\n",
                "Area is {~~$$a+b$$~>$$a+c$$~~} now.\n",
            ),
        ] {
            assert_eq!(diff_markdown(old, new), expected, "{old}");
            round_trip(old, new);
        }
    }

    #[test]
    fn escape_counts_backslashes_and_skips_code() {
        // An odd run of backslashes already escapes the opener. Adding
        // another would make the reader see a real change.
        assert_eq!(escape(r"\{++"), r"\{++");
        assert_eq!(escape(r"\\{++"), r"\\\{++");
        assert_eq!(escape(r"\~>"), r"\~>");
        assert_eq!(escape("a {++ b ++} ~> c"), "a \\{++ b ++\\} ~\\> c");
        // Inline code is literal, so a delimiter there is not escaped.
        assert_eq!(escape("a `{++}` b"), "a `{++}` b");
        assert_eq!(escape("a `{++}` {++} b"), "a `{++}` \\{++\\} b");
        let text = "call `{++}` now\n";
        assert_eq!(diff_markdown(text, text), text);
    }

    #[test]
    fn twenty_one_changed_lines_and_an_insertion_pair_by_content() {
        // 21×22 comparisons is over the old 400 cap, so pairing fell back
        // to position and each line was rewritten into its neighbour.
        let old: String = (0..21)
            .map(|i| format!("item {i:02} stays unique here\n"))
            .collect();
        let new: String = std::iter::once("item inserted at the top\n".to_string())
            .chain((0..21).map(|i| format!("item {i:02} stays unique there\n")))
            .collect();
        let diff = diff_markdown(&old, &new);
        // The inserted line continues into the paragraph, so its line break
        // belongs to the insertion.
        assert!(
            diff.starts_with(
                "{++item inserted at the top\n++}item 00 stays unique {~~here~>there~~}\n"
            ),
            "{diff}"
        );
        assert!(
            diff.contains("item 20 stays unique {~~here~>there~~}"),
            "{diff}"
        );
        assert!(!diff.contains("inserted at the top~>"), "{diff}");
        round_trip(&old, &new);
    }

    #[test]
    fn a_large_changed_hunk_still_pairs_similar_lines() {
        let n = 1500;
        let old: String = (0..n)
            .map(|i| format!("row {i} alpha beta gamma\n"))
            .collect();
        let new: String = std::iter::once("row inserted zeta eta theta\n".to_string())
            .chain((0..n).map(|i| format!("row {i} alpha beta delta\n")))
            .collect();
        let diff = diff_markdown(&old, &new);
        assert!(
            diff.contains("{++row inserted zeta eta theta\n++}row 0 alpha beta {~~gamma~>delta~~}"),
            "{}",
            &diff[..diff.len().min(400)]
        );
        assert!(diff.contains("row 1499 alpha beta {~~gamma~>delta~~}"));
    }

    #[test]
    fn list_numbers_do_not_skew_pairing() {
        let old = "1. alpha\n2. beta\n3. gamma\n";
        let new = "1. inserted\n2. alpha\n3. beta\n4. gamma\n";
        assert_eq!(
            diff_markdown(old, new),
            "{++1. inserted\n++}{~~1. alpha~>2. alpha~~}\n{~~2. beta~>3. beta~~}\n{~~3. gamma~>4. gamma~~}\n"
        );
        round_trip(old, new);
    }

    #[test]
    fn a_list_item_turned_into_a_paragraph_is_not_glued() {
        let old = "- hello world\n";
        let new = "hello world\n";
        assert_eq!(
            diff_markdown(old, new),
            "{~~- hello world~>hello world~~}\n"
        );
        round_trip(old, new);
    }

    /// Words of `text` wrapped to `width` columns, the way a hard wrap
    /// joins lines back with a space.
    fn wrap_words(text: &str, width: usize) -> String {
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in text.split_whitespace() {
            if !line.is_empty() && line.len() + 1 + word.len() > width {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines.push(String::new());
        lines.join("\n")
    }

    #[test]
    fn one_word_added_to_a_hard_wrapped_paragraph_is_one_change() {
        let base = "The quick brown fox jumps over the lazy dog and then runs around the old barn before sunset while the farmer watches quietly from the porch.";
        let added = base.replacen("lazy", "very lazy", 1);
        for width in [20, 24, 28, 32, 40, 48] {
            let old = wrap_words(base, width);
            let new = wrap_words(&added, width);
            let diff = diff_markdown(&old, &new);
            round_trip(&old, &new);
            assert!(
                diff.contains("{++very") || diff.contains("{++ very"),
                "width {width}: the added word is not one insertion\n{diff}"
            );
            // A word that only moved onto the next line is not rewritten.
            for word in ["runs", "barn", "while", "watches"] {
                assert!(
                    !diff.contains(&format!("{{--{word}"))
                        && !diff.contains(&format!("{{++{word}")),
                    "width {width}: {word} was rewritten\n{diff}"
                );
            }
        }
    }

    #[test]
    fn a_changed_list_number_is_marked() {
        // The marker is part of the change: rejecting must bring `10. a`
        // back, so the whole item is in the substitution.
        assert_eq!(diff_markdown("10. a\n", "9. a\n"), "{~~10. a~>9. a~~}\n");
        round_trip("10. a\n", "9. a\n");
    }

    #[test]
    fn marker_shapes_change_whole() {
        for (old, new) in [
            ("# Title\n", "## Title\n"),
            ("* a\n", "+ a\n"),
            ("- a\n", "  - a\n"),
        ] {
            round_trip(old, new);
        }
    }

    #[test]
    fn blank_lines_added_and_removed_are_marked() {
        assert_eq!(diff_markdown("a\nb\n", "a\n\nb\n"), "a{++\n++}\nb\n");
        assert_eq!(diff_markdown("a\n\nb\n", "a\nb\n"), "a{--\n--}\nb\n");
        // Two blanks become one: the removed blank line is a deletion.
        assert_eq!(diff_markdown("x\n\n\ny\n", "x\n\ny\n"), "x\n\n{--\n--}y\n");
        round_trip("a\nb\n", "a\n\nb\n");
        round_trip("a\n\nb\n", "a\nb\n");
        round_trip("x\n\n\ny\n", "x\n\ny\n");
    }

    #[test]
    fn the_trailing_newline_is_part_of_the_change() {
        assert_eq!(diff_markdown("a", "a\n"), "a{++\n++}");
        assert_eq!(diff_markdown("a\n", "a"), "a{--\n--}");
        round_trip("a", "a\n");
        round_trip("a\n", "a");
    }

    #[test]
    fn a_deleted_table_row_takes_its_line_with_it() {
        assert_eq!(
            diff_markdown("| a |\n|---|\n| 1 |\n", "| a |\n|---|\n"),
            "| a |\n|---|{--\n| 1 |--}\n"
        );
        round_trip("| a |\n|---|\n| 1 |\n", "| a |\n|---|\n");
        // A row deleted before another row keeps that row a line of its own.
        assert_eq!(
            diff_markdown("| a |\n|---|\n| 1 |\n| 2 |\n", "| a |\n|---|\n| 2 |\n"),
            "| a |\n|---|{--\n| 1 |--}\n| 2 |\n"
        );
        round_trip("| a |\n|---|\n| 1 |\n| 2 |\n", "| a |\n|---|\n| 2 |\n");
    }

    #[test]
    fn a_code_span_across_a_line_break_is_one_atom() {
        let (old, new) = ("x `a\nb` y\n", "x `a\nc` y\n");
        assert_eq!(diff_markdown(old, new), "x {~~`a\nb`~>`a\nc`~~} y\n");
        round_trip(old, new);
    }

    #[test]
    fn a_multiline_display_equation_is_wrapped_whole() {
        let (old, new) = (
            "Area is\n$$\na+b\n$$\nnow.\n",
            "Area is\n$$\na+c\n$$\nnow.\n",
        );
        let diff = diff_markdown(old, new);
        assert!(diff.contains("{~~$$\na+b\n$$~>$$\na+c\n$$~~}"), "{diff}");
        round_trip(old, new);
    }

    #[test]
    fn identical_documents_with_verbatim_delimiters_come_back_unchanged() {
        for text in [
            "<!-- {++x++} -->\ntext\n",
            "see <https://a.b/{++x> end\n",
            "see [t](https://a.b/{++x) more\n",
            "x `a\ncode {++i++} rest` y\n",
        ] {
            assert_eq!(diff_markdown(text, text), text, "{text:?}");
        }
    }

    #[test]
    fn verbatim_constructs_change_whole() {
        // The reader passes over autolinks, destinations and comments, so a
        // change inside one wraps the whole construct.
        round_trip("see <https://a.b/one> end\n", "see <https://a.b/two> end\n");
        round_trip(
            "see [t](https://a.b/one) more\n",
            "see [t](https://a.b/two) more\n",
        );
        round_trip("<!-- note one -->\ntext\n", "<!-- note two -->\ntext\n");
    }

    #[test]
    fn escape_skips_the_readers_verbatim_ranges() {
        assert_eq!(escape("<!-- {++x++} -->"), "<!-- {++x++} -->");
        assert_eq!(escape("see <https://a.b/{++x>"), "see <https://a.b/{++x>");
        assert_eq!(
            escape("see [t](https://a.b/{++x)"),
            "see [t](https://a.b/{++x)"
        );
        // A code span that crosses a single newline is verbatim too.
        assert_eq!(
            escape("x `a\ncode {++i++} rest` y"),
            "x `a\ncode {++i++} rest` y"
        );
        // Outside them, delimiters are still escaped.
        assert_eq!(escape("a {++ b ++} ~> c"), "a \\{++ b ++\\} ~\\> c");
    }

    #[test]
    fn accept_and_reject_rebuild_the_documents_exactly() {
        // A fixed table of generated cases: every family the probes showed
        // breaking the byte-exact invariant.
        let cases: &[(&str, &str)] = &[
            // Marker changes.
            ("10. a\n", "9. a\n"),
            ("* a\n", "+ a\n"),
            ("# t\n", "## t\n"),
            ("- a\n", "  - a\n"),
            // Blank-line insertions and removals.
            ("a\nb\n", "a\n\nb\n"),
            ("a\n\nb\n", "a\nb\n"),
            ("x\n\n\ny\n", "x\n\ny\n"),
            ("a\n\nb\nc\n", "a\nb\n\nc\n"),
            // Trailing newline added and removed.
            ("a", "a\n"),
            ("a\n", "a"),
            ("", "a\n"),
            ("a\n", ""),
            // Table row deletion.
            ("| a |\n|---|\n| 1 |\n", "| a |\n|---|\n"),
            ("| a |\n|---|\n| 1 |\n| 2 |\n", "| a |\n|---|\n| 2 |\n"),
            // List item and paragraph.
            ("- hello world\n", "hello world\n"),
            ("hello world\n", "- hello world\n"),
            ("a b c\n", "a b c\n- d\n"),
            // Code spans across a line break.
            ("x `a\nb` y\n", "x `a\nc` y\n"),
            ("x `a\nb` y\n", "x `a\nb` z\n"),
            // HTML comments, autolinks, link destinations.
            ("<!-- note one -->\ntext\n", "<!-- note two -->\ntext\n"),
            ("see <https://a.b/one> end\n", "see <https://a.b/two> end\n"),
            (
                "see [t](https://a.b/one) more\n",
                "see [t](https://a.b/two) more\n",
            ),
            // Inline and display math, also across lines.
            ("Area is $a+b$ now.\n", "Area is $a+c$ now.\n"),
            ("Area is $$a+b$$ now.\n", "Area is $$a+c$$ now.\n"),
            (
                "Area is\n$$\na+b\n$$\nnow.\n",
                "Area is\n$$\na+c\n$$\nnow.\n",
            ),
        ];
        for (old, new) in cases {
            round_trip(old, new);
        }
    }

    #[test]
    fn escape_is_linear_on_backtick_runs() {
        // 100k unclosed backtick runs: each must not rescan to the end.
        let mut text = String::with_capacity(400_000);
        for _ in 0..100_000 {
            text.push_str("`x");
        }
        text.push_str("{++}");
        let escaped = escape(&text);
        // The opener and the closer each take a backslash; the run of
        // unclosed backticks before them costs one pass.
        assert_eq!(escaped.len(), text.len() + 2);
    }

    #[test]
    fn a_large_rewrap_stays_fast() {
        // A 20k-word paragraph hard-wrapped at two widths: every line
        // differs, and the word diff must stay near-linear.
        let words: Vec<String> = (0..20_000).map(|i| format!("w{i}x")).collect();
        let text = words.join(" ");
        let wrap = |width: usize| {
            let mut lines = Vec::new();
            let mut line = String::new();
            for word in text.split(' ') {
                if !line.is_empty() && line.len() + 1 + word.len() > width {
                    lines.push(std::mem::take(&mut line));
                }
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
            if !line.is_empty() {
                lines.push(line);
            }
            lines.push(String::new());
            lines.join("\n")
        };
        let (old, new) = (wrap(40), wrap(45));
        round_trip(&old, &new);
    }

    #[test]
    fn long_lines_pair_in_linear_time() {
        // 200 lines of 252 words each: the similarity of a candidate pair
        // must cost O(words), not an O(w^2) subsequence.
        let lines = |tail: &str| -> String {
            (0..200)
                .map(|i| {
                    let mut line = format!("row{i}");
                    for w in 0..250 {
                        line.push_str(&format!(" w{w}"));
                    }
                    format!("{line} {tail}\n")
                })
                .collect()
        };
        let (old, new) = (lines("tail"), lines("tailx"));
        let diff = diff_markdown(&old, &new);
        assert!(diff.contains("row0 w0"), "{}", &diff[..diff.len().min(200)]);
        round_trip(&old, &new);
    }

    #[test]
    fn just_over_the_old_bound_pairs_by_content() {
        // 2001 by 2000 lines was just over the removed 4M-cell bound and
        // used to fall back to pairing by position, rewriting every line
        // into its neighbour.
        let old: String = (0..2001)
            .map(|i| format!("item {i:04} alpha beta gamma\n"))
            .collect();
        let new: String = std::iter::once("inserted first\n".to_string())
            .chain((0..2000).map(|i| format!("item {i:04} alpha beta delta\n")))
            .collect();
        let diff = diff_markdown(&old, &new);
        assert!(
            diff.contains("item 0000 alpha beta {~~gamma~>delta~~}"),
            "{diff}"
        );
        assert!(
            !diff.contains("item {~~0001~>0000~~}"),
            "a line was rewritten into its neighbour\n{diff}"
        );
        round_trip(&old, &new);
    }
}
