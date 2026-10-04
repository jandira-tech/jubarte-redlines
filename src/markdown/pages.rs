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
        let text = visible_text(first_cell(trimmed));
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
    if let Some(after) = rest.strip_prefix(['-', '*', '+']) {
        return after.starts_with(' ');
    }
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    digits > 0
        && rest
            .get(digits..)
            .is_some_and(|after| after.starts_with(". ") || after.starts_with(") "))
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

/// A Markdown line without what is never painted: link and image targets,
/// HTML tags and comments, footnote labels and CriticMarkup comments.
fn visible_text(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(ch) = rest.chars().next() {
        let skip = if rest.starts_with("](") {
            out.push(']');
            rest.find(')').map(|end| end.saturating_add(1))
        } else if rest.starts_with("{>>") {
            rest.find("<<}").map(|end| end.saturating_add(3))
        } else if rest.starts_with("[^") {
            rest.find(']').map(|end| end.saturating_add(1))
        } else if ch == '<' {
            rest.find('>').map(|end| end.saturating_add(1))
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
            visible_text("See [the site](https://example.com) {++now++}{>>Ann<<}"),
            "See [the site] {++now++}"
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
