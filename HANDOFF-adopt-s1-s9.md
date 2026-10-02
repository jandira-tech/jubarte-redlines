# Handoff: adopt/s1-s9-occurrence-fonts (Plan 1, Tasks 4 and 5)

Written 2026-10-02 at the end of a session that stopped after the first
red test run. No subagents were spawned; every finding below is from inline
reads of this checkout. Nothing is committed and nothing is pushed.

## 1. State of the checkout

- Repo: /home/user/jubarte-redlines, branch `adopt/s1-s9-occurrence-fonts`,
  at main's `b420d64` (no commits on the branch yet).
- Toolchain present: cargo 1.97.0 at /root/.cargo/bin, uv 0.8.17 at
  /root/.local/bin. No install needed.
- Fetched: `origin/ccr-a92b0695-1adjlu` (the plan branch) and `origin/main`.
- Plan saved at
  /tmp/claude-0/-home-user-jubarte-redlines/a5266b9b-b838-5b66-a7ba-63b19ec1b85f/scratchpad/plan.md
  (scratchpad is session-specific; re-fetch with
  `git show origin/ccr-a92b0695-1adjlu:docs/superpowers/plans/2026-10-02-provider-adoption-1-s01-s10.md`).
  Task 4 is plan lines 969-1103; Task 5 is lines 1105-1151.
- One untracked file exists: `tests/edit_occurrence.rs` (full content in
  section 4). It was run once: 10 tests, 10 failed, as expected (red).
  Nothing else in the tree is modified.
- `target/` has a compiled test build of the crate from that run.

## 2. Scope (from the session brief)

Task 4 (S1): `occurrence: Option<usize>` on `Replace`, `Insert`, `Delete`,
`Comment`; `find_range` honours it; `check_operation_keys` allows it; Python
builders take `occurrence=`; skill section 2 gotcha; CHANGELOG. One commit:
`feat(edit): occurrence selects a repeated anchor; the refusal names the range`.

Task 5 (S9): `FontReportEntry::substituted()` and a `substituted` JSON field;
`jubarte convert --fail-on-substitution` exits 4; Python
`FontResolution.substituted` and `RenderReport.substitutions`; skill section
3; CHANGELOG. Second, separate commit:
`feat(convert): substituted flag in the font report; --fail-on-substitution`.

Conflict discipline from the brief: `occurrence` goes LAST in each of the
four variants; new allowed keys go at the END of each row in
`check_operation_keys`; new CLI args go at the END of the `Convert` variant;
CHANGELOG bullets only under `## [Unreleased]`; do not reformat unrelated
code. Other sessions are adding other OperationKind variants and CLI flags
on sibling branches.

Finish: `git fetch origin main && git rebase origin/main`, rerun gates,
push `git push -u origin adopt/s1-s9-occurrence-fonts`, open a DRAFT PR
against main with `mcp__github__create_pull_request` (body: what the plan
asked, what was done, exact gate output with coverage numbers, every
deviation and why, anything undone). Commit trailer lines required by the
session reminder:

```
Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01WzJbzDqBU3aX2TgRUd7g8h
```

PR body footer:

```
🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01WzJbzDqBU3aX2TgRUd7g8h
```

Then `subscribe_pr_activity` on the PR.

## 3. Gates (run before each commit, one Cargo process at a time)

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --test edit_occurrence --test edit_plan --test agent_contracts --test convert_docx_to_pdf
# Task 5 adds: --test convert_font_report, and the bin unit test: cargo test --all-features --bin jubarte
cargo run --bin jubarte -- --help >/dev/null
uv tool run --from 'reuse[charset-normalizer]' reuse lint
cd jubarte-python && uv run --with maturin maturin develop --release && uv run --with pytest --with pytest-cov pytest -q --cov=jubarte_redlines --cov-branch --cov-report=term-missing
```

Coverage before the last commit: `cargo llvm-cov --all-features --lcov
--output-path target/lcov.info` (CI floor is 80 percent lines). Record the
numbers in the PR body.

## 4. Task 4 findings (verified by reading src/edit.rs on this tree)

Line numbers are for `src/edit.rs` at b420d64.

- `OperationKind` enum: line 131, `#[serde(tag = "kind", rename_all = "snake_case")]`.
  - `Replace` lines 133-150: fields paragraph, find, replacement, format,
    comment, whole (whole is last; put `occurrence` after `whole`).
  - `Insert` lines 154-174: paragraph, after, before, position, text, format,
    comment (put `occurrence` after `comment`).
  - `Delete` lines 176-181: paragraph, find (put `occurrence` after `find`).
  - `Comment` lines 183-191: paragraph, find (Option), text (put `occurrence`
    after `text`).
  - Field attribute to use, matching neighbours:
    `#[serde(default, skip_serializing_if = "Option::is_none")]` plus a doc
    comment. The enum derives `PartialEq, Eq, Serialize, Deserialize`, so
    `Option<usize>` is fine.
- `EditOutcome.matches` (line 474) is `usize`, "Anchor occurrences found".
  Keep it the total hit count (the test asserts 3 on success).
- `check_operation_keys` line 783. Rows at 799-802:
  `"replace" => &["find","replacement","format","comment","whole"]`,
  `"insert" => &["after","before","position","text","format","comment"]`,
  `"delete" => &["find"]`, `"comment" => &["find","text"]`. Append
  `"occurrence"` at the END of each of those four rows. It is called from
  `EditPlan::from_json` (line 759), so an unknown key fails at plan parse
  with `INVALID_PLAN ... unknown field "occurrence"` (that is what the red
  run showed for most tests).
- `find_range` lines 1938-1975, signature
  `fn find_range(&self, projection: &Projection, find: &str, outcome: &mut EditOutcome) -> Result<(usize, usize), (String, String)>`.
  It collects `hits: Vec<usize>` via `char_indices` filter `starts_with`
  (overlapping hits count), sets `outcome.matches = hits.len()`, then
  `match hits.as_slice()`: `[one]` ok, `[]` ANCHOR_NOT_FOUND
  `"{find:?} does not occur in the paragraph"`, `many` AMBIGUOUS_ANCHOR
  `"{find:?} occurs {} times in the paragraph"`. Then `check_range` and
  returns `(start, start + find.len())`.
  Replace the match with the plan's version (add `occurrence: Option<usize>`
  parameter), with these exact messages, which the new tests assert:
    - no hits: ANCHOR_NOT_FOUND, same message as today (regardless of occurrence).
    - `Some(0)`: INVALID_EDIT, message must contain "1-based"
      (plan text: "occurrence is 1-based").
    - one hit, None: use it. One hit, Some(1): use it (test
      `a_unique_anchor_accepts_occurrence_one`).
    - many, None: AMBIGUOUS_ANCHOR,
      `"{find:?} occurs {n} times in the paragraph; set \"occurrence\" to 1..={n}"`.
    - many, Some(k) with k <= n: `hits[k-1]`.
    - many, Some(k) with k > n: AMBIGUOUS_ANCHOR,
      `"{find:?} occurs {n} times in the paragraph; occurrence {k} is outside occurrence 1..={n}"`.
    Order the arms so the empty-hits check comes before the Some(0) check
    (test `an_occurrence_on_a_missing_anchor_is_still_not_found` expects
    ANCHOR_NOT_FOUND with occurrence 1 on a missing anchor; occurrence 0 on
    a missing anchor is not tested, either order is acceptable there, but
    keep `[]` first as the plan sketch does).
- Five call sites of `find_range`, all inside `resolve_one` (line 1276):
  1347 (Replace), 1370 (Delete), 1418 (Insert after), 1424 (Insert before),
  1466 (Comment). Each arm destructures `OperationKind::X { .., .. }` with a
  trailing `..`; add `occurrence` to each destructure and pass `*occurrence`.
  Replace arm destructure is at 1331-1337, Delete at 1368, Insert at
  1391-1399, Comment at 1460-1462.
- The existing test `tests/edit_plan.rs:150-178`
  (`ambiguous_anchor_fails_the_whole_plan_and_reports_every_operation`)
  asserts only `err.code == "AMBIGUOUS_ANCHOR"` and the per-op matches
  counts, not the message text, so the new message wording does not break
  it. Grep `tests/agent_contracts.rs` for "occurs" before committing to be
  sure (the grep in this session found no message assertions there).
- Serialization: the round-trip test expects `occurrence` absent from JSON
  when None (skip_serializing_if) and present as 2 when set.

Python (`jubarte-python/python/jubarte_redlines/models.py`):
- `replace` at line 459 (signature ends with `id: str | None = None`),
  `insert` at 482, `delete` at 509, `comment` at 514. Each builds `op` then
  `return self._with(_with_optional(op, id=id[, comment=comment]))`.
- `_with_optional` at line 616 is typed `**extra: str | None`. Passing
  `occurrence=occurrence` (an int) needs its annotation widened to
  `str | int | None`, or set `op["occurrence"] = occurrence` directly before
  the `_with_optional` call. Either is fine; the second keeps `_with_optional`
  untouched (less conflict surface).
- Add `occurrence: int | None = None` as a keyword to all four; raise
  `ValueError("occurrence must be 1 or more")` (any wording) when
  `occurrence is not None and occurrence < 1`.
- Add one test to `jubarte-python/tests/test_edit_models.py` (existing tests
  are plain functions, pytest.raises style; see
  `test_insert_rejects_missing_or_conflicting_positions` at line 40 for the
  ValueError pattern). Assert: `to_dict()` carries `"occurrence": 2` on the
  op when given, omits the key when not given, and `occurrence=0` raises.
- Docstrings on the four builders say "the unique occurrence"; adjust to
  mention `occurrence`.

Docs:
- `skills/jubarte-documents/SKILL.md` lines 132-134 (section 2 gotchas):
  current text "`find` must occur exactly once in that paragraph; overlapping
  occurrences count (`"aa"` occurs twice in `"aaa"`). Widen the anchor
  instead of guessing." Replace with the plan's wording: "`find` must occur
  exactly once in that paragraph unless you give `occurrence` (1-based); the
  refusal says how many times it occurs. Overlapping occurrences count
  (`"aa"` occurs twice in `"aaa"`)." Also line 105-106 mentions
  `AMBIGUOUS_ANCHOR`; leave as is unless wording conflicts.
- `CHANGELOG.md`: `## [Unreleased]` at line 16, `### Added` at 18, `### Fixed`
  at 89. Add a bullet at the end of the Unreleased `### Added` list (before
  line 89), e.g. "`occurrence` (1-based) on `replace`, `insert`, `delete` and
  `comment` picks one of a repeated anchor; the `AMBIGUOUS_ANCHOR` refusal
  now says how many times the anchor occurs and the range to choose from."

Full content of the untracked `tests/edit_occurrence.rs` as written and run
(red: 10 failed; most panicked on `from_json().unwrap()` with INVALID_PLAN
unknown field "occurrence"; `without_occurrence_the_refusal_says_how_to_fix_it`
failed on the message assertion, current message is
`"fee" occurs 3 times in the paragraph`):

```rust
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A repeated anchor is editable by `occurrence` (1-based); without it the
//! refusal says how many times the anchor occurs and how to pick one.

mod common;

use common::docx::{docx, para};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;

fn source() -> Vec<u8> {
    docx(&para("fee fee fee"))
}

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"A","operations":{operations}}}"#
    ))
    .unwrap()
}

#[test]
fn the_second_occurrence_is_replaced() {
    let plan = plan(
        r#"[{"kind":"replace","paragraph":"body:p:0","find":"fee","occurrence":2,"replacement":"cost"}]"#,
    );
    let out = apply_plan(&source(), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "fee cost fee");
    assert_eq!(out.report.operations[0].matches, 3);
}

#[test]
fn the_last_occurrence_is_deleted() {
    let plan =
        plan(r#"[{"kind":"delete","paragraph":"body:p:0","find":" fee","occurrence":2}]"#);
    let out = apply_plan(&source(), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "fee fee");
    assert_eq!(out.report.operations[0].matches, 2);
}

#[test]
fn insert_after_and_before_take_an_occurrence() {
    let plan = plan(
        r#"[{"kind":"insert","paragraph":"body:p:0","after":"fee","occurrence":1,"text":"!"},
            {"kind":"insert","paragraph":"body:p:0","before":"fee","occurrence":3,"text":"?"}]"#,
    );
    let out = apply_plan(&source(), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "fee! fee ?fee");
}

#[test]
fn a_comment_anchors_to_an_occurrence() {
    let plan = plan(
        r#"[{"kind":"comment","paragraph":"body:p:0","find":"fee","occurrence":3,"text":"why?"}]"#,
    );
    let out = apply_plan(&source(), &plan).unwrap();
    let op = &out.report.operations[0];
    assert_eq!(op.status, "ok");
    assert_eq!(op.matches, 3);
    assert_eq!(op.context.as_deref(), Some("fee fee {#fee}"));
}

#[test]
fn a_unique_anchor_accepts_occurrence_one() {
    let plan = plan(
        r#"[{"kind":"replace","paragraph":"body:p:0","find":"fee fee fee","occurrence":1,"replacement":"x"}]"#,
    );
    let out = apply_plan(&source(), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "x");
    assert_eq!(out.report.operations[0].matches, 1);
}

#[test]
fn an_out_of_range_occurrence_names_the_range() {
    let plan = plan(r#"[{"kind":"delete","paragraph":"body:p:0","find":"fee","occurrence":4}]"#);
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
    assert!(
        e.message.contains("occurs 3 times") && e.message.contains("occurrence 1..=3"),
        "{}",
        e.message
    );
    assert_eq!(e.outcomes[0].matches, 3);
}

#[test]
fn an_occurrence_on_a_missing_anchor_is_still_not_found() {
    let plan = plan(r#"[{"kind":"delete","paragraph":"body:p:0","find":"fum","occurrence":1}]"#);
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert_eq!(e.outcomes[0].matches, 0);
}

#[test]
fn without_occurrence_the_refusal_says_how_to_fix_it() {
    let plan = plan(r#"[{"kind":"insert","paragraph":"body:p:0","after":"fee","text":"!"}]"#);
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
    assert!(
        e.message.contains("occurs 3 times") && e.message.contains("set \"occurrence\" to 1..=3"),
        "{}",
        e.message
    );
}

#[test]
fn occurrence_zero_is_an_invalid_edit() {
    let plan = plan(
        r#"[{"kind":"comment","paragraph":"body:p:0","find":"fee","occurrence":0,"text":"?"}]"#,
    );
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "INVALID_EDIT");
    assert!(e.message.contains("1-based"), "{}", e.message);
}

#[test]
fn occurrence_round_trips_through_the_plan_json() {
    let plan = plan(
        r#"[{"kind":"replace","paragraph":"body:p:0","find":"fee","occurrence":2,"replacement":"cost"},
            {"kind":"delete","paragraph":"body:p:0","find":"fee"}]"#,
    );
    let json: serde_json::Value = serde_json::from_str(&plan.to_json()).unwrap();
    assert_eq!(json["operations"][0]["occurrence"], 2);
    assert!(json["operations"][1].get("occurrence").is_none());
    assert_eq!(EditPlan::from_json(&plan.to_json()).unwrap(), plan);
}
```

Caveats on that test file, to check on the green run:
- `the_last_occurrence_is_deleted` uses find `" fee"` (leading space), two
  hits; deleting the second gives "fee fee". The `context` on the
  comment test assumes `context()` renders the paragraph as
  `fee fee {#fee}` (format string in the Comment arm at line 1476 is
  `{{#{}}}`); if `context()` truncates or adds ellipses, adjust the
  assertion to `contains("{#fee}")` rather than weakening the behaviour.
- `serde_json` is a dev-dependency used by other integration tests; if it is
  not, replace the round-trip JSON check with `plan.to_json().contains(...)`.
- `run cargo fmt` may rewrap the long raw-string lines; let it.

## 5. Task 5 findings (verified by reading the tree)

`src/convert/font.rs`:
- `FontStep` enum lines 95-114 with variants Embedded, Explicit, AltName,
  Theme, WordSubstitution, OpenFallback, Generic, Unknown; `as_str` at
  117-130 gives the JSON tokens ("embedded", "explicit", "altName", "theme",
  "word_substitution", "open_fallback", "generic", "unknown").
- `FontReportEntry` struct lines 139-154: requested, step, physical, bold,
  italic, synthetic. `to_json` at 156-169 builds JSON by hand with
  `json_string`/`json_bool` helpers in a `format!`. `font_report_json` at
  172-183 joins entries into an array.
- Implement `pub fn substituted(&self) -> bool` on `FontReportEntry`:
  true for `WordSubstitution | OpenFallback | Generic | Unknown`, false for
  `Embedded | Explicit | AltName | Theme` (definition is in the plan at
  line 1117). Append `"substituted":{}` to the `to_json` format string as the
  LAST field. `RenderReport::to_json` in `src/convert/mod.rs:315-330`
  re-parses `font_report_json`, so the field flows into `--report` with no
  further change.
- Public re-exports: `jubarte::convert::font_report_json` and
  `FontReportEntry` are used from the CLI (`src/bin/jubarte.rs:967`), so
  the type is public via `jubarte::convert`. `docx_render_report(docx,
  PdfOptions) -> Result<RenderReport, ConvertError>` is at
  `src/convert/mod.rs:435`; `RenderReport { page_count, pages, fonts }` at
  303.

New Rust test `tests/convert_font_report.rs` (SPDX header as the other
tests): the plan's sketch is usable; `mod common; use common::docx::docx;`
then build a paragraph with
`<w:rPr><w:rFonts w:ascii="NoSuchFont" w:hAnsi="NoSuchFont"/></w:rPr>` and
assert `entry.substituted()` and `json["fonts"][i]["substituted"] == true`
(find the entry by `requested == "NoSuchFont"` in the JSON array too, do not
assume index 0: an unstyled default-font entry may precede it). Add a second
test on a default paragraph (no rFonts) asserting that an `explicit` or
`embedded` entry reports `substituted == false`; check what step the bundled
default font actually reports on this machine before asserting (print the
report once). Expected red: `no method named substituted`.

CLI `src/bin/jubarte.rs`:
- `Command::Convert` variant lines 195-246; last field is
  `#[command(flatten)] markdown: MarkdownArgs` after `revision_palette`.
  Append the new flag at the END (after `markdown`):
  `/// Exit 4 when a requested font was substituted (see --report). #[arg(long)] fail_on_substitution: bool`.
  Also append to the Convert doc comment or `after_help`: "Exit status: 0 ok,
  1 error, 4 a requested font was substituted".
- `ConvertJob` struct lines ~877-889 (fields file, bytes, output, force,
  compress, font_report, revisions, pdf, png, dpi, report). Add
  `fail_on_substitution: bool` at the end. It is constructed in two places:
  the edit command's PDF side output around line 1668-1674 (set `false`
  there) and the main Convert arm at 1953-1964. `run_convert_any`
  (1680) copies it with `ConvertJob { bytes, ..*job }`, fine.
- `run_convert` (890-980) returns `Result<(), String>` and is wrapped by
  `exit_code` (1405) which maps Err to exit 1. The Edit command uses
  `Result<(), (u8, String)>` with `const EXIT_PLAN_REFUSED: u8 = 3` (line
  1062) and `ExitCode::from(code)` in main (2041-2046). Cleanest additive
  route: add `const EXIT_FONT_SUBSTITUTED: u8 = 4;`, change `run_convert` and
  `run_convert_any` to return `Result<(), (u8, String)>` (wrap existing
  `String` errors with `(1, e)` via `.map_err`), and in the main Convert arm
  match like the Edit arm does. Alternative with a smaller diff: keep
  `Result<(), String>`, and after writing every output in `run_convert`,
  when `job.fail_on_substitution` and any `rendered.report.fonts` entry is
  substituted, print the lines
  `substituted: {requested} -> {physical} ({step})` to stderr and return
  a sentinel the main arm maps to 4. The first route is explicit; pick it.
  Outputs (PDF, PNG, reports) must still be written before the exit-4
  return, so the report that explains the failure exists.
- Unit test beside `convert_subcommand_parses_font_report` (line 2465, in
  the bin's `mod tests`): parse `["jubarte","convert","in.docx","--fail-on-substitution"]`
  and assert the flag; existing test destructures with `..` so the new field
  does not break it. An integration test in
  `tests/convert_docx_to_pdf.rs` style (it already shells out to the built
  binary with `std::process::Command`) can assert exit status 4 and the
  stderr line on a NoSuchFont docx, and exit 0 without the flag.

Python:
- `FontResolution` dataclass at models.py:738-746 (frozen, slots):
  requested, step, physical, bold, italic, synthetic. Add
  `substituted: bool = False` as the LAST field (a default keeps
  `FontResolution(**f)` in `_decode_render_report` at line 786 working on
  older JSON, and keeps the existing test at
  `tests/test_edit_models.py:175-187`, which feeds a dict without
  `substituted`, passing).
- `RenderReport` at 758-764: add
  `@property def substitutions(self) -> tuple[FontResolution, ...]:
  return tuple(f for f in self.fonts if f.substituted)`.
- Test: extend `test_render_report_keeps_page_order_and_font_resolution_metadata`
  or add one: a fonts list with one `substituted: true` and one false,
  assert `report.substitutions == (the_true_one,)` and that a dict without
  the key decodes with `substituted is False`.
- `jubarte-python/tests/test_agent_api.py:188` touches `rendered.report.fonts`
  on a real render; the real engine will now emit `substituted`, which the
  dataclass accepts.

Docs:
- Skill section 3 ("## 3. Verify", SKILL.md line ~178) gets: "`--report`
  lists every font and whether it was substituted; `--fail-on-substitution`
  turns that into exit 4 for CI."
- CHANGELOG Unreleased `### Added`: bullet for the `substituted` field in
  the font report (`--report`, `--font-report`, Python `FontResolution`,
  `RenderReport.substitutions`) and `convert --fail-on-substitution` exit 4.

## 6. Deviations already known (for the PR body)

- Plan's Task 4 sketch passes `occurrence=occurrence` through
  `_with_optional`, whose annotation is `str | None`; see section 4 for the
  two ways to handle it.
- Plan's Task 5 Python sketch does not say whether `substituted` has a
  default; giving it `False` is required to keep the existing decoder test
  green and older reports decodable.
- Plan step 2 of Task 4 expected "four failures with INVALID_PLAN"; the
  session's file has ten tests and all ten failed, the extra ones being
  additional coverage (insert before/after, comment, unique anchor with
  occurrence 1, missing anchor with occurrence, JSON round trip).

## 7. Order of work for the next session

1. `git status` to confirm only `tests/edit_occurrence.rs` is untracked.
2. Implement Task 4 in src/edit.rs (section 4), run
   `cargo test --all-features --test edit_occurrence`, fix any assertion
   that mismatches real `context()` output without weakening behaviour.
3. Python builders + test, skill gotcha, CHANGELOG bullet. Run all gates.
   Commit 1.
4. Write `tests/convert_font_report.rs`, run red, implement Task 5 (Rust,
   CLI, Python, docs), run all gates including the Python coverage run.
   Commit 2.
5. `cargo llvm-cov` for the numbers, rebase on origin/main, rerun gates,
   push, open DRAFT PR with the GitHub MCP tool, subscribe to PR activity.
