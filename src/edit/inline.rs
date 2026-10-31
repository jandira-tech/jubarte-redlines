// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Compile one typed action into PlanV2; mutation stays in the shared engine.

use super::{
    ExistingRevisions,
    for_edit::{ForEditRequest, for_edit},
    v2::{self, PlanV2, Target},
};
use crate::cli::edit_actions::EditAction;
use crate::inspect::source_sha256;
use crate::result::Error;

fn refusal(code: &str, message: impl Into<String>) -> Box<Error> {
    Box::new(Error::new(code).with_message(message))
}
fn edit_error(e: super::EditError) -> Box<Error> {
    let mut error = Error::new(e.code).with_message(e.message);
    error.operation = e.operation;
    error.near = e.near.map(|near| *near);
    Box::new(error)
}
fn pair(p: &super::for_edit::TargetPair) -> Target {
    Target {
        short_id: p.short_id.clone(),
        long_id: p.long_id.clone(),
    }
}

/// Compile against exact working text, preserving supplied payload characters.
/// This performs no I/O and no mutation. Callers read UTF-8 payload files once
/// and protect them as inputs during publication.
///
/// # Errors
/// Returns source/target/working-basis errors or an actionable exact-anchor refusal.
pub fn compile(
    source: &[u8],
    action: &EditAction,
    text: Option<&str>,
) -> Result<PlanV2, Box<Error>> {
    if let Some(text) = text {
        v2::check_payload_text(
            text,
            action.payload().is_some_and(|p| p.text_file.is_some()),
        )
        .map_err(edit_error)?;
    }
    let inline = action
        .inline()
        .ok_or_else(|| refusal("USAGE", "apply does not compile an inline action"))?;
    let existing = match inline.existing_revisions.as_str() {
        "keep" => ExistingRevisions::Keep,
        "accept" => ExistingRevisions::Accept,
        "reject" => ExistingRevisions::Reject,
        _ => ExistingRevisions::Refuse,
    };
    let (accept, reject) =
        super::source_refs::decisions(source, &inline.accept_change, &inline.reject_change)
            .map_err(edit_error)?;
    // Select all original identities, then match anchors in their working text.
    let view = for_edit(
        source,
        &ForEditRequest {
            all: true,
            accept: accept
                .iter()
                .map(|target| target.short_id.clone())
                .collect(),
            reject: reject
                .iter()
                .map(|target| target.short_id.clone())
                .collect(),
            existing_revisions: existing,
            track: inline.track == "on",
            ..Default::default()
        },
    )
    .map_err(edit_error)?;
    let at = match action {
        EditAction::Replace { at, .. } => at.as_deref(),
        EditAction::Rewrite { at, .. }
        | EditAction::Insert { at, .. }
        | EditAction::Delete { at, .. } => Some(at.as_str()),
        _ => unreachable!(),
    };
    let mut candidates: Vec<_> = view
        .paragraphs
        .iter()
        .filter(|row| at.is_none_or(|id| row.target.short_id == id || row.target.long_id == id))
        .collect();
    if at.is_some() && candidates.len() != 1 {
        return Err(refusal(
            "INVALID_TARGET",
            format!(
                "--at {} must identify exactly one original-source paragraph",
                at.unwrap_or_default()
            ),
        ));
    }
    let mut operations = Vec::new();
    let payload = text.unwrap_or_default().to_string();
    let comment = action
        .payload()
        .and_then(|p| p.comment.as_ref())
        .map(|text| v2::AttachedComment { text: text.clone() });
    match action {
        EditAction::Replace {
            find,
            at,
            story,
            occurrence,
            all_matches,
            expect_count,
            ..
        } => {
            if find.is_empty() {
                return Err(refusal("INVALID_EDIT", "--find must not be empty"));
            }
            if at.is_none() {
                let story = story.as_deref().unwrap_or("body");
                super::anchor_error::require_story(source, story)?;
                candidates.retain(|r| r.story == story);
            }
            let mut hits = Vec::new();
            for row in &candidates {
                if let Some(working) = &row.working {
                    let working =
                        working.value.text.as_deref().ok_or_else(|| {
                            refusal("UNSUPPORTED_TEXT", "working text is incomplete")
                        })?;
                    for (local, _) in super::v2_candidates(working, find).iter().enumerate() {
                        hits.push((*row, local + 1));
                    }
                }
            }
            let with_near = |mut error: Box<Error>| {
                if hits.is_empty() && error.code == "ANCHOR_NOT_FOUND" {
                    let mut report = super::NearMatches {
                        total: 0,
                        occurrences: Vec::new(),
                    };
                    for row in &candidates {
                        if let Some(text) =
                            row.working.as_ref().and_then(|w| w.value.text.as_deref())
                            && let Some(near) = super::anchor::near_matches(text, find)
                        {
                            report.total += near.total;
                            let remaining = 5 - report.occurrences.len();
                            report.occurrences.extend(
                                near.occurrences.into_iter().take(remaining).map(|mut hit| {
                                    hit.target = Some(row.target.short_id.clone());
                                    hit
                                }),
                            );
                        }
                    }
                    if report.total > 0 {
                        error.near = Some(report);
                    }
                }
                error
            };
            if *all_matches {
                if hits.len() != expect_count.unwrap_or_default() {
                    return Err(with_near(refusal(
                        if hits.is_empty() {
                            "ANCHOR_NOT_FOUND"
                        } else {
                            "AMBIGUOUS_ANCHOR"
                        },
                        format!(
                            "expected {} matches; found {}",
                            expect_count.unwrap_or_default(),
                            hits.len()
                        ),
                    )));
                }
                for row in &candidates {
                    if let Some(working) = &row.working {
                        let working = working.value.text.as_deref().unwrap_or_default();
                        let count = super::v2_candidates(working, find).len();
                        if count > 0 {
                            super::v2_select_all(working, find, count).map_err(edit_error)?;
                        }
                    }
                }
            } else {
                if let Some(row) = candidates.first().filter(|_| at.is_some()) {
                    let working = row
                        .working
                        .as_ref()
                        .and_then(|w| w.value.text.as_deref())
                        .ok_or_else(|| {
                            refusal(
                                "UNSUPPORTED_TARGET_MAPPING",
                                "target has no one-to-one working paragraph",
                            )
                        })?;
                    super::v2_resolve(
                        source,
                        super::Working::Settled(working),
                        &super::AnchorRefusal::new(&row.target.short_id, find)
                            .with_operation("/operations/0"),
                        *occurrence,
                    )?;
                }
                let index = match occurrence {
                    Some(n) => n - 1,
                    None if hits.len() == 1 => 0,
                    None => {
                        let mut message = format!("expected one exact match; found {}", hits.len());
                        if !hits.is_empty() {
                            let mut paragraphs: Vec<(&str, usize)> = Vec::new();
                            for (row, _) in &hits {
                                let id = row.target.short_id.as_str();
                                if let Some((_, count)) =
                                    paragraphs.last_mut().filter(|(p, _)| *p == id)
                                {
                                    *count += 1;
                                } else {
                                    paragraphs.push((id, 1));
                                }
                            }
                            let shown = paragraphs
                                .into_iter()
                                .map(|(id, count)| {
                                    if count == 1 {
                                        id.to_string()
                                    } else {
                                        format!("{id} ({count})")
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            message.push_str(&format!(
                                ": {shown}; pick one with --at pN or --occurrence 1..={}",
                                hits.len()
                            ));
                        }
                        return Err(with_near(refusal(
                            if hits.is_empty() {
                                "ANCHOR_NOT_FOUND"
                            } else {
                                "AMBIGUOUS_ANCHOR"
                            },
                            message,
                        )));
                    }
                };
                let hit = hits.get(index).copied().ok_or_else(|| {
                    with_near(refusal(
                        if hits.is_empty() {
                            "ANCHOR_NOT_FOUND"
                        } else {
                            "AMBIGUOUS_ANCHOR"
                        },
                        format!("occurrence is outside {} exact matches", hits.len()),
                    ))
                })?;
                hits = vec![hit];
            }
            for (row, n) in hits {
                operations.push(v2::Operation::Replace {
                    target: pair(&row.target),
                    find: find.clone(),
                    text: payload.clone(),
                    occurrence: Some(n as u64),
                    attached_comment: comment.clone(),
                });
            }
        }
        _ => {
            let row = candidates[0];
            let target = pair(&row.target);
            let operation = match action {
                EditAction::Rewrite { .. } => v2::Operation::Rewrite {
                    target,
                    before_sha256: row
                        .working
                        .as_ref()
                        .and_then(|w| w.before_sha256.clone())
                        .ok_or_else(|| {
                            refusal(
                                "UNSUPPORTED_TARGET_MAPPING",
                                "rewrite requires exact one-to-one working correspondence",
                            )
                        })?,
                    text: payload,
                    attached_comment: comment,
                },
                EditAction::Delete {
                    find,
                    whole_block,
                    occurrence,
                    ..
                } => {
                    if *whole_block {
                        v2::Operation::DeleteParagraph { target }
                    } else {
                        v2::Operation::Delete {
                            target,
                            find: find.clone().expect("parser requires find"),
                            occurrence: occurrence.map(|n| n as u64),
                        }
                    }
                }
                EditAction::Insert {
                    kind,
                    before_text,
                    after_text,
                    edge,
                    position,
                    occurrence,
                    ..
                } => {
                    if kind == "paragraph" {
                        v2::Operation::InsertParagraph {
                            target,
                            position: if position.as_deref() == Some("before") {
                                super::Side::Before
                            } else {
                                super::Side::After
                            },
                            text: payload,
                            attached_comment: comment,
                        }
                    } else {
                        v2::Operation::Insert {
                            target,
                            before_text: before_text.clone(),
                            after_text: after_text.clone(),
                            edge: edge.as_ref().map(|s| {
                                if s == "start" {
                                    super::Edge::Start
                                } else {
                                    super::Edge::End
                                }
                            }),
                            occurrence: occurrence.map(|n| n as u64),
                            text: payload,
                            attached_comment: comment,
                        }
                    }
                }
                _ => unreachable!(),
            };
            operations.push(operation);
        }
    }
    let targets = operations
        .iter()
        .map(|op| match op {
            v2::Operation::Replace { target, .. }
            | v2::Operation::Rewrite { target, .. }
            | v2::Operation::Insert { target, .. }
            | v2::Operation::InsertParagraph { target, .. }
            | v2::Operation::Delete { target, .. }
            | v2::Operation::DeleteParagraph { target } => target.clone(),
            _ => unreachable!(),
        })
        .fold(Vec::new(), |mut targets, target| {
            if !targets.contains(&target) {
                targets.push(target);
            }
            targets
        });
    let plan = PlanV2 {
        schema_version: 2,
        source: v2::Source {
            sha256: source_sha256(source),
        },
        policy: v2::Policy {
            track: if inline.track == "on" {
                v2::Track::On
            } else {
                v2::Track::Off
            },
            existing_revisions: existing,
            text_view: v2::TextView::Final,
        },
        resolve_revisions: v2::RevisionDecisions { accept, reject },
        removed_anchor_comments: Default::default(),
        rewrite_others_comments: None,
        author: inline.author.clone(),
        date: inline
            .datetime
            .clone()
            .unwrap_or_else(crate::convert::utc_now_iso8601),
        scope: v2::Scope::Selected { targets },
        operations,
        source_context: None,
        coverage: None,
    };
    plan.validate().map_err(edit_error)?;
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command};
    use crate::edit::{PrepareOptions, prepare_v2};
    use crate::markdown::{DocxOptions, markdown_to_docx};

    fn source(text: &str) -> Vec<u8> {
        markdown_to_docx(text, &DocxOptions::default())
            .unwrap()
            .docx
    }
    fn action(args: &[&str]) -> EditAction {
        let cli = Cli::try_parse_from(
            [
                vec!["jubarte", "edit"],
                args.to_vec(),
                vec![
                    "source.docx",
                    "--dry-run",
                    "--datetime",
                    "2026-10-17T00:00:00Z",
                ],
            ]
            .concat(),
        )
        .unwrap();
        let Some(Command::TypedEdit(typed)) = cli.command else {
            panic!("typed route")
        };
        typed.action
    }
    fn final_text(source: &[u8], action: &EditAction, text: Option<&str>) -> String {
        let plan = compile(source, action, text).unwrap();
        let prepared = prepare_v2(source, &plan, &PrepareOptions::default()).unwrap();
        crate::inspect::paragraphs(&prepared.primary)
            .unwrap()
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn inline_literal_candidates_include_overlaps_and_freeze_all_matches() {
        let bytes = source("aaa\n\nfee fee\n\nfee");
        let replace = |extra: &[&str]| {
            action(
                &[
                    vec!["replace", "--find", "aa", "--text", "X", "--at", "p0"],
                    extra.to_vec(),
                ]
                .concat(),
            )
        };
        let error = compile(&bytes, &replace(&[]), Some("X")).unwrap_err();
        assert_eq!(error.code, "AMBIGUOUS_ANCHOR");
        assert_eq!(error.matches, Some(2));
        assert_eq!(
            final_text(&bytes, &replace(&["--occurrence", "2"]), Some("X")),
            "aX\nfee fee\nfee"
        );
        let error = compile(
            &bytes,
            &replace(&["--all-matches", "--expect-count", "2"]),
            Some("X"),
        )
        .unwrap_err();
        assert_eq!(error.code, "OVERLAPPING_EDITS");
        let all = action(&[
            "replace",
            "--find",
            "fee",
            "--text",
            "charge",
            "--all-matches",
            "--expect-count",
            "3",
        ]);
        let plan = compile(&bytes, &all, Some("charge")).unwrap();
        assert_eq!(plan.operations.len(), 3);
        assert!(matches!(plan.scope,v2::Scope::Selected {ref targets} if targets.len() == 2));
        assert_eq!(
            final_text(&bytes, &all, Some("charge")),
            "aaa\ncharge charge\ncharge"
        );
        let wrong_count = action(&[
            "replace",
            "--find",
            "fee",
            "--text",
            "x",
            "--all-matches",
            "--expect-count",
            "2",
        ]);
        assert_eq!(
            compile(&bytes, &wrong_count, Some("x")).unwrap_err().code,
            "AMBIGUOUS_ANCHOR"
        );
    }

    #[test]
    fn inline_missing_or_ambiguous_literals_never_choose_another_target() {
        let bytes = source("one\n\nfee fee");
        for (args, code) in [
            (
                vec!["replace", "--find", "missing", "--text", "x"],
                "ANCHOR_NOT_FOUND",
            ),
            (
                vec!["replace", "--find", "fee", "--text", "x"],
                "AMBIGUOUS_ANCHOR",
            ),
            (
                vec![
                    "replace",
                    "--find",
                    "fee",
                    "--occurrence",
                    "3",
                    "--text",
                    "x",
                ],
                // Row 5: the anchor module owns the occurrence error code.
                "AMBIGUOUS_ANCHOR",
            ),
            (
                vec!["replace", "--at", "p0", "--find", "fee", "--text", "x"],
                "ANCHOR_NOT_FOUND",
            ),
            (
                vec![
                    "replace", "--story", "header1", "--find", "fee", "--text", "x",
                ],
                "ANCHOR_NOT_FOUND",
            ),
            (
                vec!["rewrite", "--at", "p99", "--text", "x"],
                "INVALID_TARGET",
            ),
            (vec!["replace", "--find", "", "--text", "x"], "INVALID_EDIT"),
        ] {
            assert_eq!(
                compile(&bytes, &action(&args), Some("x")).unwrap_err().code,
                code,
                "{args:?}"
            );
        }
        let explicit_long = action(&[
            "replace",
            "--at",
            "document.w:p.j:short.p1",
            "--find",
            "fee",
            "--occurrence",
            "2",
            "--text",
            "x",
        ]);
        assert_eq!(final_text(&bytes, &explicit_long, Some("x")), "one\nfee x");
    }

    #[test]
    fn inline_insert_edges_and_paragraph_sides_use_explicit_kinds() {
        let bytes = source("word");
        for (args, text, want) in [
            (
                vec![
                    "insert", "--kind", "text", "--at", "p0", "--edge", "start", "--text", "A",
                ],
                "A",
                "Aword",
            ),
            (
                vec![
                    "insert", "--kind", "text", "--at", "p0", "--edge", "end", "--text", "Z",
                ],
                "Z",
                "wordZ",
            ),
            (
                vec![
                    "insert",
                    "--kind",
                    "text",
                    "--at",
                    "p0",
                    "--before-text",
                    "word",
                    "--text",
                    "A",
                ],
                "A",
                "Aword",
            ),
            (
                vec![
                    "insert",
                    "--kind",
                    "paragraph",
                    "--at",
                    "p0",
                    "--position",
                    "before",
                    "--text",
                    "A",
                ],
                "A",
                "A\nword",
            ),
            (vec!["rewrite", "--at", "p0", "--text", ""], "", ""),
        ] {
            assert_eq!(
                final_text(&bytes, &action(&args), Some(text)),
                want,
                "{args:?}"
            );
        }
        let rewrite = action(&[
            "rewrite",
            "--at",
            "p0",
            "--text",
            "new",
            "--comment",
            "Review",
        ]);
        let prepared = prepare_v2(
            &bytes,
            &compile(&bytes, &rewrite, Some("new")).unwrap(),
            &PrepareOptions::default(),
        )
        .unwrap();
        assert_eq!(
            crate::comments::list_comments(&prepared.primary).unwrap()[0].text,
            "Review"
        );
    }

    #[test]
    fn inline_decisions_freeze_original_ids_and_working_rewrite_hash() {
        let bytes = source("Pay {++45++} days.\n\nOther {++Unrelated++} text.");
        let changes = crate::changes::list_changes(&bytes).unwrap();
        assert_eq!(changes.len(), 2);
        let id = changes[0].id.as_str();
        let rewrite = action(&[
            "rewrite",
            "--at",
            "p0",
            "--text",
            "Pay 60 days.",
            "--accept-change",
            id,
            "--accept-change",
            id,
        ]);
        let plan = compile(&bytes, &rewrite, Some("Pay 60 days.")).unwrap();
        assert_eq!(plan.resolve_revisions.accept.len(), 1);
        let long = plan.resolve_revisions.accept[0].long_id.as_str();
        let from_long = action(&[
            "rewrite",
            "--at",
            "p0",
            "--text",
            "Pay 60 days.",
            "--accept-change",
            long,
        ]);
        assert_eq!(
            compile(&bytes, &from_long, Some("Pay 60 days.")).unwrap(),
            plan
        );
        let prepared = prepare_v2(&bytes, &plan, &PrepareOptions::default()).unwrap();
        assert!(
            crate::changes::list_changes(&prepared.primary)
                .unwrap()
                .iter()
                .any(|c| c.id == changes[1].id)
        );
        let reject = action(&[
            "rewrite",
            "--at",
            "p0",
            "--text",
            "Pay 60 days.",
            "--reject-change",
            id,
        ]);
        let rejected = compile(&bytes, &reject, Some("Pay 60 days.")).unwrap();
        assert_eq!(rejected.resolve_revisions.reject.len(), 1);
        let v2::Operation::Rewrite {
            before_sha256: before,
            ..
        } = &rejected.operations[0]
        else {
            panic!("rewrite")
        };
        assert_eq!(before, &source_sha256(b"Pay  days."));
        let missing = action(&[
            "rewrite",
            "--at",
            "p0",
            "--text",
            "x",
            "--accept-change",
            "r999",
        ]);
        assert_eq!(
            compile(&bytes, &missing, Some("x")).unwrap_err().code,
            "INVALID_TARGET"
        );
        let invalid = action(&[
            "rewrite",
            "--at",
            "p0",
            "--text",
            "x",
            "--accept-change",
            "bogus",
        ]);
        assert_eq!(
            compile(&bytes, &invalid, Some("x")).unwrap_err().code,
            "INVALID_ID"
        );
    }
}
