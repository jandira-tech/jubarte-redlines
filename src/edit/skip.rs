// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `--skip-failed` (#1253): every operation that passes is applied and the
//! failed ones are left out, each with why, in a plan of their own.
//!
//! The survivors are applied as a plan of their own, so the output is the
//! bytes that plan gives: `check_conflicts` already keeps the operations
//! of one plan independent, and `resolve_revisions` settles first, as one
//! unit. A failure no operation is blamed for (`STALE_SOURCE`, a
//! `resolve_revisions` error, a plan-level refusal) is never skipped.

use super::prepared::{PrepareOptions, PreparedEdit, prepare};
use super::{EditError, EditOutcome, EditPlan, ExistingRevisions, Operation, kind_name};
use std::collections::BTreeSet;

/// The outcome code of an operation skipped because another operation of
/// its group failed.
const GROUP_FAILED: &str = "GROUP_FAILED";

/// An operation `--skip-failed` left out, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkippedOperation {
    /// Its index in the plan.
    pub index: usize,
    /// The operation, its id filled in (`op-N` when the plan gave none).
    pub operation: Operation,
    /// The failure's code; `GROUP_FAILED` for an operation skipped with its
    /// group.
    pub code: String,
    /// The failure's message.
    pub message: String,
}

/// `plan` with every failed operation, and every operation of a failed
/// operation's group, left out. A plan whose every operation fails is
/// refused with the first failure, every operation's outcome attached.
pub(super) fn prepare_skipping(
    source: &[u8],
    plan: &EditPlan,
    options: &PrepareOptions,
) -> Result<PreparedEdit, EditError> {
    let mut named = plan.clone();
    for (i, op) in named.operations.iter_mut().enumerate() {
        op.id.get_or_insert_with(|| format!("op-{}", i + 1));
    }
    let once = PrepareOptions {
        skip_failed: false,
        ..*options
    };
    let mut live: Vec<usize> = (0..named.operations.len()).collect();
    let mut skipped: Vec<(SkippedOperation, EditOutcome)> = Vec::new();
    loop {
        let attempt = EditPlan {
            operations: live.iter().map(|&i| named.operations[i].clone()).collect(),
            ..named.clone()
        };
        let mut e = match prepare(source, &attempt, &once) {
            Ok(prepared) => return Ok(merged(prepared, skipped)),
            Err(e) => e,
        };
        let Some(failed) = blamed(&e, &named, &live) else {
            return Err(e);
        };
        let groups: BTreeSet<&str> = failed
            .iter()
            .filter_map(|&i| named.operations[i].group.as_deref())
            .collect();
        let mates: Vec<usize> = live
            .iter()
            .copied()
            .filter(|i| !failed.contains(i))
            .filter(|&i| {
                named.operations[i]
                    .group
                    .as_deref()
                    .is_some_and(|g| groups.contains(g))
            })
            .collect();
        for &i in &failed {
            let id = op_id(&named, i);
            let outcome = outcome_of(&e, &named, i);
            let (code, message) = (
                outcome.code.clone().unwrap_or_else(|| e.code.clone()),
                outcome.message.clone().unwrap_or_else(|| e.message.clone()),
            );
            skipped.push((
                SkippedOperation {
                    index: i,
                    operation: named.operations[i].clone(),
                    code: code.clone(),
                    message: message.clone(),
                },
                EditOutcome {
                    id,
                    status: "skipped".into(),
                    code: Some(code),
                    message: Some(message),
                    context: None,
                    comment_id: None,
                    ..outcome
                },
            ));
        }
        for &i in &mates {
            let group = named.operations[i].group.clone().unwrap_or_default();
            let first = failed
                .iter()
                .find(|&&f| named.operations[f].group.as_deref() == Some(group.as_str()))
                .map(|&f| op_id(&named, f))
                .unwrap_or_default();
            let message = format!("skipped with group {group:?}: {first} failed");
            skipped.push((
                SkippedOperation {
                    index: i,
                    operation: named.operations[i].clone(),
                    code: GROUP_FAILED.into(),
                    message: message.clone(),
                },
                EditOutcome {
                    id: op_id(&named, i),
                    status: "skipped".into(),
                    code: Some(GROUP_FAILED.into()),
                    message: Some(message),
                    context: None,
                    comment_id: None,
                    ..outcome_of(&e, &named, i)
                },
            ));
        }
        live.retain(|i| !failed.contains(i) && !mates.contains(i));
        if live.is_empty() {
            skipped.sort_by_key(|(s, _)| s.index);
            // Refused whole, as without --skip-failed: the first failure,
            // and every operation's outcome.
            if let Some((first, _)) = skipped.iter().find(|(s, _)| s.code != GROUP_FAILED) {
                e.code.clone_from(&first.code);
                e.message.clone_from(&first.message);
                e.operation = first.operation.id.clone();
            }
            e.outcomes = skipped
                .into_iter()
                .map(|(s, outcome)| EditOutcome {
                    status: if s.code == GROUP_FAILED {
                        "skipped".into()
                    } else {
                        "failed".into()
                    },
                    ..outcome
                })
                .collect();
            return Err(e);
        }
    }
}

/// The id the report gives operation `i`: its own, or `op-N`.
fn op_id(plan: &EditPlan, i: usize) -> String {
    plan.operations[i]
        .id
        .clone()
        .unwrap_or_else(|| format!("op-{}", i + 1))
}

/// The plan indices among `live` that `e` blames: every operation it
/// reports failed, or the one it names. `None` when a failure is the
/// plan's own (no operation named), names an operation the plan does not
/// hold (one `resolve_revisions.reject.visible` added), or cannot be told
/// apart from another (an id used twice), or is `STALE_SOURCE`.
fn blamed(e: &EditError, plan: &EditPlan, live: &[usize]) -> Option<Vec<usize>> {
    if e.code == "STALE_SOURCE" {
        return None;
    }
    let mut ids: Vec<&str> = e
        .outcomes
        .iter()
        .filter(|o| o.status == "failed")
        .map(|o| o.id.as_str())
        .collect();
    if ids.is_empty() {
        ids.push(e.operation.as_deref()?);
    }
    let mut failed = Vec::new();
    for id in ids {
        let mut holders = live
            .iter()
            .copied()
            .filter(|&i| plan.operations[i].id.as_deref() == Some(id));
        let (Some(i), None) = (holders.next(), holders.next()) else {
            return None;
        };
        if !failed.contains(&i) {
            failed.push(i);
        }
    }
    Some(failed)
}

/// Operation `i`'s outcome in `e`, or a bare one when `e` reports none.
fn outcome_of(e: &EditError, plan: &EditPlan, i: usize) -> EditOutcome {
    let id = op_id(plan, i);
    e.outcomes
        .iter()
        .find(|o| o.id == id)
        .cloned()
        .unwrap_or_else(|| EditOutcome {
            id,
            kind: kind_name(&plan.operations[i].kind).to_string(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            anchor_given: None,
            anchor_read_as: None,
            code: None,
            message: None,
        })
}

/// `prepared` with the skipped operations' outcomes back in plan order.
fn merged(
    mut prepared: PreparedEdit,
    mut skipped: Vec<(SkippedOperation, EditOutcome)>,
) -> PreparedEdit {
    skipped.sort_by_key(|(s, _)| s.index);
    let report = &mut prepared.result.report;
    let mut landed = std::mem::take(&mut report.operations).into_iter();
    let total = landed.len() + skipped.len();
    let mut skips = skipped.iter().peekable();
    for i in 0..total {
        match skips.peek() {
            Some((s, outcome)) if s.index == i => {
                report.operations.push(outcome.clone());
                skips.next();
            }
            _ => report.operations.extend(landed.next()),
        }
    }
    report.skipped = skipped.len();
    report.ok &= skipped.is_empty();
    prepared.skipped = skipped.into_iter().map(|(s, _)| s).collect();
    prepared
}

/// The plan of `skipped`, to fix and run on the document written: the
/// plan's author, the date the run used, `existing_revisions` as given
/// (`keep` when the output is a redline, whose tracked changes are the
/// plan's own), `source_sha256` of that document, no `resolve_revisions`
/// (already settled), and on each operation a `why` the parser ignores.
pub fn failed_plan_json(
    plan: &EditPlan,
    skipped: &[SkippedOperation],
    date: &str,
    output_sha256: &str,
    existing_revisions: ExistingRevisions,
) -> String {
    let failed = EditPlan {
        source_sha256: Some(output_sha256.to_string()),
        date: Some(date.to_string()),
        resolve_revisions: None,
        existing_revisions,
        // update_fields cannot go with keep (check_update_fields).
        update_fields: plan.update_fields && existing_revisions != ExistingRevisions::Keep,
        operations: skipped.iter().map(|s| s.operation.clone()).collect(),
        ..plan.clone()
    };
    let mut value = serde_json::to_value(&failed).expect("plan serializes");
    if let Some(ops) = value["operations"].as_array_mut() {
        for (op, s) in ops.iter_mut().zip(skipped) {
            op["why"] = format!("{}: {}", s.code, s.message).into();
        }
    }
    let mut json = serde_json::to_string_pretty(&value).expect("plan serializes");
    json.push('\n');
    json
}
