// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The short ids as values (docs/superpowers/specs/
//! 2026-10-13-ids-and-markdown-roundtrip.md §2 and §2.1; the conflicting
//! clauses are superseded by 2026-10-17-short-and-long-id-contract.md).
//! [`ShortId`] is the
//! one type every short id is: it parses only the settled spellings (Arthur,
//! 2026-10-13: "We should strictly have the settled short IDs or long IDs.
//! Everything else should be expired") and its `Display` is the only
//! printer, so what jubarte prints is what it parses.
//!
//! | Kind | Spelling |
//! |-|-|
//! | body paragraph | `p12` |
//! | story (a part) and its paragraphs | `header1`, `header1.p0`, `footnotes.p3` |
//! | table, row, cell, cell paragraph | `t0`, `t0.row1`, `t0.row1.cell2`, `t0.row1.cell2.p1` |
//! | nested table | `t0.row1.cell2.t0.row0.cell0` |
//! | text-box paragraph | `p14.box0.p0` |
//! | content control | `cc0`, `p12.cc0` |
//! | range | `p3-p7` |
//! | change | `r12`, `r1+2`, `header1.r1`, `styles.r0` |
//! | comment, the text it covers | `c5`, `c5.text` |
//! | section, table of contents | `s0`, `toc0` |
//!
//! `t0.row1.cell2.p1` and `p14.box0.p0` are the pre-contract scoped
//! spellings: the 17 October 2026 contract respells a cell's or a text box's
//! own paragraphs `psN` (`t0.row1.cell2.ps1`, `p14.box0.ps0`), a change
//! staged in docs/superpowers/plans/2026-10-16-id-contract-implementation.md
//! and not implemented.
//!
//! Numbers are ASCII digits without leading zeros, so each element has one
//! spelling. Whether the element exists is the document's question.

use std::fmt;
use std::str::FromStr;

use super::{IdError, number};

/// Why a short id was refused, when no narrower reason applies.
pub const SHORT_EXPECTED: &str = "expected a short id such as p12, header1.p0, t0.row1.cell2, p14.box0.p0, cc0, p3-p7, r12, r1+2, header1.r1, c5, c5.text, s0 or toc0";

/// The story an id lives in: the body, or a part named by its file stem
/// (`header1`, `footer2`, `footnotes`, `endnotes`, `comments`, `styles`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Story {
    /// `word/document.xml`'s body.
    Body,
    /// Another part, by file stem.
    Part(String),
}

/// One step of a block's path inside its story.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Step {
    /// `pN`: a paragraph (in the story, a cell, or a text box; the contract
    /// splits the scoped cases out as `psN`, not implemented).
    Paragraph(usize),
    /// `tN`: a table (in the story, or nested in a cell).
    Table(usize),
    /// `rowN`: a row of the table before it.
    Row(usize),
    /// `cellN`: a cell of the row before it.
    Cell(usize),
    /// `boxN`: a text box anchored in the paragraph before it.
    TextBox(usize),
    /// `ccN`: a content control, in the story or in the paragraph before it.
    ContentControl(usize),
}

impl Step {
    fn parse(text: &str) -> Option<Self> {
        let word = |prefix: &str| text.strip_prefix(prefix).and_then(number::<usize>);
        if let Some(n) = word("row") {
            return Some(Self::Row(n));
        }
        if let Some(n) = word("cell") {
            return Some(Self::Cell(n));
        }
        if let Some(n) = word("box") {
            return Some(Self::TextBox(n));
        }
        if let Some(n) = word("cc") {
            return Some(Self::ContentControl(n));
        }
        if let Some(n) = word("p") {
            return Some(Self::Paragraph(n));
        }
        word("t").map(Self::Table)
    }
}

impl Step {
    /// The step's number.
    pub fn number(self) -> usize {
        match self {
            Self::Paragraph(n)
            | Self::Table(n)
            | Self::Row(n)
            | Self::Cell(n)
            | Self::TextBox(n)
            | Self::ContentControl(n) => n,
        }
    }

    /// The same kind of step, numbered `n`.
    fn renumbered(self, n: usize) -> Self {
        match self {
            Self::Paragraph(_) => Self::Paragraph(n),
            Self::Table(_) => Self::Table(n),
            Self::Row(_) => Self::Row(n),
            Self::Cell(_) => Self::Cell(n),
            Self::TextBox(_) => Self::TextBox(n),
            Self::ContentControl(_) => Self::ContentControl(n),
        }
    }
}

/// The settled spelling of an expired id, when one is clear (spec §2):
/// `12` → `p12, r12 or c12`, `#12` → `r12`, `#c27` → `c27`, `t0.r1.c2` →
/// `t0.row1.cell2`, `body:rev:12` → `r12`, `header1:rev:4` → `header1.r4`,
/// `body:p:12` → `p12`, `header1:p:2` → `header1.p2`, `1+2` → `r1+2` (only
/// a change id joins numbers), `t0.row1.cell2.p0` → `t0.row1.cell2` (a cell
/// is its first paragraph). A suggestion is given only when every spelling
/// it names parses (`05` suggests nothing; pi review of #464: the bare
/// number's three alternatives skipped that gate).
fn settled_spelling(token: &str) -> Option<String> {
    if !token.is_empty() && token.bytes().all(|b| b.is_ascii_digit()) {
        let n = number::<u32>(token)?;
        let each = [format!("p{n}"), format!("r{n}"), format!("c{n}")];
        return each
            .iter()
            .all(|s| ShortId::parse(s).is_some())
            .then(|| format!("{}, {} or {}", each[0], each[1], each[2]));
    }
    let settled = |id: String| ShortId::parse(&id).is_some().then_some(id);
    if token.contains('+') && token.bytes().all(|b| b.is_ascii_digit() || b == b'+') {
        return settled(format!("r{token}"));
    }
    if let Some(cell) = token.strip_suffix(".p0")
        && cell
            .rsplit('.')
            .next()
            .is_some_and(|l| l.starts_with("cell"))
    {
        return settled(cell.to_string());
    }
    let suggestion = if let Some(rest) = token.strip_prefix("#c") {
        format!("c{rest}")
    } else if let Some(rest) = token.strip_prefix('#') {
        format!("r{rest}")
    } else if let Some((story, wid)) = token.split_once(":rev:") {
        match story {
            "body" => format!("r{wid}"),
            _ => format!("{story}.r{wid}"),
        }
    } else if let Some((story, n)) = token.split_once(":p:") {
        match story {
            "body" => format!("p{n}"),
            _ => format!("{story}.p{n}"),
        }
    } else {
        let parts: Vec<&str> = token.split('.').collect();
        let [t, r, c, rest @ ..] = parts.as_slice() else {
            return None;
        };
        let (r, c) = (r.strip_prefix('r')?, c.strip_prefix('c')?);
        let mut s = format!("{t}.row{r}.cell{c}");
        // 0.12 spelled a cell's first paragraph `.p0`; it is the cell now.
        let rest = match rest {
            ["p0"] => &[][..],
            rest => rest,
        };
        for part in rest {
            s.push('.');
            s.push_str(part);
        }
        s
    };
    settled(suggestion)
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Paragraph(n) => write!(f, "p{n}"),
            Self::Table(n) => write!(f, "t{n}"),
            Self::Row(n) => write!(f, "row{n}"),
            Self::Cell(n) => write!(f, "cell{n}"),
            Self::TextBox(n) => write!(f, "box{n}"),
            Self::ContentControl(n) => write!(f, "cc{n}"),
        }
    }
}

/// The parts other than the body that hold blocks or changes the view
/// names; headers and footers are `headerN` and `footerN`.
const PARTS: &[&str] = &["footnotes", "endnotes", "comments", "styles", "numbering"];

/// Whether `head` names a part (grok review of #24a: any lowercase word was
/// a part, so `row`, `page` or `body1` parsed).
fn part_stem(head: &str) -> bool {
    PARTS.contains(&head)
        || ["header", "footer"]
            .iter()
            .any(|p| head.strip_prefix(p).and_then(number::<u32>).is_some())
}

/// Whether `path` is a block the grammar can name: a paragraph, table or
/// content control of the story; a table's rows, a row's cells; a cell's
/// paragraphs and nested tables; a paragraph's text boxes (each holding
/// paragraphs and tables) and inline content controls. A cell stands for its
/// first paragraph, so it takes that paragraph's boxes and controls
/// (`t0.row1.cell2.box0.p0`, `t0.row1.cell2.cc0`), and `.p0` after a cell is
/// refused: that paragraph has one spelling. A part may be named by itself
/// (`header1`, the part; its first paragraph is `header1.p0`); the body may
/// not. `styles` and `numbering` hold no blocks (pi review of #464:
/// `styles.p0` parsed): they are named only by themselves and their changes.
fn valid_path(story: &Story, path: &[Step]) -> bool {
    if let Story::Part(stem) = story
        && matches!(stem.as_str(), "styles" | "numbering")
    {
        return path.is_empty();
    }
    #[derive(Clone, Copy, PartialEq)]
    enum At {
        Story,
        Paragraph,
        Table,
        Row,
        Cell,
        TextBox,
        Leaf,
    }
    let mut at = At::Story;
    for step in path {
        at = match (at, step) {
            (At::Cell, Step::Paragraph(0)) => return false,
            (At::Story | At::Cell | At::TextBox, Step::Paragraph(_)) => At::Paragraph,
            (At::Story | At::Cell | At::TextBox, Step::Table(_)) => At::Table,
            (At::Table, Step::Row(_)) => At::Row,
            (At::Row, Step::Cell(_)) => At::Cell,
            (At::Paragraph | At::Cell, Step::TextBox(_)) => At::TextBox,
            (At::Story | At::Paragraph | At::Cell, Step::ContentControl(_)) => At::Leaf,
            _ => return false,
        };
    }
    match at {
        At::Story => matches!(story, Story::Part(_)),
        At::TextBox => false,
        _ => true,
    }
}

/// A short id (see the module doc).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShortId {
    /// A story (an empty path names the part itself), or a block in it.
    Block {
        /// Where the block lives.
        story: Story,
        /// Its path from the story; [`valid_path`] holds.
        path: Vec<Step>,
    },
    /// The blocks from `first` to its sibling number `last`, both included:
    /// `p3-p7`, `footnotes.p2-p3`, `t0.row1.cell2.p1-p3` (the view's
    /// spelling: the second side is the last step of the first, renumbered).
    Range {
        /// The first block: a [`ShortId::Block`] with a path.
        first: Box<ShortId>,
        /// The number of the last sibling, not below the first's.
        last: usize,
    },
    /// Tracked changes by `w:id`: one (`r12`) or a pair shown as one
    /// (`r1+2`), in a story (`header1.r1`).
    Change {
        /// The story the changes are in.
        story: Story,
        /// Their `w:id`s, at least one. A deletion and an insertion that
        /// share one `w:id` (Word writes such pairs) print it twice: `r43+43`.
        wids: Vec<u32>,
    },
    /// A comment, by `w:id` (`c5`).
    Comment(u32),
    /// The text a comment covers (`c5.text`).
    CommentText(u32),
    /// A section, in body order (`s0`).
    Section(usize),
    /// A table of contents (`toc0`).
    Toc(usize),
}

impl ShortId {
    /// A body paragraph, `p{n}`.
    pub fn paragraph(n: usize) -> Self {
        Self::Block {
            story: Story::Body,
            path: vec![Step::Paragraph(n)],
        }
    }

    /// A body change, `r{wid}`.
    pub fn change(wid: u32) -> Self {
        Self::Change {
            story: Story::Body,
            wids: vec![wid],
        }
    }

    fn block(token: &str) -> Option<Self> {
        let mut segments = token.split('.');
        let head = segments.next()?;
        let (story, first) = match Step::parse(head) {
            Some(step) => (Story::Body, Some(step)),
            None if part_stem(head) => (Story::Part(head.to_string()), None),
            None => return None,
        };
        let path: Vec<Step> = first
            .into_iter()
            .map(Some)
            .chain(segments.map(Step::parse))
            .collect::<Option<_>>()?;
        valid_path(&story, &path).then_some(Self::Block { story, path })
    }

    fn change_of(token: &str) -> Option<Self> {
        let (story, rest) = match token.rsplit_once('.') {
            Some((story, rest)) if part_stem(story) => (Story::Part(story.to_string()), rest),
            Some(_) => return None,
            None => (Story::Body, token),
        };
        let wids = rest
            .strip_prefix('r')?
            .split('+')
            .map(number::<u32>)
            .collect::<Option<Vec<_>>>()?;
        Some(Self::Change { story, wids })
    }

    /// `FIRST-N`: a block, then the number of its last step's last sibling.
    fn range(first: &str, last: &str) -> Option<Self> {
        let first = Self::block(first)?;
        let Self::Block { path, .. } = &first else {
            return None;
        };
        let from = *path.last()?;
        let to = Step::parse(last)?;
        let same_kind = std::mem::discriminant(&from) == std::mem::discriminant(&to);
        (same_kind && from.number() <= to.number()).then(|| Self::Range {
            first: Box::new(first.clone()),
            last: to.number(),
        })
    }

    fn parse(token: &str) -> Option<Self> {
        if let Some((first, last)) = token.split_once('-') {
            return Self::range(first, last);
        }
        if let Some(n) = token
            .strip_suffix(".text")
            .and_then(|h| h.strip_prefix('c'))
        {
            return number(n).map(Self::CommentText);
        }
        if let Some(n) = token.strip_prefix("toc") {
            return number(n).map(Self::Toc);
        }
        let digits = |prefix: char| {
            token
                .strip_prefix(prefix)
                .filter(|d| d.bytes().all(|b| b.is_ascii_digit()))
        };
        if let Some(n) = digits('c') {
            return number(n).map(Self::Comment);
        }
        if let Some(n) = digits('s') {
            return number(n).map(Self::Section);
        }
        Self::change_of(token).or_else(|| Self::block(token))
    }
}

impl FromStr for ShortId {
    type Err = IdError;

    fn from_str(token: &str) -> Result<Self, IdError> {
        Self::parse(token).ok_or_else(|| IdError {
            hint: settled_spelling(token),
            ..IdError::new(token, SHORT_EXPECTED)
        })
    }
}

impl fmt::Display for Story {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Body => f.write_str("body"),
            Self::Part(stem) => f.write_str(stem),
        }
    }
}

impl fmt::Display for ShortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = |f: &mut fmt::Formatter<'_>, story: &Story, more: bool| match story {
            Story::Body => Ok(()),
            Story::Part(stem) if more => write!(f, "{stem}."),
            Story::Part(stem) => f.write_str(stem),
        };
        match self {
            Self::Block { story, path } => {
                prefix(f, story, !path.is_empty())?;
                let steps: Vec<String> = path.iter().map(Step::to_string).collect();
                f.write_str(&steps.join("."))
            }
            Self::Range { first, last } => {
                let Self::Block { path, .. } = first.as_ref() else {
                    return write!(f, "{first}-{last}");
                };
                let step = path
                    .last()
                    .map_or(Step::Paragraph(*last), |s| s.renumbered(*last));
                write!(f, "{first}-{step}")
            }
            Self::Change { story, wids } => {
                prefix(f, story, true)?;
                let wids: Vec<String> = wids.iter().map(u32::to_string).collect();
                write!(f, "r{}", wids.join("+"))
            }
            Self::Comment(n) => write!(f, "c{n}"),
            Self::CommentText(n) => write!(f, "c{n}.text"),
            Self::Section(n) => write!(f, "s{n}"),
            Self::Toc(n) => write!(f, "toc{n}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_settled_spelling_parses_and_prints_back() {
        for token in [
            "p0",
            "p12",
            "header1",
            "header1.p0",
            "footnotes.p3",
            "endnotes.p0",
            "t0",
            "t0.row1",
            "t0.row1.cell2",
            "t0.row1.cell2.p1",
            "t0.row1.cell2.t0.row0.cell0",
            "header1.t0.row0.cell0",
            "r43+43",
            "p14.box0.p0",
            "p14.box0.p1.box0.p0",
            "cc0",
            "p12.cc0",
            "p3-p7",
            "p10-p11",
            "footnotes.p2-p3",
            "header1.p0-p2",
            "p4-p4",
            "t0.row1.cell2.p1-p3",
            "t0.row1.cell2.box0.p0",
            "t0.row1.cell2.cc0",
            "p14.box0.t0.row0.cell0",
            "numbering.r3",
            "r0",
            "r12",
            "r1+2",
            "r1+2+3",
            "header1.r1",
            "footnotes.r3",
            "styles.r0",
            "footer2.r1648",
            "c0",
            "c5",
            "c5.text",
            "s0",
            "s12",
            "toc0",
        ] {
            let id: ShortId = token.parse().unwrap_or_else(|e| panic!("{token}: {e}"));
            assert_eq!(id.to_string(), token, "{id:?}");
        }
    }

    #[test]
    fn expired_and_malformed_spellings_are_refused() {
        // Arthur, 2026-10-13: everything but the settled ids is expired.
        for token in [
            "",
            "12",
            "#12",
            "#c27",
            "#c27.text",
            "t0.r1.c2",
            "body:rev:12",
            "body:p:12",
            "styles.p0",
            "numbering.t0",
            "body.p2",
            "body.r12",
            "p",
            "p01",
            "r01",
            "c05",
            "s01",
            "toc01",
            "p-1",
            "p3-",
            "-p3",
            "p3-p4-p5",
            "p3-r4",
            "r1+",
            "r+1",
            "r1++2",
            "header1.r",
            "row1",
            "cell2",
            "t0.cell2",
            "t0.row1.p0",
            "t0.row1.cell2.row0",
            "p12.box0",
            "box0.p0",
            "p12.cc0.p0",
            "header1.header2",
            "Header1",
            "c5.text.p0",
            "s0.p1",
            "toc0.p0",
            "p12 ",
            "p 12",
            "p4294967296999999999999",
            // grok review of #24a
            "row",
            "cell",
            "page",
            "body1",
            "s.p0",
            "row.p0",
            "page.p0",
            "header01",
            "header",
            "p3-t0",
            "p7-p3",
            "footnotes.p2-header1.p3",
            "header1.p0-header1.p2",
            "header1-footer1",
            "cc0-p12",
            "t0.row1.cell2-p1",
            "t0.row1.cell2.p0",
            "t0.row1.cell2.p0.box0.p0",
        ] {
            assert!(
                token.parse::<ShortId>().is_err(),
                "{token} parsed as {:?}",
                token.parse::<ShortId>()
            );
        }
    }

    #[test]
    fn the_constructors_print_what_the_view_prints() {
        assert_eq!(ShortId::paragraph(12).to_string(), "p12");
        assert_eq!(ShortId::change(7).to_string(), "r7");
    }

    #[test]
    fn an_expired_id_is_refused_with_its_settled_spelling() {
        for (token, hint) in [
            ("12", Some("p12, r12 or c12")),
            ("#12", Some("r12")),
            ("#1+2", Some("r1+2")),
            ("#c27", Some("c27")),
            ("#c27.text", Some("c27.text")),
            ("t0.r1.c2", Some("t0.row1.cell2")),
            ("t0.r1.c2.p1", Some("t0.row1.cell2.p1")),
            ("body:rev:12", Some("r12")),
            ("header1:rev:4", Some("header1.r4")),
            ("body:p:12", Some("p12")),
            ("header1:p:2", Some("header1.p2")),
            ("05", None),
            ("4294967296", None),
            ("1+2", Some("r1+2")),
            ("1+", None),
            ("t0.row1.cell2.p0", Some("t0.row1.cell2")),
            ("t0.r1.c2.p0", Some("t0.row1.cell2")),
            ("p3.p0", None),
            ("body:rev:007", None),
            ("Header_1:rev:4", None),
            ("nonsense", None),
        ] {
            let error = token.parse::<ShortId>().unwrap_err();
            assert_eq!(error.hint.as_deref(), hint, "{token}");
            if let Some(hint) = hint {
                let message = error.to_string();
                assert!(
                    message.ends_with(&format!("the settled spelling is {hint}")),
                    "{message}"
                );
            }
        }
    }
}
