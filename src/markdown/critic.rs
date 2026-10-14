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
//! source is kept apart by an escape character and comes back unchanged.

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
const STAND_INS: [(Token, char); 16] = [
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

/// The source with the delimiters of every complete CriticMarkup span
/// replaced by stand-ins. A delimiter without its partner, or behind a
/// backslash, stays text.
///
/// The specification has spans never nest. Word, though, can track a change
/// inside a commented range and comment inside a change, and a document
/// read from Word says so, so here spans nest one level: a highlight can
/// hold changes and comments (`{==a {++b++}==}{>>note<<}`), a change can
/// hold highlights and comments (`{~~a~>{==b==}{>>note<<}~~}`), and anything
/// deeper, or inside a comment, is text.
///
/// A change or highlight that opens a line with a footnote label
/// (`{++[^2]: Added later.++}`, a note inserted with its reference) is read
/// as changing the note: the label goes before the delimiter, so the
/// definition stays a definition.
pub(crate) fn encode(source: &str) -> String {
    encode_spans(source, ALL, true)
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

/// Where every unescaped closer and separator sits in the source, found once
/// so that an opener without its closer does not scan the rest of the text,
/// and the parts Markdown reads literally.
struct Closers<'a> {
    patterns: [&'a str; 6],
    positions: [Vec<usize>; 6],
    literal: Literal,
    /// Every `==` that can close a highlight.
    marks: Vec<usize>,
    /// Every blank line (`\n\n`), where a paragraph ends.
    blanks: Vec<usize>,
    /// The `{/}` that closes each body field, by the field's `{`.
    fields: std::collections::HashMap<usize, usize>,
}

impl<'a> Closers<'a> {
    fn new(source: &str) -> Self {
        let literal = Literal::new(source);
        let patterns = ["++}", "--}", "~~}", "==}", "<<}", "~>"];
        let positions = patterns.map(|pattern| {
            source
                .match_indices(pattern)
                .map(|(at, _)| at)
                .filter(|&at| !escaped(source, at) && literal.verbatim_end(at).is_none())
                .collect()
        });
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
            literal,
            marks,
            blanks,
            fields,
        }
    }

    /// The first `pattern` that lies whole in `from..until`.
    fn find(&self, pattern: &str, from: usize, until: usize) -> Option<usize> {
        let slot = self.patterns.iter().position(|p| *p == pattern)?;
        let positions = &self.positions[slot];
        let at = *positions.get(positions.partition_point(|&at| at < from))?;
        (at + pattern.len() <= until).then_some(at)
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
}

impl Literal {
    fn new(source: &str) -> Self {
        let mut literal = Self {
            verbatim: Vec::new(),
            fences: Vec::new(),
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
                        if run(b) >= width && line[indent + run(b)..].trim().is_empty() {
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
                            if width >= 3 && !(b == b'`' && info.contains('`')) {
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

/// [`encode`], opening only the spans `allowed` names, and moving footnote
/// labels out of spans only when `labels`.
fn encode_spans(source: &str, allowed: Allowed, labels: bool) -> String {
    let mut out = String::with_capacity(source.len());
    encode_range(
        source,
        &Closers::new(source),
        0..source.len(),
        allowed,
        labels,
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
        let span = SPANS
            .iter()
            .find(|(open, ..)| rest.starts_with(open))
            .filter(|span| allowed & kind_of(span.3) != 0)
            .filter(|_| !escaped(source, at))
            .and_then(|&(open, close, separator, start, end)| {
                let from = at + open.len();
                let to = closers.find(close, from, range.end)?;
                let split = match separator {
                    Some(separator) => Some((closers.find(separator, from, to)?, separator.len())),
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
                        encode_range(source, closers, body.start..mid, nested, false, out);
                        out.push(stand_in(Token::SubstituteSeparator));
                        encode_range(source, closers, mid + width..body.end, nested, false, out);
                    }
                    None => encode_range(source, closers, body, nested, false, out),
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
                    encode_range(source, closers, code.end + 1..end, allowed, false, out);
                    out.push(stand_in(Token::FieldEnd));
                    at = end + FIELD_END.len();
                    continue;
                }
                if here & MARK != 0
                    && let Some(close) = mark_close(source, closers, at, range.end)
                {
                    out.push(stand_in(Token::MarkStart));
                    encode_range(source, closers, at + 2..close, allowed, false, out);
                    out.push(stand_in(Token::MarkEnd));
                    at = close + 2;
                    continue;
                }
                let c = rest.chars().next().unwrap_or_default();
                push_escaped(out, &rest[..c.len_utf8()]);
                at += c.len_utf8();
            }
        }
    }
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
/// it sits, past any field nested in its result, in the same paragraph
/// (before `until` and any blank line). Without that `{/}` it is text.
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
/// first, a blank line or a fenced block leaves every open field unclosed,
/// and code spans, link destinations and autolinks are passed over. A `{`
/// inside a field's code (`{QUOTE "{PAGE}"}`) opens nothing.
fn field_ends(source: &str, literal: &Literal) -> std::collections::HashMap<usize, usize> {
    let mut ends = std::collections::HashMap::new();
    let mut open = Vec::new();
    let mut i = 0;
    while i < source.len() {
        if let Some(skip) = literal.verbatim_end(i) {
            i = skip;
        } else if literal.in_fence(i) || source[i..].starts_with("\n\n") {
            open.clear();
            i += source[i..].chars().next().map_or(1, char::len_utf8);
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

/// The source of text that holds no stand-ins, as it was.
pub(crate) fn decode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pieces = Pieces::default();
    pieces.feed(text, |piece| {
        if let Piece::Text(text) = piece {
            out.push_str(text);
        }
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
    Pieces::default().feed(&encode_spans(markdown, ALL, false), |piece| match piece {
        Piece::Text(text) => {
            let shown = match state {
                In::Text => true,
                In::Inserted | In::New => accept,
                In::Deleted | In::Old => !accept,
                In::Comment => false,
            };
            if shown {
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
                    }
                    None
                }
                Token::InsertStart => Some(In::Inserted),
                Token::DeleteStart => Some(In::Deleted),
                Token::SubstituteStart => Some(In::Old),
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
                | Token::SubstituteEnd
                | Token::CommentEnd => {
                    state = outer.pop().unwrap_or(In::Text);
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
}
