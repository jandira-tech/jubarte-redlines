<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Review: "Jubarte short and long ID contract" v2 (17 October 2026) against three goals — historical, superseded by v3

> **Status (audited 2026-10-18, `main` @ `aba48bd5`).** This reviewed contract **version 2**. The
> current contract is **version 3** —
> [2026-10-17-short-and-long-id-contract.md](2026-10-17-short-and-long-id-contract.md), added in
> `4612a155` and merged by `b5bdccbd` (PR #570) — and its "Changes in version 3" plus the Decisions
> section below record how every item here was resolved (`j:key` → `j:short`, `cell.psN` kept as an
> alias, stages reordered, anchored mode moved into §7, `read -p` commas split at the option level,
> `md_fixed_point` and `committed_views_match_the_view` named as cutover gates). Neither version is
> implemented: `main` still runs the 13 October transitional grammar, so every "current code" row in
> the verification table — `noid`, `story:rev:` fallback, bare `header1`, `r43+43`, `.pN` cell
> paragraphs, open ranges, `usize` steps — still describes `main` today, except where a note says
> otherwise. No engine source was changed by this review.

Reviewed on branch `feat/md-table-grid-default` at 7c996819, 2026-10-16. The contract's
baseline `d6fce6e7` (`fix/md-roundtrip-fidelity`) is not an ancestor of this branch (121
commits apart), but every piece of current code the contract describes reads the same here, so
the findings hold for both.

The three goals, as given:

1. give the agent a specific place to modify, robust and reflecting the real document;
2. create or modify the docx precisely from that address;
3. docx → md → docx → md gives two identical Markdown files.

## Verdict

The contract is adequate as an **addressing** contract (goal 1) and as the **resolution** half
of goal 2. It is **not sufficient for goal 3 on its own, and it does not state the one rule goal 3
depends on**: that the standalone writer turns every `rN`/`cN` label back into `w:id="N"` and
every story alias back into its conventional part URI. The code does this today
(`src/markdown/xml.rs:449-470`, `src/markdown/write.rs:3879-3925`), so the fixed point holds by
implementation, not by contract. Add the rule and goal 3 is covered as far as IDs go; the rest
of goal 3 is content fidelity, which the contract rightly leaves to the 13 October spec.
**→ Resolved in v3: the writer's rule is now contract text (v3 §7 "The writer's rule", §9 Urgent 4,
§10 "Writer labels" gate) — Decision 1A below. The `xml.rs`/`write.rs` behaviour it pins is still
what `main` does.**

Three design decisions in the contract I would change, and two orderings:

- `psN` means two different things: a canonical identity in text boxes and a non-canonical
  selector in cells. One token, two meanings. **→ v3: kept `cell.psN` as an alias that normalises to
  the story-wide `pN` (Decision 2, discussed).**
- The synthetic long key is a SHA-256 of a positional path plus the file digest: 64 hex
  characters that encode the same information as the path itself, and that change on a re-zip.
  **→ v3: withdrawn entirely — Word's own number is the key, else `PART.ELEMENT.j:short.SHORT`
  (Decision 3).**
- The text-box grammar (`p14.box0.ps0`) has no producer and no consumer: the view prints box
  paragraphs without ids, `edit` refuses them, and the writer has no text-box output at all.
  **→ v3 §7/§9: parse but resolve `UNSUPPORTED_TARGET` until the Future stage (Decision 5A). Still
  true on `main`.**
- The long-ID subsystem (codec, `pkg:` URIs, `j:key`) is staged "Urgent 1" while no command
  accepts a long ID as input. None of the three goals consumes it. **→ v3 §9: reordered — the long
  codec and paired JSON moved to ASAP 3, after short syntax, inventory, presenters and the writer
  rule (Decision 4A).**
- Anchored md → docx (13 October §5.3), the real "modify precisely" path, is assumed in §7 but
  neither implemented nor staged. **→ v3 §7 + §9 ASAP 2: moved into the contract and staged
  (Decision 7B). Still unimplemented.**

Everything else I checked holds up: the hash framing recomputes exactly, the grammar is
internally consistent, the alias allocator is a fixed point after one pass, and every "current
code does X" claim I could verify is true. A live probe of 17 documents through
docx → md → docx → md with the binary built from this branch gave 12 fixed points; of the 5
that differ, 4 differ for content reasons (text boxes flattened, an image and two heading
styles lost, one deletion lost) and 1 for an id reason that the contract's alias rule would
remove (two `w:ins` carriers sharing one `w:id`, item 6).

## What I verified

| Claim or property | Result | Where |
|-|-|-|
| Both §10 synthetic-key vectors recompute from the stated framing | match (`/tmp/jub-contract/vec.py`) | §6, §10 |
| `Step::Paragraph(usize)`, one variant for story and nested paragraphs | true | `src/ids/short.rs:48-62` |
| `.p0` after a cell refused; cell stands for its first paragraph | true | `src/ids/short.rs:230` |
| `ChangeId::short()` falls back to `story:rev:wid` | true | `src/ids.rs:141-160` |
| Missing `w:id` prints `noid` | true | `src/markdown/from_docx/agent.rs:264-272` |
| `part_story()` uses the basename only | true | `src/ids.rs:382-396` |
| Paragraph zero of a story prints as the story (`header1`) | true | `src/ids.rs:425-440`; 13 views say `in footerN` |
| Inspect assigns a paragraph to its nearest cell; edit's cell scan includes nested-table descendants | true, they disagree | `src/inspect/tables.rs:50-55` vs `src/edit/origin.rs:142-148` |
| `read -p` has a second grammar (`p0,p1`, `p17-`, `-p3`) | true | `src/markdown/mod.rs:446-556` |
| Writer writes `rN` as `w:id="N"` while unused, else a fresh id above the view's | true | `src/markdown/xml.rs:449-470` |
| Writer keeps `cN` as `w:id="N"`, lowest free otherwise | true | `src/markdown/write.rs:3879-3925` |
| Text boxes: view prints box paragraphs as unnamed blocks under the host; edit refuses; writer has no `txbxContent` output | true | `tests/ids_roundtrip.rs:792-800`, `src/edit/origin.rs:157-159`, grep of `write.rs` |
| Shared-id substitution pairs (`r43+43`) occur in real views | 2 in the committed views (`43+43`, `55+55`) | `tests/fixtures/md-roundtrip/*.md` |
| Legacy ids (`noid`, `:rev:`, `:p:`) in committed views | none | same |
| `sections:` YAML numbers from 2 and omits the first | true, already noted as drift on 13 Oct | `tests/fixtures/md-roundtrip/ids-sections.md` |
| `md_fixed_point` gate state | red at the baseline per the 2026-10-16 handoff; not rerun here **→ still red on `main` @ `aba48bd5` (CI run 39005997156): 14 fixtures fail md-side, 5 documents docx-side** | `Markdown Round-Trip Handoff.md` |
| Live round trip, 17 documents (`/tmp/jub-contract/rt.sh`) | 12 fixed points: the 6 edge docs probed, `received`, `all`, `potpourri`, `file_170`, `quadratic`, `e4ccea1362_mcdoc`. 5 differ: `Strict01` (+1 paragraph from a `wne:txbxContent` text box, every later `pN` shifts), `ac6cd8d92f` (text box, `style_changes`, note `fmt` lost), `f8d8b26ec9` (image and two heading styles), `docx_lots_of_comments…` (one deletion), `5331b981b7_…cicerodo` (shared `w:id`, below) | `/tmp/jub-contract/rt/*/md{1,2}.md` |
| Shared `w:id` does not round-trip today | `w:id="43"` on two `w:ins` and `"55"` on two `w:del`; md1 prints `r43+43`, `r55+55`; the writer gives the second carrier a fresh id; md2 prints `r43` plus `rev r66` on the id line | `tests/corpus/…/5331b981b7_redline_cicerodo_v_plate_30.docx` |
| `read -p` today | accepts `p0,p1` and `p1-`; refuses `header1`, `t0.row0.cell0`, `.ps1`, `body:p:3` (hint `p3`) | probe on `edge-cells.docx` |
| `edit -p` today | `header1` and `header1.p0` both resolve to a paragraph; `t0.row0.cell0` resolves to `p1` (the first paragraph); `.p0` after a cell refused; `.ps1` refused as unknown; `p14.box0.p0`, `cc0`, `s0`, `p3-p5` are "settled, not editable yet"; `body:p:3` refused | same |
| `accept --id` today | `body:rev:1` refused with hint `r1`; `body:rev:007` still parses (the transitional long fallback); `007` and `noid` refused | same |

## Goal 1: a specific, robust place that reflects reality

Met, with one caveat and one flaw.

What the contract gets right: one inventory built before any filter or view; fixed traversal;
AlternateContent decided once; `header1` is the story and `header1.p0` its paragraph; a cell and
its first paragraph are different entities; one shared own-cell inventory replaces the
inspect/edit disagreement; no fallback strings anywhere; typed errors; every printed id parses.

Caveat, by design: short ids are positional and snapshot-bound. An agent that edits twice must
re-read or apply the shift note (`source p5-p40 are now output p6-p41`). The 13 October
settlement chose this (alternative A) over a key prefix or a hybrid guard, and the contract keeps
the sha pin and `STALE_SOURCE` as the safety net. That is a sound choice for one edit against one
pinned snapshot; it is the agent's job to re-read between edits. The contract should say this in
one sentence instead of implying stability through long ids, which never appear in the agent's
text.

Flaw: `ps` has two meanings (item 2 below). In a text box, `p14.box0.ps0` is the canonical
entity id. In a cell, `t0.row1.cell2.ps1` is a selector that normalises to `p23`, and the view
never prints it. An agent reading the grammar learns one rule for the token and meets another in
the output.

## Goal 2: create or modify the docx precisely

Met for resolution; the modification path it assumes is not here.

The resolver design is right: one parser for the whole grammar, a separate capability check per
operation, explicit coercions (story → paragraph zero, cell → first paragraph) that fail rather
than skip, and paired references in every structured outcome.

Two gaps:

- **Anchored mode is assumed, not staged.** §7 says "an anchored Markdown edit resolves those
  short IDs through the inventory of the pinned DOCX". No anchored conversion exists on this
  branch (grep of `src/markdown/` finds none), and the contract's stage table never schedules it.
  Standalone `convert` rebuilds everything from the Markdown, so whatever the dialect cannot say
  is lost, which is exactly what goal 2 forbids. Item 7.
- **Long ids are output-only.** `--long-id` is "a future input option". Until a command accepts a
  long id, the pair in JSON is information, not an address an agent can act on. Item 4.

## Goal 3: docx → md → docx → md identical

The contract's rules are compatible with a fixed point, but it leaves the decisive rule
unstated.

Why the alias scheme is a fixed point after one pass, using the contract's own vector: source
`w:id`s `[12, 43, 43, 007, "1+2", missing, ""]` print `r12 r0 r1 r2 r3 r4 r5`. The writer writes
`w:id` `12 0 1 2 3 4 5` (`xml.rs:449`). The second read finds seven unique canonical values and
preserves them all: `r12 r0 r1 r2 r3 r4 r5`. Identical. The same holds for comments and for
story aliases as long as `headerN` is written to `/word/headerN.xml`. The live probe confirms it:
12 of 17 documents are fixed points today, and none of the 5 differences is a label the writer
renumbered.

The contract also repairs the one id-caused difference the probe found. In the cicerodo
document two `w:ins` carriers share `w:id="43"`; today's view joins them as `r43+43`, the writer
can give only one of them `43` and invents `66` for the other, and md2 prints `r43` on the text
plus `rev r66` on the id line. Under §4 both carriers get distinct allocated labels on the first
read, the writer writes those two numbers, and md2 reads them back unchanged. A good rule,
reached for the right reason.

What breaks it: nothing in the contract, but the contract never requires the writer to behave
this way. §7 says a standalone conversion "can regenerate XML keys". If a future writer numbered
revisions `0..n` in document order, `r12` in md1 would become `r0` in md2 and the gate would go
red for every document with a tracked change. Item 1 closes this.

What the contract cannot fix and should name: the fixed-point gate is red today for content
reasons (nested changes, code fences, escaping, text boxes flattened into body paragraphs).
The schema-2 cutover will rewrite every committed view (`header1` → `header1.p0`, `sections:`
keys, every `.pN` under a cell in `read -p` docs). The contract should list
`tests/md_fixed_point.rs` and `committed_views_match_the_view` as cutover gates in §10 so the
id change lands together with the view regeneration, not before it. **→ Done in v3: both are named
cutover gates (v3 §9: "every id verdict of every listed document is `correct` before schema 2 is
released"; §10 "Writer labels"). The gate is still red on `main` — the content failures named here
are exactly what remains.**

## Changes proposed, three alternatives each

My pick is A in every item unless stated.

### 1. State the writer's rule for labels and aliases (goal 3)

Problem: the contract fixes how ids are read and never how they are written back from Markdown.
Goal 3 depends on it.

- **A. Add to §7 "Round trips":** the standalone writer writes `w:id="N"` for every `rN` and
  `cN` label the Markdown names, in document order, taking a fresh id above the Markdown's
  largest only when a label is already used; `headerN`/`footerN` write to `/word/headerN.xml`
  and `/word/footerN.xml`; `sN` is section order; the writer never renumbers a label. This is
  what `xml.rs:449` and `write.rs:3879` already do, so the cost is one paragraph and one test
  (`parse(read(write(md))) == parse(md)` on ids). Trade-off: none; it only pins existing
  behaviour.
- **B. Carry the raw source key in the note when it differs from the label**
  (`{>>r2=007 @AC<<}`), so md → docx can restore `w:id="007"`. Trade-off: a third identity
  representation in agent text, which §1 forbids, and no gain for goal 3 since `r2` → `2` is
  already stable.
- **C. Redefine the fixed point as md2 == md3** (one normalising pass allowed) and leave the
  writer free. Trade-off: weakens goal 3 as stated and hides regressions where the first pass
  changes something it should not.

### 2. One meaning for `ps`

Problem: `psN` is a canonical id in boxes and a non-canonical selector in cells (§3, "Settled
meanings"). The view prints cells as `cells t0.row0 p3-p11` and never prints `cell.psN`.

- **A. Drop `psN` for cells; keep it for boxes only.** A cell's paragraphs are `pN` (story-wide),
  which is what the view prints and what `edit -p p23` already resolves. The grammar loses
  `cell_tail → scoped_para`, the own-cell inventory is still needed for `inspect tables` and the
  cell → first-paragraph coercion, and `ps` has one meaning: "paragraph in a box inventory".
  Trade-off: an agent cannot say "second paragraph of this cell" without reading the table's
  id line first; the id line already lists each row's `pN-pM`.
- **B. Make `psN` canonical in cells too:** cell paragraphs leave the story inventory and
  `pN` means "paragraph outside tables". Trade-off: renumbers every body paragraph after the
  first table in every document; the contract itself defers this as "a separate proposal with
  renumbering and migration analysis". Right design, wrong moment.
- **C. Keep both meanings as written** and add one sentence to §2: "`ps` is an index into a
  scope; in a cell the scope is a view onto the story inventory and the entity keeps its `pN`;
  in a box the scope is the entity's own inventory." Trade-off: the inconsistency stays, but is
  documented where the agent reads the grammar.

### 3. The synthetic long key: a hash of a path is still a path

Problem: §6 hashes `(profile, file sha256, part URI, element, child-ordinal path, role,
counter)`. Within a snapshot the path alone is unique, so the 64 hex digits add no identity;
they obscure a positional address and change on any re-zip. The snapshot binding the hash
provides is already carried by `source_sha256` in the envelope and enforced by `STALE_SOURCE`.
§5 forbids `j:path` explicitly.

- **A. `j:path` in clear:** `document.w:p.j:path.0.82.0` (child ordinals from the part root,
  the same locator §6 already defines). Same uniqueness, debuggable, short, no hash provider,
  no collision counter. Snapshot safety stays in the envelope, exactly as it does for `p12`.
  Trade-off: a positional key looks like one, so the docs must say it is snapshot-scoped; the
  contract already says that of `j:key`.
- **B. Content-addressed key:** SHA-256 of the element's canonical XML bytes plus the long id
  of its nearest native-keyed ancestor. Survives re-zip and unrelated edits elsewhere; changes
  when the element itself changes, which is what "reflects reality" wants. Trade-off: two
  identical empty paragraphs under one keyed parent collide and need the counter; a repeated
  Word save that rewrites `w:rsid` changes the key unless rsids are stripped from the canonical
  form.
- **C. Keep as written.** Trade-off: a key that moves when nothing moved, and 64 characters an
  agent cannot read, in exchange for a uniform `j:key.HEX64` shape.

### 4. Stage long ids after the goals that have a consumer

Problem: "Urgent 1" includes the long codec, `pkg:` encoding and `j:key`, while "a future input
option named `--long-id`" is the first thing that would read one. Goals 1 to 3 run entirely on
short ids plus the sha pin. Arthur settled on 13 October that long ids exist and have this
shape; this item questions only the order, not the decision.

- **A. Reorder the stages:** Urgent = short grammar split, inventory and allocator, writer rule
  (item 1), fixed-point gate in §10; ASAP = long codec and paired JSON together with the first
  consumer (`--long-id` input, or anchored mode's internal use). Trade-off: schema 2 ships in
  two steps; the JSON pair arrives with the command that reads it.
- **B. Ship native keys only in schema 2** (`w14:paraId`, `w:id`) and `j:path` (item 3A) for
  the rest; defer `pkg:` encoding until a nonconventional part appears in a fixture. Trade-off:
  a smaller long grammar now, one more versioned extension later.
- **C. Keep as written.** Trade-off: the largest piece of work in the contract lands first and
  nothing in the agent's flow changes when it does.

### 5. Text boxes: reserve the grammar, do not advertise it

Problem: `p14.box0.ps0`, `cell.box0.ps0` and the box tables are in the catalog and the 166
accepted vectors, but today the view prints box paragraphs as plain blocks under the host with
no id, `edit` refuses them, and the writer cannot write a `w:txbxContent`. For goal 3 this
means any document with a text box prints its box text as body paragraphs on the second pass,
shifting every later `pN`. The probe shows it twice: `Strict01` goes from `p0-p147` to
`p0-p148` because a cover-page abstract in a `wne:txbxContent` box becomes `p48`, and
`ac6cd8d92f` loses its `unplaced:` box changes and gains a paragraph. Note the namespace: the
contract and the code say `w:txbxContent`; Word also writes `wne:txbxContent`, and the
inventory's text-box test must cover both.

- **A. Parse box forms, resolve them to `UNSUPPORTED_TARGET`, and add a stage** "box
  paragraphs get id lines in the view; the writer writes `w:txbxContent`; the fixed-point
  fixtures gain a boxed document" before any box id counts as supported. Trade-off: honest
  about coverage; the grammar does not move.
- **B. Cut box forms from schema 2;** boxes stay content until a writer exists, and a later
  extension adds them. Trade-off: 20 vectors and 15 catalog rows go; the contract reopens once.
- **C. Keep as written.** Trade-off: accepted syntax that no view prints and no writer honours,
  which §7 itself warns against ("valid syntax does not promise edit support" is true, but
  here no output produces the syntax either).

### 6. Shared `w:id` carriers: the contract is right, and the view's `r43+43` must go

I drafted this item expecting to argue for grouping Word's shared-id pairs as one change. The
corpus says otherwise. The one real instance (`5331b981b7_…cicerodo`, the source of
`ids-spacing-threads.md`) is two `w:ins` run carriers on one number and two `w:del` carriers on
another, not a del + ins substitution; today's view prints them as `r43+43` and `r55+55`, and
that spelling does not survive a round trip (md2: `r43` on the text, `rev r66` on the id line).
The contract's rule, distinct allocated labels for every carrier that shares a number, makes
this a fixed point for free. Keep §4 as written on this point. The remaining choice is what
the view prints for the *selector* when several carriers are one visible change.

- **A. Print the allocated labels joined, as the grammar already allows** (`r0+1`), never a
  repeated number. `r43+43` is an entity spelling in disguise; `r0+1` is a selector over two
  real targets, which is what it is. Trade-off: none beyond the cutover; the two committed views
  that hold `43+43`/`55+55` are regenerated with the rest.
- **B. Group the carriers as one logical target** when they share `w:id`, author, date and the
  same paragraph, and make the writer emit one `w:id` for both. Trade-off: the written docx then
  carries a duplicate `w:id`, which `jubarte validate` reports as `DUPLICATE_REVISION_ID`, and
  the grouping proof is weaker than the twin and style-pair proofs §3 admits.
- **C. Keep `r43+43` as a display convention** and teach the writer to reuse the number for
  the second carrier. Trade-off: same duplicate `w:id` as B, plus a printed id that names two
  targets while looking like one.

### 7. Anchored mode: stage it or move it in

Problem: §7's anchored sentence describes behaviour that does not exist, and the stage table
never schedules it. Goal 2's "precisely" is this mode.

- **A. Add an ASAP stage** "anchored md → docx (13 October §5.3): automatic when
  `source:`/`sha256:` match a file beside the Markdown, `--standalone` opts out; resolves id
  lines through the pinned inventory; everything the Markdown does not express survives byte
  for byte", and mark §7's sentence as governed by it. Trade-off: one row in a table.
- **B. Move the anchored-mode rules into this contract,** since the resolver it needs is the
  one this contract defines. Trade-off: a longer contract, one source of truth.
- **C. Keep as written** and cite 13 October §5.3 by name in §7. Trade-off: the dependency is
  visible, the schedule is not.

### 8. Lists in `read -p`: a CLI separator, not a second grammar

Problem: §2 retires `p0,p1`, `p17-` and `-p3` because they are a second parser. Repeating
`-p` three times is the proposed replacement, which is clumsy for an agent.

- **A. Split on commas at the option level** (clap `value_delimiter`), then parse each token
  with the one grammar. Comma is not an id character, so this is not a second grammar; open
  ranges go. Trade-off: `p17-` must become `p17-p999` or a last-paragraph lookup first.
- **B. Repeat the option, as written.** Trade-off: `-p p0 -p p1 -p p2`.
- **C. Keep the open-range productions in the grammar** (`p17-` to the end). Trade-off: the
  grammar grows for one command.

### 9. The 48 synthetic catalog hashes without their tuples

Problem: §11 says the synthetic values "are fully specified 64-digit hashes computed from
explicit test tuples" but gives tuples for two of them. The other 46 cannot be checked.

- **A. Append the tuple** (sha256, part URI, element, path, role, counter) to each synthetic
  row or as a JSON appendix, so a conformance test can regenerate the whole catalog.
- **B. Mark the 46 as illustrative** and keep the two vectors as the only normative ones.
- **C. Drop the 46 rows.**

## Smaller notes, no alternatives needed

- Two numeric ranges for one attribute: a `w:id` of `4294967295` is preserved as a short label
  (canonical `u32`, §4) but is not a native long key (`0..=2147483647`, §5), so `r4294967295`
  prints beside a `j:key`. Consistent, but surprising. One range (`0..=2147483647` for both)
  removes the wrinkle; Word reads `w:id` as a signed 32-bit value anyway. **→ Resolved in v3 the
  other way: every `w:id` key takes Word's full *signed* `int` range (the v2 `0` floor is gone),
  and `4294967295` — above `int` — is allocated `r0` with long id `document.w:ins.j:short.r0`; the
  label and the long id then agree (see "Questions 1 and 2, closed" below). `j:key` is withdrawn.**
- Header aliases: a document whose only header is `/word/header3.xml` prints `header3`, as
  today. Fine, but the agent view should not be read as "there are three headers".
- `header0` is allocated to nonconventional parts. It is Word-valid and round-trips, but it
  is a name Word itself never writes; worth one line in the docs.
- The contract's line references point at `d6fce6e7` on `fix/md-roundtrip-fidelity`. The
  branch this review ran on diverges by 121 commits. Pin the contract to the branch that will
  implement it, or make the references commit-independent (function names). **→ Partially done:
  v3's header names the commits it was checked against (`7c996819`, `aba48bd5`), but its inline
  "current code" links still point at `d6fce6e7` blobs — the deep links remain pinned to the old
  baseline.**
- The 13 October spec §4.3 decided "a table is its first row's key; a cell is its row's key
  plus `.cellN`". The contract overrides this with own `w:tbl` and `w:tc` keys. That is the
  better rule; the contract should say it supersedes 13 October §4.3 explicitly, since it
  claims to supersede only "conflicting ID clauses". **→ Done: v3's header names the §4.3
  dependent-keys rule among the superseded clauses.**

## What I did not do

- I did not run `tests/md_fixed_point.rs` itself; the 17-document probe above uses the same
  steps (`read --no-page-markers`, `convert --force -d 1970-01-01T00:00:00Z`, `read`) and the
  same `source:`/`sha256:`/page-count stripping, on a subset of its list plus three corpus
  documents chosen for text boxes and shared ids.
- I did not test the contract's 166/127 short vectors against the current parser; the current
  parser predates the `ps` split, so most refusals and acceptances would differ by design. The
  spot probes above record the current behaviour for the spellings the contract discusses.
- Nothing in the engine's source was changed. This file, the version-3 contract and the plan
  (`docs/superpowers/plans/2026-10-16-id-contract-implementation.md`) are the only additions.

## Decisions (Arthur, 2026-10-16)

Applied in `docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md`, "Contract version 3".
The header's "Changes in version 3" list is the short form of this section.

| Item | Decision | Where in v3 |
| - | - | - |
| 1 writer's rule | A | §7 "The writer's rule"; §9 Urgent 4; §10 "Writer labels" |
| 2 `ps` | Keep `cell.psN` as an alias, see below | §2 after the `ps` grammar; header item 2 |
| 3 synthetic key | Word's number is the key; `j:short` otherwise, see below | §5 "Native key eligibility", "Keyless targets take their short id"; §6; §10 "Short key vectors"; §11 |
| 4 staging | A | §9 "Concrete implementation sequence" |
| 5 text boxes | A | §7 "Valid syntax does not promise edit support"; §9 Future |
| 6 shared `w:id` | A | §3 "Groups and ranges in JSON"; §9 cutover row |
| 7 anchored mode | B (move it in) | §7 "Anchored Markdown edits"; §9 ASAP 2 |
| 8 `read -p` commas | A | §2 "A comma list…"; §9 ASAP 1 |
| 9 catalog tuples | A, now moot: the value is the short id | §11 intro and 48 rows |

**Item 2, discussed.** Arthur: "in a cell, I agree that `pN` corresponds to that specific cell, but
we also read `cell.psN`. I think it works as an alias, e.g. `t0.row0.cell0.ps1` → second part inside
that cell." I had proposed dropping `ps` in cells (A). Arthur is right that the token has one
meaning already: an index into the scope's own paragraphs. What differs between a cell and a box is
inventory membership, not the token. A cell's paragraphs are in the story inventory, so the alias
normalises to the `pN` the view prints; a box's paragraphs are not, so `boxN.psM` is the printed id.
That is alternative C with the clarifying sentence, and v3 §2 says so in those words.

**Item 3, discussed.** Arthur: "I want whatever number Word has given is the number, and if none
was given, then we use the short id." I agree, and it is simpler than my A (`j:path` ordinals): the
short id is already a deterministic, snapshot-scoped, readable position, so a second positional
spelling would duplicate it. The attribute name is `j:short`; the long id of a keyless target is
`PART.ELEMENT.j:short.SHORT`, with the short id verbatim (no escaping needed: letters, digits and
dots). Version 2's SHA-256 `j:key`, its framing, counter and vectors are withdrawn; `j:short` for a
natively keyed target is `ID_NOT_FOUND` with the real pair as the candidate.

**Ranges, decided after the first draft of v3.** My first v3 draft withdrew v2's numeric range
cutoffs on native keys, reading "whatever number Word has given" literally. Arthur: "if the below
are truthful and they are, let's keep the limits" (the `paraId` range from MS-DOCX and Word's
signed 32-bit `w:id`). Both were checked against Microsoft's pages on 2026-10-16: MS-DOCX says
`paraId` "values MUST be greater than 0 and less than 0x80000000"; MS-OI29500 says Word restricts
`ST_DecimalNumber` to the XML Schema `int` type. The limits are back in §5 "Native key eligibility",
with the eight range refusals in §10 (48 cases again). One difference from v2: v2 floored revision
and comment `w:id` at `0` as its own "conservative policy" while giving SDT `w:id` the full signed
range; the floor has no source in the specification, so v3 uses Word's `int` range for every `w:id`
key. `document.w:ins.w:id.-2D1` therefore parses in v3 where v2 refused it. If the `0` floor is
wanted back, it is one cell in the §5 table and one refusal entry.

**Questions 1 and 2 of 2026-10-16, closed.** Arthur chose 1A: Word's full `int` range for every
`w:id` key, no `0` floor. On question 2 Arthur observed that a `w:id` above Word's `int` cannot have
been generated by jubarte, which the code confirms: the comparer numbers marks from 1
(`src/comparer/produce.rs:2105`), the Markdown writer's fresh ids wrap at `0x7FFFFFFF`
(`MOST_ID`, `src/markdown/xml.rs:463`, `src/markdown/view_notes.rs:398`), and the writer drops a
view number above that rather than copy it (`view_notes.rs:387`). That settles 2 as A: §4 preserves
a label only within Word's `int`, so a source `w:id="4294967295"` is allocated (`r0`) and its long ID
is `document.w:ins.j:short.r0`; both halves then say the file gave no usable key, and the second
read agrees with the first. The alias vector for `4294967295` now expects allocation, and a
boundary vector (`2147483647` preserved, `2147483648` allocated) was added. "Label" is now defined
where §4 introduces it: the number in `rN` or `cN`, a component of the short ID, not a third identity.
