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

use super::origin::{Paragraphs, SOURCE_ONLY, written_places};
use super::prepared::{Blame, PrepareOptions, PreparedEdit, prepare, prepare_blaming};
use super::{
    ControlSelector, EditError, EditOutcome, EditPlan, ExistingRevisions, Operation, OperationKind,
    Selector, kind_name,
};
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
    // A visible reject is lowered once, here: its strikes and restores
    // follow the plan's operations in every attempt, under the names the
    // whole plan gives them, so a skip never renames one after an
    // operation still live.
    let mine = plan.operations.len();
    let mut named = super::visible::lower(source, plan)?.unwrap_or_else(|| plan.clone());
    for (i, op) in named.operations.iter_mut().enumerate() {
        op.id.get_or_insert_with(|| format!("op-{}", i + 1));
    }
    let once = PrepareOptions {
        skip_failed: false,
        ..*options
    };
    let mut live: Vec<usize> = (0..mine).collect();
    let mut skipped: Vec<(SkippedOperation, EditOutcome)> = Vec::new();
    loop {
        let attempt = EditPlan {
            operations: live
                .iter()
                .copied()
                .chain(mine..named.operations.len())
                .map(|i| named.operations[i].clone())
                .collect(),
            ..named.clone()
        };
        // A candidate refusal blames every operation behind one of its
        // defects at once, each with its own error; any other failure, the
        // operations it reports failed or names.
        let blame = match prepare_blaming(source, &attempt, live.len(), &once) {
            Ok(prepared) => return Ok(merged(prepared, skipped)),
            Err(blame) => *blame,
        };
        let (mut e, failed, reasons) = match blame {
            Blame::Plain(e) => {
                let Some(failed) = blamed(&e, &named, &live, mine) else {
                    return Err(e);
                };
                (e, failed, Vec::new())
            }
            Blame::Candidate(e, blamed) => {
                if blamed.is_empty() {
                    return Err(e);
                }
                let reasons: Vec<(usize, EditError)> =
                    blamed.into_iter().map(|(i, why)| (live[i], why)).collect();
                (e, reasons.iter().map(|(i, _)| *i).collect(), reasons)
            }
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
            let (code, message) = match reasons.iter().find(|(r, _)| *r == i) {
                Some((_, why)) => (why.code.clone(), why.message.clone()),
                None => (
                    outcome.code.clone().unwrap_or_else(|| e.code.clone()),
                    outcome.message.clone().unwrap_or_else(|| e.message.clone()),
                ),
            };
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
/// hold, or one from `mine` on (a strike or restore that
/// `resolve_revisions.reject.visible` added), or cannot be told apart from
/// another (an id used twice), or is `STALE_SOURCE`.
fn blamed(e: &EditError, plan: &EditPlan, live: &[usize], mine: usize) -> Option<Vec<usize>> {
    if e.code == "STALE_SOURCE" {
        return None;
    }
    let added = |id: &str| {
        plan.operations[mine..]
            .iter()
            .any(|op| op.id.as_deref() == Some(id))
    };
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
        if added(id) {
            return None;
        }
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
            near: None,
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

/// The plan of `skipped`, to fix and run on the document written (the
/// redline when `redline`, else the clean copy): the plan's author, the
/// date the run used, `existing_revisions` as given (`keep` when the output
/// is a redline, whose tracked changes are the plan's own), `source_sha256`
/// of that document, no `resolve_revisions` (already settled), and on each
/// operation a `why` the parser ignores. A positional paragraph (`p3`,
/// `{"index": 3}`) the operations that landed moved is renamed to its id
/// in that document, and `why` says so, or says it has none there.
pub fn failed_plan_json(
    source: &[u8],
    plan: &EditPlan,
    skipped: &[SkippedOperation],
    date: &str,
    output_sha256: &str,
    existing_revisions: ExistingRevisions,
    redline: bool,
) -> String {
    let mut operations: Vec<Operation> = skipped.iter().map(|s| s.operation.clone()).collect();
    let notes = rename_positions(source, plan, skipped, redline, &mut operations);
    let failed = EditPlan {
        source_sha256: Some(output_sha256.to_string()),
        date: Some(date.to_string()),
        resolve_revisions: None,
        existing_revisions,
        // update_fields cannot go with keep (check_update_fields).
        update_fields: plan.update_fields && existing_revisions != ExistingRevisions::Keep,
        operations,
        ..plan.clone()
    };
    let mut value = serde_json::to_value(&failed).expect("plan serializes");
    if let Some(ops) = value["operations"].as_array_mut() {
        for ((op, s), notes) in ops.iter_mut().zip(skipped).zip(&notes) {
            let keep = keep_note(kind_name(&s.operation.kind), existing_revisions);
            let mut why = format!("{}: {}", s.code, s.message);
            for note in notes.iter().chain(keep.as_ref()) {
                why.push_str("; ");
                why.push_str(note);
            }
            op["why"] = why.into();
        }
    }
    let mut json = serde_json::to_string_pretty(&value).expect("plan serializes");
    json.push('\n');
    json
}

/// What the plan of the fixes cannot do once per kind, in plan order:
/// under `existing_revisions` keep (the redline's tracked changes kept), a
/// skipped operation of a kind keep refuses (`x-jubarte-keep` false).
pub fn keep_notes(
    skipped: &[SkippedOperation],
    existing_revisions: ExistingRevisions,
) -> Vec<String> {
    let mut notes = Vec::new();
    for s in skipped {
        if let Some(note) = keep_note(kind_name(&s.operation.kind), existing_revisions)
            && !notes.contains(&note)
        {
            notes.push(note);
        }
    }
    notes
}

fn keep_note(kind: &str, existing_revisions: ExistingRevisions) -> Option<String> {
    (existing_revisions == ExistingRevisions::Keep && !super::operation_supports_keep(kind)).then(
        || {
            format!(
                "{kind} cannot run under existing_revisions keep: fix it in the original plan and run that on the source"
            )
        },
    )
}

/// What a positional name in a skipped operation points at.
enum Named {
    /// Its `n`th paragraph selector.
    Paragraph(usize),
    /// Its `fill_control` control, `PARA.ccK`: K.
    Control(String),
}

/// Name what `what` points at in `operation` as `source:SHOWN`, which no
/// document resolves: the plan of the fixes cannot run it on whatever
/// paragraph or control now has its id until it is given again.
fn source_only(operation: &mut Operation, what: &Named, shown: &str) {
    let id = format!("{SOURCE_ONLY}{shown}");
    match what {
        Named::Paragraph(n) => {
            let selector = &mut *super::selectors_mut(&mut operation.kind).swap_remove(*n);
            *selector = match selector {
                Selector::Id { .. } => Selector::Id { id },
                _ => Selector::Name(id),
            };
        }
        Named::Control(_) => {
            if let OperationKind::FillControl { control, .. } = &mut operation.kind {
                *control = match control {
                    ControlSelector::Id { .. } => ControlSelector::Id { id },
                    _ => ControlSelector::Name(id),
                };
            }
        }
    }
}

/// A `fill_control` control named through its paragraph, `PARA.ccK`
/// (`p3.cc0`, `t0.row1.cell2.cc0`): that paragraph's spelling and K.
fn control_paragraph(control: &ControlSelector) -> Option<(&str, &str)> {
    let (ControlSelector::Name(id) | ControlSelector::Id { id }) = control else {
        return None;
    };
    let (paragraph, k) = id.rsplit_once(".cc")?;
    (!paragraph.is_empty() && k.parse::<usize>().is_ok()).then_some((paragraph, k))
}

/// Rename each positional paragraph of `operations` (the skipped ones'),
/// and each control named through one, to its id in the document the
/// plan's other operations write, read off a run of them on `source` with
/// its paragraphs tagged; the notes, per operation, of what moved and what
/// has no one-to-one paragraph there, each naming the paragraph as the
/// plan spelled it. Text selectors, and ids `source` does not have, stay
/// as given.
fn rename_positions(
    source: &[u8],
    plan: &EditPlan,
    skipped: &[SkippedOperation],
    redline: bool,
    operations: &mut [Operation],
) -> Vec<Vec<String>> {
    let mut notes = vec![Vec::new(); operations.len()];
    let Ok(read) = Paragraphs::open(source) else {
        return notes;
    };
    let ids = read.ids();
    // (operation, what names it, source paragraph, the plan's spelling) of
    // each positional paragraph.
    let mut named = Vec::new();
    for (o, op) in operations.iter_mut().enumerate() {
        if let OperationKind::FillControl { control, .. } = &op.kind
            && let Some((paragraph, k)) = control_paragraph(control)
            && let Ok(Ok(g)) = ids.position(&Selector::Name(paragraph.to_string()))
        {
            named.push((o, Named::Control(k.to_string()), g, paragraph.to_string()));
        }
        for (n, selector) in super::selectors_mut(&mut op.kind).into_iter().enumerate() {
            if let Ok(Ok(g)) = ids.position(selector) {
                let spelled = match selector {
                    Selector::Name(id) | Selector::Id { id } => id.clone(),
                    _ => ids.id(g),
                };
                named.push((o, Named::Paragraph(n), g, spelled));
            }
        }
    }
    if named.is_empty() {
        return notes;
    }
    let left_out: BTreeSet<usize> = skipped.iter().map(|s| s.index).collect();
    let survivors = EditPlan {
        source_sha256: None,
        operations: plan
            .operations
            .iter()
            .enumerate()
            .filter(|(i, _)| !left_out.contains(i))
            .map(|(_, op)| op.clone())
            .collect(),
        ..plan.clone()
    };
    let landed = written_places(source, |tagged| {
        let prepared = prepare(tagged, &survivors, &PrepareOptions::default())?;
        Ok(if redline {
            prepared.result.redline
        } else {
            prepared.result.clean
        })
    });
    for (o, what, g, spelled) in named {
        let shown = match &what {
            Named::Paragraph(_) => spelled,
            Named::Control(k) => format!("{spelled}.cc{k}"),
        };
        let Ok((places, written)) = &landed else {
            source_only(&mut operations[o], &what, &shown);
            notes[o].push(format!(
                "{shown} names a paragraph of the source: give it again from the output"
            ));
            continue;
        };
        let Some(post) = places[g].filter(|&post| written.place(post).0 == read.place(g).0) else {
            source_only(&mut operations[o], &what, &shown);
            notes[o].push(format!(
                "{shown} has no one-to-one paragraph in the output: give it again from the output"
            ));
            continue;
        };
        // Its `ccK` counts the paragraph's controls: one that gained or lost
        // some (a merge) numbers them anew.
        if matches!(what, Named::Control(_)) && written.control_count(post) != read.control_count(g)
        {
            source_only(&mut operations[o], &what, &shown);
            notes[o].push(format!(
                "{shown} has no one-to-one control in the output: give it again from the output"
            ));
            continue;
        }
        let to = written.ids().id(post);
        if to == ids.id(g) {
            continue;
        }
        let renamed = match what {
            Named::Paragraph(n) => {
                let selector = &mut *super::selectors_mut(&mut operations[o].kind).swap_remove(n);
                *selector = match selector {
                    Selector::Index { story, .. } => Selector::Index {
                        index: written.place(post).1,
                        story: story.clone(),
                    },
                    Selector::Id { .. } => Selector::Id { id: to.clone() },
                    _ => Selector::Name(to.clone()),
                };
                to
            }
            Named::Control(k) => {
                let OperationKind::FillControl { control, .. } = &mut operations[o].kind else {
                    continue;
                };
                let id = format!("{to}.cc{k}");
                *control = match control {
                    ControlSelector::Id { .. } => ControlSelector::Id { id: id.clone() },
                    _ => ControlSelector::Name(id.clone()),
                };
                id
            }
        };
        notes[o].push(format!("{shown} is {renamed} in the output"));
    }
    notes
}
