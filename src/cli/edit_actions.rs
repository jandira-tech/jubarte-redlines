// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Typed edit grammar. The legacy `edit FILE` grammar remains versioned.

use std::{ffi::OsString, path::PathBuf};

use clap::{Args, Subcommand};
use serde::Serialize;

/// The typed edit route, selected before legacy flag grouping.
#[derive(Args, Debug, Serialize)]
pub struct TypedEdit {
    /// Exactly one operation grammar, or application of a saved plan.
    #[command(subcommand)]
    #[serde(flatten)]
    pub action: EditAction,
}

/// The initial PlanV2 text action profile.
#[derive(Subcommand, Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum EditAction {
    /// Replace exact literal text; without --at, search one declared story.
    #[command(
        override_usage = "jubarte edit replace [OPTIONS] <FILE> --find TEXT (--text TEXT|--text-file FILE)"
    )]
    Replace {
        /// Frozen policy and new-edit attribution.
        #[command(flatten)]
        inline: InlineOptions,
        /// Exact nonempty literal text.
        #[arg(long)]
        find: String,
        /// Original paragraph ID from read; never silently retargeted.
        #[arg(long)]
        at: Option<String>,
        /// Search story when --at is omitted (default body).
        #[arg(long, conflicts_with = "at")]
        story: Option<String>,
        /// 1-based exact match, including overlapping candidate starts.
        #[arg(long, value_parser = positive, conflicts_with = "all_matches")]
        occurrence: Option<usize>,
        /// Replace every candidate; refuses when any two candidates overlap. Requires --expect-count.
        #[arg(long, requires = "expect_count")]
        all_matches: bool,
        /// Required complete candidate count for --all-matches.
        #[arg(long, requires = "all_matches", value_parser = positive)]
        expect_count: Option<usize>,
        /// Exact text and optional attached comment.
        #[command(flatten)]
        payload: TextPayload,
    },
    /// Set exact desired paragraph text; an empty value retains the paragraph.
    #[command(
        override_usage = "jubarte edit rewrite [OPTIONS] <FILE> --at ID (--text TEXT|--text-file FILE)"
    )]
    Rewrite {
        /// Frozen policy and new-edit attribution.
        #[command(flatten)]
        inline: InlineOptions,
        /// Original paragraph ID.
        #[arg(long)]
        at: String,
        /// Exact text and optional attached comment.
        #[command(flatten)]
        payload: TextPayload,
    },
    /// Insert an explicitly typed text or paragraph payload.
    #[command(
        override_usage = "jubarte edit insert [OPTIONS] <FILE> --kind text|paragraph --at ID (--text TEXT|--text-file FILE)"
    )]
    Insert {
        /// Frozen policy and new-edit attribution.
        #[command(flatten)]
        inline: InlineOptions,
        /// Object type; position never infers the kind.
        #[arg(long, value_parser = ["text", "paragraph"])]
        kind: String,
        /// Original paragraph ID.
        #[arg(long)]
        at: String,
        /// Insert text before this exact nonempty literal.
        #[arg(long, conflicts_with_all = ["after_text", "edge", "position"])]
        before_text: Option<String>,
        /// Insert text after this exact nonempty literal.
        #[arg(long, conflicts_with_all = ["edge", "position"])]
        after_text: Option<String>,
        /// Insert text at a paragraph edge.
        #[arg(long, value_parser = ["start", "end"], conflicts_with_all = ["position", "occurrence"])]
        edge: Option<String>,
        /// Insert a paragraph beside the target paragraph.
        #[arg(long, value_parser = ["before", "after"], conflicts_with = "occurrence")]
        position: Option<String>,
        /// 1-based literal candidate.
        #[arg(long, value_parser = positive)]
        occurrence: Option<usize>,
        /// Exact text and optional attached comment.
        #[command(flatten)]
        payload: TextPayload,
    },
    /// Delete one exact range, or explicitly delete a whole paragraph.
    #[command(
        override_usage = "jubarte edit delete [OPTIONS] <FILE> --at ID (--find TEXT|--whole-block)"
    )]
    Delete {
        /// Frozen policy and new-edit attribution.
        #[command(flatten)]
        inline: InlineOptions,
        /// Original paragraph ID; mandatory even for a range deletion.
        #[arg(long)]
        at: String,
        /// Exact nonempty literal range to remove.
        #[arg(
            long,
            required_unless_present = "whole_block",
            conflicts_with = "whole_block"
        )]
        find: Option<String>,
        /// Explicit structural deletion (PlanV2 delete_paragraph).
        #[arg(long, required_unless_present = "find", conflicts_with = "occurrence")]
        whole_block: bool,
        /// 1-based literal candidate.
        #[arg(long, value_parser = positive, requires = "find")]
        occurrence: Option<usize>,
    },
    /// Apply only: never record into, consume, or rewrite the plan.
    // apply has no --save-plan, so its output is required unless it bundles
    // or dry-runs.
    #[command(
        override_usage = "jubarte edit apply [OPTIONS] <FILE> --plan FILE",
        mut_arg("output", |a| a.help("Primary edited DOCX; required unless bundling or dry-running"))
    )]
    Apply {
        /// Immutable source document.
        file: PathBuf,
        /// Input plan; v2 policy and attribution belong exclusively to it.
        #[arg(long, value_name = "FILE", value_parser = super::plan_path())]
        plan: PathBuf,
        /// Advanced v1-only waiver when neither plan nor CLI supplies a hash.
        #[arg(long)]
        allow_unbound_plan: bool,
        /// Publication and source preconditions.
        #[command(flatten)]
        delivery: DeliveryOptions,
    },
}

/// Exact text input; file bytes must be UTF-8 and are never trimmed.
#[derive(Args, Debug, Serialize)]
#[group(skip)]
pub struct TextPayload {
    /// Exact plain text (not Markdown); empty text is meaningful.
    #[arg(
        long,
        required_unless_present = "text_file",
        conflicts_with = "text_file"
    )]
    pub text: Option<String>,
    /// Exact UTF-8 file; all trailing newlines are preserved.
    #[arg(long, value_name = "FILE", required_unless_present = "text")]
    pub text_file: Option<PathBuf>,
    /// Attach a comment to the nonempty resulting text.
    #[arg(long)]
    pub comment: Option<String>,
}

/// Policy and attribution for a newly compiled inline plan.
#[derive(Args, Debug, Serialize)]
pub struct InlineOptions {
    /// Immutable source document.
    pub file: PathBuf,
    /// New revision and comment author.
    #[arg(short = 'a', long, default_value = "Jubarte", value_parser = super::parse_author)]
    pub author: String,
    /// Frozen timestamp; timezone required. Default current UTC.
    #[arg(short = 'd', long = "datetime", value_parser = super::parse_datetime)]
    pub datetime: Option<String>,
    /// Track new edits, preserving earlier history according to policy.
    #[arg(long, default_value = "on", value_parser = ["on", "off"])]
    pub track: String,
    /// Policy for history remaining after explicit selected decisions.
    #[arg(long, default_value = "keep", value_parser = ["keep", "accept", "reject", "refuse"])]
    pub existing_revisions: String,
    /// Original-source revision ID to accept; repeat to select a set.
    #[arg(long)]
    pub accept_change: Vec<String>,
    /// Original-source revision ID to reject; repeat to select a set.
    #[arg(long)]
    pub reject_change: Vec<String>,
    /// Save a source-bound PlanV2 without applying. Freezes targets, policy and time.
    #[arg(long, value_name = "FILE", value_parser = super::plan_path(), conflicts_with_all = ["output", "out_dir", "report", "emit", "expect", "dry_run"])]
    pub save_plan: Option<PathBuf>,
    /// Publication and source preconditions.
    #[command(flatten)]
    pub delivery: DeliveryOptions,
}

/// Publication and preparation options; never serialized into a saved plan.
#[derive(Args, Debug, Serialize)]
pub struct DeliveryOptions {
    /// Primary edited DOCX; required unless saving, bundling or dry-running.
    #[arg(short = 'o', long, value_name = "FILE", conflicts_with = "out_dir")]
    pub output: Option<PathBuf>,
    /// Publish a new bundle directory (document.docx, changes.diff, report.json).
    #[arg(long, value_name = "NEW_DIR")]
    pub out_dir: Option<PathBuf>,
    /// JSON receipt, irrespective of filename suffix.
    #[arg(long, value_name = "FILE")]
    pub report: Option<PathBuf>,
    /// Bundle roles: document,patch,report (primary document always included).
    #[arg(long, value_delimiter = ',', value_parser = ["document", "patch", "report"])]
    pub emit: Vec<String>,
    /// Optional expectation contract, evaluated on the prepared primary
    /// before publication (PlanV2 requests only; use `jubarte check` for a
    /// standalone pass over a published output).
    #[arg(long, value_name = "FILE")]
    pub expect: Option<PathBuf>,
    /// Redundant source digest must agree with a plan's embedded digest.
    #[arg(long, value_name = "HASH", value_parser = digest)]
    pub expect_sha256: Option<String>,
    /// Prepare and validate the actual candidate; write no files.
    #[arg(long, conflicts_with_all = ["output", "out_dir"])]
    pub dry_run: bool,
    /// Require a primary document change, independently of derived artifacts.
    #[arg(long)]
    pub require_change: bool,
    /// Replace an explicitly named existing output; never overwrite an input.
    #[arg(long)]
    pub force: bool,
    /// Refuse to accept or reject another author's tracked change, and
    /// to delete, resolve, reopen or reword their comment. Restore,
    /// strike, a comment and a reply stay allowed. JUBARTE_KEEP_OTHERS=1
    /// sets the same guard. Another author is anyone other than
    /// --author (Jubarte when --author is left out).
    #[arg(long)]
    pub keep_others: bool,
    /// One result envelope, including empty results and refusals.
    #[arg(long, conflicts_with = "jsonl")]
    pub json: bool,
    /// Header and exactly one terminal result.
    #[arg(long)]
    pub jsonl: bool,
}

pub(super) fn positive(s: &str) -> Result<usize, String> {
    s.parse()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| "must be a positive, 1-based integer".into())
}
pub(super) fn digest(s: &str) -> Result<String, String> {
    if s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(s.into())
    } else {
        Err("must be 64 lowercase hexadecimal characters".into())
    }
}

/// Route typed actions only when the tokens are not a valid legacy invocation.
/// No filesystem probing: a filename cannot change parsing by appearing/disappearing.
/// `./replace` always denotes a legacy filename; bare `replace -p ...` remains v1.
pub fn route(arguments: &[OsString], model: &clap::Command) -> Vec<OsString> {
    let mut routed = arguments.to_vec();
    if arguments.get(1).is_none_or(|a| a != "edit") {
        return routed;
    }
    let action = arguments.iter().skip(2).find(|a| {
        !a.to_str().is_some_and(|s| {
            s == "--quiet" || s.starts_with('-') && s.len() > 1 && s[1..].bytes().all(|b| b == b'q')
        })
    });
    if !action.is_some_and(|a| {
        matches!(
            a.to_str(),
            Some("replace" | "rewrite" | "insert" | "delete" | "apply" | "template")
        )
    }) {
        return routed;
    }
    if model.clone().try_get_matches_from(arguments).is_ok() {
        return routed;
    }
    if action.is_some_and(|a| a == "template") {
        routed[1] = "__edit-template".into();
        return routed;
    }
    routed[1] = "__edit-v2".into();
    routed
}

pub(super) fn validate(
    args: &clap::ArgMatches,
    model: &mut clap::Command,
) -> Result<(), clap::Error> {
    let (action, args) = args.subcommand().expect("typed action required");
    let present = |id| {
        args.try_contains_id(id).unwrap_or(false)
            && args.value_source(id) == Some(clap::parser::ValueSource::CommandLine)
    };
    let error = |message: &str| {
        model
            .clone()
            .error(clap::error::ErrorKind::ArgumentConflict, message)
    };
    if !["output", "out_dir", "dry_run", "save_plan"]
        .into_iter()
        .any(present)
    {
        return Err(error(
            "writes require -o FILE or --out-dir NEW_DIR; use --dry-run or inline --save-plan FILE",
        ));
    }
    if action == "insert" {
        let paragraph = args
            .get_one::<String>("kind")
            .is_some_and(|s| s == "paragraph");
        if paragraph != present("position")
            || !paragraph
                && !["before_text", "after_text", "edge"]
                    .into_iter()
                    .any(present)
        {
            return Err(error(
                "--kind paragraph requires --position before|after; --kind text requires --before-text, --after-text or --edge",
            ));
        }
    }
    Ok(())
}

impl EditAction {
    /// Canonical action name for receipts.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Replace { .. } => "replace",
            Self::Rewrite { .. } => "rewrite",
            Self::Insert { .. } => "insert",
            Self::Delete { .. } => "delete",
            Self::Apply { .. } => "apply",
        }
    }
    /// Inline policy, when this action compiles a new plan.
    pub fn inline(&self) -> Option<&InlineOptions> {
        match self {
            Self::Replace { inline, .. }
            | Self::Rewrite { inline, .. }
            | Self::Insert { inline, .. }
            | Self::Delete { inline, .. } => Some(inline),
            Self::Apply { .. } => None,
        }
    }
    /// Exact payload or resource input for actions with text.
    pub fn payload(&self) -> Option<&TextPayload> {
        match self {
            Self::Replace { payload, .. }
            | Self::Rewrite { payload, .. }
            | Self::Insert { payload, .. } => Some(payload),
            _ => None,
        }
    }
    /// Source path.
    pub fn file(&self) -> &std::path::Path {
        match self {
            Self::Apply { file, .. } => file,
            _ => &self.inline().expect("inline").file,
        }
    }
    /// Delivery options, independent of plan-owned policy.
    pub fn delivery(&self) -> &DeliveryOptions {
        match self {
            Self::Apply { delivery, .. } => delivery,
            _ => &self.inline().expect("inline").delivery,
        }
    }
}
