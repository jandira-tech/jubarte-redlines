// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The short ids as values (docs/superpowers/specs/
//! 2026-10-13-ids-and-markdown-roundtrip.md §2 and §2.1). [`ShortId`] is the
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
//! Numbers are ASCII digits without leading zeros, so each element has one
//! spelling. Whether the element exists is the document's question.

use std::fmt;
use std::str::FromStr;

use super::{IdError, digit_shaped, is_story, number};

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
    /// `pN`: a paragraph (in the story, a cell, or a text box).
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

/// Whether `head` names a part: a story stem that is not the body (never
/// printed as a part) and not shaped like an id of its own (`s0`, `toc0`,
/// `box0`, `cc0`).
fn part_stem(head: &str) -> bool {
    is_story(head)
        && head != "body"
        && !["s", "toc", "box", "cc"]
            .iter()
            .any(|p| digit_shaped(head, p))
}

/// Whether `path` is a block the grammar can name: a paragraph, table or
/// content control of the story; a table's rows, a row's cells; a cell's
/// paragraphs and nested tables; a paragraph's text boxes (each holding
/// paragraphs) and inline content controls. A part may be named by itself
/// (`header1`); the body may not.
fn valid_path(story: &Story, path: &[Step]) -> bool {
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
            (At::Story | At::Cell | At::TextBox, Step::Paragraph(_)) => At::Paragraph,
            (At::Story | At::Cell, Step::Table(_)) => At::Table,
            (At::Table, Step::Row(_)) => At::Row,
            (At::Row, Step::Cell(_)) => At::Cell,
            (At::Paragraph, Step::TextBox(_)) => At::TextBox,
            (At::Story | At::Paragraph, Step::ContentControl(_)) => At::Leaf,
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
    /// The blocks from `first` to `last`, both included (`p3-p7`).
    Range {
        /// The first block; never itself a range.
        first: Box<ShortId>,
        /// The last block; never itself a range.
        last: Box<ShortId>,
    },
    /// Tracked changes by `w:id`: one (`r12`) or a pair shown as one
    /// (`r1+2`), in a story (`header1.r1`).
    Change {
        /// The story the changes are in.
        story: Story,
        /// Their `w:id`s, at least one.
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

    fn parse(token: &str) -> Option<Self> {
        if let Some((first, last)) = token.split_once('-') {
            let first = Self::block(first)?;
            let last = Self::block(last)?;
            return Some(Self::Range {
                first: Box::new(first),
                last: Box::new(last),
            });
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
        Self::parse(token).ok_or_else(|| IdError::new(token, SHORT_EXPECTED))
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
            Self::Range { first, last } => write!(f, "{first}-{last}"),
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
            "header1.t0.row0.cell0.p0",
            "p14.box0.p0",
            "p14.box0.p1.box0.p0",
            "cc0",
            "p12.cc0",
            "p3-p7",
            "p10-p11",
            "header1.p0-header1.p2",
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
}
