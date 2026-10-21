// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The view's own notes read back: the YAML `authors:` table
//! (`AC: Ann Counsel  # 9 revisions, 2026-10-01T09:00:00Z`), a revision's
//! tag note (`r1+2 @AC`, `r1 @AC; r2 @JD`, `header1.r4 @AC 2026-…`) and a
//! comment's head (`c5 @AC: `, `c6 @AS re c5: `, `c7 @AC resolved: `).
//! `jubarte read` writes them (`from_docx::agent`, `from_docx::header`).

use std::collections::{BTreeMap, HashMap};

use super::xml::Label;

/// An author of the `authors:` table: the full name, and the timestamp
/// every one of their changes and comments shares, when they share one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Author {
    pub name: String,
    pub date: Option<String>,
    /// The first and last days of an author whose marks have several
    /// timestamps (`# 2 revisions, 2026-03-02..2026-03-04`).
    pub range: Option<(String, String)>,
}

/// The `authors:` table of a YAML header, by handle. `document_owner` is
/// not an author.
pub(super) fn authors(yaml: &str) -> HashMap<String, Author> {
    let mut table = HashMap::new();
    let mut inside = false;
    for line in yaml.lines() {
        // A blank or comment line does not end the block, as in YAML.
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            inside = opens_authors(line);
            continue;
        }
        if !inside {
            continue;
        }
        let Some((handle, rest)) = line.trim_start().split_once(':') else {
            continue;
        };
        if handle == "document_owner" || !is_handle(handle) {
            continue;
        }
        let (value, comment) = scalar_and_comment(rest.trim_start());
        // `??: ""` is the empty author's line: kept, so its marks keep the
        // date it says, or none.
        if value.is_empty() && handle != UNNAMED {
            continue;
        }
        let last = comment.and_then(|c| c.rsplit(',').next()).map(str::trim);
        // A bare day (`2026-03-02`) is that day's midnight.
        let date = last.and_then(|d| {
            if super::write::is_date(d) {
                Some(d.to_string())
            } else {
                let midnight = format!("{d}T00:00:00Z");
                super::write::is_date(&midnight).then_some(midnight)
            }
        });
        let range = last.and_then(day_range);
        table.insert(
            handle.to_string(),
            Author {
                name: value,
                date,
                range,
            },
        );
    }
    table
}

/// The formatting change a page or section entry names: `Letter
/// portrait, margins 1in; fmt r17 @AL` or `{p2-p9, fmt: r5 @AC, …}`.
pub(super) fn section_format(text: &str) -> Option<Tag> {
    let at = text
        .find("; fmt ")
        .map(|at| at + 6)
        .or_else(|| text.find("fmt: ").map(|at| at + 5))?;
    let rest = &text[at..];
    let rest = rest.strip_prefix('"').map_or(rest, |quoted| quoted);
    let end = [
        ", headers: ",
        ", footers: ",
        ", columns: ",
        ", page: ",
        "\"",
        "}",
    ]
    .iter()
    .filter_map(|key| rest.find(key))
    .min()
    .unwrap_or(rest.len());
    first_tag(rest[..end].trim())
}

/// The handles whose `authors:` note dates them with a range that ends
/// before it starts (`2026-03-04..2026-03-02`): no date is read for them.
pub(super) fn reversed_ranges(yaml: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut inside = false;
    for line in yaml.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            inside = opens_authors(line);
            continue;
        }
        let Some((handle, rest)) = line.trim_start().split_once(':').filter(|_| inside) else {
            continue;
        };
        let (_, comment) = scalar_and_comment(rest.trim_start());
        if let Some((first, last)) = comment
            .and_then(|c| c.rsplit(',').next())
            .map(str::trim)
            .and_then(|d| d.split_once(".."))
            && day_range(&format!("{last}..{first}")).is_some()
            && first != last
        {
            found.push(handle.to_string());
        }
    }
    found
}

/// What a Markdown file's front matter gives every comment and change
/// that names none of its own: the top-level `author:`, and the file's
/// date (`last_modified:`, else `modified:`, `date:`, `created:`).
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct FileDefaults {
    pub author: Option<String>,
    pub date: Option<String>,
    /// A date key whose value is no timestamp: `(key, value)`.
    pub unread: Option<(String, String)>,
}

/// The front matter's [`FileDefaults`]; none without front matter.
pub(super) fn file_defaults(markdown: &str) -> FileDefaults {
    const DATES: [&str; 4] = ["last_modified", "modified", "date", "created"];
    let mut defaults = FileDefaults::default();
    let text = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let Some(rest) = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
    else {
        return defaults;
    };
    let lines: Vec<&str> = rest.lines().map(|l| l.trim_end_matches('\r')).collect();
    // Front matter ends at its closing `---` or `...`; a file that only
    // opens with a thematic break has none.
    let Some(end) = lines.iter().position(|l| *l == "---" || *l == "...") else {
        return defaults;
    };
    let mut dates: Vec<(usize, String, String)> = Vec::new();
    for line in &lines[..end] {
        if line.starts_with([' ', '\t', '#']) {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        // `author: ~` or `null` names no one, as `document_owner:` does.
        if key == "author" && names_no_owner(value.trim_start()) {
            continue;
        }
        let (value, _) = scalar_and_comment(value.trim_start());
        let value = super::critic::decode(&value);
        if value.is_empty() {
            continue;
        }
        if key == "author" {
            defaults.author.get_or_insert(value);
        } else if let Some(rank) = DATES.iter().position(|d| *d == key) {
            dates.push((rank, key.to_string(), value));
        }
    }
    dates.sort_by_key(|(rank, _, _)| *rank);
    for (_, key, value) in dates {
        let midnight = format!("{value}T00:00:00Z");
        if super::write::is_date(&value) {
            defaults.date = Some(value);
            break;
        } else if super::write::is_date(&midnight) {
            defaults.date = Some(midnight);
            break;
        } else if defaults.unread.is_none() {
            defaults.unread = Some((key, value));
        }
    }
    defaults
}

/// `2026-03-02..2026-03-04`: the first and last days.
fn day_range(text: &str) -> Option<(String, String)> {
    let (first, last) = text.split_once("..")?;
    // A bare day, or a timestamp's day (`2026-03-02T10:00:00+02:00`).
    let day = |d: &str| {
        let d = d.trim();
        let day = d.get(..10)?;
        (super::write::is_date(&format!("{day}T00:00:00Z"))
            && (d.len() == 10 || super::write::is_date(d)))
        .then(|| day.to_string())
    };
    let (first, last) = (day(first)?, day(last)?);
    (first <= last).then_some((first, last))
}

/// Who owns the document, as `authors: document_owner:` and its note say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
    /// `dc:creator`; a name with no note, too.
    Creator(String),
    /// `cp:lastModifiedBy`, and no `dc:creator`.
    LastModifiedBy(String),
    /// `none`: core properties naming nobody.
    Unnamed,
    /// `none # no docProps/core.xml`: no core properties.
    NoPart,
}

/// Whether `line` opens the `authors:` block, a comment after it or not
/// (`authors: # the legend`).
fn opens_authors(line: &str) -> bool {
    line.split_once(" #")
        .map_or(line, |(key, _)| key)
        .trim_end()
        == "authors:"
}

/// The `document_owner:` of a YAML header's `authors:`; `None` also for an
/// empty or null one (`document_owner:`, `~`, `null`), which names no
/// owner to write: the document's own stays.
pub(super) fn document_owner(yaml: &str) -> Option<Owner> {
    let rest = owner_value(yaml)?;
    let (value, comment) = scalar_and_comment(rest);
    if names_no_owner(rest) {
        return None;
    }
    let comment = comment.unwrap_or("");
    // `none` names nobody, unless quoted or noted as the element that
    // holds it (`none # dc:creator`: someone called none).
    let nobody = value == "none"
        && !rest.starts_with(['"', '\''])
        && !comment.starts_with("dc:creator")
        && !comment.starts_with("cp:lastModifiedBy");
    Some(if nobody {
        if comment.starts_with("no docProps/core.xml") {
            Owner::NoPart
        } else {
            Owner::Unnamed
        }
    } else if comment.starts_with("cp:lastModifiedBy") {
        Owner::LastModifiedBy(value)
    } else {
        Owner::Creator(value)
    })
}

/// Whether the header has a `document_owner:` that names no one to
/// write: empty (`document_owner:`, `""`) or null (`~`, `null`).
pub(super) fn empty_owner(yaml: &str) -> bool {
    owner_value(yaml).is_some_and(names_no_owner)
}

fn names_no_owner(rest: &str) -> bool {
    let (value, _) = scalar_and_comment(rest);
    let quoted = rest.starts_with(['"', '\'']);
    value.is_empty() || (!quoted && matches!(value.as_str(), "~" | "null" | "Null" | "NULL"))
}

/// The text after the first `document_owner:` of `authors:`.
fn owner_value(yaml: &str) -> Option<&str> {
    let mut inside = false;
    for line in yaml.lines() {
        // A blank or comment line does not end the block, as in YAML.
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            inside = opens_authors(line);
            continue;
        }
        let Some(rest) = line
            .trim_start()
            .strip_prefix("document_owner:")
            .filter(|_| inside)
        else {
            continue;
        };
        // `document_owner:  # a note` holds no value.
        let rest = rest.trim_start();
        return Some(if rest.starts_with('#') { "" } else { rest });
    }
    None
}

/// Letters and digits: the view's handles (`from_docx::agent`).
/// An author's handle: letters and digits, or `??`, the reader's handle
/// for a mark whose author is missing or empty.
fn is_handle(text: &str) -> bool {
    text == UNNAMED || (!text.is_empty() && text.chars().all(char::is_alphanumeric))
}

/// The handle of a mark with no author's name.
pub(super) const UNNAMED: &str = "??";

/// A YAML scalar as `header::scalar` writes it (plain, or a JSON-quoted
/// string), and the `# comment` after it.
pub(super) fn scalar_and_comment(text: &str) -> (String, Option<&str>) {
    if text.starts_with('"') {
        let mut de = serde_json::Deserializer::from_str(text).into_iter::<String>();
        if let Some(Ok(value)) = de.next() {
            let rest = text[de.byte_offset()..].trim_start();
            return (value, rest.strip_prefix('#').map(str::trim));
        }
        return (String::new(), None);
    }
    // A YAML single-quoted scalar: `''` is one quote, the next lone `'` ends it.
    if let Some(body) = text.strip_prefix('\'') {
        let mut value = String::new();
        let mut chars = body.char_indices().peekable();
        while let Some((at, c)) = chars.next() {
            if c != '\'' {
                value.push(c);
            } else if chars.next_if(|&(_, next)| next == '\'').is_some() {
                value.push('\'');
            } else {
                let rest = body[at + 1..].trim_start();
                return (value, rest.strip_prefix('#').map(str::trim));
            }
        }
        return (String::new(), None);
    }
    // `key:  # note`: YAML reads a comment, and the value is null.
    if let Some(comment) = text.strip_prefix('#') {
        return (String::new(), Some(comment.trim()));
    }
    match text.split_once(" #") {
        Some((value, comment)) => (value.trim().to_string(), Some(comment.trim())),
        None => (text.trim().to_string(), None),
    }
}

/// One tag of a revision note: who and, when the note says, when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Tag {
    pub handle: String,
    pub date: Option<String>,
    /// The `w:id`s it names, in order (`r1+2` → 1, 2); none for `noid`.
    pub ids: Vec<u32>,
}

/// The tags of a revision note, `r1+2 @AC` or `r1 @AC; r2 @JD 2026-…`;
/// `None` for any other comment text. Each id part must be a change id
/// (`crate::ids`), or `noid` for a mark without a `w:id`.
pub(super) fn revision_tags(text: &str) -> Option<Vec<Tag>> {
    text.trim()
        .split("; ")
        .map(|tag| {
            let mut words = tag.split(' ');
            let id = words.next()?;
            let handle = words.next()?.strip_prefix('@')?;
            let date = words.next();
            if words.next().is_some() || !is_handle(handle) {
                return None;
            }
            // A bare number (`5 @AC`) is a person's comment, not a tag.
            let id_ok = !id.starts_with(|c: char| c.is_ascii_digit())
                && (id == "noid"
                    || id.strip_suffix(".noid").is_some()
                    || crate::ids::parse_change_ids(id).is_ok());
            let date_ok = date.is_none_or(super::write::is_date);
            (id_ok && date_ok && !id.starts_with('#')).then(|| Tag {
                handle: handle.to_string(),
                date: date.map(str::to_string),
                ids: crate::ids::parse_change_ids(id)
                    .map(|ids| {
                        ids.iter()
                            .filter_map(|c| c.wid.parse().ok())
                            .filter(|&id| id <= MOST_ID)
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect()
}

/// A plain view's revision note: the change's ids, then who made it and
/// when, as stored (`r5 Ana Lima (2026-…)`, `r1+2 (2026-…)`, `r7 Di Eng`,
/// `r7`). An absent author or date stays empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlainRevision {
    pub ids: Vec<u32>,
    pub author: String,
    pub date: String,
}

/// The tags of a nested revision note, its own change first: `r3 @BO on
/// r1 @AL` is Ben's deletion inside Ana's insertion, each tag with its
/// own date as [`revision_tags`] reads it. `None` for any other comment
/// text.
pub(super) fn nested_revision_tags(text: &str) -> Option<Vec<Tag>> {
    let tags = nested_parts(text, one_tag)?;
    (tags.len() > 1).then_some(tags)
}

/// One tag, exactly: a note names one change per ` on ` part.
fn one_tag(part: &str) -> Option<Tag> {
    match revision_tags(part)?.as_slice() {
        [tag] => Some(tag.clone()),
        _ => None,
    }
}

/// The parts of a plain nested revision note, its own change first: `r3
/// Ben Ode (2026-…) on r1 Ana Lima (2026-…)`. `None` for any other
/// comment text.
pub(super) fn nested_plain_revisions(text: &str) -> Option<Vec<PlainRevision>> {
    let parts = nested_parts(text, plain_part)?;
    (parts.len() > 1).then_some(parts)
}

/// One part of a fold note: its author holds no bare parenthesis — it
/// spells its own percent-wise (`Ana %28consultant%29`), so the date is
/// the only bare group and the split of the note stays sure. A part with
/// a bare parenthesis is no part of a fold.
fn plain_part(part: &str) -> Option<PlainRevision> {
    let rest = part.trim().split_once(' ').map_or("", |(_, rest)| rest);
    let author_end = trailing_bare_group(rest)
        .filter(|&at| super::write::is_date(&rest[at + 1..rest.len() - 1]))
        .unwrap_or(rest.len());
    let mut escaped = false;
    for c in rest[..author_end].chars() {
        if matches!(c, '(' | ')') && !escaped {
            return None;
        }
        escaped = c == '\\' && !escaped;
    }
    plain_revision(part)
}

/// Where the final bare parenthesised group of `text` opens, when one
/// closes at its very end (`… (date)`): the outermost group that does.
pub(super) fn trailing_bare_group(text: &str) -> Option<usize> {
    if !text.ends_with(')') {
        return None;
    }
    let mut escaped = false;
    let mut depth = 0usize;
    let mut open = None;
    for (at, c) in text.char_indices() {
        match c {
            '\\' => escaped = !escaped,
            '(' if !escaped => {
                if depth == 0 {
                    open = Some(at);
                }
                depth += 1;
                escaped = false;
            }
            ')' if !escaped => {
                depth = depth.saturating_sub(1);
                if at + 1 == text.len() && depth == 0 {
                    return open;
                }
                escaped = false;
            }
            _ => escaped = false,
        }
    }
    None
}

/// The parts of a note joined by ` on `, each parsed by `one`: the first
/// part is the note's own change, and each part after it names the change
/// the one before sits inside. The last ` on ` that splits the note wins:
/// the left side must parse as one part — its author may hold ` on ` of
/// its own (`Bo on r1 Kim`) — and the right side all the way down.
/// Earlier boundaries are tried only when these fail.
fn nested_parts<T>(text: &str, one: fn(&str) -> Option<T>) -> Option<Vec<T>> {
    let text = text.trim();
    let mut from = text.len();
    while let Some(at) = text[..from].rfind(" on ") {
        from = at;
        if let Some(head) = one(text[..at].trim())
            && let Some(rest) = nested_parts(&text[at + " on ".len()..], one)
        {
            return Some([head].into_iter().chain(rest).collect());
        }
    }
    // The last part, or a note that names no other change.
    one(text).map(|last| vec![last])
}

/// The plain view's revision note; `None` for any other comment text. The
/// ids come first, as in [`revision_tags`]; what follows is `Name (date)`,
/// `(date)`, a name with no colon or bracket, or nothing.
pub(super) fn plain_revision(text: &str) -> Option<PlainRevision> {
    let text = text.trim();
    let (id, rest) = text.split_once(' ').unwrap_or((text, ""));
    let id_ok = !id.starts_with(|c: char| c.is_ascii_digit())
        && !id.starts_with('#')
        && (id == "noid"
            || id.strip_suffix(".noid").is_some()
            || crate::ids::parse_change_ids(id).is_ok());
    if !id_ok {
        return None;
    }
    let (author, date) = match super::write::named(rest) {
        Some((author, date, "")) => (author.unwrap_or_default(), date.to_string()),
        Some(_) => return None,
        None if rest.contains([':', '(', ')']) || rest.starts_with('@') => return None,
        None => (rest.trim().to_string(), String::new()),
    };
    let ids = crate::ids::parse_change_ids(id)
        .map(|ids| {
            ids.iter()
                .filter_map(|c| c.wid.parse().ok())
                .filter(|&id| id <= MOST_ID)
                .collect()
        })
        .unwrap_or_default();
    Some(PlainRevision { ids, author, date })
}

/// The largest `w:id` Word reads (`ST_DecimalNumber` is an `int`): a
/// larger number a view names is left for a fresh one.
pub(super) const MOST_ID: u32 = 0x7FFF_FFFF;

/// A comment head the view writes: `c5 @AC: `, `c5 @AC 2026-…: `,
/// `c6 @AS re c5: `, `c5 @AC resolved: `, `c5: `.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CommentHead {
    /// The comment's number in the source (`c5` → 5).
    pub number: u32,
    pub handle: Option<String>,
    pub date: Option<String>,
    /// The number of the comment this one replies to.
    pub reply_to: Option<u32>,
    pub resolved: bool,
}

/// The cells a `cell-borders` group names: every cell, a row's, or one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CellsAt {
    All,
    Row(usize),
    Cell(usize, usize),
}

impl CellsAt {
    /// Whether the cell at `row`, `cell` is one of these.
    pub(super) fn holds(self, row: usize, cell: usize) -> bool {
        match self {
            Self::All => true,
            Self::Row(r) => r == row,
            Self::Cell(r, c) => (r, c) == (row, cell),
        }
    }
}

/// What a table's id line says about its rows (`<!-- t0 4x7, cells
/// t0.row0 p1-p13 t0.row1 p14-p50, header row repeats -->`): how many
/// paragraphs each row holds, when it counts them row by row, and whether
/// its first row repeats on each page. `None` for any other line.
pub(super) struct TableLine {
    /// Each row's first paragraph and paragraph count, row by row.
    pub row_paragraphs: Option<Vec<(usize, usize)>>,
    /// The first paragraph of a table whose every cell holds one (`cells
    /// p8-p16 by row`).
    pub one_per_cell: Option<usize>,
    pub header_repeats: bool,
    /// The cell paragraphs' mark changes, by paragraph (`break-ins r3 @SW
    /// in p1`): true for an insertion.
    pub marks: HashMap<usize, (bool, Tag)>,
    /// The grid's columns (`4x7`).
    pub columns: Option<usize>,
    /// The table's alignment (`t0 center 4x7`), as Word's `jc` value.
    pub align: Option<&'static str>,
    /// The merged cells by row and cell (`merged t0.row0.cell1 across 2
    /// down 3`): the grid columns each spans and the rows it covers.
    pub merges: HashMap<(usize, usize), (usize, usize)>,
    /// Cell paragraphs aligned against their column (`p5 center`), as
    /// Word's `jc` value, by paragraph.
    pub aligns: HashMap<usize, &'static str>,
    /// Cell paragraphs in a style of their own (`p5 ListParagraph`,
    /// `p6-p9 Caption`), by paragraph.
    pub styles: HashMap<usize, String>,
    /// The table's style (`style Ledger`) when tables differ in theirs.
    pub style: Option<String>,
    /// The table's own borders (`borders single 0.5pt 000000`).
    pub borders: Option<super::borders::Sides>,
    /// Its cells' own borders (`cell-borders bottom single 1pt in t0.row0`),
    /// with the cells each names.
    pub cell_borders: Vec<(CellsAt, super::borders::Sides)>,
    /// Its rows' own table borders (`row-borders top double 0.5pt in
    /// t0.row1`, Word's `w:tblPrEx`), by row.
    pub row_borders: Vec<(usize, super::borders::Sides)>,
    /// A comment's range runs through the table (`in c5`).
    pub in_comment: bool,
    /// The table's formatting changes (`fmt r16 @AL; r14 @AL in t0.row0;
    /// r15 @AL in t0.row1.cell0`): its own, its rows' and its cells'.
    pub fmt: Option<Tag>,
    /// Word's `w:tblGridChange`, which names no author (`r17 @??`).
    pub grid_fmt: Option<u32>,
    /// These maps are `BTreeMap`s, not `HashMap`s: the writer walks them to
    /// stamp dates, and an author's first undated mark takes the range's last
    /// day — a `HashMap`'s random order would pick the winner per process
    /// (#534).
    pub row_fmt: BTreeMap<usize, Tag>,
    pub cell_fmt: BTreeMap<(usize, usize), Tag>,
    /// The cell paragraphs' formatting changes (`fmt r9 @AL in p4`), by
    /// paragraph number.
    pub para_fmt: HashMap<usize, Vec<Tag>>,
    /// A row's own insertion or deletion, when it has no text to show it
    /// (`rev r6 @AC in t0.row1`, `rev-del …`): whether it is inserted.
    pub row_marks: BTreeMap<usize, (bool, Tag)>,
    /// A cell's own insertion, deletion or merge (`rev r5 @AC in
    /// t0.row1.cell2`, `rev-del …`, `rev-merge …`).
    pub cell_marks: BTreeMap<(usize, usize), (CellMark, Tag)>,
    /// The grid columns each row leaves out before and after its cells
    /// (`gaps t0.row1 before 1; t0.row2 after 1`), by row.
    pub gaps: HashMap<usize, (usize, usize)>,
}

/// A cell's own change: `w:cellIns`, `w:cellDel` or `w:cellMerge`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellMark {
    Insert,
    Delete,
    Merge,
}

/// The header's `unplaced: [ins r40 @AL in p1, …]`: each change no other
/// line shows, the paragraph it is in and its kind (`ins`, `del`, `fmt`;
/// a kindless one is an insertion). One without a paragraph, or not a
/// change's tag, is left out.
pub(super) fn unplaced(value: &str) -> Vec<(usize, &'static str, Tag)> {
    let (list, _) = value.split_once(" #").unwrap_or((value, ""));
    let Some(list) = list
        .trim()
        .strip_prefix('[')
        .and_then(|l| l.strip_suffix(']'))
    else {
        return Vec::new();
    };
    list.split(", ")
        .filter_map(|entry| {
            let (entry, _) = scalar_and_comment(entry.trim());
            let (tag, at) = entry.rsplit_once(" in ")?;
            let (kind, tag) = match tag.split_once(' ') {
                Some(("del", tag)) => ("del", tag),
                Some(("fmt", tag)) => ("fmt", tag),
                Some(("ins", tag)) => ("ins", tag),
                _ => ("ins", tag),
            };
            Some((crate::ids::short_number(at, 'p')?, kind, first_tag(tag)?))
        })
        .collect()
}

/// The header's `notes: {…, empty: [footnotes, endnotes]}`: the notes
/// parts (`footnote`, `endnote`) that hold no note; none without the
/// clause. The clause is the map's last, outside any quoted change tag;
/// one that does not read is an error saying why.
pub(super) fn empty_notes(value: &str) -> Result<Vec<&'static str>, String> {
    // The clauses outside quotes, up to a YAML comment (` # …`).
    let (mut quoted, mut escaped) = (false, false);
    let (mut clause, mut end) = (None, value.len());
    for (at, c) in value.char_indices() {
        if quoted {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => quoted = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            '#' if value[..at].ends_with([' ', '\t']) => {
                end = at;
                break;
            }
            _ if value[at..].starts_with("empty:") && value[..at].ends_with([' ', '{', ',']) => {
                clause = Some(at);
            }
            _ => {}
        }
    }
    let Some(at) = clause else {
        return Ok(Vec::new());
    };
    let opens = value[..at].trim_end();
    let list = value[at + "empty:".len()..end]
        .trim()
        .strip_suffix('}')
        .map(str::trim_end)
        .and_then(|l| l.strip_prefix('['))
        .and_then(|l| l.strip_suffix(']'))
        .filter(|_| opens.ends_with(['{', ',']))
        .ok_or_else(|| "empty: is not the map's last clause, a [list]".to_string())?;
    let mut kinds = Vec::new();
    for name in list.split(',').map(str::trim).filter(|n| !n.is_empty()) {
        let kind = match name {
            "footnotes" => "footnote",
            "endnotes" => "endnote",
            other => return Err(format!("empty: names {other}, not footnotes or endnotes")),
        };
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    Ok(kinds)
}

/// The header's `notes: {fmt: footnotes.r0 @AS; endnotes.r2 @AS, …}`:
/// each formatting change on a notes part's separators, with its part's
/// kind (`footnote`, `endnote`), in the order the view lists them; none
/// without the clause. An entry that names no notes part is skipped.
pub(super) fn note_formats(value: &str) -> Vec<(&'static str, Tag)> {
    let Some(tags) = super::view_stories::flow_text(value, "fmt") else {
        return Vec::new();
    };
    tags.split("; ")
        .filter_map(|entry| {
            let kind = match entry.trim().split_once('.')?.0 {
                "footnotes" => "footnote",
                "endnotes" => "endnote",
                _ => return None,
            };
            Some((kind, revision_tags(entry)?.into_iter().next()?))
        })
        .collect()
}

/// The most grid columns a table has, and rows one merged cell reaches: a
/// bound on what a hand-edited table line can make the reader write (Word
/// takes 63 columns).
const MOST_GRID: usize = 64;

pub(super) fn table_line(line: &str) -> Option<TableLine> {
    let inner = line
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    let (id, rest) = inner.split_once(' ').unwrap_or((inner, ""));
    crate::ids::short_number(id, 't')?;
    let mut parsed = TableLine {
        row_paragraphs: None,
        one_per_cell: None,
        header_repeats: false,
        marks: HashMap::new(),
        columns: None,
        align: None,
        merges: HashMap::new(),
        aligns: HashMap::new(),
        styles: HashMap::new(),
        style: None,
        borders: None,
        cell_borders: Vec::new(),
        row_borders: Vec::new(),
        in_comment: false,
        fmt: None,
        grid_fmt: None,
        row_fmt: BTreeMap::new(),
        cell_fmt: BTreeMap::new(),
        para_fmt: HashMap::new(),
        row_marks: BTreeMap::new(),
        cell_marks: BTreeMap::new(),
        gaps: HashMap::new(),
    };
    for (i, clause) in split_clauses(rest).into_iter().enumerate() {
        // `p5 center`: a cell paragraph aligned against its column.
        if let Some((at, word)) = clause.split_once(' ')
            && let Some(at) = crate::ids::short_number(at, 'p')
            && let Some(jc) = match word {
                "left" => Some("left"),
                "center" => Some("center"),
                "right" => Some("right"),
                "justify" => Some("both"),
                _ => None,
            }
        {
            parsed.aligns.insert(at, jc);
            continue;
        }
        // `p5 ListParagraph`, `p6-p9 Caption`, `p7 "My Style"`: cell
        // paragraphs in a style.
        if let Some((at, tail)) = clause.split_once(' ')
            && let Some((first, count)) = paragraph_range(at)
            && let Some((style, "")) = take_style(tail)
        {
            for n in first..first + count {
                parsed.styles.insert(n, style.clone());
            }
            continue;
        }
        if clause == "header row repeats" {
            parsed.header_repeats = true;
        }
        if clause.strip_prefix("in ").is_some_and(comment_numbers) {
            parsed.in_comment = true;
        }
        if let Some(rows) = clause.strip_prefix("gaps ") {
            for row in rows.split("; ") {
                let mut words = row.split(' ');
                let Some(r) = words.next().and_then(row_of) else {
                    continue;
                };
                let (mut before, mut after) = (0, 0);
                while let (Some(word), Some(Ok(n))) =
                    (words.next(), words.next().map(str::parse::<usize>))
                {
                    match word {
                        "before" => before = n.min(MOST_GRID),
                        "after" => after = n.min(MOST_GRID),
                        _ => {}
                    }
                }
                parsed.gaps.insert(r, (before, after));
            }
        }
        if let Some(changes) = clause.strip_prefix("fmt ") {
            for change in changes.split("; ") {
                let (tag, at) = change
                    .rsplit_once(" in ")
                    .map_or((change, None), |(tag, at)| (tag, Some(at)));
                // The grid's change has no author to name: `@??` on the
                // table itself.
                let grid = at.is_none() && tag.ends_with(&format!(" @{UNNAMED}"));
                let Some(tag) = first_tag(tag).filter(|_| !grid) else {
                    if at.is_none()
                        && let Some(id) = tag.strip_suffix(" @??").and_then(|id| {
                            crate::ids::parse_change_ids(id)
                                .ok()?
                                .first()?
                                .wid
                                .parse()
                                .ok()
                        })
                    {
                        parsed.grid_fmt = Some(id);
                    }
                    continue;
                };
                match at {
                    None => parsed.fmt = Some(tag),
                    Some(at) => {
                        if let Some(cell) = row_and_cell(at) {
                            parsed.cell_fmt.insert(cell, tag);
                        } else if let Some(row) = row_of(at) {
                            parsed.row_fmt.insert(row, tag);
                        } else if let Some(p) = crate::ids::short_number(at, 'p') {
                            parsed.para_fmt.entry(p).or_default().push(tag);
                        }
                    }
                }
            }
        }
        for (word, kind) in [
            ("rev ", CellMark::Insert),
            ("rev-del ", CellMark::Delete),
            ("rev-merge ", CellMark::Merge),
        ] {
            let Some(marks) = clause.strip_prefix(word) else {
                continue;
            };
            // `r6 @AC in t0.row1; r5 @AC in t0.row1.cell2`
            for mark in marks.split("; ") {
                let Some((tag, at)) = mark.rsplit_once(" in ") else {
                    continue;
                };
                let Some(tag) = first_tag(tag) else { continue };
                if let Some(cell) = row_and_cell(at) {
                    parsed.cell_marks.insert(cell, (kind, tag));
                } else if let Some(row) = row_of(at)
                    && kind != CellMark::Merge
                {
                    parsed
                        .row_marks
                        .insert(row, (kind == CellMark::Insert, tag));
                }
            }
        }
        if let Some((style, "")) = clause.strip_prefix("style ").and_then(take_style) {
            parsed.style = Some(style);
        }
        if let Some(text) = clause.strip_prefix("borders ") {
            parsed.borders = super::borders::parse(text, &super::borders::TABLE_SIDES);
        }
        if let Some(groups) = clause.strip_prefix("row-borders ") {
            // `top double 0.5pt FF0000 in t0.row1 t0.row3; left none in t0.row2`
            for group in groups.split("; ") {
                let Some((text, at)) = group.rsplit_once(" in ") else {
                    continue;
                };
                let Some(sides) = super::borders::parse(text, &super::borders::TABLE_SIDES) else {
                    continue;
                };
                for row in at.split(' ').filter_map(row_of) {
                    parsed.row_borders.push((row, sides.clone()));
                }
            }
        }
        if let Some(groups) = clause.strip_prefix("cell-borders ") {
            // `top,bottom single 1pt in all; left none in t0.row1 t0.row2.cell0`
            for group in groups.split("; ") {
                let Some((text, at)) = group.rsplit_once(" in ") else {
                    continue;
                };
                let Some(sides) = super::borders::parse(text, &super::borders::CELL_SIDES) else {
                    continue;
                };
                for target in at.split(' ') {
                    let target = if target == "all" {
                        Some(CellsAt::All)
                    } else if let Some((row, cell)) = row_and_cell(target) {
                        Some(CellsAt::Cell(row, cell))
                    } else {
                        row_of(target).map(CellsAt::Row)
                    };
                    if let Some(target) = target {
                        parsed.cell_borders.push((target, sides.clone()));
                    }
                }
            }
        }
        // The size opens the clauses, after the alignment: `center 3x2`.
        if i == 0
            && let Some((_, columns)) = clause.rsplit(' ').next().and_then(|s| s.split_once('x'))
        {
            parsed.columns = columns.parse().ok().filter(|&n| n <= MOST_GRID);
            parsed.align = match clause.split(' ').next() {
                Some("center") => Some("center"),
                Some("right") => Some("right"),
                _ => None,
            };
        }
        if let Some(cells) = clause.strip_prefix("merged ") {
            // `t0.row0.cell1 across 2 down 3; t0.row2.cell0 down 2`
            for cell in cells.split("; ") {
                let mut words = cell.split(' ');
                let Some((row, at)) = words.next().and_then(row_and_cell) else {
                    continue;
                };
                let (mut across, mut down) = (1, 1);
                while let (Some(word), Some(Ok(n))) =
                    (words.next(), words.next().map(str::parse::<usize>))
                {
                    match word {
                        "across" => across = n.clamp(1, MOST_GRID),
                        "down" => down = n.clamp(1, MOST_GRID),
                        _ => {}
                    }
                }
                parsed.merges.insert((row, at), (across, down));
            }
        }
        for (word, inserted) in [("break-ins ", true), ("break-del ", false)] {
            let Some(marks) = clause.strip_prefix(word) else {
                continue;
            };
            // `r3 @SW in p1; r4 @SW in p2`
            for mark in marks.split("; ") {
                if let Some((tag, at)) = mark.rsplit_once(" in ")
                    && let Some(at) = crate::ids::short_number(at, 'p')
                    && let Some(tag) = first_tag(tag)
                {
                    parsed.marks.insert(at, (inserted, tag));
                }
            }
        }
        let Some(rows) = clause.strip_prefix("cells ") else {
            continue;
        };
        if let Some(range) = rows.strip_suffix(" by row") {
            let first = range.split_once('-').map_or(range, |(first, _)| first);
            parsed.one_per_cell = crate::ids::short_number(first, 'p');
            continue;
        }
        // `t0.row0 p1-p13 t0.row1 p14`: a row, then its paragraphs.
        let words: Vec<&str> = rows.split(' ').collect();
        parsed.row_paragraphs = words
            .chunks(2)
            .map(|pair| {
                let range = pair.get(1)?;
                let (first, last) = range.split_once('-').unwrap_or((range, range));
                let first = crate::ids::short_number(first, 'p')?;
                let last = crate::ids::short_number(last, 'p')?;
                last.checked_sub(first)?
                    .checked_add(1)
                    .map(|count| (first, count))
            })
            .collect();
    }
    Some(parsed)
}

/// The first paragraph and count of `p5` or `p6-p9`.
fn paragraph_range(id: &str) -> Option<(usize, usize)> {
    let (first, last) = id.split_once('-').unwrap_or((id, id));
    let first = crate::ids::short_number(first, 'p')?;
    let last = crate::ids::short_number(last, 'p')?;
    let count = last.checked_sub(first)?.checked_add(1)?;
    (count <= MOST_EMPTY).then_some((first, count))
}

/// The row and cell of `t0.row2.cell1`.
/// `t0.row2` → 2.
fn row_of(id: &str) -> Option<usize> {
    let mut parts = id.split('.').skip(1);
    let row = parts.next()?.strip_prefix("row")?.parse().ok()?;
    parts.next().is_none().then_some(row)
}

fn row_and_cell(id: &str) -> Option<(usize, usize)> {
    let mut parts = id.split('.').skip(1);
    let row = parts.next()?.strip_prefix("row")?.parse().ok()?;
    let cell = parts.next()?.strip_prefix("cell")?.parse().ok()?;
    parts.next().is_none().then_some((row, cell))
}

/// The most empty paragraphs one id line stands for: a bound on what a
/// hand-edited range (`p0-p4294967295 empty`) can make the reader write.
const MOST_EMPTY: usize = 100_000;

/// The id of a note definition's id line, `endnotes.p0` in
/// `<!-- endnotes.p0 -->`, when it starts with `part`.
fn note_line<'l>(line: &'l str, part: &str) -> Option<&'l str> {
    let id = line
        .trim()
        .strip_prefix("<!--")?
        .split_whitespace()
        .next()?
        .trim_end_matches("-->");
    id.strip_prefix(part)
        .filter(|rest| {
            rest.split_once('.')
                .is_some_and(|(_, p)| p.starts_with('p'))
        })
        .map(|_| id)
}

/// How many paragraphs a note definition's id line counts: one for
/// `<!-- footnotes.p2 -->`, two for `<!-- endnotes.p2-p3, rev … -->`.
pub(super) fn note_paragraphs(line: &str) -> Option<usize> {
    let id = note_line(line, "footnotes").or_else(|| note_line(line, "endnotes"))?;
    let (_, range) = id.split_once('.')?;
    let range = range.trim_end_matches(',');
    let (first, last) = range.split_once('-').unwrap_or((range, range));
    let first = crate::ids::short_number(first, 'p')?;
    let last = crate::ids::short_number(last, 'p')?;
    last.checked_sub(first)?.checked_add(1)
}

/// The styles a note definition's id line names after its id, one per
/// paragraph, the last for every paragraph after it
/// (`<!-- endnotes.p0 Endnote -->`, `<!-- footnotes.p1-p2 FootnoteText,
/// "My Note" -->`): the note's paragraphs are in them, not in Word's note
/// style. `None` for a line that names none, or a malformed list.
pub(super) fn note_styles(line: &str) -> Option<Vec<String>> {
    note_line(line, "footnotes").or_else(|| note_line(line, "endnotes"))?;
    let inner = line
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    // Any whitespace ends the id, as for `note_line`.
    let (id, rest) = inner.split_once(char::is_whitespace)?;
    if id.ends_with(',') {
        return None;
    }
    let mut styles = Vec::new();
    let mut rest = rest.trim();
    while !rest.is_empty() {
        let (style, after) = take_style(rest)?;
        // `""` names no style a paragraph can be in.
        if style.trim().is_empty() {
            return None;
        }
        styles.push(style);
        rest = after;
    }
    (!styles.is_empty()).then_some(styles)
}

/// Whether `line` is an endnote definition's id line (`<!-- endnotes.p0 -->`).
pub(super) fn endnote_line(line: &str) -> bool {
    note_line(line, "endnotes").is_some()
}

/// Whether `line` is a footnote definition's id line (`<!-- footnotes.p1 -->`).
pub(super) fn footnote_line(line: &str) -> bool {
    note_line(line, "footnotes").is_some()
}

/// How many empty paragraphs an id line stands for: `<!-- p2 empty -->`
/// one, `<!-- p2-p4 BodyText empty -->` three; `None` for any other line.
pub(super) fn empty_paragraphs(line: &str) -> Option<usize> {
    let inner = line
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    let (id, rest) = inner.split_once(' ').unwrap_or((inner, ""));
    let count = match id.split_once('-') {
        Some((first, last)) => {
            let first = crate::ids::short_number(first, 'p')?;
            let last = crate::ids::short_number(last, 'p')?;
            last.checked_sub(first)?.checked_add(1)?
        }
        None => {
            crate::ids::short_number(id, 'p')?;
            1
        }
    };
    // `empty` is a clause of its own, also the first after the style
    // (`BodyText empty`, `"My Style" empty`).
    let rest = take_style(rest).map_or(rest, |(_, after)| after);
    let empty = split_clauses(rest).contains(&"empty");
    (empty && count <= MOST_EMPTY).then_some(count)
}

/// The words that open an id line's clauses (`center`, `left 0.5in`,
/// `break-ins r3 @AC`): any other word after the id is a style.
const CLAUSE_WORDS: [&str; 21] = [
    "empty",
    "outline",
    "center",
    "right",
    "justify",
    "left",
    "first-line",
    "hanging",
    "bullet",
    "num",
    "level",
    "page-break",
    "page-break-ins",
    "page-break-del",
    "section-break",
    "break-ins",
    "break-del",
    "fmt",
    "rev",
    "comments",
    "in",
];

/// The words that open a id line's clauses (`center`, `left 0.5in`,
/// `break-ins r3 @AC`): beside [`CLAUSE_WORDS`], what a style id must not
/// read as.
const TABLE_WORDS: [&str; 10] = [
    "cells",
    "borders",
    "cell-borders",
    "row-borders",
    "merged",
    "gaps",
    "style",
    "header",
    "rev-del",
    "rev-merge",
];

/// Whether `word` opens a clause, so a style of that id needs quotes.
fn reserved(word: &str) -> bool {
    CLAUSE_WORDS.contains(&word) || TABLE_WORDS.contains(&word)
}

/// A style id as id lines and table lines print it: as is when it reads
/// back as one word, quoted otherwise (`"My Style"`, `"center"`, `"A, B"`),
/// with `\` and `"` escaped.
pub(crate) fn style_token(style: &str) -> String {
    let plain = !style.is_empty()
        && !reserved(style)
        && !style
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, ',' | '"' | '\\'));
    if plain {
        style.to_string()
    } else {
        format!("\"{}\"", style.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// The style id opening `text` (`TitlePage center` or `"My Style" center`)
/// and the text after it; `None` when `text` opens with a clause word or an
/// unclosed quote.
fn take_style(text: &str) -> Option<(String, &str)> {
    if let Some(quoted) = text.strip_prefix('"') {
        let mut style = String::new();
        let mut chars = quoted.char_indices();
        while let Some((i, c)) = chars.next() {
            match c {
                '\\' => style.push(chars.next()?.1),
                '"' => {
                    let rest = quoted[i + 1..].trim_start_matches(',').trim_start();
                    return Some((style, rest));
                }
                c => style.push(c),
            }
        }
        return None;
    }
    let end = text.find([' ', ',']).unwrap_or(text.len());
    let word = &text[..end];
    if word.is_empty() || reserved(word) {
        return None;
    }
    Some((
        word.to_string(),
        text[end..].trim_start_matches(',').trim_start(),
    ))
}

/// The comment paragraph styles a `comments:` line names: the one for
/// every comment (`style Normal`) and each comment's own (`c6 style
/// Normal`), by the comment's number. Other clauses are counts.
pub(super) fn comment_styles(value: &str) -> (Option<String>, HashMap<u32, String>) {
    let (mut all, mut each) = (None, HashMap::new());
    for clause in split_clauses(value) {
        if let Some(rest) = clause.strip_prefix("style ") {
            if let Some((style, "")) = take_style(rest) {
                all = Some(style);
            }
        } else if let Some((number, rest)) = clause
            .strip_prefix('c')
            .and_then(|c| c.split_once(" style "))
            && let Ok(number) = number.parse()
            && let Some((style, "")) = take_style(rest)
        {
            each.insert(number, style);
        }
    }
    (all, each)
}

/// `text` split at each `, ` outside a quoted style or label.
fn split_clauses(text: &str) -> Vec<&str> {
    let mut clauses = Vec::new();
    let (mut start, mut quoted, mut escaped) = (0, false, false);
    for (i, c) in text.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            ',' if !quoted && text[i + 1..].starts_with(' ') => {
                clauses.push(&text[start..i]);
                start = i + 2;
            }
            _ => {}
        }
    }
    clauses.push(&text[start..]);
    clauses.retain(|c| !c.is_empty());
    clauses
}

/// What a paragraph's id line says about it (`<!-- p5 TitlePage center,
/// first-line 0.5in -->`): its style, alignment (`w:jc`) and indents in
/// twips.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct IdLine {
    /// The paragraph's number, as its line names it (`p5`).
    pub number: usize,
    pub style: Option<String>,
    /// The paragraph's own outline level (`outline 2`, `#`'s count), when
    /// no style carries it: 1 to 9, the levels past the heading styles
    /// (`outline 7`) included, which no `#` count reaches.
    pub outline: Option<u32>,
    pub align: Option<&'static str>,
    pub first_line: Option<i64>,
    pub hanging: Option<i64>,
    pub left: Option<i64>,
    pub right: Option<i64>,
    /// The paragraph holds a page break (`page-break`).
    pub page_break: bool,
    /// The page break's tracked change, `true` for an insertion
    /// (`page-break-ins r5 @AC`, `page-break-del r5 @AC`).
    pub page_break_change: Option<(bool, Tag)>,
    /// The paragraph ends a section (`section-break`).
    pub section_break: bool,
    /// The list label (`num "(a)"`, `bullet`).
    pub label: Option<Label>,
    /// The label's list level (`level 2`), when not the first.
    pub level: Option<u32>,
    /// The paragraph mark's insertion (`break-ins r4 @AC`).
    pub break_ins: Option<Tag>,
    /// The paragraph mark's deletion (`break-del r6 @AC`).
    pub break_del: Option<Tag>,
    /// Its formatting changes (`fmt r10 @AL; r11 @AL`).
    pub fmt: Vec<Tag>,
    /// The marks its text shows nothing of (`rev r9 @AL`, `rev-del r10
    /// @AL`); a `rev-del` tag is in `rev_del` too.
    pub rev: Vec<Tag>,
    /// The `rev` tags a `rev-del` clause named: deletions, while `rev`
    /// alone names insertions.
    pub rev_del: Vec<Tag>,
    /// The comments whose ranges run into it (`in c5 c6`).
    pub in_comments: Vec<u32>,
}

/// The id line of one paragraph (`pN`, or a range's first `pN-pM`, whose
/// facts stand for every paragraph it numbers); `None` for any other line.
pub(super) fn id_line(line: &str) -> Option<IdLine> {
    let inner = line
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    let (id, rest) = inner.split_once(' ').unwrap_or((inner, ""));
    let one = id.split_once('-').map_or(id, |(first, _)| first);
    let number = crate::ids::short_number(one, 'p')?;
    let mut parsed = IdLine {
        number,
        ..IdLine::default()
    };
    // The style is the head's second word, quoted when it is no plain
    // word: `p5 TitlePage[ clause]`, `p5 "My Style"[ clause]`.
    let rest = match take_style(rest) {
        Some((style, after)) => {
            parsed.style = Some(style);
            after
        }
        None => rest,
    };
    // Then the label: its quotes may hold `, `.
    let (rest, label) = take_label(rest);
    parsed.label = label.map(Label::Text);
    for clause in split_clauses(&rest) {
        parsed.clause(clause);
    }
    Some(parsed)
}

impl IdLine {
    fn clause(&mut self, clause: &str) {
        let twips = |value: &str| {
            let inches: f64 = value.strip_suffix("in")?.parse().ok()?;
            let twips = (inches * 1440.0).round();
            // `as` cannot overflow: the bound is checked first.
            (twips.is_finite() && twips.abs() <= 31_680.0).then_some(twips as i64)
        };
        match clause.split_once(' ') {
            None => {
                self.align = match clause {
                    "center" => Some("center"),
                    "right" => Some("right"),
                    "justify" => Some("both"),
                    "left" => Some("left"),
                    _ => self.align,
                };
                self.page_break |= clause == "page-break";
                self.section_break |= clause == "section-break";
                if clause == "bullet" {
                    self.label = Some(Label::Bullet);
                }
            }
            Some(("in", ids)) if comment_numbers(ids) => {
                self.in_comments = ids
                    .split(' ')
                    .filter_map(|id| crate::ids::short_number(id, 'c'))
                    .filter_map(|n| u32::try_from(n).ok())
                    .collect();
            }
            Some(("first-line", v)) => self.first_line = twips(v),
            Some(("hanging", v)) => self.hanging = twips(v),
            Some(("left", v)) => self.left = twips(v),
            Some(("right", v)) => self.right = twips(v),
            Some(("level", v)) => self.level = v.parse().ok().filter(|&n| n <= 8),
            Some(("outline", v)) => self.outline = v.parse().ok().filter(|&n| (1..=9).contains(&n)),
            // A mark has one change: the first tag names it.
            Some(("break-ins", v)) => self.break_ins = first_tag(v),
            Some(("break-del", v)) => self.break_del = first_tag(v),
            Some((kind @ ("page-break-ins" | "page-break-del"), v)) => {
                self.page_break = true;
                self.page_break_change = first_tag(v).map(|tag| (kind == "page-break-ins", tag));
            }
            Some(("fmt", v)) => self.fmt.extend(revision_tags(v).into_iter().flatten()),
            Some(("rev", v)) => self.rev.extend(revision_tags(v).into_iter().flatten()),
            Some(("rev-del", v)) => {
                let tags: Vec<Tag> = revision_tags(v).into_iter().flatten().collect();
                self.rev.extend(tags.iter().cloned());
                self.rev_del.extend(tags);
            }
            Some(_) => {}
        }
    }
}

/// `c5 c6`: the comment numbers of an `in` clause.
fn comment_numbers(text: &str) -> bool {
    !text.is_empty()
        && text
            .split(' ')
            .all(|id| crate::ids::short_number(id, 'c').is_some())
}

/// The clauses without their `num "…"` one, and its label (`\\` and `\"`
/// unescaped). The clause opens the line, follows `, `, or follows the
/// style (`BodyText num "(a)"`).
fn take_label(rest: &str) -> (String, Option<String>) {
    let style_end = rest.find([' ', ',']);
    let start = rest.match_indices("num \"").map(|(i, _)| i).find(|&i| {
        i == 0
            || rest[..i].ends_with(", ")
            || style_end.is_some_and(|end| end + 1 == i && rest.as_bytes()[end] == b' ')
    });
    let Some(start) = start else {
        return (rest.to_string(), None);
    };
    let mut label = String::new();
    let mut chars = rest[start + 5..].char_indices();
    let mut end = None;
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some((_, escaped)) => label.push(escaped),
                None => break,
            },
            '"' => {
                end = Some(start + 5 + i + 1);
                break;
            }
            c => label.push(c),
        }
    }
    let Some(end) = end else {
        return (rest.to_string(), None);
    };
    let before = rest[..start].trim_end_matches(", ").trim_end();
    let after = rest[end..].trim_start_matches(", ").trim_start();
    let joined = match (before.is_empty(), after.is_empty()) {
        (true, _) => after.to_string(),
        (false, true) => before.to_string(),
        // After the style the next clause joins with a space, as the
        // view prints it; after a clause, with `, `.
        (false, false) if rest[..start].ends_with(", ") => format!("{before}, {after}"),
        (false, false) => format!("{before} {after}"),
    };
    (joined, Some(label))
}

fn first_tag(text: &str) -> Option<Tag> {
    revision_tags(text)?.into_iter().next()
}

/// The head of `text` and the comment text after it.
pub(super) fn comment_head(text: &str) -> Option<(CommentHead, &str)> {
    let (head, body) = text.split_once(": ").or_else(|| {
        // A comment with no text: `c5 @AC:` at the end.
        text.strip_suffix(':').map(|head| (head, ""))
    })?;
    let mut words = head.split(' ').peekable();
    // `c5`, never a bare `5`: a person's comment such as `3: item three` is
    // text, not a head.
    let id = words.next().filter(|w| w.starts_with('c'))?;
    let number = crate::ids::parse_comment_id(id).ok()?;
    let mut parsed = CommentHead {
        number,
        handle: None,
        date: None,
        reply_to: None,
        resolved: false,
    };
    if let Some(handle) = words.peek().and_then(|w| w.strip_prefix('@')) {
        if !is_handle(handle) {
            return None;
        }
        parsed.handle = Some(handle.to_string());
        words.next();
    }
    if let Some(date) = words.peek().filter(|w| super::write::is_date(w)) {
        parsed.date = Some((*date).to_string());
        words.next();
    }
    match (words.next(), words.next(), words.next()) {
        (None, ..) => {}
        (Some("resolved"), None, _) => parsed.resolved = true,
        (Some("re"), Some(root), None) if root.starts_with('c') => {
            parsed.reply_to = Some(crate::ids::parse_comment_id(root).ok()?);
        }
        _ => return None,
    }
    Some((parsed, body))
}

#[cfg(test)]
mod tests {
    #[test]
    fn note_formats_reads_each_parts_separator_changes() {
        let kinds = |value: &str| -> Vec<(&str, Vec<u32>)> {
            note_formats(value)
                .into_iter()
                .map(|(kind, tag)| (kind, tag.ids))
                .collect()
        };
        assert_eq!(
            kinds(
                "{fmt: footnotes.r0 @AS; footnotes.r1 @AS; endnotes.r2 @AS, \
                 empty: [footnotes, endnotes]}  # changes in notes"
            ),
            [
                ("footnote", vec![0]),
                ("footnote", vec![1]),
                ("endnote", vec![2])
            ]
        );
        assert_eq!(
            kinds("{rev: \"footnotes.r4 @AS\", fmt: \"endnotes.r7 @AS 2026-09-26T14:55:00Z\"}"),
            [("endnote", vec![7])]
        );
        assert_eq!(kinds("{rev: \"fmt: footnotes.r1 @AS\"}"), []);
        assert_eq!(kinds("{rev: 'fmt: footnotes.r1 @AS'}"), []);
        assert_eq!(kinds("{rev: 'author''s fmt: footnotes.r1 @AS'}"), []);
        assert_eq!(kinds("{fmt: 'footnotes.r7 @AS'}"), [("footnote", vec![7])]);
        assert_eq!(kinds("{fmt: header1.r1 @AS}"), []);
        assert_eq!(kinds("{empty: [footnotes]}"), []);
    }

    #[test]
    fn empty_notes_reads_the_clause_outside_quotes() {
        use super::empty_notes;
        assert_eq!(
            empty_notes("{empty: [footnotes, endnotes]}  # notes parts that hold no note"),
            Ok(vec!["footnote", "endnote"])
        );
        assert_eq!(
            empty_notes("{rev: \"r1 @AC\", empty: [endnotes, endnotes]}"),
            Ok(vec!["endnote"])
        );
        assert_eq!(empty_notes("{rev: \"r1 @AC\"}"), Ok(vec![]));
        // A quoted tag that looks like the clause is not it.
        assert_eq!(empty_notes("{rev: \"empty: [footnotes] x\"}"), Ok(vec![]));
        assert_eq!(
            empty_notes("{rev: \"r1 } ] empty: [endnotes] }\"}"),
            Ok(vec![])
        );
        assert_eq!(
            empty_notes("{rev: \"a \\\" empty: [endnotes]}\"}"),
            Ok(vec![])
        );
        // A clause that does not read says why.
        assert!(empty_notes("{empty: [footnotes]} junk").is_err());
        assert!(empty_notes("{empty: footnotes}").is_err());
        assert!(empty_notes("{empty: [sidenotes]}").is_err());
        assert!(empty_notes("{empty: [endnotes], rev: \"r1 @AC\"}").is_err());
    }

    use super::*;

    #[test]
    fn quoted_scalars_unescape_and_keep_their_comment() {
        assert_eq!(
            scalar_and_comment(r#""a \"b\" #c" # note"#),
            (r#"a "b" #c"#.to_string(), Some("note"))
        );
        assert_eq!(
            scalar_and_comment("'it''s #1' # note"),
            ("it's #1".to_string(), Some("note"))
        );
        assert_eq!(scalar_and_comment("'open"), (String::new(), None));
        assert_eq!(
            scalar_and_comment("plain # c"),
            ("plain".to_string(), Some("c"))
        );
    }

    #[test]
    fn a_blank_line_inside_authors_keeps_the_block_open() {
        let yaml = "authors:\n  AL: Ann Lee\n\n# a note\n  document_owner: Bo Ray # dc:creator\n";
        assert_eq!(
            document_owner(yaml),
            Some(Owner::Creator("Bo Ray".to_string()))
        );
        let yaml = "authors:\n\n  AL: Ann Lee\n";
        assert!(authors(yaml).contains_key("AL"));
    }

    #[test]
    fn note_id_lines_name_their_part() {
        assert!(endnote_line("<!-- endnotes.p0 -->"));
        assert!(endnote_line("<!-- endnotes.p1-p2, rev endnotes.r4 @AL -->"));
        assert!(endnote_line("<!-- endnotes.p3-->"));
        assert!(!endnote_line("<!-- footnotes.p0 -->"));
        assert!(footnote_line("<!-- footnotes.p1 -->"));
        assert!(!footnote_line("<!-- p1 -->"));
        assert!(!footnote_line("<!-- footnotes -->"));
        assert!(!endnote_line("endnotes.p0"));
    }

    #[test]
    fn id_lines_carry_alignment_and_indents() {
        let line =
            id_line("<!-- p0 BodyText center, first-line 0.5in, left -0.25in, right 1in -->")
                .unwrap();
        assert_eq!(line.align, Some("center"));
        assert_eq!(line.first_line, Some(720));
        assert_eq!(line.left, Some(-360));
        assert_eq!(line.right, Some(1440));
        assert_eq!(line.hanging, None);
        let line = id_line("<!-- p1 justify, hanging 0.3in -->").unwrap();
        assert_eq!(
            (line.style, line.align, line.hanging),
            (None, Some("both"), Some(432))
        );
        // Out of Word's range, or not inches: no indent.
        let line = id_line("<!-- p2 left 99in, right 3cm, first-line NaNin -->").unwrap();
        assert_eq!((line.left, line.right, line.first_line), (None, None, None));
        // A range stands for several paragraphs; its facts are the first's.
        let line = id_line("<!-- p2-p4 empty -->").unwrap();
        assert_eq!((line.style, line.outline, line.align), (None, None, None));
        assert_eq!(id_line("<!-- t0 cells p2-p4 by row -->"), None);
    }

    #[test]
    fn id_lines_name_their_style() {
        for (line, style) in [
            ("<!-- p5 TitlePage -->", Some("TitlePage")),
            (
                "<!-- p5 TitlePage center, first-line 0.5in -->",
                Some("TitlePage"),
            ),
            ("<!-- p5 BodyText, page-break -->", Some("BodyText")),
            ("<!-- p5 BodyText right 1in -->", Some("BodyText")),
            ("<!-- p5 center -->", None),
            ("<!-- p5 left 0.5in -->", None),
            ("<!-- p5 -->", None),
            ("<!-- p5 empty -->", None),
            ("<!-- t0 TableGrid -->", None),
            ("<!-- page 3 of 9 -->", None),
            ("p5 TitlePage", None),
        ] {
            let named = id_line(line).and_then(|l| l.style);
            assert_eq!(named.as_deref(), style, "{line}");
        }
    }

    #[test]
    fn empty_id_lines_count_their_paragraphs() {
        for (line, count) in [
            ("<!-- p2 empty -->", Some(1)),
            ("<!-- p2-p4 empty -->", Some(3)),
            ("<!-- p2-p4 BodyText empty -->", Some(3)),
            ("<!-- p7 empty, page-break -->", Some(1)),
            ("  <!-- p0 empty -->  ", Some(1)),
            ("<!-- p2 -->", None),
            ("<!-- p2 BodyText center -->", None),
            ("<!-- p2 Empty -->", None),
            ("<!-- p2 BodyText, empty -->", Some(1)),
            ("<!-- p4-p2 empty -->", None),
            ("<!-- p02 empty -->", None),
            ("<!-- t0 empty -->", None),
            ("<!-- p0-p100000 empty -->", None),
            ("<!-- p0-p18446744073709551615 empty -->", None),
            ("<!-- p0-p99999 empty -->", Some(100_000)),
            ("p2 empty", None),
        ] {
            assert_eq!(empty_paragraphs(line), count, "{line}");
        }
    }

    #[test]
    fn a_table_lines_row_count_stays_inside_usize() {
        // A hand-edited range ending at usize::MAX is not a row count.
        for line in [
            "<!-- t0 1x1, cells t0.row0 p0-p18446744073709551615 -->",
            "<!-- t0 1x1, cells t0.row0 p5-p3 -->",
        ] {
            assert_eq!(table_line(line).unwrap().row_paragraphs, None, "{line}");
        }
        let line = table_line("<!-- t0 1x1, cells t0.row0 p0-p2 t0.row1 p3 -->").unwrap();
        assert_eq!(line.row_paragraphs, Some(vec![(0, 3), (3, 1)]));
    }

    #[test]
    fn the_authors_table_names_each_handle() {
        let yaml = "source: r.docx\nauthors:\n  document_owner: Arthur Souza Rodrigues  # cp:lastModifiedBy\n  AC: Ann Counsel                  # 9 revisions, 2 comments, 2026-10-01T09:00:00Z\n  AS: Arthur Souza Rodrigues       # 1 comment, 2026-10-09T16:13:00Z\n  JD: \"Jo #1 Doe\"  # 2 revisions, 2026-10-01..2026-10-02\n  R: Redline\nbody: p0-p20\n  XX: not an author\n";
        let table = authors(yaml);
        assert_eq!(table.len(), 4, "{table:?}");
        assert_eq!(
            table["AC"],
            Author {
                name: "Ann Counsel".into(),
                date: Some("2026-10-01T09:00:00Z".into()),
                range: None,
            }
        );
        assert_eq!(table["AS"].date.as_deref(), Some("2026-10-09T16:13:00Z"));
        assert_eq!(
            table["JD"],
            Author {
                name: "Jo #1 Doe".into(),
                date: None,
                range: Some(("2026-10-01".into(), "2026-10-02".into())),
            }
        );
        assert_eq!(table["R"].date, None);
    }

    #[test]
    fn plain_revision_notes_name_ids_then_author_and_date() {
        let plain = |ids: &[u32], author: &str, date: &str| {
            Some(PlainRevision {
                ids: ids.to_vec(),
                author: author.into(),
                date: date.into(),
            })
        };
        let date = "2026-03-02T10:00:00Z";
        assert_eq!(
            plain_revision(&format!("r5 Ann Lee ({date})")),
            plain(&[5], "Ann Lee", date)
        );
        assert_eq!(
            plain_revision(&format!("r2+1 ({date})")),
            plain(&[2, 1], "", date)
        );
        assert_eq!(plain_revision("r75 Di Eng"), plain(&[75], "Di Eng", ""));
        assert_eq!(plain_revision("r7"), plain(&[7], "", ""));
        assert_eq!(plain_revision("footnotes.r31 Ann"), plain(&[31], "Ann", ""));
        assert_eq!(plain_revision("noid Ann"), plain(&[], "Ann", ""));
        for text in [
            "why",
            "Di Eng",
            "5 Ann",
            "r5 @AC",
            "r5 Ann: why",
            "r5 Ann (soon)",
            &format!("r5 Ann ({date}): why"),
            &format!("Ann ({date})"),
        ] {
            assert_eq!(plain_revision(text), None, "{text}");
        }
    }

    #[test]
    fn nested_revision_notes_name_the_change_it_sits_inside() {
        let tag = |handle: &str, date: Option<&str>, ids: &[u32]| Tag {
            handle: handle.into(),
            date: date.map(str::to_string),
            ids: ids.to_vec(),
        };
        let plain = |ids: &[u32], author: &str, date: &str| {
            Some(PlainRevision {
                ids: ids.to_vec(),
                author: author.into(),
                date: date.into(),
            })
        };
        assert_eq!(
            nested_revision_tags("r3 @BO on r1 @AL"),
            Some(vec![tag("BO", None, &[3]), tag("AL", None, &[1])])
        );
        assert_eq!(
            nested_revision_tags("r3 @BO 2026-01-02T10:00:00Z on r1 @AL"),
            Some(vec![
                tag("BO", Some("2026-01-02T10:00:00Z"), &[3]),
                tag("AL", None, &[1])
            ])
        );
        // Deeper nests chain the same way, innermost first.
        assert_eq!(
            nested_revision_tags("r5 @CD on r3 @BO on r1 @AL"),
            Some(vec![
                tag("CD", None, &[5]),
                tag("BO", None, &[3]),
                tag("AL", None, &[1])
            ])
        );
        assert_eq!(
            nested_plain_revisions(
                "r3 Ben Ode (2026-01-02T10:00:00Z) on r1 Ana Lima (2026-01-01T09:00:00Z)"
            ),
            Some(vec![
                plain(&[3], "Ben Ode", "2026-01-02T10:00:00Z").unwrap(),
                plain(&[1], "Ana Lima", "2026-01-01T09:00:00Z").unwrap()
            ])
        );
        // An author's own name may hold ` on `: the ids still part the
        // changes only where both sides parse.
        assert_eq!(
            nested_plain_revisions(
                "r3 Ben Ode on High (2026-01-02T10:00:00Z) on r1 Ana Lima (2026-01-01T09:00:00Z)"
            ),
            Some(vec![
                plain(&[3], "Ben Ode on High", "2026-01-02T10:00:00Z").unwrap(),
                plain(&[1], "Ana Lima", "2026-01-01T09:00:00Z").unwrap()
            ])
        );
        // A note with no second change, or a right side that names no
        // change, is no nested note.
        assert_eq!(nested_revision_tags("r3 @BO"), None);
        assert_eq!(nested_revision_tags("r3 @BO on garbage"), None);
        assert_eq!(
            nested_plain_revisions("r5 Ana Lima on Verra (2026-…)"),
            None
        );
    }

    #[test]
    fn revision_notes_are_tags_and_nothing_else_is() {
        let tag = |handle: &str, date: Option<&str>, ids: &[u32]| Tag {
            handle: handle.into(),
            date: date.map(str::to_string),
            ids: ids.to_vec(),
        };
        assert_eq!(revision_tags("r0 @AC"), Some(vec![tag("AC", None, &[0])]));
        assert_eq!(
            revision_tags("r1+2 @AC"),
            Some(vec![tag("AC", None, &[1, 2])])
        );
        assert_eq!(
            revision_tags("r1 @AC; r2 @JD 2026-10-02T08:00:00Z"),
            Some(vec![
                tag("AC", None, &[1]),
                tag("JD", Some("2026-10-02T08:00:00Z"), &[2])
            ])
        );
        assert_eq!(
            revision_tags("header1.r4 @AC"),
            Some(vec![tag("AC", None, &[4])])
        );
        assert_eq!(
            revision_tags("Header_1:rev:4 @AC"),
            Some(vec![tag("AC", None, &[4])])
        );
        // A long id's w:id need not be a number; `noid` names none.
        assert_eq!(
            revision_tags("body:rev:x9 @AC"),
            Some(vec![tag("AC", None, &[])])
        );
        assert_eq!(revision_tags("noid @AC"), Some(vec![tag("AC", None, &[])]));
        for text in [
            "Ann Counsel (2026-10-01T09:00:00Z)",
            "r0 @AC: why?",
            "r0 AC",
            "p3 @AC",
            "r0 @A-C",
            "r0 @AC tomorrow",
            "#12 @AC",
            "please check r0 @AC",
            // A person's comment that happens to look like a tag (pi review).
            "5 @AC",
            "12 @home",
            "1+2 @AC",
            "",
        ] {
            assert_eq!(revision_tags(text), None, "{text}");
        }
    }

    /// The sizes Word's paper list names parse back to the twips the
    /// reader names them by: each name writes the sheet Word writes for
    /// it, so md -> docx -> md keeps both the name and the twips
    /// (#561 review defect 4).
    #[test]
    fn word_s_paper_list_parses_to_its_twips() {
        for (text, w, h) in [
            ("Letter portrait", 12_240, 15_840),
            ("Legal portrait", 12_240, 20_160),
            ("Executive portrait", 10_440, 15_120),
            ("Tabloid portrait", 15_840, 24_480),
            ("Ledger landscape", 24_480, 15_840),
            ("A3 portrait", 16_838, 23_811),
            ("A4 portrait", 11_906, 16_838),
            ("A5 landscape", 11_906, 8_391),
            ("B4 (JIS) portrait", 14_570, 20_636),
            ("B5 (JIS) portrait", 10_318, 14_570),
        ] {
            let page = page_setup(text).expect(text);
            assert_eq!(
                (page.width, page.height, page.landscape),
                (w, h, text.ends_with("landscape")),
                "{text}"
            );
        }
        // A size the list does not name stays the inches it spells.
        let custom = page_setup("8.5x14in portrait").unwrap();
        assert_eq!((custom.width, custom.height), (12_240, 20_160));
    }

    #[test]
    fn comment_heads_carry_author_date_thread_and_state() {
        let (head, text) = comment_head("c5 @AC: Is one percent the cap?").unwrap();
        assert_eq!(
            head,
            CommentHead {
                number: 5,
                handle: Some("AC".into()),
                date: None,
                reply_to: None,
                resolved: false
            }
        );
        assert_eq!(text, "Is one percent the cap?");
        let (head, text) = comment_head("c6 @AS re c5: Disagree.").unwrap();
        assert_eq!(
            (head.number, head.reply_to, text),
            (6, Some(5), "Disagree.")
        );
        let (head, _) = comment_head("c7 @AC 2026-10-01T09:00:00Z resolved: ok").unwrap();
        assert!(head.resolved);
        assert_eq!(head.date.as_deref(), Some("2026-10-01T09:00:00Z"));
        let (head, text) = comment_head("c8:").unwrap();
        assert_eq!((head.handle, text), (None, ""));
        assert_eq!(comment_head("c9: a: b").unwrap().1, "a: b");
        for text in [
            "Note: hi",
            "c5 @AC re: x",
            "c5 @AC maybe: x",
            "p5 @AC: x",
            "c5 @A-C: x",
        ] {
            assert_eq!(comment_head(text), None, "{text}");
        }
    }

    #[test]
    fn a_persons_comment_is_not_a_head() {
        // pi review of #462: `c5: x` without a handle and `3: item three`
        // kept their prefix only by luck; a bare number is never a head.
        for text in ["3: item three", "5 @AC: x", "Note: hi", "c5 @AC re 4: x"] {
            assert_eq!(comment_head(text), None, "{text}");
        }
        assert!(comment_head("c5 @AC re c4: x").is_some());
    }
}

/// The sheet sizes Word's own paper list names, in the twips Word writes
/// them, short side first. `Tabloid` and `Ledger` are the one sheet both
/// ways round — the portrait one is Tabloid, the landscape one Ledger —
/// and A4 small shares A4's twips, so the list names A4 only. The reader
/// (`from_docx::header`) matches a `w:pgSz` to a name by rounded twips,
/// and [`page_setup`] writes the same twips back for a name (#561).
pub(super) const WORD_PAGE_SIZES: &[(&str, u32, u32)] = &[
    ("Letter", 12_240, 15_840),
    ("Legal", 12_240, 20_160),
    ("Executive", 10_440, 15_120),
    ("Tabloid", 15_840, 24_480),
    ("Ledger", 15_840, 24_480),
    ("A3", 16_838, 23_811),
    ("A4", 11_906, 16_838),
    ("A5", 8_391, 11_906),
    ("B4 (JIS)", 14_570, 20_636),
    ("B5 (JIS)", 10_318, 14_570),
];

/// A section's page as the view's YAML header prints it (`page: A4
/// landscape, margins 0.5in, header/footer 0.49in`), in twips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PageSetup {
    pub width: u32,
    pub height: u32,
    pub landscape: bool,
    /// Top, right, bottom, left.
    pub margins: [u32; 4],
    pub header: u32,
    pub footer: u32,
    pub gutter: u32,
    /// Text columns and the gap between them.
    pub columns: u32,
    pub column_gap: u32,
    /// A line between the columns (`w:cols w:sep`).
    pub column_line: bool,
    /// The grid the lines snap to (`w:docGrid`): Word's type (`lines`,
    /// `linesAndChars`, `snapToChars`) and line pitch in twips, which a
    /// character grid may leave out.
    pub grid: Option<(&'static str, Option<u32>)>,
    /// `w:docGrid w:charSpace`, for a grid with characters.
    pub char_space: i32,
    /// How the section starts (`w:type`), when the view's page names it
    /// (`starts nextPage`); `None` writes none, Word's default.
    pub starts: Option<&'static str>,
}

impl PageSetup {
    /// The `w:sectPr` of this page.
    pub(super) fn sect_pr(&self) -> String {
        let [top, right, bottom, left] = self.margins;
        let num = if self.columns > 1 {
            format!(" w:num=\"{}\"", self.columns)
        } else {
            String::new()
        };
        let orient = if self.landscape {
            " w:orient=\"landscape\""
        } else {
            ""
        };
        let sep = if self.column_line { " w:sep=\"1\"" } else { "" };
        let grid = match self.grid {
            Some((kind, pitch)) => {
                let pitch = pitch.map_or_else(String::new, |p| format!(" w:linePitch=\"{p}\""));
                let space = if kind == "lines" {
                    String::new()
                } else {
                    format!(" w:charSpace=\"{}\"", self.char_space)
                };
                format!("<w:docGrid w:type=\"{kind}\"{pitch}{space}/>")
            }
            None => String::new(),
        };
        // `w:type` opens the page's own elements, after the references the
        // stories write into the `w:sectPr` (Word's order: references,
        // type, size, margins).
        let starts = self
            .starts
            .map_or_else(String::new, |kind| format!("<w:type w:val=\"{kind}\"/>"));
        format!(
            "<w:sectPr>{starts}<w:pgSz w:w=\"{}\" w:h=\"{}\"{orient}/><w:pgMar w:top=\"{top}\" \
             w:right=\"{right}\" w:bottom=\"{bottom}\" w:left=\"{left}\" w:header=\"{}\" \
             w:footer=\"{}\" w:gutter=\"{}\"/><w:cols w:space=\"{}\"{num}{sep}/>{grid}</w:sectPr>",
            self.width, self.height, self.header, self.footer, self.gutter, self.column_gap
        )
    }
}

/// The most text columns Word lets a section have.
const MOST_COLUMNS: u32 = 45;

/// The largest page side or margin a view can set: Word's 22 inches.
const MOST_PAGE: u32 = 31_680;

/// Twips from the view's inches (`0.49in`), within Word's page.
/// A grid's line pitch, `15.6pt`, in twips.
fn grid_pitch(value: &str) -> Option<u32> {
    let points: f64 = value.strip_suffix("pt")?.parse().ok()?;
    let twips = (points * 20.0).round();
    // `as` cannot overflow: the bound is checked first.
    (twips.is_finite() && (1.0..=f64::from(MOST_PAGE)).contains(&twips)).then_some(twips as u32)
}

fn page_twips(value: &str) -> Option<u32> {
    let inches: f64 = value.strip_suffix("in")?.parse().ok()?;
    let twips = (inches * 1440.0).round();
    // `as` cannot overflow: the bound is checked first.
    (twips.is_finite() && (0.0..=f64::from(MOST_PAGE)).contains(&twips)).then_some(twips as u32)
}

/// The page `text` names: a size of Word's paper list (`Letter portrait`,
/// `B5 (JIS) portrait`, `Ledger landscape`) or the sheet as it lies
/// (`8.5x14in portrait`), then `margins 1in` (or `margins top 1in, right
/// 1.25in, bottom 1in, left 1.25in`), `header/footer 0.5in` (or `header
/// 0.5in, footer 0.6in`), `gutter 0.2in`, `columns 2`, `column gap
/// 0.25in`, `column line`, and a grid (`line grid 15.6pt`, `line and char
/// grid 18pt`, `char grid 18pt`, then `char space 4096`). A `; fmt …` tail
/// is left out.
pub(super) fn page_setup(text: &str) -> Option<PageSetup> {
    let text = text.split(';').next()?.trim();
    let mut clauses = text.split(", ");
    let (size, orientation) = clauses.next()?.rsplit_once(' ')?;
    let landscape = match orientation {
        "landscape" => true,
        "portrait" => false,
        _ => return None,
    };
    let (width, height) =
        if let Some(&(_, w, h)) = WORD_PAGE_SIZES.iter().find(|&&(name, _, _)| name == size) {
            (w, h)
        } else {
            // `8.5x14in`: the sheet as it lies, not its short side first.
            let (w, h) = size.split_once('x')?;
            let (w, h) = (page_twips(&format!("{w}in"))?, page_twips(h)?);
            (w.min(h), w.max(h))
        };
    let (width, height) = if landscape {
        (height, width)
    } else {
        (width, height)
    };
    let mut page = PageSetup {
        width,
        height,
        landscape,
        margins: [1440; 4],
        header: 720,
        footer: 720,
        gutter: 0,
        columns: 1,
        column_gap: 720,
        column_line: false,
        grid: None,
        char_space: 0,
        starts: None,
    };
    for clause in clauses {
        if clause == "column line" {
            page.column_line = true;
            continue;
        }
        if clause == "char grid" {
            // A character grid with no line pitch.
            page.grid = Some(("snapToChars", None));
            continue;
        }
        let (name, value) = clause.rsplit_once(' ')?;
        let grid = match name {
            "line grid" => Some("lines"),
            "line and char grid" => Some("linesAndChars"),
            "char grid" => Some("snapToChars"),
            _ => None,
        };
        if let Some(kind) = grid {
            page.grid = Some((kind, Some(grid_pitch(value)?)));
            continue;
        }
        if name == "char space" {
            page.char_space = value.parse().ok()?;
            continue;
        }
        if name == "columns" {
            page.columns = value
                .parse()
                .ok()
                .filter(|n| (1..=MOST_COLUMNS).contains(n))?;
            continue;
        }
        if name == "starts" {
            page.starts = match value {
                "nextPage" => Some("nextPage"),
                "continuous" => Some("continuous"),
                "evenPage" => Some("evenPage"),
                "oddPage" => Some("oddPage"),
                "nextColumn" => Some("nextColumn"),
                // A `w:type` Word does not define: the page does not read.
                _ => return None,
            };
            continue;
        }
        let twips = page_twips(value)?;
        match name {
            "margins" => page.margins = [twips; 4],
            "margins top" | "top" => page.margins[0] = twips,
            "right" => page.margins[1] = twips,
            "bottom" => page.margins[2] = twips,
            "left" => page.margins[3] = twips,
            "header/footer" => (page.header, page.footer) = (twips, twips),
            "header" => page.header = twips,
            "footer" => page.footer = twips,
            "gutter" => page.gutter = twips,
            "column gap" => page.column_gap = twips,
            _ => return None,
        }
    }
    Some(page)
}

/// A `sections:` entry of the view's YAML header (`  2: {p30-p103, page:
/// Letter landscape, margins 1in, headers: {…}}`): the section's number and
/// its page, when it names one (else it keeps the first section's).
pub(super) fn section_page(line: &str) -> Option<(usize, Option<PageSetup>)> {
    let (number, entry) = line.trim().split_once(": ")?;
    let number: usize = number.parse().ok()?;
    let entry = entry.strip_prefix('{')?;
    let Some((_, page)) = entry.split_once(", page: ") else {
        return Some((number, None));
    };
    // The page's own clauses end where the entry's next key starts.
    let end = [", fmt: ", ", headers: ", ", footers: ", ", columns: "]
        .iter()
        .filter_map(|key| page.find(key))
        .min()
        .or_else(|| page.rfind('}'))
        .unwrap_or(page.len());
    Some((number, page_setup(&page[..end])))
}
