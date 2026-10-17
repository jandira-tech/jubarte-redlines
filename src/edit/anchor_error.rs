// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Actionable refusals for v2 literal anchors (V2-11).
//!
//! [`v2_resolve`] and [`v2_resolve_all`] select exactly like
//! [`super::v2_select`] and [`super::v2_select_all`] — every exact start
//! position counted, overlapping candidates included — and, when the anchor
//! misses or repeats, return a structured [`crate::result::Error`] instead of
//! a bare message: the stable code, the operation pointer, the source digest,
//! the requested target, the complete match count, bounded exact context and
//! a read continuation as a structured argv.
//!
//! The refusal's facts all come from one open of the source ([`snapshot`]):
//! existence, the target's own text, the elsewhere suggestions and the
//! missing-target counts agree because they are one parse, on every path,
//! including the ones that resolve. The working text is bound to the target
//! by [`Working`]: a caller with no settled text passes [`Working::InSource`]
//! and the resolver derives the text itself, while a caller holding a settled
//! working document W (V2-06/V2-10) states that provenance with
//! [`Working::Settled`].
//!
//! A target is whatever `-p` accepts ([`crate::ids::parse_location`]): a
//! paragraph (`p12`), a story or its paragraph (`header1`, `header1.p2`), a
//! table cell (`t0.row1.cell2`, its own paragraph), a comment (`c5`), the
//! text a comment covers (`c5.text`) or a revision (`r12`). A target the
//! source does not have refuses `ANCHOR_NOT_FOUND` before anything else is
//! judged — an empty anchor or an occurrence 0 included — and its message
//! says what the source does have: the story's own paragraph count when the
//! target names a story, the body's, or the table, comment or revision
//! counts. A source that cannot be parsed says so; it never claims counts it
//! could not see.
//!
//! Candidate matches in other paragraphs are suggestions only: they appear in
//! the message and as read continuations, never as an automatic retarget, and
//! the refused error keeps naming the requested target. Document text stays
//! untrusted data: it is echoed bounded (and escaped, in messages) and never
//! lands in an argv. The caller's anchor is bounded the same way: a pasted
//! clause shows its first [`WINDOW`] characters, with the cut said.
//!
//! Bounds: a match shows up to [`WINDOW`] characters on either side (a miss
//! shows the first [`WINDOW`] characters of the actual target text); at most
//! [`MAX_SUGGESTED_TARGETS`] other paragraphs are suggested.

use super::EditError;
use super::anchor::{shown, v2_candidates, v2_select, v2_select_all};
use crate::comments::CommentRecord;
use crate::ids::{
    Location, ShortId, Step, Story, paragraph_place, parse_change_ids, parse_location,
};
use crate::inspect::{InspectError, Paragraph, Snapshot, snapshot, source_sha256};
use crate::result::{CommitState, Context, Error, NextAction};

/// Characters of context on either side of a match, of the actual target
/// text shown on a miss, and of the anchor the message quotes.
const WINDOW: usize = 20;
/// Other paragraphs a refusal may suggest at most.
const MAX_SUGGESTED_TARGETS: usize = 5;
/// Blocks the read continuation of a missing target suggests.
const READ_SUGGESTION_HEAD: usize = 20;

/// The refused request's own facts: which operation asked, which target it
/// named, the anchor it sent, and where the source lives for read
/// continuations. None of this is document text.
#[derive(Clone, Copy, Debug)]
pub struct AnchorRefusal<'a> {
    /// Operation pointer or ID (`/operations/3`, an op id, ...).
    pub operation: Option<&'a str>,
    /// The requested target (`p15`, `header1.p3`, ...), kept verbatim: the
    /// refusal never retargets.
    pub target: &'a str,
    /// The exact literal anchor that missed or repeated.
    pub anchor: &'a str,
    /// Source path for read continuations. The digest always binds the
    /// bytes; without a path the argv omits the file and says so.
    pub path: Option<&'a str>,
}

impl<'a> AnchorRefusal<'a> {
    /// The refusal facts for one operation.
    pub fn new(target: &'a str, anchor: &'a str) -> Self {
        Self {
            operation: None,
            target,
            anchor,
            path: None,
        }
    }

    /// Set the operation pointer.
    #[must_use]
    pub fn with_operation(mut self, operation: &'a str) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Set the source path for read continuations.
    #[must_use]
    pub fn with_path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }
}

/// Where an anchor's working text comes from, so the refusal's facts cannot
/// drift away from the target they attest.
#[derive(Clone, Copy, Debug)]
pub enum Working<'a> {
    /// The target's own text in the original source. The resolver reads it
    /// from the one open it already makes, so a caller with no settled text
    /// cannot hand in text that is not the target's.
    InSource,
    /// The target's text in the settled working document W (V2-06/V2-10).
    /// Settlement is the caller's: the refusal still binds the digest, the
    /// target inventory and the elsewhere suggestions to `source`, and this
    /// variant is where the caller states that the text is W's, not S's.
    Settled(&'a str),
}

/// Select the unique v2 literal candidate (or the 1-based `occurrence`)
/// exactly like [`super::v2_select`], refusing with a structured, actionable
/// error when the anchor misses, repeats or is invalid.
///
/// `working` binds the target's text to where it came from
/// ([`Working`]); `source` is the original package the request is bound to
/// (its digest, its target inventory and its elsewhere suggestions), opened
/// once for all of them. A request against a target the source does not have
/// refuses before any text is searched — whatever else is wrong with it —
/// because there is nothing to edit, and no match elsewhere retargets it.
///
/// # Errors
///
/// [`Error`] (boxed to keep the [`Result`] small) with the code
/// [`super::v2_select`] chose (`INVALID_EDIT`, `ANCHOR_NOT_FOUND`,
/// `AMBIGUOUS_ANCHOR`), `ANCHOR_NOT_FOUND` for a missing target, or the
/// open failure's code for a source that cannot be read, carrying the
/// bounded context and read continuations described in the module docs.
pub fn v2_resolve(
    source: &[u8],
    working: Working<'_>,
    refusal: &AnchorRefusal<'_>,
    occurrence: Option<usize>,
) -> Result<(usize, usize), Box<Error>> {
    let snap = open(source, refusal)?;
    let target = find_target(&snap, refusal.target);
    let text = match (target, working) {
        (Target::Missing, _) => return Err(Box::new(missing_target(&snap, refusal))),
        (Target::SeveralChanges, _) => {
            return Err(Box::new(several_changes(&snap, refusal)));
        }
        (Target::Here(text), Working::InSource) => text,
        (Target::Here(_), Working::Settled(text)) => text,
    };
    let skip = skip_id(&snap, refusal.target);
    match v2_select(text, refusal.anchor, occurrence) {
        Ok(range) => Ok(range),
        Err(edit) => Err(Box::new(refused(
            &snap, text, &skip, refusal, &edit, occurrence, None,
        ))),
    }
}

/// Select every exact v2 candidate exactly like [`super::v2_select_all`],
/// requiring the complete count to equal `expected` and disjoint ranges,
/// refusing with a structured, actionable error otherwise.
///
/// # Errors
///
/// [`Error`] (boxed to keep the [`Result`] small) with the code
/// [`super::v2_select_all`] chose (`INVALID_EDIT`, `ANCHOR_NOT_FOUND`,
/// `AMBIGUOUS_ANCHOR`, `OVERLAPPING_EDITS`), `ANCHOR_NOT_FOUND` for a
/// missing target, or the open failure's code for a source that cannot be
/// read, carrying the bounded context and read continuations described in
/// the module docs.
pub fn v2_resolve_all(
    source: &[u8],
    working: Working<'_>,
    refusal: &AnchorRefusal<'_>,
    expected: usize,
) -> Result<Vec<(usize, usize)>, Box<Error>> {
    let snap = open(source, refusal)?;
    let target = find_target(&snap, refusal.target);
    let text = match (target, working) {
        (Target::Missing, _) => return Err(Box::new(missing_target(&snap, refusal))),
        (Target::SeveralChanges, _) => {
            return Err(Box::new(several_changes(&snap, refusal)));
        }
        (Target::Here(text), Working::InSource) => text,
        (Target::Here(_), Working::Settled(text)) => text,
    };
    let skip = skip_id(&snap, refusal.target);
    match v2_select_all(text, refusal.anchor, expected) {
        Ok(ranges) => Ok(ranges),
        Err(edit) => Err(Box::new(refused(
            &snap,
            text,
            &skip,
            refusal,
            &edit,
            None,
            Some(expected),
        ))),
    }
}

/// The one open every path through the resolver shares, as [`snapshot`]
/// serves every field: the source's paragraphs, stories, tables, changes and
/// comments, or the structured refusal that says it could not be read. A
/// source that fails here is never described by counts.
fn open(source: &[u8], refusal: &AnchorRefusal<'_>) -> Result<Snapshot, Box<Error>> {
    snapshot(source).map_err(|error| {
        let code = match &error {
            InspectError::Admission(refused) => refused.code(),
            _ => "INVALID_DOCUMENT",
        };
        Box::new(
            base(&source_sha256(source), refusal, code).with_message(format!(
                "the source could not be read ({error}); no anchor was searched"
            )),
        )
    })
}

/// What one request's target is, as the source's own inventory holds it.
enum Target<'s> {
    /// The target exists, and this is its own text in the source.
    Here(&'s str),
    /// The target names several changes (`r1+2`): it exists, but no one
    /// text carries an anchor.
    SeveralChanges,
    /// The source does not have the target.
    Missing,
}

/// Find `target` in the snapshot: every spelling `-p` accepts
/// ([`parse_location`]), or [`Target::Missing`].
fn find_target<'s>(snap: &'s Snapshot, target: &str) -> Target<'s> {
    match parse_location(target) {
        Err(_) => Target::Missing,
        Ok(Location::Paragraph(id)) => {
            if let Some((story, n)) = paragraph_place(&id) {
                let list: &[Paragraph] = if story == "body" {
                    &snap.paragraphs
                } else if let Some(found) = snap.stories.iter().find(|s| s.id == story) {
                    &found.paragraphs
                } else {
                    return Target::Missing;
                };
                return match list.get(n) {
                    Some(paragraph) => Target::Here(&paragraph.text),
                    None => Target::Missing,
                };
            }
            match cell_paragraph(snap, &id) {
                Some((_, text)) => Target::Here(text),
                None => Target::Missing,
            }
        }
        Ok(Location::Comment(n)) => match comment(snap, n) {
            Some(found) => Target::Here(&found.text),
            None => Target::Missing,
        },
        Ok(Location::CommentText(n)) => match comment(snap, n) {
            Some(found) => Target::Here(&found.anchor_text),
            None => Target::Missing,
        },
        Ok(Location::Changes(ids)) => {
            let spelled: Vec<String> = ids.iter().map(|change| change.short()).collect();
            if !spelled
                .iter()
                .all(|short| snap.changes.iter().any(|change| &change.id == short))
            {
                return Target::Missing;
            }
            match spelled.as_slice() {
                [one] => match snap.changes.iter().find(|change| &change.id == one) {
                    Some(change) => Target::Here(&change.text),
                    None => Target::Missing,
                },
                _ => Target::SeveralChanges,
            }
        }
    }
}

/// The comment `w:id` `n` in the snapshot.
fn comment(snap: &Snapshot, n: u32) -> Option<&CommentRecord> {
    snap.comments.iter().find(|comment| comment.id == n)
}

/// The cell `tN.rowR.cellC[.pK]`: its first (or K-th) own paragraph, as the
/// snapshot's grids hold it. Numbering follows the snapshot's table order; a
/// document with a table nested inside a cell numbers its tables differently
/// from `edit -p`, which V2-06's wiring resolves exactly through the edit
/// engine.
fn cell_paragraph<'s>(snap: &'s Snapshot, id: &str) -> Option<(String, &'s str)> {
    let Ok(ShortId::Block {
        story: Story::Body,
        path,
    }) = id.parse::<ShortId>()
    else {
        return None;
    };
    let [Step::Table(t), Step::Row(row), Step::Cell(cell), rest @ ..] = path.as_slice() else {
        return None;
    };
    let index = match rest {
        [] => 0,
        [Step::Paragraph(k)] => *k,
        _ => return None,
    };
    let table = snap.tables.iter().find(|table| table.index == *t)?;
    let held = table.rows.get(*row)?.get(*cell)?;
    let short = held.paragraph_ids.get(index)?;
    let (_, n) = paragraph_place(short)?;
    let paragraph = snap.paragraphs.get(n)?;
    Some((paragraph.id.clone(), &paragraph.text))
}

/// The paragraph a miss must not suggest back: the requested spelling for a
/// paragraph target, the cell's own paragraph for a cell target, and the
/// target itself otherwise.
fn skip_id(snap: &Snapshot, target: &str) -> String {
    if let Ok(Location::Paragraph(id)) = parse_location(target)
        && paragraph_place(&id).is_none()
        && let Some((paragraph, _)) = cell_paragraph(snap, &id)
    {
        return paragraph;
    }
    target.to_string()
}

/// The structured refusal for a target the source does not have: stable
/// code, operation, digest, the requested target kept verbatim, zero
/// matches, and a bounded read of what does exist.
fn missing_target(snap: &Snapshot, refusal: &AnchorRefusal<'_>) -> Error {
    let mut error = base(&snap.source_sha256, refusal, "ANCHOR_NOT_FOUND")
        .with_matches(0)
        .with_message(missing_target_message(snap, refusal.target));
    error.push_next_action(read_head(refusal));
    error
}

/// What a missing target's message says: the target, kept verbatim, and
/// what the source does have where that orients the caller — the story's
/// own paragraph count when the target names a story, the body's, the
/// table, comment or revision counts, and the settled spelling of an
/// expired id.
fn missing_target_message(snap: &Snapshot, target: &str) -> String {
    let body = format!(
        "the body has {}",
        plural(snap.summary.paragraphs, "paragraph")
    );
    match parse_location(target) {
        Ok(Location::Comment(_)) | Ok(Location::CommentText(_)) => format!(
            "no comment {target} in the source; the source has {}",
            plural(snap.summary.comments, "comment")
        ),
        Ok(Location::Changes(_)) => format!(
            "no revision {target} in the source; the source has {}",
            plural(snap.summary.revisions, "revision")
        ),
        Ok(Location::Paragraph(id)) => match paragraph_place(&id) {
            Some((story, _)) if story != "body" => {
                match snap.stories.iter().find(|s| s.id == story) {
                    Some(found) => format!(
                        "no paragraph {target} in the source; the story {story} has {}",
                        plural(found.paragraphs.len(), "paragraph")
                    ),
                    None => format!(
                        "no paragraph {target} in the source; the source has no {story} story; {body}"
                    ),
                }
            }
            Some(_) => format!("no paragraph {target} in the source; {body}"),
            None => format!(
                "no paragraph {target} in the source; the source has {}",
                plural(snap.tables.len(), "table")
            ),
        },
        Err(refused) => match &refused.hint {
            Some(hint) => {
                format!(
                    "no paragraph {target} in the source; the settled spelling is {hint}; {body}"
                )
            }
            None => format!("no paragraph {target} in the source; {body}"),
        },
    }
}

/// The structured refusal for a target that names several changes: it
/// exists, but no one text carries an anchor.
fn several_changes(snap: &Snapshot, refusal: &AnchorRefusal<'_>) -> Error {
    let one_by_one: Vec<String> = parse_change_ids(refusal.target)
        .map(|ids| ids.iter().map(|change| change.short()).collect())
        .unwrap_or_default();
    let spelled = match one_by_one.as_slice() {
        [rest @ .., last] if rest.len() < 3 => format!("{} or {last}", rest.join(" or ")),
        _ => String::new(),
    };
    let mut error = base(&snap.source_sha256, refusal, "INVALID_EDIT").with_message(format!(
        "{} names {} changes; an anchor targets one{}",
        refusal.target,
        one_by_one.len(),
        if spelled.is_empty() {
            String::new()
        } else {
            format!(" ({spelled})")
        }
    ));
    error.push_next_action(read_target(refusal));
    error
}

/// The shared refusal skeleton: code, operation pointer, source digest,
/// requested target, no-retry and no-commit.
fn base(digest: &str, refusal: &AnchorRefusal<'_>, code: &str) -> Error {
    let mut error = Error::new(code)
        .with_source_digest(digest)
        .with_target(refusal.target)
        .with_retryable(false)
        .with_commit_state(CommitState::NotCommitted);
    if let Some(operation) = refusal.operation {
        error = error.with_operation(operation);
    }
    error
}

/// Build the structured refusal for an anchor that missed or repeated in an
/// existing target: the complete match count, bounded exact context, the
/// paragraphs holding the anchor elsewhere (a miss only, suggestions only)
/// and read continuations. `skip` is the paragraph not to suggest back.
fn refused(
    snap: &Snapshot,
    working: &str,
    skip: &str,
    refusal: &AnchorRefusal<'_>,
    edit: &EditError,
    occurrence: Option<usize>,
    expected: Option<usize>,
) -> Error {
    let mut error = base(&snap.source_sha256, refusal, &edit.code);
    if edit.code == "INVALID_EDIT" {
        error = error.with_message(edit.message.clone());
        error.push_next_action(read_target(refusal));
        return error;
    }
    let candidates = v2_candidates(working, refusal.anchor);
    let count = candidates.len();
    error = error.with_matches(count);
    error = error.with_context(match candidates.first() {
        Some(&(start, end)) => around(working, start, end),
        None => Context::new(bounded_start(working), "", "")
            .with_truncated(working.chars().count() > WINDOW),
    });
    let shown_anchor = quoted_anchor(refusal.anchor);
    let mut message = match (count, occurrence, expected) {
        (0, _, _) => format!(
            "{shown_anchor} occurs 0 times in {}; {} starts {}",
            refusal.target,
            refusal.target,
            shown(&bounded_start(working))
        ),
        // all-matches refusals never take an occurrence: name their own
        // variant before the occurrence guidance below.
        (_, _, _) if edit.code == "OVERLAPPING_EDITS" => format!(
            "{shown_anchor} occurs {count} times in {} and those ranges overlap, so none of them is replaced",
            refusal.target
        ),
        (_, _, Some(wanted)) => format!(
            "{shown_anchor} occurs {count} times in {}, not the expected {wanted}",
            refusal.target
        ),
        (_, None, _) => format!(
            "{shown_anchor} occurs {count} times in {}; set \"occurrence\" to 1..={count}",
            refusal.target
        ),
        (_, Some(k), _) if k > count => format!(
            "{shown_anchor} occurs {count} times in {}; occurrence {k} is outside occurrence 1..={count}",
            refusal.target
        ),
        // The one case left (an occurrence inside 1..=count, no expected
        // count) is a selection v2_select resolves; spelling the arm keeps
        // the match exhaustive, so a new refusal code cannot fall through
        // to a wrong message silently.
        (_, Some(_), None) => format!("{shown_anchor} occurs {count} times in {}", refusal.target),
    };
    error.push_next_action(read_target(refusal));
    if count == 0 {
        // A miss is where elsewhere helps: bounded suggestions, reads only.
        let suggestions = suggestions_elsewhere(snap, skip, refusal.anchor);
        if !suggestions.list.is_empty() {
            message.push_str("; it occurs elsewhere: ");
            message.push_str(
                &suggestions
                    .list
                    .iter()
                    .map(|(id, hits)| format!("{id} ({hits})"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            if suggestions.more > 0 {
                message.push_str(&format!(", … (+{} more paragraphs)", suggestions.more));
            }
            for (id, hits) in &suggestions.list {
                error.push_next_action(suggest_read(refusal, id, *hits));
            }
        }
    }
    error.with_message(message)
}

/// Other paragraphs where the anchor occurs: `(id, complete count)` in
/// document order (body, then stories), capped, plus how many more were
/// cut, all from the one open the resolver already made.
fn suggestions_elsewhere(snap: &Snapshot, skip: &str, anchor: &str) -> Suggestions {
    let mut list = Vec::new();
    let mut more = 0;
    for paragraph in snap.paragraphs.iter().chain(
        snap.stories
            .iter()
            .flat_map(|story| story.paragraphs.iter()),
    ) {
        if paragraph.id == skip {
            continue;
        }
        let hits = v2_candidates(&paragraph.text, anchor).len();
        if hits == 0 {
            continue;
        }
        if list.len() == MAX_SUGGESTED_TARGETS {
            more += 1;
        } else {
            list.push((paragraph.id.clone(), hits));
        }
    }
    Suggestions { list, more }
}

/// Bounded elsewhere-candidate list.
struct Suggestions {
    /// The suggested paragraphs, at most [`MAX_SUGGESTED_TARGETS`].
    list: Vec<(String, usize)>,
    /// Suggested paragraphs cut by the cap.
    more: usize,
}

/// Up to `WINDOW` characters from the start of the actual target text.
fn bounded_start(working: &str) -> String {
    working.chars().take(WINDOW).collect()
}

/// The anchor as a message quotes it: escaped, and bounded to [`WINDOW`]
/// characters with the cut said, so a pasted clause cannot make the refusal
/// bigger than the request it answers.
fn quoted_anchor(anchor: &str) -> String {
    let count = anchor.chars().count();
    if count <= WINDOW {
        return shown(anchor);
    }
    let head: String = anchor.chars().take(WINDOW).collect();
    format!(
        "{}\u{2026} (+{} more characters)",
        shown(&head),
        count - WINDOW
    )
}

/// Bounded context around `[start, end)`: up to `WINDOW` characters on
/// either side of the matched span, and up to [`WINDOW`] characters of the
/// span itself (the caller's own anchor, echoed back bounded).
fn around(working: &str, start: usize, end: usize) -> Context {
    let matched = &working[start..end];
    let matched_cut = matched.chars().count() > WINDOW;
    let matched: String = matched.chars().take(WINDOW).collect();
    let before: String = working[..start].chars().rev().take(WINDOW).collect();
    let before: String = before.chars().rev().collect();
    let after: String = working[end..].chars().take(WINDOW).collect();
    Context::new(before, matched, after).with_truncated(
        matched_cut
            || working[..start].chars().count() > WINDOW
            || working[end..].chars().count() > WINDOW,
    )
}

/// `{n} {word}` or `{n} {word}s`.
fn plural(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

/// `jubarte read …` argv: the program, the task, the source path when known,
/// and `options`. A path that starts with `-` would parse as an option, so it
/// goes last, after `--`, and the argv stays executable as given. Without a
/// path the continuation says what is missing.
fn read_action(refusal: &AnchorRefusal<'_>, options: [String; 2]) -> NextAction {
    let mut argv = vec!["jubarte".to_string(), "read".to_string()];
    match refusal.path {
        Some(path) if path.starts_with('-') => {
            argv.extend(options);
            argv.extend(["--".to_string(), path.to_string()]);
            NextAction::new(argv)
        }
        Some(path) => {
            argv.push(path.to_string());
            argv.extend(options);
            NextAction::new(argv)
        }
        None => {
            argv.extend(options);
            NextAction::new(argv).with_prerequisites(vec![
                "the source file path (the refusal binds its digest)".to_string(),
            ])
        }
    }
}

/// The read continuation for the requested target.
fn read_target(refusal: &AnchorRefusal<'_>) -> NextAction {
    read_action(refusal, ["-p".to_string(), refusal.target.to_string()])
        .with_purpose(format!("read the actual text of {}", refusal.target))
}

/// The bounded read continuation for a target that does not exist.
fn read_head(refusal: &AnchorRefusal<'_>) -> NextAction {
    read_action(
        refusal,
        ["--head".to_string(), READ_SUGGESTION_HEAD.to_string()],
    )
    .with_purpose("list the paragraphs that exist, bounded".to_string())
}

/// The read continuation that offers a candidate elsewhere — a suggestion,
/// never a retarget.
fn suggest_read(refusal: &AnchorRefusal<'_>, id: &str, hits: usize) -> NextAction {
    let mut action = read_action(refusal, ["-p".to_string(), id.to_string()]);
    let mut prerequisites = action.prerequisites.take().unwrap_or_default();
    prerequisites.push("re-targeting is the caller's decision".to_string());
    action
        .with_purpose(format!(
            "suggestion only: the anchor occurs {hits} times in {id}; a supplied target is never retargeted"
        ))
        .with_prerequisites(prerequisites)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn a_miss_shows_the_bounded_start_of_the_target() {
        assert_eq!(bounded_start("0123456789"), "0123456789");
        let long = "x".repeat(WINDOW + 5);
        assert_eq!(bounded_start(&long).chars().count(), WINDOW);
    }

    #[test]
    fn a_match_context_is_banded_on_both_sides() {
        let text = format!("{}y{}z", "a".repeat(WINDOW + 3), "b".repeat(WINDOW + 7));
        let start = WINDOW + 3;
        let end = start + 1;
        let context = around(&text, start, end);
        assert_eq!(context.matched, "y");
        assert_eq!(context.before.chars().count(), WINDOW);
        assert_eq!(context.after.chars().count(), WINDOW);
        assert!(context.truncated);
        let context = around("abcYdef", 3, 4);
        assert_eq!(
            (
                context.before.as_str(),
                context.matched.as_str(),
                context.after.as_str()
            ),
            ("abc", "Y", "def")
        );
        assert!(!context.truncated);
    }

    #[test]
    fn a_pasted_clause_anchor_is_quoted_bounded() {
        let short = "needle";
        assert_eq!(quoted_anchor(short), shown(short));
        let clause = "c".repeat(3 * WINDOW);
        let quoted = quoted_anchor(&clause);
        assert!(!quoted.contains(&clause), "{quoted}");
        assert_eq!(
            quoted,
            format!(
                "{}\u{2026} (+{} more characters)",
                shown(&"c".repeat(WINDOW)),
                2 * WINDOW
            )
        );
        // The matched span of a long anchor is bounded too, and says so.
        let text = clause.clone();
        let context = around(&text, 0, text.len());
        assert_eq!(context.matched, "c".repeat(WINDOW));
        assert!(context.truncated);
    }

    #[test]
    fn counts_are_pluralized() {
        assert_eq!(plural(1, "paragraph"), "1 paragraph");
        assert_eq!(plural(2, "revision"), "2 revisions");
    }
}
