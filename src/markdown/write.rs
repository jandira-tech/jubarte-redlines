// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Markdown events to Word items, CriticMarkup to tracked changes.
//!
//! The internal pipeline that
//! [`markdown_to_docx`](super::markdown_to_docx) and the redline task
//! use to convert one Markdown document to a Word document package.
//! Backs `jubarte convert FILE.md`, the Markdown side of
//! [`crate::markdown::redline`], and the Python / WASM
//! `markdown_to_docx` binding.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use pulldown_cmark::{
    Alignment, CodeBlockKind, Event, HeadingLevel, LinkType, Options, Parser, Tag, TagEnd,
};
use unicode_properties::{GeneralCategory, GeneralCategoryGroup, UnicodeGeneralCategory};

use super::critic::{self, Piece, Pieces, Token};
use super::diff::{continues, marker_len, table_row, table_row_change};
use super::xml::{
    Comment, Content, Document, FormatMark, Indent, Item, Kind, Label, LabelUse, Link, List, Note,
    Paragraph, Picture, Run, RunFormat, TableFormats, TextBox,
};
use super::{DocxOptions, MarkdownError, PageSize, TrackChanges, WrittenDocx, package};
use super::{math, view_notes, view_stories, view_styles};

/// Writes Markdown as a `.docx`: CommonMark with GitHub's tables,
/// strikethrough, task lists and footnotes, and CriticMarkup as tracked
/// changes and comments (see the [module docs](super)). Backs
/// `jubarte convert FILE.md` and the redline task's Markdown side.
///
/// # Errors
///
/// [`MarkdownError::Reference`] when the reference document is not a
/// readable `.docx`. [`MarkdownError::Package`] when the package
/// cannot be assembled, or when `track_changes` is `Accept` or
/// `Reject` and accepting or rejecting every revision of the freshly
/// written package fails.
///
/// # Side effects
///
/// None on disk. The `docx` bytes of the [`WrittenDocx`] are returned
/// to the caller; everything else lives only on the stack.
///
/// ```
/// use jubarte::markdown::{DocxOptions, markdown_to_docx};
///
/// let written = markdown_to_docx(
///     "# Terms\n\nPayment is due in {~~30~>45~~} days.",
///     &DocxOptions::default(),
/// )
/// .unwrap();
/// assert!(written.docx.starts_with(b"PK"));
/// ```
pub fn markdown_to_docx(
    markdown: &str,
    options: &DocxOptions<'_>,
) -> Result<WrittenDocx, MarkdownError> {
    // A comment or change that names no author or date takes the file's
    // own (`author:`, `last_modified:`), before the options' defaults.
    let defaults = view_notes::file_defaults(markdown);
    let mut own = options.clone();
    if let Some(author) = defaults.author.filter(|_| !options.author_fixed) {
        own.author = author;
    }
    if let Some(date) = defaults.date.clone().filter(|_| !options.date_fixed) {
        own.date = date;
    }
    let options = &own;
    // A date the caller gave must be one Word takes as `w:date`.
    if !is_date(&options.date) {
        return Err(MarkdownError::Package(format!(
            "date {:?} is no timestamp such as 2026-10-01T09:00:00Z",
            options.date
        )));
    }
    let document = read(markdown, options);
    let mut warnings = document.warnings.clone();
    // Only when no other date key gave the date.
    if let Some((key, value)) = defaults.unread.filter(|_| defaults.date.is_none()) {
        warnings.push(format!(
            "{key}: {value} is no timestamp (YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DD): \
             undated comments and changes keep the default date"
        ));
    }
    if options.reference.is_some() && options.page != PageSize::default() {
        warnings.push(format!(
            "page size {} ignored: the reference document's page setup is used",
            options.page.as_str()
        ));
    }
    let docx = package::assemble(&document, options)?;
    let docx = match options.track_changes {
        _ if !options.critic => docx,
        TrackChanges::All => docx,
        TrackChanges::Accept => crate::document_comparer::accept_revisions(&docx)
            .map_err(|e| MarkdownError::Package(e.to_string()))?,
        TrackChanges::Reject => crate::document_comparer::reject_revisions(&docx)
            .map_err(|e| MarkdownError::Package(e.to_string()))?,
    };
    let docx = match options.tracking.or(document.tracking) {
        Some(on) => crate::settings::apply_settings_to_docx(
            &docx,
            &crate::settings::SettingsRequest {
                track_revisions: Some(on),
                ..Default::default()
            },
        )
        .map_err(MarkdownError::Package)?,
        None => docx,
    };
    let docx = if options.scholar {
        crate::settings::no_stretched_line_breaks(&docx).map_err(MarkdownError::Package)?
    } else {
        docx
    };
    Ok(WrittenDocx { docx, warnings })
}

/// The Markdown read into Word items.
pub(super) fn read(markdown: &str, options: &DocxOptions<'_>) -> Document {
    let scholarly;
    let markdown = if options.scholar {
        let stories = options.reference.is_some_and(super::package::has_stories);
        scholarly = scholar(markdown, stories);
        scholarly.as_str()
    } else {
        markdown
    };
    let (source, critic_warnings) = if options.critic {
        critic::encode_checked(&block_changes(markdown))
    } else {
        (markdown.to_string(), Vec::new())
    };
    let source = page_commands(&source);
    let source = table_pipes(&source);
    let mut writer = Writer::new(options);
    writer.tables = table_plans(&source);
    writer.note_breaks = note_breaks(&source);
    writer.item_levels = item_levels(&source);
    writer.label_definitions = label_definitions(&source);
    let parser = Parser::new_ext(&source, parser_options());
    writer.document.drawing_defs = parser
        .reference_definitions()
        .iter()
        .map(|(name, def)| (name.to_string(), def.dest.to_string()))
        .collect();
    writer.document.drawing_backings = super::from_docx::drawing::backing_comments(&source);
    for event in parser {
        writer.event(event);
    }
    let mut document = writer.finish();
    document.warnings.splice(..0, critic_warnings);
    document
}

/// CriticMarkup changes that hold whole blocks, rewritten so the Markdown
/// reader still sees the blocks: a change that opens a line hands its marks
/// to the blocks inside it — a deleted heading `# {--B--}` reads as a
/// heading whose text, and paragraph mark, are deleted, where `{--# B`,
/// the marks in front of the marker, reads as the plain paragraph `{--# B`.
/// A substitution whose sides hold blocks becomes its deleted blocks, then
/// its inserted ones. A change that opens inside a line is inline text and
/// stays; the diff writes whole blocks as changes that open a line.
fn block_changes(source: &str) -> String {
    let regions = critic::change_regions(source);
    let mut out = String::with_capacity(source.len());
    let mut at = 0;
    for region in regions {
        out.push_str(&source[at..region.open]);
        at = region.close;
        // Only a change that opens a line holds blocks; one that opens
        // inside a line is inline text and stays.
        let starts_line = region.open == 0 || source[..region.open].ends_with('\n');
        let after = &source[region.close..];
        let rewritten = if !starts_line {
            None
        } else if let Some(mid) = region.split {
            substitution_blocks(
                &source[region.body.start..mid],
                &source[mid + "~>".len()..region.body.end],
                after.is_empty() || after.starts_with('\n'),
            )
        } else {
            let (open, close) = if region.inserted {
                ("{++", "++}")
            } else {
                ("{--", "--}")
            };
            // The closer ends a line, or begins one: the blocks end where
            // a line does either way.
            let ends_line = after.is_empty()
                || after.starts_with('\n')
                || source[..region.body.end].ends_with('\n');
            let content = &source[region.body.clone()];
            (ends_line && blocky(content))
                .then(|| distribute(content, open, close))
                .flatten()
        };
        match rewritten {
            Some(text) => out.push_str(&text),
            None => out.push_str(&source[region.open..region.close]),
        }
    }
    out.push_str(&source[at..]);
    out
}

/// A substitution whose sides hold blocks, as the deleted side's blocks
/// then the inserted side's. `ends_line`: the substitution's closer ends
/// its line, so no text after it belongs to a side.
fn substitution_blocks(old: &str, new: &str, ends_line: bool) -> Option<String> {
    if !ends_line || !(blocky(old) || blocky(new)) {
        return None;
    }
    let (old, new) = (distribute(old, "{--", "--}"), distribute(new, "{++", "++}"));
    Some(format!("{}\n{}", old?, new?))
}

/// Whether any line of the text begins a block a change's marks would
/// hide: it has a marker, or is a table row or a code fence.
fn blocky(content: &str) -> bool {
    content.lines().any(|line| {
        let fenced = ["```", "~~~"]
            .iter()
            .any(|fence| line.trim_start().starts_with(fence));
        fenced || marker_len(line) > 0 || table_row(line)
    })
}

/// The text of a change handed to the blocks inside it, the marks around
/// each block: a heading or a list item keeps its marker and changes its
/// text, a quote run opens after its first marker and closes on its last
/// line, a table row marks its cells, and lines that continue one another
/// share one change across their line breaks. `None` when a block cannot
/// hold the marks: a code fence, whose text is literal, or a block whose
/// first line is marker and nothing else.
fn distribute(content: &str, open: &str, close: &str) -> Option<String> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut out = String::with_capacity(content.len());
    let mut at = 0;
    while at < lines.len() {
        let line = lines[at];
        if line.trim().is_empty() {
            out.push_str(line);
            at += 1;
        } else if table_row(line) {
            out.push_str(&table_row_change(line, open, close));
            at += 1;
        } else {
            let marker = marker_len(line);
            let length = if line.trim_start().starts_with('>') {
                // A quote run: every quoted line, the marks across their
                // line breaks.
                1 + lines[at + 1..]
                    .iter()
                    .take_while(|l| l.trim_start().starts_with('>'))
                    .count()
            } else if line[..marker].contains('#') {
                // A heading stands alone: the lines after it are not its.
                1
            } else {
                1 + lines[at + 1..].iter().take_while(|l| continues(l)).count()
            };
            if length == 1 && line[marker..].is_empty() {
                return None;
            }
            out.push_str(&line[..marker]);
            out.push_str(open);
            out.push_str(&line[marker..]);
            for block_line in &lines[at + 1..at + length] {
                out.push('\n');
                out.push_str(block_line);
            }
            out.push_str(close);
            at += length;
        }
        if at < lines.len() {
            out.push('\n');
        }
    }
    Some(out)
}

/// The styles [`DocxOptions::scholar`] sets, as the front matter's
/// `styles:` legend writes them: LaTeX's article class at 10pt (`\section`
/// `\Large`, `\subsection` `\large`, `\subsubsection` `\normalsize`,
/// with their spacing before and after).
const SCHOLAR_STYLES: [(&str, &str); 4] = [
    (
        "Normal",
        "Latin Modern Roman 10pt, after 6pt, line 12pt exact, justify",
    ),
    (
        "\"#\"",
        "Heading1, Latin Modern Roman bold 14.4pt, before 15pt, after 10pt, line 1, left, keep-next",
    ),
    (
        "\"##\"",
        "Heading2, Latin Modern Roman bold 12pt, before 14pt, after 6.5pt, line 1, left, keep-next",
    ),
    (
        "\"###\"",
        "Heading3, Latin Modern Roman bold 10pt, before 14pt, after 6.5pt, line 1, left, keep-next",
    ),
];

/// `markdown` with the front matter [`DocxOptions::scholar`] fills in: each
/// style of [`SCHOLAR_STYLES`] its `styles:` legend does not name (`Normal`
/// only when the legend names no default style of its own), and a footer
/// with the page number, centered, from page 2 when it has no footers and
/// no header holds `{PAGE}`, and the reference document (`stories`) has
/// no headers or footers of its own. With headers of its own, page 1 keeps
/// them, so the number starts on page 1.
fn scholar(markdown: &str, stories: bool) -> String {
    let (yaml, body) = split_front_matter(markdown).unwrap_or(("", markdown));
    let mut lines: Vec<String> = yaml.lines().map(str::to_string).collect();
    let mut named: Vec<String> = Vec::new();
    let mut styles_at = None;
    // A block holds a story only with an entry under it; only a story's
    // `{PAGE}` is the page number.
    let mut headers = false;
    let mut footers = false;
    let mut page = false;
    let mut block = "";
    for (n, line) in lines.iter().enumerate() {
        if !line.starts_with(' ') {
            block = line.trim_end();
            if block == "styles:" {
                styles_at = Some(n);
            }
            continue;
        }
        let entry = view_styles::without_comment(line).trim();
        let Some((key, _)) = entry.split_once(": ") else {
            continue;
        };
        match block {
            "styles:" => named.push(key.trim_matches('"').to_string()),
            "headers:" => headers = true,
            "footers:" => footers = true,
            _ => {}
        }
        page |= matches!(block, "headers:" | "footers:") && line.contains("{PAGE");
    }
    let own_default = named.iter().any(|key| {
        !key.bytes().all(|b| b == b'#') && !matches!(key.as_str(), "table" | "tables" | "changes")
    });
    let missing: Vec<String> = SCHOLAR_STYLES
        .iter()
        .filter(|(key, _)| {
            let key = key.trim_matches('"');
            if key == "Normal" {
                !own_default
            } else {
                !named.iter().any(|n| n == key)
            }
        })
        .map(|(key, value)| format!("  {key}: {value}"))
        .collect();
    match styles_at {
        Some(at) => {
            lines.splice(at + 1..at + 1, missing);
        }
        None if !missing.is_empty() => {
            lines.push("styles:".to_string());
            lines.extend(missing);
        }
        None => {}
    }
    if !footers && !page && !stories {
        lines.push("footers:".to_string());
        if !headers {
            lines.push("  first: none".to_string());
        }
        lines.push("  default: {id: footer1, text: \"{PAGE}\", center}".to_string());
    }
    format!("---\n{}\n---\n\n{body}", lines.join("\n"))
}

/// A YAML front matter block's text and the Markdown after it.
fn split_front_matter(markdown: &str) -> Option<(&str, &str)> {
    let rest = markdown
        .strip_prefix("---\n")
        .or_else(|| markdown.strip_prefix("---\r\n"))?;
    let mut at = 0;
    for line in rest.split_inclusive('\n') {
        if matches!(line.trim_end(), "---" | "...") {
            return Some((&rest[..at], &rest[at + line.len()..]));
        }
        at += line.len();
    }
    None
}

/// The line the writers print for a page break where no id line can carry
/// it (the plain Markdown of `convert --to md`); the reader takes it back.
pub(super) const PAGE_BREAK_LINE: &str = "<!-- page-break -->";

/// LaTeX's page commands, which Pandoc manuscripts write on a line of their
/// own: each is a page break, so `source` gets [`PAGE_BREAK_LINE`] in its
/// place (an HTML comment ends the paragraph above it). Front matter and
/// fenced or indented code keep them as text.
fn page_commands(source: &str) -> String {
    const COMMANDS: [&str; 4] = [
        r"\newpage",
        r"\pagebreak",
        r"\clearpage",
        r"\cleardoublepage",
    ];
    let mut out = String::with_capacity(source.len());
    let mut fence: Option<&str> = None;
    let mut front_matter = source.starts_with("---\n") || source.starts_with("---\r\n");
    for (n, line) in source.split_inclusive('\n').enumerate() {
        let bare = line.trim_end_matches(['\n', '\r']);
        let trimmed = bare.trim();
        if front_matter {
            if n > 0 && (trimmed == "---" || trimmed == "...") {
                front_matter = false;
            }
            out.push_str(line);
            continue;
        }
        let indent = bare.len() - bare.trim_start().len();
        if let Some(open) = fence {
            if indent < 4
                && trimmed.starts_with(open)
                && trimmed.trim_start_matches(open).trim().is_empty()
            {
                fence = None;
            }
            out.push_str(line);
            continue;
        }
        if indent < 4 && (trimmed.starts_with("```") || trimmed.starts_with("~~~")) {
            fence = Some(&trimmed[..3]);
        } else if indent < 4 && COMMANDS.contains(&trimmed) {
            out.push_str(PAGE_BREAK_LINE);
            out.push_str(&line[bare.len()..]);
            if line.len() == bare.len() {
                out.push('\n');
            }
            continue;
        }
        out.push_str(line);
    }
    out
}

/// GFM reads an even backslash run before a table row's pipe (`\\|`) as
/// escaped backslashes and then the cell separator, but pulldown-cmark
/// shields a pipe behind any backslash, so the rows the reader writes
/// (`|\\|\\\\|` for the cells `\` and `\\`) and hand-written ones lose
/// cells, or the table with them. A space moves such a run off its pipe —
/// a cell's surrounding spaces are not its text — in the rows of tables
/// whose columns then match their delimiter row, and nowhere else: front
/// matter and fenced code keep every line, and a line a space would
/// change without making the table is left as it was.
fn table_pipes(source: &str) -> String {
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut out = String::with_capacity(source.len());
    let mut fence: Option<&str> = None;
    let mut front_matter = source.starts_with("---\n") || source.starts_with("---\r\n");
    let mut in_table = false;
    for (n, line) in lines.iter().enumerate() {
        let bare = line.trim_end_matches(['\n', '\r']);
        let trimmed = bare.trim();
        if front_matter {
            if n > 0 && (trimmed == "---" || trimmed == "...") {
                front_matter = false;
            }
            out.push_str(line);
            continue;
        }
        let indent = bare.len() - bare.trim_start().len();
        if let Some(open) = fence {
            if indent < 4
                && trimmed.starts_with(open)
                && trimmed.trim_start_matches(open).trim().is_empty()
            {
                fence = None;
            }
            out.push_str(line);
            continue;
        }
        if indent < 4 && (trimmed.starts_with("```") || trimmed.starts_with("~~~")) {
            fence = Some(&trimmed[..3]);
            in_table = false;
            out.push_str(line);
            continue;
        }
        let row = indent < 4 && trimmed.starts_with('|');
        if !row {
            in_table = false;
            out.push_str(line);
            continue;
        }
        let unshielded = unshield_pipes(bare);
        if in_table || unshielded == bare {
            out.push_str(&unshielded);
            out.push_str(&line[bare.len()..]);
            continue;
        }
        // The row opens a table only when the columns it names, with the
        // runs moved off their pipes, are the delimiter row's.
        in_table = lines.get(n + 1).is_some_and(|next| {
            let next = next.trim_end_matches(['\n', '\r']);
            let columns = next.len() - next.trim_start().len();
            columns < 4 && delimiter_row(next)
        }) && row_columns(&unshielded)
            == lines
                .get(n + 1)
                .map(|next| row_columns(next.trim_end_matches(['\n', '\r'])))
                .unwrap_or(0);
        out.push_str(if in_table { &unshielded } else { bare });
        out.push_str(&line[bare.len()..]);
    }
    out
}

/// `line` with every even backslash run before a pipe moved off it by a
/// space. Code spans, equations and HTML tags are skipped: a pipe inside
/// them is part of the cell, backslashes or not, and a space would change
/// their text.
fn unshield_pipes(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    // The backslash run before the byte at `at`.
    let mut run = 0usize;
    let mut at = 0usize;
    while at < bytes.len() {
        let rest = &line[at..];
        let c = rest.chars().next().unwrap_or_default();
        match c {
            '\\' => {
                run += 1;
                out.push('\\');
                at += 1;
            }
            '`' if run.is_multiple_of(2) => {
                let ticks = rest.chars().take_while(|&c| c == '`').count();
                let close = rest[ticks..]
                    .char_indices()
                    .find(|(i, _)| {
                        rest[ticks + i..].starts_with(&"`".repeat(ticks))
                            && rest[ticks + i + ticks..]
                                .chars()
                                .next()
                                .is_none_or(|c| c != '`')
                    })
                    .map(|(i, _)| ticks + i + ticks);
                let end = close.unwrap_or(ticks);
                out.push_str(&rest[..end]);
                at += end;
                run = 0;
            }
            '$' if run.is_multiple_of(2) => {
                let fence = usize::from(rest.starts_with("$$")) + 1;
                let mut close = None;
                let mut i = at + fence;
                while i < bytes.len() {
                    let step = line[i..].chars().next().map_or(1, char::len_utf8);
                    match bytes[i] {
                        // A backslash escapes the character after it, as an
                        // equation's closing `$` is found.
                        b'\\' => i += 1 + line[i + 1..].chars().next().map_or(1, char::len_utf8),
                        b'$' if line[i..].starts_with(&"$".repeat(fence)) => {
                            close = Some(i + fence);
                            break;
                        }
                        _ => i += step,
                    }
                }
                // An unclosed `$` is text, as pulldown-cmark reads it.
                let end = close.unwrap_or(at + 1);
                out.push_str(&line[at..end]);
                at = end;
                run = 0;
            }
            '<' if run.is_multiple_of(2)
                && rest
                    .chars()
                    .nth(1)
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '/') =>
            {
                let end = rest
                    .find('>')
                    .filter(|&close| html_tag(&rest[..=close]).is_some())
                    .map_or(1, |close| close + 1);
                out.push_str(&rest[..end]);
                at += end;
                run = 0;
            }
            '|' => {
                if run >= 2 && run.is_multiple_of(2) {
                    out.push(' ');
                }
                out.push('|');
                at += 1;
                run = 0;
            }
            _ => {
                out.push(c);
                at += c.len_utf8();
                run = 0;
            }
        }
    }
    out
}

/// Whether `line` is a pipe table's delimiter row (`|---|:-:|`), every
/// column `:`-padded hyphens.
fn delimiter_row(line: &str) -> bool {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line.strip_suffix('|').unwrap_or(line);
    line.split('|').all(|cell| {
        let cell = cell.trim();
        let cell = cell.strip_prefix(':').unwrap_or(cell);
        let cell = cell.strip_suffix(':').unwrap_or(cell);
        !cell.is_empty() && cell.bytes().all(|b| b == b'-')
    })
}

/// The columns a table row names, its pipes counted the GFM way: an odd
/// backslash run before a pipe escapes it, an even one is backslashes the
/// row's cells hold.
fn row_columns(line: &str) -> usize {
    let line = line.trim_end();
    let bytes = line.as_bytes();
    let (mut pipes, mut run, mut last_pipe) = (0usize, 0usize, None);
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'\\' => run += 1,
            b'|' => {
                if run.is_multiple_of(2) {
                    pipes += 1;
                    last_pipe = Some(i);
                }
                run = 0;
            }
            _ => run = 0,
        }
    }
    if line.starts_with('|') {
        pipes -= 1;
    }
    match last_pipe {
        Some(i) if line[i + 1..].trim().is_empty() => pipes,
        _ => pipes + 1,
    }
}

/// What the view's table line says about one table, read before the table:
/// whether its first row repeats, its grid's columns, and how each row's
/// Markdown cells and `<br>`s map onto its cells and paragraphs.
#[derive(Clone, Debug, Default)]
struct TablePlan {
    header_repeats: bool,
    columns: Option<usize>,
    align: Option<&'static str>,
    rows: Vec<RowPlan>,
    /// The cell paragraphs' mark changes, by paragraph number.
    marks: HashMap<usize, (bool, view_notes::Tag)>,
    /// Cell paragraphs aligned against their column, by paragraph number.
    aligns: HashMap<usize, &'static str>,
    /// The style the table's line names (`style Ledger`).
    style: Option<String>,
    /// The table's own borders and its cells' (`borders …`, `cell-borders …`).
    borders: Option<crate::markdown::borders::Sides>,
    cell_borders: Vec<(view_notes::CellsAt, crate::markdown::borders::Sides)>,
    row_borders: Vec<(usize, crate::markdown::borders::Sides)>,
    /// Cell paragraphs in a style of their own (`p5 ListParagraph`).
    styles: HashMap<usize, String>,
    /// A comment's range runs through the table (`in c5`).
    in_comment: bool,
    /// The formatting changes the line names.
    fmt: Option<view_notes::Tag>,
    grid_fmt: Option<u32>,
    /// These four are `BTreeMap`s: `table_formats` walks them calling
    /// `format_mark`, and the first undated mark of an author dated
    /// `first..last` takes the range's last day — a `HashMap`'s random order
    /// made that winner, and the docx, different per process (#534).
    row_fmt: BTreeMap<usize, view_notes::Tag>,
    cell_fmt: BTreeMap<(usize, usize), view_notes::Tag>,
    /// The cell paragraphs' formatting changes, by paragraph number.
    para_fmt: HashMap<usize, Vec<view_notes::Tag>>,
    /// Rows' and cells' own changes the text does not show.
    row_marks: BTreeMap<usize, (bool, view_notes::Tag)>,
    cell_marks: BTreeMap<(usize, usize), (view_notes::CellMark, view_notes::Tag)>,
}

/// One row of a [`TablePlan`].
#[derive(Clone, Debug, Default)]
struct RowPlan {
    /// Whether the row's `<br>`s end paragraphs.
    breaks: bool,
    /// The row's first paragraph's number, when its paragraphs are known
    /// one by one.
    first: Option<usize>,
    /// Each Markdown cell's shape, in order.
    cells: Vec<CellShape>,
    /// The empty cells the view left out at the row's end.
    pad: Vec<Pad>,
    /// The grid columns the row leaves out before and after its cells.
    gaps: (usize, usize),
}

/// What one Markdown cell of a planned row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellShape {
    /// A cell spanning `across` grid columns, in a vertical merge or not
    /// (as [`Item::Cell`]).
    Cell { across: usize, merge: Option<bool> },
    /// A grid column the cell before spans: the view prints it as an empty
    /// cell, which is no cell of its own.
    Spanned,
}

/// For each footnote definition of `source`, in order, the paragraphs its
/// id line counts (`<!-- footnotes.p2-p3 -->` is two) and the `<br>`s it
/// holds: when they add up, every `<br>` ends a paragraph, as a table
/// row's do.
fn note_breaks(source: &str) -> Vec<(usize, usize)> {
    let mut breaks = Vec::new();
    let mut html = String::new();
    // The open definition's paragraphs, as its id line counts them, and
    // its `<br>`s.
    let mut open: Option<(usize, usize)> = None;
    for event in Parser::new_ext(source, parser_options()) {
        match event {
            Event::Start(Tag::HtmlBlock) => html.clear(),
            Event::Html(text) => html.push_str(&text),
            Event::Start(Tag::FootnoteDefinition(_)) => {
                let counted = html
                    .lines()
                    .rev()
                    .find_map(view_notes::note_paragraphs)
                    .unwrap_or(0);
                open = Some((counted, 0));
                html.clear();
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                if let Some(counts) = open.take() {
                    breaks.push(counts);
                }
            }
            Event::InlineHtml(text) if is_paragraph_break(&text) => {
                if let Some((_, lines)) = &mut open {
                    *lines += 1;
                }
            }
            Event::Start(_) if open.is_none() => html.clear(),
            _ => {}
        }
    }
    breaks
}

/// For each list item of `source`, in order, its level by its marker's
/// column: one deeper than an open item whose text column it reaches, as
/// CommonMark nests an item. An id line between items ends the parser's
/// list but not this nesting; any other block does. `None` for an item in
/// a quote, a note or a table, which keeps the parser's nesting.
fn item_levels(source: &str) -> Vec<Option<u32>> {
    let mut levels = Vec::new();
    // The open items' marker and text columns, outermost first.
    let mut open: Vec<(usize, usize)> = Vec::new();
    // Lists open, and quotes, notes and tables open.
    let (mut lists, mut aside) = (0usize, 0usize);
    for (event, range) in Parser::new_ext(source, parser_options()).into_offset_iter() {
        let block = matches!(event, Event::Rule)
            || matches!(&event, Event::Start(tag) if !matches!(tag, Tag::List(_) | Tag::HtmlBlock));
        if block && lists == 0 && aside == 0 {
            open.clear();
        }
        match event {
            Event::Start(Tag::List(_)) => lists += 1,
            Event::End(TagEnd::List(_)) => lists -= 1,
            Event::Start(Tag::BlockQuote(_) | Tag::FootnoteDefinition(_) | Tag::Table(_)) => {
                aside += 1;
            }
            Event::End(TagEnd::BlockQuote(_) | TagEnd::FootnoteDefinition | TagEnd::Table) => {
                aside -= 1;
            }
            Event::Start(Tag::Item) if aside > 0 => levels.push(None),
            Event::Start(Tag::Item) => {
                let (marker, text) = item_columns(source, range.start);
                while open.last().is_some_and(|&(_, column)| marker < column) {
                    open.pop();
                }
                levels.push(u32::try_from(open.len()).ok().map(|level| level.min(8)));
                open.push((marker, text));
            }
            _ => {}
        }
    }
    levels
}

/// The columns of a list item's marker and of its text, for the item that
/// starts at `at` (at its marker or at the indentation before it): a tab
/// stops at the next multiple of 4, and text five or more columns past the
/// marker, or none, starts one column past it.
fn item_columns(source: &str, at: usize) -> (usize, usize) {
    let start = source[..at].rfind('\n').map_or(0, |line| line + 1);
    let end = source[at..]
        .find('\n')
        .map_or(source.len(), |line| at + line);
    let mut column = 0;
    let mut marker = None;
    let mut chars = source[start..end].char_indices().peekable();
    while let Some((offset, c)) = chars.next() {
        let past = start + offset >= at;
        match c {
            '\t' => column += 4 - column % 4,
            ' ' => column += 1,
            '-' | '*' | '+' if past => {
                marker = Some(column);
                column += 1;
                break;
            }
            '0'..='9' if past => {
                marker = Some(column);
                column += 1;
                while chars.next_if(|(_, c)| c.is_ascii_digit()).is_some() {
                    column += 1;
                }
                if chars.next_if(|(_, c)| matches!(c, '.' | ')')).is_some() {
                    column += 1;
                }
                break;
            }
            _ => column += 1,
        }
    }
    let Some(marker) = marker else {
        return (column, column);
    };
    let after = column;
    for (_, c) in chars {
        match c {
            ' ' => column += 1,
            '\t' => column += 4 - column % 4,
            _ => {
                return (
                    marker,
                    if column - after > 4 {
                        after + 1
                    } else {
                        column
                    },
                );
            }
        }
    }
    (marker, after + 1)
}

/// How many definitions each footnote label has in `source`: a footnote
/// and an endnote may share one (`[^1]`).
fn label_definitions(source: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for event in Parser::new_ext(source, parser_options()) {
        if let Event::Start(Tag::FootnoteDefinition(label)) = event {
            *counts.entry(note_label(&label)).or_insert(0) += 1;
        }
    }
    counts
}

/// A footnote label as the parser matches it: decoded, and the same
/// whatever its case (`[^Note]` names `[^note]:`).
fn note_label(label: &str) -> String {
    critic::decode(label).to_lowercase()
}

/// The key of the `nth` note labelled `label` (from 0): the label itself
/// for the first, a name no Markdown label takes for the next.
fn nth_label(label: &str, nth: usize) -> String {
    if nth == 0 {
        label.to_string()
    } else {
        format!("{label}\u{0}{nth}")
    }
}

/// A plan for each table of `source`, in order; `None` for a table no table
/// line precedes. A row's `<br>`s end paragraphs when its cells and `<br>`s
/// add up to the paragraphs its table line counts: each cell holds at least
/// one paragraph and each line break or comment `<br>` adds none, so only
/// then is every `<br>` a paragraph's end.
fn table_plans(source: &str) -> Vec<Option<TablePlan>> {
    let mut plans = Vec::new();
    let mut html = String::new();
    let mut line = None;
    // Each row of the open table: whether each cell is empty, and its
    // `<br>`s.
    let mut rows: Vec<(Vec<bool>, usize)> = Vec::new();
    let mut in_cell = false;
    for event in Parser::new_ext(source, parser_options()) {
        match event {
            Event::Start(Tag::HtmlBlock) => html.clear(),
            Event::Html(text) => html.push_str(&text),
            Event::Start(Tag::Table(_)) => {
                line = html.lines().rev().find_map(view_notes::table_line);
                html.clear();
                rows.clear();
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => rows.push((Vec::new(), 0)),
            Event::Start(Tag::TableCell) => {
                in_cell = true;
                if let Some(row) = rows.last_mut() {
                    row.0.push(true);
                }
            }
            Event::End(TagEnd::TableCell) => in_cell = false,
            Event::End(TagEnd::Table) => {
                plans.push(line.take().map(|line| plan_table(line, &rows)));
            }
            Event::Start(_) if !in_cell => html.clear(),
            event if in_cell => {
                if let Some(row) = rows.last_mut() {
                    if let Event::InlineHtml(text) = &event
                        && is_paragraph_break(text)
                    {
                        row.1 += 1;
                    }
                    if let Some(empty) = row.0.last_mut() {
                        *empty = false;
                    }
                }
            }
            _ => {}
        }
    }
    plans
}

/// One empty cell the view left out at a row's end: the grid columns it
/// spans, and its vertical merge (as [`Item::Cell`]).
type Pad = (usize, Option<bool>);

/// The plan a table line makes for a table whose rows hold `rows` (each
/// Markdown cell's emptiness, and the row's `<br>`s).
fn plan_table(line: view_notes::TableLine, rows: &[(Vec<bool>, usize)]) -> TablePlan {
    // The last row each grid column's vertical merge reaches.
    let mut merged_until: HashMap<usize, usize> = HashMap::new();
    let mut shaped: Vec<(Vec<CellShape>, usize, Vec<Pad>)> = Vec::new();
    let mut gaps: Vec<(usize, usize)> = Vec::new();
    for (r, (cells, _)) in rows.iter().enumerate() {
        let mut shapes = Vec::with_capacity(cells.len());
        // The empty cells that stand for grid columns the row leaves out.
        let (before, after) = line.gaps.get(&r).copied().unwrap_or((0, 0));
        let lead = cells
            .iter()
            .take(before)
            .take_while(|&&empty| empty)
            .count();
        // Empty cells past the grid's columns pad the row to the view's
        // widest: they are no cells.
        let mut len = cells.len();
        if let Some(columns) = line.columns {
            while len > lead && before + (len - lead) > columns && cells[len - 1] {
                len -= 1;
            }
        }
        let tail = cells[..len]
            .iter()
            .skip(lead)
            .rev()
            .take(after)
            .take_while(|&&empty| empty)
            .count();
        // The gap before is the line's, also where the view shows fewer
        // empty cells for it (a row of empty cells shows one).
        let (mut cell, mut column, mut spanned) = (0, before, 0);
        for (at, &empty) in cells.iter().enumerate() {
            if at < lead || at >= len - tail {
                shapes.push(CellShape::Spanned);
                continue;
            }
            if spanned > 0 && empty {
                spanned -= 1;
                shapes.push(CellShape::Spanned);
                continue;
            }
            let (across, down) = line.merges.get(&(r, cell)).copied().unwrap_or((1, 1));
            let merge = if down > 1 {
                merged_until.insert(column, r + down - 1);
                Some(true)
            } else {
                merged_until
                    .get(&column)
                    .is_some_and(|&until| until >= r)
                    .then_some(false)
            };
            shapes.push(CellShape::Cell { across, merge });
            spanned = across - 1;
            column += across;
            cell += 1;
        }
        // `column` already counts a span the view cut at the right edge.
        // The view leaves out empty cells at a row's end: they come back,
        // but for the grid columns its `after` gap names past them.
        // The gap after is the line's too: the cells it leaves out of the
        // grid come back, each with its span and vertical merge as the
        // table line names them.
        let end = line
            .columns
            .map_or(0, |columns| columns.saturating_sub(after.max(tail)));
        let mut pad = Vec::new();
        let mut at = cell;
        while column < end {
            // A merge the line names keeps its span, also past the room
            // the gap after leaves: the row had it so.
            let (across, down) = line.merges.get(&(r, at)).copied().unwrap_or((1, 1));
            let across = across.max(1);
            let merge = if down > 1 {
                merged_until.insert(column, r + down - 1);
                Some(true)
            } else {
                merged_until
                    .get(&column)
                    .is_some_and(|&until| until >= r)
                    .then_some(false)
            };
            pad.push((across, merge));
            column += across;
            at += 1;
        }
        shaped.push((shapes, cell, pad));
        gaps.push((before, after.max(tail)));
    }
    let rows: Vec<RowPlan> = match (&line.row_paragraphs, line.one_per_cell) {
        // One paragraph a cell: a `<br>` is a line break.
        (_, Some(first)) => {
            let mut at = first;
            shaped
                .into_iter()
                .map(|(cells, count, pad)| {
                    at = at.saturating_add(count + pad.len());
                    RowPlan {
                        breaks: false,
                        first: Some(at - count - pad.len()),
                        cells,
                        pad,
                        gaps: (0, 0),
                    }
                })
                .collect()
        }
        (Some(counts), None) if counts.len() == rows.len() => counts
            .iter()
            .zip(rows)
            .zip(shaped)
            .map(|((&(first, n), &(_, breaks)), (cells, count, pad))| {
                // The view leaves out empty cells only at a row's end. A
                // row of more cells than its grid has (`cells t0.row1
                // p4-p7`) names them by its count alone: they come back
                // empty.
                let mut pad = pad;
                if breaks == 0 {
                    while count + pad.len() < n {
                        pad.push((1, None));
                    }
                }
                let pad = if count + pad.len() + breaks == n {
                    pad
                } else {
                    Vec::new()
                };
                let exact = count + pad.len() + breaks == n;
                RowPlan {
                    breaks: exact && breaks > 0,
                    first: exact.then_some(first),
                    cells,
                    pad,
                    gaps: (0, 0),
                }
            })
            .collect(),
        _ => shaped
            .into_iter()
            .map(|(cells, _, _)| RowPlan {
                cells,
                ..RowPlan::default()
            })
            .collect(),
    };
    let rows: Vec<RowPlan> = rows
        .into_iter()
        .zip(gaps.into_iter().chain(std::iter::repeat((0, 0))))
        .map(|(row, gaps)| RowPlan { gaps, ..row })
        .collect();
    TablePlan {
        header_repeats: line.header_repeats,
        columns: line.columns,
        align: line.align,
        rows,
        marks: line.marks,
        aligns: line.aligns,
        styles: line.styles,
        style: line.style,
        borders: line.borders,
        cell_borders: line.cell_borders,
        row_borders: line.row_borders,
        in_comment: line.in_comment,
        fmt: line.fmt,
        grid_fmt: line.grid_fmt,
        row_fmt: line.row_fmt,
        cell_fmt: line.cell_fmt,
        para_fmt: line.para_fmt,
        row_marks: line.row_marks,
        cell_marks: line.cell_marks,
    }
}

/// A `{>>…<<}` in a header's or footer's text that is no change's tag.
fn has_comment(text: &str) -> bool {
    let mut rest = text;
    while let Some(at) = rest.find("{>>") {
        let after = &rest[at + 3..];
        let Some(end) = after.find("<<}") else {
            return false;
        };
        if view_notes::revision_tags(&after[..end]).is_none() {
            return true;
        }
        rest = &after[end + 3..];
    }
    false
}

/// `<br>`, `<br/>` or `<br class="x">`, as the view and authors write a
/// line break.
/// `<br>`, which may end a cell's or a note's paragraph; `<br/>` is always
/// a line break (Word's `w:br`).
fn is_paragraph_break(html: &str) -> bool {
    html_tag(html).is_some_and(|tag| !tag.closing && !tag.empty && tag.name == "br")
}

/// The value of attribute `name` in the tag `html` (`<a href="u">`),
/// quoted either way or bare, with its entities decoded.
fn html_attribute(html: &str, name: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find(name) {
        let at = from + found;
        from = at + name.len();
        let before = lower[..at].chars().next_back();
        if !before.is_some_and(char::is_whitespace) {
            continue;
        }
        let Some(value) = html[from..].trim_start().strip_prefix('=') else {
            continue;
        };
        let value = value.trim_start();
        let raw = match value.chars().next() {
            Some(quote @ ('"' | '\'')) => value[1..].split(quote).next()?,
            _ => value
                .split(|c: char| c.is_whitespace() || c == '>')
                .next()?,
        };
        return Some(crate::convert::altchunk::decode_entities(raw));
    }
    None
}

/// Whether an HTML block is only comments (the view's id lines).
fn comments_only(html: &str) -> bool {
    html.lines()
        .map(str::trim)
        .all(|line| line.is_empty() || (line.starts_with("<!--") && line.ends_with("-->")))
}

/// The elements that start a paragraph of their own in HTML.
const HTML_BLOCKS: [&str; 39] = [
    "address",
    "article",
    "aside",
    "blockquote",
    "br",
    "caption",
    "dd",
    "details",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "th",
    "thead",
    "tr",
    "ul",
];

/// The first tag's name and the text of a raw HTML block: comments, tags
/// and `<script>`/`<style>` bodies dropped, entities decoded, white space
/// collapsed, a paragraph per block element.
fn html_block_text(html: &str) -> (Option<String>, Vec<String>) {
    let mut first = None;
    let mut paragraphs = Vec::new();
    let mut text = String::new();
    let flush = |text: &mut String, paragraphs: &mut Vec<String>| {
        let decoded = crate::convert::altchunk::decode_entities(text);
        let words: Vec<&str> = decoded.split_whitespace().collect();
        if !words.is_empty() {
            paragraphs.push(words.join(" "));
        }
        text.clear();
    };
    let mut rest = html;
    while let Some(lt) = rest.find('<') {
        text.push_str(&rest[..lt]);
        rest = &rest[lt..];
        if let Some(comment) = rest.strip_prefix("<!--") {
            rest = comment.find("-->").map_or("", |end| &comment[end + 3..]);
            continue;
        }
        let Some(gt) = rest.find('>') else {
            break;
        };
        let inner = rest[1..gt].trim();
        rest = &rest[gt + 1..];
        let name: String = inner
            .trim_start_matches('/')
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        if first.is_none() {
            first = Some(name.clone());
        }
        if (name == "script" || name == "style") && !inner.starts_with('/') && !inner.ends_with('/')
        {
            let close = format!("</{name}");
            rest = rest
                .to_ascii_lowercase()
                .find(&close)
                .and_then(|at| rest[at..].find('>').map(|gt| &rest[at + gt + 1..]))
                .unwrap_or("");
            continue;
        }
        if HTML_BLOCKS.contains(&name.as_str()) {
            flush(&mut text, &mut paragraphs);
        }
    }
    text.push_str(rest);
    flush(&mut text, &mut paragraphs);
    (first, paragraphs)
}

/// One inline HTML tag, as CommonMark names it.
struct HtmlTag {
    /// Lowercase, hyphens and all (`code-of-conduct`).
    name: String,
    /// `</b>`.
    closing: bool,
    /// `<b/>`: opens and closes at once.
    empty: bool,
}

/// The tag `html` is, if it is one: a comment, a declaration or a
/// processing instruction is none.
fn html_tag(html: &str) -> Option<HtmlTag> {
    let inner = html.trim().strip_prefix('<')?.strip_suffix('>')?.trim();
    if inner.starts_with(['!', '?']) {
        return None;
    }
    let (closing, inner) = match inner.strip_prefix('/') {
        Some(rest) => (true, rest.trim_start()),
        None => (false, inner),
    };
    let name: String = inner
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect::<String>()
        .to_ascii_lowercase();
    if !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }
    Some(HtmlTag {
        name,
        closing,
        empty: inner.ends_with('/'),
    })
}

/// The start token an end token closes, if it is one.
fn closes(token: Token) -> Option<Token> {
    match token {
        Token::InsertEnd => Some(Token::InsertStart),
        Token::DeleteEnd => Some(Token::DeleteStart),
        Token::SubstituteEnd => Some(Token::SubstituteStart),
        Token::HighlightEnd => Some(Token::HighlightStart),
        Token::CommentEnd => Some(Token::CommentStart),
        Token::MarkEnd => Some(Token::MarkStart),
        _ => None,
    }
}

/// The CriticMarkup tokens in `tokens` that cross the edge of text the
/// writer drops or captures (an HTML comment, image alt text), in the
/// order they were read: the closers of spans opened before it, the
/// openers of spans closed after it. Spans whole inside are dropped with
/// the text. A comment's tokens count as its own only if its `<<}` is
/// there too, because nothing nests in a comment, and a substitution
/// keeps its `~>` when its `~~}` is not there too, or the text after
/// the drop would read as its old half.
fn loose_tokens(tokens: Vec<Token>) -> Vec<Token> {
    // Which opener a closer of the same text closes, so a span whole
    // inside is known before anything replays.
    let mut closed = vec![false; tokens.len()];
    let mut open: Vec<(Token, usize)> = Vec::new();
    for (at, token) in tokens.iter().enumerate() {
        if let Some(opener) = closes(*token) {
            if let Some(index) = open.iter().rposition(|(start, _)| *start == opener) {
                let (_, start) = open.remove(index);
                closed[start] = true;
            }
        } else if matches!(
            token,
            Token::InsertStart
                | Token::DeleteStart
                | Token::SubstituteStart
                | Token::HighlightStart
                | Token::CommentStart
                | Token::MarkStart
        ) {
            open.push((*token, at));
        }
    }
    let mut loose = Vec::new();
    let mut open: Vec<(Token, usize)> = Vec::new();
    for (at, token) in tokens.into_iter().enumerate() {
        if let Some(opener) = closes(token) {
            if let Some(index) = open.iter().rposition(|(start, _)| *start == opener) {
                open.remove(index);
            } else {
                // Its opener is outside: the closer crosses the edge.
                loose.push(token);
            }
            continue;
        }
        match token {
            Token::SubstituteSeparator
                if open.last().is_some_and(|&(start, opened)| {
                    start == Token::SubstituteStart && closed[opened]
                }) => {}
            // A field's marks are part of its text, not a span.
            Token::FieldStart | Token::FieldSeparator | Token::FieldEnd => {}
            _ => {
                if matches!(
                    token,
                    Token::InsertStart
                        | Token::DeleteStart
                        | Token::SubstituteStart
                        | Token::HighlightStart
                        | Token::CommentStart
                        | Token::MarkStart
                ) {
                    open.push((token, at));
                    // An opener nothing in the text closes crosses the
                    // edge; one that closes here drops with the text.
                    if !closed[at] {
                        loose.push(token);
                    }
                } else {
                    loose.push(token);
                }
            }
        }
    }
    loose
}

/// The Markdown extensions jubarte reads, for the writer and for
/// [`super::pages`].
pub(super) fn parser_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_SUPERSCRIPT
        | Options::ENABLE_SUBSCRIPT
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_MATH
}

/// A `class="hl-<colour>"` value: `<colour>` one of Word's 16 named
/// highlight colours (`yellow`, `green`, `cyan`, …, `lightGray`, `black`,
/// `white`), read without regard to case, the prefix's included (a
/// hand-edited `HL-Cyan` is `cyan`). Anything else is the default yellow.
fn hl_color_class(class: &str) -> Option<&'static str> {
    let colour = class
        .get(..3)
        .filter(|prefix| prefix.eq_ignore_ascii_case("hl-"))
        .map(|_| &class[3..])?;
    crate::edit::HIGHLIGHTS
        .iter()
        .copied()
        .find(|h| *h != "none" && h.eq_ignore_ascii_case(colour))
}

const HEADINGS: [&str; 6] = [
    "Heading1", "Heading2", "Heading3", "Heading4", "Heading5", "Heading6",
];

/// Picture width cap: 6.5 inches in EMU, the text width of a Letter page
/// with one-inch margins.
const MAX_PICTURE_WIDTH: u64 = 5_943_600;

/// A list being written: its numbering instance, its level, and how many
/// paragraphs its current item has.
struct OpenList {
    instance: usize,
    level: u32,
    paragraphs: u32,
}

/// Where a change span opened: in which paragraph and whether before any of
/// its text, so a paragraph that is one change whole can take its mark in.
#[derive(Clone, Copy)]
struct SpanStart {
    serial: u32,
    kind: Kind,
    paragraph: u64,
    at_start: bool,
    /// Runs of the start paragraph before the span opened.
    runs_before: usize,
    /// The story index of the paragraph the span opened at the end of, when
    /// its first content is that paragraph's break (`A{++\n\nB++}`).
    at_break: Option<usize>,
}

/// The cells of a table row so far: all empty, all one change whole of one
/// kind, or anything else.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RowCells {
    Empty,
    /// The kind, and the serial of the first cell's change.
    Whole(Kind, u32),
    Mixed,
}

/// A span that just closed; a comment right after it anchors to it.
#[derive(Clone, Copy)]
struct Anchor {
    slot: usize,
    group: Option<usize>,
    /// For a change: the serials of its deleted side (a substitution's) and
    /// of the rest, which attributions after it name the author of.
    serials: [Option<u32>; 2],
    /// How many attributions followed the change so far.
    attributed: u8,
}

impl Anchor {
    /// Whether a comment here may be an attribution: the first after a
    /// change, or the second after a substitution, for its inserted side.
    fn takes_attribution(&self) -> bool {
        self.group.is_none()
            && self.serials[1].is_some()
            && (self.attributed == 0 || (self.attributed == 1 && self.serials[0].is_some()))
    }
}

struct Writer<'o, 'a> {
    options: &'o DocxOptions<'a>,
    /// A soft line break holds until the words after it arrive: the space
    /// it stands for belongs inside the change delimiters around it, and
    /// one the paragraph ends on (a change that takes the paragraph's own
    /// line break) holds no words at all and is dropped.
    soft_break: bool,
    /// The change delimiters held behind a soft line break, replayed when
    /// the words arrive (or when the paragraph ends, without the space).
    held: Vec<critic::Token>,
    body: Vec<Item>,
    /// Footnote definitions by label.
    definitions: HashMap<String, Vec<Item>>,
    /// Substitutions one tag names (`r5+6+7`): the deleted side's serial and
    /// the inserted side's, whose ids the runs each has split.
    shared_tags: Vec<(u32, u32)>,
    /// The labels whose definition the view names an endnote
    /// (`<!-- endnotes.p0 -->` before `[^2]:`).
    endnotes: HashSet<String>,
    /// The id line just read names an endnote: the next definition is one.
    endnote_next: bool,
    /// The label of each footnote reference, in order, with its change.
    references: Vec<(String, Option<(Kind, u32)>)>,
    /// The footnote definition being read.
    target: Option<String>,
    /// What the comment closing a note's definition says, by label: the
    /// marks its text does not show (`<!-- rev footnotes.r11 @AL -->`) and
    /// its formatting changes.
    note_clauses: HashMap<String, view_notes::IdLine>,

    // Block context.
    paragraph: Option<usize>,
    paragraph_serial: u64,
    paragraph_runs: usize,
    heading: Option<usize>,
    /// What is left of a numbered heading's label at the start of its text.
    label_prefix: Option<String>,
    /// The YAML header says the text is an agent view (`view: tracked`):
    /// its id lines describe paragraphs. In other Markdown an HTML comment
    /// is the author's own and writes nothing.
    view: bool,
    /// The view shows a document with its changes accepted or rejected:
    /// the marks its lines name are history, not changes to write.
    resolved: bool,
    /// The lines of the HTML block open here, read whole when it ends.
    html_block: Option<String>,
    /// What the view's table lines say, one per table in order
    /// ([`table_plans`]); the open table's, and how many tables are read.
    tables: Vec<Option<TablePlan>>,
    table_plan: Option<TablePlan>,
    table_count: usize,
    /// The open table's rows so far, and whether the open row's `<br>`s
    /// end paragraphs.
    row_number: usize,
    row_breaks: bool,
    /// Whether each footnote definition's `<br>`s end paragraphs
    /// ([`note_breaks`]), how many definitions are read, and the open one's.
    note_breaks: Vec<(usize, usize)>,
    /// Each list item's level by its indentation ([`item_levels`]), and
    /// how many items are read.
    item_levels: Vec<Option<u32>>,
    item_count: usize,
    /// How many definitions each label has, how many of them are read, and
    /// how many times the body referenced it: a label defined twice (a
    /// footnote and an endnote both `[^1]`) is two notes, the first
    /// reference the first definition's, the next the second's.
    label_definitions: HashMap<String, usize>,
    label_defined: HashMap<String, usize>,
    label_uses: HashMap<String, usize>,
    note_count: usize,
    note_paragraph_breaks: bool,
    /// The style the next note's id line names (`<!-- endnotes.p0
    /// Endnote -->`), then the open note's.
    note_style_next: Vec<String>,
    note_style: Vec<String>,
    /// The note paragraph `begin_paragraph` styles next, from 0.
    note_paragraph: usize,
    /// The number of the next cell paragraph, when the table line counts
    /// the open row's paragraphs one by one.
    cell_paragraph: Option<usize>,
    /// The open row's planned cells, and its left-out empty cells.
    row_shapes: Vec<CellShape>,
    row_pad: Vec<Pad>,
    /// The grid columns the open row leaves out before its cells (its
    /// `gaps`' `before`): its first cell's grid column.
    row_before: usize,
    /// Whether the open Markdown cell is a grid column the cell before
    /// spans ([`CellShape::Spanned`]).
    spanned: bool,
    quote: u32,
    lists: Vec<OpenList>,
    bullets: Option<usize>,
    code_block: bool,
    table: Option<Vec<Option<&'static str>>>,
    column: usize,
    cell: Option<Option<&'static str>>,
    row: Option<(usize, Option<(Kind, u32)>)>,
    /// Whether every cell of the row so far is one change whole.
    row_cells: RowCells,
    /// Runs a task list marker wrote at the start of the paragraph: a
    /// change that starts right after them takes them in.
    prefix_runs: Vec<usize>,
    /// The open image: its destination, its reference name (`image1` of
    /// `![alt][image1]`, else the destination), its alt text so far, and
    /// the CriticMarkup tokens the alt held that cross its edge.
    image: Option<(String, String, String, Vec<Token>)>,
    /// A ` ```svg NAME ` or ` ```mermaid NAME ` fence being read: its
    /// language, its backing's name and the source accumulated so far.
    drawing_block: Option<(String, String, String)>,
    metadata: Option<String>,

    // Inline formatting depth.
    bold: u32,
    italic: u32,
    strike: u32,
    underline: u32,
    superscript: u32,
    subscript: u32,
    /// Open `<mark>` tags: a yellow highlight.
    mark: u32,
    /// The colour of each open `<mark>`, innermost last (`<mark>` is the
    /// default yellow; `<mark class="hl-<colour>">` is `colour`). A
    /// non-empty stack sets the colour of the next run; pop on close.
    mark_color: Vec<&'static str>,
    /// The inline HTML tags dropped so far, each warned of once.
    dropped_html: Vec<String>,
    code: u32,
    link: u32,
    /// The `<a href>` tags open in the block, which close with it.
    html_links: u32,

    // CriticMarkup.
    pieces: Pieces,
    change: Option<(Kind, u32)>,
    serial: u32,
    slot: usize,
    span_start: Option<SpanStart>,
    closed: Option<u32>,
    /// The deleted side's serial while a substitution's inserted side is open.
    substituted: Option<u32>,
    /// Where the open substitution's deleted and inserted sides start in
    /// the story: Word writes the inserted side first.
    substitution: Option<(usize, Option<usize>)>,
    /// The change the open comment follows, while the comment may still
    /// turn out to be the change's attribution.
    comment_anchor: Option<Anchor>,
    /// Scopes (`{==…==}`) with no comment yet and no text since them
    /// outside a scope: pieces of one range whose comment follows the last
    /// piece.
    pending_scopes: Vec<(usize, usize)>,
    /// Where the comment range of a scope inside a substitution's
    /// inserted side ends, once that side moved before the deleted one.
    range_end_at: Option<usize>,
    /// The slot of the range a comment just took, while another comment
    /// may follow on the same text.
    shared_range: Option<usize>,
    /// The header's `unplaced:` marks, by the paragraph they are in: a cell
    /// paragraph has no id line to name them.
    unplaced: HashMap<usize, Vec<(&'static str, view_notes::Tag)>>,
    run_after_close: bool,
    scope: Option<(usize, usize)>,
    groups: usize,
    anchor: Option<Anchor>,
    comment: Option<Comment>,
    /// A body field's code being read (`{REF _Ref1 \h}`), up to its `}`.
    field_code: Option<String>,
    /// What the last id line says, for the paragraph after it.
    id_line: Option<view_notes::IdLine>,
    /// The comments whose ranges the open paragraph's id line says run
    /// into it (`in c5 c6`); None without an id line.
    paragraph_in_comments: Option<Vec<u32>>,
    /// The open paragraph ends in a page break (`page-break`).
    page_break_due: bool,
    /// That page break's tracked change (`page-break-del r5 @AC`).
    page_break_change: Option<(Kind, view_notes::Tag)>,
    /// The view's `authors:` table, by handle (`AC` → Ann Counsel).
    authors: HashMap<String, view_notes::Author>,
    /// The changes of each author the view dates with a range of days, in
    /// the order their tags come.
    ranged: BTreeMap<String, Vec<u32>>,
    /// The undated comments of each author dated with a range of days, by
    /// comment id.
    ranged_comments: BTreeMap<String, Vec<u32>>,
    /// The authors dated with a range whose last day an undated formatting
    /// mark already took.
    range_ended: HashSet<String>,

    document: Document,
}

impl<'o, 'a> Writer<'o, 'a> {
    fn new(options: &'o DocxOptions<'a>) -> Self {
        Self {
            soft_break: false,
            held: Vec::new(),
            options,
            body: Vec::new(),
            definitions: HashMap::new(),
            endnotes: HashSet::new(),
            endnote_next: false,
            shared_tags: Vec::new(),
            references: Vec::new(),
            target: None,
            note_clauses: HashMap::new(),
            paragraph: None,
            paragraph_serial: 0,
            paragraph_runs: 0,
            heading: None,
            label_prefix: None,
            view: false,
            resolved: false,
            html_block: None,
            tables: Vec::new(),
            table_plan: None,
            table_count: 0,
            row_number: 0,
            row_breaks: false,
            note_breaks: Vec::new(),
            item_levels: Vec::new(),
            item_count: 0,
            label_definitions: HashMap::new(),
            label_defined: HashMap::new(),
            label_uses: HashMap::new(),
            note_count: 0,
            note_paragraph_breaks: false,
            note_style_next: Vec::new(),
            note_style: Vec::new(),
            note_paragraph: 0,
            cell_paragraph: None,
            row_shapes: Vec::new(),
            row_pad: Vec::new(),
            row_before: 0,
            spanned: false,
            quote: 0,
            lists: Vec::new(),
            bullets: None,
            code_block: false,
            table: None,
            column: 0,
            cell: None,
            row: None,
            row_cells: RowCells::Empty,
            prefix_runs: Vec::new(),
            image: None,
            drawing_block: None,
            metadata: None,
            bold: 0,
            italic: 0,
            strike: 0,
            underline: 0,
            superscript: 0,
            subscript: 0,
            mark: 0,
            mark_color: Vec::new(),
            dropped_html: Vec::new(),
            code: 0,
            link: 0,
            html_links: 0,
            pieces: Pieces::default(),
            change: None,
            serial: 0,
            slot: 0,
            span_start: None,
            closed: None,
            substituted: None,
            substitution: None,
            comment_anchor: None,
            pending_scopes: Vec::new(),
            range_end_at: None,
            shared_range: None,
            unplaced: HashMap::new(),
            id_line: None,
            paragraph_in_comments: None,
            page_break_due: false,
            page_break_change: None,
            run_after_close: false,
            scope: None,
            groups: 0,
            anchor: None,
            comment: None,
            field_code: None,
            authors: HashMap::new(),
            ranged: BTreeMap::new(),
            ranged_comments: BTreeMap::new(),
            range_ended: HashSet::new(),
            document: Document::default(),
        }
    }

    fn story(&mut self) -> &mut Vec<Item> {
        match &self.target {
            Some(label) => self.definitions.entry(label.clone()).or_default(),
            None => &mut self.body,
        }
    }

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            // Inside a list item, pulldown-cmark hands an HTML block's
            // indentation past the item's content column over as text.
            Event::Text(text) if self.html_block.is_some() => {
                if let Some(block) = &mut self.html_block {
                    block.push_str(&text);
                }
            }
            Event::Text(text) if self.drawing_block.is_some() => {
                if let Some((_, _, block)) = &mut self.drawing_block {
                    block.push_str(&text);
                }
            }
            Event::Text(text) => self.text(&text),
            Event::Code(text) => {
                self.soft_break_resolved(true);
                self.code += 1;
                self.text(&text);
                self.code -= 1;
            }
            Event::InlineMath(text) => self.math(&text, false),
            Event::DisplayMath(text) => self.math(&text, true),
            Event::InlineHtml(html) => self.inline_html(&html),
            Event::Html(html) => match &mut self.html_block {
                Some(block) => block.push_str(&html),
                None => self.id_lines(&html),
            },
            Event::FootnoteReference(label) => self.note_reference(&label),
            Event::SoftBreak => self.soft_break = true,
            Event::HardBreak => {
                if let Some(comment) = &mut self.comment {
                    comment.paragraphs.push(String::new());
                } else {
                    self.push(Content::Break);
                }
            }
            Event::Rule => {
                self.end_paragraph();
                self.begin_paragraph();
                if let Some(index) = self.paragraph
                    && let Some(Item::Paragraph(paragraph)) = self.story().get_mut(index)
                {
                    paragraph.rule = true;
                }
                self.end_paragraph();
            }
            Event::TaskListMarker(checked) => {
                self.push(Content::Text(
                    if checked { "\u{2612} " } else { "\u{2610} " }.to_string(),
                ));
                self.paragraph_runs -= 1;
                let index = self.story().len() - 1;
                self.prefix_runs.push(index);
            }
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {
                self.end_paragraph();
                self.begin_paragraph();
            }
            Tag::Heading { level, .. } => {
                self.end_paragraph();
                self.heading = Some(heading_level(level));
                self.begin_paragraph();
            }
            Tag::BlockQuote(_) => {
                self.end_paragraph();
                self.quote += 1;
            }
            Tag::CodeBlock(kind) => {
                self.end_paragraph();
                self.code_block = true;
                self.begin_paragraph();
                // ` ```svg NAME ` and ` ```mermaid NAME ` are a drawing's
                // readable layer: gather the source and let the block's
                // end splice `NAME`'s backing.
                if let CodeBlockKind::Fenced(info) = &kind {
                    let mut words = info.split_whitespace();
                    if let Some(lang) = words.next()
                        && matches!(lang, "svg" | "mermaid")
                        && let Some(name) = words.next()
                    {
                        self.drawing_block =
                            Some((lang.to_string(), name.to_string(), String::new()));
                    }
                }
            }
            Tag::List(first) => {
                self.end_paragraph();
                let level = self
                    .item_levels
                    .get(self.item_count)
                    .copied()
                    .flatten()
                    .unwrap_or_else(|| u32::try_from(self.lists.len()).unwrap_or(8).min(8));
                let instance = match first {
                    Some(start) => {
                        self.document.lists.push(List {
                            ordered: true,
                            level,
                            start: u32::try_from(start).unwrap_or(u32::MAX),
                        });
                        self.document.lists.len() - 1
                    }
                    None => *self.bullets.get_or_insert_with(|| {
                        self.document.lists.push(List {
                            ordered: false,
                            level: 0,
                            start: 1,
                        });
                        self.document.lists.len() - 1
                    }),
                };
                self.lists.push(OpenList {
                    instance,
                    level,
                    paragraphs: 0,
                });
            }
            Tag::Item => {
                self.end_paragraph();
                let level = self.item_levels.get(self.item_count).copied().flatten();
                self.item_count += 1;
                if let Some(list) = self.lists.last_mut() {
                    list.paragraphs = 0;
                    list.level = level.unwrap_or(list.level);
                }
            }
            Tag::FootnoteDefinition(label) => {
                // An id line with no text before it is a body paragraph.
                self.textless_paragraph();
                self.end_paragraph();
                // The id line's paragraphs and the definition's `<br>`s:
                // every `<br>` ends a paragraph when they add up.
                let (counted, breaks) = self
                    .note_breaks
                    .get(self.note_count)
                    .copied()
                    .unwrap_or_default();
                self.note_paragraph_breaks = counted > 1 && breaks + 1 == counted;
                self.note_style = std::mem::take(&mut self.note_style_next);
                self.note_paragraph = 0;
                self.note_count += 1;
                let label = note_label(&label);
                if counted > 1 && !self.note_paragraph_breaks {
                    // One paragraph takes the first style; the rest go unused.
                    let styles = match self.note_style.split_first() {
                        Some((first, rest)) if !rest.is_empty() => {
                            format!(", in style {first}; {} are not written", rest.join(", "))
                        }
                        _ => String::new(),
                    };
                    self.document.warnings.push(format!(
                        "[^{label}]: its id line counts {counted} paragraphs and its text \
                         {} by its <br>s: the <br>s are written as line breaks{styles}",
                        breaks + 1
                    ));
                }
                // A second definition of a label is a note of its own, for
                // the label's second reference.
                let nth = {
                    let defined = self.label_defined.entry(label.clone()).or_insert(0);
                    *defined += 1;
                    *defined - 1
                };
                let label = nth_label(&label, nth);
                if std::mem::take(&mut self.endnote_next) {
                    self.endnotes.insert(label.clone());
                }
                self.target = Some(label);
            }
            Tag::Table(alignments) => {
                self.end_paragraph();
                let mut columns: Vec<_> = alignments.iter().map(|a| alignment(*a)).collect();
                self.table_plan = self
                    .tables
                    .get(self.table_count)
                    .cloned()
                    .flatten()
                    .filter(|_| self.view);
                // The grid is the line's: it keeps the columns the view left
                // out, and has none for the view's padding past it.
                let mut grid = columns.clone();
                if let Some(count) = self.table_plan.as_ref().and_then(|plan| plan.columns) {
                    grid.resize(count, None);
                    columns.resize(count.max(columns.len()), None);
                }
                let align = self.table_plan.as_ref().and_then(|plan| plan.align);
                // The style the table's line names, else the legend's for
                // every table, else Word's thin grid, as Insert Table makes
                // (Arthur, 2026-10-15). A view names Normal Table for a
                // table without borders: no tblStyle, Word's default table
                // style, as the source had it.
                let style = self
                    .table_plan
                    .as_ref()
                    .and_then(|plan| plan.style.clone())
                    .or_else(|| {
                        self.document
                            .table_style
                            .as_ref()
                            .map(|(id, _, _)| id.clone())
                    })
                    .or_else(|| Some("TableGrid".to_string()))
                    .filter(|id| id != "TableNormal");
                if let Some(id) = &style {
                    // The singular legend's word for the style, else the
                    // plural `tables:` line's entry for it.
                    let legend = self
                        .document
                        .table_style
                        .as_ref()
                        .filter(|(legend, ..)| legend == id)
                        .map(|(_, border, name)| (*border, name.clone()))
                        .or_else(|| self.document.table_legend.get(id).cloned());
                    let border = legend.as_ref().and_then(|(border, _)| *border);
                    // Word's thin Table Grid, unless the legend gives it a
                    // border or a name of its own (`TableGrid, all borders
                    // 1.5pt`, `TableGrid, named …`).
                    let name = legend.and_then(|(_, name)| name);
                    if id == "TableGrid" && border.is_none_or(|sz| sz == 4) && name.is_none() {
                        self.document.styles.insert("TableGrid");
                    } else {
                        self.document
                            .table_styles
                            .insert(id.clone(), (border, name));
                    }
                }
                let formats = self.table_formats();
                self.story().push(Item::Table {
                    columns: grid,
                    align,
                    style,
                    formats,
                });
                self.table = Some(columns);
                self.table_count += 1;
                self.row_number = 0;
            }
            Tag::TableHead => self.begin_row(true),
            Tag::TableRow => self.begin_row(false),
            Tag::TableCell => {
                // The cell's grid column is the row's `before` gap plus
                // the spans of its real cells so far: the view's cells
                // before it count grid columns they span, and a gap the
                // view shows fewer empty cells for. A plain table has no
                // plan: its cells are the grid columns, one for one.
                let grid = if self.row_shapes.is_empty() {
                    self.column
                } else {
                    self.row_before
                        + self
                            .row_shapes
                            .iter()
                            .take(self.column)
                            .map(|shape| match shape {
                                CellShape::Cell { across, .. } => *across,
                                CellShape::Spanned => 0,
                            })
                            .sum::<usize>()
                };
                let align = self
                    .table
                    .as_ref()
                    .and_then(|columns| columns.get(grid).copied())
                    .flatten();
                let shape = self.row_shapes.get(self.column).copied();
                self.column += 1;
                let (across, merge) = match shape {
                    Some(CellShape::Spanned) => {
                        self.spanned = true;
                        return;
                    }
                    Some(CellShape::Cell { across, merge }) => (across, merge),
                    None => (1, None),
                };
                self.cell = Some(align);
                self.story().push(Item::Cell {
                    align,
                    across,
                    merge,
                });
            }
            Tag::Emphasis => self.italic += 1,
            Tag::Strong => self.bold += 1,
            Tag::Strikethrough => self.strike += 1,
            Tag::Superscript => self.superscript += 1,
            Tag::Subscript => self.subscript += 1,
            Tag::Link {
                link_type,
                dest_url,
                title,
                ..
            } => {
                let title = (!title.is_empty()).then(|| critic::decode(&title));
                let dest = critic::decode(&dest_url);
                // `<ann@x.test>` is an email autolink: its destination is the
                // address, and the link Word stores is `mailto:` plus that address.
                let dest = if link_type == LinkType::Email && !dest.starts_with("mailto:") {
                    format!("mailto:{dest}")
                } else {
                    dest
                };
                self.link_start(&dest, title);
            }
            Tag::Image { dest_url, id, .. } => {
                if self.image.is_none() {
                    self.image = Some((
                        critic::decode(&dest_url),
                        critic::decode(&id),
                        String::new(),
                        Vec::new(),
                    ));
                }
            }
            Tag::MetadataBlock(_) => self.metadata = Some(String::new()),
            Tag::HtmlBlock => self.html_block = Some(String::new()),
            Tag::DefinitionList | Tag::DefinitionListTitle | Tag::DefinitionListDefinition => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        // Markdown's own marks close inside their block; an inline HTML tag
        // left open (`<b>` without `</b>`) closes with it.
        if matches!(
            tag,
            TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::Item | TagEnd::TableCell
        ) {
            self.close_inline_tags();
        }
        match tag {
            TagEnd::Paragraph => self.end_paragraph(),
            TagEnd::Heading(_) => {
                self.end_paragraph();
                self.heading = None;
            }
            TagEnd::BlockQuote(_) => {
                self.end_paragraph();
                self.quote = self.quote.saturating_sub(1);
            }
            TagEnd::CodeBlock => {
                if let Some((lang, name, text)) = self.drawing_block.take() {
                    use super::from_docx::drawing;
                    let dest = self.document.drawing_backings.get(&name).cloned();
                    let drawing = dest.and_then(|dest| drawing::decode(&dest).ok()).and_then(
                        |mut drawing| match lang.as_str() {
                            "svg" => {
                                use base64::Engine;
                                drawing
                                    .fill_slot_data(
                                        &base64::engine::general_purpose::STANDARD
                                            .encode(text.trim().as_bytes()),
                                        "image/svg+xml",
                                    )
                                    .then_some(drawing)
                            }
                            // A `mermaid` fence's text is the chart it
                            // was: unchanged, the payload writes back byte
                            // for byte; edited, its chart part is patched.
                            _ if drawing.mermaid_unchanged(&text) => Some(drawing),
                            _ if drawing.rebuild_mermaid(&text) => Some(drawing),
                            _ => {
                                self.document.warnings.push(format!(
                                    "mermaid '{name}' names a drawing this text cannot edit"
                                ));
                                Some(drawing)
                            }
                        },
                    );
                    match drawing {
                        Some(drawing) => {
                            // A drawing's paragraph: not SourceCode.
                            if let Some(index) = self.paragraph
                                && let Some(Item::Paragraph(paragraph)) =
                                    self.story().get_mut(index)
                            {
                                paragraph.style = None;
                            }
                            self.document.drawings.push(drawing);
                            let index = self.document.drawings.len() - 1;
                            self.push(Content::Drawing(index));
                        }
                        // No backing, or one that did not read: the fence
                        // stays the code it shows.
                        None => self.plain(&text),
                    }
                }
                // The block's last line ends in a newline: no break after it.
                let story = self.story();
                if matches!(
                    story.last(),
                    Some(Item::Run(Run {
                        content: Content::Break,
                        ..
                    }))
                ) {
                    story.pop();
                }
                self.end_paragraph();
                self.code_block = false;
            }
            TagEnd::List(_) => {
                self.end_paragraph();
                self.lists.pop();
            }
            TagEnd::Item => self.end_paragraph(),
            TagEnd::FootnoteDefinition => {
                self.end_paragraph();
                self.target = None;
                self.note_style.clear();
                self.note_paragraph_breaks = false;
            }
            TagEnd::Table => {
                self.story().push(Item::TableEnd);
                if !self.table_in_range() {
                    self.pending_scopes.clear();
                    self.range_end_at = None;
                    self.shared_range = None;
                }
                self.table = None;
                self.table_plan = None;
                self.row_breaks = false;
                self.cell_paragraph = None;
                self.row_shapes.clear();
                self.row_pad.clear();
            }
            TagEnd::TableHead | TagEnd::TableRow => {
                // The empty cells the view left out at the row's end: their
                // paragraphs still take their column's alignment, which the
                // view counts toward the column's majority.
                let mut column = self.row_before
                    + self
                        .row_shapes
                        .iter()
                        .map(|shape| match shape {
                            CellShape::Cell { across, .. } => *across,
                            CellShape::Spanned => 0,
                        })
                        .sum::<usize>();
                for (across, merge) in std::mem::take(&mut self.row_pad) {
                    let align = self
                        .table
                        .as_ref()
                        .and_then(|columns| columns.get(column).copied())
                        .flatten();
                    column += across;
                    self.cell = Some(align);
                    self.story().push(Item::Cell {
                        align,
                        across,
                        merge,
                    });
                    self.begin_paragraph();
                    self.end_paragraph();
                    self.cell = None;
                    self.story().push(Item::CellEnd);
                }
                self.end_row();
            }
            TagEnd::TableCell if std::mem::take(&mut self.spanned) => {}
            TagEnd::TableCell => {
                if self.paragraph.is_none() {
                    self.begin_paragraph();
                }
                self.end_paragraph();
                self.cell = None;
                self.story().push(Item::CellEnd);
            }
            TagEnd::Emphasis => self.italic = self.italic.saturating_sub(1),
            TagEnd::Strong => self.bold = self.bold.saturating_sub(1),
            TagEnd::Strikethrough => self.strike = self.strike.saturating_sub(1),
            TagEnd::Superscript => self.superscript = self.superscript.saturating_sub(1),
            TagEnd::Subscript => self.subscript = self.subscript.saturating_sub(1),
            TagEnd::Link => self.link_end(),
            TagEnd::Image => self.picture(),
            TagEnd::MetadataBlock(_) => {
                if let Some(text) = self.metadata.take() {
                    self.metadata(&text);
                }
            }
            TagEnd::HtmlBlock => {
                if let Some(block) = self.html_block.take() {
                    let view_lines = self.view
                        && self.cell.is_none()
                        && self.comment.is_none()
                        && self.target.is_none();
                    if comments_only(&block) && (block.trim() == PAGE_BREAK_LINE || view_lines) {
                        self.id_lines(&block);
                    } else {
                        // Comments of the author's own, and comments
                        // inside other markup, are dropped — but their
                        // delimiters still count.
                        self.html_text(&block);
                    }
                }
            }
            TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition => {}
        }
    }

    /// Whether the paragraph before the one at `index` ends in a break a
    /// change of `kind` crossed (`A{++\n\n++}{++B++}`). That break is the
    /// mark the change adds, so a block after it that is one change whole
    /// keeps its own mark, which was the earlier paragraph's.
    fn after_changed_break(&mut self, index: usize, kind: Kind) -> bool {
        self.story()[..index]
            .iter()
            .rev()
            .find_map(|item| match item {
                Item::Paragraph(p) => Some(p.mark == Some(kind) && !p.whole),
                Item::Table { .. } | Item::TableEnd | Item::Cell { .. } | Item::CellEnd => {
                    Some(false)
                }
                _ => None,
            })
            .unwrap_or(false)
    }

    fn begin_row(&mut self, header: bool) {
        self.column = 0;
        // In a view the table line says whether the first row repeats.
        let header = header
            && self
                .table_plan
                .as_ref()
                .is_none_or(|plan| plan.header_repeats);
        let row = self
            .table_plan
            .as_ref()
            .and_then(|plan| plan.rows.get(self.row_number))
            .cloned()
            .unwrap_or_default();
        self.row_breaks = row.breaks;
        self.cell_paragraph = row.first;
        self.row_shapes = row.cells;
        self.row_pad = row.pad;
        self.row_before = row.gaps.0;
        self.row_number += 1;
        let index = self.story().len();
        self.story().push(Item::Row {
            header,
            change: None,
            gaps: row.gaps,
        });
        self.row = Some((index, self.change));
        self.row_cells = RowCells::Empty;
    }

    /// A row that one change covers from its first cell to its last, or
    /// whose every cell is one change whole, is inserted or deleted as a row.
    fn end_row(&mut self) {
        self.story().push(Item::RowEnd);
        let Some((index, open)) = self.row.take() else {
            return;
        };
        let row_change = match (open, self.row_cells) {
            (Some((kind, serial)), _) if self.change == Some((kind, serial)) => {
                Some((kind, serial))
            }
            (_, RowCells::Whole(kind, serial)) => Some((kind, serial)),
            _ => None,
        };
        // A row the table line names a mark of its own for has that mark,
        // and its text keeps its own (`rev r80 @AL in t1.row1`). A row it
        // says nothing of, whose cells' text is one change whole, had that
        // change on the row in Word's file — an id two cells' runs share
        // is nothing Word writes — so the writer puts it back there and
        // the text carries it. Plain Markdown has no line either way, and
        // keeps the inference.
        let own = !self.resolved
            && self.table_plan.as_ref().is_some_and(|plan| {
                plan.row_marks
                    .contains_key(&self.row_number.saturating_sub(1))
            });
        if let Some(row_change) = row_change.filter(|_| !own)
            && let Some(Item::Row { change, .. }) = self.story().get_mut(index)
        {
            *change = Some(row_change);
        }
    }

    /// The agent view's id lines: an `empty` one (`<!-- p2-p4 empty -->`)
    /// stands for the empty paragraphs it numbers, which have no text of
    /// their own to carry them; any other names the text after it, or,
    /// with no text before the next line (`<!-- p7 page-break -->`), a
    /// paragraph of its own.
    fn id_lines(&mut self, html: &str) {
        if html.trim() == PAGE_BREAK_LINE
            && self.cell.is_none()
            && self.comment.is_none()
            && self.target.is_none()
        {
            self.break_page();
            return;
        }
        if !self.view || self.cell.is_some() || self.comment.is_some() || self.target.is_some() {
            return;
        }
        // The view writes id lines as blocks of their own: a comment inside
        // other HTML (`<div>`) is the author's.
        if !comments_only(html) {
            return;
        }
        for line in html.lines() {
            // A page marker sits between blocks: it ends no paragraph.
            if line.trim_start().starts_with("<!-- page ") {
                continue;
            }
            // A note's id line says which part the definition after it is
            // in; it describes no body paragraph.
            // Like the part, the styles are only the line's right before
            // the definition.
            self.endnote_next = view_notes::endnote_line(line);
            self.note_style_next = view_notes::note_styles(line).unwrap_or_default();
            if self.endnote_next || view_notes::footnote_line(line) {
                continue;
            }
            self.textless_paragraph();
            if let Some(count) = view_notes::empty_paragraphs(line) {
                self.end_paragraph();
                // An id line that says more than `empty` (`<!-- p7
                // page-break -->`, `<!-- p2 TitlePage, empty -->`) stands
                // for every paragraph it numbers, each with what it says.
                if let Some(said) =
                    view_notes::id_line(line).filter(|said| *said != view_notes::IdLine::default())
                {
                    for _ in 0..count {
                        self.id_line = Some(said.clone());
                        self.textless_paragraph();
                    }
                    continue;
                }
                for _ in 0..count {
                    let story = self.story();
                    story.push(Item::Paragraph(Paragraph::default()));
                    story.push(Item::ParagraphEnd);
                }
            } else {
                self.id_line = view_notes::id_line(line);
            }
        }
    }

    /// A raw HTML block's text, a paragraph per block element, with a
    /// warning that its markup is dropped.
    fn html_text(&mut self, html: &str) {
        // Comments inside the block are dropped: their delimiters still
        // count, as CriticMarkup was read first.
        let tokens = self.html_comment_tokens(html);
        for token in tokens {
            self.token(token);
        }
        if self.comment.is_some() {
            return;
        }
        let (name, paragraphs) = html_block_text(html);
        if let Some(name) = name
            && !self.dropped_html.contains(&name)
        {
            self.document.warnings.push(format!(
                "HTML block <{name}> dropped: its text is kept, its markup is not"
            ));
            self.dropped_html.push(name);
        }
        for text in paragraphs {
            self.end_paragraph();
            self.begin_paragraph();
            self.text(&text);
            self.end_paragraph();
        }
    }

    /// The paragraph of an id line no text followed.
    fn textless_paragraph(&mut self) {
        if self.id_line.is_some() {
            self.end_paragraph();
            self.begin_paragraph();
            self.end_paragraph();
        }
    }

    fn begin_paragraph(&mut self) {
        let mut paragraph = Paragraph::default();
        let list = self.lists.last_mut().map(|list| {
            let first = list.paragraphs == 0;
            list.paragraphs += 1;
            (list.instance, list.level, first)
        });
        let list_indent = list.map(|(_, level, _)| 720 * (level + 1));
        if let Some(level) = self.heading {
            // The legend's style for the level (`"#": Title1, …`).
            let legend = self.document.style_specs.iter().find(|spec| {
                spec.heading == u32::try_from(level).ok() && spec.id != HEADINGS[level - 1]
            });
            paragraph.style = Some(match legend {
                Some(spec) => Cow::Owned(spec.id.clone()),
                None => HEADINGS[level - 1].into(),
            });
            paragraph.heading = true;
        } else if self.code_block {
            paragraph.style = Some("SourceCode".into());
            paragraph.indent = list_indent;
        } else if let Some((instance, level, first)) = list {
            paragraph.style = Some("ListParagraph".into());
            if first {
                paragraph.numbering = Some((instance, level));
            } else {
                paragraph.indent = list_indent;
            }
        } else if self.quote > 0 {
            paragraph.style = Some("Quote".into());
            if self.quote > 1 {
                paragraph.indent = Some(720 * self.quote);
            }
        } else if let Some(target) = &self.target {
            // Word's note style unless the id line names another; the
            // document's default needs no `w:pStyle`.
            let default = self
                .document
                .style_specs
                .iter()
                .find(|s| s.default)
                .map_or("Normal", |s| s.id.as_str());
            // A note paragraph takes its own entry, past the list the last.
            let at = self
                .note_paragraph
                .min(self.note_style.len().saturating_sub(1));
            self.note_paragraph += 1;
            paragraph.style = match self.note_style.get(at) {
                Some(style) if style == default => None,
                Some(style) => Some(style.clone().into()),
                None if self.endnotes.contains(target) => Some("EndnoteText".into()),
                None => Some("FootnoteText".into()),
            };
        }
        if let Some(align) = self.cell {
            paragraph.align = align;
        }
        // What the id line before it says.
        self.paragraph_in_comments = None;
        if self.cell.is_none()
            && self.target.is_none()
            && let Some(line) = self.id_line.take()
        {
            self.paragraph_in_comments = Some(line.in_comments.clone());
            paragraph.number = Some(line.number);
            let styled = line.style.is_some();
            if let Some(style) = line.style {
                paragraph.style = Some(style.into());
                // `# Text` under its own style: the level goes with it.
                paragraph.outline = self.heading.map(|level| level - 1);
            } else if let Some(outline) = line.outline {
                // `# Text` that states its own level, with no style: the
                // writer's `HeadingN` must not step in where the document
                // had none.
                paragraph.style = None;
                // The clause's bound is 1..=9, so the level cannot be empty.
                paragraph.outline = outline.checked_sub(1).map(|level| level as usize);
            }
            paragraph.align = line.align.or(paragraph.align);
            self.page_break_due = line.page_break;
            self.page_break_change = line.page_break_change.clone().map(|(inserted, tag)| {
                let kind = if inserted { Kind::Insert } else { Kind::Delete };
                (kind, tag)
            });
            paragraph.section_break = line.section_break;
            paragraph.ind = Indent {
                first_line: line.first_line,
                hanging: line.hanging,
                left: line.left,
                right: line.right,
            };
            if let Some(label) = line.label {
                // The label stands for the list's own number, and a heading
                // prints it before its text.
                match (self.heading, &label) {
                    (Some(_), Label::Text(text)) => self.label_prefix = Some(format!("{text} ")),
                    // `## - Economics`: the bullet is the list's.
                    (Some(_), Label::Bullet) => self.label_prefix = Some("- ".to_string()),
                    _ => {}
                }
                let level = line
                    .level
                    .unwrap_or_else(|| list.map_or(0, |(_, level, _)| level));
                let start = list
                    .filter(|&(_, _, first)| first)
                    .and_then(|(instance, _, _)| self.document.lists.get(instance))
                    .filter(|list| list.ordered)
                    .map(|list| list.start);
                paragraph.label = self.label_use(label, level, start);
                if paragraph.label.is_some() {
                    paragraph.numbering = None;
                    paragraph.indent = None;
                    // The view names a list paragraph's style when it has one.
                    if !styled && list.is_some() {
                        paragraph.style = None;
                    }
                }
            }
            paragraph.formats = line.fmt.iter().map(|tag| self.format_mark(tag)).collect();
            if !self.resolved {
                paragraph.unshown = line
                    .rev
                    .iter()
                    .map(|tag| (!line.rev_del.contains(tag), self.format_mark(tag)))
                    .collect();
            }
            let mark = line
                .break_ins
                .map(|tag| (Kind::Insert, tag))
                .or_else(|| line.break_del.map(|tag| (Kind::Delete, tag)));
            self.fix_mark(&mut paragraph, mark);
        }
        // A cell paragraph the table line numbers: its mark is the line's.
        if self.cell.is_some()
            && let Some(at) = self.cell_paragraph
        {
            self.cell_paragraph = Some(at + 1);
            if let Some(align) = self
                .table_plan
                .as_ref()
                .and_then(|plan| plan.aligns.get(&at).copied())
            {
                paragraph.align = Some(align);
            }
            if let Some(style) = self
                .table_plan
                .as_ref()
                .and_then(|plan| plan.styles.get(&at).cloned())
            {
                paragraph.style = Some(style.into());
            }
            let formats = self
                .table_plan
                .as_ref()
                .and_then(|plan| plan.para_fmt.get(&at).cloned())
                .unwrap_or_default();
            paragraph.formats = formats.iter().map(|tag| self.format_mark(tag)).collect();
            if !self.resolved
                && let Some(tags) = self.unplaced.remove(&at)
            {
                for (kind, tag) in &tags {
                    let mark = self.format_mark(tag);
                    match *kind {
                        "fmt" => paragraph.formats.push(mark),
                        _ => paragraph.unshown.push((*kind == "ins", mark)),
                    }
                }
            }
            let mark = self.table_plan.as_ref().and_then(|plan| {
                let (inserted, tag) = plan.marks.get(&at)?;
                let kind = if *inserted {
                    Kind::Insert
                } else {
                    Kind::Delete
                };
                Some((kind, tag.clone()))
            });
            self.fix_mark(&mut paragraph, mark);
        }
        match &paragraph.style {
            Some(Cow::Borrowed(style)) => {
                self.document.styles.insert(style);
            }
            Some(Cow::Owned(style)) => {
                self.document.named_styles.insert(style.clone());
            }
            None => {}
        }
        let index = self.story().len();
        self.story().push(Item::Paragraph(paragraph));
        self.paragraph = Some(index);
        self.paragraph_serial += 1;
        self.paragraph_runs = 0;
        self.prefix_runs.clear();
    }

    /// The formatting changes the open table's line names.
    fn table_formats(&mut self) -> TableFormats {
        let Some(plan) = self.table_plan.as_ref() else {
            return TableFormats::default();
        };
        let (table, rows, cells) = (
            plan.fmt.clone(),
            plan.row_fmt.clone(),
            plan.cell_fmt.clone(),
        );
        let grid = plan.grid_fmt;
        let (row_marks, cell_marks) = if self.resolved {
            (BTreeMap::new(), BTreeMap::new())
        } else {
            (plan.row_marks.clone(), plan.cell_marks.clone())
        };
        if let Some(id) = grid {
            self.document.reserved_ids.insert(id);
        }
        let (borders, cell_borders) = (plan.borders.clone(), plan.cell_borders.clone());
        let row_borders = plan.row_borders.iter().cloned().collect();
        TableFormats {
            table: table.map(|tag| self.format_mark(&tag)),
            grid,
            borders,
            cell_borders,
            row_borders,
            row_marks: row_marks
                .into_iter()
                .map(|(row, (inserted, tag))| {
                    let kind = if inserted { Kind::Insert } else { Kind::Delete };
                    (row, (kind, self.format_mark(&tag)))
                })
                .collect(),
            cell_marks: cell_marks
                .into_iter()
                .map(|(cell, (kind, tag))| (cell, (kind, self.format_mark(&tag))))
                .collect(),
            rows: rows
                .into_iter()
                .map(|(row, tag)| (row, self.format_mark(&tag)))
                .collect(),
            cells: cells
                .into_iter()
                .map(|(cell, tag)| (cell, self.format_mark(&tag)))
                .collect(),
        }
    }

    /// A formatting change or unshown mark as its tag names it.
    fn format_mark(&mut self, tag: &view_notes::Tag) -> FormatMark {
        let (author, mut date) = self.view_author(&tag.handle, tag.date.as_deref());
        // A formatting mark is written where it stands, never revisited:
        // the first undated one of an author dated `first..last` takes the
        // last day now, the author's other items keep the first.
        if tag.date.is_none()
            && let Some((_, last)) = self
                .authors
                .get(&tag.handle)
                .and_then(|known| known.range.as_ref())
            && self.range_ended.insert(tag.handle.clone())
        {
            date = format!("{last}T23:59:59Z");
        }
        let id = tag.ids.first().copied();
        if let Some(id) = id {
            self.document.reserved_ids.insert(id);
        }
        FormatMark { id, author, date }
    }

    /// The open table lies in a comment's range (`in c5`): its cells'
    /// scopes are pieces of that range, which goes on past them.
    fn table_in_range(&self) -> bool {
        self.table_plan.as_ref().is_some_and(|plan| plan.in_comment)
    }

    /// A page break between blocks: the paragraph before it holds it, as
    /// Word puts one; with no paragraph before it, an empty one does.
    fn break_page(&mut self) {
        self.end_paragraph();
        let page_break = Item::Run(Run {
            content: Content::PageBreak,
            format: RunFormat::default(),
            change: None,
            scope: None,
        });
        let story = self.story();
        if matches!(story.last(), Some(Item::ParagraphEnd)) {
            let end = story.len() - 1;
            story.insert(end, page_break);
        } else {
            story.push(Item::Paragraph(Paragraph::default()));
            story.push(page_break);
            story.push(Item::ParagraphEnd);
        }
    }

    fn ensure_paragraph(&mut self) {
        if self.paragraph.is_none() {
            self.begin_paragraph();
        }
    }

    /// Ends the open paragraph. Its mark is in the change still open, or in
    /// the change that is the paragraph's whole text.
    fn end_paragraph(&mut self) {
        // A soft line break the paragraph ends on was the paragraph's own
        // line break: no words follow it, so it holds no space.
        self.soft_break_resolved(false);
        let Some(index) = self.paragraph.take() else {
            return;
        };
        self.label_prefix = None;
        let page_break_change = self.page_break_change.take().map(|(kind, tag)| {
            // The change the text before it ends in, when the tag names
            // it too: one `w:del` holds both, as in the source.
            let before = self.story().iter().rev().find_map(|item| match item {
                Item::Run(run) => Some(run.change),
                _ => None,
            });
            match before.flatten() {
                Some((k, serial))
                    if k == kind
                        && !tag.ids.is_empty()
                        && self.document.revision_ids.get(&serial) == Some(&tag.ids) =>
                {
                    (k, serial)
                }
                _ => (kind, self.tag_serial(&tag)),
            }
        });
        if std::mem::take(&mut self.page_break_due) {
            self.story().push(Item::Run(Run {
                content: Content::PageBreak,
                format: RunFormat::default(),
                change: page_break_change,
                scope: None,
            }));
        }
        if let Some(comment) = &mut self.comment {
            comment.paragraphs.push(String::new());
        }
        let after_break = self
            .span_start
            .map(|span| span.kind)
            .map(|kind| self.after_changed_break(index, kind));
        // A span opened at this paragraph's end: its first content is the break.
        if let (Some((_, serial)), Some(span)) = (self.change, self.span_start.as_mut())
            && span.serial == serial
            && span.paragraph == self.paragraph_serial
            && span.runs_before == self.paragraph_runs
            && self.cell.is_none()
        {
            span.at_break = Some(index);
        }
        // A view's cell or note says its paragraph marks by its `<br>`s,
        // in a change or not: a paragraph changed whole keeps its mark.
        let says_marks = self.view && (self.cell.is_some() || self.target.is_some());
        // A change that names another after ` on ` (a fold) is not the
        // paragraph's own mark: the writer nests it back inside the one it
        // names, and the row or cell it sits in keeps its own.
        let folded = |serial: u32| self.document.wraps.contains_key(&serial);
        let (mark, whole) = match self.change {
            Some((kind, serial)) => (Some((kind, serial)), false),
            None => match self.span_start {
                Some(span)
                    if !says_marks
                        && span.at_start
                        && span.paragraph == self.paragraph_serial
                        && self.closed == Some(span.serial)
                        && !folded(span.serial)
                        && !self.run_after_close
                        && self.paragraph_runs > 0
                        && !after_break.is_some_and(|after| after) =>
                {
                    (Some((span.kind, span.serial)), true)
                }
                _ => (None, false),
            },
        };
        // In a cell, a change opened at or before the paragraph's start and
        // open or just closed at its end covers it whole, across the `<br>`
        // that ends a cell paragraph (`{++A<br>B++}`).
        let covered = self.span_start.and_then(|span| {
            let starts = span.paragraph != self.paragraph_serial || span.at_start;
            let ends = self.change == Some((span.kind, span.serial))
                || (self.closed == Some(span.serial)
                    && !folded(span.serial)
                    && !self.run_after_close);
            (starts && ends).then_some((span.kind, span.serial))
        });
        let whole_in_cell = if whole { mark } else { covered };
        if self.cell.is_some() && self.paragraph_runs > 0 {
            // Whole is one change across the cells: the same kind naming
            // the same ids (two tags naming `r2`, or no ids at all, as
            // plain Markdown writes). A row whose cells each carry their
            // own change, all insertions, had no change on the row in
            // Word's file.
            let one_change = |before: u32, now: u32| {
                self.document.revision_ids.get(&before) == self.document.revision_ids.get(&now)
            };
            self.row_cells = match (self.row_cells, whole_in_cell) {
                (RowCells::Empty, Some((kind, serial))) => RowCells::Whole(kind, serial),
                (RowCells::Whole(before, was), Some((kind, now)))
                    if before == kind && one_change(was, now) =>
                {
                    RowCells::Whole(kind, now)
                }
                _ => RowCells::Mixed,
            };
        }
        // Blocks added (or removed) after a paragraph, from its break to the
        // end of this one: Word records them as their own marks changed, so
        // the paragraph before keeps its mark and properties.
        let shift = match (mark, self.span_start) {
            (None, Some(span))
                if span.paragraph != self.paragraph_serial
                    && self.closed == Some(span.serial)
                    && !folded(span.serial)
                    && !self.run_after_close
                    && self.paragraph_runs > 0
                    && self.cell.is_none() =>
            {
                span.at_break
                    .filter(|&first| no_table_between(self.story(), first, index))
                    .map(|first| (first, span.kind, span.serial))
            }
            _ => None,
        };
        let story = self.story();
        if let Some((first, kind, serial)) = shift
            && let Some(Item::Paragraph(paragraph)) = story.get_mut(first)
        {
            if !paragraph.mark_fixed {
                paragraph.mark = None;
                paragraph.mark_by = None;
            }
            if let Some(Item::Paragraph(paragraph)) = story.get_mut(index)
                && !paragraph.mark_fixed
            {
                paragraph.mark = Some(kind);
                paragraph.mark_by = Some(serial);
                paragraph.whole = false;
            }
        } else if let Some(Item::Paragraph(paragraph)) = story.get_mut(index)
            && !paragraph.mark_fixed
        {
            paragraph.mark = mark.map(|(kind, _)| kind);
            paragraph.mark_by = mark.map(|(_, serial)| serial);
            paragraph.whole = whole;
        }
        story.push(Item::ParagraphEnd);
    }

    fn format(&self) -> RunFormat {
        RunFormat {
            bold: self.bold > 0,
            italic: self.italic > 0,
            strike: self.strike > 0,
            underline: self.underline > 0,
            superscript: self.superscript > 0,
            subscript: self.subscript > 0,
            mark: self.mark > 0,
            mark_color: self.mark_color.last().copied(),
            code: self.code > 0,
            link: self.link > 0,
        }
    }

    fn push(&mut self, content: Content) {
        self.ensure_paragraph();
        let blank = matches!(&content, Content::Text(text) if text.trim().is_empty())
            || matches!(content, Content::Tab | Content::Break);
        if self.scope.is_none() && !blank {
            self.pending_scopes.clear();
            self.range_end_at = None;
            self.shared_range = None;
        }
        if self.code > 0 {
            self.document.styles.insert("VerbatimChar");
        }
        // Pulldown-cmark splits a stretch that holds a CommonMark delimiter
        // (`<mark>file\_</mark>` becomes `file` + `_`) even when the writer
        // holds the same marks on both sides; merge into the previous text
        // run so the round-trip keeps one run per stretch. A prefix run (a
        // task box) is not one to merge into: the paragraph's first change
        // claims the prefix runs when no text run precedes them, so text
        // merged into the box would ride along into the revision.
        if let Content::Text(ref text) = content {
            let last_prefix = self.prefix_runs.last().copied();
            if last_prefix != Some(self.story().len() - 1) {
                let now = self.format();
                let current_change = self.change;
                let current_scope = self.scope.map(|(_, group)| group);
                let story = self.story();
                let last = story.last_mut();
                let merge = matches!(
                    last,
                    Some(Item::Run(Run {
                        content: Content::Text(_),
                        format,
                        change,
                        scope,
                    })) if *format == now && *change == current_change && *scope == current_scope
                );
                if merge
                    && let Some(Item::Run(Run {
                        content: Content::Text(prev_text),
                        ..
                    })) = last
                {
                    prev_text.push_str(text);
                    return;
                }
            }
        }
        let run = Run {
            content,
            format: self.format(),
            change: self.change,
            scope: self.scope.map(|(_, group)| group),
        };
        self.story().push(Item::Run(run));
        self.paragraph_runs += 1;
        self.run_after_close = true;
        self.anchor = None;
    }

    fn text(&mut self, text: &str) {
        if self.metadata.is_some() || self.image.is_some() || !self.options.critic {
            self.soft_break_resolved(true);
        }
        if let Some(metadata) = &mut self.metadata {
            metadata.push_str(text);
            return;
        }
        if let Some((_, _, alt, loose)) = &mut self.image {
            // Alt text is kept, its markup is not; the CriticMarkup it
            // holds still took effect when it was read, before Markdown,
            // so the delimiters of spans that cross the image's edge are
            // remembered and applied where the image ends. A span whole
            // inside the alt is dropped with it.
            let mut tokens = Vec::new();
            self.pieces.feed(text, |piece| match piece {
                Piece::Text(text) => alt.push_str(text),
                Piece::Token(token) => tokens.push(token),
            });
            loose.extend(loose_tokens(tokens));
            return;
        }
        if !self.options.critic {
            self.plain(text);
            return;
        }
        let mut pieces = Vec::new();
        self.pieces.feed(text, |piece| {
            pieces.push(match piece {
                Piece::Text(text) => Ok(text.to_string()),
                Piece::Token(token) => Err(token),
            });
        });
        for piece in pieces {
            match piece {
                Ok(text) => {
                    self.soft_break_resolved(true);
                    self.plain(&text);
                }
                Err(token) => {
                    if self.soft_break {
                        self.held.push(token);
                    } else {
                        self.token(token);
                    }
                }
            }
        }
    }

    /// Resolves a held soft line break: the words after it have arrived
    /// (`words`) or the paragraph has ended. The change delimiters held
    /// behind it replay either way; the space only when words follow, and
    /// before them, inside the change it sat in.
    fn soft_break_resolved(&mut self, words: bool) {
        if std::mem::take(&mut self.soft_break) && words {
            self.plain(" ");
        }
        for token in std::mem::take(&mut self.held) {
            self.token(token);
        }
    }

    /// `$latex$` (or `$$latex$$` when `display`): an equation, or its
    /// LaTeX as text where an equation cannot go (a comment, alt text).
    fn math(&mut self, latex: &str, display: bool) {
        self.soft_break_resolved(true);
        if self.metadata.is_some() || self.image.is_some() || self.comment.is_some() {
            let fence = if display { "$$" } else { "$" };
            let text = format!("{fence}{}{fence}", critic::restore(latex));
            if let Some(metadata) = &mut self.metadata {
                metadata.push_str(&text);
            } else if let Some((_, _, alt, loose)) = &mut self.image {
                let mut tokens = Vec::new();
                self.pieces.feed(&text, |piece| match piece {
                    Piece::Text(text) => alt.push_str(text),
                    Piece::Token(token) => tokens.push(token),
                });
                loose.extend(loose_tokens(tokens));
            } else {
                self.plain(&text);
            }
            return;
        }
        // An insertion or deletion inside the math (`$A={--π--}r^2$`) marks
        // its runs; any other CriticMarkup there is text.
        let mut source = String::with_capacity(latex.len());
        Pieces::default().feed(latex, |piece| match piece {
            Piece::Text(text) => source.push_str(text),
            Piece::Token(Token::InsertStart) => source.push(math::INSERT_START),
            Piece::Token(Token::InsertEnd) => source.push(math::INSERT_END),
            Piece::Token(Token::DeleteStart) => source.push(math::DELETE_START),
            Piece::Token(Token::DeleteEnd) => source.push(math::DELETE_END),
            Piece::Token(token) => source.push_str(critic::delimiter(token)),
        });
        self.push(Content::Math {
            latex: source,
            display,
        });
    }

    /// Text without CriticMarkup: into the open comment, or as runs, with
    /// tabs and (in code blocks) line breaks as their own runs.
    fn plain(&mut self, text: &str) {
        // A numbered heading's label, which its id line holds: not text.
        let mut text = text;
        if self.comment.is_none()
            && let Some(prefix) = self.label_prefix.take()
        {
            if let Some(rest) = text.strip_prefix(prefix.as_str()) {
                if rest.is_empty() {
                    return;
                }
                text = rest;
            } else if let Some(left) = prefix.strip_prefix(text) {
                self.label_prefix = Some(left.to_string());
                return;
            }
        }
        if text.is_empty() {
            return;
        }
        if let Some(comment) = &mut self.comment {
            if let Some(last) = comment.paragraphs.last_mut() {
                last.push_str(text);
            }
            return;
        }
        if let Some(code) = &mut self.field_code {
            code.push_str(text);
            return;
        }
        let lines: Vec<&str> = if self.code_block {
            text.split('\n').collect()
        } else {
            vec![text]
        };
        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                self.push(Content::Break);
            }
            for (at, part) in line.split('\t').enumerate() {
                if at > 0 {
                    self.push(Content::Tab);
                }
                if !part.is_empty() {
                    self.push(Content::Text(part.to_string()));
                }
            }
        }
    }

    fn new_slot(&mut self) -> usize {
        self.document.slots.push(None);
        self.document.slots.len() - 1
    }

    fn open(&mut self, kind: Kind) {
        self.ensure_paragraph();
        self.serial += 1;
        self.change = Some((kind, self.serial));
        self.slot = self.new_slot();
        let slot = self.slot;
        self.story().push(Item::RangeStart(slot));
        if self.paragraph_runs == 0 {
            let change = self.change;
            for index in std::mem::take(&mut self.prefix_runs) {
                if let Some(Item::Run(run)) = self.story().get_mut(index) {
                    run.change = change;
                }
            }
        }
        self.span_start = Some(SpanStart {
            serial: self.serial,
            kind,
            paragraph: self.paragraph_serial,
            at_start: self.paragraph_runs == 0,
            runs_before: self.paragraph_runs,
            at_break: None,
        });
        self.anchor = None;
    }

    fn close(&mut self) {
        let serial = self.change.take().map(|(_, serial)| serial);
        self.anchor = Some(Anchor {
            slot: self.slot,
            group: None,
            serials: [self.substituted.take(), serial],
            attributed: 0,
        });
        self.closed = self.span_start.map(|span| span.serial);
        self.run_after_close = false;
    }

    /// `{~~old~>new~~}` is Word's substitution: the inserted text, then
    /// the deleted text, as Word's Compare writes them. The view reads a
    /// deletion before an insertion as two changes, so they keep that
    /// order. Each side moves whole with its links, bookmarks and comments;
    /// a side holding more (a paragraph's end, half a link or comment range)
    /// stays where it is. A scope that ends inside a side
    /// (`{~~old~>{==new==}~~}{>>c0 …<<}`) still has its range open: the side
    /// moves, and its range ends where the side does.
    fn inserted_side_first(&mut self) {
        /// The comment ranges a side leaves open, when it holds only runs,
        /// bookmarks, links that close within, and ranges that close
        /// within or are scopes awaiting their comment (`awaiting`).
        fn whole(items: &[Item], awaiting: &[usize]) -> Option<usize> {
            let mut links = 0usize;
            let mut ranges = Vec::new();
            for item in items {
                match item {
                    Item::Run(_) | Item::Bookmark(_) | Item::CommentReference(_) => {}
                    Item::LinkStart(..) => links += 1,
                    Item::LinkEnd if links > 0 => links -= 1,
                    Item::RangeStart(slot) => ranges.push(*slot),
                    Item::RangeEnd(_) if !ranges.is_empty() => {
                        ranges.pop();
                    }
                    _ => return None,
                }
            }
            (links == 0 && ranges.iter().all(|slot| awaiting.contains(slot)))
                .then_some(ranges.len())
        }
        let Some((start, Some(split))) = self.substitution.take() else {
            return;
        };
        let awaiting: Vec<usize> = self.pending_scopes.iter().map(|(slot, _)| *slot).collect();
        let story = self.story();
        let end = story.len();
        if start > split || split > end {
            return;
        }
        let (Some(deleted), Some(inserted)) = (
            whole(&story[start..split], &awaiting),
            whole(&story[split..end], &awaiting),
        ) else {
            return;
        };
        // One open range at most: its comment ends it.
        if deleted + inserted > 1 {
            return;
        }
        story[start..end].rotate_left(split - start);
        if inserted == 1 {
            self.range_end_at = Some(start + (end - split));
        }
    }

    fn token(&mut self, token: Token) {
        if self.comment.is_some() {
            match token {
                Token::CommentEnd => self.end_comment(),
                // A blank line inside `{>>…<<}`: another comment
                // paragraph, as `<br>` gives one.
                Token::CommentBreak => {
                    if let Some(comment) = &mut self.comment {
                        comment.paragraphs.push(String::new());
                    }
                }
                _ => {}
            }
            return;
        }
        match token {
            Token::InsertStart => self.open(Kind::Insert),
            Token::DeleteStart => self.open(Kind::Delete),
            Token::SubstituteStart => {
                self.open(Kind::Delete);
                self.substitution = Some((self.story().len(), None));
            }
            Token::SubstituteSeparator => {
                self.substituted = self.change.map(|(_, serial)| serial);
                self.serial += 1;
                self.change = Some((Kind::Insert, self.serial));
                self.span_start = None;
                let at = self.story().len();
                if let Some((_, split)) = &mut self.substitution {
                    *split = Some(at);
                }
            }
            Token::InsertEnd | Token::DeleteEnd => self.close(),
            Token::SubstituteEnd => {
                self.close();
                self.inserted_side_first();
            }
            Token::HighlightStart => {
                self.ensure_paragraph();
                let slot = self.new_slot();
                self.story().push(Item::RangeStart(slot));
                self.shared_range = None;
                self.scope = Some((slot, self.groups));
                self.groups += 1;
                self.anchor = None;
            }
            Token::HighlightEnd => {
                if let Some((slot, group)) = self.scope.take() {
                    self.pending_scopes.push((slot, group));
                    self.anchor = Some(Anchor {
                        slot,
                        group: Some(group),
                        serials: [None, None],
                        attributed: 0,
                    });
                }
            }
            Token::CommentStart => self.begin_comment(),
            Token::CommentEnd => {}
            Token::CommentBreak => {}
            Token::MarkStart => self.mark += 1,
            Token::MarkEnd => self.mark = self.mark.saturating_sub(1),
            Token::FieldStart => self.field_code = Some(String::new()),
            Token::FieldSeparator => {
                let code = self.field_code.take().unwrap_or_default();
                self.push(Content::FieldBegin(code));
            }
            Token::FieldEnd => self.push(Content::FieldEnd),
        }
    }

    /// A comment on the span that just closed, or on this point.
    fn begin_comment(&mut self) {
        let id = u32::try_from(self.document.comments.len()).unwrap_or(u32::MAX);
        match self.anchor.take() {
            // Anchored when it ends, unless it names the change's author.
            Some(anchor) if anchor.takes_attribution() => self.comment_anchor = Some(anchor),
            Some(anchor) => self.anchor_comment(anchor, id),
            None => self.anchor_here(id),
        }
        self.comment = Some(Comment {
            id,
            paragraphs: vec![String::new()],
            ..Comment::default()
        });
    }

    /// Anchors comment `id` on the span that just closed. A scope's
    /// range starts at the first piece of the range it ends.
    fn anchor_comment(&mut self, anchor: Anchor, id: u32) {
        let pieces = std::mem::take(&mut self.pending_scopes);
        match anchor.group {
            Some(group) => {
                let slot = pieces.first().map_or(anchor.slot, |(slot, _)| *slot);
                self.document.slots[slot] = Some(id);
                self.shared_range = Some(slot);
                self.document.commented.insert(group);
                for (_, group) in pieces {
                    self.document.commented.insert(group);
                }
            }
            None => self.document.slots[anchor.slot] = Some(id),
        }
    }

    /// Anchors comment `id` on the scopes just before it, when there
    /// are some (`{++{==text==}++}{>>who<<}{>>note<<}`), else on this point.
    fn anchor_here(&mut self, id: u32) {
        // The next comment on the text the last one took shares its range.
        if self.pending_scopes.is_empty()
            && let Some(slot) = self.shared_range
        {
            self.document.shared.entry(slot).or_default().push(id);
            return;
        }
        if let Some(&(slot, group)) = self.pending_scopes.last() {
            let anchor = Anchor {
                slot,
                group: Some(group),
                serials: [None, None],
                attributed: 0,
            };
            self.anchor_comment(anchor, id);
            return;
        }
        self.ensure_paragraph();
        let slot = self.new_slot();
        self.document.slots[slot] = Some(id);
        self.story().push(Item::RangeStart(slot));
    }

    fn end_comment(&mut self) {
        let Some(mut comment) = self.comment.take() else {
            return;
        };
        if let Some(mut anchor) = self.comment_anchor.take() {
            let whole = comment.paragraphs.join("\n");
            // A note that names another change after ` on `: the change it
            // follows was that one's only content (`A` inserted what `B`
            // deleted), so it goes back inside it, outermost last. Deeper
            // chains name what Word refuses to hold (a revision nested in
            // its own kind), so only the first outer is rebuilt.
            let sole = self.sole_change(&anchor);
            if let Some((kind, inner)) = sole
                && let Some(tags) = view_notes::nested_revision_tags(&whole)
            {
                let (author, date) = self.view_author(&tags[0].handle, tags[0].date.as_deref());
                self.document.attributions.insert(inner, (author, date));
                if !tags[0].ids.is_empty() {
                    self.document
                        .revision_ids
                        .insert(inner, tags[0].ids.clone());
                }
                if let Some(tag) = tags.get(1) {
                    let outer = (other(kind), self.tag_serial(tag));
                    self.document.wraps.insert(inner, outer);
                    self.unshown_rebuilt(&[outer.1]);
                }
                return;
            }
            if let Some((kind, inner)) = sole
                && let Some(parts) = view_notes::nested_plain_revisions(&whole)
            {
                let first = &parts[0];
                self.document
                    .attributions
                    .insert(inner, (first.author.clone(), first.date.clone()));
                if !first.ids.is_empty() {
                    self.document.revision_ids.insert(inner, first.ids.clone());
                }
                if let Some(part) = parts.get(1) {
                    self.serial += 1;
                    let outer = (other(kind), self.serial);
                    self.document
                        .attributions
                        .insert(outer.1, (part.author.clone(), part.date.clone()));
                    if !part.ids.is_empty() {
                        self.document.revision_ids.insert(outer.1, part.ids.clone());
                    }
                    self.document.wraps.insert(inner, outer);
                }
                return;
            }
            // The view's tag note: `r1+2 @AC` names both sides, `r1 @AC;
            // r2 @JD` each side in turn.
            if let Some(tags) = view_notes::revision_tags(&whole) {
                let serials: Vec<u32> = if anchor.attributed == 0 {
                    anchor.serials.iter().flatten().copied().collect()
                } else {
                    anchor.serials[1..].iter().flatten().copied().collect()
                };
                let ids = mark_ids(&tags, serials.len());
                if let ([_], [deleted, inserted]) = (tags.as_slice(), serials.as_slice()) {
                    self.shared_tags.push((*deleted, *inserted));
                }
                // A substitution is written insertion first: a dated range's
                // last day goes on the mark that comes last, its deletion.
                let mut ranged = Vec::new();
                for (i, serial) in serials.iter().enumerate() {
                    let tag = if tags.len() == 1 {
                        &tags[0]
                    } else {
                        &tags[i.min(tags.len() - 1)]
                    };
                    let (author, date) = self.view_author(&tag.handle, tag.date.as_deref());
                    self.document.attributions.insert(*serial, (author, date));
                    if tag.date.is_none()
                        && self
                            .authors
                            .get(&tag.handle)
                            .is_some_and(|known| known.date.is_none() && known.range.is_some())
                    {
                        ranged.push((tag.handle.clone(), *serial));
                    }
                    if let Some(ids) = ids.get(i).filter(|ids| !ids.is_empty()) {
                        self.document.revision_ids.insert(*serial, ids.clone());
                    }
                }
                if serials.len() == 2 {
                    ranged.reverse();
                }
                for (handle, serial) in ranged {
                    self.ranged.entry(handle).or_default().push(serial);
                }
                // One number after a substitution names its deleted side:
                // the inserted side's tag may follow (`{>>r0 @AC<<}{>>r1
                // @AS<<}` when two authors made them).
                let numbers: usize = tags.iter().map(|tag| tag.ids.len()).sum();
                anchor.attributed += 1;
                if serials.len() == 2 && numbers == 1 {
                    self.anchor = anchor.takes_attribution().then_some(anchor);
                }
                return;
            }
            // The plain view's note: the change's ids, then who made it and
            // when, absent ones left empty (`r5 Ana Lima (2026-…)`, `r7 Di
            // Eng`). The number says it is no comment on the change.
            if let Some(plain) = view_notes::plain_revision(&whole) {
                let serials: Vec<u32> = if anchor.attributed == 0 {
                    anchor.serials.iter().flatten().copied().collect()
                } else {
                    anchor.serials[1..].iter().flatten().copied().collect()
                };
                let tag = view_notes::Tag {
                    handle: String::new(),
                    date: None,
                    ids: plain.ids,
                };
                let ids = mark_ids(std::slice::from_ref(&tag), serials.len());
                if let [deleted, inserted] = serials.as_slice() {
                    self.shared_tags.push((*deleted, *inserted));
                }
                for (i, serial) in serials.iter().enumerate() {
                    self.document
                        .attributions
                        .insert(*serial, (plain.author.clone(), plain.date.clone()));
                    if let Some(ids) = ids.get(i).filter(|ids| !ids.is_empty()) {
                        self.document.revision_ids.insert(*serial, ids.clone());
                    }
                }
                anchor.attributed += 1;
                // One number after a substitution names its deleted side:
                // the inserted side's note may follow.
                if serials.len() == 2 && tag.ids.len() == 1 {
                    self.anchor = anchor.takes_attribution().then_some(anchor);
                }
                return;
            }
            if let Some((author, date)) = attribution(whole.trim()) {
                let author = author.unwrap_or_else(|| self.options.author.clone());
                let serials = if anchor.attributed == 0 {
                    &anchor.serials[..]
                } else {
                    &anchor.serials[1..]
                };
                for serial in serials.iter().flatten() {
                    self.document
                        .attributions
                        .insert(*serial, (author.clone(), date.to_string()));
                }
                anchor.attributed += 1;
                // A substitution's inserted side may name its own author next;
                // any other comment after an attribution is on its own point.
                self.anchor = anchor.takes_attribution().then_some(anchor);
                return;
            }
            if anchor.attributed == 0 {
                self.anchor_comment(anchor, comment.id);
            } else {
                // After an attribution, a comment is on its own point: a
                // comment on the change would have been `{==...==}{>>...<<}`.
                self.anchor_here(comment.id);
            }
        }
        while comment.paragraphs.len() > 1
            && comment
                .paragraphs
                .last()
                .is_some_and(|p| p.trim().is_empty())
        {
            comment.paragraphs.pop();
        }
        // The view's head (`c5 @AC: text`, `c6 @AS re c5: text`), else
        // `Name (date): text`, or `Name (date)` for a comment with no text.
        let head = comment
            .paragraphs
            .first()
            .and_then(|first| view_notes::comment_head(first.trim()))
            .map(|(head, text)| (head, text.to_string()));
        if let Some((head, text)) = head {
            // By the view's number, before renumbering can move it.
            comment.style = self.document.comment_styles.get(&head.number).cloned();
            comment.number = Some(head.number).filter(|&n| n <= view_notes::MOST_ID);
            comment.parent = head.reply_to.filter(|&n| n <= view_notes::MOST_ID);
            comment.done = head.resolved;
            if let Some(handle) = &head.handle {
                let (author, date) = self.view_author(handle, head.date.as_deref());
                comment.by = Some((author, date));
                if head.date.is_none()
                    && self
                        .authors
                        .get(handle)
                        .is_some_and(|known| known.range.is_some())
                {
                    self.ranged_comments
                        .entry(handle.clone())
                        .or_default()
                        .push(comment.id);
                }
                // A handle stands for initials unless it carries the
                // digit that tells two authors' initials apart (`U2`):
                // then the writer takes them from the name.
                if !handle.chars().any(|c| c.is_ascii_digit()) {
                    comment.initials = Some(handle.clone());
                }
            }
            comment.paragraphs[0] = text;
        } else if let Some(first) = comment.paragraphs.first_mut()
            && let Some((Some(author), date, rest)) = named(first.trim())
            && let Some(text) = rest.strip_prefix(':').or((rest.is_empty()).then_some(""))
        {
            comment.by = Some((author, date.to_string()));
            *first = text.trim_start().to_string();
        }
        self.ensure_paragraph();
        let id = comment.id;
        let mut end_at = self.range_end_at.take();
        // A range from an earlier block whose note is in a paragraph the
        // range does not run into (its id line's `in` names other comments
        // or none) ends before that paragraph, as Word wrote it.
        let runs_in = |ids: &Vec<u32>| comment.number.map_or(!ids.is_empty(), |n| ids.contains(&n));
        if end_at.is_none()
            && self
                .paragraph_in_comments
                .as_ref()
                .is_some_and(|ids| !runs_in(ids))
            && let Some(start) = self.paragraph
        {
            let mine: Vec<usize> = (0..self.document.slots.len())
                .filter(|slot| {
                    self.document.slots[*slot] == Some(id)
                        || self
                            .document
                            .shared
                            .get(slot)
                            .is_some_and(|ids| ids.contains(&id))
                })
                .collect();
            let earlier = self.story()[..start]
                .iter()
                .any(|item| matches!(item, Item::RangeStart(slot) if mine.contains(slot)));
            if earlier {
                end_at = Some(start);
            }
        }
        // A note inside a change: its reference, and its range end where
        // that stays here, are in the change's mark.
        if let Some(change) = self.change {
            self.document
                .note_changes
                .insert(id, (change, end_at.is_none()));
        }
        let story = self.story();
        match end_at {
            Some(at) if at <= story.len() => {
                story.insert(at, Item::RangeEnd(id));
                // The open paragraph's item moves along.
                if let Some(start) = self.paragraph.filter(|&start| at <= start) {
                    self.paragraph = Some(start + 1);
                }
            }
            _ => story.push(Item::RangeEnd(id)),
        }
        self.story().push(Item::CommentReference(id));
        self.document.styles.insert("CommentText");
        self.document.styles.insert("CommentReference");
        self.document.comments.push(comment);
    }

    fn note_reference(&mut self, label: &str) {
        if let Some(comment) = &mut self.comment {
            if let Some(last) = comment.paragraphs.last_mut() {
                last.push_str(&format!("[^{}]", critic::decode(label)));
            }
            return;
        }
        // Word holds no note inside a note: the reference stays text, and
        // takes none of the label's definitions.
        if let Some(target) = &self.target {
            let outer = target.split('\u{0}').next().unwrap_or_default();
            let text = format!("[^{}]", critic::decode(label));
            self.document.warnings.push(format!(
                "[^{outer}]: {text} in its text is written as text: a note holds no note"
            ));
            self.push(Content::Text(text));
            return;
        }
        let label = note_label(label);
        // A label referenced again names the same note, unless the label
        // has a definition for each reference so far.
        let uses = {
            let uses = self.label_uses.entry(label.clone()).or_insert(0);
            *uses += 1;
            *uses - 1
        };
        let definitions = self.label_definitions.get(&label).copied().unwrap_or(1);
        let label = nth_label(&label, if uses < definitions { uses } else { 0 });
        let at = match self
            .references
            .iter()
            .position(|(known, _)| *known == label)
        {
            Some(at) => at,
            None => {
                self.references.push((label, self.change));
                self.references.len() - 1
            }
        };
        let position = u32::try_from(at + 1).unwrap_or(u32::MAX);
        self.push(Content::NoteReference(position));
    }

    /// The tokens of text the writer drops or captures that still mark
    /// changes around it: CriticMarkup is read before Markdown, so a
    /// closer in dropped text still closes the span it belongs to.
    fn dropped_tokens(&mut self, text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        self.pieces.feed(text, |piece| {
            if let Piece::Token(token) = piece {
                tokens.push(token);
            }
        });
        loose_tokens(tokens)
    }

    /// [`Self::dropped_tokens`] over the text inside a raw HTML block's
    /// comments (`<!-- … -->`), which the block's reading drops.
    fn html_comment_tokens(&mut self, html: &str) -> Vec<Token> {
        let mut rest = html;
        let mut inner = String::new();
        while let Some(open) = rest.find("<!--") {
            let comment = &rest[open + 4..];
            match comment.find("-->") {
                Some(end) => {
                    inner.push_str(&comment[..end]);
                    rest = &comment[end + 3..];
                }
                None => {
                    inner.push_str(comment);
                    break;
                }
            }
        }
        self.dropped_tokens(&inner)
    }

    fn inline_html(&mut self, html: &str) {
        if self.note_clause(html) {
            return;
        }
        let Some(HtmlTag {
            name,
            closing,
            empty,
        }) = html_tag(html)
        else {
            // A comment, declaration or processing instruction: dropped,
            // but its delimiters still count.
            let tokens = self.dropped_tokens(html);
            for token in tokens {
                self.token(token);
            }
            return;
        };
        if name == "br" {
            if !closing {
                self.line_break(empty);
            }
            return;
        }
        // `<b/>` formats nothing; a comment's text is plain, so a tag in it
        // must not format the body after it.
        if empty || self.comment.is_some() {
            return;
        }
        // `<a href>` is a link; `<a>` without one (`<a name>`) is text.
        if name == "a" {
            if closing {
                if self.html_links > 0 {
                    self.html_links -= 1;
                    self.link_end();
                }
            } else if let Some(href) = html_attribute(html, "href") {
                let before = self.link;
                let title = html_attribute(html, "title");
                self.link_start(&href, title);
                self.html_links += u32::from(self.link > before);
            } else if let Some(name) =
                html_attribute(html, "id").or_else(|| html_attribute(html, "name"))
                && !name.is_empty()
                && self.comment.is_none()
                && self.image.is_none()
            {
                self.ensure_paragraph();
                self.story().push(Item::Bookmark(name));
            }
            return;
        }
        let counter = match name.as_str() {
            "b" | "strong" => &mut self.bold,
            "i" | "em" => &mut self.italic,
            "s" | "del" | "strike" => &mut self.strike,
            "u" | "ins" => &mut self.underline,
            "sup" => &mut self.superscript,
            "sub" => &mut self.subscript,
            "code" => &mut self.code,
            "mark" => &mut self.mark,
            _ => {
                if !self.dropped_html.contains(&name) {
                    self.document.warnings.push(format!(
                        "inline HTML <{name}> dropped: its text is kept, its markup is not"
                    ));
                    self.dropped_html.push(name);
                }
                return;
            }
        };
        if closing {
            *counter = counter.saturating_sub(1);
        } else {
            *counter += 1;
        }
        if name == "mark" {
            if closing {
                self.mark_color.pop();
            } else if let Some(colour) = html_attribute(html, "class")
                .as_deref()
                .and_then(|c| c.split_whitespace().find_map(hl_color_class))
            {
                self.mark_color.push(colour);
            } else {
                self.mark_color.push("yellow");
            }
        }
    }

    /// Reads the comment that closes a note's definition in the view
    /// (`[^1]: Text. <!-- rev footnotes.r11 @AL -->`), and drops the space
    /// before it, which the view writes only to set the comment apart.
    fn note_clause(&mut self, html: &str) -> bool {
        let Some(label) = self.target.clone().filter(|_| self.view) else {
            return false;
        };
        let Some(line) = html
            .trim()
            .strip_prefix("<!--")
            .and_then(|rest| rest.strip_suffix("-->"))
            .and_then(|inner| view_notes::id_line(&format!("<!-- p0 {} -->", inner.trim())))
            .filter(|line| line.style.is_none() && !(line.rev.is_empty() && line.fmt.is_empty()))
        else {
            return false;
        };
        let story = self.story();
        if let Some(Item::Run(Run {
            content: Content::Text(text),
            ..
        })) = story.last_mut()
        {
            text.truncate(text.trim_end().len());
            if text.is_empty() {
                story.pop();
            }
        }
        self.note_clauses.insert(label, line);
        true
    }

    /// Closes every inline formatting mark still open.
    /// A link to `url` (`#anchor` for a heading) starts.
    fn link_start(&mut self, url: &str, title: Option<String>) {
        if self.comment.is_some() || self.image.is_some() {
            return;
        }
        self.ensure_paragraph();
        let link = match url.strip_prefix('#') {
            Some(anchor) => Link::Anchor(anchor.to_string()),
            None => Link::External(url.to_string()),
        };
        self.story().push(Item::LinkStart(link, title));
        self.document.styles.insert("Hyperlink");
        self.link += 1;
    }

    fn link_end(&mut self) {
        if self.link > 0 && self.comment.is_none() && self.image.is_none() {
            self.link -= 1;
            self.story().push(Item::LinkEnd);
        }
    }

    fn close_inline_tags(&mut self) {
        while self.html_links > 0 {
            self.html_links -= 1;
            self.link_end();
        }
        for counter in [
            &mut self.bold,
            &mut self.italic,
            &mut self.strike,
            &mut self.underline,
            &mut self.superscript,
            &mut self.subscript,
            &mut self.code,
            &mut self.mark,
        ] {
            *counter = 0;
        }
        self.mark_color.clear();
    }

    /// `<br>`: a new paragraph of a comment, of a cell whose `<br>`s end
    /// paragraphs, or else a line break. `<br/>` (`empty`) is a line break
    /// in a cell or a note.
    fn line_break(&mut self, empty: bool) {
        if let Some(comment) = &mut self.comment {
            comment.paragraphs.push(String::new());
        } else if !empty
            && ((self.cell.is_some() && self.row_breaks)
                || (self.target.is_some() && self.note_paragraph_breaks))
        {
            // A cell's or a note's paragraphs, as the view joins them.
            self.ensure_paragraph();
            self.end_paragraph();
            self.begin_paragraph();
        } else {
            self.push(Content::Break);
        }
    }

    fn picture(&mut self) {
        let Some((url, id, alt, loose)) = self.image.take() else {
            return;
        };
        // A `drawing:` destination is a whole element and its parts,
        // spliced back as it was; a payload that does not read is alt text.
        if url.starts_with("drawing:") {
            match super::from_docx::drawing::decode(&url) {
                Ok(drawing) => {
                    self.document.drawings.push(drawing);
                    let index = self.document.drawings.len() - 1;
                    self.push(Content::Drawing(index));
                }
                Err(_) => self.plain(alt.trim()),
            }
            return;
        }
        // A `data:` destination names its bytes. A `<!-- imageN drawing:…
        // -->` backing makes it the drawing it was: the anchor, the
        // wrapping, the parts, with the `data:` bytes filling the slot the
        // readable layer carries. Without a backing it is a plain picture,
        // shared with any other `data:` picture of the same bytes.
        if let Some((_mime, b64)) = super::from_docx::drawing::data_uri(&url) {
            use super::from_docx::drawing;
            if let Some(drawing) = drawing::lookup(
                &id,
                &self.document.drawing_defs,
                &self.document.drawing_backings,
            ) {
                self.document.drawings.push(drawing);
                let index = self.document.drawings.len() - 1;
                self.push(Content::Drawing(index));
                return;
            }
            let picture =
                drawing::base64_decode(b64).and_then(|bytes| read_picture(bytes, alt.trim()));
            match picture {
                Some(picture) => {
                    let index = self
                        .document
                        .pictures
                        .iter()
                        .position(|seen| seen.bytes == picture.bytes)
                        .unwrap_or_else(|| {
                            self.document.pictures.push(picture);
                            self.document.pictures.len() - 1
                        });
                    self.push(Content::Picture(index));
                }
                None => {
                    self.document
                        .warnings
                        .push(format!("image '{id}' was written as its alt text"));
                    self.plain(alt.trim());
                }
            }
            return;
        }
        let loaded = self.options.images.map(|load| load(&url));
        let picture = loaded
            .flatten()
            .and_then(|bytes| read_picture(bytes, alt.trim()));
        match picture {
            Some(picture) => {
                self.document.pictures.push(picture);
                let index = self.document.pictures.len() - 1;
                self.push(Content::Picture(index));
            }
            None => {
                if self.options.images.is_some() {
                    self.document
                        .warnings
                        .push(format!("image '{url}' was written as its alt text"));
                }
                self.plain(alt.trim());
            }
        }
        for token in loose {
            self.token(token);
        }
    }

    /// Word lets no one comment in a footnote and shows no comment from
    /// one: a note's comment anchors on the note's first reference mark in
    /// the body instead, and the warnings say so.
    fn comments_out_of_notes(&mut self) {
        let mut moved: Vec<(u32, u32)> = Vec::new();
        for (index, note) in self.document.notes.iter_mut().enumerate() {
            let position = u32::try_from(index + 1).unwrap_or(u32::MAX);
            for item in &note.items {
                if let Item::RangeStart(slot) = item
                    && let Some(id) = self.document.slots.get_mut(*slot).and_then(Option::take)
                    && !moved.iter().any(|(_, seen)| *seen == id)
                {
                    moved.push((position, id));
                }
            }
            // The reference leaves the note, and the note's change with it.
            for item in &note.items {
                if let Item::RangeEnd(id) | Item::CommentReference(id) = item {
                    self.document.note_changes.remove(id);
                }
            }
            note.items
                .retain(|item| !matches!(item, Item::RangeEnd(_) | Item::CommentReference(_)));
        }
        for (position, id) in moved {
            let Some(at) = self.document.body.iter().position(|item| {
                matches!(item, Item::Run(Run { content: Content::NoteReference(p), .. }) if *p == position)
            }) else {
                continue;
            };
            let slot = self.new_slot();
            self.document.slots[slot] = Some(id);
            self.document.body.splice(
                at..=at,
                [
                    Item::RangeStart(slot),
                    self.document.body[at].clone(),
                    Item::RangeEnd(id),
                    Item::CommentReference(id),
                ],
            );
            self.document.warnings.push(format!(
                "comment in footnote {position} moved to its reference mark: Word shows comments only in the body"
            ));
        }
    }

    /// A section's headers and footers from the front matter, with a
    /// warning for each entry that did not read.
    fn stories(&mut self, number: usize, stories: view_stories::Stories) {
        for name in &stories.dropped {
            self.document.warnings.push(format!(
                "section {number}: front matter {name} does not read and is dropped; quote its text ('…' keeps backslashes)"
            ));
        }
        self.document.stories.insert(number, stories);
    }

    /// `title:`, `author:` and `track_changes:` from a YAML front matter
    /// block.
    fn metadata(&mut self, text: &str) {
        self.authors = view_notes::authors(text);
        for handle in view_notes::reversed_ranges(text) {
            self.document.warnings.push(format!(
                "authors: {handle}: its days run backwards (last..first), so its marks are written undated"
            ));
        }
        self.document.owner = view_notes::document_owner(text);
        if view_notes::empty_owner(text) {
            self.document.warnings.push(
                "authors: document_owner: names no one, so the document's own owner stays; \
                 write `none` to clear it"
                    .to_string(),
            );
        }
        self.document.table_style = view_styles::table_style(text);
        self.document.table_legend = view_styles::table_legend(text);
        // A legend name that holds a comma cannot be a `w:name` (Word
        // reads one as the start of an alias list), so the writer replaces
        // it with a space and says so (#549).
        for (id, name) in self
            .document
            .table_style
            .iter()
            .filter_map(|(id, _, name)| name.as_ref().map(|name| (id, name)))
            .chain(
                self.document
                    .table_legend
                    .iter()
                    .filter_map(|(id, (_, name))| name.as_ref().map(|name| (id, name))),
            )
            .filter(|(_, name)| name.contains(','))
        {
            self.document.warnings.push(format!(
                "table: {id}, named {name}: a style's w:name holds no comma, \
                 so it is written '{}'",
                name.replace(',', " ")
            ));
        }
        // One of Word's own table styles is written as Word defines it,
        // borders and all.
        if let Some((id, None, _)) = &self.document.table_style
            && !super::package::is_built_in_table_style(id)
            && text.lines().any(|line| {
                let line = view_styles::without_comment(line).trim();
                line.starts_with("table: ") && line.contains(", borders vary")
            })
        {
            self.document.warnings.push(format!(
                "table: {id}, borders vary: the view does not say the borders, \
                 so the table style is written without them"
            ));
        }
        self.stories(1, view_stories::first_section(text));
        self.document.style_specs = view_styles::legend(text);
        let mut key_above = "";
        for line in text.lines() {
            // A YAML comment (`  # …`) is no part of the value.
            let line = view_styles::without_comment(line);
            if !line.starts_with(' ') {
                key_above = line.split(':').next().unwrap_or("").trim();
            } else if key_above == "sections"
                && let Some((number, page)) = view_notes::section_page(line)
            {
                if let Some(page) = page {
                    self.document.pages.insert(number, page);
                }
                if let Some((_, entry)) = line.trim().split_once(": ") {
                    self.stories(number, view_stories::section(entry));
                    if let Some(tag) = view_notes::section_format(entry) {
                        let mark = self.format_mark(&tag);
                        self.document.section_formats.insert(number, mark);
                    }
                }
            }
            // A key under another (`options:` then `  track_changes: on`)
            // is that key's, not the document's.
            if line.starts_with([' ', '\t']) {
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let text = value.trim();
            let value = text.trim_matches(|c| c == '"' || c == '\'').trim();
            if value.is_empty() {
                if key.trim() == "track_changes" {
                    self.document
                        .warnings
                        .push("track_changes: with no value ignored: it is on or off".into());
                }
                continue;
            }
            match key.trim() {
                // A quoted scalar keeps the quotes inside it.
                "title" | "author" => {
                    let value = Some(critic::decode(&view_notes::scalar_and_comment(text).0));
                    if key.trim() == "title" {
                        self.document.title = value;
                    } else {
                        self.document.author = value;
                    }
                }
                "track_changes" => match value {
                    "on" => self.document.tracking = Some(true),
                    "off" => self.document.tracking = Some(false),
                    _ => self
                        .document
                        .warnings
                        .push(format!("track_changes: {value} ignored: it is on or off")),
                },
                "notes" => {
                    match view_notes::empty_notes(text) {
                        Ok(kinds) => self.document.empty_notes = kinds,
                        Err(why) => self.document.warnings.push(format!(
                            "notes: {why}, so the notes parts it names are not made"
                        )),
                    }
                    let formats = view_notes::note_formats(text);
                    for kind in ["footnote", "endnote"] {
                        let count = formats.iter().filter(|(k, _)| *k == kind).count();
                        if count > 2 {
                            self.document.warnings.push(format!(
                                "notes: fmt: {count} changes for the {kind}s' two separators; \
                                 the rest are not written"
                            ));
                        }
                    }
                    self.document.note_formats = formats;
                }
                "comments" => {
                    let value = view_notes::scalar_and_comment(text).0;
                    let (all, each) = view_notes::comment_styles(&value);
                    self.document.comment_style = all;
                    self.document.comment_styles = each;
                }
                "unplaced" => {
                    for (at, kind, tag) in view_notes::unplaced(text) {
                        self.unplaced.entry(at).or_default().push((kind, tag));
                    }
                }
                "style_changes" => {
                    // `{Normal: styles.r0 @AS, …}`: the view names who
                    // changed a style's formatting, not what it was.
                    if !self.resolved
                        && let Some(map) = value
                            .strip_prefix('{')
                            .and_then(|map| map.strip_suffix('}'))
                    {
                        for entry in map.split(", ") {
                            let Some((holder, tags)) = entry.split_once(": ") else {
                                continue;
                            };
                            let holder = holder.trim_matches(|c| c == '"' || c == '\'');
                            let tags = tags.trim_matches(|c| c == '"' || c == '\'');
                            let Some(tags) = view_notes::revision_tags(tags) else {
                                continue;
                            };
                            let marks = tags.iter().map(|tag| self.format_mark(tag)).collect();
                            self.document
                                .style_changes
                                .push((holder.to_string(), marks));
                        }
                    }
                }
                "view" => {
                    self.view = true;
                    self.resolved =
                        value.starts_with("accept-all") || value.starts_with("reject-all");
                }
                "page" => {
                    if let Some(page) = view_notes::page_setup(value) {
                        self.document.pages.insert(1, page);
                    }
                    if let Some(tag) = view_notes::section_format(value) {
                        let mark = self.format_mark(&tag);
                        self.document.section_formats.insert(1, mark);
                    }
                }
                _ => {}
            }
        }
        let handles: Vec<String> = self.authors.keys().cloned().collect();
        for handle in handles {
            let person = self.view_author(&handle, None);
            self.document.people.insert(handle, person);
        }
        // Word lets no one comment in a header or footer, and shows no
        // comment from one: the text stays, the comment goes. A change's
        // tag (`{>>header1.r3 @AC<<}`) is no comment.
        let mut dropped: Vec<String> = self
            .document
            .stories
            .values()
            .flat_map(|stories| &stories.stories)
            .filter(|story| {
                std::iter::once(&story.text)
                    .chain(&story.more)
                    .any(|text| has_comment(text))
            })
            .map(|story| story.part.clone())
            .collect();
        dropped.sort_unstable();
        dropped.dedup();
        for part in dropped {
            self.document.warnings.push(format!(
                "comment in {part} dropped: Word shows comments only in the body, never in a header or footer"
            ));
        }
    }

    /// A change of its own for `tag`: attributed and numbered as it says.
    fn tag_serial(&mut self, tag: &view_notes::Tag) -> u32 {
        self.serial += 1;
        let serial = self.serial;
        // An undated tag of an author dated `first..last` may be the end
        // of the range ([`Self::range_ends`]).
        if tag.date.is_none()
            && self
                .authors
                .get(&tag.handle)
                .is_some_and(|known| known.range.is_some())
        {
            self.ranged
                .entry(tag.handle.clone())
                .or_default()
                .push(serial);
        }
        let attribution = self.view_author(&tag.handle, tag.date.as_deref());
        self.document.attributions.insert(serial, attribution);
        if !tag.ids.is_empty() {
            self.document.revision_ids.insert(serial, tag.ids.clone());
        }
        serial
    }

    /// The single change a note follows, when the anchor names exactly one
    /// that is still the span the paragraph last closed: its kind and
    /// serial. A substitution's second side anchors no nested note.
    fn sole_change(&self, anchor: &Anchor) -> Option<(Kind, u32)> {
        let serials: Vec<u32> = anchor.serials.iter().flatten().copied().collect();
        self.span_start
            .filter(|span| serials.as_slice() == [span.serial])
            .map(|span| (span.kind, span.serial))
    }

    /// The paragraph's id line names a change a note rebuilt as unshown
    /// (`rev r1 @AL` after `{--x--}{>>r3 @BO on r1 @AL<<}`): its mark is
    /// written, so an empty one must not follow it.
    fn unshown_rebuilt(&mut self, nested: &[u32]) {
        let ids: Vec<u32> = nested
            .iter()
            .filter_map(|serial| self.document.revision_ids.get(serial))
            .flatten()
            .copied()
            .collect();
        if ids.is_empty() {
            return;
        }
        if let Some(at) = self.paragraph
            && let Some(Item::Paragraph(paragraph)) = self.story().get_mut(at)
        {
            paragraph
                .unshown
                .retain(|(_, mark)| mark.id.is_none_or(|id| !ids.contains(&id)));
        }
    }

    /// Fixes `paragraph`'s mark as a view line says it: `mark`, numbered and
    /// attributed as its tag says, or none.
    fn fix_mark(&mut self, paragraph: &mut Paragraph, mark: Option<(Kind, view_notes::Tag)>) {
        paragraph.mark_fixed = true;
        let Some((kind, tag)) = mark else {
            return;
        };
        let serial = self.tag_serial(&tag);
        paragraph.mark = Some(kind);
        paragraph.mark_by = Some(serial);
    }

    /// A paragraph's use of `label` (added to the document's labels once):
    /// its index in `Document::label_uses`; `None` for a label Word would
    /// read a number into (`%1`).
    fn label_use(&mut self, label: Label, level: u32, start: Option<u32>) -> Option<usize> {
        if let Label::Text(text) = &label
            && text
                .as_bytes()
                .windows(2)
                .any(|pair| pair[0] == b'%' && (b'1'..=b'9').contains(&pair[1]))
        {
            return None;
        }
        let labels = &mut self.document.labels;
        let label = labels
            .iter()
            .position(|known| *known == label)
            .unwrap_or_else(|| {
                labels.push(label);
                labels.len() - 1
            });
        let uses = &mut self.document.label_uses;
        uses.push(LabelUse {
            label,
            level: level.min(8),
            start,
        });
        Some(uses.len() - 1)
    }

    /// The author a view handle names and the date for one of their notes:
    /// the note's own, else the one the `authors:` table gives them all. An
    /// author the table knows without one date (several, printed as a day
    /// range) gets none: the note is written undated, never with the
    /// conversion's date. An unknown handle stands for itself, dated as the
    /// conversion is.
    fn view_author(&self, handle: &str, date: Option<&str>) -> (String, String) {
        let known = self.authors.get(handle);
        // `??` names no one: the mark's author is empty, as it was.
        let author = known.map_or_else(
            || {
                if handle == view_notes::UNNAMED {
                    String::new()
                } else {
                    handle.to_string()
                }
            },
            |a| a.name.clone(),
        );
        let date = match (date, known) {
            (Some(date), _) => date.to_string(),
            (None, Some(known)) => known.date.clone().unwrap_or_else(|| {
                // A range's first day; its last mark takes the last day.
                known
                    .range
                    .as_ref()
                    .map(|(first, _)| format!("{first}T00:00:00Z"))
                    .unwrap_or_default()
            }),
            (None, None) => self.options.date.clone(),
        };
        (author, date)
    }

    /// The text boxes the view prints as blocks with no id lines of their
    /// own, after the paragraph whose `rev` clause names their anchor's
    /// change: each becomes a box anchored in that paragraph, its content
    /// carrying the changes the header's `unplaced:` names for it, in
    /// order — a paragraph's mark, then its text.
    fn text_boxes(&mut self) {
        let mut unplaced = std::mem::take(&mut self.unplaced);
        let mut boxes: Vec<(usize, TextBox)> = Vec::new();
        let mut spans: Vec<(usize, usize)> = Vec::new();
        let mut at = 0;
        while at < self.body.len() {
            let Some(Item::Paragraph(paragraph)) = self.body.get(at) else {
                at += 1;
                continue;
            };
            at += 1;
            let Some((inserted, mark)) = paragraph.unshown.first().cloned() else {
                continue;
            };
            if paragraph.unshown.len() != 1 || paragraph.text_box.is_some() {
                continue;
            }
            let Some(number) = paragraph.number else {
                continue;
            };
            // The blocks after the paragraph's end, until the next
            // paragraph an id line names or a table.
            let end_of = self
                .body
                .iter()
                .skip(at)
                .position(|item| matches!(item, Item::ParagraphEnd))
                .map_or(self.body.len(), |skip| at + skip);
            let mut end = end_of + 1;
            let mut content = Vec::new();
            while let Some(item) = self.body.get(end) {
                match item {
                    Item::Paragraph(named) if named.number.is_some() => break,
                    Item::Table { .. } => break,
                    Item::ParagraphEnd | Item::RowEnd | Item::Cell { .. } if content.is_empty() => {
                        break;
                    }
                    _ => {}
                }
                content.push(item.clone());
                end += 1;
            }
            let holds_text = content
                .iter()
                .any(|item| matches!(item, Item::Paragraph(named) if named.number.is_none()));
            if !holds_text {
                continue;
            }
            // The changes the box's carriers take, in the header's order.
            let kind = if inserted { Kind::Insert } else { Kind::Delete };
            let word = if inserted { "ins" } else { "del" };
            let mut tags = unplaced
                .remove(&number)
                .unwrap_or_default()
                .into_iter()
                .filter(|(tagged, _)| *tagged == word)
                .map(|(_, tag)| tag)
                .collect::<Vec<_>>()
                .into_iter();
            for item in &mut content {
                match item {
                    Item::Paragraph(boxed) => {
                        if let Some(tag) = tags.next() {
                            let serial = self.tag_serial(&tag);
                            boxed.mark = Some(kind);
                            boxed.mark_by = Some(serial);
                            boxed.mark_fixed = true;
                        }
                    }
                    Item::Run(run) => {
                        if let Some(tag) = tags.next() {
                            run.change = Some((kind, self.tag_serial(&tag)));
                        }
                    }
                    _ => {}
                }
            }
            boxes.push((
                at - 1,
                TextBox {
                    inserted,
                    mark: mark.clone(),
                    items: content,
                },
            ));
            spans.push((end_of + 1, end));
        }
        for (at, boxed) in boxes {
            if let Some(Item::Paragraph(paragraph)) = self.body.get_mut(at) {
                paragraph.text_box = Some(boxed);
                paragraph.unshown.clear();
            }
        }
        for (start, end) in spans.into_iter().rev() {
            if end > start {
                self.body.drain(start..end);
            }
        }
        self.unplaced = unplaced;
    }

    /// A paragraph whose text prints comment marks holds them inside the
    /// mark its `rev` clause names, as Word writes it: written empty at
    /// the paragraph's end, the mark is hollow, and no change reads back
    /// from it. The paragraph's leading comment items — its marks print
    /// before its text — are bound to its one mark, which then wraps
    /// their runs.
    fn held_comment_marks(&mut self) {
        let held: Vec<(usize, Kind, FormatMark, Vec<u32>)> = self
            .body
            .iter()
            .enumerate()
            .filter_map(|(at, item)| {
                let Item::Paragraph(paragraph) = item else {
                    return None;
                };
                let [(inserted, mark)] = &paragraph.unshown[..] else {
                    return None;
                };
                let mut ids = Vec::new();
                let leading = self.body[at + 1..]
                    .iter()
                    .take_while(|item| !matches!(item, Item::ParagraphEnd))
                    .map_while(|item| match item {
                        Item::CommentReference(id) => {
                            ids.push(*id);
                            Some(true)
                        }
                        Item::RangeStart(_) | Item::RangeEnd(_) => Some(true),
                        _ => None,
                    })
                    .count()
                    > 0;
                let kind = if *inserted {
                    Kind::Insert
                } else {
                    Kind::Delete
                };
                (leading && !ids.is_empty()).then_some((at, kind, mark.clone(), ids))
            })
            .collect();
        for (at, kind, mark, ids) in held {
            self.serial += 1;
            let serial = self.serial;
            self.document
                .attributions
                .insert(serial, (mark.author.clone(), mark.date.clone()));
            if let Some(id) = mark.id {
                self.document.revision_ids.insert(serial, vec![id]);
            }
            for id in ids {
                self.document
                    .note_changes
                    .insert(id, ((kind, serial), false));
            }
            if let Some(Item::Paragraph(paragraph)) = self.body.get_mut(at) {
                paragraph.unshown.clear();
            }
        }
    }

    fn finish(mut self) -> Document {
        self.end_paragraph();
        self.textless_paragraph();
        if let Some(comment) = self.comment.take() {
            // An unclosed comment cannot happen with complete spans; keep
            // its text anyway.
            self.document.comments.push(comment);
        }
        self.text_boxes();
        self.held_comment_marks();
        let mut body = std::mem::take(&mut self.body);
        clear_continued_marks(&mut body, &self.document);
        finish_story(&mut body);
        let targets: HashSet<&str> = body
            .iter()
            .chain(self.definitions.values().flatten())
            .filter_map(|item| match item {
                Item::LinkStart(Link::Anchor(name), _) => Some(name.as_str()),
                _ => None,
            })
            .collect();
        // `<a id>` bookmarks are where their links land; no heading takes
        // their names.
        let anchors: HashSet<&str> = body
            .iter()
            .chain(self.definitions.values().flatten())
            .filter_map(|item| match item {
                Item::Bookmark(name) => Some(name.as_str()),
                _ => None,
            })
            .collect();
        let to_headings: HashSet<&str> = targets.difference(&anchors).copied().collect();
        let headings = heading_bookmarks(&body, &to_headings);
        // A link whose heading is missing still goes in; Word's jump goes
        // nowhere, so say so, once per name.
        let landed: HashSet<String> = headings
            .iter()
            .flat_map(|(_, _, named)| named.iter().cloned())
            .chain(anchors.iter().map(|name| (*name).to_string()))
            .collect();
        let mut missing: Vec<&str> = targets
            .iter()
            .copied()
            .filter(|name| !landed.contains(*name))
            .collect();
        missing.sort_unstable();
        for name in missing {
            self.document.warnings.push(format!(
                "link to #{name} names no heading: Word writes it, but it leads nowhere"
            ));
        }
        // Slugs Word cannot take as names, and the names they get instead.
        let mut renamed: HashMap<String, String> = HashMap::new();
        let mut used: HashSet<String> = headings
            .iter()
            .filter(|(_, slug, _)| word_takes(slug))
            .map(|(_, slug, _)| slug.clone())
            .chain(anchors.iter().map(|name| (*name).to_string()))
            .collect();
        for (at, slug, named) in headings {
            let name = bookmark_name(&slug, &used);
            if name != slug {
                used.insert(name.clone());
            }
            // Every name a link gave the heading now names its bookmark.
            for target in named.into_iter().chain([slug]) {
                if target != name {
                    // The first heading a name fits keeps it.
                    renamed.entry(target).or_insert_with(|| name.clone());
                }
            }
            if let Item::Paragraph(paragraph) = &mut body[at] {
                paragraph.bookmark = Some(name);
            }
        }
        if !renamed.is_empty() {
            for item in body
                .iter_mut()
                .chain(self.definitions.values_mut().flatten())
            {
                if let Item::LinkStart(Link::Anchor(anchor), _) = item
                    && let Some(name) = renamed.get(anchor.as_str())
                {
                    anchor.clone_from(name);
                }
            }
        }
        self.document.body = body;
        // A label defined more than once: its references name the
        // definitions in order, each a note of its own.
        let mut shared: Vec<(&String, &usize)> = self
            .label_definitions
            .iter()
            .filter(|(_, count)| **count > 1)
            .collect();
        shared.sort();
        for (label, &count) in shared {
            let uses = self.label_uses.get(label).copied().unwrap_or(0);
            self.document.warnings.push(if uses >= count {
                format!(
                    "[^{label}] is defined {count} times: its first {count} references name \
                     them in order, each written as a note of its own"
                )
            } else {
                format!(
                    "[^{label}] is defined {count} times but referenced {uses}: the \
                     definitions past the {uses} its references name are dropped"
                )
            });
        }
        for (label, change) in std::mem::take(&mut self.references) {
            let endnote = self.endnotes.contains(&label);
            let (text, reference) = if endnote {
                ("EndnoteText", "EndnoteReference")
            } else {
                ("FootnoteText", "FootnoteReference")
            };
            self.document.styles.insert(text);
            self.document.styles.insert(reference);
            let mut items = self.definitions.get(&label).cloned().unwrap_or_default();
            if items.is_empty() {
                items = vec![Item::Paragraph(Paragraph {
                    style: Some(text.into()),
                    ..Paragraph::default()
                })];
                items.push(Item::ParagraphEnd);
            }
            let clauses = self.note_clauses.remove(&label).unwrap_or_default();
            // A note without changes of its own is in its reference's
            // change, attribution and all. The agent view wraps a note in
            // its reference's change when Word's text is in it, so there a
            // bare definition is live text.
            let wrapped = has_changes(&items);
            if let Some((kind, serial)) = change
                && !wrapped
                && !self.view
            {
                mark_whole(&mut items, kind, serial);
            }
            // The note's reference mark goes with its text when all of it
            // is the reference's change.
            let mut change = change.filter(|(kind, _)| only_changed_by(&items, *kind));
            // Word's shape for a note inserted or deleted with its
            // reference: each paragraph's mark a change of its own, and one
            // more holding its text (the first's with the reference mark),
            // numbered in that order as the view names them.
            // A note whose paragraphs each name one mark (`rev
            // endnotes.r228`): each mark holds that paragraph's reference
            // and text together, so the note's change is the first of them
            // and its runs take their own.
            if let Some((kind, _)) = change
                && !self.resolved
                && in_one_change(&items, kind)
                && clauses.rev.len() == paragraphs(&items)
            {
                let serials: Vec<u32> =
                    clauses.rev.iter().map(|tag| self.tag_serial(tag)).collect();
                let mut next = serials.iter().copied();
                let mut runs = None;
                for item in &mut items {
                    match item {
                        Item::Paragraph(_) => runs = next.next(),
                        Item::Run(run) => run.change = runs.map(|serial| (kind, serial)),
                        _ => {}
                    }
                }
                change = serials.first().map(|&first| (kind, first));
            } else if let Some((kind, _)) = change
                && !self.resolved
                && in_one_change(&items, kind)
                && clauses.rev.len() == 2 * paragraphs(&items)
            {
                let serials: Vec<u32> =
                    clauses.rev.iter().map(|tag| self.tag_serial(tag)).collect();
                own_note_marks(&mut items, kind, &serials);
                change = serials.get(1).map(|&first| (kind, first));
            } else if let Some((kind, _)) = change
                && wrapped
                && clauses.rev.is_empty()
                && in_one_change(&items, kind)
            {
                // The view prints a note in its reference's change and names
                // no mark of its own: its text has none.
                for item in &mut items {
                    match item {
                        Item::Run(run) => run.change = None,
                        Item::Paragraph(paragraph) => {
                            paragraph.mark = None;
                            paragraph.mark_by = None;
                        }
                        _ => {}
                    }
                }
                change = None;
            } else if !self.resolved
                && let Some(Item::Paragraph(last)) = items
                    .iter_mut()
                    .rev()
                    .find(|item| matches!(item, Item::Paragraph(_)))
            {
                last.unshown = clauses
                    .rev
                    .iter()
                    .map(|tag| (!clauses.rev_del.contains(tag), self.format_mark(tag)))
                    .collect();
            }
            if !clauses.fmt.is_empty()
                && let Some(Item::Paragraph(first)) = items.first_mut()
            {
                first.formats = clauses
                    .fmt
                    .iter()
                    .map(|tag| self.format_mark(tag))
                    .collect();
            }
            finish_story(&mut items);
            self.document.notes.push(Note {
                items,
                change,
                endnote,
            });
        }
        self.share_substitution_ids();
        self.range_ends();
        self.comments_out_of_notes();
        renumber_comments(&mut self.document);
        self.document
    }

    /// An author dated `first..last` gets two timestamps back: everything
    /// undated is stamped on the first day, and one undated item on the
    /// last, when another item keeps the first: an undated formatting mark
    /// ([`Self::format_mark`]), else their last undated comment, else the
    /// last mark of their last change, else their header and footer marks.
    /// A date the view writes out stays as it is.
    fn range_ends(&mut self) {
        let mut ranged = std::mem::take(&mut self.ranged);
        let mut comments = std::mem::take(&mut self.ranged_comments);
        let mut handles: BTreeSet<String> = ranged.keys().cloned().collect();
        handles.extend(
            self.authors
                .iter()
                .filter(|(_, known)| known.range.is_some())
                .map(|(handle, _)| handle.clone()),
        );
        for handle in handles {
            let serials = ranged.remove(&handle).unwrap_or_default();
            let undated = comments.remove(&handle).unwrap_or_default();
            if self.range_ended.contains(&handle) {
                continue;
            }
            let Some((name, first, last)) = self.authors.get(&handle).and_then(|known| {
                let (first, last) = known.range.as_ref()?;
                Some((
                    known.name.clone(),
                    first.clone(),
                    format!("{last}T23:59:59Z"),
                ))
            }) else {
                continue;
            };
            // A tag dated outside the days widens them when read again.
            let last_day = &last[..first.len()];
            let outside = self
                .document
                .attributions
                .values()
                .chain(self.document.comments.iter().filter_map(|c| c.by.as_ref()))
                .filter(|(author, _)| *author == name)
                .filter_map(|(_, date)| date.get(..first.len()))
                .find(|day| *day < first.as_str() || *day > last_day)
                .map(str::to_string);
            if let Some(day) = outside {
                self.document.warnings.push(format!(
                    "authors: {handle}: a tag dated {day} falls outside {first}..{last_day}, \
                     so the days read back wider"
                ));
            }
            // The author's items on the first day, dated by the view or
            // stamped: one must stay there.
            let on_first = |date: &str| date.starts_with(first.as_str());
            let firsts = self
                .document
                .attributions
                .values()
                .filter(|(author, date)| *author == name && on_first(date))
                .count()
                + self
                    .document
                    .comments
                    .iter()
                    .filter(|comment| {
                        comment
                            .by
                            .as_ref()
                            .is_some_and(|(author, date)| *author == name && on_first(date))
                    })
                    .count();
            let marks: usize = serials
                .iter()
                .map(|s| {
                    self.document
                        .revision_ids
                        .get(s)
                        .map_or(1, |ids| ids.len().max(1))
                })
                .sum();
            let comment = undated.last().and_then(|id| {
                self.document
                    .comments
                    .iter()
                    .position(|comment| comment.id == *id)
            });
            if let Some(at) = comment
                && (firsts > 1 || marks > 0)
            {
                if let Some((_, date)) = &mut self.document.comments[at].by {
                    date.clone_from(&last);
                }
            } else if let Some(&serial) = serials.last()
                && (marks > 1 || firsts > 1)
            {
                // A change the view tags more than once (`r3` on each cell
                // of an inserted row) is written as one mark, from any of
                // its tags: each takes the last day.
                let ids = self.document.revision_ids.get(&serial).cloned();
                for &same in &serials {
                    if same == serial
                        || ids.as_ref().is_some_and(|ids| {
                            !ids.is_empty() && self.document.revision_ids.get(&same) == Some(ids)
                        })
                    {
                        self.document.range_ends.insert(same, last.clone());
                    }
                }
            } else if firsts + marks > 0 {
                // Every header and footer mark of theirs takes the last day;
                // a body item keeps the first.
                if let Some(person) = self.document.people.get_mut(&handle) {
                    person.1 = last;
                }
            } else {
                // Only headers and footers: their first mark keeps the
                // first day, the others take the last.
                self.document.story_range_ends.insert(handle, last);
            }
        }
    }
}

/// The longest bookmark name Word's schema takes.
const MOST_BOOKMARK: usize = 40;

/// Word keeps a bookmark named `slug`: one that starts with a letter, of
/// at most 40 characters.
fn word_takes(slug: &str) -> bool {
    slug.starts_with(char::is_alphabetic) && slug.chars().count() <= MOST_BOOKMARK
}

/// The bookmark name for `slug`: the slug itself when Word takes it; else
/// `_` before a slug that starts with no letter (hidden, like Word's own
/// `_Toc` bookmarks), cut to its first 40 characters, or fewer and `-1`,
/// `-2`, … when that cut is `used`.
pub(super) fn bookmark_name(slug: &str, used: &HashSet<String>) -> String {
    if word_takes(slug) {
        return slug.to_string();
    }
    let slug = if slug.starts_with(char::is_alphabetic) {
        Cow::Borrowed(slug)
    } else {
        Cow::Owned(format!("_{slug}"))
    };
    if slug.chars().count() <= MOST_BOOKMARK && !used.contains(slug.as_ref()) {
        return slug.into_owned();
    }
    (0..)
        .map(|n| {
            let suffix = if n == 0 {
                String::new()
            } else {
                format!("-{n}")
            };
            let head: String = slug.chars().take(MOST_BOOKMARK - suffix.len()).collect();
            head + &suffix
        })
        .find(|name| !used.contains(name))
        .unwrap_or_default()
}

/// Whether `name` is one [`bookmark_name`] gives `slug`, whatever names
/// were used before: the slug, its Word form, or that form cut shorter and
/// suffixed (`-1`, `-2`, …) after a collision.
pub(super) fn is_bookmark_of(name: &str, slug: &str) -> bool {
    if name == slug || name == bookmark_name(slug, &HashSet::new()) {
        return true;
    }
    if word_takes(slug) {
        return false;
    }
    let slug = if slug.starts_with(char::is_alphabetic) {
        Cow::Borrowed(slug)
    } else {
        Cow::Owned(format!("_{slug}"))
    };
    name.rsplit_once('-').is_some_and(|(head, n)| {
        !n.starts_with('0')
            && n.parse::<u32>().is_ok_and(|n| n > 0)
            && head
                .chars()
                .eq(slug.chars().take(MOST_BOOKMARK.saturating_sub(n.len() + 1)))
    })
}

/// The slug github-slugger gives a heading's text: lowercase, then every
/// character dropped that is no letter, mark, decimal or letter number,
/// connector punctuation (`_`) or `-`, and each plain space made `-`.
pub(super) fn github_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' => Some(c),
            c if c.is_alphabetic() => Some(c),
            c => match c.general_category_group() {
                GeneralCategoryGroup::Letter | GeneralCategoryGroup::Mark => Some(c),
                GeneralCategoryGroup::Number
                    if c.general_category() != GeneralCategory::OtherNumber =>
                {
                    Some(c)
                }
                GeneralCategoryGroup::Punctuation
                    if c.general_category() == GeneralCategory::ConnectorPunctuation =>
                {
                    Some(c)
                }
                _ => None,
            },
        })
        .collect()
}

/// The names github-slugger gives a document's headings, one at a time.
#[derive(Default)]
pub(super) struct Slugger {
    seen: HashMap<String, usize>,
}

impl Slugger {
    /// The name for the next heading of `slug`: a repeat retries its
    /// suffixed name against every name given out (`Intro`, `Intro`,
    /// `Intro 1`: `intro`, `intro-1`, `intro-1-1`).
    pub(super) fn name(&mut self, slug: &str) -> String {
        let mut name = slug.to_string();
        while self.seen.contains_key(&name) {
            let repeats = self.seen.entry(slug.to_string()).or_insert(0);
            *repeats += 1;
            name = format!("{slug}-{repeats}");
        }
        self.seen.insert(name.clone(), 0);
        name
    }
}

/// The headings a link names (`[x](#the-intro)`), by position in `body`,
/// each with the slug GitHub gives its text ([`github_slug`], a repeat
/// `-1`, `-2`, …). Deleted text is no part of it; an empty slug names
/// nothing.
fn heading_bookmarks(body: &[Item], targets: &HashSet<&str>) -> Vec<(usize, String, Vec<String>)> {
    let mut found = Vec::new();
    if targets.is_empty() {
        return found;
    }
    let mut slugger = Slugger::default();
    for (at, item) in body.iter().enumerate() {
        let Item::Paragraph(paragraph) = item else {
            continue;
        };
        if !paragraph.heading {
            continue;
        }
        let text: String = body[at + 1..]
            .iter()
            .take_while(|item| !matches!(item, Item::ParagraphEnd))
            .filter_map(|item| match item {
                Item::Run(Run {
                    content: Content::Text(text),
                    change,
                    ..
                }) if !matches!(change, Some((Kind::Delete, _))) => Some(text.as_str()),
                _ => None,
            })
            .collect();
        let slug = github_slug(&text);
        let name = slugger.name(&slug);
        if slug.is_empty() {
            continue;
        }
        // A link may name the heading by its slug or by a name the writer
        // gives it (cut to 40 characters, suffixed), as a docx read prints it.
        let mut named: Vec<String> = targets
            .iter()
            .filter(|target| is_bookmark_of(target, &name))
            .map(|target| (*target).to_string())
            .collect();
        if !named.is_empty() {
            named.sort_unstable();
            found.push((at, name, named));
        }
    }
    found
}

/// The `w:id`s of each of `serials` changes a revision note names: one
/// tag names them all (`r1+2`: the deletion 1, the insertion 2; a change of
/// several runs `r5+6`), each in turn and the last the rest; several tags
/// (`r1 @AC; r2 @JD`) one change each.
fn mark_ids(tags: &[view_notes::Tag], serials: usize) -> Vec<Vec<u32>> {
    match tags {
        [] => Vec::new(),
        [tag] => (0..serials)
            .map(|i| match i.cmp(&(serials - 1)) {
                std::cmp::Ordering::Less => tag.ids.get(i).copied().into_iter().collect(),
                _ => tag.ids.get(i..).map(<[u32]>::to_vec).unwrap_or_default(),
            })
            .collect(),
        tags => (0..serials)
            .map(|i| tags[i.min(tags.len() - 1)].ids.clone())
            .collect(),
    }
}

impl Writer<'_, '_> {
    /// A substitution's one tag (`r5+6+7`) lists the deleted side's ids,
    /// then the inserted side's. Word numbers marks in document order and
    /// writes the insertion first, so where the numbers drop (`r3+4+1+2`)
    /// the inserted side's begin; with no drop the deleted side takes one
    /// id per run it has (`plain _italic_` is two). Either side keeps one
    /// at least.
    fn share_substitution_ids(&mut self) {
        let mut runs: HashMap<u32, usize> = HashMap::new();
        let notes = self.document.notes.iter().flat_map(|note| &note.items);
        for item in self.document.body.iter().chain(notes) {
            if let Item::Run(Run {
                change: Some((_, by)),
                ..
            }) = item
            {
                *runs.entry(*by).or_default() += 1;
            }
        }
        for (deleted, inserted) in std::mem::take(&mut self.shared_tags) {
            let ids = &mut self.document.revision_ids;
            let mut all = ids.get(&deleted).cloned().unwrap_or_default();
            all.extend(ids.get(&inserted).cloned().unwrap_or_default());
            if all.len() < 3 {
                continue;
            }
            let drop = all.windows(2).position(|pair| pair[1] < pair[0]);
            let pieces = drop.map_or_else(|| runs.get(&deleted).copied().unwrap_or(1), |at| at + 1);
            let cut = pieces.clamp(1, all.len() - 1);
            let rest = all.split_off(cut);
            ids.insert(deleted, all);
            ids.insert(inserted, rest);
        }
    }
}

/// Comments keep the number the view gives them (`c5` is `w:id="5"`), so
/// a reply's `re c5` and every later `c5` still name the same comment; one
/// with no number, or with a number an earlier comment took, gets the
/// lowest free one. Replies then name their parent by its `w:id`.
fn renumber_comments(document: &mut Document) {
    let mut taken = HashSet::new();
    let claimed: Vec<Option<u32>> = document
        .comments
        .iter()
        .map(|c| c.number.filter(|&n| taken.insert(n)))
        .collect();
    let mut next = 0u32;
    let mut ids = HashMap::new();
    for (comment, claimed) in document.comments.iter().zip(&claimed) {
        let id = claimed.unwrap_or_else(|| {
            while taken.contains(&next) {
                next += 1;
            }
            taken.insert(next);
            next
        });
        ids.insert(comment.id, id);
    }
    let numbered: HashSet<u32> = claimed.iter().flatten().copied().collect();
    for comment in &mut document.comments {
        comment.id = ids[&comment.id];
        comment.parent = comment
            .parent
            .filter(|p| numbered.contains(p) && *p != comment.id);
    }
    for id in document
        .slots
        .iter_mut()
        .flatten()
        .chain(document.shared.values_mut().flatten())
    {
        *id = ids[id];
    }
    let notes = document
        .notes
        .iter_mut()
        .flat_map(|note| note.items.iter_mut());
    for item in document.body.iter_mut().chain(notes) {
        if let Item::RangeEnd(id) | Item::CommentReference(id) = item {
            *id = ids[id];
        }
    }
    document.note_changes = std::mem::take(&mut document.note_changes)
        .into_iter()
        .map(|(id, change)| (ids[&id], change))
        .collect();
}

/// A story's last paragraph mark cannot be inserted or deleted in Word; a
/// A paragraph changed whole whose change another paragraph continues
/// keeps no mark of its own: the view names the same ids there — one
/// revision across paragraphs, Word's one `w:id` on each — and the break
/// between them is not the revision's.
fn clear_continued_marks(items: &mut [Item], document: &Document) {
    // Each serial's paragraphs: the runs that carry it and the marks that
    // name it.
    let mut paragraphs: HashMap<u32, HashSet<usize>> = HashMap::new();
    let mut at = 0usize;
    for item in items.iter() {
        match item {
            Item::Paragraph(paragraph) => {
                at += 1;
                if let Some(serial) = paragraph.mark_by {
                    paragraphs.entry(serial).or_default().insert(at);
                }
            }
            Item::Run(run) => {
                if let Some((_, serial)) = run.change {
                    paragraphs.entry(serial).or_default().insert(at);
                }
            }
            _ => {}
        }
    }
    // A wrap's outer change is used wherever the change it holds is.
    for (inner, (_, outer)) in document.wraps.iter() {
        if let Some(ats) = paragraphs.get(inner).cloned() {
            paragraphs.entry(*outer).or_default().extend(ats);
        }
    }
    // Another serial naming any of this one's ids, used in another
    // paragraph, continues it there.
    let continued = |serial: u32, at: usize| {
        document.revision_ids.get(&serial).is_some_and(|ids| {
            document.revision_ids.iter().any(|(other, named)| {
                *other != serial
                    && named.iter().any(|id| ids.contains(id))
                    && paragraphs
                        .get(other)
                        .is_some_and(|serials| serials.iter().any(|p| *p != at))
            })
        })
    };
    let mut at = 0usize;
    for item in items.iter_mut() {
        let Item::Paragraph(paragraph) = item else {
            continue;
        };
        at += 1;
        if !paragraph.mark_fixed
            && paragraph
                .mark_by
                .is_some_and(|serial| continued(serial, at))
        {
            paragraph.mark = None;
            paragraph.mark_by = None;
            paragraph.whole = false;
        }
    }
}

/// last paragraph that was added or removed whole puts its change on the
/// mark before it instead. A story that ends in a table gets the empty
/// paragraph Word ends one with.
fn finish_story(items: &mut Vec<Item>) {
    if matches!(items.last(), Some(Item::TableEnd)) {
        // The table's own change, when every row of it has one: the closing
        // paragraph came with the table, or goes with it.
        let start = items
            .iter()
            .rposition(|item| matches!(item, Item::Table { .. }))
            .unwrap_or(0);
        let mut rows = items[start..].iter().filter_map(|item| match item {
            Item::Row { change, .. } => Some(*change),
            _ => None,
        });
        let first = rows.next().flatten();
        let change = first
            .filter(|(kind, _)| rows.all(|change| change.is_some_and(|(other, _)| other == *kind)));
        items.push(Item::Paragraph(Paragraph {
            mark: change.map(|(kind, _)| kind),
            mark_by: change.map(|(_, serial)| serial),
            whole: change.is_some(),
            ..Paragraph::default()
        }));
        items.push(Item::ParagraphEnd);
    }
    let mut depth = 0usize;
    let mut last: Option<usize> = None;
    for index in (0..items.len()).rev() {
        match &items[index] {
            Item::TableEnd => depth += 1,
            Item::Table { .. } => depth = depth.saturating_sub(1),
            Item::Paragraph(paragraph) if depth == 0 => match last {
                None => {
                    if !(paragraph.whole && paragraph.mark.is_some()) {
                        return;
                    }
                    last = Some(index);
                }
                Some(at) => {
                    let (kind, by) = match &mut items[at] {
                        Item::Paragraph(ending) => {
                            ending.whole = false;
                            (ending.mark.take(), ending.mark_by.take())
                        }
                        _ => (None, None),
                    };
                    let Item::Paragraph(before) = &mut items[index] else {
                        return;
                    };
                    if before.mark_fixed {
                        // Its id line said what its mark is.
                    } else if before.mark.is_none() {
                        before.mark = kind;
                        before.mark_by = by;
                    } else if before.whole && before.mark == kind {
                        // Paragraphs added (or removed) whole up to the end:
                        // the mark before the first of them takes the change.
                        let earlier = items[..index].iter().rposition(|item| match item {
                            Item::Paragraph(p) => !(p.whole && p.mark == kind),
                            _ => false,
                        });
                        if let Some(earlier) = earlier
                            && let Item::Paragraph(p) = &mut items[earlier]
                            && p.mark.is_none()
                            && !p.mark_fixed
                        {
                            p.mark = kind;
                            p.mark_by = by;
                        }
                    } else if before.whole && before.mark != kind && at == end_of(items, index) + 1
                    {
                        // A paragraph removed whole, then the last one added
                        // whole (or the other way round): the last paragraph
                        // is replaced, so both texts go in it.
                        let content: Vec<Item> = items.drain(index + 1..at - 1).collect();
                        items.drain(index..index + 2);
                        let last = index;
                        items.splice(last + 1..last + 1, content);
                    }
                    return;
                }
            },
            _ => {}
        }
    }
    if let Some(at) = last
        && let Item::Paragraph(only) = &mut items[at]
    {
        only.mark = None;
        only.mark_by = None;
        only.whole = false;
    }
}

/// The kind of change nested inside one of `kind`: the other one.
fn other(kind: Kind) -> Kind {
    match kind {
        Kind::Insert => Kind::Delete,
        Kind::Delete => Kind::Insert,
    }
}

/// The author and date of an attribution, a comment that is only
/// `Name (date)` or `(date)` (the date ISO 8601, as Word writes it): how
/// CriticMarkup records who made the change before it, and how a document
/// read from Word writes it.
fn attribution(text: &str) -> Option<(Option<String>, &str)> {
    match named(text)? {
        (author, date, "") => Some((author, date)),
        _ => None,
    }
}

/// `Name (date)` or `(date)` at the start of `text`: the name, the date and
/// the text after them. The date is the first parenthesised date after a
/// name whose own parentheses balance, for an author may hold some
/// (`Ana (consultant)`) and a comment's text may hold a date; in a fold
/// note they come percent-spelled (`%28`, `%29`, with `%` itself `%25`)
/// and are read back off the name.
pub(super) fn named(text: &str) -> Option<(Option<String>, &str, &str)> {
    // The first group that is a date after a name: a later one belongs to
    // the text after the attribution (`Ana (d1): moved to (d2)`), and an
    // earlier one that is no date to the name (`Ana (Legal) (d1)`).
    let mut from = 0;
    while let Some(open) = text[from..].find('(').map(|at| from + at) {
        from = open + 1;
        let Some(close) = text[open..].find(')').map(|at| open + at) else {
            continue;
        };
        let date = &text[open + 1..close];
        if !is_date(date) {
            continue;
        }
        let name = &text[..open];
        let author = match name.trim() {
            "" if name.is_empty() => None,
            "" => return None,
            author if name.ends_with(' ') => Some(unescaped(author).to_string()),
            _ => continue,
        };
        // The name's own parentheses, when it holds any, must balance:
        // `A (b) (date)` names `A (b)`, `Ad)a (date)` names nothing.
        let mut depth = 0i64;
        let mut unbalanced = false;
        for c in text[..open].chars() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    unbalanced |= depth < 0;
                }
                _ => {}
            }
        }
        if unbalanced || depth != 0 {
            return None;
        }
        return Some((author, date, &text[close + 1..]));
    }
    None
}

/// A name with its percent-spelled parentheses (`%28`, `%29`), backslash
/// (`%5C`) and `%` (`%25`) read back off, as a fold note writes them;
/// anything else a `%` starts stays as it is.
fn unescaped(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut rest = name;
    while let Some(at) = rest.find('%') {
        out.push_str(&rest[..at]);
        let spelled = match rest.get(at + 1..at + 3) {
            Some("25") => Some('%'),
            Some("28") => Some('('),
            Some("29") => Some(')'),
            Some("5C") | Some("5c") => Some('\\'),
            _ => None,
        };
        match spelled {
            Some(c) => out.push(c),
            None => out.push('%'),
        }
        rest = &rest[at + 1 + usize::from(spelled.is_some()) * 2..];
    }
    out.push_str(rest);
    out
}

/// `YYYY-MM-DDTHH:MM`, then optional seconds and fraction, then an
/// optional `Z` or `+HH:MM` offset.
pub(super) fn is_date(text: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let Some((day_text, time)) = text.split_once('T') else {
        return false;
    };
    let day: Vec<&str> = day_text.split('-').collect();
    if day.len() != 3
        || day[0].len() != 4
        || day[1..].iter().any(|p| p.len() != 2)
        || !day.iter().all(|p| digits(p))
    {
        return false;
    }
    let (clock_text, zone) = match time.find(['Z', '+', '-']) {
        Some(at) => time.split_at(at),
        None => (time, ""),
    };
    let zone_ok = match zone {
        "" | "Z" => true,
        zone => {
            let offset = &zone[1..];
            offset.len() == 5
                && offset.as_bytes()[2] == b':'
                && digits(&offset[..2])
                && digits(&offset[3..])
        }
    };
    let clock: Vec<&str> = clock_text.split(':').collect();
    let seconds_ok = clock.get(2).is_none_or(|seconds| {
        let (whole, fraction) = seconds.split_once('.').unwrap_or((seconds, "0"));
        whole.len() == 2 && digits(whole) && digits(fraction)
    });
    let shaped = zone_ok
        && (2..=3).contains(&clock.len())
        && clock[..2].iter().all(|p| p.len() == 2 && digits(p))
        && seconds_ok;
    // In range too (no 2026-02-30, no 25:00): Word refuses such a w:date.
    let seconds = if clock.len() == 2 { ":00" } else { "" };
    shaped
        && crate::util::datetime::parse_datetime(&format!("{day_text}T{clock_text}{seconds}{zone}"))
            .is_ok()
}

/// Whether no table starts or ends between two story indexes.
fn no_table_between(items: &[Item], from: usize, to: usize) -> bool {
    !items[from..to]
        .iter()
        .any(|item| matches!(item, Item::Table { .. } | Item::TableEnd))
}

/// The index of the `ParagraphEnd` of the paragraph that starts at `start`.
fn end_of(items: &[Item], start: usize) -> usize {
    items[start..]
        .iter()
        .position(|item| matches!(item, Item::ParagraphEnd))
        .map_or(items.len(), |offset| start + offset)
}

/// Whether every run of `items` is in a change of `kind`.
fn only_changed_by(items: &[Item], kind: Kind) -> bool {
    items.iter().all(|item| match item {
        Item::Run(run) => run.change.is_some_and(|(k, _)| k == kind),
        _ => true,
    })
}

/// Whether every run and every paragraph mark but the last of a note is in
/// one change of `kind`.
fn in_one_change(items: &[Item], kind: Kind) -> bool {
    let last = items
        .iter()
        .rposition(|item| matches!(item, Item::Paragraph(_)));
    let mut serial = None;
    items.iter().enumerate().all(|(index, item)| {
        let change = match item {
            Item::Run(run) => run.change,
            Item::Paragraph(paragraph) if Some(index) != last => {
                paragraph.mark.zip(paragraph.mark_by)
            }
            _ => return true,
        };
        change.is_some_and(|(own, by)| own == kind && *serial.get_or_insert(by) == by)
    })
}

fn paragraphs(items: &[Item]) -> usize {
    items
        .iter()
        .filter(|item| matches!(item, Item::Paragraph(_)))
        .count()
}

/// Gives a note in one change Word's own marks: paragraph by paragraph,
/// its mark the change `serials` names first and its runs the next.
fn own_note_marks(items: &mut [Item], kind: Kind, serials: &[u32]) {
    let mut next = serials.iter().copied();
    let mut runs = None;
    for item in items {
        match item {
            Item::Paragraph(paragraph) => {
                paragraph.mark = Some(kind);
                paragraph.mark_by = next.next();
                paragraph.mark_fixed = true;
                runs = next.next();
            }
            Item::Run(run) => run.change = runs.map(|serial| (kind, serial)),
            _ => {}
        }
    }
}

fn has_changes(items: &[Item]) -> bool {
    items.iter().any(|item| match item {
        Item::Run(run) => run.change.is_some(),
        Item::Paragraph(paragraph) => paragraph.mark.is_some(),
        _ => false,
    })
}

/// Puts a whole footnote in the change of its reference: its text and every
/// paragraph mark but the last, which the note keeps.
fn mark_whole(items: &mut [Item], kind: Kind, serial: u32) {
    let last = items
        .iter()
        .rposition(|item| matches!(item, Item::Paragraph(_)));
    for (index, item) in items.iter_mut().enumerate() {
        match item {
            Item::Run(run) => run.change = Some((kind, serial)),
            Item::Paragraph(paragraph) if Some(index) != last => {
                paragraph.mark = Some(kind);
                paragraph.mark_by = Some(serial);
            }
            _ => {}
        }
    }
}

fn heading_level(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn alignment(alignment: Alignment) -> Option<&'static str> {
    match alignment {
        Alignment::None => None,
        Alignment::Left => Some("left"),
        Alignment::Center => Some("center"),
        Alignment::Right => Some("right"),
    }
}

/// A picture Word can show, sized at 96 dots per inch and at most the text
/// width.
pub(crate) fn read_picture(bytes: Vec<u8>, alt: &str) -> Option<Picture> {
    use image::ImageFormat;
    let reader = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .ok()?;
    let (extension, content_type) = match reader.format()? {
        ImageFormat::Png => ("png", "image/png"),
        ImageFormat::Jpeg => ("jpeg", "image/jpeg"),
        ImageFormat::Gif => ("gif", "image/gif"),
        ImageFormat::Bmp => ("bmp", "image/bmp"),
        ImageFormat::Tiff => ("tiff", "image/tiff"),
        _ => return None,
    };
    let (width, height) = reader.into_dimensions().ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    let mut cx = u64::from(width) * 9525;
    let mut cy = u64::from(height) * 9525;
    if cx > MAX_PICTURE_WIDTH {
        cy = cy * MAX_PICTURE_WIDTH / cx;
        cx = MAX_PICTURE_WIDTH;
    }
    Some(Picture {
        bytes,
        extension,
        content_type,
        width: cx,
        height: cy.max(1),
        alt: alt.to_string(),
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn a_comment_body_holding_a_date_keeps_its_author() {
        // The date is the last parenthesised group of the attribution, not
        // of the comment's body: `(D2)` in the text names no one.
        assert_eq!(
            named("Ana (2026-01-01T00:00:00Z): moved to (2026-02-01T00:00:00Z)"),
            Some((
                Some("Ana".to_string()),
                "2026-01-01T00:00:00Z",
                ": moved to (2026-02-01T00:00:00Z)"
            ))
        );
        assert_eq!(
            named("Ana (Legal) (2026-01-01T00:00:00Z): see (2026-02-01T00:00:00Z)"),
            Some((
                Some("Ana (Legal)".to_string()),
                "2026-01-01T00:00:00Z",
                ": see (2026-02-01T00:00:00Z)"
            ))
        );
    }

    #[test]
    fn even_backslash_runs_move_off_the_pipes_they_shield() {
        // GFM reads an even backslash run before a table row's pipe as
        // escaped backslashes and then the cell separator, but
        // pulldown-cmark shields a pipe behind any backslash: the run moves
        // off it by a space, which is not a cell's text. An odd run escapes
        // the pipe, which is the cell's own.
        assert_eq!(unshield_pipes(r"|\\|\\\\|"), r"|\\ |\\\\ |");
        assert_eq!(unshield_pipes(r"a\|b \\| c"), r"a\|b \\ | c");
        assert_eq!(unshield_pipes(r"|\\\|x"), r"|\\\|x");
        // A pipe inside a code span, an equation or an HTML tag is part of
        // the cell, backslashes or not: a space there would change its
        // text, so everything is left as written.
        let spans = "|`a\\\\|b` x|$a\\\\|b$|<i t=\"a\\\\|b\"> y|";
        assert_eq!(unshield_pipes(spans), spans);
    }

    #[test]
    fn a_rows_pipes_unshield_only_where_a_table_matches_them() {
        // The rows of tables whose columns, runs moved off their pipes,
        // match the delimiter row; a row that opens none keeps its bytes,
        // and so does everything that is not a table row at all.
        assert_eq!(
            table_pipes("|\\\\|\\\\\\\\|\n|-|-|\n"),
            "|\\\\ |\\\\\\\\ |\n|-|-|\n"
        );
        assert_eq!(table_pipes("|\\\\|x|y\n|-|-|\n"), "|\\\\|x|y\n|-|-|\n");
        assert_eq!(table_pipes("\\\\|x\n"), "\\\\|x\n");
        assert_eq!(table_pipes("    \\\\|x\n"), "    \\\\|x\n");
        assert_eq!(table_pipes("```\n\\\\|x\n```\n"), "```\n\\\\|x\n```\n");
        assert_eq!(
            table_pipes("---\ntext: |\n  \\\\|x\n---\n"),
            "---\ntext: |\n  \\\\|x\n---\n"
        );
    }

    #[test]
    fn dates_are_iso_8601_to_the_minute_or_finer() {
        for date in [
            "2026-01-02T03:04",
            "2026-01-02T03:04:05",
            "2026-01-02T03:04:05Z",
            "2026-01-02T03:04:05.123Z",
            "2026-01-02T03:04:05+01:00",
            "2026-01-02T03:04-05:30",
            "2024-02-29T23:59:59Z",
            "2026-12-31T00:00+14:00",
        ] {
            assert!(is_date(date), "{date}");
        }
        for text in [
            "",
            "2026-01-02",
            "2026-1-02T03:04",
            "26-01-02T03:04",
            "2026-01-02T3:04",
            "2026-01-02T03:04:5",
            "2026-01-02T03:04:05.Z",
            "2026-01-02T03:04:05+0100",
            "2026-01-02T03:04:05+01:0x",
            "2026-01-02T03:04:05:06",
            "yesterday",
            // The shape of a date, out of range.
            "2026-02-30T00:00:00Z",
            "2025-02-29T00:00",
            "2026-13-01T00:00Z",
            "2026-00-01T00:00Z",
            "2026-01-00T00:00Z",
            "2026-01-02T24:00:00Z",
            "2026-01-02T03:60Z",
            "2026-01-02T03:04:60Z",
            "2026-01-02T03:04+15:00",
            "2026-99-99T99:99:99Z",
        ] {
            assert!(!is_date(text), "{text}");
        }
    }

    #[test]
    fn a_revision_note_numbers_the_marks_of_its_changes() {
        let tag = |ids: &[u32]| view_notes::Tag {
            handle: "AC".into(),
            date: None,
            ids: ids.to_vec(),
        };
        // A substitution: the deletion, then the insertion.
        assert_eq!(mark_ids(&[tag(&[1, 2])], 2), [vec![1], vec![2]]);
        // One change of several runs takes every id.
        assert_eq!(mark_ids(&[tag(&[5, 6])], 1), [vec![5, 6]]);
        // The insertion of a substitution in two runs takes the rest.
        assert_eq!(mark_ids(&[tag(&[1, 2, 3])], 2), [vec![1], vec![2, 3]]);
        // Too few ids: the changes left over take none.
        assert_eq!(mark_ids(&[tag(&[4])], 2), [vec![4], vec![]]);
        // One tag per change.
        assert_eq!(mark_ids(&[tag(&[1]), tag(&[2])], 2), [vec![1], vec![2]]);
        assert!(mark_ids(&[], 2).is_empty());
        assert!(mark_ids(&[tag(&[1])], 0).is_empty());
    }

    #[test]
    fn names_and_attributions() {
        assert_eq!(
            named("Ana Lima (2026-01-02T03:04Z): hi"),
            Some((Some("Ana Lima".to_string()), "2026-01-02T03:04Z", ": hi"))
        );
        assert_eq!(
            named("(2026-01-02T03:04Z)"),
            Some((None, "2026-01-02T03:04Z", ""))
        );
        for text in [
            "Ana(2026-01-02T03:04Z)",
            " (2026-01-02T03:04Z)",
            "Ad)a (2026-01-02T03:04Z)",
            "Ana (soon)",
            "Ana 2026-01-02T03:04Z",
        ] {
            assert_eq!(named(text), None, "{text}");
        }
        // The date is the last parenthesised group: an author may hold
        // parentheses of its own, spelled percent-wise in a fold note.
        assert_eq!(
            named("A (b) (2026-01-02T03:04Z)"),
            Some((Some("A (b)".to_string()), "2026-01-02T03:04Z", ""))
        );
        assert_eq!(
            named("A %28b%29 (2026-01-02T03:04Z)"),
            Some((Some("A (b)".to_string()), "2026-01-02T03:04Z", ""))
        );
        assert_eq!(
            attribution("Bo (2026-01-02T03:04:05Z)"),
            Some((Some("Bo".to_string()), "2026-01-02T03:04:05Z"))
        );
        assert_eq!(attribution("Bo (2026-01-02T03:04:05Z): why"), None);
        assert_eq!(attribution("Bo"), None);
    }

    /// A substitution whose start and separator sit in text the writer
    /// drops or captures, with its end outside, keeps its separator in
    /// the replayed tokens: without it the text after the drop reads as
    /// the old half. A substitution whole inside drops with the text.
    #[test]
    fn a_substitution_split_by_dropped_text_keeps_its_separator() {
        assert_eq!(
            loose_tokens(vec![Token::SubstituteStart, Token::SubstituteSeparator,]),
            vec![Token::SubstituteStart, Token::SubstituteSeparator]
        );
        // Whole inside: start, separator and end all drop with the text.
        assert_eq!(
            loose_tokens(vec![
                Token::SubstituteStart,
                Token::SubstituteSeparator,
                Token::SubstituteEnd,
            ]),
            vec![]
        );
        // A closed substitution inside other dropped tokens keeps only
        // the tokens that cross the edge.
        assert_eq!(
            loose_tokens(vec![
                Token::InsertEnd,
                Token::SubstituteStart,
                Token::SubstituteSeparator,
                Token::SubstituteEnd,
            ]),
            vec![Token::InsertEnd]
        );
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod markdown_authored_boundary_tests {
    use super::*;

    #[test]
    fn malformed_iso_components_cannot_become_revision_attributions() {
        let invalid = [
            "2026-10T12:34",
            "2026-10-09-01T12:34",
            "xxxx-10-09T12:34",
            "2026-xx-09T12:34",
            "2026-10-xxT12:34",
            "2026-10-09T12:34:ab",
            "2026-10-09T12:34:01.ab",
            "2026-10-09T12:34+01x30",
            "2026-10-09T12:34+ab:30",
            "2026-10-09T12:34+01:ab",
            "2026-10-09T12:34+",
            "2026-10-09T12:34:01:02",
            "2026-10-09T12:34:01.",
            "2026-10-09Txx:34",
            "2026-10-09T12:xx",
        ];
        for date in invalid {
            assert!(!is_date(date), "{date}");
            assert_eq!(
                named(&format!("Ada ({date}): owned comment")),
                None,
                "{date}"
            );
            assert_eq!(attribution(&format!("Ada ({date})")), None, "{date}");
        }
        for date in [
            "2026-10-09T12:34",
            "2026-10-09T12:34:00.5-03:30",
            "2026-10-09T12:34+00:00",
        ] {
            assert!(is_date(date), "{date}");
            let input = format!("Ada ({date}): owned comment");
            assert_eq!(
                named(&input),
                Some((Some("Ada".to_string()), date, ": owned comment"))
            );
            assert_eq!(attribution(&input), None);
        }
        let date = "2026-10-09T12:34Z";
        for author in ["Ad)a ", "Ada", " ", "\t"] {
            assert_eq!(named(&format!("{author}({date})")), None, "{author:?}");
        }
    }

    #[test]
    fn front_matter_quote_variants_preserve_decoded_authored_metadata() {
        for (title, author) in [
            ("'Agreement'", "'Ada'"),
            ("\"Agreement\"", "\"Ada\""),
            ("Agreement", "Ada"),
        ] {
            let source = format!(
                "---\ntitle: {title}\nauthor: {author}\nignored: extra\nempty: ''\n---\n\nBody"
            );
            let document = read(&source, &DocxOptions::default());
            assert_eq!(document.title.as_deref(), Some("Agreement"));
            assert_eq!(document.author.as_deref(), Some("Ada"));
            assert_eq!(
                document.body,
                vec![
                    Item::Paragraph(Paragraph::default()),
                    Item::Run(Run {
                        content: Content::Text("Body".to_string()),
                        format: RunFormat::default(),
                        change: None,
                        scope: None
                    }),
                    Item::ParagraphEnd,
                ]
            );
            assert!(document.comments.is_empty());
            assert!(document.warnings.is_empty());
        }
    }

    #[test]
    fn comment_links_and_non_attribution_suffixes_keep_the_complete_comment() {
        let date = "2026-10-09T12:34Z";
        for (source, expected, by) in [
            (
                "Body{>>[owned](https://example.invalid) tail<<}".to_string(),
                "owned tail".to_string(),
                None,
            ),
            (
                format!("Body{{>>Ada ({date}) suffix<<}}"),
                format!("Ada ({date}) suffix"),
                None,
            ),
            (
                format!("Body{{>>Ada ({date}): owned text<<}}"),
                "owned text".to_string(),
                Some(("Ada".to_string(), date.to_string())),
            ),
            (
                "Body{>>one<br>two<<}".to_string(),
                "one\ntwo".to_string(),
                None,
            ),
        ] {
            let document = read(&source, &DocxOptions::default());
            assert_eq!(document.comments.len(), 1, "{source}");
            let comment = &document.comments[0];
            assert_eq!(comment.id, 0);
            assert_eq!(comment.paragraphs.join("\n"), expected, "{source}");
            assert_eq!(comment.by, by, "{source}");
            assert_eq!(
                document
                    .body
                    .iter()
                    .filter_map(|item| match item {
                        Item::Run(run) => Some(&run.content),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                vec![&Content::Text("Body".to_string())]
            );
            assert_eq!(
                document
                    .body
                    .iter()
                    .filter(|item| matches!(item, Item::CommentReference(0)))
                    .count(),
                1
            );
            assert_eq!(
                document
                    .body
                    .iter()
                    .filter(|item| matches!(item, Item::LinkStart(..) | Item::LinkEnd))
                    .count(),
                0
            );
            assert!(document.attributions.is_empty());
            assert!(document.warnings.is_empty());
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod memory_picture_import_contract_tests {
    use super::*;

    fn encoded(format: image::ImageFormat, width: u32, height: u32) -> Vec<u8> {
        let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            width,
            height,
            image::Rgb([31, 97, 163]),
        ));
        let mut out = std::io::Cursor::new(Vec::new());
        image.write_to(&mut out, format).unwrap();
        out.into_inner()
    }

    #[test]
    fn memory_picture_formats_keep_exact_payload_alt_and_capped_aspect_ratio() {
        for (format, extension, mime) in [
            (image::ImageFormat::Png, "png", "image/png"),
            (image::ImageFormat::Jpeg, "jpeg", "image/jpeg"),
            (image::ImageFormat::Gif, "gif", "image/gif"),
            (image::ImageFormat::Bmp, "bmp", "image/bmp"),
            (image::ImageFormat::Tiff, "tiff", "image/tiff"),
        ] {
            for (width, height, cx, cy) in [
                (1, 2, 9525, 19050),
                (624, 1, 5_943_600, 9525),
                (625, 2, 5_943_600, 19019),
            ] {
                let bytes = encoded(format, width, height);
                let got = read_picture(bytes.clone(), "owned Ω alt").unwrap();
                assert_eq!(got.bytes, bytes);
                assert_eq!(got.extension, extension);
                assert_eq!(got.content_type, mime);
                assert_eq!((got.width, got.height), (cx, cy));
                assert_eq!(got.alt, "owned Ω alt");
            }
        }
    }

    #[test]
    fn rejected_memory_images_preserve_alt_text_and_report_only_requested_load_failures() {
        for bytes in [
            Vec::new(),
            b"not an image".to_vec(),
            b"\x89PNG\r\n\x1a\n".to_vec(),
            b"RIFF\x04\0\0\0WEBP".to_vec(),
        ] {
            assert!(read_picture(bytes.clone(), "owned").is_none());
            let load = |url: &str| {
                assert_eq!(url, "owned.png");
                Some(bytes.clone())
            };
            let options = DocxOptions {
                images: Some(&load),
                ..DocxOptions::default()
            };
            let doc = read("before ![owned Ω](owned.png) after", &options);
            assert!(doc.pictures.is_empty());
            assert_eq!(
                doc.warnings,
                ["image 'owned.png' was written as its alt text"]
            );
            let text: String = doc
                .body
                .iter()
                .filter_map(|item| match item {
                    Item::Run(Run {
                        content: Content::Text(text),
                        ..
                    }) => Some(text.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(text, "before owned Ω after");
        }
        let no_load = read("![owned Ω](owned.png)", &DocxOptions::default());
        assert!(no_load.warnings.is_empty());
        assert!(no_load.pictures.is_empty());
        assert!(no_load.body.iter().any(|item| matches!(item, Item::Run(Run { content: Content::Text(text), .. }) if text == "owned Ω")));
        let load = |_: &str| None;
        let missing = read(
            "![owned Ω](owned.png)",
            &DocxOptions {
                images: Some(&load),
                ..DocxOptions::default()
            },
        );
        assert_eq!(
            missing.warnings,
            ["image 'owned.png' was written as its alt text"]
        );
        assert!(missing.pictures.is_empty());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod html_tag_tests {
    use super::*;

    #[test]
    fn a_paragraph_break_is_br_by_name_and_not_self_closing() {
        for br in ["<br>", "<BR>", "<br class=\"x\">", "< br >"] {
            assert!(is_paragraph_break(br), "{br}");
        }
        for not in ["<br/>", "<BR />", "</br>", "<bra>", "<br-x>", "<b>"] {
            assert!(!is_paragraph_break(not), "{not}");
        }
    }
}
