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
/// backslash, stays text. Spans do not nest: delimiters inside a span are
/// its text.
pub(crate) fn encode(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut at = 0;
    while at < source.len() {
        let rest = &source[at..];
        let span = SPANS
            .iter()
            .find(|(open, ..)| rest.starts_with(open))
            .filter(|_| !escaped(source, at))
            .and_then(|&(open, close, separator, start, end)| {
                let inner = &rest[open.len()..];
                let length = inner.find(close)?;
                let body = &inner[..length];
                let split = match separator {
                    Some(separator) => Some((body.find(separator)?, separator.len())),
                    None => None,
                };
                Some((open.len() + length + close.len(), body, split, start, end))
            });
        match span {
            Some((consumed, body, split, start, end)) => {
                out.push(stand_in(start));
                match split {
                    Some((mid, width)) => {
                        push_escaped(&mut out, &body[..mid]);
                        out.push(stand_in(Token::SubstituteSeparator));
                        push_escaped(&mut out, &body[mid + width..]);
                    }
                    None => push_escaped(&mut out, body),
                }
                out.push(stand_in(end));
                at += consumed;
            }
            None => {
                let c = rest.chars().next().unwrap_or_default();
                push_escaped(&mut out, &rest[..c.len_utf8()]);
                at += c.len_utf8();
            }
        }
    }
    out
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
    let mut state = In::Text;
    Pieces::default().feed(&encode(markdown), |piece| match piece {
        Piece::Text(text) => {
            let shown = match state {
                In::Text => true,
                In::Inserted | In::New => accept,
                In::Deleted | In::Old => !accept,
                In::Comment => false,
            };
            if shown {
                out.push_str(text);
            }
        }
        Piece::Token(token) => {
            state = match token {
                Token::InsertStart => In::Inserted,
                Token::DeleteStart => In::Deleted,
                Token::SubstituteStart => In::Old,
                Token::SubstituteSeparator => In::New,
                Token::CommentStart => In::Comment,
                Token::HighlightStart
                | Token::HighlightEnd
                | Token::InsertEnd
                | Token::DeleteEnd
                | Token::SubstituteEnd
                | Token::CommentEnd => In::Text,
            };
        }
    });
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
    fn spans_cross_paragraphs_and_do_not_nest() {
        assert_eq!(
            pieces(&encode("A{++\n\nB {--c--}++}")),
            ["A", "<InsertStart>", "\n\nB {--c--}", "<InsertEnd>"]
        );
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
