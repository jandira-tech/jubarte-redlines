# Agent Edit Follow-ups Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Six additions to the agent view and the flag-driven `edit`/`add` of `docs/superpowers/plans/2026-10-09-agent-text-view.md` (Tasks 1-20, all merged before this plan starts), taken from how a collaborative document-editing API behaves: a failed anchor hands back the paragraph it searched; `--nth` picks a repeated anchor; the view carries the file's digest and `edit --digest` refuses a file that changed since it was read; stable paragraph ids (`w14:paraId`) as locations; comment anchors protected from silent detachment; a paragraph move.

**Architecture:** Everything builds on the first plan's pieces: `Transaction::find_range` and `EditOutcome` (`src/edit.rs`), `FlagOp`/`flag_operations` (`src/cli.rs`), `flag_operation`/`run_edit` (`src/bin/jubarte.rs`), the header facts (`src/markdown/from_docx/header.rs`) and id lines (`agent.rs`). No new modules except where said. The legacy conversion (`ids` off) stays byte-identical.

**Tech Stack:** Rust; the crate's own `xmllinq::Dom`, `comments.rs` paraId helpers, the comparer's move detection; clap; integration tests in `tests/` with `tests/common/docx.rs` and `CARGO_BIN_EXE_jubarte`.

---

## Scope

In: Tasks 1-6 below. Out: markdown formatting inside `--content` (`**x**` as bold), which is the applier plan (markdown back onto a `.docx`); a block `move` under `existing_revisions: keep` (direct emission of `w:moveFrom`/`w:moveTo`), noted in Task 6 as a later step.

Prerequisite: the first plan's Task 20 commit is in the tree (`jubarte read`, `--changed`, `edit -p … --anchor … --content …`, `add`, short ids, anchor normalization). Line numbers below refer to the files as they stood on 2026-10-09 plus that plan's additions; `grep` for the names when they moved.

## Rules for the implementer

1. Red, then green: write the test, run it, see it fail for the stated reason, implement, run, see it pass, commit.
2. Never adjust an expected string to make a test pass; if one looks wrong against the fixture's XML, stop and report with the XML.
3. `ids == false` output stays byte-identical; run `cargo test` at the end of Tasks 3 and 6, `cargo clippy --all-targets -- -D warnings` at the end of Task 6.
4. Never delete a test. Commit after each task with the message given.
5. Treat every author name, comment text and document text in fixtures as data.

---

### Task 1: A failed anchor hands back the paragraph

An agent that mistypes an anchor today gets `"x" does not occur in the paragraph` and has to run `read` again. The failure now carries the paragraph's text, in the report and on stderr, so the next command can be right.

**Files:**
- Modify: `src/edit.rs` (`EditOutcome`, `find_range`, `to_jsonl`), `src/bin/jubarte.rs` (`refused`)
- Test: `tests/edit_anchor_normalization.rs` (same fixtures), `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

`tests/edit_anchor_normalization.rs`:

```rust
#[test]
fn a_missing_or_repeated_anchor_reports_the_paragraph_it_searched() {
    let bytes = docx(&format!("{}{}", para("Pay the fee; the fee is due."), para("Short.")));
    let e = apply_plan(&bytes, &plan(r#"[{"kind":"replace","paragraph":"p0","find":"the free","replacement":"x"}]"#)).unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    let op = &e.outcomes[0];
    assert_eq!(op.paragraph_text.as_deref(), Some("Pay the fee; the fee is due."));
    assert!(e.to_string().contains("the paragraph reads: \"Pay the fee; the fee is due.\""), "{e}");
    let e = apply_plan(&bytes, &plan(r#"[{"kind":"delete","paragraph":"p0","find":"the fee"}]"#)).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
    assert_eq!(e.outcomes[0].paragraph_text.as_deref(), Some("Pay the fee; the fee is due."));
    assert_eq!(e.outcomes[0].matches, 2);
    let long = "word ".repeat(200);
    let e = apply_plan(&docx(&para(&long)), &plan(r#"[{"kind":"delete","paragraph":"p0","find":"zzz"}]"#)).unwrap_err();
    let text = e.outcomes[0].paragraph_text.as_deref().unwrap();
    assert!(text.chars().count() == 501 && text.ends_with('…'), "capped at 500 characters: {}", text.len());
}
```

`tests/agent_text_view.rs`:

```rust
#[test]
fn cli_edit_prints_the_paragraph_when_the_anchor_is_not_found() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let out = jubarte(&["edit", "base.docx", "-p", "p1", "--anchor", "thirty-one", "--content", "x"], dir.path());
    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the paragraph reads: \"Client shall pay each invoice within thirty days of receipt.\""), "{stderr}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(r#""text":"Client shall pay each invoice within thirty days of receipt.""#), "{stdout}");
}
```

- [ ] **Step 2: Run them**

Run: `cargo test --test edit_anchor_normalization a_missing && cargo test --test agent_text_view cli_edit_prints_the_paragraph`
Expected: FAIL (`paragraph_text` does not exist).

- [ ] **Step 3: Implement**

`src/edit.rs`:
- `EditOutcome` gains, after `anchor_read_as`: `#[serde(default, skip_serializing_if = "Option::is_none")] /// The paragraph's text when an anchor was not found or not unique (at most 500 characters, then `…`). pub paragraph_text: Option<String>,`. Every `EditOutcome` literal gains `paragraph_text: None` (the compiler lists them).
- `find_range`: in the two error arms (`[]` and `many`), before returning, set `outcome.paragraph_text = Some(capped(&projection.text))` where `capped` keeps the first 500 chars and appends `…` when it cut. Append `; the paragraph reads: {text:?}` to both messages (the capped text).
- `to_jsonl`: add `"text"` to the `op` event when `paragraph_text` is set.

`src/bin/jubarte.rs`, `refused()`: no change needed if it prints `e.to_string()`; confirm the stderr line carries the message.

- [ ] **Step 4: Run and commit**

Run: `cargo test --test edit_anchor_normalization && cargo test --test agent_text_view && cargo test --lib edit::`
Expected: PASS.

```bash
git add src/edit.rs src/bin/jubarte.rs tests/edit_anchor_normalization.rs tests/agent_text_view.rs
git commit -m "feat(edit): a failed anchor reports the paragraph it searched"
```

---

### Task 2: `--nth` for a repeated anchor

Plans have `occurrence`; the flag form had no way to say it. `--nth N` (1-based) belongs to the `-p` it follows and maps to `occurrence` on `replace`, `delete`, `insert`, `comment` and `format_run`.

**Files:**
- Modify: `src/cli.rs` (`Edit`, `Add`, `FlagOp`, `flag_operations`), `src/bin/jubarte.rs` (`flag_operation`, `inline_edit`)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn nth_picks_one_occurrence_of_a_repeated_anchor() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("fees.docx"), docx(&para("Pay the fee; the fee is due."))).unwrap();
    let out = jubarte(&["edit", "fees.docx", "-p", "p0", "--anchor", "the fee", "--content", "the charge", "--out-dir", "a"], dir.path());
    assert_eq!(out.status.code(), Some(3), "ambiguous without --nth");
    let out = ok(&["edit", "fees.docx", "-p", "p0", "--anchor", "the fee", "--nth", "2", "--content", "the charge", "--out-dir", "b", "--existing-revisions", "keep"], dir.path());
    assert!(out.contains("Pay the fee; {~~the fee~>the charge~~}"), "{out}");
    let out = ok(&["add", "fees.docx", "-p", "p0", "--anchor", "the fee", "--nth", "1", "--content", "Which fee?", "--out-dir", "c"], dir.path());
    assert!(out.contains("Pay {==the fee==}{>>#c0 @MU: Which fee?<<}; the fee is due."), "{out}");
    let bad = jubarte(&["edit", "fees.docx", "-p", "p0", "--nth", "2", "--content", "x"], dir.path());
    assert!(!bad.status.success(), "--nth needs --anchor");
}
```

- [ ] **Step 2: Run it**

Run: `cargo test --test agent_text_view nth_picks`
Expected: FAIL, `--nth` unknown.

- [ ] **Step 3: Implement**

`src/cli.rs`: both variants gain `/// Which occurrence of the anchor (1-based) when it repeats in WHERE. #[arg(long, value_name = "N", action = clap::ArgAction::Append)] nth: Vec<usize>`; `FlagOp` gains `pub nth: Option<usize>`; `flag_operations` reads `nth` like `anchor` (string values parsed with `get_many::<usize>`, indices from `indices_of("nth")`), refuses a second `--nth` on one `-p` and `--nth` without `--anchor` ("-p p0: --nth needs --anchor"). `src/bin/jubarte.rs`: `flag_operation` passes `op.nth` as `occurrence` to every kind that has the field (`inline_edit` takes it too).

- [ ] **Step 4: Run and commit**

Run: `cargo test --test agent_text_view && cargo test --test cli_parser`
Expected: PASS.

```bash
git add src/cli.rs src/bin/jubarte.rs tests/agent_text_view.rs
git commit -m "feat(cli): --nth selects a repeated anchor in edit and add"
```

---

### Task 3: `edit --digest` refuses a changed file

The header's `sha256:` line (cli-std Task 6, after `source:`) is the
SHA-256 of the bytes the view read: the file, or the compare shorthand's
in-memory redline; `edit`'s printed view digests the file it wrote. What is
left here: `edit` and `add` take `--digest HEX` (8 or more hex characters,
a prefix of that line) and refuse with `STALE_SOURCE` (exit 3) when the
file's SHA-256 does not start with it.

**Files:**
- Modify: `src/cli.rs` (`EditOptions.digest`), `src/bin/jubarte.rs` (`run_edit` callers)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Failing test** `digest_in_the_header_guards_an_edit`: read
  `base.docx`, take the first 16 hex of its `sha256:` line; `edit … --digest
  <16 hex>` and `--digest <8 hex, upper-cased>` succeed; after rewriting the
  file, the same edit exits 3 with `STALE_SOURCE` and the digest on stderr,
  and writes nothing; `--digest xyz` is refused; an edit's printed view
  carries the redline's own `sha256:`.
- [ ] **Step 2: Implement**
  - `src/cli.rs`, `EditOptions`: `/// Refuse unless FILE's SHA-256 starts with this (the view's sha256 line). #[arg(long, value_name = "HEX", value_parser = parse_digest)] pub digest: Option<String>`; `parse_digest`: 8 or more ASCII hex characters, lower-cased, else "a digest is 8 or more hex characters".
  - `src/bin/jubarte.rs`, the shared closure of the `Edit`/`Add` arms, right after `read_document` and before any directory is created: compare `jubarte::inspect::source_sha256(&source)` with the prefix; refuse with `EXIT_PLAN_REFUSED` and `STALE_SOURCE: {file} changed since it was read (sha256 {first 16}…, expected {prefix}…)`.
- [ ] **Step 3: Whole suite, commit** `feat(edit): --digest refuses a file changed since its view`.

---

### Task 4: Stable paragraph ids (`w14:paraId`) as locations

`pN` shifts after `add`. Word stamps every paragraph with `w14:paraId` (8 hex digits, unique in the package); jubarte already allocates them for comment paragraphs (`src/comments.rs:879`, `allocate_hex8` at 1040, `package_para_ids` at 1056). This task reads them, prints them on request, accepts them as locations, and stamps them on the paragraphs `edit`/`add` create, so an agent can chain edits across inserts without re-reading.

**Files:**
- Modify: `src/inspect.rs` (`Paragraph.pid`), `src/markdown/from_docx/agent.rs` (`LineFacts.pid`, `id_line`), `src/markdown/mod.rs` (`MarkdownOptions.pids`), `src/cli.rs` (`ReadArgs.pids`), `src/edit.rs` (`long_id`, `EditOutcome.pid`, paragraph insertion), `src/comments.rs` (make the two helpers `pub(crate)`), `src/bin/jubarte.rs`
- Test: `tests/agent_text_view.rs`, `tests/edit_short_ids.rs`

- [ ] **Step 1: Write the failing tests**

First read the fixture's ids: `unzip -p tests/fixtures/agent-view/received.docx word/document.xml | grep -o 'w14:paraId="[0-9A-F]*"' | head -6`. The spec's JSON view lists p0 `0DCBD3F9`, p1 `5F8FB75F`, p5 `7C10AC12`; the test below uses them, confirm them against that output before running (rule 2).

```rust
#[test]
fn pids_print_on_request_and_locate_paragraphs_across_inserts() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    let plain = ok(&["read", "received.docx", "-p", "p0,p5", "--no-page-markers"], dir.path());
    assert!(plain.contains("<!-- p0 center -->\n") && !plain.contains("pid "), "{plain}");
    let with = ok(&["read", "received.docx", "-p", "p0,p5", "--pids", "--no-page-markers"], dir.path());
    assert!(with.contains("<!-- p0 center, pid 0DCBD3F9 -->\n") && with.contains("<!-- p5 pid 7C10AC12 -->\n"), "{with}");

    let out = ok(&["add", "received.docx", "-p", "0DCBD3F9", "--content", "Recitals", "--existing-revisions", "keep", "--out-dir", "a"], dir.path());
    captures(&out, r"<!-- p1[^>]*pid [0-9A-F]{8}[^>]*-->\n\{\+\+Recitals\+\+\}");
    let report = std::fs::read_to_string(dir.path().join("a/report.jsonl")).unwrap();
    assert!(report.contains(r#""at":"body:p:0","pid":"0DCBD3F9""#), "{report}");
    // The same pid still names the title after the insert shifted every pN.
    let out = ok(&["edit", "a/redline.docx", "-p", "7C10AC12", "--anchor", "days of receipt", "--content", "days of the invoice", "--out-dir", "b"], dir.path());
    assert!(out.contains("<!-- p6 ") && out.contains("{~~receipt~>the invoice~~}"), "{out}");
    let bad = jubarte(&["edit", "received.docx", "-p", "DEADBEEF", "--delete"], dir.path());
    assert_eq!(bad.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&bad.stderr).contains("no paragraph has pid DEADBEEF"));
}
```

`tests/edit_short_ids.rs`:

```rust
#[test]
fn a_new_paragraph_gets_a_fresh_pid_unique_in_the_package() {
    let out = apply_plan(&source(), &plan(r#"[{"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"New"}]}]"#)).unwrap();
    let xml = common::docx::part_string(&out.clean, "word/document.xml").unwrap();
    let ids: Vec<&str> = xml.match_indices("w14:paraId=\"").map(|(at, _)| &xml[at + 12..at + 20]).collect();
    assert_eq!(ids.len(), 8, "every paragraph carries one: {xml}");
    let unique: std::collections::HashSet<&&str> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len());
    assert!(ids.iter().all(|id| u32::from_str_radix(id, 16).is_ok_and(|n| n < 0x8000_0000)), "{ids:?}");
}
```

(`source()` there has seven paragraphs without pids; the test expects the insert to stamp the new one and the others, so that `-p <pid>` works on documents Word never touched. If stamping every paragraph is judged too invasive for the clean copy, change the expectation to 1 and say so in the commit; do not leave the test failing.)

- [ ] **Step 2: Run them**

Run: `cargo test --test agent_text_view pids_print && cargo test --test edit_short_ids a_new_paragraph`
Expected: FAIL.

- [ ] **Step 3: Implement**

- `src/inspect.rs`: `Paragraph` gains `pub pid: Option<String>` (the `w14:paraId` attribute, upper-cased; `W14::name("paraId")` from `src/namespaces.rs`), filled where `index`, `id` and `text` are.
- `src/markdown/mod.rs`: `MarkdownOptions.pids: bool` (default false). `agent.rs`: `LineFacts.pid: Option<String>`; `id_line` appends the clause `pid {HEX}` last when `pids` is on and the paragraph has one. `ReadArgs`: `/// Print each paragraph's w14:paraId on its id line (a location that survives inserts). #[arg(long, help_heading = "Read options")] pub pids: bool`.
- `src/edit.rs`: `long_id` recognises eight hex digits (case-insensitive) and returns the long id of the paragraph with that `w14:paraId` in any story (build a map once per transaction, lazily), or `Err("no paragraph has pid DEADBEEF")`. `EditOutcome.pid: Option<String>` (serialized as `"pid"` in the `op` event after `"at"`), filled from the resolved paragraph. Paragraph insertion (`insert_paragraph`, `insert_table` cells, `insert_footnote` body paragraphs, split paragraphs if any op splits) stamps a fresh id with `comments::allocate_hex8(&mut used, seed)` where `used` comes from `comments::package_para_ids` plus the body's own; both helpers become `pub(crate)`. Declare `w14` on the document root when missing (`declare_ignorable`, also in `comments.rs`).
- `src/bin/jubarte.rs`: when any `-p` of the invocation was a pid, the printed view sets `pids: true`.

- [ ] **Step 4: Run and commit**

Run: `cargo test --test agent_text_view && cargo test --test edit_short_ids && cargo test --lib edit:: && cargo test --test docx_markdown_round_trip`
Expected: PASS.

```bash
git add src/inspect.rs src/markdown src/cli.rs src/edit.rs src/comments.rs src/bin/jubarte.rs tests/agent_text_view.rs tests/edit_short_ids.rs
git commit -m "feat: w14:paraId as a stable location (read --pids, -p HEX); new paragraphs are stamped"
```

---

### Task 5: Comment anchors are not detached silently

An operation that removes every character a comment is anchored on leaves the comment hanging. Word keeps such a comment on an empty range at the deletion. This task first records what jubarte does today, then refuses the operation with `COMMENT_ANCHOR` unless the plan says `allow_detach: true` (flag form: `--allow-detach`), in which case the comment stays as a point comment at the deletion.

**Files:**
- Modify: `src/edit.rs` (`Operation`/`EditPlan` field, resolution check), `src/cli.rs` (`EditOptions.allow_detach`), `src/bin/jubarte.rs`
- Test: `tests/edit_comment_anchors.rs` (new)

- [ ] **Step 1: Observe, then write the failing tests**

Build a document with a comment on `one percent` (Task 7 of the first plan has `commented_docx`/`comment`/`reference` helpers in `tests/agent_text_view.rs`; copy the three into the new file). Run `apply_plan` with `{"kind":"delete","paragraph":"p0","find":"one percent"}` and `{"kind":"replace","paragraph":"p0","find":"at one percent per","replacement":"monthly"}` and inspect `out.clean` and `out.redline` XML for `commentRangeStart`/`commentRangeEnd`/`commentReference` with the comment's id: write what you find in the commit message (kept on an empty range, dropped, or left around deleted text). Then the tests:

```rust
#[test]
fn removing_a_whole_comment_anchor_is_refused_unless_allowed() {
    let bytes = commented_docx(/* one comment c5 on "one percent" */);
    for operations in [
        r#"[{"kind":"delete","paragraph":"p0","find":"one percent"}]"#,
        r#"[{"kind":"replace","paragraph":"p0","find":"at one percent per","replacement":"monthly"}]"#,
        r#"[{"kind":"rewrite","paragraph":"p0","text":"Late amounts accrue no interest."}]"#,
        r#"[{"kind":"delete_paragraph","paragraph":"p0"}]"#,
    ] {
        let e = apply_plan(&bytes, &plan(operations)).unwrap_err();
        assert_eq!(e.code, "COMMENT_ANCHOR", "{operations}: {e}");
        assert!(e.message.contains("comment 5"), "{e}");
    }
    // Part of the anchor survives: allowed, the comment shrinks.
    let out = apply_plan(&bytes, &plan(r#"[{"kind":"replace","paragraph":"p0","find":"one percent","replacement":"two percent"}]"#)).unwrap();
    assert!(out.report.ok);
    // Allowed explicitly: the comment stays as a point at the deletion.
    let allowed = EditPlan { allow_detach: true, ..plan(r#"[{"kind":"delete","paragraph":"p0","find":"one percent"}]"#) };
    let out = apply_plan(&bytes, &allowed).unwrap();
    let view = docx_to_markdown(&out.redline, &MarkdownOptions { ids: true, comments: true, ..Default::default() }).unwrap().markdown;
    assert!(view.contains("{--one percent--}{>>#"), "{view}");
    assert!(view.contains("{>>#c5 @"), "the comment survives: {view}");
    let clean = docx_to_markdown(&out.clean, &MarkdownOptions { ids: true, comments: true, ..Default::default() }).unwrap().markdown;
    assert!(clean.contains("{>>#c5 @") && !clean.contains("{=="), "a point comment in the clean copy: {clean}");
}
```

(`delete_paragraph` with its own `comment` field is a different thing: that comment is new. Keep the test on removal of an existing anchor.) Add a CLI case to `tests/agent_text_view.rs`: `edit … -p p2 --anchor "one percent" --delete` on a document commented there exits 3 with `COMMENT_ANCHOR`, and succeeds with `--allow-detach`.

- [ ] **Step 2: Run them**

Run: `cargo test --test edit_comment_anchors`
Expected: FAIL (`allow_detach` does not exist; no refusal today, or a different code).

- [ ] **Step 3: Implement**

- `EditPlan.allow_detach: bool` (`#[serde(default, skip_serializing_if = "std::ops::Not::not")]`); `capabilities --json` lists it.
- In `Transaction::resolve`, after every operation's range is known and before `check_conflicts`: for each comment whose range lies in an edited paragraph, compute the characters of its range that survive the resolved edits (deletions, replacements, rewrites, deleted paragraphs); if none survive and `!plan.allow_detach`, fail that operation with `("COMMENT_ANCHOR", "removes every character of the anchor of comment 5 (\"one percent\"); set allow_detach to keep it as a point comment")`. The comment ranges per paragraph are already gathered at `src/edit.rs:3869` (`comment_ranges`) for another purpose; reuse that collection.
- When allowed, make sure both outputs keep the markers adjacent (a point comment) rather than dropping them; if the observation in Step 1 showed the markers are dropped, fix that in the same place.
- `EditOptions.allow_detach: bool` → `flag_plan` sets it.

- [ ] **Step 4: Run and commit**

Run: `cargo test --test edit_comment_anchors && cargo test --test agent_text_view && cargo test --lib edit::`
Expected: PASS.

```bash
git add src/edit.rs src/cli.rs src/bin/jubarte.rs src/capabilities.rs tests/edit_comment_anchors.rs tests/agent_text_view.rs
git commit -m "feat(edit): refuse to remove a comment's whole anchor unless allow_detach"
```

---

### Task 6: `move_paragraph`

A paragraph moved before or after another one. Through the comparer (the default for a clean source), the clean copy simply has the paragraph in its new place and the comparer's move detection marks `w:moveFrom`/`w:moveTo`, which `changes` already lists as `kind: move` and the view shows as deletion and insertion with the same text. Under `existing_revisions: keep` the operation is refused in this version (direct emission of move ranges is a later step).

**Files:**
- Modify: `src/edit.rs` (`OperationKind::MoveParagraph`, resolution, apply), `src/cli.rs` (`Edit`: `--move-after`, `--move-before`), `src/bin/jubarte.rs` (`flag_operation`), `src/capabilities.rs`
- Test: `tests/edit_move_paragraph.rs` (new), `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

```rust
//! move_paragraph: a paragraph before or after another, as a tracked move.
mod common;
use common::docx::{docx, para};
use jubarte::changes::{list_changes, ChangeKind};
use jubarte::edit::{apply_plan, EditPlan};
use jubarte::inspect::paragraphs;

fn plan(operations: &str) -> EditPlan { /* as in edit_short_ids.rs */ }

#[test]
fn a_moved_paragraph_lands_where_asked_and_the_redline_shows_a_move() {
    let bytes = docx(&format!("{}{}{}{}", para("A"), para("B"), para("C"), para("D")));
    let out = apply_plan(&bytes, &plan(r#"[{"kind":"move_paragraph","paragraph":"p3","after":"p0"}]"#)).unwrap();
    let texts: Vec<String> = paragraphs(&out.clean).unwrap().into_iter().map(|p| p.text).collect();
    assert_eq!(texts, ["A", "D", "B", "C"]);
    let changes = list_changes(&out.redline).unwrap();
    assert!(changes.iter().any(|c| c.kind == ChangeKind::Move && c.text == "D"), "{changes:?}");
    let out = apply_plan(&bytes, &plan(r#"[{"kind":"move_paragraph","paragraph":"p0","before":"p3"}]"#)).unwrap();
    let texts: Vec<String> = paragraphs(&out.clean).unwrap().into_iter().map(|p| p.text).collect();
    assert_eq!(texts, ["B", "C", "A", "D"]);
    for bad in [
        r#"[{"kind":"move_paragraph","paragraph":"p0","after":"p0"}]"#,
        r#"[{"kind":"move_paragraph","paragraph":"p0"}]"#,
        r#"[{"kind":"move_paragraph","paragraph":"p0","after":"p1","before":"p2"}]"#,
    ] {
        assert!(apply_plan(&bytes, &plan(bad)).is_err(), "{bad}");
    }
    let keep = EditPlan { existing_revisions: jubarte::edit::ExistingRevisions::Keep, ..plan(r#"[{"kind":"move_paragraph","paragraph":"p3","after":"p0"}]"#) };
    let e = apply_plan(&bytes, &keep).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert!(e.message.contains("keep"), "{e}");
}
```

CLI, in `tests/agent_text_view.rs`: `edit base.docx -p p2 --move-after p0 --out-dir m` prints a view whose `range:` names `p1` and `p3` (the move's two ends; the exact ids depend on where the comparer places the marks, so assert that `{--Late amounts` and `{++Late amounts` both appear) and the report has `"op":"move_paragraph"`; `-p p2 --move-after p0 --content x` is refused.

- [ ] **Step 2: Run them**

Run: `cargo test --test edit_move_paragraph`
Expected: FAIL (unknown kind).

- [ ] **Step 3: Implement**

- `OperationKind::MoveParagraph { paragraph: Selector, #[serde(default)] after: Option<Selector>, #[serde(default)] before: Option<Selector> }` (exactly one of `after`/`before`; the target must not be the paragraph itself, and both must be body paragraphs outside tables for this version). Resolution: two paragraph lookups; apply: detach the `w:p` node and re-insert it next to the target (`dom` child operations, as `insert_paragraph` does); the comparer then produces the move. Under `keep`: `UNSUPPORTED_STRUCTURE`, "move_paragraph is not supported with existing_revisions \"keep\" yet".
- `check_conflicts` must treat the moved paragraph as edited (no other operation on it in the same plan).
- `src/cli.rs`, `Edit`: `/// Move the paragraph to just after WHERE. #[arg(long, value_name = "WHERE", action = clap::ArgAction::Append)] move_after: Vec<String>` and `move_before` likewise; `FlagOp.move_after/move_before: Option<String>`; `flag_operations` groups them like `anchor`; `flag_operation`: `-p pN --move-after X` → `MoveParagraph`, refused with `--anchor`, `--content`, `--delete`, `--style`.
- `capabilities --json` lists the kind; the CHANGELOG line.

- [ ] **Step 4: Whole suite, clippy, commit**

Run: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: clean.

```bash
git add src/edit.rs src/cli.rs src/bin/jubarte.rs src/capabilities.rs tests/edit_move_paragraph.rs tests/agent_text_view.rs CHANGELOG.md
git commit -m "feat(edit): move_paragraph (compare path), -p WHERE --move-after/--move-before"
```

---

## Self-review

Order: Tasks 1-3 are independent of each other and small; 4 changes the id grammar (`pid` clause) and the insertion code, so it comes after the goldens of Task 3 are settled; 5 depends on nothing but is the one task that starts with an observation, because the current behaviour is unknown; 6 is last because it is the only new operation kind.

Risks: `w14:paraId` values above `0x7FFFFFFF` are invalid per Word, hence the mask in `allocate_hex8`'s test; a document whose paragraphs carry no pids gets them stamped on the first edit, which changes `clean.docx` beyond the edit itself (Task 4 says so and leaves the expectation adjustable in one place); the comparer's move detection may render a short moved paragraph as deletion plus insertion rather than a move (if `list_changes` shows no `Move` for `D`, use a longer paragraph in the test before concluding the feature failed); the comment-anchor check needs the resolved ranges of every operation, which `resolve` has, but `rewrite` resolves to a word-level diff later in `apply`, so for `rewrite` the check is "the new text no longer contains the anchor text", an approximation worth a comment in the code.
