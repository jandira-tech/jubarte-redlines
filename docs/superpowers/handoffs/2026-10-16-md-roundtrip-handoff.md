# Markdown round-trip: implementation handoff — resolved, kept for the record

jubarte-redlines, 2026-10-16, about 16:40. This continues the handoff behind PR #515 (`fix/md-roundtrip-fidelity`, draft, into `feat/md-story-format`). The goal: make docx -> Markdown -> docx exact, and make the `md_fixed_point` gate green.

> **Status (audited 2026-10-18; local checkout `aba48bd5`, `origin/main` `4f6a82c7`).** Almost the
> whole stack landed. Merged to `main`: #515 (`0460e47a`), #535 drawings (`7c407dea`), #536 strict
> (`75de15c4`), #537 nested (`cc1baf35`), #538 escaping (`2b68338e`), #539 defaults (`4dc724cc`),
> #540 closers (`662ba929`), #550 highlights (`3a81ce55`), and — after the local checkout —
> #557 revisions metadata (`4f6a82c7`, current `origin/main` HEAD). Still open: PR #553
> (`fix/md-521-522-diff`, the `MAX_PAIRINGS` removal). Never PR'd: `fix/md-fp-526-frames`,
> `fix/zip9-migration`, and #532/#534. **The gate is still red** (CI run 39005997156 on
> `aba48bd5`; the run for `4f6a82c7` was queued at audit time): 14 fixtures fail md-side, 5
> documents docx-side — see "Pre-existing failures" below for the current list. The text-box and
> ID questions this handoff leaves open were settled by the short/long ID contract v3
> (`docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md`, `4612a155` / `b5bdccbd` /
> PR #570), which **no branch implements yet** — `noid`, `r43+43`, `.pN` cell paragraphs and the
> colon long ids all still print at `4f6a82c7`. Each row below carries its resolution.

**Tracking issue:** #516. Sub-issues #517-#532 and #534. @coderabbitai posted plans on each.

**Base of the stack:** `test/md-fixed-point-only` (da476c07, on #515's cab8ab73). It adds `MD_FIXED_POINT_ONLY=<substr>` to `tests/md_fixed_point.rs`, so the gate replays only the fixtures whose path contains that text. Every branch below starts from it, except zip.

**Pushed with this handoff:**
- every branch that has commits;
- every agent's uncommitted worktree as `wip/<issue>` (a snapshot commit on the branch head, tracked and untracked files);
- this file, on `test/md-fixed-point-only`.

~~No PRs are open for the stack yet.~~ **Superseded — see the status note: every stack PR is merged
except #553 (open) and the two branches that never got PRs (#526 frames, #524 zip).**

## Arthur's decisions (2026-10-16)

1. Nested change: the Word reader splits the insertion around a change it holds; the parser stays one level deep. **→ shipped in #537; the printed form is the `on` grammar (`{>>r3 @BO on r1 @AL<<}`).**
2. `--commonmark` always escapes CriticMarkup and Markdown syntax. **→ shipped in #538.**
3. No bare spec-CriticMarkup output. **→ shipped in #538.**
4. Literal delimiters in code: the easiest and cleanest rule. Implemented as the fence rule (fences read CriticMarkup) plus an escape that is unwound inside fences. **→ shipped in #538.**
5. zip: full migration. **→ never landed: `fix/zip9-migration` had no PR; `Cargo.lock` still holds `zip 8.6.0` at `4f6a82c7`. See the zip section below.**
6. `MAX_PAIRINGS = 400`: remove it. The positional fallback was the bug. **→ still pending: `MAX_PAIRINGS` is 400 in `src/markdown/diff.rs:30` and 2,500 in `src/markdown/patch.rs:40` at `4f6a82c7`; the fix is open PR #553.**

## State by issue

| Issue | Branch | State at handoff | Resolution |
| --- | --- | --- | --- |
| #517 nested change | `fix/md-517-nested` 0d3bf81f | `Node::Split` + `unnest()`. Gap: an insertion whose only content is a deletion lost the outer insertion. Agent 517c (zai) is implementing `{--two--}{>>r3 @BO on r1 @AL<<}` (the `on` grammar is still to be confirmed by Arthur). WIP: `wip/517`. | **Merged as PR #537** (`cc1baf35`). The `on` grammar shipped — nested notes print `{>>r3 @BO on r1 @AL<<}` (`critic.rs`). |
| #518/#519 escaping | `fix/md-518-519-escape` e51092ce | Escape in code blocks, unwound in fences; `--commonmark` always escapes. Regression: a hand-written `\\` in a fence collapses. The fix agent (518fix) stopped on zai's rate limit; WIP in `wip/518fix`. Resume prompt: `/tmp/rt-prompts/full-518fix-resume.md`, still queued. | **Merged as PR #538** (`2b68338e`). |
| #520/#523/#525 closers | `fix/md-520-523-closers` bdeb5eef | Nested spans opaque to closers; `Token::CommentBreak` (U+2E2E) for blank lines in comments; malformed-markup warnings. Reviewer r520b (zai) is fixing a stand-in leak inside HTML blocks. WIP: `wip/520`. | **Merged as PR #540** (`662ba929`). |
| #521/#522/#523 diff | `fix/md-521-522-diff` 300987df | Atoms for code spans, image targets and math; pairing without `MAX_PAIRINGS`. The Devin review found 7 defects. Fix agent 521fix (zai) is working on them: no positional fallback, sparse inverted-index pairing, Myers reflow, `critic::Literal` reuse, an accept/reject property test. WIP: `wip/521` (scratch tests `markdown_diff_*.rs` included; prune before a PR). | **Open as PR #553** — `MAX_PAIRINGS` is still in the tree (`diff.rs:30` = 400, `patch.rs:40` = 2,500 at `4f6a82c7`). |
| #524 zip 9 | `fix/zip9-migration` 9ad624a4 (on main) | zip 9.0.4; rdocx-opc 0.2.0 ported in-tree to `src/opc/`. The handoff's premise was wrong: no rdocx-opc release drops zip 8. Review posted on #524. Output is byte-identical to main, but the commit claims a refusal of undecodable names that is dead code (zip 9's `name()` never fails) and the planned test is missing. Not run yet: wasm-pack, python binding, `scripts/security-check.sh`. | **Never PR'd, unmerged.** `Cargo.lock` still pins `zip 8.6.0` at `4f6a82c7`; the in-tree `src/opc/` port lives only on the branch. |
| #526 frames | `fix/md-fp-526-frames` 1d89c93d | `src/markdown/frame.rs`; `frame: "(hAnchor=...)"` in story `pN:` maps. The gate passes for ids-header-footer and received. Done; needs a review. | **Never PR'd, unmerged.** There is no `src/markdown/frame.rs` and no `frame:` clause in story maps at `4f6a82c7`. `ids-header-footer` and `received` fail the gate anyway — on docx-side `debug --check render` differences, not Markdown ids. |
| #527 drawings | `fix/md-fp-527-drawings` (no commit yet) | Agent 527 (Devin) is running. New `src/markdown/from_docx/drawing.rs` (about 900 lines); a header drawing now round-trips; demo.docx exact. WIP: `wip/527`. | **Merged as PR #535** (`7c407dea`) — VML, DrawingML and `mc:AlternateContent` survive as hidden payloads. |
| #528 defaults and style names | `fix/md-fp-528-defaults` c3f774f7 | 7 commits: titlePg off, section start type, stated default style, empty paragraph style, heading outline level, LineNumber, renamed note style. Agent 528 (zai) is still running. WIP: `wip/528`. | **Merged as PR #539** (`4dc724cc`). |
| #529 highlights | `fix/md-fp-529-highlights` f6bfe596 | All 16 colours and their extents; zero highlight verdicts across the gate; the colour-erasing rewrite in the gate was removed. Reviewer r529 (zai) is attacking run coalescing in `push()`. | **Merged as PR #550** (`3a81ce55`). |
| #530 revision metadata | `fix/md-fp-530-revisions` (no commit yet) | ids-spacing-threads, 68526de79b and ids-revisions pass the gate. Fixed: del/ins kind kept through `rev`/`unplaced`, repeated `w:id` that Word writes, row promotion, space-only insertions, endnote and style `rPrChange`, footer `fmt` placement, table and story style names, `titlePg`, pad-cell alignment. Two Devin runs died; agent 530c (zai) is finishing it with corrections: drop the header-picture exclusion it added to the gate, leave `pages 11->13` on ids-tables (direct `w:spacing` the view cannot express), prove that the regenerated fixtures match. WIP: `wip/530`, older snapshot in `refs/backup/530-wip-1` (local). | **Merged as PR #557** (`4f6a82c7`, current `origin/main`). It regenerated fixtures (`ids-revisions` heavily) but changed no ID semantics: `r43+43`, `noid` and `.pN` cell paragraphs still print. Its gate run was still queued at audit time. |
| #531 Strict | `fix/md-fp-531-strict` (no commit yet) | Strict01: 158 correct / 0 wrong / 0 lost / 0 new (was 65/81/6/7). The remaining 17 docx lines are drawings (#527) and page count. **Rejected:** it numbered text-box paragraphs into `pN`, so the view's numbering would differ from edit plans' `body:p:N`. Agent 531b (zai) must restore one numbering, or stop and explain. It also adds Strict measures to `normalize_universal_measures` in the comparer; that is kept only with a unit test, as its own commit. WIP: `wip/531`. | **Merged as PR #536** (`75de15c4`). Two staleness notes: `edit -p` and friends take the short `pN` grammar since `c3b906ed`/`6fc0ab0e` and *refuse* `body:p:N` — though `body:p:N` still prints in JSON output (`inspect --tables`' `paragraph_ids`, edit-plan move mappings) as the transitional long form v3 §7 retires; and the resolution kept box paragraphs **unnumbered** (the stamping walk skips `w:txbxContent`; they print as unnamed blocks), so Strict01 reads 157 correct / 0 lost / 0 new md-side today with 4 docx-side differences left. |
| #527/#531 text boxes | plan | Arthur's id form: `pN.tbK.boxfieldM`. `pN` is the host paragraph (`body:p:N`, text boxes excluded); `tbK` is the K-th text box anchored in it; `boxfieldM` is the M-th paragraph inside the box; all count from 0. The box text prints visibly inside a block that marks the box. This makes Strict01 and `ac6cd8d92f` exact past p47 without changing body numbering. | **Superseded by contract v3 §2:** the box form is `p14.box0.ps0` (and `cell.boxN.psM` in cells), staged in v3 §7 — today it parses but resolves `UNSUPPORTED_TARGET`, and box paragraphs print unnamed. `pN.tbK.boxfieldM` and the `body:p:N` host form were never implemented and are not the v3 grammar. |
| #527 drawings format | `fix/md-fp-527-drawings` 54621454, agent 527b running | Arthur's rule: Mermaid for what Mermaid can express (charts, connector flowcharts, SmartArt), base64 for rasters, a `svg` fence for SVG. The original DrawingML is a hidden payload gated by a hash of the printed Mermaid. Payloads sit in reference definitions at the end of the view. Spec: #527 comment 6225903547. | **Merged in #535** (`7c407dea`). |
| #532 Pandoc manuscripts | none | Needs Arthur: I recommend a per-block source map, so hand-written Markdown comes back as written. | **Still open** — no branch. |
| #534 md -> docx nondeterminism | none | New, pre-existing on main: the `row_marks` HashMap (`write.rs:340`) picks the range-end timestamp in random order. Fix with a `BTreeMap` plus a repeat-conversion test. Not started. | **Still unfixed** — `row_marks` is a `HashMap` at `write.rs:561` on `4f6a82c7`. |

## zip 8 is still in the tree after the migration — still true on `main`

**Status: the migration never merged.** `fix/zip9-migration` had no PR; `main` at `4f6a82c7`
still has `zip 8.6.0` in `Cargo.lock` and no in-tree `src/opc/`. The two entry points below
(`ooxmlsdk`'s `parts`, `self_update`'s `archive-zip`) are unresolved as audited.

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

## Open questions for Arthur — outcomes

1. #517: is the `on` grammar for a change inside a change acceptable (`{--two--}{>>r3 @BO on r1 @AL<<}`)? **Answered yes in effect — shipped in #537 and prints that way today.**
2. #532: a per-block source map for hand-written Markdown, yes or no? **Still open; no branch.**
3. #531: if Strict text boxes cannot round-trip without changing `pN`, which should win: one numbering (my recommendation), or the gate? **Resolved a third way: box paragraphs print *unnamed* under the host (the stamping walk excludes `w:txbxContent`), the drawing itself survives as a hidden payload (#535), and Strict01 is a Markdown fixed point today. Canonical box ids are deferred — v3 §2/§7 (`p14.box0.ps0`, `UNSUPPORTED_TARGET` until the Future stage) — so the numbering question moves to the v3 text-box stage.**

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

## Failures at handoff vs now

- `md_fixed_point` was red on 23 fixtures at the base and at #515 alike. **Now (CI run
  39005997156, `aba48bd5`): still red, on a different and smaller set — md → docx → md fails on 14
  fixtures (`docx-libraries-comparison`, `dolphins`, `eLife.00047`, `git_manuscript`, `ids-fonts`,
  `ids-header-footer`, `ids-paragraphs`, `ids-revisions`, `ids-sections`, `ids-spacing-threads`,
  `ids-table-changes`, `ids-tables`, `list_of_figures`, `received`), and docx → md → docx → md fails
  on 5 documents (`received.docx`, `Strict01.docx`, `ac6cd8d92f.docx`,
  `docx_lots_of_comments_addition_removal.docx`, `68526de79b_…_redline_7cc19c9376.docx`). Most
  md-side failures are docx-side `debug --check render` differences with the Markdown itself
  id-exact; `ids-revisions`, `ids-spacing-threads`, `ids-table-changes` and `ids-tables` still have
  wrong/lost ids. The #557 merge's gate run was still queued at audit time.**
- Lib test `convert::...::revised_script_delimiters_follow_the_script_and_empty_delimiters_emit_nothing` (`"axyZ"` vs `"a(x|y)Z"`) fails at da476c07. **#571 (`142e47fa`) got main's lib tests compiling again (LineFacts outline); this specific delimiter test was not re-verified in this audit.**
- `convert::pdf::tests::a_comment_on_a_vertical_page_turns_with_its_text` failed on the zip branch; not checked on main. **Fixed — #566 (`21d03f41`) made the test opt into comment notes.**

## Next steps — what actually happened

1. Collect each running agent's result. Review each one adversarially with a different model. Commit, and prune scratch tests. **Done — the merges above.**
2. Resume 518fix when a zai slot is free. **Moot — #538 merged.**
3. Fix #534 (small; a prerequisite for an exact gate). **Still open — `row_marks` is still a `HashMap`.**
4. Run the zip/OPC research prompt above; decide how ooxmlsdk and self_update leave zip 8. **Not done — zip 8.6.0 still in `Cargo.lock`.**
5. Rebase the stack on `test/md-fixed-point-only` with `gh stack` (order: 517, 518, 520, 521, 526, 528, 529, 530, 531, 527) and open stacked PRs into #515. zip stays its own PR into main. **Happened differently — the branches merged to `main` as PRs #535–#540, #550 and #557, not stacked into #515; #521 is open as #553, #526 and zip never got PRs.**
6. Run the full gate and the combined nextest pass once; then fmt, clippy `-D warnings`, `jubarte --help`, `jubarte validate` and `tools/validate-docx` on the written files. Then /conejo-merge. **The gate runs in CI on every merge — and stays red by design until the remaining fixtures and the v3 cutover land.**
7. The #507 border edits uncommitted in `1_jubarte-redlines` belong to another session; leave them.
