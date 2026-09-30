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

use similar::{Algorithm, DiffTag, capture_diff_slices};

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
fn marker_len(line: &str) -> usize {
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
fn continues(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.is_empty() && marker_len(line) == 0 && !trimmed.starts_with("```") && !table_row(line)
}

/// Whether text may continue after a line: not blank, not a heading, a
/// table row or a code fence.
fn open_paragraph(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.is_empty()
        && !trimmed.starts_with('#')
        && !trimmed.starts_with("```")
        && !table_row(line)
}

/// A table row: the line starts with a pipe.
fn table_row(line: &str) -> bool {
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
fn similarity(old: &str, new: &str) -> f64 {
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

/// Words, runs of whitespace, and punctuation (a run of one mark, such as
/// `**`, is one token).
fn tokens(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut previous: Option<char> = None;
    for (at, c) in text.char_indices() {
        if let Some(p) = previous {
            let same_class = (p.is_alphanumeric() && c.is_alphanumeric())
                || (p.is_whitespace() && c.is_whitespace())
                || (!p.is_alphanumeric() && !p.is_whitespace() && c == p);
            if !same_class {
                out.push(&text[start..at]);
                start = at;
            }
        }
        previous = Some(c);
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
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
    merged
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
    fn tokens_split_words_spaces_and_marks() {
        assert_eq!(
            tokens("**Bold** words, and 3.5%!"),
            [
                "**", "Bold", "**", " ", "words", ",", " ", "and", " ", "3", ".", "5", "%", "!"
            ]
        );
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
