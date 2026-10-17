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
//! Candidate matches in other paragraphs are suggestions only: they appear in
//! the message and as read continuations, never as an automatic retarget, and
//! the refused error keeps naming the requested target. Document text stays
//! untrusted data: it is echoed bounded (and escaped, in messages) and never
//! lands in an argv.
//!
//! Bounds: a match shows up to [`WINDOW`] characters on either side (a miss
//! shows the first [`WINDOW`] characters of the actual target text); at most
//! [`MAX_SUGGESTED_TARGETS`] other paragraphs are suggested.

use super::EditError;
use super::anchor::{shown, v2_candidates, v2_select, v2_select_all};
use crate::inspect::{paragraphs, source_sha256, stories};
use crate::result::{CommitState, Context, Error, NextAction};

/// Characters of context on either side of a match, and of the actual
/// target text shown on a miss.
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

/// Select the unique v2 literal candidate (or the 1-based `occurrence`)
/// exactly like [`super::v2_select`], refusing with a structured, actionable
/// error when the anchor misses, repeats or is invalid.
///
/// `working` is the target's effective working text the anchor was matched
/// against; `source` is the original package the request is bound to (its
/// digest, its target inventory and its elsewhere suggestions). A request
/// against a target the source does not have refuses before any text is
/// searched: there is nothing to edit, and no match elsewhere retargets it.
///
/// # Errors
///
/// [`Error`] (boxed to keep the [`Result`] small) with the code
/// [`super::v2_select`] chose (`INVALID_EDIT`, `ANCHOR_NOT_FOUND`,
/// `AMBIGUOUS_ANCHOR`), or `ANCHOR_NOT_FOUND` for a missing target, carrying
/// the bounded context and read continuations described in the module docs.
pub fn v2_resolve(
    source: &[u8],
    working: &str,
    refusal: &AnchorRefusal<'_>,
    occurrence: Option<usize>,
) -> Result<(usize, usize), Box<Error>> {
    if !refusal.anchor.is_empty() && occurrence != Some(0) && !target_exists(source, refusal.target)
    {
        return Err(Box::new(missing_target(source, refusal)));
    }
    match v2_select(working, refusal.anchor, occurrence) {
        Ok(range) => Ok(range),
        Err(edit) => Err(Box::new(refused(
            source, working, refusal, &edit, occurrence, None,
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
/// `AMBIGUOUS_ANCHOR`, `OVERLAPPING_EDITS`), or `ANCHOR_NOT_FOUND` for a
/// missing target, carrying the bounded context and read continuations
/// described in the module docs.
pub fn v2_resolve_all(
    source: &[u8],
    working: &str,
    refusal: &AnchorRefusal<'_>,
    expected: usize,
) -> Result<Vec<(usize, usize)>, Box<Error>> {
    if !refusal.anchor.is_empty() && !target_exists(source, refusal.target) {
        return Err(Box::new(missing_target(source, refusal)));
    }
    match v2_select_all(working, refusal.anchor, expected) {
        Ok(ranges) => Ok(ranges),
        Err(edit) => Err(Box::new(refused(
            source,
            working,
            refusal,
            &edit,
            None,
            Some(expected),
        ))),
    }
}

/// The structured refusal for a target the source does not have: stable
/// code, operation, digest, the requested target kept verbatim, zero
/// matches, and a bounded read of what does exist.
fn missing_target(source: &[u8], refusal: &AnchorRefusal<'_>) -> Error {
    let mut error = base(source, refusal, "ANCHOR_NOT_FOUND")
        .with_matches(0)
        .with_message(format!(
            "no paragraph {} in the source; the body has {} paragraphs",
            refusal.target,
            paragraphs(source).map(|body| body.len()).unwrap_or(0),
        ));
    error.push_next_action(read_head(refusal));
    error
}

/// The shared refusal skeleton: code, operation pointer, source digest,
/// requested target, no-retry and no-commit.
fn base(source: &[u8], refusal: &AnchorRefusal<'_>, code: &str) -> Error {
    let mut error = Error::new(code)
        .with_source_digest(source_sha256(source))
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
/// and read continuations.
fn refused(
    source: &[u8],
    working: &str,
    refusal: &AnchorRefusal<'_>,
    edit: &EditError,
    occurrence: Option<usize>,
    expected: Option<usize>,
) -> Error {
    let mut error = base(source, refusal, &edit.code);
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
    let shown_anchor = shown(refusal.anchor);
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
            "{shown_anchor} occurs {count} times in {} and those ranges overlap; all-matches refuses them all",
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
        (_, _, _) => format!("{shown_anchor} occurs {count} times in {}", refusal.target),
    };
    error.push_next_action(read_target(refusal));
    if count == 0 {
        // A miss is where elsewhere helps: bounded suggestions, reads only.
        let suggestions = suggestions_elsewhere(source, refusal.target, refusal.anchor);
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

/// Whether the source has a paragraph (body or story) with this id.
fn target_exists(source: &[u8], target: &str) -> bool {
    if paragraphs(source)
        .map(|body| body.iter().any(|p| p.id == target))
        .unwrap_or(false)
    {
        return true;
    }
    stories(source)
        .map(|found| {
            found
                .iter()
                .any(|story| story.paragraphs.iter().any(|p| p.id == target))
        })
        .unwrap_or(false)
}

/// Other paragraphs where the anchor occurs: `(id, complete count)` in
/// document order (body, then stories), capped, plus how many more were cut.
fn suggestions_elsewhere(source: &[u8], target: &str, anchor: &str) -> Suggestions {
    let mut all: Vec<(String, String)> = Vec::new();
    if let Ok(body) = paragraphs(source) {
        all.extend(body.iter().map(|p| (p.id.clone(), p.text.clone())));
    }
    if let Ok(found) = stories(source) {
        for story in found {
            all.extend(
                story
                    .paragraphs
                    .iter()
                    .map(|p| (p.id.clone(), p.text.clone())),
            );
        }
    }
    let mut list = Vec::new();
    let mut more = 0;
    for (id, text) in all {
        if id == target {
            continue;
        }
        let hits = v2_candidates(&text, anchor).len();
        if hits == 0 {
            continue;
        }
        if list.len() == MAX_SUGGESTED_TARGETS {
            more += 1;
        } else {
            list.push((id, hits));
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

/// Bounded context around `[start, end)`: up to `WINDOW` characters on
/// either side of the verbatim matched span.
fn around(working: &str, start: usize, end: usize) -> Context {
    let before: String = working[..start].chars().rev().take(WINDOW).collect();
    let before: String = before.chars().rev().collect();
    let after: String = working[end..].chars().take(WINDOW).collect();
    Context::new(before, working[start..end].to_string(), after).with_truncated(
        working[..start].chars().count() > WINDOW || working[end..].chars().count() > WINDOW,
    )
}

/// `jubarte read …` argv prefix: the program, the task and, when known, the
/// source path. Without a path the continuation says what is missing.
fn read_action(refusal: &AnchorRefusal<'_>) -> NextAction {
    let mut argv = vec!["jubarte".to_string(), "read".to_string()];
    match refusal.path {
        Some(path) => {
            argv.push(path.to_string());
            NextAction::new(argv)
        }
        None => NextAction::new(argv).with_prerequisites(vec![
            "the source file path (the refusal binds its digest)".to_string(),
        ]),
    }
}

/// The read continuation for the requested target.
fn read_target(refusal: &AnchorRefusal<'_>) -> NextAction {
    let mut action = read_action(refusal);
    action.argv.push("-p".to_string());
    action.argv.push(refusal.target.to_string());
    action.with_purpose(format!("read the actual text of {}", refusal.target))
}

/// The bounded read continuation for a target that does not exist.
fn read_head(refusal: &AnchorRefusal<'_>) -> NextAction {
    let mut action = read_action(refusal);
    action.argv.push("--head".to_string());
    action.argv.push(READ_SUGGESTION_HEAD.to_string());
    action.with_purpose("list the paragraphs that exist, bounded".to_string())
}

/// The read continuation that offers a candidate elsewhere — a suggestion,
/// never a retarget.
fn suggest_read(refusal: &AnchorRefusal<'_>, id: &str, hits: usize) -> NextAction {
    let mut action = read_action(refusal);
    action.argv.push("-p".to_string());
    action.argv.push(id.to_string());
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
}
