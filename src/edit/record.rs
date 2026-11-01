// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! `edit` and `add` with `--plan PLAN.json`: with operation flags they
//! record the operations into the plan instead of editing; alone they apply
//! it, and `--consume` deletes it once applied. The binary, the Python CLI
//! and the npm CLI all record here, and the shared parser refuses the flags
//! that do not go with `--plan` ([`plan_flag_conflict`]), so the three print
//! the same lines.

use std::path::{Path, PathBuf};

use super::flags::{FlagOp, Verb, plan_from_flags, record_into, renumber_notes};
use super::{EditPlan, ExistingRevisions};

/// Flags that take effect when a plan is applied, so a recording refuses
/// them.
const APPLY_ONLY: &[&str] = &[
    "--consume",
    "--out-dir",
    "--output",
    "--in-place",
    "--force",
    "--editing-mode",
    "--suggesting-mode",
    "--pdf",
    "--png",
    "--dpi",
    "--revisions",
    "--revision-palette",
];

/// Flags that describe recorded operations, so applying a plan refuses
/// them: the plan names its own.
const RECORD_ONLY: &[&str] = &["--author", "--datetime", "--existing-revisions"];

/// Why `given` (the flags the command line spells out) cannot go with
/// `--plan plan`: `recording` is `--plan` with operation flags, otherwise
/// the plan is applied. `None` when they can.
pub fn plan_flag_conflict(recording: bool, given: &[&str], plan: &str) -> Option<String> {
    if recording {
        let flag = APPLY_ONLY.iter().find(|f| given.contains(f))?;
        // A bare --plan (D16) is still unresolved here: spell it bare.
        let apply = if plan.is_empty() {
            "--plan".to_string()
        } else {
            format!("--plan {plan}")
        };
        Some(format!(
            "{flag} takes effect when the plan is applied (jubarte edit FILE {apply}), and is not recorded"
        ))
    } else {
        let flag = RECORD_ONLY.iter().find(|f| given.contains(f))?;
        if *flag == "--existing-revisions" {
            return Some(format!(
                "{flag} belongs to recorded operations; {}",
                super::KEEP_REVISIONS
            ));
        }
        Some(format!(
            "{flag} belongs to recorded operations; the plan names its own"
        ))
    }
}

/// Why a recording failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordError {
    /// The flags, or their disagreement with the plan: exit 2.
    Usage(String),
    /// The plan file held is not a plan: exit 3, as applying it would.
    Plan(String),
}

/// A recording, ready to be checked against the source and written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    /// The plan with the new operations, to preview against the source
    /// and then write.
    pub plan: EditPlan,
    /// `op-N: …` notes, numbered within the whole plan.
    pub notes: Vec<String>,
    /// How many operations this recording added.
    pub added: usize,
    /// How many revision ids this recording added to accept or reject.
    pub added_revisions: usize,
}

impl Recorded {
    /// The plan as written to PLAN.json: pretty JSON with a final newline.
    pub fn json(&self) -> String {
        serde_json::to_string_pretty(&self.plan).expect("a plan serializes") + "\n"
    }

    /// The line a recording prints: what was added, what the plan now holds,
    /// and the command that applies it. `default_plan`: `--plan` was given
    /// without a value, so the apply line spells it bare (D16).
    pub fn summary(&self, file: &str, plan: &str, default_plan: bool, dry_run: bool) -> String {
        let ids = |side: fn(&super::ResolveRevisions) -> &Option<crate::changes::ChangeFilter>| {
            self.plan
                .resolve_revisions
                .as_ref()
                .and_then(|r| side(r).as_ref())
                .and_then(|f| f.ids.as_ref())
                .map_or(0, Vec::len)
        };
        let (accept, reject) = (ids(|r| &r.accept), ids(|r| &r.reject));
        let total = self.plan.operations.len();
        let s = |n: usize| if n == 1 { "" } else { "s" };
        // What this recording added: its operations, its revisions, or both.
        let operations = format!("{} operation{}", self.added, s(self.added));
        let revisions = format!(
            "{} revision{}",
            self.added_revisions,
            s(self.added_revisions)
        );
        let added = match (self.added, self.added_revisions) {
            (_, 0) => operations,
            (0, _) => revisions,
            _ => format!("{operations} and {revisions}"),
        };
        let apply = if default_plan {
            "--plan".to_string()
        } else {
            format!("--plan {plan}")
        };
        format!(
            "{} {added} in {plan}: {total} operation{}, {accept} revision{} to accept, {reject} to reject; apply with: jubarte edit {file} {apply}",
            if dry_run { "would record" } else { "recorded" },
            s(total),
            s(accept),
        )
    }
}

/// Where a recording is written: the file a symbolic link names (the link
/// stays a link), else `plan` itself. Only the native publish path calls
/// it; wasm32 writes no files.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn plan_target(plan: &Path) -> PathBuf {
    if let Ok(real) = std::fs::canonicalize(plan) {
        return real;
    }
    // A link to a plan not written yet names where it goes.
    match std::fs::read_link(plan) {
        Ok(to) => plan.parent().unwrap_or(Path::new("")).join(to),
        Err(_) => plan.to_path_buf(),
    }
}

/// Legacy temporary-name helper, retained for API compatibility. Recordings
/// no longer open this predictable path; publication creates exclusive staging.
/// Concurrent recordings remain last-writer-wins, not a serialized append.
pub fn partial_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    target.with_file_name(format!(".{name}.{}.partial", std::process::id()))
}

/// Write `json` (a [`Recorded::json`]) to PLAN.json at `plan`, atomically:
/// a precommit error leaves the plan as it was. Returns staging cleanup warnings
/// after successful installation; publication errors retain their warning payload.
///
/// # Errors
///
/// The I/O error when the partial file cannot be written beside the plan or
/// renamed into place (a missing directory, no permission). After a successful
/// commit, a durability error explicitly reports the committed plan.
pub fn write_plan(plan: &Path, json: &str) -> std::io::Result<Vec<String>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let target = plan_target(plan);
        crate::publication::publish_paths(&[(target, json.as_bytes())], true)
            .map(|report| report.warnings)
            .map_err(|error| std::io::Error::new(error.source.kind(), error))
    }
    #[cfg(target_arch = "wasm32")]
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!(
            "cannot publish {} ({} bytes): filesystem publication is unavailable in WASM",
            plan.display(),
            json.len()
        ),
    ))
}

/// The operations `ops` recorded into `held` (PLAN.json's text, `None`
/// before the first recording), pinned to `source`'s SHA-256. `existing`
/// is `--existing-revisions` (`None` for `auto`). The caller previews the
/// result against `source` before writing it. Nothing is written here.
///
/// # Errors
///
/// [`RecordError::Usage`] when the flags describe no operation or an invalid
/// one, or disagree with the held plan (another source, author, date or
/// `existing_revisions`); [`RecordError::Plan`] when the held text is not a
/// plan.
pub fn record(
    verb: Verb,
    ops: &[FlagOp],
    author: &str,
    date: Option<&str>,
    existing: Option<ExistingRevisions>,
    source: &[u8],
    held: Option<&str>,
) -> Result<Recorded, RecordError> {
    let held = held
        .map(EditPlan::from_json)
        .transpose()
        .map_err(|e| RecordError::Plan(e.to_string()))?;
    let before = held.as_ref().map_or(0, |p| p.operations.len());
    // Each id once and in its short spelling, as `record_into` joins them:
    // `r1+2` is two ids, `r1` and `r2`, so a recording that names an id the
    // plan already holds adds nothing.
    let revisions = |plan: Option<&EditPlan>| -> std::collections::HashSet<String> {
        plan.and_then(|p| p.resolve_revisions.as_ref())
            .map(|r| {
                [&r.accept, &r.reject]
                    .into_iter()
                    .flatten()
                    .filter_map(|f| f.normalized().unwrap_or_else(|_| f.clone()).ids)
                    .flatten()
                    .collect()
            })
            .unwrap_or_default()
    };
    let revisions_before = revisions(held.as_ref());
    let mut built =
        plan_from_flags(verb, ops, author, date, existing, source).map_err(RecordError::Usage)?;
    built.plan.source_sha256 = Some(crate::inspect::source_sha256(source));
    let plan = record_into(held, built.plan).map_err(RecordError::Usage)?;
    Ok(Recorded {
        added: plan.operations.len() - before,
        added_revisions: revisions(Some(&plan)).difference(&revisions_before).count(),
        notes: renumber_notes(built.notes, before),
        plan,
    })
}

/// What [`record_json`] takes besides the operations: the command line's
/// options, as the Python and npm CLIs parse them.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RecordOptions {
    /// `--author` (default `Jubarte`).
    pub author: Option<String>,
    /// `--datetime`.
    pub date: Option<String>,
    /// `--existing-revisions`: `auto` (the default), `keep`, `accept`,
    /// `reject` or `refuse`.
    pub existing_revisions: Option<String>,
    /// FILE as the command line names it, for the summary.
    pub file: String,
    /// PLAN.json as the command line names it, for the summary.
    pub plan: String,
    /// `--plan` was given without a value: the summary prints it bare.
    pub default_plan: bool,
    /// `--dry-run`: the summary says "would record".
    pub dry_run: bool,
    /// `--rewrite-others-comments`: accepted and ignored.
    pub rewrite_others_comments: bool,
    /// `--keep-others`: the recorded plan carries `"others": "keep"`.
    pub keep_others: bool,
}

/// [`record`] over JSON, for the Python and npm CLIs: `verb` is `edit` or
/// `add`, `operations` the parsed command's `operations` array, `held`
/// PLAN.json's text, `options` a `RecordOptions` object. Returns `{plan,
/// json, notes, summary}`: `plan` to preview, `json` to write, `notes` and
/// `summary` to print.
///
/// # Errors
///
/// [`RecordError::Usage`] for an unknown verb, unparsable operations or
/// options, a bad date or `--existing-revisions` value, and every
/// [`record`] failure; [`RecordError::Plan`] when the held text is not a
/// plan.
pub fn record_json(
    verb: &str,
    operations: &str,
    source: &[u8],
    held: Option<&str>,
    options: &str,
) -> Result<String, RecordError> {
    let usage = |e: String| RecordError::Usage(e);
    let verb = match verb {
        "edit" => Verb::Edit,
        "add" => Verb::Add,
        other => return Err(usage(format!("unknown verb {other:?}: edit or add"))),
    };
    let ops: Vec<FlagOp> =
        serde_json::from_str(operations).map_err(|e| usage(format!("operations: {e}")))?;
    let options: RecordOptions =
        serde_json::from_str(options).map_err(|e| usage(format!("options: {e}")))?;
    // The CLI's --datetime is checked by its parser; the bindings' is here.
    let date = options
        .date
        .as_deref()
        .map(crate::util::datetime::parse_datetime)
        .transpose()
        .map_err(|e| usage(format!("date: {e}")))?;
    let existing = match options.existing_revisions.as_deref().unwrap_or("auto") {
        "auto" => None,
        "keep" => Some(ExistingRevisions::Keep),
        "accept" => Some(ExistingRevisions::Accept),
        "reject" => Some(ExistingRevisions::Reject),
        "refuse" => Some(ExistingRevisions::Refuse),
        other => {
            return Err(usage(format!(
                "existing revisions {other:?}: auto, keep, accept, reject or refuse"
            )));
        }
    };
    let author = options.author.as_deref().unwrap_or("Jubarte");
    let mut recorded = record(verb, &ops, author, date.as_deref(), existing, source, held)?;
    recorded.plan.rewrite_others_comments |= options.rewrite_others_comments;
    if options.keep_others {
        recorded.plan.others = super::Others::Keep;
    }
    Ok(serde_json::json!({
        "plan": serde_json::to_string(&recorded.plan).expect("a plan serializes"),
        "json": recorded.json(),
        "notes": recorded.notes,
        "summary": recorded.summary(
            &options.file,
            &options.plan,
            options.default_plan,
            options.dry_run,
        ),
    })
    .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewrite(at: &str, content: &str) -> FlagOp {
        FlagOp {
            at: at.into(),
            content: Some(content.into()),
            ..FlagOp::default()
        }
    }

    fn once(ops: &[FlagOp], held: Option<&str>) -> Result<Recorded, RecordError> {
        record(
            Verb::Edit,
            ops,
            "Ann",
            None,
            Some(ExistingRevisions::Keep),
            b"doc",
            held,
        )
    }

    #[test]
    fn a_recording_pins_the_source_and_counts_on_from_the_held_plan() {
        let first = once(&[rewrite("p1", "One.")], None).unwrap();
        assert_eq!(first.added, 1);
        assert_eq!(
            first.plan.source_sha256.as_deref(),
            Some(crate::inspect::source_sha256(b"doc").as_str())
        );
        assert_eq!(
            first.summary("a.docx", "plan.json", false, false),
            "recorded 1 operation in plan.json: 1 operation, 0 revisions to accept, 0 to reject; \
             apply with: jubarte edit a.docx --plan plan.json"
        );
        let second = once(
            &[rewrite("p2", "Two."), rewrite("p3", "Three.")],
            Some(&first.json()),
        )
        .unwrap();
        assert_eq!((second.added, second.plan.operations.len()), (2, 3));
        assert!(
            second
                .summary("a.docx", "plan.json", false, true)
                .starts_with("would record 2 operations in plan.json: 3 operations,")
        );
        assert!(first.json().ends_with("}\n") && !first.json().contains("null"));
    }

    #[test]
    fn a_recording_of_revisions_only_says_so() {
        let accept = FlagOp {
            at: "r2".into(),
            accept: true,
            ..FlagOp::default()
        };
        let both = once(&[rewrite("p1", "One."), accept.clone()], None).unwrap();
        assert!(
            both.summary("a.docx", "p.json", false, false).starts_with(
                "recorded 1 operation and 1 revision in p.json: 1 operation, 1 revision to accept,"
            ),
            "{}",
            both.summary("a.docx", "p.json", false, false)
        );
        let reject = FlagOp {
            at: "r3".into(),
            reject: true,
            ..FlagOp::default()
        };
        let only = once(&[reject], Some(&both.json())).unwrap();
        assert!(
            only.summary("a.docx", "p.json", false, false).starts_with(
                "recorded 1 revision in p.json: 1 operation, 1 revision to accept, 1 to reject"
            ),
            "{}",
            only.summary("a.docx", "p.json", false, false)
        );
    }

    /// Codex on #433: a held plan's ids count as `record_into` joins them,
    /// so `r1+2` held is two ids and a duplicate spelling one.
    #[test]
    fn recorded_revisions_count_each_long_id_once() {
        let held = |ids: &[&str]| {
            let mut plan: serde_json::Value =
                serde_json::from_str(&once(&[rewrite("p1", "One.")], None).unwrap().json())
                    .unwrap();
            plan["resolve_revisions"] = serde_json::json!({"accept": {"ids": ids}});
            plan.to_string()
        };
        let accept = |at: &str| FlagOp {
            at: at.into(),
            accept: true,
            ..FlagOp::default()
        };
        let compound = once(&[accept("r3")], Some(&held(&["r1+2"]))).unwrap();
        assert_eq!(compound.added_revisions, 1);
        let twice = once(&[accept("r1")], Some(&held(&["r1", "r1"]))).unwrap();
        assert_eq!(twice.added_revisions, 0);
    }

    #[test]
    fn a_held_file_that_is_not_a_plan_is_a_plan_error() {
        assert!(matches!(
            once(&[rewrite("p1", "One.")], Some("{}")),
            Err(RecordError::Plan(_))
        ));
        assert!(matches!(
            once(&[], None),
            Err(RecordError::Usage(m)) if m.starts_with("no operation")
        ));
    }

    #[test]
    fn flags_are_refused_on_the_side_of_plan_they_do_not_belong_to() {
        assert_eq!(
            plan_flag_conflict(true, &["--quiet", "--pdf", "--out-dir"], "p.json").unwrap(),
            "--out-dir takes effect when the plan is applied (jubarte edit FILE --plan p.json), and is not recorded"
        );
        assert_eq!(
            plan_flag_conflict(true, &["--in-place"], "p.json").unwrap(),
            "--in-place takes effect when the plan is applied (jubarte edit FILE --plan p.json), and is not recorded"
        );
        assert_eq!(
            plan_flag_conflict(true, &["--author", "--dry-run"], "p.json"),
            None
        );
        assert_eq!(
            plan_flag_conflict(false, &["--consume", "--datetime"], "p.json").unwrap(),
            "--datetime belongs to recorded operations; the plan names its own"
        );
        assert_eq!(
            plan_flag_conflict(false, &["--consume", "--out-dir"], "p.json"),
            None
        );
    }

    #[test]
    fn a_plan_is_written_through_its_link_and_leaves_no_partial() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("v3.json");
        std::fs::write(&real, "{}").unwrap();
        let link = dir.path().join("plan.json");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real, &link).unwrap();
        #[cfg(not(unix))]
        std::fs::copy(&real, &link).unwrap();
        write_plan(&link, "[1]\n").unwrap();
        assert_eq!(std::fs::read_to_string(&link).unwrap(), "[1]\n");
        #[cfg(unix)]
        {
            assert!(link.symlink_metadata().unwrap().is_symlink());
            assert_eq!(std::fs::read_to_string(&real).unwrap(), "[1]\n");
        }
        let left: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(left.len(), 2, "{left:?}");
        // A link to a plan not written yet creates the file it names.
        #[cfg(unix)]
        {
            let dangling = dir.path().join("next.json");
            std::os::unix::fs::symlink("plans-v4.json", &dangling).unwrap();
            write_plan(&dangling, "[2]\n").unwrap();
            assert!(dangling.symlink_metadata().unwrap().is_symlink());
            assert_eq!(
                std::fs::read_to_string(dir.path().join("plans-v4.json")).unwrap(),
                "[2]\n"
            );
            std::fs::remove_file(&dangling).unwrap();
            std::fs::remove_file(dir.path().join("plans-v4.json")).unwrap();
        }
        // A plan in a directory that is gone fails and leaves nothing.
        assert!(write_plan(&dir.path().join("gone/plan.json"), "x").is_err());
        assert!(
            partial_path(Path::new("a/x.json"))
                .to_string_lossy()
                .ends_with(&format!(".x.json.{}.partial", std::process::id()))
        );
    }

    #[test]
    fn record_json_reads_the_options_the_bindings_pass() {
        let out = record_json(
            "add",
            r#"[{"at":"p1","comment":true,"content":"Why?"}]"#,
            b"doc",
            None,
            r#"{"author":"Ann","existingRevisions":"keep","file":"a.docx","plan":"p.json"}"#,
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["notes"], serde_json::json!([]));
        assert!(
            v["summary"]
                .as_str()
                .unwrap()
                .starts_with("recorded 1 operation in p.json")
        );
        let plan = EditPlan::from_json(v["plan"].as_str().unwrap()).unwrap();
        assert_eq!(plan.author, "Ann");
        assert_eq!(
            EditPlan::from_json(v["json"].as_str().unwrap()).unwrap(),
            plan
        );
        for (ops, options) in [
            (
                r#"[{"at":"p1","comment":true,"content":"Why?"}]"#,
                r#"{"date":"yesterday","file":"a","plan":"p"}"#,
            ),
            (
                r#"[{"at":"p1","comment":true,"content":"Why?","style":["bold"]}]"#,
                r#"{"file":"a","plan":"p"}"#,
            ),
        ] {
            assert!(matches!(
                record_json("add", ops, b"doc", None, options),
                Err(RecordError::Usage(_))
            ));
        }
        assert!(matches!(
            record_json("move", "[]", b"", None, r#"{"file":"a","plan":"p"}"#),
            Err(RecordError::Usage(m)) if m.contains("edit or add")
        ));
        assert!(matches!(
            record_json("edit", "[]", b"", None, r#"{"file":"a","plan":"p","existingRevisions":"x"}"#),
            Err(RecordError::Usage(m)) if m.contains("auto, keep")
        ));
    }
}
