//! `jubarte` — generate a tracked-changes (redline) `.docx` from two documents.
//!
//! ```text
//! jubarte original.docx modified.docx
//!   → writes original_v_modified.docx
//! jubarte -b a.docx -m b.docx -o out.docx --author "Jane" --date 2024-01-02T00:00:00Z
//! ```
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
        jubarte a.docx b.docx --force --quiet",
)]
struct Cli {
    /// Subcommand (e.g. `revisions`); plain compare when omitted.
    #[command(subcommand)]
    command: Option<Command>,

    /// The original / base document (.docx).
    #[arg(value_name = "ORIGINAL")]
    original_pos: Option<PathBuf>,

    /// The modified document (.docx).
    #[arg(value_name = "MODIFIED")]
    modified_pos: Option<PathBuf>,

    /// Original/base document (overrides the positional ORIGINAL).
    #[arg(short = 'b', long = "original", value_name = "FILE")]
    original: Option<PathBuf>,

    /// Modified document (overrides the positional MODIFIED).
    #[arg(short = 'm', long = "modified", value_name = "FILE")]
    modified: Option<PathBuf>,

    /// Output path [default: <original-dir>/<original>_v_<modified>.docx].
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
    /// --powertools-faithful]. 0.02 = Word-style within-paragraph word diffs
    /// with weak-match voiding; 0.15 = the PowerTools-faithful coarse
    /// fallback; 0 = confetti with no voiding. An explicit value always wins
    /// over either preset (Option distinguishes unset from explicitly-set —
    /// no sentinel ambiguity).
    #[arg(long, value_name = "RATIO")]
    detail_threshold: Option<f64>,

    /// PowerTools-faithful mode: coarse paragraph fallback (threshold 0.15)
    /// and no Word-visual alignment passes. Default is Word-visual mode.
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
    /// Accept every tracked revision (package-wide) and write the result.
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
    },
}

fn run_revisions(file: &Path, json: bool) -> Result<(), String> {
    let bytes = std::fs::read(file).map_err(|e| format!("reading {}: {e}", file.display()))?;
    let settings = jubarte::comparer::WmlComparerSettings::default();
    let revs = jubarte::document_comparer::get_revisions(&bytes, &settings)
        .map_err(|e| format!("get_revisions failed: {e:?}"))?;
    if json {
        for r in &revs {
            // full JSON string escaping: backslash, quote, and ALL control
            // chars < 0x20 (document text can carry \t, \r, vertical tabs…)
            let esc = |s: &str| {
                let mut o = String::with_capacity(s.len());
                for c in s.chars() {
                    match c {
                        '\\' => o.push_str("\\\\"),
                        '"' => o.push_str("\\\""),
                        '\n' => o.push_str("\\n"),
                        '\r' => o.push_str("\\r"),
                        '\t' => o.push_str("\\t"),
                        c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
                        c => o.push(c),
                    }
                }
                o
            };
            let format_change = r.format_change.as_ref().map_or("null".to_string(), |fc| {
                let props: Vec<String> = fc
                    .changed_properties
                    .iter()
                    .map(|p| format!("\"{}\"", esc(p)))
                    .collect();
                format!("{{\"changedProperties\":[{}]}}", props.join(","))
            });
            println!(
                "{{\"type\":\"{:?}\",\"author\":\"{}\",\"date\":\"{}\",\"part\":\"{}\",\"moveGroupId\":{},\"isMoveSource\":{},\"formatChange\":{},\"text\":\"{}\"}}",
                r.revision_type,
                esc(r.author.as_deref().unwrap_or("")),
                esc(r.date.as_deref().unwrap_or("")),
                esc(&r.part_name),
                r.move_group_id
                    .map_or("null".to_string(), |v| v.to_string()),
                r.is_move_source
                    .map_or("null".to_string(), |v| v.to_string()),
                format_change,
                esc(r.text.as_deref().unwrap_or("")),
            );
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
            powertools_faithful: self.powertools_faithful,
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

fn run(job: &Job) -> Result<(), String> {
    if job.output.exists() && !job.force {
        return Err(format!(
            "output '{}' already exists (use --force to overwrite)",
            job.output.display()
        ));
    }
    let original = std::fs::read(&job.original)
        .map_err(|e| format!("reading {}: {e}", job.original.display()))?;
    let modified = std::fs::read(&job.modified)
        .map_err(|e| format!("reading {}: {e}", job.modified.display()))?;

    let base = if job.powertools_faithful {
        jubarte::comparer::WmlComparerSettings::powertools_faithful()
    } else {
        jubarte::comparer::WmlComparerSettings::default()
    };
    let settings = jubarte::comparer::WmlComparerSettings {
        author_for_revisions: job.author.clone(),
        date_time_for_revisions: job.date.clone(),
        detail_threshold: job.detail_threshold.unwrap_or(base.detail_threshold),
        merge_replaced_paragraphs: if job.no_paragraph_merge {
            false
        } else {
            base.merge_replaced_paragraphs
        },
        ..base
    };
    let out = jubarte::document_comparer::compare_documents_with_settings(
        &original, &modified, &settings,
    )
    .map_err(|e| format!("compare failed: {e:?}"))?;

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

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Revisions { file, json }) => {
            return exit_code(run_revisions(&file, json));
        }
        Some(Command::Accept {
            file,
            output,
            force,
        }) => {
            // same no-clobber contract as the compare path (PR #54 review)
            let r = if output.exists() && !force {
                Err(format!(
                    "output '{}' already exists (use --force to overwrite)",
                    output.display()
                ))
            } else {
                std::fs::read(&file).map_err(|e| format!("reading {}: {e}", file.display()))
            }
            .and_then(|bytes| {
                jubarte::document_comparer::accept_revisions(&bytes)
                    .map_err(|e| format!("accept failed: {e:?}"))
            })
            .and_then(|out| {
                std::fs::write(&output, &out)
                    .map_err(|e| format!("writing {}: {e}", output.display()))
            });
            return exit_code(r);
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
