// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word tokens for prose diffs.

/// `text` as words (runs of letters and digits), runs of whitespace, and
/// punctuation; a run of one repeated mark (`**`, `--`) is one token.
/// Concatenated, the tokens are `text`.
pub fn word_tokens(text: &str) -> Vec<&str> {
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

#[cfg(test)]
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
}
