// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Shared, platform-neutral command declarations and parsing. No runtime I/O.

pub use crate::edit::flags::FlagOp;
/// Typed edit action grammar, independent of legacy flag grouping.
pub mod edit_actions;
/// Desired-text template generation grammar.
pub mod edit_template;
use clap::{CommandFactory, FromArgMatches, Parser};
use serde::Serialize;
use std::path::{Path, PathBuf};

pub(crate) use crate::util::datetime::parse_datetime;

/// Read, edit, compare and render Word documents.
#[derive(Parser, Debug, Serialize)]
#[command(
    name = "jubarte", version,
    about = "Read, edit, compare and render Word documents",
    long_about = None,
    styles = styles(),
    term_width = 88,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    arg_required_else_help = true,
    subcommand_help_heading = "Tasks",
    after_help = "Examples:\n  jubarte compare old.docx new.docx -o redline.docx\n  jubarte contract.docx                    print the agent view (read)\n  jubarte old.docx new.docx                redline printed as the agent view (-o writes it)\n  jubarte read contract.docx --json\n  jubarte diff old.docx new.docx --format github\n  jubarte convert contract.docx -o contract.pdf\n\nRun jubarte <task> --help for task options.",
)]
pub struct Cli {
    /// Subcommand (e.g. `revisions`); plain compare when omitted.
    #[command(subcommand)]
    pub command: Option<Command>,

    /// No status on success: no `wrote` lines, SUCCESS block or edit summary.
    /// Files are still written; a reading task
    /// (`read`, `changes`, `compare` without -o, `diff`, `capabilities`)
    /// still prints what it read, and warnings and errors still print.
    /// `read`, `changes` and `comments` drop their one stderr line naming
    /// the input's validity findings; under --json and read --for-edit the
    /// output counts them, -q or not.
    #[arg(
        short = 'q',
        long,
        global = true,
        overrides_with = "quiet",
        help_heading = None::<&str>
    )]
    pub quiet: bool,

    /// Inputs and settings of the shorthands: `jubarte A B` is `compare A
    /// B`; `jubarte FILE` is `read FILE`, with `compare.read`'s options.
    #[command(flatten)]
    #[serde(flatten)]
    pub compare: CompareArgs,
}

/// Options of `read`.
#[derive(clap::Args, Debug, Default, PartialEq, Serialize)]
#[group(id = "read_options", multiple = true)]
pub struct ReadArgs {
    /// Tracked changes inline (all, the default), or the text with every
    /// change accepted or rejected (also spelled accepted, rejected); the
    /// id lines then list what changed. --json and --tables read the
    /// accepted text, or under reject the rejected one, every paragraph
    /// keeping its id; the snapshot names an accept or a reject in
    /// `track_changes`.
    #[arg(long, value_enum, value_name = "CHOICE", help_heading = "Read options")]
    pub track_changes: Option<TrackChanges>,
    /// Comments inline (default) or hidden, with their ids on the id line
    /// of the paragraph that holds them; ignored by --json, --for-edit and
    /// the --tables grids.
    #[arg(
        long,
        value_enum,
        default_value_t = CommentsArg::Inline,
        value_name = "MODE",
        help_heading = "Read options"
    )]
    pub comments: CommentsArg,
    /// Timestamps on the notes of an author whose changes do not all share
    /// one (the header shows an author's single timestamp); ignored by
    /// --json, --for-edit and the --tables grids.
    #[arg(long, help_heading = "Read options")]
    pub dates: bool,
    /// Skip the layout pass; page count from Word's cached breaks, no
    /// `<!-- page N of M -->` lines; ignored by --json, --for-edit and the
    /// --tables grids.
    #[arg(long, help_heading = "Read options")]
    pub no_page_markers: bool,
    /// Only these blocks: `p5`, `p4-p7`, `t0`, comma-separated (open-ended
    /// `p17-` / `-p3` spellings still parse, transitionally; the 17 October
    /// 2026 contract retires them — spell the end, `p17-p20`). With --tables,
    /// select top-level tables (including their nested tables) by the same
    /// tN as the view, or by a pN inside them; a pick that reaches no table
    /// is refused. Print their grid, then a blank line and the same view as
    /// read -p SPEC.
    #[arg(
        short = 'p',
        long = "paragraphs",
        visible_alias = "select",
        value_name = "SPEC",
        conflicts_with_all = ["head", "tail"],
        help_heading = "Read options"
    )]
    pub paragraphs: Option<String>,
    /// Only the first N blocks (a table is one block).
    #[arg(
        long,
        value_name = "N",
        conflicts_with = "tail",
        help_heading = "Read options"
    )]
    pub head: Option<usize>,
    /// Only the last N blocks.
    #[arg(long, value_name = "N", help_heading = "Read options")]
    pub tail: Option<usize>,
    /// Only the blocks with a tracked change or a comment.
    #[arg(
        long,
        conflicts_with_all = ["paragraphs", "head", "tail"],
        help_heading = "Read options"
    )]
    pub changed: bool,
    /// With --changed: only the blocks with this author's marks (a handle
    /// such as AC, or the full name).
    #[arg(
        long,
        value_name = "AUTHOR",
        requires = "changed",
        value_parser = parse_author,
        help_heading = "Read options"
    )]
    pub by: Option<String>,
    /// Print the snapshot as JSON instead of the view: `schema_version`,
    /// `source_sha256`, `summary`, `paragraphs`, `stories`, `tables`,
    /// `controls`, `changes` and `comments`. -p, --head, --tail, --changed
    /// and --by keep the body paragraphs they select, and every change and
    /// comment that starts in one of them, whoever made it; the rest of the
    /// snapshot stays whole. --track-changes accept or reject settles the
    /// text.
    #[arg(long, help_heading = "Read options")]
    pub json: bool,
    /// Print each top-level body table as a grid: a `table N:
    /// ROWSxCOLS header_rows=H widths=W,...` line, then one line per row of
    /// tab-separated `ids=text` cells. Use -p t0,t2 or -p pN (a cell
    /// paragraph) to select top-level tables and all their nested tables,
    /// in document order, each once. tN is the view's top-level index;
    /// nested entries follow their cell row, indented two spaces per level,
    /// labelled `nested in t0.row2.cell0` (recursively, e.g.
    /// `t0.row2.cell0.t0.row0.cell1`). A pN or a range selects the tables
    /// that hold its paragraphs; one that reaches no table is refused.
    /// With -p, follow the selected grids with a blank line and the same
    /// view as read -p SPEC, including its YAML header. Without -p, print
    /// only the inventory. --track-changes reject reads the rejected text.
    #[arg(
        long,
        conflicts_with_all = ["json", "head", "tail", "changed", "by"],
        help_heading = "Read options"
    )]
    pub tables: bool,
    /// The working document template/apply derive, as JSON: source hash,
    /// paired target ids, exact logical working text with `before_sha256`
    /// and readiness per operation and range. The selection (-p/--select,
    /// --head, --tail, --changed, --by, --find, --regex) resolves on the
    /// original source and freezes before the decisions settle it. Given
    /// filters narrow together: a paragraph is selected only when every
    /// filter given matches it, and an empty result names the filters
    /// applied in `coverage.omissions`.
    #[arg(
        long,
        conflicts_with_all = ["json", "tables", "track_changes"],
        help_heading = "Read options"
    )]
    pub for_edit: bool,
    /// --for-edit: the working view after accepting this revision
    /// (repeatable; it does not modify FILE).
    #[arg(
        long,
        value_name = "ID",
        requires = "for_edit",
        help_heading = "Read options"
    )]
    pub accept_change: Vec<String>,
    /// --for-edit: the working view after rejecting this revision
    /// (repeatable; it does not modify FILE).
    #[arg(
        long,
        value_name = "ID",
        requires = "for_edit",
        help_heading = "Read options"
    )]
    pub reject_change: Vec<String>,
    /// --for-edit: the remaining-revision policy of the working basis
    /// (default keep, matching new plans).
    #[arg(
        long,
        value_enum,
        value_name = "MODE",
        requires = "for_edit",
        help_heading = "Read options"
    )]
    pub existing_revisions: Option<ExistingArg>,
    /// --for-edit: readiness for tracked (on, the default) or direct
    /// (off) new edits.
    #[arg(
        long,
        value_enum,
        value_name = "CHOICE",
        requires = "for_edit",
        help_heading = "Read options"
    )]
    pub track: Option<TrackArg>,
    /// --for-edit: literal search of the original final text. Combined
    /// filters narrow: a paragraph must match every filter given, and an
    /// empty result names the filters applied.
    #[arg(
        long,
        value_name = "TEXT",
        requires = "for_edit",
        conflicts_with = "regex",
        help_heading = "Read options"
    )]
    pub find: Option<String>,
    /// --for-edit: regex search of the original final text; conflicts
    /// with --find.
    #[arg(
        long,
        value_name = "PATTERN",
        requires = "for_edit",
        help_heading = "Read options"
    )]
    pub regex: Option<String>,
    /// --for-edit: case-insensitive --find/--regex.
    #[arg(long, requires = "for_edit", help_heading = "Read options")]
    pub ignore_case: bool,
    /// --for-edit: this many read-only original-source neighbours around
    /// each row.
    #[arg(
        long,
        value_name = "N",
        requires = "for_edit",
        help_heading = "Read options"
    )]
    pub context: Option<usize>,
    /// --for-edit: primary records a page returns (default 100).
    #[arg(
        long,
        value_name = "N",
        requires = "for_edit",
        help_heading = "Read options"
    )]
    pub limit: Option<usize>,
    /// --for-edit: the text budget a page shares (default 24000);
    /// a truncated row's preview is never complete text.
    #[arg(
        long,
        value_name = "N",
        requires = "for_edit",
        help_heading = "Read options"
    )]
    pub max_chars: Option<usize>,
    /// --for-edit: continue an earlier bounded query (bound to its
    /// source, selection, filters and working basis).
    #[arg(
        long,
        value_name = "TOKEN",
        requires = "for_edit",
        help_heading = "Read options"
    )]
    pub cursor: Option<String>,
    /// --for-edit: no presentation limits (admission limits still hold).
    #[arg(long, requires = "for_edit", help_heading = "Read options")]
    pub all: bool,
}

impl ReadArgs {
    /// The `read --for-edit` request these flags describe (V2-10): the
    /// working basis template and apply derive — `auto` (and the absence
    /// of the flag) keeps the remaining revisions, `on` tracks new edits.
    /// `read --for-edit` keeps the open-ended range spellings its help
    /// documents; the bindings build the same request through this.
    pub fn for_edit_request(&self) -> crate::edit::for_edit::ForEditRequest {
        crate::edit::for_edit::ForEditRequest {
            select: self.paragraphs.clone(),
            head: self.head,
            tail: self.tail,
            find: self.find.clone(),
            regex: self.regex.clone(),
            ignore_case: self.ignore_case,
            changed: self.changed,
            by: self.by.clone(),
            context: self.context.unwrap_or(0),
            limit: self.limit,
            max_chars: self.max_chars,
            all: self.all,
            cursor: self.cursor.clone(),
            accept: self.accept_change.clone(),
            reject: self.reject_change.clone(),
            existing_revisions: self
                .existing_revisions
                .and_then(ExistingArg::plan_value)
                .unwrap_or(crate::edit::ExistingRevisions::Keep),
            track: self.track.unwrap_or_default() == TrackArg::On,
            ..Default::default()
        }
    }
}

/// `--comments` of `read`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommentsArg {
    /// Comments inline, with their ids.
    #[default]
    Inline,
    /// Comments hidden; their ids on the id lines.
    None,
}

/// `--track` of `read --for-edit`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrackArg {
    /// The new edits are tracked.
    #[default]
    On,
    /// The new edits land directly.
    Off,
}

impl TrackArg {
    /// `--track`'s spelling (`on`, `off`); `None` for anything else.
    pub fn from_choice(choice: &str) -> Option<Self> {
        match choice {
            "on" => Some(Self::On),
            "off" => Some(Self::Off),
            _ => None,
        }
    }
}

/// Options `edit` and `add` share.
#[derive(clap::Args, Debug, Serialize)]
pub struct EditOptions {
    /// Author of the changes and comments (operation flags; a plan names
    /// its own).
    #[arg(short = 'a', long, value_name = "NAME", default_value = "Jubarte")]
    pub author: String,
    /// Revision timestamp (ISO 8601) [default: now, UTC]; pin it for
    /// reproducible output.
    #[arg(short = 'd', long = "datetime", value_name = "ISO8601", value_parser = parse_datetime)]
    pub datetime: Option<String>,
    /// The edits are tracked changes (the default): the file written is
    /// the redline and the view shows the marks.
    #[arg(long, conflicts_with = "editing_mode")]
    pub suggesting_mode: bool,
    /// The edits land directly: the file written is the result, and the
    /// view shows it with rev tags on the id lines.
    #[arg(long)]
    pub editing_mode: bool,
    /// What to do when FILE already holds tracked changes: auto keeps them
    /// and tracks the new edits beside them; a clean file goes through the
    /// comparer.
    #[arg(long, value_enum, value_name = "MODE", default_value_t = ExistingArg::Auto)]
    pub existing_revisions: ExistingArg,
    /// The edited document, a .docx (default: `<FILE's
    /// directory>/<stem>_modified.docx`). Any other extension is written as
    /// a .docx too; .md, .json, .pdf and .png name the task that writes them.
    #[arg(
        short = 'o',
        long,
        value_name = "FILE.docx",
        conflicts_with = "out_dir"
    )]
    pub output: Option<PathBuf>,
    /// Write the bundle instead: clean.docx, redline.docx, patch.diff and
    /// report.jsonl (clean.docx and report.jsonl in editing mode) in DIR
    /// (default: `<FILE's directory>/<stem>.edit`).
    #[arg(long, value_name = "DIR", num_args = 0..=1)]
    pub out_dir: Option<Option<PathBuf>>,
    /// Replace an existing output (the bundle directory's files).
    #[arg(long)]
    pub force: bool,
}

/// What `edit` and `add` write: one document, or the bundle directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditTarget {
    /// The redline (or, in editing mode, the result) as one .docx.
    Single(PathBuf),
    /// clean.docx, redline.docx, patch.diff and report.jsonl in a directory.
    Bundle(PathBuf),
}

impl EditOptions {
    /// Where `edit FILE` (or `add FILE`) writes: `--out-dir [DIR]`'s
    /// bundle, else `-o`, else `<FILE's dir>/<stem>_modified.docx`. A FILE
    /// whose stem already ends in `_modified` keeps its name. Pure.
    pub fn target(&self, file: &Path) -> EditTarget {
        let beside = |name: String| match file.parent().filter(|p| !p.as_os_str().is_empty()) {
            Some(dir) => dir.join(name),
            None => PathBuf::from(name),
        };
        let stem = file.file_stem().map_or_else(
            || "document".to_string(),
            |s| s.to_string_lossy().into_owned(),
        );
        match (&self.out_dir, &self.output) {
            (Some(Some(dir)), _) => EditTarget::Bundle(dir.clone()),
            (Some(None), _) => EditTarget::Bundle(beside(format!("{stem}.edit"))),
            (None, Some(output)) => EditTarget::Single(output.clone()),
            (None, None) if stem.ends_with("_modified") => {
                EditTarget::Single(beside(format!("{stem}.docx")))
            }
            (None, None) => EditTarget::Single(beside(format!("{stem}_modified.docx"))),
        }
    }
}

/// `--existing-revisions` of `edit` and `add`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExistingArg {
    /// `keep` when the file has tracked changes, else the comparer path.
    #[default]
    Auto,
    /// Keep them; the new edits are tracked beside them.
    Keep,
    /// Accept them first.
    Accept,
    /// Reject them first.
    Reject,
    /// Refuse a file that has them.
    Refuse,
}

impl ExistingArg {
    /// `--existing-revisions`'s spelling (`auto`, `keep`, `accept`,
    /// `reject`, `refuse`); `None` for anything else.
    pub fn from_choice(choice: &str) -> Option<Self> {
        match choice {
            "auto" => Some(Self::Auto),
            "keep" => Some(Self::Keep),
            "accept" => Some(Self::Accept),
            "reject" => Some(Self::Reject),
            "refuse" => Some(Self::Refuse),
            _ => None,
        }
    }

    /// The plan's `existing_revisions`; `None` for `auto`.
    pub fn plan_value(self) -> Option<crate::edit::ExistingRevisions> {
        use crate::edit::ExistingRevisions as E;
        match self {
            Self::Auto => None,
            Self::Keep => Some(E::Keep),
            Self::Accept => Some(E::Accept),
            Self::Reject => Some(E::Reject),
            Self::Refuse => Some(E::Refuse),
        }
    }
}

/// Groups the operation flags of an `edit` or `add` invocation by their
/// `-p`, in command-line order. `matches` is the subcommand's.
pub fn flag_operations(matches: &clap::ArgMatches) -> Result<Vec<FlagOp>, String> {
    let mut tokens: Vec<(usize, &str, Option<String>)> = Vec::new();
    for name in ["location", "anchor", "content", "reply", "style"] {
        if let (Some(indices), Some(values)) =
            (matches.indices_of(name), matches.get_many::<String>(name))
        {
            tokens.extend(indices.zip(values).map(|(i, v)| (i, name, Some(v.clone()))));
        }
    }
    for name in [
        "delete", "resolve", "before", "comment", "accept", "reject", "strike", "restore",
        "visible",
    ] {
        // `try_get_many` is `Err` for a flag this command does not declare
        // (`add` has no --resolve or --accept); `indices_of` would panic on
        // it.
        if matches.try_get_many::<bool>(name).ok().flatten().is_none() {
            continue;
        }
        if let Some(indices) = matches.indices_of(name) {
            tokens.extend(indices.map(|i| (i, name, None)));
        }
    }
    tokens.sort_by_key(|t| t.0);
    let mut ops: Vec<FlagOp> = Vec::new();
    for (_, name, value) in tokens {
        if name == "location" {
            ops.push(FlagOp {
                at: value.unwrap_or_default(),
                ..FlagOp::default()
            });
            continue;
        }
        let Some(op) = ops.last_mut() else {
            return Err(format!("--{name} comes before the first -p/--location"));
        };
        let twice = |what: &str| Err(format!("-p {}: --{what} given twice", op.at));
        match name {
            "anchor" if op.anchor.is_some() => return twice("anchor"),
            "anchor" => op.anchor = value,
            "content" if op.content.is_some() => return twice("content"),
            "content" => op.content = value,
            "reply" if op.reply.is_some() => return twice("reply"),
            "reply" => op.reply = value,
            "style" => op.styles.push(value.unwrap_or_default()),
            "delete" => op.delete = true,
            "resolve" => op.resolve = true,
            "before" => op.before = true,
            "comment" => op.comment = true,
            "accept" => op.accept = true,
            "reject" => op.reject = true,
            "strike" => op.strike = true,
            "restore" => op.restore = true,
            "visible" => op.visible = true,
            _ => {}
        }
    }
    Ok(ops)
}

/// `--plan`'s parser: a path that may be empty, the value `--plan` takes
/// when given bare (clap's `PathBuf` parser refuses an empty one). A path
/// ending in a separator names a folder, never the plan file, so it is
/// refused here, alike in the binary, Python and npm (Codex on #433: they
/// once wrote `new.json`, failed the rename, or refused).
fn plan_path() -> impl clap::builder::TypedValueParser<Value = PathBuf> {
    use clap::builder::TypedValueParser as _;
    clap::builder::OsStringValueParser::new().try_map(|value| {
        let text = value.to_string_lossy();
        if text.ends_with('/') || cfg!(windows) && text.ends_with('\\') {
            return Err(format!(
                "{text} ends in a separator, so it names a folder; give the plan's file, such as PLAN.json"
            ));
        }
        Ok(PathBuf::from(value))
    })
}

/// `-q`/`--quiet` before the task moves after it, so `jubarte -q validate F`
/// reads as `jubarte validate -q F` (D6 follow-up). Only the quiet flag
/// moves: every other option still goes after the task. Before `help` the
/// flag is dropped, since clap's `help` takes task names only.
fn hoist_quiet<T: AsRef<std::ffi::OsStr> + Clone>(
    command: &clap::Command,
    arguments: &[T],
) -> Vec<T> {
    let is_quiet = |a: &T| match a.as_ref().to_str() {
        Some("--quiet") => true,
        Some(short) => {
            short.len() > 1 && short.starts_with('-') && short[1..].bytes().all(|b| b == b'q')
        }
        None => false,
    };
    let flags = arguments.iter().skip(1).take_while(|a| is_quiet(a)).count();
    let task = arguments.get(1 + flags).and_then(|a| a.as_ref().to_str());
    if flags > 0 && task == Some("help") {
        // clap's own `help` takes task names, not -q: help is printed anyway.
        let mut help = vec![arguments[0].clone()];
        help.extend_from_slice(&arguments[1 + flags..]);
        return help;
    }
    if flags == 0 || task.is_none_or(|name| command.find_subcommand(name).is_none()) {
        return arguments.to_vec();
    }
    let mut hoisted = Vec::with_capacity(arguments.len());
    hoisted.push(arguments[0].clone());
    hoisted.push(arguments[1 + flags].clone());
    hoisted.extend_from_slice(&arguments[1..1 + flags]);
    hoisted.extend_from_slice(&arguments[2 + flags..]);
    hoisted
}

/// `compare`'s and `diff`'s missing `-d` is now, in UTC, to the second
/// (D17): filled once here, so the binary and every host share it.
fn fill_date(date: &mut Option<String>) {
    if date.is_none() {
        *date = Some(crate::convert::utc_now_iso8601());
    }
}

/// [`fill_date`] for the task that dates its revisions.
fn fill_task_date(task: &mut Command) {
    match task {
        Command::Compare(compare) => fill_date(&mut compare.datetime),
        Command::Diff { datetime: date, .. } => fill_date(date),
        _ => {}
    }
}

/// A path as the CLI's agent-facing outputs spell it: `/` between the
/// parts on every OS (#568). The paths `parse_json` resolves and the
/// view's `source:` line are identifiers an agent reads back, so their
/// spelling must not change with the OS; human status lines (`wrote …`)
/// keep the native path. A `\` inside a file name is replaced too: these
/// outputs name files the CLI itself resolved or wrote, whose names hold
/// none.
///
/// Canonicalizing on Windows returns a verbatim path (`\\?\C:\…`), which
/// this spelling would show as `//?/C:/…` — a string that is no path at
/// all (#1222). `unverbatimed` drops that prefix from the components
/// first, so an agent-facing canonical path spells `C:/…`.
pub fn slash_path(path: &Path) -> String {
    let path = unverbatimed(path).unwrap_or_else(|| path.to_path_buf());
    path.to_string_lossy().replace('\\', "/")
}

/// `path` without the verbatim prefix `canonicalize` adds on Windows, or
/// `None` when there is none to drop. Built on `Path::components()` so the
/// head is the parsed prefix, never a string match: `\\?\C:\a` loses its
/// `VerbatimDisk` head and comes back `C:\a`, and `\\?\UNC\server\share\x`
/// comes back `\\server\share\x`, which [`slash_path`] spells
/// `//server/share/x`. A `\\?\` path neither disk nor UNC keeps its head's
/// text as the path's start.
fn unverbatimed(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    let (mut stripped, keeps_root) = verbatim_head(components.next())?;
    for component in components {
        // A disk or UNC head keeps the root: `C:` + `\` is `C:\`, while
        // `C:` + `a` would be the drive-relative `C:a` (a bare drive takes
        // no separator). Any other `\\?\` head is no prefix once stripped,
        // so its root is dropped and the parts join onto the head's text.
        if keeps_root || !matches!(component, std::path::Component::RootDir) {
            stripped.push(component.as_os_str());
        }
    }
    Some(stripped)
}

/// The path head a verbatim prefix becomes, and whether the root after it
/// stays, or `None` for any other first component (`Prefix` is a Windows
/// notion; off Windows this is always `None` and [`unverbatimed`] changes
/// nothing).
#[cfg(windows)]
fn verbatim_head(component: Option<std::path::Component>) -> Option<(PathBuf, bool)> {
    use std::path::{Component, Prefix};
    match component {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::VerbatimDisk(letter) => Some((
                PathBuf::from(format!("{}:", (letter as char).to_ascii_uppercase())),
                true,
            )),
            Prefix::VerbatimUNC(server, share) => {
                let mut head = PathBuf::from(r"\\");
                head.push(server);
                head.push(share);
                Some((head, true))
            }
            Prefix::Verbatim(rest) => Some((PathBuf::from(rest), false)),
            _ => None,
        },
        _ => None,
    }
}

/// Off Windows no component carries a prefix, so there is nothing to drop.
#[cfg(not(windows))]
fn verbatim_head(_component: Option<std::path::Component>) -> Option<(PathBuf, bool)> {
    None
}

/// `--plan` without a value (D16) is `<FILE's dir>/<stem>_plan.json`.
fn resolve_plan(task: &mut Command) {
    let (Command::Edit {
        file,
        plan: Some(plan),
        plan_default,
        ..
    }
    | Command::Add {
        file,
        plan: Some(plan),
        plan_default,
        ..
    }) = task
    else {
        return;
    };
    if !plan.as_os_str().is_empty() {
        return;
    }
    let stem = file.file_stem().map_or_else(
        || "document".to_string(),
        |s| s.to_string_lossy().into_owned(),
    );
    let name = format!("{stem}_plan.json");
    *plan = match file.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(dir) => dir.join(name),
        None => PathBuf::from(name),
    };
    *plan_default = true;
}

/// A Word document's name, by extension: what an optional value swallowed
/// when it should have been FILE.
fn names_word_document(path: &Path) -> bool {
    // By name, so a bare `.docx` (no stem, no extension to Path) counts.
    path.file_name().is_some_and(|name| {
        let name = name.to_string_lossy().to_ascii_lowercase();
        [".docx", ".docm", ".dotx", ".dotm", ".doc"]
            .iter()
            .any(|ext| name.ends_with(ext))
    })
}

/// The message for an optional value (`--plan`, `--out-dir`) that names a
/// Word document: FILE was read as the value.
fn swallowed_file(flag: &str, value: &Path) -> String {
    let instead = if flag == "--plan" { "PLAN.json" } else { "DIR" };
    format!(
        "{flag} {} names a Word document; put FILE before {flag}, or write {flag}={instead}",
        value.display()
    )
}

/// Rewrite known usage traps through clap's own error formatter. A read
/// view conflict teaches the valid invocation; `edit --plan a.docx -p p1 …`
/// identifies the swallowed FILE from the raw tokens. Other errors stay
/// clap's own (including help, unknown options and invalid values).
fn usage_error(command: &clap::Command, tokens: &[String], mut error: clap::Error) -> clap::Error {
    if error.kind() == clap::error::ErrorKind::ArgumentConflict {
        use clap::error::ContextKind;
        // Use clap's identified arguments, not option-looking values in
        // the raw tokens, so unrelated conflicts retain their diagnosis.
        let conflicts = [ContextKind::InvalidArg, ContextKind::PriorArg]
            .into_iter()
            .filter_map(|kind| error.get(kind))
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        let names = |flag: &&str| conflicts.split_whitespace().any(|word| word == *flag);
        // --json and --tables take the view's options; only --for-edit,
        // which settles through its own --accept-change, refuses one.
        if names(&"--track-changes") && names(&"--for-edit") {
            return read_usage_error(command, read_view_conflict("--track-changes"));
        }
        if names(&"--tables") {
            error.insert(
                ContextKind::Usage,
                clap::error::ContextValue::StyledStr(
                    format!("Usage: {} read [OPTIONS] <FILE>", command.get_name()).into(),
                ),
            );
        }
    }
    if error.kind() != clap::error::ErrorKind::MissingRequiredArgument {
        return error;
    }
    let Some(at) = tokens.iter().skip(1).position(|t| !t.starts_with('-')) else {
        return error;
    };
    let name = tokens[at + 1].as_str();
    let Some(sub) = command
        .find_subcommand(name)
        .filter(|_| matches!(name, "edit" | "add"))
    else {
        return error;
    };
    // `--plan VALUE` or `--plan=VALUE`, before `--` ends the options.
    let options = tokens[at + 2..].iter().take_while(|t| *t != "--");
    let mut previous: Option<&str> = None;
    let mut found = None;
    for token in options {
        let pair = match token.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => Some((flag, value)),
            _ => previous.map(|flag| (flag, token.as_str())),
        };
        if let Some((flag, value)) = pair
            && matches!(flag, "--plan" | "--out-dir")
            && !value.starts_with('-')
            && names_word_document(Path::new(value))
        {
            found = Some(swallowed_file(flag, Path::new(value)));
            break;
        }
        previous = Some(token.as_str()).filter(|t| matches!(*t, "--plan" | "--out-dir"));
    }
    let Some(message) = found else {
        return error;
    };
    let mut task = sub.clone();
    task.set_bin_name(format!("{} {name}", command.get_name()));
    task.error(clap::error::ErrorKind::ArgumentConflict, message)
}

/// Groups the flag operations of `edit` and `add` into their `operations`
/// field; a grouping error is a usage error.
fn fill_flag_operations(
    task: &mut Command,
    matches: &clap::ArgMatches,
    model: &mut clap::Command,
) -> Result<(), clap::Error> {
    let (name, operations) = match task {
        Command::Edit { operations, .. } => ("edit", operations),
        Command::Add { operations, .. } => ("add", operations),
        _ => return Ok(()),
    };
    let Some(sub) = matches.subcommand_matches(name) else {
        return Ok(());
    };
    *operations = flag_operations(sub)
        .map_err(|m| model.error(clap::error::ErrorKind::ArgumentConflict, m))?;
    Ok(())
}

/// Inputs and settings shared by explicit and shorthand comparisons.
#[derive(clap::Args, Debug, Serialize)]
#[group(id = "compare_options", multiple = true)]
pub struct CompareArgs {
    /// The original / base document (.docx or Markdown).
    #[arg(value_name = "ORIGINAL", required_unless_present = "original")]
    pub original_pos: Option<PathBuf>,

    /// The modified document (.docx or Markdown).
    #[arg(value_name = "MODIFIED")]
    pub modified_pos: Option<PathBuf>,

    /// Original/base document (overrides the positional ORIGINAL).
    #[arg(short = 'b', long = "original", value_name = "FILE")]
    pub original: Option<PathBuf>,

    /// Modified document (overrides the positional MODIFIED).
    #[arg(short = 'm', long = "modified", value_name = "FILE")]
    pub modified: Option<PathBuf>,

    /// Write the redline to FILE instead of printing its agent view; the
    /// extension picks the format: .docx (also any other), .md (the agent
    /// view; CriticMarkup when both documents are Markdown), .pdf, .png
    /// (one file per page) or .json (the snapshot).
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Author name recorded on the revisions.
    #[arg(short = 'a', long, value_name = "NAME", default_value = "Jubarte")]
    pub author: String,

    /// Revision timestamp (ISO 8601) [default: now, UTC]; pin it for
    /// reproducible output.
    // The parser fills now when absent (D17), so every host shares it.
    #[arg(short = 'd', long = "datetime", value_name = "ISO8601", value_parser = parse_datetime)]
    pub datetime: Option<String>,

    /// Overwrite the output file if it already exists.
    #[arg(long)]
    pub force: bool,

    /// Word-match detail, from 0 to 1 [default: 0.02; powertools: 0.15].
    #[arg(long, value_name = "RATIO", value_parser = parse_threshold)]
    pub detail_threshold: Option<f64>,

    /// Compare like Microsoft Word or Open-Xml-PowerTools.
    #[arg(long, value_enum, value_name = "MODE", default_value_t = CompareMode::Word)]
    pub mode: CompareMode,

    /// Same as --mode powertools.
    #[arg(long, hide = true)]
    pub powertools_faithful: bool,

    /// DEBUG: zero WmlComparerSettings::merge_replaced_paragraphs — the
    /// word-visual UMBRELLA gate — which disables the WHOLE word-visual pass
    /// family (merge, flatten, reorder, margins, …), not just the paragraph
    /// merge (pagination experiments; hidden). Redundant with
    /// --powertools-faithful, which sets the same preset.
    #[arg(long, hide = true)]
    pub no_paragraph_merge: bool,

    /// Two Markdown documents written as Word take styles, page setup,
    /// headers and footers from this .docx.
    #[arg(long, value_name = "FILE", help_heading = "Markdown inputs")]
    pub reference_doc: Option<PathBuf>,

    /// Read CriticMarkup in the Markdown documents as tracked changes. By
    /// default a document compared is text.
    #[arg(long, help_heading = "Markdown inputs")]
    pub critic: bool,

    /// Where images named by the Markdown are found (default: the
    /// original's directory when it is Markdown, else the modified's).
    /// Only images that resolve inside that directory are read, symlinks
    /// resolved; a URL or a path out of it is written as its alt text,
    /// with a warning that says why.
    #[arg(long, value_name = "DIR", help_heading = "Markdown inputs")]
    pub resource_path: Option<PathBuf>,

    /// How a .pdf or .png output is drawn.
    #[command(flatten)]
    #[serde(flatten)]
    pub render: RenderArgs,

    /// The options of the agent view printed without -o (or written to
    /// FILE.md); `jubarte FILE`'s too.
    #[command(flatten)]
    #[serde(skip)]
    pub read: ReadArgs,

    /// Whether the command line spells out any of `read`'s options (set
    /// after parsing): a `.md` of two Markdown documents is CriticMarkup,
    /// which they cannot shape.
    #[arg(skip)]
    pub view_given: bool,
}

/// The derive model with the shorthands' options hidden: `jubarte --help`
/// lists the tasks; `jubarte compare --help` and `jubarte read --help` show
/// every option.
pub fn top_level_command() -> clap::Command {
    let hidden: std::collections::HashSet<clap::Id> =
        <CompareArgs as clap::Args>::augment_args(clap::Command::new("shorthand"))
            .get_arguments()
            .map(|a| a.get_id().clone())
            .collect();
    Cli::command().mut_args(move |a| {
        if hidden.contains(a.get_id()) {
            a.hide(true)
        } else {
            a
        }
    })
}

impl std::ops::Deref for Cli {
    type Target = CompareArgs;
    fn deref(&self) -> &Self::Target {
        &self.compare
    }
}

/// The subcommands; none given is the two-file compare.
#[derive(clap::Subcommand, Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "command", content = "args")]
pub enum Command {
    /// Internal dispatch for the typed `edit ACTION` grammar.
    #[command(name = "__edit-v2", hide = true, bin_name = "jubarte edit")]
    #[serde(rename = "edit")]
    TypedEdit(edit_actions::TypedEdit),
    /// Generate a complete source-bound PlanV2 without applying edits.
    #[command(
        name = "__edit-template",
        hide = true,
        bin_name = "jubarte edit template"
    )]
    #[serde(rename = "edit")]
    EditTemplate(edit_template::TemplateDispatch),
    /// Compare documents: print the redline as the agent view, or write it with -o.
    #[command(
        visible_alias = "redline",
        after_help = "Examples:\n  jubarte compare old.docx new.docx                 the redline's agent view\n  jubarte compare old.docx new.docx -o redline.docx\n  jubarte compare old.docx new.docx -o redline.pdf --changed-pages\n  jubarte compare old.md new.md -o changes.md       CriticMarkup\n  jubarte compare -b old.docx -m new.docx --author Legal -o r.docx"
    )]
    Compare(CompareArgs),
    /// List tracked changes with IDs for accept, reject and edit plans.
    ///
    /// One line per logical change: the marks Word wrote for one edit are
    /// one change with a joined id (`r7+8+9+10`), its text followed by
    /// `(N elements)`.
    Changes {
        /// The document (.docx).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Emit one JSON object per line.
        #[arg(long)]
        json: bool,
    },
    /// Accept all tracked changes, or select by ID, author or kind.
    ///
    /// Every part counts: the body, headers, footers, notes and comment
    /// bodies. An author or kind selection reaches comment bodies too, but
    /// no ID names a comment-body change, so an ID selection leaves those
    /// tracked.
    Accept {
        /// The document (.docx) whose revisions to accept.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Output path; its extension picks the format: .docx (also any
        /// other), .md (the agent view), .pdf, .png (one file per page) or
        /// .json (the snapshot).
        #[arg(short = 'o', long, value_name = "FILE")]
        output: PathBuf,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Keep comments and replies whose whole anchor is removed, on an empty anchor.
        #[arg(long)]
        keep_nested_comment: bool,
        /// Revisions to accept or reject; empty selects all.
        #[command(flatten)]
        #[serde(flatten)]
        selection: Selection,
        /// How a .pdf or .png output is drawn.
        #[command(flatten)]
        #[serde(flatten)]
        render: RenderArgs,
    },
    /// Reject all tracked changes, or select by ID, author or kind.
    ///
    /// Every part counts: the body, headers, footers, notes and comment
    /// bodies. An author or kind selection reaches comment bodies too, but
    /// no ID names a comment-body change, so an ID selection leaves those
    /// tracked.
    Reject {
        /// The document (.docx) whose revisions to reject.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Output path; its extension picks the format: .docx (also any
        /// other), .md (the agent view), .pdf, .png (one file per page) or
        /// .json (the snapshot).
        #[arg(short = 'o', long, value_name = "FILE")]
        output: PathBuf,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Keep comments and replies whose whole anchor is removed, on an empty anchor.
        #[arg(long)]
        keep_nested_comment: bool,
        /// Revisions to accept or reject; empty selects all.
        #[command(flatten)]
        #[serde(flatten)]
        selection: Selection,
        /// How a .pdf or .png output is drawn.
        #[command(flatten)]
        #[serde(flatten)]
        render: RenderArgs,
    },
    /// Convert Word or Markdown to DOCX, PDF, PNG or Markdown.
    #[command(after_help = "Examples:\n  \
        jubarte convert contract.docx                   PDF, Word-style layout\n  \
        jubarte convert draft.md                        draft.docx, CriticMarkup as tracked changes\n  \
        jubarte convert draft.md -o draft.pdf           the changes painted in a PDF\n  \
        jubarte convert draft.md --reference-doc house.docx -o draft.docx\n  \
        jubarte convert draft.md -t md --track-changes accept   the text with every change accepted\n  \
        jubarte convert contract.docx -t md             Markdown with <!-- page N of M --> lines\n  \
        jubarte convert old.doc                         old.docx (text, headings, lists, tables)\n  \
        jubarte convert notes.md --no-critic            {++ and the other delimiters as text")]
    Convert {
        /// The document to convert: .docx, Markdown (.md, .markdown), or a
        /// Word 97-2003 .doc (read into a .docx first).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Output path; its extension picks the format: .docx, .md, .pdf,
        /// .png (pages named `<stem>-page-NN.png` beside it) or .json (the
        /// snapshot of the result). Without it: `<stem>.pdf` next to a
        /// .docx, `<stem>.docx` next to Markdown, `<stem>.json` for
        /// `--to json`; Markdown output goes to stdout.
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Retired: `--to pdf` (or `-o FILE.pdf`).
        #[arg(long, hide = true)]
        pdf: bool,
        /// Retired: `--to png` (or `-o FILE.png`).
        #[arg(long, hide = true)]
        png: bool,
        /// Write a JSON page report (`{page_count, pages:[{index,text}], fonts}`).
        #[arg(long, value_name = "FILE")]
        report: Option<PathBuf>,
        /// Deflate the PDF's streams (`/FlateDecode`). Much smaller output;
        /// the trade is that the page content is no longer plain text, so it
        /// cannot be read with `strings` or `grep`.
        #[arg(long)]
        compress: bool,
        /// Write a JSON font-resolution report (`[{requested, step, physical,
        /// bold, italic, synthetic, substituted}, …]`) for this document
        /// (plan Step 2f).
        #[arg(long, value_name = "FILE")]
        font_report: Option<PathBuf>,
        /// Formats and Markdown reading.
        #[command(flatten)]
        #[serde(flatten)]
        markdown: MarkdownArgs,
        /// How a PDF or PNG output is rendered.
        #[command(flatten)]
        #[serde(flatten)]
        render: RenderArgs,
        /// Exit 4 when a requested font was substituted (listed on stderr
        /// and in --report). Every output is still written. Exit status:
        /// 0 ok, 1 error, 4 a requested font was substituted.
        #[arg(long)]
        fail_on_substitution: bool,
        /// Refresh the cached results of PAGEREF, REF, NUMPAGES, SEQ and TOC
        /// fields from jubarte's layout (TOCs rebuilt from the headings) in
        /// the .docx written, after --track-changes. Field codes stay, so
        /// Word can update them again; page numbers are jubarte's layout, not
        /// Word's (docs/WORD_DIFFERENCES.md). Prints one line per field;
        /// --report writes `{page_count, fields}`.
        #[arg(long)]
        update_fields: bool,
        /// Give up after this many seconds: exit 124 (as `timeout(1)`) with
        /// nothing more written. An output being written at that moment
        /// may be left partial.
        #[arg(long, value_name = "SECONDS", value_parser = parse_timeout)]
        #[serde(serialize_with = "serialize_timeout")]
        timeout: Option<std::time::Duration>,
    },
    /// Review differences as GitHub, word, normal, context or side-by-side text.
    #[command(after_help = "Examples:\n  \
        jubarte diff old.docx new.docx --format github   Git/GitHub patch on stdout\n  \
        jubarte diff old.md new.md                       the patch on stdout\n  \
        jubarte diff old.md new.md --format critic       CriticMarkup on stdout, as pandiff\n  \
        jubarte diff old.docx new.docx -o changes.patch  the patch in a file\n  \
        jubarte compare old.md new.md -o changes.docx    Word tracked changes (compare writes redlines)\n  \
        jubarte compare contract.docx edited.md -o redline.pdf\n      \
        the Markdown's edits painted on the Word document\n\n\
        GIT:\n  \
        git config --global difftool.jubarte.cmd 'jubarte diff \"$LOCAL\" \"$REMOTE\"'\n  \
        git difftool -t jubarte -y -- '*.md'")]
    Diff {
        /// The old document: .docx or Markdown.
        #[arg(value_name = "OLD")]
        old: PathBuf,
        /// The new document: .docx or Markdown.
        #[arg(value_name = "NEW")]
        new: PathBuf,
        /// Write the view to FILE (.md, .markdown, .txt, .patch or .diff;
        /// stdout otherwise).
        #[arg(short = 'o', long, value_name = "FILE", help_heading = "Review")]
        output: Option<PathBuf>,
        /// Choose the review view; word accepts ALL input changes first.
        #[arg(long, value_enum, value_name = "FORMAT", default_value_t = PatchFormat::Patch, help_heading = "Review")]
        format: PatchFormat,
        /// Wrap the patch's lines at this many columns; 0 does not wrap.
        #[arg(long, value_name = "N", default_value_t = crate::markdown::DEFAULT_COLUMNS, help_heading = "Paragraph patch")]
        columns: usize,
        /// Unchanged lines around GitHub or context hunks; -U0 shows changes only.
        #[arg(short = 'U', long, value_name = "LINES", default_value_t = 3, value_parser = parse_context, help_heading = "Review")]
        context: usize,
        /// Accept both documents' changes before comparing. Word format always does this.
        #[arg(long, help_heading = "Review")]
        accept_changes: bool,
        /// Show complete lines instead of a 70-character window around changes.
        #[arg(long, help_heading = "Review")]
        full_lines: bool,
        /// Output format: md only (the patch, CriticMarkup or a text view
        /// are all text).
        // Every format parses, so another one is refused naming compare.
        #[arg(
            short = 't',
            long = "to",
            value_enum,
            value_name = "FORMAT",
            hide_possible_values = true,
            help_heading = "Formats"
        )]
        to: Option<Format>,
        /// Input format of both documents [default: from each file].
        #[arg(
            short = 'f',
            long = "from",
            value_parser = input_format_parser(),
            value_name = "FORMAT",
            help_heading = "Formats"
        )]
        from: Option<Format>,
        /// Overwrite the output file if it already exists.
        #[arg(long, help_heading = "Review")]
        force: bool,
        /// Who made the changes: the patch's owner.
        #[arg(
            short = 'a',
            long,
            value_name = "NAME",
            default_value = "Jubarte",
            help_heading = "Paragraph patch"
        )]
        author: String,
        /// Revision timestamp (ISO 8601) [default: now, UTC]; pin it for
        /// reproducible output.
        #[arg(
            short = 'd',
            long = "datetime",
            value_name = "ISO8601",
            value_parser = parse_datetime,
            help_heading = "Paragraph patch"
        )]
        datetime: Option<String>,
    },
    /// Read the agent view: YAML header, `<!-- pN -->` id lines, changes and comments with ids
    ///
    /// A YAML header, then Markdown with an `<!-- pN -->` id line before every
    /// paragraph, tracked changes as CriticMarkup
    /// followed by their ids (`{++text++}{r12 @AC}`) and comments with
    /// theirs (`{>>c5 @AC: …<<}`).
    #[command(aliases = ["text", "inspect"])]
    Read {
        /// The document (.docx) to read.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Write the view to FILE.md, or the snapshot to FILE.json,
        /// instead of printing it.
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// The view's options.
        #[command(flatten)]
        #[serde(flatten)]
        args: ReadArgs,
    },
    /// Edit a document: -p WHERE with --anchor, --content, --delete,
    /// --resolve, --reply, --accept, --reject, --strike, --restore or --style
    /// (several -p per command), or a JSON plan. Writes FILE_modified.docx
    /// (--out-dir: the bundle), then prints SUCCESS and a fenced before/after
    /// diff of the full agent views (refusal: exit 3).
    #[command(after_help = "Examples:\n  \
        jubarte edit a.docx -p p12 --anchor \"thirty days\" --content \"forty-five days\"\n  \
        jubarte edit a.docx -p p12 --anchor thirty --content \"thirty (30)\"   an insertion\n  \
        jubarte edit a.docx -p p7 --anchor \"at its sole discretion\" --delete -p p9 --delete\n  \
        jubarte edit a.docx -p p3 --content \"The parties agree as follows.\"    rewrite\n  \
        jubarte edit a.docx -p p4 --anchor Fees --style bold -p p5 --style Heading2\n  \
        jubarte edit a.docx -p c5 --content \"Agreed.\" -p c7 --resolve\n  \
        jubarte edit a.docx -p c5 --reply \"Agreed, will fix.\"\n  \
        jubarte edit a.docx -p c5.text --content \"net 30\"   the text c5 covers\n  \
        jubarte edit a.docx -p r3 --accept -p header1.r1 --reject\n  \
        jubarte edit a.docx -p p3 --delete -o b.docx --pdf   b.docx and b.pdf\n  \
        jubarte edit a.docx --plan plan.json --out-dir review   the bundle in review/\n  \
        jubarte edit a.docx --plan plan.json -p p3 --content \"New.\"   record into plan.json\n  \
        jubarte edit a.docx --plan plan.json --consume   apply plan.json, then delete it\n\n\
        WHERE is an id from `jubarte read`: p12, header1, footer2.p1,\n\
        t0.row1.cell2, c5 (a comment), c5.text (the text it covers) or r12\n\
        (a revision).\n\nTyped actions: jubarte edit replace|rewrite|insert|delete FILE [OPTIONS]\n  jubarte edit template FILE --select p12,p20-p25 -o edits.json\n  jubarte edit apply FILE --plan PLAN.json -o OUTPUT.docx\n  Inline --save-plan FILE saves without applying; writes require -o or --out-dir.\n  Run jubarte edit ACTION --help for its grammar.\nRouting: valid legacy calls, including `edit replace -p p0 --content TEXT`,\n  retain v1 semantics. Use ./replace (or ./rewrite, ./insert, ./delete, ./apply)\n  to explicitly name a legacy input whose name equals an action.")]
    Edit {
        /// The source document (.docx). Never modified.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Edit plan JSON: batches and the other operation kinds (see
        /// `jubarte capabilities --json`). Alone, it is applied to FILE; with
        /// operation flags, they are recorded into it (created on first use)
        /// and FILE is left as it is. Without a value: FILE_plan.json beside
        /// FILE.
        #[arg(long, value_name = "PLAN.json", num_args = 0..=1, default_missing_value = "", value_parser = plan_path(), required_unless_present = "location")]
        plan: Option<PathBuf>,
        /// `--plan` was given without a value (D16): the summary says so.
        #[arg(skip)]
        plan_default: bool,
        /// Delete PLAN.json once it has been applied and the outputs are
        /// written (with --plan and no operation flags). Kept when
        /// --skip-failed left operations out.
        #[arg(long, requires = "plan")]
        consume: bool,
        /// Where: p12, header1, footer2.p1, t0.row1.cell2, c5 (a comment),
        /// c5.text (the text c5 covers) or r12 (a revision, with --accept,
        /// --reject, --strike or --restore). Each -p starts an operation; the
        /// flags after it belong to it.
        #[arg(short = 'p', long = "location", value_name = "WHERE", action = clap::ArgAction::Append)]
        location: Vec<String>,
        /// Text inside WHERE the operation applies to (must occur once).
        #[arg(long, value_name = "TEXT", action = clap::ArgAction::Append)]
        anchor: Vec<String>,
        /// New text: replaces the anchor; rewrites the paragraph without
        /// one; on c5, the comment's new text.
        #[arg(long, value_name = "TEXT", action = clap::ArgAction::Append)]
        content: Vec<String>,
        /// Delete the anchor, or the whole paragraph (or comment) without one.
        #[arg(long, action = clap::ArgAction::Append, num_args = 0, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
        delete: Vec<bool>,
        /// Resolve comment c5 (with -p c5).
        #[arg(long, action = clap::ArgAction::Append, num_args = 0, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
        resolve: Vec<bool>,
        /// Reply to comment c5 (with -p c5), as `add -p c5 --content`.
        #[arg(long, value_name = "TEXT", action = clap::ArgAction::Append)]
        reply: Vec<String>,
        /// Let `-p c5 --content` change a comment another author wrote; it
        /// keeps their name. Without it that is refused
        /// (COMMENT_NOT_YOURS): one author answers another with --reply.
        #[arg(long)]
        rewrite_others_comments: bool,
        /// Accept revision r12 (with -p r12; r1+2 for several); prints the
        /// resolved change (`accepted r1+2 in p3: "from" > "to"`).
        #[arg(long, action = clap::ArgAction::Append, num_args = 0, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
        accept: Vec<bool>,
        /// Reject revision r12 (with -p r12; r1+2 for several); prints the
        /// resolved change (`rejected r0 in p3: "from" > "to"`).
        #[arg(long, action = clap::ArgAction::Append, num_args = 0, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
        reject: Vec<bool>,
        /// Strike another author's insertion r12 (with -p r12): our tracked
        /// deletion inside it, so the decision shows. --content is a comment
        /// on the struck words.
        #[arg(long, action = clap::ArgAction::Append, num_args = 0, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
        strike: Vec<bool>,
        /// Restore another author's deletion r12 (with -p r12): their deletion
        /// stays and our tracked insertion of its words follows it. --content
        /// is a comment on the restored words.
        #[arg(long, action = clap::ArgAction::Append, num_args = 0, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
        restore: Vec<bool>,
        /// With --reject: strike each insertion and restore each deletion
        /// instead of removing the change without a trace.
        #[arg(long, action = clap::ArgAction::Append, num_args = 0, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
        visible: Vec<bool>,
        /// Formatting for the anchored text: bold, italic, underline, strike,
        /// caps, highlight=yellow, font=Calibri, size=11, color=FF0000; any
        /// other value is a paragraph style (Heading2).
        #[arg(long, value_name = "SPEC", action = clap::ArgAction::Append)]
        style: Vec<String>,
        /// Author, datetime, mode and output options.
        #[command(flatten)]
        #[serde(flatten)]
        options: EditOptions,
        /// Resolve and report only; write nothing. When every operation
        /// passes, stderr ends with the command that applies it (`next:`).
        #[arg(long)]
        dry_run: bool,
        /// Apply every operation that passes; leave out the ones that fail
        /// and every operation of their "group". Exit 0 when one landed.
        /// The failed ones go to PLAN.failed.json (FILE_plan.failed.json for
        /// operation flags): a plan to fix and run on the output.
        #[arg(long)]
        skip_failed: bool,
        /// Refuse when the edit would change nothing in the document
        /// (NO_CHANGES_REQUESTED); reports, renders and bundles do not
        /// count as a change.
        #[arg(long)]
        require_change: bool,
        /// Also write the PDF beside the document (the bundle: redline.pdf
        /// and clean.pdf).
        #[arg(long)]
        pdf: bool,
        /// Also write PNG pages beside the document, STEM-page-NN.png (the
        /// bundle: redline-page-NN.png and clean-page-NN.png).
        #[arg(long)]
        png: bool,
        /// PNG resolution in dots per inch (1-1200).
        #[arg(long, default_value_t = 96.0, value_name = "DPI", value_parser = parse_dpi, help_heading = "Rendering")]
        dpi: f32,
        /// How tracked changes are painted in the redline PDF/PNG.
        #[arg(long, value_enum, default_value_t = Revisions::Conventional, requires_if("custom", "revision_palette"), help_heading = "Revision marks")]
        revisions: Revisions,
        /// Marks for --revisions custom (see `convert --help`).
        #[arg(long, value_name = "SPEC", value_parser = parse_palette, requires = "revisions", help_heading = "Revision marks")]
        revision_palette: Option<String>,
        /// The operations the flags describe, grouped by -p (filled after
        /// parsing).
        #[arg(skip)]
        operations: Vec<FlagOp>,
    },
    /// Add a paragraph, a comment or a reply: -p WHERE --content TEXT
    /// (several -p per command). Writes FILE_modified.docx, as edit does,
    /// and prints SUCCESS and a fenced before/after diff of the full agent views.
    #[command(after_help = "Examples:\n  \
        jubarte add a.docx -p p12 --content \"Time is of the essence.\"   new paragraph after p12\n  \
        jubarte add a.docx -p p12 --content Recitals --before --style Heading2\n  \
        jubarte add a.docx -p p5 --anchor \"monthly fee\" --content \"Net of taxes?\"   a comment\n  \
        jubarte add a.docx -p p5 --comment --content \"Whole clause needs a cap.\"\n  \
        jubarte add a.docx -p c5 --content \"Agreed, will fix.\"           a reply\n  \
        jubarte add a.docx -p c5 --reply \"Agreed, will fix.\"             the same reply\n  \
        jubarte add a.docx --plan plan.json -p p5 --comment --content \"Cap?\"   record into plan.json")]
    Add {
        /// The source document (.docx). Never modified.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Edit plan JSON. With operation flags, they are recorded into it
        /// (created on first use) and FILE is left as it is; alone, it is
        /// applied to FILE, as `edit --plan` does. Without a value:
        /// FILE_plan.json beside FILE.
        #[arg(long, value_name = "PLAN.json", num_args = 0..=1, default_missing_value = "", value_parser = plan_path())]
        plan: Option<PathBuf>,
        /// `--plan` was given without a value (D16): the summary says so.
        #[arg(skip)]
        plan_default: bool,
        /// Delete PLAN.json once it has been applied and the outputs are
        /// written (with --plan and no operation flags).
        #[arg(long, requires = "plan")]
        consume: bool,
        /// Where: p12, header1, footer2.p1, t0.row1.cell2, or c5 for a reply.
        /// Each -p starts an operation; the flags after it belong to it.
        #[arg(short = 'p', long = "location", value_name = "WHERE", action = clap::ArgAction::Append, required_unless_present = "plan")]
        location: Vec<String>,
        /// Text inside WHERE to comment on (must occur once).
        #[arg(long, value_name = "TEXT", action = clap::ArgAction::Append)]
        anchor: Vec<String>,
        /// The paragraph, comment or reply text.
        #[arg(long, value_name = "TEXT", action = clap::ArgAction::Append)]
        content: Vec<String>,
        /// Reply to comment c5 (with -p c5); the same as --content there.
        #[arg(long, value_name = "TEXT", action = clap::ArgAction::Append)]
        reply: Vec<String>,
        /// A new paragraph before WHERE instead of after it.
        #[arg(long, action = clap::ArgAction::Append, num_args = 0, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
        before: Vec<bool>,
        /// A comment on the whole paragraph (with --anchor the comment sits
        /// on the anchor and this flag is implied).
        #[arg(long, action = clap::ArgAction::Append, num_args = 0, default_missing_value = "true", value_parser = clap::value_parser!(bool))]
        comment: Vec<bool>,
        /// Formatting for the new paragraph's text (bold, italic, underline,
        /// highlight=yellow) or its paragraph style (Heading2).
        #[arg(long, value_name = "SPEC", action = clap::ArgAction::Append)]
        style: Vec<String>,
        /// Author, datetime, mode and output options.
        #[command(flatten)]
        #[serde(flatten)]
        options: EditOptions,
        /// The operations the flags describe, grouped by -p (filled after
        /// parsing).
        #[arg(skip)]
        operations: Vec<FlagOp>,
    },
    /// What this binary can do, for agents choosing an operation.
    Capabilities {
        /// The manifest or focused answer as JSON instead of text.
        #[arg(long)]
        json: bool,
        /// Discover one edit kind, including its fields and execution conditions.
        #[arg(long)]
        operation: Option<String>,
        /// Discover one field, optionally within --operation.
        #[arg(long)]
        field: Option<String>,
        /// Publish operation × target × policy edit support.
        #[arg(long, conflicts_with_all = ["operation", "field"])]
        matrix: bool,
    },
    /// Check or install a release from GitHub.
    #[command(after_help = "Examples:\n  \
        jubarte self-update --check          installed and latest versions\n  \
        jubarte self-update                  ask, then install the latest release\n  \
        jubarte self-update --yes            install without asking\n  \
        jubarte self-update --version 0.9.3  install that release (also older)")]
    #[command(display_order = 1000)]
    SelfUpdate {
        /// Print the installed and latest versions; install nothing.
        #[arg(long)]
        check: bool,
        /// Install without asking (needed without a terminal).
        #[arg(long, short = 'y')]
        yes: bool,
        /// Install this release instead of the latest, older ones included.
        #[arg(long, value_name = "VERSION")]
        version: Option<String>,
    },
    /// Diagnose a Word package, or compare package structures.
    #[command(after_help = "Examples:\n  \
        jubarte debug out.docx                    orphans, fields, bookmarks, package, structure\n  \
        jubarte debug out.docx --list             the package's entries\n  \
        jubarte debug old.docx new.docx --list    entries that differ\n  \
        jubarte debug old.docx new.docx -c elements -p document.xml\n  \
        jubarte debug out.docx -c ids             revision/docPr ids used twice\n  \
        jubarte debug out.docx -c textbox -g FILENAME\n  \
        jubarte debug out.docx -c text            paragraphs with ins/del marks\n  \
        jubarte debug a.docx b.docx -c text       paragraphs that differ, per part\n  \
        jubarte debug a.docx b.docx -c runs       the same, with direct formatting\n  \
        jubarte debug out.docx -c runs -g \"Q: Can\"   one paragraph's runs, whole\n  \
        jubarte debug out.docx -c changes         what each pPrChange/tcPrChange/… records\n  \
        jubarte debug a.docx b.docx -c styledefs  style definitions that differ, paired by name\n  \
        jubarte debug a.docx b.docx -c numbering  list levels that differ, by numId\n  \
        jubarte debug a.docx b.docx -c xml -p document.xml\n  \
        jubarte debug diff a.docx ours.docx word.docx   element by element, three-way")]
    #[command(
        hide = true,
        args_conflicts_with_subcommands = true,
        subcommand_negates_reqs = true
    )]
    Debug {
        /// Optional package comparison task.
        #[command(subcommand)]
        sub: Option<DebugCommand>,
        /// One package, or two to compare (A then B).
        #[arg(value_name = "FILE", num_args = 1..=2, required = true)]
        files: Vec<PathBuf>,
        /// List the package's entries (sizes); with two files, the entries
        /// that differ.
        #[arg(short = 'l', long)]
        list: bool,
        /// Reports to run [default: orphans, fields, bookmarks, package,
        /// structure].
        #[arg(short = 'c', long = "check", value_enum, value_delimiter = ',')]
        checks: Vec<DebugCheck>,
        /// Only parts whose name contains this (e.g. document.xml).
        #[arg(short = 'p', long, value_name = "NAME")]
        part: Option<String>,
        /// Only what contains this: textbox stories; text/runs/xml/changes/styledefs/numbering lines (a runs paragraph matched on its plain text, printed whole).
        #[arg(short = 'g', long, value_name = "TEXT")]
        grep: Option<String>,
        /// Examples per finding kind.
        #[arg(short = 'n', long, value_name = "N", default_value_t = 5)]
        limit: usize,
        /// text/xml/runs of two files: common lines shown around each change.
        #[arg(short = 'C', long, value_name = "N", default_value_t = 0)]
        context: usize,
        /// text/xml/runs/changes/styledefs/numbering/render: show complete
        /// lines instead of clipping them at 200 characters.
        #[arg(long)]
        full_lines: bool,
    },
    /// Compare rendered pages pixel by pixel (different pages: exit 5).
    #[command(
        name = "visual-diff",
        alias = "diff-render",
        after_help = "Examples:\n  \
        jubarte visual-diff before.docx after.docx                  changed pages on stdout\n  \
        jubarte visual-diff before.docx after.docx --out-dir diff   PNGs of the changed pages and diff.json\n  \
        jubarte visual-diff a.docx b.docx --json                    the diff.json document on stdout\n\n\
        With --out-dir, each page that differs is written as a-page-NN.png,\n\
        b-page-NN.png and diff-page-NN.png (b's page with the changed pixels\n\
        magenta and boxed); diff.json lists every page with its changed_ratio,\n\
        bbox and, for a page only one side has, only_in."
    )]
    VisualDiff {
        /// The document before.
        #[arg(value_name = "A")]
        a: PathBuf,
        /// The document after.
        #[arg(value_name = "B")]
        b: PathBuf,
        /// Raster resolution of both sides in dots per inch (1-1200).
        #[arg(long, default_value_t = 100.0, value_name = "DPI", value_parser = parse_dpi, help_heading = "Rendering")]
        dpi: f32,
        /// Write the changed pages' PNGs and diff.json here (created if
        /// missing).
        #[arg(long, value_name = "DIR")]
        out_dir: Option<PathBuf>,
        /// Print diff.json to stdout instead of one line per changed page.
        #[arg(long)]
        json: bool,
        /// Skip the diff-page-NN.png overlays.
        #[arg(long)]
        no_overlay: bool,
        /// Overwrite files already in --out-dir.
        #[arg(long)]
        force: bool,
    },
    /// List comments, threads and the text they annotate.
    Comments {
        /// The document (.docx).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Emit one JSON object per line.
        #[arg(long)]
        json: bool,
        /// Only this author's comments (exact match).
        #[arg(long, value_name = "NAME")]
        author: Option<String>,
        /// One comment per thread: the newest.
        #[arg(long)]
        latest: bool,
    },
    /// Join documents in order, preserving images, styles, lists and notes.
    #[command(after_help = "Examples:\n  \
        jubarte append a.docx b.docx -o ab.docx\n  \
        jubarte append cover.docx body.docx annex.docx -o all.docx --section-break continuous\n  \
        jubarte append letter.docx exhibit.docx -o out.docx --keep-sections\n  \
        jubarte append review_a.docx review_b.docx -o both.docx --carry-comments")]
    Append {
        /// The documents (.docx), in order.
        #[arg(value_name = "FILE", num_args = 2.., required = true)]
        files: Vec<PathBuf>,
        /// Output path; its extension picks the format: .docx (also any
        /// other), .md (the agent view), .pdf, .png (one file per page) or
        /// .json (the snapshot).
        #[arg(short = 'o', long, value_name = "FILE")]
        output: PathBuf,
        /// What separates each document from the one before it.
        #[arg(long, value_enum, default_value_t = SectionBreakArg::NextPage)]
        section_break: SectionBreakArg,
        /// Keep each appended document's final section (page size, margins,
        /// headers, footers) as a section of its own.
        #[arg(long)]
        keep_sections: bool,
        /// Carry the comments each appended document's body and notes
        /// anchor, with their threads and resolution (those in headers and
        /// footers are still dropped). Off, comments are dropped and warned.
        #[arg(long)]
        carry_comments: bool,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// How a .pdf or .png output is drawn.
        #[command(flatten)]
        #[serde(flatten)]
        render: RenderArgs,
    },
    /// Check Word validity, tracked edits, accessibility, style and
    /// structure; or repair Word validity.
    ///
    /// A word or edits finding fails the run (exit 2), and so does any
    /// error; --strict fails on warnings too. With --baseline, only the
    /// findings the baseline lacks count. An unreadable file exits 1.
    #[command(alias = "audit")]
    #[command(after_help = "Examples:\n  \
        jubarte validate contract.docx\n  \
        jubarte validate contract.docx --json\n  \
        jubarte validate contract.docx --repair fixed.docx\n  \
        jubarte validate review/redline.docx --original contract.docx --author Claude\n  \
        jubarte validate review/redline.docx --baseline contract.docx\n  \
        jubarte validate contract.docx --rules word,a11y,style,structure --strict\n\n\
        Rule sets: word (default: Word validity), edits (with --author, and \
        --original or --baseline), a11y, style, structure.\n\
        Audit codes (code, set, severity):\n  \
        HEADING_SKIP a11y warning, IMAGE_NO_DESCR a11y error,\n  \
        TABLE_NO_HEADER_ROW a11y warning, MISSING_LANG a11y warning,\n  \
        LITERAL_BULLET style warning, EMPTY_SPACER_PARAGRAPH style info,\n  \
        DIRECT_FORMATTING_OVERRIDES_STYLE style info,\n  \
        STALE_FIELD_CACHE structure warning, FONT_SUBSTITUTED structure info")]
    #[command(group(clap::ArgGroup::new("edits_against").args(["original", "baseline"]).multiple(true)))]
    Validate {
        /// The document (.docx).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// JSON Lines: one object per finding, nothing when there is none.
        #[arg(long)]
        json: bool,
        /// Rule sets or audit codes, comma-separated [default: word; with
        /// --original, or --baseline and --author, word and edits].
        #[arg(long, value_name = "RULES", value_delimiter = ',', value_parser = parse_rule)]
        rules: Vec<String>,
        /// Also fail (exit 2) on warnings from other rule sets; word and edits
        /// findings fail without it.
        #[arg(long)]
        strict: bool,
        /// Write the repaired package here; remaining findings still fail.
        ///
        /// A structural repair: it does not check the edits' history.
        /// Add --original and --author for that.
        #[arg(long, value_name = "FILE")]
        repair: Option<PathBuf>,
        /// The document the edits started from: its findings print as
        /// already in the input (not yours), and only new ones fail the run.
        /// A finding is new by identity (code, part, place and message),
        /// never by count. With --author and no --original, the edits are
        /// checked against it too.
        #[arg(long, value_name = "FILE", conflicts_with = "repair")]
        baseline: Option<PathBuf>,
        /// Check tracked edits: every text change against ORIGINAL must be a
        /// revision by --author.
        #[arg(long, value_name = "FILE", requires = "author")]
        original: Option<PathBuf>,
        /// The author every change must carry (with --original or
        /// --baseline).
        #[arg(long, value_name = "NAME", requires = "edits_against")]
        author: Option<String>,
        /// Replace an existing --repair output.
        #[arg(long)]
        force: bool,
    },
    /// Check a document against a source-bound expectation contract.
    ///
    /// The only profile this build supports is `--expect`: an optional,
    /// separately supplied, hash-bound expectations file whose typed
    /// paragraph-text predicates (exact equality, containment, absence)
    /// are evaluated on FILE's own final projection. The other check
    /// profiles (integrity, quality, layout, baseline tracking) are #961;
    /// `jubarte validate` remains the validity audit. Passing checks and
    /// natural-language requirements without executable checks never claim
    /// the task complete.
    #[command(after_help = "Examples:\n  \
        jubarte check revised.docx --expect expectations.json\n  \
        jubarte check revised.docx --expect expectations.json --json\n\n\
        The expectations' source.sha256 must name FILE's own bytes: an\n\
        expectation file bound to the pre-edit source refuses as stale on\n\
        the edited output. Use the output-bound copy an `edit apply --expect`\n\
        receipt carries to re-check a published output.")]
    Check {
        /// The document (.docx) whose own digest the expectations must name.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Expectation contract (the only supported profile in this build).
        #[arg(long, value_name = "FILE")]
        expect: Option<PathBuf>,
        /// One result envelope, including refusals.
        #[arg(long)]
        json: bool,
    },
    /// Remove authors, editing IDs, metadata and comments before sharing.
    #[command(after_help = "Examples:\n  \
        jubarte scrub redline.docx -o out.docx                     everything, alias Author\n  \
        jubarte scrub redline.docx -o out.docx --author-alias Counsel --rsids\n  \
        jubarte scrub redline.docx -o out.docx --comments          comments only")]
    Scrub {
        /// The document (.docx) to scrub.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Output path; its extension picks the format: .docx (also any
        /// other), .md (the agent view), .pdf, .png (one file per page) or
        /// .json (the snapshot).
        #[arg(short = 'o', long, value_name = "FILE")]
        output: PathBuf,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Metadata and comment scrub selection.
        #[command(flatten)]
        #[serde(flatten)]
        scrub: ScrubSelection,
        /// How a .pdf or .png output is drawn.
        #[command(flatten)]
        #[serde(flatten)]
        render: RenderArgs,
    },
}

/// What `jubarte scrub` removes; everything when no flag is given.
#[derive(clap::Args, Debug, Default, PartialEq, Serialize)]
pub struct ScrubSelection {
    /// Name every author (revisions, comments, people.xml) takes.
    #[arg(long, value_name = "NAME")]
    pub author_alias: Option<String>,
    /// Remove rsids, the edit-session ids that tie copies together.
    #[arg(long)]
    pub rsids: bool,
    /// Remove creator, last editor, revision number, dates, manager,
    /// company and custom properties.
    #[arg(long)]
    pub docprops: bool,
    /// Remove every comment.
    #[arg(long)]
    pub comments: bool,
}

#[derive(clap::Subcommand, Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "command", content = "args")]
/// Package-level diagnostic comparison tasks.
pub enum DebugCommand {
    /// What differs between two or more packages, element by element:
    /// styles paired by type and name, paragraphs by their text, headers
    /// and footers by section role. Each hunk prints the lines not every
    /// file holds; with three or more files each line names the files that
    /// hold it. rsids, paragraph ids, revision ids/authors/dates,
    /// relationship ids (shown as what they point to), docProps save
    /// stamps, attribute order, on/off values and empty property blocks
    /// are dropped unless --raw.
    #[command(after_help = "Examples:\n  \
        jubarte debug diff a.docx b.docx\n  \
        jubarte debug diff a.docx ours_rej.docx word_rej.docx -p styles\n  \
        jubarte debug diff a.docx ours_rej.docx word_rej.docx --style \"Body Text\" --full\n  \
        jubarte debug diff a.docx ours.docx --para-text \"Section 4\"")]
    Diff {
        /// Two or more packages; the first is the reference (`-` lines).
        #[arg(value_name = "FILE", num_args = 2.., required = true)]
        files: Vec<PathBuf>,
        /// Only parts whose name or role contains this (e.g. styles,
        /// document.xml, "default header").
        #[arg(short = 'p', long, value_name = "NAME")]
        part: Option<String>,
        /// Only the style with this name or id (case-insensitive).
        #[arg(long, value_name = "NAME")]
        style: Option<String>,
        /// Only paragraphs whose text contains this, in any file.
        #[arg(long = "para-text", value_name = "TEXT")]
        para_text: Option<String>,
        /// Keep rsids, ids, authors, dates, on/off values and empty blocks.
        #[arg(long)]
        raw: bool,
        /// Print each shown element's common lines too.
        #[arg(long)]
        full: bool,
        /// Hunks per part (0: all).
        #[arg(short = 'n', long, value_name = "N", default_value_t = 60)]
        limit: usize,
    },
}

/// `jubarte debug --check`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DebugCheck {
    /// Deleted text outside its story's w:del; live text inside one; bare runs
    /// in a text box whose anchor is deleted.
    Orphans,
    /// Field nesting per story; fields partly deleted.
    Fields,
    /// Duplicate/unpaired bookmarks; start and end in different sdt, cell,
    /// text box or revision; bookmarks in plain-text or list controls.
    Bookmarks,
    /// Content types, relationship ids and targets, dangling note/comment
    /// references, undeclared mc:Ignorable prefixes.
    Package,
    /// Empty field codes, cells not ending in a paragraph, rows without cells,
    /// nested same-kind revisions, a body sectPr that is not last.
    Structure,
    /// Revision and docPr ids used twice (not in the default triage: Word
    /// opens such files).
    Ids,
    /// Style links and references naming no style; two styles with one type
    /// and name (Word pairs styles by name).
    Styles,
    /// Where bookmark starts and ends sit (parent chains, tallied).
    Chains,
    /// Element counts.
    Elements,
    /// Text box stories as XML (see --grep).
    Textbox,
    /// Paragraph text per story part, with {+inserted+} / [-deleted-] runs
    /// and the mark state; with two files, the lines that differ.
    Text,
    /// Part XML one element per line, without namespace declarations,
    /// rsids or paraIds; with two files, the lines that differ.
    Xml,
    /// `text` with each paragraph's direct properties [..], its mark's «..»
    /// and each run's direct formatting «..»; with two files, the lines that
    /// differ.
    Runs,
    /// Property-change records (pPrChange, tcPrChange, sectPrChange, …):
    /// where each sits and what the live properties add (+) and drop (-)
    /// against the recorded ones; with two files, the lines that differ.
    Changes,
    /// Style definitions by type and name (localized ids pair): docDefaults,
    /// then each style's default flag and basedOn/link by name, and a line
    /// per pPr/rPr/tblPr/… block; with two files, the lines that differ.
    Styledefs,
    /// List levels by numId and level as paragraphs see them (abstract
    /// definition plus the list's overrides; abstract ids renumber, so
    /// they are left out); with two files, the lines that differ.
    Numbering,
    /// What each story part should put on the page (tables with style,
    /// float and shading; shaded, highlighted, coloured and hidden text;
    /// fonts; fields; ins/del order; frames; sections), and "(layout)":
    /// jubarte's page count and the face each font resolved to; with two
    /// files, the lines that differ.
    Render,
}

impl From<DebugCheck> for crate::debug::Check {
    fn from(c: DebugCheck) -> Self {
        use crate::debug::Check;
        match c {
            DebugCheck::Orphans => Check::Orphans,
            DebugCheck::Fields => Check::Fields,
            DebugCheck::Bookmarks => Check::Bookmarks,
            DebugCheck::Package => Check::Package,
            DebugCheck::Structure => Check::Structure,
            DebugCheck::Ids => Check::Ids,
            DebugCheck::Styles => Check::Styles,
            DebugCheck::Chains => Check::Chains,
            DebugCheck::Elements => Check::Elements,
            DebugCheck::Textbox => Check::Textbox,
            DebugCheck::Text => Check::Text,
            DebugCheck::Xml => Check::Xml,
            DebugCheck::Runs => Check::Runs,
            DebugCheck::Changes => Check::Changes,
            DebugCheck::Styledefs => Check::StyleDefs,
            DebugCheck::Numbering => Check::Numbering,
            DebugCheck::Render => Check::Render,
        }
    }
}

/// `jubarte diff --format`: what goes to stdout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PatchFormat {
    /// The changed paragraphs, as `git diff --word-diff` with CriticMarkup
    /// comments and highlights.
    Patch,
    /// CriticMarkup: current document text with tracked marks.
    Critic,
    /// Git/GitHub unified text; preserves each document's tracked marks.
    #[value(alias = "unified", alias = "text")]
    Github,
    /// Fresh word-level CriticMarkup after accepting ALL changes in both inputs.
    Word,
    /// Normal diff with line addresses and no context (a/d/c, < and >).
    Normal,
    /// Context diff with old/new ranges and !, + and - prefixes.
    Context,
    /// Old and new lines in parallel columns, with |, < and > markers.
    SideBySide,
}

impl PatchFormat {
    /// Whether this format compares document snapshots without constructing a redline.
    pub fn is_text_view(self) -> bool {
        !matches!(self, Self::Patch | Self::Critic)
    }
}

/// `jubarte --mode` (compare).
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompareMode {
    /// Microsoft Word Compare's layout: word-level detail, replaced paragraphs
    /// merged, Word's alignment passes.
    Word,
    /// Open-Xml-PowerTools: coarse paragraph fallback (threshold 0.15), no
    /// Word alignment passes.
    Powertools,
}

/// A document format for `-f/--from` and `-t/--to`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    /// Word (.docx).
    Docx,
    /// Markdown: CommonMark with GitHub tables, task lists and footnotes,
    /// and CriticMarkup.
    #[value(alias = "markdown")]
    Md,
    /// PDF, laid out as Word does.
    Pdf,
    /// PNG pages.
    Png,
    /// The JSON snapshot (`read --json`'s object): written, never read.
    /// `diff` refuses it (`json_not_yet`).
    Json,
}

/// A file name's extension: `Path::extension`, or the whole name after its
/// dot when the name is only an extension (`.json`, `out/.md`), which Rust
/// reads as a hidden file with none.
fn extension(path: &Path) -> Option<String> {
    if let Some(extension) = path.extension() {
        return extension.to_str().map(str::to_string);
    }
    let name = path.file_name()?.to_str()?;
    name.strip_prefix('.')
        .filter(|rest| !rest.is_empty() && !rest.contains('.'))
        .map(str::to_string)
}

impl Format {
    /// The format a file name says, by extension (`.json`, a name that is
    /// only an extension, says json too).
    pub fn of_path(path: &Path) -> Option<Self> {
        Self::of_name(&extension(path)?)
    }

    /// The format an extension names, in any case.
    pub fn of_name(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "docx" | "docm" | "dotx" | "dotm" => Some(Self::Docx),
            "md" | "markdown" | "mdown" | "mkd" | "mkdn" | "txt" => Some(Self::Md),
            "pdf" => Some(Self::Pdf),
            "png" => Some(Self::Png),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    /// The name `--to` takes and messages print.
    pub fn name(self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Md => "md",
            Self::Pdf => "pdf",
            Self::Png => "png",
            Self::Json => "json",
        }
    }

    /// An input's format: the one asked for, else its extension, else its
    /// bytes (a zip is Word, anything else Markdown).
    pub fn of_input(asked: Option<Self>, path: &Path, bytes: &[u8]) -> Self {
        asked
            .or_else(|| Format::of_path(path).filter(|f| matches!(f, Self::Docx | Self::Md)))
            .unwrap_or(if bytes.starts_with(b"PK\x03\x04") {
                Self::Docx
            } else {
                Self::Md
            })
    }
}

/// A task that writes a file through `-o`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Task {
    /// `compare` (alias `redline`): the redline.
    Compare,
    /// `convert`: the document in another format.
    Convert,
    /// `read`: the agent view.
    Read,
    /// `diff`: a text diff.
    Diff,
    /// `accept`: the document with changes accepted.
    Accept,
    /// `reject`: the document with changes rejected.
    Reject,
    /// `append`: the documents joined.
    Append,
    /// `scrub`: the document with metadata removed.
    Scrub,
}

impl Task {
    /// The command's name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Compare => "compare",
            Self::Convert => "convert",
            Self::Read => "read",
            Self::Diff => "diff",
            Self::Accept => "accept",
            Self::Reject => "reject",
            Self::Append => "append",
            Self::Scrub => "scrub",
        }
    }

    /// The task a command name (or its alias) runs.
    pub fn of_name(name: &str) -> Option<Self> {
        Some(match name {
            "compare" | "redline" => Self::Compare,
            "convert" => Self::Convert,
            "read" | "text" | "inspect" => Self::Read,
            "diff" => Self::Diff,
            "accept" => Self::Accept,
            "reject" => Self::Reject,
            "append" => Self::Append,
            "scrub" => Self::Scrub,
            _ => return None,
        })
    }

    /// The formats its `-o` writes (`capabilities::OUTPUT_FORMATS`).
    fn allowed(self) -> Vec<Format> {
        crate::capabilities::OUTPUT_FORMATS
            .iter()
            .find(|(task, _)| *task == self.name())
            .map(|(_, names)| {
                names
                    .iter()
                    .map(|n| Format::of_name(n).expect("OUTPUT_FORMATS names a format"))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// What `-o PATH`'s extension says for this task: `diff` reads `.patch`
    /// and `.diff` as text too.
    fn known(self, path: &Path) -> Option<Format> {
        let patch = extension(path)
            .is_some_and(|e| e.eq_ignore_ascii_case("patch") || e.eq_ignore_ascii_case("diff"));
        if self == Self::Diff && patch {
            return Some(Format::Md);
        }
        Format::of_path(path)
    }

    /// The refusal for a format this task cannot write; the advice names
    /// the file as typed (`c.EXT`/`v.EXT` when `--to` said the format).
    fn cannot_write(self, format: Format, typed: Option<&str>) -> String {
        let ext = format.name();
        let named = |stem: &str| typed.map_or_else(|| format!("{stem}.{ext}"), str::to_string);
        match (self, format) {
            (Self::Diff, Format::Json) => "diff does not write .json yet".to_string(),
            (Self::Diff, _) => format!(
                "diff writes text (.md, .markdown, .txt, .patch, .diff); `compare OLD NEW -o {}` writes the redline as {ext}",
                named("c")
            ),
            (Self::Read, _) => {
                format!(
                    "read writes .md or .json; `convert FILE -o {}` writes {ext}",
                    named("v")
                )
            }
            _ => format!("{} does not write .{ext}", self.name()),
        }
    }
}

/// The format a task writes. `-o`'s extension decides; `--to` decides when
/// the extension is missing or unknown; a known extension that disagrees with
/// `--to` is an error. `-o` with neither → the task's usual format (`convert`'s
/// depends on its input: `None`). No `-o` and no `--to` → `None`: the task
/// prints, or convert picks by input. Pure.
pub fn output_format(
    task: Task,
    output: Option<&Path>,
    to: Option<Format>,
) -> Result<Option<Format>, String> {
    let said = output.and_then(|path| task.known(path));
    let name = output.map(|path| {
        path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        )
    });
    if let (Some(name), Some(said), Some(to)) = (&name, said, to)
        && said != to
    {
        return Err(format!(
            "-o {name} says {} and --to says {}: drop one",
            said.name(),
            to.name()
        ));
    }
    let format = match (output, said.or(to)) {
        (None, None) => return Ok(None),
        (_, Some(format)) => format,
        (Some(_), None) => match task {
            Task::Convert => return Ok(None),
            Task::Read | Task::Diff => Format::Md,
            _ => Format::Docx,
        },
    };
    if task.allowed().contains(&format) {
        Ok(Some(format))
    } else {
        // The typed name only when it said the format; `--to` names its own.
        let typed = output
            .filter(|_| said == Some(format))
            .map(|path| path.display().to_string());
        Err(task.cannot_write(format, typed.as_deref()))
    }
}

/// The error line for an input no task reads, `CODE: FILE is …`: an OLE
/// (`.doc`, encrypted) or RTF file, or, when it must be `word`, a file that
/// is no ZIP package. `None` when the bytes may be read. Every host prints
/// this line, so a refusal reads the same on each.
#[must_use]
pub fn refused_input(path: &Path, bytes: &[u8], word: bool) -> Option<String> {
    let checked = if word {
        crate::admission::sniff_package(bytes)
    } else {
        crate::admission::sniff(bytes)
    };
    checked.err().map(|refused| {
        format!(
            "{}: {} is {}",
            refused.code(),
            path.display(),
            refused.message
        )
    })
}

/// [`refused_input`] for a package a task reads whole (`validate
/// --original`): the bytes must also pass admission
/// ([`crate::admission::admit`]), so a ZIP that holds no Word document is
/// refused before anything is written. `None` when the package is
/// admitted.
#[must_use]
pub fn refused_package(path: &Path, bytes: &[u8]) -> Option<String> {
    crate::admission::admit(bytes, crate::admission::InputLimits::default())
        .err()
        .map(|refused| {
            format!(
                "{}: {} is {}",
                refused.code(),
                path.display(),
                refused.message
            )
        })
}

/// The line a writing task prints on stderr for the file it wrote:
/// `wrote r.pdf (pdf, 48213 bytes, 3 pages)`; `pages` is a PDF's.
pub fn wrote_line(path: &Path, format: Format, bytes: usize, pages: Option<usize>) -> String {
    let pages = pages.map_or_else(String::new, |n| format!(", {}", count(n, "page")));
    format!(
        "wrote {} ({}, {bytes} bytes{pages})",
        path.display(),
        format.name()
    )
}

/// Confirmation of the published document, comparing the two agent views.
pub fn success_block(before_name: &str, before: &str, after_name: &str, after: &str) -> String {
    let body = if before == after {
        "no differences\n".to_string()
    } else {
        similar::TextDiff::from_lines(before, after)
            .unified_diff()
            .context_radius(3)
            .header(
                &format!("before {before_name}"),
                &format!("after {after_name}"),
            )
            .to_string()
    };
    // CommonMark closes a tilde fence with up to three leading spaces, and a
    // context line puts one before the document's text.
    let longest = body
        .lines()
        .map(|line| {
            let indent = line.bytes().take(3).take_while(|b| *b == b' ').count();
            line.bytes().skip(indent).take_while(|b| *b == b'~').count()
        })
        .max()
        .unwrap_or(0);
    let fence = "~".repeat(4.max(longest + 1));
    format!("SUCCESS:\n{fence}diff\n{body}{fence}\n")
}

#[cfg(test)]
mod success_tests {
    use super::success_block;

    #[test]
    fn equal_views() {
        assert_eq!(
            success_block("in.docx", "same", "out.docx", "same"),
            "SUCCESS:\n~~~~diff\nno differences\n~~~~\n"
        );
    }

    #[test]
    fn labels_and_known_hunk() {
        assert_eq!(
            success_block(
                "dir/in.docx",
                "one\ntwo\nthree\n",
                "out.docx",
                "one\nnew\nthree\n"
            ),
            "SUCCESS:\n~~~~diff\n--- before dir/in.docx\n+++ after out.docx\n@@ -1,3 +1,3 @@\n one\n-two\n+new\n three\n~~~~\n"
        );
    }

    #[test]
    fn fence_exceeds_leading_tildes_in_diff_body() {
        let block = success_block("a", "a\n", "b\n~~~~~", "b\n");
        assert!(block.starts_with("SUCCESS:\n~~~~~~diff\n"), "{block}");
        assert!(block.ends_with("~~~~~~\n"), "{block}");
        // Tildes after a diff marker do not begin the line.
        assert!(success_block("a", "~~~~~~\n", "b", "x\n").starts_with("SUCCESS:\n~~~~diff\n"));
    }

    /// A context line keeps the document's text after one space, and
    /// CommonMark lets up to three spaces precede a closing fence, so ` ~~~~`
    /// would close a four-tilde fence early (review gid=48).
    #[test]
    fn a_context_line_of_tildes_cannot_close_the_fence() {
        let block = success_block("a", "~~~~\nkeep\n", "b", "~~~~\nchanged\n");
        assert!(block.contains("\n ~~~~\n"), "{block}");
        let fence = block["SUCCESS:\n".len()..]
            .bytes()
            .take_while(|b| *b == b'~')
            .count();
        assert!(fence > 4, "{block}");
    }

    #[test]
    fn three_context_lines_and_missing_newline() {
        let block = success_block(
            "a",
            "0\n1\n2\n3\n4\n5\n6\n7",
            "b",
            "0\n1\n2\n3\n4\n5\n6\nnew",
        );
        assert!(block.contains("@@ -5,4 +5,4 @@\n 4\n 5\n 6\n-7\n\\ No newline at end of file\n+new\n\\ No newline at end of file\n"), "{block}");
    }
}

/// The line for PNG pages: `wrote out/r-page-NN.png (png, 3 pages, 96 dpi)`,
/// `pattern` naming them all.
pub fn wrote_png_line(pattern: &Path, pages: usize, dpi: f32) -> String {
    format!(
        "wrote {} (png, {}, {dpi} dpi)",
        pattern.display(),
        count(pages, "page")
    )
}

/// `1 page`, `3 pages`.
fn count(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

/// A spelling 0.13 still accepts, with a note, and 0.14 removes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Retired {
    /// What the command line says (`diff-render`, `--pdf`).
    pub old: &'static str,
    /// What to say instead.
    pub now: &'static str,
    /// `None`: `old` is a task name. `Some(task)`: `old` is a flag retired on
    /// that task only (another task may keep it).
    pub on: Option<&'static str>,
}

/// Every retired task name and flag (D13).
pub const RETIRED: &[Retired] = &[
    Retired {
        old: "diff-render",
        now: "visual-diff",
        on: None,
    },
    Retired {
        old: "text",
        now: "read",
        on: None,
    },
    Retired {
        old: "inspect",
        now: "read",
        on: None,
    },
    Retired {
        old: "audit",
        now: "validate --rules a11y,style,structure",
        on: None,
    },
    Retired {
        old: "--changed-only",
        now: "--changed-pages",
        on: Some("convert"),
    },
    Retired {
        old: "--changed-only",
        now: "--changed-pages",
        on: Some("compare"),
    },
    Retired {
        old: "--changed-only",
        now: "--changed-pages",
        on: Some("accept"),
    },
    Retired {
        old: "--changed-only",
        now: "--changed-pages",
        on: Some("reject"),
    },
    Retired {
        old: "--changed-only",
        now: "--changed-pages",
        on: Some("append"),
    },
    Retired {
        old: "--changed-only",
        now: "--changed-pages",
        on: Some("scrub"),
    },
    Retired {
        old: "--pdf",
        now: "--to pdf",
        on: Some("convert"),
    },
    Retired {
        old: "--png",
        now: "--to png",
        on: Some("convert"),
    },
];

/// Every task name that is gone and parses as the shorthand's first file
/// again (`jubarte revisions x.docx` compares a file called `revisions`):
/// a [`removed_task_note`] names what to say instead.
pub(crate) const REMOVED_TASKS: &[Retired] = &[Retired {
    old: "revisions",
    now: "changes",
    on: None,
}];

/// Each [`REMOVED_TASKS`] name `argv` puts where the task goes, as
/// [`retired_spellings`] finds them.
pub(crate) fn removed_tasks<'a>(
    argv: impl IntoIterator<Item = &'a str>,
) -> Vec<(&'static str, &'static str)> {
    spellings_in(REMOVED_TASKS, argv)
}

/// The note a removed task name earns: the run goes on, reading it as a
/// file.
pub(crate) fn removed_task_note(old: &str, now: &str) -> String {
    format!("note: {old} is no longer a task, so it reads as a file name; use {now}")
}

/// Every flag that is gone: it no longer parses, and the error names what
/// to say instead, where clap's own tip may name a lookalike (`compare
/// --date` would be told `--dates`). Arthur (2026-10-14): `--date` is gone.
pub const REMOVED: &[Retired] = &[
    Retired {
        old: "--date",
        now: "--datetime",
        on: Some("compare"),
    },
    Retired {
        old: "--date",
        now: "--datetime",
        on: Some("diff"),
    },
    Retired {
        old: "--date",
        now: "--datetime",
        on: Some("convert"),
    },
    Retired {
        old: "--date",
        now: "--datetime",
        on: Some("edit"),
    },
    Retired {
        old: "--date",
        now: "--datetime",
        on: Some("add"),
    },
];

/// One option as `argv` spells it: `-p`, `--location` (a long's `=value`
/// split off), and the index of its token. The value it takes is skipped,
/// not kept.
struct Spelled {
    name: String,
    at: usize,
}

/// How many values `arg` takes after it: `None` for a flag (an option `cmd`
/// does not know, or one with `num_args = 0` such as `--delete`), else
/// `Some(optional)`, where an optional value (`num_args = 0..=1`) is taken
/// only when the next token is not an option, as clap does.
fn value_taken(arg: Option<&clap::Arg>) -> Option<bool> {
    let arg = arg.filter(|a| a.get_action().takes_values())?;
    match arg.get_num_args() {
        Some(range) if range.max_values() == 0 => None,
        Some(range) => Some(range.min_values() == 0),
        None => Some(false),
    }
}

/// The argument of `cmd` spelled `--name`, by its long name or any of its
/// aliases, hidden ones included.
fn long_arg<'c>(cmd: &'c clap::Command, name: &str) -> Option<&'c clap::Arg> {
    cmd.get_arguments().find(|a| {
        a.get_long() == Some(name) || a.get_all_aliases().is_some_and(|v| v.contains(&name))
    })
}

/// The argument of `cmd` spelled `-c`, by its short name or a short alias.
fn short_arg(cmd: &clap::Command, short: char) -> Option<&clap::Arg> {
    cmd.get_arguments().find(|a| {
        a.get_short() == Some(short)
            || a.get_all_short_aliases()
                .is_some_and(|v| v.contains(&short))
    })
}

/// The options of `tokens` (read with `cmd`'s arguments: which take a value)
/// up to the first token that is not an option, and the index of that one.
/// A short cluster (`-qp t0`) is split; an option `cmd` does not know takes
/// no value.
fn spelled_options(cmd: &clap::Command, tokens: &[&str]) -> (Vec<Spelled>, usize) {
    // Skip the next token as a value, unless an optional value meets an
    // option.
    let skip_value = |i: &mut usize, optional: bool| {
        let value = tokens.get(*i).copied();
        if !(optional && value.is_none_or(|v| v.starts_with('-'))) {
            *i += 1;
        }
    };
    let mut found = Vec::new();
    let mut i = 0;
    while let Some(&token) = tokens.get(i) {
        i += 1;
        if let Some(long) = token.strip_prefix("--") {
            let (name, inline) = match long.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (long, None),
            };
            if inline.is_none()
                && let Some(optional) = value_taken(long_arg(cmd, name))
            {
                skip_value(&mut i, optional);
            }
            found.push(Spelled {
                name: format!("--{name}"),
                at: i - 1,
            });
        } else if let Some(cluster) = token.strip_prefix('-').filter(|c| !c.is_empty()) {
            let at = i - 1;
            for (offset, short) in cluster.char_indices() {
                let name = format!("-{short}");
                let Some(optional) = value_taken(short_arg(cmd, short)) else {
                    found.push(Spelled { name, at });
                    continue;
                };
                // The rest of the cluster is the value (`-pt0`, `-p=t0`),
                // else the next token is.
                if cluster[offset + short.len_utf8()..].is_empty() {
                    skip_value(&mut i, optional);
                }
                found.push(Spelled { name, at });
                break;
            }
        } else {
            return (found, i - 1);
        }
    }
    (found, i)
}

/// The retired spellings in `argv` (the arguments after the program name),
/// each once, in the order they first appear. The task is the first token
/// before `--` that is neither an option nor an option's value, compare
/// when that is no task (the shorthand); a retired flag counts only on the
/// task it is retired on. Expired ids (`12`, `#12`, `t0.r1.c2`) are not
/// notes: the id parsers refuse them, naming the settled spelling.
/// Pure: no I/O.
pub fn retired_spellings<'a>(
    argv: impl IntoIterator<Item = &'a str>,
) -> Vec<(&'static str, &'static str)> {
    spellings_in(RETIRED, argv)
}

/// Each [`REMOVED`] spelling `argv` uses, as [`retired_spellings`] finds
/// them.
pub fn removed_spellings<'a>(
    argv: impl IntoIterator<Item = &'a str>,
) -> Vec<(&'static str, &'static str)> {
    spellings_in(REMOVED, argv)
}

/// The error a removed spelling ends the run with, naming what to say
/// instead.
pub fn removed_error(old: &str, now: &str) -> String {
    format!("error: {old} is removed; use {now}")
}

fn spellings_in<'a>(
    table: &[Retired],
    argv: impl IntoIterator<Item = &'a str>,
) -> Vec<(&'static str, &'static str)> {
    let argv: Vec<&str> = argv.into_iter().take_while(|t| *t != "--").collect();
    let model = Cli::command();
    let mut found = Vec::new();
    let (before_task, at) = spelled_options(&model, &argv);
    let Some(&task) = argv.get(at) else {
        return found;
    };
    if let Some(retired) = table.iter().find(|r| r.on.is_none() && r.old == task) {
        found.push((retired.old, retired.now));
    }
    // The shorthand `jubarte A B` is compare, with the top level's options
    // (zcode-14 F1).
    let (sub, task, from) = match model.find_subcommand(task) {
        Some(sub) => (sub, sub.get_name(), at + 1),
        None => (&model, "compare", at),
    };
    let mut spot = |options: Vec<Spelled>| {
        for option in options {
            for retired in table.iter().filter(|r| r.on == Some(task)) {
                if option.name == retired.old {
                    found.push((retired.old, retired.now));
                }
            }
        }
    };
    // An unknown option before the task (`--date X compare …`) takes no
    // value, so its value reads as the shorthand's first file.
    spot(before_task);
    // Options and positionals interleave after the task.
    let mut rest = &argv[from..];
    while !rest.is_empty() {
        let (options, next) = spelled_options(sub, rest);
        spot(options);
        rest = rest.get(next + 1..).unwrap_or_default();
    }
    let mut seen = std::collections::HashSet::new();
    found.retain(|spelling| seen.insert(*spelling));
    found
}

pub use crate::util::shell_word;

/// What a retired spelling still means once parsed: `audit` with no
/// `--rules` checks `a11y,style,structure`, as the retired task did, not
/// `validate`'s default `word`.
pub(crate) fn apply_retired(task: &mut Command, retired: &[(&str, &str)]) {
    if let Command::Validate { rules, .. } = task
        && rules.is_empty()
        && retired.iter().any(|(old, _)| *old == "audit")
    {
        *rules = ["a11y", "style", "structure"].map(String::from).to_vec();
    }
}

/// The stderr lines `argv` earns before its run or its usage error: each
/// retired spelling's note, then each removed task's.
pub fn notes(argv: &[String]) -> Vec<String> {
    let argv = || argv.iter().map(String::as_str);
    // `audit --rules …` runs the rules given, not audit's own sets.
    let rules_given = argv()
        .take_while(|token| *token != "--")
        .any(|token| token == "--rules" || token.starts_with("--rules="));
    retired_spellings(argv())
        .into_iter()
        .map(|(old, now)| match old {
            "audit" if rules_given => retired_note(old, "validate"),
            _ => retired_note(old, now),
        })
        .chain(
            removed_tasks(argv())
                .into_iter()
                .map(|(old, now)| removed_task_note(old, now)),
        )
        .collect()
}

/// The stderr line a retired spelling prints once.
pub fn retired_note(old: &str, now: &str) -> String {
    format!("note: {old} is now {now}; {old} is removed in 0.14")
}

/// `--track-changes`, pandoc's values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrackChanges {
    /// Keep them: CriticMarkup becomes Word tracked changes and comments.
    All,
    /// Accept every change.
    #[value(alias = "accepted")]
    Accept,
    /// Reject every change.
    #[value(alias = "rejected")]
    Reject,
}

impl From<TrackChanges> for crate::markdown::TrackChanges {
    fn from(choice: TrackChanges) -> Self {
        match choice {
            TrackChanges::All => Self::All,
            TrackChanges::Accept => Self::Accept,
            TrackChanges::Reject => Self::Reject,
        }
    }
}

/// How `-o FILE.pdf` and `-o FILE.png` are rendered (`convert`, `diff`);
/// refused with any other output.
#[derive(clap::Args, Clone, Debug, PartialEq, Serialize)]
#[command(next_help_heading = "Rendering (-o FILE.pdf or FILE.png)")]
pub struct RenderArgs {
    /// How tracked changes are painted: `conventional` (deletions red
    /// struck through, insertions blue underlined, moves green:
    /// double-struck where they left, double-underlined where they landed),
    /// `word` (what Microsoft Word's Save as PDF paints), or `custom`
    /// (see --revision-palette).
    #[arg(long, value_enum, default_value_t = Revisions::Conventional, requires_if("custom", "revision_palette"))]
    pub revisions: Revisions,
    /// Marks for --revisions custom: `kind=#RRGGBB[:lines],...` with
    /// kinds deleted, inserted, moved-from, moved-to and lines strike,
    /// double-strike, underline, double-underline, plain. Kinds left out
    /// keep their conventional mark.
    #[arg(long, value_name = "SPEC", value_parser = parse_palette, requires = "revisions")]
    pub revision_palette: Option<String>,
    /// PNG resolution in dots per inch (1-1200).
    #[arg(long, default_value_t = 96.0, value_name = "DPI", value_parser = parse_dpi)]
    pub dpi: f32,
    /// List the comments after the last page instead of in balloons beside
    /// the text. The commented text keeps its tint and a `[JR1]` marker,
    /// and the pages keep their own width.
    #[arg(long)]
    pub move_comments: bool,
    /// Also attach each comment as a PDF note at its anchor. Viewers paint
    /// a note as an icon over the text, so none is written by default and
    /// the flag stays out of --help: it is for the exceptional case.
    #[arg(long, hide = true)]
    pub comment_notes: bool,
    /// Keep only the pages a change touches: a body or note revision, a
    /// comment anchor, or the first page on which a revised header or
    /// footer paints (the first page when none does). Page numbers stay the
    /// document's.
    #[arg(long, alias = "changed-only")]
    pub changed_pages: bool,
    /// Only these pages, by the document's page numbers: `3`, `1-3,7`.
    /// Layout still covers the whole document; the PDF carries these pages
    /// and PNGs are named by them (`STEM-page-03.png`).
    #[arg(long, value_name = "SPEC", value_parser = validate_pages)]
    pub pages: Option<String>,
}

impl Default for RenderArgs {
    fn default() -> Self {
        Self {
            revisions: Revisions::Conventional,
            revision_palette: None,
            dpi: 96.0,
            move_comments: false,
            comment_notes: false,
            changed_pages: false,
            pages: None,
        }
    }
}

impl RenderArgs {
    /// Apply comment placement and page filtering to the renderer options.
    pub fn apply(&self, options: crate::convert::PdfOptions) -> crate::convert::PdfOptions {
        crate::convert::PdfOptions {
            comments: if self.move_comments {
                crate::convert::CommentPlacement::End
            } else {
                crate::convert::CommentPlacement::Margin
            },
            comment_notes: self.comment_notes,
            changed_only: self.changed_pages,
            ..options
        }
    }
}

/// `convert`'s format and Markdown flags.
#[derive(clap::Args, Debug, Serialize)]
pub struct MarkdownArgs {
    /// Input format [default: from the file: .md and .markdown are Markdown,
    /// a zip is Word].
    #[arg(
        short = 'f',
        long = "from",
        value_parser = input_format_parser(),
        value_name = "FORMAT",
        help_heading = "Formats"
    )]
    pub from: Option<Format>,
    /// Output format [default: from --output, else pdf for Word and docx for
    /// Markdown].
    #[arg(
        short = 't',
        long = "to",
        value_enum,
        value_name = "FORMAT",
        help_heading = "Formats"
    )]
    pub to: Option<Format>,
    /// Keep tracked changes (all), or write the document with every change
    /// accepted or rejected (pandoc's flag): CriticMarkup in Markdown, Word's
    /// revisions in a .docx. With --to md, the Markdown itself is resolved.
    #[arg(long, value_enum, value_name = "CHOICE", default_value_t = TrackChanges::All)]
    pub track_changes: TrackChanges,
    /// Keep comments and replies whose whole anchor is removed; requires --track-changes accept or reject.
    #[arg(long)]
    pub keep_nested_comment: bool,
    /// Markdown: read `{++`, `{--` and the other CriticMarkup delimiters as
    /// text.
    #[arg(long)]
    pub no_critic: bool,
    /// Markdown to Word: take styles, numbering, page setup, headers and
    /// footers from this .docx (pandoc's --reference-doc).
    #[arg(long, value_name = "FILE")]
    pub reference_doc: Option<PathBuf>,
    /// Markdown to Word: where images are found [default: the Markdown
    /// file's directory]. Only images that resolve inside that directory
    /// are read, symlinks resolved; a URL or a path out of it is written
    /// as its alt text, with a warning that says why.
    #[arg(long, value_name = "DIR")]
    pub resource_path: Option<PathBuf>,
    /// Markdown to Word: author of the tracked changes and comments that
    /// name none [default: the front matter's `author:`, else the
    /// --reference-doc's creator, else Jubarte].
    #[arg(short = 'a', long, value_name = "NAME")]
    pub author: Option<String>,
    /// Markdown to Word: timestamp (ISO 8601) of the tracked changes and
    /// comments that carry none [default: the front matter's
    /// `last_modified:`/`modified:`/`date:`/`created:`, else the
    /// --reference-doc's creation time, else now]; pin it
    /// (`-d 1970-01-01T00:00:00Z`) for reproducible output.
    #[arg(
        short = 'd',
        long = "datetime",
        value_name = "ISO8601",
        value_parser = parse_datetime
    )]
    pub datetime: Option<String>,
    /// Markdown to Word: the page size when there is no --reference-doc
    /// (one-inch margins either way); a reference's page setup wins.
    #[arg(long, value_enum, value_name = "SIZE", default_value_t = Page::Letter)]
    pub page: Page,
    /// Markdown to Word: turn Word's Track Changes on or off in the .docx
    /// written, whatever the Markdown's `track_changes:` key says [default:
    /// the key; without one, the --reference-doc's setting, else off].
    #[arg(long, value_enum, value_name = "SWITCH")]
    pub tracking: Option<Tracking>,
    /// Markdown to Word: set a manuscript as LaTeX's article class does
    /// (Latin Modern Roman 10pt on 12pt lines, justified, bold headings)
    /// and number its pages from page 2, centered in the footer. The front
    /// matter's own `styles:` entries, headers and footers win.
    #[arg(long)]
    pub scholar: bool,
    /// Word to Markdown: leave out the `<!-- page N of M -->` lines, and the
    /// layout pass that places them.
    #[arg(long)]
    pub no_page_markers: bool,
    /// Word to Markdown: write plain CommonMark with CriticMarkup for
    /// readers outside jubarte's dialect, instead of the agent view (`jubarte
    /// read`'s YAML header and id lines). Fields print as their text, and
    /// underline, highlight and strike are left out.
    #[arg(long)]
    pub commonmark: bool,
}

/// `--tracking`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tracking {
    /// New edits in Word are tracked.
    On,
    /// New edits in Word are not tracked.
    Off,
}

/// `--page`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Page {
    /// US Letter, 8.5 by 11 inches.
    Letter,
    /// ISO A4, 210 by 297 mm.
    A4,
}

impl From<Page> for crate::markdown::PageSize {
    fn from(choice: Page) -> Self {
        match choice {
            Page::Letter => Self::Letter,
            Page::A4 => Self::A4,
        }
    }
}

/// `jubarte convert --revisions`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Revisions {
    /// Red strike, blue underline, green double marks for moves.
    Conventional,
    /// Microsoft Word's own markup.
    Word,
    /// --revision-palette.
    Custom,
}

#[derive(clap::Args, Debug, Default, PartialEq, Serialize)]
/// Filters selecting individual tracked changes.
pub struct Selection {
    /// Only this change: `r12` (as the view and `jubarte changes` print
    /// it), `header1.r1`, or `r1+2` (both). Repeatable. Comment-body
    /// changes have no ids and stay tracked.
    #[arg(long = "id", value_name = "ID")]
    pub ids: Vec<String>,
    /// Only changes by this author. Repeatable.
    #[arg(long = "author", value_name = "NAME")]
    pub authors: Vec<String>,
    /// Only changes of this kind. Repeatable.
    #[arg(long = "kind", value_enum, value_name = "KIND")]
    pub kinds: Vec<KindArg>,
}

/// `--kind` values.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KindArg {
    /// Inserted text or structural elements.
    Insertion,
    /// Deleted text or structural elements.
    Deletion,
    /// Content moved between document locations.
    Move,
    /// Changes to text or paragraph formatting.
    Formatting,
    /// The kind `changes` prints for a deletion and the insertion that
    /// replaces it. It names no mark, so a filter of it is refused, with
    /// how to select one (by its id, or its marks by kind); hidden, it is
    /// no value to offer.
    #[value(hide = true)]
    Replacement,
}

fn parse_timeout(value: &str) -> Result<std::time::Duration, String> {
    let seconds: f64 = value
        .parse()
        .map_err(|_| format!("'{value}' is not a number of seconds"))?;
    if !(seconds.is_finite() && seconds > 0.0) {
        return Err(format!(
            "'{value}': the timeout must be a finite number of seconds above 0"
        ));
    }
    std::time::Duration::try_from_secs_f64(seconds).map_err(|e| format!("'{value}': {e}"))
}

/// `jubarte append --section-break`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SectionBreakArg {
    /// Each document starts on a new page.
    NextPage,
    /// Each document continues on the same page (a continuous section
    /// break with --keep-sections).
    Continuous,
    /// Nothing between the documents (continuous with --keep-sections).
    None,
}

impl From<SectionBreakArg> for crate::append::SectionBreak {
    fn from(arg: SectionBreakArg) -> Self {
        match arg {
            SectionBreakArg::NextPage => Self::NextPage,
            SectionBreakArg::Continuous => Self::Continuous,
            SectionBreakArg::None => Self::None,
        }
    }
}

fn styles() -> clap::builder::Styles {
    use clap::builder::styling::{AnsiColor, Effects};
    clap::builder::Styles::styled()
        .header(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .usage(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Yellow.on_default())
}

/// `--by`: a handle (`AC`, `@AC`) or a full name, never blank.
fn parse_author(value: &str) -> Result<String, String> {
    crate::markdown::author(value).map(str::to_string)
}

fn parse_dpi(value: &str) -> Result<f32, String> {
    let dpi: f32 = value.parse().map_err(|_| "DPI must be a number")?;
    if dpi.is_finite() && dpi > 0.0 && dpi <= 1200.0 {
        Ok(dpi)
    } else {
        Err("DPI must be finite and above 0, up to 1200".into())
    }
}

fn input_format_parser() -> impl clap::builder::TypedValueParser<Value = Format> {
    use clap::builder::TypedValueParser;
    clap::builder::PossibleValuesParser::new(["docx", "md", "markdown"]).map(|value| {
        if value == "docx" {
            Format::Docx
        } else {
            Format::Md
        }
    })
}

fn parse_context(value: &str) -> Result<usize, String> {
    value
        .parse::<u32>()
        .map(|n| n as usize)
        .map_err(|_| "context must be an integer between 0 and 4294967295".to_string())
}

fn parse_threshold(value: &str) -> Result<f64, String> {
    let ratio: f64 = value.parse().map_err(|_| "threshold must be a number")?;
    if ratio.is_finite() && (0.0..=1.0).contains(&ratio) {
        Ok(ratio)
    } else {
        Err("threshold must be finite and between 0 and 1".into())
    }
}

fn parse_palette(value: &str) -> Result<String, String> {
    crate::convert::RevisionPalette::parse(value)?;
    Ok(value.to_string())
}

/// A `--rules` item: a [`crate::validate::RULE_SETS`] name or an audit code.
fn parse_rule(value: &str) -> Result<String, String> {
    if crate::validate::RULE_SETS.contains(&value) || crate::audit::is_rule(value) {
        Ok(value.to_string())
    } else {
        let codes: Vec<&str> = crate::audit::RULES.iter().map(|rule| rule.0).collect();
        Err(format!(
            "expected one of {} or an audit code ({})",
            crate::validate::RULE_SETS.join(", "),
            codes.join(", ")
        ))
    }
}

// Validate without expanding ranges: parsing help or arguments never allocates
// a potentially enormous list of page indices.
fn validate_pages(value: &str) -> Result<String, String> {
    let page = |s: &str| {
        let s = s.trim();
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("'{s}' is not a page number"));
        }
        let n = s
            .parse::<usize>()
            .map_err(|_| format!("'{s}' is not a page number"))?;
        if n == 0 {
            Err("pages are counted from 1".to_string())
        } else {
            Ok(n)
        }
    };
    for item in value.split(',') {
        let item = item.trim();
        if item.is_empty() {
            return Err("empty item in page selection".into());
        }
        if let Some((first, last)) = item.split_once('-') {
            if page(first)? > page(last)? {
                return Err(format!("'{item}' runs backwards"));
            }
        } else {
            page(item)?;
        }
    }
    Ok(value.to_string())
}

fn serialize_timeout<S: serde::Serializer>(
    timeout: &Option<std::time::Duration>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    timeout.map(|t| t.as_secs_f64()).serialize(serializer)
}

/// Cross-option constraints whose meaning depends on an enum value or output
/// extension. These are usage errors, checked before any runtime I/O.
/// The ids of [`ReadArgs`], as the top-level shorthand declares them.
const READ_OPTION_IDS: [&str; 11] = [
    "track_changes",
    "comments",
    "dates",
    "no_page_markers",
    "paragraphs",
    "head",
    "tail",
    "changed",
    "by",
    "json",
    "tables",
];

/// `diff` writes no JSON snapshot: its `-o x.json` and `--to json` are
/// refused here, so the binary and both bindings exit 2 alike instead of
/// writing a patch into a `.json`.
fn json_not_yet(matches: &clap::ArgMatches, task: Task) -> Option<String> {
    let to = matches.try_get_one::<Format>("to").ok().flatten().copied();
    let said = matches
        .try_get_one::<PathBuf>("output")
        .ok()
        .flatten()
        .and_then(|path| task.known(path));
    (to == Some(Format::Json) || said == Some(Format::Json))
        .then(|| format!("{} does not write .json yet", task.name()))
}

/// What `compare`'s `-o` writes (`None`: the view is printed), refusing
/// the options that do not go with it: read's `--json` and `--tables` (one
/// document), the view's options with a file that is not the view, and the
/// rendering options without a PDF or PNG.
fn compare_output(
    output: Option<&Path>,
    given: &dyn Fn(&str) -> bool,
) -> Result<Option<Format>, String> {
    if given("json") {
        return Err(
            "--json reads one document; `-o FILE.json` writes the redline's snapshot".to_string(),
        );
    }
    if given("tables") {
        return Err("--tables reads one document".to_string());
    }
    let kind = output_format(Task::Compare, output, None)?;
    if kind.is_some_and(|kind| kind != Format::Md) && READ_OPTION_IDS.into_iter().any(given) {
        return Err(
            "read options shape the view: print it (no -o) or write it (-o FILE.md)".to_string(),
        );
    }
    if !matches!(kind, Some(Format::Pdf | Format::Png))
        && let Some(flag) = RENDER_ONLY.into_iter().find(|flag| given(flag))
    {
        return Err(format!(
            "--{} applies to PDF or PNG output only",
            flag.replace('_', "-")
        ));
    }
    Ok(kind)
}

/// The diagnosis for a view-only flag beside --for-edit.
fn read_view_conflict(flag: &str) -> String {
    format!(
        "{flag} shapes the Markdown view, and --for-edit prints the working document JSON, \
         which has no view; drop {flag}, or drop --for-edit to read the view"
    )
}

/// As in the swallowed-FILE diagnosis, rebuild a task error through clap.
/// A generic usage line gives a valid invocation instead of repeating the
/// conflicting flags. The same model serves native and binding parsers.
fn read_usage_error(command: &clap::Command, message: String) -> clap::Error {
    let mut task = command
        .find_subcommand("read")
        .unwrap_or(command)
        .clone()
        .override_usage(format!("{} read [OPTIONS] <FILE>", command.get_name()));
    task.set_bin_name(format!("{} read", command.get_name()));
    task.error(clap::error::ErrorKind::ArgumentConflict, message)
}

/// What `read -o PATH` (and the one-document shorthand's -o) writes: `.md`
/// the view, `.json` the snapshot. `given` says which argument ids the
/// command line spells out.
fn read_output(output: &Path, given: &dyn Fn(&str) -> bool) -> Result<Format, String> {
    let kind = output_format(Task::Read, Some(output), None)?.unwrap_or(Format::Md);
    if given("tables") {
        return Err("--tables prints the grids; drop -o".to_string());
    }
    if given("for_edit") && kind != Format::Json {
        return Err("--for-edit prints JSON: name the file FILE.json or drop -o".to_string());
    }
    if given("json") && kind != Format::Json {
        return Err("--json writes the snapshot: name the file FILE.json".to_string());
    }
    // A .json snapshot is --json's: --track-changes settles its text, and
    // the view's other options change nothing in it.
    Ok(kind)
}

/// `--revision-palette` without `--revisions custom`, in any task that
/// draws revisions (convert, edit, compare, accept, reject, append, scrub):
/// a usage error before any document is read (Codex on #440 and #441).
fn palette_without_custom(args: &clap::ArgMatches) -> bool {
    args.try_get_one::<String>("revision_palette")
        .ok()
        .flatten()
        .is_some()
        && args.try_get_one::<Revisions>("revisions").ok().flatten() != Some(&Revisions::Custom)
}

fn validate_matches(
    matches: &clap::ArgMatches,
    command: &mut clap::Command,
) -> Result<(), clap::Error> {
    use clap::{error::ErrorKind, parser::ValueSource};
    let Some((name, args)) = matches.subcommand() else {
        let given = |id: &str| {
            matches.try_contains_id(id).unwrap_or(false)
                && matches.value_source(id) == Some(ValueSource::CommandLine)
        };
        let two = given("modified_pos") || given("modified");
        let output = matches.get_one::<PathBuf>("output").map(PathBuf::as_path);
        if two {
            compare_output(output, &given)
                .map_err(|message| command.error(ErrorKind::ArgumentConflict, message))?;
            if palette_without_custom(matches) {
                return Err(command.error(
                    ErrorKind::ArgumentConflict,
                    "--revision-palette needs --revisions custom",
                ));
            }
        } else {
            // One document: `read`, whose -o writes the view or the snapshot.
            if let Some(output) = output {
                read_output(output, &given)
                    .map_err(|message| read_usage_error(command, message))?;
            }
            // Compare's own options and the rendering ones mean nothing to
            // read.
            if let Some(flag) = COMPARE_ONLY.into_iter().find(|id| given(id)) {
                return Err(command.error(
                    ErrorKind::ArgumentConflict,
                    format!(
                        "--{} is a compare option: name two documents",
                        flag.replace('_', "-")
                    ),
                ));
            }
            if let Some(flag) = RENDER_ONLY.into_iter().find(|id| given(id)) {
                return Err(command.error(
                    ErrorKind::ArgumentConflict,
                    format!(
                        "--{} draws a PDF or PNG, which read does not write: \
                         `convert FILE -o FILE.pdf` or `compare A B -o FILE.pdf` does",
                        flag.replace('_', "-")
                    ),
                ));
            }
        }
        // `jubarte --head 2 read a.docx`: options before a task make clap read
        // the task's name as ORIGINAL.
        if let Some(task) = matches
            .try_get_one::<PathBuf>("original_pos")
            .ok()
            .flatten()
            .and_then(|p| p.to_str())
            .and_then(|name| command.find_subcommand(name))
            .map(|task| task.get_name().to_string())
        {
            return Err(command.error(
                ErrorKind::ArgumentConflict,
                format!(
                    "options go after the task: `{} {task} … [OPTIONS]`",
                    command.get_name()
                ),
            ));
        }
        // One bare word is more likely a mistyped task than a document.
        let lone = |id: &str| {
            matches
                .try_get_one::<PathBuf>(id)
                .ok()
                .flatten()
                .filter(|_| matches.value_source(id) == Some(ValueSource::CommandLine))
        };
        if lone("modified_pos").is_none()
            && lone("modified").is_none()
            && let Some(file) = lone("original_pos")
            && file.extension().is_none()
        {
            return Err(command.error(
                ErrorKind::InvalidSubcommand,
                format!(
                    "'{}' is not a task; to read a file without an extension, use `read FILE`",
                    file.display()
                ),
            ));
        }
        return Ok(());
    };
    if let Some(message) = Task::of_name(name)
        .filter(|task| *task == Task::Diff)
        .and_then(|task| json_not_yet(args, task))
    {
        let mut task = command.find_subcommand(name).expect("parsed task").clone();
        task.set_bin_name(format!("{} {name}", command.get_name()));
        return Err(task.error(ErrorKind::ValueValidation, message));
    }
    if name == "validate"
        && args
            .get_many::<String>("rules")
            .is_some_and(|mut rules| rules.any(|rule| rule == "edits"))
        && !args.contains_id("author")
    {
        let mut task = command.find_subcommand(name).expect("parsed task").clone();
        task.set_bin_name(format!("{} {name}", command.get_name()));
        return Err(task.error(
            ErrorKind::MissingRequiredArgument,
            "--rules edits needs --author, and --original or --baseline",
        ));
    }
    if name == "compare"
        && !["modified", "modified_pos"]
            .into_iter()
            .any(|id| args.try_contains_id(id).unwrap_or(false))
    {
        let mut task = command.find_subcommand(name).expect("parsed task").clone();
        task.set_bin_name(format!("{} {name}", command.get_name()));
        return Err(task.error(
            ErrorKind::MissingRequiredArgument,
            "compare needs <MODIFIED> (or -m FILE)",
        ));
    }
    if name == "__edit-template" {
        return edit_template::validate(args, command);
    }
    if name == "__edit-v2" {
        return edit_actions::validate(args, command);
    }
    let mut task = command.find_subcommand(name).expect("parsed task").clone();
    task.set_bin_name(format!("{} {name}", command.get_name()));
    let error =
        |task: &mut clap::Command, message: &str| task.error(ErrorKind::ArgumentConflict, message);
    let given = |id: &str| {
        args.try_contains_id(id).unwrap_or(false)
            && args.value_source(id) == Some(ValueSource::CommandLine)
    };
    if name == "read"
        && let Some(output) = args.get_one::<PathBuf>("output")
    {
        read_output(output, &given).map_err(|message| read_usage_error(command, message))?;
    }
    if name == "compare" {
        compare_output(
            args.get_one::<PathBuf>("output").map(PathBuf::as_path),
            &given,
        )
        .map_err(|message| error(&mut task, &message))?;
    }
    if let Some(writer) = Task::of_name(name).filter(|task| {
        matches!(
            task,
            Task::Accept | Task::Reject | Task::Append | Task::Scrub
        )
    }) {
        let output = args.get_one::<PathBuf>("output").map(PathBuf::as_path);
        let kind =
            output_format(writer, output, None).map_err(|message| error(&mut task, &message))?;
        if !matches!(kind, Some(Format::Pdf | Format::Png))
            && let Some(flag) = RENDER_ONLY.into_iter().find(|flag| given(flag))
        {
            return Err(error(
                &mut task,
                &format!(
                    "--{} applies to PDF or PNG output only",
                    flag.replace('_', "-")
                ),
            ));
        }
    }
    if matches!(name, "edit" | "add")
        && let Some(plan) = args.get_one::<PathBuf>("plan")
    {
        // Only the flags the command line spells out: a default is no
        // choice the plan could contradict.
        let given: Vec<String> = args
            .ids()
            .filter(|id| args.value_source(id.as_str()) == Some(ValueSource::CommandLine))
            .map(|id| format!("--{}", id.as_str().replace('_', "-")))
            .collect();
        let given: Vec<&str> = given.iter().map(String::as_str).collect();
        let recording = given.contains(&"--location");
        if let Some(message) =
            crate::edit::record::plan_flag_conflict(recording, &given, &plan.display().to_string())
        {
            return Err(error(&mut task, &message));
        }
    }
    // --out-dir's value is optional, so `edit --out-dir a.docx -p p1`
    // would read the document as the directory.
    if matches!(name, "edit" | "add")
        && let Some(dir) = args.try_get_one::<PathBuf>("out_dir").ok().flatten()
        && names_word_document(dir)
    {
        return Err(error(&mut task, &swallowed_file("--out-dir", dir)));
    }
    // So is --plan's (D16): a plan is JSON, never a Word document.
    if matches!(name, "edit" | "add")
        && let Some(plan) = args.get_one::<PathBuf>("plan")
        && names_word_document(plan)
    {
        return Err(error(&mut task, &swallowed_file("--plan", plan)));
    }
    // edit and add write one Word document: a known extension that is not
    // .docx names the task that writes it.
    if matches!(name, "edit" | "add")
        && let Some(output) = args.get_one::<PathBuf>("output")
        && let Some(kind) = Format::of_path(output).filter(|kind| *kind != Format::Docx)
    {
        // As typed (`.txt`, `.markdown`), which the named task also takes.
        let ext = output.extension().map_or_else(
            || kind.name().to_string(),
            |e| e.to_string_lossy().to_lowercase(),
        );
        let how = match kind {
            Format::Md | Format::Json => format!("`jubarte read FILE_modified.docx -o v.{ext}`"),
            _ => format!("`jubarte convert FILE_modified.docx -o r.{ext}`"),
        };
        return Err(error(
            &mut task,
            &format!("{name} writes a .docx; {how} writes .{ext} from it"),
        ));
    }
    if palette_without_custom(args) {
        return Err(error(
            &mut task,
            "--revision-palette needs --revisions custom",
        ));
    }
    if name == "diff" {
        // Text only: Word, PDF and PNG redlines are compare's.
        output_format(
            Task::Diff,
            args.get_one::<PathBuf>("output").map(PathBuf::as_path),
            args.get_one::<Format>("to").copied(),
        )
        .map_err(|message| error(&mut task, &message))?;
        let format = *args
            .get_one::<PatchFormat>("format")
            .expect("default format");
        let (text_view, theirs) = if format.is_text_view() {
            (true, &["columns", "author", "datetime"][..])
        } else {
            (false, &["context", "accept_changes", "full_lines"][..])
        };
        // CriticMarkup carries no owner and is not wrapped: the patch's
        // flags would be dropped silently (pi-14 C3).
        let patch_only: &[&str] = if format == PatchFormat::Critic {
            &["columns", "author", "datetime"]
        } else {
            &[]
        };
        let given = |flag: &&&str| args.value_source(flag) == Some(ValueSource::CommandLine);
        if let Some(flag) = patch_only.iter().find(given) {
            return Err(error(
                &mut task,
                &format!("--{flag} applies to the paragraph patch only (--format patch)"),
            ));
        }
        if let Some(flag) = theirs.iter().find(given) {
            let flag = flag.replace('_', "-");
            return Err(error(
                &mut task,
                &if text_view {
                    format!("--{flag} does not apply to this text diff format")
                } else {
                    format!(
                        "--{flag} requires a text diff format (github, word, normal, context or side-by-side)"
                    )
                },
            ));
        }
    }
    if name == "convert" {
        if args.get_flag("keep_nested_comment")
            && args.get_one::<TrackChanges>("track_changes") == Some(&TrackChanges::All)
        {
            return Err(error(
                &mut task,
                "--keep-nested-comment requires --track-changes accept or reject",
            ));
        }
        // The retired --pdf and --png name a format, as --to does.
        let alias = match (args.get_flag("pdf"), args.get_flag("png")) {
            (true, true) => return Err(error(&mut task, "--pdf and --png name two formats")),
            (true, false) => Some(Format::Pdf),
            (false, true) => Some(Format::Png),
            (false, false) => None,
        };
        let given = args.get_one::<Format>("to").copied();
        if let (Some(to), Some(alias)) = (given, alias)
            && to != alias
        {
            return Err(error(
                &mut task,
                &format!(
                    "--to {} and --{} name two formats: drop one",
                    to.name(),
                    alias.name()
                ),
            ));
        }
        let output = args.get_one::<PathBuf>("output").map(PathBuf::as_path);
        let kind = output_format(Task::Convert, output, given.or(alias))
            .map_err(|message| error(&mut task, &message))?;
        let update_fields = args.get_flag("update_fields");
        if update_fields && kind != Some(Format::Docx) {
            return Err(error(
                &mut task,
                "--update-fields writes a .docx: give -o FILE.docx or --to docx",
            ));
        }
        if matches!(kind, Some(Format::Docx | Format::Md | Format::Json)) {
            let convert_only = ["compress", "report", "font_report", "fail_on_substitution"];
            for flag in RENDER_ONLY.into_iter().chain(convert_only) {
                // The fields report is the one report a .docx output has.
                if flag == "report" && update_fields {
                    continue;
                }
                if args.value_source(flag) == Some(ValueSource::CommandLine) {
                    return Err(error(
                        &mut task,
                        &format!(
                            "--{} applies to PDF or PNG output only",
                            flag.replace('_', "-")
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

/// The ids of the options only a comparison takes; `jubarte FILE` refuses
/// them.
const COMPARE_ONLY: [&str; 9] = [
    "author",
    "datetime",
    "detail_threshold",
    "mode",
    "powertools_faithful",
    "no_paragraph_merge",
    "reference_doc",
    "critic",
    "resource_path",
];

/// The [`RenderArgs`] ids: options of a PDF or PNG output only.
const RENDER_ONLY: [&str; 7] = [
    "move_comments",
    "comment_notes",
    "changed_pages",
    "pages",
    "dpi",
    "revisions",
    "revision_palette",
];

/// Normalize edit routing identically for parsing and usage-error reporting.
pub fn edit_arguments(
    command: &clap::Command,
    arguments: &[std::ffi::OsString],
) -> Vec<std::ffi::OsString> {
    edit_actions::route(&hoist_quiet(command, arguments), command)
}

/// Requested protocol on a recognized typed edit route, respecting values and `--`:
/// the action name, whether the protocol is JSONL (else JSON), and the FILE
/// operand the tokens named when they carried one before the grammar failure
/// (used to keep the usage envelope's `source.path` honest).
pub fn typed_edit_protocol(
    arguments: &[std::ffi::OsString],
) -> Option<(String, bool, Option<String>)> {
    let model = top_level_command();
    let routed = edit_arguments(&model, arguments);
    if routed.get(1).is_none_or(|a| a != "__edit-v2") {
        return None;
    }
    let typed = model.find_subcommand("__edit-v2")?;
    let tokens: Vec<_> = routed
        .iter()
        .skip(2)
        .filter_map(|a| a.to_str())
        .take_while(|a| *a != "--")
        .collect();
    let (_, at) = spelled_options(typed, &tokens);
    let action = *tokens.get(at)?;
    let command = typed.find_subcommand(action)?;
    let mut rest = &tokens[at + 1..];
    let mut json = false;
    let mut jsonl = false;
    let mut file = None;
    while !rest.is_empty() {
        let (options, next) = spelled_options(command, rest);
        for option in options {
            json |= option.name == "--json";
            jsonl |= option.name == "--jsonl";
        }
        // The action's leading positional is its FILE operand.
        if file.is_none()
            && let Some(operand) = rest.get(next)
        {
            file = Some(operand.to_string());
        }
        rest = rest.get(next + 1..).unwrap_or_default();
    }
    (json || jsonl).then(|| (action.to_string(), jsonl, file))
}

/// `argv` (the arguments after the program name) as a dry run's `next:`
/// command: without its `--dry-run` flag, and with `--force` before any
/// `--` when `force`. A `--dry-run` that is no flag stays: a FILE after
/// `--`, or an option's value (`--text --dry-run`). Pure: no I/O.
pub fn next_command(argv: &[String], force: bool) -> Vec<String> {
    let model = top_level_command();
    let arguments: Vec<std::ffi::OsString> = std::iter::once("jubarte".into())
        .chain(argv.iter().map(Into::into))
        .collect();
    // Read as the parser reads it (the typed edit forms routed), then
    // matched back to `argv` by count: routing moves no `--dry-run`.
    let routed = edit_arguments(&model, &arguments);
    let tokens: Vec<&str> = routed
        .iter()
        .skip(1)
        .filter_map(|a| a.to_str())
        .take_while(|a| *a != "--")
        .collect();
    let mut flags = std::collections::BTreeSet::new();
    let mut command = &model;
    let mut start = 0;
    loop {
        let (options, next) = spelled_options(command, &tokens[start..]);
        flags.extend(
            options
                .iter()
                .filter(|option| option.name == "--dry-run")
                .map(|option| start + option.at),
        );
        let Some(word) = tokens.get(start + next) else {
            break;
        };
        if let Some(task) = command.find_subcommand(word) {
            command = task;
        }
        start += next + 1;
    }
    // Which of the tokens spelled `--dry-run`, in order, are the flag.
    let dropped: std::collections::BTreeSet<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| **token == "--dry-run")
        .enumerate()
        .filter(|(_, (at, _))| flags.contains(at))
        .map(|(nth, _)| nth)
        .collect();
    let mut next = Vec::with_capacity(argv.len() + 1);
    let mut spelled = 0;
    let mut delimited = false;
    for arg in argv {
        if !delimited && arg == "--" {
            delimited = true;
            if force {
                next.push("--force".to_string());
            }
        }
        if !delimited && arg == "--dry-run" {
            spelled += 1;
            if dropped.contains(&(spelled - 1)) {
                continue;
            }
        }
        next.push(arg.clone());
    }
    if force && !delimited {
        next.push("--force".to_string());
    }
    next
}

/// The first positional operand the tokens name for `cmd`, when they carry
/// one before any grammar failure: options and their values are skipped
/// through the command model's own spelling, so an option's value is never
/// mistaken for the operand. Recognized-mode usage envelopes use it to name
/// the file the caller supplied (proposal §11: `source` keeps the path).
pub fn first_positional(cmd: &clap::Command, tokens: &[&str]) -> Option<String> {
    let (_, at) = spelled_options(cmd, tokens);
    tokens.get(at).map(ToString::to_string)
}

impl Cli {
    /// Parse native arguments with clap and validate cross-option constraints.
    pub fn try_parse_from<I, T>(arguments: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let mut command = top_level_command();
        let arguments: Vec<std::ffi::OsString> = arguments.into_iter().map(Into::into).collect();
        let arguments = edit_arguments(&command, &arguments);
        let matches = command
            .try_get_matches_from_mut(arguments.clone())
            .map_err(|e| {
                let tokens: Vec<String> = arguments
                    .iter()
                    .map(|a| a.to_string_lossy().into_owned())
                    .collect();
                usage_error(&command, &tokens, e)
            })?;
        validate_matches(&matches, &mut command)?;
        let mut cli = Self::from_arg_matches(&matches)?;
        if let Some(task) = cli.command.as_mut() {
            fill_flag_operations(task, &matches, &mut command)?;
            resolve_plan(task);
            let typed: Vec<String> = arguments
                .iter()
                .skip(1)
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            apply_retired(task, &retired_spellings(typed.iter().map(String::as_str)));
            fill_task_date(task);
        } else {
            fill_date(&mut cli.compare.datetime);
        }
        match (cli.command.as_mut(), matches.subcommand()) {
            (Some(Command::Compare(compare)), Some((_, args))) => {
                compare.view_given = view_given(args);
            }
            (None, _) => cli.compare.view_given = view_given(&matches),
            _ => {}
        }
        Ok(cli)
    }
}

/// Whether `matches` spells out any of `read`'s options.
fn view_given(matches: &clap::ArgMatches) -> bool {
    READ_OPTION_IDS.into_iter().any(|id| {
        matches.try_contains_id(id).unwrap_or(false)
            && matches.value_source(id) == Some(clap::parser::ValueSource::CommandLine)
    })
}

fn compare_json(compare: &CompareArgs) -> serde_json::Value {
    let mut args = serde_json::to_value(compare).expect("UTF-8 CLI arguments");
    // Named inputs win, as in the native executor. Keep the raw positional
    // fields too, so adapters never have to reproduce precedence rules.
    args["original"] =
        serde_json::to_value(compare.original.as_ref().or(compare.original_pos.as_ref()))
            .expect("UTF-8 path");
    args["modified"] =
        serde_json::to_value(compare.modified.as_ref().or(compare.modified_pos.as_ref()))
            .expect("UTF-8 path");
    let input_format = |path: Option<&PathBuf>| {
        path.and_then(|p| Format::of_path(p))
            .filter(|f| matches!(f, Format::Docx | Format::Md))
    };
    args["old_format"] = serde_json::to_value(input_format(
        compare.original.as_ref().or(compare.original_pos.as_ref()),
    ))
    .expect("format");
    args["new_format"] = serde_json::to_value(input_format(
        compare.modified.as_ref().or(compare.modified_pos.as_ref()),
    ))
    .expect("format");
    // validate_matches refused a format compare cannot write.
    let kind = output_format(Task::Compare, compare.output.as_deref(), None)
        .ok()
        .flatten();
    args["output_format"] = serde_json::to_value(kind).expect("format");
    // The view's options, when the view may be what compare gives: printed
    // without -o, or written to a .md. Only the host, once it has read
    // both documents, knows whether a .md is CriticMarkup (both Markdown:
    // `view_given` must be false) or the view.
    if matches!(kind, None | Some(Format::Md)) {
        args["view"] = serde_json::to_value(&compare.read).expect("UTF-8 CLI arguments");
    }
    args
}

/// Whether compare's result is surely the agent view: printed, or a .md of
/// documents whose names do not both say Markdown. A host without `read`
/// refuses it.
fn view_is_output(args: &serde_json::Value) -> bool {
    let markdown = |format: &serde_json::Value| format == "md";
    args["output_format"].is_null()
        || (args["output_format"] == "md"
            && !(markdown(&args["old_format"]) && markdown(&args["new_format"])))
}

/// A comparison whose result is the agent view, in an adapter without
/// `read`: only `-o` with a file that is not the view works there.
fn view_not_supported(command: &mut clap::Command) -> clap::Error {
    command.error(
        clap::error::ErrorKind::InvalidSubcommand,
        "this adapter prints no agent view; write the redline with -o FILE.docx",
    )
}

fn error_json(error: &clap::Error) -> String {
    serde_json::json!({
        "exit_code": error.exit_code(),
        "stream": if error.use_stderr() { "stderr" } else { "stdout" },
        "text": error.to_string(),
    })
    .to_string()
}

/// Whether a command line (without the program name) asks for JSON: a
/// `--json` flag before any `--`. An error then prints on stdout too, as one
/// [`json_error`] line, for a reader that pipes stdout to a JSON parser
/// (#1158); stderr and the exit code stay as they are.
pub fn wants_json(argv: &[String]) -> bool {
    // Read as the typed edit protocol reads it: a `--json` where an option
    // takes its value (`--text --json`) is no request for JSON.
    let model = top_level_command();
    let arguments: Vec<std::ffi::OsString> = std::iter::once("jubarte".into())
        .chain(argv.iter().map(Into::into))
        .collect();
    let routed = edit_arguments(&model, &arguments);
    let tokens: Vec<&str> = routed
        .iter()
        .skip(1)
        .filter_map(|a| a.to_str())
        .take_while(|a| *a != "--")
        .collect();
    let mut command = &model;
    let mut rest = &tokens[..];
    loop {
        let (options, next) = spelled_options(command, rest);
        if options.iter().any(|option| option.name == "--json") {
            return true;
        }
        let Some(word) = rest.get(next) else {
            return false;
        };
        if let Some(task) = command.find_subcommand(word) {
            command = task;
        }
        rest = &rest[next + 1..];
    }
}

/// `{"error":{"code":…,"message":…,"usage":…}}` and a newline: the line an
/// error prints on stdout under `--json`. `usage` only when given.
pub fn json_error(code: &str, message: &str, usage: Option<&str>) -> String {
    let mut error = serde_json::json!({ "code": code, "message": message });
    if let Some(usage) = usage {
        error["usage"] = usage.into();
    }
    format!("{}\n", serde_json::json!({ "error": error }))
}

/// The [`json_error`] line of a runtime error that ends a run with `exit`,
/// `message` being what follows `error: ` on stderr. Its code is the
/// message's leading `CODE:` (`STALE_SOURCE: …`), else `USAGE` for an exit
/// 2 and `ERROR` for any other.
pub fn runtime_json_error(message: &str, exit: u8) -> String {
    let lead = message.split_once(": ").map_or("", |(lead, _)| lead);
    let coded = lead.len() >= 3
        && lead.starts_with(|c: char| c.is_ascii_uppercase())
        && lead
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    let code = if coded {
        lead
    } else if exit == 2 {
        "USAGE"
    } else {
        "ERROR"
    };
    json_error(code, message, None)
}

/// The [`json_error`] line of a usage error clap reports for `argv` (no
/// program name) run as `program`: its message without `error: `, on one
/// line (a list clap names under it joined on, each tip after it), and the
/// task's usage (`jubarte read [OPTIONS] <FILE>`; clap's own usage, which
/// names the flags given, when no task is named), a line per form. `None`
/// for help and version, which are not errors.
pub fn usage_json_error(error: &clap::Error, argv: &[String], program: &str) -> Option<String> {
    let (message, usage) = usage_parts(error, argv, program)?;
    Some(json_error("USAGE", &message, usage.as_deref()))
}

/// The [`json_error`] line of a removed spelling's error, `message` being
/// what follows `error: ` ([`removed_error`]), with the usage
/// [`usage_json_error`] gives clap's `error` for the same `argv`.
pub fn removed_json_error(
    error: &clap::Error,
    argv: &[String],
    program: &str,
    message: &str,
) -> String {
    let usage = usage_parts(error, argv, program).and_then(|(_, usage)| usage);
    json_error("USAGE", message, usage.as_deref())
}

/// [`usage_json_error`]'s message and usage.
fn usage_parts(
    error: &clap::Error,
    argv: &[String],
    program: &str,
) -> Option<(String, Option<String>)> {
    if !error.use_stderr() {
        return None;
    }
    #[derive(Clone, Copy)]
    enum Part {
        Message,
        Tip,
        Usage,
        Other,
    }
    let text = error.render().to_string();
    let (mut message, mut tips, mut usage) = (String::new(), Vec::<String>::new(), Vec::new());
    let mut part = Part::Other;
    // Blank lines part clap's text: the message, its tips, the usage and
    // the pointer to --help; an indented line goes on with its part.
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(first) = line.strip_prefix("error: ") {
            message.push_str(first);
            part = Part::Message;
        } else if let Some(tip) = trimmed.strip_prefix("tip: ") {
            tips.push(tip.to_string());
            part = Part::Tip;
        } else if let Some(first) = line.strip_prefix("Usage: ") {
            usage.push(first.trim());
            part = Part::Usage;
        } else if trimmed.is_empty() {
            part = Part::Other;
        } else {
            match (part, tips.last_mut()) {
                (Part::Message, _) => {
                    message.push(' ');
                    message.push_str(trimmed);
                }
                (Part::Tip, Some(tip)) => {
                    tip.push(' ');
                    tip.push_str(trimmed);
                }
                (Part::Usage, _) => usage.push(trimmed),
                _ => {}
            }
        }
    }
    for tip in tips {
        message.push_str(&format!(" (tip: {tip})"));
    }
    let usage = task_usage(argv, program).or_else(|| (!usage.is_empty()).then(|| usage.join("\n")));
    Some((message, usage))
}

/// The usage of the task `argv` names (`jubarte read [OPTIONS] <FILE>`), a
/// line per form, or `None` when its first word is no task.
fn task_usage(argv: &[String], program: &str) -> Option<String> {
    let mut model = Cli::command().bin_name(program.to_string());
    model.build();
    let name = argv
        .iter()
        .take_while(|a| *a != "--")
        .find(|a| !a.starts_with('-'))?;
    let usage = model.find_subcommand_mut(name)?.render_usage().to_string();
    let usage = usage.strip_prefix("Usage: ")?;
    Some(usage.lines().map(str::trim).collect::<Vec<_>>().join("\n"))
}

/// The usage error for a task the host (`program`) does not have.
fn missing_task(command: &mut clap::Command, program: &str, task: &str) -> clap::Error {
    command.error(
        clap::error::ErrorKind::InvalidSubcommand,
        format!("this {program} has no {task} task; the `jubarte` binary has it"),
    )
}

/// The `{exit_code, command, args}` value for a subcommand clap matched:
/// the task with its flag operations, plan, retired spellings and date
/// resolved, and compare's view or read routing. [`parse_json`] adds
/// `args.quiet` and `notes` to it.
fn parse_subcommand_json(
    command: &mut clap::Command,
    matches: &clap::ArgMatches,
    arguments: &[String],
    accepts: &dyn Fn(&str) -> bool,
) -> Result<serde_json::Value, clap::Error> {
    let mut task = Command::from_arg_matches(matches)?;
    fill_flag_operations(&mut task, matches, command)?;
    resolve_plan(&mut task);
    apply_retired(
        &mut task,
        &retired_spellings(arguments.iter().map(String::as_str)),
    );
    fill_task_date(&mut task);
    let mut value = serde_json::to_value(&task).expect("UTF-8 CLI arguments");
    if let Command::Compare(compare) = &mut task {
        if let Some((_, args)) = matches.subcommand() {
            compare.view_given = view_given(args);
        }
        value["args"] = compare_json(compare);
        if view_is_output(&value["args"]) && !accepts("read") {
            return Err(view_not_supported(command));
        }
    }
    if let Command::Diff {
        old,
        new,
        output,
        to,
        from,
        ..
    } = &task
    {
        let input_format = |path: &std::path::Path| {
            from.or_else(|| {
                Format::of_path(path).filter(|f| matches!(f, Format::Docx | Format::Md))
            })
        };
        value["args"]["old_format"] = serde_json::to_value(input_format(old)).expect("format");
        value["args"]["new_format"] = serde_json::to_value(input_format(new)).expect("format");
        value["args"]["output_format"] = serde_json::to_value(
            output_format(Task::Diff, output.as_deref(), *to)
                .ok()
                .flatten(),
        )
        .expect("format");
    }
    if let Command::Convert {
        output,
        markdown,
        pdf,
        png,
        ..
    } = &task
    {
        let alias = pdf
            .then_some(Format::Pdf)
            .or_else(|| png.then_some(Format::Png));
        value["args"]["output_format"] = serde_json::to_value(
            output_format(Task::Convert, output.as_deref(), markdown.to.or(alias))
                .ok()
                .flatten(),
        )
        .expect("format");
    }
    if let Command::Edit {
        file,
        options,
        plan,
        operations,
        ..
    }
    | Command::Add {
        file,
        options,
        plan,
        operations,
        ..
    } = &task
    {
        let recording = plan.is_some() && !operations.is_empty();
        let (output, out_dir) = match options.target(file) {
            _ if recording => (None, None),
            EditTarget::Single(path) => (Some(path), None),
            EditTarget::Bundle(dir) => (None, Some(dir)),
        };
        let slashed = |path: Option<&std::path::PathBuf>| {
            path.map(|p| serde_json::Value::String(slash_path(p)))
                .unwrap_or(serde_json::Value::Null)
        };
        value["args"]["plan"] = slashed(plan.as_ref());
        value["args"]["output"] = slashed(output.as_ref());
        value["args"]["out_dir"] = slashed(out_dir.as_ref());
    }
    let written = match &task {
        Command::Accept { output, .. } => Some((Task::Accept, Some(output.as_path()))),
        Command::Reject { output, .. } => Some((Task::Reject, Some(output.as_path()))),
        Command::Append { output, .. } => Some((Task::Append, Some(output.as_path()))),
        Command::Scrub { output, .. } => Some((Task::Scrub, Some(output.as_path()))),
        Command::Read { output, .. } => Some((Task::Read, output.as_deref())),
        _ => None,
    };
    if let Some((writer, output)) = written {
        value["args"]["output_format"] =
            serde_json::to_value(output_format(writer, output, None).ok().flatten())
                .expect("format");
    }
    value["exit_code"] = 0.into();
    Ok(value)
}

fn parse_implicit_compare_json(
    command: &mut clap::Command,
    matches: &clap::ArgMatches,
    model: &clap::Command,
    program: &str,
    accepts: &dyn Fn(&str) -> bool,
) -> Result<serde_json::Value, clap::Error> {
    let mut compare = CompareArgs::from_arg_matches(matches)?;
    fill_date(&mut compare.datetime);
    if let Some(name) = compare
        .original_pos
        .as_ref()
        .and_then(|p| p.to_str())
        .and_then(|name| model.find_subcommand(name))
        .map(|task| task.get_name().to_string())
        .filter(|name| !accepts(name))
    {
        return Err(missing_task(command, program, &name));
    }
    if compare.modified.is_none() && compare.modified_pos.is_none() {
        if !accepts("read") {
            return Err(command.error(
                clap::error::ErrorKind::InvalidSubcommand,
                "this task is not supported by this adapter",
            ));
        }
        let mut args = serde_json::to_value(&compare.read).expect("UTF-8 CLI arguments");
        args["file"] =
            serde_json::to_value(compare.original.as_ref().or(compare.original_pos.as_ref()))
                .expect("UTF-8 path");
        args["output"] = serde_json::to_value(&compare.output).expect("UTF-8 path");
        args["force"] = compare.force.into();
        args["output_format"] = serde_json::to_value(
            output_format(Task::Read, compare.output.as_deref(), None)
                .ok()
                .flatten(),
        )
        .expect("format");
        return Ok(serde_json::json!({"exit_code": 0, "command": "read", "args": args}));
    }
    compare.view_given = view_given(matches);
    let args = compare_json(&compare);
    if view_is_output(&args) && !accepts("read") {
        return Err(view_not_supported(command));
    }
    Ok(serde_json::json!({"exit_code": 0, "command": "compare", "args": args}))
}

/// Parse arguments (excluding argv0) for native-compatible adapters.
///
/// `supported` contains canonical command names; empty enables every command.
/// Aliases follow their canonical command. Unsupported commands are absent
/// from help and rejected. Success contains `{exit_code, command, args,
/// notes}`: `args.quiet` is the global `-q`, and `notes` holds the
/// [`retired_note`] of each retired spelling used. `args` has
/// native snake_case field names, flattened Args groups, canonical enum strings,
/// string paths, defaults and null optional fields. Nested subcommands use
/// `{command, args}`. Compare inputs include resolved named-over-positional
/// values; compare's output path stays null (the host picks it), while the
/// author is `Jubarte` (D12) and the date now (D17), as for the binary.
/// `edit` and `add` carry where they write, resolved: `output` for the one
/// document or `out_dir` for the bundle, both null while recording into a plan. Informational/error results contain
/// `{exit_code, stream, text}`.
/// This function performs no document, filesystem, network or process I/O.
pub fn parse_json(arguments: &[String], program: &str, supported: &[String]) -> String {
    let model = top_level_command();
    let accepts = |name: &str| supported.is_empty() || supported.iter().any(|s| s == name);
    // Rebuild only the outer presentation, cloning every argument/group/task
    // from the derive model. Clap itself parses both native and facade inputs.
    let mut command = clap::Command::new(program.to_string())
        .version(env!("CARGO_PKG_VERSION"))
        .about("Read, edit, compare and render Word documents")
        .color(clap::ColorChoice::Never)
        .styles(styles())
        .term_width(88)
        .subcommand_help_heading("Tasks")
        .arg_required_else_help(true)
        .subcommands(
            model
                .get_subcommands()
                .filter(|s| accepts(s.get_name()) || s.get_name() == "__edit-v2" && accepts("edit"))
                .cloned(),
        );
    if accepts("compare") {
        command = command
            .args(model.get_arguments().cloned())
            .groups(model.get_groups().cloned())
            .args_conflicts_with_subcommands(true)
            .subcommand_negates_reqs(true);
    } else {
        // The global options (-q) still apply to every task.
        command = command
            .args(model.get_arguments().filter(|a| a.is_global_set()).cloned())
            .args_conflicts_with_subcommands(true)
            .subcommand_required(true);
    }
    let examples: Vec<String> = [
        ("compare", "compare old.docx new.docx -o redline.docx"),
        ("read", "contract.docx"),
        ("read", "read contract.docx --json"),
        ("diff", "diff old.docx new.docx --format github"),
        ("convert", "convert contract.docx -o contract.pdf"),
    ]
    .into_iter()
    .filter(|(name, _)| accepts(name))
    .map(|(_, example)| format!("  {program} {example}"))
    .collect();
    command = command.after_help(format!(
        "Examples:\n{}\n\nRun {program} <task> --help for task options.",
        examples.join("\n")
    ));
    let argv = std::iter::once(program.to_string()).chain(arguments.iter().cloned());
    let mut quiet = false;
    let result = (|| {
        // A task this host lacks is one error, whatever follows its name.
        if let Some(name) = arguments
            .iter()
            .take_while(|token| *token != "--")
            .find(|token| !token.starts_with('-'))
            .and_then(|name| model.find_subcommand(name))
            .map(|task| task.get_name().to_string())
            .filter(|name| !accepts(name))
        {
            return Err(missing_task(&mut command, program, &name));
        }
        let tokens: Vec<String> = hoist_quiet(&command, &argv.collect::<Vec<_>>());
        let tokens: Vec<String> = edit_actions::route(
            &tokens
                .iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>(),
            &command,
        )
        .into_iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
        let matches = command
            .try_get_matches_from_mut(tokens.clone())
            .map_err(|e| usage_error(&command, &tokens, e))?;
        validate_matches(&matches, &mut command)?;
        quiet = matches.get_flag("quiet");
        if matches.subcommand().is_some() {
            parse_subcommand_json(&mut command, &matches, arguments, &accepts)
        } else {
            parse_implicit_compare_json(&mut command, &matches, &model, program, &accepts)
        }
    })();
    match result {
        Ok(mut value) => {
            value["args"]["quiet"] = quiet.into();
            value["notes"] = notes(arguments).into();
            value.to_string()
        }
        Err(error) => match removed_spellings(arguments.iter().map(String::as_str)).first() {
            Some((old, now)) => {
                let removed = removed_error(old, now);
                let mut value = serde_json::json!({
                    "exit_code": 2,
                    "stream": "stderr",
                    "text": format!("{removed}\n"),
                });
                // Under --json, the line a host prints on stdout too.
                if wants_json(arguments) {
                    let message = removed.strip_prefix("error: ").unwrap_or(&removed);
                    value["json"] = removed_json_error(&error, arguments, program, message).into();
                }
                value.to_string()
            }
            None => {
                let mut value: serde_json::Value =
                    serde_json::from_str(&error_json(&error)).unwrap_or_default();
                // Help, version and usage errors carry the notes too: the
                // run that fails is the one that needs the new name. Hosts
                // print them before `text`.
                value["notes"] = notes(arguments).into();
                if wants_json(arguments)
                    && let Some(line) = usage_json_error(&error, arguments, program)
                {
                    value["json"] = line.into();
                }
                value.to_string()
            }
        },
    }
}

#[cfg(test)]
mod slash_path_tests {
    use super::{slash_path, unverbatimed};
    use std::path::Path;

    #[test]
    fn slash_path_uses_forward_slashes() {
        assert_eq!(slash_path(Path::new(r"dir\sub\name.md")), "dir/sub/name.md");
        assert_eq!(slash_path(Path::new("plain")), "plain");
    }

    /// The agent-facing contract: a canonical drive-letter path spelled
    /// `C:/…` regardless of the OS the test runs on (#1244).
    #[test]
    fn slash_path_canonical_drive_letter_spells_agent_facing() {
        let formatted = slash_path(Path::new(r"C:\Users\Alice\out.docx"));
        assert_eq!(formatted, "C:/Users/Alice/out.docx");
        assert!(!formatted.contains('\\'), "backslashes: {formatted}");
    }

    #[test]
    fn unverbatimed_keeps_every_nonverbatim_path() {
        // The parsed-component contract, portable: only a Windows verbatim
        // prefix is ever stripped, so a drive or UNC path keeps its head.
        for path in ["plain", r"dir\sub", r"C:\a\b", r"\\server\share\x"] {
            assert_eq!(unverbatimed(Path::new(path)), None, "{path}");
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn a_verbatim_literal_is_an_ordinary_name_off_windows() {
        for path in [r"\\?\C:\a\b", r"\\?\UNC\s\sh\x"] {
            assert_eq!(unverbatimed(Path::new(path)), None, "{path}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn unverbatimed_strips_every_verbatim_prefix_kind() {
        for (path, stripped) in [
            (r"\\?\C:\a\b", "C:/a/b"),
            (r"\\?\c:\a", "C:/a"),
            (r"\\?\C:", "C:"),
            (r"\\?\UNC\server\share\x", "//server/share/x"),
            (r"\\?\GLOBALROOT\Device\Foo", "GLOBALROOT/Device/Foo"),
        ] {
            assert_eq!(slash_path(Path::new(path)), stripped, "{path}");
            assert!(!slash_path(Path::new(path)).contains("//?"), "{path}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn unverbatimed_returns_the_stripped_components() {
        assert_eq!(
            unverbatimed(Path::new(r"\\?\C:\a\b")).as_deref(),
            Some(Path::new(r"C:\a\b"))
        );
        assert_eq!(
            unverbatimed(Path::new(r"\\?\UNC\server\share\x")).as_deref(),
            Some(Path::new(r"\\server\share\x"))
        );
    }
}
