// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte` — generate a tracked-changes (redline) `.docx` from two documents.
//!
//! ```text
//! jubarte original.docx modified.docx
//!   → writes original_v_modified.docx
//! jubarte -b a.docx -m b.docx -o out.docx --author "Jane" --date 2024-01-02T00:00:00Z
//! jubarte diff old.md new.md                 (CriticMarkup on stdout)
//! jubarte convert draft.md -o draft.docx     (CriticMarkup as tracked changes)
//! ```
//!
//! Either document may be Markdown (`.md`, `.markdown`); see `jubarte diff`
//! and `jubarte convert --help`.
//!
//! Positional args are `<ORIGINAL> <MODIFIED>`; the `--original`/`--modified`
//! flags override them. Argument parsing, `--help`, `--version`, short/long
//! flags, and validation are handled by clap (gated behind the default `cli`
//! feature).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

/// CLI-only global allocator. The redline pipeline spends ~41% of CPU self-time
/// in allocation/copy/free/drop of xmllinq nodes (measured with samply on the
/// RFP17 fixtures — `produce::coalesce_recurse`/`reconstruct_element` churn).
/// mimalloc lowers that per-allocation cost; it changes performance only, never
/// program semantics. Library consumers are unaffected (this lives in the
/// binary). Toggle off with `--no-default-features --features cli` for A/B.
#[cfg(feature = "fast-alloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Generate a tracked-changes (redline) .docx from two documents.
///
/// The redline is the ORIGINAL document with every difference against MODIFIED
/// expressed as Word tracked changes (insertions, deletions, moves, and format
/// changes), so it opens cleanly in Microsoft Word.
#[derive(Parser, Debug)]
#[command(
    name = "jubarte",
    version,
    about = "Generate a tracked-changes (redline) .docx from two documents",
    long_about = None,
    after_help = "EXAMPLES:\n  \
        jubarte contract.docx contract-rev2.docx\n      \
        → writes contract_v_contract-rev2.docx next to the original\n\n  \
        jubarte -b old.docx -m new.docx -o redline.docx --author \"Legal\"\n  \
        jubarte a.docx b.docx --force --quiet\n  \
        jubarte contract.docx edited.md          the Markdown's edits as a Word redline\n  \
        jubarte old.md new.md -o changes.md      the changes as CriticMarkup",
)]
struct Cli {
    /// Subcommand (e.g. `revisions`); plain compare when omitted.
    #[command(subcommand)]
    command: Option<Command>,

    /// The original / base document (.docx or Markdown).
    #[arg(value_name = "ORIGINAL")]
    original_pos: Option<PathBuf>,

    /// The modified document (.docx or Markdown).
    #[arg(value_name = "MODIFIED")]
    modified_pos: Option<PathBuf>,

    /// Original/base document (overrides the positional ORIGINAL).
    #[arg(short = 'b', long = "original", value_name = "FILE")]
    original: Option<PathBuf>,

    /// Modified document (overrides the positional MODIFIED).
    #[arg(short = 'm', long = "modified", value_name = "FILE")]
    modified: Option<PathBuf>,

    /// Output path [default: <original-dir>/<original>_v_<modified>.docx]. A
    /// `.md` output writes the changes as CriticMarkup (both documents
    /// Markdown).
    #[arg(short = 'o', long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Author name recorded on the revisions.
    #[arg(short = 'a', long, value_name = "NAME", default_value = "Redline")]
    author: String,

    /// Revision timestamp (ISO 8601); pinned for reproducible output.
    #[arg(
        short = 'd',
        long,
        value_name = "ISO8601",
        default_value = "1970-01-01T00:00:00Z"
    )]
    date: String,

    /// Overwrite the output file if it already exists.
    #[arg(long)]
    force: bool,

    /// Do not print the success message.
    #[arg(short = 'q', long)]
    quiet: bool,

    /// LCS detail threshold [default: 0.02, or 0.15 under
    /// --mode powertools]. 0.02 = Word-style within-paragraph word diffs
    /// with weak-match voiding; 0.15 = the PowerTools-faithful coarse
    /// fallback; 0 = confetti with no voiding. An explicit value always wins
    /// over either preset (Option distinguishes unset from explicitly-set —
    /// no sentinel ambiguity).
    #[arg(long, value_name = "RATIO")]
    detail_threshold: Option<f64>,

    /// Whose redline to reproduce: `word` lays changes out as Microsoft Word
    /// Compare does; `powertools` is the Open-Xml-PowerTools coarse fallback.
    /// docs/WORD_DIFFERENCES.md lists where the two, and Word, differ.
    #[arg(long, value_enum, value_name = "MODE", default_value_t = CompareMode::Word)]
    mode: CompareMode,

    /// Same as --mode powertools.
    #[arg(long)]
    powertools_faithful: bool,

    /// DEBUG: zero WmlComparerSettings::merge_replaced_paragraphs — the
    /// word-visual UMBRELLA gate — which disables the WHOLE word-visual pass
    /// family (merge, flatten, reorder, margins, …), not just the paragraph
    /// merge (pagination experiments; hidden). Redundant with
    /// --powertools-faithful, which sets the same preset.
    #[arg(long, hide = true)]
    no_paragraph_merge: bool,
}

/// D.6 — `redline revisions <file> [--json]`: list the tracked revisions in
/// a redline .docx (the `WmlComparer.GetRevisions` facade).
#[derive(clap::Subcommand, Debug)]
enum Command {
    /// List the tracked revisions in a redline .docx.
    Revisions {
        /// The redline document (.docx).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Emit the list as JSON lines instead of a human summary.
        #[arg(long)]
        json: bool,
    },
    /// List each tracked change with the id `accept --id`, `reject --id` and
    /// edit plans take.
    Changes {
        /// The document (.docx).
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Emit one JSON object per line.
        #[arg(long)]
        json: bool,
    },
    /// Accept tracked changes (package-wide) and write the result: every
    /// change, or those --id/--author/--kind select (the rest stay tracked).
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
        #[command(flatten)]
        selection: Selection,
    },
    /// Reject tracked changes (package-wide) and write the result: every
    /// change, or those --id/--author/--kind select (the rest stay tracked).
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
        #[command(flatten)]
        selection: Selection,
    },
    /// Convert a .docx to PDF and/or PNG pages (independent of LibreOffice),
    /// or Markdown to .docx, PDF or PNG, with CriticMarkup as tracked changes.
    #[command(after_help = "EXAMPLES:\n  \
        jubarte convert contract.docx                   PDF, Word-style layout\n  \
        jubarte convert draft.md                        draft.docx, CriticMarkup as tracked changes\n  \
        jubarte convert draft.md -o draft.pdf           the changes painted in a PDF\n  \
        jubarte convert draft.md --reference-doc house.docx -o draft.docx\n  \
        jubarte convert draft.md -t md --track-changes accept   the text with every change accepted\n  \
        jubarte convert notes.md --no-critic            {++ and the other delimiters as text")]
    Convert {
        /// The document to convert: .docx, or Markdown (.md, .markdown).
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
        #[arg(long, default_value_t = 96.0, value_name = "DPI")]
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
        /// bold, italic, synthetic}, …]`) for this document (plan Step 2f).
        #[arg(long, value_name = "FILE")]
        font_report: Option<PathBuf>,
        /// How tracked changes are painted: `conventional` (deletions red
        /// struck through, insertions blue double-underlined, moves green),
        /// `word` (what Microsoft Word's Save as PDF paints), or `custom`
        /// (see --revision-palette).
        #[arg(long, value_enum, default_value_t = Revisions::Conventional)]
        revisions: Revisions,
        /// Marks for --revisions custom: `kind=#RRGGBB[:lines],...` with
        /// kinds deleted, inserted, moved-from, moved-to and lines strike,
        /// double-strike, underline, double-underline, plain. Kinds left out
        /// keep their conventional mark.
        #[arg(long, value_name = "SPEC")]
        revision_palette: Option<String>,
        /// Formats and Markdown reading.
        #[command(flatten)]
        markdown: MarkdownArgs,
        /// Rasterize only these pages, counted from 1: `3`, `1-3,7`. Layout
        /// still runs over the whole document. Needs PNG output.
        #[arg(long, value_name = "SPEC")]
        pages: Option<String>,
    },
    /// Compare two documents, Word or Markdown: the changed paragraphs as a
    /// patch on stdout, each change `[-old-]{+new+}` in its paragraph, and
    /// with --output a Word redline (.docx), CriticMarkup (.md) or a PDF
    /// with the changes painted.
    #[command(after_help = "EXAMPLES:\n  \
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
        /// Output path; its extension picks the format (.md, .docx, .pdf,
        /// .png) [default: none for two Markdown documents, else
        /// <old>_v_<new>.docx next to OLD]. The patch is printed either way.
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// What goes to stdout: `patch` (the changed paragraphs, with their
        /// ids) or `critic` (the whole document as CriticMarkup, as pandiff).
        #[arg(long, value_enum, value_name = "FORMAT", default_value_t = PatchFormat::Patch)]
        format: PatchFormat,
        /// Wrap the patch's lines at this many columns; 0 does not wrap.
        #[arg(long, value_name = "N", default_value_t = jubarte::markdown::DEFAULT_COLUMNS)]
        columns: usize,
        /// Output format, when --output does not say.
        #[arg(short = 't', long = "to", value_enum, value_name = "FORMAT")]
        to: Option<Format>,
        /// Input format of both documents [default: from each file].
        #[arg(short = 'f', long = "from", value_enum, value_name = "FORMAT")]
        from: Option<Format>,
        /// Overwrite the output file if it already exists.
        #[arg(long)]
        force: bool,
        /// Who made the changes: the patch's owner and the revisions'
        /// author [default: `git config user.name`, else Redline].
        #[arg(short = 'a', long, value_name = "NAME")]
        author: Option<String>,
        /// When (ISO 8601) [default: now]; pin it for reproducible output.
        #[arg(short = 'd', long, value_name = "ISO8601")]
        date: Option<String>,
        /// Whose redline to reproduce (see `jubarte --help`).
        #[arg(long, value_enum, value_name = "MODE", default_value_t = CompareMode::Word)]
        mode: CompareMode,
        /// LCS detail threshold (see `jubarte --help`).
        #[arg(long, value_name = "RATIO")]
        detail_threshold: Option<f64>,
        /// Two Markdown documents written as Word take styles, page setup,
        /// headers and footers from this .docx.
        #[arg(long, value_name = "FILE")]
        reference_doc: Option<PathBuf>,
        /// Read CriticMarkup in the Markdown documents as tracked changes
        /// (Word output). By default a document compared is text.
        #[arg(long)]
        critic: bool,
        /// Where images named by the Markdown are found [default: each
        /// Markdown file's directory].
        #[arg(long, value_name = "DIR")]
        resource_path: Option<PathBuf>,
        /// How tracked changes are painted in PDF or PNG output (see
        /// `convert --help`).
        #[arg(long, value_enum, default_value_t = Revisions::Conventional)]
        revisions: Revisions,
        /// Marks for --revisions custom (see `convert --help`).
        #[arg(long, value_name = "SPEC")]
        revision_palette: Option<String>,
    },
    /// Read a .docx: body paragraphs with ids, style, formatting spans and
    /// limitations, plus package facts.
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
    /// Print the body as Markdown with a `[body:p:N]` id before every
    /// paragraph: the coordinates an edit plan uses.
    Text {
        /// The document (.docx) to read.
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Apply an edit plan: write the clean copy, the Word redline and a
    /// per-operation report (optionally PDF and PNG pages) into a new
    /// directory. A refused plan writes nothing and exits 3.
    Edit {
        /// The source document (.docx). Never modified.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Edit plan JSON (see `jubarte capabilities --json` for the kinds).
        #[arg(long, value_name = "PLAN.json")]
        plan: PathBuf,
        /// Directory to create for clean.docx, redline.docx, report.jsonl.
        #[arg(long, value_name = "DIR")]
        out_dir: PathBuf,
        /// Resolve and report only; write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Replace an existing output directory's files.
        #[arg(long)]
        force: bool,
        /// Also write redline.pdf and clean.pdf.
        #[arg(long)]
        pdf: bool,
        /// Also write redline-page-NN.png and clean-page-NN.png.
        #[arg(long)]
        png: bool,
        /// PNG resolution in dots per inch (1-1200).
        #[arg(long, default_value_t = 96.0, value_name = "DPI")]
        dpi: f32,
        /// How tracked changes are painted in the redline PDF/PNG.
        #[arg(long, value_enum, default_value_t = Revisions::Conventional)]
        revisions: Revisions,
        /// Marks for --revisions custom (see `convert --help`).
        #[arg(long, value_name = "SPEC")]
        revision_palette: Option<String>,
        /// Print nothing on success (patch.diff and report.jsonl are still
        /// written).
        #[arg(short = 'q', long)]
        quiet: bool,
    },
    /// What this binary can do, for agents choosing an operation.
    Capabilities {
        /// Emit JSON (the default output is JSON too; the flag documents intent).
        #[arg(long)]
        json: bool,
    },
    /// Install the latest jubarte release from GitHub. Contacts GitHub only
    /// when run; nothing checks for updates otherwise.
    #[command(after_help = "EXAMPLES:\n  \
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
    /// Triage a .docx Word refuses, or compare two builds of one. Short
    /// output: counts by kind, a few examples each; with two files, only
    /// what differs.
    #[command(after_help = "EXAMPLES:\n  \
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
    /// Which pages of two .docx files look different: both are laid out and
    /// rasterized at one resolution and compared pixel for pixel. Exits 0
    /// when every page is the same, 5 when any page differs.
    #[command(after_help = "EXAMPLES:\n  \
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
        #[arg(long, default_value_t = 100.0, value_name = "DPI")]
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
    /// List every comment with its thread (`parent`, `done`) and the text
    /// it is anchored to, with its surroundings.
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
}

/// `jubarte debug` subcommands.
#[derive(clap::Subcommand, Debug)]
enum DebugCommand {
    /// What differs between two or more packages, element by element:
    /// styles paired by type and name, paragraphs by their text, headers
    /// and footers by section role. Each hunk prints the lines not every
    /// file holds; with three or more files each line names the files that
    /// hold it. rsids, paragraph ids, revision ids/authors/dates,
    /// relationship ids (shown as what they point to), docProps save
    /// stamps, attribute order, on/off values and empty property blocks
    /// are dropped unless --raw.
    #[command(after_help = "EXAMPLES:\n  \
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum DebugCheck {
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

impl From<DebugCheck> for jubarte::debug::Check {
    fn from(c: DebugCheck) -> Self {
        use jubarte::debug::Check;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum PatchFormat {
    /// The changed paragraphs, as `git diff --word-diff` with CriticMarkup
    /// comments and highlights.
    Patch,
    /// The whole document as CriticMarkup, as pandiff prints it.
    Critic,
}

/// Who a patch's changes are by when `--author` does not say: git's
/// `user.name`, else Redline.
fn default_author() -> String {
    std::process::Command::new("git")
        .args(["config", "user.name"])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Redline".to_string())
}

/// `jubarte --mode` (compare).
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum CompareMode {
    /// Microsoft Word Compare's layout: word-level detail, replaced paragraphs
    /// merged, Word's alignment passes.
    Word,
    /// Open-Xml-PowerTools: coarse paragraph fallback (threshold 0.15), no
    /// Word alignment passes.
    Powertools,
}

/// A document format for `-f/--from` and `-t/--to`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum Format {
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
    fn of_path(path: &Path) -> Option<Self> {
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
    fn of_input(asked: Option<Self>, path: &Path, bytes: &[u8]) -> Self {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum TrackChanges {
    /// Keep them: CriticMarkup becomes Word tracked changes and comments.
    All,
    /// Accept every change.
    Accept,
    /// Reject every change.
    Reject,
}

impl From<TrackChanges> for jubarte::markdown::TrackChanges {
    fn from(choice: TrackChanges) -> Self {
        match choice {
            TrackChanges::All => Self::All,
            TrackChanges::Accept => Self::Accept,
            TrackChanges::Reject => Self::Reject,
        }
    }
}

/// `convert`'s format and Markdown flags.
#[derive(clap::Args, Debug)]
struct MarkdownArgs {
    /// Input format [default: from the file: .md and .markdown are Markdown,
    /// a zip is Word].
    #[arg(short = 'f', long = "from", value_enum, value_name = "FORMAT")]
    from: Option<Format>,
    /// Output format [default: from --output, else pdf for Word and docx for
    /// Markdown].
    #[arg(short = 't', long = "to", value_enum, value_name = "FORMAT")]
    to: Option<Format>,
    /// Keep tracked changes (all), or write the document with every change
    /// accepted or rejected (pandoc's flag): CriticMarkup in Markdown, Word's
    /// revisions in a .docx. With --to md, the Markdown itself is resolved.
    #[arg(long, value_enum, value_name = "CHOICE", default_value_t = TrackChanges::All)]
    track_changes: TrackChanges,
    /// Markdown: read `{++`, `{--` and the other CriticMarkup delimiters as
    /// text.
    #[arg(long)]
    no_critic: bool,
    /// Markdown to Word: take styles, numbering, page setup, headers and
    /// footers from this .docx (pandoc's --reference-doc).
    #[arg(long, value_name = "FILE")]
    reference_doc: Option<PathBuf>,
    /// Markdown to Word: where images are found [default: the Markdown
    /// file's directory].
    #[arg(long, value_name = "DIR")]
    resource_path: Option<PathBuf>,
    /// Markdown to Word: author of the tracked changes and comments.
    #[arg(short = 'a', long, value_name = "NAME", default_value = "Redline")]
    author: String,
    /// Markdown to Word: their date (ISO 8601); pinned for reproducible
    /// output.
    #[arg(
        short = 'd',
        long,
        value_name = "ISO8601",
        default_value = "1970-01-01T00:00:00Z"
    )]
    date: String,
}

/// `jubarte convert --revisions`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum Revisions {
    /// Red strike, blue double underline, green moves.
    Conventional,
    /// Microsoft Word's own markup.
    Word,
    /// --revision-palette.
    Custom,
}

/// The library's revision style for `--revisions` / `--revision-palette`.
fn revision_style(
    revisions: Revisions,
    palette: Option<&str>,
) -> Result<jubarte::convert::RevisionStyle, String> {
    use jubarte::convert::{RevisionPalette, RevisionStyle};
    match (revisions, palette) {
        (Revisions::Custom, Some(spec)) => Ok(RevisionStyle::Custom(
            RevisionPalette::parse(spec).map_err(|e| format!("--revision-palette: {e}"))?,
        )),
        (Revisions::Custom, None) => Err("--revisions custom needs --revision-palette".to_string()),
        (_, Some(_)) => Err("--revision-palette needs --revisions custom".to_string()),
        (Revisions::Conventional, None) => Ok(RevisionStyle::Conventional),
        (Revisions::Word, None) => Ok(RevisionStyle::Word),
    }
}

/// No-clobber contract shared by every writing subcommand.
/// Whether two CLI paths name the same file, whether or not it exists yet
/// (`out.pdf` and `./out.pdf` do; the parent directory is canonicalized).
fn same_path(a: &Path, b: &Path) -> bool {
    fn key(p: &Path) -> Option<PathBuf> {
        let name = p.file_name()?;
        let parent = match p.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
            _ => PathBuf::from("."),
        };
        Some(std::fs::canonicalize(parent).ok()?.join(name))
    }
    a == b || matches!((key(a), key(b)), (Some(x), Some(y)) if x == y)
}

/// Two directory paths name the same existing directory. `same_path` keys on
/// the file name, which `.` and `./` do not have.
fn same_dir(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => same_path(a, b),
    }
}

fn ensure_writable(output: &Path, force: bool) -> Result<(), String> {
    if output.exists() && !force {
        return Err(format!(
            "output '{}' already exists (use --force to overwrite)",
            output.display()
        ));
    }
    Ok(())
}

/// Shared body for `accept` / `reject`: read the redline, apply the package-wide
/// resolution, and write the result under the compare path's no-clobber
/// contract. Generic over the resolver's error so neither `OpcError`'s path nor
/// the two arms' bodies are duplicated.
/// Which tracked changes `accept` / `reject` resolve; all of them when no
/// flag is given, else those matching every flag kind given.
#[derive(clap::Args, Debug, Default, PartialEq)]
struct Selection {
    /// Only this change (`body:rev:12`, as `jubarte changes` lists it).
    /// Repeatable.
    #[arg(long = "id", value_name = "ID")]
    ids: Vec<String>,
    /// Only changes by this author. Repeatable.
    #[arg(long = "author", value_name = "NAME")]
    authors: Vec<String>,
    /// Only changes of this kind. Repeatable.
    #[arg(long = "kind", value_enum, value_name = "KIND")]
    kinds: Vec<KindArg>,
}

/// `--kind` values.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq)]
enum KindArg {
    Insertion,
    Deletion,
    Move,
    Formatting,
}

impl Selection {
    fn filter(&self) -> jubarte::changes::ChangeFilter {
        use jubarte::changes::ChangeKind;
        let given = |v: &[String]| (!v.is_empty()).then(|| v.to_vec());
        jubarte::changes::ChangeFilter {
            ids: given(&self.ids),
            authors: given(&self.authors),
            kinds: (!self.kinds.is_empty()).then(|| {
                self.kinds
                    .iter()
                    .map(|k| match k {
                        KindArg::Insertion => ChangeKind::Insertion,
                        KindArg::Deletion => ChangeKind::Deletion,
                        KindArg::Move => ChangeKind::Move,
                        KindArg::Formatting => ChangeKind::Formatting,
                    })
                    .collect()
            }),
        }
    }
}

fn run_resolution(
    file: &Path,
    output: &Path,
    force: bool,
    filter: &jubarte::changes::ChangeFilter,
    apply: fn(
        &[u8],
        &jubarte::changes::ChangeFilter,
    ) -> Result<Vec<u8>, jubarte::changes::ChangeError>,
    what: &str,
) -> Result<(), String> {
    ensure_writable(output, force)?;
    let bytes = read_document(file)?;
    let out = apply(&bytes, filter).map_err(|e| format!("{what} failed: {e}"))?;
    std::fs::write(output, &out).map_err(|e| format!("writing {}: {e}", output.display()))
}

fn run_changes(file: &Path, json: bool) -> Result<(), String> {
    let bytes = read_document(file)?;
    let changes = jubarte::changes::list_changes(&bytes).map_err(|e| e.to_string())?;
    for c in &changes {
        if json {
            println!("{}", serde_json::to_string(c).map_err(|e| e.to_string())?);
            continue;
        }
        let kind = serde_json::to_value(c.kind).map_err(|e| e.to_string())?;
        let preview: String = c.text.chars().take(60).collect();
        let inside = c
            .inside
            .as_deref()
            .map(|id| format!("\tinside {id}"))
            .unwrap_or_default();
        println!(
            "{}\t{}\t{}\t{}\t{preview:?}{inside}",
            c.id,
            kind.as_str().unwrap_or("?"),
            c.target,
            c.author.as_deref().unwrap_or("-"),
        );
    }
    if !json {
        println!("{} change(s)", changes.len());
    }
    Ok(())
}

fn run_comments(file: &Path, json: bool, author: Option<&str>, latest: bool) -> Result<(), String> {
    let bytes = read_document(file)?;
    let comments = jubarte::comments::list_comments(&bytes).map_err(|e| e.to_string())?;
    let comments = jubarte::comments::select_comments(comments, author, latest);
    for c in &comments {
        if json {
            println!("{}", serde_json::to_string(c).map_err(|e| e.to_string())?);
            continue;
        }
        let text: String = c.text.chars().take(60).collect();
        let anchor: String = c.anchor_text.chars().take(40).collect();
        let thread = c
            .parent
            .map(|p| format!("\treply to {p}"))
            .unwrap_or_default();
        let done = if c.done { "\tresolved" } else { "" };
        println!(
            "{}\t{}\t{}\t{text:?}\ton {anchor:?}{thread}{done}",
            c.id,
            c.paragraph.as_deref().unwrap_or("-"),
            c.author,
        );
    }
    if !json {
        println!("{} comment(s)", comments.len());
    }
    Ok(())
}

/// Which artifacts `convert` writes and where.
struct ConvertJob<'a> {
    file: &'a Path,
    /// The .docx to render instead of reading `file` (written from Markdown).
    bytes: Option<&'a [u8]>,
    output: Option<&'a Path>,
    force: bool,
    compress: bool,
    font_report: Option<&'a Path>,
    revisions: jubarte::convert::RevisionStyle,
    pdf: bool,
    png: bool,
    dpi: f32,
    report: Option<&'a Path>,
    /// Zero-based pages to rasterize; `None` for all.
    pages: Option<&'a [usize]>,
}

fn run_convert(job: &ConvertJob<'_>) -> Result<(), String> {
    let output = job
        .output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| job.file.with_extension("pdf"));
    let want_pdf = job.pdf || !job.png;
    if job.pages.is_some() && !job.png {
        return Err("--pages selects PNG pages; add --png".into());
    }
    for (side, what) in [(job.font_report, "--font-report"), (job.report, "--report")] {
        if let Some(side) = side {
            // Side files are written after the PDF (or the input is read
            // first), so a shared path would silently replace one with the other.
            for (other, name) in [(output.as_path(), "PDF output"), (job.file, "input")] {
                if same_path(side, other) {
                    return Err(format!(
                        "{what} '{}' is the same file as the {name}",
                        side.display()
                    ));
                }
            }
            ensure_writable(side, job.force)?;
        }
    }
    if want_pdf {
        ensure_writable(&output, job.force)?;
    }
    let stem = output
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "page".to_string());
    let dir = output.parent().map(Path::to_path_buf).unwrap_or_default();
    let bytes = match job.bytes {
        Some(bytes) => bytes.to_vec(),
        None => read_document(job.file)?,
    };
    let options = jubarte::convert::PdfOptions {
        compress: job.compress,
        revisions: job.revisions,
    };
    let rendered = jubarte::convert::render(
        &bytes,
        options,
        jubarte::convert::RenderRequest {
            pdf: want_pdf,
            png_dpi: job.png.then_some(job.dpi),
            pages: job.pages.map(<[usize]>::to_vec),
        },
    )
    .map_err(|e| format!("convert failed: {e}"))?;
    let pages = rendered.report.page_count;
    // `render` returns the selected pages ascending and without repeats.
    let indices: Vec<usize> = match job.pages {
        Some(selected) => {
            let mut selected = selected.to_vec();
            selected.sort_unstable();
            selected.dedup();
            selected
        }
        None => (0..rendered.pngs.len()).collect(),
    };
    // The page count is known only now; check every PNG path before the
    // first write so a refused page leaves no partial bundle.
    let png_paths: Vec<PathBuf> = indices
        .iter()
        .map(|&i| dir.join(png_name(&stem, i, pages)))
        .collect();
    for path in &png_paths {
        ensure_writable(path, job.force)?;
    }
    if let Some(pdf) = &rendered.pdf {
        std::fs::write(&output, pdf).map_err(|e| format!("writing {}: {e}", output.display()))?;
        println!(
            "wrote {} ({} bytes, {pages} page{})",
            output.display(),
            pdf.len(),
            if pages == 1 { "" } else { "s" }
        );
    }
    if job.png {
        for (path, png) in png_paths.iter().zip(&rendered.pngs) {
            std::fs::write(path, png).map_err(|e| format!("writing {}: {e}", path.display()))?;
        }
        println!(
            "wrote {} PNG page{} ({}-page-NN.png, {} dpi)",
            rendered.pngs.len(),
            if rendered.pngs.len() == 1 { "" } else { "s" },
            dir.join(&stem).display(),
            job.dpi
        );
    }
    if let Some(report) = job.font_report {
        let json = jubarte::convert::font_report_json(&rendered.report.fonts);
        std::fs::write(report, json).map_err(|e| format!("writing {}: {e}", report.display()))?;
    }
    if let Some(report) = job.report {
        std::fs::write(report, rendered.report.to_json())
            .map_err(|e| format!("writing {}: {e}", report.display()))?;
    }
    Ok(())
}

/// `<stem>-page-NN.png`, zero-padded to the page count's width (at least 2).
/// `--pages` as zero-based indices, ascending and without repeats: `1-3,7`
/// is `[0, 1, 2, 6]`. Pages are counted from 1 on the command line.
fn parse_pages(spec: &str) -> Result<Vec<usize>, String> {
    let page = |text: &str| -> Result<usize, String> {
        match text.trim().parse::<usize>() {
            Ok(0) => Err(format!("--pages '{spec}': pages are counted from 1")),
            Ok(n) => Ok(n - 1),
            Err(_) => Err(format!(
                "--pages '{spec}': '{}' is not a page number",
                text.trim()
            )),
        }
    };
    let mut pages = Vec::new();
    for item in spec.split(',') {
        if item.trim().is_empty() {
            return Err(format!("--pages '{spec}': empty item"));
        }
        match item.split_once('-') {
            Some((first, last)) => {
                let (first, last) = (page(first)?, page(last)?);
                if first > last {
                    return Err(format!(
                        "--pages '{spec}': '{}' runs backwards",
                        item.trim()
                    ));
                }
                pages.extend(first..=last);
            }
            None => pages.push(page(item)?),
        }
    }
    pages.sort_unstable();
    pages.dedup();
    Ok(pages)
}

/// What `diff-render` compares and where it writes.
struct DiffRenderJob<'a> {
    a: &'a Path,
    b: &'a Path,
    out_dir: Option<&'a Path>,
    dpi: f32,
    json: bool,
    overlay: bool,
    force: bool,
}

/// Compare two documents page by page; `Ok(true)` when any page differs.
fn run_diff_render(job: &DiffRenderJob<'_>) -> Result<bool, String> {
    let a = read_document(job.a)?;
    let b = read_document(job.b)?;
    let options = jubarte::convert::DiffOptions {
        dpi: job.dpi,
        overlay: job.overlay && job.out_dir.is_some(),
        ..jubarte::convert::DiffOptions::default()
    };
    let diff = jubarte::convert::diff_render(&a, &b, &options)
        .map_err(|e| format!("diff-render failed: {e}"))?;
    let changed: Vec<&jubarte::convert::PageDiff> =
        diff.pages.iter().filter(|p| p.differs()).collect();
    /// `diff.json`. Serialized directly (not through `serde_json::Value`)
    /// so the `f32` ratios print at their own precision.
    #[derive(serde::Serialize)]
    struct Summary<'a> {
        a: String,
        b: String,
        dpi: f32,
        a_pages: usize,
        b_pages: usize,
        changed: usize,
        pages: &'a [jubarte::convert::PageDiff],
    }
    let summary = serde_json::to_string(&Summary {
        a: job.a.display().to_string(),
        b: job.b.display().to_string(),
        dpi: job.dpi,
        a_pages: diff.a_report.page_count,
        b_pages: diff.b_report.page_count,
        changed: changed.len(),
        pages: &diff.pages,
    })
    .map_err(|e| format!("diff.json: {e}"))?;
    if let Some(dir) = job.out_dir {
        let count = diff.pages.len();
        let mut files: Vec<(PathBuf, &[u8])> = Vec::new();
        for page in &changed {
            let i = page.index;
            let sides = [
                ("a", diff.a.get(i)),
                ("b", diff.b.get(i)),
                ("diff", diff.overlays[i].as_ref()),
            ];
            for (prefix, png) in sides {
                if let Some(png) = png {
                    files.push((dir.join(png_name(prefix, i, count)), png));
                }
            }
        }
        files.push((dir.join("diff.json"), summary.as_bytes()));
        // Check every path before the first write so a refused file leaves
        // no partial bundle.
        for (path, _) in &files {
            ensure_writable(path, job.force)?;
        }
        std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        for (path, bytes) in &files {
            std::fs::write(path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))?;
        }
    }
    if job.json {
        println!("{summary}");
    } else {
        for page in &changed {
            match page.only_in {
                Some(side) => println!("page {}: only in {side}", page.index + 1),
                None => println!(
                    "page {}: {:.2}% of pixels changed, box {:?}",
                    page.index + 1,
                    page.changed_ratio * 100.0,
                    page.bbox.unwrap_or_default()
                ),
            }
        }
        println!(
            "{} of {} page{} differ{}",
            changed.len(),
            diff.pages.len(),
            if diff.pages.len() == 1 { "" } else { "s" },
            job.out_dir
                .map(|dir| format!(" (wrote {})", dir.display()))
                .unwrap_or_default()
        );
    }
    Ok(!changed.is_empty())
}

fn png_name(stem: &str, index: usize, count: usize) -> String {
    let width = count.to_string().len().max(2);
    format!("{stem}-page-{:0width$}.png", index + 1)
}

fn run_inspect(file: &Path, json: bool) -> Result<(), String> {
    let bytes = read_document(file)?;
    if json {
        println!(
            "{}",
            jubarte::inspect::inspect_json(&bytes).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    let summary = jubarte::inspect::summary(&bytes).map_err(|e| e.to_string())?;
    let paragraphs = jubarte::inspect::paragraphs(&bytes).map_err(|e| e.to_string())?;
    println!(
        "sha256: {}\nparagraphs: {}  tables: {}  fields: {}  sections: {}  comments: {}  revisions: {}  footnotes: {}  endnotes: {}  headers: {}  footers: {}  images: {}  numbering: {}  track_changes: {}",
        jubarte::inspect::source_sha256(&bytes),
        summary.paragraphs,
        summary.tables,
        summary.fields,
        summary.sections,
        summary.comments,
        summary.revisions,
        summary.footnotes,
        summary.endnotes,
        summary.headers,
        summary.footers,
        summary.images,
        summary.list_numbering,
        summary.track_changes
    );
    for p in &paragraphs {
        let mut flags = Vec::new();
        if let Some(style) = &p.style {
            flags.push(style.clone());
        }
        if p.numbered {
            flags.push("numbered".into());
        }
        if p.in_table {
            flags.push("table".into());
        }
        if p.page_break {
            flags.push("page-break".into());
        }
        flags.extend(p.limitations.iter().cloned());
        let preview: String = p.text.chars().take(80).collect();
        let more = if p.text.chars().count() > 80 {
            "…"
        } else {
            ""
        };
        println!("{}\t[{}]\t{preview}{more}", p.id, flags.join(","));
    }
    Ok(())
}

fn run_inspect_tables(file: &Path) -> Result<(), String> {
    let bytes = read_document(file)?;
    let tables = jubarte::inspect::tables(&bytes).map_err(|e| e.to_string())?;
    if tables.is_empty() {
        println!("no tables");
    }
    for table in &tables {
        let columns = table.rows.iter().map(Vec::len).max().unwrap_or(0);
        let widths: Vec<String> = table.widths_dxa.iter().map(u32::to_string).collect();
        println!(
            "table {}: {}x{columns} header_rows={} widths={}",
            table.index,
            table.rows.len(),
            table.header_rows,
            widths.join(",")
        );
        for row in &table.rows {
            let cells: Vec<String> = row
                .iter()
                .map(|cell| {
                    let ids = if cell.paragraph_ids.is_empty() {
                        "-".to_string()
                    } else {
                        cell.paragraph_ids.join(",")
                    };
                    let text = cell
                        .text
                        .replace('\\', "\\\\")
                        .replace('\n', "\\n")
                        .replace('\t', "\\t");
                    format!("{ids}={text}")
                })
                .collect();
            println!("{}", cells.join("\t"));
        }
    }
    Ok(())
}

fn run_text(file: &Path) -> Result<(), String> {
    let bytes = read_document(file)?;
    print!(
        "{}",
        jubarte::inspect::markdown(&bytes).map_err(|e| e.to_string())?
    );
    Ok(())
}

/// Options for `edit`.
struct EditJob<'a> {
    file: &'a Path,
    plan: &'a Path,
    out_dir: &'a Path,
    dry_run: bool,
    force: bool,
    pdf: bool,
    png: bool,
    dpi: f32,
    revisions: jubarte::convert::RevisionStyle,
    quiet: bool,
}

/// Exit 3: the plan was refused (stale source, ambiguous anchor, ...); the
/// per-operation report is on stdout and nothing was written.
const EXIT_PLAN_REFUSED: u8 = 3;

fn run_edit(job: &EditJob<'_>) -> Result<(), (u8, String)> {
    let fail = |m: String| (1u8, m);
    let source = read_document(job.file).map_err(fail)?;
    let plan_json = std::fs::read_to_string(job.plan)
        .map_err(|e| fail(format!("reading {}: {e}", job.plan.display())))?;
    let plan = jubarte::edit::EditPlan::from_json(&plan_json)
        .map_err(|e| (EXIT_PLAN_REFUSED, e.to_string()))?;
    if !job.dry_run {
        if job.out_dir.exists() && !job.force {
            return Err(fail(format!(
                "output directory '{}' already exists (use --force to replace its files)",
                job.out_dir.display()
            )));
        }
        let input_dir = job
            .file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        if same_dir(job.out_dir, input_dir) {
            return Err(fail(
                "--out-dir must not be the input's own directory".into(),
            ));
        }
    }
    if job.dry_run {
        let report = jubarte::edit::preview_plan(&source, &plan).map_err(|e| refused(&e))?;
        print!("{}", report.to_jsonl());
        return Ok(());
    }
    let result = jubarte::edit::apply_plan(&source, &plan).map_err(|e| refused(&e))?;
    let mut jsonl = result.report.to_jsonl();
    // Render (one layout pass per document) before creating the directory,
    // so a failed render leaves no partial bundle.
    let options = jubarte::convert::PdfOptions {
        compress: true,
        revisions: job.revisions,
    };
    let request = jubarte::convert::RenderRequest {
        pdf: job.pdf,
        png_dpi: job.png.then_some(job.dpi),
        pages: None,
    };
    let mut renders = Vec::new();
    if job.pdf || job.png {
        for (name, bytes) in [("redline", &result.redline), ("clean", &result.clean)] {
            let rendered = jubarte::convert::render(bytes, options, request.clone())
                .map_err(|e| fail(format!("rendering {name}: {e}")))?;
            renders.push((name, rendered));
        }
        let mut pages = serde_json::Map::new();
        let mut starts = serde_json::Map::new();
        for (name, rendered) in &renders {
            pages.insert((*name).into(), rendered.report.page_count.into());
            let firsts: Vec<serde_json::Value> = rendered
                .report
                .pages
                .iter()
                .map(|p| {
                    p.text
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(60)
                        .collect::<String>()
                        .into()
                })
                .collect();
            starts.insert((*name).into(), firsts.into());
        }
        let render_line = serde_json::json!({
            "ev": "render",
            "engine": format!("jubarte {}", env!("CARGO_PKG_VERSION")),
            "pages": pages,
            "page_starts": starts,
        });
        insert_before_summary(&mut jsonl, &render_line.to_string());
    }
    std::fs::create_dir_all(job.out_dir)
        .map_err(|e| fail(format!("creating {}: {e}", job.out_dir.display())))?;
    let name = job.file.file_name().map_or_else(
        || job.file.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let patch = jubarte::markdown::patch_redline(
        &result.redline,
        &jubarte::markdown::PatchOptions {
            old_name: name.clone(),
            new_name: name,
            owner: jubarte::markdown::Attribution {
                author: result.report.author.clone(),
                date: result.report.date.clone(),
            },
        },
    )
    .map_err(|e| fail(format!("writing the patch: {e}")))?
    .render(jubarte::markdown::DEFAULT_COLUMNS);
    let mut outputs: Vec<(String, Vec<u8>)> = vec![
        ("clean.docx".into(), result.clean.clone()),
        ("redline.docx".into(), result.redline.clone()),
        ("patch.diff".into(), patch.clone().into_bytes()),
    ];
    for (name, rendered) in renders {
        if let Some(pdf) = rendered.pdf {
            outputs.push((format!("{name}.pdf"), pdf));
        }
        let count = rendered.pngs.len();
        for (i, png) in rendered.pngs.into_iter().enumerate() {
            outputs.push((png_name(name, i, count), png));
        }
    }
    let mut saved = Vec::new();
    for (name, bytes) in &outputs {
        let path = job.out_dir.join(name);
        std::fs::write(&path, bytes)
            .map_err(|e| fail(format!("writing {}: {e}", path.display())))?;
        saved.push(serde_json::json!({"f": name, "bytes": bytes.len(), "sha256": jubarte::inspect::source_sha256(bytes)}));
    }
    let save_line = serde_json::json!({"ev": "save", "dir": job.out_dir.display().to_string(), "outputs": saved});
    insert_before_summary(&mut jsonl, &save_line.to_string());
    std::fs::write(job.out_dir.join("report.jsonl"), &jsonl)
        .map_err(|e| fail(format!("writing report.jsonl: {e}")))?;
    if job.quiet {
        return Ok(());
    }
    let summary = jsonl.lines().last().unwrap_or("").to_string();
    println!("{summary}");
    println!(
        "wrote {} ({} files: clean.docx, redline.docx, patch.diff, report.jsonl{})",
        job.out_dir.display(),
        outputs.len() + 1,
        if outputs.len() > 3 { ", …" } else { "" }
    );
    print!("{patch}");
    Ok(())
}

/// A refused plan: its report goes to stdout, the error to stderr, exit 3.
fn refused(e: &jubarte::edit::EditError) -> (u8, String) {
    let report_lines: String = e
        .outcomes
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let mut v = serde_json::json!({"ev": "op", "i": i + 1, "id": o.id, "op": o.kind, "status": o.status, "matches": o.matches});
            let m = v.as_object_mut().expect("object");
            if let Some(p) = &o.paragraph {
                m.insert("at".into(), p.clone().into());
            }
            if let Some(c) = &o.context {
                m.insert("ctx".into(), c.clone().into());
            }
            if let Some(c) = &o.code {
                m.insert("code".into(), c.clone().into());
            }
            if let Some(msg) = &o.message {
                m.insert("message".into(), msg.clone().into());
            }
            v.to_string() + "\n"
        })
        .collect();
    let summary = serde_json::json!({
        "ev": "summary",
        "status": "failed",
        "code": e.code,
        "operation": e.operation,
        "message": e.message,
    });
    println!("{report_lines}{summary}");
    (EXIT_PLAN_REFUSED, format!("plan refused: {e}"))
}

/// Insert `line` before the trailing `summary` line of a JSONL report.
fn insert_before_summary(jsonl: &mut String, line: &str) {
    let trimmed = jsonl.trim_end_matches('\n');
    let (head, summary) = match trimmed.rfind('\n') {
        Some(i) => (&trimmed[..i], &trimmed[i + 1..]),
        None => ("", trimmed),
    };
    *jsonl = if head.is_empty() {
        format!("{line}\n{summary}\n")
    } else {
        format!("{head}\n{line}\n{summary}\n")
    };
}

fn run_revisions(file: &Path, json: bool) -> Result<(), String> {
    let bytes = read_document(file)?;
    let settings = jubarte::comparer::WmlComparerSettings::default();
    let revs = jubarte::document_comparer::get_revisions(&bytes, &settings)
        .map_err(|e| format!("get_revisions failed: {e:?}"))?;
    if json {
        // Shared serialization (also the wasm `getRevisions` shape): full JSON
        // string escaping — backslash, quote, and ALL control chars < 0x20.
        for r in &revs {
            println!("{}", jubarte::document_comparer::revision_to_json(r));
        }
    } else {
        for r in &revs {
            let text = r.text.as_deref().unwrap_or("");
            let preview: String = text.chars().take(60).collect();
            println!(
                "{:?}\t{}\t{}\t{:?}",
                r.revision_type,
                r.author.as_deref().unwrap_or("-"),
                r.part_name,
                preview
            );
        }
        println!("{} revision(s)", revs.len());
    }
    Ok(())
}

/// A fully-resolved comparison job (positional/named merged, output computed).
#[derive(Debug, PartialEq)]
struct Job {
    original: PathBuf,
    modified: PathBuf,
    output: PathBuf,
    author: String,
    date: String,
    force: bool,
    quiet: bool,
    detail_threshold: Option<f64>,
    powertools_faithful: bool,
    no_paragraph_merge: bool,
}

impl Cli {
    /// Merge positional and named inputs (named flags win), compute the default
    /// output path, and validate that both documents are supplied.
    fn resolve(self) -> Result<Job, String> {
        let original = self
            .original
            .or(self.original_pos)
            .ok_or("missing ORIGINAL document (a positional arg or --original/-b)")?;
        let modified = self
            .modified
            .or(self.modified_pos)
            .ok_or("missing MODIFIED document (a positional arg or --modified/-m)")?;
        let output = self
            .output
            .unwrap_or_else(|| default_output(&original, &modified));
        Ok(Job {
            original,
            modified,
            output,
            author: self.author,
            date: self.date,
            force: self.force,
            quiet: self.quiet,
            detail_threshold: self.detail_threshold,
            powertools_faithful: self.powertools_faithful || self.mode == CompareMode::Powertools,
            no_paragraph_merge: self.no_paragraph_merge,
        })
    }
}

/// Build the default output path: `<original-dir>/<orig-stem>_v_<mod-stem>.docx`.
fn default_output(original: &Path, modified: &Path) -> PathBuf {
    let stem = |p: &Path| {
        p.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "doc".to_string())
    };
    let name = format!("{}_v_{}.docx", stem(original), stem(modified));
    match original.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(dir) => dir.join(name),
        None => PathBuf::from(name),
    }
}

/// The comparer's settings for the `--mode`, `--detail-threshold`, author
/// and date flags.
fn comparer_settings(
    author: &str,
    date: &str,
    detail_threshold: Option<f64>,
    powertools_faithful: bool,
    no_paragraph_merge: bool,
) -> jubarte::comparer::WmlComparerSettings {
    let base = if powertools_faithful {
        jubarte::comparer::WmlComparerSettings::powertools_faithful()
    } else {
        jubarte::comparer::WmlComparerSettings::default()
    };
    jubarte::comparer::WmlComparerSettings {
        author_for_revisions: author.to_string(),
        date_time_for_revisions: date.to_string(),
        detail_threshold: detail_threshold.unwrap_or(base.detail_threshold),
        merge_replaced_paragraphs: if no_paragraph_merge {
            false
        } else {
            base.merge_replaced_paragraphs
        },
        ..base
    }
}

fn run(job: &Job) -> Result<(), String> {
    ensure_writable(&job.output, job.force)?;
    let original = read_document(&job.original)?;
    let modified = read_document(&job.modified)?;
    let settings = comparer_settings(
        &job.author,
        &job.date,
        job.detail_threshold,
        job.powertools_faithful,
        job.no_paragraph_merge,
    );
    let formats = (
        Format::of_input(None, &job.original, &original),
        Format::of_input(None, &job.modified, &modified),
    );
    let out = if formats == (Format::Docx, Format::Docx)
        && Format::of_path(&job.output) != Some(Format::Md)
    {
        jubarte::document_comparer::compare_documents_with_settings(&original, &modified, &settings)
            .map_err(|e| format!("compare failed: {e:?}"))?
    } else {
        let old = Input::new(&job.original, formats.0, original)?;
        let new = Input::new(&job.modified, formats.1, modified)?;
        let to = Format::of_path(&job.output).unwrap_or(Format::Docx);
        let options = jubarte::markdown::RedlineOptions {
            settings,
            ..Default::default()
        };
        compared(&old, &new, to, &options)?
    };

    std::fs::write(&job.output, &out)
        .map_err(|e| format!("writing {}: {e}", job.output.display()))?;

    if !job.quiet {
        println!("wrote {} ({} bytes)", job.output.display(), out.len());
    }
    Ok(())
}

/// Shared `Result → ExitCode` mapping for every command arm.
fn exit_code(r: Result<(), String>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// A document read for comparison: Word bytes or Markdown text.
struct Input<'p> {
    path: &'p Path,
    bytes: Vec<u8>,
    markdown: Option<String>,
}

impl<'p> Input<'p> {
    fn new(path: &'p Path, format: Format, bytes: Vec<u8>) -> Result<Self, String> {
        let markdown = match format {
            Format::Md => Some(markdown_text(path, bytes.clone())?),
            Format::Docx => None,
            Format::Pdf | Format::Png => {
                return Err(format!("{}: PDF and PNG are not inputs", path.display()));
            }
        };
        Ok(Self {
            path,
            bytes,
            markdown,
        })
    }

    fn source(&self) -> jubarte::markdown::Source<'_> {
        match &self.markdown {
            Some(text) => jubarte::markdown::Source::Markdown(text),
            None => jubarte::markdown::Source::Docx(&self.bytes),
        }
    }
}

/// The first bytes of an OLE compound file: a Word 97-2003 `.doc`, or a
/// password-encrypted document of any Word version.
const OLE_MAGIC: &[u8] = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1";

/// Reads an input document, refusing the OLE files Word alone can open.
fn read_document(path: &Path) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    if bytes.starts_with(OLE_MAGIC) {
        return Err(format!(
            "{} is a Word 97-2003 (.doc) or encrypted document; open it in Word and save it as .docx without a password",
            path.display()
        ));
    }
    Ok(bytes)
}

/// Markdown bytes as text, without a byte order mark.
fn markdown_text(path: &Path, bytes: Vec<u8>) -> Result<String, String> {
    let text = String::from_utf8(bytes)
        .map_err(|_| format!("{}: Markdown must be UTF-8", path.display()))?;
    Ok(text
        .strip_prefix('\u{FEFF}')
        .map(str::to_string)
        .unwrap_or(text))
}

/// Reads images a Markdown file names: local paths, relative to `dir`, with
/// `%XX` escapes decoded. URLs are not fetched.
fn image_loader(dir: PathBuf) -> impl Fn(&str) -> Option<Vec<u8>> {
    move |name: &str| {
        if name.contains("://") || name.starts_with("data:") {
            return None;
        }
        let mut decoded = Vec::with_capacity(name.len());
        let bytes = name.as_bytes();
        let mut at = 0;
        while at < bytes.len() {
            let hex = bytes
                .get(at + 1..at + 3)
                .and_then(|h| std::str::from_utf8(h).ok())
                .and_then(|h| u8::from_str_radix(h, 16).ok());
            match (bytes[at], hex) {
                (b'%', Some(byte)) => {
                    decoded.push(byte);
                    at += 3;
                }
                (byte, _) => {
                    decoded.push(byte);
                    at += 1;
                }
            }
        }
        let path = PathBuf::from(String::from_utf8(decoded).ok()?);
        std::fs::read(if path.is_absolute() {
            path
        } else {
            dir.join(path)
        })
        .ok()
    }
}

/// The directory a Markdown file's images are read from.
fn resource_dir(markdown: &Path, resource_path: Option<&Path>) -> PathBuf {
    resource_path.map(Path::to_path_buf).unwrap_or_else(|| {
        markdown
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
    })
}

/// `old` against `new` in format `to`: CriticMarkup (both Markdown) or a
/// Word redline.
fn compared(
    old: &Input<'_>,
    new: &Input<'_>,
    to: Format,
    options: &jubarte::markdown::RedlineOptions<'_>,
) -> Result<Vec<u8>, String> {
    match to {
        Format::Md => match (&old.markdown, &new.markdown) {
            (Some(old), Some(new)) => Ok(jubarte::markdown::diff_markdown(old, new).into_bytes()),
            _ => Err(format!(
                "Markdown output needs both documents in Markdown ({} is Word): \
                 write a Word redline (-o FILE.docx) or a PDF (-o FILE.pdf) instead",
                if old.markdown.is_none() {
                    old.path
                } else {
                    new.path
                }
                .display()
            )),
        },
        _ => jubarte::markdown::redline(old.source(), new.source(), options)
            .map_err(|e| format!("compare failed: {e}")),
    }
}

/// Options for `diff`.
struct DiffJob<'a> {
    old: &'a Path,
    new: &'a Path,
    output: Option<&'a Path>,
    to: Option<Format>,
    from: Option<Format>,
    force: bool,
    settings: jubarte::comparer::WmlComparerSettings,
    reference: Option<&'a Path>,
    critic: bool,
    resource_path: Option<&'a Path>,
    revisions: jubarte::convert::RevisionStyle,
    /// Print the patch, wrapped at these columns; `None` for `--format
    /// critic`.
    patch: Option<usize>,
}

fn run_diff(job: &DiffJob<'_>) -> Result<(), String> {
    let (old_bytes, new_bytes) = (read_document(job.old)?, read_document(job.new)?);
    let old = Input::new(
        job.old,
        Format::of_input(job.from, job.old, &old_bytes),
        old_bytes,
    )?;
    let new = Input::new(
        job.new,
        Format::of_input(job.from, job.new, &new_bytes),
        new_bytes,
    )?;
    let both_markdown = old.markdown.is_some() && new.markdown.is_some();
    let to = job
        .to
        .or_else(|| job.output.and_then(Format::of_path))
        .unwrap_or(if both_markdown {
            Format::Md
        } else {
            Format::Docx
        });
    let output = match (job.output, to) {
        (Some(path), _) => Some(path.to_path_buf()),
        (None, Format::Md) => None,
        (None, Format::Docx) => Some(default_output(job.old, job.new)),
        (None, Format::Pdf | Format::Png) => {
            Some(default_output(job.old, job.new).with_extension("pdf"))
        }
    };
    let reference = job.reference.map(read_document).transpose()?;
    // Images of a Markdown side are read next to it (both sides share
    // --resource-path when given).
    let markdown_side = if old.markdown.is_some() {
        job.old
    } else {
        job.new
    };
    let loader = image_loader(resource_dir(markdown_side, job.resource_path));
    let options = jubarte::markdown::RedlineOptions {
        settings: job.settings.clone(),
        reference: reference.as_deref(),
        critic: job.critic,
        images: Some(&loader),
    };
    // Two Markdown documents with no --output: the patch is all there is.
    let out = if job.patch.is_some() && output.is_none() && both_markdown {
        Vec::new()
    } else {
        compared(
            &old,
            &new,
            if to == Format::Md { to } else { Format::Docx },
            &options,
        )?
    };
    if let Some(columns) = job.patch {
        // A refused output prints no patch.
        if let (Format::Md | Format::Docx, Some(path)) = (to, &output) {
            ensure_writable(path, job.force)?;
        }
        let name = |path: &Path| path.display().to_string();
        let patch = jubarte::markdown::patch_documents(
            old.source(),
            new.source(),
            &options,
            &jubarte::markdown::PatchOptions {
                old_name: name(job.old),
                new_name: name(job.new),
                owner: jubarte::markdown::Attribution {
                    author: job.settings.author_for_revisions.clone(),
                    date: job.settings.date_time_for_revisions.clone(),
                },
            },
        )
        .map_err(|e| format!("compare failed: {e}"))?;
        print!("{}", patch.render(columns));
        if output.is_none() {
            return Ok(());
        }
    }
    match (to, output) {
        (Format::Md, None) => {
            use std::io::Write as _;
            std::io::stdout()
                .write_all(&out)
                .map_err(|e| format!("writing to stdout: {e}"))
        }
        (Format::Md | Format::Docx, Some(path)) => {
            ensure_writable(&path, job.force)?;
            std::fs::write(&path, &out).map_err(|e| format!("writing {}: {e}", path.display()))?;
            let wrote = format!("wrote {} ({} bytes)", path.display(), out.len());
            // With the patch on stdout, the rest goes to stderr.
            if job.patch.is_some() {
                eprintln!("{wrote}");
            } else {
                println!("{wrote}");
            }
            Ok(())
        }
        (Format::Pdf | Format::Png, output) => run_convert(&ConvertJob {
            file: job.old,
            bytes: Some(&out),
            output: output.as_deref(),
            force: job.force,
            compress: false,
            font_report: None,
            revisions: job.revisions,
            pdf: to == Format::Pdf,
            png: to == Format::Png,
            dpi: 96.0,
            report: None,
            pages: None,
        }),
        (Format::Docx, None) => unreachable!("a Word output always has a path"),
    }
}

/// `convert`, for every pair of formats it takes.
fn run_convert_any(job: &ConvertJob<'_>, markdown: &MarkdownArgs) -> Result<(), String> {
    let bytes = read_document(job.file)?;
    let from = Format::of_input(markdown.from, job.file, &bytes);
    let to = markdown
        .to
        .or_else(|| (job.pdf || job.png).then_some(Format::Pdf))
        .or_else(|| job.output.and_then(Format::of_path))
        .unwrap_or(match from {
            Format::Md => Format::Docx,
            _ => Format::Pdf,
        });
    if job.pages.is_some() && to != Format::Png && !job.png {
        return Err("--pages selects PNG pages; add --png".into());
    }
    let pdf_job = |bytes: Option<&[u8]>, to: Format| {
        let mut rendered = ConvertJob { bytes, ..*job };
        if to == Format::Png && !job.png {
            rendered.png = true;
        }
        run_convert(&rendered)
    };
    match (from, to) {
        (Format::Docx, Format::Pdf | Format::Png) => match markdown.track_changes {
            TrackChanges::All => pdf_job(None, to),
            // The pages of the document with every change accepted or rejected.
            choice => {
                let resolve = if choice == TrackChanges::Accept {
                    jubarte::document_comparer::accept_revisions
                } else {
                    jubarte::document_comparer::reject_revisions
                };
                let resolved = resolve(&bytes).map_err(|e| format!("convert failed: {e:?}"))?;
                pdf_job(Some(&resolved), to)
            }
        },
        (Format::Docx, Format::Md) => {
            let read = jubarte::markdown::docx_to_markdown(
                &bytes,
                &jubarte::markdown::MarkdownOptions {
                    track_changes: markdown.track_changes.into(),
                    extract_media: None,
                },
            )
            .map_err(|e| format!("convert failed: {e}"))?;
            match job.output {
                Some(output) => {
                    ensure_writable(output, job.force)?;
                    std::fs::write(output, &read.markdown)
                        .map_err(|e| format!("writing {}: {e}", output.display()))?;
                    println!("wrote {} ({} bytes)", output.display(), read.markdown.len());
                    Ok(())
                }
                None => {
                    print!("{}", read.markdown);
                    Ok(())
                }
            }
        }
        (Format::Docx, Format::Docx) => {
            let resolve = match markdown.track_changes {
                TrackChanges::Accept => jubarte::document_comparer::accept_revisions,
                TrackChanges::Reject => jubarte::document_comparer::reject_revisions,
                TrackChanges::All => {
                    return Err(format!(
                        "{} is already Word: give --track-changes accept or reject, or another --to",
                        job.file.display()
                    ));
                }
            };
            let output = job
                .output
                .ok_or("--output is required to write Word from Word")?;
            ensure_writable(output, job.force)?;
            let out = resolve(&bytes).map_err(|e| format!("convert failed: {e:?}"))?;
            std::fs::write(output, &out)
                .map_err(|e| format!("writing {}: {e}", output.display()))?;
            println!("wrote {} ({} bytes)", output.display(), out.len());
            Ok(())
        }
        (Format::Md, Format::Md) => {
            let text = markdown_text(job.file, bytes)?;
            let out = if markdown.no_critic {
                text
            } else {
                jubarte::markdown::resolve_critic(&text, markdown.track_changes.into())
            };
            match job.output {
                Some(output) => {
                    ensure_writable(output, job.force)?;
                    std::fs::write(output, &out)
                        .map_err(|e| format!("writing {}: {e}", output.display()))
                }
                None => {
                    print!("{out}");
                    Ok(())
                }
            }
        }
        (Format::Md, Format::Docx | Format::Pdf | Format::Png) => {
            let text = markdown_text(job.file, bytes)?;
            let reference = markdown
                .reference_doc
                .as_deref()
                .map(read_document)
                .transpose()?;
            let loader = image_loader(resource_dir(job.file, markdown.resource_path.as_deref()));
            let options = jubarte::markdown::DocxOptions {
                reference: reference.as_deref(),
                critic: !markdown.no_critic,
                track_changes: markdown.track_changes.into(),
                author: markdown.author.clone(),
                date: markdown.date.clone(),
                images: Some(&loader),
            };
            let written = jubarte::markdown::markdown_to_docx(&text, &options)
                .map_err(|e| format!("convert failed: {e}"))?;
            for warning in &written.warnings {
                eprintln!("warning: {warning}");
            }
            if to != Format::Docx {
                return pdf_job(Some(&written.docx), to);
            }
            let output = job
                .output
                .map_or_else(|| job.file.with_extension("docx"), Path::to_path_buf);
            ensure_writable(&output, job.force)?;
            std::fs::write(&output, &written.docx)
                .map_err(|e| format!("writing {}: {e}", output.display()))?;
            println!("wrote {} ({} bytes)", output.display(), written.docx.len());
            Ok(())
        }
        (Format::Pdf | Format::Png, _) => Err(format!(
            "{}: PDF and PNG are not inputs",
            job.file.display()
        )),
    }
}

/// `jubarte debug`: print the report for one package, or two compared.
fn run_debug(
    files: &[PathBuf],
    list: bool,
    checks: Vec<DebugCheck>,
    part: Option<String>,
    grep: Option<String>,
    limit: usize,
    context: usize,
) -> Result<(), String> {
    let read = |p: &PathBuf| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
    let a = read(&files[0])?;
    let b = files.get(1).map(read).transpose()?;
    let mut opts = jubarte::debug::Options {
        part,
        grep,
        limit,
        context,
        ..Default::default()
    };
    if !checks.is_empty() {
        opts.checks = checks.into_iter().map(Into::into).collect();
    }
    let out = if list {
        jubarte::debug::list(&a, b.as_deref(), &opts)?
    } else {
        jubarte::debug::report(&a, b.as_deref(), &opts)?
    };
    print!("{out}");
    Ok(())
}

/// `jubarte debug diff`: each file labelled by its stem (by its folder too
/// when stems repeat, by position when both do).
fn run_debug_diff(
    files: &[PathBuf],
    opts: &jubarte::debug::diff::DiffOptions,
) -> Result<(), String> {
    let stem = |p: &PathBuf| {
        p.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let with_dir = |p: &PathBuf| {
        let dir = p
            .parent()
            .and_then(|d| d.file_name())
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        format!("{dir}/{}", stem(p))
    };
    let unique = |labels: &[String]| {
        labels
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            == labels.len()
    };
    let mut labels: Vec<String> = files.iter().map(stem).collect();
    if !unique(&labels) {
        labels = files.iter().map(with_dir).collect();
    }
    if !unique(&labels) {
        labels = (0..files.len())
            .map(|i| char::from(b'A' + (i % 26) as u8).to_string())
            .collect();
    }
    let bytes = files
        .iter()
        .map(|p| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display())))
        .collect::<Result<Vec<_>, _>>()?;
    let pairs: Vec<(&str, &[u8])> = labels
        .iter()
        .map(String::as_str)
        .zip(bytes.iter().map(Vec::as_slice))
        .collect();
    print!("{}", jubarte::debug::diff::diff(&pairs, opts)?);
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Revisions { file, json }) => {
            return exit_code(run_revisions(&file, json));
        }
        Some(Command::Changes { file, json }) => {
            return exit_code(run_changes(&file, json));
        }
        Some(Command::Comments {
            file,
            json,
            author,
            latest,
        }) => {
            return exit_code(run_comments(&file, json, author.as_deref(), latest));
        }
        Some(Command::Accept {
            file,
            output,
            force,
            selection,
        }) => {
            return exit_code(run_resolution(
                &file,
                &output,
                force,
                &selection.filter(),
                jubarte::changes::accept_changes,
                "accept",
            ));
        }
        Some(Command::Reject {
            file,
            output,
            force,
            selection,
        }) => {
            return exit_code(run_resolution(
                &file,
                &output,
                force,
                &selection.filter(),
                jubarte::changes::reject_changes,
                "reject",
            ));
        }
        Some(Command::Convert {
            file,
            output,
            force,
            pdf,
            png,
            dpi,
            report,
            compress,
            font_report,
            revisions,
            revision_palette,
            markdown,
            pages: page_spec,
        }) => {
            let style = match revision_style(revisions, revision_palette.as_deref()) {
                Ok(style) => style,
                Err(e) => return exit_code(Err(e)),
            };
            let selected = match page_spec.as_deref().map(parse_pages).transpose() {
                Ok(selected) => selected,
                Err(e) => return exit_code(Err(e)),
            };
            let job = ConvertJob {
                file: &file,
                bytes: None,
                output: output.as_deref(),
                force,
                compress,
                font_report: font_report.as_deref(),
                revisions: style,
                pdf,
                png,
                dpi,
                report: report.as_deref(),
                pages: selected.as_deref(),
            };
            return exit_code(run_convert_any(&job, &markdown));
        }
        Some(Command::Diff {
            old,
            new,
            output,
            to,
            from,
            format,
            columns,
            force,
            author,
            date,
            mode,
            detail_threshold,
            reference_doc,
            critic,
            resource_path,
            revisions,
            revision_palette,
        }) => {
            let style = match revision_style(revisions, revision_palette.as_deref()) {
                Ok(style) => style,
                Err(e) => return exit_code(Err(e)),
            };
            return exit_code(run_diff(&DiffJob {
                old: &old,
                new: &new,
                output: output.as_deref(),
                to,
                from,
                force,
                settings: comparer_settings(
                    &author.unwrap_or_else(default_author),
                    &date.unwrap_or_else(jubarte::convert::utc_now_iso8601),
                    detail_threshold,
                    mode == CompareMode::Powertools,
                    false,
                ),
                reference: reference_doc.as_deref(),
                critic,
                resource_path: resource_path.as_deref(),
                revisions: style,
                patch: (format == PatchFormat::Patch).then_some(columns),
            }));
        }
        Some(Command::Inspect { file, json, tables }) => {
            return exit_code(if tables {
                run_inspect_tables(&file)
            } else {
                run_inspect(&file, json)
            });
        }
        Some(Command::Text { file }) => return exit_code(run_text(&file)),
        Some(Command::Edit {
            file,
            plan,
            out_dir,
            dry_run,
            force,
            pdf,
            png,
            dpi,
            revisions,
            revision_palette,
            quiet,
        }) => {
            let style = match revision_style(revisions, revision_palette.as_deref()) {
                Ok(style) => style,
                Err(e) => return exit_code(Err(e)),
            };
            return match run_edit(&EditJob {
                file: &file,
                plan: &plan,
                out_dir: &out_dir,
                dry_run,
                force,
                pdf,
                png,
                dpi,
                revisions: style,
                quiet,
            }) {
                Ok(()) => ExitCode::SUCCESS,
                Err((code, message)) => {
                    eprintln!("error: {message}");
                    ExitCode::from(code)
                }
            };
        }
        Some(Command::Capabilities { .. }) => {
            println!("{}", jubarte::capabilities::capabilities_json("cli"));
            return ExitCode::SUCCESS;
        }
        Some(Command::SelfUpdate {
            check,
            yes,
            version,
        }) => return exit_code(run_self_update(check, yes, version)),
        Some(Command::Debug {
            sub:
                Some(DebugCommand::Diff {
                    files,
                    part,
                    style,
                    para_text,
                    raw,
                    full,
                    limit,
                }),
            ..
        }) => {
            let opts = jubarte::debug::diff::DiffOptions {
                part,
                style,
                para_text,
                raw,
                full,
                limit,
            };
            return exit_code(run_debug_diff(&files, &opts));
        }
        Some(Command::DiffRender {
            a,
            b,
            dpi,
            out_dir,
            json,
            no_overlay,
            force,
        }) => {
            return match run_diff_render(&DiffRenderJob {
                a: &a,
                b: &b,
                out_dir: out_dir.as_deref(),
                dpi,
                json,
                overlay: !no_overlay,
                force,
            }) {
                Ok(false) => ExitCode::SUCCESS,
                // Like `edit`'s 3: a CI step can gate on a visual change.
                Ok(true) => ExitCode::from(5),
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            };
        }
        Some(Command::Debug {
            sub: None,
            files,
            list,
            checks,
            part,
            grep,
            limit,
            context,
        }) => {
            return exit_code(run_debug(&files, list, checks, part, grep, limit, context));
        }
        None => {}
    }
    let job = match cli.resolve() {
        Ok(job) => job,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("try 'jubarte --help'");
            return ExitCode::from(2);
        }
    };
    exit_code(run(&job))
}

#[cfg(feature = "self-update")]
fn run_self_update(check: bool, yes: bool, version: Option<String>) -> Result<(), String> {
    jubarte::update::run(&jubarte::update::Options {
        check,
        yes,
        version,
    })
}

#[cfg(not(feature = "self-update"))]
fn run_self_update(_check: bool, _yes: bool, _version: Option<String>) -> Result<(), String> {
    Err("this jubarte was built without the self-update feature; update it the way it was installed".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jubarte::convert::{MarkLines, RevisionStyle};

    #[test]
    fn self_update_parses_check_yes_and_a_pinned_version() {
        let cli = Cli::try_parse_from(["jubarte", "self-update", "--check"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::SelfUpdate {
                check: true,
                yes: false,
                version: None
            })
        ));
        let cli =
            Cli::try_parse_from(["jubarte", "self-update", "-y", "--version", "0.9.3"]).unwrap();
        let Some(Command::SelfUpdate {
            check,
            yes,
            version,
        }) = cli.command
        else {
            panic!("expected self-update");
        };
        assert!(!check && yes);
        assert_eq!(version.as_deref(), Some("0.9.3"));
    }

    #[test]
    fn convert_revisions_default_to_conventional_and_validate_the_palette() {
        let cli =
            Cli::try_parse_from(["jubarte", "convert", "in.docx", "--revisions", "word"]).unwrap();
        let Some(Command::Convert { revisions, .. }) = cli.command else {
            panic!("expected convert");
        };
        assert_eq!(revisions, Revisions::Word);
        assert_eq!(
            revision_style(Revisions::Conventional, None),
            Ok(RevisionStyle::Conventional)
        );
        assert!(revision_style(Revisions::Custom, None).is_err());
        assert!(revision_style(Revisions::Word, Some("deleted=#000000")).is_err());
        let Ok(RevisionStyle::Custom(p)) = revision_style(
            Revisions::Custom,
            Some("deleted=#112233:double-strike,moved-to=#00FF00:underline"),
        ) else {
            panic!("custom palette parses");
        };
        assert_eq!(p.deleted.color, [0x11, 0x22, 0x33]);
        assert_eq!(p.deleted.strike, MarkLines::Double);
        assert_eq!(p.moved_to.underline, MarkLines::Single);
        assert!(revision_style(Revisions::Custom, Some("deleted=red")).is_err());
    }
    use clap::CommandFactory;

    fn job_of(args: &[&str]) -> Job {
        Cli::try_parse_from(args)
            .expect("parse")
            .resolve()
            .expect("resolve")
    }

    /// clap's own invariants (catches derive-config mistakes like duplicate shorts).
    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn positional_args_and_default_output() {
        let j = job_of(&["jubarte", "a.docx", "b.docx"]);
        assert_eq!(j.original, PathBuf::from("a.docx"));
        assert_eq!(j.modified, PathBuf::from("b.docx"));
        assert_eq!(j.output, PathBuf::from("a_v_b.docx"));
        assert_eq!(j.author, "Redline");
        assert_eq!(j.date, "1970-01-01T00:00:00Z");
        assert!(!j.force && !j.quiet);
    }

    #[test]
    fn default_output_uses_original_directory_and_stems() {
        let o = default_output(
            Path::new("docs/contract.docx"),
            Path::new("rev/contract-2.docx"),
        );
        assert_eq!(o, PathBuf::from("docs/contract_v_contract-2.docx"));
        // no directory → bare name
        let o2 = default_output(Path::new("contract.docx"), Path::new("contract-2.docx"));
        assert_eq!(o2, PathBuf::from("contract_v_contract-2.docx"));
    }

    #[test]
    fn named_flags_override_positionals() {
        let j = job_of(&[
            "jubarte",
            "a.docx",
            "b.docx",
            "-b",
            "real-orig.docx",
            "--modified",
            "real-mod.docx",
        ]);
        assert_eq!(j.original, PathBuf::from("real-orig.docx"));
        assert_eq!(j.modified, PathBuf::from("real-mod.docx"));
    }

    #[test]
    fn all_options_long_and_short() {
        let j = job_of(&[
            "jubarte",
            "-b",
            "o.docx",
            "-m",
            "n.docx",
            "-o",
            "out.docx",
            "-a",
            "Jane Doe",
            "-d",
            "2024-01-02T00:00:00Z",
            "--force",
            "--quiet",
        ]);
        assert_eq!(j.output, PathBuf::from("out.docx"));
        assert_eq!(j.author, "Jane Doe");
        assert_eq!(j.date, "2024-01-02T00:00:00Z");
        assert!(j.force && j.quiet);
    }

    #[test]
    fn compare_mode_defaults_to_word_and_powertools_has_two_spellings() {
        assert!(!job_of(&["jubarte", "a.docx", "b.docx"]).powertools_faithful);
        assert!(!job_of(&["jubarte", "a.docx", "b.docx", "--mode", "word"]).powertools_faithful);
        assert!(
            job_of(&["jubarte", "a.docx", "b.docx", "--mode", "powertools"]).powertools_faithful
        );
        assert!(
            job_of(&["jubarte", "a.docx", "b.docx", "--powertools-faithful"]).powertools_faithful
        );
        assert!(
            Cli::try_parse_from(["jubarte", "a.docx", "b.docx", "--mode", "libreoffice"]).is_err()
        );
    }

    #[test]
    fn flags_can_supply_both_inputs_without_positionals() {
        let j = job_of(&["jubarte", "--original", "x.docx", "--modified", "y.docx"]);
        assert_eq!(j.original, PathBuf::from("x.docx"));
        assert_eq!(j.modified, PathBuf::from("y.docx"));
        assert_eq!(j.output, PathBuf::from("x_v_y.docx"));
    }

    #[test]
    fn double_dash_treats_rest_as_positionals() {
        let j = job_of(&["jubarte", "--", "-weird-name.docx", "b.docx"]);
        assert_eq!(j.original, PathBuf::from("-weird-name.docx"));
        assert_eq!(j.modified, PathBuf::from("b.docx"));
    }

    #[test]
    fn help_and_version_are_handled_by_clap() {
        use clap::error::ErrorKind;
        let help = Cli::try_parse_from(["jubarte", "--help"]).unwrap_err();
        assert_eq!(help.kind(), ErrorKind::DisplayHelp);
        let ver = Cli::try_parse_from(["jubarte", "-V"]).unwrap_err();
        assert_eq!(ver.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn missing_inputs_error_at_resolve() {
        let only_one = Cli::try_parse_from(["jubarte", "one.docx"])
            .unwrap()
            .resolve();
        assert!(only_one.unwrap_err().contains("missing MODIFIED"));
        let none = Cli::try_parse_from(["jubarte"]).unwrap().resolve();
        assert!(none.unwrap_err().contains("missing ORIGINAL"));
    }

    #[test]
    fn extra_positional_and_unknown_flag_rejected_by_clap() {
        use clap::error::ErrorKind;
        let extra = Cli::try_parse_from(["jubarte", "a.docx", "b.docx", "c.docx"]).unwrap_err();
        assert_eq!(extra.kind(), ErrorKind::UnknownArgument);
        let bogus = Cli::try_parse_from(["jubarte", "--bogus"]).unwrap_err();
        assert_eq!(bogus.kind(), ErrorKind::UnknownArgument);
        let missing_val = Cli::try_parse_from(["jubarte", "--author"]).unwrap_err();
        assert_eq!(missing_val.kind(), ErrorKind::InvalidValue);
    }

    /// D.6 — `redline revisions <file>` parses into `Command::Revisions` with
    /// `json` defaulting to `false`; the legacy positional/named compare
    /// fields are left at their defaults (`command` is a plain addition, not
    /// a replacement of the existing surface).
    #[test]
    fn revisions_subcommand_parses_with_default_json_false() {
        let cli = Cli::try_parse_from(["jubarte", "revisions", "file.docx"]).unwrap();
        match cli.command {
            Some(Command::Revisions { file, json }) => {
                assert_eq!(file, PathBuf::from("file.docx"));
                assert!(!json);
            }
            other => panic!("expected revisions subcommand, got {other:?}"),
        }
    }

    /// D.6 — `--json` sets the JSON-lines output flag.
    #[test]
    fn revisions_subcommand_json_flag_parses() {
        let cli = Cli::try_parse_from(["jubarte", "revisions", "file.docx", "--json"]).unwrap();
        match cli.command {
            Some(Command::Revisions { json, .. }) => assert!(json),
            other => panic!("expected revisions subcommand, got {other:?}"),
        }
    }

    /// D.6 — `revisions` without a FILE argument is a clap usage error.
    #[test]
    fn revisions_subcommand_missing_file_is_clap_error() {
        use clap::error::ErrorKind;
        let err = Cli::try_parse_from(["jubarte", "revisions"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    /// Prior behavior path: a two-positional invocation whose filenames do
    /// NOT collide with the subcommand name is unaffected by adding
    /// `command` to `Cli` — `cli.command` stays `None` and `resolve()`
    /// merges the positionals exactly as before this PR.
    #[test]
    fn plain_compare_positionals_leave_command_none() {
        let cli = Cli::try_parse_from(["jubarte", "a.docx", "b.docx"]).unwrap();
        assert!(cli.command.is_none());
        let job = cli.resolve().unwrap();
        assert_eq!(job.original, PathBuf::from("a.docx"));
        assert_eq!(job.modified, PathBuf::from("b.docx"));
    }

    /// `accept <file> -o <out> --force` parses into `Command::Accept` with the
    /// output and force flag captured.
    #[test]
    fn accept_subcommand_parses_file_output_and_force() {
        let cli =
            Cli::try_parse_from(["jubarte", "accept", "rl.docx", "-o", "out.docx", "--force"])
                .unwrap();
        match cli.command {
            Some(Command::Accept {
                file,
                output,
                force,
                selection,
            }) => {
                assert_eq!(
                    selection.filter(),
                    jubarte::changes::ChangeFilter::default()
                );
                assert_eq!(file, PathBuf::from("rl.docx"));
                assert_eq!(output, PathBuf::from("out.docx"));
                assert!(force);
            }
            other => panic!("expected accept subcommand, got {other:?}"),
        }
    }

    /// `reject <file> -o <out>` parses into `Command::Reject`; `force` defaults
    /// to false (the same no-clobber contract as `accept` / compare).
    #[test]
    fn reject_subcommand_parses_file_output_and_defaults_force_false() {
        let cli = Cli::try_parse_from(["jubarte", "reject", "rl.docx", "-o", "out.docx"]).unwrap();
        match cli.command {
            Some(Command::Reject {
                file,
                output,
                force,
                selection,
            }) => {
                assert_eq!(
                    selection.filter(),
                    jubarte::changes::ChangeFilter::default()
                );
                assert_eq!(file, PathBuf::from("rl.docx"));
                assert_eq!(output, PathBuf::from("out.docx"));
                assert!(!force);
            }
            other => panic!("expected reject subcommand, got {other:?}"),
        }
    }

    /// `--id`, `--author` and `--kind` repeat and select the changes to
    /// resolve; a flag left out constrains nothing.
    #[test]
    fn accept_selection_flags_build_the_change_filter() {
        use jubarte::changes::{ChangeFilter, ChangeKind};
        let cli = Cli::try_parse_from([
            "jubarte",
            "accept",
            "rl.docx",
            "-o",
            "out.docx",
            "--id",
            "body:rev:1",
            "--id",
            "header1:rev:2",
            "--kind",
            "move",
        ])
        .unwrap();
        let Some(Command::Accept { selection, .. }) = cli.command else {
            panic!("expected accept subcommand");
        };
        assert_eq!(
            selection.filter(),
            ChangeFilter {
                ids: Some(vec!["body:rev:1".into(), "header1:rev:2".into()]),
                authors: None,
                kinds: Some(vec![ChangeKind::Move]),
            }
        );
    }

    /// `reject` requires `-o/--output` (clap usage error when omitted), matching
    /// `accept`.
    #[test]
    fn reject_subcommand_requires_output() {
        use clap::error::ErrorKind;
        let err = Cli::try_parse_from(["jubarte", "reject", "rl.docx"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    /// Documents the one real interaction between the legacy compare surface
    /// and the new subcommand: a document literally named `revisions` as the
    /// first positional is parsed as the `revisions` subcommand (clap
    /// subcommand matching takes priority over positional args), not as the
    /// legacy ORIGINAL. This is the tradeoff for adding `revisions` as a
    /// subcommand rather than a flag.
    #[test]
    fn positional_named_revisions_is_parsed_as_subcommand() {
        let cli = Cli::try_parse_from(["jubarte", "revisions", "b.docx"]).unwrap();
        match cli.command {
            Some(Command::Revisions { file, .. }) => assert_eq!(file, PathBuf::from("b.docx")),
            other => panic!("expected revisions subcommand, got {other:?}"),
        }
    }

    #[test]
    fn convert_subcommand_parses_font_report() {
        let cli = Cli::try_parse_from([
            "jubarte",
            "convert",
            "in.docx",
            "--font-report",
            "out.json",
            "-o",
            "out.pdf",
        ])
        .unwrap();
        match cli.command {
            Some(Command::Convert {
                file,
                output,
                font_report,
                compress,
                force,
                revisions,
                revision_palette,
                pdf,
                png,
                dpi,
                report,
                ..
            }) => {
                assert_eq!(revisions, Revisions::Conventional);
                assert!(revision_palette.is_none());
                assert_eq!(file, PathBuf::from("in.docx"));
                assert_eq!(output.as_deref(), Some(Path::new("out.pdf")));
                assert_eq!(font_report.as_deref(), Some(Path::new("out.json")));
                assert!(!compress && !force);
                assert!(!pdf && !png && report.is_none());
                assert_eq!(dpi, 96.0);
            }
            other => panic!("expected convert subcommand, got {other:?}"),
        }
    }

    fn tiny_docx_bytes(family: &str) -> Vec<u8> {
        body_docx_bytes(&format!(
            r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="{family}" w:hAnsi="{family}"/></w:rPr><w:t>HELLO</w:t></w:r></w:p>"#
        ))
    }

    /// A Letter-size package whose body is `body` (WordprocessingML).
    fn body_docx_bytes(body: &str) -> Vec<u8> {
        use std::io::{Cursor, Write};
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opt = zip::write::SimpleFileOptions::default();
            z.start_file("[Content_Types].xml", opt).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
            )
            .unwrap();
            z.start_file("_rels/.rels", opt).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
            )
            .unwrap();
            z.start_file("word/document.xml", opt).unwrap();
            let doc = format!(
                r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#
            );
            z.write_all(doc.as_bytes()).unwrap();
            z.finish().unwrap();
        }
        buf
    }

    #[test]
    fn convert_rejects_a_font_report_path_equal_to_the_pdf() {
        let dir = tempfile::tempdir().expect("tempdir");
        let docx = dir.path().join("in.docx");
        let pdf = dir.path().join("out.pdf");
        std::fs::write(&docx, tiny_docx_bytes("Calibri")).expect("docx");
        let same = dir.path().join(".").join("out.pdf");
        let err = run_convert(&ConvertJob {
            file: &docx,
            bytes: None,
            output: Some(&pdf),
            force: false,
            compress: false,
            font_report: Some(&same),
            revisions: RevisionStyle::Word,
            pdf: false,
            png: false,
            dpi: 96.0,
            report: None,
            pages: None,
        })
        .expect_err("report over the PDF must be refused");
        assert!(err.contains("same file as the PDF output"), "{err}");
        assert!(!pdf.exists(), "nothing is written when the paths collide");
        let err = run_convert(&ConvertJob {
            file: &docx,
            bytes: None,
            output: Some(&pdf),
            force: false,
            compress: false,
            font_report: Some(&docx),
            revisions: RevisionStyle::Word,
            pdf: false,
            png: false,
            dpi: 96.0,
            report: None,
            pages: None,
        })
        .expect_err("report over the input must be refused");
        assert!(err.contains("same file as the input"), "{err}");
        assert!(std::fs::read(&docx).expect("docx").starts_with(b"PK"));
    }

    #[test]
    fn a_legacy_doc_is_refused_with_a_save_as_hint() {
        // `.doc` (Word 97-2003) and password-encrypted documents are OLE
        // compound files, not zips; without the hint compare called the file
        // Markdown that is not UTF-8.
        let dir = tempfile::tempdir().expect("tempdir");
        let (docx, doc, out) = (
            dir.path().join("a.docx"),
            dir.path().join("b.doc"),
            dir.path().join("r.docx"),
        );
        std::fs::write(&docx, tiny_docx_bytes("Calibri")).expect("docx");
        let mut ole = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1".to_vec();
        ole.resize(4096, 0);
        std::fs::write(&doc, ole).expect("doc");
        let args = [
            "jubarte",
            docx.to_str().unwrap(),
            doc.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ];
        let err = run(&job_of(&args)).expect_err("a .doc must be refused");
        assert!(
            err.contains("b.doc is a Word 97-2003 (.doc) or encrypted document"),
            "{err}"
        );
        assert!(err.contains("save it as .docx"), "{err}");
        assert!(!out.exists());
        let err = read_document(&doc).expect_err("every command reads through read_document");
        assert!(err.contains("save it as .docx"), "{err}");
    }

    #[test]
    fn convert_font_report_writes_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let docx = dir.path().join("in.docx");
        let pdf = dir.path().join("out.pdf");
        let report = dir.path().join("fonts.json");
        std::fs::write(&docx, tiny_docx_bytes("DefinitelyNotAFont")).expect("docx");
        run_convert(&ConvertJob {
            file: &docx,
            bytes: None,
            output: Some(&pdf),
            force: false,
            compress: false,
            font_report: Some(&report),
            revisions: RevisionStyle::Word,
            pdf: false,
            png: false,
            dpi: 96.0,
            report: None,
            pages: None,
        })
        .expect("convert");
        assert!(pdf.exists());
        let json = std::fs::read_to_string(&report).expect("report");
        let v: serde_json::Value = serde_json::from_str(&json).expect("json");
        let rows = v.as_array().expect("array");
        assert!(
            rows.iter().any(|row| {
                row.get("requested").and_then(|x| x.as_str()) == Some("DefinitelyNotAFont")
                    && row.get("step").and_then(|x| x.as_str()) == Some("unknown")
            }),
            "report missing unknown family: {json}"
        );
    }

    mod regression_tests {
        use super::*;

        #[test]
        fn convert_without_revision_flags_uses_conventional_marks() {
            let cli = Cli::try_parse_from(["jubarte", "convert", "in.docx"]).unwrap();
            let Some(Command::Convert {
                revisions,
                revision_palette,
                ..
            }) = cli.command
            else {
                panic!("convert")
            };
            assert_eq!(revisions, Revisions::Conventional);
            assert_eq!(revision_palette, None);
            assert_eq!(
                revision_style(revisions, None),
                Ok(RevisionStyle::Conventional)
            );
        }

        #[test]
        fn custom_revision_flags_reach_the_palette_parser() {
            let spec = "inserted=#aabbcc:plain";
            let cli = Cli::try_parse_from([
                "jubarte",
                "convert",
                "in.docx",
                "--revisions",
                "custom",
                "--revision-palette",
                spec,
            ])
            .unwrap();
            let Some(Command::Convert {
                revisions,
                revision_palette,
                ..
            }) = cli.command
            else {
                panic!("convert")
            };
            assert_eq!(revisions, Revisions::Custom);
            assert_eq!(revision_palette.as_deref(), Some(spec));
            assert_eq!(
                revision_style(revisions, revision_palette.as_deref()),
                RevisionStyle::from_choice("custom", Some(spec))
            );
        }

        #[test]
        fn invalid_revision_flags_are_rejected_with_option_context() {
            assert!(
                Cli::try_parse_from(["jubarte", "convert", "in.docx", "--revisions", "unknown"])
                    .is_err()
            );
            assert_eq!(
                revision_style(Revisions::Custom, None).unwrap_err(),
                "--revisions custom needs --revision-palette"
            );
            for mode in [Revisions::Conventional, Revisions::Word] {
                assert_eq!(
                    revision_style(mode, Some("deleted=#000000")).unwrap_err(),
                    "--revision-palette needs --revisions custom"
                );
            }
            assert!(
                revision_style(Revisions::Custom, Some("deleted=red"))
                    .unwrap_err()
                    .starts_with("--revision-palette:")
            );
        }
    }

    const PAGE_BREAK: &str = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;

    fn pages_docx(texts: &[&str]) -> Vec<u8> {
        let body: Vec<String> = texts
            .iter()
            .map(|t| format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>"))
            .collect();
        body_docx_bytes(&body.join(PAGE_BREAK))
    }

    fn file_names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("read_dir")
            .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn parse_pages_counts_from_one_and_sorts_without_repeats() {
        assert_eq!(parse_pages("1-3,7"), Ok(vec![0, 1, 2, 6]));
        assert_eq!(parse_pages("7, 2-3 ,2"), Ok(vec![1, 2, 6]));
        assert_eq!(parse_pages("4"), Ok(vec![3]));
        assert_eq!(parse_pages("5-5"), Ok(vec![4]));
    }

    #[test]
    fn parse_pages_refuses_zero_empty_backwards_and_words() {
        for (spec, why) in [
            ("0", "counted from 1"),
            ("2-0", "counted from 1"),
            ("", "empty item"),
            ("1,,2", "empty item"),
            ("3-1", "runs backwards"),
            ("two", "not a page number"),
            ("1-x", "not a page number"),
            ("-2", "not a page number"),
        ] {
            let err = parse_pages(spec).expect_err(spec);
            assert!(err.contains(why), "{spec}: {err}");
        }
    }

    #[test]
    fn convert_and_diff_render_parse_their_page_flags() {
        let cli = Cli::try_parse_from(["jubarte", "convert", "a.docx", "--png", "--pages", "2-3"])
            .expect("parse");
        let Some(Command::Convert { pages, png, .. }) = cli.command else {
            panic!("expected convert");
        };
        assert_eq!(pages.as_deref(), Some("2-3"));
        assert!(png);
        let cli = Cli::try_parse_from([
            "jubarte",
            "diff-render",
            "a.docx",
            "b.docx",
            "--dpi",
            "72",
            "--out-dir",
            "d",
            "--json",
            "--no-overlay",
            "--force",
        ])
        .expect("parse");
        let Some(Command::DiffRender {
            a,
            b,
            dpi,
            out_dir,
            json,
            no_overlay,
            force,
        }) = cli.command
        else {
            panic!("expected diff-render");
        };
        assert_eq!(
            (a.as_path(), b.as_path()),
            (Path::new("a.docx"), Path::new("b.docx"))
        );
        assert_eq!(dpi, 72.0);
        assert_eq!(out_dir.as_deref(), Some(Path::new("d")));
        assert!(json && no_overlay && force);
        let cli =
            Cli::try_parse_from(["jubarte", "diff-render", "a.docx", "b.docx"]).expect("parse");
        let Some(Command::DiffRender {
            dpi,
            out_dir,
            json,
            no_overlay,
            force,
            ..
        }) = cli.command
        else {
            panic!("expected diff-render");
        };
        assert_eq!(dpi, 100.0);
        assert!(out_dir.is_none() && !json && !no_overlay && !force);
    }

    fn convert_job<'a>(
        file: &'a Path,
        output: &'a Path,
        pages: Option<&'a [usize]>,
    ) -> ConvertJob<'a> {
        ConvertJob {
            file,
            bytes: None,
            output: Some(output),
            force: false,
            compress: false,
            font_report: None,
            revisions: RevisionStyle::Conventional,
            pdf: false,
            png: true,
            dpi: 20.0,
            report: None,
            pages,
        }
    }

    #[test]
    fn convert_pages_writes_the_selected_pages_named_by_page_number() {
        let dir = tempfile::tempdir().expect("tempdir");
        let docx = dir.path().join("in.docx");
        std::fs::write(&docx, pages_docx(&["A", "B", "C"])).expect("docx");
        let out = dir.path().join("out.pdf");
        run_convert(&convert_job(&docx, &out, Some(&[2, 0]))).expect("convert");
        assert_eq!(
            file_names(dir.path()),
            ["in.docx", "out-page-01.png", "out-page-03.png"]
        );
        let page_three = std::fs::read(dir.path().join("out-page-03.png")).expect("png");
        let all = jubarte::convert::docx_to_png(
            &pages_docx(&["A", "B", "C"]),
            jubarte::convert::PdfOptions::default(),
            20.0,
        )
        .expect("png");
        assert_eq!(
            page_three, all[2],
            "a selected page is the same bytes as in a full render"
        );
    }

    #[test]
    fn convert_pages_needs_png_and_an_existing_page() {
        let dir = tempfile::tempdir().expect("tempdir");
        let docx = dir.path().join("in.docx");
        std::fs::write(&docx, pages_docx(&["A", "B", "C"])).expect("docx");
        let out = dir.path().join("out.pdf");
        let err = run_convert(&ConvertJob {
            png: false,
            ..convert_job(&docx, &out, Some(&[0]))
        })
        .expect_err("--pages without PNG output");
        assert!(err.contains("--pages"), "{err}");
        let err = run_convert(&convert_job(&docx, &out, Some(&[5]))).expect_err("page 6 of 3");
        assert!(
            err.contains("page 6 is out of range: the document has 3 pages"),
            "{err}"
        );
        assert_eq!(file_names(dir.path()), ["in.docx"], "nothing written");
    }

    fn diff_job<'a>(a: &'a Path, b: &'a Path, out_dir: Option<&'a Path>) -> DiffRenderJob<'a> {
        DiffRenderJob {
            a,
            b,
            out_dir,
            dpi: 20.0,
            json: false,
            overlay: true,
            force: false,
        }
    }

    #[test]
    fn diff_render_writes_only_the_changed_pages_and_diff_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let a = dir.path().join("a.docx");
        let b = dir.path().join("b.docx");
        std::fs::write(&a, pages_docx(&["Page one.", "The fee is ten."])).expect("a");
        std::fs::write(
            &b,
            pages_docx(&["Page one.", "The fee is twenty.", "Extra."]),
        )
        .expect("b");
        let out = dir.path().join("diff");
        assert_eq!(run_diff_render(&diff_job(&a, &b, Some(&out))), Ok(true));
        assert_eq!(
            file_names(&out),
            [
                "a-page-02.png",
                "b-page-02.png",
                "b-page-03.png",
                "diff-page-02.png",
                "diff.json"
            ]
        );
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(out.join("diff.json")).expect("json"))
                .expect("parse");
        assert_eq!(json["dpi"], 20.0);
        assert_eq!(json["a_pages"], 2);
        assert_eq!(json["b_pages"], 3);
        assert_eq!(json["changed"], 2);
        assert_eq!(json["pages"][0]["changed_ratio"], 0.0);
        assert!(json["pages"][1]["changed_ratio"].as_f64().expect("ratio") > 0.0);
        let raw = std::fs::read_to_string(out.join("diff.json")).expect("json");
        let ratio = raw
            .split("\"changed_ratio\":")
            .nth(2)
            .expect("page 2's ratio");
        let ratio = &ratio[..ratio.find(',').expect("comma")];
        let shortest = ratio.parse::<f32>().expect("f32").to_string();
        assert_eq!(
            ratio, shortest,
            "an f32 printed at f32 precision, not widened"
        );
        assert_eq!(json["pages"][2]["only_in"], "b");

        let err = run_diff_render(&diff_job(&a, &b, Some(&out))).expect_err("files exist");
        assert!(err.contains("already exists"), "{err}");
        let again = DiffRenderJob {
            force: true,
            overlay: false,
            json: true,
            ..diff_job(&a, &b, Some(&out))
        };
        assert_eq!(run_diff_render(&again), Ok(true));
    }

    #[test]
    fn diff_render_of_equal_documents_writes_only_diff_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let a = dir.path().join("a.docx");
        std::fs::write(&a, pages_docx(&["Same."])).expect("a");
        let out = dir.path().join("diff");
        assert_eq!(run_diff_render(&diff_job(&a, &a, Some(&out))), Ok(false));
        assert_eq!(file_names(&out), ["diff.json"]);
        assert_eq!(run_diff_render(&diff_job(&a, &a, None)), Ok(false));
    }

    #[test]
    fn diff_render_without_an_overlay_writes_no_diff_png() {
        let dir = tempfile::tempdir().expect("tempdir");
        let a = dir.path().join("a.docx");
        let b = dir.path().join("b.docx");
        std::fs::write(&a, pages_docx(&["The fee is ten."])).expect("a");
        std::fs::write(&b, pages_docx(&["The fee is twenty."])).expect("b");
        let out = dir.path().join("diff");
        let job = DiffRenderJob {
            overlay: false,
            ..diff_job(&a, &b, Some(&out))
        };
        assert_eq!(run_diff_render(&job), Ok(true));
        assert_eq!(
            file_names(&out),
            ["a-page-01.png", "b-page-01.png", "diff.json"]
        );
        let missing = dir.path().join("missing.docx");
        let err = run_diff_render(&diff_job(&missing, &b, None)).expect_err("no file");
        assert!(err.contains("missing.docx"), "{err}");
    }
}
