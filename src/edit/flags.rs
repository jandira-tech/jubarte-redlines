// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! `edit` and `add` by flags: each `-p WHERE` with its `--anchor`,
//! `--content`, `--delete`, `--resolve`, `--reply`, `--before`, `--comment`
//! and `--style` becomes one operation of an [`EditPlan`]; a `-p r12` with
//! `--accept` or `--reject` joins the plan's `resolve_revisions`, which runs
//! before the operations. The binary, the Python CLI and the npm CLI all
//! build their plan here.

use serde::{Deserialize, Serialize};

use super::{
    EditPlan, ExistingRevisions, HalfPoints, Operation, OperationKind, ResolveRevisions, RunFormat,
    RunSpec, Selector, Side,
};
use crate::changes::ChangeFilter;
use crate::ids::{IdError, Location, ShortId};

/// One operation as the flags describe it, grouped by its `-p`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FlagOp {
    /// The `-p` value: `p12`, `header1`, `t0.row1.cell2`, `c5`, or a long id.
    pub at: String,
    /// `--anchor`: text inside `at`.
    pub anchor: Option<String>,
    /// `--occurrence`: 1-based match of `anchor` in this group.
    pub occurrence: Option<usize>,
    /// `--content`: new text.
    pub content: Option<String>,
    /// `--delete`.
    pub delete: bool,
    /// `--resolve` (`edit` only).
    pub resolve: bool,
    /// `--accept` (`edit` only): accept the revision `at` names.
    pub accept: bool,
    /// `--reject` (`edit` only): reject the revision `at` names.
    pub reject: bool,
    /// `--strike` (`edit` only): strike another author's insertion, as our
    /// tracked deletion inside it.
    pub strike: bool,
    /// `--restore` (`edit` only): restore another author's deletion, as our
    /// tracked insertion of its words after it.
    pub restore: bool,
    /// `--visible` (with `--reject`): a strike or a restore, by the kind of
    /// each revision `at` names.
    pub visible: bool,
    /// `--reply`: a reply to the comment `at` names.
    pub reply: Option<String>,
    /// `--before` (`add` only).
    pub before: bool,
    /// `--comment` (`add` only).
    pub comment: bool,
    /// `--style` values, in order.
    pub styles: Vec<String>,
}

/// Which command the flags came with: `edit` changes what is there, `add`
/// makes something new.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verb {
    /// `jubarte edit`.
    Edit,
    /// `jubarte add`.
    Add,
}

/// A plan built from flags, and the notes to print before the view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlagPlan {
    /// The plan to apply.
    pub plan: EditPlan,
    /// `op-N: …` notes (content that keeps Markdown marks as text).
    pub notes: Vec<String>,
}

/// The plan `ops` describe. `existing` is `--existing-revisions`; `None` is
/// `auto`: `keep` when `source` already holds tracked changes, else the
/// comparer path.
///
/// # Errors
///
/// The usage message when no operation is given, a flag combination fits no
/// operation, a `--style` token is unknown, or a revision group is malformed.
pub fn plan_from_flags(
    verb: Verb,
    ops: &[FlagOp],
    author: &str,
    date: Option<&str>,
    existing: Option<ExistingRevisions>,
    source: &[u8],
) -> Result<FlagPlan, String> {
    if ops.is_empty() {
        return Err("no operation: give -p WHERE with its flags, or --plan".into());
    }
    let mut operations = Vec::with_capacity(ops.len());
    let mut notes = Vec::new();
    let (mut accept, mut reject) = (Vec::new(), Vec::new());
    let mut listed: Option<Vec<crate::changes::Change>> = None;
    for op in ops {
        if op.occurrence.is_some() && op.anchor.is_none() {
            return Err(format!("-p {}: --occurrence requires --anchor", op.at));
        }
        if op.occurrence == Some(0) {
            return Err(format!("-p {}: --occurrence is 1-based", op.at));
        }
        match revision_group(verb, op)? {
            Group::Settle { accept: true, ids } => accept.extend(ids),
            Group::Settle { accept: false, ids } => reject.extend(ids),
            Group::Answer { how, ids, comment } => {
                let revision = || crate::changes::composite_id(&ids);
                let kinds = match how {
                    Answer::Strike => vec![OperationKind::Strike {
                        revision: revision(),
                        comment,
                    }],
                    Answer::Restore => vec![OperationKind::Restore {
                        revision: revision(),
                        comment,
                    }],
                    Answer::ByKind => {
                        let listed = listed.get_or_insert_with(|| {
                            crate::changes::list_changes(source).unwrap_or_default()
                        });
                        super::visible::reject_kinds(listed, &ids, comment)
                    }
                };
                operations.extend(kinds.into_iter().map(|kind| Operation {
                    id: None,
                    group: None,
                    kind,
                }));
            }
            Group::Operation => {
                let (kind, note) = flag_operation(verb, op)?;
                operations.push(Operation {
                    id: None,
                    group: None,
                    kind,
                });
                notes.extend(note.map(|n| format!("op-{}: {n}", operations.len())));
            }
        }
    }
    let side = |ids: Vec<String>| (!ids.is_empty()).then(|| ChangeFilter::ids(ids));
    let resolve_revisions = (!accept.is_empty() || !reject.is_empty()).then(|| ResolveRevisions {
        accept: side(accept),
        reject: side(reject),
        reject_visible: false,
    });
    let existing = existing.unwrap_or_else(|| {
        if crate::changes::list_changes(source).is_ok_and(|c| !c.is_empty()) {
            ExistingRevisions::Keep
        } else {
            ExistingRevisions::Refuse
        }
    });
    Ok(FlagPlan {
        plan: EditPlan {
            schema_version: crate::inspect::SCHEMA_VERSION,
            source_sha256: None,
            author: author.to_string(),
            date: date.map(str::to_string),
            initials: None,
            resolve_revisions,
            removed_anchor_comments: Default::default(),
            existing_revisions: existing,
            operations,
            update_fields: false,
            rewrite_others_comments: false,
        },
        notes,
    })
}

/// `--style` tokens: run formatting, and at most one paragraph style.
fn styles(tokens: &[String]) -> Result<(Option<RunFormat>, Option<String>), String> {
    let mut format = RunFormat::default();
    let mut touched = false;
    let mut paragraph: Option<String> = None;
    for token in tokens {
        let (key, value) = token
            .split_once('=')
            .map_or((token.as_str(), None), |(k, v)| (k, Some(v)));
        match (key, value) {
            ("bold", None) => format.bold = Some(true),
            ("italic", None) => format.italic = Some(true),
            ("underline", None) => format.underline = Some(true),
            ("strike", None) => format.strike = Some(true),
            ("caps", None) => format.caps = Some(true),
            ("highlight", Some(v)) => format.highlight = Some(v.to_string()),
            ("font", Some(v)) => format.font = Some(v.to_string()),
            ("color", Some(v)) => format.color = Some(v.to_string()),
            ("size", Some(v)) => {
                let points: f64 = v
                    .parse()
                    .ok()
                    .filter(|p: &f64| p.is_finite() && *p > 0.0 && *p <= 1638.0)
                    .ok_or_else(|| format!("--style size={v}: points between 0 and 1638"))?;
                let half = (points * 2.0).round();
                // In range: at most 3276 half-points.
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "checked above: 0 < half <= 3276"
                )]
                let half = half as u32;
                format.size_pt = Some(HalfPoints(half));
            }
            (_, Some(_)) => return Err(format!("--style {token}: unknown key")),
            ("", None) => return Err("--style needs a value".into()),
            (name, None) => {
                if paragraph.replace(name.to_string()).is_some() {
                    return Err("--style: one paragraph style per operation".into());
                }
                continue;
            }
        }
        touched = true;
    }
    Ok((touched.then_some(format), paragraph))
}

/// `--content` as document text: Markdown escapes resolved (`\#` → `#`); a
/// note when the view would read marks in it (emphasis, CriticMarkup, a
/// block mark), since they are written as text.
fn content_text(content: &str) -> (String, Option<&'static str>) {
    let text = crate::markdown::unescape_markdown(content);
    // The anchor drops a line's leading spaces (the view's indent); they are
    // not a mark.
    let marks = crate::markdown::plain_anchor(content).trim_start() != text.trim_start();
    (
        text,
        marks.then_some("content keeps its Markdown marks as text; use --style for formatting"),
    )
}

/// `c5` → 5: the comment a `-p` names.
fn comment_number(at: &str) -> Option<u32> {
    match crate::ids::parse_location(at) {
        Ok(crate::ids::Location::Comment(id)) => Some(id),
        _ => None,
    }
}

/// The refusal of a `-p` id this group cannot take: an expired one, named
/// with the settled spellings `keep` allows (`-p 5 --reject` → `r5`), or a
/// settled id `-p` does not edit yet (`s0`, `p3-p7`). `None` when `at`
/// parses, or when no settled spelling is known (the caller's own message
/// is then the better one).
fn expired(at: &str, keep: impl Fn(&ShortId) -> bool) -> Option<String> {
    match crate::ids::parse_location(at) {
        Err(e) if e.expected == crate::ids::LOCATION_NOT_YET => Some(format!("-p {e}")),
        Err(e) if e.hint.is_some() => {
            let hint = crate::ids::hint_where(e.hint.clone(), keep);
            Some(format!("-p {}", IdError { hint, ..e }))
        }
        _ => None,
    }
}

/// What a `-p` group does with the revisions it names.
enum Group {
    /// Not a revision group: an operation.
    Operation,
    /// `--accept` or `--reject`: the long ids join `resolve_revisions`.
    Settle { accept: bool, ids: Vec<String> },
    /// `--strike`, `--restore` or `--reject --visible`: one operation by
    /// our author on the change the marks `ids` make (a visible reject:
    /// one per kind), with `--content` as its comment.
    Answer {
        how: Answer,
        ids: Vec<String>,
        comment: Option<String>,
    },
}

/// Which visible reject a group asks for.
#[derive(Clone, Copy)]
enum Answer {
    Strike,
    Restore,
    /// `--reject --visible`: by each revision's kind.
    ByKind,
}

/// What the `-p r12` group `op` asks of its revisions. Only `edit`
/// resolves revisions.
fn revision_group(verb: Verb, op: &FlagOp) -> Result<Group, String> {
    let at = op.at.trim();
    let verbs: Vec<&str> = [
        ("accept", op.accept),
        ("reject", op.reject),
        ("strike", op.strike),
        ("restore", op.restore),
    ]
    .into_iter()
    .filter_map(|(name, set)| set.then_some(name))
    .collect();
    // Only a revision group names a change; any other group's id is
    // refused by [`flag_operation`], with the spellings it takes.
    if !verbs.is_empty()
        && let Some(refused) = expired(at, |id| matches!(id, ShortId::Change { .. }))
    {
        return Err(refused);
    }
    let bad = |m: &str| Err(format!("-p {at}: {m}"));
    let ids = match crate::ids::parse_location(at) {
        Ok(Location::Changes(ids)) => Some(ids),
        _ => None,
    };
    // A `--reply` group is an operation: off a comment it says so. It
    // resolves no revision, so a revision flag beside it is refused, not
    // ignored.
    if op.reply.is_some() {
        if !verbs.is_empty() || op.visible {
            return bad("--reply takes no other flag");
        }
        return Ok(Group::Operation);
    }
    if verb == Verb::Add && ids.is_some() {
        return bad(
            "add resolves no revision; use edit -p r12 --accept, --reject, --strike or --restore",
        );
    }
    if op.visible && !op.reject {
        return bad("--visible goes with --reject");
    }
    let Some(ids) = ids else {
        return match verbs.first() {
            None => Ok(Group::Operation),
            Some(&("accept" | "reject")) => bad("--accept and --reject take a revision (-p r12)"),
            Some(name) => bad(&format!("--{name} takes a revision (-p r12)")),
        };
    };
    if let [first, second, ..] = verbs[..] {
        return bad(&format!("--{first} and --{second} exclude each other"));
    }
    let others = op.anchor.is_some()
        || op.delete
        || op.resolve
        || op.before
        || op.comment
        || !op.styles.is_empty();
    if others || verbs.is_empty() {
        return bad("a revision takes --accept, --reject, --strike or --restore");
    }
    let how = if op.strike {
        Some(Answer::Strike)
    } else if op.restore {
        Some(Answer::Restore)
    } else {
        op.visible.then_some(Answer::ByKind)
    };
    if op.content.is_some() && how.is_none() {
        return bad(
            "--content on a revision is the comment of --strike, --restore or --reject --visible",
        );
    }
    let ids = ids.iter().map(crate::ids::ChangeId::short).collect();
    Ok(match how {
        Some(how) => Group::Answer {
            how,
            ids,
            comment: op.content.clone(),
        },
        None => Group::Settle {
            accept: op.accept,
            ids,
        },
    })
}

/// The operation one `-p` group stands for, and a note for the report.
fn flag_operation(verb: Verb, op: &FlagOp) -> Result<(OperationKind, Option<String>), String> {
    use OperationKind as K;
    let at = op.at.trim();
    let bad = |m: &str| Err(format!("-p {at}: {m}"));
    if at.is_empty() {
        return Err("-p needs a location".into());
    }
    // The settled spellings this whole group takes: a reply or a
    // resolution names a comment; a comment takes no --anchor, --style,
    // --before or --comment (nor, under add, --delete or --resolve); a
    // change takes --accept or --reject, never these.
    let names_comment = op.reply.is_some() || op.resolve;
    let comment_takes = op.anchor.is_none()
        && op.styles.is_empty()
        && !op.before
        && !op.comment
        && !(verb == Verb::Add && (op.delete || op.resolve));
    let keep = |id: &ShortId| match id {
        ShortId::Comment(_) => comment_takes,
        ShortId::Change { .. } => false,
        _ => !names_comment,
    };
    if let Some(refused) = expired(at, keep) {
        return Err(refused);
    }
    if let Some(reply) = &op.reply {
        let Some(comment_id) = comment_number(at) else {
            return bad("--reply is for a comment (-p c5)");
        };
        if op.anchor.is_some()
            || op.content.is_some()
            || op.delete
            || op.resolve
            || op.before
            || op.comment
            || !op.styles.is_empty()
        {
            return bad("--reply takes no other flag");
        }
        let (text, note) = content_text(reply);
        let note = note.map(|_| {
            "the reply keeps its Markdown marks as text; replies take no formatting".to_string()
        });
        return Ok((K::ReplyComment { comment_id, text }, note));
    }
    if op.delete && (op.content.is_some() || op.resolve) {
        return bad("--delete excludes --content and --resolve");
    }
    if op.resolve && (op.content.is_some() || op.anchor.is_some() || !op.styles.is_empty()) {
        return bad("--resolve takes no --anchor, --content or --style");
    }
    if op.before && (op.anchor.is_some() || op.comment) {
        return bad("--before is for a new paragraph, not a comment");
    }
    let paragraph = || Selector::Name(at.to_string());
    let (run_format, paragraph_style) = styles(&op.styles).map_err(|m| format!("-p {at}: {m}"))?;
    let (content, note) = match &op.content {
        Some(c) => {
            let (text, note) = content_text(c);
            (Some(text), note.map(str::to_string))
        }
        None => (None, None),
    };
    let comment_id = comment_number(at);
    let mut kind = match (verb, comment_id, &op.anchor, content) {
        (Verb::Add, _, _, _) if op.delete || op.resolve => {
            return bad("add takes no --delete or --resolve");
        }
        // Comments.
        (_, Some(_), Some(_), _) => return bad("a comment takes no --anchor"),
        (_, Some(_), _, _) if !op.styles.is_empty() => return bad("a comment takes no --style"),
        (Verb::Add, Some(id), None, Some(text)) if !op.before && !op.comment && !op.delete => {
            K::ReplyComment {
                comment_id: id,
                text,
            }
        }
        (Verb::Add, Some(_), None, _) => return bad("a reply takes --content only"),
        (Verb::Edit, Some(id), None, Some(text)) => K::EditComment {
            comment_id: id,
            text,
        },
        (Verb::Edit, Some(id), None, None) if op.delete => K::DeleteComment { comment_id: id },
        (Verb::Edit, Some(id), None, None) if op.resolve => K::ResolveComment {
            comment_id: id,
            done: true,
        },
        (Verb::Edit, Some(_), None, None) => {
            return bad("a comment takes --content, --delete or --resolve");
        }
        // Add.
        (Verb::Add, None, _, Some(_))
            if (op.anchor.is_some() || op.comment) && !op.styles.is_empty() =>
        {
            return bad("a comment takes no --style");
        }
        (Verb::Add, None, Some(anchor), Some(text)) => K::Comment {
            paragraph: paragraph(),
            find: Some(anchor.clone()),
            text,
            through: None,
            occurrence: None,
        },
        (Verb::Add, None, None, Some(text)) if op.comment => K::Comment {
            paragraph: paragraph(),
            find: None,
            text,
            through: None,
            occurrence: None,
        },
        (Verb::Add, None, None, Some(text)) => {
            let format = run_format.unwrap_or_default();
            // A new paragraph's runs carry these four; the rest would vanish.
            if format.strike.is_some()
                || format.caps.is_some()
                || format.font.is_some()
                || format.size_pt.is_some()
                || format.color.is_some()
            {
                return bad(
                    "a new paragraph takes --style bold, italic, underline, highlight=COLOR or a paragraph style",
                );
            }
            K::InsertParagraph {
                paragraph: paragraph(),
                position: if op.before { Side::Before } else { Side::After },
                runs: vec![RunSpec {
                    text,
                    bold: format.bold,
                    italic: format.italic,
                    underline: format.underline,
                    highlight: format.highlight,
                }],
                like: None,
                style: paragraph_style,
                comment: None,
            }
        }
        (Verb::Add, None, _, None) => return bad("add needs --content"),
        // Edit.
        (Verb::Edit, None, _, _) if op.resolve => {
            return bad("--resolve is for a comment (-p c5)");
        }
        (Verb::Edit, None, _, _) if op.before || op.comment => {
            return bad("--before and --comment belong to add");
        }
        (Verb::Edit, None, Some(anchor), Some(text)) => {
            if paragraph_style.is_some() {
                return bad("a paragraph style takes no --anchor");
            }
            inline_edit(paragraph(), anchor, text, run_format)
                .map_err(|m| format!("-p {at}: {m}"))?
        }
        (Verb::Edit, None, Some(_), None) if op.delete && !op.styles.is_empty() => {
            return bad("--delete takes no --style");
        }
        (Verb::Edit, None, Some(anchor), None) if op.delete => K::Delete {
            paragraph: paragraph(),
            find: anchor.clone(),
            occurrence: None,
            comment: None,
        },
        (Verb::Edit, None, Some(anchor), None) => match (run_format, paragraph_style) {
            (Some(format), None) => K::FormatRun {
                paragraph: paragraph(),
                find: anchor.clone(),
                format,
                occurrence: None,
            },
            (_, Some(_)) => return bad("a paragraph style takes no --anchor"),
            (None, None) => return bad("--anchor needs --content, --delete or --style"),
        },
        (Verb::Edit, None, None, Some(text))
            if paragraph_style.is_none() && run_format.is_none() =>
        {
            K::Rewrite {
                paragraph: paragraph(),
                text,
                comment: None,
            }
        }
        (Verb::Edit, None, None, Some(_)) => {
            return bad(
                "a rewrite takes no --style: format with --anchor, or set the paragraph style in a second -p",
            );
        }
        (Verb::Edit, None, None, None) if op.delete => {
            if !op.styles.is_empty() {
                return bad("--delete takes no --style");
            }
            K::DeleteParagraph {
                paragraph: paragraph(),
                comment: None,
            }
        }
        (Verb::Edit, None, None, None) => match (run_format, paragraph_style) {
            (None, Some(style)) => K::FormatParagraph {
                paragraph: paragraph(),
                style: Some(style),
                alignment: None,
                line_spacing: None,
                space_before: None,
                space_after: None,
            },
            (Some(_), _) => return bad("run formatting needs --anchor (the text to format)"),
            (None, None) => return bad("edit needs --content, --delete, --resolve or --style"),
        },
    };
    match &mut kind {
        K::Replace { occurrence, .. }
        | K::Insert { occurrence, .. }
        | K::Delete { occurrence, .. }
        | K::FormatRun { occurrence, .. }
        | K::Comment { occurrence, .. } => *occurrence = op.occurrence,
        _ => {}
    }
    Ok((kind, note))
}

/// A replacement that keeps the anchor at its start or end is an insertion,
/// so the redline marks only the new words. The anchor is compared as given
/// and without its Markdown marks (`# Fees` → `Fees`), and passed on as
/// given, so the applier still records the normalization.
fn inline_edit(
    paragraph: Selector,
    anchor: &str,
    text: String,
    format: Option<RunFormat>,
) -> Result<OperationKind, String> {
    use OperationKind as K;
    let plain = crate::markdown::plain_anchor(anchor);
    if text == anchor || text == plain {
        // Same words with a style: "make the anchor bold".
        return match format {
            Some(format) => Ok(K::FormatRun {
                paragraph,
                find: anchor.to_string(),
                format,
                occurrence: None,
            }),
            None => Err("--content equals the anchor; nothing to change".into()),
        };
    }
    let after = text.strip_prefix(anchor).or_else(|| {
        text.strip_prefix(plain.as_str())
            .filter(|_| !plain.is_empty())
    });
    if let Some(added) = after.filter(|a| !a.is_empty()) {
        return Ok(K::Insert {
            paragraph,
            after: Some(anchor.to_string()),
            before: None,
            position: None,
            text: added.to_string(),
            format,
            comment: None,
            occurrence: None,
        });
    }
    let before = text.strip_suffix(anchor).or_else(|| {
        text.strip_suffix(plain.as_str())
            .filter(|_| !plain.is_empty())
    });
    if let Some(added) = before.filter(|a| !a.is_empty()) {
        return Ok(K::Insert {
            paragraph,
            after: None,
            before: Some(anchor.to_string()),
            position: None,
            text: added.to_string(),
            format,
            comment: None,
            occurrence: None,
        });
    }
    Ok(K::Replace {
        paragraph,
        find: anchor.to_string(),
        replacement: text,
        format,
        comment: None,
        whole: false,
        occurrence: None,
    })
}

/// Why `--editing-mode` cannot apply `plan` to `source`: under `keep` the
/// tracked changes `plan` does not itself accept or reject (`-p r12
/// --accept`) stay, so the "clean" copy would still carry revisions; and a
/// reply to a comment on a paragraph the plan deletes has no thread left in
/// the clean copy to join.
pub fn editing_mode_conflict(plan: &EditPlan, source: &[u8]) -> Option<String> {
    if plan.existing_revisions == ExistingRevisions::Keep
        && crate::edit::revisions_left(source, plan)
    {
        return Some(
            "--editing-mode needs a document without tracked changes; it has some, so pass --existing-revisions accept or reject".into(),
        );
    }
    let (parent, paragraph) = crate::edit::replies_on_deleted_paragraphs(source, plan)
        .into_iter()
        .next()?;
    let paragraph = paragraph.unwrap_or_else(|| "its paragraph".into());
    Some(format!(
        "--editing-mode deletes {paragraph} outright, and c{parent}'s thread with it, so the reply to c{parent} would join nothing; drop --editing-mode to track the deletion and keep the reply on the struck text, or delete without replying"
    ))
}

/// [`editing_mode_conflict`] for a plan as JSON, as the Python and npm CLIs
/// hold it. A plan that does not parse conflicts with nothing here: running
/// it reports why.
pub fn editing_mode_conflict_json(plan: &str, source: &[u8]) -> Option<String> {
    EditPlan::from_json(plan)
        .ok()
        .and_then(|plan| editing_mode_conflict(&plan, source))
}

/// `plan` with `more` recorded after it: what `--plan PLAN.json` with
/// operation flags writes back. `None` (no plan yet) takes `more` as it is.
/// The two must agree on the source (`source_sha256`), the author, the date
/// and `existing_revisions`, since one plan names each once; `more`'s
/// operations follow `plan`'s and its revision ids join `plan`'s accept and
/// reject lists, which must select by id alone. A plan pinned to no source
/// takes `more`'s. An error records nothing.
///
/// # Errors
///
/// The usage message when the plans disagree on the source, the author, the
/// date or `existing_revisions`, an operation id collides, a revision is both
/// accepted and rejected, or a recorded id would narrow a filter that selects
/// by author or kind.
pub(crate) fn record_into(plan: Option<EditPlan>, more: EditPlan) -> Result<EditPlan, String> {
    let Some(mut plan) = plan else {
        return Ok(more);
    };
    if let (Some(pinned), Some(source)) = (&plan.source_sha256, &more.source_sha256)
        && pinned != source
    {
        return Err(format!(
            "the plan was recorded against another document (sha256 {pinned}); this one is {source}"
        ));
    }
    if plan.author.is_empty() {
        return Err("the plan names no author, so it cannot be applied; start a new plan".into());
    }
    if plan.author != more.author {
        return Err(format!(
            "the plan's author is {:?}; pass --author {:?}, or start a new plan",
            plan.author, plan.author
        ));
    }
    if plan.date != more.date {
        return Err(match &plan.date {
            Some(date) => {
                format!("the plan's date is {date}; pass --datetime {date}, or start a new plan")
            }
            None => {
                "the plan takes its date when it runs; drop --datetime, or start a new plan".into()
            }
        });
    }
    if plan.existing_revisions != more.existing_revisions {
        return Err(format!(
            "the plan's existing_revisions is {mode}; pass --existing-revisions {mode} to match it, or start a new plan",
            mode = existing_name(plan.existing_revisions),
        ));
    }
    let before = plan.operations.len();
    let added = before + 1..=before + more.operations.len();
    if let Some(id) = plan
        .operations
        .iter()
        .filter_map(|op| op.id.as_deref())
        .find(|id| added.clone().any(|n| *id == format!("op-{n}")))
    {
        return Err(format!(
            "the plan names an operation {id}, the id a recorded operation takes; rename it, or start a new plan"
        ));
    }
    if let Some(more) = more.resolve_revisions {
        let resolve = plan.resolve_revisions.get_or_insert_with(Default::default);
        // The plan's reject list strikes and restores (`visible`); a
        // recorded `--reject` joined to it would turn visible too.
        if let Some(ids) = more.reject.as_ref().and_then(|r| r.ids.as_ref())
            && resolve.reject_visible
            && !more.reject_visible
        {
            return Err(format!(
                "the plan's reject is visible, and a recorded --reject is silent: record -p {} --reject --visible, or start a new plan",
                ids.join(" -p ")
            ));
        }
        // Every id in its short spelling, as recorded ones are, so `r1+2`
        // and `r1`, `r2` are the same ids; a malformed one is left for the
        // preview to report.
        for side in [&mut resolve.accept, &mut resolve.reject] {
            if let Some(normal) = side.as_ref().and_then(|f| f.normalized().ok()) {
                *side = Some(normal);
            }
        }
        join_ids(&mut resolve.accept, more.accept, "accept")?;
        join_ids(&mut resolve.reject, more.reject, "reject")?;
        let (accept, reject) = (filter_ids(&resolve.accept), filter_ids(&resolve.reject));
        if let Some(both) = accept.iter().find(|id| reject.contains(id)) {
            return Err(format!("{both} is both accepted and rejected"));
        }
    }
    plan.operations.extend(more.operations);
    // A permission recorded once holds for the whole plan.
    plan.rewrite_others_comments |= more.rewrite_others_comments;
    // The merged plan is previewed against `more`'s source before it is
    // written, so an unpinned plan may take that pin.
    plan.source_sha256 = plan.source_sha256.or(more.source_sha256);
    Ok(plan)
}

/// `--existing-revisions`' spelling of `mode`.
fn existing_name(mode: ExistingRevisions) -> &'static str {
    match mode {
        ExistingRevisions::Keep => "keep",
        ExistingRevisions::Accept => "accept",
        ExistingRevisions::Reject => "reject",
        ExistingRevisions::Refuse => "refuse",
    }
}

/// `more`'s ids after `filter`'s, each once. A filter that also selects by
/// author or kind, or one naming no ids (every change), would be narrowed by
/// the ids, since a change must match every list: that is refused.
fn join_ids(
    filter: &mut Option<ChangeFilter>,
    more: Option<ChangeFilter>,
    side: &str,
) -> Result<(), String> {
    let Some(more) = more.and_then(|m| m.ids) else {
        return Ok(());
    };
    if let Some(held) = filter.as_ref()
        && (held.authors.is_some() || held.kinds.is_some() || held.ids.is_none())
    {
        return Err(format!(
            "the plan's {side} selects revisions by author or kind, or all of them; a recorded revision id would narrow it, so start a new plan"
        ));
    }
    let ids = filter
        .get_or_insert_with(Default::default)
        .ids
        .get_or_insert_with(Vec::new);
    for id in more {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    Ok(())
}

/// The ids `filter` names.
fn filter_ids(filter: &Option<ChangeFilter>) -> &[String] {
    filter
        .as_ref()
        .and_then(|f| f.ids.as_deref())
        .unwrap_or_default()
}

/// `notes` (`op-N: …`, numbered within one recording) renumbered after the
/// `before` operations the plan already held.
pub(crate) fn renumber_notes(notes: Vec<String>, before: usize) -> Vec<String> {
    notes
        .into_iter()
        .map(|note| {
            let parsed = note
                .strip_prefix("op-")
                .and_then(|rest| rest.split_once(':'))
                .and_then(|(n, rest)| Some((n.parse::<usize>().ok()?, rest)));
            match parsed {
                Some((n, rest)) => format!("op-{}:{rest}", n + before),
                None => note,
            }
        })
        .collect()
}

/// [`plan_from_flags`] over JSON, for the Python and npm CLIs: `verb` is
/// `edit` or `add`, `operations` the `operations` array their parsed
/// command carries, `existing` the `--existing-revisions` value (`auto`,
/// `keep`, `accept`, `reject`, `refuse`). Returns the plan as JSON and the
/// notes.
///
/// # Errors
///
/// The usage message for an unknown verb, unparsable operations, a bad
/// `--existing-revisions` value, and every [`plan_from_flags`] failure.
pub fn plan_from_flags_json(
    verb: &str,
    operations: &str,
    author: &str,
    date: Option<&str>,
    existing: &str,
    source: &[u8],
) -> Result<(String, Vec<String>), String> {
    let verb = match verb {
        "edit" => Verb::Edit,
        "add" => Verb::Add,
        other => return Err(format!("unknown verb {other:?}: edit or add")),
    };
    let ops: Vec<FlagOp> =
        serde_json::from_str(operations).map_err(|e| format!("operations: {e}"))?;
    let existing = match existing {
        "auto" => None,
        "keep" => Some(ExistingRevisions::Keep),
        "accept" => Some(ExistingRevisions::Accept),
        "reject" => Some(ExistingRevisions::Reject),
        "refuse" => Some(ExistingRevisions::Refuse),
        other => {
            return Err(format!(
                "existing revisions {other:?}: auto, keep, accept, reject or refuse"
            ));
        }
    };
    let built = plan_from_flags(verb, &ops, author, date, existing, source)?;
    let json = serde_json::to_string(&built.plan).map_err(|e| e.to_string())?;
    Ok((json, built.notes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(at: &str) -> FlagOp {
        FlagOp {
            at: at.into(),
            ..FlagOp::default()
        }
    }

    fn kind(verb: Verb, op: &FlagOp) -> Result<OperationKind, String> {
        flag_operation(verb, op).map(|(k, _)| k)
    }

    /// The plan `ops` record by `author`, pinned to `sha`.
    fn recorded(verb: Verb, ops: &[FlagOp], author: &str, sha: &str) -> EditPlan {
        let mut plan = plan_from_flags(verb, ops, author, None, Some(ExistingRevisions::Keep), b"")
            .unwrap()
            .plan;
        plan.source_sha256 = Some(sha.into());
        plan
    }

    fn revision(at: &str, accept: bool) -> FlagOp {
        FlagOp {
            accept,
            reject: !accept,
            ..op(at)
        }
    }

    fn rewrite(at: &str, content: &str) -> FlagOp {
        FlagOp {
            content: Some(content.into()),
            ..op(at)
        }
    }

    /// `--plan` with operation flags: the first recording is the plan, each
    /// later one appends its operations and joins its revision ids.
    #[test]
    fn recording_appends_operations_and_joins_revision_ids() {
        let first = recorded(
            Verb::Edit,
            &[rewrite("p1", "One."), revision("r2", true)],
            "Ann",
            "aa",
        );
        let plan = record_into(None, first.clone()).unwrap();
        assert_eq!(plan, first);
        let second = recorded(
            Verb::Add,
            &[FlagOp {
                comment: true,
                ..rewrite("p3", "Why?")
            }],
            "Ann",
            "aa",
        );
        let third = recorded(
            Verb::Edit,
            &[revision("r2+5", true), revision("header1.r1", false)],
            "Ann",
            "aa",
        );
        let plan = record_into(Some(record_into(Some(plan), second).unwrap()), third).unwrap();
        assert_eq!(plan.operations.len(), 2);
        assert!(matches!(
            plan.operations[1].kind,
            OperationKind::Comment { .. }
        ));
        let resolve = plan.resolve_revisions.unwrap();
        assert_eq!(resolve.accept.unwrap().ids.unwrap(), ["r2", "r5"]);
        assert_eq!(resolve.reject.unwrap().ids.unwrap(), ["header1.r1"]);
        assert_eq!(plan.source_sha256.as_deref(), Some("aa"));
    }

    /// One plan names one source, author, date and mode: a recording that
    /// disagrees is refused and says what to pass.
    #[test]
    fn recording_refuses_what_the_plan_names_differently() {
        let base = || recorded(Verb::Edit, &[rewrite("p1", "One.")], "Ann", "aa");
        let refused = |more: EditPlan| record_into(Some(base()), more).unwrap_err();
        assert!(
            refused(recorded(Verb::Edit, &[rewrite("p2", "Two.")], "Ann", "bb"))
                .starts_with("the plan was recorded against another document (sha256 aa)")
        );
        assert_eq!(
            refused(recorded(Verb::Edit, &[rewrite("p2", "Two.")], "Bob", "aa")),
            r#"the plan's author is "Ann"; pass --author "Ann", or start a new plan"#
        );
        let mut dated = recorded(Verb::Edit, &[rewrite("p2", "Two.")], "Ann", "aa");
        dated.date = Some("2026-10-12T00:00:00Z".into());
        assert_eq!(
            refused(dated),
            "the plan takes its date when it runs; drop --datetime, or start a new plan"
        );
        let mut refusing = recorded(Verb::Edit, &[rewrite("p2", "Two.")], "Ann", "aa");
        refusing.existing_revisions = ExistingRevisions::Refuse;
        assert_eq!(
            refused(refusing),
            "the plan's existing_revisions is keep; pass --existing-revisions keep to match it, or start a new plan"
        );
        let accepted = recorded(Verb::Edit, &[revision("r2", true)], "Ann", "aa");
        let plan = record_into(Some(base()), accepted).unwrap();
        assert_eq!(
            record_into(
                Some(plan),
                recorded(Verb::Edit, &[revision("r2", false)], "Ann", "aa")
            )
            .unwrap_err(),
            "r2 is both accepted and rejected"
        );
    }

    /// pi review 06b F1, F2, F4.
    #[test]
    fn recording_pins_an_unpinned_plan_and_keeps_hand_written_meaning() {
        let mut unpinned = recorded(Verb::Edit, &[rewrite("p1", "One.")], "Ann", "aa");
        unpinned.source_sha256 = None;
        let more = recorded(Verb::Edit, &[rewrite("p2", "Two.")], "Ann", "bb");
        let plan = record_into(Some(unpinned), more).unwrap();
        assert_eq!(plan.source_sha256.as_deref(), Some("bb"));
        // An accept by author (or of everything) would be narrowed by an id.
        for held in [
            ChangeFilter {
                authors: Some(vec!["Ann".into()]),
                ..ChangeFilter::default()
            },
            ChangeFilter::default(),
        ] {
            let mut plan = recorded(Verb::Edit, &[rewrite("p1", "One.")], "Ann", "aa");
            plan.resolve_revisions = Some(ResolveRevisions {
                accept: Some(held),
                reject: None,
                reject_visible: false,
            });
            let accept = recorded(Verb::Edit, &[revision("r9", true)], "Ann", "aa");
            assert!(
                record_into(Some(plan), accept)
                    .unwrap_err()
                    .starts_with("the plan's accept selects revisions by author or kind, or all")
            );
        }
        // A held id that a recorded operation's position would repeat.
        let mut named = recorded(Verb::Edit, &[rewrite("p1", "One.")], "Ann", "aa");
        named.operations[0].id = Some("op-2".into());
        let more = recorded(Verb::Edit, &[rewrite("p2", "Two.")], "Ann", "aa");
        assert_eq!(
            record_into(Some(named), more).unwrap_err(),
            "the plan names an operation op-2, the id a recorded operation takes; rename it, or start a new plan"
        );
    }

    /// CodeRabbit on #1286: a recorded `--reject` is silent, and the plan's
    /// reject list strikes and restores; merged, the recorded id would turn
    /// visible.
    #[test]
    fn a_silent_reject_is_not_recorded_into_a_visible_one() {
        let mut plan = recorded(Verb::Edit, &[rewrite("p1", "One.")], "Ann", "aa");
        plan.resolve_revisions = Some(ResolveRevisions {
            accept: None,
            reject: Some(ChangeFilter::ids(["r4"])),
            reject_visible: true,
        });
        let reject = recorded(Verb::Edit, &[revision("r5", false)], "Ann", "aa");
        assert_eq!(
            record_into(Some(plan.clone()), reject).unwrap_err(),
            "the plan's reject is visible, and a recorded --reject is silent: record -p r5 --reject --visible, or start a new plan"
        );
        // An accept still records beside it.
        let accept = recorded(Verb::Edit, &[revision("r6", true)], "Ann", "aa");
        let merged = record_into(Some(plan), accept).unwrap();
        let resolve = merged.resolve_revisions.unwrap();
        assert!(resolve.reject_visible);
        assert_eq!(resolve.accept.unwrap().ids.unwrap(), ["r6"]);
    }

    /// pi review 06c: id spellings, string op ids, an empty author.
    #[test]
    fn recording_reads_ids_and_names_as_the_plan_spells_them() {
        let mut short = recorded(Verb::Edit, &[rewrite("p1", "One.")], "Ann", "aa");
        short.resolve_revisions = Some(ResolveRevisions {
            accept: Some(ChangeFilter::ids(["r1"])),
            reject: None,
            reject_visible: false,
        });
        let again = recorded(Verb::Edit, &[revision("r1", true)], "Ann", "aa");
        let plan = record_into(Some(short.clone()), again).unwrap();
        let accept = plan.resolve_revisions.unwrap().accept.unwrap().ids.unwrap();
        assert_eq!(accept, ["r1"]);
        let reject = recorded(Verb::Edit, &[revision("r1", false)], "Ann", "aa");
        assert_eq!(
            record_into(Some(short), reject).unwrap_err(),
            "r1 is both accepted and rejected"
        );
        let mut padded = recorded(Verb::Edit, &[rewrite("p1", "One.")], "Ann", "aa");
        padded.operations[0].id = Some("op-02".into());
        let more = recorded(Verb::Edit, &[rewrite("p2", "Two.")], "Ann", "aa");
        assert!(record_into(Some(padded), more).is_ok());
        let mut nobody = recorded(Verb::Edit, &[rewrite("p1", "One.")], "", "aa");
        nobody.author = String::new();
        let more = recorded(Verb::Edit, &[rewrite("p2", "Two.")], "", "aa");
        assert!(
            record_into(Some(nobody), more)
                .unwrap_err()
                .starts_with("the plan names no author")
        );
    }

    #[test]
    fn notes_count_on_from_the_operations_the_plan_held() {
        assert_eq!(
            renumber_notes(vec!["op-1: kept as text".into(), "other".into()], 3),
            ["op-4: kept as text", "other"]
        );
    }

    /// pi review av4 F5: deleting an anchor has nothing to format, with or
    /// without the anchor.
    #[test]
    fn delete_refuses_a_style_with_an_anchor_too() {
        let mut delete = op("p4");
        delete.anchor = Some("Fees".into());
        delete.delete = true;
        delete.styles = vec!["bold".into()];
        assert_eq!(
            kind(Verb::Edit, &delete).unwrap_err(),
            "-p p4: --delete takes no --style"
        );
        delete.styles = vec!["Heading2".into()];
        assert_eq!(
            kind(Verb::Edit, &delete).unwrap_err(),
            "-p p4: --delete takes no --style"
        );
    }

    #[test]
    fn add_refuses_what_it_cannot_write() {
        let mut new = op("p1");
        new.content = Some("Pay on time.".into());
        for style in ["strike", "caps", "font=Calibri", "size=11", "color=FF0000"] {
            new.styles = vec![style.into()];
            let e = kind(Verb::Add, &new).unwrap_err();
            assert!(e.contains("a new paragraph takes --style"), "{style}: {e}");
        }
        new.styles = vec!["bold".into(), "highlight=yellow".into(), "Heading2".into()];
        assert!(kind(Verb::Add, &new).is_ok());
        let mut comment = op("p1");
        comment.anchor = Some("Pay".into());
        comment.content = Some("Why?".into());
        comment.styles = vec!["bold".into()];
        assert!(
            kind(Verb::Add, &comment)
                .unwrap_err()
                .contains("a comment takes no --style")
        );
        let mut delete = op("c0");
        delete.delete = true;
        assert_eq!(
            kind(Verb::Add, &delete).unwrap_err(),
            "-p c0: add takes no --delete or --resolve"
        );
    }

    #[test]
    fn the_json_entry_point_reads_what_the_cli_schema_serializes() {
        let ops = r#"[{"at":"p0","anchor":null,"content":"**Fees**","delete":false,"resolve":false,"before":false,"comment":false,"styles":[]}]"#;
        let (plan, notes) = plan_from_flags_json(
            "edit",
            ops,
            "Ann",
            Some("2026-10-01T09:00:00Z"),
            "keep",
            b"",
        )
        .unwrap();
        let plan = EditPlan::from_json(&plan).unwrap();
        assert_eq!(plan.author, "Ann");
        assert_eq!(plan.existing_revisions, ExistingRevisions::Keep);
        assert_eq!(plan.operations.len(), 1);
        assert_eq!(
            notes,
            ["op-1: content keeps its Markdown marks as text; use --style for formatting"]
        );
        for (verb, existing) in [("move", "keep"), ("edit", "sometimes")] {
            assert!(plan_from_flags_json(verb, ops, "Ann", None, existing, b"").is_err());
        }
        assert!(plan_from_flags_json("edit", "{", "Ann", None, "keep", b"").is_err());
    }

    #[test]
    fn an_anchor_kept_at_either_end_is_an_insertion() {
        let k = inline_edit(
            Selector::Name("p1".into()),
            "thirty",
            "thirty (30)".into(),
            None,
        )
        .unwrap();
        assert!(
            matches!(k, OperationKind::Insert { after: Some(ref a), ref text, .. } if a == "thirty" && text == " (30)")
        );
        let k = inline_edit(
            Selector::Name("p2".into()),
            "Late",
            "Note: Late".into(),
            None,
        )
        .unwrap();
        assert!(
            matches!(k, OperationKind::Insert { before: Some(ref b), ref text, .. } if b == "Late" && text == "Note: ")
        );
        let k = inline_edit(
            Selector::Name("p0".into()),
            "# Fees",
            "Fees * Costs".into(),
            None,
        )
        .unwrap();
        assert!(
            matches!(k, OperationKind::Insert { after: Some(ref a), ref text, .. } if a == "# Fees" && text == " * Costs")
        );
        let k = inline_edit(Selector::Name("p1".into()), "thirty", "sixty".into(), None).unwrap();
        assert!(
            matches!(k, OperationKind::Replace { ref replacement, .. } if replacement == "sixty")
        );
        assert!(inline_edit(Selector::Name("p1".into()), "# x", "x".into(), None).is_err());
    }

    /// pi review av4 F12: the note fires whenever the content reads
    /// differently as Markdown, CriticMarkup and block marks included, and
    /// not for escaped marks or a lone `*`.
    #[test]
    fn content_notes_every_mark_the_view_would_read() {
        for noted in [
            "**bold**",
            "{++x++}",
            "{~~a~>b~~}",
            "# Title",
            "_x_",
            "<u>u</u>",
        ] {
            assert!(content_text(noted).1.is_some(), "{noted}");
        }
        for plain in [
            "\\*\\*x\\*\\*",
            "*italics?",
            "2 * 3",
            "snake__case",
            "plain",
            " lead",
            "trail ",
            "two\nlines",
            "a = b",
            "x - y",
        ] {
            assert!(content_text(plain).1.is_none(), "{plain}");
        }
    }

    /// pi review av4 F8: "make thirty bold" written as `--anchor thirty
    /// --content thirty --style bold` formats the run instead of refusing.
    #[test]
    fn content_equal_to_the_anchor_with_a_style_formats_the_run() {
        let bold = RunFormat {
            bold: Some(true),
            ..RunFormat::default()
        };
        for content in ["thirty", "**thirty**"] {
            let k = inline_edit(
                Selector::Name("p1".into()),
                "**thirty**",
                content.into(),
                Some(bold.clone()),
            )
            .unwrap();
            assert!(
                matches!(k, OperationKind::FormatRun { ref find, ref format, .. } if find == "**thirty**" && format.bold == Some(true)),
                "{content}"
            );
        }
    }

    #[test]
    fn each_flag_combination_maps_to_one_kind() {
        let with = |f: &dyn Fn(&mut FlagOp)| {
            let mut o = op("p1");
            f(&mut o);
            o
        };
        assert!(matches!(
            kind(Verb::Edit, &with(&|o| o.content = Some("New text.".into()))),
            Ok(OperationKind::Rewrite { .. })
        ));
        assert!(matches!(
            kind(Verb::Edit, &with(&|o| o.delete = true)),
            Ok(OperationKind::DeleteParagraph { .. })
        ));
        assert!(matches!(
            kind(
                Verb::Edit,
                &with(&|o| {
                    o.anchor = Some("a".into());
                    o.delete = true;
                })
            ),
            Ok(OperationKind::Delete { .. })
        ));
        assert!(matches!(
            kind(
                Verb::Edit,
                &with(&|o| {
                    o.anchor = Some("a".into());
                    o.styles = vec!["bold".into(), "size=10.5".into()];
                })
            ),
            Ok(OperationKind::FormatRun {
                format: RunFormat {
                    bold: Some(true),
                    size_pt: Some(HalfPoints(21)),
                    ..
                },
                ..
            })
        ));
        assert!(matches!(
            kind(Verb::Edit, &with(&|o| o.styles = vec!["Heading2".into()])),
            Ok(OperationKind::FormatParagraph { style: Some(_), .. })
        ));
        assert!(matches!(
            kind(
                Verb::Add,
                &with(&|o| {
                    o.content = Some("New".into());
                    o.before = true;
                    o.styles = vec!["Heading2".into(), "bold".into()];
                })
            ),
            Ok(OperationKind::InsertParagraph {
                position: Side::Before,
                style: Some(_),
                ..
            })
        ));
        assert!(matches!(
            kind(
                Verb::Add,
                &with(&|o| {
                    o.anchor = Some("a".into());
                    o.content = Some("Why?".into());
                })
            ),
            Ok(OperationKind::Comment { find: Some(_), .. })
        ));
        assert!(matches!(
            kind(
                Verb::Add,
                &with(&|o| {
                    o.comment = true;
                    o.content = Some("Why?".into());
                })
            ),
            Ok(OperationKind::Comment { find: None, .. })
        ));
        let c5 = |f: &dyn Fn(&mut FlagOp)| {
            let mut o = op("c5");
            f(&mut o);
            o
        };
        assert!(matches!(
            kind(Verb::Add, &c5(&|o| o.content = Some("Yes.".into()))),
            Ok(OperationKind::ReplyComment { comment_id: 5, .. })
        ));
        assert!(matches!(
            kind(Verb::Edit, &c5(&|o| o.content = Some("Yes.".into()))),
            Ok(OperationKind::EditComment { comment_id: 5, .. })
        ));
        assert!(matches!(
            kind(Verb::Edit, &c5(&|o| o.delete = true)),
            Ok(OperationKind::DeleteComment { comment_id: 5 })
        ));
        assert!(matches!(
            kind(Verb::Edit, &c5(&|o| o.resolve = true)),
            Ok(OperationKind::ResolveComment {
                comment_id: 5,
                done: true
            })
        ));
    }

    #[test]
    fn contradictory_flags_are_usage_errors() {
        let cases: Vec<(Verb, FlagOp)> = vec![
            (Verb::Edit, op("p1")),
            (
                Verb::Edit,
                FlagOp {
                    delete: true,
                    content: Some("x".into()),
                    ..op("p1")
                },
            ),
            (
                Verb::Edit,
                FlagOp {
                    resolve: true,
                    ..op("p1")
                },
            ),
            (
                Verb::Edit,
                FlagOp {
                    anchor: Some("a".into()),
                    content: Some("b".into()),
                    ..op("c0")
                },
            ),
            (
                Verb::Edit,
                FlagOp {
                    content: Some("x".into()),
                    styles: vec!["bold".into()],
                    ..op("p1")
                },
            ),
            (
                Verb::Edit,
                FlagOp {
                    styles: vec!["size=big".into()],
                    anchor: Some("a".into()),
                    ..op("p1")
                },
            ),
            (
                Verb::Edit,
                FlagOp {
                    styles: vec!["A".into(), "B".into()],
                    ..op("p1")
                },
            ),
            (Verb::Add, op("p1")),
            (
                Verb::Add,
                FlagOp {
                    anchor: Some("x".into()),
                    content: Some("y".into()),
                    before: true,
                    ..op("p1")
                },
            ),
            (
                Verb::Add,
                FlagOp {
                    content: Some("y".into()),
                    before: true,
                    ..op("c0")
                },
            ),
            (
                Verb::Add,
                FlagOp {
                    content: Some("y".into()),
                    delete: true,
                    ..op("p1")
                },
            ),
            (
                Verb::Edit,
                FlagOp {
                    anchor: Some("same".into()),
                    content: Some("same".into()),
                    ..op("p1")
                },
            ),
        ];
        for (verb, o) in cases {
            assert!(flag_operation(verb, &o).is_err(), "{verb:?} {o:?}");
        }
        assert!(plan_from_flags(Verb::Edit, &[], "A", None, None, b"").is_err());
    }

    #[test]
    fn content_escapes_resolve_and_kept_marks_get_a_note() {
        let (text, note) = content_text("Fees \\* Costs");
        assert_eq!((text.as_str(), note), ("Fees * Costs", None));
        let (text, note) = content_text("**Fees**");
        assert_eq!(text, "**Fees**");
        assert!(note.is_some());
        // A line copied from the agent view keeps its leading space as
        // `&#32;`: it is a space, and no mark.
        let (text, note) = content_text("&#32;Fees");
        assert_eq!((text.as_str(), note), (" Fees", None));
        let plan = plan_from_flags(
            Verb::Edit,
            &[FlagOp {
                content: Some("**Fees**".into()),
                ..op("p0")
            }],
            "Redline",
            Some("2026-10-01T09:00:00Z"),
            Some(ExistingRevisions::Refuse),
            b"",
        )
        .unwrap();
        assert_eq!(
            plan.notes,
            ["op-1: content keeps its Markdown marks as text; use --style for formatting"]
        );
        assert_eq!(plan.plan.author, "Redline");
    }

    fn with(at: &str, set: impl Fn(&mut FlagOp)) -> FlagOp {
        let mut o = op(at);
        set(&mut o);
        o
    }

    /// `-p r1 --accept` and `-p header1.r2 --reject` join the plan's
    /// `resolve_revisions` as long ids; the other groups stay operations,
    /// numbered among themselves.
    #[test]
    fn revisions_go_to_resolve_revisions() {
        let ops = [
            with("r1", |o| o.accept = true),
            with("p0", |o| o.content = Some("**x**".into())),
            with("header1.r2", |o| o.reject = true),
            with("r3+4", |o| o.accept = true),
        ];
        let built = plan_from_flags(Verb::Edit, &ops, "A", None, None, b"").unwrap();
        assert_eq!(
            built.plan.resolve_revisions,
            Some(ResolveRevisions {
                accept: Some(ChangeFilter::ids(["r1", "r3", "r4"])),
                reject: Some(ChangeFilter::ids(["header1.r2"])),
                reject_visible: false,
            })
        );
        assert_eq!(built.plan.operations.len(), 1);
        assert_eq!(
            built.notes,
            ["op-1: content keeps its Markdown marks as text; use --style for formatting"]
        );
        let only = plan_from_flags(
            Verb::Edit,
            &[with("r7", |o| o.reject = true)],
            "A",
            None,
            None,
            b"",
        )
        .unwrap();
        assert!(only.plan.operations.is_empty());
        assert_eq!(
            only.plan.resolve_revisions.unwrap().reject,
            Some(ChangeFilter::ids(["r7"]))
        );
    }

    /// `-p r4 --strike` and `-p r5 --restore` are operations by our
    /// author, with `--content` as the comment on them.
    #[test]
    fn strike_and_restore_groups_are_operations() {
        let built = plan_from_flags(
            Verb::Edit,
            &[
                with("r4", |o| {
                    o.strike = true;
                    o.content = Some("No.".into());
                }),
                with("header1.r2", |o| o.restore = true),
            ],
            "A",
            None,
            None,
            b"",
        )
        .unwrap();
        assert!(built.plan.resolve_revisions.is_none());
        let kinds: Vec<OperationKind> = built.plan.operations.into_iter().map(|o| o.kind).collect();
        assert_eq!(
            kinds,
            [
                OperationKind::Strike {
                    revision: "r4".into(),
                    comment: Some("No.".into()),
                },
                OperationKind::Restore {
                    revision: "header1.r2".into(),
                    comment: None,
                },
            ]
        );
    }

    #[test]
    fn strike_restore_and_visible_refuse_what_they_cannot_mean() {
        for (verb, group, want) in [
            (
                Verb::Edit,
                with("r4", |o| {
                    o.strike = true;
                    o.restore = true;
                }),
                "-p r4: --strike and --restore exclude each other",
            ),
            (
                Verb::Edit,
                with("r4", |o| {
                    o.accept = true;
                    o.strike = true;
                }),
                "-p r4: --accept and --strike exclude each other",
            ),
            (
                Verb::Edit,
                with("r4", |o| o.visible = true),
                "-p r4: --visible goes with --reject",
            ),
            (
                Verb::Edit,
                with("p0", |o| o.strike = true),
                "-p p0: --strike takes a revision (-p r12)",
            ),
            (
                Verb::Edit,
                with("r4", |o| {
                    o.strike = true;
                    o.anchor = Some("x".into());
                }),
                "-p r4: a revision takes --accept, --reject, --strike or --restore",
            ),
            (
                Verb::Edit,
                with("r4", |o| {
                    o.reject = true;
                    o.content = Some("No.".into());
                }),
                "-p r4: --content on a revision is the comment of --strike, --restore or --reject --visible",
            ),
            (
                Verb::Add,
                with("r4", |o| o.strike = true),
                "-p r4: add resolves no revision; use edit -p r12 --accept, --reject, --strike or --restore",
            ),
        ] {
            let got = plan_from_flags(verb, &[group], "A", None, None, b"").map(|_| ());
            assert_eq!(got, Err(want.to_string()));
        }
    }

    /// `--reply` on `edit` and `add` is the reply `add -p c5 --content`
    /// writes.
    #[test]
    fn reply_answers_a_comment_on_edit_and_add() {
        let want = kind(Verb::Add, &with("c5", |o| o.content = Some("Yes.".into()))).unwrap();
        for verb in [Verb::Edit, Verb::Add] {
            assert_eq!(
                kind(verb, &with("c5", |o| o.reply = Some("Yes.".into()))),
                Ok(want.clone()),
                "{verb:?}"
            );
            // 0.12's `#c5` is expired, refused with the settled spelling.
            let refused = kind(verb, &with("#c5", |o| o.reply = Some("Yes.".into())));
            assert!(
                refused
                    .as_ref()
                    .is_err_and(|m| m.ends_with("the settled spelling is c5")),
                "{refused:?}"
            );
        }
    }

    #[test]
    fn revision_and_reply_flags_pair_only_with_their_kind() {
        let refusal =
            |verb: Verb, o: FlagOp| plan_from_flags(verb, &[o], "A", None, None, b"").unwrap_err();
        let cases: Vec<(FlagOp, &str)> = vec![
            (
                with("p1", |o| o.accept = true),
                "-p p1: --accept and --reject take a revision (-p r12)",
            ),
            (
                with("c5", |o| o.reject = true),
                "-p c5: --accept and --reject take a revision (-p r12)",
            ),
            (
                with("r0", |o| o.content = Some("x".into())),
                "-p r0: a revision takes --accept, --reject, --strike or --restore",
            ),
            (
                with("r0", |_| {}),
                "-p r0: a revision takes --accept, --reject, --strike or --restore",
            ),
            (
                with("r0", |o| {
                    o.accept = true;
                    o.reject = true;
                }),
                "-p r0: --accept and --reject exclude each other",
            ),
            (
                with("r0", |o| {
                    o.accept = true;
                    o.delete = true;
                }),
                "-p r0: a revision takes --accept, --reject, --strike or --restore",
            ),
            (
                with("p1", |o| o.reply = Some("x".into())),
                "-p p1: --reply is for a comment (-p c5)",
            ),
            (
                with("r1", |o| o.reply = Some("x".into())),
                "-p r1: --reply is for a comment (-p c5)",
            ),
            (
                with("c5", |o| {
                    o.reply = Some("x".into());
                    o.content = Some("y".into());
                }),
                "-p c5: --reply takes no other flag",
            ),
            (
                with("c5", |o| {
                    o.reply = Some("x".into());
                    o.resolve = true;
                }),
                "-p c5: --reply takes no other flag",
            ),
            (
                with("c5", |o| {
                    o.reply = Some("x".into());
                    o.comment = true;
                }),
                "-p c5: --reply takes no other flag",
            ),
            (
                with("c5", |o| {
                    o.reply = Some("x".into());
                    o.accept = true;
                }),
                "-p c5: --reply takes no other flag",
            ),
            (
                with("c5", |o| {
                    o.reply = Some("x".into());
                    o.reject = true;
                }),
                "-p c5: --reply takes no other flag",
            ),
        ];
        for (o, message) in cases {
            assert_eq!(refusal(Verb::Edit, o), message);
        }
        assert_eq!(
            refusal(
                Verb::Add,
                with("c5", |o| {
                    o.reply = Some("x".into());
                    o.before = true;
                })
            ),
            "-p c5: --reply takes no other flag"
        );
        for o in [
            with("r1", |o| o.accept = true),
            with("r1", |o| o.content = Some("x".into())),
        ] {
            assert_eq!(
                refusal(Verb::Add, o.clone()),
                format!(
                    "-p {}: add resolves no revision; use edit -p r12 --accept, --reject, --strike or --restore",
                    o.at
                )
            );
        }
        assert_eq!(
            refusal(Verb::Add, with("r1", |o| o.reply = Some("x".into()))),
            "-p r1: --reply is for a comment (-p c5)"
        );
        assert_eq!(
            refusal(Verb::Add, with("c5", |o| o.accept = true)),
            "-p c5: --accept and --reject take a revision (-p r12)"
        );
    }
}
