// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Git/GitHub unified patches of complete document text snapshots.
//!
//! DOCX snapshots use the same paragraph, table, and revision-mark view as
//! `debug --check text`, without clipping or a hunk limit. These are text
//! patches for review; they do not patch the binary DOCX package.

use crate::markdown::Source;
use similar::{ChangeTag, DiffOp, DiffTag, TextDiff};
use std::fmt::Write as _;
use std::ops::Range;

/// Names in the patch headers and the number of unchanged context lines.
#[derive(Clone, Debug)]
pub struct UnifiedOptions {
    /// Original document label (Git's `a/` prefix is added automatically).
    pub old_name: String,
    /// Modified document label (Git's `b/` prefix is added automatically).
    pub new_name: String,
    /// Number of unchanged lines around each change; defaults to three.
    pub context: usize,
}

impl Default for UnifiedOptions {
    fn default() -> Self {
        Self {
            old_name: "old.docx".into(),
            new_name: "new.docx".into(),
            context: 3,
        }
    }
}

/// Presentation of a document's line changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextFormat {
    /// Git headers and unified hunks, keeping the snapshots' tracked marks.
    #[default]
    Github,
    /// Accept both inputs, then show fresh word-level CriticMarkup only.
    Word,
    /// Traditional `a`, `d`, `c` line addresses, without context.
    Normal,
    /// Traditional two-sided context blocks.
    Context,
    /// Aligned replacement, insertion and deletion rows, without numbers.
    SideBySide,
}

/// Review presentation options; the raw unified patch APIs remain unclipped.
#[derive(Clone, Debug)]
pub struct TextOptions {
    /// Snapshot labels and surrounding unchanged lines.
    pub unified: UnifiedOptions,
    /// Output presentation; defaults to GitHub unified hunks.
    pub format: TextFormat,
    /// Accept existing revisions before comparison (always true in Word mode).
    pub accept_changes: bool,
    /// Unicode scalar window per line; `None` preserves complete lines.
    pub window: Option<usize>,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            unified: UnifiedOptions::default(),
            format: TextFormat::Github,
            accept_changes: false,
            window: Some(70),
        }
    }
}

/// Compare text snapshots in the requested review presentation.
///
/// Existing CriticMarkup stays inside the changed lines unless acceptance is
/// requested. Word mode always accepts both inputs and emits only changed
/// lines with newly generated CriticMarkup. Clipping happens after comparison,
/// so it cannot hide a hunk or change its line ranges.
pub fn diff_text_view(old: &str, new: &str, options: &TextOptions) -> String {
    let accept = options.accept_changes || options.format == TextFormat::Word;
    let old = accepted_text(old, accept);
    let new = accepted_text(new, accept);
    render_view(&old, &new, options)
}

/// Compare complete DOCX or Markdown snapshots in a review presentation.
///
/// DOCX revision acceptance runs on the package before text extraction, so
/// removed paragraphs and table rows cannot contribute phantom line numbers.
/// Preserved revisions are rendered directly as proper CriticMarkup; the raw
/// [`document_text`] API retains its legacy debug snapshot notation.
pub fn diff_documents_view(
    old: Source<'_>,
    new: Source<'_>,
    options: &TextOptions,
) -> Result<String, String> {
    let accept = options.accept_changes || options.format == TextFormat::Word;
    let critic = options.format != TextFormat::Word;
    let old = view_snapshot(old, accept, critic)?;
    let new = view_snapshot(new, accept, critic)?;
    Ok(render_view(&old, &new, options))
}

fn accepted_text(text: &str, accept: bool) -> std::borrow::Cow<'_, str> {
    if accept {
        crate::markdown::accepted_clauses(text).into()
    } else {
        text.into()
    }
}

fn view_snapshot(source: Source<'_>, accept: bool, critic: bool) -> Result<String, String> {
    match source {
        Source::Markdown(text) => Ok(accepted_text(text, accept).into_owned()),
        Source::Docx(bytes) => crate::debug::document_text_view(bytes, critic, accept),
    }
}

fn render_view(old: &str, new: &str, options: &TextOptions) -> String {
    // CRLF and LF end the same document line. A missing final newline is
    // marked by the formats that have a marker for it (GitHub, normal and
    // context); word and side-by-side compare the lines it ends.
    let terminate = !matches!(
        options.format,
        TextFormat::Github | TextFormat::Normal | TextFormat::Context
    );
    let (old, new) = (line_endings(old, terminate), line_endings(new, terminate));
    let (old, new) = (old.as_ref(), new.as_ref());
    if old == new {
        return String::new();
    }
    if options.format == TextFormat::Github && options.window.is_none() {
        return diff_text(old, new, &options.unified);
    }
    let diff = TextDiff::from_lines(old, new);
    let context = options
        .unified
        .context
        .min(diff.old_len().max(diff.new_len()));
    match options.format {
        TextFormat::Github => github_view(&diff, options, context),
        TextFormat::Normal => normal_view(&diff, options),
        TextFormat::Context => context_view(&diff, options, context),
        TextFormat::Word | TextFormat::SideBySide => rows_view(&diff, options, context),
    }
}

/// `text` with LF line ends, and with a final newline when `terminate`.
fn line_endings(text: &str, terminate: bool) -> std::borrow::Cow<'_, str> {
    let mut text = std::borrow::Cow::Borrowed(text);
    if text.contains("\r\n") {
        text = text.replace("\r\n", "\n").into();
    }
    if terminate && !text.is_empty() && !text.ends_with('\n') {
        text.to_mut().push('\n');
    }
    text
}

/// GNU diff's marker after a last line with no newline.
fn mark_unterminated(out: &mut String, line: Option<&str>) {
    if line.is_some_and(|line| !line.ends_with('\n')) {
        out.push_str("\\ No newline at end of file\n");
    }
}

fn line(text: &str) -> &str {
    let text = text.strip_suffix('\n').unwrap_or(text);
    text.strip_suffix('\r').unwrap_or(text)
}

/// Window offsets and widths count Unicode scalars, never UTF-8 bytes.
fn clip(text: &str, changed_at: usize, window: Option<usize>) -> String {
    let Some(width) = window else {
        return text.to_string();
    };
    let len = text.chars().count();
    if len <= width {
        return text.to_string();
    }
    let start = changed_at.saturating_sub(width / 2).min(len);
    let end = start.saturating_add(width).min(len);
    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    out.extend(text.chars().skip(start).take(end - start));
    if end < len {
        out.push('…');
    }
    out
}

fn first_difference(old: &str, new: &str) -> usize {
    old.chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .count()
}

#[derive(Debug, PartialEq, Eq)]
enum Row {
    Pair(usize, usize),
    Delete(usize),
    Insert(usize),
}

/// Use the upstream word diff to align similar clauses within a replacement.
/// A bounded search avoids quadratic work for large, wholly rewritten hunks.
/// Equal-sized unrelated tails still pair by position, as standard diff does.
fn replacement_rows(old: &[&str], new: &[&str]) -> Vec<Row> {
    if old.len().saturating_mul(new.len()) > 400 {
        return positional_rows(old.len(), new.len());
    }
    let old_words: Vec<_> = old.iter().map(|text| matching_words(text)).collect();
    let new_words: Vec<_> = new.iter().map(|text| matching_words(text)).collect();
    let mut rows = Vec::new();
    let mut next = 0;
    for (i, a) in old_words.iter().enumerate() {
        let mut best = None;
        let mut score = 0.3;
        for (j, b) in new_words.iter().enumerate().skip(next) {
            let ratio = TextDiff::from_slices(a, b).ratio();
            if ratio > score {
                score = ratio;
                best = Some(j);
            }
        }
        // A later old clause may be a much stronger match for this new row.
        if let Some(j) = best {
            let stronger = old_words
                .iter()
                .skip(i + 1)
                .any(|a| TextDiff::from_slices(a, &new_words[j]).ratio() > score);
            if stronger {
                rows.push(Row::Delete(i));
                continue;
            }
        }
        let best = best
            .or_else(|| (old.len() - i == new.len() - next && next < new.len()).then_some(next));
        if let Some(j) = best {
            rows.extend((next..j).map(Row::Insert));
            rows.push(Row::Pair(i, j));
            next = j + 1;
        } else {
            rows.push(Row::Delete(i));
        }
    }
    rows.extend((next..new.len()).map(Row::Insert));
    rows
}

fn matching_words(text: &str) -> Vec<&str> {
    crate::util::word_tokens(text)
        .into_iter()
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .collect()
}

fn positional_rows(old_len: usize, new_len: usize) -> Vec<Row> {
    let paired = old_len.min(new_len);
    (0..paired)
        .map(|i| Row::Pair(i, i))
        .chain((paired..old_len).map(Row::Delete))
        .chain((paired..new_len).map(Row::Insert))
        .collect()
}

fn op_lines<'a>(diff: &'a TextDiff<'_, '_, str>, op: &DiffOp) -> (Vec<&'a str>, Vec<&'a str>) {
    let old = op
        .old_range()
        .map(|i| line(&diff.old_lookup()[i]))
        .collect();
    let new = op
        .new_range()
        .map(|i| line(&diff.new_lookup()[i]))
        .collect();
    (old, new)
}

fn op_windows(
    diff: &TextDiff<'_, '_, str>,
    op: &DiffOp,
    window: Option<usize>,
) -> (Vec<String>, Vec<String>) {
    let (old, new) = op_lines(diff, op);
    let mut old_starts = vec![0; old.len()];
    let mut new_starts = vec![0; new.len()];
    if op.tag() == DiffTag::Replace {
        for row in replacement_rows(&old, &new) {
            if let Row::Pair(i, j) = row {
                let at = first_difference(old[i], new[j]);
                old_starts[i] = at;
                new_starts[j] = at;
            }
        }
    }
    let olds = old
        .iter()
        .zip(old_starts)
        .map(|(text, at)| clip(text, at, window))
        .collect();
    let news = new
        .iter()
        .zip(new_starts)
        .map(|(text, at)| clip(text, at, window))
        .collect();
    (olds, news)
}

fn github_view(diff: &TextDiff<'_, '_, str>, options: &TextOptions, context: usize) -> String {
    let old_name = git_path("a", &options.unified.old_name);
    let new_name = git_path("b", &options.unified.new_name);
    let mut out = format!("diff --git {old_name} {new_name}\n--- {old_name}\n+++ {new_name}\n");
    for ops in diff.grouped_ops(context) {
        let _ = writeln!(out, "{}", similar::udiff::UnifiedHunkHeader::new(&ops));
        for op in &ops {
            let (old, new) = op_windows(diff, op, options.window);
            for change in diff.iter_changes(op) {
                let text = match change.tag() {
                    ChangeTag::Equal | ChangeTag::Delete => {
                        &old[change.old_index().unwrap() - op.old_range().start]
                    }
                    ChangeTag::Insert => &new[change.new_index().unwrap() - op.new_range().start],
                };
                let _ = writeln!(out, "{}{text}", change.tag());
                if change.missing_newline() {
                    out.push_str("\\ No newline at end of file\n");
                }
            }
        }
    }
    out
}

/// Nonempty ranges use inclusive, one-based addresses; empty ones name the
/// preceding line (including line zero for an insertion at the beginning).
fn address(range: &Range<usize>) -> String {
    match range.len() {
        0 => range.start.to_string(),
        1 => range.end.to_string(),
        _ => format!("{},{}", range.start + 1, range.end),
    }
}

fn normal_view(diff: &TextDiff<'_, '_, str>, options: &TextOptions) -> String {
    let mut out = String::new();
    for op in diff.ops().iter().filter(|op| op.tag() != DiffTag::Equal) {
        let letter = match op.tag() {
            DiffTag::Insert => 'a',
            DiffTag::Delete => 'd',
            _ => 'c',
        };
        let _ = writeln!(
            out,
            "{}{letter}{}",
            address(&op.old_range()),
            address(&op.new_range())
        );
        let (old, new) = op_windows(diff, op, options.window);
        for (index, text) in op.old_range().zip(old) {
            let _ = writeln!(out, "< {text}");
            mark_unterminated(&mut out, diff.old_slice(index));
        }
        if op.tag() == DiffTag::Replace {
            out.push_str("---\n");
        }
        for (index, text) in op.new_range().zip(new) {
            let _ = writeln!(out, "> {text}");
            mark_unterminated(&mut out, diff.new_slice(index));
        }
    }
    out
}

fn context_view(diff: &TextDiff<'_, '_, str>, options: &TextOptions, context: usize) -> String {
    // Reuse Git quoting for labels, dropping only the synthetic side prefix.
    let label = |name: &str| git_path("", name).replacen('/', "", 1);
    let mut out = format!(
        "*** {}\n--- {}\n",
        label(&options.unified.old_name),
        label(&options.unified.new_name)
    );
    for ops in diff.grouped_ops(context) {
        let Some(first) = ops.first() else { continue };
        let last = ops.last().unwrap();
        let olds = first.old_range().start..last.old_range().end;
        let news = first.new_range().start..last.new_range().end;
        out.push_str("***************\n");
        let _ = writeln!(out, "*** {} ****", address(&olds));
        let windows: Vec<_> = ops
            .iter()
            .map(|op| op_windows(diff, op, options.window))
            .collect();
        for (op, (old, _)) in ops.iter().zip(&windows) {
            let prefix = match op.tag() {
                DiffTag::Equal => "  ",
                DiffTag::Replace => "! ",
                _ => "- ",
            };
            for (index, text) in op.old_range().zip(old) {
                let _ = writeln!(out, "{prefix}{text}");
                mark_unterminated(&mut out, diff.old_slice(index));
            }
        }
        let _ = writeln!(out, "--- {} ----", address(&news));
        for (op, (_, new)) in ops.iter().zip(&windows) {
            let prefix = match op.tag() {
                DiffTag::Equal => "  ",
                DiffTag::Replace => "! ",
                _ => "+ ",
            };
            for (index, text) in op.new_range().zip(new) {
                let _ = writeln!(out, "{prefix}{text}");
                mark_unterminated(&mut out, diff.new_slice(index));
            }
        }
    }
    out
}

/// Escape literal text before surrounding it with generated revision marks.
/// Backslashes must be doubled before the renderer escapes mark delimiters.
pub(crate) fn critic_literal(text: &str) -> String {
    let text = text.replace('\\', "\\\\");
    crate::markdown::diff_markdown(&text, &text)
}

/// One stretch of a word-level change line.
enum Piece {
    Same(String),
    Delete(String),
    Insert(String),
    Replace(String, String),
}

impl Piece {
    /// The texts the line shows, in order: one, or the old and new text.
    fn texts(&self) -> [&str; 2] {
        match self {
            Self::Same(t) | Self::Delete(t) | Self::Insert(t) => [t, ""],
            Self::Replace(a, b) => [a, b],
        }
    }

    fn len(&self) -> usize {
        self.texts().iter().map(|t| t.chars().count()).sum()
    }
}

fn word_pieces(old: &str, new: &str) -> Vec<Piece> {
    let old_tokens = crate::util::word_tokens(old);
    let new_tokens = crate::util::word_tokens(new);
    let diff = TextDiff::from_slices(&old_tokens, &new_tokens);
    diff.ops()
        .iter()
        .map(|op| {
            let a = old_tokens[op.old_range()].concat();
            let b = new_tokens[op.new_range()].concat();
            match op.tag() {
                DiffTag::Equal => Piece::Same(a),
                DiffTag::Delete => Piece::Delete(a),
                DiffTag::Insert => Piece::Insert(b),
                DiffTag::Replace => Piece::Replace(a, b),
            }
        })
        .collect()
}

#[cfg(test)]
fn word_change(old: &str, new: &str) -> String {
    critic_line(&word_pieces(old, new), None)
}

/// The pieces as CriticMarkup, keeping `window` scalars of text around the
/// first change. The window counts text, never delimiters: a mark it
/// reaches keeps its delimiters and loses only text, so every mark the
/// line opens, it closes. Text left out becomes `…`.
fn critic_line(pieces: &[Piece], window: Option<usize>) -> String {
    let len: usize = pieces.iter().map(Piece::len).sum();
    let shown = match window {
        Some(width) if len > width => {
            let at: usize = pieces
                .iter()
                .take_while(|p| matches!(p, Piece::Same(_)))
                .map(Piece::len)
                .sum();
            let start = at.saturating_sub(width / 2).min(len);
            start..start.saturating_add(width).min(len)
        }
        _ => 0..len,
    };
    // The visible part of the text at `offset`, as scalar offsets into it.
    let visible = |offset: usize, n: usize| {
        let from = shown.start.clamp(offset, offset + n) - offset;
        let to = shown.end.clamp(offset, offset + n) - offset;
        (from < to).then_some((from, to))
    };
    let slice = |text: &str, (from, to): (usize, usize)| {
        critic_literal(&text.chars().skip(from).take(to - from).collect::<String>())
    };
    let mut out = String::new();
    let mut offset = 0;
    // Text up to here is written or stands behind a `…`.
    let mut covered = 0;
    for piece in pieces {
        let end = offset + piece.len();
        let (open, close) = match piece {
            Piece::Same(text) => {
                let n = text.chars().count();
                if let Some(part) = visible(offset, n) {
                    if offset + part.0 > covered {
                        out.push('…');
                    }
                    out.push_str(&slice(text, part));
                    covered = offset + part.1;
                }
                offset = end;
                continue;
            }
            Piece::Delete(_) => ("{--", "--}"),
            Piece::Insert(_) => ("{++", "++}"),
            Piece::Replace(..) => ("{~~", "~~}"),
        };
        if shown.start >= end || offset >= shown.end {
            offset = end;
            continue;
        }
        if offset > covered {
            out.push('…');
        }
        out.push_str(open);
        let texts = piece.texts();
        let parts = if matches!(piece, Piece::Replace(..)) {
            2
        } else {
            1
        };
        for (i, text) in texts.iter().take(parts).enumerate() {
            if i == 1 {
                out.push_str("~>");
            }
            let n = text.chars().count();
            match visible(offset, n) {
                Some(part) => {
                    if part.0 > 0 {
                        out.push('…');
                    }
                    out.push_str(&slice(text, part));
                    if part.1 < n {
                        out.push('…');
                    }
                }
                None if n > 0 => out.push('…'),
                None => {}
            }
            offset += n;
        }
        out.push_str(close);
        covered = end;
    }
    if covered < len {
        out.push('…');
    }
    out
}

fn rows_view(diff: &TextDiff<'_, '_, str>, options: &TextOptions, context: usize) -> String {
    let word = options.format == TextFormat::Word;
    let mut out = String::new();
    // Side-by-side rows: the old cell, the gutter mark, the new cell.
    let mut side: Vec<(String, char, String)> = Vec::new();
    let cut = |text: &str, at: usize| clip(text, at, options.window);
    for ops in diff.grouped_ops(if word { 0 } else { context }) {
        for op in &ops {
            let (old, new) = op_lines(diff, op);
            if op.tag() == DiffTag::Equal {
                if !word {
                    for text in old {
                        let text = cut(text, 0);
                        side.push((text.clone(), ' ', text));
                    }
                }
                continue;
            }
            for row in replacement_rows(&old, &new) {
                let (a, b) = match row {
                    Row::Pair(i, j) => (old[i], new[j]),
                    Row::Delete(i) => (old[i], ""),
                    Row::Insert(j) => ("", new[j]),
                };
                if word {
                    let _ = writeln!(out, "{}", critic_line(&word_pieces(a, b), options.window));
                    continue;
                }
                side.push(match row {
                    Row::Pair(..) => {
                        let at = first_difference(a, b);
                        (cut(a, at), '|', cut(b, at))
                    }
                    Row::Delete(..) => (cut(a, 0), '<', String::new()),
                    Row::Insert(..) => (String::new(), '>', cut(b, 0)),
                });
            }
        }
    }
    // Pad the old column to its widest cell, in scalars like the window, so
    // every gutter mark and every new cell starts in the same column.
    let width = side
        .iter()
        .map(|(a, ..)| a.chars().count())
        .max()
        .unwrap_or(0);
    for (a, mark, b) in side {
        if b.is_empty() {
            let _ = writeln!(out, "{}", format!("{a:<width$} {mark}").trim_end());
        } else {
            let _ = writeln!(out, "{a:<width$} {mark} {b}");
        }
    }
    out
}

/// Complete textual snapshot, preserving existing tracked marks.
pub fn document_text(source: Source<'_>) -> Result<String, String> {
    match source {
        Source::Markdown(text) => Ok(text.to_string()),
        Source::Docx(bytes) => crate::debug::document_text(bytes),
    }
}

/// A standard unified patch; identical text produces an empty string.
pub fn diff_text(old: &str, new: &str, options: &UnifiedOptions) -> String {
    if old == new {
        return String::new();
    }
    let old_name = git_path("a", &options.old_name);
    let new_name = git_path("b", &options.new_name);
    let diff = similar::TextDiff::from_lines(old, new);
    let hunks = diff
        .unified_diff()
        .context_radius(
            options
                .context
                .min(old.lines().count().max(new.lines().count())),
        )
        .header(&old_name, &new_name)
        .to_string();
    format!("diff --git {old_name} {new_name}\n{hunks}")
}

// Git C-style path quoting keeps control characters out of patch headers.
fn git_path(side: &str, name: &str) -> String {
    let path = format!("{side}/{name}");
    if !path
        .chars()
        .any(|c| c.is_control() || matches!(c, '"' | '\\' | ' '))
    {
        return path;
    }
    let mut out = String::from("\"");
    for byte in path.bytes() {
        match byte {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\t' => out.push_str("\\t"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            0..=31 | 127..=255 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\{byte:03o}");
            }
            _ => out.push(char::from(byte)),
        }
    }
    out.push('"');
    out
}

/// Compare DOCX bytes or Markdown text without constructing a redline.
pub fn diff_documents(
    old: Source<'_>,
    new: Source<'_>,
    options: &UnifiedOptions,
) -> Result<String, String> {
    Ok(diff_text(
        &document_text(old)?,
        &document_text(new)?,
        options,
    ))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::markdown::{DocxOptions, markdown_to_docx};

    fn word(text: &str) -> Vec<u8> {
        markdown_to_docx(text, &DocxOptions::default())
            .unwrap()
            .docx
    }

    fn word_view(old: &str, new: &str, window: Option<usize>) -> String {
        let options = TextOptions {
            format: TextFormat::Word,
            window,
            ..Default::default()
        };
        diff_text_view(old, new, &options)
    }

    /// Every mark a line opens, it closes, in CriticMarkup order.
    fn marks_close(line: &str) -> bool {
        let mut rest = line;
        while let Some(at) = ["{++", "{--", "{~~"]
            .iter()
            .filter_map(|open| rest.find(open))
            .min()
        {
            let close: &[&str] = match &rest[at..at + 3] {
                "{++" => &["++}"],
                "{--" => &["--}"],
                _ => &["~>", "~~}"],
            };
            rest = &rest[at + 3..];
            for delimiter in close {
                let Some(end) = rest.find(delimiter) else {
                    return false;
                };
                if ["{++", "{--", "{~~"]
                    .iter()
                    .any(|o| rest[..end].contains(o))
                {
                    return false;
                }
                rest = &rest[end + delimiter.len()..];
            }
        }
        !["++}", "--}", "~~}", "~>"].iter().any(|c| rest.contains(c))
    }

    #[test]
    fn a_word_window_keeps_every_mark_closed() {
        let tail = "and the tail of an inserted clause that runs well past the window";
        let cases = [
            (
                "Pay within thirty days.".to_string(),
                format!("Pay within thirty days {tail}."),
            ),
            (format!("Pay {tail} now."), "Pay now.".to_string()),
            ("Keep old wording here".to_string(), format!("Keep {tail}")),
            (
                format!("A {tail} B one C"),
                format!("A {tail} B two C three D four"),
            ),
            (
                format!("{} lead then old", "\\".repeat(2_000)),
                format!("{} lead then new", "\\".repeat(2_000)),
            ),
        ];
        for (old, new) in &cases {
            for width in 0..90 {
                let out = word_view(old, new, Some(width));
                assert!(marks_close(&out), "width {width}: {out}");
                // Any window wider than nothing shows the first change.
                assert_eq!(out.contains('{'), width > 0, "width {width}: {out}");
            }
            assert!(marks_close(&word_view(old, new, None)));
        }
        // Text is cut, never a delimiter: the window counts text only.
        assert_eq!(
            word_view("a b", "a b c d e f g h i j", Some(10)),
            "a b{++ c d e …++}\n"
        );
        assert_eq!(
            word_view("keep one two three four five", "keep six", Some(10)),
            "keep {~~one t…~>…~~}\n"
        );
        assert_eq!(
            word_view(
                "lead words here, then old",
                "lead words here, then new",
                Some(12)
            ),
            "… then {~~old~>new~~}\n"
        );
        assert_eq!(word_view("a b", "a b c d e f g h i j", Some(0)), "…\n");
    }

    #[test]
    fn side_by_side_columns_line_up_on_every_row() {
        let options = TextOptions {
            format: TextFormat::SideBySide,
            ..Default::default()
        };
        let view = |old: &str, new: &str| diff_text_view(old, new, &options);
        assert_eq!(
            view("same\nold\nend\n", "same\nnew\nend\n"),
            "same   same\nold  | new\nend    end\n"
        );
        assert_eq!(
            view("a\nlong line\n", "a\n"),
            "a           a\nlong line <\n"
        );
        assert_eq!(view("a\n", "a\nadded\n"), "a   a\n  > added\n");
        // Widths count scalars, as windows do.
        assert_eq!(view("é界\nx\n", "é界\ny\n"), "é界   é界\nx  | y\n");
        // An empty unchanged line leaves no trailing spaces.
        assert_eq!(view("\nold\n", "\nnew\n"), "\nold | new\n");
    }

    #[test]
    fn review_windows_count_scalars_and_allow_unbounded_and_zero_width() {
        assert_eq!(clip("é界xyz", 3, Some(2)), "…xy…");
        assert_eq!(clip("é界xyz", 0, Some(2)), "é界…");
        assert_eq!(clip("é界xyz", 5, Some(2)), "…z");
        assert_eq!(clip("é界xyz", 3, None), "é界xyz");
        assert_eq!(clip("short", 100, Some(70)), "short");
        assert_eq!(clip("abc", 0, Some(0)), "…");
        assert_eq!(clip("abc", 1, Some(0)), "……");
        assert_eq!(clip("", 0, Some(0)), "");
        assert_eq!(first_difference("é界", "é中"), 1);
        assert_eq!(first_difference("same", "same suffix"), 4);
    }

    #[test]
    fn replacement_alignment_handles_inserted_and_deleted_clauses_and_large_blocks() {
        assert_eq!(
            replacement_rows(
                &["Payment 30 days", "Security assets"],
                &["Prepayment allowed", "Payment 60 days", "Security property"]
            ),
            [Row::Insert(0), Row::Pair(0, 1), Row::Pair(1, 2)]
        );
        assert_eq!(
            replacement_rows(
                &["Payment 60 days", "Payment 30 days"],
                &["Payment 60 days"]
            ),
            [Row::Pair(0, 0), Row::Delete(1)]
        );
        assert_eq!(
            replacement_rows(&["Payment stale", "Payment 30 days"], &["Payment 60 days"]),
            [Row::Delete(0), Row::Pair(1, 0)]
        );
        assert_eq!(replacement_rows(&["a"], &["b"]), [Row::Pair(0, 0)]);
        assert_eq!(replacement_rows(&["a"], &[]), [Row::Delete(0)]);
        assert_eq!(replacement_rows(&[], &["b"]), [Row::Insert(0)]);
        let rows = replacement_rows(&vec!["old"; 21], &vec!["new"; 22]);
        assert_eq!(rows.len(), 22);
        assert_eq!(rows.last(), Some(&Row::Insert(21)));
    }

    #[test]
    fn word_changes_preserve_literal_syntax_and_changed_clause_numbers() {
        assert_eq!(word_change("1. Terms", "2. Terms"), "{~~1~>2~~}. Terms");
        assert_eq!(
            word_change("pay 30 days", "pay 60 days"),
            "pay {~~30~>60~~} days"
        );
        assert_eq!(word_change("", "new"), "{++new++}");
        assert_eq!(word_change("old", ""), "{--old--}");
        assert_eq!(word_change("same", "same"), "same");
        // Literal markup is text, escaped.
        assert_eq!(
            word_change("{++literal++} old", "{++literal++} new"),
            r"\{++literal++\} {~~old~>new~~}"
        );
        assert_eq!(
            critic_literal("{+literal+}[-literal-]"),
            "{+literal+}[-literal-]"
        );
        assert_eq!(critic_literal("{++literal++}"), r"\{++literal++\}");
    }

    #[test]
    fn review_context_and_unterminated_github_lines_use_original_coordinates() {
        let options = TextOptions {
            window: Some(70),
            ..Default::default()
        };
        assert_eq!(
            diff_text_view("a", "b", &options),
            diff_text("a", "b", &options.unified)
        );
        let options = TextOptions {
            format: TextFormat::SideBySide,
            ..options
        };
        assert_eq!(
            diff_text_view("same\nold\nend\n", "same\nnew\nend\n", &options),
            "same   same\nold  | new\nend    end\n"
        );
        let options = TextOptions {
            format: TextFormat::Context,
            unified: UnifiedOptions {
                old_name: "old\nfile".into(),
                new_name: "new file".into(),
                context: 0,
            },
            ..options
        };
        assert!(
            diff_text_view("a\n", "b\n", &options)
                .starts_with("*** \"old\\nfile\"\n--- \"new file\"\n")
        );
    }

    #[test]
    fn patch_has_git_headers_ranges_and_context() {
        let out = diff_text(
            "Intro\nThirty days\nSigned\n",
            "Intro\nSixty days\nSigned\n",
            &UnifiedOptions::default(),
        );
        assert_eq!(
            out,
            "diff --git a/old.docx b/new.docx\n--- a/old.docx\n+++ b/new.docx\n@@ -1,3 +1,3 @@\n Intro\n-Thirty days\n+Sixty days\n Signed\n"
        );
    }

    #[test]
    fn line_terminators_never_show_as_identical_changed_lines() {
        let formats = [
            TextFormat::Github,
            TextFormat::Word,
            TextFormat::Normal,
            TextFormat::Context,
            TextFormat::SideBySide,
        ];
        for format in formats {
            for window in [Some(70), None] {
                let options = TextOptions {
                    format,
                    window,
                    ..Default::default()
                };
                // CRLF against LF is the same document text.
                assert_eq!(
                    diff_text_view("one\r\ntwo\r\n", "one\ntwo\n", &options),
                    "",
                    "{format:?} {window:?}"
                );
                // A missing final newline is marked where the format has a
                // marker for it, and is no change where it has none.
                let out = diff_text_view("same", "same\n", &options);
                match format {
                    TextFormat::Github | TextFormat::Normal | TextFormat::Context => assert_eq!(
                        out.matches("\\ No newline at end of file").count(),
                        1,
                        "{format:?} {window:?}: {out}"
                    ),
                    TextFormat::Word | TextFormat::SideBySide => {
                        assert_eq!(out, "", "{format:?} {window:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn empty_ranges_and_missing_final_newlines_follow_unified_convention() {
        let options = UnifiedOptions {
            context: 0,
            ..Default::default()
        };
        let inserted = diff_text("", "Hello\n", &options);
        assert!(inserted.contains("@@ -0,0 +1 @@\n+Hello\n"), "{inserted}");
        let deleted = diff_text("Hello\n", "", &options);
        assert!(deleted.contains("@@ -1 +0,0 @@\n-Hello\n"), "{deleted}");
        let unterminated = diff_text("Old", "New", &options);
        assert_eq!(
            unterminated.matches("\\ No newline at end of file").count(),
            2
        );
    }

    #[test]
    fn identical_content_has_no_patch_even_with_different_names() {
        assert_eq!(
            diff_text("same\n", "same\n", &UnifiedOptions::default()),
            ""
        );
        assert_eq!(diff_text("", "", &UnifiedOptions::default()), "");
    }

    #[test]
    fn names_with_control_characters_are_git_quoted() {
        let options = UnifiedOptions {
            old_name: "a\t\".docx".into(),
            new_name: "b\n.docx".into(),
            context: 0,
        };
        let out = diff_text("a\n", "b\n", &options);
        assert!(
            out.starts_with("diff --git \"a/a\\t\\\".docx\" \"b/b\\n.docx\"\n"),
            "{out}"
        );
        assert!(out.contains("--- \"a/a\\t\\\".docx\"\n"));
    }

    #[test]
    fn every_change_and_long_unicode_line_is_preserved() {
        let old = (0..12)
            .map(|n| format!("old {n} {}\n", "é".repeat(300)))
            .collect::<String>();
        let new = old.replace("old ", "new ");
        let out = diff_text(&old, &new, &UnifiedOptions::default());
        assert_eq!(out.lines().filter(|l| l.starts_with("-old ")).count(), 12);
        assert!(out.contains(&"é".repeat(300)));
        assert!(!out.contains('…'));
    }

    #[test]
    fn word_snapshot_preserves_existing_marks_and_compares_them() {
        let old = word("Due in {~~30~>45~~} days.\n");
        let new = word("Due in {~~30~>60~~} days.\n");
        let snapshot = document_text(Source::Docx(&old)).unwrap();
        assert!(snapshot.contains("[-30-]{+45+}"), "{snapshot}");
        let out = diff_documents(
            Source::Docx(&old),
            Source::Docx(&new),
            &UnifiedOptions::default(),
        )
        .unwrap();
        assert!(out.contains("[-30-]{+45+}"), "{out}");
        assert!(out.contains("[-30-]{+60+}"), "{out}");
        assert_eq!(
            diff_documents(
                Source::Docx(&old),
                Source::Docx(&old),
                &UnifiedOptions::default()
            )
            .unwrap(),
            ""
        );
    }

    #[test]
    fn invalid_docx_is_refused() {
        assert!(document_text(Source::Docx(b"not a document")).is_err());
    }

    #[test]
    fn markdown_and_mixed_sources_share_the_same_formatter() {
        let old = word("Hello\n");
        let snapshot = document_text(Source::Docx(&old)).unwrap();
        assert_eq!(
            diff_documents(
                Source::Docx(&old),
                Source::Markdown(&snapshot),
                &UnifiedOptions::default()
            )
            .unwrap(),
            ""
        );
        assert!(
            diff_documents(
                Source::Markdown("Old\n"),
                Source::Markdown("New\n"),
                &UnifiedOptions::default()
            )
            .unwrap()
            .contains("-Old\n+New\n")
        );
    }
}
