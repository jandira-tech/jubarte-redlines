// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The one grammar of the ids jubarte prints and reads.
//!
//! - **Revisions** (tracked changes): `r12`, `12`, `r1+2` (both), `header1.r1`,
//!   `footnotes.r3`, the long `body:rev:12`; 0.12's printed `#12` and `#1+2`
//!   still parse.
//! - **Comments:** `c27`, `27`; 0.12's `#c27` still parses.
//! - **Paragraphs:** `p12`, `12`, `header1` (its first paragraph),
//!   `header1.p2`, `footnotes.p3`, a cell's `t0.row1.cell2[.p1]`, the long
//!   `body:p:12`; 0.12's cell form `t0.r1.c2[.p1]` still parses.
//!
//! This module reads syntax only. Whether a story, paragraph, change or
//! comment exists is decided against a document by `edit` (`-p`) and
//! [`crate::changes`] (`--id`). Every number has one spelling: ASCII digits
//! without leading zeros, so two tokens never name one element; a long
//! change id carries the `w:id` as the document spells it, so a change whose
//! `w:id` the short form cannot spell (`007`) is still named. Every short
//! id it prints uses only `[a-z0-9.+:-]` and never starts with `#`, `=`, `~`
//! or `-`, so it is safe unquoted in bash, csh and zsh; a long id carries
//! the story and `w:id` the document gives it. `body.r12` and `body.p2`
//! parse (the body is a story too) but are never printed.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};

/// Why a change id was refused.
pub const CHANGE_EXPECTED: &str = "expected a change id such as r12, header1.r1 or body:rev:12";
/// Why a comment id was refused.
pub const COMMENT_EXPECTED: &str = "expected a comment id such as c27 or 27";
/// Why a `-p` value was refused, when no narrower reason applies.
pub const LOCATION_EXPECTED: &str = "-p takes a paragraph (p12 or 12), a story paragraph (header1, header1.p2, footnotes.p3), a cell (t0.row1.cell2), a comment (c5) or a revision (r12)";
/// Why a table id without its row and cell was refused.
pub const TABLE_NEEDS_CELL: &str = "a table id needs a row and a cell, as t0.row1.cell2";
/// Why a row or cell without its table was refused.
pub const CELL_NEEDS_TABLE: &str = "a row or cell needs its table, as t0.row1.cell2";

/// The change spellings the manifest advertises.
pub const CHANGE_SPELLINGS: &[&str] = &[
    "r12",
    "12",
    "r1+2",
    "header1.r1",
    "footnotes.r3",
    "body:rev:12",
];
/// The comment spellings the manifest advertises.
pub const COMMENT_SPELLINGS: &[&str] = &["c27", "27"];
/// The paragraph spellings the manifest advertises.
pub const PARAGRAPH_SPELLINGS: &[&str] = &[
    "p12",
    "12",
    "header1",
    "header1.p2",
    "footnotes.p3",
    "t0.row1.cell2",
    "t0.row1.cell2.p1",
    "body:p:12",
];

/// The table spellings the manifest advertises (rows and cells sit under
/// their table, as in [`PARAGRAPH_SPELLINGS`]).
pub const TABLE_SPELLINGS: &[&str] = &["t0"];

/// A refused id: what was given and what was expected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdError {
    /// The token as given.
    pub given: String,
    /// What the token should have been.
    pub expected: &'static str,
}

impl IdError {
    fn new(given: &str, expected: &'static str) -> Self {
        Self {
            given: given.to_string(),
            expected,
        }
    }
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.given, self.expected)
    }
}

impl std::error::Error for IdError {}

/// One tracked change: its story (`body`, `header1`, `footnotes`, …) and
/// its `w:id`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChangeId {
    /// The story: `body`, or the part's file stem.
    pub story: String,
    /// The change's `w:id`, as the document spells it.
    pub wid: String,
}

impl ChangeId {
    /// The change `wid` of `story`.
    pub fn new(story: impl Into<String>, wid: impl Into<String>) -> Self {
        Self {
            story: story.into(),
            wid: wid.into(),
        }
    }

    /// `{story}:rev:{wid}`, the id JSON and machine hints carry.
    pub fn long(&self) -> String {
        format!("{}:rev:{}", self.story, self.wid)
    }

    /// The id the view and listings print: `r12` in the body,
    /// `header1.r1` elsewhere, the long id when the story or the `w:id`
    /// has no short spelling.
    pub fn short(&self) -> String {
        if number::<u32>(&self.wid).is_none() {
            return self.long();
        }
        if self.story == "body" {
            format!("r{}", self.wid)
        } else if is_story(&self.story) {
            format!("{}.r{}", self.story, self.wid)
        } else {
            self.long()
        }
    }

    /// The change a long id names; `None` when `long` is not one. The
    /// `w:id` is taken as the document spells it (`body:rev:007` names a
    /// `w:id="007"`, not `r7`), without `:`, `+` or whitespace, so every id
    /// [`ChangeId::short`] prints reads back. The short forms keep one
    /// spelling per number.
    pub fn from_long(long: &str) -> Option<Self> {
        let (story, wid) = long.split_once(":rev:")?;
        let wid_ok =
            !wid.is_empty() && !wid.contains([':', '+']) && !wid.contains(char::is_whitespace);
        (!story.is_empty() && !story.contains(':') && wid_ok).then(|| Self::new(story, wid))
    }
}

/// What a `-p` value names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Location {
    /// A paragraph, in a form `edit` resolves (`p12`, `header1.p2`,
    /// `t0.row1.cell2`, `body:p:12`); a bare `12` arrives as `p12`.
    Paragraph(String),
    /// A comment, by `w:id`.
    Comment(u32),
    /// Tracked changes (`r12`, `header1.r1`, `r1+2`).
    Changes(Vec<ChangeId>),
}

/// The number `digits` spells: ASCII digits, no leading zero (`0` alone is
/// zero), within `T`.
fn number<T: FromStr>(digits: &str) -> Option<T> {
    let canonical = !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'));
    canonical.then(|| digits.parse().ok()).flatten()
}

/// Whether `text` is `prefix` followed by ASCII digits (canonical or not):
/// the shape that keeps such a word from being a story stem.
fn digit_shaped(text: &str, prefix: &str) -> bool {
    text.strip_prefix(prefix)
        .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
}

/// The number after `prefix` in a part of a short id (`p12`, `t0`): ASCII
/// digits without leading zeros.
pub fn short_number(text: &str, prefix: char) -> Option<usize> {
    number(text.strip_prefix(prefix)?)
}

/// The number after a word prefix (`row1`, `cell2`).
fn word_number(text: &str, prefix: &str) -> Option<usize> {
    number(text.strip_prefix(prefix)?)
}

/// Whether `stem` can be a short story id: `[a-z][a-z0-9]*`, neither a bare
/// `p`, `t`, `c` or `r` nor shaped like `p12`, `t0`, `c5`, `r1`, `row1` or
/// `cell1`. Syntactic only; the document decides whether the story exists.
pub fn is_story(stem: &str) -> bool {
    let mut bytes = stem.bytes();
    let well_formed = bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit());
    well_formed
        && !["p", "t", "c", "r"].contains(&stem)
        && !["p", "t", "c", "r", "row", "cell"]
            .iter()
            .any(|prefix| digit_shaped(stem, prefix))
}

/// `1+2+3` → the `w:id`s, each a canonical `u32`.
fn plus_list(text: &str) -> Option<Vec<String>> {
    text.split('+')
        .map(|part| number::<u32>(part).map(|n| n.to_string()))
        .collect()
}

/// The changes a change id names: `r12`, `12`, `#12`, `r1+2`, `header1.r1`,
/// `header1.r1+2`, `body:rev:12`. A story is checked only syntactically.
pub fn parse_change_ids(token: &str) -> Result<Vec<ChangeId>, IdError> {
    change_ids(token).ok_or_else(|| IdError::new(token, CHANGE_EXPECTED))
}

fn change_ids(token: &str) -> Option<Vec<ChangeId>> {
    if token.contains(":rev:") {
        return ChangeId::from_long(token).map(|id| vec![id]);
    }
    let body = |wids: Vec<String>| wids.into_iter().map(|w| ChangeId::new("body", w)).collect();
    if let Some(rest) = token.strip_prefix('#') {
        return plus_list(rest).map(body);
    }
    if let Some(wids) = plus_list(token) {
        return Some(body(wids));
    }
    if let Some(wids) = token.strip_prefix('r').and_then(plus_list) {
        return Some(body(wids));
    }
    let (story, rest) = token.split_once('.')?;
    let wids = plus_list(rest.strip_prefix('r')?)?;
    is_story(story).then(|| wids.into_iter().map(|w| ChangeId::new(story, w)).collect())
}

/// The comment a comment id names: `c27`, `27`, or 0.12's `#c27`.
pub fn parse_comment_id(token: &str) -> Result<u32, IdError> {
    let digits = token
        .strip_prefix("#c")
        .or_else(|| token.strip_prefix('c'))
        .unwrap_or(token);
    number(digits).ok_or_else(|| IdError::new(token, COMMENT_EXPECTED))
}

/// A serde `deserialize_with` for a comment id: a number, or a string
/// [`parse_comment_id`] reads.
pub fn comment_id_de<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Number(u32),
        Text(String),
    }
    match Raw::deserialize(deserializer)? {
        Raw::Number(n) => Ok(n),
        Raw::Text(text) => parse_comment_id(&text).map_err(serde::de::Error::custom),
    }
}

/// A cell's paragraph: `t0.row1.cell2[.p1]`, or 0.12's `t0.r1.c2[.p1]`, as
/// `(table, row, cell, paragraph)`. The two forms do not mix.
pub fn cell_path(token: &str) -> Option<(usize, usize, usize, Option<usize>)> {
    let parts: Vec<&str> = token.split('.').collect();
    let (table, row, cell, paragraph) = match parts.as_slice() {
        [t, r, c] => (*t, *r, *c, None),
        [t, r, c, p] => (*t, *r, *c, Some(*p)),
        _ => return None,
    };
    let table = short_number(table, 't')?;
    let (row, cell) = match (word_number(row, "row"), word_number(cell, "cell")) {
        (Some(row), Some(cell)) => (row, cell),
        _ => (short_number(row, 'r')?, short_number(cell, 'c')?),
    };
    let paragraph = match paragraph {
        Some(p) => Some(short_number(p, 'p')?),
        None => None,
    };
    Some((table, row, cell, paragraph))
}

/// Whether `token` is a cell in 0.12's retired form (`t0.r1.c2`).
pub fn is_retired_cell(token: &str) -> bool {
    cell_path(token).is_some()
        && token
            .split('.')
            .nth(1)
            .is_some_and(|row| short_number(row, 'r').is_some())
}

/// What a `-p` value names (see the module doc). The first matching form
/// wins: a comment, a change, a bare number (a body paragraph), a body
/// paragraph, a table cell, a story paragraph, a long paragraph id.
pub fn parse_location(token: &str) -> Result<Location, IdError> {
    let refuse = |why| Err(IdError::new(token, why));
    if digit_shaped(token, "c") || digit_shaped(token, "#c") {
        return match parse_comment_id(token) {
            Ok(id) => Ok(Location::Comment(id)),
            Err(_) => refuse(LOCATION_EXPECTED),
        };
    }
    let change_shaped = token.starts_with('#')
        || token.contains(":rev:")
        || token
            .strip_prefix('r')
            .is_some_and(|rest| plus_list(rest).is_some())
        || token.split_once('.').is_some_and(|(story, rest)| {
            is_story(story)
                && rest
                    .strip_prefix('r')
                    .is_some_and(|r| plus_list(r).is_some())
        });
    if change_shaped {
        return match change_ids(token) {
            Some(ids) => Ok(Location::Changes(ids)),
            None => refuse(LOCATION_EXPECTED),
        };
    }
    if let Some(n) = number::<usize>(token) {
        return Ok(Location::Paragraph(format!("p{n}")));
    }
    if short_number(token, 'p').is_some() {
        return Ok(Location::Paragraph(token.to_string()));
    }
    let head = token.split('.').next().unwrap_or(token);
    if short_number(head, 't').is_some() {
        if cell_path(token).is_some() {
            return Ok(Location::Paragraph(token.to_string()));
        }
        // Only a table, or a table and a row, lacks its cell.
        let missing_cell = match token.split('.').collect::<Vec<_>>().as_slice() {
            [_] => true,
            [_, row] => word_number(row, "row").is_some() || short_number(row, 'r').is_some(),
            _ => false,
        };
        return refuse(if missing_cell {
            TABLE_NEEDS_CELL
        } else {
            LOCATION_EXPECTED
        });
    }
    let rest_follows = token.len() > head.len();
    if word_number(head, "row").is_some()
        || word_number(head, "cell").is_some()
        || (rest_follows
            && (short_number(head, 'r').is_some() || short_number(head, 'c').is_some()))
    {
        return refuse(CELL_NEEDS_TABLE);
    }
    if is_story(head) {
        let rest: Vec<&str> = token.split('.').skip(1).collect();
        return match rest.as_slice() {
            [] => Ok(Location::Paragraph(token.to_string())),
            [p] if short_number(p, 'p').is_some() => Ok(Location::Paragraph(token.to_string())),
            _ => refuse(LOCATION_EXPECTED),
        };
    }
    if let Some((story, n)) = token.split_once(":p:")
        && !story.is_empty()
        && !story.contains(':')
        && number::<usize>(n).is_some()
    {
        return Ok(Location::Paragraph(token.to_string()));
    }
    refuse(LOCATION_EXPECTED)
}

/// The short form of a long paragraph id: `body:p:12` → `p12`,
/// `header1:p:0` → `header1`, `header1:p:2` → `header1.p2`; the long id
/// itself when its story or number has no short spelling.
pub fn short_paragraph(long: &str) -> String {
    let Some((story, n)) = long.split_once(":p:") else {
        return long.to_string();
    };
    match number::<usize>(n) {
        Some(n) if story == "body" => format!("p{n}"),
        Some(0) if is_story(story) => story.to_string(),
        Some(n) if is_story(story) => format!("{story}.p{n}"),
        _ => long.to_string(),
    }
}

/// The body paragraph `p12` or `12` names.
pub fn paragraph_number(token: &str) -> Option<usize> {
    short_number(token, 'p').or_else(|| number(token))
}

/// The table `t0` names.
pub fn table_number(token: &str) -> Option<usize> {
    short_number(token, 't')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(wid: &str) -> ChangeId {
        ChangeId::new("body", wid)
    }

    #[test]
    fn every_change_spelling_names_the_same_change() {
        for token in ["r12", "#12", "12", "body:rev:12", "body.r12"] {
            assert_eq!(parse_change_ids(token), Ok(vec![body("12")]), "{token}");
        }
        for token in ["header1.r1", "header1:rev:1"] {
            assert_eq!(
                parse_change_ids(token),
                Ok(vec![ChangeId::new("header1", "1")]),
                "{token}"
            );
        }
        for token in ["r1+2", "#1+2", "1+2"] {
            assert_eq!(
                parse_change_ids(token),
                Ok(vec![body("1"), body("2")]),
                "{token}"
            );
        }
        let pair = parse_change_ids("header1.r1+2").unwrap();
        assert_eq!(pair[1].long(), "header1:rev:2");
        assert_eq!(
            parse_change_ids("bogus.r1"),
            Ok(vec![ChangeId::new("bogus", "1")])
        );
        assert_eq!(
            parse_change_ids("Header_1:rev:x9"),
            Ok(vec![ChangeId::new("Header_1", "x9")])
        );
    }

    #[test]
    fn short_change_ids_print_without_a_hash() {
        assert_eq!(body("12").short(), "r12");
        assert_eq!(ChangeId::new("footnotes", "3").short(), "footnotes.r3");
        assert_eq!(ChangeId::new("styles", "0").short(), "styles.r0");
        assert_eq!(ChangeId::new("Header_1", "4").short(), "Header_1:rev:4");
        assert_eq!(ChangeId::new("r1", "4").short(), "r1:rev:4");
        assert_eq!(body("007").short(), "body:rev:007");
        // A w:id the short form cannot spell keeps its long id, and that id
        // reads back: it names the document's own `w:id`, not the number.
        for wid in ["007", "4294967296", "x9", "-5"] {
            let printed = body(wid).short();
            assert_eq!(parse_change_ids(&printed), Ok(vec![body(wid)]), "{printed}");
        }
        assert_ne!(body("007"), body("7"));
        for long in ["body:rev: 7", "body:rev:12+13", "body:rev:1:2"] {
            assert_eq!(ChangeId::from_long(long), None, "{long}");
            assert!(parse_change_ids(long).is_err(), "{long}");
        }
        assert_eq!(
            ChangeId::from_long("body:rev:x9"),
            Some(ChangeId::new("body", "x9"))
        );
        assert_eq!(body("x").short(), "body:rev:x");
        assert_eq!(
            ChangeId::from_long("header1:rev:2"),
            Some(ChangeId::new("header1", "2"))
        );
        assert_eq!(ChangeId::from_long("header1:p:2"), None);
        assert_eq!(ChangeId::from_long(":rev:2"), None);
    }

    #[test]
    fn malformed_change_ids_are_refused_with_one_message() {
        for token in [
            "p12",
            "#",
            "#x",
            "r",
            "rx",
            "header1.r",
            "r1+",
            "+2",
            "r1++2",
            "c5",
            "header1",
            "row1",
            "header1#1",
            "#c5",
            "R1",
            "r01",
            "r 1",
            "r-1",
            "r+1",
            "r٣",
            "r4294967296",
            "",
            "header1.p2",
            "Header_1.r1",
            "body:rev:",
            "a:b:rev:1",
        ] {
            let err = parse_change_ids(token).unwrap_err();
            assert_eq!(
                err.to_string(),
                format!("{token}: expected a change id such as r12, header1.r1 or body:rev:12")
            );
        }
        assert_eq!(
            parse_change_ids("r4294967295"),
            Ok(vec![body("4294967295")])
        );
    }

    #[test]
    fn comment_ids_take_three_spellings() {
        for token in ["c27", "#c27", "27"] {
            assert_eq!(parse_comment_id(token), Ok(27), "{token}");
        }
        assert_eq!(parse_comment_id("c4294967295"), Ok(u32::MAX));
        for token in [
            "p27",
            "r27",
            "c",
            "c027",
            "c4294967296",
            "#27",
            "C5",
            "c 5",
            "c5.p1",
        ] {
            assert_eq!(
                parse_comment_id(token).unwrap_err().to_string(),
                format!("{token}: expected a comment id such as c27 or 27")
            );
        }
    }

    #[test]
    fn comment_ids_deserialize_from_numbers_and_strings() {
        #[derive(Deserialize)]
        struct Op {
            #[serde(deserialize_with = "comment_id_de")]
            comment_id: u32,
        }
        for json in [
            r#"{"comment_id":5}"#,
            r#"{"comment_id":"c5"}"#,
            r##"{"comment_id":"#c5"}"##,
            r#"{"comment_id":"5"}"#,
        ] {
            let op: Op = serde_json::from_str(json).unwrap();
            assert_eq!(op.comment_id, 5, "{json}");
        }
        let err = serde_json::from_str::<Op>(r#"{"comment_id":"p5"}"#)
            .err()
            .unwrap()
            .to_string();
        assert!(
            err.contains("p5: expected a comment id such as c27 or 27"),
            "{err}"
        );
    }

    #[test]
    fn locations_follow_the_table() {
        use Location::*;
        let ok = |token: &str| parse_location(token).unwrap();
        assert_eq!(ok("c5"), Comment(5));
        assert_eq!(ok("#c5"), Comment(5));
        assert_eq!(ok("r12"), Changes(vec![body("12")]));
        assert_eq!(ok("#12"), Changes(vec![body("12")]));
        assert_eq!(ok("r1+2"), Changes(vec![body("1"), body("2")]));
        assert_eq!(
            ok("header1.r1"),
            Changes(vec![ChangeId::new("header1", "1")])
        );
        assert_eq!(ok("body:rev:3"), Changes(vec![body("3")]));
        assert_eq!(ok("27"), Paragraph("p27".into()));
        assert_eq!(ok("0"), Paragraph("p0".into()));
        for token in [
            "p51000",
            "t0.row1.cell2",
            "t0.row1.cell2.p1",
            "t0.r1.c2",
            "t0.r1.c2.p1",
            "header1",
            "header1.p2",
            "footnotes.p3",
            "endnotes.p0",
            "body:p:3",
            "header",
            "rows",
        ] {
            assert_eq!(ok(token), Paragraph(token.into()), "{token}");
        }
    }

    #[test]
    fn refused_locations_say_why() {
        let why = |token: &str| parse_location(token).unwrap_err().to_string();
        for token in ["t0", "t0.row1", "t0.r1", "t12"] {
            assert_eq!(
                why(token),
                format!("{token}: a table id needs a row and a cell, as t0.row1.cell2")
            );
        }
        for token in ["row1", "row1.cell2", "cell2", "cell2.p1", "r1.c2", "c2.p1"] {
            assert_eq!(
                why(token),
                format!("{token}: a row or cell needs its table, as t0.row1.cell2"),
            );
        }
        for token in [
            "1+2",
            "p5-p7",
            "p5-",
            "-p7",
            "5-7",
            "#x",
            "#c",
            "Header_1",
            "c05",
            "#c05",
            "c4294967296",
            "t0.p1",
            "t0.row1.c2",
            "t0.r1.cell2",
            "t0.row01.cell2",
            "t0.row1.cell2.p1.p2",
        ] {
            assert_eq!(
                why(token),
                format!(
                    "{token}: -p takes a paragraph (p12 or 12), a story paragraph (header1, header1.p2, footnotes.p3), a cell (t0.row1.cell2), a comment (c5) or a revision (r12)"
                ),
            );
        }
    }

    #[test]
    fn the_malformed_ids_stay_refused() {
        for token in [
            "p",
            "p-1",
            "p+1",
            "p１",
            "p1.p0",
            "p1.",
            "P1",
            "p18446744073709551616",
            "header1.p-1",
            "header1.p0.extra",
            "t0.r0",
            "t0.r-1.c0",
            "t0.r0.c+1",
            "t0.r0.c0.p",
            "t0.r0.c0.p0.extra",
            "t0.r0.c18446744073709551616",
            "t0.row0",
            "t0.row-1.cell0",
            "t0.row0.cell+1",
            "t0.row0.cell0.p",
            "t0.row0.cell0.p0.extra",
            "t0.row0.cell18446744073709551616",
        ] {
            assert!(parse_location(token).is_err(), "{token}");
        }
    }

    #[test]
    fn each_number_has_one_spelling() {
        for token in [
            "p012",
            "012",
            "t01.row1.cell2",
            "t0.row01.cell2",
            "t0.row1.cell02",
            "t0.row1.cell2.p01",
            "c05",
            "r007",
            "header1.p02",
            "body:p:012",
        ] {
            assert!(parse_location(token).is_err(), "{token}");
        }
        assert_eq!(paragraph_number("p012"), None);
        assert_eq!(paragraph_number("p0"), Some(0));
    }

    #[test]
    fn cell_paths_read_both_forms_and_never_mix_them() {
        let want = Some((0, 1, 2, Some(1)));
        assert_eq!(cell_path("t0.row1.cell2.p1"), want);
        assert_eq!(cell_path("t0.r1.c2.p1"), want);
        assert_eq!(cell_path("t11.row119.cell11"), Some((11, 119, 11, None)));
        assert_eq!(cell_path("t0.row1.c2"), None);
        assert_eq!(cell_path("t0.r1.cell2"), None);
        assert!(is_retired_cell("t0.r1.c2"));
        assert!(is_retired_cell("t0.r1.c2.p1"));
        assert!(!is_retired_cell("t0.row1.cell2"));
        assert!(!is_retired_cell("t0.r1"));
    }

    #[test]
    fn short_paragraph_ids() {
        for (long, short) in [
            ("body:p:12", "p12"),
            ("body:p:0", "p0"),
            ("header1:p:0", "header1"),
            ("header1:p:2", "header1.p2"),
            ("footnotes:p:3", "footnotes.p3"),
            ("Header_1:p:2", "Header_1:p:2"),
            ("body:p:012", "body:p:012"),
            ("p12", "p12"),
        ] {
            assert_eq!(short_paragraph(long), short, "{long}");
        }
        assert_eq!(paragraph_number("p12"), Some(12));
        assert_eq!(paragraph_number("12"), Some(12));
        assert_eq!(paragraph_number("p"), None);
        assert_eq!(table_number("t0"), Some(0));
        assert_eq!(table_number("t"), None);
    }

    #[test]
    fn story_stems() {
        for stem in [
            "header1",
            "footnotes",
            "styles",
            "hdr",
            "body",
            "row",
            "cell",
            "rows",
        ] {
            assert!(is_story(stem), "{stem}");
        }
        for stem in [
            "", "Header1", "header_1", "1header", "p1", "t0", "c5", "r1", "row1", "cell2", "p01",
            "p", "t", "c", "r",
        ] {
            assert!(!is_story(stem), "{stem}");
        }
    }

    /// Rule 6: what jubarte prints or advertises is safe unquoted in bash,
    /// csh and zsh.
    #[test]
    fn printed_ids_are_shell_safe() {
        let safe = |id: &str| {
            !id.is_empty()
                && id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b".+:-".contains(&b))
                && !id.starts_with(['#', '=', '~', '-'])
        };
        let mut printed: Vec<String> = [
            CHANGE_SPELLINGS,
            COMMENT_SPELLINGS,
            PARAGRAPH_SPELLINGS,
            TABLE_SPELLINGS,
        ]
        .concat()
        .iter()
        .map(|s| s.to_string())
        .collect();
        for story in ["body", "header1", "footnotes", "styles", "hdr"] {
            for wid in ["0", "9", "51000", "4294967295"] {
                printed.push(ChangeId::new(story, wid).short());
                printed.push(short_paragraph(&format!("{story}:p:{wid}")));
            }
        }
        for id in &printed {
            assert!(safe(id), "{id}");
        }
    }

    /// A seeded generator draws tokens from the grammar's alphabet: no
    /// parser panics, and whatever parses prints back to the same value.
    #[test]
    fn generated_tokens_round_trip() {
        let pieces = [
            "p",
            "r",
            "c",
            "t",
            "#",
            ".",
            "+",
            ":",
            "-",
            "0",
            "1",
            "7",
            "12",
            "51000",
            "body",
            "header1",
            "footnotes",
            "row",
            "cell",
            ":rev:",
            ":p:",
            "#c",
            "01",
            "4294967296",
        ];
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for _ in 0..200_000 {
            let len = 1 + next() % 5;
            let token: String = (0..len)
                .map(|_| pieces[(next() % pieces.len() as u64) as usize])
                .collect();
            if let Ok(ids) = parse_change_ids(&token) {
                for id in &ids {
                    assert_eq!(
                        parse_change_ids(&id.long()),
                        Ok(vec![id.clone()]),
                        "{token}"
                    );
                    assert_eq!(
                        parse_change_ids(&id.short()),
                        Ok(vec![id.clone()]),
                        "{token}"
                    );
                }
            }
            if let Ok(n) = parse_comment_id(&token) {
                assert_eq!(parse_comment_id(&format!("c{n}")), Ok(n), "{token}");
            }
            match parse_location(&token) {
                Ok(Location::Comment(n)) => assert_eq!(parse_comment_id(&token), Ok(n)),
                Ok(Location::Changes(ids)) => assert_eq!(parse_change_ids(&token), Ok(ids)),
                Ok(Location::Paragraph(p)) => {
                    assert!(
                        parse_change_ids(&token).is_err()
                            || token.bytes().all(|b| b.is_ascii_digit()),
                        "{token}"
                    );
                    assert_eq!(
                        parse_location(&p),
                        Ok(Location::Paragraph(p.clone())),
                        "{token}"
                    );
                }
                Err(_) => {}
            }
        }
    }
}
