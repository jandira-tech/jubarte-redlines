// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! `edit` and `add` by flags: each `-p WHERE` with its `--anchor`,
//! `--content`, `--delete`, `--resolve`, `--before`, `--comment` and
//! `--style` becomes one operation of an [`EditPlan`]. The binary, the
//! Python CLI and the npm CLI all build their plan here.

use serde::{Deserialize, Serialize};

use super::{
    EditPlan, ExistingRevisions, HalfPoints, Operation, OperationKind, RunFormat, RunSpec,
    Selector, Side,
};

/// One operation as the flags describe it, grouped by its `-p`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FlagOp {
    /// The `-p` value: `p12`, `header1`, `t0.r1.c2`, `c5`, or a long id.
    pub at: String,
    /// `--anchor`: text inside `at`.
    pub anchor: Option<String>,
    /// `--content`: new text.
    pub content: Option<String>,
    /// `--delete`.
    pub delete: bool,
    /// `--resolve` (`edit` only).
    pub resolve: bool,
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
/// comparer path. An error is a usage error.
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
    for (i, op) in ops.iter().enumerate() {
        let (kind, note) = flag_operation(verb, op)?;
        operations.push(Operation { id: None, kind });
        notes.extend(note.map(|n| format!("op-{}: {n}", i + 1)));
    }
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
            resolve_revisions: None,
            existing_revisions: existing,
            operations,
            update_fields: false,
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
/// note when emphasis marks remain, since they are written as text.
fn content_text(content: &str) -> (String, Option<&'static str>) {
    let text = crate::markdown::unescape_markdown(content);
    let marks = ["**", "__", "~~", "==", "<u>", "</u>"]
        .iter()
        .any(|m| text.contains(m));
    (
        text,
        marks.then_some("content keeps its Markdown marks as text; use --style for formatting"),
    )
}

/// `c5` → 5.
fn comment_number(at: &str) -> Option<u32> {
    let digits = at.strip_prefix('c')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// The operation one `-p` group stands for, and a note for the report.
fn flag_operation(verb: Verb, op: &FlagOp) -> Result<(OperationKind, Option<String>), String> {
    use OperationKind as K;
    let at = op.at.trim();
    let bad = |m: &str| Err(format!("-p {at}: {m}"));
    if at.is_empty() {
        return Err("-p needs a location".into());
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
    let kind = match (verb, comment_id, &op.anchor, content) {
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
        (Verb::Edit, None, Some(anchor), None) if op.delete => K::Delete {
            paragraph: paragraph(),
            find: anchor.clone(),
            occurrence: None,
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
        return Err("--content equals the anchor; nothing to change".into());
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

/// Why `--editing-mode` cannot apply `plan`: under `keep` the document's
/// tracked changes stay, so its "clean" copy would still carry revisions.
pub fn editing_mode_conflict(plan: &EditPlan) -> Option<&'static str> {
    (plan.existing_revisions == ExistingRevisions::Keep).then_some(
        "--editing-mode needs a document without tracked changes; it has some, so pass --existing-revisions accept or reject",
    )
}

/// [`plan_from_flags`] over JSON, for the Python and npm CLIs: `verb` is
/// `edit` or `add`, `operations` the `operations` array their parsed
/// command carries, `existing` the `--existing-revisions` value (`auto`,
/// `keep`, `accept`, `reject`, `refuse`). Returns the plan as JSON and the
/// notes; an error is a usage error.
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
        let plan = plan_from_flags(
            Verb::Edit,
            &[FlagOp {
                content: Some("**Fees**".into()),
                ..op("p0")
            }],
            "Modified User",
            Some("2026-10-01T09:00:00Z"),
            Some(ExistingRevisions::Refuse),
            b"",
        )
        .unwrap();
        assert_eq!(
            plan.notes,
            ["op-1: content keeps its Markdown marks as text; use --style for formatting"]
        );
        assert_eq!(plan.plan.author, "Modified User");
    }
}
