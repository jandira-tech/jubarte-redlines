// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Page markers for Markdown read from a `.docx`.
//!
//! Markdown has no pages, so the page a block starts on comes from laying
//! the document out (the same pass that writes its PDF) and finding each
//! block's opening text, in order, in the text painted on each page. The
//! markers are HTML comments on their own lines, which Markdown readers
//! (jubarte's included) drop, so the Markdown still converts back to the
//! same document.

/// Characters of a block's opening text that must match the page text.
const KEY_CHARS: usize = 24;

/// The shorter key tried beside it, for text the page wraps beside other
/// text (table cells, columns) after its first few words.
const SHORT_KEY_CHARS: usize = 12;

/// How far past the last match a block's opening text is looked for, in
/// letters and digits. A block that is not found (text Word does not paint,
/// a field whose result differs) keeps the page of the block before it.
const SEARCH_WINDOW: usize = 20_000;

/// `markdown` with a `<!-- page N of M -->` line before the first block that
/// starts on each page, and `<!-- page 1 of M -->` first. `pages` holds the
/// text painted on each page, in order (`convert::RenderReport::pages`).
/// Markers go only before a block (a line after a blank line), never inside
/// a table, a list (tight or loose) or a code fence, so a page that starts
/// mid-block is named at the next block. With no pages, `markdown` is
/// returned as it is.
#[must_use]
pub fn paginate(markdown: &str, pages: &[&str]) -> String {
    let total = pages.len();
    if total == 0 {
        return markdown.to_string();
    }
    let stream: Vec<(char, usize)> = pages
        .iter()
        .enumerate()
        .flat_map(|(page, text)| letters(text).map(move |ch| (ch, page)))
        .collect();
    let defined = definitions(markdown);
    let mut out = String::with_capacity(markdown.len().saturating_add(total.saturating_mul(24)));
    out.push_str(&marker(1, total));
    let mut cursor = 0usize;
    let mut current = 0usize;
    let mut block_start = true;
    // The open code fence's character and length, while inside one.
    let mut fence: Option<(char, usize)> = None;
    // The last block is a list item (or its indented continuation): a
    // marker before the next item would split the list in two.
    let mut in_list = false;
    for line in markdown.split_inclusive('\n') {
        let trimmed = line.trim();
        let fenced = fence.is_some();
        match (fence, fence_run(trimmed)) {
            // A fence opens a block, never mid-paragraph text.
            (None, Some((ch, len, _))) if block_start => fence = Some((ch, len)),
            (Some((ch, len)), Some((other, run, true))) if other == ch && run >= len => {
                fence = None;
            }
            _ => {}
        }
        let row = trimmed.starts_with('|');
        let continues_list =
            in_list && (list_item(line) || line.starts_with([' ', '\t'])) && !trimmed.is_empty();
        // A link reference definition is never painted.
        let text = if !fenced && definition(line.trim_end()).is_some() {
            String::new()
        } else {
            visible_text(first_cell(trimmed), &defined)
        };
        let line_letters: Vec<char> = letters(&text).collect();
        let key = line_letters
            .get(..line_letters.len().min(KEY_CHARS))
            .unwrap_or(&[]);
        let short = key.get(..key.len().min(SHORT_KEY_CHARS)).unwrap_or(&[]);
        let full = find(&stream, cursor, key);
        let found = if row {
            // A row's cells interleave on the page, so its long key can miss
            // its own page and find a later repeat: take the earlier match.
            [full, find(&stream, cursor, short)]
                .into_iter()
                .flatten()
                .min()
        } else {
            // Elsewhere the full opening is on the page; the short key is a
            // fallback, since a shared opening ("The Supplier shall") can
            // occur earlier.
            full.or_else(|| find(&stream, cursor, short))
        };
        if let Some(found) = found {
            // Past everything of this line the page agrees with, so a later
            // block cannot match text quoted inside this one.
            let matched = if row {
                short.len()
            } else {
                common_prefix(&stream, found, &line_letters).max(short.len())
            };
            cursor = found.saturating_add(matched);
            let page = stream.get(found).map_or(current, |&(_, page)| page);
            if block_start && page > current && !fenced && !continues_list {
                current = page;
                out.push_str(&marker(page.saturating_add(1), total));
            }
        }
        out.push_str(line);
        if !trimmed.is_empty() {
            in_list = list_item(line) || continues_list;
        }
        block_start = trimmed.is_empty() && fence.is_none();
    }
    out
}

/// A line that opens with three or more backticks or tildes: the
/// character, how many, and whether nothing else follows (a closing fence).
fn fence_run(trimmed: &str) -> Option<(char, usize, bool)> {
    let ch = trimmed.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let run = trimmed.chars().take_while(|&c| c == ch).count();
    (run >= 3).then(|| (ch, run, trimmed.chars().skip(run).all(char::is_whitespace)))
}

/// A list item line: `-`, `*` or `+`, or digits and `.` or `)`, then a space.
fn list_item(line: &str) -> bool {
    let rest = line.trim_start();
    // A marker ends the line or is followed by a space or tab.
    let marker_ends = |after: &str| after.is_empty() || after.starts_with([' ', '\t']);
    if let Some(after) = rest.strip_prefix(['-', '*', '+']) {
        return marker_ends(after);
    }
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    digits > 0
        && rest
            .get(digits..)
            .and_then(|after| after.strip_prefix(['.', ')']))
            .is_some_and(marker_ends)
}

/// How many of `line` the page text repeats from `at` on.
fn common_prefix(stream: &[(char, usize)], at: usize, line: &[char]) -> usize {
    stream
        .get(at..)
        .unwrap_or(&[])
        .iter()
        .zip(line)
        .take_while(|((page_char, _), line_char)| page_char == *line_char)
        .count()
}

/// A table row's first cell; any other line as it is. The page text runs
/// one line per baseline, so cells side by side interleave there and only
/// the first cell's opening reads as it does in the Markdown.
fn first_cell(line: &str) -> &str {
    match line.strip_prefix('|') {
        Some(row) => row
            .split('|')
            .find(|cell| !cell.trim().is_empty())
            .unwrap_or(""),
        None => line,
    }
}

fn marker(page: usize, total: usize) -> String {
    format!("<!-- page {page} of {total} -->\n\n")
}

/// The letters and digits of `text`, lower-cased: what survives the trip
/// from Markdown to painted glyphs (bullets, numbering punctuation, spacing
/// and Markdown syntax do not).
fn letters(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
}

/// A Markdown line without what is never painted: images (alt text and
/// all), link targets, HTML tags and comments, footnote labels and
/// CriticMarkup comments. An autolink (`<https://...>`, `<ann@x.com>`)
/// keeps its text, which the page paints.
fn visible_text(line: &str, defined: &[String]) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(ch) = rest.chars().next() {
        let skip = if rest.starts_with("![") {
            image_len(rest, defined)
        } else if rest.starts_with("](") {
            out.push(']');
            rest.find(')').map(|end| end.saturating_add(1))
        } else if rest.starts_with("{>>") {
            rest.find("<<}").map(|end| end.saturating_add(3))
        } else if rest.starts_with("[^") {
            rest.find(']').map(|end| end.saturating_add(1))
        } else if ch == '<' {
            match autolink(rest) {
                Some(text) => {
                    out.push_str(text);
                    Some(text.len().saturating_add(2))
                }
                None => rest.find('>').map(|end| end.saturating_add(1)),
            }
        } else {
            None
        };
        let step = skip.unwrap_or_else(|| {
            out.push(ch);
            ch.len_utf8()
        });
        rest = rest.get(step..).unwrap_or("");
    }
    out
}

/// The length of the image `rest` opens: inline (`![alt](destination)`,
/// brackets balanced in the alt text, parentheses in the destination,
/// backslash escapes and a `<...>` destination honoured), or a full,
/// collapsed or shortcut reference (`![alt][label]`, `![alt][]`, `![alt]`)
/// whose label `defined` holds. `None` when `rest` opens no image: an
/// undefined reference is text CommonMark paints.
fn image_len(rest: &str, defined: &[String]) -> Option<usize> {
    let alt_end = bracket_end(rest.get(1..)?)?.checked_add(1)?;
    let after = rest.get(alt_end..)?;
    if let Some(destination) = after.strip_prefix('(') {
        let len = destination_len(destination)?;
        // `(`, the destination, `)`.
        return alt_end.checked_add(len)?.checked_add(2);
    }
    let alt = rest.get(2..alt_end.checked_sub(1)?)?;
    let (label, len) = match after.strip_prefix('[') {
        Some(reference) => {
            let close = reference.find(']')?;
            let label = reference.get(..close)?;
            // `![alt][]` is labelled by its alt text.
            let label = if label.trim().is_empty() { alt } else { label };
            (label, close.checked_add(2)?)
        }
        None => (alt, 0),
    };
    defined
        .contains(&normalize_label(label))
        .then(|| alt_end.saturating_add(len))
}

/// The length of the bracketed text `text` opens (`[` first), through its
/// matching `]`, with nested brackets balanced and backslash escapes.
fn bracket_end(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut chars = text.char_indices();
    chars.next().filter(|&(_, c)| c == '[')?;
    while let Some((at, ch)) = chars.next() {
        match ch {
            '\\' => {
                chars.next();
            }
            '[' => depth = depth.saturating_add(1),
            ']' if depth == 0 => return at.checked_add(1),
            ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

/// The length of an inline link's destination and optional title up to
/// its closing `)` (excluded): a `<...>` destination or one with balanced
/// parentheses and backslash escapes, then a title in `"..."`, `'...'` or
/// `(...)`, whose own `)` does not close the link.
fn destination_len(text: &str) -> Option<usize> {
    let mut at = if let Some(inner) = text.strip_prefix('<') {
        inner.find('>')?.checked_add(2)?
    } else {
        let mut depth = 0usize;
        let mut end = text.len();
        let mut chars = text.char_indices();
        while let Some((at, ch)) = chars.next() {
            match ch {
                '\\' => {
                    chars.next();
                }
                '(' => depth = depth.saturating_add(1),
                ')' if depth == 0 => {
                    end = at;
                    break;
                }
                ')' => depth = depth.saturating_sub(1),
                // A destination outside `<>` holds no whitespace: a title
                // or the closing `)` follows.
                ch if ch.is_whitespace() => {
                    end = at;
                    break;
                }
                _ => {}
            }
        }
        end
    };
    at = at.checked_add(leading_whitespace(text.get(at..)?))?;
    let close = match text.get(at..)?.chars().next()? {
        '"' => Some('"'),
        '\'' => Some('\''),
        '(' => Some(')'),
        _ => None,
    };
    if let Some(close) = close {
        let mut chars = text.get(at..)?.char_indices().skip(1);
        let end = loop {
            match chars.next()? {
                (_, '\\') => {
                    chars.next();
                }
                (offset, ch) if ch == close => break offset,
                _ => {}
            }
        };
        at = at.checked_add(end)?.checked_add(1)?;
        at = at.checked_add(leading_whitespace(text.get(at..)?))?;
    }
    text.get(at..)?.starts_with(')').then_some(at)
}

fn leading_whitespace(text: &str) -> usize {
    text.len().saturating_sub(text.trim_start().len())
}

/// The label a link reference definition line (`[label]: destination`)
/// defines, as CommonMark matches labels: case-folded, inner whitespace
/// collapsed.
fn definition(line: &str) -> Option<String> {
    // Four columns of indentation (a tab reaches the next stop of four) make
    // an indented code block.
    let mut columns = 0usize;
    for ch in line.chars() {
        columns = match ch {
            ' ' => columns.saturating_add(1),
            '\t' => (columns / 4).saturating_add(1).saturating_mul(4),
            _ => break,
        };
        if columns > 3 {
            return None;
        }
    }
    let rest = line.trim_start();
    let end = bracket_end(rest)?;
    let label = rest.get(1..end.checked_sub(1)?)?;
    if label.trim().is_empty() || label.starts_with('^') {
        return None;
    }
    let after = rest.get(end..)?.strip_prefix(':')?;
    (!after.trim().is_empty()).then(|| normalize_label(label))
}

/// The labels `markdown`'s link reference definitions define.
fn definitions(markdown: &str) -> Vec<String> {
    let mut labels: Vec<String> = Vec::new();
    // The open code fence's character and length, while inside one.
    let mut fence: Option<(char, usize)> = None;
    for line in markdown.lines() {
        match (fence, fence_run(line.trim())) {
            (None, Some((ch, len, _))) => {
                fence = Some((ch, len));
                continue;
            }
            (Some((ch, len)), Some((other, run, true))) if other == ch && run >= len => {
                fence = None;
                continue;
            }
            _ => {}
        }
        if fence.is_none()
            && let Some(label) = definition(line)
            && !labels.contains(&label)
        {
            labels.push(label);
        }
    }
    labels
}

fn normalize_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The text of the CommonMark autolink `rest` opens: `<scheme:...>` (a
/// scheme of 2 to 32 letters, digits, `+`, `.` or `-`, starting with a
/// letter) or `<local@domain>`. Raw HTML (`<span>`, `<br/>`) is none.
fn autolink(rest: &str) -> Option<&str> {
    let inner = rest.strip_prefix('<')?;
    let inner = inner.get(..inner.find('>')?)?;
    // CommonMark forbids only ASCII controls, ASCII space and `<` here.
    if inner.is_empty()
        || inner
            .chars()
            .any(|c| c.is_ascii_control() || matches!(c, ' ' | '<'))
    {
        return None;
    }
    let uri = inner.split_once(':').is_some_and(|(scheme, _)| {
        (2..=32).contains(&scheme.len())
            && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
    });
    let email = inner.split_once('@').is_some_and(|(local, domain)| {
        !local.is_empty()
            && !domain.is_empty()
            && local
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(c))
            && domain
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.'))
    });
    (uri || email).then_some(inner)
}

/// Where `key` starts in `stream` at or after `from`, within the window.
fn find(stream: &[(char, usize)], from: usize, key: &[char]) -> Option<usize> {
    if key.is_empty() {
        return None;
    }
    let end = from
        .saturating_add(SEARCH_WINDOW)
        .min(stream.len())
        .checked_sub(key.len())?;
    (from..=end).find(|&start| {
        stream
            .get(start..start.saturating_add(key.len()))
            .is_some_and(|window| window.iter().map(|&(c, _)| c).eq(key.iter().copied()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_the_first_block_on_each_page() {
        let markdown =
            "# Title\n\nFirst paragraph here.\n\nSecond paragraph starts on page two.\n\nThird.\n";
        let pages = [
            "Title\nFirst paragraph here.",
            "Second paragraph starts\non page two.\nThird.",
        ];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\n# Title\n\nFirst paragraph here.\n\n<!-- page 2 of 2 -->\n\nSecond paragraph starts on page two.\n\nThird.\n"
        );
    }

    #[test]
    fn never_splits_a_table_and_names_the_page_at_the_next_block() {
        let markdown = "| a | b |\n|---|---|\n| row one | x |\n| row two | y |\n\nAfter.\n";
        let pages = ["a b\nrow one x", "row two y\nAfter."];
        let out = paginate(markdown, &pages);
        assert_eq!(
            out,
            "<!-- page 1 of 2 -->\n\n| a | b |\n|---|---|\n| row one | x |\n| row two | y |\n\n<!-- page 2 of 2 -->\n\nAfter.\n"
        );
    }

    #[test]
    fn interleaved_table_cells_do_not_jump_to_a_later_repeat() {
        // Page 1 paints the two cells side by side, one baseline at a time;
        // page 3 repeats the first cell's text as a list item.
        let markdown = "| Microsoft Support Track Changes | Word supports views |\n|-|-|\n\nNext section heading here.\n\nThe sources again.\n\nMicrosoft Support Track Changes: Word supports views.\n";
        let pages = [
            "Microsoft Support Word supports\nTrack Changes views",
            "Next section heading here.",
            "The sources again.\nMicrosoft Support Track Changes: Word supports views.",
        ];
        let out = paginate(markdown, &pages);
        assert!(
            out.contains("<!-- page 2 of 3 -->\n\nNext section heading here."),
            "{out}"
        );
        assert!(
            out.contains("<!-- page 3 of 3 -->\n\nThe sources again."),
            "{out}"
        );
    }

    #[test]
    fn a_shared_opening_does_not_pull_a_paragraph_back_a_page() {
        // "The Supplier shall" opens the second paragraph and also recurs
        // inside the first; only the second paragraph's full opening is on
        // page 2.
        let markdown = "The Supplier shall deliver. The Supplier shall pay.\n\nThe Supplier shall indemnify the buyer.\n";
        let pages = [
            "The Supplier shall deliver. The Supplier shall pay.",
            "The Supplier shall indemnify the buyer.",
        ];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\nThe Supplier shall deliver. The Supplier shall pay.\n\n<!-- page 2 of 2 -->\n\nThe Supplier shall indemnify the buyer.\n"
        );
    }

    #[test]
    fn a_backtick_line_inside_a_paragraph_does_not_silence_later_markers() {
        let markdown = "Text\\\n```\nmore text.\n\nSecond page starts here.\n";
        let pages = ["Text\n```\nmore text.", "Second page starts here."];
        assert!(
            paginate(markdown, &pages).contains("<!-- page 2 of 2 -->\n\nSecond page"),
            "{}",
            paginate(markdown, &pages)
        );
    }

    #[test]
    fn no_marker_inside_a_real_code_fence() {
        let markdown = "```\ncode one\n\ncode two on page two\n```\n\nAfter the fence.\n";
        let pages = ["code one", "code two on page two\nAfter the fence."];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\n```\ncode one\n\ncode two on page two\n```\n\n<!-- page 2 of 2 -->\n\nAfter the fence.\n"
        );
    }

    #[test]
    fn a_title_quoted_earlier_does_not_pin_its_heading_to_that_page() {
        let markdown = "Long paragraph. The section Definitions of duties and obligations governs everything.\n\n## Definitions of duties and obligations\n\nBody.\n";
        let pages = [
            "Long paragraph. The section Definitions of duties and obligations governs everything.",
            "Definitions of duties and obligations\nBody.",
        ];
        assert!(
            paginate(markdown, &pages)
                .contains("<!-- page 2 of 2 -->\n\n## Definitions of duties and obligations"),
            "{}",
            paginate(markdown, &pages)
        );
    }

    #[test]
    fn a_tab_or_nothing_after_the_bullet_is_still_a_list_item() {
        for line in ["-\tfoo", "-", "1.\tfoo", "2)", "  * x"] {
            assert!(list_item(line), "{line:?}");
        }
        for line in ["-foo", "1.5 percent", "**bold**"] {
            assert!(!list_item(line), "{line:?}");
        }
    }

    #[test]
    fn a_loose_list_is_not_split_by_a_marker() {
        let markdown = "- First item.\n\n- Second item on page two.\n\nAfter the list.\n";
        let pages = ["First item.", "Second item on page two.\nAfter the list."];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\n- First item.\n\n- Second item on page two.\n\n<!-- page 2 of 2 -->\n\nAfter the list.\n"
        );
    }

    #[test]
    fn ignores_link_targets_and_critic_comments() {
        assert_eq!(
            visible_text(
                "See [the site](https://example.com) {++now++}{>>Ann<<}",
                &[]
            ),
            "See [the site] {++now++}"
        );
    }

    #[test]
    fn an_image_alt_text_is_not_painted_text() {
        assert_eq!(visible_text("![Confidential](logo.png) Body", &[]), " Body");
        // An escaped bracket in the alt text, a destination in `<>` with a
        // space and a parenthesis in it.
        assert_eq!(
            visible_text("![a \\] b](<my logo (1).png>) after", &[]),
            " after"
        );
        // A link label is still painted.
        assert_eq!(visible_text("[Terms](terms.md)", &[]), "[Terms]");
        // The alt text recurs as body text on page 2: the image must not
        // pull the page 2 marker up to itself.
        let markdown = "Intro.\n\n![Confidential](logo.png)\n\nBody one.\n\nConfidential notice on page two.\n";
        let pages = ["Intro.\nBody one.", "Confidential notice on page two."];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\nIntro.\n\n![Confidential](logo.png)\n\nBody one.\n\n<!-- page 2 of 2 -->\n\nConfidential notice on page two.\n"
        );
    }

    #[test]
    fn an_image_with_nested_brackets_or_parentheses_is_dropped_whole() {
        // Balanced brackets in the alt text, balanced parentheses in the
        // destination (jubarte's own image_markdown leaves `(` and `)` in a
        // file name).
        assert_eq!(
            visible_text("![Confidential [notice]](logo.png) Body", &[]),
            " Body"
        );
        assert_eq!(visible_text("![alt](my_(logo).png) Body", &[]), " Body");
        assert_eq!(visible_text("![alt](a\\)b.png) Body", &[]), " Body");
    }

    #[test]
    fn a_reference_image_is_dropped_only_when_its_label_is_defined() {
        let defined = definitions("Text.\n\n[Logo]:  logo.png\n[other]: x.png \"t\"\n");
        assert_eq!(defined, ["logo", "other"]);
        // Full, collapsed and shortcut references to a defined label.
        assert_eq!(
            visible_text("![Confidential][logo] Body", &defined),
            " Body"
        );
        assert_eq!(visible_text("![Logo][] Body", &defined), " Body");
        assert_eq!(visible_text("![logo] Body", &defined), " Body");
        // An undefined label is not an image: CommonMark paints the text.
        assert_eq!(
            visible_text("![Confidential][missing] Body", &defined),
            "![Confidential][missing] Body"
        );
        assert_eq!(visible_text("![nothing] Body", &defined), "![nothing] Body");
        // The definition line itself is not painted, and the image's alt
        // text must not pull the page 2 marker up to it.
        let markdown = "Intro.\n\n![Confidential][logo]\n\nBody one.\n\nConfidential notice on page two.\n\n[logo]: logo.png\n";
        let pages = ["Intro.\nBody one.", "Confidential notice on page two."];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\nIntro.\n\n![Confidential][logo]\n\nBody one.\n\n<!-- page 2 of 2 -->\n\nConfidential notice on page two.\n\n[logo]: logo.png\n"
        );
    }

    #[test]
    fn an_image_title_with_a_parenthesis_does_not_end_the_image() {
        for line in [
            "![alt](logo.png \"version ) one\") Body",
            "![alt](logo.png 'version ) one') Body",
            "![alt](logo.png (version \\) one)) Body",
            "![alt](<my logo.png> \"a ) b\") Body",
        ] {
            assert_eq!(visible_text(line, &[]), " Body", "{line}");
        }
    }

    #[test]
    fn code_that_looks_like_a_definition_is_not_one() {
        // Four spaces or a tab make an indented code block, which
        // CommonMark paints verbatim; up to three spaces is a definition.
        assert_eq!(definition("    [logo]: logo.png"), None);
        assert_eq!(definition("\t[logo]: logo.png"), None);
        assert_eq!(definition("   [logo]: logo.png").as_deref(), Some("logo"));
        // A definition-shaped line inside a fence defines nothing.
        assert!(definitions("```\n[logo]: logo.png\n```\n").is_empty());
        assert!(definitions("~~~~\n[logo]: a\n~~~\n[x]: b\n").is_empty());
        // The indented code line keeps its page key.
        let markdown = "Intro.\n\n    [code]: sample text\n\nAfter.\n";
        let pages = ["Intro.", "[code]: sample text\nAfter."];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\nIntro.\n\n<!-- page 2 of 2 -->\n\n    [code]: sample text\n\nAfter.\n"
        );
    }

    #[test]
    fn an_autolink_keeps_its_text_and_html_tags_do_not() {
        assert_eq!(
            visible_text("<https://example.com/terms> or <ann@example.com>", &[]),
            "https://example.com/terms or ann@example.com"
        );
        assert_eq!(visible_text("<span class=\"x\">hi</span><br/>", &[]), "hi");
        // A URI autolink forbids only ASCII controls, ASCII space and angle
        // brackets: an ideographic space is part of the link.
        assert_eq!(
            visible_text("<ab:\u{3000}terms> x", &[]),
            "ab:\u{3000}terms x"
        );
        assert_eq!(visible_text("<ab: terms> x", &[]), " x");
        let markdown = "First page text.\n\n<https://example.com/terms>\n\nMore.\n";
        let pages = ["First page text.", "https://example.com/terms\nMore."];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\nFirst page text.\n\n<!-- page 2 of 2 -->\n\n<https://example.com/terms>\n\nMore.\n"
        );
    }

    #[test]
    fn unmatched_blocks_keep_the_current_page() {
        let markdown = "One.\n\nNot painted anywhere.\n\nTwo.\n";
        let pages = ["One.", "Two."];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\nOne.\n\nNot painted anywhere.\n\n<!-- page 2 of 2 -->\n\nTwo.\n"
        );
    }

    #[test]
    fn no_pages_leaves_the_markdown_alone() {
        assert_eq!(paginate("Text.\n", &[]), "Text.\n");
    }
}
