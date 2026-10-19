@coderabbitai plan against `src/cli.rs` (`ReadArgs` conflicts at 86-137, `read_output`/`compare_output`), `src/bin/jubarte.rs` (`main`'s clap error path), `src/inspect.rs` (`Summary`); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 5: option conflicts, and JSON-shaped usage errors (#1158)

**Stack.** Step E4 of the jubarte 0.13 contract-redlining plan; branch `fix/read-option-conflicts-1158`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 5.

**Problem.** `read --no-page-markers --json` was refused 165 times in 78
trials; `--tables -p t0` 9 times; `--track-changes accepted --json` 7
(brief item 8). The refusal is a clap usage error on stderr, so `2>&1 |
jq` sees a parse error and the agent retries blind.

**Design (§A-detail 5, A).**

- `--no-page-markers` with `--json` or `--tables`: accepted, no-op,
  byte-identical output; help says `ignored with --json and --tables`.
- `--tables -p t0`: selects the table (closes #1161; `--tables` and
  `--json` number tables the same way, #1173).
- `--track-changes accepted|rejected` with `--json`: works; each
  paragraph's `text` is the settled view and `summary.track_changes`
  records the choice.
- With `--json` on any command, a usage or runtime error prints on stdout
  as one line `{"error":{"code":"USAGE","message":"…","usage":"jubarte
  read [OPTIONS] <FILE>"}}` and the same text on stderr; exit code
  unchanged.
- Audit every `conflicts_with` in `src/cli.rs` (86, 94, 104, 126, 135,
  168, 186): keep only pairs whose combination is ambiguous (`-p` with
  `--head`/`--tail` stays; `--json` with `--comments`/`--dates` becomes a
  no-op with a help note).

**Files.** `src/cli.rs` (`ReadArgs` 50-137; `read_output`/`compare_output`
~2458-2560; the clap error handler in `src/bin/jubarte.rs` `main`:
detect `--json` in argv before parsing and print the object),
`src/inspect.rs` (`Summary` 94: `track_changes: Option<String>`),
`tests/cli_parser.rs`, `tests/agent_text_view.rs`,
`tests/read_options_json.rs` (new).

**Steps.**

- [ ] Red: `no_page_markers_with_json_is_byte_identical`,
      `tables_with_paragraph_selects_the_table`,
      `track_changes_accepted_with_json_settles_text`,
      `usage_error_with_json_is_an_object_on_stdout` (exit 2 kept),
      `help_says_ignored_with_json`.
- [ ] Implement the clap changes, the snapshot's settled text, the error
      object; update `--help` lines.
- [ ] fmt, clippy, the three test files, `jubarte read --help` smoke;
      commit on `fix/read-option-conflicts-1158`.

**Acceptance.** No clap conflict remains for the three combinations;
`jubarte read x.docx --json --bogus | jq .error.code` prints `"USAGE"`.

**Related.** #1158, #1161, #1173, brief item 8.

---
