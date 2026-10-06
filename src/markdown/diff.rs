// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The changes between two Markdown documents as CriticMarkup, as pandiff
//! writes them.
//!
//! Lines are aligned first, then words within the lines that pair up. A
//! change never starts a line: a list, heading or quote marker stays before
//! it (`- {++new item++}`, `# {~~Old~>New~~} title`), so the marked-up
//! document keeps the structure of both versions, and a block added or
//! removed whole is one change over its whole text, which
//! [`markdown_to_docx`](super::markdown_to_docx) writes as a paragraph added
//! or removed whole. A line added or removed inside a paragraph takes the
//! line break with it (`first line{++\nnew line++}`).

use std::borrow::Cow;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use similar::{Algorithm, DiffTag, capture_diff_slices};

use crate::util::word_tokens as tokens;

/// Lines that share less than this share of their words are replaced whole
/// rather than word by word.
const WORD_DIFF_SIMILARITY: f64 = 0.3;

/// Pairing lines in a changed hunk compares every old line with every new
/// one; above this many comparisons lines pair by position.
const MAX_PAIRINGS: usize = 400;

/// `old` and `new` as one Markdown document whose CriticMarkup holds the
/// changes: accepting them all gives `new`, rejecting them all gives `old`.
/// CriticMarkup delimiters already in the text are escaped, so they stay
/// text.
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
    for (index, op) in ops.iter().enumerate() {
        let (tag, olds, news) = op.as_tag_tuple();
        match tag {
            DiffTag::Equal => {
                for line in &new_lines[news] {
                    out.line(&escape(line));
                }
            }
            DiffTag::Delete | DiffTag::Insert | DiffTag::Replace => {
                // The unchanged line after the hunk, which a change that ends
                // inside its paragraph closes on.
                let following = ops
                    .get(index + 1)
                    .map(|next| next.new_range().start)
                    .and_then(|at| new_lines.get(at).copied());
                hunk(&mut out, &old_lines[olds], &new_lines[news], following);
            }
        }
    }
    out.finish(new.ends_with('\n') || (new.is_empty() && old.ends_with('\n')))
}

fn lines(text: &str) -> Vec<&str> {
    let text = text.strip_suffix('\n').unwrap_or(text);
    if text.is_empty() {
        return Vec::new();
    }
    text.split('\n').collect()
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
fn table_row_change(line: &str, open: &str, close: &str) -> String {
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
fn escape(text: &str) -> Cow<'_, str> {
    const DELIMITERS: [(&str, &str); 11] = [
        ("{++", "\\{++"),
        ("{--", "\\{--"),
        ("{~~", "\\{~~"),
        ("{==", "\\{=="),
        ("{>>", "\\{>>"),
        ("++}", "++\\}"),
        ("--}", "--\\}"),
        ("~~}", "~~\\}"),
        ("==}", "==\\}"),
        ("<<}", "<<\\}"),
        ("~>", "~\\>"),
    ];
    if !DELIMITERS.iter().any(|(d, _)| text.contains(d)) {
        return Cow::Borrowed(text);
    }
    let mut out = text.to_string();
    for (delimiter, escaped) in DELIMITERS {
        out = out.replace(delimiter, escaped);
    }
    Cow::Owned(out)
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

    fn finish(mut self, newline: bool) -> String {
        if let Some(close) = self.carry.take() {
            self.text.push_str(close);
        }
        if newline && self.started {
            self.text.push('\n');
        }
        self.text
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

fn hunk(out: &mut Output, olds: &[&str], news: &[&str], following: Option<&str>) {
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
                let next = match entries.get(end) {
                    Some(Entry::Pair(_, new)) => Some(*new),
                    Some(Entry::Only(_, line)) => Some(*line),
                    None => following,
                };
                let deleted = only(Side::Delete);
                let inserted = only(Side::Insert);
                // Only the last group may close on the next line.
                group(
                    out,
                    Side::Delete,
                    &deleted,
                    next.filter(|_| inserted.is_empty()),
                );
                group(out, Side::Insert, &inserted, next);
                at = end;
            }
        }
    }
}

/// Aligns the lines of a hunk: the most similar lines with the same markers
/// pair up, in order; lines left between two pairs pair by position when
/// their markers match.
fn pair<'a>(olds: &[&'a str], news: &[&'a str]) -> Vec<Entry<'a>> {
    let mut matched: Vec<(usize, usize)> = Vec::new();
    if olds.len() * news.len() <= MAX_PAIRINGS {
        // Weighted longest common subsequence over similar lines.
        let (k, m) = (olds.len(), news.len());
        let mut score = vec![vec![0.0f64; m + 1]; k + 1];
        for i in (0..k).rev() {
            for j in (0..m).rev() {
                let s = similarity(olds[i], news[j]);
                let take = if s >= WORD_DIFF_SIMILARITY && same_marker(olds[i], news[j]) {
                    s + score[i + 1][j + 1]
                } else {
                    f64::MIN
                };
                score[i][j] = take.max(score[i + 1][j]).max(score[i][j + 1]);
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < k && j < m {
            let s = similarity(olds[i], news[j]);
            if s >= WORD_DIFF_SIMILARITY
                && same_marker(olds[i], news[j])
                && (score[i][j] - (s + score[i + 1][j + 1])).abs() < 1e-9
            {
                matched.push((i, j));
                i += 1;
                j += 1;
            } else if (score[i][j] - score[i + 1][j]).abs() < 1e-9 {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
    // Lines left between two pairs (or at the ends) pair by position.
    let mut entries = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (mi, mj) in matched
        .into_iter()
        .chain(std::iter::once((olds.len(), news.len())))
    {
        while i < mi && j < mj && same_marker(olds[i], news[j]) && !olds[i].trim().is_empty() {
            entries.push(Entry::Pair(olds[i], news[j]));
            i += 1;
            j += 1;
        }
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

/// Whether two lines open the same kind of block. An ordered list's numbers
/// do not count: Word numbers items itself.
fn same_marker(old: &str, new: &str) -> bool {
    fn shape(marker: &str) -> String {
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
    shape(&old[..marker_len(old)]) == shape(&new[..marker_len(new)])
}

/// The share of words two lines have in common (0 to 1).
pub(super) fn similarity(old: &str, new: &str) -> f64 {
    let a: Vec<&str> = words(old);
    let b: Vec<&str> = words(new);
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let common: usize = capture_diff_slices(Algorithm::Myers, &a, &b)
        .iter()
        .filter(|op| op.as_tag_tuple().0 == DiffTag::Equal)
        .map(|op| op.old_range().len())
        .sum();
    2.0 * common as f64 / (a.len() + b.len()) as f64
}

fn words(line: &str) -> Vec<&str> {
    tokens(line)
        .into_iter()
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .collect()
}

/// A line in both versions: its markers, then its text word by word.
fn paired(out: &mut Output, old: &str, new: &str) {
    let marker = &new[..marker_len(new)];
    let (old_text, new_text) = (&old[marker_len(old)..], &new[marker_len(new)..]);
    let mut line = escape(marker).into_owned();
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

/// A span that is read whole: a change that touches its `triggers` covers
/// all of `range`.
struct Atom {
    range: (usize, usize),
    triggers: Vec<(usize, usize)>,
}

/// Numbers (`$5,000,000`, `04/20/26`, `3.5%`), the delimiters of emphasis,
/// strikethrough and code, and the syntax of links (all but their text).
fn atoms(text: &str) -> Vec<Atom> {
    let mut out = numbers(text);
    let mut open: Vec<(usize, Option<(usize, usize)>)> = Vec::new();
    let options = Options::ENABLE_STRIKETHROUGH;
    for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
        match event {
            Event::Start(Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. }) => {
                open.push((range.start, None));
            }
            Event::End(
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link,
            ) => {
                if let Some((start, inner)) = open.pop() {
                    let (inner_start, inner_end) = inner.unwrap_or((range.end, range.end));
                    out.push(Atom {
                        range: (start, range.end),
                        triggers: vec![(start, inner_start), (inner_end, range.end)],
                    });
                    extend(&mut open, (start, range.end));
                }
            }
            Event::Code(_) => {
                let ticks = text[range.clone()]
                    .bytes()
                    .take_while(|b| *b == b'`')
                    .count();
                out.push(Atom {
                    range: (range.start, range.end),
                    triggers: vec![
                        (range.start, range.start + ticks),
                        (range.end - ticks, range.end),
                    ],
                });
                extend(&mut open, (range.start, range.end));
            }
            _ => extend(&mut open, (range.start, range.end)),
        }
    }
    out
}

/// Widens the innermost open span's text to cover `range`.
fn extend(open: &mut [(usize, Option<(usize, usize)>)], range: (usize, usize)) {
    if let Some((_, inner)) = open.last_mut() {
        *inner = Some(match inner {
            Some((start, end)) => ((*start).min(range.0), (*end).max(range.1)),
            None => range,
        });
    }
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

/// Lines only in one version. Lines that continue the paragraph written
/// just before take its line break into the change; the rest are marked
/// block by block, markers outside, and the last block takes the line break
/// before `next` when `next` continues it.
fn group(out: &mut Output, side: Side, lines: &[&str], next: Option<&str>) {
    if lines.is_empty() {
        return;
    }
    let (open, close) = side.delimiters();
    let mut rest = lines;
    if out.started && open_paragraph(&out.last) && continues(rest[0]) {
        let length = rest.iter().take_while(|l| continues(l)).count();
        let joined: Vec<Cow<'_, str>> = rest[..length].iter().map(|l| escape(l)).collect();
        out.append(&format!("{open}\n{}{close}", joined.join("\n")));
        rest = &rest[length..];
    }
    let mut at = 0;
    while at < rest.len() {
        let line = rest[at];
        if line.trim().is_empty() {
            out.line(line);
            at += 1;
            continue;
        }
        // A block: this line and the lines that continue it.
        let length = 1 + rest[at + 1..].iter().take_while(|l| continues(l)).count();
        let block = &rest[at..at + length];
        if table_row(line) {
            out.line(&table_row_change(line, open, close));
            at += 1;
            continue;
        }
        let marker = marker_len(block[0]);
        let first = &block[0][marker..];
        // A block that follows other text of its paragraph gets a blank line,
        // so it stays a block of its own.
        if out.started && open_paragraph(&out.last) && marker == 0 {
            out.line("");
        }
        if first.is_empty() && block.len() == 1 {
            out.line(&escape(block[0]));
            at += length;
            continue;
        }
        let mut text = format!("{}{open}{}", escape(&block[0][..marker]), escape(first));
        for line in &block[1..] {
            out.line(&text);
            text = escape(line).into_owned();
        }
        let last_block = at + length == rest.len();
        if last_block && next.is_some_and(continues) {
            out.line(&text);
            out.carry = Some(close);
        } else {
            text.push_str(close);
            out.line(&text);
        }
        at += length;
    }
}

#[cfg(test)]
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
            "- a\n- {++b++}\n- c\n"
        );
        assert_eq!(
            diff_markdown("A\n\nOld paragraph.\n\nC\n", "A\n\nC\n"),
            "A\n\n{--Old paragraph.--}\n\nC\n"
        );
        assert_eq!(
            diff_markdown("# One\n", "# One\n\n## Two\n"),
            "# One\n\n## {++Two++}\n"
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
        assert_eq!(
            diff_markdown("# Title\n", "## Title\n"),
            "# {--Title--}\n## {++Title++}\n"
        );
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
}
