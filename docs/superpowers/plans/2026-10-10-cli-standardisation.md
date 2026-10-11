# CLI standardisation: one `-o` rule, one id grammar, one count, one writer

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development or superpowers:executing-plans, task by task. Steps use `- [ ]` checkboxes. Move this file to `docs/superpowers/plans/2026-10-10-cli-standardisation.md` when work starts (Task 0); a copy would leave an untracked original that `jubarte-wasm/build-npm.sh` refuses.

**Goal:** Every jubarte task reads the same way to an agent and a human: the `-o` extension picks the format and each extension means one thing; every change, comment and paragraph id has one grammar accepted everywhere; every revision count is the same number; every write says `wrote PATH (FORMAT, N bytes)` on stderr and `-q` silences it; retired names keep working for one release with a `note:`.

**Architecture:** `src/cli.rs` (public, `cli` feature) gains `Format::Json`, `Task`, the pure `output_format()` rule, one `RenderArgs` group, a global `quiet`, and `retired_spellings()`; `validate_matches` applies the rule at parse time (exit 2) and `parse_json` serialises `output_format`, `quiet` and `notes` so the Python and npm CLIs stop re-deriving them. `src/ids.rs` (unconditional) is the one id parser (pure syntax); `edit::origin::Ids::long_id` stays the one place a short paragraph id meets a document. `changes::carriers` folds Word's one-id style pairs and text-box twins, so `list_changes` is the one count. `src/capabilities.rs` (unconditional) holds the `-o` table so the manifest and the rule share it. The binary gains one `write_output()`; `convert::render` learns to narrow the PDF and to keep comment-anchor and revised-chrome pages. `inspect::snapshot()` is the one JSON shape. Listing text (`changes`, `comments`, tables, findings, capabilities) moves into the engine and is exported to PyO3 and wasm so the three CLIs print one copy.

**Tech stack:** Rust 2024 (MSRV 1.94), clap 4.6.7 (locked; `Command::mut_args`, `Arg::hide`, `Arg::alias`, `Arg::global` verified in `clap_builder-4.6.7`), serde; tests under `tests/` with `tests/common/docx.rs` builders and `CARGO_BIN_EXE_jubarte`; `jubarte-python` (PyO3, `maturin develop --release`), `jubarte-wasm` (`build-npm.sh`, `node --test`).

**Ships in:** 0.13.0 at the earliest (main is v0.12.0, released 2026-10-11). It must be a minor bump: a new field on a public struct that is not `#[non_exhaustive]` breaks struct literals (`docs/VERSIONING.md:145-148`). None of the structs this plan grows is (`Change`, `CommentRecord`, `Snapshot`, `PageText`, `validate::Finding`, `Capabilities`).
- Every retired spelling is announced for removal in 0.14, one release after the notes appear.
- The repo has not kept such a promise before: the `sha1_fingerprint*` aliases, deprecated since 0.10.2, are still at `src/util/sha1.rs:160-170`. Task 21 therefore adds a TODO item for the removal.

---

## Context

Three read-only surveys of the tree on 2026-10-10 found the following; every line number was re-checked against ab96fe0f on 2026-10-11:

- **`-o` lies.** `Format::of_path` (src/cli.rs:1153) knows docx/md/pdf/png only; `.json` and unknown extensions fall through to each task's default. `convert -o x.json` writes a PDF (src/bin/jubarte.rs:1759-1767); `diff -o x.json` with a Word input writes Word (`run_diff` 1586-1593); `compare -o x.pdf` writes Word bytes (jubarte.rs:1328-1346); `convert --pdf -o x.png` writes a PDF named x.png (jubarte.rs:1761; `want_pdf` at :387). `diff` with a Word input and no `-o` silently writes `<old>_v_<new>.docx` beside the input (jubarte.rs:1604; the Docx default at 1607). `compare A B` writes `_v_.docx`; `jubarte A B` prints the agent view (jubarte.rs:1242-1267, 1348-1352). diff's `--to` beats `-o` at runtime while its help says the opposite (cli.rs:513).
- **Three id families, no bridge.** The view prints `#12 @AC` (agent.rs:130-138); `accept --id` matches only the long `body:rev:12` by string equality (`ChangeFilter::matches`, changes.rs:112-123); nothing parses `#12`. Comments are `5`, `c5`, `#c5`, `comment_id: 5` in four places; `-p #c5` fails. Footnote changes print a bare `#N` that can clash with the body; header/footer changes get no tag at all.
- **Six revision counts** (`changes`, `inspect` summary, `read` header, two edit-report paths, the comparer) disagree on scope, grouping and nesting. A text box's `mc:Fallback` copy is listed twice; Word gives that copy a different, consecutive `w:id` (10 of the 100 `rejected_tracking` redlines in `tests/corpus/neurotic_docx_bench`). Word also gives a style's `w:pPrChange` and `w:rPrChange` one `w:id` (62 of those 100), so a `w:id` is not unique within a story.
- **Two "changed"s.** `read --changed` counts comments; `--changed-only` does not, and ignores header/footer revisions (`!in_chrome`, convert/mod.rs:25273). `--pages` is convert-only, PNG-only, and under `--changed-only` names PNGs by kept-page index.
- **Small things agents trip on:** "wrote" on stdout for five tasks and stderr for two; `-q` on three tasks only; `capabilities --json` is a no-op; edit/add default author "Modified User"; no `sha256:` in `read`; `validate` and `audit` have different finding shapes and exit rules; `-p` means three things. The tests already list with `changes` (`tests/m7_cli.rs:223-361`), but the examples and README still run or name the removed `jubarte revisions` (`examples/cli/visualize/run.sh:112-113`, `examples/adoption/08-accept-reject/run.sh:100, 102`, README.md:188-190).

**Live tree:** main is at ab96fe0f (v0.12.0). The tree is clean except for three untracked docs: this plan, `docs/glm_new_cli_review.md` and `docs/2026-10-09-agent-edit-follow-ups.md`. The `same_path` hardening the first draft waited for has landed (`same_path` jubarte.rs:150-189, `ensure_writable` 197-198). Line numbers below are ab96fe0f's. **Anchor by names**, and never `git checkout`/`git stash` the tree (project rule).

**Precedence:** this plan lands before `docs/2026-10-09-agent-edit-follow-ups.md` (none of it is implemented) and wins where they conflict:
- Task 6 rewrites its Task 3: the view gains `sha256:`, never `digest:`.
- Task 2's grammar binds its Task 4 and its `--move-after`: a Word paraId is spelled `pid:0DCBD3F9` wherever it is printed or accepted. A bare `-p 0DCBD3F9` would collide with the bare-number row whenever the hex is all digits (`00000005` is `p5`).

## Decisions (final 2026-10-10; amended 2026-10-11 against ab96fe0f)

| # | Decision |
|---|---|
| D1 | **Extension picks the format; each extension means one thing per task.** An `-o` with no extension or an unknown one → the task's usual format (temp files work). `--to` decides only when the extension is absent or unknown; **a known, different extension exits 2** ("-o x.png says png and --to says pdf: drop one"). A known extension the task cannot write → exit 2 naming the task that can. Without `-o`: compare, read and diff print; convert writes beside its input as today; accept, reject, append and scrub require `-o`. `Format` gains `Json`; `.txt` stays text. Every `-o` help line lists its extensions. The "wrote" line names the format. |
| D2 | **The table** (what the tests assert): |

| task | `.docx` | `.md` `.markdown` `.txt` | `.pdf` | `.png` | `.json` | none / unknown | `--to` |
|---|---|---|---|---|---|---|---|
| `compare`, `A B` | redline | agent view of the redline; **CriticMarkup when both inputs are Markdown** (kept; help states it) | rendered redline | `STEM-page-NN.png` | snapshot | docx | none |
| `convert` | Word | today's Markdown conversion | rendered | PNG pages | snapshot | md→docx, .doc→docx, else pdf | `-t`; `--pdf`/`--png` hidden aliases |
| `read` | exit 2 → `convert` | agent view | exit 2 | exit 2 | snapshot | md | none |
| `diff` | exit 2 → `compare` | the view `--format` names; also `.patch`, `.diff` | exit 2 | exit 2 | exit 2 "not yet" | md (stdout without `-o`) | `-t md` only |
| `accept` `reject` `append` `scrub` | result | agent view of the result | rendered result | PNG pages | snapshot | docx | none |

"none / unknown" is an `-o` whose extension is missing or unknown; no `-o` at all is D1's last rule. `NN` is the **document** page number, zero-padded to the width of the document's page count (at least 2).

| # | Decision |
|---|---|
| D3 | `compare` without `-o` prints the agent view (as `jubarte A B` does); `jubarte A B` ≡ `compare A B`. compare gains `ReadArgs`, `--reference-doc`, `--critic`, `--resource-path` and `RenderArgs`. `--powertools-faithful` hidden. `jubarte --help` lists tasks only. |
| D4 | `diff` writes text views only, never a file without `-o`, author default "Redline" (no git lookup). `diff-render` → `visual-diff`. |
| D5 | `--changed-only` → `--changed-pages` (hidden alias). Keeps: pages a body or note revision touches (as today), pages with a comment anchor, and the **first page of each section on which a revised header or footer paints** (Q2); the first page when none. `--pages` narrows the PDF too; PNG names use the **document** page number. Both live in `RenderArgs`, flattened into every task that can render. |
| D6 | `--pdf`/`--png` → hidden aliases of `--to pdf`/`--to png` on `convert` (so `--pdf --png` and `--to md --png` exit 2; `edit --pdf/--png` keep their "also write" meaning). `--quiet/-q` on every task (`global = true`; like every option it goes after the task, since `args_conflicts_with_subcommands` reads a task named after an option as a document). One `write_output` prints `wrote PATH (FORMAT, N bytes)` on **stderr**. |
| D7 | **One id grammar** (`src/ids.rs`; the table is in Task 2). Each kind of id takes every spelling of its kind wherever that kind is accepted. **Revisions (changes):** `r12`, `12`, `r1+2`/`1+2` (= both), `header1.r1`, `footnotes.r3`, long `body:rev:12`/`header1:rev:1`. **Comments:** `c27`, `27`. **Paragraphs:** `p12`, `12`, `header1.p2`, `footnotes.p3`, `t0.row1.cell2[.p1]`, long `body:p:12`. **Tables:** `t0`; rows `row1`, cells `cell2`, always under their table (`t0.row1.cell2`). 0.12's printed forms stay accepted as input and are never printed again: `#12`, `#1+2`, `#c27`. 0.12's cell form `t0.r1.c2[.p1]` parses through 0.13 with a `note:` and is removed in 0.14. A story is a part stem (`changes::story_id`, `inspect::story_parts`), checked syntactically and resolved against the document, never against a fixed list. Body ids print without a story (`r12`); every other story prints `{story}.r{id}` (`header1.r1`, `footnotes.r3`), as paragraphs print `header1.p2`. No printed id contains `#`. Word's own pairs are one change: a style's `w:pPrChange` + `w:rPrChange` sharing a `w:id` (Task 3), and a text box's `mc:Fallback` copy (Task 4). Any other `w:id` that names two listed changes in one story exits 2, listing the candidates. Text listings print short ids. JSON keeps `id` (long) and adds `ref` (short). Machine hints and error payloads keep the long id. |
| D7b | **`edit` resolves single changes and replies by flag.** `edit FILE -p r12 --accept` / `--reject` fills the plan's `resolve_revisions`, which runs before the plan's operations whatever the flag order; `-p` repeats (`-p r1 --accept -p r3 --reject`). `--reply TEXT` on `edit` and `add` with `-p c5` writes a `ReplyComment`; `add -p c5 --content TEXT` keeps meaning a reply. A change takes only `--accept` or `--reject`; `--accept`/`--reject` take only a change; `--reply` takes only a comment and excludes `--content`, `--delete`, `--resolve`, `--anchor` and `--style`. Any other mix exits 2 naming what fits (Task 3b). |
| D8 | **One revision count = `list_changes(docx).len()`:** one per listed change over the six revision-bearing parts that `accept`/`reject` process (main, headers, footers, footnotes, endnotes, styles; `revision_processor.rs:3371-3409`). Formatting changes count; Word's one-id style pair and a text box's twins count once. Strict documents are normalized first. Marks in other parts (`comments.xml`, `numbering.xml`) leave `inspect`'s count. `changes`, `inspect`, `read`'s header and both edit-report paths agree. |
| D9 | `Snapshot` gains `changes` and `comments` (each item with `ref`); `schema_version` stays 1 (it is shared with edit plans). `read --json` prints it, and the selection flags filter it (Q4). `changes --json` / `comments --json` stay JSON Lines whose lines are the snapshot's items. `read` gains `--json` and `--tables` (in `ReadArgs`; refused with two inputs, Q3), `-o .md\|.json`, and a `sha256:` line after `source:` (the follow-ups plan's `digest:` line becomes this one). |
| D10 | `validate --rules word\|edits\|a11y\|style\|structure` (default `word`; `--original --author` adds `edits`), `--strict`, one `Finding` shape. **Exit rule: any `word` or `edits` finding exits 2 as today; a11y/style/structure findings exit 2 on `error`, and on `warning` only with `--strict`.** A usage error in `--rules` (an unknown set or code, `edits` without `--original`) exits 2 (Q5; today `audit --rules BOGUS` exits 1). `audit` → hidden alias of `validate --rules a11y,style,structure`; `audit --json`'s one object becomes JSON Lines. MCP `docx_audit` stays. |
| D11 | `capabilities` prints text; `--json` the manifest, which gains `formats`, `ids`, `validate_rules`. That is additive for JSON readers, but the new `Capabilities` fields break Rust struct literals. The WASM text renders the manifest WASM reports (its `capabilities()` adjusts pdf, fields, png and the slim codes). |
| D12 | Default author "Redline" wherever jubarte invents one: edit/add (cli.rs:125), WASM `flagPlan`, the Rust `diff` (D4), the Python CLI's `cmd_diff` (`__main__.py:584`) and the npm CLI's `defaultAuthor()` (mjs 397-403; both drop their git lookup). The Python library's `Document.diff()` follows (Q7). |
| D13 | Retired names and flags (`text`, `inspect`, `audit`, `diff-render`, `--changed-only`, `--pdf`, `--png`) keep working as hidden aliases and print `note: X is now Y; X is removed in 0.14` on stderr once. `debug` hidden. `self-update` listed last. |
| D14 | The Python and npm CLIs follow every change to the commands they already have, in the same PR; listing text lives in the engine. Neither gains commands here (Q6): npm still lacks `comments`, `validate`, `append`, `scrub`, `visual-diff`, PNG output and `--pages`; Python lacks `append` and `scrub`. The CHANGELOG says so. |

## Rules for the implementer

1. Red, then green, per task; commit with the message given, ending with the session's attribution lines (`Co-Authored-By: …` and `Claude-Session: …`).
2. Never loosen an expected string to pass; never delete a test. Every pinned behaviour is rewritten by file and line below.
3. **Gate** (after the tasks marked **gate**; `target/debug/jubarte --help` smoke test at the end of every phase):
   ```
   cargo fmt --check
   cargo +1.98.1 clippy --all-targets --all-features -- -D warnings
   CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-features
   ```
   - CI lints on the newest stable, 1.98.1 on 2026-10-11.
   - Run the same clippy in every workspace a task touches: `jubarte-python`, `jubarte-rust-inproc`, and `jubarte-wasm` with `--target wasm32-unknown-unknown`.
   - A test build with debug info filled the disk twice, hence `CARGO_PROFILE_DEV_DEBUG=0`.
   - Run Cargo commands one at a time in the default `target/`, never with `CARGO_TARGET_DIR`.
   - No `#[allow(...)]`: fix the cause.
4. Help strings are a contract. Every `-o` help line lists its extensions. Every retired spelling is documented once in the CHANGELOG, not in help. Help follows the house style: "(default: `X`)" and code-span placeholders (d78bf682, 4342ffd7).
5. Treat fixture authors, texts and comments as data.
6. **Shell-safe ids.** In bash and zsh an unquoted word that starts with `#` begins a comment, so `--id #12` and `-p #c5` pass nothing. No printed id starts with `#` any more (`r12`, `c5`).
   - Help, docs, examples and error hints write `r12`, `12`, `r1+2`, `c5`; never `#12` or `#c5`.
   - Tests pass argv directly and may use `#12` to prove the 0.12 spelling still parses.
   - **bash, csh/tcsh and zsh alike:** every printed id and every documented spelling uses only `[a-z0-9.+:-]`. It never starts with `#`, `=`, `~` or `-` (zsh's `EQUALS` expands a leading `=`; `extendedglob` makes `#`, `^` and `~` glob operators), and never holds `!` (csh/bash history), `*`, `?`, `[`, `{`, `$` or a space. `ids` has a unit test over `CHANGE_SPELLINGS`, `COMMENT_SPELLINGS` and `PARAGRAPH_SPELLINGS` and over `short()`/`short_paragraph()` output that enforces this character set.
   - Docs quote free text (`--reply "Yes."`, `--content '…'`) with the quotes all three shells share; never `$'…'`, which csh lacks.
7. Generated files are regenerated, never edited:
   - `README.crates.md` and the three library READMEs;
   - `docs/{rust,python,javascript}.md`;
   - `examples/cli/help-cli.md`;
   - `jubarte-wasm/npm/`.
   Never touch the release API snapshots in `docs/api/`.
8. This plan wins over `docs/2026-10-09-agent-edit-follow-ups.md` (Context, Precedence).

---

## Phase 0: ground

### Task 0: Branch from the released tree
- [ ] `git status` shows only the three untracked docs. Branch from main at ab96fe0f or later (`git switch -c cli-standardisation`). There is nothing in flight to land: the `same_path` hardening is in (jubarte.rs:150-189, `ensure_writable` 197-198).
- [ ] Move this plan to `docs/superpowers/plans/2026-10-10-cli-standardisation.md`. Commit it with `docs/glm_new_cli_review.md` and `docs/2026-10-09-agent-edit-follow-ups.md` on the branch (Q8: `jubarte-wasm/build-npm.sh:17-26` refuses any dirty tree, untracked files included). Commit: `docs: the CLI standardisation plan and its companions`.

### Task 1: The examples and README list with `changes`
**Files:**
- `examples/cli/visualize/run.sh:112-113`; `VISUALIZE.md:266-280` regenerates in Task 21.
- `examples/adoption/08-accept-reject/run.sh:100, 102` (the LibreOffice line sits inside the `if` at 101-103), its committed `revisions_accepted_{jubarte,libreoffice}.log`, and its `README.md:65, 71`.
- `README.md:188-190`, the shared command set.

Steps:
- [ ] Nothing to rewrite in `tests/m7_cli.rs`: its four tests already call `changes` (223-361). The row pin at 309-318 has 5 tab fields; Task 3 makes it 6.
- [ ] Delete visualize's `revisions` lines (`changes` is at :108-111).
- [ ] In 08-accept-reject:
  - :100 → `"$JUBARTE" changes accepted_jubarte.docx > changes_accepted_jubarte.log 2>&1 || true`.
  - :102 → the same for `accepted_libreoffice.docx` → `changes_accepted_libreoffice.log`.
  - `git mv` the two logs to their `changes_` names. Regenerate them by running those two lines with `JUBARTE=$PWD/target/debug/jubarte`, not the whole script, which also re-renders PNGs.
  - Fix README :65 and :71.
- [ ] README.md:188-190: drop `revisions` (`inspect`/`text` go in Task 21). Regenerate with `scripts/gen_docs.sh`.
- [ ] Run `scripts/gen_docs.sh --check` and `cargo test --test adoption`. Commit: `docs(examples): the revisions examples list with changes`.

---

## Phase 1: ids and counts (library first; no `-o` change yet)

### Task 2: The id grammar, `src/ids.rs`
**Files:**
- Create `src/ids.rs`, with its unit tests inside.
- `src/lib.rs`: `pub mod ids;` after `pub mod fields;` (:119), unconditional, with a `///` doc.
- `src/edit.rs`: `short_number` (748-756) moves into `ids`, with no shim (a private shim would be dead code).
- `src/edit/origin.rs`: its import (:20) and uses (:48, :51, :63, :110-114).

**The grammar.** `ids` parses syntax only; resolution against a document stays in `edit::origin::Ids` (`-p`) and `changes::resolve` (`--id`). The first matching row wins:

| Token | `-p` → `parse_location` | `--id`, plan `resolve_revisions` → `parse_change_ids` | plan `comment_id` → `parse_comment_id` | `read -p` → `Select::parse` (unchanged, Q11) |
|---|---|---|---|---|
| `c27`, `#c27` (whole token) | comment 27 | Err | 27 | Err |
| `r12`, `r1+2`, `header1.r1`, `footnotes.r3`, `header1.r1+2`, `body:rev:12`, `header1:rev:1`; 0.12's `#12`, `#1+2` | those changes (`Location::Changes`; only `--accept`/`--reject` take them) | those changes | Err | Err |
| `12`, `1+2` | `p12` (`1+2`: Err) | changes 12, 1+2 | 12 (`1+2`: Err) | `p12` (`1+2`: Err) |
| `p12` | body paragraph 12 | Err | Err | `p12` |
| `p5-p7`, `p5-`, `-p7`, `5-7` | Err | Err | Err | that range |
| `t0`, `t0.row1` | Err: a table id needs a row and a cell, as `t0.row1.cell2` | Err | Err | table 0 (`t0.row1`: Err) |
| `t0.row1.cell2`, `t0.row1.cell2.p1`; retired `t0.r1.c2[.p1]` | the cell's paragraph (`.p0` when bare) | Err | Err | Err |
| `row1`, `row1.cell2`, `cell2`, `cell2.p1`, 0.12's `r1.c2`, `c2.p1` | Err: a row or cell needs its table, as `t0.row1.cell2` | Err | Err | Err |
| `header1`, `header1.p2`, `footer2`, `footnotes.p3`, `endnotes.p0` (story stems) | that story paragraph (`.p0` when bare) | Err | Err | Err |
| `body:p:12`, `header1:p:2` (long) | that paragraph | Err | Err | Err |
| anything else | Err: `-p takes a paragraph (p12 or 12), a story paragraph (header1, header1.p2, footnotes.p3), a cell (t0.row1.cell2), a comment (c5) or a revision (r12)` | Err | Err | Err (today's message) |

Every `-p` refusal reads `{given}: {why}`; the `why` is the row's message above. Every `--id` and plan-id refusal reads `{given}: expected a change id such as r12, header1.r1 or body:rev:12`. Every `comment_id` refusal reads `{given}: expected a comment id such as c27 or 27`.

Rules behind the table:
- **Short story stem:** `[a-z][a-z0-9]*` that is not itself shaped `p12`, `t0`, `c5`, `r1`, `row1` or `cell1`.
  - It is checked syntactically and resolved against the document's stories. A fixed list would refuse `hdr:rev:N`, which jubarte prints for a part named `hdr.xml`.
  - A stem outside the rule (uppercase, `_`, `-`, `.`) or a `w:id` that is not a plain decimal keeps only its long id. `short()` prints the long form for them, so every printed id parses.
- **Long ids** (`{story}:rev:{wid}`, `{story}:p:{n}`) accept any story without `:`.
- **`cN` alone is always a comment, `rN` alone always a revision.** Rows and cells are `rowN` and `cellN`, and only under their table. 0.12's view printed `in r1.c2`; `-p r1.c2` is refused with the row/cell message, never read as revision 1.
- **The retired cell form** `t0.r1.c2[.p1]` parses to the same paragraph. The binary prints `note: tN.rR.cC is now tN.rowR.cellC; tN.rR.cC is removed in 0.14` once when a `-p` value has that shape (Task 9's `retired_spellings`); a plan's JSON gets no note (CHANGELOG, Deprecated).
- **The malformed ids** of `tests/edit_short_ids.rs:266-296` stay refused end to end (ANCHOR_NOT_FOUND, as JSON strings and as `{id}`), and the new cell form gets the same six shapes.
  - 16 are parse errors: `p`, `p-1`, `p+1`, `p１`, `p1.p0`, `p1.`, `P1`, `p18446744073709551616`, `header1.p-1`, `header1.p0.extra`, `t0.r0`, `t0.r-1.c0`, `t0.r0.c+1`, `t0.r0.c0.p`, `t0.r0.c0.p0.extra`, `t0.r0.c18446744073709551616`.
  - New, also parse errors: `t0.row0`, `t0.row-1.cell0`, `t0.row0.cell+1`, `t0.row0.cell0.p`, `t0.row0.cell0.p0.extra`, `t0.row0.cell18446744073709551616`.
  - `header` parses as a story stem and is refused because the document has no story named `header`.
- **Shell spellings:** every printed id is shell-safe (Rule 6).
- **Behaviour change for edit:** `-p 6` ≡ `-p p6`. Today it is refused with "unknown paragraph id 6". CHANGELOG.

- [ ] **Failing tests** (in-module):
  - `parse_change_ids`:
    - `r12`, `#12`, `12` and `body:rev:12` → `[ChangeId{body,12}]`; `header1.r1` and `header1:rev:1` → `ChangeId{header1,1}`.
    - `ChangeId::new("footnotes","3").short() == "footnotes.r3"`; `r1+2`, `#1+2` and `1+2` → 2 ids; `header1.r1+2`[1].long() == `header1:rev:2`; `ChangeId::new("body","12").short() == "r12"`.
    - `ChangeId::new("Header_1","4").short() == "Header_1:rev:4"` (outside the stem rule, prints long); `bogus.r1` → Ok (resolution refuses it).
    - Err with `{given}: expected a change id such as r12, header1.r1 or body:rev:12`: `p12`, `#`, `#x`, `r`, `rx`, `header1.r`, `r1+`, `c5`, `header1`, `row1`, `header1#1`.
  - `parse_comment_id`: `c27`, `#c27`, `27` → 27; `p27`, `r27` → Err.
  - `parse_location`:
    - `c5`/`#c5` → `Comment(5)`; `r12`, `#12`, `header1.r1`, `r1+2` → `Changes(..)`; `27` → `Paragraph("p27")`; `t0.row1.cell2`, `t0.r1.c2`, `header1`, `footnotes.p3` and `body:p:3` → `Paragraph`.
    - `t0` and `t0.row1` → the table Err; `row1`, `row1.cell2`, `cell2`, `r1.c2` and `c2.p1` → the row/cell Err.
    - The 22 malformed ids above → Err.
  - `cell_path("t0.row1.cell2.p1")` and `cell_path("t0.r1.c2.p1")` → `Some((0, 1, 2, Some(1)))`; `is_retired_cell("t0.r1.c2") == true`, `is_retired_cell("t0.row1.cell2") == false`.
  - `short_paragraph`: `body:p:12`→`p12`, `header1:p:0`→`header1`, `header1:p:2`→`header1.p2`, `footnotes:p:3`→`footnotes.p3`; `Header_1:p:2` stays long.
  - `paragraph_number("p12"|"12")` → 12, `"p"` → None; `table_number("t0")` → 0.
- [ ] **Implement:**
```rust
pub struct ChangeId { pub story: String, pub wid: String }
impl ChangeId { pub fn new(..); pub fn long(&self)->String /*{story}:rev:{wid}*/; pub fn short(&self)->String /*r12 | header1.r1 | long when not short-safe*/; pub fn from_long(&str)->Option<Self> }
pub fn parse_change_ids(&str) -> Result<Vec<ChangeId>, IdError>   // `+` expands; stems checked by is_story
pub fn parse_comment_id(&str) -> Result<u32, IdError>
pub enum Location { Paragraph(String), Comment(u32), Changes(Vec<ChangeId>) }  // bare N → pN; edit::origin::Ids::long_id resolves the paragraph form
pub fn parse_location(&str) -> Result<Location, IdError>
pub fn cell_path(&str) -> Option<(usize, usize, usize, Option<usize>)>   // t0.row1.cell2[.p1] and the retired t0.r1.c2[.p1]
pub fn is_retired_cell(&str) -> bool
pub fn short_paragraph(long: &str) -> String
pub fn paragraph_number(&str) -> Option<usize>; pub fn table_number(&str) -> Option<usize>
pub fn short_number(text: &str, prefix: char) -> Option<usize>       // moved from edit.rs:748-756
pub fn is_story(&str) -> bool                                        // the short-stem rule above; syntactic
pub struct IdError { pub given: String, pub expected: &'static str } // Display "{given}: {expected}"
pub const CHANGE_SPELLINGS / COMMENT_SPELLINGS / PARAGRAPH_SPELLINGS: &[&str]   // the manifest's `ids` (Task 17)
pub fn comment_id_de<'de, D>(D) -> Result<u32, D::Error>             // serde: number or "c27"/"#c27"/"27"
```
- [ ] Run `cargo test --lib ids`, clippy. Commit: `feat(ids): one grammar for change, comment and paragraph ids`.

### Task 3: Short ids read and printed everywhere
**Files:**
- `src/changes.rs`:
  - `Change` 53-84; `ChangeFilter` 86-101, with `matches` at 112-123; `ChangeError` 126-144.
  - `resolve` 181-226. Its freeze compares `Group`s for equality (201-216).
  - `carriers` 278-385.
- `src/comments.rs`: `CommentRecord` 42-72, `list_comments` 113-136.
- `src/edit.rs`: `resolve_selected` 1079-1151, with its `change_err` closure at 1089-1096; the `comment_id` variants 321-352 (fields at 326, 334, 343, 351).
- `src/edit/origin.rs`: `Ids::long_id` 43-84.
- `src/edit/flags.rs`: `comment_number` 163-169 → `ids::parse_location`.
- `src/bin/jubarte.rs`:
  - `run_changes` 249-276, `run_comments` 278-305.
  - `run_resolution` 228-247 must carry the error kind: `exit_code` gives 1 for every `ChangeError` today.
  - The Accept/Reject arms 2117-2146 exit 2 on `Ambiguous`/`InvalidId`.
- `src/cli.rs`: the `Selection.ids` doc, 1326-1329.
- `Select::parse` (markdown/mod.rs:327-374) does not change (Q11).

**Tests:**
- `tests/m_selective_revisions.rs`, new:
  - Every spelling selects the same change.
  - A duplicate `w:ins` id → `ChangeError::Ambiguous`, message `r4 names 2 changes (duplicate w:id; validate --repair renumbers them): r4 insertion "one" in p0; r4 insertion "two" in p0`.
  - Two paragraphs' `w:pPrChange` sharing an id → `Ambiguous` without the `validate --repair` clause.
  - A style's `w:pPrChange` + `w:rPrChange` sharing one `w:id` lists once as `styles.rN`, and `reject --id styles.rN` resolves both.
  - A header change prints `header1.r7`, and `reject --id header1.r7` works.
  - `changes_text` exact output.
  - Keep `resolving_in_two_halves_ends_where_resolving_all_does` (232-379; `step_by(2)` and `.unwrap()` over the corpus) green. It cannot pass while a style pair is `Ambiguous`.
  - `UnknownChange`'s payload stays long (:130); its Display prints the short form.
- `tests/agent_text_view.rs:1401-1425`: rows carry `ref`; `reject --id rN`, `--id N` and (argv) `--id #N`, mixed with one long id.
- `tests/edit_plan.rs:332-358`: `"ids":["r1"]` and `"ids":["#1"]`.
- `tests/edit_short_ids.rs`:
  - Plan `comment_id:"c0"` and `"#c0"` apply.
  - `-p #c0` (argv); `-p 6` ≡ `-p p6`; `-p footnotes.p0` edits the note.
  - The cell tests (220-264) write `t0.row0.cell1`; one row keeps `t0.r0.c1` and expects the same edit.
  - The 17 malformed ids (266-296) stay ANCHOR_NOT_FOUND.
- `tests/edit_opaque_refusals.rs:57, :71`: the machine hints keep the long id. Unchanged.
- `tests/m_cli_comments.rs`: rows start `c0\tp0\t…`; the text pin at :89 loses `body:p:0`; `reply to c0`; JSON `"ref":"c0"`.
- `tests/m7_cli.rs:309-318`: the row now has 6 tab fields.

- [ ] **Implement:**
  - **`Change`:**
    - Gains `#[serde(rename = "ref")] r#ref: String` after `id`. `Change` derives `Serialize` only, so it needs no default.
    - Gains `#[serde(skip_serializing_if = "Option::is_none")] paragraph: Option<String>`: the long `body:p:N`/`header1:p:N` of the nearest `w:p`, via `inspect::body_paragraph_nodes`/`story_paragraph_nodes`. A row or cell gives its first paragraph; `styles` gives None.
  - **Word's one-id style pair** (blocker). In `carriers()`, carriers that share a `w:id` under one `w:style` are one change. The first is listed; the rest get `change: None` and the first's `group`. Without this, `Ambiguous` refuses most Word redlines.
  - **Normalizing ids.**
    - `ChangeFilter::normalized()` expands every `ids` entry to long form through `ids::parse_change_ids` (`ChangeError::InvalidId` on failure).
    - `resolve()` and `edit::resolve_selected` call it; `change_err` maps the new variants.
    - `ChangeError::Ambiguous { id, candidates }` fires when a long id still matches more than one listed change. Its message names `validate --repair` only when every candidate is an `ins`, `del`, `moveFrom` or `moveTo`: those are what `UNIQUE_REVISION_IDS` (validate.rs:570-577) renumbers.
    - The new variants break a public enum (no `#[non_exhaustive]`): CHANGELOG.
  - **`Ids::long_id`** learns bare `N`, `footnotes`/`endnotes[.pK]`, Task 2's stems and the cell path through `ids::cell_path` (`t0.row1.cell2[.pK]` and the retired `t0.r1.c2[.pK]`), resolved against the document's stories. Today it would refuse `-p footnotes.p3`, which `changes` will print. Its messages (origin.rs:120, 141, 154) spell `t0.row1.cell2`, `{head}.row{row}` and `{head}.row{row}.cell{cell}`; the module doc (7, 39-41, 86) too.
  - **Text listings:**
    - `pub fn changes_text(&[Change]) -> String`: `{ref}\t{kind}\t{target}\t{author|-}\t{short paragraph|-}\t{text:?60}[\tinside {ref}]`, then `{n} change(s)`.
    - `CommentRecord` gains `#[serde(default, rename = "ref")] r#ref: String` (= `c{id}`).
    - `pub fn comments_text(&[CommentRecord]) -> String`: `c{id}\t{short paragraph|-}\t{author}\t{text:?60}\ton {anchor:?40}[\treply to c{parent}][\tresolved]`, then `{n} comment(s)`.
  - **`comment_id` fields** get `deserialize_with = "crate::ids::comment_id_de"`; the wire format stays numeric.
  - **Long ids stay** in machine hints and error payloads.
  - **`Selection.ids` doc:** ``Only this change: `r12` (as the view and `changes` print it), `12`, `header1.r1`, `r1+2` (both), or the long `body:rev:12`. Repeatable.``
- [ ] Run the **gate**. Commit: `feat(ids): every spelling of a change or comment id is accepted; listings print the short form`.

### Task 3b: `edit -p r12 --accept|--reject`; `--reply` on edit and add
**Files:**
- `src/edit/flags.rs`: `FlagOp` (17-36) gains `accept: bool`, `reject: bool`, `reply: Option<String>` (serde default, so the bindings' JSON keeps parsing). `flag_operation` (172-345) reads `-p` through `ids::parse_location` (Task 3 already moved `comment_number` there). `plan_from_flags` (61-101) collects the change groups into `resolve_revisions` (long ids) and the rest into `operations`.
- `src/cli.rs`: `flag_operations` (189-236) groups `accept`, `reject` and `reply` by their `-p`. `Edit` (~620-683) gains `--accept`, `--reject` and `--reply TEXT`; `Add` (~690-726) gains `--reply TEXT`. Both `-p` help lines: "Where: p12, header1, footer2.p1, t0.row1.cell2, c5 (a comment) or r12 (a revision, with --accept or --reject)". The `--plan` conflict list (633) gains the three.
- `src/edit.rs`: a plan whose `operations` are empty but whose `resolve_revisions` is set applies (check `apply`'s guards; today flags never build one).

**The pairing rules** (each refusal is `-p {at}: {why}`, exit 2):
- A change (`r12`, `header1.r1`, `r1+2`, a long `…:rev:…`) takes exactly one of `--accept`/`--reject` and nothing else: "a revision takes --accept or --reject".
- `--accept`/`--reject` with anything that is not a change: "--accept and --reject take a revision (-p r12)". Both on one `-p`: "--accept and --reject exclude each other".
- `--reply` with anything that is not a comment: "--reply is for a comment (-p c5)". With `--content`, `--delete`, `--resolve`, `--anchor` or `--style`: "--reply takes no other flag".
- `add -p c5 --content TEXT` still replies; `add -p c5 --reply TEXT` is the same plan.
- The same change accepted and rejected: the engine's `REVISION_CONFLICT`, unchanged.
- A change group is not an operation: `op-N` notes number the operations only.

**Tests:**
- `src/edit/flags.rs` unit tests: `each_flag_combination_maps_to_one_kind` (637-742) gains the reply rows; `contradictory_flags_are_usage_errors` (744-833) every refusal above; a new `revisions_go_to_resolve_revisions`: `-p r1 --accept -p p0 --content x -p header1.r2 --reject` → `resolve_revisions {accept: ids [body:rev:1], reject: ids [header1:rev:2]}` and one operation; `r1+2 --accept` → two ids.
- `tests/edit_short_ids.rs`, new:
  - `edit received.docx -p r0 --accept` → exit 0; the output's `changes` no longer lists `body:rev:0`, and the other changes stay.
  - `-p r0 --reject -p p1 --content …` applies both, the rejection first.
  - `edit -p c5 --reply "Yes."` and `add -p c5 --reply "Yes."` write the same reply as `add -p c5 --content "Yes."`.
  - `-p p1 --accept`, `-p r0 --content x`, `-p r0 --accept --reject`, `-p p1 --reply x`, `-p c5 --reply x --content y` → exit 2 with the messages above.
- `tests/cli_parser.rs`: `edit a -p r1 --accept` → `operations[0].accept == true`; `add a -p c5 --reply x` → `operations[0].reply == "x"`.
- WASM: `flagPlan` passes the new keys through untouched (it calls `plan_from_flags_json`); one `jubarte-wasm/cli/test/cli.test.mjs` row for `edit -p r0 --accept` lands with Task 20.

- [ ] Run `cargo test --lib edit::flags`, `cargo test --test edit_short_ids --test cli_parser`, then the **gate**. Commit: `feat(edit): -p r12 --accept/--reject resolve one revision; --reply answers a comment`.

### Task 4: One revision count
**Files:**
- `src/changes.rs`:
  - `open` (228-232) normalizes Strict, as inspect.rs:362 does.
  - `carriers`: text-box twins.
  - A `list_changes` variant over an opened `&PartFs`, for `inspect`.
- `src/inspect.rs`:
  - `Summary.revisions` 104-105 (in `Summary` 91-120).
  - `Opened::summary` 379-471, with the revision lines at 407-418; the public `summary()` is at 231-235.
  - Delete `REVISION_CARRIERS`/`revision_count` (562-590) once `edit::settle` stops calling it.
- `src/edit.rs`:
  - `settle()` calls `inspect::revision_count` for `has_revisions` (1174-1181); switch it first.
  - `RevisionCounts` 879-893; `revision_counts` 1549-1597 (called at 1515) → `RevisionCounts::from_changes`.
  - Its unit tests at 6318-6326 and 7467-7495.
- `src/edit/tracked.rs:185-201`.
- `src/markdown/from_docx/mod.rs`: the `Facts` build at 364 and 571-600.
- `src/markdown/from_docx/header.rs`: `Facts` 67-100; `render` 230-259 and 325-368; `view:` 192-216 (the accept/reject arm at 207-215 counts `f.tags.len()`).
- Delete `agent::count_marks` (agent.rs:1203-1214) and `agent::format_change_authors` (702-716). Once the header stops calling the latter it is dead code, and clippy fails.

**Goldens:** `tests/fixtures/agent-view/received.{tracked,no-comments,accept,reject}.md` change on three lines:
- Line 5, `revisions:`, in all four.
- Line 3 in accept and reject (`# 5 revisions by AC shown as …`).
- Line 9 in all four (`AC: Ann Counsel # 5 revisions, 2 comments, …`).

**Tests:**
- `tests/m_selective_revisions.rs`: a text-box change lists once and resolves in both copies. On a corpus redline whose `mc:Fallback` copy carries a different, consecutive `w:id`, each change lists once.
- `tests/inspect_paragraphs.rs`: a body + header + styles revision → `summary.revisions == 3 == list_changes().len()`. Keep 40-47 and 217-230 green, and `tests/edit_plan.rs:309`.
- `tests/agent_text_view.rs:940, 973, 1387, 1436, 1495, 1816, 2013, 2912, 3085`.
- New:
  - Keep path: the edit report's `revisions.total` equals `list_changes(out)` filtered by the edit's author and date, on a source that already holds another author's revision.
  - Comparer path: `revisions.total == summary(redline).revisions`.
  - A Strict redline counts the same as its transitional twin.
- `docs/MARKDOWN.md:138`.

- [ ] **Header line (exact):** `revisions: 9                       # 9 marks shown as 5 changes: 1 insertion, 4 substitutions`.
  - With more: `revisions: 12 … # 9 marks shown as 5 changes: …; 2 formatting; 1 in header1`.
  - None: `revisions: 0`. Only formatting: `revisions: 1 … # 1 formatting`.
  - The per-author and `view: accept-all` lines count marks.
- [ ] **Implement:**
  - **Text-box twins** (blocker). Word gives an `mc:Fallback` copy a different, consecutive `w:id`, so pairing by `(alternate_content, w:id)` never matches.
    - In `carriers()`, pair each `mc:Fallback` carrier with the `mc:Choice` carrier at the same ordinal within that `mc:AlternateContent`: the copy `agent::shown_children` (agent.rs:408-417) shows.
    - The Fallback carrier gets `change: None` and the Choice carrier's own `group` value.
    - No new `Group` variant: `resolve` freezes by `Group` equality (201-216), so a `Group::Twin` would split the copies.
    - A carrier without a partner stays listed.
  - **Summary:** `inspect::summary`/`snapshot` set `revisions = list_changes(docx)?.len()`, over D8's six parts.
  - **Edit reports:** `RevisionCounts::from_changes(impl IntoIterator<Item = &Change>)`. The comparer path takes all changes of the redline; the keep path filters by author and date.
  - **Header:** `Facts.changes: Vec<Change>`; drop `marks`, `format_changes`, `format_authors`.
- [ ] Run the **gate**. Commit: `feat(changes): one revision count: changes, inspect, the view header and edit reports agree`.

### Task 5: The view prints `r12`, `c5`, `row1.cell2`; story tags; header and footer changes visible
**Files:**
- `src/markdown/from_docx/agent.rs`:
  - `tag_of` 97-101, `handle_of` 104-109, `join_tags` 113-127, `format_tag` 130-138 (doc 129). New `split_tag`.
  - The comment ids: id lines 882, 886 (`comments c5`, `in c9`); the table line 1011, 1069; the comment head 1123 (`c5`) and 1132 (` re c5`).
  - The table line's rows and cells: 968 (`in row{r}`), 976 (`in row{r}.cell{c}`), 1036/1038 (`row{i} p{a}[-p{b}]`).
  - `has_handle` (1326-1345) reads comment words through one helper that takes `c5` and the old `#c5`; its doc comment shows the new tags.
- `src/markdown/from_docx/mod.rs`:
  - `Writer.story: Option<&'static str>`, in `Writer` 1013-1087.
  - `agent_tag` 1984-1996 and its callers, `change_of` 1998-2007 and `revision_marks` 2009-2025.
  - The note loop at 297.
  - The header/footer `StoryFact` loop 476-541: the literal is at 525-540, and `story_text` is called at 519.
  - The doc comments that quote the view (1044, 1285, 1438, 1453).
- `src/markdown/mod.rs:481`: the doc test's tag.
- `src/markdown/anchor.rs`: `plain_anchor` (194) strips any `{>>…<<}`, so it needs no code; its test (226-237) switches to `{>>c5 @AC: Cap?<<}` and `{>>r0 @AC<<}` and keeps one 0.12 row (`{>>#0 @AC<<}`), since an agent may paste an old view.
- `src/markdown/from_docx/header.rs`:
  - `story_text` 851-920 gains `agent: Option<(&Handles, &str)>`. Its callers: mod.rs:519 and header.rs tests 987, 988, 997, 1011, 1012.
  - The `headers:`/`footers:` `text:` goes through `scalar(.., true)`.
- `src/markdown/from_docx/critic.rs`:
  - `TAG` 59-63; `display` 77-91 (calls `format_tag`); `join_tagged` 104-118 (calls `join_tags`).
  - `same_author` 93-102 also compares the story.

**What the view prints** (received.docx, plus shapes it lacks):
- `{++quarterly++}{>>r0 @AC<<}`, `{~~weekly~>monthly~~}{>>r1+2 @AC<<}`, two authors `{>>r1 @AC; r2 @JD<<}`.
- `{==Late amounts…==}{>>c5 @AC: Is one percent…<<}{>>c6 @AS re c5: Disagree.<<}`.
- Id lines `rev r12 @AC`, `comments c5`, `in c9`; table lines `cells row0 p8 row1 p9-p10`, `rev r3 @AC in row1.cell2`, `c7 in p9`.
- The header's legend line (`# 3 comments: c5 (+ reply c6), c11`) already has no `#`.

**Goldens:** all four `tests/fixtures/agent-view/received.*.md` move (every tag and comment id). Regenerate them with the test's own update path, then read the diff: only `#N` → `rN`, `#cN` → `cN` and `rN.cM` → `rowN.cellM` may change.

**Tests:**
- Pins that move with the spelling: `tests/agent_text_view.rs` (73 lines), `tests/edit_anchor_normalization.rs` (4), `tests/edit_resolved_ids.rs` (2), `tests/m_cli_agent.rs` (1), `tests/edit_short_ids.rs` (the view pins). Rewrite each to the new spelling; never loosen one.
- `tests/agent_text_view.rs`, new:
  - `[^1]: Source{++ x++}{>>footnotes.r3 @AC<<}`.
  - `default: {id: header1, text: "{++DRAFT++}{>>header1.r1 @AC<<}"}`.
  - `revisions: 3 ` and `AC: … # 3 revisions`.
  - `--changed --by AC` mentions the header change.
  - `--by AS` still finds the block whose only mark is AS's hidden comment (`has_handle` reads `c6`).
- Guard: `a_selection_keeps_the_definition_of_an_inserted_note` (3557-3596) keeps `{++[^1]: Added note++}{>>r5 @AC<<}`. Its `w:ins` wraps the body's `footnoteReference`, so the change is the body's.
- `--by` with a header-only author already works (`agent::revised_by` 376-391; test 3598-3630); nothing to add.
- Unit tests:
  - `format_tag("0@AC") == "r0 @AC"`; `format_tag("footnotes#1+2@AC") == "footnotes.r1+2 @AC"`.
  - `format_tag("1@AC|header1#2@JD") == "r1 @AC; header1.r2 @JD"`.
  - `join_tags` and `join_tagged` join only when story and handle match.
- [ ] The internal tag becomes `[{story}#]{ids}@{HH}` (internal only; never printed). `format_tag` prints `r{ids} @HH` without a story and `{story}.r{ids} @HH` with one.
- [ ] Run `cargo test --test agent_text_view`, then the **gate**. Commit (split to five files a commit; the tip is green): `feat(read): the view prints r12, c5 and row1.cell2; tags name their story; header and footer changes are visible`.

### Task 6: `sha256:` header line
**Files:**
- `header.rs`: `Facts.sha256`. `source:` renders at 159 and its block is 161-165; insert the `kv` after 165.
- `from_docx/mod.rs`: in the agent branch, call `inspect::source_sha256` (public, inspect.rs:217-223) on the bytes at :81. `Facts.source` is never None (:572).
- The four goldens gain a new line 3.
- `tests/agent_text_view.rs`: every header index after `source:` shifts by one (931-955, 970, 974, 976, 985, 990, 994, 998, 1014, 1016, 1270).
- `docs/MARKDOWN.md:134-146`.
- `docs/2026-10-09-agent-edit-follow-ups.md` Task 3 (150-213) reads `sha256:`. Its `digest:` line (:152, :163), its test (169-189) and its implementation (202-204) go.

Steps:
- [ ] Failing test: `header_lines(&view)[1] == format!("sha256: {}", source_sha256(&bytes))`, plus the goldens. `jubarte A B` digests the in-memory redline.
- [ ] Run `cargo test --test agent_text_view`, then the **gate** (other suites pin the line after `source:`). Commit: `feat(read): sha256 line after source in the header`.

---

## Phase 2: the output rule

### Task 7: `Format::Json`, `Task`, the pure `-o` rule, the shared table
**Files:**
- `src/cli.rs`:
  - 1135-1175: `Format`, with a doc comment on `Json`; `of_path` maps `"json" => Json`.
  - 1229-1241: `MarkdownArgs.from` → `value_parser = input_format_parser()`, so `--from json|pdf|png` is a parse error.
  - New `Task` and `output_format`.
- `src/capabilities.rs`: new `pub const OUTPUT_FORMATS: &[(&str, &[&str])]`, task → allowed extensions per D2. `cli.rs` is feature-gated, so the table lives here.
- `src/bin/jubarte.rs`:
  - `Input::new` (1380-1393): its "not inputs" arm (1384-1386) gains `Json`.
  - The `run_convert_any` match (1797-1939; the `Pdf|Png` arm is at 1936-1938) gets a temporary `(_, Json)` Err until Task 12.
  - `write_diff_output` (1673-1711): `Json` → "diff does not write .json yet".

**Test:** `tests/cli_parser.rs`, new `the_output_rule_is_one_table`:
- No `-o` and no `--to` → `Ok(None)` for every task. Compare, read and diff print, and convert picks by input; accept, reject, append and scrub require `-o`, so that case never reaches the rule.
- Compare, Accept, Reject, Append, Scrub: `.docx .md .txt .pdf .PNG .json` → their formats; `.tmp` and no extension → `Docx`.
- Convert:
  - `(x.tmp, Some(Png))` → Png; `(x.pdf, Some(Pdf))` → Pdf; `(None, Some(Png))` → Png.
  - `(x.png, Some(Pdf))` → Err `-o x.png says png and --to says pdf: drop one`.
- Read: `.md` and `.tmp` → Md; `.json` → Json; `.docx`, `.pdf`, `.png` → Err ``read writes .md or .json; `convert FILE -o v.{ext}` writes {ext}``.
- Diff:
  - `.md .txt .patch .diff .markdown` and `.tmp` → Md.
  - `.docx`, `.pdf`, `.png` → Err ``diff writes text (.md, .markdown, .txt, .patch, .diff); `compare OLD NEW -o c.{ext}` writes the redline as {ext}``.
  - `.json` → Err `diff does not write .json yet`; `(None, Some(Docx))` → Err naming `FILE.docx`.
- `Format::of_path("a.json") == Some(Json)`. `--to json` parses once `Json` joins the enum.
- `tests/markdown_cli.rs:765-766`: `-f pdf` becomes clap's "invalid value".
- Unit test (feature `cli`): every subcommand with a long `output` arg has a key in `OUTPUT_FORMATS`.

- [ ] **Implement:**
```rust
pub enum Format { Docx, Md, Pdf, Png, Json }   // Json: a snapshot; written, never read
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Task { Compare, Convert, Read, Diff, Accept, Reject, Append, Scrub }
impl Task { pub fn name(self)->&'static str; pub fn of_name(&str)->Option<Self> /* redline|compare, text|read, … */ }
/// `-o`'s extension decides; `--to` decides when it is missing or unknown; a known, different
/// extension is an error. `-o` with neither → the task's usual format; no `-o` and no `--to`
/// → `None` (the task prints, or convert picks by input). Pure.
pub fn output_format(task: Task, output: Option<&Path>, to: Option<Format>) -> Result<Option<Format>, String>
```
  - `allowed(task)` reads `capabilities::OUTPUT_FORMATS`; `known(Diff, path)` adds `patch`/`diff` → Md.
  - The bindings still compute diff's output themselves until Tasks 19/20, so their `diff -o x.json` writes patch text into a `.json` until then.
- [ ] Run `cargo test --test cli_parser --test markdown_cli`, then the **gate**. Commit: `feat(cli): Format::Json and the pure -o rule, one table for every task`.

### Task 8: Renderer: changed pages keep more; `pages` narrows the PDF by document number
**Files:**
- `src/convert/pdf.rs`:
  - `Page` 173-210 (struct 174, `new` 196-209). `changed` and `number` already exist; add `chrome_changed: bool` and `section: u32`.
  - `emit` (:370) takes the kept pages, and so do the font helpers `face_used_glyphs` (:1124) and `face_unicode_map` (:1634). `Page` and `Op` are not `Clone`. A generic `P: Borrow<Page>` keeps every call site unchanged, both `&[Page]` and `&[&Page]`: pdf.rs tests 2252, 2301, 2785, 2789, 2914, 3030, 3056, 3118, and mod.rs:384, 484.
- `src/convert/mod.rs`:
  - `ConvertError` gains `PageNotKept { requested }`. `PageOutOfRange` Display: "page N is out of range: the document has M pages".
  - The `PdfOptions` doc (331-335); `PageText` (387-394) gains `number` (literal at 505); the `RenderRequest::pages` doc (437-443).
  - `render` (463-525) hands the kept subset to `pdf::emit`. `selected_pages` (527-545, fn 529) maps a document index to a kept index; an absent one → `PageNotKept`.
  - `docx_to_pdf_body` (384); the layout closure (764-777); `keep_changed_pages` (7656-7664).
  - `list_comments_at_end` (7731-7834, fn 7735): listing pages have `number` 0 today; number them from `pages.last().number + 1`.
  - `Layout.section_i`: init at 22804, `+= 1` at the top of `apply_section` (22853). `fresh_page` (23145-23151) sets `page.section = self.section_i`.
  - Where `chrome_changed` gets set:
    - `close_rev_bar_at` (25268-25281): `!in_chrome` at 25273.
    - The table bar (30266-30274, 30270).
    - `hf_rev_bar` (31370; callers 31068, 31111, 31141, 31281, 31325, 31343, 31353).
- `src/bin/jubarte.rs`: PNG names keep the requested number (426-440) until Task 10.

```rust
/// `PdfOptions::changed_only`: the pages a body or note change bar marks, the pages a
/// comment is anchored on, and the first page of every section on which a revised
/// header or footer paints; the first page when none qualifies.
fn keep_changed_pages(pages: &mut Vec<Page>) { /* first_painted: BTreeMap<section, number> over chrome_changed pages; keep = changed || !comments.is_empty() || first_painted.get(&section) == Some(&number); retain, or truncate(1) */ }
```

**Edges** (the doc comment says so):
- Continuous sections never reach `apply_section`, so their pages keep the previous ordinal. Word starts no new header there either.
- In `hard_page_break` (23244-23339), the parity blank page (23303) is pushed after `apply_section` (23281), so it carries the new ordinal. The else-if branch (23331-23337) keeps the old one.
- With `titlePg`, a revised default header first paints on the section's second page. The painted rule keeps that page; the literal rule would keep the first page, which shows no mark (Q2).

**Tests:**
- `tests/convert_comment_placement.rs`, fixture `four_pages` (Alpha / Bravo ins+comment / Charlie comment / Delta del):
  - `:173` → 3 pages, numbers `[2,3,4]`.
  - `:187` → 4 pages; the listing holds both notes.
  - `:212` → keeps "Alpha" only. It needs a comment-free fixture, which does not exist yet; add one.
  - `:223` → `pdf_page_count == 4`.
  - `:542` → rename to `changed_pages_keep_the_first_page_where_a_revised_header_or_footer_paints`. Its fixture has `body_changed = true`, so one section keeps `[all[0], all[1]]`. Two sections (inline `w:sectPr`, revised header on section 2) add section 2's first page; `titlePg` on section 2 keeps its second page instead.
  - `:742` → anchors keep pages (3 pages, no "Alpha"); new: no anchor and no change → 1 page.
  - `:781` → rename to `png_selection_uses_document_page_numbers`. The full render becomes 4 pages, so 825-828 flip. `Some(vec![4])` → the listing; `Some(vec![1])` → a 1-page PDF; `Some(vec![0])` → `PageNotKept{0}` "page 1 is not among the kept pages (changed pages only)".
  - `:832` → bounds by the highest number. The pairs become `(Margin,4)`/`(End,5)`, and "document has" is at 856.
  - `:864` → 4 pages; the listing moves to `pages[3]`, and 882 flips.
  - `:925` (`cli_convert_png_selects_the_comment_listing_after_changed_pages`) → `--pages 5`, library `Some(vec![4])`, `expected.report.pages[3]` holds the listing, and `selected-page-05.png` is written.
- In-module tests at convert/mod.rs:47111 and 50349.
- `tests/convert_page_ranges.rs`, new: `the_pdf_carries_only_the_selected_pages`.
- Binary test `convert_pages_needs_png_and_an_existing_page` (3465-3484).
- Python `jubarte-python/tests/test_diff_render.py:52-54` runs with the Python suite in Task 19.

Stale docs to fix in this task: Python `render` (jubarte-python/src/lib.rs:278-316) and `docx_to_pdf` (219); the WASM `docxToPdf` doc (jubarte-wasm/src/lib.rs:196-197).

CHANGELOG (Breaking, library): `ConvertError::PageNotKept`, `PageText.number`, `RenderRequest::pages` counts document pages.

- [ ] Run `cargo test --test convert_comment_placement --test convert_page_ranges`, `cargo test --lib -- convert::`, then the **gate**. Commit: `feat(convert): changed pages keep comment anchors and revised chrome; pages narrow the PDF by document number`.

The library field keeps the name `changed_only` (Python `PdfOptions(changed_only=)`, WASM `docxToPdf(…, changedOnly)`); renaming it is a follow-up.

### Task 9: Global `-q`, `visual-diff`, the retired-spelling helper
**Files:**
- `src/cli.rs`:
  - `Cli.quiet` (in `Cli` 27-42): `#[arg(short='q', long, global=true)]`.
  - Remove `CompareArgs.quiet` (302-304), `EditOptions.quiet` (151-153) and `Append.quiet` (873-875).
  - `DiffRender` → `#[command(name="visual-diff", alias="diff-render")] VisualDiff`. Update its help examples (799-801) and the capabilities.rs:82 doc.
  - New `pub const RETIRED`. Each entry carries its task scope: `--pdf`/`--png` retire on `convert` only, since `edit` keeps them.
  - `pub fn retired_spellings<'a>(argv: impl IntoIterator<Item=&'a str>) -> Vec<(&'static str, &'static str)>`: the first non-`-` token before `--` is matched against the task names, then every retired flag token in scope; dedupe.
  - `pub fn retired_note(old, now) -> String` = `note: {old} is now {now}; {old} is removed in 0.14`.
  - A `-p` value in 0.12's cell form (`ids::is_retired_cell`) adds `("tN.rR.cC", "tN.rowR.cellC")` (Task 2).
  - `parse_json` adds `"notes": [...]` and `args.quiet` to every success object. The non-compare branch must add the global args too (1890-1898).
- `src/bin/jubarte.rs`:
  - `cli_main` (2100) prints each note on stderr before dispatch.
  - `resolve_compare` (1242: `(compare, read, explicit)`) gains `quiet`. Its callers: 2104, 2478, `job_of` 2669-2678, 2829, 2841.
  - `Job` 1223-1237; `EditJob` 871-882 (`quiet` at 881, set at 1157); `run_append` 2482-2507; the `DiffRender` arm 2365-2391.
- `jubarte-python/python/jubarte_redlines/__main__.py`: `_HANDLERS["diff-render"]` → `"visual-diff"` (:727) and the supported lists (736, 746). Without this, the Python CLI breaks (`test_diff_render.py:143-157`).

Until Tasks 12 and 14, some tasks ignore `-q`.

**Tests:**
- `tests/cli_parser.rs`:
  - `diff-render a b` → command `visual-diff`, `notes[0] == "note: diff-render is now visual-diff; diff-render is removed in 0.14"`.
  - `visual-diff` → empty notes.
  - `compare a b -q`, `a.docx b.docx --quiet` and `edit a --plan p -q` → `args.quiet == true`.
  - `edit a --plan p --pdf` → no note.
  - `-q read a.docx` → 2 with jubarte's "options go after the task" (cli.rs:1598-1615). `-q compare a b` exits 2 too, but with clap's own error.
- `tests/markdown_cli.rs:481` → `diff_takes_quiet_after_the_task`: exit 0, empty stderr. It writes no file (there is no `-o`); the `-o` case gets its empty-stderr assertion in Task 14.
- `tests/adoption.rs:203-223`: `diff-render` still runs, now with the note.
- Binary unit tests:
  - `append_parses_files_output_and_break` (2564-2615).
  - `all_options_long_and_short` (2725-2745; `quiet` at 2694).
  - `convert_and_diff_render_parse_their_page_flags` (3358-3414; `DiffRender` at 3380, 3401).
- [ ] Run the named tests, `cargo test --bin jubarte`, and the Python CLI tests touched. Commit: `feat(cli)!: -q is global, diff-render is visual-diff (old name kept with a note)`.

### Task 10: `RenderArgs`, the parse-time `-o` rule, `--pages` for PDF, PNG names by page number
**Files:**
- `src/cli.rs`:
  - `PageOptions` (1199-1227) → `RenderArgs`.
  - Convert's rendering fields (430-442 and 443-446 + 451-454) become one flatten. The `MarkdownArgs` at 447-450 stays out.
  - Diff (565-575) → `#[command(flatten)] #[serde(flatten)] render: RenderArgs`, removed in Task 14.
  - Remove the per-field `help_heading`s at 415, 435, 441, 567 and 570; the struct carries the heading.
  - `validate_matches` (1567-1804): the diff id lists (1670-1675, 1727-1740) and the convert refusal list (1774-1788), which gains `pages`, `dpi`, `revisions`, `revision_palette`.
  - Arg ids spelled as strings follow the rename: cli.rs:1672, 1730, 1777, and jubarte.rs:1597, 1782.
  - `parse_json`: `output_format` per task. `RETIRED` gains `--changed-only`, `--pdf`, `--png`.
- `src/bin/jubarte.rs`:
  - `ConvertJob` (307-329); `run_convert` (382-508).
  - `png_name` (657-660; its doc comment is stranded at 519): the caller at 426-440 names each PNG by its page's document `number` (`PageText.number`), also without `--pages` (today 1..=kept count). It pads to the width of the document's page count.
  - `run_convert_any` (1759-1788): delete the "--pages selects PNG" checks (1768-1770 and 388-390).
  - `write_diff_output` (1673-1711: `dpi` 96.0 at 1701, `pages` None at 1703); the convert arm (2147-2195).
  - `revision_style` (134-148) → `pdf_options(render: &RenderArgs, compress) -> Result<PdfOptions, String>`.

```rust
/// How `-o FILE.pdf` and `-o FILE.png` are rendered; refused with any other output.
#[derive(clap::Args, Clone, Debug, Serialize)]
pub struct RenderArgs {   // next_help_heading = "Rendering (-o FILE.pdf or FILE.png)"
    pub revisions: Revisions, pub revision_palette: Option<String>, pub dpi: f32,
    pub move_comments: bool,
    /// Keep only the pages a change touches: a body or note revision, a comment anchor, or the first page on which a revised header or footer paints. Page numbers stay the document's.
    #[arg(long, alias = "changed-only")] pub changed_pages: bool,
    /// Only these pages, by the document's page numbers: `3`, `1-3,7`. Layout still covers the whole document; the PDF carries these pages and PNGs are named by them (`STEM-page-03.png`).
    #[arg(long, value_name = "SPEC", value_parser = validate_pages)] pub pages: Option<String>,
}
```

**The rule in `validate_matches`:** this task applies it to **compare and convert only**. Diff keeps its current check until Task 14. Applying it earlier breaks `cli_parser` 603-611 and :578, `comment_placement:978`, `cli_failure_paths` 588-611, Python `test_diff_views.py:77-81`, npm `api.test.mjs:122-127` and the bindings' diff defaults.
- `to = effective_to(args)`. For convert: `--to`, else `--pdf` → Pdf, else `--png` → Png. Both aliases → `--pdf and --png name two formats`. `--to X` with an alias naming another format → exit 2.
- `kind = output_format(task, output, to)?`; exit 2 with the rule's message.
- **Rule first:** it fires before the `--update-fields` check. `cli_parser.rs:31` (`--to pdf -o out.docx --update-fields`) moves to the rule's "drop one" message; the other eight rows of that test keep `--update-fields writes a .docx`.
- When `kind ∈ {Docx, Md, Json}`, refuse any command-line-given `RENDER_ONLY = [move_comments, changed_pages, pages, dpi, revisions, revision_palette]`, plus convert's `compress`, `font_report`, `fail_on_substitution` and `report` (unless `--update-fields`): `--{flag} applies to PDF or PNG output only`.

**`parse_json`:** `args.output_format` for compare and convert, null without `-o` (diff in Task 14; accept, reject, append, scrub and read in Task 12). The key already exists for compare (`compare_json`, cli.rs:1846) and diff (1943). The bindings read it: Python `__main__.py:551, 557, 575`; npm mjs 130, 132, 149. They also read the old `changed_only` key (Python 150, 408, 433-434; npm 173, 232, 251; npm's message at mjs:233). Both move in Tasks 19/20.

**Tests:**
- `tests/cli_parser.rs`:
  - 599-628: `changed_pages` keys; `--changed-only` → true + note; `--pdf` → `output_format "pdf"` + note; `--pdf --png` → 2; `--to md --png` → 2.
  - 406-422: `--pages 1` alone (416) is now exit 0; `-o out.md --pages 1` is 2; the `--to md --png` row (415) is 2.
  - 102-139: rows 104-105 and 110.
  - Every compare/convert result carries `output_format`.
- `tests/convert_page_ranges.rs:124-163`: `-t pdf -o out.pdf --pages 1` → a 1-page PDF; `md`/`docx` stay 2.
- `tests/cli_failure_paths.rs:564` (test 558-623; rows 570, 590): `--changed-pages`; the refusal names the canonical flag.
- `tests/convert_comment_placement.rs:925-975`: `--changed-only` → `--changed-pages` (the pages moved in Task 8).
- Binary unit tests: 2643; 2993-3030; 3072-3207 (`ConvertJob` literals at 3072, 3096, 3167, 3207); 3238-3311; 3358; 3416-3465 (the `convert_job` helper at 3416).
- [ ] Run the named tests, then the **gate**. Commit: `feat(cli)!: one Rendering group; --pages for PDF by document page; --changed-only is --changed-pages`.

**Note:** these pass `--png … -o X.pdf`, which is now exit 2: `examples/cli/visualize/run.sh` (rows in Task 21), `examples/adoption/18-legacy-doc/run.sh:22, 32`, and the bench consumers (Task 21). Task 21 moves them to `-o X.png`.

### Task 11: The snapshot, `read --json`, `read --tables`
**Files:**
- `src/inspect.rs`:
  - `Snapshot` (123-139) gains `changes: Vec<Change>` and `comments: Vec<CommentRecord>` after `controls`. `Snapshot` derives `Serialize` only, so no `#[serde(default)]`.
  - `pub fn snapshot(docx) -> Result<Snapshot, InspectError>`; `inspect_json` = `to_string(&snapshot()?)`.
  - `impl Snapshot { pub fn retain(&mut self, body_paragraphs: &[usize]) }` keeps `paragraphs` by index, and the `changes`/`comments` whose `paragraph` is a kept `body:p:N`. It leaves `summary`, `stories`, `tables` and `controls` alone.
  - `pub fn tables_text(&[Table]) -> String`, lifted from `run_inspect_tables` (jubarte.rs:716-753).
- `src/markdown/mod.rs`:
  - `ReadView.selected: Option<Vec<usize>>` from the kept `Block.span`s (agent.rs `Block` 1216-1224). It flows `agent::select_blocks` (agent.rs:1366-1378; caller from_docx/mod.rs:350-359) → `ReadDocx` (markdown/mod.rs:379-385) → `read()` (484-522).
  - `pub fn read_snapshot(docx, &ReadOptions) -> Result<Snapshot, MarkdownError>`: no selection → `snapshot`; otherwise `read` with `page_markers: false`, then `retain`. Map `InspectError` → `MarkdownError::Docx`.
- `src/cli.rs`:
  - `ReadArgs` gains `json: bool` and `tables: bool`, `conflicts_with = "json"`. Inspect's help (582-591) moves here, `controls` included.
  - Both join `READ_OPTION_IDS` (1555-1565) and exit 2 with two inputs: ``--json reads one document; `-o FILE.json` writes the redline's snapshot`` (Q3).
  - Selection flags (`-p`, `--head`, `--tail`, `--changed`, `--by`) filter the JSON. View-only flags exit 2 with `--json` (Q4).
- `src/bin/jubarte.rs`: `run_text` (799-805) branches.

**Tests:**
- `tests/m_cli_agent.rs:45-59`, rewritten on `tests/fixtures/agent-view/received.docx`. The current fixture has no change, comment or table; hand XML works too.
  - `read FILE --json` has `schema_version 1` and `paragraphs[1].id == "body:p:1"`. `changes[0].ref` and `comments[0].ref` equal what `changes`/`comments` list first: `r0` and `c5` on received.docx.
  - Every `changes --json`/`comments --json` line is a substring of the snapshot. `inspect --json` equals it.
  - `read --json -p p1` keeps one paragraph and only its changes; `summary.paragraphs` is unchanged.
  - `read --json --changed` drops the unchanged paragraphs.
  - `read --tables` starts `table 0: `.
  - `a.docx b.docx --json` → 2.
- Library: `snapshot(&bytes).changes == list_changes(&bytes)`.
- `tests/inspect_tables.rs` goes through `tables_text`.
- `tests/m_cli_no_panic.rs:205-242`, the closed-pipe test, gains a `read --json` row on the big document.
- [ ] Run the **gate**. Commit: `feat(read): --json prints the snapshot with changes and comments; --tables the grid`.

### Task 12: The shared writer; convert, accept, reject, append, scrub and read go through it
**Files:**
- `src/cli.rs`:
  - `Accept`/`Reject` (353-384), `Append` (851-876) and `Scrub` (909-923) gain `#[command(flatten)] #[serde(flatten)] render: RenderArgs`.
  - `Read` (600-608) gains `output: Option<PathBuf>` (`-o`: "`.md` the agent view, `.json` a snapshot") and `force`.
  - `validate_matches` applies the rule to these, and `parse_json` emits their `output_format`.
  - In the shorthand branch (1572-1638), drop the one-file `-o` refusal (1592-1597). The read-options conflict (1575-1587) fires only when the resolved kind is not `Md`.
  - `pub fn wrote_line(path, format, bytes, pages: Option<usize>) -> String` lives here; the bindings use it in Task 19.
- `src/bin/jubarte.rs`, add:
```rust
/// One task's `-o`: where, as what, and how a Word result becomes it.
struct Output<'a> { task: Task, kind: Format, path: &'a Path, force: bool, quiet: bool, view: &'a ReadArgs, source: String, render: &'a RenderArgs, extras: ConvertExtras<'a> /* compress, report, font_report, fail_on_substitution */ }
/// A Word package, or text already in the output's format (CriticMarkup, convert's Markdown, a patch).
enum Produced<'a> { Docx(&'a [u8]), Text(&'a str) }
/// Writes `produced` as `out.kind`; says `wrote_line(..)` on stderr unless `-q`.
fn write_output(out: &Output<'_>, produced: Produced<'_>) -> Result<(), ConvertFailure>
```

`write_output`'s arms:
- `Docx` → `ensure_writable`, then write.
- `Md` + `Text` → write.
- `Md` + `Docx` → `markdown::read(bytes, ReadOptions { source, ..read_options(view)? })`, lifted from `print_agent_view` (809-833). Warnings go to stderr.
- `Json` + `Docx` → `inspect::snapshot`; `read_snapshot` for `read` with selection flags.
- `Pdf|Png` + `Docx` → `run_convert(&ConvertJob { bytes: Some(..), output: Some(path), pages: render.pages.as_deref().map(parse_pages).transpose()?, quiet, … })`.
- `Text` for the rest → internal error.

`source:` for accept, reject, append, scrub and convert is `path.with_extension("docx")` + ` (not written)`; for `read` it is the input name.

Callers that move onto the writer:
- `run_resolution` (228-247), `run_scrub` (106-116), `run_append` (2482-2507).
- The `run_convert_any` arms (1813-1935). md→md with `-o` now prints its line; with no `-o`, convert keeps `file.with_extension(usual)`.
- `run_text` (799-805).

`run_convert`: `job.quiet` replaces `say` (510-517; the field at 327-328, uses at 446-454 and 460-469). Its lines become `wrote X (pdf, N bytes, P page[s])` and `wrote DIR/STEM-page-NN.png (png, K page[s], D dpi)`. Keep `check_side_file` (208-226) for `convert --update-fields --report` (`cli_failure_paths` 323, 387, 437, 473).

Accept, reject and scrub print a success line for the first time; the CHANGELOG lists it.

**Tests:**
- New `tests/cli_output.rs`, a matrix over accept, reject, scrub, append, convert(docx) and convert(md) × `.docx .md .pdf .json .tmp`:
  - The file starts with the right magic (`PK\x03\x04`, `---\nsource: `, `%PDF`, `{"schema_version"`).
  - stdout is empty; stderr contains `wrote {path} ({kind}, `.
  - `.tmp` → `docx`, except convert(docx), where it is `pdf`.
  - convert(docx) cells pass `--track-changes accept`.
- Further cases:
  - `.png --dpi 20` → `r-page-01.png` and `(png, 1 page, 20 dpi)`; `-q` → both streams empty.
  - `convert tracked.docx -t md -o t.md --track-changes accept` has no `---\nsource:` header.
  - `read -o v.md` and `read -o v.json` work. `read -o v.docx` → 2 with "convert" and "Usage:".
  - `jubarte tracked.docx -o v.md --head 1` → 0.
  - `accept x -o r.docx --changed-pages`, `--pages 1` or `--dpi 50` → 2 "applies to PDF or PNG output only".
  - `accept x -o r.pdf --changed-pages`: the bytes equal `render(accept_changes(x), changed_only)`.
- Pin moves:
  - `tests/markdown_cli.rs` stdout pins at :193, :301, :353, :689 and :722 ("PNG page") move to stderr (`(md, `, `(pdf, `, `(png, `).
  - `tests/cli_parser.rs`: `read a -o v.md` → "md"; `read a -o v.pdf` → 2; shorthand `a.docx -o v.md` → command `read`; `a.docx -o v.docx` → 2.
  - `tests/cli_parser.rs` rows that flip: 142-162 (154-157), 231-233, 803-826, 828-861 (849 "drop -o"), 863-874 (868).
  - Binary unit tests at 2849 and 2912. The `ConvertJob` literals (3072-3222) and the `Append` pattern (2567-2575) gain the new fields.
- [ ] Run the **gate**. Commit: `feat(cli): one writer for -o: docx, md (agent view), pdf, png, json on every writing task`.

### Task 13: `compare`: the view's options, Markdown-input flags, rendering; tasks-only `jubarte --help`
**Files:**
- `src/cli.rs`:
  - In `Cli` (13-42), remove `Cli.read` (36-41) and `next_help_heading` (23).
  - `CompareArgs` (258-320):
    - The `output` help lists `.docx .md .pdf .png .json`, plus the sentence on CriticMarkup for a Markdown pair.
    - `reference_doc`, `critic` and `resource_path` move from Diff (553-564), under `help_heading = "Markdown inputs"`.
    - Add `#[command(flatten)] #[serde(flatten)] render: RenderArgs` and `#[command(flatten)] #[serde(skip)] read: ReadArgs`.
    - `powertools_faithful` gets `hide = true`.
  - `validate_matches` shorthand and compare branches (1572-1650): the conflict message becomes `read options shape the view: print it (no -o) or write it (-o FILE.md)`. It fires only when the kind is not `Md`.
  - `compare_json` (1824-1849) emits `args.view` from `compare.read`, and only when the view is the output (no `-o`, or `.md`).
  - `parse_json` (1915-1998): `compare` without `-o` needs `read` in `supported`.
  - New `top_level_command()`, used by `Cli::try_parse_from` (1813) and by `parse_json`'s model (1872).
- `src/bin/jubarte.rs`:
  - `Job` (1223-1237) gains `kind`, `render`, `reference`, `critic`, `resource_path`.
  - `resolve_compare` (1242-1267): `explicit` is a parameter, not a `Job` field; it is fed from `cli.read` at 2476, 2478.
  - `run` (1310-1359):
    - A Word pair → `compare_documents_with_settings`; otherwise `markdown::redline` with `RedlineOptions { reference, critic, images }`.
    - `kind == Md` with both inputs Markdown → `Produced::Text(diff_markdown)`.
    - No `-o` → `print_agent_view`. `-o` → `write_output`, with `source: "{default_output} (not written; -o keeps it)"`.
  - `compared` (1494-1517): the `Md` refusal goes.
  - `cli_main` (2103-2105, 2470-2478).
```rust
/// The derive model with the shorthand's options hidden: `jubarte --help` lists the tasks; `jubarte compare --help` shows every option.
pub fn top_level_command() -> clap::Command {
    let hidden: HashSet<clap::Id> = CompareArgs::augment_args(clap::Command::new("shorthand")).get_arguments().map(|a| a.get_id().clone()).collect();
    Cli::command().mut_args(move |a| if hidden.contains(a.get_id()) { a.hide(true) } else { a })
}
```

**Tests:**
- `tests/m7_cli.rs`:
  - :44 → `compare_prints_the_view_and_writes_only_with_o`. stdout starts `---\nsource: ` and contains `a_v_b.docx (not written; -o keeps it)`; no file is written. With `-o a_v_b.docx`: the file exists, stdout is empty, stderr has `wrote … (docx, `.
  - :64-83 → `--quiet`: both streams empty.
- `tests/markdown_cli.rs`:
  - 636-646: `compare old.md new.md` prints the view and writes no file.
  - 854-864: `a.docx b.docx -o changes.md` succeeds; the view has `source: a_v_b.docx` and the inserted ` and more`. (The fixture has no `Returns`.)
  - 816-866: `diff … --critic --reference-doc … -o d.docx` → `compare …`. `diff old.md new.md -t png` → `compare old.md new.md -o old_v_new.png` → `old_v_new-page-01.png`.
- `tests/agent_text_view.rs`:
  - :2519: stdout empty, stderr `wrote r.docx (docx, `.
  - New: `old.docx new.docx -o v.md --head 1` → 0 with `range:`; `-o x.docx --head 1` → 2.
- `tests/convert_comment_placement.rs:978`: `cli_diff_passes_page_options_to_pdf_rendering` → `cli_compare_passes_page_options_to_pdf_rendering`.
- `tests/cli_parser.rs`:
  - `compare a b --head 2` → `args.view.head == 2`.
  - `compare a b` with `supported=["compare"]` → 2; `compare a b -o x.docx` with the same → 0.
  - Help test 465-473: the top level lacks `--detail-threshold` and `<ORIGINAL>`. `compare --help` has `--detail-threshold`, `--changed-pages` and `--head`, and lacks `--powertools-faithful`.
- Binary unit tests 2669-2695, 2762-2777 and 2826-2844 (the `explicit` callers at 2673, 2829, 2841).
- npm `jubarte-wasm/cli/test/api.test.mjs:172` (`/drop -o/`) flips with the Task 20 rebuild.
- [ ] Run the named tests, then the **gate**. Commit: `feat(compare)!: prints the view without -o; -o decides docx, md (view), pdf, png or json; the shorthand's options leave jubarte --help`.

### Task 14: `diff` writes text only
**Files:**
- `src/cli.rs`:
  - Diff (475-576) drops `mode`, `detail_threshold`, `reference_doc`, `critic`, `resource_path` and `render`. `author: String` defaults to `"Redline"`.
  - `-o` help: "Write the view to FILE (.md, .markdown, .txt, .patch or .diff; stdout otherwise)". `--to`: "md only".
  - after_help examples: the `-o changes.docx/.pdf` and `contract.docx edited.md -o redline.docx` lines become `compare` lines.
  - `validate_matches` diff branch (1664-1752) → `output_format(Task::Diff, ..)`, plus the text-view flag checks (1687-1703). `parse_json`'s diff `output_format` (1943) follows the rule: `-o unknown.ext` → "md".
- `src/bin/jubarte.rs`:
  - Delete `default_author` (120-131) and `write_diff_output` (1673-1711).
  - `DiffJob` (1520-1537) keeps `old, new, output, from, force, settings, patch`.
  - `run_diff` (1573-1669) gives patch (`patch_documents`) or critic (`diff_markdown`, both Markdown only). Otherwise: `Err("--format critic needs both documents in Markdown; `jubarte compare OLD NEW` prints their redline as the agent view")`. stdout without `-o`; with `-o`, `write_output(Md, Produced::Text)`.
  - `run_text_diff` (1540-1571) → the writer. The diff arm: 2196-2273.

**Tests:**
- `tests/markdown_cli.rs`:
  - :461 → `the_owner_is_redline_whatever_git_says`.
  - :489 → writes nothing: stderr empty, no `_v_.docx`.
  - :513 → `-o changes.docx` and `.pdf` exit 2 with `compare OLD NEW -o changes.` and `Usage:`. Its accept/reject assertions move to `compare_writes_the_redline_markdown_asks_for`.
  - :564 → an existing `-o taken.patch` → 1 `already exists`. `-o fresh.patch` → stderr `wrote fresh.patch (md, `; `-o fresh.png` → 2.
  - 603-648, with `-t md` at 627-635: `diff contract.docx new.md` prints the patch and writes no file; `--format critic` with a Word side → 1.
  - The `-q -o` case of `diff_takes_quiet_after_the_task` (Task 9): stderr empty.
- `tests/cli_parser.rs`:
  - :426 (test 425-463) → `diff_has_no_rendering_flags`: `--revisions word` → 2; `-o r.pdf` → 2 naming compare.
  - 562-578: `-o unknown.ext` → `output_format "md"` (:578 pins null today).
  - 599-628: the diff rows → 2.
- `tests/cli_failure_paths.rs:588-611`: clap now rejects these flags before `validate_matches`; move the rows to `compare`.
- `tests/convert_comment_placement.rs:351-389`: `["convert","compare"]`. Skip the `-t` rows for compare; pass `old`.
- Examples that call `diff … -o *.docx|*.png|*.pdf` move to `compare` here:
  - `examples/cli/visualize/run.sh` 66-71, 262-274, 279, 282 (`--critic` at 271); `--changed-only` → `--changed-pages`.
  - `examples/adoption/16-compare/run.sh:61-63`: delete its `rm -f v1_v_v2.docx`.
- Docs: `docs/MARKDOWN.md` 16-19, 381-502; `docs/TEXT_DIFF.md:77`.
- [ ] Run the named tests, then the **gate**. Commit: `feat(diff)!: text only; -o takes .md/.txt/.patch/.diff; Word, PDF and PNG belong to compare; the author is Redline`.

---

## Phase 3: the remaining surface

### Task 15: validate and audit are one command
**Files:**
- `src/validate.rs`:
  - `Finding` (76-116) → `{ code, set, severity, part, location, message, word_fatal, repairable }`. `path` is renamed `location`, and the sort at :263 follows.
  - `Finding::new` sets `set="word"` and `severity = if word_fatal {"error"} else {"warning"}`.
  - `pub const RULE_SETS = ["word","edits","a11y","style","structure"]`. audit.rs:50 has a private `RULE_SETS`; reuse one.
  - `pub fn validate_with(docx, rules: &[&str], original: Option<(&[u8], &str)>, layout: …) -> Result<Vec<Finding>, ValidateError>`:
    - Empty rules → `word`. A given `original` adds `edits`.
    - `edits` without `original` → `ValidateError::Usage("rules edits needs --original and --author")`. `ValidateError` (184-196) gains this variant: CHANGELOG.
    - a11y/style/structure/codes → `audit::audit_report_with`, converted: `part = story_part(location)` for paragraph ids, `word_fatal: false`, `repairable: false`.
    - Slim WASM calls `audit_report_with(docx, rules, None)` (jubarte-wasm/src/lib.rs:1113-1116); hence the layout parameter.
  - `pub fn findings_text(&[Finding], strict) -> String`:
    - Each line: `{'*'|' '} {code}\t{severity}\t{part}#{location}` for word/edits, `{location}` for rule sets, then `\t{message}`.
    - Summary: `no findings`, or `{n} finding(s): {e} error, {w} warning, {i} info`.
  - `pub fn fails(&[Finding], strict) -> bool` = **any `word`/`edits` finding, or any `error`, or (`strict`) any `warning`**.
  - `audit_tracked` (1823) sets `set:"edits"`.
- `src/audit.rs`: `impl From<AuditFinding> for validate::Finding`. `audit_report` stays as library API.
- `src/cli.rs`:
  - `Validate` (877-903) gains `rules: Vec<String>` (`value_delimiter=','`, a value parser over `RULE_SETS` + codes, so a bad name exits 2) and `strict: bool`. Its help states the two-sentence exit rule.
  - `edits` without `--original` → exit 2 in `validate_matches`.
  - Delete `Audit` (924-945).
- `src/bin/jubarte.rs`:
  - `run_validate` (1985-2032; `ValidateJob` 1973-1981) uses the three functions.
  - `validate --repair` reports on stdout (2001-2005); its wrote line moves to stderr through `wrote_line`.
  - Delete `run_audit` (755-799) and the Audit arm (2457-2466).
- `src/capabilities.rs`: `validate_rules: Vec<String>` from `RULE_SETS`, `#[serde(default)]`. It lands here, not again in Task 17.
- The `path` → `location` rename also reaches the bindings:
  - `tests/common/validity.rs:49` (`f.path`).
  - `jubarte-python/python/jubarte_redlines/models.py:1538`.
  - The npm README fragment (:124).
  - MCP `docx_validate` (`mcp_server.py:299-311`).

**Tests:**
- `tests/validate_cli.rs:38-60, 97-147`:
  - Text: `* TEXT_INSIDE_DELETION\terror\tword/document.xml#w:body[0]/w:p[0]` and `1 finding(s): 1 error, 0 warning, 0 info`.
  - JSON row: `set "word"`, `severity "error"`, `location "w:body[0]/w:p[0]"`.
  - A word-only lead fixture still exits 2.
  - `--rules style` on bullets → 0 with `  LITERAL_BULLET\twarning\tbody:p:0\t`; with `--strict` → 2.
  - `--rules a11y` on an image without a description → 2.
  - `--rules edits` alone → 2 `edits needs --original and --author`; `--rules BOGUS` → 2.
  - `--original --author` finds `UNTRACKED_EDIT` with set `edits`.
- `tests/audit_rules.rs`, CLI block 486-558: it runs `validate --rules a11y,style,structure`, and the JSON becomes JSONL rows with `set`/`severity`/`location`. `--rules BOGUS` (553-557) goes from 1 to 2 (Q5).
- `tests/validate_public.rs:25, 64-79` (they pin `"path"`) and 444-446.
- `tests/m_cli_agent.rs`: the capabilities keys.
- [ ] Run the **gate**, plus clippy in jubarte-wasm. Commit: `feat(validate): --rules and --strict; audit findings share one shape; word findings still fail`.

### Task 16: Retired names: `text`, `inspect`, `audit`; `debug` hidden; `self-update` last
**Files:**
- `src/cli.rs`:
  - `Read` (593-608): `visible_alias = "text"` (:599) → `#[command(aliases = ["text", "inspect"])]`, no visible alias.
  - Delete `Inspect` (577-592). `Validate` gets `aliases = ["audit"]`, `Debug` gets `hide = true`, `SelfUpdate` gets `display_order = 1000`.
  - `Cli`'s after_help (25) and the `parse_json` examples (1899-1909; inspect at 1902) say `read contract.docx --json`.
  - `RETIRED` gains `text` and `inspect` → `read`, and `audit` → `validate --rules a11y,style,structure`.
  - `pub fn apply_retired(&mut Option<Command>, &[(..)])`: `audit` with empty `Validate.rules` → `["a11y","style","structure"]`.
- `src/bin/jubarte.rs`:
  - `cli_main` calls `apply_retired` after the parse.
  - Delete the Inspect arm (2274-2280) and `run_inspect` (662-714).
  - `inspect FILE` prints the view + note; `inspect --json` the snapshot; `--tables` the grid.

**Tests:**
- `tests/m_cli_agent.rs`, new `retired_names_work_once_with_a_note`:
  - `text FILE` → 0, the view, stderr exactly the note.
  - `inspect FILE --json` → `{"schema_version":1`.
  - `audit bullets` → `LITERAL_BULLET\twarning\tbody:p:0` + its note.
  - `--help` lacks `\n  inspect ` and `\n  debug `; the last task line is `self-update`.
- `tests/cli_parser.rs` 310-338 (312) and :478:
  - `supported=["read","diff"]`.
  - `inspect a --json` → command `read`, `args.json == true`, `notes[0]`.
  - `audit a` → `args.rules == ["a11y","style","structure"]`; `read a` → `notes == []`.
- `tests/agent_text_view.rs`: the tests that invoke `text` see the note on stderr.
- `tests/adoption.rs` moves to the new names, keeping one alias row each in the test above:
  - `text` (136, 289-290, 344, 430-431, 607, 617, 629, 694-695) → `read`.
  - `inspect --json` (139) → `read --json`.
  - `--png` (152, 183, 712) → `-o X.png`.
  - `diff-render` (207, 223) → `visual-diff`.
- [ ] Run the **gate**. Commit: `feat(cli): text, inspect and audit are hidden aliases that print a note; debug hidden; self-update last`.

**Note:** `scripts/gen_cli_docs.py:30-33` and `examples/cli/gen-help.sh` discover tasks from `--help`, so `inspect`, `audit` and `debug` leave the generated references by themselves.

### Task 17: `capabilities` text by default; `formats`, `ids`
**Files:**
- `src/capabilities.rs`: `Capabilities` (21-40) gains:
  - `formats: BTreeMap<String, Vec<String>>`, from `OUTPUT_FORMATS` (Task 7).
  - `ids: Ids { change, comment, paragraph: Vec<String>, r#ref: String }`, from `ids::*_SPELLINGS` plus the sentence "body ids print without a story (r12); every other story prints {story}.r{id} (header1.r1, footnotes.r3); rows and cells sit under their table (t0.row1.cell2); every id is safe unquoted in bash, csh and zsh".
  - `pub fn capabilities_text(&Capabilities) -> String`.
- `src/cli.rs:728-733`: the `json` doc reads "The manifest as JSON instead of the text summary".
- `src/bin/jubarte.rs:2333-2336`.
- `jubarte-wasm/src/lib.rs`: `capabilities()` (~1018-1032) adjusts the manifest. Extract one manifest function that both the JSON and `capabilitiesText` use, so the text never describes the native engine.

**Text layout:**
- `engine: jubarte {version} ({runtime})`.
- One `{operation}: yes|no` per `Operations` field, in struct order.
- `edit plan versions: 1`; `edit operations: …`; `stories: …`; `limits: …`.
- `validate rules: word (default), edits, a11y, style, structure`; `audit codes: …`.
- `formats: compare .docx .md .pdf .png .json; read .md .json; …`.
- `ids: revisions r12, 12, header1.r1, footnotes.r3, r1+2 (both); comments c27, 27; paragraphs p12, 12, header1.p2, t0.row1.cell2; tables t0`.
- Last line: `run with --json for the manifest`.

**Tests:**
- `tests/m_cli_agent.rs:316-346`: keep, and add the text assertions above. `--json` has `validate_rules`, `formats.read == [".md",".json"]` and `ids.comment == ["c27","27"]` (0.12's `#c27` parses but is not advertised).
- `tests/agent_contracts.rs:81`: the roundtrip.
- `tests/readme_coverage.rs` still passes.
- [ ] Run the **gate**. Commit: `feat(capabilities): text summary by default; manifest lists formats and ids`.

### Task 18: Default author "Redline"
**Files:**
- `src/cli.rs:125`; `src/edit/flags.rs:847, 857` and its unit test; `jubarte-wasm/src/lib.rs:372, 384` (`flagPlan`).
- `tests/agent_text_view.rs` has 16 `@MU`/`Modified User` pins in four tests. They become `R: Redline`, `@R`, `changed by @R` and `{>>#N @R<<}`:
  - test 1974: 2009-2026;
  - test 2102: 2138, 2142, 2172;
  - test 2200: 2290, 2310;
  - test 2334: 2365, 2370, 2392, 2430, 2435, 2456.
- Python `jubarte-python/tests/test_cli.py:379`.
- D12's binding defaults:
  - Python `cmd_diff` (`__main__.py:584`).
  - npm `defaultAuthor()` (mjs 397-403, used at 164).
  - Python `Document.diff()` (document.py:566-645; `_default_author` at 637, doc at :585; Q7).
- The generated docs that print "Modified User" are regenerated in Task 21, never edited: docs/rust.md 906/1025, docs/python.md 806/925, docs/javascript.md 432/1435/1554, examples/cli/help-cli.md 795/913, jubarte-wasm/npm/README.md:118, and the npm `.d.ts`/`.js`.
- [ ] Run the **gate**. Commit: `feat(edit): the default author of edit and add is Redline`.

---

## Phase 4: bindings parity (same PR)

Python is rebuilt from source here; npm tests run against the checked-in `jubarte-wasm/npm/` build. Each task ends with its suite green.

### Task 19: Engine exports and the Python CLI
**Files:**
- `src/cli.rs`: `compare_json` + `view_source`, the exact `source:` string for a redline view, so no binding rebuilds `_v_` names.
- `jubarte-python/src/lib.rs`:
  - New `#[pyfunction]`s: `changes_text`, `comments_text(docx, author=None, latest=False)`, `tables_text`, `read_json(docx, *, paragraphs, head, tail, changed, by)` via `read_snapshot`, `capabilities_text`, `validate_run(docx, *, rules, original, author, strict) -> (jsonl, text, failed)`, `wrote_line`.
  - `validate_json` and `audit_tracked_json` return the unified `Finding`.
- `jubarte-python/python/jubarte_redlines/__main__.py`:
  - `_pdf_options` (136-151): `changed_pages`.
  - `cmd_convert` (398-458) obeys `args.output_format`, with no `--pdf --png` pair. `_write_updated_fields` (461-476) prints on stdout; move it to stderr.
  - `cmd_compare` (537-562): no default file. `output_format` → docx; md (`_native.read_view` of the redline with `args.view_source`); json (`read_json`); pdf or png via `Document.render`.
  - `cmd_diff` (565-621) and `_diff_done` (624-631): text only, author Redline.
  - `_resolution` (665-672) and `cmd_read` take `-o`.
  - `cmd_changes`/`cmd_comments` print the engine text; `cmd_validate` → `validate_run`; `cmd_capabilities` prints text unless `--json`.
  - `_HANDLERS` (723-729): no `inspect` (`visual-diff` moved in Task 9).
  - `parse_args` (742-759) prints `parsed["notes"]` to stderr.
  - `_validate_host` (761-796) drops the `pdf/png` shim and the `inspect --tables` refusal.
  - Every `wrote` goes to stderr via `wrote_line`.
- `models.py`:
  - `Change` gains `ref` and `paragraph`; `Comment` gains `ref`; `Snapshot` gains `changes` and `comments`.
  - `Finding` is unified (`location`, :1538). The :330 comment says `read --json`.
- `document.py`: `audit()` keeps `AuditFinding`; its docstring (:481) says `validate --help`.
- `mcp_server.py`:
  - `docx_text` returns the agent view via `read_view`.
  - The docstrings at 39/121 lose `[body:p:N]`.
  - `docx_validate` (299-311) gains `rules` and `strict`, and reports `location`.

**Tests:**
- `jubarte-python/tests/test_cli.py`: 33-37, 101-102, 203, 232, 238, 255, 309-319 (311), 328-343, 403-411, 482-507 (504-505).
- `test_shared_cli.py`: :73 and 79-171.
- `test_diff_render.py`: 52-54 (Task 8's pages) and 103-158.
- `test_pdf_pages.py`: 88-108.
- `test_mcp_server.py`: :119.
- `test_agent_api.py` is unchanged: JSON ids stay long.

Python has no `append` or `scrub` (D14).

- [ ] Failing tests first. Then:
  ```
  cd jubarte-python
  uv run --with maturin maturin develop --release
  uv run --with pytest --with 'mcp>=2.2,<3' --with anyio pytest -q
  ```
  A debug build fails in zstd-sys and runs the stale extension silently.
- [ ] Commit: `feat(python): CLI parity: output_format, short ids, read --json, validate rules, capabilities text, retired notes`.

**Follow-up (not here):** Python PNG naming under `--changed-pages` needs `number` in the bindings' `PageText`, together with the `changed_only` library rename.

### Task 20: The npm CLI
**Files:**
- `jubarte-wasm/src/lib.rs`:
  - New exports: `changesText`, `commentsText`, `tablesText`, `readJson(docx, optionsJson)`, `capabilitiesText` (Task 17's manifest), `wroteLine`.
  - `validateDocument` returns unified findings. `flagPlan` defaults to `Redline` (372/384).
  - Update the `auditDocument` doc (~1100). `docxToPdf` (199-226) takes no pages parameter, so npm keeps no `--pages`.
- `jubarte-wasm/cli/bin/jubarte-redlines.mjs`:
  - `COMMANDS` (120-263): `compare` drops the `_v_` fallback and uses `args.view_source`/`output_format`. `diff` writes text only with author Redline (`defaultAuthor()` goes). `convert` obeys `output_format`. `resolution`/`read` take `-o`. `changes` prints `changesText`. `capabilities` prints text unless `--json`. No `inspect`.
  - `changed_pages` wherever the CLI reads `changed_only` (173, 232, 251; the message at :233). `validateHost` (407-447) still refuses PNG.
  - `main` (449-463) prints `parsed.notes`.
- `jubarte-wasm/cli/test/cli.test.mjs`:
  - :82, 90-100, :113, 161-165, :169, 177-192, 260-338, :356, 359-369, :379, 401-407, 420-450 (:445), 475-485 (479), 487-499, :498, 524-555, :582, 585-631, 593-610, 695-715 (711-714).
  - `jubarte-wasm/cli/test/api.test.mjs:122-127` and `:172`. CI runs only `cli.test.mjs` (ci.yml:266), so run both by hand.
  - `jubarte-wasm/npm-smoke.mjs:45`: `changes[0].ref` is short.

npm lacks `comments`, `validate`, `append`, `scrub`, `visual-diff`, PNG output and `--pages` (D14).

- [ ] Red/green loop: `ALLOW_DIRTY=1 jubarte-wasm/build-npm.sh` (it refuses a dirty tree), then `node --test jubarte-wasm/cli/test/cli.test.mjs jubarte-wasm/cli/test/api.test.mjs`.
- [ ] Commit the source: `feat(npm): CLI parity: output_format, short ids, read --json, capabilities text, retired notes`.
- [ ] Rebuild clean (`jubarte-wasm/build-npm.sh`, no `ALLOW_DIRTY`) and commit the regenerated `jubarte-wasm/npm/` on its own (precedent 9246d6f3, ab96fe0f). The `.d.ts` `Modified User` at :217/:253 becomes `Redline`.

---

## Phase 5: docs

### Task 21: Docs, skill, examples, generators, CHANGELOG
**`README.md`:**
- Quick start 75-126: line 84 is stale (the shorthand prints), plus 93-95 and 120-122; use `--id 12`.
- The shared command set, 188-191: `inspect`, `text` → `read`.
- CLI reference 362-770:
  - Compare 393-421: the `-o` table and "prints the view without -o"; 399-400 "as before" goes. Then 422-430 and 432-448.
  - Convert 450-498: `--changed-pages`, `--pages` for PDF.
  - Inspect 499-530 → Read: `--json`, `--tables`, `sha256:`.
  - Validate 531-567: `--rules`, `--strict`, the two-sentence exit rule. The Audit section becomes one sentence naming the alias.
  - 617-626.
  - Capabilities 726-735: text by default.
  - diff-render → visual-diff.
  - Debugging 752 marks `debug` hidden.
- `README.crates.md` and the library READMEs are regenerated, never edited.

**Every old id spelling, everywhere** (Arthur: anything that points to old syntax moves to current): `#12`/`#N` → `r12`/`rN`, `#1+2` → `r1+2`, `#c5` → `c5`, `t0.r1.c2` → `t0.row1.cell2`, `in r1.c2` → `in row1.cell2`, `--id body:rev:12` in prose → `--id r12` (the long form stays where JSON is shown). Sweep with `git grep -nE "#c?[0-9]|\\bt[0-9]+\\.r[0-9]|\\{>>#"` over `README.md`, `docs/`, `skills/`, `examples/`, the binding READMEs and help strings; the only survivors are `docs/api/` (release snapshots), the CHANGELOG's released sections, and tests that prove the 0.12 spelling parses.

**`docs/`:**
- `docs/MARKDOWN.md`: 11-19; 134-146 (`sha256:`, `revisions:`); 240-250 (ids); 287 and 347 (`Redline`); 315-363 (commands); Diffs 381-469 and 470-502 (diff → compare).
- `docs/TEXT_DIFF.md`: 54-58 (the `-o` rules) and 77.
- `docs/TODO.md` §7 (233-257): mark it shipped and note how it differs from D5 (the no-change case, page labels, context, the report). Add an item: remove the retired spellings in 0.14.
- `docs/python.md:30`.
- `docs/adoption/anthropic-docx-skill.md:40-52`; `docs/adoption/openai-doc-skill.md:44-56, 80-82`; `docs/adoption/mcp.md:23-34` (MCP `docx_audit` at :34 stays); `docs/adoption/plans.md:146-151`.
- `docs/2026-10-09-agent-edit-follow-ups.md`: Task 4 spells paraIds `pid:HEX` (Task 6 already moved its Task 3 to `sha256:`).

**Skill and top-level files:** `skills/jubarte-documents/SKILL.md`:
- Lines 17, 20-21, 25, 50-52, 58-73, 86, 94-99, 114, 334-350, 359-360, 369, 391.
- Every `audit` → `validate --rules`; `capabilities --json`; shell-safe ids.

Also `GEMINI.md:15`, `examples/agents/*/README.md`, and `AGENTS.md` (`debug` is hidden but documented). The first draft's `examples/cli/view-forms/README.md` item is dropped: that file does not exist.

**`examples/cli/visualize/run.sh`:**
- Rows 66-71, 93-99, 119-123, 136, 140, 149-151, 155, 189, 192-196, 262-274, 279, 282:
  - `--png … -o out/received.pdf` → `-o out/received.png`;
  - `--changed-only` → `--changed-pages`;
  - `text`/`inspect` → `read`;
  - `diff-render` → `visual-diff`.
- `show()` records `2>&1`, so a leftover retired spelling pastes its note into `VISUALIZE.md`; none may remain. Regenerate `VISUALIZE.md` (235, 266-280).

**`examples/adoption/`** (full migration, Q9):
- Hard breaks first:
  - `18-legacy-doc/run.sh:22, :32`: `--png … -o X.pdf` exits 2. Write `-o X.png` and drop the `rm -f X.pdf`.
  - `18-legacy-doc/run.sh:35`: `text` → `read`; otherwise the committed `refusal_text.txt` gains the note.
  - validate's new text changes the committed logs of `07-validate` (run.sh:46-56; README:72) and `11` (run.sh:52-77).
- Retired spellings:
  - `text` in 05:21, 08:89-96, 11:40-41, 13:17, 18:21, :30; `inspect` in 11:11.
  - `--png` in 01, 02, 04, 08, 09:25, 10:36, 11:86-87, 12:16, 13:35, 14:82, 15:71, 16:68.
  - `03-diff-render/*` and `examples/adoption/README.md:30`.
- Regenerate `09-edit-tracked/read_jubarte.md`.

**Binding docs:**
- `jubarte-python/README.fragment.md`: 34-53, 47, 76-78 (the stale `revisions`), 114.
- `jubarte-wasm/cli/README.fragment.md`: 13-28.
- `jubarte-wasm/npm/README.fragment.md`: 98, 101, 104, 107, and 124 (`path` → `location`).

**Generators**, in this order, then commit their output:
1. `cargo build`
2. `cd jubarte-python && uv run --with maturin maturin develop --release`
3. `jubarte-wasm/build-npm.sh`
4. `scripts/gen_docs.sh`: docs/rust.md, docs/python.md, docs/javascript.md, the README summary, README.crates.md, and the three library READMEs via `scripts/library_readmes.py`.
5. `JUBARTE="$PWD/target/debug/jubarte" bash examples/cli/gen-help.sh`
6. `JUBARTE="$PWD/target/debug/jubarte" bash examples/cli/visualize/run.sh`
7. The adoption examples, with the same `JUBARTE`.

The `jubarte` on PATH is 0.11.2, so always pass `JUBARTE`. Verify with `scripts/gen_docs.sh --check` (CI `docs.yml:75`) and `cargo test --test readme_coverage --test release_metadata --test adoption`.

**`CHANGELOG.md` `## [Unreleased]`:**
- **Added:**
  - `read --json` (the snapshot, now with `changes` and `comments`, filtered by the selection flags), `read --tables`, `read -o`; the `sha256:` header line.
  - One id grammar: revisions `r12`/`12`, `header1.r1`, `r1+2`, `body:rev:12`; comments `c27`/`27`; paragraphs `p12`/`12`, `footnotes.p3`, `t0.row1.cell2`; on `accept`/`reject --id`, `edit`/`add -p`, plan `resolve_revisions`/`comment_id`. 0.12's `#12` and `#c27` still parse. Every id is safe unquoted in bash, csh and zsh.
  - `edit -p r12 --accept|--reject` resolves one revision; `--reply TEXT` on `edit` and `add` answers a comment.
  - `-o FILE.md|.pdf|.png|.json` on compare, convert, accept, reject, append, scrub.
  - `--pages` for PDF. `--changed-pages` keeps comment-anchor pages and the first page where a revised header or footer paints.
  - `validate --rules`, `--strict`.
  - The `capabilities` text summary; manifest `formats`, `ids`, `validate_rules`.
  - `changes` JSON `ref` and `paragraph`; `comments` JSON `ref`.
  - Global `-q`.
  - Engine `changes_text`, `comments_text`, `tables_text`, `findings_text`, `capabilities_text` in Rust, Python and WASM.
- **Changed:** each of these is **Breaking:**
  - The `-o` extension picks the format on every task. A known extension a task cannot write exits 2; `--to` with a different known extension exits 2.
  - `compare` without `-o` prints the agent view (no `_v_.docx`).
  - `diff` writes text only and never a file without `-o`; its author defaults to Redline (Python and npm too).
  - `wrote PATH (FORMAT, N bytes)` is on stderr; accept, reject and scrub print it too.
  - The agent view prints revisions as `r12` (`header1.r1` outside the body) and comments as `c5`; table lines name rows and cells `row1`, `row1.cell2`. `changes`/`comments` listings print the same short ids, and `changes` adds a paragraph column.
  - `summary.revisions`, the view's `revisions:` and the edit report count every listed change once, across the six revision-bearing parts (formatting and styles included). Marks in `comments.xml` and `numbering.xml` no longer count.
  - Library: `ChangeError` gains `Ambiguous` and `InvalidId`; `ValidateError` gains `Usage`; `ConvertError` gains `PageNotKept`; `RenderRequest::pages` counts document pages.
  - Library: new fields on public structs that are not `#[non_exhaustive]` break struct literals: `Change.ref`/`paragraph`, `CommentRecord.ref`, `Snapshot.changes`/`comments`, `PageText.number`, `Capabilities.formats`/`ids`/`validate_rules`, and the reshaped `validate::Finding`.
  - `validate` findings have one shape (`set`, `severity`; `location` replaces `path`) and one text line. a11y/style/structure findings fail on error, and on warning with `--strict`; word/edits findings fail as before. `--rules` usage errors exit 2. `audit --json` prints JSON Lines.
  - `capabilities` prints text; `--json` for the manifest.
  - `edit`/`add` default author `Redline`.
  - PNG pages are named by document page.
  - `--pdf`/`--png` are aliases of `--to`, so `convert --pdf --png` and `--to md --png` are usage errors.
  - `edit -p 6` means `p6`.
- **Changed:** not breaking:
  - The view tags note, header and footer changes with their story, and shows header/footer changes on the `headers:`/`footers:` lines.
  - MCP `docx_text` returns the agent view.
  - The npm CLI still has no `comments`, `validate`, `append`, `scrub`, `visual-diff`, PNG or `--pages`; the Python CLI has no `append` or `scrub`.
- **Deprecated:** `text`, `inspect`, `audit`, `diff-render`, `--changed-only`, `--pdf`, `--png` and the cell id `t0.r1.c2` keep working and print `note: X is now Y; X is removed in 0.14` (a plan's JSON prints none); `debug` is hidden.
- **Fixed:**
  - `convert -o x.json` wrote a PDF; `diff -o x.json` wrote Word; `diff` wrote `_v_.docx` beside the input without `-o`; `convert --pdf -o x.png` wrote a PDF named x.png.
  - `--pages 3` under `--changed-only` wrote `-page-01.png`.
  - `changes` listed a text box's change twice.
  - `edit -p #c5` and plan `"comment_id":"c5"` were refused; `accept --id #12` was refused.
- **The released `[0.12.0]` section** still says `fields update` (:20, :72-78), which shipped as `convert --update-fields`. Its :14 links `VERSIONING.md` instead of `docs/VERSIONING.md`. Fix both in place (Q1).

**Bench consumers** (separate repo, on a branch, before any bench run uses this binary): `neurotic_docx_bench` `pdf_png_bench.py:84, 137` and `tryout_remote.py:325` pass `--png … -o X.pdf`. Under D1 that exits 2, and a jubarte failure scores zero. Move them to `-o X.png`.

- [ ] Commit: `docs: the -o rule, one id grammar, one count, read --json, validate rules, capabilities text, retired names`.

---

## Verification (end to end)

1. **Engine:**
   ```
   cargo fmt --check
   cargo +1.98.1 clippy --all-targets --all-features -- -D warnings
   CARGO_PROFILE_DEV_DEBUG=0 cargo test --all-features
   ```
   - Run the same clippy in `jubarte-python`, `jubarte-rust-inproc`, and `jubarte-wasm` (`--target wasm32-unknown-unknown`).
   - MSRV, as CI's job does (ci.yml:139-149): `cargo +1.94 clippy --all-targets --all-features -- -D warnings` and `cargo +1.94 test --all-features`.
2. **Coverage**, as CI's nightly job does (ci.yml:95-133; there is no local script). Floors: 95 % lines, 88 % branches. Report lines and branches in the PR.
   - The touched modules: `CARGO_PROFILE_DEV_DEBUG=0 cargo +nightly llvm-cov --all-features --branch --test cli_output --test cli_parser --test m_selective_revisions --test convert_comment_placement --test validate_cli --summary-only`.
   - The whole workspace: `cargo +nightly llvm-cov --all-features --workspace --branch --doctests --summary-only`.
3. **Smoke:**
   - `target/debug/jubarte --help`: tasks only, `self-update` last, no `inspect` or `debug`.
   - `jubarte compare --help`: the Rendering, Markdown inputs and Read options headings; `-o` lists five extensions.
   - `jubarte diff --help`: `-o` lists `.md .markdown .txt .patch .diff`.
4. **The table by hand**, in a scratch directory holding a copy of `tests/fixtures/agent-view/received.docx`. Here `jubarte` is the branch's `target/debug/jubarte` (the one on PATH is 0.11.2):
   - `jubarte received.docx -o v.md`, `-o v.json`, and `-o v.docx` (exit 2).
   - `jubarte accept received.docx -o a.pdf --changed-pages`: stderr `wrote a.pdf (pdf, …, N pages)`.
   - `jubarte accept received.docx -o a.docx`, `jubarte scrub received.docx -o s.docx`, `jubarte append received.docx received.docx -o ap.docx`.
   - `jubarte convert received.docx --to pdf -o x.png`: exit 2, "drop one".
   - `jubarte text received.docx 2>&1 >/dev/null | head -1`: the note.
5. **Word validity** of `a.docx`, `s.docx` and `ap.docx`:
   - Run `scripts/word_pdf.py` from the neurotic_docx_bench root: one Word job machine-wide, `--timeout 300`. A repair prompt fails there.
   - Run `tools/validate-docx/bin/Release/net8.0/validate-docx` on each; it must exit 0.
   - A failure follows AGENTS.md "When Word cannot open a jubarte file".
6. **Bindings:**
   - The Python suite (Task 19's command).
   - `node --test jubarte-wasm/cli/test/cli.test.mjs jubarte-wasm/cli/test/api.test.mjs`.
   - `target/debug/jubarte capabilities`, `node jubarte-wasm/cli/bin/jubarte-redlines.mjs capabilities` and (in `jubarte-python`) `uv run python -m jubarte_redlines capabilities` print the same text. Only line 1 (`engine: … (runtime)`) and the operations a runtime lacks (npm `png: no`) differ.
7. **Docs:** `scripts/gen_docs.sh --check`; `cargo test --test readme_coverage --test release_metadata --test adoption`.
8. **PR:**
   - One branch off main (Task 0); one commit per task, with Task 20's npm artifacts in their own commit.
   - Title: `feat(cli)!: one -o rule, one id grammar, one count, one writer`. The CHANGELOG's `**Breaking:**` items mirror the list above.
   - Merging to main waits for Arthur.

## Out of scope (follow-ups, named so nobody drifts into them)
- Renaming the library field `PdfOptions::changed_only` and the bindings' `changed_only`/`changedOnly` keywords; `number` in the bindings' `PageText`, and Python PNG naming by document page.
- `diff -o x.json` (the patch as JSON).
- npm `comments`, `validate`, `append`, `scrub`, `visual-diff`, PNG and `--pages`; Python `append` and `scrub` (D14, Q6).
- WASM `getRevisions` (jubarte-wasm/src/lib.rs:170-180) keeps its own grouping; aligning it with `list_changes` is a follow-up.
- `read -p` learning story, cell and comment ids (Q11).
- The follow-ups plan's `--nth`, `--digest` (it reads `sha256:`), `--pids` and `--move-after`. They are bound by Task 2's grammar, with paraIds as `pid:HEX` (Q12).
- `-p` meaning `--part` on the hidden `debug`.
- `jubarte FILE --author X` (and every compare-only flag with one file) is silently ignored today (docs/glm_new_cli_review.md). Refusing it is the next small fix (Q13).

## Decisions taken with Arthur (2026-10-11)

Arthur accepted every default below; Q10 was replaced by his own decision.

| Q | Question | Decision |
|---|---|---|
| Q1 | Fix the released `[0.12.0]` CHANGELOG text (`fields update`, the VERSIONING link) in place? | Yes, in place (Task 21) |
| Q2 | `--changed-pages` keeps the first page where a revised header/footer **paints**, or the section's first page literally? | Paints (D5, Task 8) |
| Q3 | `--json`/`--tables` with two inputs? | Exit 2, pointing at `-o FILE.json` (Task 11) |
| Q4 | Which read options apply to JSON? | Selection flags filter; view-only flags exit 2 (Task 11) |
| Q5 | `--rules` usage errors exit 2 (today `audit --rules BOGUS` exits 1)? | 2 (D10, Task 15) |
| Q6 | D14: bindings only follow the commands they already have? | Yes; npm `comments` is a follow-up |
| Q7 | Python library `Document.diff()` default author → Redline? | Yes (D12, Task 18) |
| Q8 | Commit the plan and its two companion docs on the branch? | Yes (Task 0) |
| Q9 | Migrate every retired spelling in `examples/adoption`, or only the hard breaks? | Every one (Task 21) |
| Q10 | Ids for revisions, rows and cells? | Arthur: `rN` is a revision, `rowN` a row (and `cellN` a cell); `edit -p rN --accept/--reject`; `--reply` on edit and add (D7, D7b, Tasks 2, 3b, 5). Every id is bash/csh/zsh-safe (Rule 6). |
| Q11 | `read -p` keeps today's grammar (pN, N, ranges, tN)? | Yes; stories/cells/comments are a follow-up |
| Q12 | The follow-ups plan spells paraIds `pid:0DCBD3F9`? | Yes (Precedence) |
| Q13 | Refuse compare-only flags with one file? | Follow-up, not this PR |

## How this plan ships (Arthur, 2026-10-11)

- Stacked PRs, one per task or small group of tasks, each based on the one before; Arthur merges.
- No commit touches more than five files; commit often. A commit inside a PR may be red when one change spans more files; the PR's tip is green.
- No Claude subagents. After each PR's work, `pi -p` (a different model) reviews it adversarially; its confirmed findings are fixed on the same branch.
- Nobody waits on CI.
