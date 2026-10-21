// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! CriticMarkup delimiters in Markdown source.
//!
//! CriticMarkup (<https://criticmarkup.com>) is read before the Markdown, as
//! its specification asks: [`encode`] swaps the delimiters of every complete
//! span for single stand-in characters, the Markdown parser runs on the
//! result, and [`Pieces`] splits the parser's text back into text and
//! delimiters. A change can therefore cross emphasis, links and paragraph
//! breaks, and `{~~a~>b~~}` is never read as `~~` strikethrough.
//!
//! The stand-ins are Unicode punctuation (general category Po), so emphasis
//! next to a delimiter parses as it would next to the `+` of `{++`, and
//! Markdown gives them no meaning. A stand-in character already in the
//! source is kept apart by an escape character and comes back unchanged,
//! and so is a numeric character reference naming one (`&#x2E0E;`): the
//! Markdown parser decodes references only after CriticMarkup was read.

/// A CriticMarkup delimiter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Token {
    /// `{++`
    InsertStart,
    /// `++}`
    InsertEnd,
    /// `{--`
    DeleteStart,
    /// `--}`
    DeleteEnd,
    /// `{~~`
    SubstituteStart,
    /// `~>`
    SubstituteSeparator,
    /// `~~}`
    SubstituteEnd,
    /// `{==`
    HighlightStart,
    /// `==}`
    HighlightEnd,
    /// `{>>`
    CommentStart,
    /// `<<}`
    CommentEnd,
    /// A blank line inside a comment's body: a comment paragraph break.
    CommentBreak,
    /// `==` opening a highlight run (`==x==`), not CriticMarkup
    MarkStart,
    /// `==` closing a highlight run
    MarkEnd,
    /// `{` opening a body field's code (`{REF _Ref1 \h}result{/}`)
    FieldStart,
    /// `}` ending the code: the result follows
    FieldSeparator,
    /// `{/}` ending the field
    FieldEnd,
}

/// Every delimiter with its stand-in (Supplemental Punctuation, all Po).
const STAND_INS: [(Token, char); 17] = [
    (Token::InsertStart, '\u{2E0E}'),
    (Token::InsertEnd, '\u{2E0F}'),
    (Token::DeleteStart, '\u{2E10}'),
    (Token::DeleteEnd, '\u{2E11}'),
    (Token::SubstituteStart, '\u{2E12}'),
    (Token::SubstituteSeparator, '\u{2E13}'),
    (Token::SubstituteEnd, '\u{2E14}'),
    (Token::HighlightStart, '\u{2E15}'),
    (Token::HighlightEnd, '\u{2E16}'),
    (Token::CommentStart, '\u{2E18}'),
    (Token::CommentEnd, '\u{2E19}'),
    (Token::CommentBreak, '\u{2E2E}'),
    (Token::MarkStart, '\u{2E1B}'),
    (Token::MarkEnd, '\u{2E1E}'),
    (Token::FieldStart, '\u{2E1F}'),
    (Token::FieldSeparator, '\u{2E2A}'),
    (Token::FieldEnd, '\u{2E2B}'),
];

/// Put before a stand-in character (or itself) that was in the source.
const ESCAPE: char = '\u{F0000}';

/// A span's opening delimiter, its closing one, and the separator a
/// substitution needs between them.
const SPANS: [(&str, &str, Option<&str>, Token, Token); 5] = [
    ("{++", "++}", None, Token::InsertStart, Token::InsertEnd),
    ("{--", "--}", None, Token::DeleteStart, Token::DeleteEnd),
    (
        "{~~",
        "~~}",
        Some("~>"),
        Token::SubstituteStart,
        Token::SubstituteEnd,
    ),
    (
        "{==",
        "==}",
        None,
        Token::HighlightStart,
        Token::HighlightEnd,
    ),
    ("{>>", "<<}", None, Token::CommentStart, Token::CommentEnd),
];

fn stand_in(token: Token) -> char {
    STAND_INS
        .iter()
        .find(|(t, _)| *t == token)
        .map(|(_, c)| *c)
        .unwrap_or(ESCAPE)
}

fn token_of(c: char) -> Option<Token> {
    STAND_INS.iter().find(|(_, s)| *s == c).map(|(t, _)| *t)
}

fn push_escaped(out: &mut String, text: &str) {
    for c in text.chars() {
        if c == ESCAPE || token_of(c).is_some() {
            out.push(ESCAPE);
        }
        out.push(c);
    }
}

/// Whether the character at byte `at` follows an odd number of backslashes,
/// so Markdown reads it as a literal.
fn escaped(source: &str, at: usize) -> bool {
    source[..at]
        .bytes()
        .rev()
        .take_while(|b| *b == b'\\')
        .count()
        % 2
        == 1
}

/// The length of the numeric character reference `rest` starts with
/// (`&#x2E0E;`, `&#11790;`) when the character it names is a stand-in or
/// the escape: the Markdown parser decodes references only after
/// CriticMarkup was read, so the decoded character must arrive escaped or
/// it would act as a delimiter.
fn reference_stand_in(rest: &str) -> Option<usize> {
    let semi = rest.as_bytes().iter().take(12).position(|&b| b == b';')?;
    let number = rest.get(1..semi)?.strip_prefix('#')?;
    let (digits, hex) = match number.strip_prefix(['x', 'X']) {
        Some(hex) => (hex, true),
        None => (number, false),
    };
    // As CommonMark reads them: at most seven decimal digits, six
    // hexadecimal ones.
    if digits.is_empty() || digits.len() > if hex { 6 } else { 7 } {
        return None;
    }
    if !digits
        .bytes()
        .all(|b| b.is_ascii_digit() || hex && b.is_ascii_hexdigit())
    {
        return None;
    }
    let named = char::from_u32(u32::from_str_radix(digits, if hex { 16 } else { 10 }).ok()?)?;
    (token_of(named).is_some() || named == ESCAPE).then_some(semi + 1)
}

/// The source with the delimiters of every complete CriticMarkup span
/// replaced by stand-ins. A delimiter without its partner, or behind a
/// backslash, stays text.
///
/// The specification has spans never nest. Word, though, can track a change
/// inside a commented range and comment inside a change, and a document
/// read from Word says so, so here spans nest one level: a highlight can
/// hold changes and comments (`{==a {++b++}==}{>>note<<}`), a change can
/// hold highlights and comments (`{~~a~>{==b==}{>>n<<}~~}`), and anything
/// deeper, or inside a comment, is text.
///
/// A complete nested comment or highlight is opaque to the enclosing
/// span's closer and (for a substitution) its `~>` separator: a
/// `{++a{>>b++}c++}` looks for `++}` past the comment, not at the one
/// inside it, and a `{~~a{>>x ~> y<<} b~>c~~}` keeps its `~>` separator
/// outside the comment. An opener whose closer never comes, or whose
/// closer falls outside the enclosing range, opens nothing: only a
/// complete, fully nested span becomes opaque. A blank line inside a
/// comment is a comment paragraph break ([`Token::CommentBreak`]), kept
/// out of the text around it.
///
/// A change or highlight that opens a line with a footnote label
/// (`{++[^2]: Added later.++}`, a note inserted with its reference) is read
/// as changing the note: the label goes before the delimiter, so the
/// definition stays a definition.
#[cfg(test)]
pub(crate) fn encode(source: &str) -> String {
    encode_spans(source, ALL, true)
}

/// A top-level change span of a source: an insertion, a deletion or a
/// substitution that no other span holds.
pub(crate) struct ChangeRegion {
    /// Where the opener (`{++`, `{--`, `{~~`) starts.
    pub(crate) open: usize,
    /// The change's text, between its delimiters.
    pub(crate) body: std::ops::Range<usize>,
    /// Where a substitution's `~>` sits, inside `body`'s span in `source`.
    pub(crate) split: Option<usize>,
    /// Just past the closing delimiter.
    pub(crate) close: usize,
    /// An insertion (`{++`) against a deletion (`{--}`); a substitution is
    /// both and names neither.
    pub(crate) inserted: bool,
}

/// The top-level change spans of `source`, in order, outside the parts
/// Markdown reads literally (code spans and fenced code blocks, where the
/// marks are printed, not tracked). What
/// [`write`](super::write) uses to rewrite a change that holds whole
/// blocks so the Markdown reader still sees the blocks.
pub(crate) fn change_regions(source: &str) -> Vec<ChangeRegion> {
    let closers = Closers::new(source);
    let mut regions = Vec::new();
    let mut at = 0;
    while at < source.len() {
        if let Some(end) = closers.literal.verbatim_end(at) {
            at = end;
            continue;
        }
        let rest = &source[at..];
        let here = if closers.literal.in_fence(at) {
            ALL & !(MARK | FIELD)
        } else {
            ALL
        };
        let span = SPANS
            .iter()
            .find(|(open, ..)| rest.starts_with(*open))
            .filter(|span| here & kind_of(span.3) != 0)
            .filter(|_| !escaped(source, at))
            .and_then(|&(open, close, separator, start, _)| {
                let from = at + open.len();
                let to = closers.find(close, from, source.len())?;
                let split = match separator {
                    Some(separator) => Some((closers.find(separator, from, to)?, separator.len())),
                    None => None,
                };
                Some((from..to, to + close.len(), split, start))
            });
        match span {
            Some((body, after, split, start)) => {
                // Fenced code and the front matter print their marks; a
                // change there is not blocks to hand them to.
                if kind_of(start) & CHANGES != 0 && !closers.literal.in_fence(at) {
                    regions.push(ChangeRegion {
                        open: at,
                        body,
                        split: split.map(|(mid, _)| mid),
                        close: after,
                        inserted: start == Token::InsertStart,
                    });
                }
                // Top level only: the span, nested changes and all, is one.
                at = after;
            }
            None => {
                let c = rest.chars().next().unwrap_or_default();
                at += c.len_utf8();
            }
        }
    }
    regions
}

/// [`encode`], plus a warning for every malformed delimiter that comes
/// out text: an opener without its closer or separator, a closer that
/// closed nothing.
pub(crate) fn encode_checked(source: &str) -> (String, Vec<String>) {
    let closers = Closers::new(source);
    let mut check = Check::default();
    let mut out = String::with_capacity(source.len());
    encode_range(
        source,
        &closers,
        0..source.len(),
        ALL,
        true,
        &mut check,
        &mut out,
    );
    check.consumed.sort_unstable();
    check.consumed.dedup();
    check.unclosed.sort_unstable();
    check.unclosed.dedup();
    // A closer that closed nothing is text. One inside a comment or a
    // complete highlight is that span's text, and one paired with an
    // escaped or verbatim opener is the literal's own — none is a stray.
    let mut stray: Vec<(usize, &'static str)> = Vec::new();
    for (slot, open, close) in SPANS
        .iter()
        .map(|&(open, close, ..)| (Closers::slot(close), open, close))
    {
        let escaped_openers: Vec<usize> = source
            .match_indices(open)
            .map(|(at, _)| at)
            .filter(|&at| escaped(source, at) || closers.literal.verbatim_end(at).is_some())
            .collect();
        let mut used = 0;
        for &at in &closers.positions[slot] {
            if check.consumed.binary_search(&at).is_ok()
                || Closers::in_spans(&closers.comments, at)
                || Closers::in_spans(&closers.highlights, at)
            {
                continue;
            }
            if let Some(&opener) = escaped_openers.get(used)
                && opener < at
            {
                used += 1;
                continue;
            }
            stray.push((at, close));
        }
    }
    stray.sort_unstable();
    stray.dedup();
    // The warnings in source order, each line counted in one pass.
    let events: Vec<(usize, &'static str, &'static str, bool)> = check
        .unclosed
        .iter()
        .map(|&(at, open, missing, separator)| (at, open, missing, separator))
        .chain(stray.iter().map(|&(at, close)| (at, "", close, false)))
        .collect();
    let mut ordered = events;
    ordered.sort_by_key(|&(at, ..)| at);
    let mut warnings = Vec::with_capacity(ordered.len());
    let mut line = 1;
    let mut past = 0;
    for (at, open, missing, separator) in ordered {
        line += source[past..at].bytes().filter(|&b| b == b'\n').count();
        past = at;
        warnings.push(if open.is_empty() {
            format!("'{missing}' with nothing open on line {line} is text")
        } else if separator {
            format!("'{open}' without its '{missing}' separator on line {line} is text")
        } else {
            format!("'{open}' without its '{missing}' on line {line} is text")
        });
    }
    warnings.sort();
    warnings.dedup();
    (out, warnings)
}

/// Which spans a text may open: bits of [`INSERT`], [`DELETE`]...
type Allowed = u8;
const INSERT: Allowed = 1;
const DELETE: Allowed = 2;
const SUBSTITUTE: Allowed = 4;
const HIGHLIGHT: Allowed = 8;
const COMMENT: Allowed = 16;
const FIELD: Allowed = 32;
/// A `==highlight==` run.
const MARK: Allowed = 64;
const CHANGES: Allowed = INSERT | DELETE | SUBSTITUTE;
const ALL: Allowed = CHANGES | HIGHLIGHT | COMMENT | FIELD | MARK;

fn kind_of(start: Token) -> Allowed {
    match start {
        Token::InsertStart => INSERT,
        Token::DeleteStart => DELETE,
        Token::SubstituteStart => SUBSTITUTE,
        Token::HighlightStart => HIGHLIGHT,
        _ => COMMENT,
    }
}

/// What the body of a span of `kind`, itself opened where `allowed`, may open.
fn inside(kind: Allowed, allowed: Allowed) -> Allowed {
    match kind {
        COMMENT => 0,
        HIGHLIGHT => allowed & !HIGHLIGHT,
        _ => allowed & !CHANGES,
    }
}

/// A nested span's cover of one delimiter position: the innermost span
/// covering it, the earliest start and the latest end of every span
/// covering it.
type Cover = ((usize, usize), usize, usize);

/// A delimiter's run of consecutive covered positions: the first index
/// after it, its earliest covering start, its latest covering end.
type Run = (usize, usize, usize);

/// Where every unescaped closer and separator sits in the source, found once
/// so that an opener without its closer does not scan the rest of the text,
/// and the parts Markdown reads literally.
struct Closers<'a> {
    patterns: [&'a str; 6],
    positions: [Vec<usize>; 6],
    /// Every complete comment (`{>>…<<}`) as (start, end), disjoint and
    /// sorted by start: delimiters inside a comment are its text, so a
    /// stray-closer warning skips them.
    comments: Vec<(usize, usize)>,
    /// Every complete highlight (`{==…==}`) the same way: a closer
    /// inside one is its text too.
    highlights: Vec<(usize, usize)>,
    /// Per slot and position: the innermost nested span covering the
    /// delimiter, the earliest start and the latest end of every span
    /// covering it. `None` where no span covers it.
    cover: [Vec<Option<Cover>>; 6],
    /// Per slot and covered position: the [`Run`] the delimiter sits in.
    runs: [Vec<Option<Run>>; 6],
    literal: Literal,
    /// Every `==` that can close a highlight.
    marks: Vec<usize>,
    /// Every blank line (`\n\n`), where a paragraph ends.
    blanks: Vec<usize>,
    /// The `{/}` that closes each body field, by the field's `{`.
    fields: std::collections::HashMap<usize, usize>,
    /// Whether the reader's fence escapes are unwound inside a code fence:
    /// the converter reads the text a fence holds, while a resolve writes
    /// Markdown back and keeps what the reader escaped.
    unwind: bool,
}

impl<'a> Closers<'a> {
    fn new(source: &str) -> Self {
        let literal = Literal::new(source);
        let patterns = ["++}", "--}", "~~}", "==}", "<<}", "~>"];
        let positions: [Vec<usize>; 6] = patterns.map(|pattern| {
            source
                .match_indices(pattern)
                .map(|(at, _)| at)
                // A closer behind an odd run of backslashes is Markdown's
                // escape, but never one inside a code fence: there the
                // reader breaks every literal delimiter itself, so a bare
                // one always closes, and the backslash is text before it.
                .filter(|&at| {
                    (literal.in_code_fence(at) || !escaped(source, at))
                        && literal.verbatim_end(at).is_none()
                })
                .collect()
        });
        // The complete comments and highlights, as (start, end) pairs:
        // delimiters inside one are its text, so a complete nested span
        // is opaque to the span around it. Pairing is greedy, left to
        // right; an opener inside a completed pair opens nothing (a
        // comment's own text holds no span), so the pairs stay disjoint
        // and sorted.
        let openers = |open: &str| -> Vec<usize> {
            source
                .match_indices(open)
                .map(|(at, _)| at)
                .filter(|&at| !escaped(source, at) && literal.verbatim_end(at).is_none())
                .collect()
        };
        let comments = Self::paired(&openers("{>>"), "<<}", &[], &positions);
        let highlights = Self::paired(&openers("{=="), "==}", &comments, &positions);
        // Comments and highlights sorted by start, overlaps kept: a span
        // that holds another is opaque to a lookup only inside its own
        // extent, so each position's innermost cover decides.
        let mut nested = comments
            .iter()
            .chain(&highlights)
            .copied()
            .collect::<Vec<_>>();
        nested.sort_unstable();

        // For each slot's positions: the innermost nested span covering
        // each one (the last-started, decided alone — a wider span
        // containing it starts earlier and ends later, so it can only
        // be *less* inside the search range), the earliest start and
        // latest end of every span covering it, and the run of
        // consecutive covered positions it sits in. The lookup can then
        // jump a whole run of hidden delimiters in one step.
        let mut cover: [Vec<Option<Cover>>; 6] = std::array::from_fn(|_| Vec::new());
        let mut runs: [Vec<Option<Run>>; 6] = std::array::from_fn(|_| Vec::new());
        for (slot, positions) in positions.iter().enumerate() {
            let mut covered = Vec::with_capacity(positions.len());
            let mut open: std::collections::VecDeque<(usize, usize)> =
                std::collections::VecDeque::new();
            let mut s = 0;
            for &pos in positions {
                while s < nested.len() && nested[s].0 <= pos {
                    open.push_back(nested[s]);
                    s += 1;
                }
                while open.back().is_some_and(|&(_, end)| end <= pos) {
                    open.pop_back();
                }
                while open.front().is_some_and(|&(_, end)| end <= pos) {
                    open.pop_front();
                }
                covered.push(open.back().map(|&inner| {
                    (
                        inner,
                        open.front().copied().unwrap_or(inner).0,
                        open.iter().map(|&(_, end)| end).max().unwrap_or(0),
                    )
                }));
            }
            let mut run = vec![None; positions.len()];
            let mut i = 0;
            while i < positions.len() {
                let Some((_, mut first, mut last)) = covered[i] else {
                    i += 1;
                    continue;
                };
                let mut j = i + 1;
                while j < positions.len() {
                    let Some((_, f, l)) = covered[j] else { break };
                    first = first.min(f);
                    last = last.max(l);
                    j += 1;
                }
                for item in run.iter_mut().take(j).skip(i) {
                    *item = Some((j, first, last));
                }
                i = j;
            }
            cover[slot] = covered;
            runs[slot] = run;
        }

        let bytes = source.as_bytes();
        let marks = (0..bytes.len())
            .filter(|&at| bytes[at..].starts_with(b"==") && mark_closes(source, &literal, at))
            .collect();
        let blanks = (0..bytes.len())
            .filter(|&at| bytes[at..].starts_with(b"\n\n"))
            .collect();
        let fields = field_ends(source, &literal);
        Self {
            patterns,
            positions,
            comments,
            highlights,
            cover,
            runs,
            literal,
            marks,
            blanks,
            fields,
            unwind: true,
        }
    }

    /// The delimiter list slot `pattern` occupies.
    fn slot(pattern: &str) -> usize {
        match pattern {
            "++}" => 0,
            "--}" => 1,
            "~~}" => 2,
            "==}" => 3,
            "<<}" => 4,
            "~>" => 5,
            _ => unreachable!("unknown pattern {pattern}"),
        }
    }

    /// Whether `at` is inside one of `spans`, which must be disjoint and
    /// sorted.
    fn in_spans(spans: &[(usize, usize)], at: usize) -> bool {
        let i = spans.partition_point(|&(_, end)| end <= at);
        spans.get(i).is_some_and(|&(start, _)| start <= at)
    }

    /// The spans `open` opens: each opener, skipping any inside an earlier
    /// pair or a span of `inner`, with the first `close` after its opener
    /// text that `inner` does not hold, so a `{==` whose first `==}` sits
    /// in a comment closes past it, as the recursive reader closes it.
    /// Disjoint and sorted by start. The closer cursor is shared: an
    /// opener never rescans the closers an earlier opener skipped, so
    /// many openers before many `inner` spans stay linear.
    fn paired(
        openers: &[usize],
        close: &str,
        inner: &[(usize, usize)],
        positions: &[Vec<usize>; 6],
    ) -> Vec<(usize, usize)> {
        let mut spans = Vec::new();
        let closers = &positions[Self::slot(close)];
        let mut at = 0;
        for &opener in openers {
            if Self::in_spans(&spans, opener) || Self::in_spans(inner, opener) {
                continue;
            }
            at = at.max(closers.partition_point(|&c| c <= opener));
            let close_at = loop {
                let Some(&c) = closers.get(at) else {
                    break None;
                };
                if Self::in_spans(inner, c) {
                    at += 1;
                    continue;
                }
                break Some(c);
            };
            if let Some(close_at) = close_at {
                spans.push((opener, close_at + close.len()));
                at += 1;
            }
        }
        spans
    }

    /// The reader's fence escapes kept as written, for Markdown a resolve
    /// writes back: its code fences still name the same document.
    fn keeping_fences(mut self) -> Self {
        self.unwind = false;
        self
    }

    /// The first `pattern` that lies whole in `from..until`.
    fn find(&self, pattern: &str, from: usize, until: usize) -> Option<usize> {
        let slot = self.patterns.iter().position(|p| *p == pattern)?;
        let positions = &self.positions[slot];
        let at = *positions.get(positions.partition_point(|&at| at < from))?;
        (at + pattern.len() <= until).then_some(at)
    }

    /// The first `pattern` that lies whole in `from..until` outside every
    /// complete nested span, or inside one whose opener is before `from`
    /// or whose closer is past `until`: a comment or highlight is opaque
    /// only while it is whole inside the range, so a `~>` or `++}` in its
    /// text cannot split or shorten the span around it, and an
    /// unfinished one keeps its delimiters text.
    fn find_outside_nested(&self, pattern: &str, from: usize, until: usize) -> Option<usize> {
        let slot = Self::slot(pattern);
        let positions = &self.positions[slot];
        let mut at = positions.partition_point(|&pos| pos < from);
        loop {
            let pos = *positions.get(at)?;
            if pos + pattern.len() > until {
                return None;
            }
            let Some((inner, ..)) = self.cover[slot][at] else {
                return Some(pos);
            };
            if inner.0 < from || inner.1 > until {
                // The span holding this delimiter crosses the range: the
                // delimiter counts.
                return Some(pos);
            }
            let (next, first, last) = self.runs[slot][at].expect("a covered position has a run");
            at = if first >= from && last <= until {
                // Every span covering a position of the run is inside
                // the range: one jump past them all.
                next
            } else {
                // The run reaches out of the range: only this
                // delimiter's innermost span is certainly opaque.
                positions.partition_point(|&p| p < inner.1)
            };
        }
    }

    /// Where the paragraph `from` lies in ends: at its blank line, or at
    /// `until` if that comes first.
    fn paragraph_end(&self, from: usize, until: usize) -> usize {
        self.blanks
            .get(self.blanks.partition_point(|&at| at < from))
            .filter(|&&at| at + 2 <= until)
            .map_or(until, |&at| at)
    }
}

/// The parts of a Markdown source that Markdown reads literally: code spans,
/// link destinations and autolinks (`verbatim`), where nothing opens, and
/// fenced code blocks and the YAML front matter (`fences`), where
/// CriticMarkup still marks tracked changes (the reader prints them there)
/// but `==` and `{CODE}` open nothing: a header's `{PAGE}` is read by the
/// front matter's own reader. Sorted, not overlapping.
struct Literal {
    verbatim: Vec<std::ops::Range<usize>>,
    fences: Vec<std::ops::Range<usize>>,
    /// Where the YAML front matter ends, when the source opens with one:
    /// a fence for Markdown's marks, but no code block whose escapes the
    /// reader writes and [`encode_range`] unwinds.
    front_matter: usize,
}

impl Literal {
    fn new(source: &str) -> Self {
        let mut literal = Self {
            verbatim: Vec::new(),
            fences: Vec::new(),
            front_matter: 0,
        };
        let bytes = source.as_bytes();
        let mut fence: Option<(u8, usize, usize)> = None;
        // Whether a link reference definition may start here: it cannot
        // interrupt a paragraph.
        let mut can_define = true;
        let destinations = destination_ends(source);
        let mut line_start = 0;
        if source.starts_with("---\n") || source.starts_with("---\r\n") {
            let mut end = source.find('\n').map_or(source.len(), |n| n + 1);
            while end < source.len() {
                let line_end = source[end..].find('\n').map_or(source.len(), |n| end + n);
                let closes = matches!(source[end..line_end].trim_end(), "---" | "...");
                end = (line_end + 1).min(source.len());
                if closes {
                    literal.fences.push(0..line_end);
                    literal.front_matter = line_end;
                    line_start = end;
                    break;
                }
            }
        }
        let mut at = line_start;
        while at < bytes.len() {
            if at == line_start {
                let line_end = source[at..].find('\n').map_or(bytes.len(), |n| at + n);
                let line = &source[at..line_end];
                let indent = line.len() - line.trim_start().len();
                let run = |b: u8| line.bytes().skip(indent).take_while(|&c| c == b).count();
                match fence {
                    Some((b, width, start)) => {
                        if indent < 4
                            && run(b) >= width
                            && line[indent + run(b)..].trim().is_empty()
                        {
                            literal.fences.push(start..line_end);
                            fence = None;
                        }
                        can_define = true;
                        at = line_end + 1;
                        line_start = at;
                        continue;
                    }
                    None => {
                        for b in *b"`~" {
                            let width = run(b);
                            let info = &line[indent + width..];
                            // Four spaces of indent make indented code, not
                            // a fence: Markdown reads its text verbatim.
                            if indent < 4 && width >= 3 && !(b == b'`' && info.contains('`')) {
                                fence = Some((b, width, at));
                            }
                        }
                        let destination =
                            can_define.then(|| definition_destination(line)).flatten();
                        can_define = fence.is_some()
                            || destination.is_some()
                            || line.trim().is_empty()
                            || line.trim_start().starts_with('#');
                        if let Some(from) = destination {
                            literal.verbatim.push(at + from..line_end);
                        }
                        if fence.is_some() || destination.is_some() {
                            at = line_end + 1;
                            line_start = at;
                            continue;
                        }
                    }
                }
            }
            let from = at;
            let end = match bytes[at] {
                b'`' if !escaped(source, at) => code_span_end(source, at),
                b'(' if at > 0 && bytes[at - 1] == b']' && !escaped(source, at - 1) => {
                    destinations.get(&(at + 1)).copied()
                }
                b'<' if !escaped(source, at) => autolink_end(source, at),
                _ => None,
            };
            match end {
                Some(end) => {
                    literal.verbatim.push(at..end);
                    at = end;
                }
                None if bytes[at] == b'`' => {
                    at += bytes[at..].iter().take_while(|&&b| b == b'`').count();
                }
                None => at += source[at..].chars().next().map_or(1, char::len_utf8),
            }
            if let Some(n) = source[from..at].rfind('\n') {
                line_start = from + n + 1;
            }
        }
        if let Some((.., start)) = fence {
            literal.fences.push(start..bytes.len());
        }
        // CommonMark's indented code blocks hold their text verbatim,
        // like a code span: nothing opens in them, and the reader never
        // writes them, so no fence escape is unwound there either.
        literal.verbatim.extend(indented_code(source));
        literal.verbatim.sort_by_key(|range| range.start);
        literal
    }

    /// Where the code span, link destination or autolink that `at` lies in
    /// ends, if it lies in one.
    fn verbatim_end(&self, at: usize) -> Option<usize> {
        let i = self.verbatim.partition_point(|r| r.end <= at);
        self.verbatim
            .get(i)
            .filter(|r| r.start <= at)
            .map(|r| r.end)
    }

    /// Whether `at` lies in a fenced code block.
    fn in_fence(&self, at: usize) -> bool {
        let i = self.fences.partition_point(|r| r.end <= at);
        self.fences.get(i).is_some_and(|r| r.start <= at)
    }

    /// Whether `at` lies in a fenced code block, not the front matter: the
    /// reader escapes a code block's text for the writer to unwind here.
    fn in_code_fence(&self, at: usize) -> bool {
        self.in_fence(at) && at >= self.front_matter
    }
}

/// The byte ranges of CommonMark's indented code blocks: a line indented
/// four or more spaces that nothing contains is one, and pulldown-cmark
/// finds it, a list item's own indent included. Its text is verbatim, like
/// a code span's.
fn indented_code(source: &str) -> Vec<std::ops::Range<usize>> {
    use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};

    // Nothing indented: no such block to find, and a long document is not
    // parsed for one it cannot hold.
    if !source
        .lines()
        .any(|line| line.starts_with("    ") || line.starts_with('\t'))
    {
        return Vec::new();
    }
    Parser::new_ext(source, super::write::parser_options())
        .into_offset_iter()
        .filter_map(|(event, range)| {
            matches!(event, Event::Start(Tag::CodeBlock(CodeBlockKind::Indented))).then_some(range)
        })
        .collect()
}

/// The end of the code span a backtick run at `at` opens: after the next
/// run of as many backticks, in the same paragraph.
fn code_span_end(source: &str, at: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let width = bytes[at..].iter().take_while(|&&b| b == b'`').count();
    let mut i = at + width;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let run = bytes[i..].iter().take_while(|&&b| b == b'`').count();
            if run == width {
                return Some(i + run);
            }
            i += run;
        } else if bytes[i..].starts_with(b"\n\n") {
            return None;
        } else {
            i += 1;
        }
    }
    None
}

/// Where the destination (and title) of a link reference definition
/// (`[label]: destination "title"`) starts in `line`, if the line is one.
/// A footnote definition (`[^1]:`) is not one.
fn definition_destination(line: &str) -> Option<usize> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    let label = line.get(indent..)?.strip_prefix('[')?;
    if indent > 3 || label.starts_with('^') {
        return None;
    }
    let mut chars = label.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '[' => return None,
            ']' => {
                return (i > 0 && label[i + 1..].starts_with(':')).then_some(indent + 1 + i + 2);
            }
            _ => {}
        }
    }
    None
}

/// Where each link destination (and title) closes, by where it starts,
/// after `](`: its closing `)`, parentheses balanced, on one line. One pass
/// over each line and one back, so a line of unclosed `](` is not scanned
/// again from each.
fn destination_ends(source: &str) -> std::collections::HashMap<usize, usize> {
    let mut ends = std::collections::HashMap::new();
    let mut line_start = 0;
    for line in source.split('\n') {
        // Each character's position, with the parenthesis level before and
        // after it; a destination starting at `s` closes at the first `)`
        // that brings the level below the one before `s`.
        let mut levels = Vec::new();
        let mut starts = Vec::new();
        let (mut level, mut backslashes) = (0i64, 0usize);
        // The last two characters: whether an unescaped `]` came just
        // before, and whether `](` did.
        let (mut bracket, mut opened) = (false, false);
        for (i, c) in line.char_indices() {
            let at = line_start + i;
            if opened {
                starts.push(at);
            }
            let free = backslashes % 2 == 0;
            let before = level;
            if free {
                match c {
                    '(' => level += 1,
                    ')' => level -= 1,
                    _ => {}
                }
            }
            opened = bracket && c == '(';
            bracket = free && c == ']';
            backslashes = if c == '\\' { backslashes + 1 } else { 0 };
            levels.push((at, before, level));
        }
        let mut nearest = std::collections::HashMap::new();
        let mut starts = starts.iter().rev().peekable();
        for &(at, before, after) in levels.iter().rev() {
            nearest.insert(after, at);
            if starts.next_if(|&&s| s == at).is_some()
                && let Some(&end) = nearest.get(&(before - 1))
            {
                ends.insert(at, end);
            }
        }
        line_start += line.len() + 1;
    }
    ends
}

/// The end of the HTML comment that starts at `at` (`<!-- … -->`), if it
/// closes. [`verbatim_ranges`] counts a comment whole; the CriticMarkup
/// reader does not (#520: a closer inside one still closes its span).
fn comment_end(source: &str, at: usize) -> Option<usize> {
    if !source[at..].starts_with("<!--") {
        return None;
    }
    source[at + 4..].find("-->").map(|close| at + 4 + close + 3)
}

/// The end of an autolink (`<https://a.example>`) at `at`: a scheme, `:`,
/// then no space or `<` up to `>`.
fn autolink_end(source: &str, at: usize) -> Option<usize> {
    let rest = &source[at + 1..];
    let scheme = rest
        .bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'.' | b'-'))
        .count();
    if !(2..=32).contains(&scheme) || rest.as_bytes().get(scheme) != Some(&b':') {
        return None;
    }
    let close = rest.find(|c: char| c == '>' || c == '<' || c.is_whitespace())?;
    (rest.as_bytes()[close] == b'>').then_some(at + 1 + close + 1)
}

/// What a whole-`encode` pass found amiss, for a caller that warns:
/// positions, line numbers resolved at the end so the scan stays cheap.
#[derive(Default)]
struct Check {
    /// Every opener that closed nothing: (position, opener, what it
    /// lacks, whether what it lacks is a separator).
    unclosed: Vec<(usize, &'static str, &'static str, bool)>,
    /// Every delimiter position that closed or split a span.
    consumed: Vec<usize>,
}

/// [`encode`], opening only the spans `allowed` names, moving footnote
/// labels out of spans only when `labels`, and noting malformed
/// delimiters in `check`.
fn encode_spans_checked(source: &str, allowed: Allowed, labels: bool, check: &mut Check) -> String {
    let mut out = String::with_capacity(source.len());
    encode_range(
        source,
        &Closers::new(source),
        0..source.len(),
        allowed,
        labels,
        check,
        &mut out,
    );
    out
}

/// [`encode`], opening only the spans `allowed` names, and moving footnote
/// labels out of spans only when `labels`.
fn encode_spans(source: &str, allowed: Allowed, labels: bool) -> String {
    encode_spans_checked(source, allowed, labels, &mut Check::default())
}

/// [`encode_spans`] with the reader's fence escapes kept as written: the
/// resolved Markdown still names the document its code blocks hold.
fn encode_spans_kept(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    encode_range(
        source,
        &Closers::new(source).keeping_fences(),
        0..source.len(),
        ALL,
        false,
        &mut Check::default(),
        &mut out,
    );
    out
}

/// [`encode_spans`] for the part of `source` in `range`.
fn encode_range(
    source: &str,
    closers: &Closers<'_>,
    range: std::ops::Range<usize>,
    allowed: Allowed,
    labels: bool,
    check: &mut Check,
    out: &mut String,
) {
    let mut at = range.start;
    while at < range.end {
        if let Some(end) = closers
            .literal
            .verbatim_end(at)
            .filter(|&end| end <= range.end)
        {
            push_escaped(out, &source[at..end]);
            at = end;
            continue;
        }
        let rest = &source[at..range.end];
        let here = if closers.literal.in_fence(at) {
            allowed & !(MARK | FIELD)
        } else {
            allowed
        };
        // The reader broke the delimiter shapes of a code block's text for
        // this fence, where Markdown itself reads no backslash escape, so
        // they are unbroken here and nowhere else: a bare delimiter is the
        // change the reader wrote, never Markdown-escaped by what precedes
        // it.
        if closers.unwind
            && closers.literal.in_code_fence(at)
            && let Some((text, len)) = fence_escape_at(source, at)
        {
            push_escaped(out, &text);
            at += len;
            continue;
        }
        let span = SPANS
            .iter()
            .find(|(open, ..)| rest.starts_with(open))
            .filter(|span| allowed & kind_of(span.3) != 0)
            .filter(|_| closers.literal.in_code_fence(at) || !escaped(source, at))
            .and_then(|&(open, close, separator, start, end)| {
                let from = at + open.len();
                // A span's closer (and a substitution's `~>` separator)
                // skips every complete nested span: a `{++a{>>b++}c++}`
                // looks for the insertion's `++}` after `a`, the `++}`
                // inside the comment is the comment's text, and the
                // second `++}` is the closer. Comments close at the
                // first `<<}` as before: nothing nests in one.
                let skip_nested = start != Token::CommentStart;
                let to = if skip_nested {
                    closers.find_outside_nested(close, from, range.end)
                } else {
                    closers.find(close, from, range.end)
                };
                let Some(to) = to else {
                    check.unclosed.push((at, open, close, false));
                    return None;
                };
                check.consumed.push(to);
                let split = match separator {
                    Some(separator) => {
                        let mid = if skip_nested {
                            closers.find_outside_nested(separator, from, to)
                        } else {
                            closers.find(separator, from, to)
                        };
                        match mid {
                            Some(mid) => {
                                check.consumed.push(mid);
                                Some((mid, separator.len()))
                            }
                            None => {
                                check.unclosed.push((at, open, separator, true));
                                return None;
                            }
                        }
                    }
                    None => None,
                };
                Some((from..to, to + close.len(), split, start, end))
            });
        match span {
            Some((mut body, after, split, start, end)) => {
                let nested = inside(kind_of(start), allowed);
                if labels
                    && start != Token::CommentStart
                    && (at == 0 || source[..at].ends_with('\n'))
                    && let Some(label) = footnote_label(&source[body.clone()])
                    && split.is_none_or(|(mid, _)| mid >= body.start + label.len())
                {
                    push_escaped(out, label);
                    body.start += label.len();
                }
                out.push(stand_in(start));
                match split {
                    Some((mid, width)) => {
                        encode_range(source, closers, body.start..mid, nested, false, check, out);
                        out.push(stand_in(Token::SubstituteSeparator));
                        encode_range(
                            source,
                            closers,
                            mid + width..body.end,
                            nested,
                            false,
                            check,
                            out,
                        );
                    }
                    None if start == Token::CommentStart => {
                        encode_comment(source, closers, body, out);
                    }
                    None => encode_range(source, closers, body, nested, false, check, out),
                }
                out.push(stand_in(end));
                at = after;
            }
            None => {
                if here & FIELD != 0
                    && let Some((code, end)) = field_at(source, closers, at, range.end)
                {
                    out.push(stand_in(Token::FieldStart));
                    // The code is text to the end: no Markdown reads it.
                    for c in source[code.clone()].trim().chars() {
                        if c.is_ascii_punctuation() {
                            out.push('\\');
                        }
                        push_escaped(out, c.encode_utf8(&mut [0; 4]));
                    }
                    out.push(stand_in(Token::FieldSeparator));
                    encode_range(
                        source,
                        closers,
                        code.end + 1..end,
                        allowed,
                        labels,
                        check,
                        out,
                    );
                    out.push(stand_in(Token::FieldEnd));
                    at = end + FIELD_END.len();
                    continue;
                }
                if here & MARK != 0
                    && let Some(close) = mark_close(source, closers, at, range.end)
                {
                    out.push(stand_in(Token::MarkStart));
                    encode_range(source, closers, at + 2..close, allowed, false, check, out);
                    out.push(stand_in(Token::MarkEnd));
                    at = close + 2;
                    continue;
                }
                let c = rest.chars().next().unwrap_or_default();
                // A character reference naming a stand-in or the escape:
                // escaped here, so the character the parser decodes
                // stays text. In a fence the parser decodes nothing, so
                // the spelling keeps its own.
                if c == '&'
                    && !escaped(source, at)
                    && !closers.literal.in_fence(at)
                    && let Some(len) = reference_stand_in(rest)
                {
                    out.push(ESCAPE);
                    push_escaped(out, &rest[..len]);
                    at += len;
                    continue;
                }
                push_escaped(out, &rest[..c.len_utf8()]);
                at += c.len_utf8();
            }
        }
    }
}

/// A comment's body: text, with every run of blank or whitespace-only
/// lines as one [`Token::CommentBreak`] stand-in so the comment's
/// paragraphs survive the Markdown parser without splitting the
/// paragraph the comment is anchored on. A single newline stays text,
/// and a character reference naming a stand-in stays literal as in any
/// other text. A comment anchored in a code block unwinds the reader's
/// fence escapes in its body, as the fence's own text does.
fn encode_comment(
    source: &str,
    closers: &Closers<'_>,
    body: std::ops::Range<usize>,
    out: &mut String,
) {
    let mut at = body.start;
    while at < body.end {
        if closers.unwind
            && closers.literal.in_code_fence(at)
            && let Some((text, len)) = fence_escape_at(&source[..body.end], at)
        {
            push_escaped(out, &text);
            at += len;
            continue;
        }
        if source.as_bytes()[at] == b'&'
            && let Some(len) = reference_stand_in(&source[at..body.end])
        {
            out.push(ESCAPE);
            push_escaped(out, &source[at..at + len]);
            at += len;
            continue;
        }
        if source.as_bytes()[at] == b'\n' {
            // The whitespace run after this newline; a second newline in
            // it makes a blank line.
            let rest = &source[at + 1..body.end];
            let white = rest.len() - rest.trim_start_matches([' ', '\t', '\r', '\n']).len();
            if rest[..white].contains('\n') {
                out.push(stand_in(Token::CommentBreak));
                at += 1 + white;
                continue;
            }
        }
        let c = source[at..body.end].chars().next().unwrap_or_default();
        push_escaped(out, &source[at..at + c.len_utf8()]);
        at += c.len_utf8();
    }
}

/// The reader's fence escape (`from_docx::critic::escape_code`) at `at`,
/// if one sits there: the document text it names, and how many bytes it
/// takes. A delimiter shape with an odd run of backslashes at its break
/// point names the run halved and the delimiter as text; one with an even
/// run cannot come from the reader, so it stays as written and opens
/// nothing. A bare delimiter (no run) is no escape: it is the change the
/// reader wrote, which Markdown reads here, where it reads no backslash
/// escape of its own.
fn fence_escape_at(source: &str, at: usize) -> Option<(String, usize)> {
    /// What can follow the backslash run an opener holds (`{` + run).
    const OPENER: [&str; 5] = ["++", "--", "~~", ">>", "=="];
    /// What can hold the run a closer escapes (`++` + run + `}`).
    const CLOSER: [&str; 5] = ["++", "--", "~~", "<<", "=="];
    let bytes = source.as_bytes();
    let (head, run, tail) = {
        let shape = |head: usize, tails: &[&str]| {
            let mut run = at + head;
            while bytes.get(run) == Some(&b'\\') {
                run += 1;
            }
            let tail = tails
                .iter()
                .find(|tail| source[run..].starts_with(*tail))?
                .len();
            Some((head, run - (at + head), tail))
        };
        if bytes.get(at) == Some(&b'{') {
            shape(1, &OPENER)
        } else if CLOSER.iter().any(|head| source[at..].starts_with(head)) {
            shape(2, &["}"])
        } else if bytes.get(at) == Some(&b'~') {
            shape(1, &[">"])
        } else {
            None
        }
    }?;
    if run == 0 {
        return None;
    }
    let whole = head + run + tail;
    let mut text = String::with_capacity(whole);
    if run % 2 == 0 {
        // An even run cannot come from the reader: it stays as written,
        // every backslash of it, and the delimiter opens nothing.
        text.push_str(&source[at..at + whole]);
    } else {
        text.push_str(&source[at..at + head]);
        for _ in 0..(run - 1) / 2 {
            text.push('\\');
        }
        text.push_str(&source[at + head + run..at + whole]);
    }
    Some((text, whole))
}

/// What closes a body field.
const FIELD_END: &str = "{/}";

/// The field names ECMA-376 Part 1 §17.16.5 lists. Only a code that starts
/// with one is a field, so a placeholder such as `{Client Name}` stays text.
const FIELD_NAMES: [&str; 76] = [
    "ADDRESSBLOCK",
    "ADVANCE",
    "ASK",
    "AUTHOR",
    "AUTONUM",
    "AUTONUMLGL",
    "AUTONUMOUT",
    "AUTOTEXT",
    "AUTOTEXTLIST",
    "BARCODE",
    "BIBLIOGRAPHY",
    "BIDIOUTLINE",
    "CITATION",
    "COMMENTS",
    "COMPARE",
    "CREATEDATE",
    "DATABASE",
    "DATE",
    "DISPLAYBARCODE",
    "DOCPROPERTY",
    "DOCVARIABLE",
    "EDITTIME",
    "EQ",
    "FILENAME",
    "FILESIZE",
    "FILLIN",
    "FORMCHECKBOX",
    "FORMDROPDOWN",
    "FORMTEXT",
    "GOTOBUTTON",
    "GREETINGLINE",
    "HYPERLINK",
    "IF",
    "INCLUDEPICTURE",
    "INCLUDETEXT",
    "INDEX",
    "INFO",
    "KEYWORDS",
    "LASTSAVEDBY",
    "LINK",
    "LISTNUM",
    "MACROBUTTON",
    "MERGEBARCODE",
    "MERGEFIELD",
    "MERGEREC",
    "MERGESEQ",
    "NEXT",
    "NEXTIF",
    "NOTEREF",
    "NUMCHARS",
    "NUMPAGES",
    "NUMWORDS",
    "PAGE",
    "PAGEREF",
    "PRINT",
    "PRINTDATE",
    "QUOTE",
    "RD",
    "REF",
    "REVNUM",
    "SAVEDATE",
    "SECTION",
    "SECTIONPAGES",
    "SEQ",
    "SET",
    "SKIPIF",
    "STYLEREF",
    "SUBJECT",
    "SYMBOL",
    "TA",
    "TC",
    "TEMPLATE",
    "TIME",
    "TITLE",
    "TOA",
    "TOC",
];

/// Whether `code` is a field code jubarte prints as `{CODE}`: one line,
/// braces only inside a quoted argument (`QUOTE "a}b"`), quotes closed, and
/// a field name from [`FIELD_NAMES`] (in any case) first.
pub(crate) fn is_field_code(code: &str) -> bool {
    let code = code.trim();
    let name = code.split([' ', '\\']).next().unwrap_or_default();
    let closed = format!("{code}}}");
    field_code_close(closed.char_indices()) == Some(code.len())
        && FIELD_NAMES.iter().any(|f| f.eq_ignore_ascii_case(name))
}

/// The longest field code read, in characters: no field code comes near it,
/// and it keeps a paragraph of `{` from being scanned once per brace.
const LONGEST_FIELD_CODE: usize = 1024;

/// Where the `}` that closes a field code sits, given the characters after
/// its `{` with their positions: a brace inside a quoted argument (`"a}b"`,
/// `\"` quoting a quote) belongs to the code. None at an unquoted `{`, a
/// line end, or no `}` within [`LONGEST_FIELD_CODE`].
pub(crate) fn field_code_close(chars: impl Iterator<Item = (usize, char)>) -> Option<usize> {
    let mut quoted = false;
    let mut backslash = false;
    for (at, c) in chars.take(LONGEST_FIELD_CODE) {
        match c {
            '\n' | '\r' => return None,
            '"' if !backslash => quoted = !quoted,
            '{' if !quoted => return None,
            '}' if !quoted => return Some(at),
            _ => {}
        }
        backslash = c == '\\' && !backslash;
    }
    None
}

/// The code of a body field opened at `at` (`{` then a field code, up to the
/// next `}` on the line), if `at` opens one.
fn field_code(source: &str, at: usize, until: usize) -> Option<std::ops::Range<usize>> {
    let rest = source.get(at..until)?;
    if !rest.starts_with('{') || escaped(source, at) {
        return None;
    }
    let close = 1 + field_code_close(rest[1..].char_indices())?;
    is_field_code(&rest[1..close]).then_some(at + 1..at + close)
}

/// A body field opened at `at`: its code, and where the `{/}` that closes
/// it sits (before `until`), past any field nested in its result: in the
/// same paragraph, or a later one when its `{CODE}` begins a line (see
/// [`field_ends`]). Without that `{/}` it is text.
fn field_at(
    source: &str,
    closers: &Closers<'_>,
    at: usize,
    until: usize,
) -> Option<(std::ops::Range<usize>, usize)> {
    let code = field_code(source, at, until)?;
    let end = *closers.fields.get(&at)?;
    (end + FIELD_END.len() <= until).then_some((code, end))
}

/// The `{/}` that closes each body field, by the field's `{`, found in one
/// pass with a stack of open fields: a nested field's `{/}` closes it
/// first, and code spans, link destinations and autolinks are passed over.
/// A blank line leaves unclosed every open field but one whose `{CODE}`
/// begins its paragraph's text (a table of contents: its result is whole
/// paragraphs, its `{/}` in a later one); a fenced block leaves every open
/// field unclosed. A `{`
/// inside a field's code (`{QUOTE "{PAGE}"}`) opens nothing.
fn field_ends(source: &str, literal: &Literal) -> std::collections::HashMap<usize, usize> {
    let mut ends = std::collections::HashMap::new();
    let mut open: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < source.len() {
        if let Some(skip) = literal.verbatim_end(i) {
            i = skip;
        } else if literal.in_fence(i) {
            open.clear();
            i += source[i..].chars().next().map_or(1, char::len_utf8);
        } else if source[i..].starts_with("\n\n") {
            open.retain(|&start| opens_paragraph(source, start));
            i += 1;
        } else if source[i..].starts_with(FIELD_END) && !escaped(source, i) {
            if let Some(start) = open.pop() {
                ends.insert(start, i);
            }
            i += FIELD_END.len();
        } else if let Some(code) = field_code(source, i, source.len()) {
            open.push(i);
            i = code.end + 1;
        } else {
            i += source[i..].chars().next().map_or(1, char::len_utf8);
        }
    }
    ends
}

/// Whether the `{` at `at` begins its paragraph's text: only a heading,
/// list or quote marker (`## `, `- `, `1. `, `> `) and spaces before it on
/// its line.
fn opens_paragraph(source: &str, at: usize) -> bool {
    let line = &source[source[..at].rfind('\n').map_or(0, |n| n + 1)..at];
    let mut rest = line.trim_start();
    loop {
        let marker = if let Some(after) = rest.strip_prefix('>') {
            after
        } else if rest.starts_with('#') {
            rest.trim_start_matches('#')
                .strip_prefix(' ')
                .unwrap_or(rest)
        } else if let Some(after) = rest.strip_prefix(['-', '*', '+']) {
            after.strip_prefix(' ').unwrap_or(rest)
        } else {
            let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
            rest[digits..]
                .strip_prefix(['.', ')'])
                .filter(|_| digits > 0)
                .and_then(|after| after.strip_prefix(' '))
                .unwrap_or(rest)
        };
        if marker.len() == rest.len() {
            return rest.is_empty();
        }
        rest = marker.trim_start();
    }
}

/// Where the `==` that closes a highlight opened at `at` sits, if `at`
/// opens one: `==` touching text on its inner side and not part of `{==`,
/// `==}` or a longer run of `=`, closed by such an `==` in the same
/// paragraph (before `until` and any blank line).
fn mark_close(source: &str, closers: &Closers<'_>, at: usize, until: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let is = |i: usize, b: u8| i < bytes.len() && bytes[i] == b;
    let blank = |i: usize| i >= bytes.len() || bytes[i].is_ascii_whitespace();
    let opens = source[at..until].starts_with("==")
        && !escaped(source, at)
        && !(at > 0 && (is(at - 1, b'=') || is(at - 1, b'{')))
        && !is(at + 2, b'=')
        && !is(at + 2, b'}')
        && !blank(at + 2);
    if !opens {
        return None;
    }
    let end = closers.paragraph_end(at + 2, until);
    let from = at + 2 + source[at + 2..].chars().next().map_or(1, char::len_utf8);
    let marks = &closers.marks;
    let close = *marks.get(marks.partition_point(|&close| close < from))?;
    (close + 2 <= end).then_some(close)
}

/// Whether the `==` at `close` can close a highlight: touching text on its
/// inner side, not part of `==}` or a longer run of `=`, and read as
/// Markdown.
fn mark_closes(source: &str, literal: &Literal, close: usize) -> bool {
    let bytes = source.as_bytes();
    let is = |i: usize, b: u8| i < bytes.len() && bytes[i] == b;
    let blank = |i: usize| i >= bytes.len() || bytes[i].is_ascii_whitespace();
    close > 0
        && !blank(close - 1)
        && !is(close - 1, b'=')
        && !is(close + 2, b'=')
        && !is(close + 2, b'}')
        && !escaped(source, close)
        && literal.verbatim_end(close).is_none()
        && !literal.in_fence(close)
}

/// The footnote label that starts `text` with the spaces after it
/// (`[^note]: `), as CommonMark footnote definitions are written.
fn footnote_label(text: &str) -> Option<&str> {
    let name = text.strip_prefix("[^")?;
    let close = name.find(']')?;
    if close == 0 || name[..close].contains(char::is_whitespace) {
        return None;
    }
    let after = name[close..].strip_prefix("]:")?;
    let spaces = after.len() - after.trim_start_matches([' ', '\t']).len();
    Some(&text[..2 + close + 2 + spaces])
}

/// [`encode`] without moving footnote labels: the spans as written, for
/// reading the markup rather than the Markdown.
pub(crate) fn encode_spans_only(source: &str) -> String {
    encode_spans(source, ALL, false)
}

/// The ranges of `source` the Markdown diff escaper must leave alone:
/// the reader's own verbatim (code spans across single line breaks, link
/// destinations, autolinks) plus every complete HTML comment — a
/// backslash inside one changes the comment's text, and the delimiter
/// the escaper would escape drops with the comment anyway. The
/// CriticMarkup reader still counts a delimiter inside a comment (#520:
/// [`Literal`] leaves comments to it), so the comment ranges live here,
/// not in [`Literal`]. Sorted, disjoint.
pub(super) fn verbatim_ranges(source: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Literal::new(source).verbatim;
    let mut at = 0;
    while let Some(open) = source[at..].find("<!--") {
        let open = at + open;
        match comment_end(source, open) {
            Some(end) => {
                ranges.push(open..end);
                at = end;
            }
            // An unclosed comment is text to the reader, not a comment.
            None => at = open + "<!--".len(),
        }
    }
    ranges.sort_by_key(|range| range.start);
    let mut disjoint: Vec<std::ops::Range<usize>> = Vec::with_capacity(ranges.len());
    for range in ranges {
        match disjoint.last_mut() {
            Some(last) if last.end >= range.start => last.end = last.end.max(range.end),
            _ => disjoint.push(range),
        }
    }
    disjoint
}

/// The source of text that holds no stand-ins, as it was.
pub(crate) fn decode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pieces = Pieces::default();
    pieces.feed(text, |piece| match piece {
        Piece::Text(text) => out.push_str(text),
        // A comment paragraph break decodes as a space: `decode` is for
        // text that holds no stand-ins, and one in a caller's text means
        // the caller saw a blank line.
        Piece::Token(Token::CommentBreak) => out.push(' '),
        Piece::Token(_) => {}
    });
    out
}

/// The source of encoded text as it was written, delimiters and all.
pub(crate) fn restore(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pieces = Pieces::default();
    pieces.feed(text, |piece| match piece {
        Piece::Text(text) => out.push_str(text),
        Piece::Token(token) => out.push_str(delimiter(token)),
    });
    out
}

/// The CriticMarkup a token stands in for.
pub(crate) fn delimiter(token: Token) -> &'static str {
    match token {
        Token::InsertStart => "{++",
        Token::InsertEnd => "++}",
        Token::DeleteStart => "{--",
        Token::DeleteEnd => "--}",
        Token::SubstituteStart => "{~~",
        Token::SubstituteSeparator => "~>",
        Token::SubstituteEnd => "~~}",
        Token::HighlightStart => "{==",
        Token::HighlightEnd => "==}",
        Token::CommentStart => "{>>",
        Token::CommentEnd => "<<}",
        Token::CommentBreak => "\n\n",
        Token::FieldStart => "{",
        Token::FieldSeparator => "}",
        Token::FieldEnd => FIELD_END,
        Token::MarkStart | Token::MarkEnd => "==",
    }
}

/// Text or a delimiter, in source order.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Piece<'a> {
    Text(&'a str),
    Token(Token),
}

/// Splits parsed text into [`Piece`]s. The escape character can end one
/// text event and its character begin the next, so the splitter carries it.
#[derive(Default)]
pub(crate) struct Pieces {
    escape_pending: bool,
}

impl Pieces {
    /// Calls `emit` with the pieces of `text`. Escaped characters come out as
    /// text; escape characters themselves never do.
    pub(crate) fn feed(&mut self, text: &str, mut emit: impl FnMut(Piece<'_>)) {
        let mut start = 0;
        for (at, c) in text.char_indices() {
            if self.escape_pending {
                self.escape_pending = false;
                start = at;
                continue;
            }
            if c == ESCAPE {
                if start < at {
                    emit(Piece::Text(&text[start..at]));
                }
                self.escape_pending = true;
                start = at + c.len_utf8();
                continue;
            }
            if let Some(token) = token_of(c) {
                if start < at {
                    emit(Piece::Text(&text[start..at]));
                }
                emit(Piece::Token(token));
                start = at + c.len_utf8();
            }
        }
        if start < text.len() {
            emit(Piece::Text(&text[start..]));
        }
    }
}

/// The Markdown with its CriticMarkup resolved: insertions kept and
/// deletions dropped (`accept`), or the other way round; highlights keep
/// their text and comments go.
pub(crate) fn resolve(markdown: &str, accept: bool) -> String {
    resolve_with_policy(markdown, accept, false)
}

/// Whether `note` names, after ` on `, a change the one it follows sits
/// inside (either view's grammar): resolving that change removes its text
/// either way, for rejecting the outer removes what it holds and accepting
/// the inner removes the text.
fn names_outer(note: &str) -> bool {
    super::view_notes::nested_revision_tags(note).is_some_and(|tags| tags.len() > 1)
        || super::view_notes::nested_plain_revisions(note).is_some_and(|parts| parts.len() > 1)
}

/// Accepted document clauses omit a wholly deleted paragraph's line break.
pub(crate) fn accept_clauses(markdown: &str) -> String {
    resolve_with_policy(markdown, true, true)
}

fn resolve_with_policy(markdown: &str, accept: bool, remove_deleted_lines: bool) -> String {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum In {
        Text,
        Inserted,
        Deleted,
        Old,
        New,
        Comment,
    }
    let mut out = String::with_capacity(markdown.len());
    let mut line_start = 0;
    let mut visible_text = false;
    let mut removed_text = false;
    // A wholly deleted line was just dropped, so the blank separator that
    // follows it should collapse rather than leave a phantom blank line.
    let mut dropped_line = false;
    // The state each open span interrupted, innermost last.
    let mut outer: Vec<In> = Vec::new();
    let mut state = In::Text;
    // The change span just closed, where its note would sit: a note that
    // names the change it sits inside (` on `) folds it, and resolving
    // removes its text either way — rejecting the outer removes what it
    // holds, accepting the inner removes the text.
    let mut folded_at: Option<usize> = None;
    let mut comment = String::new();
    Pieces::default().feed(&encode_spans_kept(markdown), |piece| match piece {
        Piece::Text(text) => {
            if state == In::Comment {
                comment.push_str(text);
            }
            let shown = match state {
                In::Text => true,
                In::Inserted | In::New => accept,
                In::Deleted | In::Old => !accept,
                In::Comment => false,
            };
            if shown {
                if state == In::Text {
                    // Text after the change ended: a note now is on this
                    // point, not on the change.
                    folded_at = None;
                }
                if remove_deleted_lines {
                    for ch in text.chars() {
                        if ch == '\n' {
                            let empty_line = out.len() == line_start;
                            if removed_text && !visible_text {
                                out.truncate(line_start);
                                dropped_line = true;
                            } else if dropped_line
                                && empty_line
                                && (out.is_empty() || out.ends_with("\n\n"))
                            {
                                // The blank separator left by a dropped
                                // paragraph: skip it so an accepted
                                // whole-paragraph deletion collapses the
                                // surrounding blank lines to one.
                                dropped_line = false;
                            } else {
                                out.push(ch);
                                dropped_line = false;
                            }
                            line_start = out.len();
                            visible_text = false;
                            removed_text = false;
                        } else {
                            out.push(ch);
                            visible_text |= !ch.is_whitespace();
                        }
                    }
                } else {
                    out.push_str(text);
                }
            }
        }
        Piece::Token(token) => {
            if remove_deleted_lines && matches!(token, Token::DeleteStart | Token::SubstituteStart)
            {
                removed_text = true;
            }
            let opened = match token {
                // A highlight run or a field is Markdown, not a change: it
                // stays.
                Token::MarkStart
                | Token::MarkEnd
                | Token::FieldStart
                | Token::FieldSeparator
                | Token::FieldEnd => {
                    let shown = match state {
                        In::Text => true,
                        In::Inserted | In::New => accept,
                        In::Deleted | In::Old => !accept,
                        In::Comment => false,
                    };
                    if shown {
                        out.push_str(delimiter(token));
                        // Markup after the change ended: a note now is on
                        // this point, not on the change.
                        folded_at = None;
                    }
                    None
                }
                Token::InsertStart | Token::DeleteStart => {
                    folded_at = Some(out.len());
                    match token {
                        Token::InsertStart => Some(In::Inserted),
                        _ => Some(In::Deleted),
                    }
                }
                Token::SubstituteStart => {
                    folded_at = None;
                    Some(In::Old)
                }
                Token::CommentStart => Some(In::Comment),
                // A highlight shows its text as the text around it does.
                Token::HighlightStart => Some(state),
                Token::SubstituteSeparator => {
                    state = In::New;
                    None
                }
                Token::HighlightEnd
                | Token::InsertEnd
                | Token::DeleteEnd
                | Token::SubstituteEnd => {
                    state = outer.pop().unwrap_or(In::Text);
                    None
                }
                // A comment paragraph break: comments resolve to nothing,
                // so the break is dropped with the comment's text.
                Token::CommentBreak => None,
                Token::CommentEnd => {
                    state = outer.pop().unwrap_or(In::Text);
                    // The note folds the change it follows, naming the one
                    // it sits inside after ` on `: its text goes, either
                    // way it resolves — rejecting the outer removes what it
                    // holds, accepting the inner removes the text.
                    if state == In::Text
                        && let Some(at) = folded_at.take()
                        && names_outer(&comment)
                    {
                        out.truncate(at);
                        line_start = out.rfind('\n').map_or(0, |at| at + 1);
                        visible_text = false;
                        dropped_line = false;
                    }
                    comment.clear();
                    None
                }
            };
            if let Some(opened) = opened {
                outer.push(state);
                state = opened;
            }
        }
    });
    if remove_deleted_lines && removed_text && !visible_text {
        out.truncate(line_start);
    }
    out
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn pieces(encoded: &str) -> Vec<String> {
        let mut out = Vec::new();
        Pieces::default().feed(encoded, |piece| {
            out.push(match piece {
                Piece::Text(text) => text.to_string(),
                Piece::Token(token) => format!("<{token:?}>"),
            });
        });
        out
    }

    #[test]
    fn a_field_crosses_paragraphs_only_from_a_paragraph_start() {
        for (line, opens) in [
            ("{TOC}", true),
            ("  {TOC}", true),
            ("## {TOC}", true),
            ("- {TOC}", true),
            ("> 1. {TOC}", true),
            ("12) {TOC}", true),
            ("See {TOC}", false),
            ("#x {TOC}", false),
            ("1.{TOC}", false),
        ] {
            let source = format!("<!-- p0 -->\n{line}a\n\nb{{/}}\n");
            let at = source.find("{TOC").unwrap();
            assert_eq!(opens_paragraph(&source, at), opens, "{line}");
            let ends = field_ends(&source, &Literal::new(&source));
            assert_eq!(ends.contains_key(&at), opens, "{line}");
        }
        // A field inside the spanning one closes first; a fence ends both.
        let source = "{TOC}a {PAGE}1{/}\n\nb{/}";
        let ends = field_ends(source, &Literal::new(source));
        assert_eq!(ends.get(&0), Some(&(source.len() - 3)));
        let source = "{TOC}a\n\n```\nx\n```\n\nb{/}";
        assert!(field_ends(source, &Literal::new(source)).is_empty());
    }

    #[test]
    fn every_span_kind_is_encoded() {
        assert_eq!(
            pieces(&encode("a{++b++}c{--d--}e{~~f~>g~~}h{==i==}{>>j<<}")),
            [
                "a",
                "<InsertStart>",
                "b",
                "<InsertEnd>",
                "c",
                "<DeleteStart>",
                "d",
                "<DeleteEnd>",
                "e",
                "<SubstituteStart>",
                "f",
                "<SubstituteSeparator>",
                "g",
                "<SubstituteEnd>",
                "h",
                "<HighlightStart>",
                "i",
                "<HighlightEnd>",
                "<CommentStart>",
                "j",
                "<CommentEnd>",
            ]
        );
    }

    #[test]
    fn incomplete_or_escaped_delimiters_stay_text() {
        for source in [
            "a {++ b",
            "a ++} b",
            "{~~no separator~~}",
            r"\{++not a change++}",
            "{--}",
        ] {
            assert_eq!(encode(source), source, "{source}");
        }
        // An even number of backslashes escapes the backslash, not the brace.
        assert_ne!(encode(r"\\{++x++}"), r"\\{++x++}");
    }

    #[test]
    fn an_escaped_closer_or_separator_is_text_inside_the_span() {
        assert_eq!(
            pieces(&encode(r"{++a \++} b++}")),
            ["<InsertStart>", r"a \++} b", "<InsertEnd>"]
        );
        assert_eq!(
            pieces(&encode(r"{~~a\~>b~>c~~}")),
            [
                "<SubstituteStart>",
                r"a\~>b",
                "<SubstituteSeparator>",
                "c",
                "<SubstituteEnd>"
            ]
        );
        assert_eq!(
            pieces(&encode(r"{>>x \<<} y<<}")),
            ["<CommentStart>", r"x \<<} y", "<CommentEnd>"]
        );
        // Only an escaped closer: the span never closes.
        assert_eq!(encode(r"{++a \++}"), r"{++a \++}");
        assert_eq!(encode(r"{~~a\~>b~~}"), r"{~~a\~>b~~}");
        // An escaped backslash leaves the closer a closer.
        assert_eq!(
            pieces(&encode(r"{++a\\++} b++}")),
            ["<InsertStart>", r"a\\", "<InsertEnd>", " b++}"]
        );
        assert_eq!(resolve(r"{++a \++} b++}", true), r"a \++} b");
        assert_eq!(resolve(r"{++a \++} b++}", false), "");
    }

    #[test]
    fn indented_code_is_verbatim_not_a_fence() {
        // CommonMark reads a line indented four or more spaces as indented
        // code, so its text is verbatim (#518): a ``` line there is text,
        // no fence opens, and CriticMarkup and its escapes open nothing.
        // The reader never writes indented fences, so nothing is lost.
        for hand in [
            "    ```\n    a{\\++b\n    ```\n",
            "     ```\n     x++\\}y\n     ```\n",
            "    a{++b++}c\n",
            "\tcode {++x++} too\n",
        ] {
            assert_eq!(encode(hand), hand, "{hand:?}");
        }
    }

    #[test]
    fn an_escape_from_a_code_block_unwinds_inside_its_fence_only() {
        // The reader breaks a code block's delimiter shapes by making
        // their backslash run odd; Markdown itself reads no escape inside
        // a fence, so the unwinding happens there, and only there:
        // outside one the escapes are Markdown's own. Every other
        // backslash is left as written, so code a human wrote reaches
        // Word byte-identical.
        for (document, written) in [
            ("fenced {++x++} code", r"fenced {\++x++\} code"),
            (r"{\++x++}", r"{\\\++x++\}"),
            ("a~>b", r"a~\>b"),
            (r"a\~>b", r"a\~\>b"),
            (r"back\slash", r"back\slash"),
            (r"C:\path\x", r"C:\path\x"),
            (
                r"$x_{--}$ and ![a {++b++}](u\v)",
                r"$x_{\--}$ and ![a {\++b++\}](u\v)",
            ),
        ] {
            let fence = format!("```\n{written}\n```\n");
            assert_eq!(encode(&fence), format!("```\n{document}\n```\n"));
            let text = format!("a {written} b");
            assert_eq!(encode(&text), text, "outside a fence it is Markdown's");
        }
        // Backslashes a human wrote in a fence are not the reader's
        // escape: `C:\\path`, a regex `\\d+`, LaTeX `\\` and `a\\b`
        // stay as written. A delimiter behind an even run stays as
        // written too and opens nothing; it cannot come from the reader.
        for hand in [
            r"C:\\path\\x",
            r"\\d+",
            r"a\\b",
            r"\\\\",
            r"even {\\++ odd and \{++ text",
            r"close ++\\} and sep ~\\>",
        ] {
            let fence = format!("```\n{hand}\n```\n");
            assert_eq!(encode(&fence), fence, "{hand}");
        }
        // The changes the reader writes in a code block still open there,
        // and the text inside them unwinds with the rest of the fence.
        assert_eq!(
            pieces(&encode("```\nkeep {++new++} old\n```\n")),
            [
                "```\nkeep ",
                "<InsertStart>",
                "new",
                "<InsertEnd>",
                " old\n```\n"
            ]
        );
        // A change beside the code's own backslash keeps its delimiters:
        // inside a fence they are never Markdown-escaped.
        assert_eq!(
            pieces(&encode("```\nold\\{++new++}\n```\n")),
            [
                "```\nold\\",
                "<InsertStart>",
                "new",
                "<InsertEnd>",
                "\n```\n"
            ]
        );
        assert_eq!(
            pieces(&encode("```\n{++new\\++}\n```\n")),
            ["```\n", "<InsertStart>", "new\\", "<InsertEnd>", "\n```\n"]
        );
        assert_eq!(
            pieces(&encode("```\n{~~old\\~>new~~}\n```\n")),
            [
                "```\n",
                "<SubstituteStart>",
                "old\\",
                "<SubstituteSeparator>",
                "new",
                "<SubstituteEnd>",
                "\n```\n"
            ]
        );
        assert_eq!(
            pieces(&encode("```\n{~~a~\\>c~>b~~}\n```\n")),
            [
                "```\n",
                "<SubstituteStart>",
                "a~>c",
                "<SubstituteSeparator>",
                "b",
                "<SubstituteEnd>",
                "\n```\n"
            ]
        );
        // The front matter is no code block: nothing unwinds there.
        assert_eq!(
            encode("---\ntitle: a~\\>b\n---\n"),
            "---\ntitle: a~\\>b\n---\n"
        );
    }

    #[test]
    fn unclosed_openers_encode_in_linear_time() {
        // Every opener once scanned the rest of the text for its closer.
        let source = "{++ {-- {~~ {== {>> ~> ".repeat(40_000);
        let started = std::time::Instant::now();
        assert_eq!(encode(&source), source);
        let elapsed = started.elapsed();
        assert!(elapsed.as_secs() < 2, "took {elapsed:?}");
    }

    #[test]
    fn unclosed_links_fields_and_highlights_encode_in_linear_time() {
        // Each `](`, `{CODE}` and `==` once scanned the rest of its line or
        // paragraph for its close.
        for source in [
            "[a](b".repeat(6_000),
            "{PAGE}a ".repeat(6_000),
            format!("{}{{/}}", "{PAGE}a ".repeat(6_000)),
            "==a ".repeat(6_000),
        ] {
            let started = std::time::Instant::now();
            encode(&source);
            let elapsed = started.elapsed();
            assert!(elapsed.as_secs() < 2, "{source:.12}: took {elapsed:?}");
        }
    }

    #[test]
    fn spans_cross_paragraphs_and_changes_do_not_nest() {
        assert_eq!(
            pieces(&encode("A{++\n\nB {--c--}++}")),
            ["A", "<InsertStart>", "\n\nB {--c--}", "<InsertEnd>"]
        );
    }

    #[test]
    fn a_highlight_holds_changes_and_comments() {
        assert_eq!(
            pieces(&encode("{==a {++b++}{>>Ana<<} {~~c~>d~~}==}{>>note<<}")),
            [
                "<HighlightStart>",
                "a ",
                "<InsertStart>",
                "b",
                "<InsertEnd>",
                "<CommentStart>",
                "Ana",
                "<CommentEnd>",
                " ",
                "<SubstituteStart>",
                "c",
                "<SubstituteSeparator>",
                "d",
                "<SubstituteEnd>",
                "<HighlightEnd>",
                "<CommentStart>",
                "note",
                "<CommentEnd>",
            ]
        );
        // Highlights themselves do not nest, and an incomplete span inside
        // one stays text.
        assert_eq!(
            pieces(&encode("{==a {==b==} {++c==}")),
            ["<HighlightStart>", "a {==b", "<HighlightEnd>", " {++c==}"]
        );
        assert_eq!(resolve("{==a{++b++}{--c--}==}{>>n<<}", true), "ab");
        assert_eq!(resolve("{==a{++b++}{--c--}==}{>>n<<}", false), "ac");
    }

    #[test]
    fn a_mark_run_closes_past_a_wide_first_character() {
        // The closer search starts past the first inner character, and `é`
        // is two bytes and `→` three: an `==` run starting with either still
        // finds its `==`.
        assert_eq!(
            pieces(&encode("==é== and ==→x==")),
            [
                "<MarkStart>",
                "é",
                "<MarkEnd>",
                " and ",
                "<MarkStart>",
                "→x",
                "<MarkEnd>"
            ]
        );
    }

    #[test]
    fn a_change_holds_highlights_and_comments_one_level_deep() {
        assert_eq!(
            pieces(&encode("{~~a~>{==b==}{>>n<<}~~}")),
            [
                "<SubstituteStart>",
                "a",
                "<SubstituteSeparator>",
                "<HighlightStart>",
                "b",
                "<HighlightEnd>",
                "<CommentStart>",
                "n",
                "<CommentEnd>",
                "<SubstituteEnd>",
            ]
        );
        // No change in a change, nor in a highlight in a change, nor any
        // span in a comment.
        assert_eq!(
            pieces(&encode("{++a {--b--}++}")),
            ["<InsertStart>", "a {--b--}", "<InsertEnd>"]
        );
        assert_eq!(
            pieces(&encode("{++{==a {--b--}==}++}")),
            [
                "<InsertStart>",
                "<HighlightStart>",
                "a {--b--}",
                "<HighlightEnd>",
                "<InsertEnd>"
            ]
        );
        assert_eq!(
            pieces(&encode("{>>a {++b++}<<}")),
            ["<CommentStart>", "a {++b++}", "<CommentEnd>"]
        );
        // A comment inside a change does not end it.
        let text = "x{++a{>>n<<}b++}{~~c~>{==d==}{>>m<<}e~~}";
        assert_eq!(resolve(text, true), "xabde");
        assert_eq!(resolve(text, false), "xc");
    }

    /// Every payload that, inside a comment, would shorten the enclosing
    /// span or steal its separator, with every change kind and every
    /// substitution half. The comment is opaque to the boundary lookup;
    /// the change and the comment both survive.
    #[test]
    fn a_complete_comment_inside_a_change_is_opaque_to_its_boundaries() {
        // The exact ticket example: a `~>` inside the comment must not
        // become the substitution's separator.
        let text = "{~~a{>>x ~> y<<} b~>c~~}";
        assert_eq!(
            pieces(&encode(text)),
            [
                "<SubstituteStart>",
                "a",
                "<CommentStart>",
                "x ~> y",
                "<CommentEnd>",
                " b",
                "<SubstituteSeparator>",
                "c",
                "<SubstituteEnd>",
            ]
        );
        assert_eq!(resolve(text, true), "c");
        assert_eq!(resolve(text, false), "a b");

        for payload in ["~>", "++}", "--}", "~~}", "==}"] {
            // Insertion with the payload inside a comment.
            let text = format!("{{++a{{>>{payload}<<}}b++}}");
            assert_eq!(
                pieces(&encode(&text)),
                [
                    "<InsertStart>",
                    "a",
                    "<CommentStart>",
                    payload,
                    "<CommentEnd>",
                    "b",
                    "<InsertEnd>"
                ],
                "insertion {payload:?}",
            );
            assert_eq!(resolve(&text, true), "ab", "insertion {payload:?} accept");
            assert_eq!(resolve(&text, false), "", "insertion {payload:?} reject");

            // Deletion with the payload inside a comment.
            let text = format!("{{--a{{>>{payload}<<}}b--}}");
            assert_eq!(
                pieces(&encode(&text)),
                [
                    "<DeleteStart>",
                    "a",
                    "<CommentStart>",
                    payload,
                    "<CommentEnd>",
                    "b",
                    "<DeleteEnd>"
                ],
                "deletion {payload:?}",
            );
            assert_eq!(resolve(&text, true), "", "deletion {payload:?} accept");
            assert_eq!(resolve(&text, false), "ab", "deletion {payload:?} reject");

            // Substitution with the payload inside a comment in the old
            // half; the outer `~>` is the separator.
            let text = format!("{{~~a{{>>{payload}<<}} b~>c~~}}");
            assert_eq!(
                pieces(&encode(&text)),
                [
                    "<SubstituteStart>",
                    "a",
                    "<CommentStart>",
                    payload,
                    "<CommentEnd>",
                    " b",
                    "<SubstituteSeparator>",
                    "c",
                    "<SubstituteEnd>",
                ],
                "substitution-old {payload:?}",
            );
            assert_eq!(resolve(&text, true), "c", "sub-old {payload:?} accept");
            assert_eq!(resolve(&text, false), "a b", "sub-old {payload:?} reject");

            // Substitution with the payload inside a comment in the new
            // half; the `~>` is before the comment.
            let text = format!("{{~~a~>b{{>>{payload}<<}} c~~}}");
            assert_eq!(
                pieces(&encode(&text)),
                [
                    "<SubstituteStart>",
                    "a",
                    "<SubstituteSeparator>",
                    "b",
                    "<CommentStart>",
                    payload,
                    "<CommentEnd>",
                    " c",
                    "<SubstituteEnd>",
                ],
                "substitution-new {payload:?}",
            );
            assert_eq!(resolve(&text, true), "b c", "sub-new {payload:?} accept");
            assert_eq!(resolve(&text, false), "a", "sub-new {payload:?} reject");

            // Highlight with the payload inside a comment.
            let text = format!("{{==a{{>>{payload}<<}}b==}}");
            assert_eq!(
                pieces(&encode(&text)),
                [
                    "<HighlightStart>",
                    "a",
                    "<CommentStart>",
                    payload,
                    "<CommentEnd>",
                    "b",
                    "<HighlightEnd>",
                ],
                "highlight {payload:?}",
            );
            assert_eq!(resolve(&text, true), "ab", "highlight {payload:?} accept");
            assert_eq!(resolve(&text, false), "ab", "highlight {payload:?} reject");
        }
    }

    /// Incomplete comments and comment boundaries that fall outside the
    /// enclosing span must not hide its closer or separator.
    #[test]
    fn incomplete_comments_and_escaped_comment_openers_do_not_break_changes() {
        // An incomplete comment inside an insertion: the insertion still
        // closes at its `++}` and the comment opener stays text.
        let text = "{++a{>>b++} c";
        assert_eq!(
            pieces(&encode(text)),
            ["<InsertStart>", "a{>>b", "<InsertEnd>", " c"]
        );
        assert_eq!(resolve(text, true), "a{>>b c");
        assert_eq!(resolve(text, false), " c");

        // An escaped `{>>` opener does not start a comment: the backslash
        // breaks the escape, so the whole thing stays text.
        let text = r"\{++a{>>b++} c++}";
        assert_eq!(encode(text), text);
        assert_eq!(resolve(text, true), text);
        assert_eq!(resolve(text, false), text);

        // A `{>>` inside a code span does not start a comment: code
        // spans are verbatim, the opener is text.
        let text = "before `{>>a++}` text after\n";
        assert_eq!(encode(text), text);
        assert_eq!(resolve(text, true), text);
        assert_eq!(resolve(text, false), text);
    }

    /// Many failed change openers before many complete comments must not
    /// rescann every comment: the lookup stays linear in the input size.
    #[test]
    fn many_failed_openers_with_comments_stay_linear() {
        // Each `{~~x` is a failed substitution opener (no separator, no
        // closer); each `{>>` opens a complete comment containing both
        // `~~}` and `~>`.
        let failed = "{~~x".repeat(2_000);
        let comment = "{>>payload ~~} ~> body<<} ".repeat(2_000);
        let source = format!("{failed}{comment}");
        let started = std::time::Instant::now();
        let output = encode(&source);
        let elapsed = started.elapsed();
        assert!(
            elapsed.as_secs() < 2,
            "took {elapsed:?} for {} bytes",
            source.len()
        );
        // Sanity: the failed openers leave the markers as text, the
        // comments are recognised as complete.
        assert!(output.contains("~>"));
        assert!(output.contains("payload"));
    }

    /// Many highlight openers before many comments holding their kind's
    /// closer must not re-skip every comment from each opener: the
    /// pairing pass that finds complete spans stays linear too.
    #[test]
    fn highlight_openers_before_comment_held_closers_stay_linear() {
        // Every `{==` fails (each `==}` sits inside a comment), and each
        // comment is complete.
        let openers = "{==".repeat(20_000);
        let comments = "{>>x==}y<<} ".repeat(20_000);
        let source = format!("{openers}{comments}");
        let started = std::time::Instant::now();
        let output = encode(&source);
        let elapsed = started.elapsed();
        assert!(
            elapsed.as_secs() < 2,
            "took {elapsed:?} for {} bytes",
            source.len()
        );
        // The openers stay text and the comments are recognised.
        assert!(output.contains("==}"));
        assert!(output.contains('y'));
    }

    #[test]
    fn a_footnote_label_goes_before_a_change_that_opens_its_line() {
        assert_eq!(
            pieces(&encode(
                "x\n{++[^2]: Added.++}\n{--[^a]:\tGone--} {++[^b]: y++}"
            )),
            [
                "x\n[^2]: ",
                "<InsertStart>",
                "Added.",
                "<InsertEnd>",
                "\n[^a]:\t",
                "<DeleteStart>",
                "Gone",
                "<DeleteEnd>",
                " ",
                "<InsertStart>",
                "[^b]: y",
                "<InsertEnd>",
            ]
        );
        for text in [
            "{++[^]: a++}",
            "{++[^a b]: c++}",
            "{++[^a] b++}",
            "{>>[^a]: b<<}",
        ] {
            assert!(!pieces(&encode(text))[0].starts_with("[^"), "{text}");
        }
        // Resolving drops the whole definition with its change.
        assert_eq!(resolve("a\n{++[^2]: b++}\n", false), "a\n\n");
    }

    #[test]
    fn stand_in_characters_in_the_source_come_back_unchanged() {
        let source = "odd \u{2E0E} and \u{F0000} {++\u{2E0F}++}";
        let encoded = encode(source);
        assert_eq!(
            pieces(&encoded),
            [
                "odd ",
                "\u{2E0E} and ",
                "\u{F0000} ",
                "<InsertStart>",
                "\u{2E0F}",
                "<InsertEnd>"
            ]
        );
        assert_eq!(
            decode(&encode("plain \u{2E0E} text")),
            "plain \u{2E0E} text"
        );
    }

    #[test]
    fn resolving_keeps_one_side() {
        let text = "a{++b++}c{--d--}e{~~f~>g~~}h{==i==}{>>j<<}k \\{++l++}";
        assert_eq!(resolve(text, true), "abcegh".to_string() + "ik \\{++l++}");
        assert_eq!(resolve(text, false), "acdefh".to_string() + "ik \\{++l++}");
    }

    #[test]
    fn resolving_keeps_a_fence_s_escapes() {
        // The resolved Markdown still names the document its code blocks
        // hold, so the reader's fence escapes stay: converting the output
        // back cannot invent a change from them.
        let fence = "```\nfenced {\\++x++\\} code\n```\n";
        assert_eq!(resolve(fence, true), fence);
        assert_eq!(resolve(fence, false), fence);
    }

    #[test]
    fn accepting_a_whole_deleted_paragraph_collapses_its_blank_separator() {
        // A paragraph deleted between blank-line-separated paragraphs must
        // not leave a phantom blank line once the deletion is accepted (the
        // dropped line's trailing separator is collapsed, so `diff_text_view`
        // with accepted changes reports no spurious blank-line change).
        assert_eq!(
            accept_clauses("Para A\n\n{--Para B--}\n\nPara C\n"),
            "Para A\n\nPara C\n"
        );
        // Two consecutive deleted paragraphs collapse to a single separator.
        assert_eq!(
            accept_clauses("Para A\n\n{--Para B--}\n\n{--Para D--}\n\nPara C\n"),
            "Para A\n\nPara C\n"
        );
        // A surviving paragraph between kept ones is untouched.
        assert_eq!(
            accept_clauses("Para A\n\nPara B\n\nPara C\n"),
            "Para A\n\nPara B\n\nPara C\n"
        );
    }

    #[test]
    fn an_escape_split_across_two_texts_is_carried() {
        let encoded = encode("x\u{2E0E}y");
        let split = encoded.char_indices().nth(2).map(|(i, _)| i).unwrap();
        let mut splitter = Pieces::default();
        let mut out = Vec::new();
        for part in [&encoded[..split], &encoded[split..]] {
            splitter.feed(part, |piece| {
                if let Piece::Text(text) = piece {
                    out.push(text.to_string());
                }
            });
        }
        assert_eq!(out.concat(), "x\u{2E0E}y");
    }

    /// A comment after another comment is still opaque: an earlier comment
    /// must not hide the later one's delimiters from the boundary lookup.
    #[test]
    fn every_comment_hides_its_own_delimiters() {
        let text = "x {>>one<<} {~~a{>>y ~> z<<}b~>c~~} d{>>two<<}e";
        assert_eq!(
            pieces(&encode(text)),
            [
                "x ",
                "<CommentStart>",
                "one",
                "<CommentEnd>",
                " ",
                "<SubstituteStart>",
                "a",
                "<CommentStart>",
                "y ~> z",
                "<CommentEnd>",
                "b",
                "<SubstituteSeparator>",
                "c",
                "<SubstituteEnd>",
                " d",
                "<CommentStart>",
                "two",
                "<CommentEnd>",
                "e",
            ]
        );
        assert_eq!(resolve(text, true), "x  c de");
        assert_eq!(resolve(text, false), "x  ab de");
        // A comment that crosses a blank line and holds the kind's
        // closer still nests whole: the change ends after it.
        let text = "{++a{>>x++}y\n\n<<}z++}";
        assert_eq!(
            pieces(&encode(text)),
            [
                "<InsertStart>",
                "a",
                "<CommentStart>",
                "x++}y",
                "<CommentBreak>",
                "<CommentEnd>",
                "z",
                "<InsertEnd>",
            ]
        );
        assert_eq!(resolve(text, true), "az");
        assert_eq!(resolve(text, false), "");
    }

    /// A complete nested highlight is opaque like a comment: a `~>` or a
    /// closer inside it does not split or shorten the change around it,
    /// and a `==}` inside its own nested comment does not close it.
    #[test]
    fn a_complete_highlight_inside_a_change_is_opaque_to_its_boundaries() {
        let text = "{~~a{==x ~> y==} b~>c~~}";
        assert_eq!(
            pieces(&encode(text)),
            [
                "<SubstituteStart>",
                "a",
                "<HighlightStart>",
                "x ~> y",
                "<HighlightEnd>",
                " b",
                "<SubstituteSeparator>",
                "c",
                "<SubstituteEnd>",
            ]
        );
        assert_eq!(resolve(text, true), "c");
        assert_eq!(resolve(text, false), "ax ~> y b");

        let text = "{++a{==b ++}c==}d++}";
        assert_eq!(
            pieces(&encode(text)),
            [
                "<InsertStart>",
                "a",
                "<HighlightStart>",
                "b ++}c",
                "<HighlightEnd>",
                "d",
                "<InsertEnd>",
            ]
        );
        assert_eq!(resolve(text, true), "ab ++}cd");
        assert_eq!(resolve(text, false), "");

        // A comment inside the highlight makes its `==}` text.
        let text = "{==a{>>x==}y<<}b==}{>>done<<}";
        assert_eq!(
            pieces(&encode(text)),
            [
                "<HighlightStart>",
                "a",
                "<CommentStart>",
                "x==}y",
                "<CommentEnd>",
                "b",
                "<HighlightEnd>",
                "<CommentStart>",
                "done",
                "<CommentEnd>",
            ]
        );
        // An incomplete highlight keeps its delimiters usable.
        let text = "{++a{==x++}y++}";
        assert_eq!(
            pieces(&encode(text)),
            ["<InsertStart>", "a{==x", "<InsertEnd>", "y++}"]
        );
    }

    /// A blank line inside `{>>…<<}` becomes a comment paragraph break,
    /// not a paragraph break in the text the comment is anchored on.
    #[test]
    fn a_blank_line_inside_a_comment_is_a_comment_paragraph_break() {
        let text = "a{>>first\n\nsecond<<}b";
        assert_eq!(
            pieces(&encode(text)),
            [
                "a",
                "<CommentStart>",
                "first",
                "<CommentBreak>",
                "second",
                "<CommentEnd>",
                "b",
            ]
        );
        // One line break is text; blank-line runs collapse to one break,
        // whitespace-only lines and the next line's indent with them.
        assert_eq!(
            pieces(&encode("a{>>first\nsecond<<}b")),
            ["a", "<CommentStart>", "first\nsecond", "<CommentEnd>", "b"]
        );
        assert_eq!(
            pieces(&encode("a{>>first\n\n\n  \n\nsecond<<}b")),
            [
                "a",
                "<CommentStart>",
                "first",
                "<CommentBreak>",
                "second",
                "<CommentEnd>",
                "b",
            ]
        );
        assert_eq!(decode(&encode("a{>>first\n\nsecond<<}b")), "afirst secondb");
    }

    /// Malformed CriticMarkup is text, and says so: an opener without its
    /// closer or separator, and a closer nothing opened.
    #[test]
    fn malformed_critic_markup_is_text_with_a_warning() {
        // Delimiters inside a comment are text of the comment: the
        // comment still encodes, and nothing warns.
        let (encoded, warnings) = encode_checked("{>>x ++} ~> y<<}");
        assert_eq!(
            encoded,
            format!(
                "{}{}{}",
                stand_in(Token::CommentStart),
                "x ++} ~> y",
                stand_in(Token::CommentEnd)
            )
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        // Inside a code span nothing opens: it stays literal, and the
        // comment around a `++}` warns nothing.
        let source = "`{++x ++}` {>>y ++}<<}";
        let (encoded, warnings) = encode_checked(source);
        assert!(encoded.starts_with("`{++x ++}` "), "{encoded}");
        assert!(warnings.is_empty(), "{warnings:?}");

        let cases = [
            ("{++a", "'{++' without its '++}'"),
            ("a ++} b", "'++}' with nothing open"),
            ("{~~a~~}", "'{~~' without its '~>' separator"),
            ("{>>a", "'{>>' without its '<<}'"),
        ];
        for (source, warning) in cases {
            let (encoded, warnings) = encode_checked(source);
            assert_eq!(encoded, source, "{source}");
            assert!(
                warnings.iter().any(|w| w.contains(warning)),
                "{source}: {warnings:?} lacks {warning:?}"
            );
        }
        // A wrong-kind closer is both: the opener never closed, and the
        // closer is a stray.
        let (encoded, warnings) = encode_checked("{++a--}");
        assert_eq!(encoded, "{++a--}");
        assert!(warnings.iter().any(|w| w.contains("'{++'")), "{warnings:?}");
        assert!(warnings.iter().any(|w| w.contains("'--}'")), "{warnings:?}");
    }

    /// A numeric character reference that names a stand-in character or
    /// the escape is literal text: the Markdown parser decodes references
    /// only after CriticMarkup was read, so the decoded character must
    /// not arrive as an unescaped delimiter.
    #[test]
    fn character_references_naming_stand_ins_stay_literal() {
        for (text, guarded) in [
            ("a &#x2E0E;x&#x2E0F; b", 2),
            ("a &#11790;x&#11791; b", 2),
            ("a &#X2E0E;x b", 1),
            ("{>>a &#x2E19; b<<}", 1),
            ("a &#xF0000; b", 1),
        ] {
            let encoded = encode(text);
            assert_eq!(
                encoded.chars().filter(|&c| c == ESCAPE).count(),
                guarded,
                "{text}: {encoded:?}"
            );
            // What a reader pieces together is the text as written.
            assert_eq!(restore(&encoded), text, "{text}");
        }
        // References to other characters, malformed references,
        // references behind a backslash and references in a code span
        // keep their spelling, unguarded.
        for text in [
            "a &#65; b",
            "a &#x2E0 b",
            "a &#x110000; b",
            r"a \&#x2E0E; b",
            "`&#x2E0E;`",
            "```
&#x2E0E;
```
",
        ] {
            assert_eq!(encode(text), text, "{text}");
        }
    }

    /// A closer inside a complete highlight is the highlight's text, as a
    /// closer inside a comment is the comment's: neither warns as a
    /// stray.
    #[test]
    fn a_closer_inside_a_complete_highlight_is_the_highlight_s_text() {
        for text in [
            "{==b++}c==}",
            "{++a{==b++}c==}d++}",
            "{~~a{==b~~}c==}d~>e~~}",
            "{==a{>>x==}y<<}b==}",
        ] {
            let (encoded, warnings) = encode_checked(text);
            assert!(warnings.is_empty(), "{text}: {warnings:?} in {encoded:?}");
        }
    }
}
