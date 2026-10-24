// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Deterministic shared CLI parsing, with no document or platform I/O.
#![cfg(feature = "cli")]

use jubarte::cli::{
    Cli, Command, Format, PatchFormat, Task, output_format, parse_json, retired_note,
    retired_spellings,
};
use serde_json::Value;
use std::path::Path;

fn parse(arguments: &[&str], supported: &[&str]) -> Value {
    serde_json::from_str(&parse_json(
        &arguments.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "jubarte-python",
        &supported.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    ))
    .unwrap()
}

#[test]
fn convert_update_fields_requires_a_word_output_format() {
    for output in [
        vec![],
        vec!["-o", "out.pdf"],
        vec!["-o", "out.png"],
        vec!["-o", "out.md"],
        vec!["-o", "out.unknown"],
        vec!["--to", "pdf"],
        vec!["--to", "png"],
        vec!["--to", "md"],
    ] {
        let args = [vec!["convert", "missing.docx", "--update-fields"], output].concat();
        let result = parse(&args, &["convert"]);
        assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
        assert_eq!(result["stream"], "stderr");
        assert!(
            result["text"]
                .as_str()
                .unwrap()
                .contains("--update-fields writes a .docx"),
            "{result}"
        );
    }
    // The -o rule fires first.
    let result = parse(
        &[
            "convert",
            "missing.docx",
            "--update-fields",
            "--to",
            "pdf",
            "-o",
            "out.docx",
        ],
        &["convert"],
    );
    assert_eq!(result["exit_code"], 2, "{result}");
    assert!(
        result["text"]
            .as_str()
            .unwrap()
            .contains("-o out.docx says docx and --to says pdf: drop one"),
        "{result}"
    );
}

#[test]
fn convert_update_fields_serializes_reports_resolution_and_format_overrides() {
    for output in [
        vec!["-o", "out.docx"],
        vec!["-o", "out.DOCX"],
        vec!["--to", "docx"],
        vec!["--to", "docx", "-o", "out.bin"],
    ] {
        for mode in ["all", "accept", "reject"] {
            let args = [
                vec![
                    "convert",
                    "missing.docx",
                    "--update-fields",
                    "--report",
                    "fields.json",
                    "--track-changes",
                    mode,
                    "--force",
                ],
                output.clone(),
            ]
            .concat();
            let result = parse(&args, &["convert"]);
            assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
            assert_eq!(result["command"], "convert");
            assert_eq!(result["args"]["update_fields"], true);
            assert_eq!(result["args"]["report"], "fields.json");
            assert_eq!(result["args"]["track_changes"], mode);
            assert_eq!(result["args"]["force"], true);
        }
    }
    let ordinary = parse(&["convert", "missing.docx"], &["convert"]);
    assert_eq!(ordinary["args"]["update_fields"], false);
    let report_without_refresh = parse(
        &[
            "convert",
            "missing.docx",
            "-o",
            "out.docx",
            "--report",
            "fields.json",
        ],
        &["convert"],
    );
    assert_eq!(report_without_refresh["exit_code"], 2);
    assert!(
        report_without_refresh["text"]
            .as_str()
            .unwrap()
            .contains("--report applies to PDF or PNG output only")
    );
}

#[test]
fn convert_update_fields_does_not_enable_render_only_options() {
    for flags in [
        vec!["--dpi", "96"],
        vec!["--font-report", "fonts.json"],
        vec!["--fail-on-substitution"],
        vec!["--move-comments"],
        vec!["--changed-pages"],
        vec!["--pages", "1"],
        vec!["--compress"],
        vec!["--revisions", "word"],
    ] {
        let result = parse(
            &[
                vec![
                    "convert",
                    "missing.docx",
                    "-o",
                    "out.docx",
                    "--update-fields",
                    "--report",
                    "fields.json",
                ],
                flags.clone(),
            ]
            .concat(),
            &[],
        );
        assert_eq!(result["exit_code"], 2, "{flags:?}: {result}");
        assert!(
            result["text"]
                .as_str()
                .unwrap()
                .contains(&format!("{} applies to PDF or PNG output only", flags[0])),
            "{result}"
        );
    }
    // The retired --pdf and --png name a format the -o rule weighs.
    for (alias, said) in [("--pdf", "pdf"), ("--png", "png")] {
        let result = parse(&["convert", "missing.docx", "-o", "out.docx", alias], &[]);
        assert_eq!(result["exit_code"], 2, "{result}");
        assert!(
            result["text"].as_str().unwrap().contains(&format!(
                "-o out.docx says docx and --to says {said}: drop one"
            )),
            "{result}"
        );
    }
}

#[test]
fn removed_commands_leave_help_and_restore_legacy_filename_parsing() {
    let help = parse(&["--help"], &[]);
    let text = help["text"].as_str().unwrap();
    assert!(text.contains("\n  changes "));
    assert!(!text.contains("\n  revisions ") && !text.contains("\n  fields "));
    for name in ["revisions", "fields"] {
        let result = parse(&[name, "other.docx", "-o", "result.docx"], &[]);
        assert_eq!(result["exit_code"], 0, "{result}");
        assert_eq!(result["command"], "compare");
        assert_eq!(result["args"]["original"], name);
        assert_eq!(result["args"]["modified"], "other.docx");
    }
    assert_eq!(
        parse(&["revisions", "in.docx", "--json"], &[])["exit_code"],
        2
    );
    assert_eq!(
        parse(&["fields", "update", "in.docx", "-o", "out.docx"], &[])["exit_code"],
        2
    );
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
        assert_eq!(result["args"]["author"], "Jubarte");
        assert_eq!(result["args"]["mode"], "word");
    }
    assert_eq!(
        parse(&["compare", "-b", "a", "-m", "b"], &[])["exit_code"],
        0
    );
    for args in [&[][..], &["compare"][..], &["compare", "-b", "a"][..]] {
        assert_eq!(parse(args, &[])["exit_code"], 2);
    }
    // One file is the agent view (`read`); two print their redline's view.
    let read = parse(&["a.docx"], &[]);
    assert_eq!(read["exit_code"], 0, "{read}");
    assert_eq!(read["command"], "read");
    assert_eq!(read["args"]["file"], "a.docx");
    assert!(read["args"]["head"].is_null() && read["args"]["changed"] == false);
    assert_eq!(parse(&["a.docx", "--head", "2"], &[])["args"]["head"], 2);
    // A bare word is a mistyped task, not a document.
    let typo = parse(&["frobnicate"], &[]);
    assert_eq!(typo["exit_code"], 2, "{typo}");
    assert!(
        typo["text"].as_str().unwrap().contains("read FILE"),
        "{typo}"
    );
    assert_eq!(parse(&["read", "frobnicate"], &[])["exit_code"], 0);
    // -o has nothing to write for one document; read options before a task
    // would be silently dropped.
    let one = parse(&["a.docx", "-o", "out.docx"], &[]);
    assert_eq!(one["exit_code"], 2, "{one}");
    assert!(one["text"].as_str().unwrap().contains("-o"), "{one}");
    for args in [
        &["--head", "2", "compare", "a.docx", "b.docx"][..],
        &["--changed", "read", "a.docx"][..],
    ] {
        assert_eq!(parse(args, &[])["exit_code"], 2, "{args:?}");
    }
    assert_eq!(
        parse(&["a.docx"], &["compare"])["exit_code"],
        2,
        "read not supported"
    );
    assert_eq!(parse(&["compare", "a.docx"], &[])["exit_code"], 2);
    assert_eq!(
        parse(&["a.docx", "b.docx", "--head", "2"], &[])["args"]["view"]["head"],
        2
    );
    assert_eq!(
        parse(&["a.docx", "b.docx", "-o", "x.docx", "--head", "2"], &[])["exit_code"],
        2
    );
    // pi review av5 F3: an adapter without read cannot print the A B view,
    // so the bare shorthand is refused as the one-file form is; with -o it
    // is a plain compare and carries no view.
    assert_eq!(
        parse(&["a.docx", "b.docx"], &["compare"])["exit_code"],
        2,
        "read not supported"
    );
    let written = parse(&["a.docx", "b.docx", "-o", "x.docx"], &["compare"]);
    assert_eq!(written["command"], "compare", "{written}");
    assert!(written["args"].get("view").is_none(), "{written}");
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
    let supported = ["read", "diff"];
    let help = parse(&["--help"], &supported);
    assert_eq!(help["exit_code"], 0);
    assert_eq!(help["stream"], "stdout");
    let text = help["text"].as_str().unwrap();
    assert!(text.contains("jubarte-python"));
    assert!(text.contains("read") && text.contains("diff"));
    assert!(!text.contains("inspect"), "{text}");
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
    // Without -o compare prints the agent view, which needs read.
    assert_eq!(
        parse(&["redline", "a", "b", "-o", "x.docx"], &["compare"])["command"],
        "compare"
    );
    for args in [
        &["compare", "a", "b"][..],
        &["a", "b"],
        &["compare", "a", "b", "-o", "v.md"],
    ] {
        let result = parse(args, &["compare"]);
        assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
        assert!(
            result["text"]
                .as_str()
                .unwrap()
                .contains("prints no agent view"),
            "{result}"
        );
    }
    assert_eq!(
        parse(&["compare", "a", "b"], &["compare", "read"])["exit_code"],
        0
    );
    assert_eq!(parse(&["--version"], &supported)["stream"], "stdout");
    let inspect = parse(&["inspect", "a", "--json"], &supported);
    assert_eq!(inspect["command"], "read", "{inspect}");
    assert_eq!(inspect["args"]["json"], true);
    assert_eq!(
        inspect["notes"][0],
        "note: inspect is now read; inspect is removed in 0.14"
    );
    assert_eq!(
        parse(&["read", "a"], &supported)["notes"],
        serde_json::json!([])
    );
    let audit = parse(&["audit", "a"], &[]);
    assert_eq!(audit["command"], "validate", "{audit}");
    assert_eq!(
        audit["args"]["rules"],
        serde_json::json!(["a11y", "style", "structure"])
    );
    // Rules given to the alias win.
    let audit = parse(&["audit", "a", "--rules", "style"], &[]);
    assert_eq!(audit["args"]["rules"], serde_json::json!(["style"]));
    assert_eq!(
        parse(&["validate", "a"], &[])["args"]["rules"],
        serde_json::json!([])
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
            "r12",
            "--kind",
            "insertion",
        ],
        &[],
    );
    assert_eq!(accept["args"]["ids"][0], "r12");
    assert_eq!(accept["args"]["kinds"][0], "insertion");
    let fields = parse(&["convert", "a", "-o", "b.docx", "--update-fields"], &[]);
    assert_eq!(fields["args"]["update_fields"], true);
    assert_eq!(fields["args"]["output"], "b.docx");
}

#[test]
fn invalid_numbers_pages_palettes_and_render_combinations_are_usage_errors() {
    for dpi in ["0", "-1", "NaN", "inf", "1201"] {
        for command in ["convert", "edit", "visual-diff"] {
            let base = match command {
                "edit" => vec!["edit", "a", "--plan", "p", "--out-dir", "d"],
                "visual-diff" => vec!["visual-diff", "a", "b"],
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
        vec!["-o", "out.md", "--pages", "1"],
        vec!["--pdf", "--png"],
    ] {
        assert_eq!(
            parse(&[vec!["convert", "a"], flags].concat(), &[])["exit_code"],
            2
        );
    }
    // Codex on #440 and #441: every task that draws revisions refuses a
    // palette without `custom` before reading a document, as convert does.
    let palette = [
        "--revisions",
        "word",
        "--revision-palette",
        "deleted=#112233",
    ];
    for task in [
        &["compare", "a.docx", "b.docx", "-o", "c.pdf"][..],
        &["a.docx", "b.docx", "-o", "c.pdf"],
        &["accept", "a.docx", "-o", "c.pdf"],
        &["reject", "a.docx", "-o", "c.pdf"],
        &["scrub", "a.docx", "-o", "c.pdf"],
    ] {
        let refused = parse(&[task, &palette[..]].concat(), &[]);
        assert_eq!(refused["exit_code"], 2, "{task:?}: {refused}");
        assert!(
            refused["text"]
                .as_str()
                .unwrap()
                .contains("--revision-palette needs --revisions custom"),
            "{task:?}: {refused}"
        );
    }
    // --pages narrows a PDF too.
    let pdf = parse(&["convert", "a", "--pages", "1"], &[]);
    assert_eq!(pdf["exit_code"], 0, "{pdf}");
    assert_eq!(pdf["args"]["pages"], "1");
}

/// Task 14: diff writes text, so it takes no rendering flag; a Word, PDF or
/// PNG output names compare.
#[test]
fn diff_has_no_rendering_flags() {
    for flags in [
        &["--revisions", "word"][..],
        &["--revision-palette", "deleted=#000000"],
        &["--mode", "word"],
        &["--detail-threshold", "0.5"],
        &["--reference-doc", "r.docx"],
        &["--critic"],
        &["--resource-path", "."],
        &["--move-comments"],
        &["--dpi", "96"],
    ] {
        let result = parse(&[&["diff", "a.md", "b.md"][..], flags].concat(), &[]);
        assert_eq!(result["exit_code"], 2, "{flags:?}: {result}");
    }
    // pi-14 C3: CriticMarkup has no owner and no wrap, so the patch's
    // flags are refused rather than dropped.
    for flags in [
        &["-a", "Ann"][..],
        &["--datetime", "2026-01-01T00:00:00Z"],
        &["--columns", "40"],
    ] {
        let result = parse(
            &[&["diff", "a.md", "b.md", "--format", "critic"][..], flags].concat(),
            &[],
        );
        assert_eq!(result["exit_code"], 2, "{flags:?}: {result}");
        assert!(
            result["text"]
                .as_str()
                .unwrap()
                .contains("applies to the paragraph patch only"),
            "{result}"
        );
    }
    // --to alone names no path: the advice names a placeholder.
    let result = parse(&["diff", "a.md", "b.md", "--to", "pdf"], &[]);
    assert_eq!(result["exit_code"], 2, "{result}");
    assert!(
        result["text"]
            .as_str()
            .unwrap()
            .contains("`compare OLD NEW -o c.pdf`"),
        "{result}"
    );
    for output in ["r.docx", "r.pdf", "r.png"] {
        let result = parse(&["diff", "a.md", "b.md", "-o", output], &[]);
        assert_eq!(result["exit_code"], 2, "{output}: {result}");
        let text = result["text"].as_str().unwrap();
        assert!(
            text.contains(&format!("`compare OLD NEW -o {output}`")),
            "{text}"
        );
    }
}

/// Codex on #442: add draws no PDF or PNG, so its help names none; edit's
/// does. Both say what another -o extension writes.
#[test]
fn add_help_names_only_what_add_writes() {
    let help = |task: &str| {
        parse(&[task, "--help"], &[])["text"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let (add, edit) = (help("add"), help("edit"));
    assert!(!add.contains("--pdf") && !add.contains("--png"), "{add}");
    assert!(edit.contains("--pdf") && edit.contains("--png"), "{edit}");
    for text in [&add, &edit] {
        assert!(text.contains("Any other extension is written as"), "{text}");
    }
}

#[test]
fn help_explains_tasks_and_critic_markup() {
    let help = parse(&["--help"], &[]);
    let text = help["text"].as_str().unwrap();
    assert!(text.contains("compare") && text.contains("redline") && text.contains("Examples:"));
    // The top level lists the tasks; the shorthand's options live in
    // `compare --help` and `read --help`.
    for hidden in [
        "--detail-threshold",
        "<ORIGINAL>",
        "--head",
        "--changed-pages",
    ] {
        assert!(!text.contains(hidden), "{hidden}: {text}");
    }
    assert!(text.contains("--quiet"), "{text}");
    let native = Cli::try_parse_from(["jubarte", "--help"])
        .unwrap_err()
        .to_string();
    assert!(!native.contains("--detail-threshold") && !native.contains("<ORIGINAL>"));
    let compare = parse(&["compare", "--help"], &[]);
    let text = compare["text"].as_str().unwrap();
    for shown in [
        "--detail-threshold",
        "--changed-pages",
        "--head",
        "--reference-doc",
        ".json",
    ] {
        assert!(text.contains(shown), "{shown}: {text}");
    }
    assert!(!text.contains("--powertools-faithful"), "{text}");
    let diff = parse(&["diff", "--help"], &[]);
    let text = diff["text"].as_str().unwrap();
    assert!(text.contains("GitHub") && text.contains("current document text with tracked marks"));
}

#[test]
fn supported_compare_never_accepts_a_disabled_task_as_filenames() {
    for command in ["convert", "self-update", "debug", "validate"] {
        let result = parse(&[command, "a.docx"], &["compare", "read"]);
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
    // An unknown extension is diff's usual text.
    assert_eq!(result["args"]["output_format"], "md");
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

#[test]
fn pdf_page_options_are_shared_and_rejected_for_text_outputs() {
    let note = "note: --changed-only is now --changed-pages; --changed-only is removed in 0.14";
    for (args, notes) in [
        (
            vec!["convert", "a.docx", "--move-comments", "--changed-only"],
            vec![note],
        ),
        (
            vec!["convert", "a.docx", "--move-comments", "--changed-pages"],
            vec![],
        ),
        (
            vec![
                "compare",
                "a.docx",
                "b.docx",
                "-o",
                "out.pdf",
                "--move-comments",
                "--changed-only",
            ],
            vec![note],
        ),
        // zcode-14 F1: the shorthand and every RenderArgs task note it too.
        (
            vec![
                "a.docx",
                "b.docx",
                "--changed-only",
                "-o",
                "out.pdf",
                "--move-comments",
            ],
            vec![note],
        ),
        (
            vec![
                "accept",
                "a.docx",
                "-o",
                "out.pdf",
                "--move-comments",
                "--changed-only",
            ],
            vec![note],
        ),
        (
            vec![
                "scrub",
                "a.docx",
                "-o",
                "out.pdf",
                "--move-comments",
                "--changed-only",
            ],
            vec![note],
        ),
    ] {
        let result = parse(&args, &[]);
        assert_eq!(result["exit_code"], 0, "{result}");
        assert_eq!(result["args"]["move_comments"], true, "{result}");
        assert_eq!(result["args"]["changed_pages"], true, "{result}");
        assert_eq!(result["notes"], serde_json::json!(notes), "{result}");
    }
    // The retired --pdf and --png name the format.
    for (alias, format) in [("--pdf", "pdf"), ("--png", "png")] {
        let result = parse(&["convert", "a.docx", alias], &[]);
        assert_eq!(result["exit_code"], 0, "{result}");
        assert_eq!(result["args"]["output_format"], format, "{result}");
        assert_eq!(
            result["notes"][0],
            format!("note: {alias} is now --to {format}; {alias} is removed in 0.14")
        );
    }
    let named = parse(&["convert", "a.docx", "-o", "out.png"], &[]);
    assert_eq!(named["args"]["output_format"], "png", "{named}");
    assert_eq!(named["notes"], serde_json::json!([]));
    let by_input = parse(&["convert", "a.docx"], &[]);
    assert_eq!(by_input["args"]["output_format"], Value::Null, "{by_input}");
    // --pdf belongs to edit, where it is not retired.
    let edit = parse(&["edit", "a.docx", "--plan", "p.json", "--pdf"], &[]);
    assert_eq!(edit["notes"], serde_json::json!([]), "{edit}");
    for flag in ["--move-comments", "--changed-pages", "--changed-only"] {
        for args in [
            vec!["convert", "a.docx", "-t", "md", flag],
            vec!["diff", "a.docx", "b.docx", "--format", "github", flag],
            vec!["diff", "a.docx", "b.docx", "-o", "out.docx", flag],
            vec!["diff", "a.md", "b.md", flag],
        ] {
            assert_eq!(parse(&args, &[])["exit_code"], 2, "{args:?}");
        }
    }
}

/// D15: a host reads where edit and add write from `args`: `output` for the
/// one document, `out_dir` for the bundle, the other null.
#[test]
fn parse_json_resolves_where_edit_writes() {
    let row = |args: &[&str]| {
        let parsed = parse(args, &["edit", "add"]);
        assert_eq!(parsed["exit_code"], 0, "{args:?}: {parsed}");
        (
            parsed["args"]["output"].clone(),
            parsed["args"]["out_dir"].clone(),
        )
    };
    let edit = ["edit", "d/a.docx", "-p", "p1", "--content", "x"];
    assert_eq!(row(&edit), (Value::from("d/a_modified.docx"), Value::Null));
    assert_eq!(
        row(&["add", "d/a_modified.docx", "-p", "p1", "--content", "x"]),
        (Value::from("d/a_modified.docx"), Value::Null),
        "a _modified file is edited onto itself"
    );
    assert_eq!(
        row(&[&edit[..], &["-o", "b.docx"]].concat()),
        (Value::from("b.docx"), Value::Null)
    );
    assert_eq!(
        row(&[&edit[..], &["--out-dir"]].concat()),
        (Value::Null, Value::from("d/a.edit"))
    );
    assert_eq!(
        row(&[&edit[..], &["--out-dir", "r"]].concat()),
        (Value::Null, Value::from("r"))
    );
    // pi-13b F7: recording writes the plan only, so neither is named.
    assert_eq!(
        row(&[&edit[..], &["--plan", "p.json"]].concat()),
        (Value::Null, Value::Null)
    );
    for bad in [&["-o", "b.pdf"][..], &["--out-dir", "x.docx"][..]] {
        let parsed = parse(&[&edit[..], bad].concat(), &["edit"]);
        assert_eq!(parsed["exit_code"], 2, "{bad:?}: {parsed}");
    }
}

/// D16: the shared parser resolves `--plan` without a value, so a host
/// gets the path and whether it was the default.
#[test]
fn parse_json_resolves_a_bare_plan() {
    let bare = parse(
        &["edit", "d/a.docx", "--plan", "-p", "p1", "--content", "x"],
        &["edit"],
    );
    assert_eq!(bare["args"]["plan"], "d/a_plan.json", "{bare}");
    assert_eq!(bare["args"]["plan_default"], true, "{bare}");
    // Codex on #433: a path ending in a separator names a folder, so the
    // parser refuses it for every host alike.
    let folder = parse(&["add", "a.docx", "--plan", "new.json/"], &["add"]);
    assert_eq!(folder["exit_code"], 2, "{folder}");
    assert!(
        folder["text"]
            .as_str()
            .unwrap()
            .contains("new.json/ ends in a separator"),
        "{folder}"
    );
    let named = parse(&["add", "a.docx", "--plan", "p.json"], &["add"]);
    assert_eq!(named["args"]["plan"], "p.json", "{named}");
    assert_eq!(named["args"]["plan_default"], false, "{named}");
    let swallowed = parse(
        &["edit", "--plan", "a.docx", "-p", "p1", "--content", "x"],
        &["edit"],
    );
    assert_eq!(swallowed["exit_code"], 2, "{swallowed}");
    assert!(
        swallowed["text"]
            .as_str()
            .unwrap()
            .contains("--plan a.docx names a Word document"),
        "{swallowed}"
    );
}

/// #568: the JSON fields an agent parses name the CLI-resolved paths
/// with `/` on every OS, whatever spelling FILE was given in (a Windows
/// `d\a.docx` must not leak `\` into `plan`, `output` or `out_dir`).
#[test]
fn parse_json_prints_resolved_paths_with_forward_slashes() {
    let edit = ["edit", "d\\a.docx", "-p", "p1", "--content", "x"];
    let bare = parse(&[&edit[..], &["--plan"]].concat(), &["edit"]);
    assert_eq!(bare["args"]["plan"], "d/a_plan.json", "{bare}");
    let row = |args: &[&str]| {
        let parsed = parse(args, &["edit"]);
        assert_eq!(parsed["exit_code"], 0, "{args:?}: {parsed}");
        (
            parsed["args"]["output"].clone(),
            parsed["args"]["out_dir"].clone(),
        )
    };
    assert_eq!(row(&edit), (Value::from("d/a_modified.docx"), Value::Null));
    let bundle = [&edit[..], &["--out-dir"]].concat();
    assert_eq!(row(&bundle), (Value::Null, Value::from("d/a.edit")));
}

/// pi review av4 F11: a revision timestamp is an xsd:dateTime (`w:date`);
/// clap refuses anything else instead of writing it into the document.
#[test]
fn revision_timestamps_must_be_iso_8601() {
    for bad in [
        "yesterday",
        "2026-10-01",
        "2026-13-01T09:00:00Z",
        "2026-10-01 09:00",
        // pi review r392 F4/F5: offsets stop at 14:00, there is no year 0,
        // and Word's subset has no 24:00:00.
        "2026-10-01T09:00:00+14:59",
        "0000-01-01T00:00:00Z",
        "2026-10-01T24:00:00Z",
    ] {
        for args in [
            vec!["compare", "a.docx", "b.docx", "-d", bad],
            vec![
                "edit",
                "a.docx",
                "-p",
                "p0",
                "--content",
                "x",
                "--datetime",
                bad,
            ],
        ] {
            assert_eq!(parse(&args, &[])["exit_code"], 2, "{args:?}");
        }
    }
    for good in [
        "2026-10-01T09:00:00Z",
        "2026-10-01T09:00:00",
        "2026-10-01T09:00:00.125+02:00",
        "2026-10-01T09:00:00-14:00",
    ] {
        for args in [
            vec!["compare", "a.docx", "b.docx", "-d", good],
            vec![
                "edit",
                "a.docx",
                "-p",
                "p0",
                "--content",
                "x",
                "--datetime",
                good,
            ],
        ] {
            assert_eq!(parse(&args, &[])["exit_code"], 0, "{args:?}");
        }
    }
}

#[test]
fn pr392_revision_dates_cover_calendar_fraction_and_offset_boundaries() {
    for (date, valid) in [
        ("2000-02-29T23:59:59.123456789+14:00", true),
        ("2024-02-29T00:00:00-14:00", true),
        ("0001-01-01T00:00:00", true),
        ("9999-12-31T23:59:59.0Z", true),
        ("2026-04-30T12:00:00+13:59", true),
        ("1900-02-29T00:00:00Z", false),
        ("2100-02-29T00:00:00Z", false),
        ("2026-02-29T00:00:00Z", false),
        ("2026-04-31T00:00:00Z", false),
        ("2026-01-00T00:00:00Z", false),
        ("2026-01-01T00:60:00Z", false),
        ("2026-01-01T00:00:60Z", false),
        ("2026-01-01T00:00:00.Z", false),
        ("2026-01-01T00:00:00.éZ", false),
        ("2026-01-01T00:00:00+14:01", false),
        ("2026-01-01T00:00:00-14:01", false),
        ("2026-01-01T00:00:00+00:60", false),
        ("2026-01-01T00:00:00+0a:00", false),
        ("2026-01-01T00:00:00Ztrailing", false),
        ("2026-01-01T00:00:00z", false),
        ("２０２６-01-01T00:00:00Z", false),
        ("", false),
    ] {
        for args in [
            vec!["compare", "a.docx", "b.docx", "--datetime", date],
            vec!["a.docx", "b.docx", "-d", date],
            vec![
                "edit",
                "a.docx",
                "-p",
                "p0",
                "--content",
                "x",
                "--datetime",
                date,
            ],
            vec![
                "add",
                "a.docx",
                "-p",
                "p0",
                "--content",
                "x",
                "--datetime",
                date,
            ],
            vec!["convert", "a.md", "-t", "docx", "--datetime", date],
            vec!["diff", "a.md", "b.md", "--datetime", date],
        ] {
            let result = parse(&args, &[]);
            assert_eq!(
                result["exit_code"],
                if valid { 0 } else { 2 },
                "{args:?}: {result}"
            );
            if !valid {
                assert!(
                    result["text"].as_str().unwrap().contains("ISO 8601"),
                    "{result}"
                );
            }
        }
        if valid {
            assert_eq!(
                parse(&["compare", "a", "b", "-d", date], &[])["args"]["datetime"],
                date
            );
        }
    }
}

#[test]
fn compare_and_diff_date_their_revisions_now_unless_d_pins_it() {
    // D17: the parser fills now (UTC, to the second), so the binary and
    // every host get the same value; the fixed-width form orders as text.
    let before = jubarte::convert::utc_now_iso8601();
    let runs = [
        parse(&["compare", "a.docx", "b.docx"], &[]),
        parse(&["a.docx", "b.docx"], &[]),
        parse(&["diff", "a.docx", "b.docx"], &[]),
    ];
    let native = jubarte::cli::Cli::try_parse_from(["jubarte", "a.docx", "b.docx"]).unwrap();
    let after = jubarte::cli::Cli::try_parse_from(["jubarte", "compare", "a.docx", "b.docx"])
        .map(|_| jubarte::convert::utc_now_iso8601())
        .unwrap();
    let mut dates: Vec<String> = runs
        .iter()
        .map(|run| {
            run["args"]["datetime"]
                .as_str()
                .expect("a date")
                .to_string()
        })
        .collect();
    dates.push(native.compare.datetime.expect("the shorthand's date"));
    for date in &dates {
        assert!(
            before.as_str() <= date.as_str() && date.as_str() <= after.as_str(),
            "{before} {date} {after}"
        );
        assert!(date.ends_with('Z') && date.len() == 20, "{date}");
    }
    for task in [&["compare"][..], &[][..], &["diff"][..]] {
        let args = [
            task,
            &["a.docx", "b.docx", "-d", "1970-01-01T00:00:00Z"][..],
        ]
        .concat();
        assert_eq!(
            parse(&args, &[])["args"]["datetime"],
            "1970-01-01T00:00:00Z",
            "{args:?}"
        );
    }
}

#[test]
fn pr392_author_filters_share_validation_with_the_library() {
    use jubarte::markdown::{Select, author};

    for (given, expected) in [
        (" AC ", "AC"),
        ("\t@AC\n", "@AC"),
        ("\u{2003}Ánn Counsel\u{2003}", "Ánn Counsel"),
        ("Ann  Counsel", "Ann  Counsel"),
    ] {
        assert_eq!(author(given).unwrap(), expected);
        let selected = Select::from_flags(None, None, None, true, Some(given)).unwrap();
        assert!(matches!(selected, Some(Select::Changed { by: Some(ref by) }) if by == expected));
        for prefix in [vec!["read", "a.docx"], vec!["a.docx"]] {
            let args = [prefix, vec!["--changed", "--by", given]].concat();
            let result = parse(&args, &[]);
            assert_eq!(result["exit_code"], 0, "{result}");
            assert_eq!(result["args"]["by"], expected, "{result}");
        }
    }
    for bad in [
        "",
        "\t\n",
        "\u{2003}",
        "@",
        "@@AC",
        "@ AC",
        "@\tAC",
        "@\u{2003}AC",
    ] {
        assert!(author(bad).is_err(), "{bad:?}");
        assert!(
            Select::from_flags(None, None, None, true, Some(bad)).is_err(),
            "{bad:?}"
        );
        let result = parse(&["read", "a.docx", "--changed", "--by", bad], &[]);
        assert_eq!(result["exit_code"], 2, "{bad:?}: {result}");
        assert!(
            result["text"].as_str().unwrap().contains("needs an author"),
            "{result}"
        );
    }
}

#[test]
fn shorthand_forwards_each_read_option_like_the_explicit_read_task() {
    for flags in [
        vec![],
        vec!["--track-changes", "reject"],
        vec!["--comments", "none"],
        vec!["--dates"],
        vec!["--no-page-markers"],
        vec!["-p", "p0-p2,t0"],
        vec!["--head", "0"],
        vec!["--tail", "1"],
        vec!["--changed", "--by", "Ann Counsel"],
    ] {
        let explicit = parse(&[vec!["read", "a.docx"], flags.clone()].concat(), &[]);
        let single = parse(&[vec!["a.docx"], flags.clone()].concat(), &[]);
        assert_eq!(single["exit_code"], 0, "{flags:?}: {single}");
        assert_eq!(single, explicit, "{flags:?}");
        let pair = parse(&[vec!["a.docx", "b.docx"], flags].concat(), &[]);
        assert_eq!(pair["exit_code"], 0, "{pair}");
        let mut expected = explicit["args"].clone();
        // -q is the whole command's, and read's own -o goes with read: the
        // view takes only the read options.
        for key in ["file", "quiet", "output", "force", "output_format"] {
            expected.as_object_mut().unwrap().remove(key);
        }
        assert_eq!(pair["args"]["view"], expected);
    }
}

#[test]
fn every_explicit_read_option_conflicts_with_writing_even_at_its_default() {
    for flags in [
        vec!["--track-changes", "all"],
        vec!["--comments", "inline"],
        vec!["--dates"],
        vec!["--no-page-markers"],
        vec!["--paragraphs", "p0"],
        vec!["--head", "0"],
        vec!["--tail", "0"],
        vec!["--changed"],
        vec!["--changed", "--by", "AC"],
    ] {
        let args = [
            vec!["missing-a.docx", "missing-b.docx", "-o", "out.docx"],
            flags,
        ]
        .concat();
        let result = parse(&args, &[]);
        assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
        assert_eq!(result["stream"], "stderr");
        assert!(
            result["text"]
                .as_str()
                .unwrap()
                .contains("read options shape the view: print it (no -o) or write it (-o FILE.md)"),
            "{result}"
        );
        assert_eq!(
            Cli::try_parse_from([vec!["jubarte"], args.clone()].concat())
                .unwrap_err()
                .kind(),
            clap::error::ErrorKind::ArgumentConflict
        );
        // compare refuses them alike; a .md output is the view, which takes
        // them.
        let explicit = [vec!["compare"], args.clone()].concat();
        assert_eq!(parse(&explicit, &[])["exit_code"], 2, "{explicit:?}");
        let view = [&args[..2], &["-o", "out.md"], &args[4..]].concat();
        assert_eq!(parse(&view, &[])["exit_code"], 0, "{view:?}");
    }
    assert_eq!(
        parse(&["a.docx", "b.docx", "-o", "out.docx"], &[])["exit_code"],
        0
    );
}

/// Task 13: `compare A B` is the shorthand `A B`: the view is its output
/// without -o and with a .md (unless both documents are Markdown, whose .md
/// is CriticMarkup); any other -o has no view.
#[test]
fn explicit_compare_and_alias_print_the_view_like_the_shorthand() {
    for task in ["compare", "redline"] {
        let result = parse(&[task, "-b", "a.docx", "-m", "b.docx"], &[]);
        assert_eq!(result["exit_code"], 0, "{result}");
        let shorthand = parse(&["-b", "a.docx", "-m", "b.docx"], &[]);
        assert_eq!(
            result["args"]["view"], shorthand["args"]["view"],
            "{result}"
        );
        assert!(result["args"]["view"].is_object(), "{result}");
        let head = parse(&[task, "a.docx", "b.docx", "--head", "2"], &[]);
        assert_eq!(head["args"]["view"]["head"], 2, "{head}");
        assert_eq!(parse(&[task, "-b", "a.docx"], &[])["exit_code"], 2);
    }
    for (args, view) in [
        (&["compare", "a.docx", "b.docx", "-o", "r.docx"][..], false),
        (&["compare", "a.docx", "b.docx", "-o", "r.pdf"], false),
        (&["compare", "a.docx", "b.docx", "-o", "r.json"], false),
        (&["compare", "a.docx", "b.docx", "-o", "r.tmp"], false),
        (&["compare", "a.docx", "b.docx", "-o", "v.md"], true),
        (&["compare", "a.md", "b.docx", "-o", "v.md"], true),
        // A .md's view or CriticMarkup is the host's to decide once it has
        // read both documents (names may not say Markdown).
        (&["compare", "a.md", "b.md", "-o", "c.md"], true),
        (&["compare", "a.md", "b.md"], true),
    ] {
        let result = parse(args, &[]);
        assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
        assert_eq!(
            result["args"]["view"].is_object(),
            view,
            "{args:?}: {result}"
        );
    }
    assert_eq!(
        parse(&["compare", "a.docx", "b.docx", "-o", "r.tmp"], &[])["args"]["output_format"],
        "docx"
    );
    // Whether the command line names a view option, at its default too: a
    // .md of two Markdown documents is CriticMarkup, which none shapes.
    for (args, given) in [
        (&["compare", "a.md", "b.md", "-o", "c.md"][..], false),
        (
            &[
                "compare",
                "a.md",
                "b.md",
                "-o",
                "c.md",
                "--comments",
                "inline",
            ],
            true,
        ),
        (&["a", "b", "--head", "1"], true),
        (&["a", "b"], false),
    ] {
        let result = parse(args, &[]);
        assert_eq!(result["args"]["view_given"], given, "{args:?}: {result}");
        let native = Cli::try_parse_from([&["jubarte"][..], args].concat()).unwrap();
        let compare = match native.command {
            Some(Command::Compare(compare)) => compare,
            _ => native.compare,
        };
        assert_eq!(compare.view_given, given, "{args:?}");
    }
    // A host without read takes CriticMarkup, never the view.
    assert_eq!(
        parse(&["compare", "a.md", "b.md", "-o", "c.md"], &["compare"])["exit_code"],
        0
    );
    // `jubarte FILE` is read: compare's own options and the rendering ones
    // need a second document.
    for (flags, message) in [
        (
            &["--author", "Legal"][..],
            "--author is a compare option: name two documents",
        ),
        (&["--critic"], "--critic is a compare option"),
        (&["--mode", "powertools"], "--mode is a compare option"),
        (
            &["--dpi", "50"],
            "--dpi draws a PDF or PNG, which read does not write",
        ),
        (&["--changed-pages"], "--changed-pages draws a PDF or PNG"),
    ] {
        let result = parse(&[&["a.docx"][..], flags].concat(), &[]);
        assert_eq!(result["exit_code"], 2, "{flags:?}: {result}");
        assert!(
            result["text"].as_str().unwrap().contains(message),
            "{result}"
        );
    }
    let read = parse(&["placeholder.docx", "--original", "actual.docx"], &[]);
    assert_eq!(read["exit_code"], 0, "{read}");
    assert_eq!(read["args"]["file"], "actual.docx");
}

/// `-p r1 --accept`, `-p header1.r2 --reject` and `-p c5 --reply TEXT` group
/// with their -p like every other operation flag.
#[test]
fn edit_and_add_group_accept_reject_and_reply_by_their_location() {
    let ops = |argv: &[&str]| -> Vec<jubarte::edit::flags::FlagOp> {
        match Cli::try_parse_from(argv).unwrap().command {
            Some(Command::Edit { operations, .. } | Command::Add { operations, .. }) => operations,
            other => panic!("{other:?}"),
        }
    };
    let edit = ops(&[
        "jubarte",
        "edit",
        "a.docx",
        "-p",
        "r1",
        "--accept",
        "-p",
        "header1.r2",
        "--reject",
        "-p",
        "c5",
        "--reply",
        "Yes.",
    ]);
    assert_eq!(
        edit.iter()
            .map(|o| (o.at.as_str(), o.accept, o.reject, o.reply.as_deref()))
            .collect::<Vec<_>>(),
        [
            ("r1", true, false, None),
            ("header1.r2", false, true, None),
            ("c5", false, false, Some("Yes.")),
        ]
    );
    let add = ops(&["jubarte", "add", "a.docx", "-p", "c5", "--reply", "Yes."]);
    assert_eq!(add[0].reply.as_deref(), Some("Yes."));
    for argv in [
        &[
            "jubarte", "edit", "a.docx", "-p", "c5", "--reply", "x", "--reply", "y",
        ][..],
        &["jubarte", "edit", "a.docx", "--accept", "-p", "r1"][..],
        &["jubarte", "edit", "a.docx", "--plan", "p.json", "--accept"][..],
        &["jubarte", "add", "a.docx", "-p", "r1", "--accept"][..],
        &["jubarte", "add", "a.docx"][..],
        &[
            "jubarte",
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--delete",
            "--consume",
        ][..],
        &[
            "jubarte",
            "add",
            "a.docx",
            "-p",
            "p1",
            "--content",
            "x",
            "--consume",
        ][..],
    ] {
        assert!(Cli::try_parse_from(argv).is_err(), "{argv:?}");
    }
    // --plan with operation flags records them; alone it applies the plan,
    // for add too, and --consume rides along.
    for argv in [
        &[
            "jubarte",
            "edit",
            "a.docx",
            "--plan",
            "p.json",
            "-p",
            "p1",
            "--content",
            "x",
            "--author",
            "Ann",
        ][..],
        &[
            "jubarte",
            "add",
            "a.docx",
            "--plan",
            "p.json",
            "-p",
            "p1",
            "--comment",
            "--content",
            "x",
        ][..],
        &["jubarte", "edit", "a.docx", "--plan", "p.json", "--consume"][..],
        &["jubarte", "add", "a.docx", "--plan", "p.json", "--consume"][..],
    ] {
        assert!(Cli::try_parse_from(argv).is_ok(), "{argv:?}");
    }
    let twice = Cli::try_parse_from([
        "jubarte", "edit", "a.docx", "-p", "c5", "--reply", "x", "--reply", "y",
    ])
    .unwrap_err()
    .to_string();
    assert!(twice.contains("-p c5: --reply given twice"), "{twice}");
}

/// D1/D2: `-o`'s extension decides, `--to` fills a missing or unknown one,
/// a disagreement is an error, and each task writes only its own formats.
#[test]
fn the_output_rule_is_one_table() {
    use Format::{Docx, Json, Md, Pdf, Png};
    let rule = |task, output: Option<&str>, to| output_format(task, output.map(Path::new), to);
    let all = [
        Task::Compare,
        Task::Convert,
        Task::Read,
        Task::Diff,
        Task::Accept,
        Task::Reject,
        Task::Append,
        Task::Scrub,
    ];
    for task in all {
        assert_eq!(rule(task, None, None), Ok(None), "{task:?}");
        assert_eq!(Task::of_name(task.name()), Some(task));
    }
    assert_eq!(Task::of_name("redline"), Some(Task::Compare));
    assert_eq!(Task::of_name("text"), Some(Task::Read));
    assert_eq!(Task::of_name("inspect"), Some(Task::Read));
    assert_eq!(Task::of_name("edit"), None);
    for task in [
        Task::Compare,
        Task::Accept,
        Task::Reject,
        Task::Append,
        Task::Scrub,
    ] {
        for (path, format) in [
            ("x.docx", Docx),
            ("x.md", Md),
            ("x.txt", Md),
            ("x.pdf", Pdf),
            ("x.PNG", Png),
            ("x.json", Json),
            ("x.tmp", Docx),
            ("x", Docx),
        ] {
            assert_eq!(
                rule(task, Some(path), None),
                Ok(Some(format)),
                "{task:?} {path}"
            );
        }
    }
    // convert
    assert_eq!(rule(Task::Convert, Some("x.tmp"), Some(Png)), Ok(Some(Png)));
    assert_eq!(rule(Task::Convert, Some("x.pdf"), Some(Pdf)), Ok(Some(Pdf)));
    assert_eq!(rule(Task::Convert, None, Some(Png)), Ok(Some(Png)));
    assert_eq!(rule(Task::Convert, Some("x.tmp"), None), Ok(None));
    assert_eq!(
        rule(Task::Convert, Some("x.png"), Some(Pdf)),
        Err("-o x.png says png and --to says pdf: drop one".to_string())
    );
    // read
    for (path, format) in [("v.md", Md), ("v.tmp", Md), ("v.json", Json)] {
        assert_eq!(
            rule(Task::Read, Some(path), None),
            Ok(Some(format)),
            "{path}"
        );
    }
    for ext in ["docx", "pdf", "png"] {
        assert_eq!(
            rule(Task::Read, Some(&format!("v.{ext}")), None),
            Err(format!(
                "read writes .md or .json; `convert FILE -o v.{ext}` writes {ext}"
            ))
        );
    }
    // diff
    for path in ["d.md", "d.txt", "d.patch", "d.DIFF", "d.markdown", "d.tmp"] {
        assert_eq!(rule(Task::Diff, Some(path), None), Ok(Some(Md)), "{path}");
    }
    for ext in ["docx", "pdf", "png"] {
        assert_eq!(
            rule(Task::Diff, Some(&format!("d.{ext}")), None),
            Err(format!(
                "diff writes text (.md, .markdown, .txt, .patch, .diff); `compare OLD NEW -o d.{ext}` writes the redline as {ext}"
            ))
        );
    }
    // The advice names the path as typed.
    assert!(
        rule(Task::Diff, Some("out/changes.pdf"), None)
            .unwrap_err()
            .contains("`compare OLD NEW -o out/changes.pdf`")
    );
    assert_eq!(
        rule(Task::Diff, Some("d.json"), None),
        Err("diff does not write .json yet".to_string())
    );
    assert!(
        rule(Task::Diff, None, Some(Docx))
            .unwrap_err()
            .contains("-o c.docx")
    );
    assert_eq!(Format::of_path(Path::new("a.json")), Some(Json));
    // pi-07 F1: a name that is only an extension says it.
    assert_eq!(Format::of_path(Path::new(".json")), Some(Json));
    assert_eq!(Format::of_path(Path::new("out/.md")), Some(Md));
    assert_eq!(Format::of_path(Path::new(".hidden")), None);
    assert_eq!(
        rule(Task::Diff, Some(".json"), None),
        rule(Task::Diff, Some("d.json"), None)
    );
    assert_eq!(rule(Task::Read, Some(".json"), None), Ok(Some(Json)));
    assert_eq!(rule(Task::Compare, Some("out/.md"), None), Ok(Some(Md)));
    assert_eq!(rule(Task::Diff, Some(".patch"), None), Ok(Some(Md)));
}

/// pi-07 F2, F3: until a task writes the snapshot, its `.json` output exits
/// 2 in the shared parser, so the binary and both bindings refuse alike.
/// Diff still refuses it; the writing tasks, convert and read write it
/// (Task 12), and compare (Task 13).
#[test]
fn json_output_is_refused_until_a_task_writes_it() {
    for args in [
        vec!["diff", "a.docx", "b.docx", "-t", "json"],
        vec!["diff", "a.docx", "b.docx", "-o", ".json"],
    ] {
        let task = if args[0].ends_with(".docx") {
            "compare"
        } else {
            args[0]
        };
        let result = parse(&args, &[task]);
        assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
        let text = result["text"].as_str().unwrap();
        assert!(
            text.contains(&format!("{task} does not write .json yet")),
            "{args:?}: {text}"
        );
    }
    for (args, task) in [
        (vec!["convert", "a.docx", "--to", "json"], "convert"),
        (vec!["convert", "a.md", "-o", "a.json"], "convert"),
        (vec!["accept", "a.docx", "-o", "a.json"], "accept"),
        (vec!["reject", "a.docx", "-o", "a.json"], "reject"),
        (vec!["scrub", "a.docx", "-o", "a.json"], "scrub"),
        (vec!["append", "a.docx", "b.docx", "-o", "a.json"], "append"),
        (vec!["read", "a.docx", "-o", "a.json"], "read"),
        (vec!["a.docx", "-o", "a.json"], "compare"),
        (
            vec!["compare", "a.docx", "b.docx", "-o", "c.JSON"],
            "compare",
        ),
        (vec!["a.docx", "b.docx", "-o", "c.json"], "compare"),
    ] {
        // The one-document shorthand needs compare's positionals (and read).
        let supported: &[&str] = if task == "compare" { &[] } else { &[task] };
        let result = parse(&args, supported);
        assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
        assert_eq!(result["args"]["output_format"], "json", "{args:?}");
    }
    let result = parse(&["convert", "a.docx", "--to", "md"], &["convert"]);
    assert_eq!(result["exit_code"], 0, "{result}");
}

/// Task 12: `-o` decides what accept, reject, append, scrub and read write;
/// the render options go with a PDF or PNG only; read writes the view or
/// the snapshot, and the one-document shorthand is read.
#[test]
fn every_writing_task_takes_the_formats_its_output_names() {
    for (ext, format) in [
        ("docx", "docx"),
        ("md", "md"),
        ("pdf", "pdf"),
        ("png", "png"),
        ("json", "json"),
        ("tmp", "docx"),
    ] {
        let file = format!("r.{ext}");
        for task in ["accept", "reject", "scrub"] {
            let result = parse(&[task, "a.docx", "-o", &file], &[]);
            assert_eq!(
                result["args"]["output_format"], format,
                "{task} {file}: {result}"
            );
        }
        let result = parse(&["append", "a.docx", "b.docx", "-o", &file], &[]);
        assert_eq!(
            result["args"]["output_format"], format,
            "append {file}: {result}"
        );
    }
    let read = parse(&["read", "a.docx", "-o", "v.md"], &[]);
    assert_eq!(read["args"]["output_format"], "md", "{read}");
    let read = parse(&["read", "a.docx"], &[]);
    assert_eq!(read["args"]["output_format"], serde_json::Value::Null);
    let single = parse(&["a.docx", "-o", "v.md", "--head", "1"], &[]);
    assert_eq!(single["exit_code"], 0, "{single}");
    assert_eq!(single["command"], "read");
    assert_eq!(single["args"]["output"], "v.md");
    assert_eq!(single["args"]["output_format"], "md");
    for (args, says) in [
        (
            vec!["read", "a.docx", "-o", "v.pdf"],
            "read writes .md or .json",
        ),
        (
            vec!["read", "a.docx", "-o", "v.docx"],
            "convert FILE -o v.docx",
        ),
        (vec!["a.docx", "-o", "v.docx"], "read writes .md or .json"),
        (
            vec!["read", "a.docx", "-o", "v.md", "--json"],
            "--json writes the snapshot",
        ),
        (
            vec!["read", "a.docx", "-o", "v.json", "--tables"],
            "--tables writes the grids as text",
        ),
        (
            vec!["accept", "a.docx", "-o", "r.docx", "--changed-pages"],
            "--changed-pages applies to PDF or PNG output only",
        ),
        (
            vec!["reject", "a.docx", "-o", "r.md", "--pages", "1"],
            "--pages applies to PDF or PNG output only",
        ),
        (
            vec!["scrub", "a.docx", "-o", "r.json", "--dpi", "50"],
            "--dpi applies to PDF or PNG output only",
        ),
        (
            vec![
                "append",
                "a.docx",
                "b.docx",
                "-o",
                "r.docx",
                "--revisions",
                "word",
            ],
            "--revisions applies to PDF or PNG output only",
        ),
    ] {
        let result = parse(&args, &[]);
        assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
        let text = result["text"].as_str().unwrap();
        assert!(
            text.contains(says) && text.contains("Usage:"),
            "{args:?}: {text}"
        );
    }
    for args in [
        vec![
            "accept",
            "a.docx",
            "-o",
            "r.pdf",
            "--changed-pages",
            "--pages",
            "1",
        ],
        vec!["scrub", "a.docx", "-o", "r.png", "--dpi", "50"],
    ] {
        assert_eq!(parse(&args, &[])["exit_code"], 0, "{args:?}");
    }
}

/// Every task with a long `--output` has a row in the `-o` table.
#[test]
fn every_output_flag_has_a_row_in_the_table() {
    use clap::CommandFactory;
    for sub in Cli::command().get_subcommands() {
        if sub.get_arguments().any(|a| a.get_long() == Some("output")) {
            let name = sub.get_name();
            assert!(
                jubarte::capabilities::OUTPUT_FORMATS
                    .iter()
                    .any(|(t, _)| *t == name),
                "{name} writes -o but has no OUTPUT_FORMATS row"
            );
        }
    }
}

#[test]
fn diff_render_is_visual_diff_with_a_note() {
    let old = parse(&["diff-render", "a.docx", "b.docx"], &[]);
    assert_eq!(old["exit_code"], 0, "{old}");
    assert_eq!(old["command"], "visual-diff");
    assert_eq!(
        old["notes"][0],
        "note: diff-render is now visual-diff; diff-render is removed in 0.14"
    );
    assert_eq!(old["notes"].as_array().unwrap().len(), 1);
    let new = parse(&["visual-diff", "a.docx", "b.docx"], &[]);
    assert_eq!(new["command"], "visual-diff");
    assert_eq!(new["notes"], serde_json::json!([]));
    // An adapter lists the canonical name; the old one follows it.
    let facade = parse(&["diff-render", "a", "b"], &["visual-diff"]);
    assert_eq!(facade["command"], "visual-diff", "{facade}");
}

#[test]
fn quiet_is_global_before_or_after_the_task() {
    for argv in [
        &["compare", "a", "b", "-q"][..],
        &["a.docx", "b.docx", "--quiet"],
        &["edit", "a", "--plan", "p", "-q"],
        &["append", "a", "b", "-o", "c.docx", "-q"],
        &["read", "a.docx", "-q"],
        &["a.docx", "-q"],
    ] {
        let parsed = parse(argv, &[]);
        assert_eq!(parsed["exit_code"], 0, "{argv:?}: {parsed}");
        assert_eq!(parsed["args"]["quiet"], true, "{argv:?}: {parsed}");
    }
    let loud = parse(&["compare", "a", "b"], &[]);
    assert_eq!(loud["args"]["quiet"], false);
    // A facade without compare still takes the global flag.
    let facade = parse(&["edit", "a", "--plan", "p", "-q"], &["edit"]);
    assert_eq!(facade["args"]["quiet"], true, "{facade}");
    // -q also goes before the task (D6 follow-up): it moves after it.
    for argv in [
        &["-q", "read", "a.docx"][..],
        &["--quiet", "compare", "a", "b"],
        &["-q", "validate", "a.docx", "--rules", "style"],
    ] {
        let parsed = parse(argv, &[]);
        assert_eq!(parsed["exit_code"], 0, "{argv:?}: {parsed}");
        assert_eq!(parsed["args"]["quiet"], true, "{argv:?}: {parsed}");
    }
    let native = jubarte::cli::Cli::try_parse_from(["jubarte", "-q", "read", "a.docx"]).unwrap();
    assert!(native.quiet);
    // The bare shorthand takes a leading -q too, and saying it twice
    // (an alias plus an explicit -q) is still just quiet.
    for argv in [
        &["-q", "a.docx", "b.docx", "-o", "r.docx"][..],
        &["-q", "-q", "read", "a.docx"],
        &["-q", "--quiet", "read", "a.docx"],
        &["-qq", "read", "a.docx"],
        &["read", "a.docx", "-q", "-q"],
    ] {
        let parsed = parse(argv, &[]);
        assert_eq!(parsed["exit_code"], 0, "{argv:?}: {parsed}");
        assert_eq!(parsed["args"]["quiet"], true, "{argv:?}: {parsed}");
    }
    // -q before `help` prints the help, as `help` alone does, instead of
    // advising `help -q`, which clap refuses.
    for (quiet, plain) in [
        (&["-q", "help"][..], &["help"][..]),
        (&["-q", "help", "read"], &["help", "read"]),
    ] {
        let (quiet, plain) = (parse(quiet, &[]), parse(plain, &[]));
        assert_eq!(quiet["exit_code"], 0, "{quiet}");
        assert_eq!(quiet["text"], plain["text"], "{quiet}");
    }
    // So does a facade without compare: one rule for every binding.
    let facade_early = parse(&["-q", "edit", "a", "--plan", "p"], &["edit"]);
    assert_eq!(facade_early["exit_code"], 0, "{facade_early}");
    assert_eq!(facade_early["args"]["quiet"], true, "{facade_early}");
    // Every other option still goes after the task.
    let early = parse(&["--head", "2", "read", "a.docx"], &[]);
    assert_eq!(early["exit_code"], 2);
    assert!(
        early["text"]
            .as_str()
            .unwrap()
            .contains("options go after the task"),
        "{early}"
    );
}

/// Codex on #437: help under a retired name still says it is retired, in
/// the binary and in the hosts (the parser's help object carries the notes).
#[test]
fn a_retired_names_help_carries_its_note() {
    let help = parse(&["diff-render", "--help"], &[]);
    assert_eq!(help["exit_code"], 0, "{help}");
    assert!(
        help["notes"][0]
            .as_str()
            .is_some_and(|n| n.contains("diff-render") && n.contains("visual-diff")),
        "{help}"
    );
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["diff-render", "--help"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("visual-diff"), "{stderr}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("Usage:"));
}

#[test]
fn retired_spellings_are_scoped_and_listed_once() {
    let notes = |argv: &[&str]| retired_spellings(argv.iter().copied());
    assert_eq!(notes(&["edit", "a", "--plan", "p", "--pdf"]), vec![]);
    assert_eq!(notes(&["visual-diff", "a", "b"]), vec![]);
    assert_eq!(
        notes(&["diff-render", "a", "b", "--json"]),
        vec![("diff-render", "visual-diff")]
    );
    // Only the task position names a task.
    assert_eq!(notes(&["read", "diff-render"]), vec![]);
    assert_eq!(notes(&["--", "diff-render"]), vec![]);
    // An option's value is not the task.
    assert_eq!(notes(&["-o", "diff-render", "a", "b"]), vec![]);
    assert_eq!(notes(&["--author", "diff-render", "a", "b"]), vec![]);
    // A flag takes no value, so the token after it is the task.
    assert_eq!(
        notes(&["-q", "diff-render", "a", "b"]),
        vec![("diff-render", "visual-diff")]
    );
    assert_eq!(
        notes(&["visual-diff", "-o", "edit", "a", "b"]),
        vec![],
        "a value after the task is not a task either"
    );
    // 0.12's cell form is expired, not retired: the id parsers refuse it
    // with its settled spelling, so it earns no note. Every spelling of the
    // -p value is still skipped as a value (a short cluster, `=`, after a
    // flag that takes none).
    for argv in [
        &["edit", "a", "-p", "t0.r1.c2", "--content", "x"][..],
        &["edit", "a", "-pt0.r1.c2.p1", "--content", "x"],
        &[
            "add",
            "a",
            "--location=t1.r0.c0",
            "--comment",
            "--content",
            "x",
        ],
        &["edit", "a", "-qp", "t0.r1.c2", "--content", "x"],
        &["edit", "a", "-qpt0.r1.c2", "--content", "x"],
        &["edit", "--content", "x", "a", "-p", "t0.r1.c2"],
        &["edit", "a", "--delete", "-p", "t0.r1.c2"],
        &["read", "a", "-p", "t0.r1.c2"],
    ] {
        assert_eq!(notes(argv), vec![], "{argv:?}");
    }
    // A removed spelling (--date for --datetime) is found as a retired one
    // is, and takes its value.
    assert_eq!(
        jubarte::cli::removed_spellings([
            "edit",
            "a",
            "--date",
            "2026-10-12",
            "-p",
            "t0.r1.c2",
            "--content",
            "x",
        ]),
        vec![("--date", "--datetime")]
    );
    // A -p value spelled like a retired flag is a value, not the flag.
    assert_eq!(
        notes(&["edit", "a", "-p", "--date", "--content", "x"]),
        vec![]
    );
    assert_eq!(
        retired_note("diff-render", "visual-diff"),
        "note: diff-render is now visual-diff; diff-render is removed in 0.14"
    );
}

/// The `revisions` task is gone (`changes` lists what a document tracks),
/// and its name parses as the shorthand's first file again
/// (`removed_commands_leave_help_and_restore_legacy_filename_parsing`). So
/// `jubarte revisions x.docx` failed with only "reading revisions: No such
/// file" (pi review of #422): a note now names `changes`, on every host.
/// An explicit `compare`, or the name in second place, earns no note.
#[test]
fn the_removed_revisions_task_names_changes() {
    let note = "note: revisions is no longer a task, so it reads as a file name; use changes";
    for args in [
        &["revisions", "x.docx"][..],
        &["-q", "revisions", "x.docx"][..],
    ] {
        let result = parse(args, &[]);
        assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
        assert_eq!(
            result["args"]["original"], "revisions",
            "{args:?}: {result}"
        );
        assert_eq!(
            result["notes"],
            serde_json::json!([note]),
            "{args:?}: {result}"
        );
    }
    // `revisions FILE --json` is no compare (the legacy-filename test pins
    // exit 2); the note goes with the error, which hosts print first.
    let result = parse(&["revisions", "x.docx", "--json"], &[]);
    assert_eq!(result["exit_code"], 2, "{result}");
    assert_eq!(result["notes"], serde_json::json!([note]), "{result}");
    for args in [
        &["compare", "revisions", "b.docx"][..],
        &["a.docx", "revisions"][..],
    ] {
        let result = parse(args, &[]);
        assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
        assert_eq!(result["notes"], serde_json::json!([]), "{args:?}: {result}");
    }
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["revisions", "x.docx"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with(&format!("{note}\n")), "{stderr}");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["revisions", "x.docx", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with(&format!("{note}\n")), "{stderr}");
}

/// A retired task name is the task while it lasts, not a file of that
/// name: `text other.docx` reads other.docx (`read text` reads a file
/// called `text`). Ledger review of #446, INFO-4.
#[test]
fn a_retired_task_name_is_the_task_not_a_file() {
    let result = parse(&["text", "other.docx"], &[]);
    assert_eq!(result["command"], "read", "{result}");
    assert_eq!(result["args"]["file"], "other.docx", "{result}");
    let result = parse(&["read", "text"], &[]);
    assert_eq!(result["args"]["file"], "text", "{result}");
    assert_eq!(result["notes"], serde_json::json!([]), "{result}");
}

/// A retired spelling's note goes with a usage error too: the run that
/// fails is the one that needs to hear the new name (ledger reviews of
/// #444, C1 and H2, and #446, 4 and 5). Hosts print `notes` before `text`.
#[test]
fn a_retired_spelling_is_noted_when_the_parse_fails() {
    let changed = "note: --changed-only is now --changed-pages; --changed-only is removed in 0.14";
    let audit =
        "note: audit is now validate --rules a11y,style,structure; audit is removed in 0.14";
    // With --rules, audit's own sets are not what runs (ledger review of
    // #446, 3).
    let audit_rules = "note: audit is now validate; audit is removed in 0.14";
    for (args, note) in [
        (
            &[
                "compare",
                "a.docx",
                "b.docx",
                "-o",
                "out.docx",
                "--changed-only",
            ][..],
            changed,
        ),
        (
            &["convert", "a.docx", "-o", "out.docx", "--changed-only"][..],
            changed,
        ),
        (&["audit", "a.docx", "--rules", "edits"][..], audit_rules),
        (&["audit", "a.docx", "--rules=bogus"][..], audit_rules),
        (&["audit", "a.docx", "--original"][..], audit),
    ] {
        let result = parse(args, &[]);
        assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
        assert_eq!(
            result["notes"],
            serde_json::json!([note]),
            "{args:?}: {result}"
        );
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.starts_with(&format!("{note}\nerror: ")),
            "{args:?}: {stderr}"
        );
    }
}

/// Arthur (2026-10-13): one spelling for the timestamp flag: `-d,
/// --datetime` on edit, add, compare, diff and convert (md->docx). Arthur
/// (2026-10-14): `--date` is gone, not an alias; the error names
/// `--datetime`.
#[test]
fn timestamp_flag_is_d_datetime_and_date_is_gone() {
    let date = "2026-10-01T09:00:00Z";
    let bases: &[&[&str]] = &[
        &["edit", "a.docx", "-p", "p0", "--content", "x"],
        &["add", "a.docx", "-p", "p0", "--content", "x"],
        &["compare", "a.docx", "b.docx"],
        &["diff", "a.md", "b.md"],
        &["convert", "a.md", "-t", "docx"],
    ];
    for base in bases {
        for flag in ["-d", "--datetime"] {
            let args = [base.to_vec(), vec![flag, date]].concat();
            let result = parse(&args, &[]);
            assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
            assert_eq!(
                result["args"]["datetime"].as_str(),
                Some(date),
                "{args:?}: {result}"
            );
            // Current flags earn no note.
            assert_eq!(result["notes"], serde_json::json!([]), "{args:?}: {result}");
        }
        let args = [base.to_vec(), vec!["--date", date]].concat();
        let result = parse(&args, &[]);
        assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
        assert!(
            result["text"]
                .as_str()
                .is_some_and(|text| text.contains("--datetime")),
            "{args:?}: {result}"
        );
    }
    // Before the task too, where clap would suggest compare's `--dates`.
    let inline = format!("--date={date}");
    for args in [
        &["--date", date, "compare", "a.docx", "b.docx"][..],
        &["--date", date, "a.docx", "b.docx"][..],
        &[inline.as_str(), "a.docx", "b.docx"][..],
    ] {
        let result = parse(args, &[]);
        assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
        let text = result["text"].as_str().unwrap_or_default();
        assert!(
            text.contains("use --datetime") && !text.contains("--dates"),
            "{args:?}: {result}"
        );
    }
    // No task's `--help` shows `--date`.
    for task in ["edit", "add", "compare", "diff", "convert"] {
        let help = parse(&[task, "--help"], &[]);
        let text = help["text"].as_str().unwrap();
        assert!(text.contains("--datetime"), "{task}: {text}");
        assert!(
            !text.contains("--date ") && !text.contains("--date\n"),
            "{task}: {text}"
        );
    }
}

/// `-a, --author` on every task that writes changes: edit and add take the
/// short form as compare, diff and convert do.
#[test]
fn edit_and_add_take_dash_a_for_the_author() {
    for base in [
        &["edit", "a.docx", "-p", "p0", "--content", "x"][..],
        &["add", "a.docx", "-p", "p0", "--content", "x"][..],
    ] {
        for flag in ["-a", "--author"] {
            let args = [base.to_vec(), vec![flag, "Ann Lee"]].concat();
            let result = parse(&args, &[]);
            assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
            assert_eq!(
                result["args"]["author"].as_str(),
                Some("Ann Lee"),
                "{args:?}: {result}"
            );
        }
    }
}

#[test]
fn writer_status_lines_preserve_paths_and_pluralize_page_counts() {
    use jubarte::cli::{wrote_line, wrote_png_line};

    for (format, name) in [
        (Format::Docx, "docx"),
        (Format::Md, "md"),
        (Format::Json, "json"),
    ] {
        assert_eq!(
            wrote_line(Path::new("out/ação final"), format, 0, None),
            format!("wrote out/ação final ({name}, 0 bytes)")
        );
    }
    for (pages, label) in [(0, "0 pages"), (1, "1 page"), (2, "2 pages")] {
        assert_eq!(
            wrote_line(Path::new("r.pdf"), Format::Pdf, 123, Some(pages)),
            format!("wrote r.pdf (pdf, 123 bytes, {label})")
        );
        assert_eq!(
            wrote_png_line(Path::new("out/r-page-NN.png"), pages, 72.5),
            format!("wrote out/r-page-NN.png (png, {label}, 72.5 dpi)")
        );
    }
}

#[test]
fn all_writer_render_flags_require_a_rendered_output_even_at_the_default() {
    let flags: &[&[&str]] = &[
        &["--revisions", "conventional"],
        &[
            "--revisions",
            "custom",
            "--revision-palette",
            "inserted=#112233",
        ],
        &["--dpi", "96"],
        &["--move-comments"],
        &["--changed-pages"],
        &["--pages", "1"],
    ];
    for task in ["accept", "reject", "append", "scrub"] {
        for output in ["r.docx", "r.md", "r.json", "r.unknown", "r"] {
            let mut base = vec![task, "a.docx"];
            if task == "append" {
                base.push("b.docx");
            }
            base.extend(["-o", output]);
            let defaults = parse(&base, &[]);
            assert_eq!(defaults["exit_code"], 0, "{base:?}: {defaults}");
            for flag in flags {
                let args = [base.as_slice(), flag].concat();
                let result = parse(&args, &[]);
                assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
                assert_eq!(result["stream"], "stderr");
                assert!(
                    result["text"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("{} applies to PDF or PNG output only", flag[0])),
                    "{args:?}: {result}"
                );
            }
        }
    }
}

#[test]
fn writer_render_options_survive_shared_parser_serialization() {
    for task in ["accept", "reject", "append", "scrub"] {
        for output in ["result.PDF", "result.PNG"] {
            let mut args = vec![task, "a.docx"];
            if task == "append" {
                args.push("b.docx");
            }
            args.extend([
                "-o",
                output,
                "--revisions",
                "custom",
                "--revision-palette",
                "inserted=#112233:underline",
                "--dpi",
                "72.5",
                "--move-comments",
                "--changed-pages",
                "--pages",
                "1-3,7",
                "--force",
                "-q",
            ]);
            let result = parse(&args, &[task]);
            assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
            let values = &result["args"];
            assert_eq!(values["output"], output);
            assert_eq!(
                values["output_format"],
                if output.ends_with("PDF") {
                    "pdf"
                } else {
                    "png"
                }
            );
            assert_eq!(values["revisions"], "custom");
            assert_eq!(values["revision_palette"], "inserted=#112233:underline");
            assert_eq!(values["dpi"], 72.5);
            assert_eq!(values["move_comments"], true);
            assert_eq!(values["changed_pages"], true);
            assert_eq!(values["pages"], "1-3,7");
            assert_eq!(values["force"], true);
            assert_eq!(values["quiet"], true);
        }
    }
}

#[test]
fn read_output_conflicts_apply_to_alias_and_shorthand_too() {
    for prefix in [&["read", "a.docx"][..], &["text", "a.docx"], &["a.docx"]] {
        // A .json snapshot is --json's (#1255): --track-changes settles its
        // text, and the view's other options change nothing in it.
        for flags in [
            vec!["--track-changes", "all"],
            vec!["--track-changes", "rejected"],
            vec!["--comments", "inline"],
            vec!["--dates"],
            vec!["--no-page-markers"],
        ] {
            let args = [prefix, &["-o", "v.json"], &flags].concat();
            let result = parse(&args, &[]);
            assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
            assert_eq!(result["args"]["output_format"], "json", "{args:?}");
        }
        for (output, flags, message) in [
            (
                "v.json",
                vec!["--tables"],
                "--tables writes the grids as text",
            ),
            ("v.unknown", vec!["--json"], "--json writes the snapshot"),
            ("v.png", vec![], "read writes .md or .json"),
        ] {
            let args = [prefix, &["-o", output], &flags].concat();
            let result = parse(&args, &[]);
            assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
            assert_eq!(result["stream"], "stderr");
            assert!(
                result["text"].as_str().unwrap().contains(message),
                "{args:?}: {result}"
            );
        }
        // --tables -o FILE writes the grids it prints (#1690).
        let args = [prefix, &["-o", "v.md", "--tables"]].concat();
        let result = parse(&args, &[]);
        assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
        assert_eq!(result["args"]["output_format"], "md", "{args:?}");
        assert_eq!(result["args"]["tables"], true, "{args:?}");
        // Implicit view defaults must not reject JSON; explicit --json is redundant but valid.
        let args = [
            prefix,
            &["-o", "v.JSON", "--json", "--head", "1", "--force", "-q"],
        ]
        .concat();
        let result = parse(&args, &[]);
        assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
        assert_eq!(result["command"], "read");
        assert_eq!(result["args"]["output_format"], "json");
        assert_eq!(result["args"]["head"], 1);
        assert_eq!(result["args"]["force"], true);
        assert_eq!(result["args"]["quiet"], true);
    }
}

/// A task the host lacks is one usage error, whatever follows it: before,
/// `visual-diff` alone was "not a task", with a file "not supported by this
/// adapter", and with two files a clap conflict (ledger review of #454,
/// finding 8).
#[test]
fn a_task_the_host_lacks_is_one_usage_error_whatever_follows() {
    let npm = [
        "compare",
        "diff",
        "changes",
        "accept",
        "reject",
        "read",
        "convert",
        "edit",
        "add",
        "capabilities",
    ];
    for (task, args) in [
        ("visual-diff", &["visual-diff"][..]),
        ("visual-diff", &["visual-diff", "a.docx"]),
        ("visual-diff", &["visual-diff", "a.docx", "b.docx"]),
        ("scrub", &["-q", "scrub", "a.docx", "-o", "b.docx"]),
        ("validate", &["validate", "a.docx"]),
        ("comments", &["comments", "a.docx", "--json"]),
        ("append", &["append", "a.docx", "b.docx", "-o", "c.docx"]),
    ] {
        let result = parse(args, &npm);
        assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
        assert_eq!(result["stream"], "stderr", "{args:?}");
        let text = result["text"].as_str().unwrap();
        let expected =
            format!("error: this jubarte-python has no {task} task; the `jubarte` binary has it\n");
        assert!(text.starts_with(&expected), "{args:?}: {text}");
    }
    // A host with the task parses it as before.
    let result = parse(&["validate", "a.docx"], &[]);
    assert_eq!(result["command"], "validate", "{result}");
}

#[test]
fn typed_edit_actions_have_one_order_independent_grammar() {
    for args in [
        vec!["replace", "f.docx", "--find", "old", "--text", "new"],
        vec![
            "replace",
            "--text",
            "new",
            "--occurrence",
            "2",
            "f.docx",
            "--find",
            "old",
            "--at",
            "p0",
        ],
        vec!["rewrite", "f.docx", "--at", "p0", "--text", ""],
        vec![
            "insert",
            "f.docx",
            "--kind",
            "text",
            "--at",
            "p0",
            "--edge",
            "end",
            "--text-file",
            "exact.txt",
        ],
        vec![
            "insert",
            "f.docx",
            "--kind",
            "paragraph",
            "--at",
            "p0",
            "--position",
            "after",
            "--text",
            "",
        ],
        vec!["delete", "f.docx", "--at", "p0", "--find", "old"],
        vec!["delete", "f.docx", "--at", "p0", "--whole-block"],
        vec!["apply", "f.docx", "--plan", "p.json"],
    ] {
        let argv = [vec!["jubarte", "edit"], args, vec!["--dry-run"]].concat();
        assert!(Cli::try_parse_from(&argv).is_ok(), "{argv:?}");
    }
}

#[test]
fn typed_edit_scope_and_plan_ownership_are_explicit() {
    for args in [
        vec!["replace", "f", "--text", "new"],
        vec![
            "replace",
            "f",
            "--find",
            "old",
            "--text",
            "new",
            "--text-file",
            "t",
        ],
        vec!["rewrite", "f", "--text", "new"],
        vec![
            "rewrite",
            "f",
            "--at",
            "p0",
            "--text",
            "new",
            "--occurrence",
            "1",
        ],
        vec!["delete", "f", "--at", "p0"],
        vec!["delete", "f", "--at", "p0", "--find", "x", "--whole-block"],
        vec![
            "delete",
            "f",
            "--at",
            "p0",
            "--whole-block",
            "--occurrence",
            "1",
        ],
        vec!["insert", "f", "--at", "p0", "--text", "x", "--edge", "end"],
        vec!["insert", "f", "--kind", "text", "--at", "p0", "--text", "x"],
        vec![
            "replace",
            "f",
            "--find",
            "x",
            "--text",
            "y",
            "--occurrence",
            "0",
        ],
        vec!["apply", "f", "--plan", "p", "--consume"],
        vec!["apply", "f", "--plan", "p", "--author", "override"],
        vec![
            "apply",
            "f",
            "--plan",
            "p",
            "--datetime",
            "2026-10-17T00:00:00Z",
        ],
        vec!["apply", "f", "--plan", "p", "--track", "off"],
        vec![
            "apply",
            "f",
            "--plan",
            "p",
            "--existing-revisions",
            "accept",
        ],
        vec!["apply", "f", "--plan", "p", "--accept-change", "r0"],
        vec!["apply", "f", "--plan", "p", "--reject-change", "r0"],
        vec!["apply", "f", "--plan", "p", "--save-plan", "q"],
    ] {
        assert!(
            Cli::try_parse_from(
                [vec!["jubarte", "edit"], args.clone(), vec!["--dry-run"]].concat()
            )
            .is_err(),
            "{args:?}"
        );
    }
    for extra in [
        vec!["-o", "out"],
        vec!["--out-dir", "out"],
        vec!["--report", "r"],
        vec!["--emit", "document"],
        vec!["--expect", "e"],
        vec!["--dry-run"],
    ] {
        let args = [
            vec![
                "jubarte",
                "edit",
                "rewrite",
                "f",
                "--at",
                "p0",
                "--text",
                "x",
                "--save-plan",
                "p",
            ],
            extra,
        ]
        .concat();
        assert!(Cli::try_parse_from(&args).is_err(), "{args:?}");
    }
}

#[test]
fn action_names_remain_legacy_filenames_with_v1_flags() {
    for name in ["replace", "rewrite", "insert", "delete", "apply"] {
        let cli =
            Cli::try_parse_from(["jubarte", "edit", name, "-p", "p0", "--content", "new"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Edit { file, .. }) if file == Path::new(name)));
        let result = parse(&["edit", name, "-p", "p0", "--content", "new"], &["edit"]);
        assert_eq!(result["exit_code"], 0, "{result}");
    }
    let help = Cli::try_parse_from(["jubarte", "edit", "--help"])
        .unwrap_err()
        .to_string();
    assert!(help.contains("./replace"), "{help}");
}

#[test]
fn replace_help_explains_all_matches_overlap_refusal() {
    let error = Cli::try_parse_from(["jubarte", "edit", "replace", "--help"]).unwrap_err();
    assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    let help = error.to_string();
    let text = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        text.contains(
            "--all-matches Replace every candidate; refuses when any two candidates overlap. Requires --expect-count"
        ),
        "{help}"
    );
    assert!(!help.contains("nonoverlapping"), "{help}");
}

#[test]
fn apply_help_names_only_the_ways_apply_publishes() {
    let flat = |action: &str| {
        let help = Cli::try_parse_from(["jubarte", "edit", action, "--help"])
            .unwrap_err()
            .to_string();
        help.split_whitespace().collect::<Vec<_>>().join(" ")
    };
    // apply has no --save-plan: its output is required unless it bundles
    // or dry-runs.
    let apply = flat("apply");
    assert!(
        apply.contains("Primary edited DOCX; required unless bundling or dry-running"),
        "{apply}"
    );
    assert!(!apply.contains("saving"), "{apply}");
    // An inline action saves a plan instead.
    let replace = flat("replace");
    assert!(
        replace.contains("Primary edited DOCX; required unless saving, bundling or dry-running"),
        "{replace}"
    );
}

#[test]
fn typed_edit_help_and_facade_use_the_public_route() {
    let help = Cli::try_parse_from(["jubarte", "edit", "replace", "--help"])
        .unwrap_err()
        .to_string();
    assert!(help.contains("jubarte edit replace"), "{help}");
    assert!(!help.contains("__edit-v2"), "{help}");
    let value = parse(
        &[
            "edit",
            "replace",
            "f.docx",
            "--find",
            "x",
            "--text",
            "y",
            "--dry-run",
        ],
        &["edit"],
    );
    assert_eq!(value["exit_code"], 0, "{value}");
    assert_eq!(value["command"], "edit");
    assert_eq!(value["args"]["action"], "replace");
    assert!(
        Cli::try_parse_from([
            "jubarte",
            "-qq",
            "edit",
            "replace",
            "f.docx",
            "--find",
            "x",
            "--text",
            "y",
            "--dry-run"
        ])
        .is_ok()
    );
}

#[test]
fn review_09_no_file_action_plan_spelling_keeps_valid_legacy_grammar() {
    for action in ["apply", "replace"] {
        let cli = Cli::try_parse_from(["jubarte", "edit", action, "--plan", "p.json"]).unwrap();
        let Some(Command::Edit { file, plan, .. }) = cli.command else {
            panic!("valid legacy grammar wins");
        };
        assert_eq!(file, Path::new(action));
        assert_eq!(plan.as_deref(), Some(Path::new("p.json")));
    }
}

#[test]
fn edit_template_help_names_the_public_route_and_preserves_legacy_filenames() {
    let help = Cli::try_parse_from(["jubarte", "edit", "template", "--help"])
        .unwrap_err()
        .to_string();
    assert!(help.contains("jubarte edit template"), "{help}");
    assert!(!help.contains("__edit-template"), "{help}");
    for forbidden in ["--runs", "run_operation", "--scaffold"] {
        assert!(!help.contains(forbidden), "{help}");
    }
    assert!(
        Cli::try_parse_from([
            "jubarte",
            "edit",
            "template",
            "-p",
            "p0",
            "--content",
            "legacy"
        ])
        .is_ok()
    );
    assert!(
        Cli::try_parse_from([
            "jubarte",
            "edit",
            "./template",
            "-p",
            "p0",
            "--content",
            "legacy"
        ])
        .is_ok()
    );
}

#[test]
fn read_no_page_markers_accepts_structured_outputs_in_the_shared_parser() {
    for prefix in [
        &["read", "source.docx"][..],
        &["text", "source.docx"],
        &["source.docx"],
    ] {
        for output in [
            vec!["--json"],
            vec!["--tables"],
            vec!["--for-edit"],
            vec!["-o", "snap.json"],
        ] {
            let args = [prefix, &output, &["--no-page-markers"]].concat();
            let result = parse(&args, &[]);
            assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
            assert_eq!(
                result["args"]["no_page_markers"], true,
                "{args:?}: {result}"
            );
            assert!(Cli::try_parse_from([vec!["jubarte"], args].concat()).is_ok());
        }
    }
}

#[test]
fn read_view_conflict_usage_is_shared_with_bindings() {
    let result = parse(
        &["source.docx", "--track-changes=all", "--for-edit"],
        &["compare"],
    );
    assert_eq!(result["exit_code"], 2, "{result}");
    assert_eq!(result["stream"], "stderr", "{result}");
    for prefix in [
        &["read", "source.docx"][..],
        &["text", "source.docx"],
        &["source.docx"],
    ] {
        for flags in [
            vec!["--comments=inline"],
            vec!["--track-changes=all"],
            vec!["--dates"],
        ] {
            for output in [
                vec!["--json"],
                vec!["--tables"],
                vec!["--for-edit"],
                vec!["-o", "snap.json"],
            ] {
                // Both argument orders, including flags before FILE for shorthand.
                for args in [
                    [prefix, &flags, &output].concat(),
                    [&flags, &output, prefix].concat(),
                ] {
                    if args.last() == Some(&"source.docx") && prefix.len() > 1 {
                        continue;
                    }
                    let result = parse(&args, &[]);
                    // Only --for-edit, which settles through its own
                    // --accept-change, refuses a view option (#1255).
                    if flags[0] != "--track-changes=all" || output[0] != "--for-edit" {
                        assert_eq!(result["exit_code"], 0, "{args:?}: {result}");
                        continue;
                    }
                    assert_eq!(result["exit_code"], 2, "{args:?}: {result}");
                    assert_eq!(result["stream"], "stderr", "{result}");
                    let text = result["text"].as_str().unwrap();
                    assert!(
                        text.contains("shapes the Markdown view"),
                        "{args:?}: {text}"
                    );
                    assert!(text.contains("has no view; drop"), "{args:?}: {text}");
                    assert_eq!(
                        text.lines().find(|line| line.starts_with("Usage:")),
                        Some("Usage: jubarte-python read [OPTIONS] <FILE>")
                    );
                }
            }
        }
    }
}

#[test]
fn edit_takes_skip_failed_and_add_does_not() {
    let result = parse(
        &[
            "edit",
            "a.docx",
            "--plan",
            "p.json",
            "--skip-failed",
            "-o",
            "b.docx",
        ],
        &[],
    );
    assert_eq!(result["exit_code"], 0, "{result}");
    assert_eq!(result["args"]["skip_failed"], true, "{result}");
    let result = parse(&["edit", "a.docx", "--plan", "p.json"], &[]);
    assert_eq!(result["args"]["skip_failed"], false, "{result}");
    // `add` writes one paragraph or comment: nothing to skip.
    let result = parse(
        &[
            "add",
            "a.docx",
            "-p",
            "p0",
            "--content",
            "x",
            "--skip-failed",
        ],
        &[],
    );
    assert_ne!(result["exit_code"], 0, "{result}");
}
