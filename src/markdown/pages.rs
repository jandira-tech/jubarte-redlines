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
    let unpainted = unpainted(markdown);
    // Byte offset of the line being read.
    let mut offset = 0usize;
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
        let line_offset = offset;
        offset = offset.saturating_add(line.len());
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
        let painted = mask(line, line_offset, &unpainted);
        let text = visible_text(first_cell(painted.trim()));
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
    let Some(row) = line.strip_prefix('|') else {
        return line;
    };
    // A cell ends at a `|` no backslash escapes (`\|` is a pipe in the text).
    let mut start = 0;
    let mut escaped = false;
    for (at, ch) in row.char_indices() {
        match ch {
            '\\' => escaped = !escaped,
            '|' if !escaped => {
                let cell = row.get(start..at).unwrap_or("");
                if !cell.trim().is_empty() {
                    return cell;
                }
                start = at.saturating_add(1);
            }
            _ => escaped = false,
        }
    }
    row.get(start..)
        .filter(|cell| !cell.trim().is_empty())
        .unwrap_or("")
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
            match autolink(rest) {
                Some(text) => {
                    out.push_str(text);
                    Some(text.len().saturating_add(2))
                }
                None if rest.starts_with("<!--") => {
                    rest.find("-->").map(|end| end.saturating_add(3))
                }
                None => html_tag_len(rest),
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

/// HTML elements a Markdown line may carry: the writer's own (`br`, `sup`,
/// `sub`) and the inline and block elements an author writes. Any other
/// name in angle brackets (`<Draft>`, a placeholder in Word text) is text
/// the page paints.
const HTML_ELEMENTS: [&str; 52] = [
    "a",
    "abbr",
    "b",
    "bdi",
    "bdo",
    "big",
    "blockquote",
    "br",
    "center",
    "cite",
    "code",
    "col",
    "colgroup",
    "data",
    "dd",
    "del",
    "details",
    "dfn",
    "div",
    "dl",
    "dt",
    "em",
    "font",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "img",
    "ins",
    "kbd",
    "li",
    "mark",
    "ol",
    "p",
    "pre",
    "q",
    "s",
    "samp",
    "small",
    "span",
    "strong",
    "sub",
    "summary",
    "sup",
    "table",
    "u",
    "ul",
    "wbr",
];

/// The length of the HTML tag `rest` opens (`<sup>`, `</span>`,
/// `<br/>`, `<span class="x">`), if its name is one of [`HTML_ELEMENTS`].
fn html_tag_len(rest: &str) -> Option<usize> {
    let body = rest.strip_prefix('<')?;
    let body = body.strip_prefix('/').unwrap_or(body);
    let name_len = body
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .unwrap_or(body.len());
    let name = body.get(..name_len)?;
    let after = body.get(name_len..)?;
    if !after.starts_with(|c: char| c.is_ascii_whitespace() || c == '/' || c == '>')
        || !HTML_ELEMENTS.contains(&name.to_ascii_lowercase().as_str())
    {
        return None;
    }
    rest.find('>').map(|end| end.saturating_add(1))
}

/// The byte ranges of `markdown` that are never painted, as pulldown-cmark
/// (with the extensions jubarte's writer reads) parses them: images (alt
/// text, destination and title, inline or by reference) and link reference
/// definitions. A definition never interrupts a paragraph, nothing may
/// follow its title, and a footnote is not one.
fn unpainted(markdown: &str) -> Vec<std::ops::Range<usize>> {
    let parser = pulldown_cmark::Parser::new_ext(markdown, super::write::parser_options());
    let mut ranges: Vec<std::ops::Range<usize>> = parser
        .reference_definitions()
        .iter()
        .map(|(_, definition)| definition.span.clone())
        .collect();
    ranges.extend(parser.into_offset_iter().filter_map(|(event, range)| {
        matches!(
            event,
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image { .. })
        )
        .then(|| {
            // A collapsed reference's range stops before its `[]`.
            let collapsed = markdown
                .get(range.end..)
                .is_some_and(|rest| rest.starts_with("[]"));
            range.start..range.end.saturating_add(if collapsed { 2 } else { 0 })
        })
    }));
    ranges
}

/// `line` (which starts at byte `offset` of the document) without the
/// characters `unpainted` covers.
fn mask(line: &str, offset: usize, unpainted: &[std::ops::Range<usize>]) -> String {
    let end = offset.saturating_add(line.len());
    let ranges: Vec<&std::ops::Range<usize>> = unpainted
        .iter()
        .filter(|range| range.start < end && offset < range.end)
        .collect();
    if ranges.is_empty() {
        return line.to_string();
    }
    line.char_indices()
        .filter(|&(at, _)| {
            let at = offset.saturating_add(at);
            !ranges.iter().any(|range| range.contains(&at))
        })
        .map(|(_, ch)| ch)
        .collect()
}

/// The labels the document's link reference definitions define.
#[cfg(test)]
fn definitions(markdown: &str) -> Vec<String> {
    let parser = pulldown_cmark::Parser::new_ext(markdown, super::write::parser_options());
    let mut labels: Vec<String> = parser
        .reference_definitions()
        .iter()
        .map(|(label, _)| label.to_lowercase())
        .collect();
    labels.sort();
    labels
}

/// The page text of `markdown`'s first line, as `paginate` keys it.
#[cfg(test)]
fn line_text(markdown: &str) -> String {
    let line = markdown.split_inclusive('\n').next().unwrap_or("");
    visible_text(mask(line, 0, &unpainted(markdown)).trim_end_matches('\n'))
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

    /// A `|` in a cell's text is written `\|` and does not end the cell.
    #[test]
    fn an_escaped_pipe_stays_inside_the_first_cell() {
        assert_eq!(
            first_cell(r"| Terms \| conditions | x |"),
            r" Terms \| conditions "
        );
        assert_eq!(first_cell(r"| | a\|b |"), r" a\|b ");
        assert_eq!(first_cell("plain line"), "plain line");
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
            visible_text("See [the site](https://example.com) {++now++}{>>Ann<<}"),
            "See [the site] {++now++}"
        );
    }

    #[test]
    fn an_image_alt_text_is_not_painted_text() {
        assert_eq!(line_text("![Confidential](logo.png) Body"), " Body");
        // An escaped bracket in the alt text, a destination in `<>` with a
        // space and a parenthesis in it.
        assert_eq!(line_text("![a \\] b](<my logo (1).png>) after"), " after");
        // A link label is still painted.
        assert_eq!(line_text("[Terms](terms.md)"), "[Terms]");
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
            line_text("![Confidential [notice]](logo.png) Body"),
            " Body"
        );
        assert_eq!(line_text("![alt](my_(logo).png) Body"), " Body");
        assert_eq!(line_text("![alt](a\\)b.png) Body"), " Body");
    }

    #[test]
    fn a_reference_image_is_dropped_only_when_its_label_is_defined() {
        let defs = "[Logo]:  logo.png\n[other]: x.png \"t\"\n";
        assert_eq!(definitions(&format!("Text.\n\n{defs}")), ["logo", "other"]);
        // Full, collapsed and shortcut references to a defined label.
        assert_eq!(
            line_text(&format!("{}\n\n{defs}", "![Confidential][logo] Body")),
            " Body"
        );
        assert_eq!(
            line_text(&format!("{}\n\n{defs}", "![Logo][] Body")),
            " Body"
        );
        assert_eq!(line_text(&format!("{}\n\n{defs}", "![logo] Body")), " Body");
        // An undefined label is not an image: CommonMark paints the text.
        assert_eq!(
            line_text(&format!("{}\n\n{defs}", "![Confidential][missing] Body")),
            "![Confidential][missing] Body"
        );
        assert_eq!(
            line_text(&format!("{}\n\n{defs}", "![nothing] Body")),
            "![nothing] Body"
        );
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
            assert_eq!(line_text(line), " Body", "{line}");
        }
    }

    #[test]
    fn escaped_closers_inside_an_image_do_not_end_it() {
        // An escaped `>` in an angle-bracket destination, an escaped `]`
        // in a reference label.
        assert_eq!(line_text("![alt](<foo\\>bar>) Body"), " Body");
        assert_eq!(
            line_text("![alt][foo\\]bar] Body\n\n[foo\\]bar]: image.png\n"),
            " Body"
        );
    }

    #[test]
    fn code_that_looks_like_a_definition_is_not_one() {
        // Four spaces or a tab make an indented code block, which
        // CommonMark paints verbatim; up to three spaces is a definition.
        assert!(definitions("    [logo]: logo.png\n").is_empty());
        assert!(definitions("\t[logo]: logo.png\n").is_empty());
        assert_eq!(definitions("   [logo]: logo.png\n"), ["logo"]);
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
    fn only_what_commonmark_reads_as_a_definition_is_one() {
        // Text after the title, a definition-shaped line that would
        // interrupt a paragraph, and a footnote: all painted.
        assert!(definitions("[foo]: /url \"title\" trailing\n").is_empty());
        assert!(definitions("Some text\n[bar]: /baz\n").is_empty());
        assert!(definitions("Text[^1].\n\n[^1]: The note.\n").is_empty());
        // A definition right after another one is a definition.
        assert_eq!(definitions("[a]: /x\n[B]: /y\n"), ["a", "b"]);
        // The painted line keeps its key: it is on page 2.
        let markdown = "Intro.\n\nSome text\n[bar]: /baz on page two\n\nAfter.\n";
        let pages = ["Intro.\nSome text", "[bar]: /baz on page two\nAfter."];
        let out = paginate(markdown, &pages);
        assert!(out.contains("<!-- page 2 of 2 -->\n\nAfter."), "{out}");
        let markdown = "Intro.\n\n[foo]: /url \"title\" trailing words\n\nAfter.\n";
        let pages = ["Intro.", "[foo]: /url \"title\" trailing words\nAfter."];
        assert!(
            paginate(markdown, &pages).contains("<!-- page 2 of 2 -->\n\n[foo]: /url"),
            "{}",
            paginate(markdown, &pages)
        );
    }

    #[test]
    fn an_autolink_keeps_its_text_and_html_tags_do_not() {
        assert_eq!(
            line_text("<https://example.com/terms> or <ann@example.com>"),
            "https://example.com/terms or ann@example.com"
        );
        assert_eq!(line_text("<span class=\"x\">hi</span><br/>"), "hi");
        // A URI autolink forbids only ASCII controls, ASCII space and angle
        // brackets: an ideographic space is part of the link.
        assert_eq!(line_text("<ab:\u{3000}terms> x"), "ab:\u{3000}terms x");
        // Not an autolink and not a tag (a tag name holds no `:` or space):
        // CommonMark renders it as text, and the page paints it.
        assert_eq!(line_text("<ab: terms> x"), "<ab: terms> x");
        let markdown = "First page text.\n\n<https://example.com/terms>\n\nMore.\n";
        let pages = ["First page text.", "https://example.com/terms\nMore."];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\nFirst page text.\n\n<!-- page 2 of 2 -->\n\n<https://example.com/terms>\n\nMore.\n"
        );
    }

    /// Word text in angle brackets reaches the Markdown as it is (the writer
    /// escapes only CriticMarkup), and the page paints it: only HTML the
    /// writer or an author could mean (`<sup>`, `<br>`, comments) is unpainted.
    #[test]
    fn literal_angle_bracket_text_keys_its_block() {
        assert_eq!(line_text("<Draft> Agreement"), "<Draft> Agreement");
        assert_eq!(line_text("x<sup>2</sup> <!-- note --> y"), "x2  y");
        let markdown = "Intro.\n\n<Draft> Agreement opens page two.\n\nEnd.\n";
        let pages = ["Intro.", "<Draft> Agreement opens page two.\nEnd."];
        assert_eq!(
            paginate(markdown, &pages),
            "<!-- page 1 of 2 -->\n\nIntro.\n\n<!-- page 2 of 2 -->\n\n<Draft> Agreement opens page two.\n\nEnd.\n"
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
