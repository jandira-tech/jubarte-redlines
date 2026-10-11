// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word tokens for prose diffs.

use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};

const ZERO_WIDTH_JOINER: char = '\u{200D}';

/// A combining mark (Unicode category Mn, Mc or Me), an emoji skin tone
/// modifier (U+1F3FB to U+1F3FF, a symbol that behaves like a mark) or the
/// zero width joiner: it continues the token of the character before it.
fn is_continuation(c: char) -> bool {
    c == ZERO_WIDTH_JOINER
        || ('\u{1F3FB}'..='\u{1F3FF}').contains(&c)
        || c.general_category_group() == GeneralCategoryGroup::Mark
}

/// A symbol (an emoji, `+`, `$`): a base that marks and joiners attach to.
fn is_symbol(c: char) -> bool {
    c.general_category_group() == GeneralCategoryGroup::Symbol
}

/// What the token being built is made of.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// Letters and digits, with the marks and joiners that follow them.
    Word,
    /// Whitespace.
    Space,
    /// Marks and joiners with no letter, digit or symbol before them.
    Orphan,
    /// Punctuation or a symbol (the first one starts it; a symbol takes marks).
    Other(char),
}

/// `text` as words (runs of letters and digits), runs of whitespace, and
/// punctuation; a run of one repeated mark (`**`, `--`) is one token.
/// Concatenated, the tokens are `text`.
///
/// Combining marks (categories Mn, Mc, Me) and the zero width joiner stay in
/// the token of the letter, digit or symbol they follow, so decomposed text
/// (`cafe` + U+0301) is one word like its composed form, and an emoji
/// sequence joined by U+200D, with or without a skin tone modifier or
/// variation selector, is one token. (Flags, keycaps and tag sequences are
/// not covered and may split.) A joiner also links the letter,
/// digit or symbol after it. A mark with no such base before it (after
/// whitespace, punctuation or at the start of the text) does not join that
/// whitespace or punctuation: a run of such marks is a token of its own, and
/// what follows starts a new token.
pub fn word_tokens(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut state: Option<(Kind, char)> = None; // the token's kind and its last character
    // The last character is a joiner attached to a base.
    let mut joined = false;
    for (at, c) in text.char_indices() {
        if let Some((kind, last)) = state {
            let continues = if is_continuation(c) {
                match kind {
                    Kind::Word | Kind::Orphan => true,
                    Kind::Other(base) => is_symbol(base),
                    Kind::Space => false,
                }
            } else if joined {
                c.is_alphanumeric() || is_symbol(c)
            } else {
                match kind {
                    Kind::Word => c.is_alphanumeric(),
                    Kind::Space => c.is_whitespace(),
                    Kind::Orphan => false,
                    Kind::Other(_) => c == last,
                }
            };
            if continues {
                let kind = if c.is_alphanumeric() && !is_continuation(c) {
                    Kind::Word
                } else {
                    kind
                };
                joined = c == ZERO_WIDTH_JOINER && kind != Kind::Orphan;
                state = Some((kind, c));
                continue;
            }
            out.push(&text[start..at]);
            start = at;
        }
        // Some marks are also alphabetic (Devanagari vowel signs): with no base
        // before them they are still marks.
        let kind = if is_continuation(c) {
            Kind::Orphan
        } else if c.is_alphanumeric() {
            Kind::Word
        } else if c.is_whitespace() {
            Kind::Space
        } else {
            Kind::Other(c)
        };
        joined = false;
        state = Some((kind, c));
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn words_spaces_and_marks() {
        assert_eq!(
            word_tokens("**Bold** words, and 3.5%!"),
            [
                "**", "Bold", "**", " ", "words", ",", " ", "and", " ", "3", ".", "5", "%", "!"
            ]
        );
        assert_eq!(word_tokens(""), Vec::<&str>::new());
        assert_eq!(word_tokens("a\t\tb"), ["a", "\t\t", "b"]);
    }

    /// Same text, composed and decomposed: the same number of tokens.
    #[test]
    fn composed_and_decomposed_text_tokenize_alike() {
        for (composed, decomposed) in [
            ("caf\u{e9}", "cafe\u{301}"),
            (
                "Un caf\u{e9} tr\u{e8}s fort.",
                "Un cafe\u{301} tre\u{300}s fort.",
            ),
            (
                "na\u{ef}ve r\u{e9}sum\u{e9}",
                "nai\u{308}ve re\u{301}sume\u{301}",
            ),
            // Vietnamese: base + dot below + circumflex.
            ("Vi\u{1ec7}t Nam", "Vie\u{323}\u{302}t Nam"),
        ] {
            let (a, b) = (word_tokens(composed), word_tokens(decomposed));
            assert_eq!(
                a.len(),
                b.len(),
                "{composed:?} vs {decomposed:?}: {a:?} {b:?}"
            );
            assert_eq!(b.concat(), decomposed);
        }
    }

    #[test]
    fn a_decomposed_word_is_one_token() {
        assert_eq!(word_tokens("cafe\u{301}"), ["cafe\u{301}"]);
        assert_eq!(word_tokens("e\u{301}"), ["e\u{301}"]);
        assert_eq!(
            word_tokens("un cafe\u{301}, s'il"),
            ["un", " ", "cafe\u{301}", ",", " ", "s", "'", "il"]
        );
        // Marks stack: base, dot below, circumflex.
        assert_eq!(word_tokens("Vie\u{323}\u{302}t"), ["Vie\u{323}\u{302}t"]);
        // The word goes on after the mark.
        assert_eq!(word_tokens("cafe\u{301}s"), ["cafe\u{301}s"]);
        // Scripts whose vowel signs and viramas are marks, not letters.
        assert_eq!(
            word_tokens("\u{939}\u{93f}\u{928}\u{94d}\u{926}\u{940}"),
            ["\u{939}\u{93f}\u{928}\u{94d}\u{926}\u{940}"]
        );
    }

    /// A mark with no letter or digit before it (it follows whitespace,
    /// punctuation or the start of the text) never joins that punctuation or
    /// whitespace: a run of such marks is a token of its own, and the next
    /// word starts a new token.
    #[test]
    fn a_mark_without_a_base_is_a_token_of_its_own() {
        assert_eq!(word_tokens("\u{301}"), ["\u{301}"]);
        assert_eq!(word_tokens("\u{301}\u{308}"), ["\u{301}\u{308}"]);
        assert_eq!(word_tokens(".\u{301}"), [".", "\u{301}"]);
        assert_eq!(word_tokens("a \u{301}b"), ["a", " ", "\u{301}", "b"]);
        assert_eq!(word_tokens("a, \u{301}"), ["a", ",", " ", "\u{301}"]);
        assert_eq!(word_tokens("\u{301}a"), ["\u{301}", "a"]);
        // Whitespace after a mark is its own token too.
        assert_eq!(
            word_tokens("e\u{301} \u{301}"),
            ["e\u{301}", " ", "\u{301}"]
        );
    }

    #[test]
    fn a_zero_width_joiner_stays_inside_its_token() {
        // Between letters.
        assert_eq!(word_tokens("a\u{200d}b"), ["a\u{200d}b"]);
        // After a combining mark.
        assert_eq!(word_tokens("e\u{301}\u{200d}b"), ["e\u{301}\u{200d}b"]);
        // An emoji sequence (man, ZWJ, woman, ZWJ, girl).
        let family = "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}";
        assert_eq!(word_tokens(family), [family]);
        assert_eq!(
            word_tokens(&format!("x {family}!")),
            ["x", " ", family, "!"]
        );
        // A heart with its variation selector.
        assert_eq!(word_tokens("\u{2764}\u{fe0f}"), ["\u{2764}\u{fe0f}"]);
        // With no base it is a token of its own, like any other mark.
        assert_eq!(word_tokens("a \u{200d}"), ["a", " ", "\u{200d}"]);
    }

    #[test]
    fn a_skin_tone_modifier_stays_in_its_emoji_sequence() {
        for sequence in ["\u{1F469}\u{1F3FD}\u{200D}\u{1F4BB}", "\u{1F44D}\u{1F3FD}"] {
            assert_eq!(word_tokens(sequence), [sequence]);
        }
        assert_eq!(
            word_tokens("a \u{1F44D}\u{1F3FD} b"),
            ["a", " ", "\u{1F44D}\u{1F3FD}", " ", "b"]
        );
    }

    #[test]
    fn an_alphabetic_mark_with_no_base_does_not_start_a_word() {
        // U+093F is a Devanagari vowel sign: a mark that is also alphabetic.
        assert_eq!(word_tokens("a \u{93F}b"), ["a", " ", "\u{93F}", "b"]);
        assert_eq!(word_tokens("ab\u{93F}"), ["ab\u{93F}"]);
    }

    #[test]
    fn ascii_and_cjk_tokens_do_not_change() {
        assert_eq!(
            word_tokens("It's 3.5%--ok"),
            ["It", "'", "s", " ", "3", ".", "5", "%", "--", "ok"]
        );
        assert_eq!(
            word_tokens("\u{4e2d}\u{6587}\u{5b57} \u{6f22}\u{5b57}!"),
            ["\u{4e2d}\u{6587}\u{5b57}", " ", "\u{6f22}\u{5b57}", "!"]
        );
        // Distinct symbols are still separate tokens.
        assert_eq!(
            word_tokens("\u{1f600}\u{1f601}"),
            ["\u{1f600}", "\u{1f601}"]
        );
    }

    #[test]
    fn tokens_always_concatenate_to_the_text() {
        for text in [
            "",
            "cafe\u{301} \u{301}\u{200d}x.\u{308}",
            "\u{200d}\u{200d}",
            "\u{1f468}\u{200d}\u{1f469} e\u{301}\u{200d}",
            "**\u{301}**",
        ] {
            assert_eq!(word_tokens(text).concat(), text, "{text:?}");
        }
    }
}
