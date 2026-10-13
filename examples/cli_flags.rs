// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Prints every flag of the `jubarte` command line as Markdown, read from
//! clap's own model rather than from `--help` text, so hidden flags, hidden
//! aliases, defaults, possible values and conflicts are all listed.
//! `scripts/cli_flags.sh` runs it and writes `docs/flags_datetime.md`.
//!
//! Sections: one table per task (hidden tasks included), then the flags by
//! name across tasks (one long name with different shorts, values or
//! defaults; one short letter for different long names), the timestamp
//! flags, and the retired spellings.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use clap::{Arg, Command};

/// One option or positional of one task, flattened for the tables.
struct Row {
    task: String,
    short: Option<char>,
    long: Option<String>,
    id: String,
    aliases: Vec<String>,
    value: String,
    default: String,
    possible: String,
    notes: Vec<String>,
    help: String,
}

/// `text` on one Markdown table line.
fn cell(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}

/// What `arg` takes after it: `<FILE>`, `[<PLAN.json>]`, `<SPEC>...`, or
/// nothing for a flag.
fn value_shape(arg: &Arg) -> String {
    let Some(range) = arg.get_num_args() else {
        return String::new();
    };
    if !range.takes_values() {
        return String::new();
    }
    let names = arg
        .get_value_names()
        .map(|names| {
            names
                .iter()
                .map(|n| format!("<{n}>"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_else(|| format!("<{}>", arg.get_id().as_str().to_uppercase()));
    let shaped = if range.min_values() == 0 {
        format!("[{names}]")
    } else {
        names
    };
    if range.max_values() > 1 {
        format!("{shaped}...")
    } else {
        shaped
    }
}

fn row(task: &str, cmd: &Command, arg: &Arg) -> Row {
    let mut aliases = Vec::new();
    let visible = arg.get_visible_aliases().unwrap_or_default();
    for alias in arg.get_all_aliases().unwrap_or_default() {
        let hidden = if visible.contains(&alias) {
            ""
        } else {
            " (hidden)"
        };
        aliases.push(format!("--{alias}{hidden}"));
    }
    let visible_short = arg.get_visible_short_aliases().unwrap_or_default();
    for alias in arg.get_all_short_aliases().unwrap_or_default() {
        let hidden = if visible_short.contains(&alias) {
            ""
        } else {
            " (hidden)"
        };
        aliases.push(format!("-{alias}{hidden}"));
    }
    let mut notes = Vec::new();
    if arg.is_hide_set() {
        notes.push("hidden".to_string());
    }
    if arg.is_global_set() {
        notes.push("global".to_string());
    }
    if arg.is_required_set() {
        notes.push("required".to_string());
    }
    if matches!(
        arg.get_action(),
        clap::ArgAction::Append | clap::ArgAction::Count
    ) {
        notes.push(format!("{:?}", arg.get_action()).to_lowercase());
    }
    if let Some(heading) = arg.get_help_heading() {
        notes.push(format!("under \"{heading}\""));
    }
    let conflicts: Vec<String> = cmd
        .get_arg_conflicts_with(arg)
        .iter()
        .map(|other| match other.get_long() {
            Some(long) => format!("--{long}"),
            None => other.get_id().as_str().to_string(),
        })
        .collect();
    if !conflicts.is_empty() {
        notes.push(format!("conflicts {}", conflicts.join(" ")));
    }
    let default = arg
        .get_default_values()
        .iter()
        .map(|v| v.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(",");
    let possible = arg
        .get_possible_values()
        .iter()
        .filter(|v| !v.is_hide_set())
        .map(|v| v.get_name_and_aliases().collect::<Vec<_>>().join("="))
        .collect::<Vec<_>>()
        .join(", ");
    let help = arg
        .get_long_help()
        .or_else(|| arg.get_help())
        .map(|h| h.to_string())
        .unwrap_or_default();
    Row {
        task: task.to_string(),
        short: arg.get_short(),
        long: arg.get_long().map(str::to_string),
        id: arg.get_id().as_str().to_string(),
        aliases,
        value: value_shape(arg),
        default,
        possible,
        notes,
        help,
    }
}

/// The flag as written: `-p, --location`, or `<FILE>` for a positional.
fn spelled(row: &Row) -> String {
    match (row.short, &row.long) {
        (Some(s), Some(l)) => format!("-{s}, --{l}"),
        (Some(s), None) => format!("-{s}"),
        (None, Some(l)) => format!("--{l}"),
        (None, None) => format!("<{}>", row.id.to_uppercase()),
    }
}

/// Every task, depth first, as `(path, command)`; the generated `help`
/// subcommand is left out.
fn tasks<'a>(path: &str, cmd: &'a Command, out: &mut Vec<(String, &'a Command)>) {
    out.push((path.to_string(), cmd));
    for sub in cmd.get_subcommands() {
        if sub.get_name() == "help" {
            continue;
        }
        let child = if path.is_empty() {
            sub.get_name().to_string()
        } else {
            format!("{path} {}", sub.get_name())
        };
        tasks(&child, sub, out);
    }
}

fn main() {
    let mut root = jubarte::cli::top_level_command();
    root.build();
    let mut all = Vec::new();
    tasks("", &root, &mut all);

    let mut out = String::new();
    let mut rows: Vec<Row> = Vec::new();
    for (path, cmd) in &all {
        let title = if path.is_empty() {
            "jubarte (no task: the compare/read shorthand)".to_string()
        } else {
            format!("jubarte {path}")
        };
        let mut flags = Vec::new();
        if cmd.is_hide_set() {
            flags.push("hidden task".to_string());
        }
        let aliases: Vec<&str> = cmd.get_all_aliases().collect();
        if !aliases.is_empty() {
            flags.push(format!("aliases: {}", aliases.join(", ")));
        }
        let suffix = if flags.is_empty() {
            String::new()
        } else {
            format!(" ({})", flags.join("; "))
        };
        let _ = writeln!(out, "## {title}{suffix}\n");
        if let Some(about) = cmd.get_about() {
            let _ = writeln!(out, "{}\n", cell(&about.to_string()));
        }
        let _ = writeln!(
            out,
            "| Flag | Aliases | Value | Default | Possible values | Notes | Help |"
        );
        let _ = writeln!(out, "|-|-|-|-|-|-|-|");
        for arg in cmd.get_arguments() {
            let id = arg.get_id().as_str();
            if id == "help" || id == "version" {
                continue;
            }
            let r = row(path, cmd, arg);
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} | {} | {} | {} |",
                spelled(&r),
                cell(&r.aliases.join(", ")),
                cell(&r.value),
                cell(&r.default),
                cell(&r.possible),
                cell(&r.notes.join("; ")),
                cell(&r.help),
            );
            rows.push(r);
        }
        out.push('\n');
    }

    // One long name across tasks: the shorts, values and defaults it takes.
    let mut by_long: BTreeMap<&str, Vec<&Row>> = BTreeMap::new();
    for r in &rows {
        if let Some(long) = &r.long {
            by_long.entry(long).or_default().push(r);
        }
    }
    let _ = writeln!(out, "## Flags by long name\n");
    let _ = writeln!(
        out,
        "A long name whose short, value, default or possible values differ between tasks is marked **differs**.\n"
    );
    let _ = writeln!(out, "| Flag | Tasks | Shapes |");
    let _ = writeln!(out, "|-|-|-|");
    for (long, uses) in &by_long {
        let mut shapes: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for r in uses {
            let short = r.short.map(|s| format!("-{s} ")).unwrap_or_default();
            let shape = format!(
                "{short}{} default={} values={}",
                r.value, r.default, r.possible
            );
            shapes.entry(shape).or_default().push(display_task(&r.task));
        }
        let tasks: Vec<&str> = uses.iter().map(|r| display_task(&r.task)).collect();
        let shape_text = if shapes.len() > 1 {
            let listed: Vec<String> = shapes
                .iter()
                .map(|(shape, on)| format!("`{}` on {}", shape.trim(), on.join(", ")))
                .collect();
            format!("**differs**: {}", listed.join("; "))
        } else {
            shapes
                .keys()
                .next()
                .map(|s| format!("`{}`", s.trim()))
                .unwrap_or_default()
        };
        let _ = writeln!(
            out,
            "| `--{long}` | {} | {} |",
            tasks.join(", "),
            cell(&shape_text)
        );
    }
    out.push('\n');

    // One short letter across tasks: the long names it stands for.
    let mut by_short: BTreeMap<char, BTreeMap<String, Vec<&str>>> = BTreeMap::new();
    for r in &rows {
        if let Some(short) = r.short {
            let long = r
                .long
                .clone()
                .map(|l| format!("--{l}"))
                .unwrap_or_else(|| "(no long name)".to_string());
            by_short
                .entry(short)
                .or_default()
                .entry(long)
                .or_default()
                .push(display_task(&r.task));
        }
    }
    let _ = writeln!(out, "## Short flags\n");
    let _ = writeln!(
        out,
        "A letter that stands for more than one long name is marked **several**.\n"
    );
    let _ = writeln!(out, "| Short | Means |");
    let _ = writeln!(out, "|-|-|");
    for (short, meanings) in &by_short {
        let listed: Vec<String> = meanings
            .iter()
            .map(|(long, on)| format!("`{long}` on {}", on.join(", ")))
            .collect();
        let mark = if meanings.len() > 1 {
            "**several**: "
        } else {
            ""
        };
        let _ = writeln!(out, "| `-{short}` | {mark}{} |", listed.join("; "));
    }
    out.push('\n');

    // Timestamp flags (the `-d, --datetime` rule): every flag whose name,
    // alias or help speaks of a date or a time.
    let _ = writeln!(out, "## Timestamp flags\n");
    let _ = writeln!(out, "| Task | Flag | Aliases | Help |");
    let _ = writeln!(out, "|-|-|-|-|");
    for r in &rows {
        let words = format!(
            "{} {} {}",
            r.long.as_deref().unwrap_or(""),
            r.aliases.join(" "),
            r.help
        )
        .to_lowercase();
        if ["date", "time", "clock", "now"].iter().any(|w| {
            words
                .split(|c: char| !c.is_alphanumeric())
                .any(|t| t.starts_with(w))
        }) {
            let _ = writeln!(
                out,
                "| {} | `{}` | {} | {} |",
                display_task(&r.task),
                spelled(r),
                cell(&r.aliases.join(", ")),
                cell(&r.help)
            );
        }
    }
    out.push('\n');

    let _ = writeln!(out, "## Retired spellings\n");
    let _ = writeln!(
        out,
        "Refused with a pointer to the new spelling (`jubarte::cli::RETIRED`).\n"
    );
    let _ = writeln!(out, "| Retired | Say instead | On |");
    let _ = writeln!(out, "|-|-|-|");
    for retired in jubarte::cli::RETIRED {
        let _ = writeln!(
            out,
            "| `{}` | `{}` | {} |",
            retired.old,
            retired.now,
            retired.on.unwrap_or("(a task name)")
        );
    }
    print!("{out}");
}

/// The task path as the tables name it.
fn display_task(task: &str) -> &str {
    if task.is_empty() { "(shorthand)" } else { task }
}
