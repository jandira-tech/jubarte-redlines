// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The one grammar of the ids jubarte prints and reads: [`ShortId`]
//! (`src/ids/short.rs`, spec 2026-10-13 §2 and §2.1). The functions here are
//! the options' views of it: a change (`r12`, `r1+2`, `header1.r1`), a
//! comment (`c27`), the text a comment covers (`c27.text`), and what `-p`
//! takes (a paragraph `p12`, a story `header1` or its paragraph
//! `header1.p2`, a cell `t0.row1.cell2[.p1]`, a comment, or changes).
//!
//! Arthur, 2026-10-13: "We should strictly have the settled short IDs or long
//! IDs. Everything else should be expired." Bare numbers (`12`), 0.12's
//! `#12`, `#c27` and `t0.r1.c2` are refused, naming the settled spelling.
//! Arthur, 2026-10-15: only those two types, and an error otherwise. The
//! colon long ids (`body:rev:12`, `body:p:12`) are refused, naming the short
//! id, whenever one exists. Until the contract's long ids land — the
//! four-field `PART.ELEMENT.ATTRIBUTE.VALUE` family of
//! `docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md` §3 —
//! jubarte prints a colon id only for what has no short spelling (a part
//! outside the settled stories, `Header_1.xml`; a `w:id` such as `007`),
//! and exactly that id still parses: jubarte never prints an id it refuses.
//! Those colon ids are transitional: the contract retires the whole family
//! at the schema-2 cutover.
//!
//! This module reads syntax only. Whether a story, paragraph, change or
//! comment exists is decided against a document by `edit` (`-p`) and
//! [`crate::changes`] (`--id`). Every number has one spelling: ASCII digits
//! without leading zeros. A colon long change id carries the `w:id` as the
//! document spells it, so a change whose `w:id` the short form cannot spell
//! (`007`) is still named. Every short id is safe unquoted in bash, csh and
//! zsh.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};

mod short;
pub use short::{SHORT_EXPECTED, ShortId, Step, Story};

/// Why a change id was refused.
pub const CHANGE_EXPECTED: &str = "expected a change id such as r12, r1+2 or header1.r1";
/// Why a comment id was refused.
pub const COMMENT_EXPECTED: &str = "expected a comment id such as c27";
/// Why a `-p` value was refused, when no narrower reason applies.
pub const LOCATION_EXPECTED: &str = "-p takes a paragraph (p12), a story or its paragraph (header1, header1.p2, footnotes.p3), a cell (t0.row1.cell2), a comment (c5), the text a comment covers (c5.text) or a revision (r12)";
/// Why a settled id `-p` cannot edit yet was refused.
pub const LOCATION_NOT_YET: &str = "a settled id, but -p does not edit tables, rows, nested tables, text boxes, content controls, ranges, sections or tables of contents yet";
/// Why a table id without its row and cell was refused.
pub const TABLE_NEEDS_CELL: &str = "a table id needs a row and a cell, as t0.row1.cell2";
/// Message returned for a `-p` value that names a row or cell without its
/// table.
pub const CELL_NEEDS_TABLE: &str = "a row or cell needs its table, as t0.row1.cell2";

/// The change spellings the manifest advertises: the settled ones only
/// (module doc).
pub const CHANGE_SPELLINGS: &[&str] = &["r12", "r1+2", "header1.r1", "footnotes.r3"];
/// The comment spellings the manifest advertises.
pub const COMMENT_SPELLINGS: &[&str] = &["c27"];
/// The spellings of the text a comment covers the manifest advertises.
pub const COMMENT_TEXT_SPELLINGS: &[&str] = &["c27.text"];
/// The paragraph spellings the manifest advertises. `t0.row1.cell2.p1` is
/// the pre-contract nested spelling; the 17 October 2026 contract spells a
/// cell's own paragraphs `t0.row1.cell2.ps1`.
pub const PARAGRAPH_SPELLINGS: &[&str] = &[
    "p12",
    "header1",
    "header1.p2",
    "footnotes.p3",
    "t0.row1.cell2",
    "t0.row1.cell2.p1",
];

/// Sample table-id spellings the capabilities manifest advertises
/// ([`crate::capabilities::Ids::table`]); rows and cells sit under the
/// table, as in [`PARAGRAPH_SPELLINGS`].
pub const TABLE_SPELLINGS: &[&str] = &["t0"];

/// The content control spellings the manifest advertises: a block control,
/// one in a paragraph, and one in a cell's first paragraph.
pub const CONTROL_SPELLINGS: &[&str] = &["cc0", "p12.cc0", "t0.row1.cell2.cc0"];

/// A refused id: what was given and what was expected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdError {
    /// The token as it was given to the parser.
    pub given: String,
    /// Human-readable hint naming the form the token should have taken
    /// (one of the [`CHANGE_EXPECTED`] / [`COMMENT_EXPECTED`] /
    /// [`LOCATION_EXPECTED`] / [`TABLE_NEEDS_CELL`] / [`CELL_NEEDS_TABLE`]
    /// constants).
    pub expected: &'static str,
    /// The settled spelling of an expired one (`12` → `p12, r12 or c12`),
    /// when one is clear.
    pub hint: Option<String>,
}

impl IdError {
    /// Builds an [`IdError`] with the given token and the matching
    /// `expected` message constant.
    fn new(given: &str, expected: &'static str) -> Self {
        Self {
            given: given.to_string(),
            expected,
            hint: None,
        }
    }
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.given, self.expected)?;
        match &self.hint {
            Some(hint) => write!(f, "; the settled spelling is {hint}"),
            None => Ok(()),
        }
    }
}

impl std::error::Error for IdError {}

/// One tracked change, addressed by story and the change's `w:id` value.
/// The grammar of `short` and of the transitional colon `long` is safe
/// unquoted in bash, csh and zsh and never collides with another change when
/// its `w:id` is canonical.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChangeId {
    /// The owning story: `body` for body changes, otherwise the part's file
    /// stem (`header1`, `footnotes`, `endnotes`, …).
    pub story: String,
    /// The change's `w:id`, as the document spells it. A canonical
    /// `u32` (`12`) prints and parses back unchanged; values that do not fit
    /// the short form (`007`, `4294967296`, `x9`) travel as strings so the
    /// short/long round-trip stays exact.
    pub wid: String,
}

impl ChangeId {
    /// Builds a [`ChangeId`] from its story and `w:id`. Both strings are
    /// stored verbatim; no canonicalisation is performed here.
    pub fn new(story: impl Into<String>, wid: impl Into<String>) -> Self {
        Self {
            story: story.into(),
            wid: wid.into(),
        }
    }

    /// The id jubarte prints for the change, everywhere: `r12` in the
    /// body, `header1.r1` elsewhere; the transitional colon id
    /// `{story}:rev:{wid}` only when the story or the `w:id` has no short
    /// spelling ([`ShortId`] decides: what is printed parses). The 17
    /// October 2026 contract allocates a label instead (§4); that is staged,
    /// not landed.
    pub fn short(&self) -> String {
        let short = if self.story == "body" {
            format!("r{}", self.wid)
        } else {
            format!("{}.r{}", self.story, self.wid)
        };
        // Only a short id that reads back as this one change: `w:id="1+2"`
        // would print `r1+2`, which names two.
        match short.parse::<ShortId>() {
            Ok(ShortId::Change { wids, .. })
                if wids.len() == 1 && wids[0].to_string() == self.wid =>
            {
                short
            }
            _ => format!("{}:rev:{}", self.story, self.wid),
        }
    }

    /// The change a transitional colon long id names; `None` when `long` is
    /// not one. Everything after the story's `:rev:` is the `w:id` as the
    /// document spells it (`body:rev:007` names a `w:id="007"`, not `r7`;
    /// `body:rev:1+2` one `w:id="1+2"`), so every id [`ChangeId::short`]
    /// prints reads back. The short forms keep one spelling per number.
    pub fn from_long(long: &str) -> Option<Self> {
        let (story, wid) = long.split_once(":rev:")?;
        (!story.is_empty() && !story.contains(':')).then(|| Self::new(story, wid))
    }
}

/// The one change a printed change id names (`r12`, `header1.r1`, or the
/// transitional colon id of a change with no short spelling); `None` for any
/// other token, a pair `r1+2` among them.
pub fn change_id(id: &str) -> Option<ChangeId> {
    match parse_change_ids(id) {
        Ok(ids) if ids.len() == 1 => ids.into_iter().next(),
        _ => None,
    }
}

/// What a `-p` value names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Location {
    /// A paragraph, in a form `edit` resolves (`p12`, `header1.p2`,
    /// `t0.row1.cell2`; the transitional colon id of a part with no short
    /// spelling).
    Paragraph(String),
    /// A comment, addressed by its `w:id`.
    Comment(u32),
    /// The text a comment covers (`c5.text`), by the comment's `w:id`.
    CommentText(u32),
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

/// The number after `prefix` in a part of a short id (`p12`, `t0`): ASCII
/// digits without leading zeros.
pub fn short_number(text: &str, prefix: char) -> Option<usize> {
    number(text.strip_prefix(prefix)?)
}

/// The number after a word prefix (`row1`, `cell2`).
fn word_number(text: &str, prefix: &str) -> Option<usize> {
    number(text.strip_prefix(prefix)?)
}

/// The changes a change id names: `r12`, `r1+2`, `header1.r1`, and the
/// transitional colon id of a change with no short spelling (module doc).
/// A story is checked only syntactically.
pub fn parse_change_ids(token: &str) -> Result<Vec<ChangeId>, IdError> {
    if token.contains(":rev:") {
        return match ChangeId::from_long(token) {
            // Only the id jubarte prints for a change without a short
            // spelling; any other colon long id names its short one (module
            // doc).
            Some(id) if id.short() == token => Ok(vec![id]),
            Some(id) => Err(IdError {
                hint: Some(id.short()),
                ..IdError::new(token, CHANGE_EXPECTED)
            }),
            None => Err(IdError::new(token, CHANGE_EXPECTED)),
        };
    }
    match token.parse::<ShortId>() {
        Ok(ShortId::Change { story, wids }) => Ok(wids
            .into_iter()
            .map(|wid| ChangeId::new(story.to_string(), wid.to_string()))
            .collect()),
        Ok(_) => Err(IdError::new(token, CHANGE_EXPECTED)),
        Err(e) => Err(IdError {
            hint: settled_as(token, 'r')
                .or_else(|| hint_where(e.hint, |id| matches!(id, ShortId::Change { .. }))),
            ..IdError::new(token, CHANGE_EXPECTED)
        }),
    }
}

/// The comment a comment id names: `c27`.
pub fn parse_comment_id(token: &str) -> Result<u32, IdError> {
    match token.parse::<ShortId>() {
        Ok(ShortId::Comment(n)) => Ok(n),
        Ok(_) => Err(IdError::new(token, COMMENT_EXPECTED)),
        Err(e) => Err(IdError {
            hint: settled_as(token, 'c')
                .or_else(|| hint_where(e.hint, |id| matches!(id, ShortId::Comment(_)))),
            ..IdError::new(token, COMMENT_EXPECTED)
        }),
    }
}

/// The spellings of `hint` (`p12, r12 or c12`) that name what an option
/// takes: a refusal never suggests a spelling the same option refuses
/// (`--id body:p:12` suggests nothing, `read -p 12` only `p12`).
pub fn hint_where(hint: Option<String>, keep: impl Fn(&ShortId) -> bool) -> Option<String> {
    let hint = hint?;
    let kept: Vec<&str> = hint
        .split(" or ")
        .flat_map(|part| part.split(", "))
        .filter(|s| s.parse::<ShortId>().is_ok_and(|id| keep(&id)))
        .collect();
    match kept.as_slice() {
        [] => None,
        [one] => Some((*one).to_string()),
        [rest @ .., last] => Some(format!("{} or {last}", rest.join(", "))),
    }
}

/// The settled spelling of bare numbers where the option says what they
/// name (spec §2, "by the option"): `--id 12` → `r12`, `--id 1+2` → `r1+2`,
/// a comment's `27` → `c27`; only when that spelling parses.
fn settled_as(token: &str, prefix: char) -> Option<String> {
    let bare = !token.is_empty() && token.bytes().all(|b| b.is_ascii_digit() || b == b'+');
    let id = format!("{prefix}{token}");
    (bare && id.parse::<ShortId>().is_ok()).then_some(id)
}

/// `c5.text` → 5: the comment whose covered text a `-p` or a plan's
/// `paragraph` names.
pub fn comment_text_id(token: &str) -> Option<u32> {
    match token.parse::<ShortId>() {
        Ok(ShortId::CommentText(n)) => Some(n),
        _ => None,
    }
}

/// A serde `deserialize_with` for a comment id: a JSON number (a typed
/// value, not a spelling), or a string [`parse_comment_id`] reads (`"c27"`).
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

/// What a `-p` value names (see the module doc).
pub fn parse_location(token: &str) -> Result<Location, IdError> {
    let refuse = |why| Err(IdError::new(token, why));
    // A colon long id parses only when it is what jubarte prints: an id
    // with no short spelling (module doc).
    if token.contains(":rev:") {
        return parse_change_ids(token).map(Location::Changes);
    }
    if let Some((story, n)) = token.split_once(":p:")
        && !story.is_empty()
        && !story.contains(':')
        && number::<usize>(n).is_some()
    {
        let short = short_paragraph(token);
        return if short == token {
            Ok(Location::Paragraph(token.to_string()))
        } else {
            Err(IdError {
                hint: Some(short),
                ..IdError::new(token, LOCATION_EXPECTED)
            })
        };
    }
    let head = token.split('.').next().unwrap_or(token);
    if word_number(head, "row").is_some() || word_number(head, "cell").is_some() {
        return refuse(CELL_NEEDS_TABLE);
    }
    let id = match token.parse::<ShortId>() {
        Ok(id) => id,
        Err(e) => {
            return Err(IdError {
                hint: e.hint,
                ..IdError::new(token, LOCATION_EXPECTED)
            });
        }
    };
    match id {
        ShortId::Comment(n) => Ok(Location::Comment(n)),
        ShortId::CommentText(n) => Ok(Location::CommentText(n)),
        ShortId::Change { .. } => parse_change_ids(token).map(Location::Changes),
        ShortId::Block { story, path } => match (&story, path.as_slice()) {
            (Story::Body, [Step::Paragraph(_)]) | (Story::Part(_), [] | [Step::Paragraph(_)]) => {
                Ok(Location::Paragraph(token.to_string()))
            }
            (Story::Body, [Step::Table(_), Step::Row(_), Step::Cell(_)])
            | (
                Story::Body,
                [
                    Step::Table(_),
                    Step::Row(_),
                    Step::Cell(_),
                    Step::Paragraph(_),
                ],
            ) => Ok(Location::Paragraph(token.to_string())),
            (Story::Body, [Step::Table(_)] | [Step::Table(_), Step::Row(_)]) => {
                refuse(TABLE_NEEDS_CELL)
            }
            _ => refuse(LOCATION_NOT_YET),
        },
        _ => refuse(LOCATION_NOT_YET),
    }
}

/// The story id of a part: its file stem (`word/header1.xml` → `header1`),
/// each byte outside `A-Z`, `a-z`, `0-9`, `.`, `_` and `-` spelled `%XX`,
/// and a leading `-` too, so every id built on it is safe unquoted in bash,
/// csh and zsh: `word/hdr!old.xml` → `hdr%21old`.
pub fn part_story(part: &str) -> String {
    let file = part.rsplit('/').next().unwrap_or(part);
    let stem = file.strip_suffix(".xml").unwrap_or(file);
    let mut out = String::with_capacity(stem.len());
    for (i, b) in stem.bytes().enumerate() {
        if b.is_ascii_alphanumeric() || b"._".contains(&b) || (b == b'-' && i > 0) {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The id jubarte prints for paragraph `index` of `story` (`body`,
/// `header1`, `footnotes`, …): `p12`, `header1` for a story's first
/// paragraph, `header1.p2`; the transitional colon id `{story}:p:{index}`
/// only when the story has no short spelling ([`short_paragraph`]).
pub fn paragraph_id(story: &str, index: usize) -> String {
    short_paragraph(&format!("{story}:p:{index}"))
}

/// The story and index of a paragraph id [`paragraph_id`] prints
/// (`p12` → `("body", 12)`, `header1` → `("header1", 0)`); `None` for any
/// other token.
pub fn paragraph_place(id: &str) -> Option<(String, usize)> {
    let (story, n) = if let Some((story, n)) = id.split_once(":p:") {
        (story, number::<usize>(n)?)
    } else if let Some(n) = id.strip_prefix('p').and_then(number::<usize>) {
        ("body", n)
    } else if let Some((story, p)) = id.split_once('.') {
        (story, p.strip_prefix('p').and_then(number::<usize>)?)
    } else {
        (id, 0)
    };
    (paragraph_id(story, n) == id).then(|| (story.to_string(), n))
}

/// The short form of a transitional colon paragraph id (`body:p:N` → `pN`,
/// `header1:p:0` → `header1`, `header1:p:N` → `header1.pN`); the input is
/// returned verbatim when it does not have one `:p:` separator, when the
/// story does not fit the short grammar, or when the digit run cannot be
/// canonicalised.
pub fn short_paragraph(long: &str) -> String {
    let Some((story, n)) = long.split_once(":p:") else {
        return long.to_string();
    };
    let short = match number::<usize>(n) {
        Some(n) if story == "body" => format!("p{n}"),
        Some(0) => story.to_string(),
        Some(n) => format!("{story}.p{n}"),
        None => return long.to_string(),
    };
    // Only a story (`header1`) or its paragraph (`p12`, `header1.p2`):
    // `cc0:p:0` must not shorten to the content control `cc0`.
    match short.parse::<ShortId>() {
        Ok(ShortId::Block { path, .. }) if matches!(path.as_slice(), [] | [Step::Paragraph(_)]) => {
            short
        }
        _ => long.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_story_or_its_paragraph_is_a_paragraph_place() {
        // pi review of #487: `cc0` read as story "cc0", paragraph 0.
        // A cell's paragraph prints as its `pN`, so `t0.row1.cell2` is no
        // paragraph place either.
        for id in [
            "cc0",
            "t0",
            "p3.cc0",
            "t0.row1",
            "t0.row1.cell2",
            "p14.box0",
        ] {
            assert_eq!(paragraph_place(id), None, "{id}");
        }
        assert_eq!(short_paragraph("cc0:p:0"), "cc0:p:0");
        for (id, story, n) in [
            ("p12", "body", 12),
            ("header1", "header1", 0),
            ("header1.p2", "header1", 2),
        ] {
            assert_eq!(paragraph_place(id), Some((story.to_string(), n)), "{id}");
        }
    }

    #[test]
    fn hints_keep_only_what_the_option_takes() {
        let all = || Some("p12, r12 or c12".to_string());
        let change = |id: &ShortId| matches!(id, ShortId::Change { .. });
        assert_eq!(hint_where(all(), change).as_deref(), Some("r12"));
        assert_eq!(
            hint_where(all(), |_| true).as_deref(),
            Some("p12, r12 or c12")
        );
        assert_eq!(
            hint_where(all(), |id| !matches!(id, ShortId::Comment(_))).as_deref(),
            Some("p12 or r12")
        );
        assert_eq!(hint_where(Some("p12".into()), change), None);
        assert_eq!(hint_where(None, |_| true), None);
        // An option's refusal never suggests what the same option refuses.
        assert_eq!(parse_change_ids("body:p:12").unwrap_err().hint, None);
        assert_eq!(parse_comment_id("1+2").unwrap_err().hint, None);
        assert_eq!(
            parse_comment_id("#c5").unwrap_err().hint.as_deref(),
            Some("c5")
        );
    }

    fn body(wid: &str) -> ChangeId {
        ChangeId::new("body", wid)
    }

    #[test]
    fn every_change_spelling_names_the_same_change() {
        assert_eq!(parse_change_ids("r12"), Ok(vec![body("12")]));
        assert_eq!(
            parse_change_ids("header1.r1"),
            Ok(vec![ChangeId::new("header1", "1")])
        );
        // Arthur, 2026-10-15: only the two id types. A long id whose change
        // has a short spelling is refused, naming it.
        for (long, short) in [("body:rev:12", "r12"), ("header1:rev:1", "header1.r1")] {
            let refused = parse_change_ids(long).unwrap_err();
            assert_eq!(refused.hint.as_deref(), Some(short), "{long}");
            assert_eq!(refused.expected, CHANGE_EXPECTED, "{long}");
        }
        assert_eq!(parse_change_ids("r1+2"), Ok(vec![body("1"), body("2")]));
        let pair = parse_change_ids("header1.r1+2").unwrap();
        assert_eq!(pair[1].short(), "header1.r2");
        for (expired, hint) in [
            ("#12", Some("r12")),
            ("12", Some("r12")),
            ("#1+2", Some("r1+2")),
            ("1+2", Some("r1+2")),
            ("01", None),
            ("body.r12", None),
            ("bogus.r1", None),
        ] {
            let refused = parse_change_ids(expired).unwrap_err();
            assert_eq!(refused.hint.as_deref(), hint, "{expired}");
        }
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
        for wid in ["007", "4294967296", "x9", "-5", "1+2", "1:2", " 7", ""] {
            let printed = body(wid).short();
            assert_eq!(parse_change_ids(&printed), Ok(vec![body(wid)]), "{printed}");
        }
        assert_ne!(body("007"), body("7"));
        for long in [":rev:7", "a:b:rev:7", "body:p:7"] {
            assert_eq!(ChangeId::from_long(long), None, "{long}");
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
            "a:b:rev:1",
        ] {
            let err = parse_change_ids(token).unwrap_err();
            assert_eq!(err.given, token);
            assert_eq!(err.expected, CHANGE_EXPECTED, "{token}");
        }
        assert_eq!(
            parse_change_ids("r4294967295"),
            Ok(vec![body("4294967295")])
        );
    }

    #[test]
    fn comment_ids_take_one_spelling() {
        assert_eq!(parse_comment_id("c27"), Ok(27));
        for (expired, hint) in [("#c27", "c27"), ("27", "c27")] {
            let refused = parse_comment_id(expired).unwrap_err();
            assert_eq!(refused.hint.as_deref(), Some(hint), "{expired}");
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
            let err = parse_comment_id(token).unwrap_err();
            assert_eq!(
                (err.given.as_str(), err.expected),
                (token, COMMENT_EXPECTED)
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
        // A JSON number is a typed value, not a spelling; a string is an id.
        for json in [r#"{"comment_id":5}"#, r#"{"comment_id":"c5"}"#] {
            let op: Op = serde_json::from_str(json).unwrap();
            assert_eq!(op.comment_id, 5, "{json}");
        }
        for (json, message) in [
            (
                r#"{"comment_id":"p5"}"#,
                "p5: expected a comment id such as c27",
            ),
            (
                r##"{"comment_id":"#c5"}"##,
                "#c5: expected a comment id such as c27; the settled spelling is c5",
            ),
            (
                r#"{"comment_id":"5"}"#,
                "5: expected a comment id such as c27; the settled spelling is c5",
            ),
        ] {
            let err = serde_json::from_str::<Op>(json).err().unwrap().to_string();
            assert!(err.contains(message), "{err}");
        }
    }

    #[test]
    fn locations_follow_the_table() {
        use Location::*;
        let ok = |token: &str| parse_location(token).unwrap();
        assert_eq!(ok("c5"), Comment(5));
        assert_eq!(ok("c5.text"), CommentText(5));
        assert_eq!(ok("r12"), Changes(vec![body("12")]));
        assert_eq!(ok("r1+2"), Changes(vec![body("1"), body("2")]));
        assert_eq!(
            ok("header1.r1"),
            Changes(vec![ChangeId::new("header1", "1")])
        );
        for token in [
            "p51000",
            "t0.row1.cell2",
            "t0.row1.cell2.p1",
            "header1",
            "header1.p2",
            "footnotes.p3",
            "endnotes.p0",
        ] {
            assert_eq!(ok(token), Paragraph(token.into()), "{token}");
        }
        // Arthur, 2026-10-15: only the two id types. A long id with a short
        // spelling is refused, naming it; one without (a part outside the
        // settled stories) is the id jubarte prints, so it reads back.
        for (long, short) in [
            ("body:rev:3", "r3"),
            ("body:p:3", "p3"),
            ("header1:p:0", "header1"),
            ("header1:p:2", "header1.p2"),
        ] {
            let refused = parse_location(long).unwrap_err();
            assert_eq!(refused.hint.as_deref(), Some(short), "{long}");
        }
        assert_eq!(ok("Header_1:p:2"), Paragraph("Header_1:p:2".into()));
        assert_eq!(
            ok("Header_1:rev:4"),
            Changes(vec![ChangeId::new("Header_1", "4")])
        );
        // Arthur, 2026-10-13: everything but the settled ids is expired,
        // and refused with the settled spelling.
        for (expired, hint) in [
            ("#c5", "c5"),
            ("#c5.text", "c5.text"),
            ("#12", "r12"),
            ("27", "p27, r27 or c27"),
            ("0", "p0, r0 or c0"),
            ("t0.r1.c2", "t0.row1.cell2"),
            ("t0.r1.c2.p1", "t0.row1.cell2.p1"),
        ] {
            let refused = parse_location(expired).unwrap_err();
            assert_eq!(refused.hint.as_deref(), Some(hint), "{expired}");
            assert!(refused.to_string().contains(LOCATION_EXPECTED), "{refused}");
        }
        // Settled ids -p does not edit yet.
        for token in [
            "p3-p7",
            "t0.row1.cell2.box0.p0",
            "p14.box0.p0",
            "cc0",
            "s0",
            "toc0",
        ] {
            let refused = parse_location(token).unwrap_err();
            assert_eq!(refused.expected, LOCATION_NOT_YET, "{token}");
        }
    }

    #[test]
    fn refused_locations_say_why() {
        let why = |token: &str| parse_location(token).unwrap_err().to_string();
        for token in ["t0", "t0.row1", "t12"] {
            assert_eq!(
                why(token),
                format!("{token}: a table id needs a row and a cell, as t0.row1.cell2")
            );
        }
        for token in ["row1", "row1.cell2", "cell2", "cell2.p1"] {
            assert_eq!(
                why(token),
                format!("{token}: a row or cell needs its table, as t0.row1.cell2"),
            );
        }
        for token in [
            "1+2",
            "t0.r1",
            "r1.c2",
            "c2.p1",
            "header",
            "rows",
            "bogus.p1",
            "p5-",
            "-p7",
            "5-7",
            "#x",
            "#c",
            "Header_1",
            "c05",
            "#c05",
            "c05.text",
            "c.text",
            "c4294967296",
            "c4294967296.text",
            "t0.p1",
            "t0.row1.c2",
            "t0.r1.cell2",
            "t0.row01.cell2",
            "t0.row1.cell2.p1.p2",
        ] {
            assert!(
                why(token).starts_with(&format!("{token}: {LOCATION_EXPECTED}")),
                "{}",
                why(token)
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
            // A story ShortId does not name keeps its long id: what is
            // printed parses.
            ("hdr:p:2", "hdr:p:2"),
            ("styles:p:2", "styles:p:2"),
            ("p12", "p12"),
        ] {
            assert_eq!(short_paragraph(long), short, "{long}");
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
            COMMENT_TEXT_SPELLINGS,
            PARAGRAPH_SPELLINGS,
            TABLE_SPELLINGS,
            CONTROL_SPELLINGS,
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
        // A part's stem is any file name: the story id spells what a shell
        // reads (`!`, `$`, space, `*`, a leading `-`, non-ASCII) as `%XX`.
        let shell_safe = |id: &str| {
            id.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._%:+-".contains(&b))
                && !id.starts_with('-')
        };
        for part in [
            "word/hdr!old.xml",
            "word/a$b c*.xml",
            "word/-x.xml",
            "word/ヘッダー.xml",
            "word/50%.xml",
            "word/Header_1.xml",
        ] {
            let story = part_story(part);
            for id in [
                ChangeId::new(&story, "4").short(),
                short_paragraph(&format!("{story}:p:0")),
                short_paragraph(&format!("{story}:p:3")),
            ] {
                assert!(shell_safe(&id), "{part} → {id}");
            }
        }
        assert_eq!(part_story("word/hdr!old.xml"), "hdr%21old");
        assert_eq!(part_story("word/Header_1.xml"), "Header_1");
        assert_eq!(part_story("word/header1.xml"), "header1");
        assert_eq!(part_story("word/50%.xml"), "50%25");
        assert_eq!(part_story("word/-x.xml"), "%2Dx");
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
            ".text",
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
                Ok(Location::CommentText(n)) => assert_eq!(
                    parse_location(&format!("c{n}.text")),
                    Ok(Location::CommentText(n)),
                    "{token}"
                ),
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
