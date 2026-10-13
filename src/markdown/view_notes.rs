// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The view's own notes read back: the YAML `authors:` table
//! (`AC: Ann Counsel  # 9 revisions, 2026-10-01T09:00:00Z`), a revision's
//! tag note (`r1+2 @AC`, `r1 @AC; r2 @JD`, `header1.r4 @AC 2026-…`) and a
//! comment's head (`c5 @AC: `, `c6 @AS re c5: `, `c7 @AC resolved: `).
//! `jubarte read` writes them (`from_docx::agent`, `from_docx::header`).

use std::collections::HashMap;

/// An author of the `authors:` table: the full name, and the timestamp
/// every one of their changes and comments shares, when they share one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Author {
    pub name: String,
    pub date: Option<String>,
}

/// The `authors:` table of a YAML header, by handle. `document_owner` is
/// not an author.
pub(super) fn authors(yaml: &str) -> HashMap<String, Author> {
    let mut table = HashMap::new();
    let mut inside = false;
    for line in yaml.lines() {
        if !line.starts_with(' ') {
            inside = line.trim_end() == "authors:";
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
        if value.is_empty() {
            continue;
        }
        let date = comment
            .and_then(|c| c.rsplit(',').next())
            .map(str::trim)
            .filter(|d| super::write::is_date(d))
            .map(str::to_string);
        table.insert(handle.to_string(), Author { name: value, date });
    }
    table
}

/// Letters and digits: the view's handles (`from_docx::agent`).
fn is_handle(text: &str) -> bool {
    !text.is_empty() && text.chars().all(char::is_alphanumeric)
}

/// A YAML scalar as `header::scalar` writes it (plain, or a JSON-quoted
/// string), and the `# comment` after it.
fn scalar_and_comment(text: &str) -> (String, Option<&str>) {
    if text.starts_with('"') {
        let mut de = serde_json::Deserializer::from_str(text).into_iter::<String>();
        if let Some(Ok(value)) = de.next() {
            let rest = text[de.byte_offset()..].trim_start();
            return (value, rest.strip_prefix('#').map(str::trim));
        }
        return (String::new(), None);
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
                    .map(|ids| ids.iter().filter_map(|c| c.wid.parse().ok()).collect())
                    .unwrap_or_default(),
            })
        })
        .collect()
}

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

/// The most empty paragraphs one id line stands for: a bound on what a
/// hand-edited range (`p0-p4294967295 empty`) can make the reader write.
const MOST_EMPTY: usize = 100_000;

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
            last.checked_sub(first)? + 1
        }
        None => {
            crate::ids::short_number(id, 'p')?;
            1
        }
    };
    // `empty` is a clause of its own, or the first clause after the style
    // name (`BodyText empty`).
    let empty = rest.split(", ").enumerate().any(|(i, clause)| {
        clause == "empty"
            || (i == 0
                && clause
                    .split_once(' ')
                    .is_some_and(|(_, tail)| tail == "empty"))
    });
    (empty && count <= MOST_EMPTY).then_some(count)
}

/// The words that open an id line's clauses (`center`, `left 0.5in`,
/// `break-ins r3 @AC`): any other word after the id is a style.
const CLAUSE_WORDS: [&str; 17] = [
    "empty",
    "center",
    "right",
    "justify",
    "left",
    "first-line",
    "hanging",
    "bullet",
    "num",
    "page-break",
    "section-break",
    "break-ins",
    "break-del",
    "fmt",
    "rev",
    "comments",
    "in",
];

/// What a paragraph's id line says about it (`<!-- p5 TitlePage center,
/// first-line 0.5in -->`): its style, alignment (`w:jc`) and indents in
/// twips.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct IdLine {
    pub style: Option<String>,
    pub align: Option<&'static str>,
    pub first_line: Option<i64>,
    pub hanging: Option<i64>,
    pub left: Option<i64>,
    pub right: Option<i64>,
    /// The paragraph holds a page break (`page-break`).
    pub page_break: bool,
}

/// The id line of one paragraph (`pN`, not a range); `None` for any
/// other line.
pub(super) fn id_line(line: &str) -> Option<IdLine> {
    let inner = line
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    let (id, rest) = inner.split_once(' ').unwrap_or((inner, ""));
    crate::ids::short_number(id, 'p')?;
    let mut parsed = IdLine::default();
    let mut clauses = rest.split(", ").filter(|c| !c.is_empty()).peekable();
    // The style is the head's second word: `p5 TitlePage[ clause]`.
    if let Some(first) = clauses.peek().copied() {
        let (word, tail) = first.split_once(' ').unwrap_or((first, ""));
        if !CLAUSE_WORDS.contains(&word) {
            parsed.style = Some(word.to_string());
            clauses.next();
            if !tail.is_empty() {
                parsed.clause(tail);
            }
        }
    }
    for clause in clauses {
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
            }
            Some(("first-line", v)) => self.first_line = twips(v),
            Some(("hanging", v)) => self.hanging = twips(v),
            Some(("left", v)) => self.left = twips(v),
            Some(("right", v)) => self.right = twips(v),
            Some(_) => {}
        }
    }
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
    use super::*;

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
        assert_eq!(id_line("<!-- p2-p4 empty -->"), None);
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
            ("<!-- p0-p99999 empty -->", Some(100_000)),
            ("p2 empty", None),
        ] {
            assert_eq!(empty_paragraphs(line), count, "{line}");
        }
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
                date: Some("2026-10-01T09:00:00Z".into())
            }
        );
        assert_eq!(table["AS"].date.as_deref(), Some("2026-10-09T16:13:00Z"));
        assert_eq!(
            table["JD"],
            Author {
                name: "Jo #1 Doe".into(),
                date: None
            }
        );
        assert_eq!(table["R"].date, None);
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
