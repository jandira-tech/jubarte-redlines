# Consolidated review: `docs/superpowers/plans/2026-10-09-agent-text-view.md`

> **Status (2026-10-18, corrected 2026-10-19 at `aba48bd5`).** Review
> evidence — the `body:rev:N`/`body:p:N`/`[body:p:N]` spellings cited below
> describe the transitional ids as they were on 2026-10-09. They are no
> longer printed: `inspect` moved to `[pN]`/`[header1.pN]` short ids in
> `6fc0ab0e`, and `changes --json` holds the short id in both `id` and
> `ref` (`9872a374`, `52e96ced`) — the colon family survives only as a
> fallback for values with no short spelling (e.g. `body:rev:007`).
> `inspect::SCHEMA_VERSION` is still 1; the 17 October contract v3
> (`docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md`,
> unimplemented) retires these spellings at the schema-2 cutover, which
> replaces `id`/`ref`/`at` with paired `short_id`/`long_id` references.

Reviewed 2026-10-09 against the working tree of this checkout. The plan was
split into six chunks and each chunk was reviewed by a separate agent whose
brief was to check everything upside down — every file path, line anchor,
type/function/field name, behavioral claim, and test expectation verified
against the real source and the goldens — and to report only what is
necessary. Findings below are deduplicated across chunks; line references
point at the current 5,486-line plan text (Tasks 1–20).

**Caveat first:** the plan and the four goldens were edited by another session
*while* this review ran (4,884 → 5,486 lines; Tasks 16–19 became 16–20 with a
new Task 18 "Markdown marks"; header/footer short ids changed from `h0`/`f1`
to part stems `header2`/`footer1`). Everything below was re-verified against
the final text; one mid-review blocker (story ids contradicting the goldens)
is already resolved by that edit — the plan and goldens now agree on part
stems.

## Overall assessment

The plan is real and deeply grounded: the overwhelming majority of its anchors
are exact — `MarkdownOptions` and its derived `Default`, `docx_to_markdown`,
`from_docx::Options`, every quoted struct literal, the whole `ooxml::Element`
API surface, `inspect::body_paragraph_nodes`' numbering rule, `critic::tidy`/
`render`'s adjacency semantics, `write.rs`'s reserved authorship position, the
comment-thread model (`commentsExtended`, `w15:paraIdParent`, `w15:done`), the
`convert::RenderReport::pages` → `markdown::paginate` wiring, edit's selector
grammar, and the fixture XML behind the goldens (tag algebra, handles,
timestamps, 35-column header padding) all check out. The architecture — the
existing converter behind one `ids` option, numbering stamped before
resolution so ids match `edit` — is sound and fits the repo's rules.

But it is **not executable as written**. Three blockers dead-end the
implementer under the plan's own Rule 2 ("never adjust an expected string"),
and Tasks 1, 13, 14, 18, 19 and 20 each contain a wrong claim or a missing
rewrite that breaks a build or a scheduled green gate. All are one-step fixes;
none require redesign.

**Verdict: feasible after the fixes below. The plan's own quality bar —
names exact, tests failing for the stated reason — is currently not met in
the listed places.**

## Blockers (implementation dead-ends)

1. **Task 3 — `Blocks::push_line`'s `tight` flag breaks its own test and the
   goldens** (plan ~924–962, 1240–1263). `push_line` sets `tight = true`
   unconditionally and nothing resets it when no paragraph body follows:
   `push_paragraph` (ooxml.rs:769–802) early-returns for a blank body and only
   clears `tight` when it runs. So `flush_empty`'s `<!-- pN-pM empty -->` lines
   and the next id line join with `\n`, and after a body-less id line
   (`<!-- p5 page-break -->`) the page marker gets separator `\n` instead of a
   blank line. The plan's own test (plan 877) and the golden
   (`received.tracked.md` ~65–69: `<!-- p19 page-break -->`, blank,
   `<!-- page 2 of 2 -->`) both require the blank line. In the layout path it
   also mis-places the marker *above* the held p19 line. Fix: only an id line
   immediately followed by its body sets `tight`; empty-run lines, body-less
   id lines and page markers leave `tight = false`.

2. **Task 8 — the reject-view double spaces are unreachable** (plan 2106–2119;
   `received.reject.md:39`). `Inline::render` ends in `tidy_inline`
   (ooxml.rs:551, 616–631), which collapses runs of spaces; no task modifies
   it. Rejecting `Deliver ` + ins + ` reports ` renders one space, not the
   asserted `Deliver  reports weekly`; `including  route-cost` likewise (the
   fixture XML confirms the double space is what rejection leaves). Fix: add
   an explicit step (Task 8 or earlier) preserving inter-word double spaces in
   the agent view, e.g. an agent-gated `tidy_inline` variant.

3. **Task 18 — `plain_anchor`'s escape ordering fails its own Step-1 test**
   (plan ~4547–4626 vs test at ~4435). `unescape_markdown` runs *after* the
   even-count `*`/`_` removal, so the escaped needle `{\*\*stars\*\*}` becomes
   `\stars\` and never matches `**stars** here` → `ANCHOR_NOT_FOUND`, and the
   test can never pass against the Step-3 code. Fix: resolve backslash
   escapes before the emphasis-mark removal (or skip backslash-preceded
   marks when pairing).

## Major (wrong claims / missing rewrites that break builds or gates)

4. **Task 1 — literal inventory incomplete; Step 5's "everything passes" is
   unreachable.** Two exhaustive `MarkdownOptions`/`Options` literals are
   missing from the update list: `src/bin/jubarte.rs:1610–1615` (the convert
   docx→md path; verified) and `src/markdown/from_docx/mod.rs:3072–3078`
   (inside `#[cfg(test)]`; breaks `cargo test --lib`). Both need
   `..Default::default()` appended. Conversely `jubarte-wasm/src/lib.rs:926–929`
   is a `DocxOptions` literal that already has `..default()` — drop it from
   the list.

5. **Task 2 — `nested_tables_are_not_numbered` cannot pass as written**
   (plan ~433–439). `Element::find_all` does not descend into matches
   (ooxml.rs:295–304), so the test's `body.find_all("tbl", …)` never sees the
   nested table and the `[Some("0"), None, Some("1")]` expectation is
   unreachable. The `stamp` code itself is correct; only the test's
   verification method is wrong. Fix: walk manually (like the `indices`
   helper) or reach the nested table via `path(&["tr","tc","tbl"])`.

6. **Task 13 — `page_texts` neither compiles nor matches the layout path**
   (plan ~3640/3670/3691). `jubarte::convert::Revisions` does not exist — the
   render parameter is `convert::RevisionStyle` (src/convert/mod.rs:216) and
   there is no `From<cli::TrackChanges> for RevisionStyle` behind
   `revisions.into()`. Worse, `paginated` (src/bin/jubarte.rs:1498–1539)
   resolves the bytes via accept/reject *before* rendering; `run_text` passes
   raw tracked bytes, so accept/reject views would paginate resolved markdown
   against the tracked document's page texts (the goldens survive only because
   the hard break at p19 forces the same 2 pages). Fix: `page_texts` takes the
   track-changes choice, resolves exactly as `paginated` does, then renders.

7. **Task 14 — misses `tests/markdown_cli_page.rs`** (file list at plan ~3710).
   `text_reads_tracked_changes_as_critic_markup_or_resolves_them`
   (tests/markdown_cli_page.rs:110–138) asserts `[body:p:`, and exact
   `text --track-changes accept|reject` outputs — all false once `text` always
   prints the agent view; `help_describes_the_new_flags` (144–152) asserts
   `text --help` contains `[body:p:N]`. Both break; neither is in the rewrite
   list. The bare-resolved-text expectations belong on `convert -t md`.

8. **Task 19 Step 5 — false claim about patch tests** (plan ~5294). "`adoption.rs`
   and `m_cli_agent.rs` read `patch.diff` from disk, not from stdout" is
   wrong: `edit_writes_and_prints_the_patch_of_the_redline_unless_quiet`
   (tests/m_cli_agent.rs:441–445) asserts `stdout.contains(&patch)`, and Task
   19 replaces the patch print with the agent view. `tests/m_cli_agent.rs` is
   not in the task's Files list. Fix: name the rewrite (assert the patch from
   `patch.diff`; stdout carries the view).

9. **Task 20 — bare `jubarte A B` stops writing `<A>_v_<B>.docx`**
   (plan ~5305–5478). `diff_and_compare_take_word_against_markdown`
   (tests/markdown_cli.rs:636–638) runs the bare shorthand and then reads
   `old_v_new.docx` from disk; the file appears in no Files list. Fix: add the
   test with the `-o`-adding change.

10. **Task 20 Step 3.4 — `jubarte FILE -o OUT` silently ignores `-o`**
    (plan ~5388). The new validation only rejects `-o` combined with a *read*
    option; `-o` with a single file and no read option parses, routes to
    `run_text`, and ignores it (today it is a usage error). Compare-only flags
    (`--author`, `--date`, `--mode`, …) with a single file are likewise
    silently ignored. Fix: error whenever `output` (or any compare-only flag)
    is present and MODIFIED is absent.

## Minor (necessary, by task)

**Plan-wide / rules**
- Gates under AGENTS.md: no coverage runs and no binary `--help` smoke test
  anywhere (clippy only at Tasks 15/19-ish); AGENTS.md requires both after
  source changes. Rule 2's `pages.rs` count is stale (816 lines, not 510; the
  other three counts are exact). New files as given lack SPDX headers
  (`agent.rs`, `header.rs`, `tests/agent_text_view.rs`,
  `tests/edit_short_ids.rs`, `tests/edit_anchor_normalization.rs`) — REUSE
  lint will flag them.

**Task 1** — "keep the existing doc comments" freezes three doc comments the
plan's own goldens contradict (`track_changes` "without comments",
`Revisions::Accept` "comments are left out", `docx_to_markdown`'s "author and
date" note): no task owns updating them.

**Task 3**
- The `paginate` hold is not byte-identical for legacy documents whose blocks
  already start with HTML-comment lines (a marker can now appear where none
  did). State the exception in rule 3 or gate the hold on agent markdown.
- Cached-break counting is inconsistent: `page_lines` advances only on
  `w:lastRenderedPageBreak` but the total falls back to `1 + hard`, so
  hard-break-only documents get `page 1 of M` and never a marker for 2..M.
- Numbered headings get list labels in agent mode (`## 2. Numbered heading`)
  but not in `convert -t md` — intended (Task 4 tests it) but an unstated
  divergence; say so in docs.
- Tracked empty paragraphs are flattened into `<!-- pN empty -->`, dropping
  `break-ins`/`break-del` tags and comment ids the agent would need for
  `accept --id` (cf. the repo's empty-spacer corpus). Carry the tags or record
  the gap as out of scope.
- Interleaved id lines turn tight lists loose (`\n\n` between items);
  received.docx has no lists and no test covers the shape — round-trip tasks
  will meet it. Leading empties precede the page-1 marker in the cached path,
  violating "page 1 first".

**Task 5** — `table_line` walks rows/cells with `children_named`, missing the
`sdt`/`sdtContent`/`customXml`/`ins` wrappers that `table_rows`/`row_cells`
(mod.rs:1157–1177) collect through (they exist because real fixtures needed
them): wrong `{rows}x{cols}`, short `cells` ranges, lost tags. Reuse the
converter's walkers.

**Task 6** — `legacy_output_keeps_author_notes` fails *today* as written
(`space_into_last_change` runs only for flattened cells, mod.rs:779–781; the
mechanism claim is wrong — the hedge recovers, but state the real expectation).
The footnote/endnote definitions loop calls the free `paragraph_mark`
(mod.rs:198), so a multi-paragraph footnote with a tracked mark renders a
legacy `{>>Author (date)<<}` note beside tagged notes in agent view.

**Task 7** — the inserted `comment_range` block runs before the existing
`!self.in_comment` filter (mod.rs:654–656), so a `commentRangeStart` inside a
comment body pollutes the enclosing paragraph's id line in hidden mode; add
`&& !self.in_comment`.

**Task 8** — Step 3's "likely causes" are stale (markers verifiably survive
`revise::resolve_in`; the comments filter already changed in Task 3), and
`join_paragraphs` (revise.rs:143–163) discards the joined-away paragraph's
element, losing its INDEX/REVS stamps — a paragraph after a resolved tracked
break loses its id line and rev tags. Rewrite Step 3 around the real gaps.

**Task 9** — the underline site list omits `critic::join_text`
(critic.rs:320–342) and `critic::wrap_markers`' format closure
(critic.rs:485–486): missing `join_text` silently merges an underlined text
with a following plain one (agent-only corruption no listed test catches);
missing `wrap_markers` splits `<u>` runs around change markers. Name both.

**Task 10** — `CommentFact.date` is written but never read (dead-code warning
under `clippy -D warnings`); `Owner::None` prints `# no docProps/core.xml`
even when the part exists and simply names nobody.

**Task 12** — the expected `body: p0-p9, 1 table, 1 page` string pads to 34
columns but `kv` pads to 35 (goldens confirm 35): a correct implementation
fails by one space, and Rule 2 points the implementer the wrong way.

**Task 19** — `jubarte::edit::SCHEMA_VERSION` does not exist;
`SCHEMA_VERSION` is `pub` in `crate::inspect` (src/inspect.rs:31) — use
`jubarte::inspect::SCHEMA_VERSION` or add a re-export.

**Task 20 Step 1** — two more bin unit tests break with
`Job.output: Option<PathBuf>`: `all_options_long_and_short`
(src/bin/jubarte.rs:2517) and `flags_can_supply_both_inputs_without_positionals`
(2544). Name them per rule 4.

**Cross-task semantics** — a comment range starting/ending exactly between a
deletion and an insertion inserts a Highlight span that blocks `render`'s
substitution pairing while `revision_tags` (which skips the markers) still
pairs them: grammar line 86's claim that rev clauses and notes always agree
overstates. Worth one sentence in the grammar.

## Per-task verdicts

| Task | Verdict |
|---|---|
| 1 Fixture, goldens, options plumbing | feasible with changes (findings 4, Task-1 minors) |
| 2 The stamp | feasible with changes (finding 5) |
| 3 Id lines, empties, page markers | feasible with changes (**blocker 1** + minors) |
| 4 Id line annotations | feasible as written |
| 5 Table lines | feasible with changes (walkers) |
| 6 Attribution notes | feasible as written (two expectation notes) |
| 7 Comments | feasible as written (one guard) |
| 8 Accept/reject views | **broken as written** (blocker 2, stale Step 3) |
| 9 Underline | feasible with changes (two omitted sites) |
| 10 Header part 1 | feasible as written (dead field, owner comment) |
| 11 Header part 2 | feasible as written against the current goldens |
| 12 Block selection | feasible with changes (one-space padding) |
| 13 CLI wiring | feasible with changes (finding 6) |
| 14 Old-layout tests | **broken as written** (finding 7) |
| 15 Goldens, round trips, docs | feasible as written (gates under AGENTS.md) |
| 16 Short ids in edit | feasible as written |
| 17 `--changed`/`--by` | feasible as written |
| 18 Markdown marks | **broken as written** (blocker 3) |
| 19 `edit`/`add` by flags | feasible with changes (findings 8, SCHEMA_VERSION) |
| 20 Implicit commands, docs | feasible with changes (findings 9, 10, bin tests) |
| Self-reviews (both) | mostly honest; they wave away findings 8–9 and miss blocker 3 |

## What verified clean (evidence the plan is grounded)

Every quoted anchor in Tasks 1–16 was checked and found exact, including:
`MarkdownOptions` at mod.rs:199–208 (derived `Default` today; no field
collisions; `Select`/`Pick` collide with nothing), `Options` at
from_docx/mod.rs:37, the literals at bin/jubarte.rs:752–757,
jubarte-python/lib.rs:678–682, jubarte-wasm/lib.rs:271–275; the full
`ooxml::Element` API (`parse_xml` :330, `attrs`/`children` :223, local-name
`attr`, `is/local/elements/child/children_named/find_all/path`);
`inspect::body_paragraph_nodes` (src/inspect.rs:593–598) matches the
numbering rule exactly; `tests/common/docx.rs` builders (`docx`, `docx_with`,
`para`, `Part`, `W_NS`); fixtures present and `received.docx` byte-identical
to `examples/cli/visualize/out/received.docx`; the goldens internally
consistent with the grammar and the docx XML end to end (tag algebra
`#0`/`#1+2`/`#3+4`/`#7+8`/`#9+10`, handle derivation AC/AC2/AS, timestamps,
empty-range comment c11, 35-column padding incl. the >33 two-space rule);
`critic::tidy`/`render` adjacency semantics match the stamp's grouping;
`write.rs` reserves the post-change authorship position (write.rs:855–930,
1180); the comment-thread model matches the fixture byte-for-byte; the
`RenderReport::pages` → `paginate` wiring is real (bin/jubarte.rs:1513–1539);
`revise::resolve` clones so stamps survive; edit's selector machinery
(`select` at src/edit.rs:3134, `Selector` :738, `Transaction` :1593, story
paragraphs, `body:rev:N` ids) supports every short id the view prints; clap
is locked at 4.6.7 as the plan states; Task 14's quoted assertions exist at
the quoted lines; Task 15's round-trip targets (`changes --json` id shapes,
edit plan CLI, docs anchors) all exist as described.

## Recommended order of fixes before handing the plan to an implementer

1. Fix blocker 1 (`tight`), blocker 2 (double spaces) and blocker 3 (escape
   ordering) — each is a small, local edit to the plan's code blocks.
2. Add the missing literal sites (finding 4) and the missing test rewrites
   (findings 7–9) to their tasks' Files lists.
3. Correct `page_texts` (finding 6) and the `-o` validation (finding 10).
4. Sweep the minors into their tasks — the Task 3 paginate/byte-identical
   exception and the tracked-empty-paragraph gap are the two most likely to
   bite an implementer who trusts the plan text.
5. Re-run the golden-derived expectations after any edit to the plan's code
   blocks (the goldens are the truth; three findings are exactly
   plan-code-vs-golden contradictions).

---

# Verification (2026-10-10): were the suggestions implemented correctly?

The plan has since been fully implemented (Tasks 1–20 merged; dozens of
follow-up `fix(read)`/`fix(cli)`/`fix(edit)` commits). Six fresh agents
verified every finding above against the merged code — status, correctness,
and whether any different approach taken was appropriate. The five test
suites covering the fixes were then run once at HEAD:
`agent_text_view` (99), `edit_anchor_normalization` (3), `m_cli_agent` (12),
`markdown_cli` (24), `markdown_cli_page` (4) — **142 passed, 0 failed**.

**Headline: all 3 blockers and all 7 majors are resolved. 24 of 28 findings
resolved appropriately — 16 as suggested, 8 via different but sound (often
better) mechanisms. 4 items remain open, all small.**

## Blockers and majors

| # | Finding | Status | Assessment |
|---|---|---|---|
| B1 | `tight` flag breaks blank lines | different mechanism | `push_line` now self-separates with `\n\n` and `tight` only joins the block *under* a line (ooxml.rs:797–844). Fixes all three symptoms by construction; goldens asserted byte-for-byte; ids-off never sets `tight`. Appropriate. |
| B2 | reject-view double spaces unreachable | as suggested | `keep_spaces` threaded through `Inline`/`Critic` (ooxml.rs:482–489, 569, 640–655), set only under `self.agent` (mod.rs:1402). Legacy branch byte-identical to the old `tidy_inline`; `including  route-cost` pinned by golden + test. |
| B3 | `plain_anchor` escape ordering | different mechanism | `anchor.rs` hides `\X` escapes behind private-use stand-ins *before* stripping, reveals after (anchor.rs:146–171). **More correct than the review's primary fix** — unescape-first would have produced `stars` and failed its own test; only the review's parenthetical alternative worked, and hide/reveal is a clean total version of it. One theoretical nit: genuine U+F0000-range chars in anchors would be remapped. |
| M4 | missing struct literals | as suggested + better | Both missed sites got `..Default::default()`; beyond that, four construction sites were replaced by one `markdown::read()` entry point (mod.rs:482) used by bin/python/wasm. Convert path keeps `ids: false` as intended. |
| M5 | nested-table test method | as suggested | Manual walker that recurses into matches, with a comment stating why (`find_all` stops at a match); expectation now reachable (agent.rs:1563–1582). |
| M6 | `page_texts` type + resolution | stronger than suggested | Promoted to `markdown::page_texts(docx, TrackChanges, RevisionStyle)` (mod.rs:559–585): resolves accept/reject *before* rendering, same order as `paginated`; both CLI paths share it. Accept/reject goldens with layout pages pass end-to-end; failed layout degrades to cached breaks with an honest warning. |
| M7 | missed `markdown_cli_page.rs` | as suggested | Both tests rewritten in place (commit 071ac380 explicitly added the file); bare resolved-text expectations now live on `convert -t md` (both directions); help assertion updated. No test deleted. |
| M8 | patch-stdout claim | as suggested | Test renamed to `edit_writes_the_patch_and_prints_the_changed_blocks_unless_quiet`; patch asserted from `patch.diff` only; stdout asserts the agent view and `!contains("\n@@ ")`. |
| M9 | bare `A B` stops writing the docx | as suggested + documented | `Job.output: Option<PathBuf>`; shorthand prints the view (header says `not written; -o keeps it`) and writes nothing; explicit `compare`/`diff` still write the default file. README:401–404, CHANGELOG, MARKDOWN.md document it; tests cover all three paths. |
| M10 | `-o` silently ignored | half implemented | `-o` with a single file now errors ("`-o` writes a redline of two documents…"), as does `-o` + read options — both tested. But compare-only flags (`--author`, `--date`, `--mode`, …) with a single file are *still silently ignored*. Defensible (they have defaults, low harm) but the residual footgun the review named remains. |

## Minors

| Finding | Status | Assessment |
|---|---|---|
| t3a paginate hold vs legacy | as suggested | Hold gated on the `<!-- pN`/`<!-- tN` shape (pages.rs:17–23), not a flag; legacy byte-identical, test added. Residual: literal document text shaped like an id line would still be held — theoretical, acknowledged in a comment. |
| t3b cached-break counting | different mechanism, sound | Single-basis rule: rendered markers are authoritative when present; otherwise hard/section breaks count and advance (mod.rs:146–158, 1317–1355). Tested for hard-only, section-break, and mixed documents. |
| t3c tracked empties flattened | as suggested | An empty paragraph holding a tracked mark, fmt change or comment keeps its own id line (splitting the run) with `break-ins`/`comments` clauses (mod.rs:1208–1222); tested; documented in MARKDOWN.md. |
| t3d id lines loosen tight lists | kept loose by design | Loose separation is structural: `-p`/`--head`/`--changed` select blocks by blank lines, so items must be separate blocks (agent.rs:1182–1245). Defensible — but the shape is undocumented and no test pins the inter-item separator. |
| t3e leading empties before page 1 | as suggested | `flush_empty` emits page lines first (mod.rs:1303–1313); exact-output tests added. |
| t3f heading-numbering divergence undocumented | **not implemented** | The behavior exists (`## 2. Numbered heading` in agent view only, tested) but no doc states it — MARKDOWN.md documents the analogous escape divergence, not this one. |
| t5g table walkers | as suggested | Table line reuses `table_rows`/`row_cells` (agent.rs:886–923); sdt-wrapped rows and tracked rows tested. |
| t6 legacy author-note test | as suggested | Expectation corrected to `a {++b++}{>>…<<}` (the real mechanism: `space_into_last_change` runs only for flattened cells); agent path now skips tagged changes there entirely. |
| t6 footnote `paragraph_mark` | different mechanism | New `Writer::note_mark` routes agent view to tagged marks, legacy to the free function (mod.rs:1792–1802); the exact scenario is tested. |
| t7 `in_comment` guard | as suggested | Guard present at both lookups (mod.rs:1108–1124). Gap: no test constructs a comment body containing a `commentRangeStart`. |
| t8 `join_paragraphs` stamp loss | different mechanism, better | REVS merged into the surviving paragraph at join time (revise.rs:149–158) instead of restamping — restamp would renumber and disagree with `edit`; both tested. |
| t9 underline sites | as suggested | `join_text` and `wrap_markers` both carry underline (critic.rs:429, 596–640); the join boundary is tested; the marker-inside-underlined-run half is implemented but unexercised. |
| t10 dead `CommentFact.date` / owner comment | as suggested | Field dropped; `Owner::Unnamed` distinguishes "part names nobody" from "no part"; both tested. |
| t12 `body:` padding | as suggested | 35-column rule implemented (header.rs:97–110); goldens compared byte-for-byte. (Commit bdbd959a's "padding" is unrelated — it trims `--by` values.) |
| pw gates (coverage/`--help`) | substance satisfied | Plan text not updated, but CI carries fmt + clippy `-D warnings` + a coverage job with branch floor, and `--help` smokes of the built binary exist as tests (m7_cli.rs, markdown_cli_page.rs). |
| pw stale `pages.rs` count | not fixed | Plan text only; irrelevant now the code is merged. |
| pw SPDX headers | as suggested | All new files carry the standard header; goldens covered by REUSE.toml's catch-all. |
| pw three stale doc comments | **not implemented** | `track_changes` ("without comments"), `Revisions::Accept` ("comments are left out") and `docx_to_markdown` ("author and date") are still stated unconditionally — accurate for `ids: false` only. MARKDOWN.md documents the real behavior, but the rustdoc is stale. |
| pw comment-range vs substitution note | different: documented | Still possible, now documented in MARKDOWN.md's "Limits of the agent view" (precisely scoped to inline-comments mode); a render-side fix is structurally blocked. The mirror-image bidi case got a *code* fix (b5d5410d). Appropriate. |
| pw wasm `DocxOptions` non-site | as suggested | Only the `MarkdownOptions` literal was touched; the `DocxOptions` site untouched. |

## Still open (all small)

*(Superseded by the reassessment below: at 16a6f045 all four remain open,
unchanged.)*

1. `jubarte FILE --author X` (any compare-only flag with a single file)
   silently ignores the flag — the other half of finding M10.
2. Document the heading-numbering divergence (t3f).
3. Update the three stale rustdoc comments (pw finding 4).
4. Pin the loose-list separator shape in a test, and — if cheap — add tests
   for the two untested guards: a `commentRangeStart` inside a comment body,
   and a change marker inside an underlined run.

---

# Reassessment (2026-10-10, second wave: the renames and the CodeRabbit/pi fixes)

Scope: 408a9a16..16a6f045 — 52 commits since the first verification: the
`revisions` → `changes` and `fields update` → `convert --update-fields`
breaking renames, fixes from CodeRabbit's review of #392 and the "pi" review,
closed-pipe and timestamp hardening, and 0.12.0's release evidence. Three
read-only agents re-verified the findings above against 16a6f045, plus direct
runs; the tree has since moved again (a follow-ups session is implementing
`2026-10-09-agent-edit-follow-ups.md`; its work-in-progress is not assessed
here). Line references are as of 16a6f045.

**Headline: none of the four open items was closed; nothing the first
verification passed regressed; but the `revisions` rename left the test suite
red and both renames left dangling documentation.**

## The four open items — all still open

1. **Compare-only flags with a single file: unchanged.** `validate_matches`
   still refuses only `output` (src/cli.rs:1588–1597); `jubarte received.docx
   --author "X Y"` prints the view and exits 0 (verified against the built
   binary). The other half of M10 remains.
2. **Heading-numbering divergence: documented nowhere still.** The behavior
   is pinned agent-side only (`id_line_shows_direct_alignment_indent_style_
   and_number`, tests/agent_text_view.rs:336–373 → `## 2. Numbered heading`);
   the plain side (`convert -t md` drops the label) is pinned by no test, and
   docs/MARKDOWN.md, README.md and CHANGELOG.md all stay silent.
3. **The three stale rustdoc comments, verbatim at HEAD:**
   `MarkdownOptions::track_changes` ("without comments", src/markdown/mod.rs
   ~204), `from_docx::Revisions::Accept` ("comments are left out",
   src/markdown/from_docx/mod.rs:32–33) and `docx_to_markdown`'s "each change
   followed by its author and date" (src/markdown/mod.rs:387–390). All three
   are wrong for `ids: true` (accept/reject views keep comments; tags are
   `#N @handle`).
4. **The three test gaps, still missing.** No test pins the loose `\n\n`
   separator between list items carrying id lines (the closest,
   `an_empty_numbered_paragraph_spends_its_number` at tests/agent_text_view.rs
   :1683–1714, matches fragments only). No test builds a `commentRangeStart`
   inside a comment body (the guard is intact at src/markdown/from_docx/mod.rs
   :1162 and :1186; every test comment body is marker-free). No test puts a
   change marker inside an underlined run (`wrap_markers` carries underline at
   src/markdown/from_docx/critic.rs:595–640; only bold and link are tested).
   Note: d7916043 ("test: close the review's test gaps on flags, ids and
   --by") closed a *different* review's gaps (pi review av3 F8 / av4 F14:
   `--existing-revisions reject`, `--force` with flag operations, cell/story
   `-p` ids, bindings `--changed --by`) — not these three.

## No regressions in the verified mechanisms

All ten re-checked at 16a6f045: B1 (`push_line`/`tight` — ooxml.rs untouched
except a `Rels` helper), B2 (`keep_spaces` still agent-gated), B3 (anchor
hide/reveal intact; the `_` pairing was reworked into full CommonMark pairing
with a rule-of-three — d93bfecc, 01afc453 — plus a differential test against
pulldown-cmark at src/markdown/anchor.rs:284–315: stronger than what the
review asked for), t3b (single-basis page count; 718e3a85 documented its two
limits), t3c/t3e, t7 (the `!self.in_comment` guards survived 711d2802's
hidden-range rework, which added `in #c9` clauses *with* tests), t9, t8
(revise.rs untouched), M6 (`page_texts` unchanged) and legacy byte-identity
(every markdown hunk in the range is agent-gated; the four goldens are
unchanged on disk). One note: d6343026 (a `w:lvlText` naming a deeper level
gets no list label) is the only ungated change reachable from the plain
conversion — a deliberate, documented Word-parity fix, but tested only
through the agent view; one legacy-side test would pin it.

## New blocker: the `revisions` rename left the suite red

9b6809ae (feat(cli)!: drop the revisions command) did not migrate
tests/m7_cli.rs. Running them confirms:

- **FAIL** `revisions_subcommand_json_escapes_quotes_in_author`
  (tests/m7_cli.rs:242) and `revisions_subcommand_lists_tracked_changes_in_
  both_output_modes` (tests/m7_cli.rs:276).
- `revisions_subcommand_invalid_docx_errors` (:226) and
  `revisions_subcommand_missing_file_errors` (:355) still pass — **by
  accident**: with the command gone, clap parses `revisions <file>` as the
  compare shorthand, which fails reading a file literally named `revisions`
  with exit 1 and "reading revisions: …" on stderr — exactly what both tests
  assert. They are green only because the failure modes coincide.

The untracked note docs/with-the-goal-of-delightful-fog.md:21 already names
these four tests. Fix: migrate them to `changes` (the listing is the same
surface; the `--json` escaping and row-shape assertions carry over) — as it
stands `cargo test` cannot go green, and every later commit inherits the red.

## The renames' documentation fallout

`revisions` → `changes`, user-visible dangling references:

- README.md:188–189 (and README.crates.md:189) — the shared command set still
  lists `revisions`.
- docs/python.md:30 — the same list.
- jubarte-python/README.md:58 ("`revisions` and `changes` list what a
  document tracks") and :87; README.fragment.md:76 ("same shape as the CLI
  `jubarte revisions --json`").
- docs/javascript.md:445 — `getRevisions` is described against
  "`jubarte revisions --json`", a command that no longer exists.
- examples/cli/visualize/VISUALIZE.md:266–279 — a full `jubarte revisions`
  walkthrough section (f54084bd fixed only the task table).
- examples/adoption/08-accept-reject/README.md:71 — "`jubarte revisions`
  reports **0 revisions**".

`fields update` → `convert --update-fields`:

- CHANGELOG.md:66 (Unreleased/Added) still says "`fields update` writes …" —
  the removed subcommand's name.
- **Neither breaking rename appears in the Unreleased section at all** (it
  has Added/Security/Changed/Fixed; no Removed/Breaking heading). Both
  renames postdate the 0.12.0 evidence (888b490b, this morning; the renames
  are this afternoon), so they ship in the next release — the entries must
  exist before it.

(Historical snapshots under docs/api/*.d.ts and dated docs that quote
`jubarte revisions --json` for *released* versions are accurate for those
versions and need no change.)

## The rest of the delta — verified sound

- **M9/M10 semantics hold**: `Job.output: Option<PathBuf>`
  (src/bin/jubarte.rs:1199–1213), the bare `A B` shorthand prints and writes
  nothing, explicit `compare`/`diff` write the default file, the `-o`
  single-file refusal stands (src/cli.rs:1588–1597). The shorthand docs now
  live at README.md:399–403 (5114b195 shifted the lines the first
  verification cited).
- **cd59739c**: the JSON adapter's bare `A B` now requires `read` support and
  is refused otherwise (exit 2). The native CLI is unaffected and both
  shipped adapters support `read`, so nothing the review documented changed.
- **e97848ee/16a6f045**: a `--report`/`--font-report` that names the input or
  output *by identity* (hard link, symlink, case variant) is refused even
  with `--force` (`same-file` crate; tests/cli_failure_paths.rs:225–266).
  Sound, and it does not touch the shorthand or `-o` paths.
- **`convert --update-fields`**: docx-only enforcement, Python parity, npm
  refuses before any read; the report-on-input/output refusal is tested
  natively and in Python (0705ea10, e2e04702).
- **Closed pipes**: `outln!`/`write_stdout` treat `BrokenPipe` as end of
  output and exit 0 (src/bin/jubarte.rs:33–49, :823), pinned across six
  commands (tests/m_cli_no_panic.rs:202–242).
- **Timestamps**: clap `value_parser = parse_datetime` — ISO 8601, offsets
  within ±14:00 (minutes counted), year ≥ 1 (3ec2aa3a, 470fddc4; tested in
  tests/cli_parser.rs:481–509).
- **Bindings**: a `changed_view` failure after a successful edit is a warning
  with exit 0 in all three CLIs (789957c0), pinned by injection tests in
  Python and npm (d6c25453); the Rust branch has no injection point and is
  untested (the commit says so).
- **6c875735** closed a real gap the first verification missed: `--dates`
  CLI wiring was untested because the golden has one timestamp per author;
  the new test uses two and asserts both modes.

## Test evidence at the reassessment

Lib 2486 passed / 0 failed and bin unittests 41/0 on the evening tree
(through 2c6cd986); `--test m7_cli` has the two failures above. The remaining
integration targets could not be completed: the follow-ups session began
editing the tree while the suite ran (`src/edit.rs` mid-edit breaks
compilation — `mod origin;` without its file), and AGENTS.md forbids racing a
second Cargo against the same target/. fmt, clippy and a fresh `--help` smoke
were skipped for the same reason. The grep evidence above plus the rename
commits' own updated tests make further integration fallout from the renames
unlikely, but unproven.

---

# Addendum (2026-10-19, `main` @ `aba48bd5`)

Re-check of the items this file last left open, against current code:

- **Compare-only flags with a single file (M10 residual): CLOSED.**
  `d163706a` (2026-10-12) made `validate_matches` refuse any `COMPARE_ONLY`
  option on the one-document path (`src/cli.rs:2555`–`2590`); `jubarte
  FILE --author X` is now a usage error, not a silent ignore.
- **The `revisions`-rename fallout: resolved.** The `m7_cli` tests now
  exercise `changes` (no `revisions_subcommand_*` tests remain), and the
  dangling `jubarte revisions` references in README.md, docs/python.md and
  docs/javascript.md are gone. `getRevisions` survives as the
  Docxodus-parity WASM function, unrelated to the retired subcommand.
- **Heading-numbering divergence: still open.** The agent view prints
  `## 2. Numbered heading`; `convert -t md` still drops the label, and no
  doc or test states the divergence.
- **The three stale rustdoc comments: still open, verbatim.**
  `MarkdownOptions::track_changes` still says "without comments"
  (`src/markdown/mod.rs:309`) and `Revisions::Accept`/`Reject` still say
  "comments are left out" (`src/markdown/from_docx/mod.rs:36-38`) —
  accurate only for `ids: false`.
- **The three test gaps: still open.** Nothing pins the loose `\n\n`
  separator between list items carrying id lines, no test builds a
  `commentRangeStart` inside a comment body, and no test puts a change
  marker inside an underlined run.

The review's quoted `body:rev:N`/`body:p:N`/`[body:p:N]` spellings and the
tag-algebra goldens (`#0`, `#1+2`) are transitional schema-1 forms; the 17
October contract v3 (unimplemented at `aba48bd5`) retires them at the
schema-2 cutover. The id claims above are updated where they asserted
current behavior; the review narrative itself is historical evidence of
the 2026-10-09 tree and is left as written.
