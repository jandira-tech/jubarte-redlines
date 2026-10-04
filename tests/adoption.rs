// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Every row of `docs/adoption/anthropic-docx-skill.md` and
//! `docs/adoption/openai-doc-skill.md`, run through the `jubarte` command
//! line as those pages write it. One test per row: when a page's claim stops
//! being true, the test that names it fails. The side-by-side outputs with
//! the replaced tools are in `examples/adoption/`; `.github/workflows/adoption.yml`
//! runs this file and those folders.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::validity::assert_word_valid_package;

const CONTRACT: &str = "\
# Services Agreement

This Agreement is made between **Acme Corp** and *Beta LLC*.

## 1. Fees

The fee is due monthly. The fee is fixed. The fee includes support.

| Item | Price |
|---|---|
| Setup | 500 |
| Support | 100 |

## 2. Term

The term is twelve months.

Either party may terminate on thirty days notice.
";

fn jubarte(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run jubarte")
}

#[track_caller]
fn ok(args: &[&str], dir: &Path) -> String {
    let out = jubarte(args, dir);
    assert!(
        out.status.success(),
        "jubarte {args:?} exited {:?}\nstdout: {}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8 stdout")
}

/// A scratch folder holding `input.docx`, written from [`CONTRACT`].
fn workspace() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().to_path_buf();
    std::fs::write(path.join("input.md"), CONTRACT).unwrap();
    ok(&["convert", "input.md", "-o", "input.docx"], &path);
    (dir, path)
}

/// `docx` with one XML part rewritten by `edit`.
fn rewrite_part(docx: &[u8], name: &str, edit: impl Fn(&str) -> String) -> Vec<u8> {
    let xml = part(docx, name);
    let edited = edit(&xml);
    assert_ne!(edited, xml, "{name} unchanged");
    common::docx::replace_entry(docx, name, edited.as_bytes())
}

fn part(docx: &[u8], name: &str) -> String {
    common::docx::part_string(docx, name).unwrap_or_else(|| panic!("no {name}"))
}

fn write_plan(dir: &Path, name: &str, body: &str) {
    std::fs::write(dir.join(name), body).unwrap();
}

/// `count` paragraphs of filler: enough for several Letter pages.
fn long_markdown(count: usize) -> String {
    (1..=count)
        .map(|n| {
            format!(
                "## Section {n}\n\nParagraph {n} says what the parties agreed about item {n}, at enough length to wrap over several lines of a Letter page and fill it.\n\n"
            )
        })
        .collect()
}

/// Pages in a PDF, counted from its page objects (the trick the page
/// markers rely on: the PDF says how many pages there are).
fn pdf_pages(pdf: &[u8]) -> usize {
    let text = String::from_utf8_lossy(pdf);
    text.matches("/Type /Page").count() - text.matches("/Type /Pages").count()
}

/// Read row: `pandoc -t markdown` -> `jubarte text` with paragraph ids and
/// stories; `jubarte inspect --json` carries `source_sha256`.
#[test]
fn read_text_has_paragraph_ids_and_inspect_has_the_source_hash() {
    let (_dir, dir) = workspace();
    let text = ok(&["text", "input.docx"], &dir);
    assert!(text.contains("[body:p:0"), "{text}");
    assert!(text.contains("Services Agreement"), "{text}");
    let inspect = ok(&["inspect", "input.docx", "--json"], &dir);
    assert!(inspect.contains("source_sha256"), "{inspect}");
}

/// Render row: `soffice` + `pdftoppm` -> `jubarte convert --png --dpi 100
/// --report pages.json`, with `page_count`, page text and fonts.
#[test]
fn render_png_writes_pages_and_a_report() {
    let (_dir, dir) = workspace();
    ok(
        &[
            "convert",
            "input.docx",
            "--png",
            "--dpi",
            "100",
            "--report",
            "pages.json",
        ],
        &dir,
    );
    assert!(dir.join("input-page-01.png").is_file());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("pages.json")).unwrap()).unwrap();
    assert_eq!(report["page_count"], 1);
    assert!(
        report["pages"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Services Agreement")
    );
    assert!(report["fonts"].is_array());
}

/// Page ranges (#38313): `--pages 2-3` rasterizes only those pages.
#[test]
fn page_range_rasterizes_only_the_pages_asked_for() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("long.md"), long_markdown(40)).unwrap();
    ok(&["convert", "long.md", "-o", "long.docx"], dir.path());
    ok(
        &[
            "convert",
            "long.docx",
            "--png",
            "--pages",
            "2-3",
            "--dpi",
            "36",
        ],
        dir.path(),
    );
    let mut pngs: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|name| name.ends_with(".png"))
        .collect();
    pngs.sort();
    assert_eq!(pngs, ["long-page-02.png", "long-page-03.png"]);
}

/// "Did my edit change the layout?": `diff-render` exits 5 when a page
/// differs and 0 when none does.
#[test]
fn diff_render_exits_five_on_a_changed_page_and_zero_otherwise() {
    let (_dir, dir) = workspace();
    let same = jubarte(
        &[
            "diff-render",
            "input.docx",
            "input.docx",
            "--out-dir",
            "same",
        ],
        &dir,
    );
    assert_eq!(same.status.code(), Some(0), "{same:?}");
    std::fs::write(
        dir.join("after.md"),
        CONTRACT.replace("twelve", "twenty-four"),
    )
    .unwrap();
    ok(&["convert", "after.md", "-o", "after.docx"], &dir);
    let changed = jubarte(
        &["diff-render", "input.docx", "after.docx", "--out-dir", "d"],
        &dir,
    );
    assert_eq!(changed.status.code(), Some(5), "{changed:?}");
    assert!(dir.join("d/diff.json").is_file());
}

/// "Did a font fall back?": `--fail-on-substitution` exits 4 and still
/// writes the PDF.
#[test]
fn a_substituted_font_exits_four_with_the_pdf_written() {
    let (_dir, dir) = workspace();
    let docx = std::fs::read(dir.join("input.docx")).unwrap();
    let edited = rewrite_part(&docx, "word/styles.xml", |xml| {
        xml.replace(
            "w:ascii=\"Calibri\" w:hAnsi=\"Calibri\"",
            "w:ascii=\"Jubarte Absent Serif\" w:hAnsi=\"Jubarte Absent Serif\"",
        )
    });
    std::fs::write(dir.join("absent.docx"), edited).unwrap();
    let out = jubarte(
        &[
            "convert",
            "absent.docx",
            "--fail-on-substitution",
            "--font-report",
            "fonts.json",
        ],
        &dir,
    );
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(dir.join("absent.pdf").is_file());
    let fonts = std::fs::read_to_string(dir.join("fonts.json")).unwrap();
    assert!(fonts.contains("Jubarte Absent Serif"), "{fonts}");
}

/// Edit row: one plan writes `clean.docx`, `redline.docx`, `report.jsonl`
/// and `patch.diff`; the tracked-check row: accepting the redline gives the
/// clean copy's text.
#[test]
fn an_edit_plan_writes_four_outputs_and_every_edit_is_tracked() {
    let (_dir, dir) = workspace();
    write_plan(
        &dir,
        "plan.json",
        r#"{"schema_version":1,"author":"Reviewer","operations":[
            {"kind":"replace","paragraph":{"starts_with":"The term"},"find":"twelve","replacement":"twenty-four"},
            {"kind":"delete_paragraph","paragraph":{"starts_with":"Either party"}}]}"#,
    );
    ok(
        &[
            "edit",
            "input.docx",
            "--plan",
            "plan.json",
            "--out-dir",
            "review",
        ],
        &dir,
    );
    for name in ["clean.docx", "redline.docx", "report.jsonl", "patch.diff"] {
        assert!(dir.join("review").join(name).is_file(), "{name}");
    }
    assert_word_valid_package(&std::fs::read(dir.join("review/redline.docx")).unwrap());
    ok(&["accept", "review/redline.docx", "-o", "check.docx"], &dir);
    assert_eq!(
        ok(&["text", "check.docx"], &dir),
        ok(&["text", "review/clean.docx"], &dir)
    );
}

/// A stale or ambiguous plan is refused with exit 3 and nothing written.
#[test]
fn an_ambiguous_anchor_is_refused_with_exit_three() {
    let (_dir, dir) = workspace();
    write_plan(
        &dir,
        "plan.json",
        r#"{"schema_version":1,"author":"R","operations":[
            {"kind":"replace","paragraph":{"starts_with":"The fee"},"find":"fee","replacement":"cost"}]}"#,
    );
    let out = jubarte(
        &[
            "edit",
            "input.docx",
            "--plan",
            "plan.json",
            "--out-dir",
            "review",
        ],
        &dir,
    );
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("AMBIGUOUS_ANCHOR")
            || String::from_utf8_lossy(&out.stdout).contains("AMBIGUOUS_ANCHOR")
    );
    assert!(!dir.join("review/clean.docx").exists());
}

/// Repeated anchors: `occurrence` (1-based) picks the Nth match.
#[test]
fn occurrence_edits_only_the_nth_match() {
    let (_dir, dir) = workspace();
    write_plan(
        &dir,
        "plan.json",
        r#"{"schema_version":1,"author":"R","operations":[
            {"kind":"replace","paragraph":{"starts_with":"The fee"},"find":"fee","occurrence":2,"replacement":"price"}]}"#,
    );
    ok(
        &[
            "edit",
            "input.docx",
            "--plan",
            "plan.json",
            "--out-dir",
            "review",
        ],
        &dir,
    );
    let text = ok(&["text", "review/clean.docx"], &dir);
    assert!(
        text.contains("The fee is due monthly. The price is fixed. The fee includes support."),
        "{text}"
    );
}

/// Their tracked changes stay theirs: `"existing_revisions": "keep"`, and
/// without it `EXISTING_REVISIONS`.
#[test]
fn existing_revisions_keep_leaves_both_authors_tracked() {
    let (_dir, dir) = workspace();
    std::fs::write(
        dir.join("theirs.md"),
        CONTRACT.replace("thirty days", "sixty days"),
    )
    .unwrap();
    ok(&["convert", "theirs.md", "-o", "theirs.docx"], &dir);
    ok(
        &[
            "input.docx",
            "theirs.docx",
            "-o",
            "received.docx",
            "--author",
            "Counterparty",
        ],
        &dir,
    );
    let body = |keep: &str| {
        format!(
            r#"{{"schema_version":1,"author":"Us",{keep}"operations":[
            {{"kind":"replace","paragraph":{{"starts_with":"The term"}},"find":"twelve","replacement":"eighteen"}}]}}"#
        )
    };
    write_plan(&dir, "refused.json", &body(""));
    let refused = jubarte(
        &[
            "edit",
            "received.docx",
            "--plan",
            "refused.json",
            "--out-dir",
            "r1",
        ],
        &dir,
    );
    assert_eq!(refused.status.code(), Some(3), "{refused:?}");
    write_plan(&dir, "keep.json", &body(r#""existing_revisions":"keep","#));
    ok(
        &[
            "edit",
            "received.docx",
            "--plan",
            "keep.json",
            "--out-dir",
            "r2",
        ],
        &dir,
    );
    let changes = ok(&["changes", "r2/redline.docx", "--json"], &dir);
    assert!(
        changes.contains("Counterparty") && changes.contains("\"Us\""),
        "{changes}"
    );
}

/// Clean copy row: `accept` and `reject`, all at once or per change.
#[test]
fn accept_and_reject_resolve_every_change() {
    let (_dir, dir) = workspace();
    std::fs::write(dir.join("v2.md"), CONTRACT.replace("twelve", "six")).unwrap();
    ok(&["convert", "v2.md", "-o", "v2.docx"], &dir);
    ok(
        &[
            "input.docx",
            "v2.docx",
            "-o",
            "redline.docx",
            "--author",
            "Ann",
        ],
        &dir,
    );
    ok(&["accept", "redline.docx", "-o", "accepted.docx"], &dir);
    ok(&["reject", "redline.docx", "-o", "rejected.docx"], &dir);
    assert!(ok(&["text", "accepted.docx"], &dir).contains("The term is six months."));
    assert!(ok(&["text", "rejected.docx"], &dir).contains("The term is twelve months."));
    assert!(
        ok(&["changes", "accepted.docx", "--json"], &dir)
            .trim()
            .is_empty()
    );
}

/// Comments row: the engine places the anchors; threads reply and resolve;
/// `comments --json` reads them back.
#[test]
fn a_comment_thread_is_added_replied_to_and_resolved() {
    let (_dir, dir) = workspace();
    write_plan(
        &dir,
        "one.json",
        r#"{"schema_version":1,"author":"Ann","operations":[
            {"kind":"comment","paragraph":{"starts_with":"The term"},"find":"twelve months","text":"Why not six?"}]}"#,
    );
    ok(
        &["edit", "input.docx", "--plan", "one.json", "--out-dir", "a"],
        &dir,
    );
    write_plan(
        &dir,
        "two.json",
        r#"{"schema_version":1,"author":"Bob","operations":[
            {"kind":"reply_comment","comment_id":0,"text":"Twelve matches the budget."},
            {"kind":"resolve_comment","comment_id":0}]}"#,
    );
    ok(
        &[
            "edit",
            "a/clean.docx",
            "--plan",
            "two.json",
            "--out-dir",
            "b",
        ],
        &dir,
    );
    let threads = ok(&["comments", "b/clean.docx", "--json"], &dir);
    assert!(
        threads.contains("Why not six?") && threads.contains("Twelve matches the budget."),
        "{threads}"
    );
    assert!(threads.contains("\"done\":true"), "{threads}");
    assert_word_valid_package(&std::fs::read(dir.join("b/clean.docx")).unwrap());
}

/// Create row: Markdown to `.docx` on US Letter (`--page letter`), A4 on
/// request, CriticMarkup as tracked changes.
#[test]
fn markdown_creates_a_letter_or_a4_document() {
    let (_dir, dir) = workspace();
    let letter = part(
        &std::fs::read(dir.join("input.docx")).unwrap(),
        "word/document.xml",
    );
    assert!(
        letter.contains("w:w=\"12240\"") && letter.contains("w:h=\"15840\""),
        "{letter}"
    );
    ok(
        &["convert", "input.md", "-o", "a4.docx", "--page", "a4"],
        &dir,
    );
    let a4 = part(
        &std::fs::read(dir.join("a4.docx")).unwrap(),
        "word/document.xml",
    );
    assert!(
        a4.contains("w:w=\"11906\"") && a4.contains("w:h=\"16838\""),
        "{a4}"
    );
    std::fs::write(dir.join("critic.md"), "Due in {~~30~>45~~} days.").unwrap();
    ok(&["convert", "critic.md", "-o", "critic.docx"], &dir);
    assert!(
        !ok(&["changes", "critic.docx", "--json"], &dir)
            .trim()
            .is_empty()
    );
}

/// Tables and lists: `insert_table` and `list` plan operations, tracked.
#[test]
fn tables_and_lists_are_tracked_plan_operations() {
    let (_dir, dir) = workspace();
    write_plan(
        &dir,
        "plan.json",
        r#"{"schema_version":1,"author":"R","operations":[
            {"kind":"insert_table","paragraph":{"starts_with":"The term"},"position":"after","header_row":true,
             "rows":[["Party","Role"],["Acme","Client"],["Beta","Vendor"]]},
            {"kind":"list","paragraphs":[{"starts_with":"The term"},{"starts_with":"Either party"}],"kind_of_list":"decimal"}]}"#,
    );
    ok(
        &[
            "edit",
            "input.docx",
            "--plan",
            "plan.json",
            "--out-dir",
            "review",
        ],
        &dir,
    );
    let redline = std::fs::read(dir.join("review/redline.docx")).unwrap();
    assert_word_valid_package(&redline);
    let xml = part(&redline, "word/document.xml");
    assert!(
        xml.contains("<w:tbl>") && xml.contains("Vendor"),
        "table missing"
    );
    assert!(xml.contains("<w:numPr>"), "list missing");
    assert!(
        !ok(&["changes", "review/redline.docx", "--json"], &dir)
            .trim()
            .is_empty()
    );
}

/// Compare row: `jubarte a.docx b.docx -o redline.docx --author Name`.
#[test]
fn compare_writes_a_word_redline_by_the_author_named() {
    let (_dir, dir) = workspace();
    std::fs::write(dir.join("v2.md"), CONTRACT.replace("Beta LLC", "Gamma Inc")).unwrap();
    ok(&["convert", "v2.md", "-o", "v2.docx"], &dir);
    ok(
        &[
            "input.docx",
            "v2.docx",
            "-o",
            "redline.docx",
            "--author",
            "Reviewer",
        ],
        &dir,
    );
    let redline = std::fs::read(dir.join("redline.docx")).unwrap();
    assert_word_valid_package(&redline);
    let changes = ok(&["changes", "redline.docx", "--json"], &dir);
    assert!(
        changes.contains("Reviewer") && changes.contains("Gamma"),
        "{changes}"
    );
}

/// Check row: `jubarte validate` reports a Word-fatal file and passes ours.
#[test]
fn validate_passes_our_output_and_reports_a_broken_package() {
    let (_dir, dir) = workspace();
    ok(&["validate", "input.docx"], &dir);
    let docx = std::fs::read(dir.join("input.docx")).unwrap();
    let broken = rewrite_part(&docx, "word/document.xml", |xml| {
        xml.replacen(
            "<w:p>",
            "<w:p><w:del w:id=\"901\" w:author=\"A\" w:date=\"2026-01-01T00:00:00Z\"><w:r><w:t>x</w:t></w:r></w:del>",
            1,
        )
    });
    std::fs::write(dir.join("broken.docx"), broken).unwrap();
    let out = jubarte(&["validate", "broken.docx", "--json"], &dir);
    assert!(!out.status.success(), "{out:?}");
}

/// Legacy `.doc` row: `jubarte convert old.doc` writes `old.docx` (text,
/// headings, tables), where the page used to say "keep soffice".
#[test]
fn a_legacy_doc_converts_to_docx_markdown_and_pdf() {
    let dir = tempfile::tempdir().unwrap();
    let doc = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/legacy/services.doc");
    std::fs::copy(&doc, dir.path().join("old.doc")).unwrap();
    ok(&["convert", "old.doc"], dir.path());
    let docx = std::fs::read(dir.path().join("old.docx")).unwrap();
    assert_word_valid_package(&docx);
    let text = ok(&["text", "old.docx"], dir.path());
    assert!(text.contains("Heading1] Services Agreement"), "{text}");
    let markdown = ok(&["convert", "old.doc", "-t", "md"], dir.path());
    assert!(markdown.contains("|Setup|500|one-off|"), "{markdown}");
    ok(&["convert", "old.doc", "-o", "old.pdf"], dir.path());
    assert_eq!(
        pdf_pages(&std::fs::read(dir.path().join("old.pdf")).unwrap()),
        1
    );
    // The other commands still refuse it, and say how to convert it.
    let refused = jubarte(&["text", "old.doc"], dir.path());
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("jubarte convert old.doc -o old.docx"),
        "{stderr}"
    );
}

/// Markdown read from Word names its pages: as many as the PDF has.
#[test]
fn markdown_page_markers_count_the_pdf_pages() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("long.md"), long_markdown(40)).unwrap();
    ok(&["convert", "long.md", "-o", "long.docx"], dir.path());
    ok(&["convert", "long.docx", "-o", "long.pdf"], dir.path());
    let pages = pdf_pages(&std::fs::read(dir.path().join("long.pdf")).unwrap());
    assert!(pages >= 3, "{pages}");
    let markdown = ok(&["convert", "long.docx", "-t", "md"], dir.path());
    assert!(
        markdown.starts_with(&format!("<!-- page 1 of {pages} -->\n\n")),
        "{markdown}"
    );
    let last = format!("<!-- page {pages} of {pages} -->");
    assert!(markdown.contains(&last), "{markdown}");
    let numbers: Vec<usize> = markdown
        .lines()
        .filter_map(|l| {
            l.strip_prefix("<!-- page ")?
                .split(' ')
                .next()?
                .parse()
                .ok()
        })
        .collect();
    assert!(numbers.windows(2).all(|w| w[0] < w[1]), "{numbers:?}");
    // The markers are comments: the Markdown converts back to the same text.
    std::fs::write(dir.path().join("back.md"), &markdown).unwrap();
    ok(&["convert", "back.md", "-o", "back.docx"], dir.path());
    assert_eq!(
        ok(&["text", "back.docx"], dir.path()),
        ok(&["text", "long.docx"], dir.path())
    );
    let plain = ok(
        &["convert", "long.docx", "-t", "md", "--no-page-markers"],
        dir.path(),
    );
    assert!(!plain.contains("<!-- page"), "{plain}");
}

/// Codex #38313's other half: `--timeout` exits 124 once the deadline
/// passes, and does not get in the way of a conversion that finishes.
#[test]
fn convert_timeout_exits_124_past_the_deadline() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("long.md"), long_markdown(400)).unwrap();
    ok(&["convert", "long.md", "-o", "long.docx"], dir.path());
    let late = jubarte(
        &["convert", "long.docx", "--png", "--timeout", "0.001"],
        dir.path(),
    );
    assert_eq!(late.status.code(), Some(124), "{late:?}");
    assert!(String::from_utf8_lossy(&late.stderr).contains("timed out"));
    ok(&["convert", "long.docx", "--timeout", "600"], dir.path());
    let zero = jubarte(&["convert", "long.docx", "--timeout", "0"], dir.path());
    assert_eq!(zero.status.code(), Some(2), "{zero:?}");
}
