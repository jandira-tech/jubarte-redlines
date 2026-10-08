// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Deterministic shared CLI parsing, with no document or platform I/O.
#![cfg(feature = "cli")]

use jubarte::cli::{Cli, Command, PatchFormat, parse_json};
use serde_json::Value;

fn parse(arguments: &[&str], supported: &[&str]) -> Value {
    serde_json::from_str(&parse_json(
        &arguments.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "jubarte-python",
        &supported.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    ))
    .unwrap()
}

#[test]
fn explicit_compare_alias_and_shorthand_preserve_overrides() {
    for prefix in [vec![], vec!["compare"], vec!["redline"]] {
        let args = [prefix, vec!["a.docx", "b.docx", "-b", "real.docx"]].concat();
        let result = parse(&args, &[]);
        assert_eq!(result["exit_code"], 0, "{result}");
        assert_eq!(result["command"], "compare");
        assert_eq!(result["args"]["original"], "real.docx");
        assert_eq!(result["args"]["modified"], "b.docx");
        assert_eq!(result["args"]["author"], "Redline");
        assert_eq!(result["args"]["mode"], "word");
    }
    assert_eq!(
        parse(&["compare", "-b", "a", "-m", "b"], &[])["exit_code"],
        0
    );
    for args in [
        &[][..],
        &["a.docx"][..],
        &["compare"][..],
        &["compare", "-b", "a"][..],
    ] {
        assert_eq!(parse(args, &[])["exit_code"], 2);
    }
}

#[test]
fn github_aliases_context_and_native_types() {
    for name in ["github", "unified", "text"] {
        let result = parse(&["diff", "a.docx", "b.md", "--format", name], &[]);
        assert_eq!(result["command"], "diff", "{result}");
        assert_eq!(result["args"]["format"], "github");
        assert_eq!(result["args"]["context"], 3);
        assert!(result["args"]["output"].is_null());
        let cli = Cli::try_parse_from(["jubarte", "diff", "a", "b", "--format", name]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Diff {
                format: PatchFormat::Github,
                ..
            })
        ));
    }
    assert_eq!(
        parse(
            &["diff", "a", "b", "--format", "github", "--context", "0"],
            &[]
        )["args"]["context"],
        0
    );
    for args in [
        vec!["diff", "a", "b", "--context", "3"],
        vec!["diff", "a", "b", "--format", "critic", "--context", "2"],
        vec!["diff", "a", "b", "--format", "github", "--context", "-1"],
    ] {
        assert_eq!(parse(&args, &[])["exit_code"], 2);
    }
}

#[test]
fn github_output_contract_is_checked_before_documents_exist() {
    for extension in ["patch", "diff", "txt"] {
        let path = format!("changes.{extension}");
        assert_eq!(
            parse(
                &[
                    "diff",
                    "missing-a",
                    "missing-b",
                    "--format",
                    "github",
                    "-o",
                    &path
                ],
                &[]
            )["exit_code"],
            0
        );
    }
    for extra in [
        vec!["--to", "docx"],
        vec!["--to", "pdf"],
        vec!["--to", "png"],
        vec!["-o", "out.docx"],
        vec!["-o", "out.pdf"],
        vec!["-o", "out.png"],
        vec!["--to", "md", "-o", "out.docx"],
    ] {
        let args = [
            vec!["diff", "missing-a", "missing-b", "--format", "github"],
            extra,
        ]
        .concat();
        let result = parse(&args, &[]);
        assert_eq!(result["exit_code"], 2, "{result}");
        assert_eq!(result["stream"], "stderr");
        assert!(result["text"].as_str().unwrap().contains("Usage:"));
    }
}

#[test]
fn facade_restricts_help_acceptance_aliases_and_shorthand() {
    let supported = ["inspect", "diff"];
    let help = parse(&["--help"], &supported);
    assert_eq!(help["exit_code"], 0);
    assert_eq!(help["stream"], "stdout");
    let text = help["text"].as_str().unwrap();
    assert!(text.contains("jubarte-python"));
    assert!(text.contains("inspect") && text.contains("diff"));
    assert!(!text.contains("\n  self-update ") && !text.contains("\n  compare "));
    assert!(!text.contains('\u{1b}'));
    for args in [
        &["convert", "a.docx"][..],
        &["redline", "a", "b"][..],
        &["a", "b"][..],
        &["help", "self-update"][..],
    ] {
        assert_eq!(parse(args, &supported)["exit_code"], 2);
    }
    assert_eq!(
        parse(&["redline", "a", "b"], &["compare"])["command"],
        "compare"
    );
    assert_eq!(parse(&["--version"], &supported)["stream"], "stdout");
    assert_eq!(
        parse(&["inspect", "a", "--json"], &supported)["args"]["json"],
        true
    );
}

#[test]
fn facade_serializes_native_names_canonical_enums_and_nested_subcommands() {
    let convert = parse(
        &[
            "convert", "draft.md", "--from", "markdown", "--to", "png", "--dpi", "72", "--pages",
            "1-3,7",
        ],
        &[],
    );
    assert_eq!(convert["args"]["from"], "md");
    assert_eq!(convert["args"]["dpi"], 72.0);
    assert_eq!(convert["args"]["track_changes"], "all");
    assert_eq!(convert["args"]["pages"], "1-3,7");
    let accept = parse(
        &[
            "accept",
            "a.docx",
            "-o",
            "out.docx",
            "--id",
            "body:rev:12",
            "--kind",
            "insertion",
        ],
        &[],
    );
    assert_eq!(accept["args"]["ids"][0], "body:rev:12");
    assert_eq!(accept["args"]["kinds"][0], "insertion");
    let fields = parse(&["fields", "update", "a", "-o", "b"], &[]);
    assert_eq!(fields["args"]["sub"]["command"], "update");
    assert_eq!(fields["args"]["sub"]["args"]["output"], "b");
}

#[test]
fn invalid_numbers_pages_palettes_and_render_combinations_are_usage_errors() {
    for dpi in ["0", "-1", "NaN", "inf", "1201"] {
        for command in ["convert", "edit", "diff-render"] {
            let base = match command {
                "edit" => vec!["edit", "a", "--plan", "p", "--out-dir", "d"],
                "diff-render" => vec!["diff-render", "a", "b"],
                _ => vec!["convert", "a", "--png"],
            };
            assert_eq!(
                parse(&[base, vec!["--dpi", dpi]].concat(), &[])["exit_code"],
                2
            );
        }
    }
    for ratio in ["NaN", "inf", "-0.1", "1.1"] {
        assert_eq!(
            parse(&["compare", "a", "b", "--detail-threshold", ratio], &[])["exit_code"],
            2
        );
    }
    for ratio in ["0", "1"] {
        assert_eq!(
            parse(&["compare", "a", "b", "--detail-threshold", ratio], &[])["exit_code"],
            0
        );
    }
    for pages in ["0", "3-1", "1,,2", "one", "1-2-3"] {
        assert_eq!(
            parse(&["convert", "a", "--png", "--pages", pages], &[])["exit_code"],
            2
        );
    }
    for flags in [
        vec!["--revisions", "custom"],
        vec![
            "--revisions",
            "word",
            "--revision-palette",
            "deleted=#000000",
        ],
        vec!["--revisions", "custom", "--revision-palette", "deleted=red"],
        vec!["--to", "md", "--png"],
        vec!["--pages", "1"],
    ] {
        assert_eq!(
            parse(&[vec!["convert", "a"], flags].concat(), &[])["exit_code"],
            2
        );
    }
}

#[test]
fn help_explains_tasks_and_critic_markup() {
    let help = parse(&["--help"], &[]);
    let text = help["text"].as_str().unwrap();
    assert!(text.contains("compare") && text.contains("redline") && text.contains("Examples:"));
    let diff = parse(&["diff", "--help"], &[]);
    let text = diff["text"].as_str().unwrap();
    assert!(text.contains("GitHub") && text.contains("current document text with tracked marks"));
}

#[test]
fn supported_compare_never_accepts_a_disabled_task_as_filenames() {
    for command in ["convert", "self-update", "debug", "audit"] {
        let result = parse(&[command, "a.docx"], &["compare", "inspect"]);
        assert_eq!(result["exit_code"], 2, "{result}");
        assert_eq!(result["stream"], "stderr");
    }
    let help = parse(&["convert", "--help"], &["convert"]);
    let text = help["text"].as_str().unwrap();
    assert!(text.contains("--reference-doc") && text.contains("--track-changes"));
}

#[test]
fn palette_and_timeout_defaults_are_platform_neutral() {
    let result = parse(
        &[
            "convert",
            "a",
            "--png",
            "--revisions",
            "custom",
            "--revision-palette",
            "deleted=#112233:strike",
            "--timeout",
            "0.5",
        ],
        &[],
    );
    assert_eq!(result["exit_code"], 0, "{result}");
    assert_eq!(result["args"]["revisions"], "custom");
    assert_eq!(result["args"]["timeout"], 0.5);
    for seconds in ["0", "-1", "NaN", "inf"] {
        assert_eq!(
            parse(&["convert", "a", "--timeout", seconds], &[])["exit_code"],
            2
        );
    }
    for args in [
        vec!["convert", "a", "-o", "out.md", "--png"],
        vec!["convert", "a", "-o", "out.docx", "--report", "report.json"],
    ] {
        assert_eq!(parse(&args, &[])["exit_code"], 2);
    }
}

#[test]
fn text_views_options_and_host_metadata_share_clap() {
    for format in ["github", "word", "normal", "context", "side-by-side"] {
        let result = parse(
            &[
                "diff",
                "a.docx",
                "b.txt",
                "--format",
                format,
                "-U0",
                "--accept-changes",
                "--full-lines",
                "-o",
                "review.txt",
            ],
            &[],
        );
        assert_eq!(result["exit_code"], 0, "{result}");
        assert_eq!(result["args"]["context"], 0);
        assert_eq!(result["args"]["accept_changes"], true);
        assert_eq!(result["args"]["full_lines"], true);
        assert_eq!(result["args"]["old_format"], "docx");
        assert_eq!(result["args"]["new_format"], "md");
        assert_eq!(result["args"]["output_format"], "md");
    }
    let compared = parse(&["compare", "a.txt", "b.markdown", "-o", "out.md"], &[]);
    assert_eq!(compared["args"]["old_format"], "md");
    assert_eq!(compared["args"]["new_format"], "md");
    assert_eq!(compared["args"]["output_format"], "md");
    for args in [
        vec!["--format", "critic", "--full-lines"],
        vec!["--format", "patch", "--accept-changes"],
        vec!["--format", "word", "--to", "docx"],
        vec!["--format", "github", "--context", "4294967296"],
        vec!["--from", "pdf"],
    ] {
        assert_eq!(
            parse(&[vec!["diff", "a", "b"], args].concat(), &[])["exit_code"],
            2
        );
    }
    let result = parse(
        &[
            "diff",
            "a.docx",
            "b.docx",
            "--from",
            "md",
            "--format",
            "patch",
            "-o",
            "unknown.ext",
        ],
        &[],
    );
    assert_eq!(result["args"]["old_format"], "md");
    assert_eq!(result["args"]["new_format"], "md");
    assert!(result["args"]["output_format"].is_null());
}

#[test]
fn page_errors_explain_the_invalid_item() {
    for (pages, phrase) in [
        ("x", "is not a page number"),
        ("+1", "is not a page number"),
        ("0", "pages are counted from 1"),
        ("1,,2", "empty item in page selection"),
        ("3-1", "runs backwards"),
    ] {
        let result = parse(&["convert", "a", "--png", "--pages", pages], &[]);
        assert_eq!(result["exit_code"], 2);
        assert!(
            result["text"].as_str().unwrap().contains(phrase),
            "{result}"
        );
    }
}
