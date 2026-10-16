# Markdown round-trip: implementation handoff

jubarte-redlines, 2026-10-16, about 16:40. This continues the handoff behind PR #515 (`fix/md-roundtrip-fidelity`, draft, into `feat/md-story-format`). The goal: make docx -> Markdown -> docx exact, and make the `md_fixed_point` gate green.

**Tracking issue:** #516. Sub-issues #517-#532 and #534. @coderabbitai posted plans on each.

**Base of the stack:** `test/md-fixed-point-only` (da476c07, on #515's cab8ab73). It adds `MD_FIXED_POINT_ONLY=<substr>` to `tests/md_fixed_point.rs`, so the gate replays only the fixtures whose path contains that text. Every branch below starts from it, except zip.

**Pushed with this handoff:**
- every branch that has commits;
- every agent's uncommitted worktree as `wip/<issue>` (a snapshot commit on the branch head, tracked and untracked files);
- this file, on `test/md-fixed-point-only`.

No PRs are open for the stack yet.

## Arthur's decisions (2026-10-16)

1. Nested change: the Word reader splits the insertion around a change it holds; the parser stays one level deep.
2. `--commonmark` always escapes CriticMarkup and Markdown syntax.
3. No bare spec-CriticMarkup output.
4. Literal delimiters in code: the easiest and cleanest rule. Implemented as the fence rule (fences read CriticMarkup) plus an escape that is unwound inside fences.
5. zip: full migration.
6. `MAX_PAIRINGS = 400`: remove it. The positional fallback was the bug.

## State by issue

| Issue | Branch | State |
| --- | --- | --- |
| #517 nested change | `fix/md-517-nested` 0d3bf81f | `Node::Split` + `unnest()`. Gap: an insertion whose only content is a deletion lost the outer insertion. Agent 517c (zai) is implementing `{--two--}{>>r3 @BO on r1 @AL<<}` (the `on` grammar is still to be confirmed by Arthur). WIP: `wip/517`. |
| #518/#519 escaping | `fix/md-518-519-escape` e51092ce | Escape in code blocks, unwound in fences; `--commonmark` always escapes. Regression: a hand-written `\\` in a fence collapses. The fix agent (518fix) stopped on zai's rate limit; WIP in `wip/518fix`. Resume prompt: `/tmp/rt-prompts/full-518fix-resume.md`, still queued. |
| #520/#523/#525 closers | `fix/md-520-523-closers` bdeb5eef | Nested spans opaque to closers; `Token::CommentBreak` (U+2E2E) for blank lines in comments; malformed-markup warnings. Reviewer r520b (zai) is fixing a stand-in leak inside HTML blocks. WIP: `wip/520`. |
| #521/#522/#523 diff | `fix/md-521-522-diff` 300987df | Atoms for code spans, image targets and math; pairing without `MAX_PAIRINGS`. The Devin review found 7 defects. Fix agent 521fix (zai) is working on them: no positional fallback, sparse inverted-index pairing, Myers reflow, `critic::Literal` reuse, an accept/reject property test. WIP: `wip/521` (scratch tests `markdown_diff_*.rs` included; prune before a PR). |
| #524 zip 9 | `fix/zip9-migration` 9ad624a4 (on main) | zip 9.0.4; rdocx-opc 0.2.0 ported in-tree to `src/opc/`. The handoff's premise was wrong: no rdocx-opc release drops zip 8. Review posted on #524. Output is byte-identical to main, but the commit claims a refusal of undecodable names that is dead code (zip 9's `name()` never fails) and the planned test is missing. Not run yet: wasm-pack, python binding, `scripts/security-check.sh`. |
| #526 frames | `fix/md-fp-526-frames` 1d89c93d | `src/markdown/frame.rs`; `frame: "(hAnchor=...)"` in story `pN:` maps. The gate passes for ids-header-footer and received. Done; needs a review. |
| #527 drawings | `fix/md-fp-527-drawings` (no commit yet) | Agent 527 (Devin) is running. New `src/markdown/from_docx/drawing.rs` (about 900 lines); a header drawing now round-trips; demo.docx exact. WIP: `wip/527`. |
| #528 defaults and style names | `fix/md-fp-528-defaults` c3f774f7 | 7 commits: titlePg off, section start type, stated default style, empty paragraph style, heading outline level, LineNumber, renamed note style. Agent 528 (zai) is still running. WIP: `wip/528`. |
| #529 highlights | `fix/md-fp-529-highlights` f6bfe596 | All 16 colours and their extents; zero highlight verdicts across the gate; the colour-erasing rewrite in the gate was removed. Reviewer r529 (zai) is attacking run coalescing in `push()`. |
| #530 revision metadata | `fix/md-fp-530-revisions` (no commit yet) | ids-spacing-threads, 68526de79b and ids-revisions pass the gate. Fixed: del/ins kind kept through `rev`/`unplaced`, repeated `w:id` that Word writes, row promotion, space-only insertions, endnote and style `rPrChange`, footer `fmt` placement, table and story style names, `titlePg`, pad-cell alignment. Two Devin runs died; agent 530c (zai) is finishing it with corrections: drop the header-picture exclusion it added to the gate, leave `pages 11->13` on ids-tables (direct `w:spacing` the view cannot express), prove that the regenerated fixtures match. WIP: `wip/530`, older snapshot in `refs/backup/530-wip-1` (local). |
| #531 Strict | `fix/md-fp-531-strict` (no commit yet) | Strict01: 158 correct / 0 wrong / 0 lost / 0 new (was 65/81/6/7). The remaining 17 docx lines are drawings (#527) and page count. **Rejected:** it numbered text-box paragraphs into `pN`, so the view's numbering would differ from edit plans' `body:p:N`. Agent 531b (zai) must restore one numbering, or stop and explain. It also adds Strict measures to `normalize_universal_measures` in the comparer; that is kept only with a unit test, as its own commit. WIP: `wip/531`. |
| #532 Pandoc manuscripts | none | Needs Arthur: I recommend a per-block source map, so hand-written Markdown comes back as written. |
| #534 md -> docx nondeterminism | none | New, pre-existing on main: the `row_marks` HashMap (`write.rs:340`) picks the range-end timestamp in random order. Fix with a `BTreeMap` plus a repeat-conversion test. Not started. |

## zip 8 is still in the tree after the migration

`fix/zip9-migration`'s `Cargo.lock` holds two zips:
- `zip 9.0.4`, ours: `src/opc`;
- `zip 8.6.0`, yanked, pulled by `ooxmlsdk 0.13.1` (feature `parts` = `dep:zip`) and by `self_update 1.3.0` (`archive-zip`, Windows releases).

cargo-deny flags yanked crates anywhere in the graph, so `security` most likely stays red until both move. This is unverified: `scripts/security-check.sh` was not run.

The options:
- release ooxmlsdk on zip 9 (it is a separate project);
- drop `archive-zip` from self_update (ship Windows as `.tar.gz`, or unpack the zip with our own zip 9);
- a `[patch]`;
- a cargo-deny exception with a reason.

The research prompt below asks for a survey of the alternatives first.

### Research prompt (for a research model with web access)

> You are researching Rust crates for jubarte-redlines (github.com/jandira-tech/jubarte-redlines), a Word (.docx) redline engine whose priority is byte-reproducible, Word-valid output. Report facts with links (crates.io, docs.rs, GitHub source at a pinned commit, RustSec advisories), the date of each fact, and say "unknown" when you cannot verify one.
>
> **Situation.** Every `zip` 8.x release is yanked, so cargo-deny fails our CI. We depended on `rdocx-opc` 0.2.0 (OPC package layer: parts, `[Content_Types].xml`, `.rels`, part-name to rels-path) on top of `zip` 8. No rdocx-opc release moves off zip 8: 0.16.1 depends on `oxml-opc` ^0.14.1, which depends on `zip` ^8.1. Our branch ported rdocx-opc 0.2.0 in-tree (`src/opc/`, MIT attribution) on `zip` 9.0.4. `zip` 8.6.0 still enters through two other dependencies: `ooxmlsdk` 0.13.1 (feature `parts`) and `self_update` 1.3.0 (feature `archive-zip`; its `zipsign-api` verifier also uses zip).
>
> **Requirements for any ZIP or OPC crate:**
> - deterministic output: our own entry order, fixed timestamps (1980-01-01, no wall clock), Deflate with a chosen level, no data descriptors, no ZIP64 unless needed, no extra fields;
> - safe reading: refuse `../` and absolute names, cap entry count and sizes, and expose raw name bytes so undecodable names can be refused;
> - streaming or in-memory reading and writing;
> - `wasm32-unknown-unknown` support (we ship a WASM build) and a Python binding (pyo3);
> - permissive licence (MIT/Apache/BSD/Zlib), compatible with AGPL-3.0-only;
> - maintained: a release within 12 months, no open RustSec advisory, and not yanked.
>
> **Questions.**
> 1. Which ZIP crates meet these requirements today? Consider at least `zip` 9.x, `rc-zip` (sans-IO), `async_zip`, `zip-rs` forks, `piz`, `zune-inflate`/`miniz_oxide`/`flate2`/`zlib-rs` as a hand-rolled writer, `rawzip`, and anything newer. For each, give the version, licence, last release, MSRV, wasm support, deterministic-write controls, raw-name access, and size/entry caps.
> 2. Which OPC/OOXML crates (package layer, not a full document model) exist? Consider `oxml-opc`, `opc`, `docx-rs`, `docx-rust`, `ooxml`, `office-opc`, `rdocx-*`, `ooxmlsdk`'s `parts`, and others. Do any already sit on zip 9 or another non-yanked ZIP? Is one a better long-term base than our in-tree port, judged on API, maintenance, tests and determinism?
> 3. Why was every zip 8.x yanked (the advisory or issue, with link)? Is the defect reachable through how an OPC reader or writer uses zip? Is a cargo-deny `[advisories]`/`[bans]` exception defensible, and what is its exact syntax?
> 4. `self_update`: is there a release, branch or fork that drops zip 8 or makes `archive-zip` use zip 9? Do alternatives exist (`axoupdater`, `cargo-dist`'s updater, `self-replace` plus our own download/verify), and do they support zipsign or minisign verification?
> 5. `ooxmlsdk`: what does its `parts` feature need from zip? Would a zip 9 bump be a small change? Point to the exact files.
> 6. Recommend one path: keep the in-tree port, adopt crate X, or write a minimal ZIP writer and reader on `miniz_oxide`/`zlib-rs`. Give the risks, the migration size, and what to test (byte-identical outputs on existing fixtures, refusal cases, wasm build).
>
> **Output:** a comparison table per question, then the recommendation, then the list of facts you could not verify.

## Open questions for Arthur

1. #517: is the `on` grammar for a change inside a change acceptable (`{--two--}{>>r3 @BO on r1 @AL<<}`)?
2. #532: a per-block source map for hand-written Markdown, yes or no?
3. #531: if Strict text boxes cannot round-trip without changing `pN`, which should win: one numbering (my recommendation), or the gate?

## Machine and tooling

- **Cargo:** runs through mbx (PR #533 updates AGENTS.md).
  - `mbx setup` is global. The cargo shim is `~/Library/Application Support/mbx/bin/cargo`, first on PATH from `~/.zshrc`.
  - Each worktree's `target` is a symbolic link into `~/Library/Caches/mbx/targets/v1/<hash>`. Free space with `mbx gc`/`mbx clean`, never `rm -rf target`.
  - kache is gone: `~/.cargo/config.toml` is empty, and its `~/.zshrc` lines were removed.
- **Lock:** `/tmp/cargo-seq` serialises cargo across worktrees (a mkdir lock at `/tmp/jubarte-cargo.lock`) and calls the mbx shim by its full path. Touching `/tmp/jubarte-cargo.pause` pauses new runs; `/tmp/resume-cargo.sh` resumes them.
- **Agents:**
  - Prompts are in `/tmp/rt-prompts/`, logs in `/tmp/ag-<name>.log`, `name:pid` pairs in `/tmp/ag-pids`. `/tmp/wait-any-agent.sh` exits when one finishes.
  - pi's `-p` writes its log only at the end.
  - Never use `git stash` in a worktree: stashes are shared by every worktree of the repository.
- **Providers:** no OpenRouter, no Claude agents.
  - MiniMax's plan is used up.
  - zai `glm-5.3` rate-limits (code 1302) at about 8 concurrent agents.
  - Devin (`subscriptions/devin/swe-2`) dropped two long runs and then refused one (`permission_denied`).
  - Grok has no balance; Cursor is exhausted.
- **Disk:** 63 GiB free at handoff.

## Pre-existing failures (not caused by this work)

- `md_fixed_point` was red on 23 fixtures at the base and at #515 alike.
- Lib test `convert::...::revised_script_delimiters_follow_the_script_and_empty_delimiters_emit_nothing` (`"axyZ"` vs `"a(x|y)Z"`) fails at da476c07.
- `convert::pdf::tests::a_comment_on_a_vertical_page_turns_with_its_text` failed on the zip branch; not checked on main.

## Next steps

1. Collect each running agent's result. Review each one adversarially with a different model. Commit, and prune scratch tests.
2. Resume 518fix when a zai slot is free.
3. Fix #534 (small; a prerequisite for an exact gate).
4. Run the zip/OPC research prompt above; decide how ooxmlsdk and self_update leave zip 8.
5. Rebase the stack on `test/md-fixed-point-only` with `gh stack` (order: 517, 518, 520, 521, 526, 528, 529, 530, 531, 527) and open stacked PRs into #515. zip stays its own PR into main.
6. Run the full gate and the combined nextest pass once; then fmt, clippy `-D warnings`, `jubarte --help`, `jubarte validate` and `tools/validate-docx` on the written files. Then /conejo-merge.
7. The #507 border edits uncommitted in `1_jubarte-redlines` belong to another session; leave them.
