<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# The short and long ID contract, version 3: implementation plan (2026-10-16)

Implements `docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md` (the contract), with
no backward compatibility (Arthur, 2026-10-16): retired spellings and fields are refused or removed
outright, no migration tool is built, no deprecated-ids table is kept. The plan continues
`2026-10-13-ids-consolidation.md`, whose tasks 2 to 4 (one set of id types, every option through
them, grammar completion) are restated here against the contract's grammar.

> **Status (reconciled 2026-10-17, local `aba48bd5`).** Task 0 partially done:
> the contract and review note landed on `main` as `4612a155` (docs PR #570,
> merge `b5bdccbd`); the plan pointer is in the contract header. **Tasks 1–9
> have not started** — no `feat/ids-v3-grammar` branch, none of
> `tests/{id_grammar,id_long,inventory,id_resolution,md_anchored,md_writer_labels,presenters}.rs`
> exists, `SCHEMA_VERSION` is still 1 (`inspect.rs:32`). The rebase premise is
> stale: the "open stack" this plan expected to rebase (#509 … #557) has since
> *merged* — #515/#535–#540/#550 are in this checkout and #557 (`4f6a82c7`)
> landed on `origin/main` after it — so task 1 branches from current `main`,
> not `fix/md-fp-530-revisions`. Consequence: the `#557 disagreement` section
> below is now about *reverting landed code* (#557's unconditional reuse and
> the `named_numbers`/`asked_for` validity exception are in `origin/main`), and
> every `file:line` anchor in this plan needs re-surveying against post-#557
> `main`. The transitional behavior the survey describes (`noid`, `r43+43`,
> `.pN` cell paragraphs, colon longs in JSON, `Select::parse` open ranges and
> comma lists, `SCHEMA_VERSION 1` DTOs) is still what `main` ships.

## How it ships

- **Base.** The contract lands on `main` first, as a docs-only commit, with the review note moved
  beside it (`docs/superpowers/specs/2026-10-16-id-contract-review.md`). The open stack
  (#509 → #510 → #511 → #515 → #535 → … → #557) rebases on that `main`. The PRs below stack on
  the rebased tip, `fix/md-fp-530-revisions`, because that branch holds the newest view, reader and
  writer code and the only Markdown-exact fixtures for revisions. Nothing here branches from
  `feat/md-table-grid-default`, which is 121 commits behind the contract's own base (`d6fce6e7`).
- **Anchors.** Every `file:line` below is from the survey of `7c996819` (2026-10-16). The id files
  are byte-identical at `d6fce6e7`; the Markdown files are not (`agent.rs` is about three lines
  off, `write.rs` and `xml.rs` moved in #557). Re-anchor at the base before editing.
- **One PR per task, stacked, red test first.** At most five files per commit; commit and push
  often; rebase the stack when a base moves. Gates per PR: `cargo fmt --check`, `cargo clippy
  --all-targets --all-features -- -D warnings`, the touched tests under `cargo nextest run --test
  NAME`, `jubarte --help`, and `committed_views_match_the_view` whenever a view changes (regenerate
  the committed views in the same PR; `tests/fixtures/md-roundtrip/views.txt` says how).
- **Verdicts travel with the PR.** Each PR quotes `md_fixed_point`'s id verdicts (correct / wrong /
  lost / new) before and after, on the `ids` dimension. The whole-document exact test is red today
  for content reasons and is not this plan's gate; the id verdicts are.
- **Schema 2 is one release.** Tasks 1 to 9 land on the stack and ship together as the schema-2
  release; no intermediate state is published. The release needs a fresh six-file `release_info/`
  set from the bench (`AGENTS.md`), so the cutover PR is not the release.

## What the contract changes about the code

Facts from the survey that the tasks below rest on.

- **One `Step::Paragraph(usize)`** (`src/ids/short.rs:47-61`); no `ps`; `valid_path` forbids
  `cell.p0` (`short.rs:230`); block positions are `usize`, so `p18446744073709551616` refusals are
  host-width dependent; `u32::MAX` is accepted as a revision label (`ids.rs` tests near L599).
- **Three cell-ownership conventions.** `inspect/tables.rs:43-58` nearest cell;
  `edit/origin.rs:142-148` all descendants minus text boxes ("as in the view"); the view's table
  line prints a row's flat range `cells t0.row0 p1-p3` (`agent.rs:1862-1879`), nested paragraphs
  included. `Table.index` counts nested and boxed tables (`tables.rs:59`); `tN` counts top-level
  only (`origin.rs:109-113`).
- **Fallbacks the contract retires.** `ChangeId::short` → `story:rev:raw` (`ids.rs:146-162`),
  which `changes.rs:456-523` gives every change with a missing id, so they all collide and resolve
  `Ambiguous`; the view prints `noid` instead (`agent.rs:271`); paragraph zero of a story prints as
  the bare story (`ids.rs:428`); `r43+43` from `join_tags` (`agent.rs:237-258`); `#id` move names
  (`changes.rs:461`); `Header_1:p:0` for a nonconventional part.
- **Printed ids parsed back into raw `w:id`.** `edit/tracked.rs:241-251` (`revision_counts`),
  `changes.rs:650` (`fold_style_pairs`), `validate.rs:2016` (`story_part`), `inspect.rs:154-182`
  (`Snapshot::retain`). Label allocation breaks all four; each moves to an inventory lookup.
- **Two `read -p` grammars.** `markdown::Select::parse` (`markdown/mod.rs:446-557`) splits on `,`,
  accepts `p17-`/`-p3`, is body-only, exits 1; everything else exits 2.
- **Text boxes.** `inspect::text_box_names()` (`inspect.rs:716-718`) already matches `w:` and
  `wne:txbxContent`; the comparer's `revision_processor.rs:935, 962, 2132` matches `w:` only. The
  view prints a box's text as unnumbered blocks (`from_docx/mod.rs:2538-2542`); the writer has no
  `txbxContent` output.
- **The writer today.** `revision_id` (`xml.rs:449-472`) takes the view's number per serial, with a
  global `used_revisions` set and a fresh counter that wraps at `MOST_ID = 0x7FFF_FFFF`
  (`view_notes.rs:398`); #557 makes the reuse unconditional so a number the view names twice is
  written twice, and `tests/md_roundtrip_valid.rs` (`named_numbers`, `asked_for`) stops counting
  those `DUPLICATE_REVISION_ID` findings. `renumber_comments` (`write.rs:3883-3930`) keeps `cN`.
  Sections are written 1-based from the YAML keys (`xml.rs:1447-1451`); `header.rs:1195-1215`
  prints them `i+1`. Header stories write to `/word/{stem}.xml` (`view_stories.rs:1017`).
- **`convert` reads neither `source:` nor `sha256:`** (`write.rs:122`, `:3085-3234`); the only
  `STALE_SOURCE` is `edit.rs:1960`. There is no `--standalone` and no anchored path.
- **JSON today.** `Change{id, ref, …, paragraph, inside}` (`changes.rs:54-94`, `ref` is a serde
  rename), `CommentRecord{id: u32, ref, parent: Option<u32>, paragraph, last_paragraph}`
  (`comments.rs:43-79`), `Paragraph{index, id}` (`inspect.rs:48-73`), `Story{id, kind, part}`,
  `ContentControl{id, paragraph_ids}`, `Table{index}`/`TableCell{paragraph_ids}` with no ids of
  their own (`inspect/tables.rs:18-40`), JSONL `op` event with `at` (`edit.rs:1028-1030`),
  `EditOutcome{id (operation label), paragraph, comment_id}` (`edit.rs:864-899`),
  `Hunk.at` mixing `pN` and `line:N`. One `SCHEMA_VERSION = 1` (`inspect.rs:32`) plus a second
  literal in `capabilities.rs:354`; `UNSUPPORTED_SCHEMA` already refuses a mismatched plan
  (`edit.rs:1384-1393`).
- **Validation.** `PARA_ID_OUT_OF_RANGE` exists (`validate.rs:858-893`, a text scan that skips
  non-`u32` values); `DUPLICATE_REVISION_ID` covers six kinds per part (`validate.rs:767-837`);
  there is no duplicate check for `w14:paraId` outside the comment family and no range, missing or
  noncanonical check for revision or comment `w:id`.
- **Bindings.** Python `models.py` mirrors every field above (`Change` 195-219, `Comment` 262-288,
  `Paragraph` 370-381, `Snapshot` 461-477, `EditOutcome` 1265-1280, `EditReport` 1311-1334 without
  the `paragraph_ids` move mapping Rust emits at `edit.rs:972`), hardcodes `schema_version: 1`
  (`models.py:1202`), decodes with strict `Class(**row)` constructors, and `__main__.py:320-322`
  builds the `op` refusal row itself; so does the npm CLI (`jubarte-redlines.mjs:371-382`). The
  npm package under `jubarte-wasm/npm/` is a checked-in build, 215 commits stale; `read-view.test.mjs:134`
  still expects `body:p:N` and passes only against it; CI runs only `cli.test.mjs`.

## The one place the contract and PR 557 disagree

PR 557 found that Word itself repeats a revision `w:id` when one mark is split around a hyperlink,
and made the writer reproduce the duplicate and the validity test accept it, so that
`ids-spacing-threads.md` with its `r43+43` and `r55+55` is Markdown-exact. The contract (§3, §4,
decided 2026-10-16) says a printed id never repeats a number: the two carriers get two labels,
printed `r0+1`, and the writer writes two ids.

Census of 2026-10-16 (`/tmp/jub-contract/census/dup_ids.py`, every revision carrier in every
`word/*.xml` of 1195 corpus documents):

| Documents with a repeated revision id | Repeated groups | Shape |
| - | - | - |
| 1 source document, present under 3 names (`Redline_CiceroDo_v_plate(30`, `5331b981b7_…`) | 6 | one carrier inside `w:hyperlink`, the other its sibling |
| | 3 | two carriers in one paragraph with a run between them (a field-coded hyperlink) |
| | 2 | two carriers in one paragraph with a `w:hyperlink` between them |

Every repeat in the corpus is Word's hyperlink split, and there is exactly one such document. The
plan follows the contract (task 4 reverts #557's unconditional reuse and the validity exception,
and regenerates `ids-spacing-threads.md`); the alternative, a third proven group for the hyperlink
split, is in "Open with Arthur".

## Decisions taken from "no backward compatibility"

1. **Refused, not mapped.** `body:p:12`, `header1:rev:3`, `#12`, `#c5`, `t0.r1.c2`, nested `.pN`
   in a cell, bare `header1` for a paragraph, `noid`, `r43+43`, `p17-`, `-p3`, `p0,p1` inside one
   id, bare numbers in `--id`/`-p`, numeric `comment_id`, `{"index": N}` paragraph selectors. The
   one refusal is `INVALID_ID` with the grammar summary. `settled_spelling` (`short.rs:119-173`),
   `hint_where`, the `:rev:`/`:p:` branches of `parse_change_ids` and `parse_location`
   (`ids.rs:223-234, 317-334`) and `paragraph_place`'s bare-story arm (`ids.rs:411-412`) are
   deleted, not kept as hint producers. The bare-digit hint (`12` → "p12, r12 or c12?") stays:
   it shows the grammar, not a retired family.
2. **Wire fields go, they are not aliased.** `id` and `ref` on changes and comments, `id` on
   paragraphs, stories and controls, `at` in JSONL, `comment_id` in outcomes, numeric comment ids,
   `Hunk.at`'s two meanings. Schema 1 plans get `UNSUPPORTED_SCHEMA` (exists) and nothing else.
3. **No deprecated-ids table.** `docs/locations.md`'s "Deprecated ids still in use" section and
   `scripts/cli_locations.py`'s probes for `body:p:0`, `body:rev:0` and `header1`
   (`cli_locations.py:83, 88, 117, 128-131, 438`) are removed with the families they probe.
4. **Tests that pin retired shapes are rewritten, not kept beside new ones** (`ids_roundtrip.rs`
   1074-1189, `agent_text_view.rs` 123, 2250, 4914, 5614-5686, `md_reader_id_lines.rs:231`,
   `edit_short_ids.rs` descendant cases, `inspect_tables.rs:124-146` only where it pins
   `Table.index` counting nested tables, `m_cli_comments.rs:94`, the Python and npm tests in
   task 8).
5. **The 13 October spec is annotated, not rewritten**: a note at its top names the contract as
   superseding its §4.3 and §5.3; its §2 table stays as history.

## Tasks

### 0. Land the contract, rebase the stack, branch — partially done

- `main` ← the contract and the review note (`git mv 2026-10-16-id-contract-review.md
  docs/superpowers/specs/`), plus a one-line "Plan:" pointer in the contract's header to this file.
  [Done: `4612a155` (PR #570, merge `b5bdccbd`) landed contract v3, the review note and this plan.]
- Rebase #509 … #557 on `main`. Conflicts fall on `AGENTS.md`, `CHANGELOG.md` and nothing else,
  since the contract adds only docs. [Moot: the stack merged instead — #515, #535, #536, #537,
  #538, #539, #540, #550 are on `main`, and #557 (`4f6a82c7`) is on `origin/main`.]
- Branch `feat/ids-v3-grammar` from `fix/md-fp-530-revisions`; every later task branches from the
  previous task's branch. [Not done; with the stack merged, branch from `main` post-#557.]
- Done when: `git merge-base --is-ancestor main fix/md-fp-530-revisions` is true and
  `cargo nextest run --all-features --profile ci -E 'not test(markdown_and_docx_round_trip_exactly)'`
  is green at the tip. Record the `md_fixed_point` id verdicts at the tip as the plan's baseline.
  [Baseline still unrecorded — take it on post-#557 `main`.]

### 1. The short grammar (contract §2, §10 "Complete short syntax vectors"; stage Urgent 1)

Red: `tests/id_grammar.rs` loads `tests/fixtures/ids/contract-vectors.json` and requires every
`accept` string to parse and print back byte for byte, every `refuse` string to fail, and
`print(parse(s)) == s` for the canonical subset. `scripts/id_contract_vectors.py` extracts the
JSON blocks from the contract into that fixture and runs in `docs.yml` with `--check`, so the
fixture cannot drift from the spec.

- `src/ids/short.rs`: `Step::{StoryParagraph(u32), ScopedParagraph(u32), Table(u32), Row(u32),
  Cell(u32), TextBox(u32), ContentControl(u32)}`; `ShortId::Range{first, last}` with the end's kind
  checked (story `pN-pM`, scoped `psN-psM` in one scope, `rN-rM` never); `Section(u32)`, `Toc(u32)`;
  `Step::parse` reads `ps` before `p`; `valid_path` becomes the contract's state machine (`ps`
  only after a cell or a box, `p` only at story level, `cell.ps0` valid, nested `t` after a cell
  or a box, `cc` after a story paragraph or a cell paragraph, `box` after a story paragraph or a
  scoped paragraph); numbers are `u32` written `0 | [1-9][0-9]*`, with a checked conversion to a
  host index at the inventory boundary.
- The 19 literals and the EBNF of §2 are the only grammar: no comma, no open range, no
  whitespace, no `#`, no `:`.
- Spelling lists for `capabilities` (`ids.rs:54-76`, `capabilities.rs:343, 445-450`): `header1.p0`
  replaces `header1`, `t0.row1.cell2.ps1` replaces `t0.row1.cell2.p1`, `p14.box0.ps0`, `s0`, `toc0`
  and `cc0` are added; `IDS_NOTE` says a story's paragraph zero is `story.p0`.
- Not chosen: keeping `usize` positions with a `u32` check at print time (two widths, two sets of
  refusals); accepting `cell.pN` as an alias of `cell.psN` (two spellings for one entity, which §3
  forbids).
- Tests: `tests/id_grammar.rs` (the vectors); unit tests in `short.rs` for each `valid_path`
  transition and each `Display`; `tests/id_types.rs` keeps parsing every id in the committed views
  (it goes red here for `header1` and the cell ranges, and green in task 3).

### 2. The inventory and the allocator (contract §3, §4; stage Urgent 2)

Red: `tests/inventory.rs` over synthetic packages built with the existing test helpers, and over
`Strict01.docx`, `ac6cd8d92f.docx` and the `edges/` documents:

- the §4 allocation table (`12`, `43`, `43`, `007`, `1+2`, missing, empty, `4294967295` →
  `r12 r0 r1 r2 r3 r4 r5 r6`), the §10 `alias_cases` and `grouped_alias_cases`, and the boundary
  (`2147483647` kept, `2147483648` allocated);
- comment labels (`5`, `005`, `5`, missing, empty → `c0 … c4`) and the comment index keyed by
  label, never by raw string (`comments.rs:270, 343, 479-485`);
- own-cell paragraphs by nearest cell, a nested table's paragraphs under the nested table, box
  paragraphs under the box, both `txbxContent` namespaces (`Strict01.docx` holds `wne:`);
- story aliases per §4 "Header and footer aliases", with `Header_A.xml` → `header0` and
  `/word/header1.xml` → `header1`;
- determinism: the same package read twice, and re-zipped with its entries reversed, gives the
  same inventory; a `HashMap` iteration order cannot reach an id;
- filtering invariance: `read -p`, `--changed`, author filters and pagination select from the one
  inventory and never renumber; part collisions: two header parts whose friendly names collide
  keep distinct aliases (`header0`, `header1`) and the nonconventional one is marked for `pkg:`
  in task 7; singleton stories (`footnotes`, `endnotes`, `comments`, `styles`, `numbering`) exist
  once or not at all.

Fix:

- New `src/ids/inventory.rs`: `Inventory::read(&Package, IdentityProfile) -> Inventory`, built once
  per document and shared by `inspect`, the view, `edit` and `validate`. It holds, per story, the
  paragraph list (every `w:p` outside text boxes, `mc:Choice` only), top-level tables with rows,
  cells, own-cell paragraphs and nested tables, boxes with their own paragraph lists, sections
  (explicit `w:sectPr` plus the final one), TOCs, content controls, revisions (logical targets
  with their carriers, labels allocated per §4 within Word's `int`, proven groups only:
  AlternateContent twins and `pPrChange`/`rPrChange` pairs), and comment definitions with labels.
  Each entry carries its canonical `ShortId` and its raw source keys (`w14:paraId`, `w:id`,
  `w:sdtPr/w:id`) for task 7.
- The one text-box test is `inspect::text_box_names()`; the comparer's three sites in
  `revision_processor.rs` call it.
- `inspect/tables.rs` keeps nearest-cell ownership and gains `tN` numbering of top-level tables
  per story, nested tables under their cell; `edit/origin.rs:109-148` and `Ids::paragraph_id`
  (`origin.rs:48-88`) read the inventory instead of walking.
- `WORD_INT_MAX: u32 = 0x7FFF_FFFF` moves from `view_notes.rs:398` into `src/ids` and is the one
  ceiling for labels, the writer and the validator.
- Not chosen: a lazy per-command walk (three walks disagree today; the contract wants one); storing
  the inventory in the package (reading is not stamping, §4).
- Tests: `tests/inventory.rs`; `tests/inspect_tables.rs` updated for `tN`/nested numbering;
  `tests/edit_short_ids.rs:124-238` rewritten from the descendants convention to own-cell;
  `tests/m_cli_agent.rs:206-240` (Strict `wne:`) kept green.

### 3. Every presenter prints inventory ids (contract §1, §3; stage Urgent 3)

Red: `tests/id_types.rs` green on regenerated views; `tests/md_fixed_point.rs::ids_name_blocks_notes_and_yaml_entries`
green; a new `tests/presenters.rs` asserts the shapes below on `edges/` documents.

- The view (`from_docx/agent.rs`, `header.rs`, `from_docx/mod.rs`): story paragraph zero prints
  `header1.p0` (`ids.rs:428` and `from_docx/mod.rs:954` go through `ShortId`); the `in …` of a
  story mark (`agent.rs:1080`) likewise; revision tags print allocated labels, so a change made of
  two carriers prints `r0+1` and never `r43+43`, and `noid` and the `:rev:` fallback
  (`agent.rs:269-290`) are deleted; `format_tag` prints through `Display`; the table line's
  `cells` clause lists each cell's own paragraphs and a nested table gets its own
  `<!-- t0.row1.cell2.t0 … -->` line; YAML `sections:` is keyed `s0, s1, …` (`header.rs:1195-1215`).
- `changes.rs:456-523`: `id`/`ref` carry the label spelling (`r4`, `header1.r2`) until task 7
  renames them; `move_name` is never a generated `#id` (`changes.rs:461`); `fold_style_pairs`
  (`changes.rs:650`) groups through the inventory. `comments.rs:587` and `agent.rs:1980` print the
  comment label. `inspect.rs` `Paragraph.id`, `Story.id`, `ContentControl.id` print inventory ids;
  `Snapshot::retain` (`inspect.rs:154-182`) filters by `ShortId`, not by string.
- `edit/tracked.rs:241-274` (`revision_counts`, `first_free_id`), `edit.rs:3922` hints,
  `validate.rs:2016` (`story_part`): inventory lookups.
- `ChangeId` (`ids.rs:120-172`) is deleted; its users take `ShortId::Change` plus the inventory.
- Regenerate the 40 committed views; quote the id verdicts.
- Not chosen: printing `r0+1` only when the carriers are adjacent (the join rule is the existing
  `join_tags` one; only the numbers change).
- Tests: `tests/presenters.rs`; `tests/agent_text_view.rs` 4906-4915 (`noid`), 5614-5686
  (`Header_1:p:0`, `body:rev:1+2`) rewritten to the new shapes; `tests/ids_roundtrip.rs` 1074-1189
  rewritten; `tests/m_cli_comments.rs`, `tests/edit_stories.rs`, `tests/inspect_paragraphs.rs`
  updated.

### 4. The writer's rule (contract §7 "The writer's rule"; stage Urgent 4)

Red: `tests/md_writer_labels.rs`: for every view in `views.txt`, `read(convert(view))` names the
same labels as the view, in the same places; two carriers of one logical target share one
`w:id`; two different labels never share one; fresh ids are above the view's largest and below
`WORD_INT_MAX`; `cN` writes `w:id="N"`; `headerN` writes `/word/headerN.xml`; `s0` is the first
section. `tests/md_roundtrip_valid.rs` requires zero findings from `check_word_valid_package`,
`DUPLICATE_REVISION_ID` included.

- `xml.rs:449-472` `revision_id`: the id for label `N` on every carrier of that label's logical
  target is `N`; a second logical target can never name `N` (the view does not print repeats after
  task 3), so the unconditional reuse #557 added (`used_revisions.insert(id); return id;`) becomes
  "reuse within the same target, fresh otherwise", and the `LinkEnd` filter that keyed on a
  repeated id (`xml.rs`, #557 hunk at `Item::LinkEnd`) goes. `tests/md_roundtrip_valid.rs`
  `named_numbers`/`asked_for` are deleted.
- `xml.rs:1447-1451`, `with_section_change`: sections 0-based from `sN`.
- `view_notes.rs:363-394` `revision_tags`: `noid` and `.noid` are refused like any invalid id;
  the `MOST_ID` filter at :387 becomes a refusal too (a label above `WORD_INT_MAX` cannot come
  from the view).
- `ids-spacing-threads.md` regenerated: its five repeated groups become ten allocated labels
  joined per visible change; it must stay Markdown-exact, as #557 left it.
- Not chosen: writing Word's duplicate for a hyperlink split (see "Open with Arthur"); a per-part
  `used_revisions` for footnotes (`package.rs:949-1010`) stays as it is, since labels are per story.
- Tests: `tests/md_writer_labels.rs`; `tests/md_reader_id_lines.rs:231` and the `r2147483648`
  case rewritten as refusals; `md_fixed_point` id verdicts all `correct` on `docx.txt` and
  `views.txt` is this task's exit condition.

### 5. One resolver for every option (contract §3, §7 "Valid syntax does not promise edit support"; stage ASAP 1)

Red: `tests/id_resolution.rs`: for each (operation, id kind) cell of the capability table the
resolver returns the target or the typed refusal; `read -p` accepts repeated `-p` and a
comma-separated option value whose every piece parses with the one grammar; `-p p17-` and
`-p 12` exit 2 with `INVALID_ID`; `edit -p t0.row1.cell2` and `edit -p t0.row1.cell2.ps0` resolve
to the same paragraph; `edit -p p14.box0.ps0` is `UNSUPPORTED_TARGET` (not "unknown paragraph",
not `LOCATION_NOT_YET`); `accept --id r0+1` accepts both carriers.

- New `src/ids/resolve.rs`: `Resolver<'a>(&'a Inventory)` with `resolve(&ShortId) -> Result<Target,
  IdError>`; `Target` is an enum over the inventory's entries; `Capability::check(op, &Target)`
  returns `UNSUPPORTED_TARGET` for boxes, controls, sections, TOCs and nested cells where the
  operation cannot act. `IdError` codes: `INVALID_ID`, `ID_NOT_FOUND`, `UNSUPPORTED_TARGET`,
  `AMBIGUOUS_TARGET`, `STALE_SOURCE`, `ID_ALLOCATION_FAILED`, `UNSUPPORTED_STRUCTURE`, each with the
  text and JSON shape of §7 "Errors must obey the same output policy".
- `markdown::Select`/`Pick` and `Select::parse` (`markdown/mod.rs:359-557`, `SELECT_EXPECTED`)
  are deleted; `read -p` (`cli.rs:79-87`, `jubarte.rs:831-838`) takes `Vec<ShortId>` through a
  clap `value_parser` that splits the option value on commas and parses each piece; the
  `Story::Body` filter goes, so `read -p header1.p0` and `read -p t0.row1.cell2` select those
  blocks. Exit 2 for every invalid id, in the binary, Python (`jubarte-python/src/lib.rs:247, 886`)
  and WASM (`jubarte-wasm/src/lib.rs:361, 426`).
- `ids::parse_location` (`ids.rs:313-373`) and `Location` are replaced by `ShortId` plus the
  resolver; `edit/flags.rs:190-262`, `edit/origin.rs:48-88`, `edit/controls.rs:296`
  (`cell_control`'s hand parser) call it. `LOCATION_NOT_YET`, `TABLE_NEEDS_CELL`,
  `CELL_NEEDS_TABLE` become resolver refusals with the typed codes.
- `accept`/`reject --id` (`jubarte.rs:84-103, 359-395`) resolve labels to carriers through the
  inventory; `resolve_selected` (`edit.rs:1121-1191`) stops comparing id strings and re-listing.
- Not chosen: keeping `Select::parse` behind `read -p` with a translation layer (the contract
  calls it a second grammar and retires it).
- Tests: `tests/id_resolution.rs`; `tests/edit_short_ids.rs:284-358, 551+`,
  `tests/agent_text_view.rs:121-175, 2006-2012, 4632-4712` (comma and open-range selection)
  rewritten; `tests/cli_parser.rs` exit codes; `tests/m_selective_revisions.rs` for `r0+1`.

### 6. Anchored Markdown edits (contract §7 "Anchored Markdown edits"; stage ASAP 2)

Red: `tests/md_anchored.rs`: a committed view with one paragraph's text changed, saved beside its
source, converted with `convert X.md -o Y.docx`, prints `note: anchored on SOURCE.docx` and
produces a package whose only differences from the source, by `jubarte debug SOURCE Y`, are in
that paragraph; `w:rsid*`, `w14:paraId` of untouched paragraphs, crops and OLE payloads are byte
for byte; a view whose `sha256:` does not match the file beside it exits with `STALE_SOURCE`;
`--standalone` takes the existing path; a block with no id line is inserted, a source block whose
id line is absent is deleted, and the report lists both.

- New `src/markdown/anchored.rs`: read the YAML `source:`/`sha256:` (`header.rs:262-265` writes
  them; `write.rs:122` learns to read them), hash the file beside the Markdown, compare; resolve
  every id line through the pinned inventory (task 2) with the resolver (task 5); render the
  source's own view of each resolved entity and diff it against the Markdown's block; turn the
  differences into `edit` operations (`apply_plan`) on the source package; emit the edit report
  (§6) with paired references (task 7 fills `long_id`; until then the short half).
- First slice: body and story paragraphs' text, tracked changes and comments. A difference the
  slice does not handle (table structure, images, sections) is `UNSUPPORTED_STRUCTURE` naming the
  block, never a silent standalone rebuild.
- `cli.rs` `MarkdownArgs`: `--standalone`; `STALE_SOURCE` is one typed error shared with
  `edit.rs:1960`.
- Not chosen: anchoring only on an explicit flag (the 13 October settlement and the contract make
  it automatic on a matching digest); comparing XML instead of views (the view is what the agent
  edited).
- Tests: `tests/md_anchored.rs`; `tests/md_view_roundtrip.rs` comments that cite "automatic
  anchoring (spec §5.3)" point at the contract.

### 7. Long ids and the paired wire (contract §1, §5, §6, §10, §11; stage ASAP 3)

Red: `tests/id_long.rs`: the contract's 48 long refusals fail to parse; the 130 catalog pairs
parse and print back; the short-key vectors and the codec vectors hold; for every JSON and JSONL
output of the CLI on the `edges/` documents, every document reference is an object with exactly
`short_id` and `long_id`, both nonempty, `long_id` parses, and a `j:short` value equals the
`short_id` beside it; `read --json --long-id document.w:p.w14:paraId.XXXXXXXX` selects the same
paragraph as its `pN`; a `j:short` long id for a natively keyed target is `ID_NOT_FOUND` with the
real pair as candidate; the agent view and every text listing contain no long id; the snapshot,
report and JSONL `load` envelopes carry `schema_version`, `id_contract`, `identity_profile` and
`source_sha256`; a package re-zipped with its entries reversed gives the same pairs, native and
`j:short` alike (§6).

- New `src/ids/long.rs`: `LongId{part: Part, element: Element, key: Key}`; `Part::{Conventional
  (document, headerN, footerN, footnotes, endnotes, comments, styles, numbering), Pkg(uri)}` with
  the `pkg:` byte escaping; `Element` as the registered set of §5 plus `j:toc`; `Key::{ParaId(String),
  WId(String), SdtId(String), Short(ShortId)}`; the `-XX` VALUE codec; the parser splits on the
  first three dots; eligibility (lexical form, Word's ranges, uniqueness in the bucket) is a
  question to the inventory, which recorded the raw keys in task 2. `IdRef{short_id, long_id}` is
  the one serde struct for a reference.
- `SCHEMA_VERSION = 2` (`inspect.rs:32`; the second literal at `capabilities.rs:354` reads the
  const); the snapshot, report and JSONL `load` envelope carry `id_contract: "jubarte.ids/2"` and
  `identity_profile: "canonical-wml-v1"`.
- DTOs: `Change{short_id, long_id, kind, target, author, date, text, paragraph: IdRef, move_name,
  move_side, inside: Option<IdRef>}` (`id` and `ref` removed); `CommentRecord{short_id, long_id,
  parent: Option<IdRef>, paragraph: IdRef, last_paragraph: Option<IdRef>, …}` (`id: u32` removed);
  `Paragraph{index, short_id, long_id, …}`; `Story{short_id, long_id, kind, part}`;
  `ContentControl{short_id, long_id, paragraphs: Vec<IdRef>}`; `Table{short_id, long_id, rows:
  Vec<Row{short_id, long_id, cells: Vec<Cell{short_id, long_id, paragraphs: Vec<IdRef>, text}>}>}`
  (`index` and `paragraph_ids` removed); `EditOutcome{label, paragraph: Option<IdRef>, comment:
  Option<IdRef>, …}` and `Operation.label` (the operation label leaves the name `id` to
  identity); the JSONL `op` event carries `paragraph: IdRef`, never `at`; `ParagraphMove{from:
  IdRef, to: Option<IdRef>}`; `ResolvedRevisions` lists `IdRef`s; `Finding.location: IdRef`;
  `Hunk{paragraph: Option<IdRef>, line: Option<u32>}` instead of `at`.
- Inputs: a plan address (`paragraph`, `comment`, `change_ids`, `control`) is one string, short or
  long; a long id is recognised by the colon in its second field (`w:`/`j:`), a short id never has
  one. `--long-id VALUE` (repeatable) joins every id-taking option (`read -p`, `edit -p`, `add -p`,
  `accept`/`reject --id`) and resolves through the same resolver. `Selector::Index` is removed;
  `comment_id_de` (`ids.rs:299-310`) is removed with numeric comment ids.
- Not chosen: a `--long-id` output mode (the pair is always printed, §1); long ids in Markdown
  (§7); accepting a long id in the same string position as a short one on the CLI (the contract
  names a separate option; the plan JSON takes either because a field, unlike a shell word, is
  typed by its schema).
- Tests: `tests/id_long.rs`; every test that reads `id`, `ref`, `at` or `comment_id` from JSON
  (`tests/m_cli_agent.rs:110-117, 357-358, 845-900`, `tests/edit_plan.rs:832, 978`,
  `tests/edit_resolved_ids.rs`, `tests/audit_rules.rs:553`, `tests/validate_*`) rewritten to the
  pair; `tests/ids_roundtrip.rs:737-741`.

### 8. Bindings (contract §8; stage ASAP 4)

Red: `jubarte-python/tests/test_ids.py` and `jubarte-wasm/cli/test/ids.test.mjs`: the same `edges/`
documents give the same `short_id`/`long_id` pairs in Rust, Python and npm; schema 1 plans are
refused with `UNSUPPORTED_SCHEMA` in all three; a bad id exits 2 with one message in all three.

- Python `models.py`: `IdRef` dataclass; `Change`, `Comment` (ids are `IdRef`, `parent:
  IdRef | None`), `Paragraph`, `Story`, `ContentControl`, `Table`/`Row`/`Cell` with ids;
  `Snapshot` refuses `schema_version != 2`; `EditPlan.to_dict` writes 2 (`models.py:1202`);
  `Selector` drops `int`; comment builders take `"c5"` or an `IdRef`; `EditReport.paragraphs`
  carries the move mapping (`edit.rs:972`); the strict `Class(**row)` decoders (`models.py:551,
  561, 1354, 1371, 1403`) become explicit field maps so a new field is an error in one place.
  `__main__.py:320-322` prints refusals through `_native.report_jsonl`; `mcp_server.py`: one shape
  per tool (`docx_validate`, `docx_edit`), docstrings say `schema_version: 2` and the pair;
  `document.py:282, 578-579` and `README.fragment.md:113` stop saying `[pN]` and `line:N`.
- WASM/npm: `.d.ts` doc comments (`npm/node/jubarte_wasm.d.ts:191, 255, 281, 357-377`) describe the
  pair; `jubarte-redlines.mjs:371-382` `printRefusal` prints the engine's JSONL; `read-view.test.mjs:134`
  (`body:p:N`), `npm-smoke.mjs:45-66` (`/^r\d/`, `/\[p0\]/`, `schema_version: 1`) rewritten;
  `jubarte-wasm/npm/**` rebuilt from this commit with `build-npm.sh`, and `ci.yml` gains a job
  that builds the wasm and runs all three npm test files against it, so the npm tests stop passing
  against a stale build.
- Not chosen: a Python or JavaScript long-id constructor (§8: pass Rust's through).
- Tests: as above, plus `test_agent_api.py` 45-60, 84-88, 129-135, 283-298, 391-404, 474,
  `test_edit_models.py`, `test_comments.py`, `test_cli.py:441`, `test_mcp_server.py:180-186, 322-327`
  rewritten to the pair and `header1.p0`.

### 9. Gates, docs and the schema-2 cutover (contract §9 "Compatibility rule", §10; stage ASAP 5)

- Gates added to CI: `tests/id_grammar.rs`, `id_long.rs`, `id_resolution.rs`, `inventory.rs`,
  `md_writer_labels.rs`, `md_anchored.rs`, the binding parity tests, `scripts/id_contract_vectors.py
  --check` in `docs.yml`; `md_fixed_point` id verdicts all `correct` stays a required check in
  `md-roundtrip.yml` even while the exact test is red for content.
- Docs regenerated and gated by `docs.yml`: `docs/locations.md` (probe lists and the deprecated
  table removed in `scripts/cli_locations.py`), `docs/flags_datetime.md`, the `-p` help quoted in
  `docs/{rust,python,javascript}.md`, README blocks. Hand-written: `docs/MARKDOWN.md:224-228,
  446-481, 607-609` (the tag grammar, "the ids are `w:id` values" becomes "the ids are inventory
  labels; the writer writes them as `w:id`"), `docs/DIALECT.md:47-70`, `skills/jubarte-documents/SKILL.md`,
  `docs/adoption/{mcp,plans,anthropic-docx-skill,openai-doc-skill}.md`, `GEMINI.md`, the three
  README fragments, `docs/WORD_DIFFERENCES.md` §9 (the writer's rule beside Word's one counter),
  `scripts/id_census.py` spellings, `src/inspect/tables.rs:35` doc comment. The 13 October spec
  gets its superseded note. `docs/2026-10-09-agent-edit-follow-ups.md:181-247` (`--pids`) gets a
  "not adopted, see the contract" line.
- `CHANGELOG.md` `[Unreleased]`: one `### Breaking (schema 2)` section listing every removed field
  and spelling by name.
- Delete: `settled_spelling`, `ChangeId`, `Location`, `parse_location`, `Select`, the `:rev:` and
  `:p:` code paths, `comment_id_de`, `Selector::Index`, the `named_numbers` validity exception,
  and every fixture expectation the tasks above rewrote. `adoption.yml` fails on an `examples/`
  diff instead of printing a notice, and `examples/**` are regenerated.
- Version: `0.13.0` (pre-1.0 minor, as `docs/VERSIONING.md` marks breaking changes today), the
  bench's six `release_info/` files for it, and the API-drift snapshot (`scripts/api_snapshot.py`).

### 10. Later: text boxes as targets (contract §9 "Future")

Not in this plan's release. Recorded so the refusals of task 5 have a destination: the view prints
a box's paragraphs under `<!-- p14.box0.ps0 … -->` lines, the writer writes `w:txbxContent` (and
`wne:` where the source had it), `Strict01.docx` and `ac6cd8d92f.docx` become Markdown-exact, and
only then do box ids leave `UNSUPPORTED_TARGET`.

## Order, dependencies, parallelism

```
0 → 1 → 2 → 3 → 4 → 5 → 7 → 8 → 9
                    └→ 6 ─┘        (6 needs 2 and 5; 7 fills its long half)
```

Tasks 1 and 2 can be built by two people at once (the grammar has no inventory dependency; the
inventory needs only `ShortId`'s constructors). Task 6 can start as soon as 5 lands and run
beside 7. Everything else is serial because each task rewrites the tests of the one before.

## Decided (Arthur, 2026-10-16: "1a 2a 3a 4b")

1. **The hyperlink split: A.** The contract stands. Two carriers of one mark that share a `w:id`
   get two labels, print `r0+1`, and the writer writes two ids; `DUPLICATE_REVISION_ID` stays
   strict. Task 4 reverts #557's unconditional reuse and the validity exception. Not chosen: a
   third proven group for the split (one document of evidence) with or without a validator
   exemption.
2. **Version: A.** `0.13.0` with a `### Breaking (schema 2)` changelog section. Not chosen: `1.0.0`.
3. **Operation labels: A.** `Operation.id` and `EditOutcome.id` become `label`, so `id` never
   means two things in one JSON; saved plans break, which no backward compatibility allows.
4. **Validator ranges: B.** `validate` gains its `w:id` range check and its duplicate-`paraId`
   check in a separate task after the schema-2 release (task 11), not inside task 2. Until then
   identity alone reflects an out-of-range or duplicated key, through `j:short`.

### 11. Later: validator checks for the keys identity refuses

After the release. `validate` reports a revision or comment `w:id` outside Word's `int`
(`-2147483648..=2147483647`), a missing or noncanonical one, and a `w14:paraId` duplicated within a
part outside the comment family (`PARA_ID_OUT_OF_RANGE` exists at `validate.rs:858-893`;
`DUPLICATE_REVISION_ID` at `:767-837` covers six kinds per part). The inventory of task 2 already
computes every one of these facts, so the checks read it rather than rescanning text. Each check
follows AGENTS.md: a broken probe in `tests/m_validity_ring1.rs`, and Word-fatal only when Word
refused the file.
