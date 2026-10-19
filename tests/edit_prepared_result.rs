// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `PreparedEdit` (V2-08): one preparation path shared by dry run and
//! apply, the primary bytes chosen before final checks, and checks that run
//! on the serialized candidate.

mod common;

use common::docx::{Part, docx, docx_with_sect, para};
use common::validity::assert_word_valid_package;
use jubarte::edit::{
    EditPlan, PlanSyntax, PrepareOptions, PrimaryBytes, PrimaryRole, apply_plan_json,
    apply_plan_json_with, introduced_findings, prepare, preview_plan_with,
};
use jubarte::inspect::{controls, paragraphs, source_sha256, stories};
use jubarte::result::Effect;
use jubarte::validate::Finding;

const DEFAULT: PrepareOptions = PrepareOptions {
    require_change: false,
    exact_text: false,
    role: PrimaryRole::Clean,
    syntax: PlanSyntax::V1,
    skip_failed: false,
};
const REQUIRE_CHANGE: PrepareOptions = PrepareOptions {
    require_change: true,
    exact_text: false,
    role: PrimaryRole::Clean,
    syntax: PlanSyntax::V1,
    skip_failed: false,
};
const EXACT: PrepareOptions = PrepareOptions {
    require_change: false,
    exact_text: true,
    role: PrimaryRole::Clean,
    syntax: PlanSyntax::V1,
    skip_failed: false,
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
    // The operations match too, comment ids included: the dry run reports
    // the apply's outcomes, never a resolve-only sketch of them.
    let noted = plan(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[{"kind":"replace","paragraph":"p0","find":"words","replacement":"lines","comment":"Wording."}]}"#,
    );
    let prepared = prepare(&source, &noted, &DEFAULT).unwrap();
    let report = preview_plan_with(&source, &noted, &DEFAULT).unwrap();
    assert_eq!(report.operations, prepared.result.report.operations);
    assert_eq!(
        report.operations[0].comment_id,
        prepared.result.report.operations[0].comment_id
    );
    assert!(report.operations[0].comment_id.is_some());
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
            role: PrimaryRole::Clean,
            syntax: PlanSyntax::V1,
            skip_failed: false,
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

/// Under `exact_text` a rewrite and a disjoint co-resident edit compose:
/// the postcondition is the composed text — the rewrite's request with
/// the co-resident edit applied — never the bare request discarding the
/// neighbour.
#[test]
fn rewrite_composes_with_coresident_edits_under_exact_text() {
    let source = docx(&para("alpha beta"));
    let plan = plan(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[
            {"kind":"rewrite","paragraph":"p0","text":"alpha BETA"},
            {"kind":"replace","paragraph":"p0","find":"alpha","replacement":"ALPHA"}]}"#,
    );
    let prepared = prepare(&source, &plan, &EXACT).unwrap();
    assert_eq!(paragraphs(&prepared.primary).unwrap()[0].text, "ALPHA BETA");
}

/// A co-resident edit cannot smuggle a normalized-away rewrite through
/// exact mode: the rewrite's own resolved edits must compose to exactly
/// the requested text, with or without neighbours on the paragraph.
#[test]
fn coresident_edit_does_not_smuggle_a_normalized_rewrite_through() {
    let source = docx(&para("a b c"));
    let plan = plan(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[
            {"kind":"rewrite","paragraph":"p0","text":"a  b c"},
            {"kind":"replace","paragraph":"p0","find":"c","replacement":"C"}]}"#,
    );
    let e = prepare(&source, &plan, &EXACT).unwrap_err();
    assert_eq!(e.code, "POSTCONDITION_FAILED");
    assert_eq!(e.operation.as_deref(), Some("op-1"));
}

/// Two disjoint replacements in one paragraph compose through the shared
/// preparation, and an allowed comment anchor beside them stays correct
/// (plan §3 P2, gate iii).
#[test]
fn disjoint_replaces_compose_through_prepare() {
    let source = docx(&para("alpha beta gamma"));
    let plan = plan(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[
            {"kind":"replace","paragraph":"p0","find":"alpha","replacement":"ALPHA"},
            {"kind":"replace","paragraph":"p0","find":"beta","replacement":"BETA"},
            {"kind":"comment","paragraph":"p0","find":"gamma","text":"Keep."}]}"#,
    );
    let prepared = prepare(&source, &plan, &DEFAULT).unwrap();
    assert_eq!(
        paragraphs(&prepared.primary).unwrap()[0].text,
        "ALPHA BETA gamma"
    );
    assert_eq!(prepared.result.report.comments_added, 1);
    assert_eq!(prepared.primary_effect, Effect::Changed);
    assert_eq!(prepared.primary_bytes, PrimaryBytes::Candidate);
}

/// An edit whose redline introduces a bookmark crossing a revision is
/// refused before publication (plan §3 P2, gate iv): the finding comes
/// from the real `validate` pass on the serialized candidate bytes, and
/// the dry run refuses the same bytes the apply refuses.
#[test]
fn introduced_bookmark_crossing_fails_the_candidate() {
    let source = docx(
        r#"<w:p><w:bookmarkStart w:id="7" w:name="Clause"/><w:r><w:t>alpha beta</w:t></w:r><w:bookmarkEnd w:id="7"/></w:p>"#,
    );
    assert!(
        jubarte::validate::validate(&source)
            .unwrap()
            .iter()
            .all(|f| !f.code.starts_with("BOOKMARK_")),
        "the source carries no bookmark finding to begin with"
    );
    // The page setup is no change the emitter writes, so the plan's redline
    // comes from the comparer, which rewrites "beta" whole and takes the
    // bookmark's end into the insertion (#1293). Alone, the replacement is
    // emitted and keeps the end outside it (#1156).
    let plan = plan(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[{"kind":"replace","paragraph":"p0","find":"beta","replacement":"BETA"},{"kind":"page_setup","section":"all","orientation":"landscape"}]}"#,
    );
    let e = prepare(&source, &plan, &DEFAULT).unwrap_err();
    assert_eq!(e.code, "CANDIDATE_INVALID");
    assert!(
        e.message.contains("BOOKMARK_CROSSES_REVISION"),
        "{}",
        e.message
    );
    assert_eq!(
        preview_plan_with(&source, &plan, &DEFAULT)
            .unwrap_err()
            .code,
        "CANDIDATE_INVALID"
    );
}

/// Finding identity is (set, code, part, location) — message prose and
/// the `repairable` remedy hint are not identity: redline candidates
/// renumber entities (bookmark ids among them) inside otherwise
/// unchanged baseline messages, and keying on the prose would call a
/// persisting baseline finding introduced.
#[test]
fn finding_identity_ignores_message_prose() {
    let baseline = finding("X_CODE", "word/document.xml", "w:body[0]/w:p[0]", "warning");
    let mut reworded = baseline.clone();
    reworded.message = "X_CODE because bookmark 9 moved".into();
    assert!(introduced_findings(std::slice::from_ref(&baseline), &[reworded]).is_empty());
    let mut repairable = baseline;
    repairable.repairable = true;
    assert!(
        introduced_findings(
            &[finding(
                "X_CODE",
                "word/document.xml",
                "w:body[0]/w:p[0]",
                "warning"
            )],
            &[repairable]
        )
        .is_empty()
    );
}

/// The JSON entry point takes the preparation's options — `require_change`
/// and `exact_text` reach the Python and WASM callers — while
/// `apply_plan_json` keeps its v1 default behaviour.
#[test]
fn apply_plan_json_with_threads_prepare_options() {
    let source = docx(&para("Same words."));
    let empty = empty_plan().to_json();
    assert!(apply_plan_json(&source, &empty).is_ok());
    let e = apply_plan_json_with(&source, &empty, &REQUIRE_CHANGE).unwrap_err();
    assert_eq!(e.code, "NO_CHANGES_REQUESTED");
    let squashed = rewrite("Same  words.").to_json();
    assert!(
        apply_plan_json(&source, &squashed).is_ok(),
        "v1's documented whitespace normalization is unchanged"
    );
    let e = apply_plan_json_with(&source, &squashed, &EXACT).unwrap_err();
    assert_eq!(e.code, "POSTCONDITION_FAILED");
}

/// Watermark-, settings- and footnote-only plans resolve no text
/// operation, yet each is a changed primary — never the source verbatim
/// under `--require-change`.
#[test]
fn watermark_settings_and_footnote_only_plans_change_the_primary() {
    let source = docx(&para("Body text."));
    for (name, ops) in [
        ("watermark", r#"[{"kind":"watermark","text":"DRAFT"}]"#),
        (
            "settings",
            r#"[{"kind":"settings","track_revisions":true}]"#,
        ),
        (
            "footnote",
            r#"[{"kind":"insert_footnote","paragraph":"p0","after":"text","text":"A note."}]"#,
        ),
    ] {
        let plan = plan(&format!(
            r#"{{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":{ops}}}"#
        ));
        let prepared = match prepare(&source, &plan, &REQUIRE_CHANGE) {
            Ok(p) => p,
            Err(e) => panic!("{name}: {} ({})", e.code, e.message),
        };
        assert_eq!(prepared.primary_effect, Effect::Changed, "{name}");
        assert_eq!(prepared.primary_bytes, PrimaryBytes::Candidate, "{name}");
        assert_ne!(prepared.primary, source, "{name}");
    }
}

/// A filled control's realized text equals the requested value: the fill
/// carries its own exact postcondition on the control's content, instead
/// of the paragraph standing exempt from every check.
#[test]
fn a_filled_control_realizes_the_requested_value() {
    let name = r#"<w:p><w:r><w:t xml:space="preserve">Name: </w:t></w:r><w:sdt><w:sdtPr><w:tag w:val="Name"/><w:id w:val="101"/><w:showingPlcHdr/><w:text/></w:sdtPr><w:sdtContent><w:r><w:rPr><w:rStyle w:val="PlaceholderText"/></w:rPr><w:t>Click here</w:t></w:r></w:sdtContent></w:sdt></w:p>"#;
    let source = docx(name);
    let plan = plan(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[{"kind":"fill_control","control":{"tag":"Name"},"text":"Ada Lovelace"}]}"#,
    );
    let prepared = prepare(&source, &plan, &REQUIRE_CHANGE).unwrap();
    assert_eq!(prepared.primary_effect, Effect::Changed);
    let filled = controls(&prepared.primary).unwrap();
    assert_eq!(filled[0].text, "Ada Lovelace");
    assert_word_valid_package(&prepared.primary);
}

const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
const HEADER_XML: &str = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>Header words.</w:t></w:r></w:p></w:hdr>"#;

fn header_source() -> Vec<u8> {
    docx_with_sect(
        &para("Body."),
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: HEADER_XML,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

/// The DOM-vs-serialized check covers every story a plan touched: a
/// header edit's paragraphs are compared against the bytes the writer
/// produced, not only the body's.
#[test]
fn header_edits_are_checked_dom_against_serialized() {
    let source = header_source();
    let plan = plan(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[{"kind":"replace","paragraph":"header1","find":"words","replacement":"lines"}]}"#,
    );
    let prepared = prepare(&source, &plan, &REQUIRE_CHANGE).unwrap();
    assert_eq!(prepared.primary_effect, Effect::Changed);
    let reopened = stories(&prepared.primary).unwrap();
    let header = reopened
        .iter()
        .find(|s| s.part == "word/header1.xml")
        .expect("the header story survived");
    assert_eq!(header.paragraphs[0].text, "Header lines.");
    assert_word_valid_package(&prepared.primary);
}

const REDLINE_REQUIRE_CHANGE: PrepareOptions = PrepareOptions {
    require_change: true,
    exact_text: false,
    role: PrimaryRole::Redline,
    syntax: PlanSyntax::V1,
    skip_failed: false,
};

/// For a changed v1 typed apply, the receipt digest names the bytes that
/// are actually published: the chosen primary, byte for byte. The
/// `PreparedEdit` the library hands the CLI carries the same bytes the
/// CLI then writes — no out-of-band swap after `prepare` returns.
#[test]
fn review_typed_v1_primary_digest_describes_the_redline_bytes() {
    let source = docx(&para("alpha beta"));
    let plan = plan(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[{"kind":"replace","paragraph":"p0","find":"alpha","replacement":"ALPHA"}]}"#,
    );
    let prepared = prepare(&source, &plan, &REDLINE_REQUIRE_CHANGE).unwrap();
    assert_eq!(prepared.primary_effect, Effect::Changed);
    assert_eq!(prepared.primary_bytes, PrimaryBytes::Candidate);
    // The chosen primary is the redline, not a sanitizing wrapper or a
    // reserialized clean copy: the library owns the choice, and the
    // digest the receipt reports is the sha256 of those exact bytes.
    assert_eq!(prepared.primary, prepared.result.redline);
    assert_eq!(
        prepared.primary_sha256,
        source_sha256(&prepared.result.redline)
    );
    assert_eq!(
        prepared.primary_sha256,
        source_sha256(&prepared.primary),
        "the digest describes the primary bytes the CLI publishes"
    );
}
