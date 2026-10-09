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
}

/// Every delimiter with its stand-in (Supplemental Punctuation, all Po).
const STAND_INS: [(Token, char); 11] = [
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
const CHANGES: Allowed = INSERT | DELETE | SUBSTITUTE;
const ALL: Allowed = CHANGES | HIGHLIGHT | COMMENT;

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
/// so that an opener without its closer does not scan the rest of the text.
struct Closers<'a> {
    patterns: [&'a str; 6],
    positions: [Vec<usize>; 6],
}

impl<'a> Closers<'a> {
    fn new(source: &str) -> Self {
        let patterns = ["++}", "--}", "~~}", "==}", "<<}", "~>"];
        let positions = patterns.map(|pattern| {
            source
                .match_indices(pattern)
                .map(|(at, _)| at)
                .filter(|&at| !escaped(source, at))
                .collect()
        });
        Self {
            patterns,
            positions,
        }
    }

    /// The first `pattern` that lies whole in `from..until`.
    fn find(&self, pattern: &str, from: usize, until: usize) -> Option<usize> {
        let slot = self.patterns.iter().position(|p| *p == pattern)?;
        let positions = &self.positions[slot];
        let at = *positions.get(positions.partition_point(|&at| at < from))?;
        (at + pattern.len() <= until).then_some(at)
    }
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
        let rest = &source[at..range.end];
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
                let c = rest.chars().next().unwrap_or_default();
                push_escaped(out, &rest[..c.len_utf8()]);
                at += c.len_utf8();
            }
        }
    }
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
