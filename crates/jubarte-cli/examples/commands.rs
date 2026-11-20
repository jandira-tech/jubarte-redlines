// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Walk the `jubarte` command line's clap model: every task and what it
//! does, and parse an argument list without running anything.
//! Run: `cargo run -p jubarte-cli --example commands`

// ANCHOR: example
use jubarte_cli::cli::top_level_command;

fn main() -> Result<(), clap::Error> {
    let command = top_level_command();
    for task in command.get_subcommands().filter(|t| !t.is_hide_set()) {
        let about = task
            .get_about()
            .map(ToString::to_string)
            .unwrap_or_default();
        println!("{:<14} {about}", task.get_name());
    }

    // Parsing checks the flags exactly as the binary does.
    let matches = top_level_command().try_get_matches_from([
        "jubarte",
        "compare",
        "original.docx",
        "modified.docx",
        "--author",
        "Reviewer",
    ])?;
    println!("parsed task: {:?}", matches.subcommand_name());
    Ok(())
}
// ANCHOR_END: example
