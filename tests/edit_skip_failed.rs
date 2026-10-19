// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Plan 3 (#1253): `--skip-failed` lands every operation that passes and
//! leaves the failed ones out, with a runnable plan of the failures beside
//! the plan; a clean dry run prints the command that applies it; every
//! write prints the output's SHA-256.

use jubarte::edit::{
    EditPlan, ExistingRevisions, PrepareOptions, SkippedOperation, failed_plan_json, prepare,
};
use jubarte::inspect::source_sha256;
use serde_json::{Value, json};
use std::path::Path;
use std::process::{Command, Output};

/// Four plain paragraphs, no tracked changes:
/// p0 "Payment is due within 30 days.", p1 "The fee is due; the fee is
/// payable.", p2 "One  space.", p3 "Notice is due within 30 days."
const SOURCE: &[u8] = include_bytes!("fixtures/edit-v2/example-source.docx");
const DATE: &str = "2026-10-19T10:00:00Z";

fn request(ops: &Value) -> Value {
    json!({"schema_version":1,"author":"Carol Diaz","date":DATE,"operations":ops})
}

fn plan(ops: &Value) -> EditPlan {
    EditPlan::from_json(&request(ops).to_string()).unwrap()
}

/// Five operations; the third is `bad`.
fn five(bad: &Value) -> Value {
    json!([
        {"kind":"replace","paragraph":"p0","find":"30 days","replacement":"45 days"},
        {"kind":"replace","paragraph":"p1","find":"fee is payable","replacement":"fee is owed"},
        bad,
        {"kind":"comment","paragraph":"p3","find":"Notice","text":"Check the notice period."},
        {"kind":"insert","paragraph":"p0","after":"Payment","text":" in full"},
    ])
}

/// An anchor p2 does not hold.
fn missing() -> Value {
    json!({"kind":"replace","paragraph":"p2","find":"two spaces","replacement":"one space"})
}

fn skipping() -> PrepareOptions {
    PrepareOptions {
        skip_failed: true,
        ..PrepareOptions::default()
    }
}

fn statuses(report: &jubarte::edit::EditReport) -> Vec<(String, String)> {
    report
        .operations
        .iter()
        .map(|o| (o.id.clone(), o.status.clone()))
        .collect()
}

fn texts(docx: &[u8]) -> Vec<String> {
    jubarte::inspect::paragraphs(docx)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

#[test]
fn skip_failed_writes_the_survivors_byte_for_byte() {
    let ops = five(&missing());
    let skipped = prepare(SOURCE, &plan(&ops), &skipping()).unwrap();
    let mut four = ops.as_array().unwrap().clone();
    four.remove(2);
    let survivors = prepare(
        SOURCE,
        &plan(&Value::Array(four)),
        &PrepareOptions::default(),
    )
    .unwrap();
    assert_eq!(skipped.result.redline, survivors.result.redline);
    assert_eq!(skipped.result.clean, survivors.result.clean);
    let report = &skipped.result.report;
    assert!(!report.ok, "a skipped operation is not ok");
    assert_eq!(report.skipped, 1);
    let want: Vec<(String, String)> = [
        ("op-1", "ok"),
        ("op-2", "ok"),
        ("op-3", "skipped"),
        ("op-4", "ok"),
        ("op-5", "ok"),
    ]
    .iter()
    .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
    .collect();
    assert_eq!(statuses(report), want);
    assert_eq!(
        report.operations[2].code.as_deref(),
        Some("ANCHOR_NOT_FOUND")
    );
    let [SkippedOperation { index, code, .. }] = skipped.skipped.as_slice() else {
        panic!("{:?}", skipped.skipped);
    };
    assert_eq!((*index, code.as_str()), (2, "ANCHOR_NOT_FOUND"));
}

#[test]
fn without_skip_failed_the_plan_stays_atomic() {
    let e = prepare(SOURCE, &plan(&five(&missing())), &PrepareOptions::default()).unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND", "{e}");
}

#[test]
fn a_group_falls_together() {
    let ops = json!([
        {"kind":"replace","paragraph":"p0","find":"30 days","replacement":"45 days","group":"cap"},
        {"kind":"replace","paragraph":"p1","find":"fee is payable","replacement":"fee is owed"},
        {"kind":"replace","paragraph":"p3","find":"absent","replacement":"x","group":"cap"},
    ]);
    let prepared = prepare(SOURCE, &plan(&ops), &skipping()).unwrap();
    let report = &prepared.result.report;
    let got: Vec<(&str, &str, Option<&str>)> = report
        .operations
        .iter()
        .map(|o| (o.id.as_str(), o.status.as_str(), o.code.as_deref()))
        .collect();
    assert_eq!(
        got,
        [
            ("op-1", "skipped", Some("GROUP_FAILED")),
            ("op-2", "ok", None),
            ("op-3", "skipped", Some("ANCHOR_NOT_FOUND")),
        ]
    );
    let message = report.operations[0].message.as_deref().unwrap_or_default();
    assert_eq!(message, r#"skipped with group "cap": op-3 failed"#);
    assert_eq!(report.skipped, 2);
    // Only op-2 landed.
    let clean = texts(&prepared.result.clean);
    assert_eq!(clean[0], "Payment is due within 30 days.");
    assert_eq!(clean[1], "The fee is due; the fee is owed.");
}

#[test]
fn a_group_without_skip_failed_changes_nothing() {
    let grouped = json!([
        {"kind":"replace","paragraph":"p0","find":"30 days","replacement":"45 days","group":"cap"},
        {"kind":"replace","paragraph":"p1","find":"fee is payable","replacement":"fee is owed","group":"cap"},
    ]);
    let plain = json!([
        {"kind":"replace","paragraph":"p0","find":"30 days","replacement":"45 days"},
        {"kind":"replace","paragraph":"p1","find":"fee is payable","replacement":"fee is owed"},
    ]);
    let a = prepare(SOURCE, &plan(&grouped), &PrepareOptions::default()).unwrap();
    let b = prepare(SOURCE, &plan(&plain), &PrepareOptions::default()).unwrap();
    assert_eq!(a.result.redline, b.result.redline);
    // The group survives a plan's round trip.
    let p = plan(&grouped);
    assert_eq!(EditPlan::from_json(&p.to_json()).unwrap(), p);
}

#[test]
fn resolve_revisions_failure_is_never_skipped() {
    let mut wire = request(&five(&missing()));
    wire["resolve_revisions"] = json!({"accept":{"ids":["r9"]}});
    let plan = EditPlan::from_json(&wire.to_string()).unwrap();
    let e = prepare(SOURCE, &plan, &skipping()).unwrap_err();
    assert_eq!(e.code, "UNKNOWN_CHANGE", "{e}");
}

#[test]
fn stale_source_is_never_skipped() {
    let mut wire = request(&five(&missing()));
    wire["source_sha256"] = json!("0".repeat(64));
    let plan = EditPlan::from_json(&wire.to_string()).unwrap();
    let e = prepare(SOURCE, &plan, &skipping()).unwrap_err();
    assert_eq!(e.code, "STALE_SOURCE", "{e}");
}

#[test]
fn all_failed_is_refused_under_skip_failed() {
    let ops = json!([
        {"kind":"replace","paragraph":"p0","find":"absent","replacement":"x"},
        {"kind":"replace","paragraph":"p1","find":"missing","replacement":"y"},
    ]);
    let e = prepare(SOURCE, &plan(&ops), &skipping()).unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND", "{e}");
    let failed: Vec<&str> = e.outcomes.iter().map(|o| o.status.as_str()).collect();
    assert_eq!(failed, ["failed", "failed"], "{e}");
}

#[test]
fn the_failed_plan_is_a_runnable_plan_of_the_failures() {
    let ops = five(&missing());
    let original = plan(&ops);
    let prepared = prepare(SOURCE, &original, &skipping()).unwrap();
    let output = source_sha256(&prepared.result.redline);
    let failed: Value = serde_json::from_str(&failed_plan_json(
        &original,
        &prepared.skipped,
        DATE,
        &output,
        ExistingRevisions::Keep,
    ))
    .unwrap();
    assert_eq!(failed["schema_version"], 1);
    assert_eq!(failed["author"], "Carol Diaz");
    assert_eq!(failed["date"], DATE);
    assert_eq!(failed["existing_revisions"], "keep");
    assert_eq!(failed["source_sha256"], output.as_str());
    assert!(failed.get("resolve_revisions").is_none(), "{failed}");
    let ops = failed["operations"].as_array().unwrap();
    assert_eq!(ops.len(), 1, "{failed}");
    assert_eq!(ops[0]["id"], "op-3");
    assert_eq!(ops[0]["find"], "two spaces");
    let why = ops[0]["why"].as_str().unwrap();
    assert!(why.starts_with("ANCHOR_NOT_FOUND: "), "{why}");
    // `why` is read and ignored: the plan parses and, fixed, applies on
    // the output.
    let mut fixed = failed.clone();
    fixed["operations"][0]["find"] = json!("One  space");
    fixed["operations"][0]["replacement"] = json!("One space");
    let fixed = EditPlan::from_json(&fixed.to_string()).unwrap();
    let applied = prepare(&prepared.result.redline, &fixed, &PrepareOptions::default()).unwrap();
    assert!(applied.result.report.ok);
}

#[test]
fn skipping_then_fixing_equals_fixing_first() {
    let good = [
        ("p0", "30 days", "45 days"),
        ("p1", "fee is payable", "fee is owed"),
        ("p2", "One  space", "One space"),
        ("p3", "Notice", "Note"),
    ];
    let op = |(paragraph, find, replacement): (&str, &str, &str), broken: bool| {
        let find = if broken {
            format!("{find}!")
        } else {
            find.to_string()
        };
        json!({"kind":"replace","paragraph":paragraph,"find":find,"replacement":replacement})
    };
    let fixed: Value = good.iter().map(|&g| op(g, false)).collect();
    let once = prepare(SOURCE, &plan(&fixed), &PrepareOptions::default()).unwrap();
    let want = texts(&once.result.clean);
    // A seeded walk over which operations break.
    let mut seed: u64 = 0x5eed_1253;
    for _ in 0..12 {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let broken: Vec<bool> = (0..4).map(|i| (seed >> (40 + i)) & 1 == 1).collect();
        if broken.iter().all(|&b| b) {
            continue;
        }
        let ops: Value = good.iter().zip(&broken).map(|(&g, &b)| op(g, b)).collect();
        let original = plan(&ops);
        let first = prepare(SOURCE, &original, &skipping()).unwrap();
        assert_eq!(
            first.skipped.len(),
            broken.iter().filter(|&&b| b).count(),
            "{broken:?}"
        );
        let output = first.result.redline.clone();
        let second = if first.skipped.is_empty() {
            output
        } else {
            let mut failed: Value = serde_json::from_str(&failed_plan_json(
                &original,
                &first.skipped,
                DATE,
                &source_sha256(&output),
                ExistingRevisions::Keep,
            ))
            .unwrap();
            for op in failed["operations"].as_array_mut().unwrap() {
                let find = op["find"]
                    .as_str()
                    .unwrap()
                    .trim_end_matches('!')
                    .to_string();
                op["find"] = json!(find);
            }
            let failed = EditPlan::from_json(&failed.to_string()).unwrap();
            prepare(&output, &failed, &PrepareOptions::default())
                .unwrap()
                .result
                .redline
        };
        let accepted = jubarte::changes::accept_changes(&second, &Default::default()).unwrap();
        assert_eq!(texts(&accepted), want, "{broken:?}");
    }
}

#[test]
fn discovery_offers_group_on_every_v1_operation() {
    for kind in ["replace", "strike", "settings"] {
        let answer = jubarte::capabilities::discovery("cli", Some(kind), None);
        let profile = answer["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["plan_version"] == 1)
            .unwrap()
            .clone();
        assert!(
            profile["fields"].get("group").is_some(),
            "{kind}: {profile}"
        );
    }
}

fn cli(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// `in.docx` and `plan.json` holding `ops` in a fresh directory.
fn workspace(ops: &Value) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), SOURCE).unwrap();
    std::fs::write(
        dir.path().join("plan.json"),
        serde_json::to_string_pretty(&request(ops)).unwrap(),
    )
    .unwrap();
    dir
}

#[test]
fn the_cli_skips_and_writes_the_failed_plan_beside_the_plan() {
    let dir = workspace(&five(&missing()));
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "--skip-failed",
            "-o",
            "out.docx",
        ],
    );
    let says = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{says}");
    assert!(
        says.contains("skipped 1 of 5 operations; the fixes are in plan.failed.json"),
        "{says}"
    );
    let written = std::fs::read(dir.path().join("out.docx")).unwrap();
    let failed: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("plan.failed.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(failed["source_sha256"], source_sha256(&written).as_str());
    assert_eq!(failed["operations"][0]["id"], "op-3");
    // Fixed, it applies on the output.
    let mut fixed = failed;
    fixed["operations"][0]["find"] = json!("One  space");
    std::fs::write(dir.path().join("fixed.json"), fixed.to_string()).unwrap();
    let again = cli(
        dir.path(),
        &[
            "edit",
            "out.docx",
            "--plan",
            "fixed.json",
            "-o",
            "out2.docx",
        ],
    );
    assert_eq!(again.status.code(), Some(0), "{}", stderr(&again));
}

#[test]
fn the_cli_flags_skip_into_the_files_plan_name() {
    let dir = workspace(&json!([]));
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "-p",
            "p2",
            "--anchor",
            "two spaces",
            "--content",
            "x",
            "-p",
            "p0",
            "--anchor",
            "30 days",
            "--content",
            "45 days",
            "--skip-failed",
            "-a",
            "Carol Diaz",
            "-d",
            DATE,
            "-o",
            "out.docx",
        ],
    );
    let says = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{says}");
    assert!(
        says.contains("skipped 1 of 2 operations; the fixes are in in_plan.failed.json"),
        "{says}"
    );
    let failed: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("in_plan.failed.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(failed["operations"][0]["find"], "two spaces");
}

#[test]
fn all_failed_exits_3_and_writes_nothing() {
    let dir = workspace(&json!([
        {"kind":"replace","paragraph":"p0","find":"absent","replacement":"x"},
    ]));
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "--skip-failed",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(out.status.code(), Some(3), "{}", stderr(&out));
    assert!(!dir.path().join("out.docx").exists());
    assert!(!dir.path().join("plan.failed.json").exists());
}

#[test]
fn dry_run_prints_the_next_command() {
    let good = json!([
        {"kind":"replace","paragraph":"p0","find":"30 days","replacement":"45 days"},
    ]);
    let dir = workspace(&good);
    let bin = env!("CARGO_BIN_EXE_jubarte");
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
            "--dry-run",
        ],
    );
    let says = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{says}");
    assert_eq!(
        says.lines().last(),
        Some(format!("next: {bin} edit in.docx --plan plan.json -o out.docx").as_str()),
        "{says}"
    );
    assert!(!dir.path().join("out.docx").exists());
    // A value with a space is quoted for the shell.
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "--dry-run",
            "-p",
            "p0",
            "--anchor",
            "30 days",
            "--content",
            "45 days",
            "-o",
            "out.docx",
        ],
    );
    assert!(
        stderr(&out).contains(&format!(
            "next: {bin} edit in.docx -p p0 --anchor '30 days' --content '45 days' -o out.docx"
        )),
        "{}",
        stderr(&out)
    );
    // An apostrophe closes the quote, escapes itself and reopens it.
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "-o",
            "O'Brien.docx",
            "--dry-run",
        ],
    );
    assert!(
        stderr(&out).contains(&format!(
            "next: {bin} edit in.docx --plan plan.json -o 'O'\\''Brien.docx'"
        )),
        "{}",
        stderr(&out)
    );
    // --quiet drops it.
    let out = cli(
        dir.path(),
        &[
            "-q",
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
            "--dry-run",
        ],
    );
    assert!(!stderr(&out).contains("next:"), "{}", stderr(&out));
    // A refused dry run has no next command.
    let dir = workspace(&five(&missing()));
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
            "--dry-run",
        ],
    );
    assert_eq!(out.status.code(), Some(3));
    assert!(!stderr(&out).contains("next:"), "{}", stderr(&out));
    // Nor does one that skipped an operation: it is not clean.
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
            "--dry-run",
            "--skip-failed",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(!stderr(&out).contains("next:"), "{}", stderr(&out));
}

#[test]
fn every_write_prints_the_sha256() {
    let good = json!([
        {"kind":"replace","paragraph":"p0","find":"30 days","replacement":"45 days"},
    ]);
    let dir = workspace(&good);
    let out = cli(
        dir.path(),
        &["edit", "in.docx", "--plan", "plan.json", "-o", "out.docx"],
    );
    let says = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{says}");
    let written = std::fs::read(dir.path().join("out.docx")).unwrap();
    let line = format!("sha256: {}", source_sha256(&written));
    assert!(says.lines().any(|l| l == line), "{says}");
    // The bundle's report names it in its summary.
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "--out-dir",
            "bundle",
        ],
    );
    let says = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{says}");
    let redline = std::fs::read(dir.path().join("bundle/redline.docx")).unwrap();
    let digest = source_sha256(&redline);
    assert!(
        says.lines().any(|l| l == format!("sha256: {digest}")),
        "{says}"
    );
    let report = std::fs::read_to_string(dir.path().join("bundle/report.jsonl")).unwrap();
    let summary: Value = report
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|v| v["ev"] == "summary")
        .unwrap();
    assert_eq!(summary["output_sha256"], digest.as_str(), "{summary}");
    // --quiet drops the line.
    let out = cli(
        dir.path(),
        &[
            "-q",
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "-o",
            "out3.docx",
        ],
    );
    assert!(!stderr(&out).contains("sha256:"), "{}", stderr(&out));
}

#[test]
fn the_manifest_says_edit_skips_failed_operations() {
    let manifest = jubarte::capabilities::capabilities("cli");
    assert!(manifest.operations.edit_skip_failed);
    let json: Value =
        serde_json::from_str(&jubarte::capabilities::capabilities_json("cli")).unwrap();
    assert_eq!(json["operations"]["edit_skip_failed"], true);
    let text = jubarte::capabilities::capabilities_text(&manifest);
    assert!(text.contains("\nedit_skip_failed: yes\n"), "{text}");
}
