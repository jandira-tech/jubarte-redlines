// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Shared, platform-neutral command declarations and parsing. No runtime I/O.

pub use crate::edit::flags::FlagOp;
use clap::{CommandFactory, FromArgMatches, Parser};
use serde::Serialize;
use std::path::{Path, PathBuf};

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
    next_help_heading = "Compare options",
    after_help = "Examples:\n  jubarte compare old.docx new.docx -o redline.docx\n  jubarte old.docx new.docx                shorthand for compare\n  jubarte inspect contract.docx --json\n  jubarte diff old.docx new.docx --format github\n  jubarte convert contract.docx -o contract.pdf\n\nRun jubarte <task> --help for task options.",
)]
pub struct Cli {
    /// Subcommand (e.g. `revisions`); plain compare when omitted.
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Inputs and settings for the shorthand document comparison.
    #[command(flatten)]
    #[serde(flatten)]
    pub compare: CompareArgs,
}

/// Options of `read`.
#[derive(clap::Args, Debug, Default, PartialEq, Serialize)]
#[group(id = "read_options", multiple = true)]
pub struct ReadArgs {
    /// Tracked changes inline (all, the default), or the text with every
    /// change accepted or rejected; the id lines then list what changed.
    #[arg(long, value_enum, value_name = "CHOICE", help_heading = "Read options")]
    pub track_changes: Option<TrackChanges>,
    /// Comments inline (default) or hidden, with their ids on the id line
    /// of the paragraph that holds them.
    #[arg(
        long,
        value_enum,
        default_value_t = CommentsArg::Inline,
        value_name = "MODE",
        help_heading = "Read options"
    )]
    pub comments: CommentsArg,
    /// Timestamps on the notes of an author whose changes do not all share
    /// one (the header shows an author's single timestamp).
    #[arg(long, help_heading = "Read options")]
    pub dates: bool,
    /// Skip the layout pass; page count from Word's cached breaks, no
    /// `<!-- page N of M -->` lines.
    #[arg(long, help_heading = "Read options")]
    pub no_page_markers: bool,
    /// Only these blocks: `p5`, `p4-p7`, `p17-`, `-p3`, `t0`, comma-separated.
    #[arg(
        short = 'p',
        long = "paragraphs",
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
        help_heading = "Read options"
    )]
    pub by: Option<String>,
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

/// Options `edit` and `add` share.
#[derive(clap::Args, Debug, Serialize)]
pub struct EditOptions {
    /// Author of the changes and comments (operation flags; a plan names
    /// its own).
    #[arg(long, value_name = "NAME", default_value = "Modified User")]
    pub author: String,
    /// Their timestamp (ISO 8601) [default: now, UTC].
    #[arg(long, value_name = "ISO8601", alias = "date")]
    pub datetime: Option<String>,
    /// The edits are tracked changes (the default): redline.docx,
    /// clean.docx, patch.diff and report.jsonl are written and the view
    /// shows the marks.
    #[arg(long, conflicts_with = "editing_mode")]
    pub suggesting_mode: bool,
    /// The edits land directly: clean.docx and report.jsonl only; the view
    /// shows the result with rev tags on the id lines.
    #[arg(long)]
    pub editing_mode: bool,
    /// What to do when FILE already holds tracked changes: auto keeps them
    /// and tracks the new edits beside them; a clean file goes through the
    /// comparer.
    #[arg(long, value_enum, value_name = "MODE", default_value_t = ExistingArg::Auto)]
    pub existing_revisions: ExistingArg,
    /// Directory to create for the outputs [default: <FILE's
    /// directory>/<stem>.edit].
    #[arg(long, value_name = "DIR")]
    pub out_dir: Option<PathBuf>,
    /// Replace an existing output directory's files.
    #[arg(long)]
    pub force: bool,
    /// Print nothing on success (the files are still written).
    #[arg(short = 'q', long)]
    pub quiet: bool,
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
    for name in ["location", "anchor", "content", "style"] {
        if let (Some(indices), Some(values)) =
            (matches.indices_of(name), matches.get_many::<String>(name))
        {
            tokens.extend(indices.zip(values).map(|(i, v)| (i, name, Some(v.clone()))));
        }
    }
    for name in ["delete", "resolve", "before", "comment"] {
        // `try_get_many` is `Err` for a flag this command does not declare
        // (`add` has no --resolve); `indices_of` would panic on it.
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
            "style" => op.styles.push(value.unwrap_or_default()),
            "delete" => op.delete = true,
            "resolve" => op.resolve = true,
            "before" => op.before = true,
            "comment" => op.comment = true,
            _ => {}
        }
    }
    Ok(ops)
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
    #[arg(value_name = "MODIFIED", required_unless_present = "modified")]
    pub modified_pos: Option<PathBuf>,

    /// Original/base document (overrides the positional ORIGINAL).
    #[arg(short = 'b', long = "original", value_name = "FILE")]
    pub original: Option<PathBuf>,

    /// Modified document (overrides the positional MODIFIED).
    #[arg(short = 'm', long = "modified", value_name = "FILE")]
    pub modified: Option<PathBuf>,

    /// Output path [default: <original-dir>/<original>_v_<modified>.docx]. A
    /// `.md` output writes the changes as CriticMarkup (both documents
    /// Markdown).
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Author name recorded on the revisions.
    #[arg(short = 'a', long, value_name = "NAME", default_value = "Redline")]
    pub author: String,

    /// Revision timestamp (ISO 8601); pinned for reproducible output.
    #[arg(
        short = 'd',
        long,
        value_name = "ISO8601",
        default_value = crate::document_comparer::DEFAULT_DATE
    )]
    pub date: String,

    /// Overwrite the output file if it already exists.
    #[arg(long)]
    pub force: bool,

    /// Do not print the success message.
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Word-match detail, from 0 to 1 [default: 0.02; powertools: 0.15].
    #[arg(long, value_name = "RATIO", value_parser = parse_threshold)]
    pub detail_threshold: Option<f64>,

    /// Compare like Microsoft Word or Open-Xml-PowerTools.
    #[arg(long, value_enum, value_name = "MODE", default_value_t = CompareMode::Word)]
    pub mode: CompareMode,

    /// Same as --mode powertools.
    #[arg(long)]
    pub powertools_faithful: bool,

    /// DEBUG: zero WmlComparerSettings::merge_replaced_paragraphs — the
    /// word-visual UMBRELLA gate — which disables the WHOLE word-visual pass
    /// family (merge, flatten, reorder, margins, …), not just the paragraph
    /// merge (pagination experiments; hidden). Redundant with
    /// --powertools-faithful, which sets the same preset.
    #[arg(long, hide = true)]
    pub no_paragraph_merge: bool,
}

impl std::ops::Deref for Cli {
    type Target = CompareArgs;
    fn deref(&self) -> &Self::Target {
        &self.compare
    }
}

/// D.6 — `redline revisions <file> [--json]`: list the tracked revisions in
/// a redline .docx (the `WmlComparer.GetRevisions` facade).
#[derive(clap::Subcommand, Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "command", content = "args")]
pub enum Command {
    /// Compare documents and write a Word redline.
    #[command(
        visible_alias = "redline",
        after_help = "Examples:\n  jubarte compare old.docx new.docx -o redline.docx\n  jubarte compare -b old.docx -m new.docx --author Legal"
    )]
    Compare(CompareArgs),
    /// List the tracked revisions in a redline .docx.
    Revisions {
        /// The redline document (.docx).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Emit the list as JSON lines instead of a human summary.
        #[arg(long)]
        json: bool,
    },
    /// List tracked changes with IDs for accept, reject and edit plans.
    Changes {
        /// The document (.docx).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Emit one JSON object per line.
        #[arg(long)]
        json: bool,
    },
    /// Accept all tracked changes, or select by ID, author or kind.
    Accept {
        /// The document (.docx) whose revisions to accept.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Output path.
        #[arg(short = 'o', long, value_name = "FILE")]
        output: PathBuf,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Revisions to accept or reject; empty selects all.
        #[command(flatten)]
        #[serde(flatten)]
        selection: Selection,
    },
    /// Reject all tracked changes, or select by ID, author or kind.
    Reject {
        /// The document (.docx) whose revisions to reject.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Output path.
        #[arg(short = 'o', long, value_name = "FILE")]
        output: PathBuf,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Revisions to accept or reject; empty selects all.
        #[command(flatten)]
        #[serde(flatten)]
        selection: Selection,
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
        /// Output path [default: <stem>.pdf next to a .docx, <stem>.docx next
        /// to Markdown; Markdown output goes to stdout]. PNG pages are named
        /// <stem>-page-NN.png beside it.
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Write the PDF (the default when neither --pdf nor --png is given).
        #[arg(long)]
        pdf: bool,
        /// Rasterize every page to PNG (<stem>-page-NN.png).
        #[arg(long)]
        png: bool,
        /// PNG resolution in dots per inch (1-1200).
        #[arg(long, default_value_t = 96.0, value_name = "DPI", value_parser = parse_dpi, help_heading = "Rendering")]
        dpi: f32,
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
        /// How tracked changes are painted: `conventional` (deletions red
        /// struck through, insertions blue underlined, moves green:
        /// double-struck where they left, double-underlined where they landed),
        /// `word` (what Microsoft Word's Save as PDF paints), or `custom`
        /// (see --revision-palette).
        #[arg(long, value_enum, default_value_t = Revisions::Conventional, requires_if("custom", "revision_palette"), help_heading = "Revision marks")]
        revisions: Revisions,
        /// Marks for --revisions custom: `kind=#RRGGBB[:lines],...` with
        /// kinds deleted, inserted, moved-from, moved-to and lines strike,
        /// double-strike, underline, double-underline, plain. Kinds left out
        /// keep their conventional mark.
        #[arg(long, value_name = "SPEC", value_parser = parse_palette, requires = "revisions", help_heading = "Revision marks")]
        revision_palette: Option<String>,
        /// Comment placement and page selection in PDF or PNG output.
        #[command(flatten)]
        #[serde(flatten)]
        page: PageOptions,
        /// Formats and Markdown reading.
        #[command(flatten)]
        #[serde(flatten)]
        markdown: MarkdownArgs,
        /// Rasterize only these pages, counted from 1: `3`, `1-3,7`. Layout
        /// still runs over the whole document. Needs PNG output.
        #[arg(long, value_name = "SPEC", value_parser = validate_pages)]
        pages: Option<String>,
        /// Exit 4 when a requested font was substituted (listed on stderr
        /// and in --report). Every output is still written. Exit status:
        /// 0 ok, 1 error, 4 a requested font was substituted.
        #[arg(long)]
        fail_on_substitution: bool,
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
        jubarte diff old.md new.md -o changes.docx       Word tracked changes\n  \
        jubarte diff old.md new.md -o changes.pdf        the changes painted in a PDF\n  \
        jubarte diff contract.docx edited.md -o redline.docx\n      \
        the Markdown's edits as tracked changes on the Word document\n\n\
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
        /// Write to FILE. Text views default to stdout and write only text.
        /// Patch/critic infer Word, Markdown, PDF or PNG from the extension.
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
        /// Output format, when --output does not say.
        #[arg(
            short = 't',
            long = "to",
            value_enum,
            value_name = "FORMAT",
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
        /// Who made the changes: the patch's owner and the revisions'
        /// author [default: `git config user.name`, else Redline].
        #[arg(short = 'a', long, value_name = "NAME", help_heading = "Word redline")]
        author: Option<String>,
        /// When (ISO 8601) [default: now]; pin it for reproducible output.
        #[arg(
            short = 'd',
            long,
            value_name = "ISO8601",
            help_heading = "Word redline"
        )]
        date: Option<String>,
        /// Whose redline to reproduce (see `jubarte --help`).
        #[arg(long, value_enum, value_name = "MODE", default_value_t = CompareMode::Word, help_heading = "Word redline")]
        mode: CompareMode,
        /// LCS detail threshold (see `jubarte --help`).
        #[arg(long, value_name = "RATIO", value_parser = parse_threshold, help_heading = "Word redline")]
        detail_threshold: Option<f64>,
        /// Two Markdown documents written as Word take styles, page setup,
        /// headers and footers from this .docx.
        #[arg(long, value_name = "FILE", help_heading = "Word redline")]
        reference_doc: Option<PathBuf>,
        /// Read CriticMarkup in the Markdown documents as tracked changes
        /// (Word output). By default a document compared is text.
        #[arg(long, help_heading = "Word redline")]
        critic: bool,
        /// Where images named by the Markdown are found [default: each
        /// Markdown file's directory].
        #[arg(long, value_name = "DIR", help_heading = "Word redline")]
        resource_path: Option<PathBuf>,
        /// How tracked changes are painted in PDF or PNG output (see
        /// `convert --help`).
        #[arg(long, value_enum, default_value_t = Revisions::Conventional, requires_if("custom", "revision_palette"), help_heading = "Revision marks")]
        revisions: Revisions,
        /// Marks for --revisions custom (see `convert --help`).
        #[arg(long, value_name = "SPEC", value_parser = parse_palette, requires = "revisions", help_heading = "Revision marks")]
        revision_palette: Option<String>,
        /// Comment placement and page selection in PDF or PNG output.
        #[command(flatten)]
        #[serde(flatten)]
        page: PageOptions,
    },
    /// Inspect document facts, paragraphs, styles and tables.
    Inspect {
        /// The document (.docx) to read.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Emit the snapshot as JSON (`schema_version`, `source_sha256`,
        /// `summary`, `paragraphs`, `stories`, `tables`) instead of a human
        /// summary.
        #[arg(long)]
        json: bool,
        /// Print each body table as a grid instead of the paragraphs: a
        /// `table N: ROWSxCOLS header_rows=H widths=W,...` line, then one
        /// line per row of tab-separated `ids=text` cells.
        #[arg(long, conflicts_with = "json")]
        tables: bool,
    },
    /// Read the agent view: YAML header, `<!-- pN -->` id lines, changes and comments with ids
    ///
    /// A YAML header, then Markdown with an `<!-- pN -->` id line before every
    /// paragraph (`pN` is `body:p:N`), tracked changes as CriticMarkup
    /// followed by their ids (`{++text++}{>>#12 @AC<<}`) and comments with
    /// theirs (`{>>#c5 @AC: …<<}`).
    #[command(visible_alias = "text")]
    Read {
        /// The document (.docx) to read.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// The view's options.
        #[command(flatten)]
        #[serde(flatten)]
        args: ReadArgs,
    },
    /// Edit a document: -p WHERE with --anchor, --content, --delete,
    /// --resolve or --style (several -p per command), or a JSON plan. Writes
    /// clean copy, redline, patch and report, then prints the changed
    /// paragraphs as the agent view (refusal: exit 3).
    #[command(after_help = "Examples:\n  \
        jubarte edit a.docx -p p12 --anchor \"thirty days\" --content \"forty-five days\"\n  \
        jubarte edit a.docx -p p12 --anchor thirty --content \"thirty (30)\"   an insertion\n  \
        jubarte edit a.docx -p p7 --anchor \"at its sole discretion\" --delete -p p9 --delete\n  \
        jubarte edit a.docx -p p3 --content \"The parties agree as follows.\"    rewrite\n  \
        jubarte edit a.docx -p p4 --anchor Fees --style bold -p p5 --style Heading2\n  \
        jubarte edit a.docx -p c5 --content \"Agreed.\" -p c7 --resolve\n  \
        jubarte edit a.docx --plan plan.json --out-dir review\n\n\
        WHERE is an id from `jubarte read`: p12, header1, footer2.p1, t0.r1.c2,\n\
        c5 (a comment), or a long id (body:p:12).")]
    Edit {
        /// The source document (.docx). Never modified.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Edit plan JSON: batches and the other operation kinds (see
        /// `jubarte capabilities --json`). Excludes the operation flags.
        #[arg(
            long,
            value_name = "PLAN.json",
            required_unless_present = "location",
            conflicts_with_all = ["location", "anchor", "content", "delete", "resolve", "style", "author", "datetime", "existing_revisions"]
        )]
        plan: Option<PathBuf>,
        /// Where: p12, header1, footer2.p1, t0.r1.c2, or c5 for a comment.
        /// Each -p starts an operation; the flags after it belong to it.
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
        /// Formatting for the anchored text: bold, italic, underline, strike,
        /// caps, highlight=yellow, font=Calibri, size=11, color=FF0000; any
        /// other value is a paragraph style (Heading2).
        #[arg(long, value_name = "SPEC", action = clap::ArgAction::Append)]
        style: Vec<String>,
        /// Author, date, mode and output options.
        #[command(flatten)]
        #[serde(flatten)]
        options: EditOptions,
        /// Resolve and report only; write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Also write redline.pdf and clean.pdf.
        #[arg(long)]
        pdf: bool,
        /// Also write redline-page-NN.png and clean-page-NN.png.
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
    /// (several -p per command). Writes the same files as edit and prints
    /// the changed paragraphs as the agent view.
    #[command(after_help = "Examples:\n  \
        jubarte add a.docx -p p12 --content \"Time is of the essence.\"   new paragraph after p12\n  \
        jubarte add a.docx -p p12 --content Recitals --before --style Heading2\n  \
        jubarte add a.docx -p p5 --anchor \"monthly fee\" --content \"Net of taxes?\"   a comment\n  \
        jubarte add a.docx -p p5 --comment --content \"Whole clause needs a cap.\"\n  \
        jubarte add a.docx -p c5 --content \"Agreed, will fix.\"           a reply")]
    Add {
        /// The source document (.docx). Never modified.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Where: p12, header1, footer2.p1, t0.r1.c2, or c5 for a reply.
        /// Each -p starts an operation; the flags after it belong to it.
        #[arg(short = 'p', long = "location", value_name = "WHERE", action = clap::ArgAction::Append, required = true)]
        location: Vec<String>,
        /// Text inside WHERE to comment on (must occur once).
        #[arg(long, value_name = "TEXT", action = clap::ArgAction::Append)]
        anchor: Vec<String>,
        /// The paragraph, comment or reply text.
        #[arg(long, value_name = "TEXT", action = clap::ArgAction::Append)]
        content: Vec<String>,
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
        /// Author, date, mode and output options.
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
        /// Emit JSON (the default output is JSON too; the flag documents intent).
        #[arg(long)]
        json: bool,
    },
    /// Check or install a release from GitHub.
    #[command(after_help = "Examples:\n  \
        jubarte self-update --check          installed and latest versions\n  \
        jubarte self-update                  ask, then install the latest release\n  \
        jubarte self-update --yes            install without asking\n  \
        jubarte self-update --version 0.9.3  install that release (also older)")]
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
    #[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
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
    },
    /// Compare rendered pages pixel by pixel (different pages: exit 5).
    #[command(after_help = "Examples:\n  \
        jubarte diff-render before.docx after.docx                  changed pages on stdout\n  \
        jubarte diff-render before.docx after.docx --out-dir diff   PNGs of the changed pages and diff.json\n  \
        jubarte diff-render a.docx b.docx --json                    the diff.json document on stdout\n\n\
        With --out-dir, each page that differs is written as a-page-NN.png,\n\
        b-page-NN.png and diff-page-NN.png (b's page with the changed pixels\n\
        magenta and boxed); diff.json lists every page with its changed_ratio,\n\
        bbox and, for a page only one side has, only_in.")]
    DiffRender {
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
        /// Output path.
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
        /// Print nothing on success.
        #[arg(short, long)]
        quiet: bool,
    },
    /// Check or repair Word validity (findings: exit 2; unreadable: exit 1).
    #[command(after_help = "Examples:\n  \
        jubarte validate contract.docx\n  \
        jubarte validate contract.docx --json\n  \
        jubarte validate contract.docx --repair fixed.docx\n  \
        jubarte validate review/redline.docx --original contract.docx --author Claude")]
    Validate {
        /// The document (.docx).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// JSON Lines: one object per finding, nothing when there is none.
        #[arg(long)]
        json: bool,
        /// Write the repaired package here; remaining findings still exit 2.
        #[arg(long, value_name = "FILE")]
        repair: Option<PathBuf>,
        /// Audit tracked edits: every text change against ORIGINAL must be a
        /// revision by --author.
        #[arg(long, value_name = "FILE", requires = "author")]
        original: Option<PathBuf>,
        /// The author every change must carry (with --original).
        #[arg(long, value_name = "NAME", requires = "original")]
        author: Option<String>,
        /// Replace an existing --repair output.
        #[arg(long)]
        force: bool,
    },
    /// Field results written back into the document from jubarte's layout.
    Fields {
        /// Field operation to execute.
        #[command(subcommand)]
        sub: FieldsCommand,
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
        /// Output path.
        #[arg(short = 'o', long, value_name = "FILE")]
        output: PathBuf,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Metadata and comment scrub selection.
        #[command(flatten)]
        #[serde(flatten)]
        scrub: ScrubSelection,
    },
    /// Audit accessibility, style and structure (findings: exit 2).
    #[command(after_help = "Rules (code, set, severity):\n  \
        HEADING_SKIP a11y warning, IMAGE_NO_DESCR a11y error,\n  \
        TABLE_NO_HEADER_ROW a11y warning, MISSING_LANG a11y warning,\n  \
        LITERAL_BULLET style warning, EMPTY_SPACER_PARAGRAPH style info,\n  \
        DIRECT_FORMATTING_OVERRIDES_STYLE style info,\n  \
        STALE_FIELD_CACHE structure warning, FONT_SUBSTITUTED structure info")]
    Audit {
        /// The document (.docx) to audit.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Emit `{findings, rules, layout}` as JSON.
        #[arg(long)]
        json: bool,
        /// Rule sets (a11y, style, structure) or rule codes, comma-separated
        /// [default: every rule].
        #[arg(long, value_name = "RULES", value_delimiter = ',')]
        rules: Vec<String>,
        /// Fail (exit 2) on warnings too, not only on errors.
        #[arg(long)]
        strict: bool,
    },
}

/// `jubarte fields` subcommands.
#[derive(clap::Subcommand, Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "command", content = "args")]
pub enum FieldsCommand {
    /// Refresh the cached results of PAGEREF, REF, NUMPAGES, SEQ and TOC
    /// fields from jubarte's layout; TOCs are rebuilt from the headings.
    /// Field codes stay, so Word can update them again. Page numbers are
    /// jubarte's layout, not Word's (docs/WORD_DIFFERENCES.md).
    #[command(after_help = "Examples:\n  \
        jubarte fields update in.docx -o out.docx          one line per field written\n  \
        jubarte fields update in.docx -o out.docx --json   {\"page_count\", \"fields\": [...]}")]
    Update {
        /// The document (.docx).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Output path.
        #[arg(short = 'o', long, value_name = "FILE")]
        output: PathBuf,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Print the fields written as JSON.
        #[arg(long)]
        json: bool,
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
}

impl Format {
    /// The format a file name says, by extension.
    pub fn of_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "docx" | "docm" | "dotx" | "dotm" => Some(Self::Docx),
            "md" | "markdown" | "mdown" | "mkd" | "mkdn" | "txt" => Some(Self::Md),
            "pdf" => Some(Self::Pdf),
            "png" => Some(Self::Png),
            _ => None,
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

/// `--track-changes`, pandoc's values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrackChanges {
    /// Keep them: CriticMarkup becomes Word tracked changes and comments.
    All,
    /// Accept every change.
    Accept,
    /// Reject every change.
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

/// Where a PDF's comments go and which pages it keeps (`convert`, `diff`).
#[derive(clap::Args, Clone, Copy, Debug, Default, Serialize)]
pub struct PageOptions {
    /// List the comments after the last page instead of in balloons beside
    /// the text. The commented text keeps its tint and a `[JR1]` marker,
    /// and the pages keep their own width.
    #[arg(long)]
    pub move_comments: bool,
    /// Keep only the pages a tracked change touches. The whole document is
    /// laid out first, so page numbers stay the document's; a document
    /// without changes keeps its first page. --pages counts the kept pages.
    #[arg(long)]
    pub changed_only: bool,
}

impl PageOptions {
    /// Apply comment placement and page filtering to the renderer options.
    pub fn apply(self, options: crate::convert::PdfOptions) -> crate::convert::PdfOptions {
        crate::convert::PdfOptions {
            comments: if self.move_comments {
                crate::convert::CommentPlacement::End
            } else {
                crate::convert::CommentPlacement::Margin
            },
            changed_only: self.changed_only,
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
        value_enum,
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
    /// Markdown: read `{++`, `{--` and the other CriticMarkup delimiters as
    /// text.
    #[arg(long)]
    pub no_critic: bool,
    /// Markdown to Word: take styles, numbering, page setup, headers and
    /// footers from this .docx (pandoc's --reference-doc).
    #[arg(long, value_name = "FILE")]
    pub reference_doc: Option<PathBuf>,
    /// Markdown to Word: where images are found [default: the Markdown
    /// file's directory].
    #[arg(long, value_name = "DIR")]
    pub resource_path: Option<PathBuf>,
    /// Markdown to Word: author of the tracked changes and comments.
    #[arg(short = 'a', long, value_name = "NAME", default_value = "Redline")]
    pub author: String,
    /// Markdown to Word: their date (ISO 8601); pinned for reproducible
    /// output.
    #[arg(
        short = 'd',
        long,
        value_name = "ISO8601",
        default_value = crate::document_comparer::DEFAULT_DATE
    )]
    pub date: String,
    /// Markdown to Word: the page size when there is no --reference-doc
    /// (one-inch margins either way); a reference's page setup wins.
    #[arg(long, value_enum, value_name = "SIZE", default_value_t = Page::Letter)]
    pub page: Page,
    /// Word to Markdown: leave out the `<!-- page N of M -->` lines, and the
    /// layout pass that places them.
    #[arg(long)]
    pub no_page_markers: bool,
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
    /// Only this change (`body:rev:12`, as `jubarte changes` lists it).
    /// Repeatable.
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
fn validate_matches(
    matches: &clap::ArgMatches,
    command: &mut clap::Command,
) -> Result<(), clap::Error> {
    use clap::{error::ErrorKind, parser::ValueSource};
    let Some((name, args)) = matches.subcommand() else {
        return Ok(());
    };
    let mut task = command.find_subcommand(name).expect("parsed task").clone();
    task.set_bin_name(format!("{} {name}", command.get_name()));
    let error =
        |task: &mut clap::Command, message: &str| task.error(ErrorKind::ArgumentConflict, message);
    if matches!(name, "convert" | "edit" | "diff")
        && args.get_one::<String>("revision_palette").is_some()
        && args.get_one::<Revisions>("revisions") != Some(&Revisions::Custom)
    {
        return Err(error(
            &mut task,
            "--revision-palette needs --revisions custom",
        ));
    }
    if name == "diff" {
        let output_format = args.get_one::<Format>("to").copied().or_else(|| {
            args.get_one::<PathBuf>("output")
                .and_then(|path| Format::of_path(path))
        });
        if !matches!(output_format, Some(Format::Pdf | Format::Png)) {
            for flag in [
                "move_comments",
                "changed_only",
                "revisions",
                "revision_palette",
            ] {
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
        let format = *args
            .get_one::<PatchFormat>("format")
            .expect("default format");
        let github = format.is_text_view();
        if !github {
            for flag in ["context", "accept_changes", "full_lines"] {
                if args.value_source(flag) == Some(ValueSource::CommandLine) {
                    return Err(error(
                        &mut task,
                        &format!(
                            "--{} requires a text diff format (github, word, normal, context or side-by-side)",
                            flag.replace('_', "-")
                        ),
                    ));
                }
            }
        }
        if github {
            if args
                .get_one::<Format>("to")
                .is_some_and(|f| *f != Format::Md)
            {
                return Err(error(
                    &mut task,
                    "this diff format writes text; --to must be md (markdown)",
                ));
            }
            if args.get_one::<PathBuf>("output").is_some_and(|path| {
                !path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                    matches!(
                        e.to_ascii_lowercase().as_str(),
                        "patch" | "diff" | "txt" | "md" | "markdown"
                    )
                })
            }) {
                return Err(error(
                    &mut task,
                    "text diff output must end in .patch, .diff, .txt or .md",
                ));
            }
            for flag in [
                "columns",
                "move_comments",
                "changed_only",
                "revisions",
                "revision_palette",
                "reference_doc",
                "resource_path",
                "critic",
                "mode",
                "detail_threshold",
                "author",
                "date",
            ] {
                if args.value_source(flag) == Some(ValueSource::CommandLine) {
                    return Err(error(
                        &mut task,
                        &format!(
                            "--{} does not apply to this text diff format",
                            flag.replace('_', "-")
                        ),
                    ));
                }
            }
        }
    }
    if name == "convert" {
        let to = args.get_one::<Format>("to").copied();
        let output = args
            .get_one::<PathBuf>("output")
            .and_then(|p| Format::of_path(p));
        let png = args.get_flag("png") || to.or(output) == Some(Format::Png);
        if args.get_one::<String>("pages").is_some() && !png {
            return Err(error(
                &mut task,
                "--pages selects PNG pages; add --png or --to png",
            ));
        }
        if matches!(to.or(output), Some(Format::Docx | Format::Md)) {
            for flag in [
                "pdf",
                "png",
                "move_comments",
                "changed_only",
                "compress",
                "report",
                "font_report",
                "fail_on_substitution",
                "pages",
                "dpi",
                "revisions",
                "revision_palette",
            ] {
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

impl Cli {
    /// Parse native arguments with clap and validate cross-option constraints.
    pub fn try_parse_from<I, T>(arguments: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let mut command = Self::command();
        let matches = command.try_get_matches_from_mut(arguments)?;
        validate_matches(&matches, &mut command)?;
        let mut cli = Self::from_arg_matches(&matches)?;
        if let Some(task) = cli.command.as_mut() {
            fill_flag_operations(task, &matches, &mut command)?;
        }
        Ok(cli)
    }
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
    args["output_format"] =
        serde_json::to_value(compare.output.as_deref().and_then(Format::of_path)).expect("format");
    args
}

fn error_json(error: &clap::Error) -> String {
    serde_json::json!({
        "exit_code": error.exit_code(),
        "stream": if error.use_stderr() { "stderr" } else { "stdout" },
        "text": error.to_string(),
    })
    .to_string()
}

/// Parse arguments (excluding argv0) for native-compatible adapters.
///
/// `supported` contains canonical command names; empty enables every command.
/// Aliases follow their canonical command. Unsupported commands are absent
/// from help and rejected. Success contains `{exit_code, command, args}` with
/// native snake_case field names, flattened Args groups, canonical enum strings,
/// string paths, defaults and null optional fields. Nested subcommands use
/// `{command, args}`. Compare inputs include resolved named-over-positional
/// values; runtime-dependent defaults (output path, git author, current date)
/// remain null. Informational/error results contain `{exit_code, stream, text}`.
/// This function performs no document, filesystem, network or process I/O.
pub fn parse_json(arguments: &[String], program: &str, supported: &[String]) -> String {
    let model = Cli::command();
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
                .filter(|s| accepts(s.get_name()))
                .cloned(),
        );
    if accepts("compare") {
        command = command
            .args(model.get_arguments().cloned())
            .groups(model.get_groups().cloned())
            .args_conflicts_with_subcommands(true)
            .subcommand_negates_reqs(true);
    } else {
        command = command.subcommand_required(true);
    }
    let examples: Vec<String> = [
        ("compare", "compare old.docx new.docx -o redline.docx"),
        ("inspect", "inspect contract.docx --json"),
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
    let result = (|| {
        let matches = command.try_get_matches_from_mut(argv)?;
        validate_matches(&matches, &mut command)?;
        if matches.subcommand().is_some() {
            let mut task = Command::from_arg_matches(&matches)?;
            fill_flag_operations(&mut task, &matches, &mut command)?;
            let mut value = serde_json::to_value(&task).expect("UTF-8 CLI arguments");
            if let Command::Compare(compare) = &task {
                value["args"] = compare_json(compare);
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
                let input_format = |path: &Path| {
                    from.or_else(|| {
                        Format::of_path(path).filter(|f| matches!(f, Format::Docx | Format::Md))
                    })
                };
                value["args"]["old_format"] =
                    serde_json::to_value(input_format(old)).expect("format");
                value["args"]["new_format"] =
                    serde_json::to_value(input_format(new)).expect("format");
                value["args"]["output_format"] = serde_json::to_value(
                    to.or_else(|| output.as_deref().and_then(Format::of_path)),
                )
                .expect("format");
            }
            value["exit_code"] = 0.into();
            Ok(value.to_string())
        } else {
            let compare = CompareArgs::from_arg_matches(&matches)?;
            if compare
                .original_pos
                .as_ref()
                .and_then(|p| p.to_str())
                .is_some_and(|name| {
                    model
                        .find_subcommand(name)
                        .is_some_and(|task| !accepts(task.get_name()))
                })
            {
                return Err(command.error(
                    clap::error::ErrorKind::InvalidSubcommand,
                    "this task is not supported by this adapter",
                ));
            }
            Ok(serde_json::json!({"exit_code": 0, "command": "compare", "args": compare_json(&compare)}).to_string())
        }
    })();
    result.unwrap_or_else(|error| error_json(&error))
}
