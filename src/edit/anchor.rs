// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Anchors that differ from the paragraph only in characters a reader
//! cannot tell apart: a non-breaking space typed as a space, a curly quote
//! as a straight one, a non-breaking hyphen as a hyphen, a zero-width
//! character left out. Such an anchor is found, and an anchor that is not
//! found is echoed with those characters escaped.

/// What `c` reads as, or `None` for a character that shows nothing.
fn reads_as(c: char) -> Option<char> {
    match c {
        '\u{A0}' | '\u{2007}' | '\u{202F}' | '\u{2000}'..='\u{200A}' => Some(' '),
        // Curled and other look-alike quotes read as the straight one. A
        // backtick-like mark (`\u{2035}`, `\u{B4}`) does not: the straight
        // quotes and the backtick are what a shell quotes with.
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' | '\u{2032}' | '\u{02B9}'
        | '\u{02BB}' | '\u{02BC}' | '\u{FF07}' => Some('\''),
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' | '\u{2033}' | '\u{2036}'
        | '\u{02BA}' | '\u{02EE}' | '\u{FF02}' | '\u{301D}' | '\u{301E}' | '\u{301F}'
        | '\u{05F4}' => Some('"'),
        '\u{2010}' | '\u{2011}' => Some('-'),
        '\u{AD}' | '\u{200B}'..='\u{200D}' | '\u{2060}' | '\u{FEFF}' => None,
        c => Some(c),
    }
}

/// `text` as it reads, with the byte range of `text` each byte of it
/// comes from.
fn reading(text: &str) -> (String, Vec<(usize, usize)>) {
    let mut out = String::with_capacity(text.len());
    let mut from = Vec::with_capacity(text.len());
    for (i, c) in text.char_indices() {
        if let Some(r) = reads_as(c) {
            out.push(r);
            from.extend(std::iter::repeat_n((i, i + c.len_utf8()), r.len_utf8()));
        }
    }
    (out, from)
}

/// Byte ranges of `text` that read as `find`, in order (overlapping ones
/// too); none when `find` reads as nothing.
pub(super) fn reading_hits(text: &str, find: &str) -> Vec<(usize, usize)> {
    let (needle, _) = reading(find);
    if needle.is_empty() {
        return Vec::new();
    }
    let (haystack, from) = reading(text);
    haystack
        .char_indices()
        .map(|(i, _)| i)
        .filter(|&i| haystack[i..].starts_with(&needle))
        .map(|i| (from[i].0, from[i + needle.len() - 1].1))
        .collect()
}

/// `text` quoted, with every character that looks like another or shows
/// nothing written as `\u{…}`, so an agent sees why its anchor missed.
pub(super) fn shown(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() || reads_as(c) != Some(c) && !c.is_alphanumeric() => {
                out.push_str(&format!("\\u{{{:x}}}", u32::from(c)));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn text_is_found_by_what_it_reads_as() {
        let text =
            "\u{201D} shall mean (1)\u{A0}any officer\u{2019}s attorney\u{2011}in\u{2011}fact";
        let hit = |find: &str| {
            reading_hits(text, find)
                .into_iter()
                .map(|(s, e)| &text[s..e])
                .collect::<Vec<_>>()
        };
        assert_eq!(
            hit("\" shall mean (1) any officer's"),
            ["\u{201D} shall mean (1)\u{A0}any officer\u{2019}s"]
        );
        assert_eq!(hit("attorney-in-fact"), ["attorney\u{2011}in\u{2011}fact"]);
        assert!(hit("attorney in fact").is_empty());
        // A zero-width character reads as nothing, on either side.
        assert_eq!(reading_hits("ab\u{200B}c", "abc"), [(0, 6)]);
        assert_eq!(reading_hits("abc", "a\u{AD}bc"), [(0, 3)]);
        assert!(reading_hits("abc", "\u{200B}").is_empty());
        // Repeated text overlaps, as the exact search counts it.
        assert_eq!(reading_hits("aaa", "aa"), [(0, 2), (1, 3)]);
        // Every character that reads as a quote is one; a backtick-like
        // mark is not, and a straight quote finds only itself or a curled one.
        for (quote, straight) in [
            ('\u{201C}', '"'),
            ('\u{201D}', '"'),
            ('\u{201E}', '"'),
            ('\u{201F}', '"'),
            ('\u{2033}', '"'),
            ('\u{2036}', '"'),
            ('\u{02BA}', '"'),
            ('\u{02EE}', '"'),
            ('\u{FF02}', '"'),
            ('\u{301D}', '"'),
            ('\u{301E}', '"'),
            ('\u{301F}', '"'),
            ('\u{05F4}', '"'),
            ('\u{2018}', '\''),
            ('\u{2019}', '\''),
            ('\u{02B9}', '\''),
            ('\u{02BB}', '\''),
            ('\u{02BC}', '\''),
            ('\u{FF07}', '\''),
        ] {
            let text = format!("say {quote}hi{quote}");
            let find = format!("say {straight}hi{straight}");
            assert_eq!(reading_hits(&text, &find).len(), 1, "{quote:?}");
        }
        for mark in ['`', '\u{2035}', '\u{B4}'] {
            assert!(
                reading_hits(&format!("it{mark}s"), "it's").is_empty(),
                "{mark:?}"
            );
        }
        assert!(reading_hits("say \"hi\"", "say 'hi'").is_empty());
    }

    #[test]
    fn lookalikes_and_invisible_characters_are_shown_escaped() {
        assert_eq!(
            shown("(1)\u{A0}any \u{2019}x\u{2019} a\u{2011}b\u{200B}"),
            r#""(1)\u{a0}any \u{2019}x\u{2019} a\u{2011}b\u{200b}""#
        );
        assert_eq!(shown("say \"hi\"\t\\"), r#""say \"hi\"\t\\""#);
        assert_eq!(shown("café naïve"), r#""café naïve""#);
    }
}
