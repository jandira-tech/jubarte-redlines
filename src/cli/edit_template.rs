// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Generator arguments, kept separate from apply's document publication flags.
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Internal native routing wrapper; the public spelling is edit template.
#[derive(Args, Debug, Serialize)]
pub struct TemplateDispatch {
    /// Generator action, separate from the legacy edit FILE grammar.
    #[command(subcommand)]
    #[serde(flatten)]
    pub action: TemplateAction,
}

/// A template is an authoring action, with JSON output rather than DOCX delivery.
#[derive(Subcommand, Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum TemplateAction {
    /// Generate a source-bound JSON plan without applying edits.
    #[command(override_usage = "jubarte edit template [OPTIONS] <FILE> (-o PLAN.json|--dry-run)")]
    Template(TemplateOptions),
}

/// Generate unchanged desired-text rows; deliberately author edits before apply.
#[derive(Args, Debug, Serialize)]
#[command(
    after_help = "Change a row's text to request a rewrite. Keeping or removing an unchanged row requests no rewrite.\nUse explicit typed operations for insertions, deletions and comments.\n\nExamples:\n  jubarte edit template contract.docx --select p12,p20-p25 -o edits.json\n  jubarte edit template contract.docx --story all -o edits.json\n  jubarte edit apply contract.docx --plan edits.json -o revised.docx --json"
)]
pub struct TemplateOptions {
    /// Original DOCX. Generation never modifies it.
    pub file: PathBuf,
    /// Complete PlanV2 JSON; explicit - writes only the raw plan to stdout.
    #[arg(short, long, value_name = "PLAN.json")]
    pub output: Option<PathBuf>,
    /// Printed paragraph/container IDs or closed ranges, frozen on source S.
    #[arg(long, value_name = "IDS")]
    pub select: Option<String>,
    /// body, a printed story ID, or all (default: all).
    #[arg(long, default_value = "all")]
    pub story: String,
    /// Literal search of original-source final logical text.
    #[arg(long, conflicts_with = "regex")]
    pub find: Option<String>,
    /// Explicit regex search of original-source final logical text.
    #[arg(long)]
    pub regex: Option<String>,
    /// Search only; editing remains exact.
    #[arg(long)]
    pub ignore_case: bool,
    /// Blocks intersecting original revisions/comments.
    #[arg(long)]
    pub changed: bool,
    /// Existing-mark author filter; repeat for a union of authors. Requires --changed.
    #[arg(long, requires = "changed", value_parser = author_filter)]
    pub by: Vec<String>,
    /// Read-only original-source neighbours; never additional editing scope.
    #[arg(long, default_value_t = 0)]
    pub context: usize,
    /// Original revision ID to accept in the saved plan; repeatable.
    #[arg(long)]
    pub accept_change: Vec<String>,
    /// Original revision ID to reject in the saved plan; repeatable.
    #[arg(long)]
    pub reject_change: Vec<String>,
    /// Author for later revisions/comments.
    #[arg(short = 'a', long, default_value = "Jubarte", value_parser = super::parse_author)]
    pub author: String,
    /// Frozen timestamp; default current UTC.
    #[arg(short = 'd', long, value_parser = super::parse_datetime)]
    pub datetime: Option<String>,
    /// Track new edits when the plan is applied.
    #[arg(long, default_value = "on", value_parser = ["on", "off"])]
    pub track: String,
    /// Remaining history after the explicitly selected decisions.
    #[arg(long, default_value = "keep", value_parser = ["keep", "accept", "reject", "refuse"])]
    pub existing_revisions: String,
    /// Refuse to accept or reject another author's tracked change, and
    /// to delete, resolve, reopen or reword their comment. Restore,
    /// strike, a comment and a reply stay allowed. JUBARTE_KEEP_OTHERS=1
    /// sets the same guard. Another author is anyone other than
    /// --author (Jubarte when --author is left out). The generated plan
    /// carries `"others": "keep"`.
    #[arg(long)]
    pub keep_others: bool,
    /// Require these exact source bytes.
    #[arg(long, value_name = "HASH", value_parser = super::edit_actions::digest)]
    pub expect_sha256: Option<String>,
    /// Report coverage and readiness; write no files. No -o required.
    #[arg(long)]
    pub dry_run: bool,
    /// Explicit receipt path (not another plan); not written during dry-run.
    #[arg(long, value_name = "FILE")]
    pub report: Option<PathBuf>,
    /// Replace only an explicitly named existing output; never an input alias.
    #[arg(long)]
    pub force: bool,
    /// One compact result envelope; conflicts with -o -.
    #[arg(long, conflicts_with = "jsonl")]
    pub json: bool,
    /// Result records ending in a terminal result; never a plan encoding.
    #[arg(long)]
    pub jsonl: bool,
}

pub(super) fn validate(
    args: &clap::ArgMatches,
    model: &mut clap::Command,
) -> Result<(), clap::Error> {
    let (_, args) = args.subcommand().expect("template action required");
    let error = |message| {
        model
            .clone()
            .error(clap::error::ErrorKind::ArgumentConflict, message)
    };
    let output = args.get_one::<PathBuf>("output");
    let raw = output.is_some_and(|p| p == std::path::Path::new("-"));
    if output.is_none() && !args.get_flag("dry_run") {
        return Err(error(
            "template generation requires -o PLAN.json, explicit -o -, or --dry-run",
        ));
    }
    if raw
        && ["json", "jsonl", "dry_run"]
            .iter()
            .any(|flag| args.get_flag(flag))
    {
        return Err(error(
            "-o - writes only raw PlanV2 JSON; it conflicts with --json, --jsonl and --dry-run",
        ));
    }
    Ok(())
}

fn author_filter(value: &str) -> Result<String, String> {
    crate::markdown::author(value).map(str::to_string)
}
