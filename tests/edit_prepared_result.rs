// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `PreparedEdit` (V2-08): one preparation path shared by dry run and
//! apply, the primary bytes chosen before final checks, and checks that run
//! on the serialized candidate.

mod common;

use common::docx::{docx, para};
use jubarte::edit::{
    EditPlan, PrepareOptions, PrimaryBytes, introduced_findings, prepare, preview_plan_with,
};
use jubarte::inspect::{paragraphs, source_sha256};
use jubarte::result::Effect;
use jubarte::validate::Finding;

const DEFAULT: PrepareOptions = PrepareOptions {
    require_change: false,
    exact_text: false,
};
const REQUIRE_CHANGE: PrepareOptions = PrepareOptions {
    require_change: true,
    exact_text: false,
};
const EXACT: PrepareOptions = PrepareOptions {
    require_change: false,
    exact_text: true,
};

fn plan(json: &str) -> EditPlan {
    EditPlan::from_json(json).unwrap()
}

fn empty_plan() -> EditPlan {
    plan(r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[]}"#)
}

fn rewrite(new_text: &str) -> EditPlan {
    plan(&format!(
        r#"{{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[{{"kind":"rewrite","paragraph":"p0","text":{}}}]}}"#,
        serde_json::to_string(new_text).unwrap()
    ))
}

fn finding(code: &str, part: &str, location: &str, severity: &str) -> Finding {
    let (word_fatal, repairable) = (severity == "error", false);
    Finding {
        code: code.into(),
        set: "word".into(),
        severity: severity.into(),
        part: part.into(),
        location: location.into(),
        r#ref: location.into(),
        message: format!("{code} at {location}"),
        word_fatal,
        repairable,
    }
}

/// Dry run and apply prepare — and check — the same serialized candidate.
/// A plan that produces no change (operations: [] — a request that edits nothing) is a
/// valid request; v2's `require_change` is the opt-in for needing a nonempty effect.
#[test]
fn dry_run_and_apply_check_the_same_serialized_candidate() {
    let source = docx(&para("Same words."));
    // Both paths refuse the same candidate defect identically.
    let bad = rewrite("Same  words.");
    let applied = prepare(&source, &bad, &EXACT).unwrap_err();
    let previewed = preview_plan_with(&source, &bad, &EXACT).unwrap_err();
    assert_eq!(applied.code, "POSTCONDITION_FAILED");
    assert_eq!(previewed.code, applied.code);
    // And accept the same good candidate with the same counts.
    let good = rewrite("Other words.");
    let prepared = prepare(&source, &good, &DEFAULT).unwrap();
    let report = preview_plan_with(&source, &good, &DEFAULT).unwrap();
    assert_eq!(report.revisions, prepared.result.report.revisions);
    assert_eq!(report.paragraphs, prepared.result.report.paragraphs);
    assert_eq!(report.comments_added, prepared.result.report.comments_added);
    // The serialized candidate the checks describe is the produced bytes.
    assert_eq!(prepared.primary, prepared.result.clean);
    assert_eq!(prepared.primary_bytes, PrimaryBytes::Candidate);
}

/// A dry run writes nothing at all: no document, plan, report, render,
/// temporary file or cache in the working directory.
#[test]
fn dry_run_leaves_no_files() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("a.docx");
    std::fs::write(&src, docx(&para("Same words."))).unwrap();
    let plan_path = dir.path().join("plan.json");
    std::fs::write(&plan_path, empty_plan().to_json()).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args(["edit", "a.docx", "--plan", "plan.json", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let mut left: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(left, ["a.docx", "plan.json"]);
}

/// An unchanged transaction selects the original bytes before checks and
/// digests — never a validated reserialization swapped in at commit.
#[test]
fn unchanged_primary_is_chosen_before_checks_and_ids() {
    let source = docx(&para("Same words."));
    let prepared = prepare(&source, &empty_plan(), &DEFAULT).unwrap();
    assert_eq!(prepared.primary_effect, Effect::Unchanged);
    assert_eq!(prepared.primary_bytes, PrimaryBytes::Source);
    // The checked and digested bytes are S itself.
    assert_eq!(prepared.primary, source);
    assert_eq!(prepared.primary_sha256, source_sha256(&source));
    assert_eq!(prepared.source_sha256, source_sha256(&source));
}

/// Findings are compared by identity (set/code/part/entity), not counts:
/// a baseline warning healed beside a new warning of the same code still
/// reports the introduced one.
#[test]
fn baseline_finding_does_not_hide_new_same_count_finding() {
    let old = finding("X_CODE", "word/document.xml", "w:body[0]/w:p[0]", "warning");
    let healed = old.clone();
    let new_elsewhere = finding("X_CODE", "word/document.xml", "w:body[0]/w:p[2]", "warning");
    // Same aggregate count (1 → 1): a count comparison misses the defect.
    assert_eq!(
        introduced_findings(&[old], std::slice::from_ref(&new_elsewhere)),
        vec![new_elsewhere]
    );
    // An unrelated baseline warning is disclosed without arbitrary blockage.
    assert!(introduced_findings(&[healed], &[]).is_empty());
    // An introduced Word-fatal finding blocks even at matching counts.
    let fatal = finding(
        "MC_UNBOUND_PREFIX",
        "word/document.xml",
        "w:body[0]/w:p[1]",
        "error",
    );
    assert_eq!(
        introduced_findings(
            &[finding(
                "MC_UNBOUND_PREFIX",
                "word/document.xml",
                "w:body[0]/w:p[0]",
                "error"
            )],
            std::slice::from_ref(&fatal)
        ),
        vec![fatal]
    );
}

/// Whitespace and soft breaks are exact in v2: a request the word-level
/// rewrite normalizes away is `POSTCONDITION_FAILED` (the legacy flag's
/// documented normalization keeps its v1 meaning), and a requested soft
/// break is realized exactly — a newline, not a space.
#[test]
fn rewrite_whitespace_and_soft_break_are_exact() {
    let source = docx(&para("a b"));
    // Requested but normalized away: exact mode refuses, default keeps v1.
    let squashed = rewrite("a  b");
    let e = prepare(&source, &squashed, &EXACT).unwrap_err();
    assert_eq!(e.code, "POSTCONDITION_FAILED");
    assert_eq!(e.operation.as_deref(), Some("op-1"));
    assert!(
        prepare(&source, &squashed, &DEFAULT).is_ok(),
        "v1's documented whitespace normalization is unchanged"
    );
    // A tab the source has, requested as a space, stays: the request is
    // normalized away and exact mode refuses it.
    let tabbed = docx("<w:p><w:r><w:t>a</w:t><w:tab/><w:t>b</w:t></w:r></w:p>");
    let e = prepare(&tabbed, &rewrite("a b"), &EXACT).unwrap_err();
    assert_eq!(e.code, "POSTCONDITION_FAILED");
    // A requested tab or soft break is realized exactly in the serialized
    // primary — not restated as a space.
    let prepared = prepare(&source, &rewrite("a\tb"), &EXACT).unwrap();
    assert_eq!(paragraphs(&prepared.primary).unwrap()[0].text, "a\tb");
    let broken = rewrite("a\nb");
    let prepared = prepare(&source, &broken, &EXACT).unwrap();
    assert_eq!(prepared.primary_effect, Effect::Changed);
    assert_eq!(paragraphs(&prepared.primary).unwrap()[0].text, "a\nb");
}

/// An unchanged apply is a truthful success whose primary is an exact copy
/// of the source bytes, `--require-change` is a separate opt-in refusal,
/// and a structural effect with no new text is still `changed`.
#[test]
fn unchanged_copy_and_require_change_are_distinct() {
    let source = docx(&para("Same words."));
    let prepared = prepare(&source, &empty_plan(), &DEFAULT).unwrap();
    assert_eq!(prepared.primary_effect, Effect::Unchanged);
    assert_eq!(prepared.primary, source);
    // --require-change judges the primary only: the same plan refuses.
    let e = prepare(&source, &empty_plan(), &REQUIRE_CHANGE).unwrap_err();
    assert_eq!(e.code, "NO_CHANGES_REQUESTED");
    // A postcondition failure outranks the require-change condition.
    let e = prepare(
        &source,
        &rewrite("Same  words."),
        &PrepareOptions {
            require_change: true,
            exact_text: true,
        },
    )
    .unwrap_err();
    assert_eq!(e.code, "POSTCONDITION_FAILED");
    // An empty-paragraph deletion is a structural effect: `changed`, not
    // `unchanged`, even though no visible characters were added.
    let empty = docx(&format!("<w:p/>{}", para("Kept.")));
    let delete = plan(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[{"kind":"delete_paragraph","paragraph":"p0"}]}"#,
    );
    let prepared = prepare(&empty, &delete, &REQUIRE_CHANGE).unwrap();
    assert_eq!(prepared.primary_effect, Effect::Changed);
    assert_eq!(prepared.primary_bytes, PrimaryBytes::Candidate);
}
